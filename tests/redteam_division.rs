//! Red team of finding 11: `/` and `%` on `Int` as Rust's `i64` gives
//! them, by `Exact`; bounds that hold every value `Exact` gives; and the
//! types: no number converted.

use mitra_core::check::check;
use mitra_core::expression::{eval, Arg, Expression, Fault};
use mitra_core::parse::parse;
use mitra_core::simulation::{
    covers, Bounds, Exact, Numbers, Op, Ranges, Rational, Type, Undefined, Value,
};
use num_bigint::BigInt;
use proptest::prelude::*;
use std::collections::BTreeMap;

type E = Expression<Op>;

fn name(n: &str) -> E {
    Expression::Name(n.to_string())
}

fn two(op: Op, a: E, b: E) -> E {
    let arg = |e| Arg {
        binds: vec![],
        body: e,
    };
    Expression::Apply(op, vec![arg(a), arg(b)])
}

fn int(n: i64) -> Value {
    Value::Int(BigInt::from(n))
}

#[test]
fn exact_divides_as_rust_does() {
    for a in -20i64..=20 {
        for b in -7i64..=7 {
            let at = BTreeMap::from([("a".to_string(), int(a)), ("b".to_string(), int(b))]);
            let div = eval(&Exact::default(), &at, &two(Op::Div, name("a"), name("b")));
            let rem = eval(&Exact::default(), &at, &two(Op::Rem, name("a"), name("b")));
            match b {
                0 => {
                    let zero = Err(Fault::Op(Undefined::DivisionByZero));
                    assert_eq!((div, rem), (zero.clone(), zero), "{a} / 0");
                }
                _ => assert_eq!((div, rem), (Ok(int(a / b)), Ok(int(a % b))), "{a} / {b}"),
            }
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4000))]

    /// Bounds of `a / b` and `a % b`, `a` and `b` each in a span of
    /// whole numbers, a half-line or no end: every value `Exact` gives at
    /// the points of the spans lies in them, and a fault only where they
    /// say one may be.
    #[test]
    fn bounds_hold_what_exact_gives(
        al in -12i64..12, aw in 0i64..8, bl in -6i64..6, bw in 0i64..5,
        open in 0usize..5, rem in any::<bool>(),
    ) {
        let (ah, bh) = (al + aw, bl + bw);
        let end = |n: i64| Some(Rational::integer(n));
        // Half-lines: the points tried stay in the span.
        let (a_low, a_high) = match open { 1 => (None, end(ah)), 2 => (end(al), None), 3 => (None, None), _ => (end(al), end(ah)) };
        let span = mitra_core::simulation::Span::Number(Numbers::Int, a_low, a_high);
        let bounds = BTreeMap::from([
            ("a".to_string(), Bounds { span, may_fault: false }),
            ("b".to_string(), Bounds::between(Numbers::Int, Rational::integer(bl), Rational::integer(bh)).unwrap()),
        ]);
        let op = if rem { Op::Rem } else { Op::Div };
        let e = two(op, name("a"), name("b"));
        let by_bounds = eval(&Ranges::default(), &bounds, &e);
        for a in al..=ah {
            for b in bl..=bh {
                let at = BTreeMap::from([("a".to_string(), int(a)), ("b".to_string(), int(b))]);
                let exact = eval(&Exact::default(), &at, &e);
                prop_assert!(covers(&Type::Int, &by_bounds, &exact), "{} {} {}: {:?} not in {:?}", a, if rem { "%" } else { "/" }, b, exact, by_bounds);
            }
        }
    }
}

#[test]
fn at_one_point_the_bounds_are_the_value() {
    let at = |a: i64, b: i64| {
        BTreeMap::from([
            ("a".to_string(), Bounds::exactly(&int(a))),
            ("b".to_string(), Bounds::exactly(&int(b))),
        ])
    };
    for (op, a, b, gives) in [
        (Op::Div, -7, 2, -3),
        (Op::Rem, -7, 2, -1),
        (Op::Div, 7, -2, -3),
        (Op::Rem, 7, -2, 1),
    ] {
        let e = two(op, name("a"), name("b"));
        assert_eq!(
            eval(&Ranges::default(), &at(a, b), &e),
            Ok(Bounds::exactly(&int(gives)))
        );
    }
}

fn errors(text: &str) -> Vec<String> {
    check(&parse(text).unwrap())
        .iter()
        .map(|e| e.to_string())
        .collect()
}

#[test]
fn whole_numbers_divide_and_types_stay_apart() {
    // Main's first example, in full.
    let div = "subject calc {
    fn div(a: Int, b: Int) -> q: Int { if b == 0 { 0 } else { a / b } }
}";
    assert_eq!(errors(div), Vec::<String>::new());
    // `/` on two Real stays exact; `%` on Int only; no mix.
    let ok = "subject s {
    fn f(x: Real) -> y: Real { x / 2.0 }
    fn g(x: Int) -> y: Int { x % 3 }
}";
    assert_eq!(errors(ok), Vec::<String>::new());
    let mixed = "subject s {
    fn f(x: Int) -> y: Int { x / 2.0 }
}";
    let e = errors(mixed);
    assert!(e[0].contains("`x` is Int and `2.0` is Real"), "{e:?}");
    assert!(
        e[0].contains("= note: `/` takes two Int or two Real"),
        "{e:?}"
    );
    let real_rem = "subject s {
    fn f(x: Real) -> y: Real { x % 2.0 }
}";
    let e = errors(real_rem);
    assert!(e[0].contains("= note: `%` takes two Int"), "{e:?}");
}

#[test]
fn agree_reads_whole_division() {
    use mitra_core::agree::{agree, Agreement, Settings};
    // True of Rust's `/` and `%`: a == q * b + r, at every point.
    let text = "subject s {
    fn f(a: Int, b: Int) -> r: Int { if b == 0 { 0 } else { a % b } } where {
        claim parts: b == 0 || a == (a / b) * b + r;
        claim smaller: !(0 < b) || r < b;
    }
}";
    // Bounds prove no law of algebra: unknown, and said (A007), never a
    // failure at any of the points tried.
    let (agreed, findings) = agree(&parse(text).unwrap(), &Settings::default());
    assert!(
        agreed
            .iter()
            .all(|a| !matches!(a.agreement, Agreement::Fails { .. })),
        "{agreed:#?}"
    );
    let codes: Vec<String> = findings.iter().map(|f| f.code.to_string()).collect();
    assert_eq!(codes, ["A007", "A007"]);
    // Floor's rule is not Rust's: -7 / 2 is -3, not -4. Agree says where.
    let floor = text.replace(
        "claim smaller: !(0 < b) || r < b;",
        "claim floor: !(0 < b) || 0 <= r;",
    );
    let (_, findings) = agree(&parse(&floor).unwrap(), &Settings::default());
    let shown: Vec<String> = findings.iter().map(|f| f.to_string()).collect();
    assert!(
        shown
            .iter()
            .any(|f| f.starts_with("finding[A001]: the body breaks claim `floor` at a = -")),
        "{shown:?}"
    );
}
