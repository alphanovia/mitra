//! A measured caller checks its callees' claims (finding 14, the user's
//! go 14:02, read by the model): explain follows the model's run of a
//! measured function through its calls, and reads each callee's claims,
//! and its record's, at the point it was called. The test saw only the
//! caller: each instance is taken by `assumed:path`, the way the code
//! went assumed to be the model's, and said so.

use mitra_lang::blame::{blame, Suspect};
use mitra_lang::check::check;
use mitra_lang::evidence::{self, Ledger, Measurement, Review};
use mitra_lang::explain::{explain, Explanation, Settings, Source, Verdict, PATH};
use mitra_lang::judgement::Prop;
use mitra_lang::model::{Model, Path};
use mitra_lang::parse::parse;
use mitra_lang::simulation::{Rational, Value};

const QUARTER: &str = "subject s {
    fn half(x: rational) -> y: rational at \"src/lib.rs/half\" { x / 2.0 } where {
        claim smaller: y <= x;
    }
    fn quarter(x: rational) -> y: rational at \"src/lib.rs/quarter\" { half(half(x)) }
}";

fn model(text: &str) -> Model {
    let m = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&m), vec![]);
    m
}
fn real(n: i64) -> Value {
    Value::Real(Rational::new(n, 1).unwrap())
}
fn p(s: &str) -> Path {
    Path::to(&s.split("::").collect::<Vec<_>>())
}
fn measured(f: &str, x: i64, y: i64, seen: bool) -> Measurement {
    Measurement {
        inputs: vec![(p(&format!("s::{f}::x")), real(x))],
        outputs: vec![(p(&format!("s::{f}::y")), real(y))],
        seen,
        count: 1,
        by: format!("test:{f}_{x}"),
        effects: Default::default(),
    }
}
fn ledger(ms: Vec<Measurement>) -> Ledger {
    Ledger {
        measurements: ms,
        reviews: vec![],
    }
}
fn of(e: &Explanation, prop: &str) -> Option<Verdict> {
    let prop = Prop::Named(prop.to_string());
    e.verdicts
        .iter()
        .find(|v| v.prop == prop)
        .map(|v| v.verdict.clone())
}
const AT_8: &str = "s::half::smaller at (-8.0) on the way of s::quarter at (-8.0)";
const AT_4: &str = "s::half::smaller at (-4.0) on the way of s::quarter at (-8.0)";

/// The test ran `half` at -8 and -4 through `quarter`: `smaller` is false
/// at both, and contradicted; the way is what blame may doubt.
#[test]
fn a_callee_claim_the_run_breaks_is_contradicted() {
    let m = model(QUARTER);
    let e = explain(
        &m,
        &ledger(vec![measured("quarter", -8, -2, true)]),
        &Settings::default(),
    );
    assert!(
        matches!(
            of(&e, "s::half::smaller"),
            Some(Verdict::Contradicted { .. })
        ),
        "{:?}",
        e.verdicts
    );
    for at in [AT_8, AT_4] {
        assert!(
            matches!(of(&e, at), Some(Verdict::Contradicted { .. })),
            "{at}"
        );
        let taking = e
            .sources
            .iter()
            .find(|(t, _)| t.prop == Prop::Named(at.to_string()));
        let (t, source) = taking.unwrap();
        assert_eq!(
            (t.by.as_str(), t.value, source),
            (PATH, false, &Source::Path(0))
        );
    }
    let b = blame(&m, &e, &Default::default());
    let ways: Vec<&Suspect> = b
        .diagnoses
        .iter()
        .flat_map(|d| &d.suspects)
        .filter(|s| matches!(s, Suspect::Taking(t) if t.by == PATH))
        .collect();
    assert!(!ways.is_empty(), "{:?}", b.diagnoses);
}

/// Where the claims hold, each instance is assumed: on the way only.
#[test]
fn a_callee_claim_the_run_keeps_is_assumed() {
    let m = model(QUARTER);
    let e = explain(
        &m,
        &ledger(vec![measured("quarter", 8, 2, true)]),
        &Settings::default(),
    );
    let at = "s::half::smaller at (8.0) on the way of s::quarter at (8.0)";
    assert_eq!(
        of(&e, at),
        Some(Verdict::Assumed {
            links: vec![],
            requires: vec![],
            paths: vec![Prop::Named(at.to_string())],
        })
    );
    // The claim itself is not lifted by one way.
    assert_eq!(of(&e, "s::half::smaller"), Some(Verdict::Unsupported));
}

/// A point a test saw itself is read from what it saw, not the way.
#[test]
fn a_point_a_test_saw_is_left_to_it() {
    let m = model(QUARTER);
    let e = explain(
        &m,
        &ledger(vec![
            measured("quarter", -8, -2, true),
            measured("half", -8, -4, true),
        ]),
        &Settings::default(),
    );
    assert_eq!(of(&e, AT_8), None);
    assert!(of(&e, AT_4).is_some());
    assert!(matches!(
        of(&e, "s::half::smaller at (-8.0)"),
        Some(Verdict::Contradicted { .. })
    ));
}

/// Where the system gave another value than the model, or the test
/// failed, the model's way is not the code's: nothing is read on it.
#[test]
fn no_way_is_read_where_the_model_and_the_system_part() {
    let m = model(QUARTER);
    for wrong in [
        measured("quarter", -8, -3, true),
        measured("quarter", -8, -2, false),
    ] {
        let e = explain(&m, &ledger(vec![wrong]), &Settings::default());
        assert!(
            e.verdicts
                .iter()
                .all(|v| !format!("{:?}", v.prop).contains("on the way")),
            "{:?}",
            e.verdicts
        );
    }
}

/// The record a callee gives: its claims, on the way too.
#[test]
fn a_callee_record_is_read_on_the_way() {
    let m = model(
        "subject s at \"src/lib.rs\" {
            subject record Span { lo: rational, hi: rational } where { claim ordered: self.lo <= self.hi; }
            fn around(x: rational) -> r: Span { Span { lo: x, hi: x - 1.0 } }
            fn width(x: rational) -> y: rational { around(x).hi - around(x).lo }
        }",
    );
    let ms = vec![Measurement {
        inputs: vec![(p("s::width::x"), real(3))],
        outputs: vec![(p("s::width::y"), real(-1))],
        seen: true,
        count: 1,
        by: "test:w".to_string(),
        effects: Default::default(),
    }];
    let e = explain(&m, &ledger(ms), &Settings::default());
    let at = "s::Span::Span::ordered of what s::around(3.0) gave on the way of s::width at (3.0)";
    assert!(
        matches!(of(&e, at), Some(Verdict::Contradicted { .. })),
        "{:?}",
        e.verdicts
    );
    assert!(matches!(
        of(&e, "s::Span::Span::ordered"),
        Some(Verdict::Contradicted { .. })
    ));
}

/// `assumed:path` is Mitra's own voice: no ledger says it.
#[test]
fn the_path_voice_is_mitras_own() {
    let m = model(QUARTER);
    let l = Ledger {
        measurements: vec![],
        reviews: vec![Review {
            prop: p("s::half::smaller"),
            from: None,
            holds: true,
            by: PATH.to_string(),
            seeing: vec![],
        }],
    };
    let errors = evidence::check(&m, &l);
    assert!(
        errors
            .iter()
            .any(|e| e.message.contains("`assumed:path` is Mitra's own voice")),
        "{errors:?}"
    );
}

/// A function that calls itself: the top call is left to its test, the
/// calls under it are read on the way.
#[test]
fn a_function_calling_itself_is_read_below_its_top() {
    let m = model(
        "subject s {
            subject enum Tree { Leaf, Node(Tree, integer, Tree) }
            fn size(t: Tree) -> n: integer at \"src/lib.rs/size\" {
                match t { Leaf => 0, Node(l, _, r) => size(l) + 1 + size(r) }
            } where {
                claim small: n <= 1;
            }
        }",
    );
    let store = "measure s::size (Node(Node(Leaf, 1, Leaf), 2, Leaf)) -> 2 by \"test:size\";";
    let l = mitra_lang::store::read_store("s", store, &m, &m.ties)
        .unwrap()
        .0;
    let e = explain(&m, &l, &Settings::default());
    let way = " on the way of s::size at (Node(Node(Leaf, 1, Leaf), 2, Leaf))";
    let below = format!("s::size::small at (Node(Leaf, 1, Leaf)){way}");
    assert!(
        matches!(of(&e, &below), Some(Verdict::Assumed { .. })),
        "{below}: {:?}",
        e.verdicts
    );
    let top = format!("s::size::small at (Node(Node(Leaf, 1, Leaf), 2, Leaf)){way}");
    assert_eq!(of(&e, &top), None);
}

/// The way rests on the run: blame may doubt the measurement with it.
#[test]
fn the_way_rests_on_the_run() {
    let m = model(QUARTER);
    let e = explain(
        &m,
        &ledger(vec![measured("quarter", -8, -2, true)]),
        &Settings::default(),
    );
    let b = blame(&m, &e, &Default::default());
    assert!(
        b.conflicts
            .iter()
            .any(|c| c.contains(&Suspect::Measurement(0))
                && c.iter()
                    .any(|s| matches!(s, Suspect::Taking(t) if t.by == PATH))),
        "{:?}",
        b.conflicts
    );
}

/// A way weighs as a link no one reviewed, and blame doubts it first.
#[test]
fn a_way_weighs_as_a_link() {
    let m = model(QUARTER);
    let e = explain(
        &m,
        &ledger(vec![measured("quarter", 8, 2, true)]),
        &Settings::default(),
    );
    let (t, _) = e.sources.iter().find(|(t, _)| t.by == PATH).unwrap();
    let none = mitra_lang::numbers::Settings {
        link: 0.0,
        ..Default::default()
    };
    assert_eq!(mitra_lang::numbers::fused([t], &none).belief, 0.0);
    let half = mitra_lang::numbers::fused([t], &Default::default()).belief;
    assert!((half - 0.2).abs() < 1e-9, "{half}");
    assert_eq!(mitra_lang::blame::kind(&e, &Suspect::Taking(t.clone())), 0);
}
