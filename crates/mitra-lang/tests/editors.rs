//! The editor grammar (`editors/vscode`) names every word the reader
//! reserves: a new keyword fails here until the grammar has it.

use std::fs;
use std::path::Path;

/// The clauses of an evidence block, and the words of a declaration that
/// are not reserved: the reader matches them where they stand.
const CLAUSES: [&str; 23] = [
    "for",
    "given",
    "holds",
    "fails",
    "passes",
    "gives",
    "sampled",
    "trust",
    "source",
    "rev",
    "uses",
    "only",
    "when",
    "then",
    "impl",
    "self",
    "Self",
    "is",
    "deterministic",
    "uniform",
    "represented",
    "ensure",
    "where",
];

#[test]
fn the_grammar_has_every_keyword() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../editors/vscode");
    let grammar = fs::read_to_string(root.join("syntaxes/mitra.tmLanguage.json")).unwrap();
    let words = mitra_lang::parse::KEYWORDS
        .iter()
        .chain(mitra_lang::parse::ITEMS.iter())
        .chain(CLAUSES.iter());
    for word in words.filter(|w| **w != "_") {
        let listed = grammar
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .any(|w| w == *word);
        assert!(listed, "`{word}` is not in the grammar");
    }
    // The clauses above are the reader's: each is matched in its source.
    let reader = include_str!("../src/parse.rs");
    for word in CLAUSES {
        assert!(
            reader.contains(&format!("\"{word}\"")),
            "`{word}` is not read"
        );
    }
}
