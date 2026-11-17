//! Mitra's embeddable library.
//!
//! Parse a claim model and an evidence list, then compute, for each claim, a
//! verdict and its two supports: `p` for pass and `f` for fail.
//!
//! ```
//! let claims = "claim not_correct(a: bool): not(a) = !a";
//! let evidence = "evidence test(t1) : not_correct(true) = pass @ 0.9";
//! let report = mitra_core::run(claims, evidence, &mitra_core::Policy::default()).unwrap();
//! assert_eq!(report.claims[0].instances[1].verdict, mitra_core::Verdict::Pass);
//! ```

pub mod ast;
pub mod eval;
pub mod parse;

pub use ast::{Claim, Evidence, Kind, Model, Outcome, Value};
pub use eval::{evaluate, ClaimResult, InstanceResult, Policy, Report, Verdict};
pub use parse::{parse_claims, parse_evidence};

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
    Claims,
    Evidence,
}

/// Parse both inputs, check the evidence against the claims, and evaluate.
pub fn run(claims: &str, evidence: &str, policy: &Policy) -> Result<Report, (Input, Error)> {
    let claims = parse_claims(claims).map_err(|e| (Input::Claims, e))?;
    let evidence = parse_evidence(evidence).map_err(|e| (Input::Evidence, e))?;
    let model = Model::new(claims, evidence)?;
    Ok(evaluate(&model, policy))
}
