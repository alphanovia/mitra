use super::*;
use crate::verdicts::Logic;

// One input per test, for Mitra: each `mitra: test` line gives the
// output the test checks.

// mitra: evidence for t_holds
#[test]
fn t_holds() {
    assert_eq!(Logic::Holds.as_str(), "holds");
}

// mitra: evidence for t_assuming
#[test]
fn t_assuming() {
    assert_eq!(Logic::Assuming.as_str(), "holds assuming");
}

// mitra: evidence for t_unverified
#[test]
fn t_unverified() {
    assert_eq!(Logic::Unverified.as_str(), "unverified");
}

// mitra: evidence for t_fails
#[test]
fn t_fails() {
    assert_eq!(Logic::Fails.as_str(), "fails");
}

// mitra: evidence for t_plus
#[test]
fn t_plus() {
    assert_eq!(
        binary("+", &Value::Int(2), &Value::Int(3)),
        Some(Value::Int(5))
    );
}

// mitra: evidence for t_plus_swapped
#[test]
fn t_plus_swapped() {
    assert_eq!(
        binary("+", &Value::Int(3), &Value::Int(2)),
        Some(Value::Int(5))
    );
}

// mitra: evidence for t_plus_overflow
#[test]
fn t_plus_overflow() {
    assert_eq!(
        binary("+", &Value::Int(9223372036854775807), &Value::Int(1)),
        None
    );
}

// mitra: evidence for t_plus_overflow_swapped
#[test]
fn t_plus_overflow_swapped() {
    assert_eq!(
        binary("+", &Value::Int(1), &Value::Int(9223372036854775807)),
        None
    );
}

// mitra: evidence for t_plus_mixed
#[test]
fn t_plus_mixed() {
    assert_eq!(binary("+", &Value::Int(2), &Value::Bool(true)), None);
}

// mitra: evidence for t_plus_mixed_swapped
#[test]
fn t_plus_mixed_swapped() {
    assert_eq!(binary("+", &Value::Bool(true), &Value::Int(2)), None);
}

// mitra: evidence for t_plus_floats
#[test]
fn t_plus_floats() {
    assert_eq!(
        binary("+", &Value::Float(0.5), &Value::Float(0.25)),
        Some(Value::Float(0.75))
    );
}

// mitra: evidence for t_plus_floats_swapped
#[test]
fn t_plus_floats_swapped() {
    assert_eq!(
        binary("+", &Value::Float(0.25), &Value::Float(0.5)),
        Some(Value::Float(0.75))
    );
}

// mitra: evidence for t_minus
#[test]
fn t_minus() {
    assert_eq!(
        binary("-", &Value::Int(2), &Value::Int(3)),
        Some(Value::Int(-1))
    );
}

// mitra: evidence for t_plus_negated
#[test]
fn t_plus_negated() {
    assert_eq!(
        binary("+", &Value::Int(2), &Value::Int(-3)),
        Some(Value::Int(-1))
    );
}

// mitra: evidence for t_times
#[test]
fn t_times() {
    assert_eq!(
        binary("*", &Value::Int(4), &Value::Int(-3)),
        Some(Value::Int(-12))
    );
}

// mitra: evidence for t_times_swapped
#[test]
fn t_times_swapped() {
    assert_eq!(
        binary("*", &Value::Int(-3), &Value::Int(4)),
        Some(Value::Int(-12))
    );
}

// mitra: evidence for t_divide
#[test]
fn t_divide() {
    assert_eq!(
        binary("/", &Value::Int(-12), &Value::Int(-3)),
        Some(Value::Int(4))
    );
}

// mitra: evidence for t_divide_towards_zero
#[test]
fn t_divide_towards_zero() {
    assert_eq!(
        binary("/", &Value::Int(-7), &Value::Int(2)),
        Some(Value::Int(-3))
    );
}

// mitra: evidence for t_divide_by_zero
#[test]
fn t_divide_by_zero() {
    assert_eq!(binary("/", &Value::Int(1), &Value::Int(0)), None);
}

// mitra: evidence for t_divide_floats
#[test]
fn t_divide_floats() {
    assert_eq!(
        binary("/", &Value::Float(1.0), &Value::Float(4.0)),
        Some(Value::Float(0.25))
    );
}

// mitra: evidence for t_equal
#[test]
fn t_equal() {
    assert_eq!(
        binary("==", &Value::Int(2), &Value::Int(2)),
        Some(Value::Bool(true))
    );
}

// mitra: evidence for t_different
#[test]
fn t_different() {
    assert_eq!(
        binary("!=", &Value::Int(2), &Value::Int(2)),
        Some(Value::Bool(false))
    );
}

// mitra: evidence for t_equal_mixed
#[test]
fn t_equal_mixed() {
    assert_eq!(
        binary("==", &Value::Int(2), &Value::Str("2".into())),
        Some(Value::Bool(false))
    );
}

// mitra: evidence for t_different_mixed
#[test]
fn t_different_mixed() {
    assert_eq!(
        binary("!=", &Value::Int(2), &Value::Str("2".into())),
        Some(Value::Bool(true))
    );
}

// mitra: evidence for t_less
#[test]
fn t_less() {
    assert_eq!(
        binary("<", &Value::Int(2), &Value::Int(3)),
        Some(Value::Bool(true))
    );
}

// mitra: evidence for t_and
#[test]
fn t_and() {
    assert_eq!(
        binary("&&", &Value::Bool(true), &Value::Bool(false)),
        Some(Value::Bool(false))
    );
}

// mitra: evidence for t_or
#[test]
fn t_or() {
    assert_eq!(
        binary("||", &Value::Bool(true), &Value::Bool(false)),
        Some(Value::Bool(true))
    );
}

// mitra: evidence for t_unknown_operator
#[test]
fn t_unknown_operator() {
    assert_eq!(binary("%", &Value::Int(2), &Value::Int(3)), None);
}

// mitra: evidence for t_not_true
#[test]
fn t_not_true() {
    assert_eq!(unary("!", &Value::Bool(true)), Some(Value::Bool(false)));
}

// mitra: evidence for t_not_false
#[test]
fn t_not_false() {
    assert_eq!(unary("!", &Value::Bool(false)), Some(Value::Bool(true)));
}

// mitra: evidence for t_negate
#[test]
fn t_negate() {
    assert_eq!(unary("-", &Value::Int(3)), Some(Value::Int(-3)));
}

// mitra: evidence for t_negate_back
#[test]
fn t_negate_back() {
    assert_eq!(unary("-", &Value::Int(-3)), Some(Value::Int(3)));
}

// mitra: evidence for t_negate_float
#[test]
fn t_negate_float() {
    assert_eq!(unary("-", &Value::Float(0.5)), Some(Value::Float(-0.5)));
}

// mitra: evidence for t_negate_string
#[test]
fn t_negate_string() {
    assert_eq!(unary("-", &Value::Str("a".into())), None);
}

fn ints(xs: &[i64]) -> Value {
    Value::set_of(xs.iter().map(|x| Value::Int(*x)).collect())
}

fn text(s: &str) -> Value {
    Value::Str(s.into())
}

fn map(pairs: &[(&str, i64)]) -> Value {
    let pairs = pairs.iter().map(|(k, v)| (text(k), Value::Int(*v)));
    Value::map_of(pairs.collect()).unwrap()
}

// mitra: evidence for t_in_set
#[test]
fn t_in_set() {
    assert_eq!(
        binary("in", &Value::Int(2), &ints(&[1, 2])),
        Some(Value::Bool(true))
    );
}

// mitra: evidence for t_in_list
#[test]
fn t_in_list() {
    assert_eq!(
        binary(
            "in",
            &Value::Int(3),
            &Value::List(vec![Value::Int(1), Value::Int(2)])
        ),
        Some(Value::Bool(false))
    );
}

// mitra: evidence for t_in_map
#[test]
fn t_in_map() {
    assert_eq!(
        binary("in", &text("a"), &map(&[("a", 1)])),
        Some(Value::Bool(true))
    );
}

// mitra: evidence for t_in_number
#[test]
fn t_in_number() {
    assert_eq!(binary("in", &Value::Int(1), &Value::Int(2)), None);
}

// mitra: evidence for t_union
#[test]
fn t_union() {
    assert_eq!(
        binary("|", &ints(&[1, 2]), &ints(&[2, 3])),
        Some(ints(&[1, 2, 3]))
    );
}

// mitra: evidence for t_union_swapped
#[test]
fn t_union_swapped() {
    assert_eq!(
        binary("|", &ints(&[2, 3]), &ints(&[1, 2])),
        Some(ints(&[1, 2, 3]))
    );
}

// mitra: evidence for t_intersection
#[test]
fn t_intersection() {
    assert_eq!(
        binary("&", &ints(&[1, 2]), &ints(&[2, 3])),
        Some(ints(&[2]))
    );
}

// mitra: evidence for t_apart
#[test]
fn t_apart() {
    assert_eq!(binary("&", &ints(&[1]), &ints(&[2, 3])), Some(ints(&[])));
}

// mitra: evidence for t_difference
#[test]
fn t_difference() {
    assert_eq!(
        binary("\\", &ints(&[1, 2]), &ints(&[2, 3])),
        Some(ints(&[1]))
    );
}

// mitra: evidence for t_map_update
#[test]
fn t_map_update() {
    assert_eq!(
        binary("|", &map(&[("a", 1), ("b", 2)]), &map(&[("a", 5)])),
        Some(map(&[("a", 5), ("b", 2)]))
    );
}

// mitra: evidence for t_map_remove
#[test]
fn t_map_remove() {
    assert_eq!(
        binary(
            "\\",
            &map(&[("a", 1), ("b", 2)]),
            &Value::set_of(vec![text("a"), text("z")])
        ),
        Some(map(&[("b", 2)]))
    );
}

// mitra: evidence for t_set_with_map
#[test]
fn t_set_with_map() {
    assert_eq!(binary("|", &ints(&[1]), &map(&[("a", 1)])), None);
}

// mitra: evidence for t_keys
#[test]
fn t_keys() {
    assert_eq!(
        unary("keys", &map(&[("a", 1), ("b", 2)])),
        Some(Value::set_of(vec![text("a"), text("b")]))
    );
}

// mitra: evidence for t_keys_of_a_set
#[test]
fn t_keys_of_a_set() {
    assert_eq!(unary("keys", &ints(&[1])), None);
}

// Sweeps: each runs a claim of operators.mitra on every input of a
// stated set. The comment says the set and how many inputs it is.

fn small() -> Vec<Value> {
    (-12..=12).map(Value::Int).collect()
}

fn sets() -> Vec<Value> {
    vec![ints(&[]), ints(&[1]), ints(&[1, 2]), ints(&[2, 3])]
}

// mitra: evidence for eval::twice_negated given v in -100..=100 sampled 201
#[test]
fn twice_negated() {
    for v in (-100..=100).map(Value::Int) {
        let n = unary("-", &v).unwrap();
        assert_eq!(unary("-", &n), Some(v));
    }
}

// mitra: evidence for eval::plus_commutes given x in -12..=12, y in -12..=12 sampled 625
#[test]
fn plus_commutes() {
    for (x, y) in small()
        .iter()
        .flat_map(|x| small().into_iter().map(move |y| (x.clone(), y)))
    {
        assert_eq!(binary("+", &x, &y), binary("+", &y, &x));
    }
}

// mitra: evidence for eval::times_commutes given x in -12..=12, y in -12..=12 sampled 625
#[test]
fn times_commutes() {
    for (x, y) in small()
        .iter()
        .flat_map(|x| small().into_iter().map(move |y| (x.clone(), y)))
    {
        assert_eq!(binary("*", &x, &y), binary("*", &y, &x));
    }
}

// mitra: evidence for eval::minus_is_plus_negated given x in -12..=12, y in -12..=12 sampled 625
#[test]
fn minus_is_plus_negated() {
    for (x, y) in small()
        .iter()
        .flat_map(|x| small().into_iter().map(move |y| (x.clone(), y)))
    {
        let n = unary("-", &y).unwrap();
        assert_eq!(binary("-", &x, &y), binary("+", &x, &n));
    }
}

// mitra: evidence for eval::different_is_not_equal given x in -5..=5 | {true, "a"}, y in -5..=5 | {true, "a"} sampled 169
#[test]
fn different_is_not_equal() {
    let mut all: Vec<Value> = (-5..=5).map(Value::Int).collect();
    all.extend([Value::Bool(true), text("a")]);
    for (x, y) in all.iter().flat_map(|x| all.iter().map(move |y| (x, y))) {
        let e = binary("==", x, y).unwrap();
        assert_eq!(binary("!=", x, y), unary("!", &e));
    }
}

// mitra: evidence for eval::union_commutes given x in {{}, {1}, {1, 2}, {2, 3}}, y in {{}, {1}, {1, 2}, {2, 3}} sampled 16
#[test]
fn union_commutes() {
    for (x, y) in sets()
        .iter()
        .flat_map(|x| sets().into_iter().map(move |y| (x.clone(), y)))
    {
        assert_eq!(binary("|", &x, &y), binary("|", &y, &x));
    }
}

// mitra: evidence for eval::difference_apart given x in {{}, {1}, {1, 2}, {2, 3}}, y in {{}, {1}, {1, 2}, {2, 3}} sampled 16
#[test]
fn difference_apart() {
    for (x, y) in sets()
        .iter()
        .flat_map(|x| sets().into_iter().map(move |y| (x.clone(), y)))
    {
        let d = binary("\\", &x, &y).unwrap();
        assert_eq!(binary("&", &d, &y), Some(ints(&[])));
    }
}

// mitra: evidence for eval::divides_back given x in -12..=12, y in -12..=12 \ {0} sampled 600
#[test]
fn divides_back() {
    let ys = small().into_iter().filter(|y| *y != Value::Int(0));
    for (x, y) in ys.flat_map(|y| small().into_iter().map(move |x| (x, y.clone()))) {
        let p = binary("*", &x, &y).unwrap();
        assert_eq!(binary("/", &p, &y), Some(x));
    }
}
