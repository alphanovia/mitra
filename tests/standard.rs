//! The claims of `model/standard.mitra`.

use mitra_core::standard::*;
use mitra_core::value::Expression::*;
use mitra_core::value::{check, eval, type_of, Operation};

type Expression = mitra_core::value::Expression<Exact>;
type Value = mitra_core::value::Value<Exact>;

fn n(x: i64) -> Expression {
    Given(number(x))
}
fn point() -> Shape {
    Shape {
        name: "Point".to_string(),
        fields: vec![whole(), truth()],
    }
}
fn plain() -> Table {
    Table::default()
}
fn value_of(e: &Expression) -> Option<Value> {
    eval(e, &[], &plain())
}

// mitra: evidence for standard::yes_and_no_are_truths
#[test]
fn yes_and_no_are_truths() {
    assert_eq!((type_of(&yes()), type_of(&no())), (truth(), truth()));
    assert_ne!(yes(), no());
}

// mitra: evidence for standard::a_number_is_itself
#[test]
fn a_number_is_itself() {
    for x in [i64::MIN, -1, 0, 42, i64::MAX] {
        assert_eq!(value_of(&n(x)), Some(number(x)));
        assert_eq!(type_of(&number(x)), whole());
    }
}

// mitra: evidence for tests::two_and_two
#[test]
fn two_and_two() {
    assert_eq!(
        value_of(&Apply(add(&whole()), vec![n(2), n(2)])),
        Some(number(4))
    );
    assert_eq!(
        value_of(&Apply(sub(&whole()), vec![n(2), n(5)])),
        Some(number(-3))
    );
    assert_eq!(
        value_of(&Apply(mul(&whole()), vec![n(6), n(7)])),
        Some(number(42))
    );
}

// mitra: evidence for tests::one_is_less_than_two
#[test]
fn one_is_less_than_two() {
    assert_eq!(
        value_of(&Apply(less(&whole()), vec![n(1), n(2)])),
        Some(yes())
    );
    assert_eq!(
        value_of(&Apply(less(&whole()), vec![n(2), n(2)])),
        Some(no())
    );
    let both = Apply(and(), vec![Given(yes()), Apply(not(), vec![Given(no())])]);
    assert_eq!(value_of(&both), Some(yes()));
}

// mitra: evidence for tests::equal_is_on_one_type
#[test]
fn equal_is_on_one_type() {
    assert_eq!(
        value_of(&Apply(equal(&whole()), vec![n(1), n(1)])),
        Some(yes())
    );
    assert_eq!(
        value_of(&Apply(equal(&truth()), vec![Given(yes()), Given(no())])),
        Some(no())
    );
    let mixed = Apply(equal(&whole()), vec![Given(yes()), n(1)]);
    assert_eq!((check(&mixed), value_of(&mixed)), (None, None));
}

// mitra: evidence for standard::truth_is_not_a_number
#[test]
fn truth_is_not_a_number() {
    assert_eq!(check(&Apply(add(&whole()), vec![Given(yes()), n(1)])), None);
}

// mitra: evidence for tests::an_operation_it_does_not_know_has_no_value
#[test]
fn an_operation_it_does_not_know_has_no_value() {
    let double = Operation {
        name: "double".to_string(),
        takes: vec![whole()],
        gives: whole(),
    };
    let twice = Apply(double, vec![n(2)]);
    assert_eq!((check(&twice), value_of(&twice)), (Some(whole()), None));
    // A standard name on other types is another operation.
    let odd = Operation {
        name: "+".to_string(),
        takes: vec![truth(), truth()],
        gives: truth(),
    };
    assert_eq!(
        value_of(&Apply(odd, vec![Given(yes()), Given(yes())])),
        None
    );
}

// mitra: evidence for tests::a_number_too_large_is_no_value
#[test]
fn a_number_too_large_is_no_value() {
    assert_eq!(
        value_of(&Apply(mul(&whole()), vec![n(i64::MAX), n(2)])),
        None
    );
    assert_eq!(
        value_of(&Apply(add(&whole()), vec![n(i64::MAX), n(1)])),
        None
    );
    assert_eq!(
        value_of(&Apply(sub(&whole()), vec![n(i64::MIN), n(1)])),
        None
    );
}

// mitra: evidence for tests::if_chooses
#[test]
fn if_chooses() {
    let pick = |test: Value| Apply(choose(&whole()), vec![Given(test), n(1), n(2)]);
    assert_eq!(value_of(&pick(yes())), Some(number(1)));
    assert_eq!(value_of(&pick(no())), Some(number(2)));
    let mixed = Apply(choose(&whole()), vec![Given(yes()), n(1), Given(no())]);
    assert_eq!(check(&mixed), None);
}

// mitra: evidence for tests::if_gives_the_branch_it_takes
#[test]
fn if_gives_the_branch_it_takes() {
    let too_large = Apply(mul(&whole()), vec![n(i64::MAX), n(2)]);
    let pick = |test: Value| Apply(choose(&whole()), vec![Given(test), n(1), too_large.clone()]);
    assert_eq!(check(&pick(yes())), Some(whole()));
    assert_eq!(value_of(&pick(yes())), Some(number(1)));
    assert_eq!(value_of(&pick(no())), None);
    // A test with no value is no value.
    let unknown = Apply(
        choose(&whole()),
        vec![Input("t".to_string(), truth()), n(1), n(2)],
    );
    assert_eq!(value_of(&unknown), None);
}

// mitra: evidence for tests::a_missing_value_is_no_sum
#[test]
fn a_missing_value_is_no_sum() {
    let sum = Apply(add(&whole()), vec![n(1), Input("x".to_string(), whole())]);
    assert_eq!(value_of(&sum), None);
}

// mitra: evidence for tests::a_field_reads_what_was_made
#[test]
fn a_field_reads_what_was_made() {
    let table = Table {
        shapes: vec![point()],
    };
    let made = Apply(make(&point()), vec![n(1), Given(yes())]);
    assert_eq!(check(&made).unwrap().name, "Point");
    let first = Apply(field(&point(), 0).unwrap(), vec![made.clone()]);
    let second = Apply(field(&point(), 1).unwrap(), vec![made.clone()]);
    assert_eq!(eval(&first, &[], &table), Some(number(1)));
    assert_eq!(eval(&second, &[], &table), Some(yes()));
    // Two records made of the same are equal.
    let same = Apply(equal(&check(&made).unwrap()), vec![made.clone(), made]);
    assert_eq!(eval(&same, &[], &table), Some(yes()));
}

// mitra: evidence for tests::no_field_past_the_last
#[test]
fn no_field_past_the_last() {
    assert_eq!(field(&point(), 2), None);
}

// mitra: evidence for tests::a_record_the_table_was_not_given_is_not_made
#[test]
fn a_record_the_table_was_not_given_is_not_made() {
    let made = Apply(make(&point()), vec![n(1), Given(yes())]);
    assert_eq!((check(&made).is_some(), value_of(&made)), (true, None));
}

fn q(n: i64, d: i64) -> Expression {
    Given(ratio(n, d).unwrap())
}

// mitra: evidence for tests::one_or_the_other
#[test]
fn one_or_the_other() {
    let either = |x: Value, y: Value| value_of(&Apply(or(), vec![Given(x), Given(y)]));
    assert_eq!(either(no(), yes()), Some(yes()));
    assert_eq!(either(yes(), no()), Some(yes()));
    assert_eq!(either(no(), no()), Some(no()));
}

// mitra: evidence for tests::the_four_orders
#[test]
fn the_four_orders() {
    let holds =
        |op: Operation, x: i64, y: i64| value_of(&Apply(op, vec![n(x), n(y)])) == Some(yes());
    let int = whole();
    assert!(holds(less(&int), 1, 2) && !holds(less(&int), 1, 1));
    assert!(holds(at_most(&int), 1, 1) && !holds(at_most(&int), 2, 1));
    assert!(holds(more(&int), 2, 1) && !holds(more(&int), 1, 1));
    assert!(holds(at_least(&int), 1, 1) && !holds(at_least(&int), 1, 2));
}

// mitra: evidence for standard::a_real_is_one_ratio
#[test]
fn a_real_is_one_ratio() {
    assert_eq!(ratio(2, 4), ratio(1, 2));
    assert_eq!(ratio(1, -2), ratio(-1, 2));
    assert_eq!(ratio(0, 5), ratio(0, 1));
    assert_eq!(type_of(&ratio(1, 2).unwrap()), real());
    let same = Apply(equal(&real()), vec![q(2, 4), q(1, 2)]);
    assert_eq!(value_of(&same), Some(yes()));
}

// mitra: evidence for standard::no_real_over_zero
#[test]
fn no_real_over_zero() {
    for x in [-1, 0, 1, i64::MAX] {
        assert_eq!(ratio(x, 0), None);
    }
}

// mitra: evidence for tests::a_third_and_a_sixth_are_a_half
#[test]
fn a_third_and_a_sixth_are_a_half() {
    let r = real();
    assert_eq!(
        value_of(&Apply(add(&r), vec![q(1, 3), q(1, 6)])),
        ratio(1, 2)
    );
    assert_eq!(
        value_of(&Apply(sub(&r), vec![q(1, 2), q(1, 3)])),
        ratio(1, 6)
    );
    assert_eq!(
        value_of(&Apply(mul(&r), vec![q(2, 3), q(3, 4)])),
        ratio(1, 2)
    );
    // A tenth and two tenths are three tenths, exactly.
    let sum = Apply(add(&r), vec![q(1, 10), q(2, 10)]);
    assert_eq!(
        value_of(&Apply(equal(&r), vec![sum, q(3, 10)])),
        Some(yes())
    );
}

// mitra: evidence for tests::a_third_is_less_than_a_half
#[test]
fn a_third_is_less_than_a_half() {
    let r = real();
    assert_eq!(
        value_of(&Apply(less(&r), vec![q(1, 3), q(1, 2)])),
        Some(yes())
    );
    assert_eq!(
        value_of(&Apply(at_least(&r), vec![q(1, 3), q(1, 2)])),
        Some(no())
    );
    assert_eq!(
        value_of(&Apply(at_most(&r), vec![q(-1, 2), q(-1, 3)])),
        Some(yes())
    );
}

// mitra: evidence for tests::within_is_a_distance
#[test]
fn within_is_a_distance() {
    let r = real();
    let near = |x: Expression, y: Expression| value_of(&Apply(within(&r), vec![x, y, q(2, 1)]));
    assert_eq!(near(q(9801, 100), q(981, 10)), Some(yes()));
    assert_eq!(near(q(981, 10), q(9801, 100)), Some(yes()));
    assert_eq!(near(q(4, 1), q(981, 10)), Some(no()));
    // Exactly as far as the tolerance is within it.
    assert_eq!(near(q(1, 1), q(3, 1)), Some(yes()));
    let whole_near = Apply(within(&whole()), vec![n(7), n(10), n(2)]);
    assert_eq!(value_of(&whole_near), Some(no()));
}

// mitra: evidence for standard::a_real_is_not_a_whole_number
#[test]
fn a_real_is_not_a_whole_number() {
    assert_eq!(check(&Apply(add(&real()), vec![n(1), q(1, 2)])), None);
    assert_eq!(check(&Apply(less(&whole()), vec![n(1), q(1, 2)])), None);
}

// mitra: evidence for tests::a_real_too_large_is_no_value
#[test]
fn a_real_too_large_is_no_value() {
    let big = q(i64::MAX, 1);
    assert_eq!(
        value_of(&Apply(mul(&real()), vec![big.clone(), q(2, 1)])),
        None
    );
    let small = Apply(mul(&real()), vec![q(1, i64::MAX), q(1, 2)]);
    assert_eq!(value_of(&small), None);
}
