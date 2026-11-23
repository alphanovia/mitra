//! Red team of a measured value outside its range (your answer, 18:05):
//! not refused, but evidence against the model there. Explain says X011
//! and takes that the function is not correct at that point, by its
//! ties; blame lists the ties (a drift between model and code) beside
//! the measurement. A test that expected such a value and failed takes
//! nothing. An example row outside a range stays the error M026.

use mitra_core::blame::{blame, Settings as BlameSettings, Suspect};
use mitra_core::check::check as check_model;
use mitra_core::evidence::{check as check_evidence, Ledger, Measurement};
use mitra_core::explain::{explain, Explanation, Settings, Source, Verdict};
use mitra_core::judgement::{Prop, Term};
use mitra_core::model::{Model, Path};
use mitra_core::parse::parse;
use mitra_core::reader::read_store;
use mitra_core::simulation::Value;

const DIV: &str = "subject calc {
    fn div(a: 0..=9, b: 1..=9) -> q: 0..=9 at \"src/lib.rs/div\" { a / b }
}";

fn run(text: &str, store: &str) -> (Model, Explanation, Vec<String>) {
    let model = parse(text).unwrap();
    assert_eq!(check_model(&model), vec![]);
    let ledger = read_store("s", store, &model, &model.ties).unwrap().0;
    assert_eq!(check_evidence(&model, &ledger), vec![]);
    let e = explain(&model, &ledger, &Settings::default());
    let shown = e.findings.iter().map(|f| f.to_string()).collect();
    (model, e, shown)
}

fn correct_at(f: &str, x: &[i64]) -> Prop {
    Prop::Correct {
        function: f.to_string(),
        at: Term::Point(x.iter().map(|n| Value::Int((*n).into())).collect()),
    }
}

fn verdict(e: &Explanation, p: &Prop) -> Option<String> {
    e.verdicts
        .iter()
        .find(|v| &v.prop == p)
        .map(|v| format!("{:?}", v.verdict))
}

fn blamed(model: &Model, e: &Explanation) -> Vec<String> {
    blame(model, e, &BlameSettings::default())
        .diagnoses
        .iter()
        .map(|d| {
            let names: Vec<String> = d
                .suspects
                .iter()
                .map(|s| match s {
                    Suspect::Measurement(i) => format!("measurement {i}"),
                    Suspect::Review(i) => format!("review {i}"),
                    Suspect::Taking(t) => format!("{:?}", t.prop),
                })
                .collect();
            names.join(" + ")
        })
        .collect()
}

#[test]
fn an_input_outside_its_range_counts_against_the_model() {
    let store = "measure calc::div (12, 2) -> 6 by \"test:big\";\nmeasure calc::div (6, 2) -> 3 by \"test:small\";";
    let (model, e, shown) = run(DIV, store);
    assert_eq!(shown.len(), 1, "{shown:?}");
    assert!(
        shown[0].starts_with("finding[X011]: at (12, 2), `a` = 12 lies outside its range 0..=9"),
        "{shown:?}"
    );
    let v = verdict(&e, &correct_at("calc::div", &[12, 2])).unwrap();
    assert!(v.starts_with("Contradicted"), "{v}");
    assert_eq!(
        e.sources
            .iter()
            .filter(|(t, s)| **s == Source::Measurement(0) && !t.value)
            .count(),
        1
    );
    // The point inside its range is explained as before.
    let v = verdict(&e, &correct_at("calc::div", &[6, 2])).unwrap();
    assert!(v.starts_with("Explained"), "{v}");
    // Blame lists the measurement and the tie, the model's side.
    let b = blamed(&model, &e);
    assert!(b.iter().any(|d| d == "measurement 0"), "{b:?}");
    assert!(
        b.iter()
            .any(|d| d.contains("calc::div") && d.contains("src/lib.rs/div")),
        "{b:?}"
    );
}

#[test]
fn an_output_outside_its_range_counts_against_the_model() {
    // The code gave 12 where the model's range stops at 9.
    let (model, e, shown) = run(DIV, "measure calc::div (9, 1) -> 12 by \"test:big\";");
    assert_eq!(
        shown,
        ["finding[X011]: at (9, 1), `q` = 12 lies outside its range 0..=9: the system and the model disagree there, a drift between them or a measurement taken wrong\n --> measurements::0"]
    );
    let v = verdict(&e, &correct_at("calc::div", &[9, 1])).unwrap();
    assert!(v.starts_with("Contradicted"), "{v}");
    let b = blamed(&model, &e);
    assert!(b.iter().any(|d| d.contains("src/lib.rs/div")), "{b:?}");
}

#[test]
fn both_sides_outside_are_said_together() {
    let (_, _, shown) = run(DIV, "measure calc::div (12, 0) -> 20 by \"test:big\";");
    assert_eq!(shown.len(), 1, "{shown:?}");
    assert!(
        shown[0].contains("`a` = 12 lies outside its range 0..=9, and `b` = 0 lies outside its range 1..=9, and `q` = 20 lies outside its range 0..=9"),
        "{shown:?}"
    );
}

#[test]
fn a_failed_test_outside_takes_nothing() {
    let model = parse(DIV).unwrap();
    // A test expected 12 and the system gave something else.
    let mut ledger = read_store(
        "s",
        "measure calc::div (9, 1) -> 12 by \"test:big\";",
        &model,
        &model.ties,
    )
    .unwrap()
    .0;
    ledger.measurements[0].seen = false;
    let e = explain(&model, &ledger, &Settings::default());
    let shown: Vec<String> = e.findings.iter().map(|f| f.to_string()).collect();
    assert_eq!(
        shown,
        ["finding[X011]: the test expected, at (9, 1), `q` = 12 lies outside its range 0..=9: the model says nothing there\n --> measurements::0"]
    );
    assert!(e.sources.iter().all(|(_, s)| *s != Source::Measurement(0)));
}

#[test]
fn an_opaque_value_outside_its_range_counts_against_it() {
    let text = "environment sensor {
    fn read() -> v: 0..=100 at \"src/sensor.rs/read\";
}
subject calc {
    fn half(x: Int) -> y: Int at \"src/lib.rs/half\" { x / 2 }
}";
    let (model, e, shown) = run(text, "measure sensor::read () -> 140 by \"test:hot\";");
    assert_eq!(shown.len(), 1, "{shown:?}");
    assert!(
        shown[0].starts_with("finding[X011]: at (), `v` = 140 lies outside its range 0..=100"),
        "{shown:?}"
    );
    let v = verdict(&e, &correct_at("sensor::read", &[])).unwrap();
    assert!(v.starts_with("Contradicted"), "{v}");
    let b = blamed(&model, &e);
    assert!(b.iter().any(|d| d.contains("src/sensor.rs/read")), "{b:?}");
}

#[test]
fn an_opaque_row_outside_its_range_is_not_read_by_its_caller() {
    // What the sensor gave, as an input to what reads it: outside its
    // range it counts against the sensor, and is not a row for `twice`.
    let text = "environment sensor {
    fn read() -> v: 0..=100 at \"src/sensor.rs/read\";
}
subject calc {
    use sensor::read;
    fn twice() -> y: Int at \"src/lib.rs/twice\" { read() * 2 }
}";
    let model = parse(text).unwrap();
    assert_eq!(check_model(&model), vec![]);
    let at = |p: &str| Path::to(&p.split("::").collect::<Vec<_>>());
    let ledger = Ledger {
        measurements: vec![Measurement {
            inputs: vec![(at("sensor::read::v"), Value::Int(140.into()))],
            outputs: vec![(at("calc::twice::y"), Value::Int(280.into()))],
            seen: true,
            count: 1,
            by: "test:hot".to_string(),
        }],
        reviews: vec![],
    };
    assert_eq!(check_evidence(&model, &ledger), vec![]);
    let e = explain(&model, &ledger, &Settings::default());
    let shown: Vec<String> = e.findings.iter().map(|f| f.to_string()).collect();
    assert!(
        shown
            .iter()
            .any(|s| s.starts_with("finding[X011]: at (), `v` = 140")),
        "{shown:?}"
    );
    let v = verdict(&e, &correct_at("sensor::read", &[])).unwrap();
    assert!(v.starts_with("Contradicted"), "{v}");
}

#[test]
fn an_example_outside_its_range_stays_an_error() {
    let model = parse(
        "subject calc {\n    fn div(a: 0..=9, b: 1..=9) -> q: 0..=9 { a / b } where { examples { (12, 2) -> 6 } }\n}",
    )
    .unwrap();
    let errors: Vec<String> = check_model(&model).iter().map(|e| e.to_string()).collect();
    assert!(
        errors.iter().any(|e| e.starts_with("error[M026]")),
        "{errors:?}"
    );
}

#[test]
fn inside_every_range_nothing_changes() {
    let (_, e, shown) = run(DIV, "measure calc::div (9, 1) -> 9 by \"test:edge\";\nmeasure calc::div (0, 9) -> 0 by \"test:edge0\";");
    assert_eq!(shown, Vec::<String>::new());
    for x in [[9, 1], [0, 9]] {
        let v = verdict(&e, &correct_at("calc::div", &x)).unwrap();
        assert!(v.starts_with("Explained"), "{v}");
    }
    let _ = Verdict::Unsupported;
}
