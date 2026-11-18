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
pub mod print;
pub mod report;
pub mod set;
pub mod tuples;

pub use ast::{Claim, Evidence, Expr, Function, Judgment, Model, Pattern, Position, Target, Test};
pub use eval::{
    evaluate, failed_output, Assumed, Candidate, Conflict, Contradiction, Display, Flag,
    Implication, Kind, Logic, Piece, Problem, Report, Root, Suspect, Unranked, Verdict,
};
pub use next::next;
pub use parse::{parse_evidence, parse_model};
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
    let mut m = parse_model(model).map_err(|e| (Input::Model, e))?;
    m.evidence = parse_evidence(evidence).map_err(|e| (Input::Evidence, e))?;
    m.check()
}

/// Parse both inputs, check them against each other, and evaluate.
pub fn run(model: &str, evidence: &str) -> Result<Report, (Input, Error)> {
    evaluate(&load(model, evidence)?)
}
