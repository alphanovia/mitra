//! The reader of code: what any language owes the core.
//!
//! The core reads no code. A reader does, for one language, and brings
//! what it finds in the core's own words. This is only the interface:
//! nothing here opens a file.

use crate::results::{Outcome, Tied};
use crate::Model;

/// What comparing the code with the model found: evidence lines, and
/// what is printed of them.
// mitra: implements frontend::Found
pub struct Found {
    pub evidence: Vec<String>,
    pub notes: Vec<String>,
    pub agree: usize,
    pub differ: usize,
    /// The functions, records and enums of the model no code is tied to.
    pub untied: Vec<String>,
    /// The lines of the code that are code: not blank, not a comment,
    /// not a unit test.
    pub code_lines: usize,
}

/// A reader of the code of one language. `Place` is where it finds the
/// code: a folder, for Rust.
pub trait Reader<Place: ?Sized> {
    /// A report of the language's test runner, as plain results.
    // mitra: implements Reader::results
    fn results(&self, text: &str) -> Vec<Outcome>;

    /// Every test of the code tied to a claim.
    // mitra: implements Reader::tests
    fn tests(&self, root: &Place) -> Result<Vec<Tied>, String>;

    /// The code tied to the model, compared with it.
    // mitra: implements Reader::tie
    fn tie(&self, root: &Place, model: &Model, rev: Option<&str>) -> Result<Found, String>;

    /// The erasures of this reader, each with its cost: what it may
    /// forget of the code to reach the model.
    // mitra: implements Reader::erasures
    fn erasures(&self) -> Vec<(&'static str, f64)>;
}
