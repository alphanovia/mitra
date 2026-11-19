//! The points and the sweeps behind the claims of the model in `.mitra/`
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

// What a model is made of: one test per point claim of functions.mitra,
// claims.mitra, intent.mitra, ledger.mitra, model.mitra, modules.mitra.

fn claims(model: &str, ledger: &str) -> Vec<String> {
    let m = load(model, ledger).unwrap_or_else(|(_, e)| panic!("{e}"));
    m.claims.iter().map(|c| c.name.clone()).collect()
}

fn named(model: &str, ledger: &str) -> Vec<(String, Logic)> {
    let report = run(model, ledger).unwrap_or_else(|(_, e)| panic!("{e}"));
    let each = report.verdicts.iter();
    each.map(|v| (v.claim.clone(), v.logic)).collect()
}

const C: &str = "subject fn f(a: Int) -> Int; claim c(a: Int) { f(a) >= 0 }";
const UNTOUCHED: [Logic; 3] = [Logic::Unverified, Logic::Unverified, Logic::Unverified];

// mitra: evidence for model::nothing_written_nothing_held
#[test]
fn nothing_written_nothing_held() {
    let m = load("", "").unwrap();
    assert!(m.functions.is_empty() && m.claims.is_empty());
    assert!(m.ensures.is_empty() && m.evidence.is_empty());
}

// mitra: evidence for model::world_is_listed
#[test]
fn world_is_listed() {
    let m = load("boundary fn now() -> Int; subject fn f() -> Int;", "").unwrap();
    assert_eq!(m.boundaries, ["now"]);
}

// mitra: evidence for model::blocks_are_listed
#[test]
fn blocks_are_listed() {
    let text = "boundary clock { fn now() -> Int; } subject s { fn f() -> Int; }";
    let m = load(text, "").unwrap();
    let said: Vec<_> = m.blocks.iter().map(|b| (b.name.as_str(), b.side)).collect();
    use mitra_lang::Side::{Boundary, Subject};
    assert_eq!(said, [("clock", Boundary), ("s", Subject)]);
    assert!(m.blocks.iter().all(|b| b.from.is_none()));
    assert_eq!(m.block_of("s::f").map(|b| b.side), Some(Subject));
    assert!(m.block_of("s").is_none() && m.block_of("other::f").is_none());
}

// mitra: evidence for model::a_block_written_twice_is_one
#[test]
fn a_block_written_twice_is_one() {
    let m = load(
        "subject s { fn f() -> Int; } subject s { fn g() -> Int; }",
        "",
    )
    .unwrap();
    assert_eq!(m.blocks.len(), 1);
}

// mitra: evidence for functions::code_is_claimed_correct
#[test]
fn code_is_claimed_correct() {
    assert_eq!(claims(F, ""), ["f::correct", "f::deterministic"]);
}

// mitra: evidence for functions::spec_replaces_correct
#[test]
fn spec_replaces_correct() {
    let m = "subject fn f() -> Bool { true }";
    assert_eq!(claims(m, ""), ["f::spec", "f::deterministic"]);
}

// mitra: evidence for functions::helper_claims_nothing
#[test]
fn helper_claims_nothing() {
    let m = load("fn h() -> Bool { true }", "").unwrap();
    assert!(m.claims.is_empty());
    let names: Vec<&str> = m.functions.iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names, ["h"]);
}

// mitra: evidence for functions::world_claims_nothing
#[test]
fn world_claims_nothing() {
    assert!(claims("boundary fn now() -> Int;", "").is_empty());
}

// mitra: evidence for claims::written_claim_is_named
#[test]
fn written_claim_is_named() {
    let m = "subject fn f(a: Int) -> Int; claim c(a: Int) { f(a) == f(a) }";
    assert_eq!(claims(m, ""), ["c", "f::correct", "f::deterministic"]);
}

// mitra: evidence for claims::assume_is_named_and_assumed
#[test]
fn assume_is_named_and_assumed() {
    let m = "subject fn f(a: Int) -> Int; assume c(a: Int) { f(a) >= 0 }";
    let all = named(m, "");
    let names: Vec<&str> = all.iter().map(|v| v.0.as_str()).collect();
    assert_eq!(names, ["c", "f::correct", "f::deterministic"]);
    let words: Vec<Logic> = all.iter().map(|v| v.1).collect();
    assert_eq!(
        words,
        [Logic::Assuming, Logic::Unverified, Logic::Unverified]
    );
}

// mitra: evidence for claims::uses_is_named
#[test]
fn uses_is_named() {
    let m = "subject fn f() -> Int; subject fn g() -> Int uses f;";
    assert!(claims(m, "").contains(&"g::uses".to_string()));
}

// mitra: evidence for claims::uses_is_read_from_a_body
#[test]
fn uses_is_read_from_a_body() {
    let m = "subject fn f() -> Int; subject fn g() -> Int { f() }";
    assert!(claims(m, "").contains(&"g::uses".to_string()));
}

// mitra: evidence for claims::tie_is_named_once_evidence_names_it
#[test]
fn tie_is_named_once_evidence_names_it() {
    let all = claims("subject fn f() -> Int;", "evidence t { for f::tied holds }");
    assert_eq!(all, ["f::correct", "f::deterministic", "f::tied"]);
}

// mitra: evidence for claims::trust_is_kept
#[test]
fn trust_is_kept() {
    let m = "subject fn f(a: Int) -> Int; claim c(a: Int) trust 0.5 { f(a) >= 0 }";
    let m = load(m, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    let trust: Vec<Option<f64>> = m.claims.iter().map(|c| c.trust).collect();
    assert_eq!(trust, [Some(0.5), None, None]);
}

const SAFE: &str = "subject s { ensure safe = \"Never negative.\"; fn f(a: Int) -> Int; \
                    claim c(a: Int) { f(a) >= 0 } }";
const CLOCK: &str = "boundary clock { ensure ticks = \"Time goes forward.\"; fn now() -> Int; }";

// mitra: evidence for intent::promise_lists_its_claims
#[test]
fn promise_lists_its_claims() {
    let m = load(SAFE, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    assert_eq!(m.ensures.len(), 1);
    assert_eq!(m.ensures[0].name, "s::safe");
    assert_eq!(m.ensures[0].claims, ["s::f::correct", "s::c"]);
}

// mitra: evidence for intent::capture_is_unverified_until_vouched
#[test]
fn capture_is_unverified_until_vouched() {
    let m = "subject s { ensure safe = \"Never negative.\"; fn f(a: Int) -> Int; }";
    let all = named(m, "");
    let names: Vec<&str> = all.iter().map(|v| v.0.as_str()).collect();
    let expected = [
        "s::f::correct",
        "s::f::deterministic",
        "s::safe::captured",
        "s::safe",
    ];
    assert_eq!(names, expected);
    let words: Vec<Logic> = all.iter().map(|v| v.1).collect();
    let unverified = Logic::Unverified;
    assert_eq!(words, [unverified, unverified, unverified, Logic::Assuming]);
}

// mitra: evidence for intent::need_names_its_promise
#[test]
fn need_names_its_promise() {
    let m = format!(
        "{CLOCK} subject s {{ require time = clock::ticks; fn f() -> Int uses clock::now; }}"
    );
    let m = load(&m, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    let names: Vec<&str> = m.ensures.iter().map(|x| x.name.as_str()).collect();
    assert_eq!(names, ["clock::ticks", "s::time"]);
    let needs: Vec<Option<&str>> = m.ensures.iter().map(|x| x.needs.as_deref()).collect();
    assert_eq!(needs, [None, Some("clock::ticks")]);
}

// mitra: evidence for intent::world_promise_is_assumed
#[test]
fn world_promise_is_assumed() {
    assert_eq!(
        named(CLOCK, ""),
        [("clock::ticks".to_string(), Logic::Assuming)]
    );
}

// mitra: evidence for ledger::source_rev_and_trust_are_kept
#[test]
fn source_rev_and_trust_are_kept() {
    let e = "evidence r { for f holds source \"review:me\" rev \"abc\" trust 0.9 }";
    let m = load(F, e).unwrap_or_else(|(_, e)| panic!("{e}"));
    assert_eq!(m.evidence.len(), 1);
    assert_eq!(m.evidence[0].source.as_deref(), Some("review:me"));
    assert_eq!(m.evidence[0].rev.as_deref(), Some("abc"));
    assert_eq!(m.evidence[0].confidence, 0.9);
}

// mitra: evidence for ledger::pinned_keeps_the_output
#[test]
fn pinned_keeps_the_output() {
    let m = "subject fn f(a: Int) -> Int;";
    let m = load(m, "evidence t { for f given a = 1 gives 2 }").unwrap();
    let seen: Vec<_> = m.evidence.iter().map(|x| x.judgment.clone()).collect();
    assert_eq!(
        seen,
        [mitra_lang::Judgment::Output(mitra_lang::Value::Int(2))]
    );
}

// mitra: evidence for ledger::pinned_output_refutes_a_claim
#[test]
fn pinned_output_refutes_a_claim() {
    let e = "evidence t { for f given a = 1 gives -2 }";
    assert_eq!(
        words(C, e),
        [Logic::Fails, Logic::Unverified, Logic::Unverified]
    );
}

// mitra: evidence for ledger::vouched_holds
#[test]
fn vouched_holds() {
    let e = "evidence r { for c holds }";
    assert_eq!(
        words(C, e),
        [Logic::Holds, Logic::Unverified, Logic::Unverified]
    );
}

// mitra: evidence for ledger::passes_stays_unverified
#[test]
fn passes_stays_unverified() {
    assert_eq!(words(C, "evidence t { for c passes }"), UNTOUCHED);
}

// mitra: evidence for ledger::sample_stays_unverified
#[test]
fn sample_stays_unverified() {
    assert_eq!(words(C, "evidence t { for c sampled 100 }"), UNTOUCHED);
}

// mitra: evidence for ledger::failing_sample_fails
#[test]
fn failing_sample_fails() {
    let e = "evidence t { for c sampled 100 fails }";
    assert_eq!(
        words(C, e),
        [Logic::Fails, Logic::Unverified, Logic::Unverified]
    );
}

// mitra: evidence for modules::unknown_module_is_refused
#[test]
fn unknown_module_is_refused() {
    let (_, e) = load("import lib::thing;", "").unwrap_err();
    assert!(e.message.contains("no module `lib`"), "{e}");
}

// mitra: evidence for modules::line_is_placed
#[test]
fn line_is_placed() {
    let m = mitra_lang::Modules::alone("subject fn f() -> Bool;\nsubject fn g() -> Bool;");
    assert_eq!(m.place(2), "model:2");
}

/// `a.mitra` names a type of `b.mitra`, and is read after it.
#[test]
fn a_type_is_read_before_its_use() {
    let file = |name: &str, text: &str| mitra_lang::Module {
        name: name.to_string(),
        text: text.to_string(),
    };
    let files = vec![
        file("a.mitra", "subject a { fn f() -> b::R; }"),
        file("b.mitra", "subject b { record R { x: Int } }"),
    ];
    let m = mitra_lang::Modules::new(files, mitra_lang::Handed::default());
    assert_eq!(m.place(1), "b.mitra:1");
    assert!(mitra_lang::load_modules(&m, "").is_ok());
}

/// mitra-lang reads nothing but what it is handed: no source file of the
/// crate names the file system, the environment or the network.
#[test]
fn the_crate_reads_no_disk() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut todo = vec![src];
    let mut seen = 0;
    while let Some(dir) = todo.pop() {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                todo.push(path);
                continue;
            }
            let text = fs::read_to_string(&path).unwrap();
            for word in [
                "std::fs",
                "std::path",
                "std::env",
                "std::net",
                "std::io",
                "std::process",
            ] {
                assert!(!text.contains(word), "{}: {word}", path.display());
            }
            seen += 1;
        }
    }
    assert!(seen > 10);
}

/// never_runs: the same model and evidence give the same report, on each
/// pair below, run twice.
// mitra: evidence for verdicts::same_report sampled 8
#[test]
fn same_report() {
    let pairs = [
        (F, ""),
        (F, "evidence a { for f holds }"),
        (F, "evidence a { for f holds } evidence b { for f fails }"),
        (C, ""),
        (C, "evidence t { for f given a = 1 gives -2 }"),
        (C, "evidence t { for c sampled 100 fails }"),
        (SAFE, ""),
        (CLOCK, ""),
    ];
    for (model, ledger) in pairs {
        let text = |_: usize| run(model, ledger).unwrap().to_text();
        assert_eq!(text(0), text(1), "{model}");
    }
}

// mitra: evidence for modules::one_text_is_read_as_it_is
#[test]
fn one_text_is_read_as_it_is() {
    let m = mitra_lang::Modules::alone("subject fn f() -> Bool;");
    assert_eq!(m.text(), "subject fn f() -> Bool;");
}

// mitra: evidence for modules::imports_are_listed
#[test]
fn imports_are_listed() {
    let text = "import a::b::x;\nimport std::rust::{Option};\nimport c::{y, z};";
    assert_eq!(mitra_lang::parse::imports(text), ["a::b", "c"]);
}

// mitra: evidence for modules::an_import_is_listed_once
#[test]
fn an_import_is_listed_once() {
    assert_eq!(
        mitra_lang::parse::imports("import a::x;\nimport a::y;"),
        ["a"]
    );
}

// mitra: evidence for modules::fault_is_located
#[test]
fn fault_is_located() {
    let m = mitra_lang::Modules::alone("subject fn f() -> Bool;\nsubject fn g() -> Bool;");
    let e = mitra_lang::Error::new(2, "wrong");
    assert_eq!(m.locate(&e), "model: line 2: wrong");
}

// The rules that turn a test result into evidence (results.mitra).

fn said(
    model: &str,
    pinned: &str,
    passed: bool,
    rev: Option<&str>,
) -> mitra_lang::results::Ingested {
    use mitra_lang::results::{from_tests, Outcome, Tied};
    let m = load(model, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    let claim = if model.contains("claim c") { "c" } else { "f" };
    let tied = Tied {
        test: "t".into(),
        claim: claim.into(),
        pinned: pinned.into(),
        place: "lib.rs:1".into(),
    };
    let outcome = Outcome {
        name: "t".into(),
        passed,
    };
    from_tests(&m, &[tied], &[outcome], rev).unwrap()
}

const FA: &str = "subject fn f(a: Int) -> Int;";

// mitra: evidence for results::unpinned_pass_only_passes
#[test]
fn unpinned_pass_only_passes() {
    let said = said(C, "", true, None);
    assert_eq!(
        said.evidence,
        ["evidence test { for c passes source \"test:t\" }"]
    );
    assert_eq!(said.loose.len(), 1);
}

// mitra: evidence for results::pass_on_a_point_holds
#[test]
fn pass_on_a_point_holds() {
    let m = "subject fn f(a: Int) -> Int; claim c() { f(1) >= 0 }";
    let said = said(m, "", true, None);
    assert_eq!(
        said.evidence,
        ["evidence test { for c holds source \"test:t\" }"]
    );
}

// mitra: evidence for results::pinned_pass_is_written_as_it_stands
#[test]
fn pinned_pass_is_written_as_it_stands() {
    let said = said(FA, "given a = 6 gives 3", true, Some("abc"));
    let expected = [
        "evidence test { for f given a = 6 gives 3 source \"test:t\" rev \"abc\" }",
        "evidence test { for f given a = 6 holds source \"test:t\" rev \"abc\" }",
    ];
    assert_eq!(said.evidence, expected);
}

// mitra: evidence for results::sample_keeps_its_count
#[test]
fn sample_keeps_its_count() {
    let said = said(C, "given a in 0..=9 sampled 10", true, None);
    let expected = "evidence test { for c given a in 0..=9 sampled 10 source \"test:t\" }";
    assert_eq!(said.evidence, [expected]);
}

// mitra: evidence for results::pinned_failure_denies_the_pin
#[test]
fn pinned_failure_denies_the_pin() {
    let said = said(FA, "given a = 6 gives 3", false, None);
    let expected = "evidence test { for f given a = 6 fails source \"test:t\" }";
    assert_eq!(said.evidence, [expected]);
}

// mitra: evidence for results::unpinned_failure_fails_the_claim
#[test]
fn unpinned_failure_fails_the_claim() {
    let said = said(C, "", false, None);
    assert_eq!(
        said.evidence,
        ["evidence test { for c fails source \"test:t\" }"]
    );
}

// mitra: evidence for results::failing_sample_says_fails
#[test]
fn failing_sample_says_fails() {
    let said = said(C, "given a in 0..=9 sampled 10", false, None);
    let expected = "evidence test { for c given a in 0..=9 sampled 10 fails source \"test:t\" }";
    assert_eq!(said.evidence, [expected]);
}
