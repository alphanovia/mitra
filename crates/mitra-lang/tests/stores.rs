//! What a model brings from the model of another store.

use mitra_lang::{Handed, Module, Modules};

const THERE: &str = r#"
subject results {
    ensure kept = "A failure is kept.";
    record Outcome { name: String, passed: Bool }
    record Whole { first: Outcome, model: Model }
    enum Word { Yes, No(Int) }
    enum Wide { One(Model) }
    fn first(xs: Vec<Outcome>) -> Outcome;
    claim never() { first([]) == first([]) }
}
subject other {
    record Model { name: String }
    ensure kept = "Another promise.";
}
"#;

fn read(texts: &[&str]) -> Result<mitra_lang::Model, String> {
    let mut handed = Handed::default();
    handed.stores.insert("there".into(), THERE.into());
    handed.stores.insert("second".into(), THERE.into());
    let file = |(i, t): (usize, &&str)| Module {
        name: format!("f{i}.mitra"),
        text: t.to_string(),
    };
    let files = texts.iter().enumerate().map(file).collect();
    let modules = Modules::new(files, handed);
    mitra_lang::load_modules(&modules, "").map_err(|(_, e)| e.message)
}

// mitra: evidence for modules::bought_not_built
#[test]
fn a_block_of_another_store_is_a_boundary() {
    let m = read(&["import there::results::{Outcome, kept};
        subject here {
            require kept_there = results::kept;
            fn one(text: String) -> results::Outcome;
        }"])
    .unwrap();
    // The record with its fields, the promise as an assumption.
    let r = m.records.iter().find(|r| r.name == "results::Outcome");
    assert_eq!(r.map(|r| r.fields.len()), Some(2));
    let b = m.block_of("results::Outcome").unwrap();
    assert_eq!(
        (b.side, b.from.as_deref()),
        (mitra_lang::Side::Boundary, Some("there"))
    );
    // Its functions and claims stay there.
    assert!(!m.functions.iter().any(|f| f.name == "results::first"));
    assert!(!m.claims.iter().any(|c| c.name == "results::never"));
}

#[test]
fn two_files_bring_from_one_block() {
    let m = read(&[
        "import there::results::{Outcome};\nsubject a { fn f(o: results::Outcome) -> Bool; }",
        "import there::results::{Word, Outcome};\nsubject b { fn g(n: Int) -> results::Word; }",
    ])
    .unwrap();
    assert!(m.enums.iter().any(|e| e.name == "results::Word"));
}

#[test]
fn a_record_naming_what_is_not_brought_comes_without_fields() {
    let m = read(&["import there::results::{Whole};"]).unwrap();
    let r = m
        .records
        .iter()
        .find(|r| r.name == "results::Whole")
        .unwrap();
    assert!(r.open && r.fields.iter().all(|f| f.0 == ".."), "{r:?}");
}

#[test]
fn refusals() {
    let said = |text: &str| read(&[text]).unwrap_err();
    let e = said("import there::results::{Nope};");
    assert!(
        e.contains("has no record, enum or `ensure` named `Nope`"),
        "{e}"
    );
    let e = said("import there::nowhere::{Outcome};");
    assert!(e.contains("that store has no block `nowhere`"), "{e}");
    let e = said("import there::results::{first};");
    assert!(e.contains("named `first`"), "{e}");
    let e = said("import there::results::{Wide};");
    assert!(
        e.contains("`Wide` holds a `Model`, which must be brought with it"),
        "{e}"
    );
    let e = said("import there::results::{kept};\nsubject results { }");
    assert!(e.contains("is a block of this model too"), "{e}");
    let e = said("import there::results::{kept};\nimport second::results::{Outcome};");
    assert!(e.contains("was brought from `there` already"), "{e}");
}
