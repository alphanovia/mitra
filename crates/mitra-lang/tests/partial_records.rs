//! Records with `..` (your go, 2026-11-24 19:38): `record { a: T, .. }`
//! says the code holds fields the model does not state. Only a value
//! known in part is written; `==` of two is not known; what an example
//! says is compared on the fields it states.

use mitra_lang::agree::{agree, Agreement, Settings};
use mitra_lang::check::check;
use mitra_lang::parse::parse;
use mitra_lang::print::print;

const HEATER: &str = r#"
subject Heater {
    record { target: integer, on: bool, .. }

    fn decide(self, t: integer) -> h: Heater {
        Heater { on: t < self.target, ..self }
    } where {
        claim keeps_target: h.target == self.target;
        examples { (Heater { target: 20, on: false, .. }, 18) -> Heater { target: 20, on: true, .. } }
    }

    fn same(a: Heater, b: Heater) -> s: bool { a == b } where {
        claim stated: a.target != b.target || a.on != b.on || s;
    }
}
"#;

fn agreements(text: &str) -> Vec<(String, Agreement)> {
    let model = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![]);
    agree(&model, &Settings::default())
        .0
        .into_iter()
        .map(|a| (a.against.clone(), a.agreement))
        .collect()
}

#[test]
fn a_record_with_rest_reads_back() {
    let model = parse(HEATER).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![]);
    let printed = print(&model);
    assert!(
        printed.contains("record { target: integer, on: bool, .. }"),
        "{printed}"
    );
    let again = parse(&printed).unwrap_or_else(|e| panic!("{e}\n{printed}"));
    assert_eq!(model, again);
}

#[test]
fn rest_comes_last() {
    let refused = parse("subject H {\n    record { a: integer, .., b: bool }\n}\n").unwrap_err();
    assert!(
        refused.to_string().contains("`..` comes last in a record"),
        "{refused}"
    );
}

#[test]
fn no_whole_value_of_a_record_with_rest_is_written() {
    let text = "subject G {\n    record { a: integer, .. }\n    fn make(x: integer) -> g: G { G { a: x } }\n}\n";
    let refused = parse(text).unwrap_err().to_string();
    assert!(
        refused.starts_with("error[P004]: `G` has `..`, fields the model does not state, so no whole value of it is written: write one known in part, `G { a: …, .. }`"),
        "{refused}"
    );
    // Known in part, in an example: read.
    let text = "subject G {\n    record { a: integer, .. }\n    fn a(g: G) -> x: integer { g.a } where {\n        examples { (G { a: 1, .. }) -> 1 }\n    }\n}\n";
    let model = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![]);
}

#[test]
fn an_update_keeps_the_rest_and_an_example_is_compared_on_what_it_states() {
    let found = agreements(HEATER);
    let example = found
        .iter()
        .find(|(w, _)| w == "example 0")
        .expect("example 0");
    assert_eq!(example.1, Agreement::Holds, "{found:?}");
    assert!(
        found
            .iter()
            .all(|(_, a)| !matches!(a, Agreement::Fails { .. })),
        "{found:?}"
    );
}

#[test]
fn two_values_alike_in_what_the_model_states_are_not_surely_equal() {
    let found = agreements(HEATER);
    let stated = found
        .iter()
        .find(|(w, _)| w == "claim `stated`")
        .expect("stated");
    match &stated.1 {
        Agreement::Unknown { why } => assert!(
            why.contains("the fields of `Heater` the model does not state (`..`)"),
            "{why}"
        ),
        other => panic!("{other:?}"),
    }
}
