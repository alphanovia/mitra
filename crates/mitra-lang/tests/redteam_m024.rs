//! Red team of change 10a: M024 (two examples at one point that differ),
//! `Calls::with` refusing two rows at one point, and `Calls::given`.
//! Each defect is a failing test; each held property a passing one.

mod common;

use common::apply;
use mitra_lang::agree::{agree, Settings};
use mitra_lang::check::check;
use mitra_lang::expression::{eval, Expression};
use mitra_lang::model::{calls, Model};
use mitra_lang::parse::parse;
use mitra_lang::simulation::{covers, Callee, Calls, Exact, Op, Ranges, Type, Value};
use num_bigint::BigInt;
use std::collections::BTreeMap;
use std::sync::Arc;

type E = Expression<Op>;

fn codes(text: &str) -> Vec<String> {
    let model = parse(text).unwrap();
    check(&model)
        .iter()
        .map(|e| format!("{} at {}", e.code, e.at))
        .collect()
}

fn model(text: &str) -> Model {
    let model = parse(text).unwrap();
    assert_eq!(check(&model), vec![], "{text}");
    model
}

/// Equal values written differently are one value: 1/2 and 0.5 at one
/// point, giving 1/4 and 0.25, are no M024. Field order is not a
/// different point. 0.0 and -0.0 are one point.
#[test]
fn held_equal_values_written_differently_are_one_value() {
    let same = r#"
subject record P { x: rational, y: rational }
subject s {
    fn f(h: rational) -> a: rational where { examples { (0.5) -> 1/4, (1/2) -> 0.25 } }
    fn g(p: P) -> a: rational where {
        examples { (P { x: 1.0, y: 2.0 }) -> 3.0, (P { y: 2.0, x: 1.0 }) -> 3.0 }
    }
    fn k(h: rational) -> a: rational where { examples { (-0.0) -> 1.0, (0.0) -> 1.0 } }
}
"#;
    assert_eq!(codes(same), Vec::<String>::new());
    let differ = same
        .replace("(1/2) -> 0.25", "(1/2) -> 0.26")
        .replace("y: 2.0, x: 1.0 }) -> 3.0", "y: 2.0, x: 1.0 }) -> 4.0")
        .replace("(0.0) -> 1.0", "(0.0) -> 2.0");
    assert_eq!(
        codes(&differ),
        ["M024 at s::f", "M024 at s::g", "M024 at s::k"]
    );
}

/// An example not of the types (Int for Real) is M015 alone, never also
/// M024: `2` and `2.0` are not compared.
#[test]
fn held_int_against_real_is_m015_only() {
    let text = r#"
subject s { fn f(h: rational) -> a: rational where { examples { (2.0) -> 4.0, (2.0) -> 4 } } }
"#;
    assert_eq!(codes(text), ["M015 at s::f"]);
}

/// Check and the library agree: on random examples of a function with no
/// body (and of one with a body), check gives no error exactly when
/// `model::calls` builds the library. So agree and explain never read an
/// empty library (`unwrap_or_default`) of a model check accepts.
#[test]
fn held_check_accepts_exactly_what_the_library_takes() {
    let values = ["0", "1", "2", "1/2", "0.5", "true", "false"];
    let mut seed: u64 = 7;
    let mut next = |n: usize| {
        seed = seed
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((seed >> 33) as usize) % n
    };
    let (mut accepted, mut refused) = (0, 0);
    for round in 0..400 {
        let mut rows = Vec::new();
        for _ in 0..next(5) {
            let a = values[next(values.len())];
            let b = values[next(values.len())];
            let q = values[next(values.len())];
            rows.push(format!("({a}, {b}) -> {q}"));
        }
        let body = if round % 2 == 0 { "" } else { "{ a }" };
        let text = format!(
            "subject s {{ fn f(a: integer, b: rational) -> q: integer {body} where {{ examples {{ {} }} }} }}",
            rows.join(", ")
        );
        let Ok(m) = parse(&text) else { continue };
        let ok = check(&m).is_empty();
        match ok {
            true => accepted += 1,
            false => refused += 1,
        }
        assert_eq!(ok, calls(&m).is_ok(), "{text}: {:?}", check(&m));
    }
    assert!(accepted > 20 && refused > 20, "{accepted} {refused}");
}

/// A value of the world a model example gives, against the function's own
/// example there: still compared (A006), as before the change.
#[test]
fn held_a_model_example_against_the_functions_own_example() {
    let text = r#"
environment sun { fn light() -> w: rational where { examples { () -> 3.0 } } }
subject stone {
    use sun::light;
    fn speed(h: rational) -> v2: rational { 2.0 * h * light() }
}
examples { (sun::light::w = 2.0, stone::speed::h = 5.0) -> stone::speed::v2 = 20.0 }
"#;
    let (agreed, findings) = agree(&model(text), &Settings::default());
    let shown: Vec<String> = findings.iter().map(|f| f.to_string()).collect();
    assert!(
        shown
            .iter()
            .any(|f| f.contains("A006") && f.contains("against example 0")),
        "{shown:?}"
    );
    // The output is computed with the example's 2.0, not the function's 3.0.
    assert!(
        !shown.iter().any(|f| f.contains("`stone::speed::v2`")),
        "{shown:?} {agreed:?}"
    );
}

fn int(n: i64) -> E {
    apply(Op::Int(BigInt::from(n)), vec![])
}
fn n(v: i64) -> Value {
    Value::Int(BigInt::from(v))
}
fn library(f: Callee) -> Calls {
    Calls::with([("f".to_string(), f)].into_iter().collect()).unwrap()
}

/// WAS A DEFECT (fixed: `given` refuses a function with a body). `Calls::given` is public and read by `Exact` before a body,
/// but `Ranges` never reads it. On one library the two readings then
/// disagree, and the covering (S5, `simulation::ranges_sound`: every
/// value Exact gives lies in the bounds Ranges gives) fails. No caller
/// does this today (agree and explain give only to functions with no
/// body, and read only `Exact` with it), but nothing in the type says so.
#[test]
fn held_given_to_a_body_is_refused_so_ranges_cover_exact() {
    let f = Callee {
        promises: None,
        ranges: Default::default(),
        takes: vec![("a".to_string(), Type::Int)],
        gives: Type::Int,
        body: Some(apply(Op::Add, vec![Expression::Name("a".into()), int(1)])),
        rows: vec![],
    };
    // Fixed: a function with a body takes no given value.
    assert!(library(f.clone()).given("f", vec![n(1)], n(7)).is_err());
    let calls = Arc::new(
        library(f)
            .given("f", vec![n(1)], n(7))
            .unwrap_or_else(|same| same),
    );
    let e = apply(Op::Call("f".into()), vec![int(1)]);
    let exact = eval(
        &Exact {
            calls: calls.clone(),
        },
        &BTreeMap::new(),
        &e,
    );
    let bounds = eval(&Ranges { calls }, &BTreeMap::new(), &e);
    assert_eq!(exact, Ok(n(2)));
    assert!(
        covers(&Type::Int, &bounds, &exact),
        "Ranges {bounds:?} does not cover Exact {exact:?}"
    );
}

/// WAS A DEFECT (fixed: `given` checks types). `Calls::given` took a value of any type, which `Calls::with`
/// refused as a row: a call of an `Int` function then gives a truth, so
/// preservation (S2: a value of the type checked) fails. Callers guard it
/// today (V003 refuses the measurement, M022 the model example).
#[test]
fn held_given_is_type_checked() {
    let f = Callee {
        promises: None,
        ranges: Default::default(),
        takes: vec![("a".to_string(), Type::Int)],
        gives: Type::Int,
        body: None,
        rows: vec![],
    };
    // Fixed: a value of another type is refused.
    assert!(library(f.clone())
        .given("f", vec![n(1)], Value::Truth(true))
        .is_err());
    let calls = Arc::new(
        library(f)
            .given("f", vec![n(1)], Value::Truth(true))
            .unwrap_or_else(|same| same),
    );
    let e = apply(Op::Call("f".into()), vec![int(1)]);
    let got = eval(&Exact { calls }, &BTreeMap::new(), &e);
    assert!(
        !matches!(got, Ok(Value::Truth(_))),
        "an integer function gave {got:?}"
    );
}
