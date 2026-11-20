//! Pieces, opinions, evidence (`model/piece.mitra`).
//!
//! A piece is one thing said about a system, which can be right or
//! wrong. An opinion is what is believed of a piece. Evidence is a
//! piece that says an opinion about another piece.

mod calculus;
mod known;

pub use calculus::{against, all, conflict, fuse, moved, no, nothing, projected, reaches, yes};
pub use known::{base, conflicts, opinion, trusted};

/// Something said, right or wrong. Known by its name only.
// mitra: implements piece::Piece
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Piece(pub String);

impl Piece {
    pub fn named(name: &str) -> Piece {
        Piece(name.to_string())
    }
}

// mitra: implements piece::piece_id
pub fn piece_id(x: &Piece) -> &str {
    &x.0
}

/// What is believed of a piece. `b + d + u == 1`; `a` is the base rate:
/// what to expect with no evidence.
// mitra: implements piece::Opinion
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opinion {
    pub b: f64,
    pub d: f64,
    pub u: f64,
    pub a: f64,
}

/// Who says it.
// mitra: implements piece::Voice
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Voice {
    pub name: String,
}

impl Voice {
    pub fn named(name: &str) -> Voice {
        Voice {
            name: name.to_string(),
        }
    }
}

/// A piece that says `value` about the piece `about`. `by` is its voice.
// mitra: implements piece::Evidence
#[derive(Clone, Debug, PartialEq)]
pub struct Evidence {
    pub piece: Piece,
    pub about: Piece,
    pub value: Opinion,
    pub by: Voice,
}

/// How pieces stand to each other.
// mitra: implements piece::Relation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    /// The first holds when all the others do.
    RestsOn,
    /// They cannot all be right.
    Exclude,
}

// mitra: implements piece::Related
#[derive(Clone, Debug, PartialEq)]
pub struct Related {
    pub how: Relation,
    pub pieces: Vec<Piece>,
}

/// A model: all that is said of a system, put together. Its pieces
/// are known by the evidence about them, the relations between them
/// (one relation can hold many pieces) and the base rate of each piece
/// that has one. A piece with none has 0: nothing is assumed.
// mitra: implements piece::Model
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Model {
    pub evidence: Vec<Evidence>,
    pub related: Vec<Related>,
    pub base: Vec<(Piece, f64)>,
}

/// How independent two voices are, from 0 (one voice) to 1.
///
/// Only the two ends for now: a voice is its name, not read further.
// mitra: implements piece::apart
pub fn apart(a: &Voice, b: &Voice) -> f64 {
    match a == b {
        true => 0.0,
        false => 1.0,
    }
}
