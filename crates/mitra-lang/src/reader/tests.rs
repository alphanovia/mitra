//! Tests of the reader as a whole.

use crate::reader::prelude::*;

fn error(text: &str) -> Error {
    model(text).map(|_| ()).unwrap_err()
}

fn entry(about: &str, source: Option<&str>, rev: Option<&str>, lines: (usize, usize)) -> Entry {
    Entry {
        about: about.to_string(),
        source: source.map(String::from),
        rev: rev.map(String::from),
        seen: None,
        first: lines.0,
        last: lines.1,
    }
}

// mitra: evidence for ledger::t_entries_one
#[test]
fn t_entries_one() {
    let text = "evidence r { for a::b holds source \"review:me\" rev \"1\" }";
    let one = entry("a::b", Some("review:me"), Some("1"), (1, 1));
    assert_eq!(entries(text), [one]);
}

// mitra: evidence for ledger::t_entries_label_is_the_source
#[test]
fn t_entries_label_is_the_source() {
    let text = "evidence ci {\n  for f given a = 1 gives 2\n}";
    assert_eq!(entries(text), [entry("f", Some("ci"), None, (1, 3))]);
}

// mitra: evidence for ledger::t_entries_none
#[test]
fn t_entries_none() {
    assert_eq!(entries("// nothing"), []);
}

#[test]
fn entries_read_each_block() {
    let text = "// head\nevidence a { for f holds seen \"12\" }\nevidence b { for g given s = \"source\" gives 1 }";
    let read = entries(text);
    assert_eq!(read.len(), 2);
    assert_eq!(read[0].seen.as_deref(), Some("12"));
    assert_eq!((read[1].about.as_str(), read[1].first), ("g", 3));
    assert_eq!(read[1].source.as_deref(), Some("b"));
}

#[test]
fn reads_the_64_bit_extremes() {
    let ok = "subject fn f(a: Int) -> Int;\n\
              claim c() { f(-9223372036854775808) == 9223372036854775807 }";
    assert!(model(ok).is_ok());
    let e = error("claim c() { 9223372036854775808 == 1 }");
    assert!(e.message.contains("outside the 64-bit integers"), "{e}");
}

#[test]
fn set_operators_bind_as_documented() {
    let (m, _) = model("subject fn f(a: 0..=9 \\ {3} | {20} & 15..=25) -> Int;").unwrap();
    let a = &m.functions[0].params[0].1;
    assert!(a.contains(&Value::Int(20)) && a.contains(&Value::Int(4)));
    assert!(!a.contains(&Value::Int(3)) && !a.contains(&Value::Int(15)));
}

#[test]
fn errors_point_at_their_line() {
    let e = error("subject fn f(a: Int) -> Int;\n\nsubject fn g(a: Nope) -> Int;");
    assert_eq!(e.line, 3);
    assert!(e.message.contains("unknown type `Nope`"), "{e}");
    let e = error("subject fn if(a: Int) -> Int;");
    assert!(e.message.contains("`if` is a keyword"), "{e}");
    let e = error("subject fn f(a: String) -> Int;\nclaim c() { f(\"a\\qb\") == 1 }");
    assert_eq!(e.line, 2);
    assert!(e.message.contains("in a string"), "{e}");
}

#[test]
fn rust_names_are_the_same_types() {
    let (m, _) =
        model("import std::rust::{Vec};\nsubject fn f(a: Vec<i32>, b: usize) -> bool;").unwrap();
    let (n, _) = model("subject fn f(a: List<Int32>, b: 0..) -> Bool;").unwrap();
    assert_eq!(m.functions[0].params, n.functions[0].params);
    assert_eq!(m.functions[0].range, n.functions[0].range);
}

#[test]
fn trust_goes_from_0_to_1() {
    let (m, names) = model("subject fn f(a: Int) -> Int;").unwrap();
    let read = |e: &str| evidence_alone(e, &m, &names);
    assert!(read("evidence t { for f given a = 1 holds trust 0.5 }").is_ok());
    let e = read("evidence t { for f given a = 1 holds trust 2 }").unwrap_err();
    assert!(e.message.contains("from 0 to 1"), "{e}");
    let e = read("evidence t { for f given b = 1 holds }").unwrap_err();
    assert!(e.message.contains("has no input `b`"), "{e}");
}

// mitra: evidence for reader::reads_a_subject
#[test]
fn reads_a_subject() {
    assert!(model("subject fn f() -> Bool;").is_ok());
}

// mitra: evidence for reader::refuses_a_bare_fn
#[test]
fn refuses_a_bare_fn() {
    assert!(model("fn f() -> Bool;").is_err());
}

/// A sweep over faulty texts at hand, not every text.
// mitra: evidence for reader::fault_is_placed sampled 13
#[test]
fn fault_is_placed() {
    let faulty = [
        "",
        "fn",
        "fn f() -> Bool;",
        "subject s {",
        "subject s { fn f( -> Bool; }",
        "claim c() { 1 + }",
        "record R { }",
        "enum E { A, A }\nsubject fn f(e: E) -> Bool;",
        "subject s { ensure w \"x\"; }",
        "subject s { require t::x; }",
        "\n\nsubject fn f() -> Nope;",
        "\"unclosed",
        "import std::nothing::{x};",
    ];
    for text in faulty {
        if let Err(e) = model(text) {
            assert!(e.line >= 1 && !e.message.is_empty(), "{text:?}: {e:?}");
        }
    }
}

#[test]
fn a_function_with_no_body_is_code_or_the_world() {
    let e = error("fn f(a: Int) -> Int;");
    assert!(e.message.contains("write `subject fn f(…)`"), "{e}");
    // Named as written, with or without a name on the block.
    let (m, names) = model(
        "subject fn f(a: Int) -> Int;\nboundary fn now() -> Int;\n\
         subject { fn g(a: Int) -> Int { a } }\nboundary { fn h() -> Int; }\n\
         subject s { fn k() -> Int; }",
    )
    .unwrap();
    let fns: Vec<&str> = m.functions.iter().map(|f| f.name.as_str()).collect();
    let code: Vec<&str> = fns
        .into_iter()
        .filter(|f| !f.ends_with("::prose"))
        .collect();
    assert_eq!(code, ["f", "now", "g", "g::body", "h", "s::k"]);
    let claims: Vec<&str> = m.claims.iter().map(|c| c.name.as_str()).collect();
    let claims: Vec<&str> = (claims.into_iter())
        .filter(|c| !c.ends_with("::deterministic"))
        .collect();
    assert_eq!(claims, ["f::correct", "g::spec", "s::k::correct"]);
    let outside: Vec<&String> = names.boundaries.iter().collect();
    assert_eq!(outside, ["h", "now"]);
    // Several unnamed blocks share the names of the file.
    let twice = "subject { fn f() -> Int; }\nsubject { fn f() -> Int; }";
    let e = crate::load(twice, "").unwrap_err().1;
    assert!(e.message.contains("`f`"), "{e}");
}

#[test]
fn an_import_is_read_in_place() {
    let (m, _) = model(
        "import std::claims::{commutative};\n\
         subject fn add(a: 0..=3, b: 0..=3) -> Int;\n\
         claim add_commutes = commutative(add, 0..=3);",
    )
    .unwrap();
    let names: Vec<&str> = m.claims.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        ["add::correct", "add::deterministic", "add_commutes"]
    );
    let e = error("import std::claims::{nope};");
    assert!(e.message.contains("declares no `nope`"), "{e}");
}
