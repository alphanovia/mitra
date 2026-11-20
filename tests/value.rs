//! The claims of `model/value.mitra`. The core has no operation of its
//! own, so these bring theirs: a type `A`, a type `B`, and one thing
//! that applies `swap` and nothing else.

use mitra_core::value::Expression::*;
use mitra_core::value::*;

fn of(name: &str) -> Type {
    Type {
        name: name.to_string(),
    }
}
fn value(n: i64, name: &str) -> Value {
    Value {
        quantity: Quantity::Whole(n),
        of: of(name),
    }
}
fn operation(name: &str, takes: &[&str], gives: &str) -> Operation {
    Operation {
        name: name.to_string(),
        takes: takes.iter().map(|t| of(t)).collect(),
        gives: of(gives),
    }
}
fn given(name: &str, v: Value) -> Vec<(String, Value)> {
    vec![(name.to_string(), v)]
}

/// Applies `swap`, from `A` to `B`, and knows no other operation.
struct OnlySwap;
impl Applies for OnlySwap {
    fn apply(&self, op: &Operation, args: &[Option<Value>]) -> Option<Quantity> {
        match op == &operation("swap", &["A"], "B") {
            true => Some(args[0].as_ref()?.quantity.clone()),
            false => None,
        }
    }
}
/// Applies nothing.
struct Nothing;
impl Applies for Nothing {
    fn apply(&self, _: &Operation, _: &[Option<Value>]) -> Option<Quantity> {
        None
    }
}

/// A few expressions, some that check and some that do not.
fn some() -> Vec<Expression> {
    let swap = operation("swap", &["A"], "B");
    vec![
        Given(value(7, "A")),
        Input("x".to_string(), of("A")),
        Apply(swap.clone(), vec![Given(value(1, "A"))]),
        Apply(swap.clone(), vec![Given(value(1, "B"))]),
        Apply(swap.clone(), vec![]),
        Apply(swap.clone(), vec![Apply(swap, vec![Given(value(1, "A"))])]),
        Apply(operation("other", &["A"], "A"), vec![Given(value(1, "A"))]),
    ]
}

// mitra: evidence for value::same_value
#[test]
fn same_value() {
    for e in some() {
        assert_eq!(eval(&e, &[], &OnlySwap), eval(&e, &[], &OnlySwap));
    }
}

// mitra: evidence for value::given_is_itself
#[test]
fn given_is_itself() {
    for v in [value(i64::MIN, "A"), value(0, "B"), value(42, "A")] {
        assert_eq!(eval(&Given(v.clone()), &[], &Nothing), Some(v));
    }
}

// mitra: evidence for value::input_is_what_was_given
#[test]
fn input_is_what_was_given() {
    let x = Input("x".to_string(), of("A"));
    let v = value(3, "A");
    assert_eq!(eval(&x, &given("x", v.clone()), &Nothing), Some(v));
    assert_eq!(eval(&x, &[], &Nothing), None);
}

// mitra: evidence for value::checked_gives_its_type
#[test]
fn checked_gives_its_type() {
    let mut checked = 0;
    for e in some() {
        if let (Some(t), Some(v)) = (check(&e), eval(&e, &[], &OnlySwap)) {
            assert_eq!(type_of(&v), t, "{e:?}");
            checked += 1;
        }
    }
    assert!(checked >= 2);
    // What does not check has no value either, whoever applies it.
    for e in some() {
        if check(&e).is_none() {
            assert_eq!(eval(&e, &[], &OnlySwap), None, "{e:?}");
        }
    }
}

// mitra: evidence for value::an_operation_takes_its_types
#[test]
fn an_operation_takes_its_types() {
    let f = operation("f", &["A"], "B");
    assert_eq!(
        check(&Apply(f.clone(), vec![Given(value(1, "A"))])),
        Some(of("B"))
    );
    assert_eq!(check(&Apply(f.clone(), vec![Given(value(1, "B"))])), None);
    assert_eq!(check(&Apply(f, vec![])), None);
}

// mitra: evidence for value::an_input_of_another_type_is_no_value
#[test]
fn an_input_of_another_type_is_no_value() {
    let x = Input("x".to_string(), of("A"));
    assert_eq!(eval(&x, &given("x", value(3, "B")), &Nothing), None);
}

// mitra: evidence for value::a_name_means_nothing_here
#[test]
fn a_name_means_nothing_here() {
    // `+` on a type `A` checks, and has a value only if someone applies it.
    let plus = Apply(operation("+", &["A"], "A"), vec![Given(value(1, "A"))]);
    assert_eq!(check(&plus), Some(of("A")));
    assert_eq!(eval(&plus, &[], &Nothing), None);
    assert_eq!(eval(&plus, &[], &OnlySwap), None);
}
