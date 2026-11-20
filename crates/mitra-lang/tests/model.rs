//! The points and the sweeps behind the claims of the model in `model/`
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

/// With the setting, a sample drawn from every input counts.
#[test]
fn a_sample_counts_when_set() {
    let mut m = load(C, "evidence t { for c sampled 100 }").unwrap();
    m.settings.counts = true;
    let report = evaluate(&m).unwrap();
    assert_eq!(report.verdicts[0].logic, Logic::Assuming);
    assert_eq!(report.verdicts[0].assuming, ["c::sample"]);
}

// mitra: evidence for ledger::part_sample_stays_unverified
#[test]
fn part_sample_stays_unverified() {
    let e = "evidence t { for c given a in 0..=9 sampled 100 }";
    assert_eq!(words(C, e), UNTOUCHED);
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
    let text = "import a::b::x;\nimport std::core::{Option};\nimport c::{y, z};";
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
) -> mitra_lang::ledger::results::Ingested {
    use mitra_lang::ledger::results::{from_tests, Outcome, Tied};
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

// What a ledger holds that judges nothing (ledger.mitra), and the
// fingerprints that say so (fingerprint.mitra).

// mitra: evidence for ledger::orphan_is_set_aside
#[test]
fn orphan_is_set_aside() {
    let e = "evidence r { for g holds source \"review:me\" }";
    let m = load("subject fn f() -> Bool;", e).unwrap();
    assert!(m.evidence.is_empty());
    let [a] = &m.aside[..] else {
        panic!("{:?}", m.aside)
    };
    assert_eq!(
        (a.about.as_str(), &a.why, a.line),
        ("g", &mitra_lang::Why::Orphaned, 1)
    );
    assert_eq!(a.source.as_deref(), Some("review:me"));
}

// mitra: evidence for ledger::stale_is_set_aside
#[test]
fn stale_is_set_aside() {
    let e = "evidence r { for f holds seen \"000000000000\" }";
    let m = load("subject fn f() -> Bool;", e).unwrap();
    assert!(m.evidence.is_empty());
    let now = mitra_lang::build::fingerprint::all(&m)["f"].clone();
    let why = mitra_lang::Why::Stale {
        seen: "000000000000".into(),
        now: now.clone(),
    };
    assert_eq!(m.aside[0].why, why);
    // Said of the piece as it reads now, it judges.
    let e = format!("evidence r {{ for f holds seen \"{now}\" }}");
    let m = load("subject fn f() -> Bool;", &e).unwrap();
    assert!(m.aside.is_empty() && m.evidence.len() == 1);
}

// mitra: evidence for ledger::seen_as_it_is_judges
#[test]
fn seen_as_it_is_judges() {
    let m = load("subject fn f() -> Bool;", "evidence r { for f holds }").unwrap();
    assert!(m.aside.is_empty() && m.evidence.len() == 1);
}

#[test]
fn evidence_set_aside_judges_nothing() {
    let model = "subject fn f(a: Int) -> Int; claim c(a: Int) { f(a) >= 0 }";
    let stale = "evidence r { for c holds seen \"000000000000\" }";
    let out = mitra_lang::run(model, stale).unwrap().to_text();
    assert!(out.starts_with("c: unverified"), "{out}");
    assert!(out.contains("warning[M108]"), "{out}");
}

const ONE: &str = "subject s { fn f(a: Int) -> Int; claim c(a: Int) { f(a) >= 0 } }";

// mitra: evidence for fingerprint::writing_does_not_count
#[test]
fn writing_does_not_count() {
    let other = "// why\nsubject s {\n  claim c(a: Int) {   f(a) >= 0 }\n  fn f(a: Int) -> Int;\n}";
    let (m, n) = (load(ONE, "").unwrap(), load(other, "").unwrap());
    assert_eq!(
        mitra_lang::build::fingerprint::all(&m),
        mitra_lang::build::fingerprint::all(&n)
    );
}

// mitra: evidence for fingerprint::another_claim_counts
#[test]
fn another_claim_counts() {
    let other = ONE.replace(">=", ">");
    let m = mitra_lang::build::fingerprint::all(&load(ONE, "").unwrap());
    let n = mitra_lang::build::fingerprint::all(&load(&other, "").unwrap());
    assert_ne!(m["s::c"], n["s::c"]);
    assert_eq!(m["s::f"], n["s::f"]);
}

// mitra: evidence for fingerprint::a_new_claim_changes_the_capture
#[test]
fn a_new_claim_changes_the_capture() {
    let one = "subject s { ensure e = \"x\"; fn f(a: Int) -> Int; }";
    let two = "subject s { ensure e = \"x\"; fn f(a: Int) -> Int; claim c(a: Int) { f(a) > 0 } }";
    let m = mitra_lang::build::fingerprint::all(&load(one, "").unwrap());
    let n = mitra_lang::build::fingerprint::all(&load(two, "").unwrap());
    assert_ne!(m["s::e::captured"], n["s::e::captured"]);
    assert_eq!(m["s::e"], n["s::e"]);
    // Another prose is another promise, and another capture.
    let p = mitra_lang::build::fingerprint::all(&load(&one.replace("\"x\"", "\"y\""), "").unwrap());
    assert_ne!(m["s::e"], p["s::e"]);
    assert_ne!(m["s::e::captured"], p["s::e::captured"]);
}

// A model, compiled, and what the levels add (build.mitra).

const PROMISE: &str =
    "subject s { ensure e = \"x\"; fn f(a: Int) -> Int; claim c(a: Int) { f(a) >= 0 } }";

// mitra: evidence for build::implied_pieces_are_listed
#[test]
fn implied_pieces_are_listed() {
    let built = mitra_lang::build::build(&load(PROMISE, "").unwrap());
    let said: Vec<(&str, &str)> = (built.items.iter())
        .map(|i| (i.kind.as_str(), i.name.as_str()))
        .collect();
    let want = [
        ("claim", "s::c"),
        ("ensure", "s::e"),
        ("capture", "s::e::captured"),
        ("function", "s::f"),
        ("implied", "s::f::correct"),
        ("implied", "s::f::deterministic"),
    ];
    assert_eq!(said, want);
    assert!(built.items.iter().all(|i| i.print.len() == 12));
    assert_eq!(built.count("implied"), 2);
    let lock = built.lock();
    let head = format!("mitra build 1\nmodel {}\nclaim s::c ", built.model);
    assert!(lock.starts_with(&head), "{lock}");
    assert!(
        lock.contains(" <- s::f::correct, s::c, s::e::captured\n"),
        "{lock}"
    );
}

// mitra: evidence for build::a_promise_lists_what_it_rests_on
#[test]
fn a_promise_lists_what_it_rests_on() {
    let m = load("subject s { ensure e = \"x\"; fn f(a: Int) -> Int; }", "").unwrap();
    let built = mitra_lang::build::build(&m);
    assert_eq!(built.items[0].on, ["s::f::correct", "s::e::captured"]);
    assert!(built.items[1..].iter().all(|i| i.on.is_empty()));
}

// mitra: evidence for build::writing_does_not_change_the_build
#[test]
fn writing_does_not_change_the_build() {
    let m = load("subject s { fn f(a: Int) -> Int; }", "").unwrap();
    let n = load("// why\nsubject s {\n    fn f(a: Int) -> Int;\n}", "").unwrap();
    assert_eq!(mitra_lang::build::build(&m), mitra_lang::build::build(&n));
}

// mitra: evidence for build::another_type_changes_the_build
#[test]
fn another_type_changes_the_build() {
    let m = load("subject s { fn f(a: Int) -> Int; }", "").unwrap();
    let n = load("subject s { fn f(a: Int) -> Bool; }", "").unwrap();
    assert_ne!(
        mitra_lang::build::build(&m).model,
        mitra_lang::build::build(&n).model
    );
}

// mitra: evidence for build::the_world_is_not_in_the_interface
#[test]
fn the_world_is_not_in_the_interface() {
    let text = "boundary clock { fn now() -> Int; } subject s { fn f(a: Int) -> Int; }";
    let said = mitra_lang::build::interface(&load(text, "").unwrap());
    assert!(
        said.ends_with("\n\n// mitra: implements s::f\nfn f(a: Int) -> Int;\n"),
        "{said}"
    );
    assert!(!said.contains("now"), "{said}");
}

// mitra: evidence for levels::a_capture_is_a_person_s
#[test]
fn a_capture_is_a_person_s() {
    let text = "subject s { ensure e = \"x\"; fn f(a: 0..=1) -> Int; }";
    let (m, r) = (load(text, "").unwrap(), run(text, "").unwrap());
    let left = mitra_lang::build::levels::left(&m, &r);
    assert_eq!(left.person.len(), 2, "{left:?}");
    assert!(
        left.person[1].starts_with("s::e::captured: a reviewer's word"),
        "{left:?}"
    );
    assert_eq!(left.delegate.len(), 1, "{left:?}");
    assert!(left.delegate[0].starts_with("s::f::correct:"), "{left:?}");
}

#[test]
fn an_observation_is_compared() {
    let model = "subject s { fn f(a: Int) -> Int; claim c() { f(1) == 2 } }";
    let before = run(model, "evidence t { for s::f given a = 1 gives 2 }").unwrap();
    let files = mitra_lang::Files {
        model: "m",
        evidence: "e",
    };
    let display = mitra_lang::Display::default();
    let same = mitra_lang::build::levels::diagnosis(&before, &before, &display, &files);
    assert!(!same.contradicts && same.changed.is_empty());
    assert!(same.text.contains("no verdict changes"), "{}", same.text);
    let after = run(model, "evidence t { for s::f given a = 1 gives 3 }").unwrap();
    let said = mitra_lang::build::levels::diagnosis(&before, &after, &display, &files);
    assert!(said.contradicts);
    assert!(
        said.changed.contains(&"s::c: holds -> fails".to_string()),
        "{:?}",
        said.changed
    );
    assert!(said.text.contains("error[M001]"), "{}", said.text);
}

#[test]
fn a_type_is_another_name_for_a_collection() {
    let read = |src: &str| mitra_lang::load(src, "").map(|m| m.functions[0].params.clone());
    let list = read("subject fn f(a: List<Int>) -> Int;").unwrap();
    // Rust's name, from the module of Rust's names.
    assert_eq!(
        read("import std::rust::{Vec};\nsubject fn f(a: Vec<Int>) -> Int;").unwrap(),
        list
    );
    assert_eq!(
        read("type Seq<T> = List<T>;\nsubject fn f(a: Seq<Int>) -> Int;").unwrap(),
        list
    );
    let said = |src: &str| read(src).unwrap_err().1.to_string();
    assert!(said("subject fn f(a: Vec<Int>) -> Int;").contains("import std::rust::{Vec};"));
    assert!(said("subject fn f(a: usize) -> Int;").contains("import std::rust::{usize};"));
    assert!(said("type Seq<T> = Int;").contains("another name for a type of the language"));
    assert!(said("type Seq = Seq;").contains("another name for a type of the language"));
    // A plain type too: Rust's `usize` is `Nat`, zero and up.
    assert_eq!(
        read("import std::rust::{usize};\nsubject fn f(a: usize) -> Int;").unwrap(),
        read("subject fn f(a: Nat) -> Int;").unwrap()
    );
    assert_eq!(
        read("type Count = Nat;\nsubject fn f(a: Count) -> Int;").unwrap(),
        read("subject fn f(a: 0..) -> Int;").unwrap()
    );
    assert!(said("type Seq<T> = List<U>;").contains("other parameters"));
}
