//! The seam (`model/claim.mitra`): here the content of a statement is
//! filled in. Computing a formal one makes evidence about its piece.
//!
//! The only way across: a value never depends on a piece, and an
//! opinion is only moved by evidence.

use crate::model::{no, whole, yes, Evidence, Piece, Statement, Tie, Voice};
use crate::standard::truth;
use crate::value::{check, eval, Applies, Expression, Quantity, Value};

/// What a claim says: an expression that gives a truth. A claim is a
/// statement with this content.
// mitra: implements claims::Claim
#[derive(Clone, Debug, PartialEq)]
pub struct Claim {
    pub expression: Expression,
}

/// The expression gives a truth.
// mitra: implements claims::reads
pub fn reads(s: &Statement<Claim>) -> bool {
    check(&s.content.expression) == Some(truth())
}

/// What computing the statement at these inputs says of it: yes where
/// it is true, no where it is false, nothing where it has no value. The
/// evidence is named after the statement, the inputs and the voice, so
/// the same observation twice is one piece.
// mitra: implements claims::observe
pub fn observe(
    s: &Statement<Claim>,
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

/// What computing the statement on what was measured at the place of
/// a tie says: the same yes or no, about the whole of that tie, the
/// statement, the tie and its completeness together. Nothing when the
/// tie does not list the statement.
// mitra: implements claims::measured
pub fn measured(
    s: &Statement<Claim>,
    at: &Tie,
    inputs: &[(String, Value)],
    by: &Voice,
    with: &impl Applies,
) -> Option<Evidence> {
    if !at.statements.contains(&s.piece) {
        return None;
    }
    let e = observe(s, inputs, by, with)?;
    Some(Evidence {
        piece: Piece {
            name: format!("{} at {}", e.piece.name, at.piece.name),
        },
        about: whole(at),
        ..e
    })
}
