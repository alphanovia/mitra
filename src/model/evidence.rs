//! How an opinion moves: a piece of evidence says an opinion about
//! another piece, and counts as far as it is itself believed.

use super::piece::{Opinion, Piece};
use super::statement::Statement;

/// Who says it.
// mitra: implements evidence::Voice
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
// mitra: implements evidence::Evidence
#[derive(Clone, Debug, PartialEq)]
pub struct Evidence {
    pub piece: Piece,
    pub about: Piece,
    pub value: Opinion,
    pub by: Voice,
}

/// What the reliability of a voice says: `voice` can be relied on.
// mitra: implements evidence::Reliable
#[derive(Clone, Debug, PartialEq)]
pub struct Reliable {
    pub voice: Voice,
}

/// The reliability of a voice: a statement, implied once per voice,
/// known by the voice's name. It stands behind all that voice says.
// mitra: implements evidence::reliable
pub fn reliable(by: &Voice) -> Statement<Reliable> {
    Statement {
        piece: Piece::named(&format!("reliable: {}", by.name)),
        content: Reliable { voice: by.clone() },
    }
}
