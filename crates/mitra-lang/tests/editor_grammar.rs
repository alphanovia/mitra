//! The VS Code grammar (`editors/vscode/syntaxes/mitra.tmLanguage.json`)
//! colours as keywords exactly the words the parser keeps as keywords
//! (`parse::KEYWORDS`). A keyword added to or taken from the parser
//! without the grammar, or the other way round, fails here.

use mitra_lang::parse::{KEYWORDS, RESERVED};
use std::collections::BTreeSet;

const GRAMMAR: &str = include_str!("../../../editors/vscode/syntaxes/mitra.tmLanguage.json");

/// The words of every `"keyword-…"` entry's `\b(a|b|c)\b` match.
fn grammar_keywords() -> BTreeSet<String> {
    grammar_entries().into_iter().flat_map(|(_, w)| w).collect()
}

/// Each `"keyword-…"` entry: whether its match keeps names out (a
/// lookbehind before the word), and its words.
fn grammar_entries() -> Vec<(bool, Vec<String>)> {
    let mut entries = Vec::new();
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
            let guarded = line[..open].contains("(?<!\\\\.)");
            entries.push((
                guarded,
                line[open..close].split('|').map(str::to_string).collect(),
            ));
            entry = false;
        }
    }
    entries
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

/// A reserved word is never a name, so the grammar colours it
/// everywhere; any other keyword is a name outside its place, so the
/// grammar leaves it plain after `.` (a field) and the like.
#[test]
fn only_reserved_words_are_coloured_everywhere() {
    for (guarded, words) in grammar_entries() {
        for w in words {
            let reserved = RESERVED.contains(&w.as_str());
            assert_eq!(
                guarded, !reserved,
                "`{w}`: a reserved word is coloured everywhere, any other keyword only where it is one"
            );
        }
    }
}
