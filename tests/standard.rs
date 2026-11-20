//! The claims of `model/standard.mitra`.

use mitra_core::standard::*;
use mitra_core::value::Expression::*;
use mitra_core::value::*;

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

// mitra: evidence for standard::two_and_two
#[test]
fn two_and_two() {
    assert_eq!(value_of(&Apply(add(), vec![n(2), n(2)])), Some(number(4)));
    assert_eq!(value_of(&Apply(sub(), vec![n(2), n(5)])), Some(number(-3)));
    assert_eq!(value_of(&Apply(mul(), vec![n(6), n(7)])), Some(number(42)));
}

// mitra: evidence for standard::one_is_less_than_two
#[test]
fn one_is_less_than_two() {
    assert_eq!(value_of(&Apply(less(), vec![n(1), n(2)])), Some(yes()));
    assert_eq!(value_of(&Apply(less(), vec![n(2), n(2)])), Some(no()));
    let both = Apply(and(), vec![Given(yes()), Apply(not(), vec![Given(no())])]);
    assert_eq!(value_of(&both), Some(yes()));
}

// mitra: evidence for standard::equal_is_on_one_type
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
    assert_eq!(check(&Apply(add(), vec![Given(yes()), n(1)])), None);
}

// mitra: evidence for standard::an_operation_it_does_not_know_has_no_value
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

// mitra: evidence for standard::a_number_too_large_is_no_value
#[test]
fn a_number_too_large_is_no_value() {
    assert_eq!(value_of(&Apply(mul(), vec![n(i64::MAX), n(2)])), None);
    assert_eq!(value_of(&Apply(add(), vec![n(i64::MAX), n(1)])), None);
    assert_eq!(value_of(&Apply(sub(), vec![n(i64::MIN), n(1)])), None);
}

// mitra: evidence for standard::if_chooses
#[test]
fn if_chooses() {
    let pick = |test: Value| Apply(choose(&whole()), vec![Given(test), n(1), n(2)]);
    assert_eq!(value_of(&pick(yes())), Some(number(1)));
    assert_eq!(value_of(&pick(no())), Some(number(2)));
    let mixed = Apply(choose(&whole()), vec![Given(yes()), n(1), Given(no())]);
    assert_eq!(check(&mixed), None);
}

// mitra: evidence for standard::if_computes_both_branches
#[test]
fn if_computes_both_branches() {
    let too_large = Apply(mul(), vec![n(i64::MAX), n(2)]);
    let pick = Apply(choose(&whole()), vec![Given(yes()), n(1), too_large]);
    assert_eq!((check(&pick), value_of(&pick)), (Some(whole()), None));
}

// mitra: evidence for standard::a_field_reads_what_was_made
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

// mitra: evidence for standard::no_field_past_the_last
#[test]
fn no_field_past_the_last() {
    assert_eq!(field(&point(), 2), None);
}

// mitra: evidence for standard::a_record_the_table_was_not_given_is_not_made
#[test]
fn a_record_the_table_was_not_given_is_not_made() {
    let made = Apply(make(&point()), vec![n(1), Given(yes())]);
    assert_eq!((check(&made).is_some(), value_of(&made)), (true, None));
}
