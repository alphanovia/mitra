//! Mitra's embeddable library.
//!
//! Parse a model (subjects, boundaries, claims) and a list of evidence,
//! then compute a verdict for each claim, and for each error the pieces
//! most plausibly wrong.
//!
//! ```
//! let model = "subject s { fn flip(a: bool) -> bool;\n\
//!              claim flip_correct(a: bool) { correct(flip(a)) } }";
//! let evidence = "evidence t { for flip holds }";
//! let report = mitra_core::run(model, evidence).unwrap();
//! assert_eq!(report.to_text(), "s::flip_correct: holds\n");
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
    if let Some((line, hint)) = surface::earlier(model) {
        let msg = format!("this syntax is no longer read: {hint}");
        return Err((Input::Model, Error::new(line, msg)));
    }
    load_surface(model, evidence)
}

/// The same for the internal form the model is translated to. It is not
/// a language to write in: the engine's own tests use it.
#[doc(hidden)]
pub fn core_load(model: &str, evidence: &str) -> Result<Model, (Input, Error)> {
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
    for (name, f, set, line) in &names.represented {
        m.claims.push(Claim {
            name: name.clone(),
            func: f.clone(),
            pattern: Vec::new(),
            target: Target::Represented(set.clone()),
            from: Vec::new(),
            auto: false,
            trust: names.trust.get(name).copied(),
            line: *line,
        });
    }
    let lines = evidence.lines().count();
    let earlier = surface::evidence(evidence, &names).map_err(|e| (Input::Evidence, e))?;
    tie(&mut m, &earlier, &names);
    m.evidence = parse_evidence_for(&earlier, &m).map_err(|e| (Input::Evidence, e))?;
    points(&mut m);
    let own = |s: &str| s.lines().map(String::from).collect::<Vec<_>>();
    let mut written = own(evidence);
    written.truncate(lines);
    m.written = Some((own(model), written));
    m.check()
}

/// What evidence on a point claim says of the code. A test tied to
/// `claim t() { f(6, 2) == 3 }` that passes is the output `f(6, 2) = 3`;
/// failing, it refutes the specs that give 3 there. On
/// `claim t() { correct(f(6, 2)) }` it judges that call.
fn points(m: &mut Model) {
    let mut more = Vec::new();
    for e in &m.evidence {
        let (Judgment::Holds(ok), true) = (&e.judgment, e.pattern.iter().all(|p| p.set.is_none()))
        else {
            continue;
        };
        // An assumption is nobody's observation.
        if e.source.as_deref() == Some("assumed") {
            continue;
        }
        let Some(c) = m
            .claim(&e.about)
            .filter(|c| c.pattern.iter().all(|p| p.var.is_none()))
        else {
            continue;
        };
        let on = |about: &str, pattern: &Pattern, judgment: Judgment| Evidence {
            about: about.to_string(),
            pattern: pattern.clone(),
            judgment,
            ..e.clone()
        };
        let judged = |f: &str| {
            let covers = |d: &Claim| d.func == f && d.target == Target::Correct;
            m.claims.iter().any(covers)
        };
        match &c.target {
            Target::Correct if !c.pattern.is_empty() || m.function(&c.func).is_some() => {
                more.push(on(&c.func, &c.pattern, Judgment::Correct(*ok)));
            }
            Target::Property(Expr::Binary("==", l, r)) => {
                let side = [(l, r), (r, l)].into_iter().find_map(|(call, other)| {
                    let Expr::Call(f, args) = &**call else {
                        return None;
                    };
                    let g = m.declared_function(f).filter(|g| !g.is_spec())?;
                    let x: Option<Vec<Value>> =
                        args.iter().map(|a| eval::computed(m, a, &[])).collect();
                    Some((g, x?, eval::computed(m, other, &[])?))
                });
                let Some((g, x, v)) = side else {
                    continue;
                };
                let at: Pattern = x
                    .iter()
                    .map(|v| Position {
                        var: None,
                        set: Some(Set::point(v)),
                    })
                    .collect();
                let other = match &v {
                    Value::Bool(b) if g.range == Set::bool() => Some(Value::Bool(!b)),
                    _ => None,
                };
                match (ok, other) {
                    (true, _) => more.push(on(&g.name, &at, Judgment::Output(v.clone()))),
                    // Not this output: with two possible, the other one.
                    (false, Some(other)) => more.push(on(&g.name, &at, Judgment::Output(other))),
                    (false, None) => {
                        // The specs that give this output there are refuted.
                        for d in m.claims.iter().filter(|d| d.func == g.name) {
                            let Target::Matches(target) = &d.target else {
                                continue;
                            };
                            let names: Vec<(String, Value)> = d
                                .pattern
                                .iter()
                                .zip(&x)
                                .filter_map(|(p, v)| Some((p.var.clone()?, v.clone())))
                                .collect();
                            let inside = m.covers(&d.func, &d.pattern).contains(&x);
                            if inside && eval::computed(m, target, &names).as_ref() == Some(&v) {
                                more.push(on(&d.name, &at, Judgment::Holds(false)));
                            }
                        }
                    }
                }
                if judged(&g.name) {
                    more.push(on(&g.name, &at, Judgment::Correct(*ok)));
                }
            }
            _ => {}
        }
    }
    m.evidence.extend(more);
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
    let (text, names) = surface::model(model)?;
    let mut m = parse_model(&text)?;
    let earlier = surface::evidence_alone(evidence, &names)?;
    tie(&mut m, &earlier, &names);
    parse_evidence_for(&earlier, &m)
}

/// Parse both inputs, check them against each other, and evaluate.
pub fn run(model: &str, evidence: &str) -> Result<Report, (Input, Error)> {
    evaluate(&load(model, evidence)?)
}

/// [`run`] on the internal form.
#[doc(hidden)]
pub fn core_run(model: &str, evidence: &str) -> Result<Report, (Input, Error)> {
    evaluate(&core_load(model, evidence)?)
}
