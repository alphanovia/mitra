//! Mitra's embeddable library.
//!
//! Parse a model (subjects, boundaries, claims) and a list of evidence,
//! then compute a verdict for each claim, and for each error the pieces
//! most plausibly wrong.
//!
//! ```
//! let model = "subject s { fn flip(a: Bool) -> Bool;\n\
//!              claim flip_correct(a: Bool) { correct(flip(a)) } }";
//! let evidence = "evidence t { for flip holds }";
//! let report = mitra_lang::run(model, evidence).unwrap();
//! assert!(report.to_text().contains("s::flip_correct: holds\n"));
//! ```

pub mod build;
mod claims;
mod contracts;
pub mod eval;
pub mod frontend;
mod functions;
mod intent;
pub mod ledger;
pub mod model;
pub mod modules;
pub mod parse;
mod pipeline;
mod reader;
pub mod report;
pub mod settings;
pub mod types;
pub mod verdicts;

pub use frontend::{Found, Reader};
pub use model::{
    Arm, Aside, Block, Bounded, Claim, Decl, Enum, Evidence, Expr, Function, Judgment, Model,
    Pattern, Position, Record, Side, Target, Why,
};
pub use modules::{Module, Modules};
pub use parse::Handed;
pub use pipeline::{load, load_modules, read_evidence, run};
pub use report::next::next;
pub use report::plan::plan;
pub use report::{explain, Files, Places};
pub use settings::Settings;
pub use types::set::{Set, Value};
pub use types::tuples::Tuples;
pub use verdicts::{
    evaluate, Assumed, Candidate, Conflict, Contradiction, Display, Flag, Implication, Kind, Logic,
    Piece, Problem, Report, Root, Suspect, Unranked, Verdict,
};

use std::fmt;

/// A problem in an input file, with the 1-based line it was found on.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements reader::Error
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
// mitra: implements types::Input
pub enum Input {
    Model,
    Evidence,
}
