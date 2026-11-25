//! Numbers (`src/numbers.rs`): Subjective Logic on counts, read after
//! judging. One taking, fused takings, a proposition from its rests,
//! and blame ranked by doubt.

use mitra_lang::blame::{blame, Suspect};
use mitra_lang::check::check;
use mitra_lang::evidence::{Ledger, Measurement};
use mitra_lang::explain::{self, explain, MODEL};
use mitra_lang::judgement::{Prop, Taking, Term};
use mitra_lang::model::{Model, Path};
use mitra_lang::numbers::{fused, Numbers, Settings};
use mitra_lang::parse::parse;
use mitra_lang::simulation::Value;
use std::collections::BTreeSet;

fn near(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}
fn taking(value: bool, count: u64, by: &str) -> Taking {
    Taking {
        prop: Prop::Named("p".to_string()),
        value,
        count,
        by: by.to_string(),
    }
}
fn div() -> Model {
    let path = format!(
        "{}/examples/div/tied.mitra",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../..")
    );
    let model = parse(&std::fs::read_to_string(path).unwrap()).unwrap();
    assert_eq!(check(&model), vec![]);
    model
}
fn divided(a: i64, b: i64, q: i64, count: u64) -> Measurement {
    let path = |p: &str| Path::to(&p.split("::").collect::<Vec<_>>());
    Measurement {
        inputs: vec![
            (path("calc::div::a"), Value::Int(a.into())),
            (path("calc::div::b"), Value::Int(b.into())),
        ],
        outputs: vec![(path("calc::div::q"), Value::Int(q.into()))],
        seen: true,
        count,
        by: format!("test:div_{a}_{b}"),
        effects: Default::default(),
    }
}

/// 97 runs for, 3 against, a prior weight of 2.
#[test]
fn takings_fused() {
    let s = Settings::default();
    let b = fused(
        &[taking(true, 97, "test:a"), taking(false, 3, "test:a")],
        &s,
    );
    assert!(near(b.belief, 97.0 / 102.0));
    assert!(near(b.disbelief, 3.0 / 102.0));
    assert!(near(b.uncertainty, 2.0 / 102.0));
    assert!(near(b.expected, 97.0 / 102.0 + 0.5 * 2.0 / 102.0));
    // What the model alone says weighs nothing: the base rate.
    let m = fused(&[taking(true, 0, MODEL)], &s);
    assert_eq!((m.belief, m.uncertainty, m.expected), (0.0, 1.0, 0.5));
    // A voice weighed twice: one count is two runs.
    let heavy = Settings {
        voices: [("human:ana".to_string(), 2.0)].into(),
        ..Settings::default()
    };
    let h = fused(&[taking(true, 1, "human:ana")], &heavy);
    assert!(near(h.belief, 2.0 / 4.0));
}

/// A proposition from its rests: one test at a point is a third of a
/// belief; the all of a hundred points by its cases, the product.
#[test]
fn propositions_from_their_rests() {
    let model = div();
    let mut ledger = Ledger::default();
    for a in 0..10 {
        for b in 0..10 {
            let q = if b == 0 { 0 } else { a / b };
            ledger.measurements.push(divided(a, b, q, 1));
        }
    }
    let e = explain(&model, &ledger, &explain::Settings::default());
    let n = Numbers::of(&e, &Settings::default());
    let at = Prop::Correct {
        function: "calc::div".to_string(),
        at: Term::Point(vec![Value::Int(6.into()), Value::Int(2.into())]),
    };
    let b = n.prop(&at);
    assert!(near(b.belief, 1.0 / 3.0), "{b:?}");
    assert!(near(b.expected, 2.0 / 3.0));
    // The all: a conjunction of a hundred uncertain points, uncertain.
    let all = e
        .verdicts
        .iter()
        .find(|v| matches!(v.prop, Prop::All { .. }))
        .unwrap();
    let b = n.prop(&all.prop);
    assert!(b.belief < 1e-3, "{b:?}");
    assert!(near(b.belief + b.disbelief + b.uncertainty, 1.0));
}

/// A claim a failed test contradicts through its tie, which only the
/// model says: some disbelief all the same (a plain product of beliefs
/// would give none, the tie's belief being 0).
#[test]
fn a_contradiction_through_what_the_model_alone_says() {
    let model = div();
    let ledger = Ledger {
        measurements: vec![divided(7, 2, 4, 1)],
        reviews: vec![],
        supports: vec![],
    };
    let e = explain(&model, &ledger, &explain::Settings::default());
    let n = Numbers::of(&e, &Settings::default());
    let spec = n.prop(&Prop::Named("calc::div::spec".to_string()));
    assert!(spec.disbelief > 0.05, "{spec:?}");
    assert!(near(spec.belief + spec.disbelief + spec.uncertainty, 1.0));
}

/// Blame by doubt: one failed run is less doubtful than the tie and the
/// rule only the model says; a failed test of one run against 97 that
/// passed at the same point is the first suspect.
#[test]
fn blame_ranked_by_doubt() {
    let model = div();
    let ledger = Ledger {
        measurements: vec![divided(7, 2, 4, 1), divided(6, 2, 3, 1)],
        reviews: vec![],
        supports: vec![],
    };
    let e = explain(&model, &ledger, &explain::Settings::default());
    let b = blame(&model, &e, &mitra_lang::blame::Settings::default());
    let n = Numbers::of(&e, &Settings::default());
    let ranked = n.rank(&b);
    assert_eq!(
        ranked[0].0.suspects,
        BTreeSet::from([Suspect::Measurement(0)])
    );
    assert!(near(ranked[0].1, 1.0 / 3.0), "{}", ranked[0].1);
    // Now 97 runs at the failed point say div gives 3: the one that saw
    // 4 is the least likely right, and the first suspect.
    let mut many = divided(7, 2, 3, 97);
    many.by = "test:div_7_2_again".to_string();
    let ledger = Ledger {
        measurements: vec![divided(7, 2, 4, 1), many],
        reviews: vec![],
        supports: vec![],
    };
    let e = explain(&model, &ledger, &explain::Settings::default());
    let b = blame(&model, &e, &mitra_lang::blame::Settings::default());
    let n = Numbers::of(&e, &Settings::default());
    let ranked = n.rank(&b);
    assert_eq!(
        ranked[0].0.suspects,
        BTreeSet::from([Suspect::Measurement(0)])
    );
    assert!(ranked[0].1 > 0.9, "{}", ranked[0].1);
}
