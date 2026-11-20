//! The claims of `model/value.mitra`.

use mitra_core::value::Expression::*;
use mitra_core::value::Type::*;
use mitra_core::value::Value::*;
use mitra_core::value::*;

fn op(op: &str, args: &[Expression]) -> Expression {
    Op(op.to_string(), args.to_vec())
}
fn given(name: &str, v: Value) -> Vec<(String, Value)> {
    vec![(name.to_string(), v)]
}
/// A few expressions of every form, some that check and some that do not.
fn some() -> Vec<Expression> {
    let point = Make("Point".to_string(), vec![Literal(1), Yes]);
    vec![
        Yes,
        Literal(7),
        point.clone(),
        Field(Box::new(point.clone()), 1),
        Field(Box::new(point.clone()), 2),
        Field(Box::new(Yes), 0),
        op("!", &[No]),
        op("&&", &[Yes, No]),
        op("==", &[point.clone(), point]),
        op("==", &[Yes, Literal(1)]),
        op("<", &[Literal(1), Literal(2)]),
        op("+", &[Literal(1), Literal(2)]),
        op("+", &[Yes, Literal(1)]),
        op("*", &[Literal(i64::MAX), Literal(2)]),
        If(Box::new(Yes), Box::new(Literal(1)), Box::new(Literal(2))),
        If(Box::new(Yes), Box::new(Literal(1)), Box::new(No)),
        Input("x".to_string()),
    ]
}

// mitra: evidence for value::same_value
#[test]
fn same_value() {
    for e in some() {
        assert_eq!(eval(&e, &[]), eval(&e, &[]));
    }
}

// mitra: evidence for value::literal_is_itself
#[test]
fn literal_is_itself() {
    for n in [i64::MIN, -1, 0, 42, i64::MAX] {
        assert_eq!(eval(&Literal(n), &[]), Some(Number(n)));
    }
    assert_eq!(eval(&Yes, &[]), Some(Truth(true)));
    assert_eq!(eval(&No, &[]), Some(Truth(false)));
}

// mitra: evidence for value::input_is_what_was_given
#[test]
fn input_is_what_was_given() {
    for v in [
        Truth(true),
        Number(3),
        Made("P".to_string(), vec![Number(1)]),
    ] {
        assert_eq!(
            eval(&Input("x".to_string()), &given("x", v.clone())),
            Some(v)
        );
    }
    assert_eq!(eval(&Input("x".to_string()), &[]), None);
}

// mitra: evidence for value::checked_gives_its_type
#[test]
fn checked_gives_its_type() {
    let mut checked = 0;
    for e in some() {
        if let (Some(t), Some(v)) = (check(&e, &[]), eval(&e, &[])) {
            assert_eq!(type_of(&v), t, "{e:?}");
            checked += 1;
        }
    }
    assert!(checked >= 9);
}

// mitra: evidence for value::truth_is_not_a_number
#[test]
fn truth_is_not_a_number() {
    assert_eq!(type_of(&Truth(true)), OfTruth);
    assert_eq!(type_of(&Number(0)), OfNumber);
    assert_eq!(check(&op("+", &[Yes, Literal(1)]), &[]), None);
}

#[test]
fn a_number_too_large_is_no_value() {
    assert_eq!(eval(&op("*", &[Literal(i64::MAX), Literal(2)]), &[]), None);
    assert_eq!(
        eval(&op("+", &[Literal(40), Literal(2)]), &[]),
        Some(Number(42))
    );
}
