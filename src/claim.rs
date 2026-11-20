//! The seam (`model/claim.mitra`): a claim is a piece with a statement,
//! and computing the statement makes evidence about that piece.
//!
//! The only way across: a value never depends on a piece, and an
//! opinion is only moved by evidence.

use crate::piece::{no, yes, Evidence, Piece};
use crate::value::{check, eval, Expression, Type, Value};

// mitra: implements claims::Claim
#[derive(Clone, Debug, PartialEq)]
pub struct Claim {
    pub piece: Piece,
    pub statement: Expression,
}

/// The statement gives a truth when its inputs have these types.
// mitra: implements claims::reads
pub fn reads(c: &Claim, inputs: &[(String, Type)]) -> bool {
    check(&c.statement, inputs) == Some(Type::OfTruth)
}

/// What computing the statement at these inputs says of the claim:
/// yes where it is true, no where it is false, nothing where it has no
/// value. The evidence is named after the claim, the inputs and the
/// voice, so the same observation twice is one piece.
// mitra: implements claims::observe
pub fn observe(c: &Claim, inputs: &[(String, Value)], by: &str) -> Option<Evidence> {
    let value = match eval(&c.statement, inputs)? {
        Value::Truth(true) => yes(),
        Value::Truth(false) => no(),
        _ => return None,
    };
    Some(Evidence {
        piece: Piece(format!("{} at {:?} by {}", c.piece.0, inputs, by)),
        about: c.piece.clone(),
        value,
        by: by.to_string(),
    })
}
