//! A tied record against its struct, a tied enum against its enum (your
//! go, 2026-11-24 19:38): R011, R012, R013, and the reader's review.

use mitra_lang::location::Location;
use mitra_lang::model::Model;
use mitra_lang::model::{Path, Tie};
use mitra_lang::parse::{load, Project};
use mitra_rust::reader::{ledger, read_rust};
use mitra_rust::shape::{shapes, VOICE};

const MODEL: &str = r#"
subject Heater {
    record { target: integer, on: bool }
}
subject Thermostat {
    use Option from "std";
    record { name: [~char], heater: Heater, readings: [integer], last: Option<integer>, .. }
}
subject enum Mode { Off, Heat(integer), Pair(bool, [~char]) }
"#;

/// The model, with std mounted (`Option`).
fn model() -> Model {
    let project = Project {
        files: vec![],
        model: None,
    };
    load(MODEL, &project, &Default::default()).unwrap_or_else(|e| panic!("{}", e.error))
}

fn run(code: &str) -> (Vec<(String, bool)>, Vec<String>) {
    let model = model();
    let reading = read_rust("src/lib.rs", code, &model);
    assert_eq!(reading.findings, vec![], "{:?}", reading.findings);
    let (reviews, findings) = shapes(
        &model,
        &reading.ties,
        &[("src/lib.rs".to_string(), code.to_string())],
    );
    for r in &reviews {
        assert_eq!(r.by, VOICE);
    }
    (
        reviews
            .iter()
            .map(|r| (r.prop.to_string(), r.holds))
            .collect(),
        findings.iter().map(|f| f.to_string()).collect(),
    )
}

const GOOD: &str = r#"
// mitra: implements Heater
#[derive(Clone, Debug)]
pub struct Heater {
    pub target: i64,
    on: bool,
}

/// A thermostat: what the model states, and a cache it does not.
// mitra: implements Thermostat
pub struct Thermostat {
    pub(crate) name: String,
    pub heater: Box<Heater>,
    #[allow(dead_code)]
    pub readings: Vec<u8>,
    pub last: Option<i32>,
    cache: std::collections::HashMap<String, f64>, // not modelled: `..`
}

// mitra: implements Mode
pub enum Mode {
    Off,
    Heat(u16),
    Pair(bool, &'static str),
}
"#;

#[test]
fn shapes_that_agree_are_reviewed_as_holding() {
    let (reviews, findings) = run(GOOD);
    assert_eq!(findings, Vec::<String>::new());
    assert_eq!(
        reviews,
        [
            ("Heater".to_string(), true),
            ("Mode".to_string(), true),
            ("Thermostat".to_string(), true)
        ]
    );
}

#[test]
fn a_field_on_one_side_is_r011_unless_the_record_has_rest() {
    let code = GOOD.replace("    on: bool,\n}", "    on: bool,\n    cache: Vec<u8>,\n}");
    let (reviews, findings) = run(&code);
    assert_eq!(
        findings,
        ["finding[R011]: field `cache` of struct `Heater` is not in the model's `Heater`: model it, or end the record with `..`, the fields the model does not state\n --> src/lib.rs:4"]
    );
    assert!(
        reviews.contains(&("Heater".to_string(), false)),
        "{reviews:?}"
    );
    // A field of the model the struct lacks; a renamed one is both.
    let code = GOOD.replace(
        "    pub last: Option<i32>,\n",
        "    pub latest: Option<i32>,\n",
    );
    let (_, findings) = run(&code);
    assert_eq!(findings.len(), 1, "{findings:?}");
    assert!(
        findings[0].starts_with(
            "finding[R011]: field `last` of the model's `Thermostat` is not in struct `Thermostat`"
        ),
        "{findings:?}"
    );
}

#[test]
fn a_type_that_holds_other_values_is_r012() {
    let code = GOOD.replace("pub target: i64", "pub target: String");
    let (reviews, findings) = run(&code);
    assert_eq!(
        findings,
        ["finding[R012]: field `target` of struct `Heater` is `String`: not every value of it is of `integer`\n --> src/lib.rs:4"]
    );
    assert!(reviews.contains(&("Heater".to_string(), false)));
    let code = GOOD.replace("Heat(u16)", "Heat(u16, u16)");
    let (_, findings) = run(&code);
    assert!(findings[0].starts_with("finding[R012]: variant `Heat` holds 1 values in the model's `Mode`, and 2 in enum `Mode`"), "{findings:?}");
    let code = GOOD.replace("Pair(bool, &'static str)", "Pair(bool, char)");
    let (_, findings) = run(&code);
    assert!(
        findings[0].contains("value 1 of variant `Pair` of enum `Mode` is `char`"),
        "{findings:?}"
    );
}

#[test]
fn a_form_mitra_cannot_write_is_r013_and_no_review() {
    let code = GOOD.replace("pub target: i64", "pub target: f64");
    let (reviews, findings) = run(&code);
    assert_eq!(findings.len(), 1);
    assert!(findings[0].starts_with("finding[R013]: field `target` of struct `Heater` is `f64`: Mitra has no form for a float yet"), "{findings:?}");
    assert!(!reviews.iter().any(|(p, _)| p == "Heater"), "{reviews:?}");
    let code = GOOD.replace(
        "pub struct Heater {\n    pub target: i64,\n    on: bool,\n}",
        "pub struct Heater(i64, bool);",
    );
    let (_, findings) = run(&code);
    assert!(
        findings[0].contains("struct `Heater` is a tuple struct"),
        "{findings:?}"
    );
    let code = GOOD.replace("Heat(u16),", "Heat { level: u16 },");
    let (_, findings) = run(&code);
    assert!(
        findings[0].contains("variant `Heat` of enum `Mode` has named fields"),
        "{findings:?}"
    );
    let code = GOOD.replace("pub struct Heater {", "pub struct Heater<T> {");
    let (_, findings) = run(&code);
    assert!(
        findings[0].contains("struct `Heater` has type parameters"),
        "{findings:?}"
    );
}

#[test]
fn a_struct_of_another_name_is_its_record_through_its_tie() {
    let model = model();
    let code = GOOD
        .replace("pub struct Heater {", "pub struct Boiler {")
        .replace("Box<Heater>", "Box<Boiler>");
    let tie = |item: &str, rust: &str| Tie {
        item: Path::to(&[item]),
        at: Location::at(&["src", "lib.rs", rust]),
    };
    let ties = [tie("Heater", "Boiler"), tie("Thermostat", "Thermostat")];
    let (reviews, findings) = shapes(&model, &ties, &[("src/lib.rs".to_string(), code)]);
    assert_eq!(findings.len(), 0, "{findings:?}");
    assert_eq!(reviews.len(), 2);
    assert!(reviews.iter().all(|r| r.holds));
}

#[test]
fn the_ledger_takes_the_reviews_of_shapes() {
    let model = model();
    let files = [("src/lib.rs".to_string(), GOOD.to_string())];
    let (_, ledger, findings) = ledger(&model, &files, None, None).unwrap();
    assert_eq!(findings, vec![]);
    assert_eq!(ledger.reviews.len(), 3);
}

/// A row prints a struct whole: of a record with `..`, it is read known
/// in what the model states, the struct's other fields its `..`; and
/// it measures the function on what it states.
#[test]
fn a_row_of_a_record_with_rest_is_read_in_part() {
    use mitra_lang::parse::parse;
    let model = parse(
        "subject Heater {\n    record { target: integer, on: bool, .. }\n    fn decide(self, t: integer) -> h: Heater { Heater { on: t < self.target, ..self } }\n}\n",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    let ties = [Tie {
        item: Path::to(&["Heater", "decide"]),
        at: Location::at(&["src", "lib.rs", "Heater", "decide"]),
    }];
    let report = "---- tests::warms stdout ----\nmitra: row Heater::decide (Heater { target: 20, on: false, cache: [1, 2] }, 18) -> Heater { target: 20, on: true, cache: [1, 2] }\n\ntest result: ok. 1 passed; 0 failed\n";
    let (found, findings) = mitra_rust::reader::rows(&model, &ties, "cargo test", report);
    assert_eq!(findings, vec![]);
    assert_eq!(found.len(), 1);
    let shown = mitra_lang::print::value(&found[0].outputs[0].1);
    assert_eq!(shown, "Heater { target: 20, on: true, .. }");
    // Explained by the model: the update keeps what is known.
    let ledger = mitra_lang::evidence::Ledger {
        measurements: found,
        ..Default::default()
    };
    let mut tied = model.clone();
    tied.ties = ties.to_vec();
    let explained = mitra_lang::explain::explain(&tied, &ledger, &Default::default());
    let text = format!("{explained:?}");
    assert!(!text.contains("X010"), "{text}");
    assert!(text.contains("Explained"), "{text}");
}

/// Red team: forms around the struct that must not mislead the reader.
#[test]
fn comments_lifetimes_and_functions_are_read_as_written() {
    // A block comment, a doc comment and a lifetime: still `Heater`.
    let code = GOOD.replace(
        "pub struct Heater {\n    pub target: i64,",
        "pub struct Heater<'a> {\n    /* target } in degrees */\n    /// the target\n    pub target: &'a i64,",
    );
    let (reviews, findings) = run(&code);
    assert_eq!(findings, Vec::<String>::new());
    assert!(reviews.contains(&("Heater".to_string(), true)));
    // A function in a field: no form yet, not a wrong type.
    let code = GOOD.replace("pub target: i64", "pub target: Box<dyn Fn(i64) -> i64>");
    let (_, findings) = run(&code);
    assert!(findings[0].starts_with("finding[R013]"), "{findings:?}");
    // A field named `by`: no field of the model can be so named yet.
    let code = GOOD.replace("    on: bool,\n}", "    on: bool,\n    by: String,\n}");
    let (reviews, findings) = run(&code);
    assert!(
        findings[0].contains("field `by` of struct `Heater` is named by a keyword of Mitra"),
        "{findings:?}"
    );
    assert!(!reviews.iter().any(|(p, _)| p == "Heater"));
}
