//! Mitra's embeddable library.
//!
//! Parse a model (subjects, specs, claims, profiles, policies) and a list of
//! evidence, then compute a verdict for each claim under each policy.
//!
//! ```
//! let model = "subject not(a: bool) -> bool\nclaim not_correct: not(_) correct\npolicy strict";
//! let evidence = "evidence not(true) correct @ 1\nevidence not(false) correct @ 1";
//! let report = mitra_core::run(model, evidence).unwrap();
//! assert_eq!(
//!     report.to_text(),
//!     "verdict not_correct [strict] = { logic: holds, covered: bool, domain: bool, belief: 1 }\n"
//! );
//! ```

pub mod ast;
pub mod eval;
pub mod parse;
pub mod set;

pub use ast::{
    Claim, Evidence, Expr, Function, Judgment, Model, Pattern, Policy, PolicyKind, Profile, Target,
};
pub use eval::{evaluate, Flag, Logic, Report, Verdict};
pub use parse::{parse_evidence, parse_model};
pub use set::{Set, Value};

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

/// Parse both inputs, check them against each other, and evaluate.
pub fn run(model: &str, evidence: &str) -> Result<Report, (Input, Error)> {
    let mut m = parse_model(model).map_err(|e| (Input::Model, e))?;
    m.evidence = parse_evidence(evidence).map_err(|e| (Input::Evidence, e))?;
    evaluate(&m.check()?)
}
