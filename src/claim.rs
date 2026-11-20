//! The seam (`model/claim.mitra`): here the content of a statement is
//! filled in. Computing a formal one makes evidence about its piece.
//!
//! The only way across: a value never depends on a piece, and an
//! opinion is only moved by evidence.

use crate::model::{no, yes, Evidence, Piece, Statement, Voice};
use crate::standard::truth;
use crate::value::{check, eval, Applies, Expression, Quantity, Value};

/// A content that is computed: an expression that gives a truth.
// mitra: implements claims::Formal
#[derive(Clone, Debug, PartialEq)]
pub struct Formal {
    pub expression: Expression,
}

/// A content that is read by people: nothing computes it.
// mitra: implements claims::Prose
#[derive(Clone, Debug, PartialEq)]
pub struct Prose {
    pub text: String,
}

/// The expression gives a truth.
// mitra: implements claims::reads
pub fn reads(s: &Statement<Formal>) -> bool {
    check(&s.content.expression) == Some(truth())
}

/// What computing the statement at these inputs says of it: yes where
/// it is true, no where it is false, nothing where it has no value. The
/// evidence is named after the statement, the inputs and the voice, so
/// the same observation twice is one piece.
// mitra: implements claims::observe
pub fn observe(
    s: &Statement<Formal>,
    inputs: &[(String, Value)],
    by: &Voice,
    with: &impl Applies,
) -> Option<Evidence> {
    let seen = eval(&s.content.expression, inputs, with)?;
    let value = match (seen.of == truth(), seen.quantity) {
        (true, Quantity::Truth(true)) => yes(),
        (true, Quantity::Truth(false)) => no(),
        _ => return None,
    };
    Some(Evidence {
        piece: Piece {
            name: format!("{} at {:?} by {}", s.piece.name, inputs, by.name),
        },
        about: s.piece.clone(),
        value,
        by: by.clone(),
    })
}
