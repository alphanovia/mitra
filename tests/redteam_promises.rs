//! Red team of finding 8 (A3): bounds read what a function with no body
//! promises, and `!p || c` reads `c` where `p` keeps its names. Both stay
//! sound given the promises: what agree says holds is true at every point
//! and every value of the world that keeps them.

use mitra_core::agree::{agree, hints, Agreement, Settings};
use mitra_core::check::check;
use mitra_core::expression::eval;
use mitra_core::model::calls;
use mitra_core::parse::parse;
use mitra_core::simulation::{Exact, Value};
use num_bigint::BigInt;
use proptest::prelude::*;
use proptest::test_runner::TestCaseError;
use std::collections::BTreeMap;
use std::sync::Arc;

fn int(n: i64) -> Value {
    Value::Int(BigInt::from(n))
}

fn small() -> impl Strategy<Value = String> {
    (-3i32..4).prop_map(|n| {
        if n < 0 {
            format!("(0 - {})", -n)
        } else {
            n.to_string()
        }
    })
}

/// A term over `names`, small numbers and `+ - *`.
fn term(names: &'static [&'static str]) -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        prop::sample::select(names).prop_map(str::to_string),
        small()
    ];
    leaf.prop_recursive(2, 6, 2, |inner| {
        (inner.clone(), inner, 0..3usize)
            .prop_map(|(a, b, o)| format!("({a} {} {b})", ["+", "-", "*"][o]))
    })
}

fn cmp(names: &'static [&'static str]) -> impl Strategy<Value = String> {
    (term(names), term(names), 0..3usize)
        .prop_map(|(a, b, o)| format!("{a} {} {b}", ["<", "<=", "=="][o]))
}

/// A claim, often `!p || c`.
fn claim(names: &'static [&'static str]) -> impl Strategy<Value = String> {
    prop_oneof![
        cmp(names),
        (cmp(names), cmp(names)).prop_map(|(p, c)| format!("!({p}) || {c}")),
        (cmp(names), cmp(names), cmp(names))
            .prop_map(|(p, q, c)| format!("!({p}) || !({q}) || {c}")),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    /// Random claims, bodies and promises.
    #[test]
    fn what_holds_given_promises_holds_in_every_world_that_keeps_them(
        promise in claim(&["x", "w"]),
        body in term(&["x", "g(x)"]),
        c in claim(&["x", "y"]),
    ) {
        let text = format!(
            "environment env {{\n    fn g(x: Int) -> w: Int where {{ claim p = {promise}; }}\n}}\n\
             subject s {{\n    use env::g;\n    fn f(x: Int) -> y: Int {{ {body} }} where {{ claim c = {c}; }}\n}}\n"
        );
        sound(&text)?;
    }
}

/// A promise that bounds `w` by numbers, or by `x`.
fn bound() -> impl Strategy<Value = String> {
    (small(), small(), 0..4usize).prop_map(|(a, b, k)| match k {
        0 => format!("{a} <= w"),
        1 => format!("w <= {a}"),
        2 => format!("{a} <= w && w <= {b}"),
        _ => "x <= w".to_string(),
    })
}

/// A premise on `x`.
fn premise() -> impl Strategy<Value = String> {
    (small(), small(), 0..3usize).prop_map(|(a, b, k)| match k {
        0 => format!("{a} <= x"),
        1 => format!("x <= {a}"),
        _ => format!("{a} <= x && x <= {b}"),
    })
}

/// Runs the soundness check of one model: whatever agree says holds of
/// `s::f`'s claim, by bounds alone, is true at x from -6 to 6 in every
/// world where `g(x)` keeps the promise. Gives whether it held given.
fn sound(text: &str) -> Result<bool, TestCaseError> {
    let Ok(model) = parse(text) else {
        return Ok(false);
    };
    if !check(&model).is_empty() {
        return Ok(false);
    }
    let (agreed, _) = agree(
        &model,
        &Settings {
            points: 0,
            ..Settings::default()
        },
    );
    let Some(a) = agreed
        .iter()
        .find(|a| a.against == "claim `c`" && a.agreement == Agreement::Holds)
    else {
        return Ok(false);
    };
    let given = !a.given.is_empty();
    let library = calls(&model).unwrap();
    let g = &model.parts[0].functions[0];
    let f = &model.parts[1].functions[0];
    for x in -6..=6 {
        for w in -12..=12 {
            let at = BTreeMap::from([("x".to_string(), int(x)), ("w".to_string(), int(w))]);
            if eval(&Exact::default(), &at, &g.claims[0].holds) != Ok(Value::Truth(true)) {
                continue;
            }
            let world = library
                .clone()
                .given("env::g", vec![int(x)], int(w))
                .unwrap();
            let exact = Exact {
                calls: Arc::new(world),
            };
            let x_only = BTreeMap::from([("x".to_string(), int(x))]);
            let y = eval(&exact, &x_only, f.body.as_ref().unwrap());
            prop_assert!(
                y.is_ok(),
                "holds, yet no value at x = {}, g(x) = {}\n{}",
                x,
                w,
                text
            );
            let y = y.unwrap();
            let point = BTreeMap::from([("x".to_string(), int(x)), ("y".to_string(), y.clone())]);
            prop_assert_eq!(
                eval(&exact, &point, &f.claims[0].holds),
                Ok(Value::Truth(true)),
                "x = {}, g(x) = {}, y = {:?}\n{}",
                x,
                w,
                y,
                text
            );
        }
    }
    Ok(given)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3000))]

    /// Built to use the promise: `g` bounds `w`, `f` adds to `g(x)`, and
    /// `c` reads `y` where a premise keeps `x`.
    #[test]
    fn bounded_promises_are_read_soundly(
        promise in bound(),
        add in term(&["x"]),
        p in premise(),
        conclusion in (small(), 0..3usize).prop_map(|(a, k)| match k {
            0 => format!("{a} <= y"),
            1 => format!("y <= {a}"),
            _ => format!("{a} < y"),
        }),
    ) {
        let text = format!(
            "environment env {{\n    fn g(x: Int) -> w: Int where {{ claim p = {promise}; }}\n}}\n\
             subject s {{\n    use env::g;\n    fn f(x: Int) -> y: Int {{ g(x) + {add} }} where {{ claim c = !({p}) || {conclusion}; }}\n}}\n"
        );
        sound(&text)?;
    }
}

#[test]
fn bounded_promises_hold_given_often() {
    // The property above must not pass by deciding nothing.
    let text = "environment env {\n    fn g(x: Int) -> w: Int where { claim p = 0 <= w; }\n}\n\
                subject s {\n    use env::g;\n    fn f(x: Int) -> y: Int { g(x) + x } where { claim c = !(1 <= x) || 1 <= y; }\n}\n";
    assert!(sound(text).unwrap());
}

const SUN: &str = "environment sun {
    fn light() -> w: Real where {
        claim bright = 200.0 <= w;
        examples { () -> 850.0 }
    }
}
subject stone {
    use sun::light;
    fn heat(h: Real) -> e: Real { h * light() } where {
        claim warm = !(1.0 <= h) || 200.0 <= e;
    }
}";

#[test]
fn the_example_of_finding_8_holds_given_bright() {
    let (agreed, findings) = agree(&parse(SUN).unwrap(), &Settings::default());
    assert_eq!(findings, vec![]);
    let shown: Vec<String> = hints(&agreed).iter().map(|h| h.to_string()).collect();
    assert_eq!(
        shown,
        ["hint[A010]: claim `warm` holds, given `sun::light::bright`\n --> stone::heat"]
    );
}

#[test]
fn without_the_promise_it_is_unknown() {
    let text = SUN.replace("claim bright = 200.0 <= w;", "");
    let (_, findings) = agree(&parse(&text).unwrap(), &Settings::default());
    let codes: Vec<String> = findings.iter().map(|f| f.code.to_string()).collect();
    assert_eq!(codes, ["A007"]);
}

#[test]
fn a_promise_that_calls_its_own_function_ends() {
    let text = "environment e {
    fn g(x: Int) -> w: Int where { claim c = w <= g(x); }
}
subject s {
    use e::g;
    fn f(x: Int) -> y: Int { g(x) + 1 } where { claim d = !(0 <= x) || y <= g(x) + 1; }
}";
    let model = parse(text).unwrap();
    // It ends: a promise reads its own function within the budget only.
    let _ = agree(&model, &Settings::default());
}

#[test]
fn a_premise_on_a_name_a_let_binds_is_not_read() {
    // Inside the claim, `y` is what the body gives, bound by a `let`: a
    // premise on it says nothing of the input, and must not cut it.
    let text = "subject s {
    fn f(x: Int) -> y: Int { 0 - x } where {
        claim c = !(0 <= y) || x <= 0;
    }
}";
    let (agreed, _) = agree(
        &parse(text).unwrap(),
        &Settings {
            points: 0,
            ..Settings::default()
        },
    );
    // True (y >= 0 means x <= 0), but bounds cannot see it through y:
    // not Holds by a wrong cut, and never a failure.
    assert!(agreed
        .iter()
        .all(|a| !matches!(a.agreement, Agreement::Fails { .. })));
}

/// By bounds alone, never a wrong `Holds`: what the guards of `implied`
/// stop (each found by a mutation that removed it).
#[test]
fn bounds_alone_keep_out_what_faults_or_shadows() {
    let text = "subject s {
    fn f(x: Int) -> y: Int { x } where {
        claim c = let x = 0 - x in !(1 <= x) || 1 <= 0 - x;
    }
    fn g(x: Real) -> y: Real { x } where {
        claim d = !(6.0 / x <= 6.0 && 1.0 <= x) || 1.0 <= x;
    }
}";
    let model = parse(text).unwrap();
    assert_eq!(check(&model), vec![]);
    let (agreed, _) = agree(
        &model,
        &Settings {
            points: 0,
            ..Settings::default()
        },
    );
    // `c` is false at x = -1: its premise is on the `x` the let binds,
    // not the input's. `d` has no value at x = 0: the premise divides by
    // it before `1.0 <= x` is read.
    for claim in ["claim `c`", "claim `d`"] {
        let a = agreed.iter().find(|a| a.against == claim).unwrap();
        assert_ne!(a.agreement, Agreement::Holds, "{claim}");
    }
    // At points, both are found.
    let (_, findings) = agree(&model, &Settings::default());
    let codes: Vec<String> = findings.iter().map(|f| f.code.to_string()).collect();
    assert_eq!(codes, ["A001", "A001"]);
}

/// A value the world does not give is covered by any bounds; a division
/// by zero only by bounds that may fault.
#[test]
fn covers_lets_unknown_in_and_nothing_else() {
    use mitra_core::expression::Fault;
    use mitra_core::simulation::{covers, Bounds, Type, Undefined};
    let one = Bounds::exactly(&int(1));
    assert!(covers(
        &Type::Int,
        &Ok(one.clone()),
        &Err(Fault::Op(Undefined::Unknown("e::g".into())))
    ));
    assert!(!covers(
        &Type::Int,
        &Ok(one.clone()),
        &Err(Fault::Op(Undefined::DivisionByZero))
    ));
    assert!(!covers(
        &Type::Int,
        &Ok(one),
        &Err(Fault::Op(Undefined::Budget("b".into())))
    ));
}
