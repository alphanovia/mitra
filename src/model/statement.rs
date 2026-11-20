//! An interpretation of the system: a piece with a content, which is
//! what it says the system is or does. The content is not read here.

use super::piece::Piece;

/// Who a statement is owed by, or given by.
// mitra: implements Owner
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Owner {
    /// Ours to show: it starts with nothing assumed.
    Owed,
    /// Taken from someone: it starts at their word.
    Given,
}

// mitra: implements statement::Statement
#[derive(Clone, Debug, PartialEq)]
pub struct Statement<C> {
    pub piece: Piece,
    pub content: C,
    pub owner: Owner,
}

/// Where the opinion of a statement starts: its base rate. `word` is
/// how far whoever gave it is believed.
// mitra: implements statement::stance
pub fn stance<C>(s: &Statement<C>, word: f64) -> f64 {
    match s.owner {
        Owner::Owed => 0.0,
        Owner::Given => word,
    }
}
