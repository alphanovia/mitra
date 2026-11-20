//! The claims of `model/bounds.mitra`: the standard operations read on
//! bounds, against the same operations read exactly.

use mitra_core::bounds::*;
use mitra_core::standard::*;
use mitra_core::value::Expression::*;
use mitra_core::value::{eval, Operation, Type};

type Exactly = mitra_core::value::Expression<Exact>;
type Value = mitra_core::value::Value<Exact>;
type Ranged = mitra_core::value::Value<Bounds>;

fn span(of: &Type, low: i64, high: i64) -> Ranged {
    between(of, (low, 1), (high, 1)).unwrap()
}
fn exactly(e: &Exactly, inputs: &[(String, Value)]) -> Option<Value> {
    eval(e, inputs, &Table::default())
}
fn by_bounds(e: &Exactly, inputs: &[(String, Ranged)]) -> Option<Ranged> {
    eval(&lifted(e)?, inputs, &Ranges)
}
fn x_and_y(op: Operation) -> Exactly {
    let of = op.takes[0].clone();
    Apply(
        op,
        vec![Input("x".into(), of.clone()), Input("y".into(), of)],
    )
}
/// The exact value lies in the bounds.
fn inside(v: &Value, b: &Bounds) -> bool {
    match (&b.within, point(v).map(|p| p.quantity.within)) {
        (Span::Truths { yes, no }, Some(Span::Truths { yes: a, no: c })) => {
            (*yes || !a) && (*no || !c)
        }
        (Span::Numbers { low, high }, Some(Span::Numbers { low: at, .. })) => {
            let under = |a: (i64, i64), b: (i64, i64)| {
                a.0 as i128 * b.1 as i128 <= b.0 as i128 * a.1 as i128
            };
            under(*low, at) && under(at, *high)
        }
        _ => false,
    }
}

// mitra: evidence for bounds::a_whole_number_is_its_own_bounds
// mitra: evidence for bounds::a_sure_truth_is_sure
#[test]
fn a_point_is_itself() {
    let five = point(&number(5)).unwrap();
    assert_eq!(Some(five), between(&whole(), (5, 1), (5, 1)));
    let half = point(&ratio(2, 4).unwrap()).unwrap();
    assert_eq!(Some(half), between(&real(), (1, 2), (1, 2)));
    assert_eq!(surely(&point(&yes()).unwrap()), Some(true));
    assert_eq!(surely(&point(&no()).unwrap()), Some(false));
}

// mitra: evidence for tests::bounds_are_in_order
#[test]
fn bounds_are_in_order() {
    assert_eq!(between(&whole(), (2, 1), (1, 1)), None);
    assert_eq!(between(&whole(), (1, 2), (1, 1)), None);
    assert!(between(&real(), (1, 2), (1, 1)).is_some());
    assert_eq!(between(&real(), (1, 0), (1, 1)), None);
}

/// Every standard operation on numbers, on every pair of points of two
/// small ranges: what the exact table gives is inside what the ranges
/// table gives, and where the exact one has no value the other says it
/// may fault.
#[test]
fn bounds_hold_the_exact() {
    let int = whole();
    let i8 = int8();
    let mut seen = 0;
    let ops: Vec<(Operation, i64, i64, i64, i64)> = vec![
        (add(&int), -3, 4, -2, 5),
        (sub(&int), -3, 4, -2, 5),
        (mul(&int), -3, 4, -2, 5),
        (mul(&int), -6, -2, 3, 5),
        (less(&int), -1, 3, 2, 4),
        (at_most(&int), -1, 3, 3, 4),
        (more(&int), 5, 7, 2, 4),
        (at_least(&int), 1, 2, 2, 4),
        (widening_add(), 100, 127, 100, 127),
        (wrapping_add(), 100, 127, 20, 40),
        (wrapping_add(), -128, -120, -10, -1),
        (checked_add(), 100, 127, 20, 40),
        (checked_add(), -5, 5, -5, 5),
    ];
    for (op, xl, xh, yl, yh) in ops {
        let of = op.takes[0].clone();
        let e = x_and_y(op.clone());
        let ranges = [
            ("x".to_string(), span(&of, xl, xh)),
            ("y".to_string(), span(&of, yl, yh)),
        ];
        let b = by_bounds(&e, &ranges).unwrap().quantity;
        for x in xl..=xh {
            for y in yl..=yh {
                let make = |n: i64| match of == i8 {
                    true => small(n).unwrap(),
                    false => number(n),
                };
                let at = [("x".to_string(), make(x)), ("y".to_string(), make(y))];
                match exactly(&e, &at) {
                    Some(v) => assert!(inside(&v, &b), "{} at {x}, {y}", op.name),
                    None => assert!(b.may_fault, "{} at {x}, {y}", op.name),
                }
                seen += 1;
            }
        }
    }
    assert!(seen > 2000);
}

// mitra: evidence for tests::an_order_is_sure_or_not
#[test]
fn an_order_is_sure_or_not() {
    let int = whole();
    let read = |x: Ranged, y: Ranged| {
        let ranges = [("x".to_string(), x), ("y".to_string(), y)];
        surely(&by_bounds(&x_and_y(less(&int)), &ranges).unwrap())
    };
    assert_eq!(read(span(&int, 0, 3), span(&int, 4, 9)), Some(true));
    assert_eq!(read(span(&int, 4, 9), span(&int, 0, 4)), Some(false));
    assert_eq!(read(span(&int, 0, 5), span(&int, 4, 9)), None);
}

/// Int8 plus Int8, three ways. Widened to Int16 it always has a value.
/// Wrapped it stays an Int8 and may be anywhere in the type. Checked,
/// it has no value when the sum does not fit.
// mitra: evidence for tests::widened_it_is_two_hundred
// mitra: evidence for tests::wrapped_it_is_less_than_nothing
// mitra: evidence for tests::checked_it_has_no_value
// mitra: evidence for tests::no_int8_past_127
#[test]
fn int8_three_ways() {
    let at = |x: i64, y: i64| {
        [
            ("x".to_string(), small(x).unwrap()),
            ("y".to_string(), small(y).unwrap()),
        ]
    };
    let wide = exactly(&x_and_y(widening_add()), &at(100, 100)).unwrap();
    assert_eq!((wide.quantity, wide.of), (Exact::Whole(200), int16()));
    let wrapped = exactly(&x_and_y(wrapping_add()), &at(100, 100));
    assert_eq!(wrapped, small(-56));
    assert_eq!(exactly(&x_and_y(checked_add()), &at(100, 100)), None);
    assert_eq!(exactly(&x_and_y(checked_add()), &at(100, 27)), small(127));
    assert_eq!(small(128), None);
}

/// The same three over the whole type: bounds say which one can fault.
// mitra: evidence for tests::bounds_say_which_addition_faults
// mitra: evidence for tests::int8_three_ways
#[test]
fn bounds_say_which_addition_faults() {
    let all = || [("x".to_string(), any_int8()), ("y".to_string(), any_int8())];
    let read = |op: Operation| by_bounds(&x_and_y(op), &all()).unwrap().quantity;
    let numbers = |low: i64, high: i64| Span::Numbers {
        low: (low, 1),
        high: (high, 1),
    };
    let wide = read(widening_add());
    assert_eq!((wide.within, wide.may_fault), (numbers(-256, 254), false));
    let wrapped = read(wrapping_add());
    assert_eq!(
        (wrapped.within, wrapped.may_fault),
        (numbers(-128, 127), false)
    );
    let checked = read(checked_add());
    assert_eq!(
        (checked.within, checked.may_fault),
        (numbers(-128, 127), true)
    );
    // Small enough, the checked one cannot fault.
    let i8 = int8();
    let small_ones = [
        ("x".to_string(), span(&i8, 0, 60)),
        ("y".to_string(), span(&i8, 0, 60)),
    ];
    let safe = by_bounds(&x_and_y(checked_add()), &small_ones)
        .unwrap()
        .quantity;
    assert_eq!((safe.within, safe.may_fault), (numbers(0, 120), false));
    // Always too large, it never has a value.
    let large = [
        ("x".to_string(), span(&i8, 100, 127)),
        ("y".to_string(), span(&i8, 100, 127)),
    ];
    let never = by_bounds(&x_and_y(checked_add()), &large).unwrap().quantity;
    assert_eq!((never.within, never.may_fault), (Span::Empty, true));
}

/// A fault is carried by whatever is computed from it.
#[test]
fn a_fault_is_carried() {
    let i8 = int8();
    let sum = x_and_y(checked_add());
    let limit = Given(small(100).unwrap());
    let twice = Apply(checked_add(), vec![sum, limit]);
    let ranges = [
        ("x".to_string(), span(&i8, 0, 100)),
        ("y".to_string(), span(&i8, 0, 100)),
    ];
    assert!(by_bounds(&twice, &ranges).unwrap().quantity.may_fault);
    let never = [
        ("x".to_string(), span(&i8, 100, 127)),
        ("y".to_string(), span(&i8, 100, 127)),
    ];
    let b = by_bounds(&twice, &never).unwrap().quantity;
    assert_eq!((b.within, b.may_fault), (Span::Empty, true));
}

/// `if` reads the branches its test may take: one when the test is
/// sure, both joined when it is not.
// mitra: evidence for tests::if_reads_the_branches_it_may_take
#[test]
fn if_reads_the_branches_it_may_take() {
    let int = whole();
    let test = Apply(
        less(&int),
        vec![Input("x".into(), int.clone()), Given(number(10))],
    );
    let too_large = Apply(mul(&int), vec![Given(number(i64::MAX)), Given(number(2))]);
    let pick = Apply(
        choose(&int),
        vec![test.clone(), Given(number(1)), too_large],
    );
    let sure = [("x".to_string(), span(&int, 0, 5))];
    assert_eq!(by_bounds(&pick, &sure), between(&int, (1, 1), (1, 1)));
    let either = Apply(choose(&int), vec![test, Given(number(1)), Given(number(7))]);
    let unsure = [("x".to_string(), span(&int, 0, 20))];
    assert_eq!(by_bounds(&either, &unsure), between(&int, (1, 1), (7, 1)));
}

/// What the ranges table lacks is unknown, not wrong: `==`, and
/// records.
// mitra: evidence for tests::a_lacking_operation_is_unknown
#[test]
fn a_lacking_operation_is_unknown() {
    let int = whole();
    let same = x_and_y(equal(&int));
    let ranges = [
        ("x".to_string(), span(&int, 1, 1)),
        ("y".to_string(), span(&int, 1, 1)),
    ];
    assert_eq!(by_bounds(&same, &ranges), None);
    let shape = Shape {
        name: "Point".to_string(),
        fields: vec![int.clone()],
    };
    let made = eval(
        &Apply(make(&shape), vec![Given(number(1))]),
        &[],
        &Table {
            shapes: vec![shape],
        },
    )
    .unwrap();
    assert_eq!(point(&made), None);
}

/// A truth that may be either, or that may fault, is not sure.
// mitra: evidence for bounds::a_truth_that_may_fault_is_not_sure
#[test]
fn a_truth_that_may_fault_is_not_sure() {
    let said = |yes: bool, no: bool, may_fault: bool| Ranged {
        quantity: Bounds {
            within: Span::Truths { yes, no },
            may_fault,
        },
        of: truth(),
    };
    assert_eq!(surely(&said(true, false, true)), None);
    assert_eq!(surely(&said(true, true, false)), None);
    assert_eq!(surely(&said(true, false, false)), Some(true));
}
