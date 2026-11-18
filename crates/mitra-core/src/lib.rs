//! Mitra's embeddable library.
//!
//! Parse a model (subjects, specs, claims, tests) and a list of evidence,
//! then compute a verdict for each claim, and for each error the pieces
//! most plausibly wrong.
//!
//! ```
//! let model = "opaque flip(a: Bool) -> Bool\nclaim flip_correct : forall a, correct(flip(a))";
//! let evidence = "evidence correct(flip(true)) at 1\nevidence correct(flip(false)) at 1";
//! let report = mitra_core::run(model, evidence).unwrap();
//! assert_eq!(report.to_text(), "flip_correct: holds\n");
//! ```

pub mod ast;
pub mod eval;
pub mod next;
pub mod parse;
pub mod plan;
pub mod print;
pub mod report;
pub mod set;
pub mod surface;
pub mod tuples;

pub use ast::{
    Arm, Claim, Decl, Enum, Evidence, Expr, Function, Judgment, Model, Pattern, Position, Record,
    Target, Test,
};
pub use eval::{
    evaluate, expected_output, failed_output, Assumed, Candidate, Conflict, Contradiction, Display,
    Flag, Implication, Kind, Logic, Piece, Problem, Report, Root, Suspect, Unranked, Verdict,
};
pub use next::next;
pub use parse::{parse_evidence, parse_evidence_for, parse_model};
pub use plan::plan;
pub use print::{print_evidence, print_model};
pub use report::{explain, Files, Places};
pub use set::{Set, Value};
pub use tuples::Tuples;

use std::fmt;

/// A problem in an input file, with the 1-based line it was found on.
#[derive(Debug, Clone, PartialEq)]
pub struct Error {
    pub line: usize,
    pub message: String,
}

impl Error {
    pub fn new(line: usize, message: impl Into<String>) -> Self {
        Error {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for Error {}

/// Which input an error belongs to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Input {
    Model,
    Evidence,
}

/// Parse both inputs and check them against each other.
pub fn load(model: &str, evidence: &str) -> Result<Model, (Input, Error)> {
    if surface::is_surface(model) {
        return load_surface(model, evidence);
    }
    let mut m = parse_model(model).map_err(|e| (Input::Model, e))?;
    m.evidence = parse_evidence_for(evidence, &m).map_err(|e| (Input::Evidence, e))?;
    m.check()
}

/// The same for a model in the reworked syntax. Its evidence is written
/// in blocks, or in the earlier lines (what `mitra add` appends).
fn load_surface(model: &str, evidence: &str) -> Result<Model, (Input, Error)> {
    let (text, names) = surface::model(model).map_err(|e| (Input::Model, e))?;
    let mut m = parse_model(&text).map_err(|e| (Input::Model, e))?;
    // A claim follows from the specs of the functions it calls, when
    // their claims are above it.
    let above: Vec<String> = m.claims.iter().map(|c| c.name.clone()).collect();
    for (i, c) in m.claims.iter_mut().enumerate() {
        c.trust = names.trust.get(&c.name).copied();
        let specs = names.derived.get(&c.name).into_iter().flatten();
        let specs: Vec<String> = specs.filter(|s| above[..i].contains(s)).cloned().collect();
        if c.from.is_empty() && !specs.is_empty() {
            (c.from, c.auto) = (specs, true);
        }
    }
    let outside = |f: &&Function| {
        let scope = f.name.split("::").next().unwrap_or("");
        !f.is_spec() && f.name.contains("::") && names.boundaries.contains(scope)
    };
    m.boundaries = m
        .functions
        .iter()
        .filter(outside)
        .map(|f| f.name.clone())
        .collect();
    let lines = evidence.lines().count();
    let earlier = surface::evidence(evidence, &names).map_err(|e| (Input::Evidence, e))?;
    tie(&mut m, &earlier, &names);
    m.evidence = parse_evidence_for(&earlier, &m).map_err(|e| (Input::Evidence, e))?;
    let own = |s: &str| s.lines().map(String::from).collect::<Vec<_>>();
    let mut written = own(evidence);
    written.truncate(lines);
    m.written = Some((own(model), written));
    m.check()
}

/// A tie is a claim once someone brings evidence on it.
fn tie(m: &mut Model, evidence: &str, names: &surface::Names) {
    for f in surface::tied(evidence, names) {
        let line = m.function(&f).map_or(0, |f| f.line);
        m.claims.push(Claim {
            name: format!("{f}::tied"),
            func: f,
            pattern: Vec::new(),
            target: Target::Tied,
            from: Vec::new(),
            auto: false,
            trust: None,
            line,
        });
    }
}

/// Parse new evidence for a model, in the syntax the model is written in.
/// Nothing is checked against the model beyond its names.
pub fn read_evidence(model: &str, evidence: &str) -> Result<Vec<Evidence>, Error> {
    if surface::is_surface(model) {
        let (text, names) = surface::model(model)?;
        let mut m = parse_model(&text)?;
        let earlier = surface::evidence_alone(evidence, &names)?;
        tie(&mut m, &earlier, &names);
        return parse_evidence_for(&earlier, &m);
    }
    parse_evidence_for(evidence, &parse_model(model).unwrap_or_default())
}

/// Parse both inputs, check them against each other, and evaluate.
pub fn run(model: &str, evidence: &str) -> Result<Report, (Input, Error)> {
    evaluate(&load(model, evidence)?)
}
