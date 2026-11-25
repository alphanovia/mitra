//! Names the model can use (finding 12c, the user's go 2026-11-25): a
//! word the grammar uses is a keyword only where it uses it, a name
//! anywhere else; `r#match` writes a reserved one; and the model's own
//! function comes before a built-in of the same name.

use mitra_lang::agree::{agree, Agreement, Settings};
use mitra_lang::check::check;
use mitra_lang::model::Model;
use mitra_lang::parse::{parse, RESERVED};
use mitra_lang::print::print;

fn model(text: &str) -> Model {
    let m = parse(text).unwrap_or_else(|e| panic!("{}", e.render("t.mitra", text)));
    assert_eq!(check(&m), vec![], "{text}");
    m
}

/// Each comparison holds: the names read as written.
fn holds(text: &str) {
    let m = model(text);
    let (agreed, findings) = agree(&m, &Settings::default());
    assert!(!agreed.is_empty(), "{text}");
    for a in &agreed {
        assert_eq!(a.agreement, Agreement::Holds, "{a:?}\n{findings:?}");
    }
    // And print writes what parse reads back.
    let printed = print(&m);
    assert_eq!(
        parse(&printed).unwrap_or_else(|e| panic!("{}", e.render("p", &printed))),
        m,
        "{printed}"
    );
}

/// Rust's `Taking` has a field `by`: a field, a field read, a literal.
#[test]
fn a_field_named_by_a_keyword() {
    holds(
        "subject s {
            subject record Taking { count: integer, by: integer }
            fn total(t: Taking) -> n: integer { t.count + t.by } where {
                examples { (Taking { count: 2, by: 3 }) -> 5 }
            }
        }",
    );
}

/// Parameters, a result, a function and a let, each named by a word
/// the grammar uses elsewhere.
#[test]
fn names_where_no_keyword_is_read() {
    holds(
        "subject s {
            fn record(from: integer, at: integer) -> by: integer { let where = from + at; where } where {
                examples { (1, 2) -> 3 }
            }
            fn keep(x: integer) -> y: integer { record(x, 0) } where {
                examples { (4) -> 4 }
            }
        }",
    );
}

/// An ensure and a require named by keywords, and a claim whose result
/// is `by` with a proof after it.
#[test]
fn statements_named_by_keywords() {
    let m = model(
        "subject t {
            ensure from: \"it holds\";
        }
        subject s {
            require at: t::from;
            fn f(x: integer) -> by: integer { x } where {
                claim same: by == x;
            }
        }",
    );
    assert_eq!(m.parts[0].ensures[0].name, "from");
    assert_eq!(m.parts[1].requires[0].name, "at");
}

/// `within` of the model's own: it comes before the built-in.
#[test]
fn the_models_function_comes_before_a_built_in() {
    holds(
        "subject s {
            fn within(a: integer, b: integer) -> yes: bool { a <= b }
            fn g(a: integer) -> y: bool { within(a, a + 1) } where {
                examples { (3) -> true }
            }
        }",
    );
    // From another part, by `use`.
    holds(
        "subject location {
            fn within(a: integer, b: integer) -> yes: bool { a <= b }
        }
        subject s {
            use location::within;
            fn g(a: integer) -> y: bool { within(a, a - 1) } where {
                examples { (3) -> false }
            }
        }",
    );
    // Elsewhere the built-in stays.
    holds(
        "subject s {
            fn near(a: rational) -> y: bool { within(a, 1.0, 0.5) } where {
                examples { (1.25) -> true }
            }
        }",
    );
}

/// A reserved word is a name written raw, as Rust writes it.
#[test]
fn a_raw_name_writes_a_reserved_word() {
    holds(
        "subject s {
            fn r#match(r#if: integer) -> r#let: integer { let r#else = r#if; r#else + 1 } where {
                examples { (1) -> 2 }
            }
            fn g(x: integer) -> y: integer { r#match(x) } where {
                examples { (1) -> 2 }
            }
        }",
    );
    // `subject record` is the shorthand: a part named `record` is raw.
    let m = model("subject r#record { fn f(x: integer) -> y: integer { x } }");
    assert_eq!(m.parts[0].name, "record");
}

/// Writing a reserved word as a name says which, and how to write it.
#[test]
fn a_reserved_word_is_said() {
    // The words an expression can begin with, as docs/reference/2-parts.md lists them.
    assert_eq!(
        RESERVED,
        ["let", "if", "else", "match", "true", "false", "all", "any", "crate"]
    );
    for w in RESERVED {
        let text = format!("subject s {{ fn f({w}: integer) -> y: integer {{ 0 }} }}");
        let e = parse(&text).unwrap_err();
        assert_eq!(
            e.message,
            format!("expected a name, found `{w}`: `{w}` is reserved, and a raw name writes it, `r#{w}`"),
            "{w}"
        );
    }
}

/// A typo near a keyword still reads as one.
#[test]
fn a_typo_near_a_keyword_is_still_said() {
    let e = parse("subject s { ensure e: \"x\" form (a); }").unwrap_err();
    assert_eq!(e.message, "expected `;`, found `form`");
    let e = parse("subject s { fn f(x: integer) -> y: integer wher { claim c: y == x; } }")
        .unwrap_err();
    assert_eq!(e.message, "expected `;`, found `wher`");
}
