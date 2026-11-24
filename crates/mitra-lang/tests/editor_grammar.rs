//! The VS Code grammar (`editors/vscode/syntaxes/mitra.tmLanguage.json`)
//! colours as keywords exactly the words the parser keeps as keywords
//! (`parse::KEYWORDS`). A keyword added to or taken from the parser
//! without the grammar, or the other way round, fails here.

use mitra_lang::parse::KEYWORDS;
use std::collections::BTreeSet;

const GRAMMAR: &str = include_str!("../../../editors/vscode/syntaxes/mitra.tmLanguage.json");

/// The words of every `"keyword-…"` entry's `\b(a|b|c)\b` match.
fn grammar_keywords() -> BTreeSet<String> {
    let mut words = BTreeSet::new();
    let mut entry = false;
    for line in GRAMMAR.lines() {
        let line = line.trim();
        if line.starts_with("\"keyword-") && line.ends_with('{') {
            entry = true;
        } else if entry && line.starts_with("\"match\"") {
            let open = line
                .find("\\\\b(")
                .expect("a keyword match is \\\\b(a|b)\\\\b")
                + 4;
            let close = line
                .rfind(")\\\\b")
                .expect("a keyword match is \\\\b(a|b)\\\\b");
            words.extend(line[open..close].split('|').map(str::to_string));
            entry = false;
        }
    }
    words
}

#[test]
fn the_grammar_colours_the_parser_keywords() {
    let parser: BTreeSet<String> = KEYWORDS.iter().map(|k| k.to_string()).collect();
    let grammar = grammar_keywords();
    assert!(
        !grammar.is_empty(),
        "no \"keyword-…\" entry read in the grammar"
    );
    let missing: Vec<_> = parser.difference(&grammar).collect();
    let extra: Vec<_> = grammar.difference(&parser).collect();
    assert!(
        missing.is_empty() && extra.is_empty(),
        "editors/vscode/syntaxes/mitra.tmLanguage.json drifted from src/parse.rs KEYWORDS:\n  \
         keywords the grammar does not colour: {missing:?}\n  \
         grammar keywords the parser does not have: {extra:?}\n\
         Update the grammar's \"keyword-…\" entries in the same commit as the parser."
    );
}
