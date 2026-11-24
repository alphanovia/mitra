//! Red team of finding 2: a record known in part, `R { a: 1, .. }`. What
//! `Exact` gives of it is what it gives of every record that fills in
//! the fields not known, or `Undefined::Unknown`; its bounds hold every
//! such record; and it reads back as printed.

use mitra_core::expression::{eval, Arg, Expression, Fault};
use mitra_core::parse::parse;
use mitra_core::print::print;
use mitra_core::simulation::{
    covers, same, Bounds, Exact, Op, Ranges, Record, Type, Undefined, Value,
};
use num_bigint::BigInt;
use proptest::prelude::*;
use std::collections::BTreeMap;

type E = Expression<Op>;

fn r() -> Record {
    Record {
        args: vec![],
        name: "R".to_string(),
        fields: ["a", "b", "c"]
            .iter()
            .map(|n| (n.to_string(), Type::Int))
            .collect(),
    }
}

fn apply(op: Op, args: Vec<E>) -> E {
    Expression::Apply(
        op,
        args.into_iter()
            .map(|body| Arg {
                binds: vec![],
                body,
            })
            .collect(),
    )
}

/// An `Int` expression over the fields of `x` and `y`, both `R`.
fn int_expr() -> BoxedStrategy<E> {
    let leaf = prop_oneof![
        (0..2usize, 0..3usize).prop_map(|(v, f)| apply(
            Op::Field(["a", "b", "c"][f].to_string()),
            vec![Expression::Name(["x", "y"][v].to_string())]
        )),
        (-2i64..3).prop_map(|n| apply(Op::Int(BigInt::from(n)), vec![])),
    ];
    leaf.prop_recursive(3, 12, 3, |inner| {
        prop_oneof![
            (inner.clone(), inner.clone(), 0..3usize)
                .prop_map(|(a, b, o)| apply([Op::Add, Op::Sub, Op::Mul][o].clone(), vec![a, b])),
            (truth(inner.clone()), inner.clone(), inner)
                .prop_map(|(c, a, b)| apply(Op::If, vec![c, a, b])),
        ]
    })
    .boxed()
}

fn truth(int: BoxedStrategy<E>) -> BoxedStrategy<E> {
    prop_oneof![
        (int.clone(), int.clone()).prop_map(|(a, b)| apply(Op::Less, vec![a, b])),
        (int.clone(), int).prop_map(|(a, b)| apply(Op::Equal, vec![a, b])),
        Just(apply(
            Op::Equal,
            vec![Expression::Name("x".into()), Expression::Name("y".into())]
        )),
    ]
    .boxed()
}

/// A record known where `mask` says, and one that fills in the rest.
fn known_and_filled() -> impl Strategy<Value = (Value, Value)> {
    (
        prop::collection::vec(-2i64..3, 3),
        prop::collection::vec(-2i64..3, 3),
        0u8..8,
    )
        .prop_map(|(known, filled, mask)| {
            let int = |n: i64| Value::Int(BigInt::from(n));
            let fields = (0..3)
                .map(|i| (mask & (1 << i) != 0).then(|| int(known[i])))
                .collect::<Vec<_>>();
            let full = (0..3)
                .map(|i| fields[i].clone().unwrap_or_else(|| int(filled[i])))
                .collect();
            (Value::partial(r(), fields), Value::Record(r(), full))
        })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3000))]

    #[test]
    fn what_is_known_in_part_holds_of_every_record_that_fills_it_in(
        e in prop_oneof![int_expr(), truth(int_expr())],
        (px, fx) in known_and_filled(),
        (py, fy) in known_and_filled(),
    ) {
        let part = BTreeMap::from([("x".to_string(), px.clone()), ("y".to_string(), py.clone())]);
        let full = BTreeMap::from([("x".to_string(), fx.clone()), ("y".to_string(), fy.clone())]);
        let of_part = eval(&Exact::default(), &part, &e);
        let of_full = eval(&Exact::default(), &full, &e);
        match &of_part {
            Ok(v) => prop_assert_eq!(Ok(v.clone()), of_full.clone(), "{:?} {:?}", px, py),
            // Not known: what a filled-in record gives may be anything.
            Err(Fault::Op(Undefined::Unknown(_))) => {}
            // Any other fault: every filled-in record has it too.
            Err(_) => prop_assert!(of_full.is_err(), "{:?} but {:?}", of_part, of_full),
        }
        // `same` never says what a filling in contradicts.
        if let Ok(b) = same(&px, &py) {
            prop_assert_eq!(b, fx == fy);
        }
        // The bounds of what is known in part hold every filling in.
        let bounds = BTreeMap::from([
            ("x".to_string(), Bounds::exactly(&px)),
            ("y".to_string(), Bounds::exactly(&py)),
        ]);
        let by_bounds = eval(&Ranges::default(), &bounds, &e);
        let ty = match e { Expression::Apply(Op::Less | Op::Equal, _) => Type::Truth, _ => Type::Int };
        prop_assert!(covers(&ty, &by_bounds, &of_full), "{:?} not in {:?}", of_full, by_bounds);
    }
}

#[test]
fn a_value_known_in_part_reads_back_as_printed() {
    let text = "environment std {
    record String;
}
subject location {
    use std::String;
    record Location { first: String, depth: integer }
    fn depth(l: Location) -> d: integer { l.depth } where {
        examples { (Location { depth: 3, .. }) -> 3 }
    }
}
";
    let model = parse(text).unwrap();
    assert_eq!(parse(&print(&model)).unwrap(), model);
    assert!(print(&model).contains("(Location { depth: 3, .. }) -> 3"));
    // Known in full, it is a record, not a value known in part.
    let full = Value::partial(r(), vec![Some(Value::Int(1.into())); 3]);
    assert!(matches!(full, Value::Record(..)));
}

#[test]
fn known_in_part_is_written_only_of_a_declared_record() {
    let refused = [
        ("subject s {\n    fn f(x: integer) -> y: integer where { examples { (Q { a: 1, .. }) -> 1 } }\n}", "must be a record of the model"),
        ("subject s {\n    record P { a: integer }\n    fn f(p: P) -> y: integer where { examples { (P { z: 1, .. }) -> 1 } }\n}", "`P` has no field `z`"),
        ("subject s {\n    record P { a: integer, b: integer }\n    fn f(p: P) -> y: integer where { examples { (P { a: 1, a: 2, .. }) -> 1 } }\n}", "field `a` is given twice"),
    ];
    for (text, says) in refused {
        let e = parse(text).unwrap_err();
        assert!(
            e.render("t", text).contains(says),
            "{}",
            e.render("t", text)
        );
    }
}

#[test]
fn agree_compares_only_what_is_known() {
    use mitra_core::agree::{agree, Settings};
    let text = "environment std { record String; }
subject location {
    use std::String;
    record Location { first: String, depth: integer }
    fn depth(l: Location) -> d: integer { l.depth } where {
        examples { (Location { depth: 3, .. }) -> 3 }
    }
    fn first_is(l: Location, m: Location) -> same: bool { l.first == m.first } where {
        examples { (Location { depth: 1, .. }, Location { depth: 1, .. }) -> true }
    }
    fn keep(l: Location) -> m: Location { l } where {
        examples { (Location { depth: 1, .. }) -> Location { depth: 1, .. } }
    }
}";
    let (_, findings) = agree(&parse(text).unwrap(), &Settings::default());
    let shown: Vec<String> = findings.iter().map(|f| f.to_string()).collect();
    // `depth` holds; `first_is` reads what no example knows: unknown,
    // and said, never a failure.
    // `keep` gives what it is given: the same where known, and only
    // `first`, not known, could tell them apart.
    assert_eq!(
        shown,
        [
            "finding[A007]: example 0: unknown, no value for `Location::first` here\n --> location::first_is",
            "finding[A007]: example 0: unknown, `Location::first` is not known, and only it could tell\n --> location::keep",
        ]
    );
}
