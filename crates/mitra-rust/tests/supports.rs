//! A test tied to an ensure (finding 1, the user's decision 2026-11-25):
//! `// mitra: supports a::e` above a test. The ensure is said in words,
//! so the test measures nothing the model can evaluate: that it shows
//! the sentence is assumed, and Mitra asks for a review; a review of the
//! ensure seeing the test lifts it. A tied test that fails is loud: a
//! finding, and it counts against the ensure.

use mitra_lang::catalogue::{R003, R004, V009, X012, X015};
use mitra_lang::evidence;
use mitra_lang::evidence::Support;
use mitra_lang::explain::{explain, Explanation, Settings, Verdict, SHOWS};
use mitra_lang::judgement::Prop;
use mitra_lang::model::Model;
use mitra_lang::parse::parse;
use mitra_rust::reader::ledger;

const MODEL: &str = "subject codes at \"src/lib.rs\" {
    ensure documented: \"every code has a page that explains it\";
}";

const CODE: &str = "pub const CODES: &[&str] = &[\"R001\"];

#[cfg(test)]
mod tests {
    // mitra: supports codes::documented
    #[test]
    fn every_code_has_a_page() {}

    #[test]
    fn something_else() {}
}
";

fn model() -> Model {
    parse(MODEL).unwrap()
}

fn run(report: &str, store: Option<&str>) -> (Explanation, Vec<String>) {
    let m = model();
    let files = vec![("src/lib.rs".to_string(), CODE.to_string())];
    let (_, l, findings) = ledger(
        &m,
        &files,
        Some(("report", report)),
        store.map(|s| ("evidence.mitra", s)),
    )
    .unwrap();
    assert_eq!(evidence::check(&m, &l), vec![]);
    let e = explain(&m, &l, &Settings::default());
    (e, findings.iter().map(|f| f.to_string()).collect())
}

fn verdict(e: &Explanation, p: &str) -> Option<Verdict> {
    let p = Prop::Named(p.to_string());
    e.verdicts
        .iter()
        .find(|v| v.prop == p)
        .map(|v| v.verdict.clone())
}

const PASSED: &str =
    "test tests::every_code_has_a_page ... ok\ntest tests::something_else ... ok\n";
const FAILED: &str =
    "test tests::every_code_has_a_page ... FAILED\ntest tests::something_else ... ok\n";

/// Passing, the tie takes the ensure to assumed, and says what a review
/// would lift; the test is used, so no R004 for it.
#[test]
fn a_passing_tied_test_assumes_the_ensure() {
    let (e, findings) = run(PASSED, None);
    let shows =
        Prop::Named("the test `tests::every_code_has_a_page` shows codes::documented".to_string());
    assert_eq!(
        verdict(&e, "codes::documented"),
        Some(Verdict::Assumed {
            links: vec![],
            requires: vec![],
            paths: vec![],
            tests: vec![shows.clone()],
        })
    );
    assert!(e.sources.keys().any(|t| t.prop == shows && t.by == SHOWS));
    // One ask, naming the test and the line to write.
    let asks: Vec<_> = e.asks.iter().filter(|a| a.code == X012).collect();
    assert_eq!(asks.len(), 1, "{asks:?}");
    assert_eq!(
        asks[0].line,
        "review codes::documented holds by \"human:<name>\" seeing \"test:tests::every_code_has_a_page\";"
    );
    assert!(asks[0]
        .message
        .contains("assumes the test `tests::every_code_has_a_page` shows `codes::documented`"));
    let r004: Vec<&String> = findings
        .iter()
        .filter(|f| f.contains(&R004.to_string()))
        .collect();
    assert_eq!(r004.len(), 1, "{findings:?}");
    assert!(r004[0].contains("something_else"));
}

/// A review of the ensure seeing the test lifts the assumption: explained,
/// and nothing more is asked.
#[test]
fn a_review_seeing_the_test_explains_it() {
    let store = "review codes::documented holds by \"human:ana\" seeing \"test:tests::every_code_has_a_page\";";
    let (e, _) = run(PASSED, Some(store));
    assert_eq!(verdict(&e, "codes::documented"), Some(Verdict::Explained));
    assert!(e.asks.is_empty(), "{:?}", e.asks);
    assert!(!e.sources.keys().any(|t| t.by == SHOWS));
}

/// Failing, it is loud: a finding, and it counts against the ensure.
#[test]
fn a_failing_tied_test_is_loud_and_counts_against() {
    let (e, _) = run(FAILED, None);
    assert!(matches!(
        verdict(&e, "codes::documented"),
        Some(Verdict::Contradicted { .. })
    ));
    let x015: Vec<String> = e
        .findings
        .iter()
        .filter(|f| f.code == X015)
        .map(|f| f.to_string())
        .collect();
    assert_eq!(
        x015,
        ["finding[X015]: the test `tests::every_code_has_a_page`, tied to `codes::documented`, failed: it counts against it, assuming the test shows what it is tied to\n --> codes::documented"]
    );
    // Blame may doubt the run or the tie.
    let b = mitra_lang::blame::blame(&model(), &e, &Default::default());
    assert!(!b.diagnoses.is_empty());
}

/// `supports` above what is no test, and a tie to what is no ensure.
#[test]
fn a_tie_out_of_place_is_said() {
    let m = model();
    let code = "// mitra: supports codes::documented\npub fn helper() {}\n";
    let files = vec![("src/lib.rs".to_string(), code.to_string())];
    let (_, _, findings) = ledger(&m, &files, Some(("report", "")), None).unwrap();
    assert!(
        findings
            .iter()
            .any(|f| f.code == R003 && f.message.contains("`supports` is above `fn helper`")),
        "{findings:?}"
    );
    let l = mitra_lang::evidence::Ledger {
        supports: vec![Support {
            prop: mitra_lang::model::Path::to(&["codes", "nothing"]),
            by: "test:t".to_string(),
            passed: true,
        }],
        ..Default::default()
    };
    let errors = evidence::check(&m, &l);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].code, V009);
}

/// A review that does not see the test reviews the sentence, not the
/// tie: the tie is still assumed, and still asked.
#[test]
fn a_review_not_seeing_the_test_leaves_the_tie_assumed() {
    let (e, _) = run(
        PASSED,
        Some("review codes::documented holds by \"human:ana\";"),
    );
    assert!(e.sources.keys().any(|t| t.by == SHOWS));
    assert_eq!(e.asks.len(), 1, "{:?}", e.asks);
}

/// `assumed:test` weighs as a link no one reviewed, blame doubts it
/// first, and no ledger says it; a tie's voice is a test's.
#[test]
fn the_tie_voice_is_mitras_own_and_weighs_as_a_link() {
    let (e, _) = run(PASSED, None);
    let (t, _) = e.sources.iter().find(|(t, _)| t.by == SHOWS).unwrap();
    let none = mitra_lang::numbers::Settings {
        link: 0.0,
        ..Default::default()
    };
    assert_eq!(mitra_lang::numbers::fused([t], &none).belief, 0.0);
    assert_eq!(
        mitra_lang::blame::kind(&e, &mitra_lang::blame::Suspect::Taking(t.clone())),
        0
    );
    let m = model();
    let by = |voice: &str| mitra_lang::evidence::Ledger {
        reviews: vec![mitra_lang::evidence::Review {
            prop: mitra_lang::model::Path::to(&["codes", "documented"]),
            from: None,
            holds: true,
            by: voice.to_string(),
            seeing: vec![],
        }],
        ..Default::default()
    };
    let errors = evidence::check(&m, &by(SHOWS));
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("`assumed:test` is Mitra's own voice")),
        "{errors:?}"
    );
    let l = mitra_lang::evidence::Ledger {
        supports: vec![Support {
            prop: mitra_lang::model::Path::to(&["codes", "documented"]),
            by: "human:ana".to_string(),
            passed: true,
        }],
        ..Default::default()
    };
    let errors = evidence::check(&m, &l);
    assert_eq!(errors.len(), 1, "{errors:?}");
    assert_eq!(errors[0].message, "`human:ana` is no test: `test:name`");
}
