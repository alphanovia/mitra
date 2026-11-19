//! The points and the sweeps behind the claims of `.mitra/model.mitra`
//! on the language itself: each test is tied to the claim it checks.

use mitra_lang::{evaluate, load, run, Logic};
use std::fs;
use std::path::Path;

const F: &str = "subject fn f() -> Bool;";

fn words(model: &str, ledger: &str) -> Vec<Logic> {
    let report = run(model, ledger).unwrap_or_else(|(_, e)| panic!("{e}"));
    report.verdicts.iter().map(|v| v.logic).collect()
}

// mitra: evidence for types::good_is_read
#[test]
fn good_is_read() {
    assert!(load(F, "").is_ok());
}

// mitra: evidence for types::unknown_is_refused
#[test]
fn unknown_is_refused() {
    assert!(load("claim c() { g() }", "").is_err());
}

// mitra: evidence for types::twice_is_refused
#[test]
fn twice_is_refused() {
    assert!(load("subject fn f() -> Bool; subject fn f() -> Bool;", "").is_err());
}

// mitra: evidence for types::arity_is_refused
#[test]
fn arity_is_refused() {
    assert!(load("subject fn f(a: Int) -> Bool; claim c() { f(1, 2) }", "").is_err());
}

// mitra: evidence for types::endless_is_refused
#[test]
fn endless_is_refused() {
    let m = "fn f(a: Int) -> Int { g(a) } fn g(a: Int) -> Int { f(a) }";
    assert!(load(m, "").is_err());
}

// mitra: evidence for verdicts::untested_is_unverified
#[test]
fn untested_is_unverified() {
    assert_eq!(words(F, ""), [Logic::Unverified, Logic::Unverified]);
}

// mitra: evidence for verdicts::tested_holds
#[test]
fn tested_holds() {
    assert_eq!(
        words(F, "evidence a { for f holds }"),
        [Logic::Holds, Logic::Unverified]
    );
}

// mitra: evidence for verdicts::refuted_fails
#[test]
fn refuted_fails() {
    assert_eq!(
        words(F, "evidence a { for f fails }"),
        [Logic::Fails, Logic::Unverified]
    );
}

// mitra: evidence for verdicts::trust_changes_no_verdict
#[test]
fn trust_changes_no_verdict() {
    assert_eq!(
        words(F, "evidence a { for f holds trust 0.1 }"),
        [Logic::Holds, Logic::Unverified]
    );
}

// mitra: evidence for verdicts::conflict_counts_for_neither
#[test]
fn conflict_counts_for_neither() {
    let ledger = "evidence a { for f holds } evidence b { for f fails }";
    let report = run(F, ledger).unwrap_or_else(|(_, e)| panic!("{e}"));
    assert_eq!(report.problems.len(), 1);
    assert_eq!(words(F, ledger), [Logic::Unverified, Logic::Unverified]);
}

// mitra: evidence for verdicts::follows_from_a_vouched_claim
#[test]
fn follows_from_a_vouched_claim() {
    let m = "subject fn add(a: Int, b: Int) -> Int; \
             claim c(a: Int, b: Int) { add(a, b) == add(b, a) } \
             claim d(a: Int, b: Int) { add(b, a) == add(a, b) }";
    let got = words(m, "evidence r { for c holds }");
    assert_eq!(
        got,
        [
            Logic::Holds,
            Logic::Holds,
            Logic::Unverified,
            Logic::Unverified
        ]
    );
}

/// Every golden case: as many verdicts as the model has claims. A sweep
/// over what is at hand, not every model.
// mitra: evidence for verdicts::one_verdict_each sampled 50
#[test]
fn one_verdict_each() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden");
    let mut seen = 0;
    for group in fs::read_dir(root).unwrap() {
        let group = group.unwrap().path();
        if !group.is_dir() {
            continue;
        }
        for case in fs::read_dir(&group).unwrap() {
            let case = case.unwrap().path();
            let read = |f: &str| fs::read_to_string(case.join(f)).unwrap_or_default();
            let Ok(model) = load(&read("model.mitra"), &read("evidence.mitra")) else {
                continue;
            };
            let Ok(report) = evaluate(&model) else {
                continue;
            };
            assert_eq!(
                report.verdicts.len(),
                model.claims.len(),
                "{}",
                case.display()
            );
            seen += 1;
        }
    }
    assert!(seen >= 50, "{seen} cases");
}
