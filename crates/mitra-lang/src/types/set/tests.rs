use super::*;

// One input per test, for Mitra: each `mitra: test` line gives the
// output the test checks.

// mitra: evidence for t_quote_plain
#[test]
fn t_quote_plain() {
    assert_eq!(quote("a b"), "\"a b\"");
}

// mitra: evidence for t_quote_empty
#[test]
fn t_quote_empty() {
    assert_eq!(quote(""), "\"\"");
}

// mitra: evidence for t_quote_escapes
#[test]
fn t_quote_escapes() {
    assert_eq!(quote("a\"b\\c\nd\te"), "\"a\\\"b\\\\c\\nd\\te\"");
}

#[test]
fn algebra() {
    let small = Set::interval(0, 100);
    assert!(small.is_subset(&Set::int()));
    assert_eq!(
        Set::int().union(&Set::bool()).complement(),
        Set::string().union(&Set::float()).union(&Set {
            lists: Set::any().lists,
            composites: Composites {
                rest: true,
                shapes: BTreeMap::new(),
            },
            ..Set::empty()
        })
    );
    assert_eq!(
        small.minus(&Set::point(&Value::Int(5))).count(),
        Some(100.0)
    );
    assert_eq!(
        Set::interval(1, 1).union(&Set::interval(2, 3)),
        Set::interval(1, 3)
    );
    assert_eq!(Set::bool().count(), Some(2.0));
    assert_eq!(Set::string().count(), None);
}

#[test]
fn display() {
    let show = |s: Set| s.to_string();
    assert_eq!(show(Set::any()), "Any");
    assert_eq!(show(Set::empty()), "{}");
    assert_eq!(show(Set::bool()), "Bool");
    assert_eq!(show(Set::point(&Value::Bool(true))), "{true}");
    assert_eq!(
        show(
            Set::interval(0, 4)
                .union(&Set::interval(6, 100))
                .union(&Set::string())
        ),
        "0..=4 | 6..=100 | String"
    );
    assert_eq!(
        show(Set::point(&Value::Int(1)).union(&Set::point(&Value::Int(42)))),
        "{1, 42}"
    );
    assert_eq!(
        show(Set::int().minus(&Set::point(&Value::Int(5)))),
        "Int \\ {5}"
    );
    assert_eq!(
        show(Set::any().minus(&Set::point(&Value::Int(5)))),
        "Any \\ {5}"
    );
    // Found by the grammar fuzzer: the part after `\` needs parentheses.
    assert_eq!(
        show(Set::any().minus(&Set::interval(-3, 2).union(&Set::string()))),
        "Any \\ (-3..=2 | String)"
    );
}
