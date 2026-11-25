//! A check does not outweigh the test it came from (finding 18): the
//! rules explain writes from a measurement have what the run gave as a
//! premise, taken by the test, so what they give weighs what the test
//! weighs, and the test is on the line.

use mitra_lang::check::check;
use mitra_lang::evidence::{Ledger, Measurement, Review};
use mitra_lang::explain::{explain, Explanation, Settings, Source, CHECK};
use mitra_lang::judgement::{Prop, Term};
use mitra_lang::model::{Model, Path};
use mitra_lang::numbers::{self, Numbers};
use mitra_lang::parse::parse;
use mitra_lang::simulation::Value;

const TIE: &str = "calc::div at src/lib.rs/div";

fn div() -> Model {
    let path = format!(
        "{}/examples/div/tied.mitra",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../..")
    );
    let model = parse(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(check(&model), vec![]);
    model
}
fn path(p: &str) -> Path {
    Path::to(&p.split("::").collect::<Vec<_>>())
}
fn int(n: i64) -> Value {
    Value::Int(n.into())
}
fn divided(a: i64, b: i64, q: i64, count: u64) -> Measurement {
    Measurement {
        inputs: vec![
            (path("calc::div::a"), int(a)),
            (path("calc::div::b"), int(b)),
        ],
        outputs: vec![(path("calc::div::q"), int(q))],
        seen: true,
        count,
        by: format!("test:div_{a}_{b}"),
        effects: Default::default(),
    }
}
fn tie_reviewed() -> Review {
    Review {
        from: None,
        seeing: vec![],
        prop: path("calc::div"),
        holds: true,
        by: "agent:a".to_string(),
    }
}
fn correct(a: i64, b: i64) -> Prop {
    Prop::Correct {
        function: "calc::div".to_string(),
        at: Term::Point(vec![int(a), int(b)]),
    }
}
fn voices(e: &Explanation, p: &Prop) -> Vec<String> {
    let n = Numbers::of(e, &numbers::Settings::default());
    let (r_for, _) = n.strongest_rests(p);
    let mut by: Vec<String> = r_for.unwrap().iter().map(|t| t.by.clone()).collect();
    by.sort();
    by.dedup();
    by
}

/// With the tie reviewed, the point the test measured rests on the test:
/// no stronger than the test with the review alone.
#[test]
fn the_test_is_in_what_the_check_gives() {
    let m = div();
    let ledger = Ledger {
        measurements: vec![divided(6, 2, 3, 1)],
        reviews: vec![tie_reviewed()],
        supports: vec![],
    };
    let e = explain(&m, &ledger, &Settings::default());
    let p = correct(6, 2);
    assert!(
        voices(&e, &p).contains(&"test:div_6_2".to_string()),
        "{:?}",
        voices(&e, &p)
    );
    let n = Numbers::of(&e, &numbers::Settings::default());
    // One run of a test, alone: 0.67. The rule no longer lifts it to 0.78.
    let b = n.prop(&p);
    assert!(b.expected < 0.70, "{b:?}");
    // The claim's instance too: what the run gave is one of its premises.
    let spec = Prop::Named("calc::div::spec at (6, 2)".to_string());
    let rules = &e.opinions[&spec];
    assert!(rules
        .rests_for()
        .iter()
        .any(|r| r.iter().any(|t| t.by == CHECK)));
    assert!(rules
        .rests_for()
        .iter()
        .filter(|r| r.iter().any(|t| t.by == CHECK))
        .all(|r| r.iter().any(|t| t.by == "test:div_6_2")));
}

/// What the run gave is taken by the test, with its count, as part of
/// the measurement: one suspect with what it saw.
#[test]
fn what_the_run_gave_is_the_tests() {
    let m = div();
    let ledger = Ledger {
        measurements: vec![divided(6, 2, 3, 3)],
        reviews: vec![],
        supports: vec![],
    };
    let e = explain(&m, &ledger, &Settings::default());
    let gave = Prop::Named("measurements::0 gave calc::div at (6, 2)".to_string());
    let taken: Vec<(&str, u64, Source)> = e
        .sources
        .iter()
        .filter(|(t, _)| t.prop == gave)
        .map(|(t, s)| (t.by.as_str(), t.count, *s))
        .collect();
    assert_eq!(taken, vec![("test:div_6_2", 3, Source::Measurement(0))]);
    // It gives no verdict line of its own.
    assert!(!e.verdicts.iter().any(|v| v.prop == gave));
    // The written rule has it as a premise, with the tie.
    let rule = e
        .verdicts
        .iter()
        .find(|v| matches!(&v.prop, Prop::Correct { .. }))
        .unwrap();
    assert_eq!(rule.prop, correct(6, 2));
    let tie = Prop::Named(TIE.to_string());
    let o = &e.opinions[&correct(6, 2)];
    assert!(o
        .rests_for()
        .iter()
        .any(|r| r.iter().any(|t| t.prop == gave) && r.iter().any(|t| t.prop == tie)));
}

/// A function with no value in the model, decided by its claims: what
/// the run gave is taken all the same, a premise of each claim's rule.
#[test]
fn claims_alone_take_what_the_run_gave() {
    let m = parse(
        "subject s {
            fn f(x: integer) -> y: integer at \"src/lib.rs/f\" where {
                claim up: y > x;
            }
        }",
    )
    .unwrap();
    let ledger = Ledger {
        measurements: vec![Measurement {
            inputs: vec![(path("s::f::x"), int(1))],
            outputs: vec![(path("s::f::y"), int(2))],
            seen: true,
            count: 1,
            by: "test:t".to_string(),
            effects: Default::default(),
        }],
        reviews: vec![],
        supports: vec![],
    };
    let e = explain(&m, &ledger, &Settings::default());
    let gave = Prop::Named("measurements::0 gave s::f at (1)".to_string());
    assert!(e.sources.keys().any(|t| t.prop == gave && t.by == "test:t"));
    let instance = Prop::Named("s::f::up at (1)".to_string());
    let o = &e.opinions[&instance];
    assert!(o
        .rests_for()
        .iter()
        .any(|r| r.iter().any(|t| t.by == CHECK)));
    assert!(o
        .rests_for()
        .iter()
        .filter(|r| r.iter().any(|t| t.by == CHECK))
        .all(|r| r.iter().any(|t| t.prop == gave)));
}

/// A claim the run could not decide leaves the function's correctness
/// unsaid: what the run gave is taken all the same, for the claims it
/// decided.
#[test]
fn a_claim_undecided_still_takes_what_the_run_gave() {
    let m = parse(
        "subject s {
            fn g(x: integer) -> z: integer;
            fn f(x: integer) -> y: integer at \"src/lib.rs/f\" where {
                claim up: y > x;
                claim odd: g(x) == y;
            }
        }",
    )
    .unwrap();
    let ledger = Ledger {
        measurements: vec![Measurement {
            inputs: vec![(path("s::f::x"), int(1))],
            outputs: vec![(path("s::f::y"), int(2))],
            seen: true,
            count: 1,
            by: "test:t".to_string(),
            effects: Default::default(),
        }],
        reviews: vec![],
        supports: vec![],
    };
    let e = explain(&m, &ledger, &Settings::default());
    assert!(!e.findings.is_empty());
    assert!(!e
        .verdicts
        .iter()
        .any(|v| matches!(&v.prop, Prop::Correct { .. })));
    let gave = Prop::Named("measurements::0 gave s::f at (1)".to_string());
    assert!(e.sources.keys().any(|t| t.prop == gave && t.by == "test:t"));
}
