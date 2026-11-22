//! What a model is made of (`model/atom.mitra`): parts, with their
//! records and functions, each specified as far as its author chose,
//! and measurements of functions on the system. It reads `value` for
//! types, values and expressions, and `model` for a location.

use crate::model::Location;
use crate::standard::Exact;
use crate::value::{Expression, Type, Value};

/// What a part is: built here, or bought.
// mitra: implements part::Kind
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Subject,
    Boundary,
}

/// Data of the system. With no fields it is only declared.
// mitra: implements part::Record
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub name: String,
    pub fields: Option<Vec<(String, Type)>>,
}

/// Something the system does: from its inputs, an output, each with a
/// name and a type. With no body it is only declared; the body says
/// what the output is, for every input.
// mitra: implements part::Function
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub inputs: Vec<(String, Type)>,
    pub output: (String, Type),
    pub body: Option<Expression<Exact>>,
}

/// A named part of the system model, with its records and functions.
// mitra: implements part::Part
#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub name: String,
    pub kind: Kind,
    pub records: Vec<Record>,
    pub functions: Vec<Function>,
}

/// How far something is specified.
///
/// - `Opaque`: only declared.
/// - `Partial`: something is said of it, not all.
/// - `Total`: fully defined.
// mitra: implements part::Level
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    Opaque,
    Partial,
    Total,
}

/// How far a record is specified: total with its fields; else partial
/// with `claims` about it; else opaque.
// mitra: implements part::record_level
pub fn record_level(r: &Record, claims: usize) -> Level {
    match (&r.fields, claims) {
        (Some(_), _) => Level::Total,
        (None, 0) => Level::Opaque,
        (None, _) => Level::Partial,
    }
}

/// How far a function is specified: total with its body; else partial
/// with `claims` about it; else opaque.
// mitra: implements part::function_level
pub fn function_level(f: &Function, claims: usize) -> Level {
    match (&f.body, claims) {
        (Some(_), _) => Level::Total,
        (None, 0) => Level::Opaque,
        (None, _) => Level::Partial,
    }
}

/// How far a part is specified, from the levels of its records and
/// functions: total when all are, opaque when all are or it has none,
/// partial otherwise.
// mitra: implements part::part_level
pub fn part_level(levels: &[Level]) -> Level {
    let all = |level: Level| levels.iter().all(|l| *l == level);
    match (levels.is_empty() || all(Level::Opaque), all(Level::Total)) {
        (true, _) => Level::Opaque,
        (false, true) => Level::Total,
        (false, false) => Level::Partial,
    }
}

/// An instance of a function, taken on the system at a location: each
/// input and the output with what it was. `Q` is the kind of
/// quantity: exact values for a point, bounds for a set.
// mitra: implements part::Measurement
#[derive(Clone, Debug, PartialEq)]
pub struct Measurement<Q> {
    pub function: String,
    pub at: Location,
    pub values: Vec<(String, Value<Q>)>,
}
