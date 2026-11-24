//! The reader (`docs/reader.md`): ties and tests from Rust code, the
//! results of `cargo test`, a store written by hand; and every piece it
//! cannot use is a finding.

use mitra_core::location::Location;
use mitra_core::model::{Model, Path, Tie};
use mitra_core::parse::parse;
use mitra_core::reader::{
    ledger, measurements, print_store, read_cargo_test, read_rust, read_store, Measurement,
    Outcome, Review, TestRun,
};
use mitra_core::simulation::Value;

fn div() -> Model {
    let path = format!("{}/models/div.mitra", env!("CARGO_MANIFEST_DIR"));
    parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}
fn fall() -> Model {
    let path = format!("{}/models/fall.mitra", env!("CARGO_MANIFEST_DIR"));
    parse(&std::fs::read_to_string(path).unwrap()).unwrap()
}
fn path(p: &str) -> Path {
    Path::to(&p.split("::").collect::<Vec<_>>())
}
fn int(n: i64) -> Value {
    Value::Int(n.into())
}

const LIB: &str = r#"
/// Division, 0 when b is 0.
// mitra: implements calc::div
#[inline]
pub fn div(a: u32, b: u32) -> u32 {
    if b == 0 { 0 } else { a / b }
}

// mitra: implements calc
pub struct Calc;

impl Calc {
    // mitra: implements calc::div
    pub fn div(&self, a: u32, b: u32) -> u32 { div(a, b) }
}

#[cfg(test)]
mod tests {
    use super::*;

    // mitra: measures calc::div (6, 2) -> 3
    #[test]
    fn div_by_two() {
        assert_eq!(div(6, 2), 3);
    }

    // mitra: measures div (7, 0) -> 0
    // mitra: measures div (0, 0) -> 0
    #[test]
    fn div_by_zero() {
        assert_eq!(div(7, 0), 0);
    }

    // mitra: measures calc::div (1, 1) -> 1
    fn helper() {}

    // mitra: evidence for div given a = 6, b = 2 gives 3
    #[test]
    fn old() {}

    // mitra: implements calc::dvi
    fn typo() {}
}
"#;

const REPORT: &str = "
running 4 tests
test tests::div_by_two ... ok
test tests::div_by_zero ... FAILED
test tests::old ... ok
test tests::slow ... ignored, needs a tower

failures:

---- tests::div_by_zero stdout ----
thread 'tests::div_by_zero' panicked at src/lib.rs:27:9:
test result: FAILED. 2 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out
";

#[test]
fn ties_come_from_comments() {
    let reading = read_rust("src/lib.rs", LIB, &div());
    let ties: Vec<String> = reading
        .ties
        .iter()
        .map(|t| format!("{} at {}", t.item, t.at.path.join("/")))
        .collect();
    assert_eq!(
        ties,
        [
            "calc::div at src/lib.rs/div",
            "calc at src/lib.rs/Calc",
            "calc::div at src/lib.rs/Calc/div"
        ]
    );
}

#[test]
fn tests_say_what_they_measure() {
    let reading = read_rust("src/lib.rs", LIB, &div());
    let points: Vec<String> = reading
        .points
        .iter()
        .map(|p| {
            format!(
                "{} {} {:?}->{:?} line {}",
                p.test, p.function, p.takes, p.gives, p.line
            )
        })
        .collect();
    assert_eq!(points.len(), 3, "{points:?}");
    assert_eq!(reading.points[0].test, "tests::div_by_two");
    assert_eq!(reading.points[0].function, path("calc::div"));
    assert_eq!(reading.points[0].takes, vec![int(6), int(2)]);
    assert_eq!(reading.points[1].function, path("div"));
    let findings: Vec<String> = reading
        .findings
        .iter()
        .map(|f| format!("{} line {}", f.code, f.line))
        .collect();
    assert_eq!(findings, ["R003 line 34", "R001 line 37", "R002 line 41"]);
    assert_eq!(
        reading.findings[2].to_string(),
        "finding[R002]: `calc::dvi` is no part, record or function of the model\n --> src/lib.rs:41"
    );
}

#[test]
fn the_report_reads() {
    let runs = read_cargo_test(REPORT);
    let run = |n: &str, outcome| TestRun {
        name: n.to_string(),
        outcome,
    };
    assert_eq!(
        runs,
        vec![
            run("tests::div_by_two", Outcome::Passed),
            run("tests::div_by_zero", Outcome::Failed),
            run("tests::old", Outcome::Passed),
            run("tests::slow", Outcome::Ignored),
        ]
    );
}

#[test]
fn a_test_that_passed_saw_its_row_and_one_that_failed_did_not() {
    let model = div();
    let reading = read_rust("src/lib.rs", LIB, &model);
    let (found, findings) = measurements(
        &model,
        &model.ties,
        &reading.points,
        &read_cargo_test(REPORT),
    );
    let io = |a: i64, b: i64, q: i64, seen: bool, test: &str| Measurement {
        inputs: vec![
            (path("calc::div::a"), int(a)),
            (path("calc::div::b"), int(b)),
        ],
        outputs: vec![(path("calc::div::q"), int(q))],
        seen,
        count: 1,
        by: format!("test:tests::{test}"),
    };
    assert_eq!(
        found,
        vec![
            io(6, 2, 3, true, "div_by_two"),
            io(7, 0, 0, false, "div_by_zero"),
            io(0, 0, 0, false, "div_by_zero")
        ]
    );
    let findings: Vec<String> = findings.iter().map(|f| f.to_string()).collect();
    assert_eq!(findings, ["finding[R004]: the test `tests::old` measures nothing the reader knows\n --> cargo test:0"]);
}

#[test]
fn a_measurement_is_through_a_tie() {
    let mut model = div();
    model.ties.clear();
    let reading = read_rust(
        "tests/div.rs",
        "// mitra: measures calc::div (6, 2) -> 3\n#[test]\nfn t() {}\n",
        &model,
    );
    let runs = read_cargo_test("test t ... ok\n");
    let (found, findings) = measurements(&model, &model.ties, &reading.points, &runs);
    assert_eq!(found, vec![]);
    assert_eq!(findings[0].code.to_string(), "R007");
    // A tie in the code is a tie.
    let tie = Tie {
        item: path("calc::div"),
        at: Location::at(&["src", "lib.rs", "div"]),
    };
    let (found, findings) = measurements(&model, &[tie], &reading.points, &runs);
    assert_eq!((found.len(), findings.len()), (1, 0));
}

const STORE: &str = r#"
// What people saw and said, by hand.
measure calc::div (6, 2) -> 3 by "human:ana";
measure div (9, 3) -> 3 count 4 by "human:ana";
measure calc::div (7, 0) -> 0 failed by "agent:anthropic/opus-5.5/s-1";
review calc::div::spec holds by "human:ana";
review calc::inputs fails by "human:bo";
review calc::div holds by "human:ana";
measure calc::div (6.0, 2) -> 3 by "human:ana";
review calc::nothing holds by "human:ana";
"#;

#[test]
fn the_store_reads_and_comes_back() {
    let model = div();
    let (ledger, findings) = read_store("evidence.mitra", STORE, &model, &model.ties).unwrap();
    assert_eq!(ledger.measurements.len(), 3);
    assert_eq!(
        (ledger.measurements[1].count, ledger.measurements[2].seen),
        (4, false)
    );
    assert_eq!(
        ledger.reviews,
        vec![
            Review {
                prop: path("calc::div::spec"),
                holds: true,
                by: "human:ana".to_string()
            },
            Review {
                prop: path("calc::inputs"),
                holds: false,
                by: "human:bo".to_string()
            },
            Review {
                prop: path("calc::div"),
                holds: true,
                by: "human:ana".to_string()
            },
        ]
    );
    let findings: Vec<String> = findings
        .iter()
        .map(|f| format!("{} line {}", f.code, f.line))
        .collect();
    assert_eq!(findings, ["R006 line 9", "R006 line 10"]);
    let text = print_store(&ledger);
    let (again, none) = read_store("evidence.mitra", &text, &model, &model.ties).unwrap();
    assert_eq!((again, none), (ledger, vec![]));
}

#[test]
fn a_store_that_does_not_read_says_where() {
    let model = div();
    let e = read_store(
        "e.mitra",
        "measure calc::div (6, 2) -> 3;\n",
        &model,
        &model.ties,
    )
    .unwrap_err();
    assert_eq!(
        e.to_string(),
        "error[P003]: expected `by`, found `;`\n --> 1:30"
    );
}

#[test]
fn the_whole_ledger() {
    let model = fall();
    let code = "// mitra: implements stone::speed\npub fn speed(h: f64) -> f64 { 2.0 * 9.81 * h }\n\n// mitra: measures stone::speed (5.0) -> 98.1\n#[test]\nfn at_five() {}\n";
    let store = "measure stone::drop (5.0) -> 9.9 by \"human:timer\";\nreview stone::air holds by \"human:ana\";\n";
    let files = vec![("src/fall.rs".to_string(), code.to_string())];
    let (ties, ledger, findings) = ledger(
        &model,
        &files,
        Some("test at_five ... ok\n"),
        Some(("evidence.mitra", store)),
    )
    .unwrap();
    assert_eq!(ties.len(), 2);
    assert_eq!(ledger.measurements.len(), 2);
    assert_eq!(ledger.reviews.len(), 1);
    assert_eq!(findings, vec![]);
}
