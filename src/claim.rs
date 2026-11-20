//! The seam (`model/claim.mitra`): here the content of a statement is
//! filled in. Computing a formal one makes evidence about its piece.
//!
//! The only way across: a value never depends on a piece, and an
//! opinion is only moved by evidence.

use crate::bounds::{lifted, surely, Bounds};
use crate::model::{no, whole, yes, Evidence, Piece, Statement, Tie, Voice};
use crate::standard::{truth, Exact};
use crate::value::{check, eval, Applies, Expression, Value};

/// What a claim says: an expression that gives a truth. A claim is a
/// statement with this content.
// mitra: implements claims::Claim
#[derive(Clone, Debug, PartialEq)]
pub struct Claim {
    pub expression: Expression<Exact>,
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
    inputs: &[(String, Value<Exact>)],
    by: &Voice,
    with: &impl Applies<Exact>,
) -> Option<Evidence> {
    let seen = eval(&s.content.expression, inputs, with)?;
    let value = match (seen.of == truth(), seen.quantity) {
        (true, Exact::Truth(true)) => yes(),
        (true, Exact::Truth(false)) => no(),
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

/// What reading the statement by bounds says of it, with each input
/// known to lie in a range: `count` yes where it is true everywhere in
/// the ranges, `count` no where it is false everywhere, and nothing
/// where it may be either, may have no value, or the reading is
/// unknown. `count` is how many observations a whole range is worth:
/// it is given, and nothing here holds a value for it.
// mitra: implements claims::bounded
pub fn bounded(
    s: &Statement<Claim>,
    ranges: &[(String, Value<Bounds>)],
    count: f64,
    by: &Voice,
    with: &impl Applies<Bounds>,
) -> Option<Evidence> {
    let seen = eval(&lifted(&s.content.expression)?, ranges, with)?;
    let value = match surely(&seen)? {
        true => crate::model::Opinion { r: count, ..yes() },
        false => crate::model::Opinion { s: count, ..no() },
    };
    Some(Evidence {
        piece: Piece {
            name: format!("{} within {:?} by {}", s.piece.name, ranges, by.name),
        },
        about: s.piece.clone(),
        value,
        by: by.clone(),
    })
}

/// Evidence about a statement, aimed at the whole of a tie that lists
/// it: the statement, the tie and its completeness together. Nothing
/// when the tie does not list the statement.
// mitra: implements claims::aimed
pub fn aimed(e: Evidence, at: &Tie) -> Option<Evidence> {
    if !at.statements.contains(&e.about) {
        return None;
    }
    Some(Evidence {
        piece: Piece {
            name: format!("{} at {}", e.piece.name, at.piece.name),
        },
        about: whole(at),
        ..e
    })
}

/// What computing the statement on what was measured at the place of
/// a tie says: the same yes or no, about the whole of that tie.
/// Nothing when the tie does not list the statement.
// mitra: implements claims::measured
pub fn measured(
    s: &Statement<Claim>,
    at: &Tie,
    inputs: &[(String, Value<Exact>)],
    by: &Voice,
    with: &impl Applies<Exact>,
) -> Option<Evidence> {
    aimed(observe(s, inputs, by, with)?, at)
}
