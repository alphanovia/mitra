//! A believed link between an interpretation and a named part of the
//! system. A tie is a piece: it can be wrong like any other.

use super::piece::Piece;
use super::place::{within, Place};
use super::statement::Statement;

/// Says that statements, each named by its piece, are about a place:
/// one object, and what the model says of it.
// mitra: implements tie::Tie
#[derive(Clone, Debug, PartialEq)]
pub struct Tie {
    pub piece: Piece,
    pub statements: Vec<Piece>,
    pub place: Place,
}

/// The places a statement is tied to.
// mitra: implements tie::places
pub fn places(ties: &[Tie], statement: &Piece) -> Vec<Place> {
    let of_it = ties.iter().filter(|t| t.statements.contains(statement));
    of_it.map(|t| t.place.clone()).collect()
}

/// The statements tied at a place or within it.
// mitra: implements tie::concerned
pub fn concerned(ties: &[Tie], place: &Place) -> Vec<Piece> {
    let mut found = Vec::new();
    for t in ties {
        for statement in &t.statements {
            if within(&t.place, place) && !found.contains(statement) {
                found.push(statement.clone());
            }
        }
    }
    found
}

/// What a tie leaves unsaid: the statements tied there are all that
/// matters there.
// mitra: implements tie::Complete
#[derive(Clone, Debug, PartialEq)]
pub struct Complete {
    pub tie: Piece,
}

/// The completeness of a tie: a statement, implied once per tie, known
/// by the tie's name.
// mitra: implements tie::complete
pub fn complete(t: &Tie) -> Statement<Complete> {
    Statement {
        piece: Piece::named(&format!("complete: {}", t.piece.name)),
        content: Complete {
            tie: t.piece.clone(),
        },
    }
}

/// What an observation at the place is about: the tie, its statements
/// and its completeness, all together. Implied once per tie, known by
/// the tie's name.
// mitra: implements tie::whole
pub fn whole(t: &Tie) -> Piece {
    Piece::named(&format!("whole: {}", t.piece.name))
}
