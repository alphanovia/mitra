//! Mitra's embeddable library.
//!
//! Parse a model (subjects, specs, claims, profiles, policies) and a list of
//! evidence, then compute a verdict for each claim under each policy.
//!
//! ```
//! let model = "opaque not (a : Bool) : Bool\nclaim not_correct : ∀ a, correct (not a)\npolicy strict";
//! let evidence = "evidence correct (not true) at 1\nevidence correct (not false) at 1";
//! let report = mitra_core::run(model, evidence).unwrap();
//! assert_eq!(
//!     report.to_text(),
//!     "not_correct [strict] := { logic := .holds, covered := Bool, domain := Bool, belief := 1 }\n"
//! );
//! ```

pub mod ast;
pub mod eval;
pub mod parse;
pub mod print;
pub mod set;
pub mod tuples;

pub use ast::{
    Claim, Evidence, Expr, Function, Judgment, Model, Pattern, Policy, PolicyKind, Position,
    Profile, Target,
};
pub use eval::{evaluate, Flag, Implication, Logic, Report, Verdict};
pub use parse::{parse_evidence, parse_model};
pub use print::{print_evidence, print_model};
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
