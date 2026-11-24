//! Blame (`src/blame.rs`): which evidence to doubt, given the model
//! explains the system. On main's div, a failed test is blamed first,
//! whole; every diagnosis meets every conflict and is smallest. Every
//! code of suit `B` is given.

use mitra_core::blame::{blame, Settings, Suspect};
use mitra_core::catalogue::{Suit, CATALOGUE};
use mitra_core::check::check;
use mitra_core::evidence::{Ledger, Measurement, Review};
use mitra_core::explain::{self, explain, Explanation, Source, MODEL};
use mitra_core::judgement::{Opinion, Prop, Taking};
use mitra_core::model::{Model, Path};
use mitra_core::parse::parse;
use mitra_core::simulation::Value;
use std::collections::BTreeSet;

fn div() -> Model {
    let path = format!("{}/examples/div/tied.mitra", env!("CARGO_MANIFEST_DIR"));
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
fn divided(a: i64, b: i64, q: i64) -> Measurement {
    Measurement {
        inputs: vec![
            (path("calc::div::a"), int(a)),
            (path("calc::div::b"), int(b)),
        ],
        outputs: vec![(path("calc::div::q"), int(q))],
        seen: true,
        count: 1,
        by: format!("test:div_{a}_{b}"),
    }
}
fn explained(model: &Model, ledger: &Ledger) -> Explanation {
    explain(model, ledger, &explain::Settings::default())
}

/// Each diagnosis meets every conflict, and no smaller set does.
fn sound(conflicts: &[BTreeSet<Suspect>], diagnoses: &[BTreeSet<Suspect>]) {
    for d in diagnoses {
        assert!(
            conflicts.iter().all(|c| !c.is_disjoint(d)),
            "{d:?} misses one"
        );
        for s in d {
            let mut less = d.clone();
            less.remove(s);
            assert!(
                conflicts.iter().any(|c| c.is_disjoint(&less)),
                "{d:?} is not smallest"
            );
        }
    }
}

/// A failed test, with one that passed: the failed one, whole, is the
/// first diagnosis; then the tie and the rule, which only the model
/// says.
#[test]
fn a_failed_test_is_blamed_first() {
    let model = div();
    let ledger = Ledger {
        measurements: vec![divided(7, 2, 4), divided(6, 2, 3)],
        reviews: vec![],
    };
    let e = explained(&model, &ledger);
    let b = blame(&model, &e, &Settings::default());
    assert!(b.complete);
    assert_eq!(b.findings, vec![]);
    let suspects: Vec<BTreeSet<Suspect>> = b.diagnoses.iter().map(|d| d.suspects.clone()).collect();
    sound(&b.conflicts, &suspects);
    assert_eq!(suspects[0], BTreeSet::from([Suspect::Measurement(0)]));
    // Dropping it drops all it says, and nothing of the other.
    assert!(b.diagnoses[0].drop.iter().all(|t| t.by == "test:div_7_2"));
    assert!(b.diagnoses[0].drop.len() >= 2);
    // Next: what only the model says, the tie and the rule `everywhere`.
    let tie = Suspect::Taking(Taking {
        prop: Prop::Named("calc::div at src/lib.rs/div".to_string()),
        value: true,
        count: 0,
        by: MODEL.to_string(),
    });
    let everywhere = Suspect::Taking(Taking {
        prop: Prop::Named("calc::div::everywhere".to_string()),
        value: true,
        count: 0,
        by: MODEL.to_string(),
    });
    assert_eq!(suspects[1], BTreeSet::from([tie, everywhere]));
    // The claims hold: no diagnosis drops one.
    assert!(b
        .diagnoses
        .iter()
        .flat_map(|d| &d.drop)
        .all(|t| t.prop != Prop::Named("calc::div::spec".to_string())));
}

/// Nothing conflicts: nothing to blame.
#[test]
fn where_nothing_conflicts_nothing_is_blamed() {
    let model = div();
    let ledger = Ledger {
        measurements: vec![divided(7, 2, 3), divided(6, 2, 3)],
        reviews: vec![],
    };
    let b = blame(&model, &explained(&model, &ledger), &Settings::default());
    assert_eq!(
        (b.conflicts.len(), b.diagnoses.len(), b.complete),
        (0, 0, true)
    );
}

/// A review against a claim is blamed: the claim holds.
#[test]
fn a_review_against_a_claim() {
    let model = div();
    let ledger = Ledger {
        measurements: vec![],
        reviews: vec![Review {
            from: None,
            seeing: vec![],
            prop: path("calc::div::spec"),
            holds: false,
            by: "human:ana".to_string(),
        }],
    };
    let b = blame(&model, &explained(&model, &ledger), &Settings::default());
    let first: Vec<_> = b.diagnoses[0].suspects.iter().collect();
    assert_eq!(first, vec![&Suspect::Review(0)]);
}

#[test]
fn each_finding_is_given() {
    let model = div();
    let mut given = BTreeSet::new();
    // More than the budget: the first only, and said so.
    let ledger = Ledger {
        measurements: vec![divided(7, 2, 4), divided(9, 3, 4)],
        reviews: vec![],
    };
    let tight = Settings {
        diagnoses: 1,
        ..Settings::default()
    };
    let b = blame(&model, &explained(&model, &ledger), &tight);
    assert_eq!(b.diagnoses.len(), 1);
    assert!(!b.complete);
    let codes: Vec<String> = b.findings.iter().map(|f| f.code.to_string()).collect();
    assert_eq!(codes, vec!["B002"]);
    given.insert("B002".to_string());
    // A conflict of the model's claims alone, which today's explain
    // never makes: nothing to blame, and said so.
    let spec = Prop::Named("calc::div::spec".to_string());
    let taking = |value| Taking {
        prop: spec.clone(),
        value,
        count: 0,
        by: MODEL.to_string(),
    };
    let mut e = Explanation::default();
    e.opinions.insert(
        spec.clone(),
        Opinion::taken(spec.clone(), vec![taking(true), taking(false)]).unwrap(),
    );
    e.sources.insert(taking(true), Source::Model);
    e.sources.insert(taking(false), Source::Model);
    let b = blame(&model, &e, &Settings::default());
    let codes: Vec<String> = b.findings.iter().map(|f| f.code.to_string()).collect();
    assert_eq!(codes, vec!["B001"]);
    given.insert("B001".to_string());
    // An all by cases, a point measured twice: a diagnosis that drops the
    // first taking there alone may leave the all standing, and says so.
    let cased = parse(
        r#"
subject calc {
    fn div(a: integer, b: integer) -> q: integer at "src/lib.rs/div" where {
        claim spec: if b == 0 { q == 0 } else { q * b <= a && a < (q + 1) * b };
        claim bounded: b == 0 || q <= a;
        rule r: all x in (a: 0..2, b: 1..2): correct(div, x) |- div::bounded;
    }
}
"#,
    )
    .unwrap();
    assert_eq!(check(&cased), vec![]);
    let mut again = divided(0, 1, 0);
    again.by = "test:again".to_string();
    let ledger = Ledger {
        measurements: vec![divided(0, 1, 0), divided(1, 1, 1), again],
        reviews: vec![Review {
            from: None,
            seeing: vec![],
            prop: path("calc::div::bounded"),
            holds: false,
            by: "human:zoe".to_string(),
        }],
    };
    let b = blame(&cased, &explained(&cased, &ledger), &Settings::default());
    let codes: Vec<String> = b.findings.iter().map(|f| f.code.to_string()).collect();
    assert_eq!(codes, vec!["B003"]);
    given.insert("B003".to_string());
    let listed = CATALOGUE
        .iter()
        .filter(|e| e.code.suit == Suit::Blame && !e.retired);
    let listed: BTreeSet<String> = listed.map(|e| e.code.to_string()).collect();
    assert_eq!(given, listed);
}
