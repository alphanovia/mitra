//! The catalogue: one list of codes for every layer, so that each thing
//! Mitra reports is known by a code that never changes. A code is a
//! suit, the layer that gives it, and a number in that suit: `M005`. Each
//! code has a page, `docs/reference/M005.md`, with an example and the fix.
//!
//! An `Error` says that the model is not well formed: it must be fixed
//! before anything reads it. A `Finding` says what a reading found in a
//! well-formed model: a verdict, not a fault. They share the catalogue.

use crate::model::Path;

/// The layer that gives a code.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suit {
    /// `E`: an expression, in the core.
    Expression,
    /// `M`: the model as written, `check`.
    Model,
    /// `A`: the model against itself, with no evidence.
    Agree,
    /// `V`: the evidence.
    Evidence,
    /// `X`: the model and the evidence, explaining the system.
    Explain,
    /// `B`: which evidence is wrong.
    Blame,
    /// `P`: the text of a model, read (`src/parse.rs`).
    Parse,
}

impl Suit {
    pub fn letter(self) -> char {
        match self {
            Suit::Expression => 'E',
            Suit::Model => 'M',
            Suit::Agree => 'A',
            Suit::Evidence => 'V',
            Suit::Explain => 'X',
            Suit::Blame => 'B',
            Suit::Parse => 'P',
        }
    }
}

/// A code: its suit and its number in it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Code {
    pub suit: Suit,
    pub number: u16,
}

/// `M005`.
impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}{:03}", self.suit.letter(), self.number)
    }
}

/// A code in the catalogue, and what it says in a few words. A retired
/// code is no longer given, and never given to anything else.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Entry {
    pub code: Code,
    pub title: &'static str,
    pub retired: bool,
}

const fn model(number: u16) -> Code {
    Code {
        suit: Suit::Model,
        number,
    }
}

pub const M001: Code = model(1);
pub const M002: Code = model(2);
pub const M003: Code = model(3);
pub const M004: Code = model(4);
pub const M005: Code = model(5);
pub const M006: Code = model(6);
pub const M007: Code = model(7);
pub const M008: Code = model(8);
pub const M009: Code = model(9);
pub const M010: Code = model(10);
pub const M011: Code = model(11);
pub const M012: Code = model(12);
pub const M013: Code = model(13);
pub const M014: Code = model(14);
pub const M015: Code = model(15);

const fn parse(number: u16) -> Code {
    Code {
        suit: Suit::Parse,
        number,
    }
}

pub const P001: Code = parse(1);
pub const P002: Code = parse(2);
pub const P003: Code = parse(3);
pub const P004: Code = parse(4);
pub const P005: Code = parse(5);

const fn entry(code: Code, title: &'static str) -> Entry {
    Entry {
        code,
        title,
        retired: false,
    }
}

/// Every code, in order. Nothing is ever taken out: a code no longer
/// given is marked retired.
pub const CATALOGUE: &[Entry] = &[
    entry(M001, "a name is not a word"),
    entry(M002, "a name is declared twice"),
    entry(M003, "a type that no record declares"),
    entry(M004, "a record made other than as declared"),
    entry(M005, "a body not of the type its function gives"),
    entry(M006, "a claim that is not a truth"),
    entry(M007, "a name that names no proposition"),
    entry(M008, "a proposition of another part"),
    entry(M009, "a require from no ensure of another part"),
    entry(M010, "a tie to no part, record or function"),
    entry(M011, "an empty list of an ensure"),
    entry(M012, "a proof that does not show its rule"),
    entry(M013, "a declaration that rests on itself"),
    entry(M014, "a rule named through itself"),
    entry(M015, "an example not of its function's types"),
    entry(P001, "a character that starts nothing"),
    entry(P002, "a string that is not closed"),
    entry(P003, "not what the language has here"),
    entry(P004, "a tuple or record whose type cannot be told"),
    entry(P005, "a fraction with zero below"),
];

/// The entry of `code`, `None` when the catalogue has none.
pub fn entry_of(code: Code) -> Option<&'static Entry> {
    CATALOGUE.iter().find(|e| e.code == code)
}

/// The model is not well formed at `at`, the declaration to fix.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Error {
    pub code: Code,
    pub at: Path,
    pub message: String,
}

/// What a reading found in a well-formed model, at `at`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Finding {
    pub code: Code,
    pub at: Path,
    pub message: String,
}

/// As rustc writes it: `error[M005]: …`, then where.
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "error[{}]: {}\n --> {}",
            self.code, self.message, self.at
        )
    }
}

/// `finding[A001]: …`, then where.
impl std::fmt::Display for Finding {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "finding[{}]: {}\n --> {}",
            self.code, self.message, self.at
        )
    }
}
