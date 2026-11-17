//! Mitra's embeddable library.
//!
//! Parse a model (subjects and claims) and a list of evidence, then compute,
//! for each claim, a verdict and a certainty.
//!
//! ```
//! let model = "subject not(a: bool) -> bool\nclaim not_correct: not(_) correct";
//! let evidence = "evidence not(true) correct @ 1\nevidence not(false) correct @ 1";
//! let report = mitra_core::run(model, evidence).unwrap();
//! assert_eq!(report.to_text(), "not_correct  holds @ 1\n");
//! ```

pub mod ast;
pub mod eval;
pub mod parse;

pub use ast::{Claim, Evidence, Model, Pattern, Subject};
pub use eval::{evaluate, ClaimResult, Report, Verdict};
pub use parse::{parse_evidence, parse_model};

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
    let (subjects, claims) = parse_model(model).map_err(|e| (Input::Model, e))?;
    let evidence = parse_evidence(evidence).map_err(|e| (Input::Evidence, e))?;
    let model = Model::new(subjects, claims, evidence)?;
    Ok(evaluate(&model))
}
