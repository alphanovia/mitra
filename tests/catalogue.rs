//! The catalogue (`src/catalogue.rs`): every code listed once, and
//! numbered in its suit with no gap, so that none is taken out. Each has
//! its page in `docs/reference`, which names it as listed, with an example
//! and the fix, and every page there is of a listed code.

use mitra_core::catalogue::{entry_of, Code, Error, Finding, Suit, CATALOGUE, M002, M005};
use mitra_core::model::Path;
use std::collections::BTreeSet;

#[test]
fn every_code_is_listed_once_and_kept() {
    let codes: Vec<Code> = CATALOGUE.iter().map(|e| e.code).collect();
    let once: BTreeSet<Code> = codes.iter().copied().collect();
    assert_eq!(once.len(), codes.len());
    let suits = [
        Suit::Expression,
        Suit::Model,
        Suit::Agree,
        Suit::Evidence,
        Suit::Explain,
        Suit::Blame,
    ];
    for suit in suits {
        let numbers: Vec<u16> = codes
            .iter()
            .filter(|c| c.suit == suit)
            .map(|c| c.number)
            .collect();
        let kept: Vec<u16> = (1..=numbers.len() as u16).collect();
        assert_eq!(numbers, kept, "{suit:?}: a gap, or out of order");
    }
}

#[test]
fn every_code_is_documented() {
    let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/docs/reference");
    for entry in CATALOGUE {
        let file = format!("{dir}/{}.md", entry.code);
        let page = std::fs::read_to_string(&file).unwrap_or_else(|_| panic!("no page {file}"));
        let title = format!("# {}: {}", entry.code, entry.title);
        assert_eq!(page.lines().next(), Some(title.as_str()));
        assert!(page.contains("\n## Example\n"), "{file}: no example");
        assert!(page.contains("\n## Fix\n"), "{file}: no fix");
        assert!(page.lines().count() < 200, "{file}: too long");
    }
    let listed: BTreeSet<String> = CATALOGUE.iter().map(|e| format!("{}.md", e.code)).collect();
    for file in std::fs::read_dir(dir).unwrap() {
        let name = file.unwrap().file_name().into_string().unwrap();
        assert!(listed.contains(&name), "{name} is no code of the catalogue");
    }
}

/// A code is its suit's letter and three digits. An error and a
/// finding read as rustc writes its errors.
#[test]
fn codes_read_as_written() {
    assert_eq!(M005.to_string(), "M005");
    let agree = Code {
        suit: Suit::Agree,
        number: 1,
    };
    assert_eq!(agree.to_string(), "A001");
    assert_eq!(
        entry_of(M002).map(|e| e.title),
        Some("a name is declared twice")
    );
    assert_eq!(entry_of(agree), None);
    let at = Path::to(&["stone", "drop", "falls"]);
    let error = Error {
        code: M002,
        at: at.clone(),
        message: "`falls` is declared twice".to_string(),
    };
    let written = "error[M002]: `falls` is declared twice\n --> stone::drop::falls";
    assert_eq!(error.to_string(), written);
    let finding = Finding {
        code: agree,
        at,
        message: "holds".to_string(),
    };
    assert_eq!(
        finding.to_string(),
        "finding[A001]: holds\n --> stone::drop::falls"
    );
}
