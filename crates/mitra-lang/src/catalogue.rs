//! The catalogue: one list of codes for every layer, so that each thing
//! Mitra reports is known by a code that never changes. A code is a
//! suit, the layer that gives it, and a number in that suit: `M005`. Each
//! code has a page, `docs/reference/codes/M005.md`, with an example and the fix.
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
    /// `R`: what the reader is given from outside the model: Rust code,
    /// a `cargo test` report, a store (`src/reader.rs`).
    Reader,
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
            Suit::Reader => 'R',
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
pub const M016: Code = model(16);
pub const M017: Code = model(17);
pub const M018: Code = model(18);
pub const M019: Code = model(19);
pub const M020: Code = model(20);
pub const M021: Code = model(21);
pub const M022: Code = model(22);
pub const M023: Code = model(23);
pub const M024: Code = model(24);
pub const M025: Code = model(25);
pub const M026: Code = model(26);
pub const M027: Code = model(27);
pub const M028: Code = model(28);
pub const M029: Code = model(29);
pub const M030: Code = model(30);
pub const M031: Code = model(31);
pub const M032: Code = model(32);
pub const M033: Code = model(33);
pub const M034: Code = model(34);

const fn agree(number: u16) -> Code {
    Code {
        suit: Suit::Agree,
        number,
    }
}

pub const A001: Code = agree(1);
pub const A002: Code = agree(2);
pub const A003: Code = agree(3);
pub const A004: Code = agree(4);
pub const A005: Code = agree(5);
pub const A006: Code = agree(6);
pub const A007: Code = agree(7);
pub const A008: Code = agree(8);
pub const A009: Code = agree(9);
pub const A010: Code = agree(10);
pub const A011: Code = agree(11);
pub const A012: Code = agree(12);
pub const A013: Code = agree(13);
pub const A014: Code = agree(14);
pub const A015: Code = agree(15);
pub const A016: Code = agree(16);
pub const A017: Code = agree(17);
pub const A018: Code = agree(18);
pub const A019: Code = agree(19);
pub const A020: Code = agree(20);
pub const A021: Code = agree(21);

const fn evidence(number: u16) -> Code {
    Code {
        suit: Suit::Evidence,
        number,
    }
}

pub const V001: Code = evidence(1);
pub const V002: Code = evidence(2);
pub const V003: Code = evidence(3);
pub const V004: Code = evidence(4);
pub const V005: Code = evidence(5);
pub const V006: Code = evidence(6);
pub const V007: Code = evidence(7);
pub const V008: Code = evidence(8);
pub const V009: Code = evidence(9);

const fn explain(number: u16) -> Code {
    Code {
        suit: Suit::Explain,
        number,
    }
}

pub const X001: Code = explain(1);
pub const X002: Code = explain(2);
pub const X003: Code = explain(3);
pub const X004: Code = explain(4);
pub const X005: Code = explain(5);
pub const X006: Code = explain(6);
pub const X007: Code = explain(7);
pub const X008: Code = explain(8);
pub const X009: Code = explain(9);
pub const X010: Code = explain(10);
pub const X011: Code = explain(11);
pub const X012: Code = explain(12);
pub const X013: Code = explain(13);
pub const X014: Code = explain(14);
pub const X015: Code = explain(15);
pub const X016: Code = explain(16);

const fn blame(number: u16) -> Code {
    Code {
        suit: Suit::Blame,
        number,
    }
}

pub const B001: Code = blame(1);
pub const B002: Code = blame(2);
pub const B003: Code = blame(3);

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
pub const P006: Code = parse(6);
pub const P007: Code = parse(7);
pub const P008: Code = parse(8);
pub const P009: Code = parse(9);
pub const P010: Code = parse(10);
pub const P011: Code = parse(11);
pub const P012: Code = parse(12);
pub const P013: Code = parse(13);
pub const P014: Code = parse(14);
pub const P015: Code = parse(15);
pub const P016: Code = parse(16);
pub const P017: Code = parse(17);

const fn reader(number: u16) -> Code {
    Code {
        suit: Suit::Reader,
        number,
    }
}

pub const R001: Code = reader(1);
pub const R002: Code = reader(2);
pub const R003: Code = reader(3);
pub const R004: Code = reader(4);
pub const R005: Code = reader(5);
pub const R006: Code = reader(6);
pub const R007: Code = reader(7);
pub const R008: Code = reader(8);
pub const R009: Code = reader(9);
pub const R010: Code = reader(10);
pub const R011: Code = reader(11);
pub const R012: Code = reader(12);
pub const R013: Code = reader(13);
pub const R014: Code = reader(14);

const fn entry(code: Code, title: &'static str) -> Entry {
    Entry {
        code,
        title,
        retired: false,
    }
}

/// A code no longer given, kept so that it is never given to anything
/// else.
const fn retired(code: Code, title: &'static str) -> Entry {
    Entry {
        code,
        title,
        retired: true,
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
    entry(
        M016,
        "a use that names nothing of another part, or a name twice",
    ),
    entry(M017, "a call of no function"),
    entry(M018, "a function or record of another part, not used"),
    entry(
        M019,
        "a function that calls itself, not on a part of its input",
    ),
    entry(
        M020,
        "an impl that does not fit its contract, or names no function",
    ),
    entry(
        M021,
        "a model example that names no input or output, or states no output",
    ),
    entry(
        M022,
        "a model example not of its types, or missing an input",
    ),
    entry(M023, "a correct, all or any a rule cannot write"),
    entry(M024, "two examples at one point that differ"),
    entry(M025, "a range type that holds no number"),
    entry(M026, "an example outside the range of its type"),
    entry(
        M027,
        "an enum or a record declared so that no value fits it",
    ),
    entry(
        M028,
        "a part holding parts that holds more than parts, requires, ensures and examples",
    ),
    entry(
        M029,
        "an ensure of a part holding parts that rests on what is not its parts' ensures",
    ),
    entry(
        M030,
        "a `self` that is not the first of a method of a part with a record",
    ),
    entry(
        M031,
        "a claim `by induction` on what is not an input that is a tree",
    ),
    entry(
        M032,
        "a proof written as its step alone that no proof, or two, fill in",
    ),
    entry(
        M033,
        "a claim `by invariant` of what is not a loop's function",
    ),
    entry(M034, "a set of a rule whose range holds no point"),
    entry(A001, "a body that breaks a claim"),
    entry(A002, "a body that disagrees with an example"),
    entry(A003, "an example that breaks a claim"),
    entry(A004, "claims no value keeps together"),
    entry(A005, "a contract claim a function that fits breaks"),
    entry(A006, "an example of the model the model does not give"),
    entry(A007, "a comparison not decided"),
    // Now M024: a model with it is not well formed.
    retired(A008, "two examples at one point that differ"),
    entry(A009, "an input of a model example no output reads"),
    entry(
        A010,
        "a comparison that holds given the claims of a function with no body",
    ),
    entry(
        A011,
        "a call a row's `then` lists that the run does not make",
    ),
    entry(
        A012,
        "a value written in a row that breaks its record's claims",
    ),
    entry(A013, "a loop that never ends"),
    entry(A014, "an example whose run calls for a value nothing gives"),
    entry(A015, "a given an example's run never calls"),
    entry(A016, "an example whose run reaches a budget"),
    entry(A017, "an example the claims of a function with no body refuse"),
    entry(A018, "a given that breaks a claim of its function"),
    entry(A019, "an enum case no body makes"),
    entry(A020, "a failure a body turns into an ordinary value"),
    entry(A021, "an input a body never reads"),
    entry(V001, "a measurement that names no input or output"),
    entry(V002, "a measurement through no tie"),
    entry(
        V003,
        "a measured value not of its type, or of an opaque record",
    ),
    entry(V004, "a measurement of no run"),
    entry(V005, "a review of nothing a review can name"),
    entry(V006, "a measurement missing an input, or with no output"),
    entry(
        V007,
        "a voice missing, not of a kind, Mitra's own, or saying twice",
    ),
    retired(V008, "a measured value outside the range of its type"),
    entry(V009, "a test tied to what is no ensure"),
    entry(
        X001,
        "a measurement nothing in the model says what to expect of",
    ),
    entry(
        X002,
        "a failed test whose expected value the model does not give",
    ),
    entry(X003, "a claim a measurement does not decide"),
    entry(X004, "an output the model cannot compute from the inputs"),
    entry(X005, "an input nothing measured reads"),
    entry(X006, "judging stopped before it was done"),
    entry(X007, "an opaque function measured the same each time"),
    entry(
        X008,
        "two measurements that say one thing, by one voice and count",
    ),
    entry(X009, "a budget of steps or depth reached"),
    entry(
        X010,
        "a value known in part, where only what is not known could tell",
    ),
    entry(X011, "a measured value outside the range of its type"),
    entry(
        X012,
        "a link, or a piece said only in words, no voice reviewed",
    ),
    entry(X013, "an all or an any over a set no point is in"),
    entry(
        X014,
        "a measured value that breaks, or may break, its record's claims",
    ),
    entry(X015, "a test tied to an ensure that failed"),
    entry(X016, "a measured point at which a loop never ends"),
    entry(B001, "a conflict the model's claims alone hold"),
    entry(B002, "more diagnoses than blame looked for"),
    entry(B003, "a diagnosis a proof by cases may outlive"),
    entry(P001, "a character that starts nothing"),
    entry(P002, "a string that is not closed"),
    entry(P003, "not what the language has here"),
    entry(P004, "a tuple or record whose type cannot be told"),
    entry(P005, "a fraction with zero below"),
    entry(P006, "a `match` that misses a variant, or names one twice"),
    entry(
        P007,
        "an unknown variant, or one given the wrong number of values",
    ),
    entry(P008, "a type given the wrong type arguments"),
    entry(P009, "a value whose type cannot be told here"),
    entry(P010, "a part no file of the project declares"),
    entry(P011, "a text nested past the parser's depth budget"),
    entry(P012, "a part reopened as another kind"),
    entry(
        P013,
        "a method call on a value whose part has no such method",
    ),
    entry(P014, "a part a `use` names that two folders declare"),
    entry(P015, "a `mitra.toml` Mitra cannot read"),
    entry(P016, "a value `;` drops that is not `()`"),
    entry(P017, "a name alone names a part at two levels"),
    entry(R001, "a `mitra:` comment the reader cannot read"),
    entry(R002, "a tie comment on no item, or to nothing of the model"),
    entry(R003, "a `measures` comment on no test"),
    entry(R004, "a test that measures nothing the reader knows"),
    entry(R005, "a measuring test that did not run"),
    entry(
        R006,
        "what names nothing of the model, or is not of its types",
    ),
    entry(R007, "a measurement through no tie"),
    entry(R008, "a review seeing a test that did not pass"),
    entry(R009, "a test binary that stopped before its result"),
    entry(R010, "a test Mitra could not read in the report"),
    entry(R011, "a field or variant on one side of a tie only"),
    entry(
        R012,
        "a field whose Rust type holds values its model type does not",
    ),
    entry(R013, "a Rust form Mitra cannot write yet"),
    entry(R014, "a given no row follows"),
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

/// Less than a finding: what a reading saw that may help, at `at`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Hint {
    pub code: Code,
    pub at: Path,
    pub message: String,
}

/// A review a reading asks for: of the piece at `at`, or of its link
/// from `from`. `line` is what to write in the store to answer it.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ask {
    pub code: Code,
    pub at: Path,
    pub from: Option<Vec<Path>>,
    pub message: String,
    pub line: String,
}

impl std::fmt::Display for Ask {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "ask[{}]: {}\n --> {}\n  = help: once reviewed, write in the store `{}`",
            self.code, self.message, self.at, self.line
        )
    }
}

impl std::fmt::Display for Hint {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "hint[{}]: {}\n --> {}", self.code, self.message, self.at)
    }
}

/// As rustc writes it: `error[M005]: …`, then where.
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        // The first line says the error; the lines after it, below
        // where, show and explain it, as rustc does.
        let (first, rest) = self.message.split_once('\n').unwrap_or((&self.message, ""));
        write!(f, "error[{}]: {first}\n --> {}", self.code, self.at)?;
        if !rest.is_empty() {
            write!(f, "\n{rest}")?;
        }
        Ok(())
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
