//! The reworked syntax: each folder of `examples/reworked/` holds
//! `model.mitra`, `evidence.mitra` and the report Mitra must give
//! (`MITRA_RECORD=1 cargo test` writes it again).

use std::{fs, path::Path};

#[test]
fn examples() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/reworked");
    let mut seen = 0;
    for entry in fs::read_dir(root).unwrap() {
        let dir = entry.unwrap().path();
        let read = |f: &str| fs::read_to_string(dir.join(f)).unwrap();
        mitra_core::surface::imports_from(Some(&dir));
        let report = mitra_core::run(&read("model.mitra"), &read("evidence.mitra"))
            .unwrap_or_else(|(_, e)| panic!("{}: {e}", dir.display()));
        if std::env::var("MITRA_RECORD").is_ok() {
            fs::write(dir.join("expected.txt"), report.to_text()).unwrap();
        }
        assert_eq!(report.to_text(), read("expected.txt"), "{}", dir.display());
        seen += 1;
    }
    assert!(seen >= 3);
}

fn verdicts(model: &str, evidence: &str) -> String {
    mitra_core::run(model, evidence)
        .unwrap_or_else(|(_, e)| panic!("{e}"))
        .to_text()
}

fn refused(model: &str, evidence: &str) -> String {
    match mitra_core::run(model, evidence) {
        Err((_, e)) => e.to_string(),
        Ok(r) => panic!("accepted: {}", r.to_text()),
    }
}

const HALF: &str = "subject math {
    fn half(n: 0..=5) -> 0..=2;
    claim even_exact(n: 0..=5) when n == 0 || n == 2 || n == 4 { half(n) + half(n) == n }
}";

#[test]
fn when_on_inputs_takes_them_out_of_the_claim() {
    let e = "evidence a { for half given n = 0 gives 0 }
evidence b { for half given n = 2 gives 1 }";
    let out = verdicts(HALF, e);
    assert!(out.contains("no evidence on  `n = 4`"), "{out}");
    let e = format!("{e}\nevidence c {{ for math::half given n = 4 gives 2 }}");
    assert_eq!(
        verdicts(HALF, &e).lines().next(),
        Some("math::even_exact: holds")
    );
}

#[test]
fn evidence_names_what_it_is_for() {
    let e = "evidence a { for half given m = 0 gives 0 }";
    assert_eq!(refused(HALF, e), "line 1: `math::half` has no input `m`");
    let e = "evidence a { for third given n = 0 gives 0 }";
    assert_eq!(
        refused(HALF, e),
        "line 1: the model has no function or claim `third`"
    );
    let e = "evidence a { for half given n = 0 }";
    assert!(refused(HALF, e).contains("says nothing"));
}

#[test]
fn earlier_evidence_lines_are_read_beside_blocks() {
    let e = "-- a line that `mitra add` wrote
@[source \"x\"] evidence math::half(0) = 0 at 1
evidence b { for half given n in {2, 4} holds }
evidence c { for even_exact given n = 2 fails trust 0.5 }";
    let out = verdicts(HALF, e);
    assert!(out.starts_with("math::even_exact: fails"), "{out}");
}

#[test]
fn a_body_in_a_subject_is_a_spec_and_a_claim() {
    let m = "subject s { fn double(n: 0..=2) -> 0..=4 { n + n } }";
    let e = "evidence a { for double given n = 2 gives 3 }";
    let out = verdicts(m, e);
    assert!(out.starts_with("s::double::spec: fails"), "{out}");
}

#[test]
fn assume_is_taken_as_true_until_evidence_says_otherwise() {
    let m = "boundary clock { fn now() -> 0..=9; assume starts_at_zero() { now() == 0 } }";
    let first = |e: &str| verdicts(m, e).lines().next().map(String::from);
    // Nothing but its own word: said so, and no warning for it.
    assert_eq!(
        first("").as_deref(),
        Some("clock::starts_at_zero: holds assuming [itself]")
    );
    assert!(!verdicts(m, "").contains("warning"), "{}", verdicts(m, ""));
    // Observed: it holds.
    let seen = "evidence seen { for now gives 0 }";
    assert_eq!(first(seen).as_deref(), Some("clock::starts_at_zero: holds"));
    let out = verdicts(m, "evidence late { for now gives 3 }");
    assert!(out.contains("error[M"), "{out}");
}

#[test]
fn a_boundary_may_answer_differently_each_time() {
    let m = "boundary dice { fn roll() -> 1..=6; }\n\
             subject game { claim small() { dice::roll() <= 3 } }";
    let two = "evidence a { for dice::roll gives 1 }\nevidence b { for dice::roll gives 2 }";
    let out = verdicts(m, two);
    assert!(out.starts_with("game::small: holds"), "{out}");
    assert!(!out.contains("M003"), "{out}");
    // True of one answer and not of another: the claim is about each.
    let out = verdicts(
        m,
        &format!("{two}\nevidence c {{ for dice::roll gives 5 }}"),
    );
    assert!(out.starts_with("game::small: fails"), "{out}");
    assert!(!out.contains("the code of `dice::roll`"), "{out}");
    // A subject can't: two outputs on one input contradict each other.
    let m = m.replace("boundary", "subject");
    assert!(verdicts(&m, two).contains("M003"));
}

#[test]
fn a_claim_with_parameters_is_written_once() {
    let m = "claim commutes(f, T) { given a: T, b: T then f(a, b) == f(b, a) }\n\
             subject s { fn add(a: 0..=1, b: 0..=1) -> 0..=2;\n\
             claim add_commutes = commutes(add, 0..=1); }";
    let model = mitra_core::load(m, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    assert_eq!(model.claims.len(), 1);
    assert_eq!(model.claims[0].name, "s::add_commutes");
    let bad = m.replace("commutes(add, 0..=1)", "commutes(add)");
    let e = mitra_core::load(&bad, "").unwrap_err().1.message;
    assert!(e.contains("takes 2 parameter(s), got 1"), "{e}");
    let bad = m.replace("= commutes(add", "= swaps(add");
    let e = mitra_core::load(&bad, "").unwrap_err().1.message;
    assert!(e.contains("no claim with parameters `swaps`"), "{e}");
}

#[test]
fn an_inline_function_and_the_elements_that_fit() {
    let m = "subject s { claim c() { (|a, b| a - b)(3, 1) == 2 && (|a, b| a - b)(1, 3) == -2 }\n\
             claim d(a: 0..=3, b: 0..=3) { (|a, b| a - b)(b, a) == b - a }\n\
             claim e() { { x in {1, 2, 3, 4} | x > 2 } == {3, 4} } }";
    let out = verdicts(m, "");
    assert!(
        out.starts_with("s::c: holds\ns::d: holds\ns::e: holds"),
        "{out}"
    );
}

#[test]
fn tests_in_the_model() {
    let m = format!("{HALF}\ntest t {{ for half given n = 4 gives 2 }}");
    let model = mitra_core::load(&m, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    assert_eq!(model.tests.len(), 1);
    assert_eq!(model.tests[0].name, "t");
}

#[test]
fn import_takes_what_it_names() {
    let m = "import std::claims::{commutative};\n\
             subject s { fn add(a: 0..=1, b: 0..=1) -> 0..=2;\n\
             claim add_commutes = commutative(add, 0..=1); }";
    let model = mitra_core::load(m, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    assert_eq!(model.claims.len(), 1);
    let error = |m: &str| mitra_core::load(m, "").unwrap_err().1.message;
    // Not named, not there.
    let e = error(&m.replace("{commutative}", "{neutral}"));
    assert!(e.contains("no claim with parameters `commutative`"), "{e}");
    let e = error("import std::claims::{nothing};\nsubject s { }");
    assert!(e.contains("`std::claims` declares no `nothing`"), "{e}");
    let e = error("import std::nowhere::x;\nsubject s { }");
    assert!(e.contains("no module `std::nowhere` in `std`"), "{e}");
}

#[test]
fn represented_says_nothing_outside_its_set() {
    let m = "import std::claims::{represented};\n\
             subject s { fn half(n: 0..=9) -> 0..=4 { n / 2 }\n\
             claim low = represented(half, 0..=4);\n\
             claim high = represented(half, { n | n > 4 }); }";
    // One tested input stands for its set only, and the claim that says
    // so is an assumption until someone vouches for it.
    let out = verdicts(m, "evidence a { for half given n = 2 gives 1 }");
    assert!(
        out.contains("assumed on      n in 0..=1 | 3..=4, by `s::low`"),
        "{out}"
    );
    assert!(out.contains("no evidence on  n in 5..=9"), "{out}");
    let both = "evidence a { for half given n = 2 gives 1 }\n\
                evidence b { for half given n = 8 gives 4 }";
    let out = verdicts(m, both);
    assert!(
        out.starts_with("s::half::spec: holds assuming [s::low, s::high]"),
        "{out}"
    );
    let vouched = format!(
        "{both}\nevidence c {{ for low holds source \"review:me\" }}\n\
         evidence d {{ for high holds source \"review:me\" }}"
    );
    assert!(verdicts(m, &vouched).starts_with("s::half::spec: holds\n"));
    // Two inputs of one set that do not behave the same deny the claim.
    let out = verdicts(
        m,
        &format!("{vouched}\nevidence e {{ for half given n = 6 gives 0 }}"),
    );
    assert!(out.contains("s::high: fails"), "{out}");
    assert!(out.contains("error[M002]"), "{out}");
    assert!(out.contains("do not behave the same"), "{out}");
    // Not imported, not there.
    let e = mitra_core::load(&m.replace("import std::claims::{represented};", ""), "");
    assert!(e
        .unwrap_err()
        .1
        .message
        .contains("no claim with parameters `represented`"));
}
