//! Evidence (`src/evidence.rs`): a ledger read against a model. Every
//! code of suit `V` is given, each by one change to a ledger that reads.

use mitra_lang::catalogue::{Suit, CATALOGUE};
use mitra_lang::evidence::{check, Ledger, Measurement, Review};
use mitra_lang::model::{Model, Path};
use mitra_lang::parse::parse;
use mitra_lang::simulation::{Rational, Value};

fn div() -> Model {
    let path = format!(
        "{}/examples/div/tied.mitra",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../..")
    );
    parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}
fn fall() -> Model {
    let path = format!(
        "{}/examples/fall/fall.mitra",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../..")
    );
    parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}
fn path(p: &str) -> Path {
    Path::to(&p.split("::").collect::<Vec<_>>())
}
fn int(n: i64) -> Value {
    Value::Int(n.into())
}
fn real(n: i64, d: i64) -> Value {
    Value::Real(Rational::new(n, d).unwrap())
}
fn ports(these: &[(&str, Value)]) -> Vec<(Path, Value)> {
    these.iter().map(|(p, v)| (path(p), v.clone())).collect()
}
/// `div(a, b)` seen to give `q`, once.
fn divided(a: i64, b: i64, q: i64) -> Measurement {
    Measurement {
        inputs: ports(&[("calc::div::a", int(a)), ("calc::div::b", int(b))]),
        outputs: ports(&[("calc::div::q", int(q))]),
        seen: true,
        count: 1,
        by: "test:div_by_two".to_string(),
        effects: Default::default(),
    }
}
fn review(p: &str, by: &str) -> Review {
    Review {
        from: None,
        seeing: vec![],
        prop: path(p),
        holds: true,
        by: by.to_string(),
    }
}
/// A ledger of div that reads: two tests, a review of the spec, of a
/// require and of the tie.
fn ledger() -> Ledger {
    Ledger {
        measurements: vec![divided(6, 2, 3), divided(7, 0, 0)],
        reviews: vec![
            review("calc::div::spec", "human:ana"),
            review("calc::inputs", "human:ana"),
            review("calc::div", "agent:reviewer"),
        ],
    }
}
/// The errors of div's ledger with one change, each as its code and where.
fn changed(change: impl FnOnce(&mut Ledger)) -> Vec<String> {
    let mut ledger = ledger();
    change(&mut ledger);
    let errors = check(&div(), &ledger).into_iter();
    errors.map(|e| format!("{} at {}", e.code, e.at)).collect()
}

#[test]
fn a_ledger_that_reads() {
    assert_eq!(check(&div(), &ledger()), vec![]);
    assert_eq!(check(&div(), &Ledger::default()), vec![]);
}

type Change = Box<dyn FnOnce(&mut Ledger)>;

/// Each code of the evidence is given, where it is, by one change. The
/// cases give every code the catalogue lists for the evidence.
#[test]
fn each_error_is_found() {
    let cases: Vec<(&str, Change)> = vec![
        (
            "V001 at measurements::0",
            Box::new(|l| {
                l.measurements[0]
                    .inputs
                    .push((path("calc::div::x"), int(1)))
            }),
        ),
        (
            "V003 at measurements::1",
            Box::new(|l| l.measurements[1].outputs[0].1 = real(1, 2)),
        ),
        (
            "V004 at measurements::0",
            Box::new(|l| l.measurements[0].count = 0),
        ),
        (
            "V005 at reviews::0",
            Box::new(|l| l.reviews[0].prop = path("calc::nothing")),
        ),
        (
            "V006 at measurements::0",
            Box::new(|l| _ = l.measurements[0].inputs.pop()),
        ),
        (
            "V007 at reviews::1",
            Box::new(|l| l.reviews[1].prop = path("calc::div::spec")),
        ),
    ];
    let mut given = Vec::new();
    for (expected, change) in cases {
        assert_eq!(changed(change), [expected]);
        given.push(expected[..4].to_string());
    }
    // div has one function, and it is tied: V002 is given on the fall,
    // in `a_measurement_is_through_a_tie`.
    given.push("V002".to_string());
    given.sort();
    let listed = CATALOGUE
        .iter()
        .filter(|e| e.code.suit == Suit::Evidence && !e.retired);
    let listed: Vec<String> = listed.map(|e| e.code.to_string()).collect();
    assert_eq!(given, listed);
}

/// A measurement is always through a tie: of the function, or of its
/// part. The fall ties only the drop.
#[test]
fn a_measurement_is_through_a_tie() {
    let speed = |model: &Model| {
        let m = Measurement {
            inputs: ports(&[("stone::speed::h", real(5, 1))]),
            outputs: ports(&[("stone::speed::v2", real(981, 10))]),
            seen: true,
            count: 3,
            by: "human:ana".to_string(),
            effects: Default::default(),
        };
        let ledger = Ledger {
            measurements: vec![m],
            reviews: vec![],
        };
        let errors = check(model, &ledger).into_iter();
        errors
            .map(|e| format!("{} at {}", e.code, e.at))
            .collect::<Vec<_>>()
    };
    assert_eq!(speed(&fall()), ["V002 at measurements::0"]);
    let mut tied = fall();
    let mut part = tied.ties[0].clone();
    part.item = path("stone");
    tied.ties.push(part);
    assert_eq!(speed(&tied), Vec::<String>::new());
    // End to end: the sun's light as the world gave it, into the speed.
    // An input on what a function with a body gives is refused.
    let drop = Measurement {
        inputs: ports(&[
            ("stone::drop::h", real(5, 1)),
            ("stone::speed::v2", real(1, 1)),
        ]),
        outputs: ports(&[
            ("stone::drop::v", real(99, 10)),
            ("stone::drop::h", real(1, 1)),
        ]),
        seen: false,
        count: 1,
        by: "human:ana".to_string(),
        effects: Default::default(),
    };
    let ledger = Ledger {
        measurements: vec![drop],
        reviews: vec![
            review("stone::drop", "human:ana"),
            review("stone::speed", "human:ana"),
        ],
    };
    let errors: Vec<String> = check(&fall(), &ledger)
        .into_iter()
        .map(|e| format!("{} at {}: {}", e.code, e.at, e.message))
        .collect();
    let expected = [
        "V001 at measurements::0: `stone::drop::h` is both an input and an output: it would be its own expected value",
        "V001 at measurements::0: `stone::drop::h` is what `stone::drop` takes: an output is what a function gives",
        "V001 at measurements::0: `stone::speed::v2` is what `stone::speed` gives, and the model computes it: an output, not an input",
        "V002 at measurements::0: `stone::speed` is tied to nothing: a measurement is always through a tie",
        "V005 at reviews::1: `stone::speed` is no claim, rule, require, ensure or tied item",
    ];
    assert_eq!(errors, expected);
}

/// A measured value outside the range of its type is not refused: it
/// is evidence against the model there (X011, `tests/redteam_drift.rs`).
#[test]
fn a_measured_value_outside_its_range() {
    let model = mitra_lang::parse::parse(
        "subject calc {\n    fn div(a: 0..=9, b: 1..=9) -> q: 0..=9 at \"src/lib.rs/div\" { a / b }\n}",
    )
    .unwrap();
    let ledger = Ledger {
        measurements: vec![divided(12, 2, 6), divided(6, 2, 3)],
        reviews: vec![],
    };
    assert_eq!(check(&model, &ledger), vec![]);
}
