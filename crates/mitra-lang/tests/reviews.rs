//! Reviews of any piece (the user's design, 2026-11-24 17:22): a review
//! names a claim, rule, require, ensure or tied item, or one of its
//! links, `review X from …`; `seeing "test:name"` makes it count only
//! while that test passes. `mitra explain` asks for each link, and each
//! ensure said only in words, no voice reviewed (X012).

use mitra_lang::evidence::check as check_ledger;
use mitra_lang::explain::{asks, explain, Settings, Source};
use mitra_lang::judgement::Prop;
use mitra_lang::model::{Model, Path};
use mitra_lang::parse::parse;
use mitra_lang::store::{
    print_store, read_store, read_store_seeing, Ledger, Outcome, Review, TestRun,
};
use std::collections::BTreeSet;

const MODEL: &str = r#"subject sun {
    ensure shines: "the sun shines";
}
subject calc {
    require light: sun::shines;
    fn half(n: 0..=9) -> h: 0..=9 { n / 2 } where {
        claim small: h <= n;
    }
    ensure halves: "half halves" from (half::small);
    ensure twice: "two ways" from (half::small) | (light);
}"#;

fn model() -> Model {
    parse(MODEL).unwrap()
}
fn path(p: &str) -> Path {
    Path::to(&p.split("::").collect::<Vec<_>>())
}
fn store(text: &str) -> (Ledger, Vec<String>) {
    let m = model();
    let (ledger, findings) = read_store("s", text, &m, &m.ties).unwrap();
    (ledger, findings.iter().map(|f| f.to_string()).collect())
}
fn asked(ledger: &Ledger) -> Vec<String> {
    asks(&model(), ledger, &BTreeSet::new())
        .iter()
        .map(|a| a.line.clone())
        .collect()
}

/// With no review, each link of an ensure and the ensure said only in
/// words is asked for, with the line to write; a piece of one link by
/// `from` alone. A require from an ensure is that ensure: not asked.
#[test]
fn explain_asks_for_each_link_no_voice_reviewed() {
    assert_eq!(
        asked(&Ledger::default()),
        [
            r#"review calc::halves from holds by "human:<name>";"#,
            r#"review calc::twice from (calc::half::small) holds by "human:<name>";"#,
            r#"review calc::twice from (calc::light) holds by "human:<name>";"#,
            r#"review sun::shines holds by "human:<name>" seeing "test:<name>";"#,
        ]
    );
    let m = model();
    let e = explain(&m, &Ledger::default(), &Settings::default());
    let first = e.asks[0].to_string();
    assert_eq!(
        first,
        "ask[X012]: Mitra takes that `calc::half::small` gives `calc::halves` as written, with no review: what rests on it is assumed; review the link\n --> calc::halves\n  = help: once reviewed, write in the store `review calc::halves from holds by \"human:<name>\";`"
    );
    assert_eq!(
        e.asks[1].message,
        "Mitra takes that `calc::half::small` gives `calc::twice` as written, with no review: what rests on it is assumed; review the link"
    );
}

/// The store's lines answer them: holds or fails, someone looked. A
/// review of the piece is not one of its link.
#[test]
fn a_review_of_a_link_answers_its_ask() {
    let (ledger, findings) = store(
        r#"review calc::halves from holds by "human:ana";
review calc::light from fails by "human:ana";
review calc::twice from (calc::light) holds by "human:ana";
review calc::twice holds by "human:ana";
review sun::shines holds by "human:ana" seeing "test:sunny";"#,
    );
    assert_eq!(findings, Vec::<String>::new());
    assert_eq!(
        ledger.reviews[0].from,
        Some(vec![path("calc::half::small")])
    );
    assert_eq!(ledger.reviews[1].from, Some(vec![path("sun::shines")]));
    assert_eq!(ledger.reviews[3].from, None);
    assert_eq!(ledger.reviews[4].seeing, ["test:sunny"]);
    assert_eq!(
        asked(&ledger),
        [r#"review calc::twice from (calc::half::small) holds by "human:<name>";"#]
    );
    assert_eq!(check_ledger(&model(), &ledger), vec![]);
}

/// A refused review answers nothing: the ask stays.
#[test]
fn a_refused_review_answers_no_ask() {
    let (ledger, _) = store(r#"review calc::halves from holds by "test:sunny";"#);
    let m = model();
    let refused: BTreeSet<Path> = check_ledger(&m, &ledger)
        .into_iter()
        .map(|e| e.at)
        .collect();
    assert_eq!(refused.len(), 1);
    let lines: Vec<String> = asks(&m, &ledger, &refused)
        .iter()
        .map(|a| a.line.clone())
        .collect();
    assert!(
        lines
            .iter()
            .any(|l| l.starts_with("review calc::halves from")),
        "{lines:?}"
    );
    let e = explain(&m, &ledger, &Settings::default());
    assert!(e
        .asks
        .iter()
        .any(|a| a.line.starts_with("review calc::halves from")));
}

/// A reviewed link is taken by its voice, as the link judgement reads:
/// the ensure from its list.
#[test]
fn a_reviewed_link_is_taken_by_its_voice() {
    let (ledger, _) = store(r#"review calc::halves from holds by "human:ana";"#);
    let m = model();
    let e = explain(&m, &ledger, &Settings::default());
    let link = Prop::Implies {
        pre: vec![Prop::Named("calc::half::small".to_string())],
        subject: "calc::halves".to_string(),
        post: vec![Prop::Named("calc::halves".to_string())],
    };
    let taken: Vec<&str> = e
        .sources
        .iter()
        .filter(|(_, s)| matches!(s, Source::Review(0)))
        .map(|(t, _)| t.by.as_str())
        .collect();
    assert_eq!(taken, ["human:ana"]);
    assert!(e
        .sources
        .iter()
        .any(|(t, s)| matches!(s, Source::Review(0)) && t.prop == link));
}

/// What names no link of the piece is R006, and says what does.
#[test]
fn a_link_the_piece_does_not_have_is_a_finding() {
    let cases = [
        (
            r#"review calc::twice from holds by "human:ana";"#,
            "`calc::twice` is from (calc::half::small) | (calc::light): name the one reviewed, `from (calc::half::small)`",
        ),
        (
            r#"review calc::half::small from holds by "human:ana";"#,
            "`calc::half::small` is from nothing: no link to review",
        ),
        (
            r#"review calc::halves from (calc::light) holds by "human:ana";"#,
            "`calc::halves` is not from (calc::light): it is from (calc::half::small)",
        ),
        (
            r#"review sun::shines holds by "human:ana" seeing "sunny";"#,
            "`sunny` is no test: a review sees `test:name`",
        ),
    ];
    for (line, message) in cases {
        let (ledger, findings) = store(line);
        assert_eq!(ledger.reviews, vec![], "{line}");
        assert_eq!(findings.len(), 1, "{line}");
        assert!(findings[0].contains(message), "{line}: {findings:?}");
    }
}

/// `seeing` counts while the test passes: one that failed, was ignored,
/// or did not run drops the review, R008.
#[test]
fn a_review_seeing_a_test_counts_while_it_passes() {
    let m = model();
    let line = r#"review sun::shines holds by "human:ana" seeing "test:sunny";"#;
    let run = |outcome| TestRun {
        name: "tests::sunny".to_string(),
        outcome,
        binary: String::new(),
    };
    let read = |runs: &[TestRun]| read_store_seeing("s", line, &m, &m.ties, Some(runs)).unwrap();
    let (ledger, findings) = read(&[run(Outcome::Passed)]);
    assert_eq!((ledger.reviews.len(), findings), (1, vec![]));
    for (runs, why) in [
        (vec![run(Outcome::Failed)], "the test `tests::sunny` failed"),
        (
            vec![run(Outcome::Ignored)],
            "the test `tests::sunny` was ignored",
        ),
        (vec![], "the test `sunny` is not in the report"),
    ] {
        let (ledger, findings) = read(&runs);
        assert_eq!(ledger.reviews, vec![]);
        assert_eq!(
            findings[0].to_string().lines().next().unwrap(),
            format!("finding[R008]: {why}: the review of `sun::shines` by `human:ana` counts only while it passes")
        );
    }
    // Dropped, it is asked for again.
    let (ledger, _) = read(&[]);
    assert!(asked(&ledger)
        .iter()
        .any(|l| l.starts_with("review sun::shines")));
}

/// The store prints as it reads.
#[test]
fn reviews_of_links_print_as_written() {
    let text = r#"review calc::twice from (calc::light) holds by "human:ana";
review sun::shines fails by "human:bo" seeing "test:sunny";
"#;
    let (ledger, _) = store(text);
    assert_eq!(print_store(&ledger), text);
}

/// A ledger made by hand: a link the piece does not have is V005; one
/// voice may review a piece and its link apart, but each once.
#[test]
fn the_evidence_checks_links() {
    let review = |prop: &str, from: Option<&str>| Review {
        prop: path(prop),
        from: from.map(|f| vec![path(f)]),
        holds: true,
        by: "human:ana".to_string(),
        seeing: vec![],
    };
    let ledger = Ledger {
        measurements: vec![],
        reviews: vec![
            review("calc::halves", Some("calc::light")),
            review("calc::halves", None),
            review("calc::halves", Some("calc::half::small")),
            review("calc::halves", Some("calc::half::small")),
        ],
    };
    let errors: Vec<String> = check_ledger(&model(), &ledger)
        .iter()
        .map(|e| e.to_string())
        .collect();
    assert_eq!(
        errors,
        [
            "error[V005]: `calc::halves` is not from (calc::light): a link reviewed is an ensure's `from` list, or a require's ensure\n --> reviews::0",
            "error[V007]: `human:ana` reviews `calc::halves from (calc::half::small)` twice\n --> reviews::3",
        ]
    );
}

/// A review may see several tests: it counts while every one passes.
#[test]
fn a_review_seeing_tests_counts_while_all_pass() {
    let m = model();
    let line = r#"review sun::shines holds by "human:ana" seeing "test:sunny", "test:bright";"#;
    let run = |name: &str, outcome| TestRun {
        name: name.to_string(),
        outcome,
        binary: String::new(),
    };
    let both = [
        run("sunny", Outcome::Passed),
        run("bright", Outcome::Passed),
    ];
    let (ledger, findings) = read_store_seeing("s", line, &m, &m.ties, Some(&both)).unwrap();
    assert_eq!((ledger.reviews.len(), findings), (1, vec![]));
    assert_eq!(print_store(&ledger), format!("{line}\n"));
    let one = [
        run("sunny", Outcome::Passed),
        run("bright", Outcome::Failed),
    ];
    let (ledger, findings) = read_store_seeing("s", line, &m, &m.ties, Some(&one)).unwrap();
    assert_eq!(ledger.reviews, vec![]);
    assert!(findings[0]
        .to_string()
        .starts_with("finding[R008]: the test `bright` failed"));
    // One name in two binaries: the review names which.
    let twice = |a, b| {
        [
            TestRun {
                name: "sunny".to_string(),
                outcome: a,
                binary: "tests/a.rs".to_string(),
            },
            TestRun {
                name: "sunny".to_string(),
                outcome: b,
                binary: "tests/b.rs".to_string(),
            },
        ]
    };
    let line = r#"review sun::shines holds by "human:ana" seeing "test:sunny";"#;
    let runs = twice(Outcome::Passed, Outcome::Failed);
    let (_, findings) = read_store_seeing("s", line, &m, &m.ties, Some(&runs)).unwrap();
    assert!(
        findings[0].to_string().contains(
            "the test `sunny` is any of `tests/a.rs::sunny`, `tests/b.rs::sunny`: name its binary"
        ),
        "{findings:?}"
    );
    let line = r#"review sun::shines holds by "human:ana" seeing "test:tests/a.rs::sunny";"#;
    let (ledger, findings) = read_store_seeing("s", line, &m, &m.ties, Some(&runs)).unwrap();
    assert_eq!((ledger.reviews.len(), findings), (1, vec![]));
}

/// A test measures; it never reviews (the user's rule, 2026-11-24): a
/// review by a test is refused, V007, and says to write who reviewed.
#[test]
fn a_test_is_no_reviewer() {
    let (ledger, _) = store(r#"review sun::shines holds by "test:sunny";"#);
    let errors: Vec<String> = check_ledger(&model(), &ledger)
        .iter()
        .map(|e| e.to_string())
        .collect();
    assert_eq!(
        errors,
        ["error[V007]: `test:sunny` is a test, and a test is no reviewer: it measures; write who reviewed, `seeing \"test:sunny\"`\n --> reviews::0"]
    );
}

/// Any test counts (finding 15): a unit test is named by its crate's
/// binary, a doc test as the report names it, with or without its line.
#[test]
fn a_review_sees_unit_and_doc_tests() {
    let m = model();
    let run = |binary: &str, name: &str, outcome| TestRun {
        name: name.to_string(),
        outcome,
        binary: binary.to_string(),
    };
    let runs = [
        run(
            "sun_core/src/lib.rs",
            "light::tests::sunny",
            Outcome::Passed,
        ),
        run(
            "sun_cli/src/main.rs",
            "light::tests::sunny",
            Outcome::Failed,
        ),
        run(
            "sun_core/doc",
            "src/lib.rs - light::shine (line 12)",
            Outcome::Passed,
        ),
    ];
    for seeing in [
        "test:sun-core/src/lib.rs::light::tests::sunny",
        "test:src/lib.rs - light::shine",
        "test:src/lib.rs - light::shine (line 12)",
    ] {
        let line = format!(r#"review sun::shines holds by "human:ana" seeing "{seeing}";"#);
        let (ledger, findings) = read_store_seeing("s", &line, &m, &m.ties, Some(&runs)).unwrap();
        assert_eq!((ledger.reviews.len(), findings), (1, vec![]), "{seeing}");
    }
    let line =
        r#"review sun::shines holds by "human:ana" seeing "test:sun-cli/src/main.rs::sunny";"#;
    let (ledger, findings) = read_store_seeing("s", line, &m, &m.ties, Some(&runs)).unwrap();
    assert_eq!(ledger.reviews, vec![]);
    assert!(findings[0].to_string().contains("failed"), "{findings:?}");
}
