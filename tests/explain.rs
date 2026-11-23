//! Explain (`src/explain.rs`): the model and the evidence, together. On
//! main's div, tests decide where it is correct, its claims at those
//! points, and its all by cases; a failed test contradicts what it went
//! through. Every code of suit `X` is given.

use mitra_core::catalogue::{Suit, CATALOGUE};
use mitra_core::check::check;
use mitra_core::evidence::{Ledger, Measurement, Review};
use mitra_core::explain::{explain, Explanation, Settings, Verdict, CHECK, MODEL};
use mitra_core::judgement::{Prop, Term};
use mitra_core::model::{Model, Path};
use mitra_core::parse::{parse, parse_prop};
use mitra_core::simulation::{Rational, Value};

fn read(name: &str) -> Model {
    let path = format!("{}/models/{name}.mitra", env!("CARGO_MANIFEST_DIR"));
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
fn real(n: i64, d: i64) -> Value {
    Value::Real(Rational::new(n, d).unwrap())
}
fn ports(these: &[(&str, Value)]) -> Vec<(Path, Value)> {
    these.iter().map(|(p, v)| (path(p), v.clone())).collect()
}
/// `div(a, b)` seen to give `q`, or not.
fn divided(a: i64, b: i64, q: i64, seen: bool) -> Measurement {
    Measurement {
        inputs: ports(&[("calc::div::a", int(a)), ("calc::div::b", int(b))]),
        outputs: ports(&[("calc::div::q", int(q))]),
        seen,
        count: 1,
        by: format!("test:div_{a}_{b}"),
    }
}
fn right(a: i64, b: i64) -> i64 {
    match b {
        0 => 0,
        _ => a / b,
    }
}
fn correct(f: &str, at: &[Value]) -> Prop {
    Prop::Correct {
        function: f.to_string(),
        at: Term::Point(at.to_vec()),
    }
}
fn named(n: &str) -> Prop {
    Prop::Named(n.to_string())
}
/// The verdict on `prop`.
fn of(e: &Explanation, prop: &Prop) -> Verdict {
    let found = e.verdicts.iter().find(|v| &v.prop == prop);
    found
        .unwrap_or_else(|| panic!("no verdict on {prop:?}"))
        .verdict
        .clone()
}
fn everywhere(model: &Model) -> Prop {
    parse_prop(
        "all x in (a: 0..10, b: 0..10): correct(calc::div, x)",
        model,
    )
    .unwrap()
}
const TIE: &str = "calc::div at src/lib.rs/div";

/// A hundred tests, one at each point: div is correct everywhere, by
/// its cases, from the tests alone. What the model alone says stays
/// unsupported.
#[test]
fn a_hundred_tests_explain_div_everywhere() {
    let model = read("div");
    let mut ledger = Ledger::default();
    for a in 0..10 {
        for b in 0..10 {
            ledger.measurements.push(divided(a, b, right(a, b), true));
        }
    }
    let e = explain(&model, &ledger, &Settings::default());
    assert_eq!(e.findings, vec![]);
    assert_eq!(of(&e, &everywhere(&model)), Verdict::Explained);
    assert_eq!(
        of(&e, &correct("calc::div", &[int(6), int(2)])),
        Verdict::Explained
    );
    assert_eq!(
        of(&e, &named("calc::div::spec at (6, 2)")),
        Verdict::Explained
    );
    for p in [
        "calc::div::spec",
        TIE,
        "calc::div::everywhere",
        "calc::divides",
    ] {
        assert_eq!(of(&e, &named(p)), Verdict::Unsupported, "{p}");
    }
    // The all rests on the hundred tests, and nothing else.
    let all = &e.opinions[&everywhere(&model)];
    let evidenced: Vec<_> = all
        .rests_for()
        .iter()
        .filter(|rest| rest.iter().all(|t| t.by != MODEL))
        .collect();
    assert_eq!(evidenced.len(), 1);
    assert_eq!(evidenced[0].len(), 100);
}

/// One test fails: it contradicts div at its point, the claim that
/// breaks there, the tie it went through, and the all with its rule.
/// The claim that holds there is not touched.
#[test]
fn a_failed_test_contradicts_what_it_went_through() {
    let model = read("div");
    let ledger = Ledger {
        measurements: vec![divided(7, 2, 4, true), divided(6, 2, 3, true)],
        reviews: vec![],
    };
    let e = explain(&model, &ledger, &Settings::default());
    assert_eq!(e.findings, vec![]);
    let at_7_2 = correct("calc::div", &[int(7), int(2)]);
    let Verdict::Contradicted { against } = of(&e, &at_7_2) else {
        panic!("{:?}", of(&e, &at_7_2))
    };
    let by: Vec<Vec<&str>> = against
        .iter()
        .map(|rest| rest.iter().map(|t| t.by.as_str()).collect())
        .collect();
    assert_eq!(by, vec![vec!["test:div_7_2"]]);
    for p in [
        "calc::div::spec",
        TIE,
        "calc::div::everywhere",
        "calc::div::spec at (7, 2)",
    ] {
        assert!(
            matches!(of(&e, &named(p)), Verdict::Contradicted { .. }),
            "{p}: {:?}",
            of(&e, &named(p))
        );
    }
    assert!(matches!(
        of(&e, &everywhere(&model)),
        Verdict::Contradicted { .. }
    ));
    assert_eq!(
        of(&e, &named("calc::div::bounded at (7, 2)")),
        Verdict::Explained
    );
    assert_eq!(
        of(&e, &correct("calc::div", &[int(6), int(2)])),
        Verdict::Explained
    );
    // The tie is false from the test and the rule explain wrote, which
    // the checker takes.
    let tie = &e.opinions[&named(TIE)];
    assert!(tie.rests_against().iter().any(|rest| {
        let by: Vec<&str> = rest.iter().map(|t| t.by.as_str()).collect();
        by == vec![CHECK, "test:div_7_2"]
    }));
}

/// A failed test reads as one: what the model gives there was not seen.
#[test]
fn a_test_that_failed_where_the_model_gives_its_value() {
    let model = read("div");
    let ledger = Ledger {
        measurements: vec![divided(7, 2, 3, false)],
        reviews: vec![],
    };
    let e = explain(&model, &ledger, &Settings::default());
    assert!(matches!(
        of(&e, &correct("calc::div", &[int(7), int(2)])),
        Verdict::Contradicted { .. }
    ));
    // What the system gave is not known: its claims are not read.
    assert!(!e
        .verdicts
        .iter()
        .any(|v| v.prop == named("calc::div::spec at (7, 2)")));
}

/// A review takes its proposition; a review of a tied item, its tie.
#[test]
fn reviews_are_takings() {
    let model = read("div");
    let review = |p: &str, holds: bool| Review {
        prop: path(p),
        holds,
        by: "human:ana".to_string(),
    };
    let ledger = Ledger {
        measurements: vec![],
        reviews: vec![review("calc::div::spec", true), review("calc::div", false)],
    };
    let e = explain(&model, &ledger, &Settings::default());
    assert_eq!(of(&e, &named("calc::div::spec")), Verdict::Explained);
    assert!(matches!(of(&e, &named(TIE)), Verdict::Contradicted { .. }));
    // The ensure follows from the spec and a require in words, which
    // only the model says.
    assert_eq!(of(&e, &named("calc::divides")), Verdict::Unsupported);
}

/// Each code of suit `X`, given.
const WORLD: &str = r#"
environment sun {
    fn light() -> w: Real at "sky/sun";
}
subject stone {
    use sun::light;
    fn speed(h: Real) -> v2: Real at "src/stone.rs/speed" { 2.0 * 9.81 * h * light() } where {
        claim slow = !(h <= 20.0) || v2 < 400.0;
    }
    fn lit(h: Real) -> l: Real at "src/stone.rs/lit" where {
        claim bright = l <= light();
    }
    fn noise(a: Int) -> n: Int at "src/stone.rs/noise";
    fn square(h: Real) -> s: Real at "src/stone.rs/square" { h * h }
}
"#;

#[test]
fn each_finding_is_given() {
    let model = parse(WORLD).unwrap();
    assert_eq!(check(&model), vec![]);
    let measured = |inputs: &[(&str, Value)], outputs: &[(&str, Value)], seen: bool| Measurement {
        inputs: ports(inputs),
        outputs: ports(outputs),
        seen,
        count: 1,
        by: "test:t".to_string(),
    };
    let five = real(5, 1);
    let cases = [
        (
            vec![
                measured(
                    &[("stone::noise::a", int(1))],
                    &[("stone::noise::n", int(5))],
                    true,
                ),
                measured(
                    &[("stone::noise::a", int(1))],
                    &[("stone::noise::n", int(5))],
                    true,
                ),
            ],
            "X001 at measurements::0",
        ),
        (
            vec![measured(
                &[
                    ("sun::light::w", real(2, 1)),
                    ("stone::speed::h", five.clone()),
                ],
                &[("stone::speed::v2", real(98, 1))],
                false,
            )],
            "X002 at measurements::0",
        ),
        (
            vec![measured(
                &[("stone::lit::h", five.clone())],
                &[("stone::lit::l", real(3, 1))],
                true,
            )],
            "X003 at measurements::0",
        ),
        (
            vec![measured(
                &[("stone::speed::h", five.clone())],
                &[("stone::speed::v2", real(98, 1))],
                true,
            )],
            "X004 at measurements::0",
        ),
        (
            vec![measured(
                &[
                    ("sun::light::w", real(2, 1)),
                    ("stone::square::h", five.clone()),
                ],
                &[("stone::square::s", real(25, 1))],
                true,
            )],
            "X005 at measurements::0",
        ),
        (
            // One run seen alone and in an end-to-end test, by one voice:
            // the same taking, read once.
            vec![
                measured(
                    &[("stone::square::h", five.clone())],
                    &[("stone::square::s", real(25, 1))],
                    true,
                ),
                measured(
                    &[
                        ("stone::square::h", five.clone()),
                        ("stone::noise::a", int(1)),
                    ],
                    &[
                        ("stone::square::s", real(25, 1)),
                        ("stone::noise::n", int(5)),
                    ],
                    true,
                ),
            ],
            "X008 at measurements::1",
        ),
    ];
    let mut given = std::collections::BTreeSet::new();
    for (measurements, expected) in cases {
        let ledger = Ledger {
            measurements,
            reviews: vec![],
        };
        let e = explain(&model, &ledger, &Settings::default());
        let got: Vec<String> = e
            .findings
            .iter()
            .map(|f| format!("{} at {}", f.code, f.at))
            .collect();
        assert!(
            got.contains(&expected.to_string()),
            "{expected} not in {got:?}"
        );
        given.insert(expected[..4].to_string());
        if expected.starts_with("X001") {
            let hints: Vec<String> = e.hints.iter().map(|h| h.to_string()).collect();
            assert_eq!(
                hints,
                vec!["hint[X007]: `stone::noise` gave one value at each point measured: copy them as examples? `examples { (1) -> 5 }`\n --> stone::noise"]
            );
            given.insert("X007".to_string());
        }
    }
    // Judging that cannot finish within its rounds says so.
    let stopped = Settings {
        rounds: 0,
        ..Settings::default()
    };
    let e = explain(&read("div"), &Ledger::default(), &stopped);
    assert_eq!(
        e.findings
            .iter()
            .map(|f| f.code.to_string())
            .collect::<Vec<_>>(),
        vec!["X006"]
    );
    given.insert("X006".to_string());
    let listed = CATALOGUE
        .iter()
        .filter(|e| e.code.suit == Suit::Explain && !e.retired);
    let listed: std::collections::BTreeSet<String> = listed.map(|e| e.code.to_string()).collect();
    assert_eq!(given, listed);
}

/// What the model alone says is taken by `model`, with no weight, and a
/// rule its proof shows by the checker.
#[test]
fn what_the_model_alone_says() {
    let model = read("fall");
    let e = explain(&model, &Ledger::default(), &Settings::default());
    assert_eq!(e.findings, vec![]);
    let lands = &e.opinions[&named("stone::drop::lands")];
    let by: Vec<(&str, u64)> = lands
        .rests_for()
        .iter()
        .flatten()
        .map(|t| (t.by.as_str(), t.count))
        .collect();
    assert_eq!(by, vec![(CHECK, 1)]);
    let air = &e.opinions[&named("stone::air")];
    let by: Vec<(&str, u64)> = air
        .rests_for()
        .iter()
        .flatten()
        .map(|t| (t.by.as_str(), t.count))
        .collect();
    assert_eq!(by, vec![(MODEL, 0)]);
    // A proved rule is a check, not evidence of the system: what it
    // gives rests on the model's own leaves too.
    assert!(e
        .verdicts
        .iter()
        .all(|v| v.verdict != Verdict::Explained || v.prop == named("stone::drop::lands")));
}
