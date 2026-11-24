//! The reader (`docs/design/reader.md`): ties and tests from Rust code, the
//! results of `cargo test`, a store written by hand; and every piece it
//! cannot use is a finding.

use mitra_lang::location::Location;
use mitra_lang::model::{Model, Path, Tie};
use mitra_lang::parse::parse;
use mitra_lang::simulation::Value;
use mitra_rust::reader::{
    ledger, measurements, print_store, read_cargo_test, read_rust, read_store, Measurement,
    Outcome, Review, TestRun,
};

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
        binary: String::new(),
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
                from: None,
                seeing: vec![],
                prop: path("calc::div::spec"),
                holds: true,
                by: "human:ana".to_string()
            },
            Review {
                from: None,
                seeing: vec![],
                prop: path("calc::inputs"),
                holds: false,
                by: "human:bo".to_string()
            },
            Review {
                from: None,
                seeing: vec![],
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
        Some(("report", "test at_five ... ok\n")),
        Some(("evidence.mitra", store)),
    )
    .unwrap();
    assert_eq!(ties.len(), 2);
    assert_eq!(ledger.measurements.len(), 2);
    assert_eq!(ledger.reviews.len(), 1);
    assert_eq!(findings, vec![]);
}

/// A test binary that began and gave no result stopped: R009, by its
/// `Running` line. One that gave its result, failed or not, did not.
#[test]
fn a_test_binary_that_stopped_is_a_finding() {
    let report = "     Running tests/a.rs (target/debug/deps/a-1)

running 1 test
test one ... FAILED

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out
     Running tests/b.rs (target/debug/deps/b-2)

running 2 tests
test two ... ok

thread 'three' has overflowed its stack
error: test failed, to rerun pass `--test b`
   Doc-tests div

running 0 tests
";
    let found: Vec<String> = mitra_rust::reader::stopped("cargo test", report)
        .iter()
        .map(|f| f.to_string())
        .collect();
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(
        found[0]
            .starts_with("finding[R009]: the test binary `tests/b.rs` stopped before its result"),
        "{found:?}"
    );
    assert!(found[0].contains("cargo test:7"), "{found:?}");
    assert!(found[1].contains("`div/doc` stopped"), "{found:?}");
}

/// The rows `mitra::check_eq!` prints, each a measurement by the test
/// it is under, of the model's function through the tie on the Rust one.
#[test]
fn rows_are_measurements_through_ties() {
    let model = parse(
        "subject calc {\n    fn div(a: integer, b: integer) -> q: integer;\n    fn half(a: integer) -> h: integer;\n}\nsubject geo {\n    fn half(a: integer) -> h: integer;\n}",
    )
    .unwrap();
    let ties = vec![
        Tie {
            item: path("calc::div"),
            at: Location::at(&["src/lib.rs", "div"]),
        },
        Tie {
            item: path("calc::half"),
            at: Location::at(&["src/lib.rs", "calc", "half"]),
        },
        Tie {
            item: path("geo::half"),
            at: Location::at(&["src/lib.rs", "geo", "half"]),
        },
    ];
    let report = "running 2 tests
mitra: row div (1, 1) -> 1
test tests::two ... ok

successes:

---- tests::two stdout ----
mitra: row crate::div (6, 2) -> 3
mitra: row calc::half (6) -> 3
mitra: row half (6) -> 3
mitra: row mul (6, 2) -> 12
mitra: row div (6, true) -> 3
mitra: row div

successes:
    tests::two
mitra: row div (2, 1) -> 2
";
    let (found, findings) = mitra_rust::reader::rows(&model, &ties, "report", report);
    let found: Vec<(String, String)> = found
        .iter()
        .map(|m| (m.outputs[0].0.to_string(), m.by.clone()))
        .collect();
    assert_eq!(
        found,
        [
            ("calc::div::q".to_string(), "test:tests::two".to_string()),
            ("calc::half::h".to_string(), "test:tests::two".to_string()),
        ]
    );
    let findings: Vec<String> = findings.iter().map(|f| f.to_string()).collect();
    let starts = [
        ("finding[R001]: a row under no test's output", "report:2"),
        (
            "finding[R007]: the row's `half` is tied to `calc::half` and `geo::half`",
            "report:10",
        ),
        (
            "finding[R007]: the row's `mul` is no function tied to the model",
            "report:11",
        ),
        (
            "finding[R006]: `(6, true) -> 3` is not of `calc::div`",
            "report:12",
        ),
        ("finding[R001]: `div` is no row", "report:13"),
        ("finding[R001]: a row under no test's output", "report:17"),
    ];
    assert_eq!(findings.len(), starts.len(), "{findings:?}");
    for (f, (start, at)) in findings.iter().zip(starts) {
        assert!(f.starts_with(start) && f.contains(at), "{f}");
    }
}

/// A `#[should_panic]` test is reported `name - should panic`: its name
/// is `name`.
#[test]
fn a_should_panic_test_is_read_by_its_name() {
    let runs = read_cargo_test("test fails_loudly - should panic ... ok\n");
    assert_eq!(runs[0].name, "fails_loudly");
    assert_eq!(runs[0].outcome, Outcome::Passed);
}

// Any kind and place of test counts (finding 15): unit tests in `src/`
// of every crate, `tests/`, examples, benches and doc tests.
const WORKSPACE_REPORT: &str = "
     Running unittests src/lib.rs (target/release/deps/mitra_lang-52858b2b966162b3)
running 1 test
test numbers::tests::opinion ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/release/deps/mitra_store-0a1b2c3d4e5f6789)
running 1 test
test numbers::tests::opinion ... FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/model.rs (target/release/deps/model-dd1fb30afd57462c)
running 1 test
test opinion ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running examples/demo.rs (target/release/examples/demo-1234567890abcdef)
running 1 test
test opinion ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests mitra_lang
running 1 test
test crates/mitra-lang/src/lib.rs - numbers::of (line 12) ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
";

#[test]
fn every_test_binary_is_named_and_read() {
    let runs = read_cargo_test(WORKSPACE_REPORT);
    let named: Vec<(&str, &str)> = runs
        .iter()
        .map(|r| (r.binary.as_str(), r.name.as_str()))
        .collect();
    assert_eq!(
        named,
        [
            ("mitra_lang/src/lib.rs", "numbers::tests::opinion"),
            ("mitra_store/src/lib.rs", "numbers::tests::opinion"),
            ("tests/model.rs", "opinion"),
            ("examples/demo.rs", "opinion"),
            (
                "mitra_lang/doc",
                "crates/mitra-lang/src/lib.rs - numbers::of (line 12)"
            ),
        ]
    );
    assert!(mitra_rust::reader::unread("cargo test", WORKSPACE_REPORT).is_empty());
    assert!(mitra_rust::reader::stopped("cargo test", WORKSPACE_REPORT).is_empty());
}

#[test]
fn a_unit_test_in_src_measures_by_its_crate_and_module() {
    let model = div();
    let test = |file: &str| {
        let source = "#[cfg(test)]\nmod tests {\n    // mitra: measures calc::div (6, 2) -> 3\n    #[test]\n    fn opinion() {}\n}\n";
        read_rust(file, source, &model).points
    };
    let runs = read_cargo_test(WORKSPACE_REPORT);
    // Same module path in two crates: each crate's own run.
    for (file, seen) in [
        ("crates/mitra-lang/src/numbers.rs", true),
        ("crates/mitra-store/src/numbers.rs", false),
    ] {
        let (found, findings) = measurements(&model, &model.ties, &test(file), &runs);
        assert_eq!(found.len(), 1, "{file}: {findings:?}");
        assert_eq!(found[0].seen, seen, "{file}");
        assert_eq!(found[0].by, "test:numbers::tests::opinion");
    }
    // A test in `tests/model.rs` and one in an example, both `opinion`.
    for file in [
        "crates/mitra-lang/tests/model.rs",
        "crates/mitra-lang/examples/demo.rs",
    ] {
        let source = "// mitra: measures calc::div (6, 2) -> 3\n#[test]\nfn opinion() {}\n";
        let points = read_rust(file, source, &model).points;
        let (found, findings) = measurements(&model, &model.ties, &points, &runs);
        assert_eq!(
            (found.len(), found[0].seen),
            (1, true),
            "{file}: {findings:?}"
        );
        assert!(
            findings.iter().all(|f| f.code.to_string() == "R004"),
            "{findings:?}"
        );
    }
}

#[test]
fn a_test_mitra_cannot_read_is_a_finding() {
    let report = "
     Running tests/b.rs (target/debug/deps/b-1a2b)
running 2 tests
test one ... ok
test two ... hello from two
ok
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
";
    let found: Vec<String> = mitra_rust::reader::unread("cargo test", report)
        .iter()
        .map(|f| f.to_string())
        .collect();
    assert_eq!(found.len(), 2, "{found:?}");
    assert!(
        found[0].starts_with(
            "finding[R010]: `test two ... hello from two` is a test line Mitra cannot read"
        ),
        "{found:?}"
    );
    assert!(found[0].contains("cargo test:5"), "{found:?}");
    assert!(
        found[1].starts_with(
            "finding[R010]: the test binary `tests/b.rs` ran 2 tests and Mitra read 1"
        ),
        "{found:?}"
    );
}
