//! Evidence with an act and a voice: what an act can say of a claim, on
//! whose word a verdict rests, and what a policy asks.

use mitra_lang::ast::apart;
use mitra_lang::voices::{missing, summary, support};
use mitra_lang::{evaluate, load, Logic};

const M: &str = "subject shipping {\n\
    ensure never_free = \"A parcel is never shipped for nothing.\";\n\
    fn fee(zone: 0..=2, weight: 1..=3) -> Int;\n\
    claim positive(zone, weight) { fee(zone, weight) > 0 }\n\
    claim one() { fee(0, 3) == 12 }\n}";

const TEST: &str = "evidence e1 { for shipping::fee given zone = 0, weight = 3 gives 12 act test \
    by \"machine:cargo\" written \"agent:anthropic/opus-5.5/s-8f2a\" source \"test:t_fee\" }\n";

fn refused(evidence: &str, why: &str) {
    let e = load(M, evidence).unwrap_err().1.to_string();
    assert!(e.contains(why), "{e}");
}

// mitra: evidence for voices::voices_apart
#[test]
fn voices_apart() {
    let a = "agent:anthropic/opus-5.5/s-1";
    assert_eq!(apart(a, a), 0);
    assert_eq!(apart(a, "agent:anthropic/opus-5.5/s-2"), 1);
    assert_eq!(apart(a, "agent:anthropic/fable-5.2/s-1"), 2);
    assert_eq!(apart(a, "agent:other/opus-5.5/s-1"), 3);
    assert_eq!(apart(a, "human:ana"), 4);
    assert_eq!(apart("human:ana", "human:bo"), 4);
    assert_eq!(apart(a, "unknown"), 5);
}

// mitra: evidence for ledger::an_act_is_checked_against_its_claim
#[test]
fn an_act_is_checked_against_its_claim() {
    // A test is a point.
    load(M, TEST).unwrap();
    refused(
        "evidence e { for shipping::positive holds act test by \"machine:cargo\" }",
        "a test is a point",
    );
    // A whole claim takes a review or a proof.
    load(
        M,
        "evidence e { for shipping::positive holds act review by \"human:ana\" }",
    )
    .unwrap();
    load(
        M,
        "evidence e { for shipping::positive holds act proof by \"machine:lean\" }",
    )
    .unwrap();
    // A capture is nobody's test.
    refused(
        "evidence e { for shipping::never_free::captured holds act exhaustive }",
        "it takes a review, a proof or a tie",
    );
    // A report is of a failure.
    refused(
        "evidence e { for shipping::positive holds act report by \"human:ana\" }",
        "a report is of a failure",
    );
    load(
        M,
        "evidence e { for shipping::positive fails act report by \"human:ana\" }",
    )
    .unwrap();
    // A sample says its count; a set run through is exhaustive.
    load(
        M,
        "evidence e { for shipping::positive act sample 100 by \"machine:proptest\" }",
    )
    .unwrap();
    load(
        M,
        "evidence e { for shipping::positive given zone in 0..=2, weight in 1..=3 holds act exhaustive }",
    )
    .unwrap();
    refused(
        "evidence e { for shipping::positive holds act guess }",
        "no act `guess`",
    );
    refused(
        "evidence e { for shipping::positive holds by \"claude\" }",
        "is no voice",
    );
}

// mitra: evidence for voices::machine_is_not_the_author
#[test]
fn machine_is_not_the_author() {
    let unsigned = "evidence e { for shipping::fee given zone = 0, weight = 3 gives 12 act test \
                    by \"machine:cargo\" source \"test:t\" }";
    let m = load(M, unsigned).unwrap();
    let s = support(&m, "shipping::one");
    assert_eq!(s.words[0].voice, "unknown");
    assert_eq!(s.doubt, 0.5);
    let m = load(M, TEST).unwrap();
    let s = support(&m, "shipping::one");
    assert_eq!(s.words[0].voice, "agent:anthropic/opus-5.5/s-8f2a");
    assert_eq!(s.doubt, 0.3);
}

// mitra: evidence for voices::review_of_a_test_is_a_second_voice
#[test]
fn review_of_a_test_is_a_second_voice() {
    let reviewed = format!(
        "{TEST}evidence r {{ for shipping::fee holds act review \
         by \"agent:anthropic/fable-5.2/s-9\" of \"test:t_fee\" cold }}"
    );
    let m = load(M, &reviewed).unwrap();
    // It judges nothing: the function is no more covered than by the test.
    let report = evaluate(&m).unwrap();
    assert_eq!(report.verdicts[0].logic, Logic::Unverified);
    assert_eq!(m.vouches.len(), 1);
    let s = support(&m, "shipping::one");
    assert_eq!(s.words.len(), 2);
    assert_eq!(s.apart, Some("model"));
    assert!((s.doubt - 0.3 * 0.15).abs() < 1e-9, "{}", s.doubt);
    let text = report.to_text();
    let want =
        "shipping::one: holds\n  by              review (agent:anthropic/fable-5.2/s-9 cold), \
                test (agent:anthropic/opus-5.5/s-8f2a); voices a model apart; doubt 0.05\n";
    assert!(text.contains(want), "{text}");
}

// mitra: evidence for voices::point_does_not_speak_for_every_input
#[test]
fn point_does_not_speak_for_every_input() {
    let m = load(M, TEST).unwrap();
    assert!(support(&m, "shipping::positive").words.is_empty());
    assert_eq!(support(&m, "shipping::fee::correct").words.len(), 1);
}

// mitra: evidence for voices::policy_never_changes_a_verdict
#[test]
fn policy_never_changes_a_verdict() {
    let e = format!(
        "{TEST}evidence r {{ for shipping::positive holds act review by \"agent:anthropic/opus-5.5/s-8f2a\" }}"
    );
    let mut m = load(M, &e).unwrap();
    let before = evaluate(&m).unwrap();
    m.settings
        .policy
        .insert("claim".to_string(), "cold, doubt 0.1".to_string());
    let after = evaluate(&m).unwrap();
    let words = |r: &mitra_lang::Report| r.verdicts.iter().map(|v| v.logic).collect::<Vec<_>>();
    assert_eq!(words(&before), words(&after));
    let s = support(&m, "shipping::positive");
    assert_eq!(s.below, ["cold", "doubt 0.1"]);
    assert!(after
        .to_text()
        .contains("  below policy    lacks cold, doubt 0.1\n"));
    // A person's word meets `cold`: they are nobody's session.
    let asked = "cold, human, review, voices 1, apart kind";
    assert_eq!(missing(asked, &s, 1), ["cold", "human", "apart kind"]);
}

// mitra: evidence for voices::promise_is_doubted_as_its_weakest_part
#[test]
fn promise_is_doubted_as_its_weakest_part() {
    let e = format!(
        "{TEST}evidence r {{ for shipping::positive holds act review by \"agent:anthropic/fable-5.2/s-2\" cold }}\n\
         evidence c {{ for shipping::never_free::captured holds act review by \"human:ana\" }}\n\
         evidence f {{ for shipping::fee holds act review by \"human:ana\" }}"
    );
    let m = load(M, &e).unwrap();
    let report = evaluate(&m).unwrap();
    // positive: a cold agent, 0.15; one: an agent's test, 0.3 times a person, 0.05.
    let s = support(&m, "shipping::never_free");
    assert!((s.doubt - 0.15).abs() < 1e-9, "{}", s.doubt);
    let text = summary(&m, &report);
    assert!(
        text.starts_with("1 of 1 promises hold: 0 with a proof, 1 with a person's review"),
        "{text}"
    );
}

// mitra: evidence for voices::summary_says_what_a_person_would_lift
#[test]
fn summary_says_what_a_person_would_lift() {
    let e = format!(
        "{TEST}evidence r {{ for shipping::positive holds act review by \"agent:anthropic/fable-5.2/s-2\" cold }}\n\
         evidence c {{ for shipping::never_free::captured holds act review by \"agent:anthropic/fable-5.2/s-2\" cold }}\n\
         evidence f {{ for shipping::fee holds act review by \"agent:anthropic/fable-5.2/s-2\" cold }}"
    );
    let m = load(M, &e).unwrap();
    let text = summary(&m, &evaluate(&m).unwrap());
    let want = "1 of 1 promises hold: 0 with a proof, 0 with a person's review, 1 with a cold agent's word, \
                0 on agent work alone, 0 on words nobody signed\n  \
                a person's review would lift the most:\n    shipping::never_free: doubt 0.15 to 0.0075\n";
    assert_eq!(text, want);
}
