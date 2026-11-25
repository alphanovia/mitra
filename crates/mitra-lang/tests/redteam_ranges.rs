//! Red team of finding 11's range types, `a: 0..=9`, an `Int` that is
//! checked: what agree says holds is true at every point of the ranges,
//! what it says fails, fails at a point in them; a call by bounds holds
//! what `Exact` gives, a value outside a range being none; and a range
//! reads back as printed.

use mitra_lang::agree::{agree, Agreement, Settings};
use mitra_lang::check::check;
use mitra_lang::expression::{eval, Arg, Expression};
use mitra_lang::model::calls;
use mitra_lang::parse::parse;
use mitra_lang::print::print;
use mitra_lang::simulation::{covers, Bounds, Exact, Numbers, Op, Ranges, Rational, Type, Value};
use num_bigint::BigInt;
use proptest::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;

/// A point of the ranges, whether the body keeps its range there, and
/// whether the claim holds.
type Point = ((i64, i64), bool, bool);

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

/// A term over `a`, `b` and small numbers, with `/` and `%`.
fn term() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![Just("a".to_string()), Just("b".to_string()), small()];
    leaf.prop_recursive(2, 6, 2, |inner| {
        (inner.clone(), inner, 0..5usize)
            .prop_map(|(x, y, o)| format!("({x} {} {y})", ["+", "-", "*", "/", "%"][o]))
    })
}

/// A range of a few numbers about zero.
fn range() -> impl Strategy<Value = (i64, i64)> {
    (-4i64..4, 0i64..5).prop_map(|(l, w)| (l, l + w))
}

fn shown((l, h): (i64, i64)) -> String {
    format!("{l}..={h}")
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1500))]

    #[test]
    fn agree_is_right_within_the_ranges(
        ra in range(), rb in range(), rq in prop::option::of(range()),
        body in term(), c in term(), k in small(), op in 0..3usize,
    ) {
        let gives = rq.map_or("integer".to_string(), shown);
        let cmp = ["<", "<=", "=="][op];
        let text = format!(
            "subject s {{\n    fn f(a: {}, b: {}) -> q: {gives} {{ {body} }} where {{ claim c: {c} {cmp} q + {k}; }}\n}}\n",
            shown(ra), shown(rb)
        );
        let Ok(model) = parse(&text) else { return Ok(()) };
        prop_assert_eq!(check(&model), vec![], "{}", text);
        // It reads back as printed.
        prop_assert_eq!(parse(&print(&model)).ok(), Some(model.clone()), "{}", text);
        let f = &model.parts[0].functions[0];
        let (agreed, _) = agree(&model, &Settings::default());
        // Exact at every point of the ranges.
        let mut every = vec![];
        for a in ra.0..=ra.1 {
            for b in rb.0..=rb.1 {
                let at = BTreeMap::from([("a".to_string(), int(a)), ("b".to_string(), int(b))]);
                let q = eval(&Exact::default(), &at, f.body.as_ref().unwrap());
                let in_range = match (&q, rq) {
                    (Ok(Value::Int(n)), Some((l, h))) => &BigInt::from(l) <= n && n <= &BigInt::from(h),
                    (Ok(_), None) => true,
                    _ => false,
                };
                let claim = q.clone().ok().map(|q| {
                    let mut at = at.clone();
                    at.insert("q".to_string(), q);
                    eval(&Exact::default(), &at, &f.claims[0].holds) == Ok(Value::Truth(true))
                });
                every.push(((a, b), in_range, claim == Some(true)));
            }
        }
        for a in &agreed {
            let keeps: Box<dyn Fn(&Point) -> bool> = match a.against.as_str() {
                "claim `c`" => Box::new(|p| p.2),
                x if x.starts_with("its range") => Box::new(|p| p.1),
                _ => continue,
            };
            match &a.agreement {
                Agreement::Holds => prop_assert!(every.iter().all(&keeps), "{} holds, and is false at {:?}\n{}", a.against, every.iter().find(|p| !keeps(p)), text),
                Agreement::Fails { at, .. } => {
                    let point: Vec<i64> = at.iter().map(|(_, v)| match v { Value::Int(n) => i64::try_from(n).unwrap(), _ => 99 }).collect();
                    let p = every.iter().find(|p| [p.0.0, p.0.1] == point[..]);
                    prop_assert!(p.is_some_and(|p| !keeps(p)), "{} fails at {:?}, which is in no range or keeps it\n{}", a.against, point, text);
                }
                Agreement::Unknown { .. } | Agreement::Uncalled { .. } => {}
            }
        }
    }

    /// A call by bounds holds what `Exact` gives at each point of the
    /// bounds of its argument: outside the range what it takes, or the
    /// range it gives, no value.
    #[test]
    fn a_call_by_bounds_holds_what_exact_gives(
        ra in range(), rq in prop::option::of(range()), body in term(),
        xl in -6i64..6, xw in 0i64..6, open in 0..4usize,
    ) {
        let body = body.replace('b', "a");
        let gives = rq.map_or("integer".to_string(), shown);
        let text = format!("subject s {{\n    fn f(a: {}) -> q: {gives} {{ {body} }}\n}}\n", shown(ra));
        let Ok(model) = parse(&text) else { return Ok(()) };
        let library = Arc::new(calls(&model).unwrap());
        let e = Expression::Apply(Op::Call("s::f".to_string()), vec![Arg { binds: vec![], body: Expression::Name("x".to_string()) }]);
        let end = |n: i64| Some(Rational::integer(n));
        let (lo, hi) = match open { 1 => (None, end(xl + xw)), 2 => (end(xl), None), _ => (end(xl), end(xl + xw)) };
        let span = mitra_lang::simulation::Span::Number(Numbers::Int, lo, hi);
        let bounds = BTreeMap::from([("x".to_string(), Bounds { span, may_fault: false })]);
        let by_bounds = eval(&Ranges { calls: library.clone() }, &bounds, &e);
        for x in xl..=xl + xw {
            let at = BTreeMap::from([("x".to_string(), int(x))]);
            let exact = eval(&Exact { calls: library.clone(), log: Default::default() }, &at, &e);
            prop_assert!(covers(&Type::Int, &by_bounds, &exact), "x = {}: {:?} not in {:?}\n{}", x, exact, by_bounds, text);
        }
    }
}

#[test]
fn main_s_div_in_full() {
    let text = "subject calc {
    fn div(a: 0..=9, b: 0..=9) -> q: 0..=9 { if b == 0 { 0 } else { a / b } } where {
        claim spec: if b == 0 { q == 0 } else { q * b <= a && a < (q + 1) * b };
    }
}";
    let model = parse(text).unwrap();
    assert_eq!(check(&model), vec![]);
    // 100 points: every one, with the points setting at 100.
    let (agreed, findings) = agree(
        &model,
        &Settings {
            points: 100,
            ..Settings::default()
        },
    );
    assert_eq!(findings, vec![]);
    assert!(
        agreed.iter().all(|a| a.agreement == Agreement::Holds),
        "{agreed:#?}"
    );
    // At the default, 64, it is said unknown: never silently held.
    let (_, findings) = agree(&model, &Settings::default());
    let codes: Vec<String> = findings.iter().map(|f| f.code.to_string()).collect();
    assert_eq!(codes, ["A007", "A007"]);
}

#[test]
fn a_caller_outside_the_range_has_no_value_and_agree_says_where() {
    let text = "subject calc {
    fn digit(a: 0..=9) -> q: integer { a }
    fn next(x: integer) -> y: integer { digit(x + 1) } where {
        claim c: y == x + 1;
    }
}";
    let (_, findings) = agree(&parse(text).unwrap(), &Settings::default());
    let shown: Vec<String> = findings.iter().map(|f| f.to_string()).collect();
    assert_eq!(shown.len(), 1, "{shown:?}");
    assert!(
        shown[0].starts_with("finding[A001]: the body breaks claim `c` at x = "),
        "{shown:?}"
    );
    assert!(shown[0].contains(", where it has no value"), "{shown:?}");
    // Inside its range, by bounds alone: a claim bounds decide holds, the
    // call never out of range.
    let inside = text
        .replace("fn next(x: integer)", "fn next(x: 0..=8)")
        .replace("claim c: y == x + 1;", "claim c: 1 <= y && y <= 9;");
    let (agreed, _) = agree(
        &parse(&inside).unwrap(),
        &Settings {
            points: 0,
            ..Settings::default()
        },
    );
    let c = agreed.iter().find(|a| a.against == "claim `c`").unwrap();
    assert_eq!(c.agreement, Agreement::Holds);
    // One past the range: no longer held, the call may have no value.
    let past = inside.replace("0..=8", "0..=9");
    let (agreed, _) = agree(
        &parse(&past).unwrap(),
        &Settings {
            points: 0,
            ..Settings::default()
        },
    );
    let c = agreed.iter().find(|a| a.against == "claim `c`").unwrap();
    assert_ne!(c.agreement, Agreement::Holds);
}

/// A conclusion read before its premise is read everywhere: it may not
/// fault anywhere (the premise-first reading of `implied` is only for
/// premises read first).
#[test]
fn a_conclusion_read_first_must_not_fault_anywhere() {
    let text = "subject s {
    fn f(x: rational) -> y: rational { 1.0 / x } where {
        claim first: !(1.0 <= x) || y <= 1.0;
        claim last: y <= 1.0 || !(1.0 <= x);
        claim never: y <= 1.0 || !(1.0 <= x && x <= 0.0);
    }
}";
    let (agreed, findings) = agree(
        &parse(text).unwrap(),
        &Settings {
            points: 0,
            ..Settings::default()
        },
    );
    let of = |c: &str| {
        agreed
            .iter()
            .find(|a| a.against == c)
            .unwrap()
            .agreement
            .clone()
    };
    assert_eq!(of("claim `first`"), Agreement::Holds);
    // `last` has no value at x = 0: never held.
    assert_ne!(of("claim `last`"), Agreement::Holds, "{findings:?}");
    // No value keeps its premise, yet its conclusion is read first, and
    // has no value at x = 0.
    assert_ne!(of("claim `never`"), Agreement::Holds, "{findings:?}");
    let (_, findings) = agree(&parse(text).unwrap(), &Settings::default());
    assert!(
        findings.iter().any(|f| f
            .message
            .starts_with("the body breaks claim `last` at x = 0.0")),
        "{findings:?}"
    );
}
