//! A believed link between an interpretation and a named part of the
//! system. A tie is a piece: it can be wrong like any other.

use super::piece::Piece;
use super::place::{within, Place};

/// Says that a statement, named by its piece, is about a place.
// mitra: implements tie::Tie
#[derive(Clone, Debug, PartialEq)]
pub struct Tie {
    pub piece: Piece,
    pub statement: Piece,
    pub place: Place,
}

/// The places a statement is tied to.
// mitra: implements tie::places
pub fn places(ties: &[Tie], statement: &Piece) -> Vec<Place> {
    let of_it = ties.iter().filter(|t| &t.statement == statement);
    of_it.map(|t| t.place.clone()).collect()
}

/// The statements tied at a place or within it.
// mitra: implements tie::concerned
pub fn concerned(ties: &[Tie], place: &Place) -> Vec<Piece> {
    let mut found = Vec::new();
    for t in ties {
        if within(&t.place, place) && !found.contains(&t.statement) {
            found.push(t.statement.clone());
        }
    }
    found
}
