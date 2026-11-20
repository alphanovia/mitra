//! The trust system, step by step: each folder of `examples/trust/`
//! holds `model.mitra`, `evidence.mitra`, `settings.toml` and the report
//! Mitra must give (`MITRA_RECORD=1 cargo test` writes it again).
//!
//! A test owes two things: that it passes, and that it is correct. A
//! test nobody reviewed is believed at the bias of the settings; a
//! reviewed one as far as its reviewer said; a claim holds when what it
//! stands on reaches what the policy asks.

use std::{fs, path::Path};

fn report(case: &str) -> (String, String) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/trust").join(case);
    let read = |f: &str| fs::read_to_string(dir.join(f)).unwrap();
    let mut handed = mitra_lang::Handed::default();
    handed.settings = mitra_lang::Settings::read(&read("settings.toml")).unwrap();
    let name = "model.mitra".to_string();
    let files = vec![mitra_lang::Module { name, text: read("model.mitra") }];
    let model = mitra_lang::Modules::new(files, handed);
    let report = mitra_lang::load_modules(&model, &read("evidence.mitra"))
        .and_then(|m| mitra_lang::evaluate(&m))
        .unwrap_or_else(|(_, e)| panic!("{case}: {e}"));
    if std::env::var("MITRA_RECORD").is_ok() {
        fs::write(dir.join("expected.txt"), report.to_text()).unwrap();
    }
    (report.to_text(), read("expected.txt"))
}

/// The first lines of a report: the verdict of `name` and what is under it.
fn verdict(text: &str, name: &str) -> String {
    let mut out = Vec::new();
    for l in text.lines().skip_while(|l| !l.starts_with(&format!("{name}: "))) {
        if !out.is_empty() && !l.starts_with("  ") {
            break;
        }
        out.push(l);
    }
    out.join("\n")
}

#[test]
fn every_case_gives_its_expected_report() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/trust");
    let mut seen = 0;
    for entry in fs::read_dir(root).unwrap() {
        let case = entry.unwrap().file_name().to_string_lossy().to_string();
        let (said, expected) = report(&case);
        assert_eq!(said, expected, "{case}");
        seen += 1;
    }
    assert_eq!(seen, 13);
}

#[test]
fn a_passing_test_nobody_reviewed_is_believed_at_the_bias() {
    let v = verdict(&report("1-passing").0, "clock::wraps");
    assert!(v.starts_with("clock::wraps: unverified\n  confidence      0.5"), "{v}");
    assert!(v.contains("not reviewed: bias 0.5"), "{v}");
}

#[test]
fn a_test_reviewed_at_1_holds_its_claim() {
    let v = verdict(&report("2-reviewed").0, "clock::wraps");
    assert!(v.starts_with("clock::wraps: holds\n  confidence      1"), "{v}");
}

#[test]
fn a_review_counts_as_far_as_its_author_said() {
    let v = verdict(&report("3-reviewed-lower").0, "clock::wraps");
    assert!(v.starts_with("clock::wraps: unverified\n  confidence      0.8"), "{v}");
}

#[test]
fn a_failing_test_fails_the_claim_whatever_the_policy() {
    let v = verdict(&report("4-failing").0, "clock::wraps");
    assert!(v.starts_with("clock::wraps: fails\n  confidence      0.5"), "{v}");
}

#[test]
fn the_policy_is_the_threshold() {
    let v = verdict(&report("5-policy-lower").0, "clock::wraps");
    assert!(v.starts_with("clock::wraps: holds\n  confidence      0.5"), "{v}");
}

#[test]
fn a_derived_claim_is_believed_as_the_product_of_what_it_stands_on() {
    let text = report("6-derived").0;
    let wraps = verdict(&text, "clock::wraps");
    assert!(wraps.starts_with("clock::wraps: holds\n  confidence      1"), "{wraps}");
    // Two tests, one of them not reviewed.
    let twice = verdict(&text, "clock::twice");
    assert!(twice.starts_with("clock::twice: unverified\n  confidence      0.5"), "{twice}");
    // The promise follows from both claims.
    let promise = verdict(&text, "clock::midnight");
    assert!(promise.starts_with("clock::midnight: unverified\n  confidence      0.5"), "{promise}");
}

#[test]
fn without_a_bias_nothing_changes() {
    let text = report("7-no-bias").0;
    assert!(text.starts_with("clock::next::correct: unverified"), "{text}");
    assert!(verdict(&text, "clock::wraps").starts_with("clock::wraps: holds"), "{text}");
    assert!(!text.contains("confidence"), "{text}");
}

#[test]
fn a_failing_test_reviewed_as_correct_is_a_failure_believed_at_1() {
    let v = verdict(&report("9-failing-reviewed-correct").0, "clock::wraps");
    assert!(v.starts_with("clock::wraps: fails\n  confidence      1"), "{v}");
}

#[test]
fn a_passing_test_reviewed_as_wrong_counts_for_nothing() {
    let v = verdict(&report("10-passing-reviewed-wrong").0, "clock::wraps");
    assert!(v.starts_with("clock::wraps: unverified\n  confidence      0,"), "{v}");
    assert!(v.contains("who says it is wrong"), "{v}");
}

/// The review replaces the bias, upward or downward: 0.3, 0.5, 0.8 are
/// under a policy of 0.9, and 1 is over it.
#[test]
fn the_review_replaces_the_bias() {
    for (case, said, word) in [
        ("12-reviewed-under-the-bias", "0.3", "unverified"),
        ("11-reviewed-at-half", "0.5", "unverified"),
        ("3-reviewed-lower", "0.8", "unverified"),
        ("2-reviewed", "1", "holds"),
    ] {
        let v = verdict(&report(case).0, "clock::wraps");
        assert!(v.starts_with(&format!("clock::wraps: {word}\n  confidence      {said}")), "{v}");
    }
}

/// KNOWN WRONG, kept as Mitra says it today: a review that says the
/// test is wrong is not weighed against one that says it is correct.
/// The better review wins and the claim holds at 1.
#[test]
fn two_reviews_that_disagree_are_not_weighed_yet() {
    let v = verdict(&report("8-reviews-disagree").0, "clock::wraps");
    assert!(v.starts_with("clock::wraps: holds\n  confidence      1"), "{v}");
}

/// KNOWN WRONG, kept as Mitra says it today: two claims that cannot
/// both be true are read as one function giving two outputs. The only
/// suspect is the code; neither test, neither claim, and the review at
/// 1 against the bias at 0.5 ranks nothing.
#[test]
fn two_claims_that_disagree_are_not_ranked_yet() {
    let text = report("13-two-claims-disagree").0;
    assert!(text.contains("clock::next::deterministic: fails"), "{text}");
    assert!(text.contains("certain   the code of `clock::next`"), "{text}");
    assert!(verdict(&text, "clock::wraps").starts_with("clock::wraps: holds"), "{text}");
    assert!(verdict(&text, "clock::stops").starts_with("clock::stops: unverified"), "{text}");
}
