//! An interpretation of the system: a piece with a content, which is
//! what it says the system is or does. The content is not read here.

use super::piece::Piece;

// mitra: implements statement::Statement
#[derive(Clone, Debug, PartialEq)]
pub struct Statement<C> {
    pub piece: Piece,
    pub content: C,
}
