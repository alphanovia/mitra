//! Finding 45 J6 (the user's go, 2026-11-28 07:13): three gaps no
//! comparison shows, each a finding that refuses nothing. A019: a case
//! of an enum bodies make that none makes (the world's enums aside).
//! A020: a failure a body turns into an ordinary value, with no example
//! where the callee fails and no claim on it. A021: an input a body
//! never reads, `_` marking one as meant.

use mitra_lang::agree::{agree, Settings};
use mitra_lang::check::check;
use mitra_lang::parse::parse;

fn findings(text: &str) -> Vec<String> {
    let m = parse(text).unwrap();
    assert_eq!(check(&m), vec![], "{text}");
    agree(&m, &Settings::default())
        .1
        .iter()
        .map(|f| f.to_string())
        .filter(|f| ["A019", "A020", "A021"].iter().any(|c| f.contains(c)))
        .collect()
}

// mitra: supports check::cases_made
#[test]
fn a_case_no_body_makes_is_said() {
    let text = "subject enum Receipt { Received, Delivered, Read }
subject Store {
    fn post(n: integer) -> r: Receipt { if n > 0 { Receipt::Received } else { Receipt::Delivered } }
}";
    assert_eq!(
        findings(text),
        ["finding[A019]: `Receipt::Read` is never made: bodies make `Receipt::Received` (in `Store::post`), `Receipt::Delivered` (in `Store::post`), and none makes `Receipt::Read`\n --> Receipt"]
    );
    // Made by none: the world's, or not yet used; nothing said.
    let none = "subject enum Receipt { Received, Read }\nsubject Store {\n    fn post(n: integer) -> r: integer { n }\n}";
    assert_eq!(findings(none), Vec::<String>::new());
    // Given by a function with no body, in a `Result` too: the world
    // makes the others.
    let world = text.replace(
        "subject Store {",
        "environment Net {\n    fn ack(n: integer) -> r: Result<Receipt, integer>;\n}\nsubject Store {\n    use Result from \"std\";",
    );
    assert_eq!(findings(&world), Vec::<String>::new());
    // An environment's own enum: aside.
    let env = text.replace("subject enum Receipt", "environment enum Receipt");
    assert_eq!(findings(&env), Vec::<String>::new());
}

// mitra: supports check::failures_carried
#[test]
fn a_failure_turned_into_a_value_is_said() {
    let base = "environment Llm {
    fn classify(t: integer) -> r: Result<bool, integer>;
}
subject Guard {
    use Result from \"std\";
    use Llm::classify;
    fn screen(t: integer) -> ok: bool { match classify(t) { Ok(b) => b, Err(_) => true } }
}";
    assert_eq!(
        findings(base),
        ["finding[A020]: `Guard::screen` calls `Llm::classify`, which can fail, and gives bool, which has no failure: the `Err` arm gives `true`; give an example where `Llm::classify` fails, or a claim on it\n --> Guard::screen"]
    );
    // Through a `let`.
    let let_ = base.replace("match classify(t) {", "let c = classify(t); match c {");
    assert_eq!(findings(&let_).len(), 1, "{:?}", findings(&let_));
    // An example where it fails: answered.
    let tried = base.replace(
        "Err(_) => true } }",
        "Err(_) => true } } where {\n        examples { given Llm::classify(1) -> Err(5) when (1) then true }\n    }",
    );
    assert_eq!(findings(&tried), Vec::<String>::new());
    // An example where it does not fail: still said.
    let ok = tried.replace("-> Err(5) when (1) then true", "-> Ok(false) when (1) then false");
    assert_eq!(findings(&ok).len(), 1, "{:?}", findings(&ok));
    // A name bound again to what is no call: not followed (a gap of
    // this check: what a body builds from calls is not traced).
    let again = base.replace(
        "match classify(t) {",
        "let c = classify(t); let c = if t > 0 { c } else { classify(0) }; match c {",
    );
    assert_eq!(findings(&again), Vec::<String>::new());
    // A claim on it: answered.
    let claimed = base.replace(
        "Err(_) => true } }",
        "Err(_) => true } } where {\n        claim open: classify(t) == Ok(false) || ok;\n    }",
    );
    assert_eq!(findings(&claimed), Vec::<String>::new());
    // Carried: what it gives holds the failure.
    let carried = base.replace(
        "fn screen(t: integer) -> ok: bool { match classify(t) { Ok(b) => b, Err(_) => true } }",
        "fn screen(t: integer) -> ok: Result<bool, integer> { match classify(t) { Ok(b) => Ok(b), Err(e) => Err(e) } }",
    );
    assert_eq!(findings(&carried), Vec::<String>::new());
}

// mitra: supports check::inputs_read
#[test]
fn an_input_never_read_is_said() {
    let text = "subject Server {\n    fn post(store: integer, n: integer) -> r: integer { n + 1 }\n}";
    assert_eq!(
        findings(text),
        ["finding[A021]: `store` is taken by `Server::post` and never read by its body: what it is for is only in words; name it `_store` if that is meant\n --> Server::post"]
    );
    assert_eq!(findings(&text.replace("store: integer", "_store: integer")), Vec::<String>::new());
    // Read in a nested binding: read.
    let read = text.replace("{ n + 1 }", "{ let k = store; k + n }");
    assert_eq!(findings(&read), Vec::<String>::new());
    // Hidden by a binding of the same name: not read.
    let hidden = text.replace("{ n + 1 }", "{ let store = 2; store + n }");
    assert_eq!(findings(&hidden).len(), 1);
    // A function with no body takes what it takes.
    assert_eq!(findings("subject S {\n    fn f(a: integer) -> b: integer;\n}"), Vec::<String>::new());
}
