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
    assert_eq!(verdicts(HALF, &e).lines().next(), Some("math::even_exact: holds"));
}

#[test]
fn evidence_names_what_it_is_for() {
    let e = "evidence a { for half given m = 0 gives 0 }";
    assert_eq!(refused(HALF, e), "line 1: `math::half` has no input `m`");
    let e = "evidence a { for third given n = 0 gives 0 }";
    assert_eq!(refused(HALF, e), "line 1: the model has no function or claim `third`");
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
    assert_eq!(verdicts(m, "").lines().next(), Some("clock::starts_at_zero: holds"));
    let out = verdicts(m, "evidence late { for now gives 3 }");
    assert!(out.contains("error[M"), "{out}");
}

#[test]
fn tests_in_the_model() {
    let m = format!("{HALF}\ntest t {{ for half given n = 4 gives 2 }}");
    let model = mitra_core::load(&m, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    assert_eq!(model.tests.len(), 1);
    assert_eq!(model.tests[0].name, "t");
}
