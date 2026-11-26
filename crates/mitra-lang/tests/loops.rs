//! Loops (finding 3, step 2b): `fold` and `until`, carrying one value
//! bare or a tuple with labels.

use mitra_lang::agree::{agree, Agreed, Agreement, Settings};
use mitra_lang::check::check;
use mitra_lang::parse::parse;
use mitra_lang::print::print;

const STATS: &str = "subject Stats {
    fn total(xs: [integer]) -> t: integer { xs.fold(sum: 0) { x -> sum + x } } where {
        examples { ([1, 2, 3]) -> 6, ([]) -> 0 }
    }
    fn count(xs: [integer]) -> c: (sum: integer, n: integer) {
        xs.fold((sum: 0, n: 0)) { x -> (sum + x, n + 1) }
    } where {
        claim counted: c.n == xs.len() && c.1 == xs.len();
        examples { ([1, 2]) -> (sum: 3, n: 2), ([]) -> (sum: 0, n: 0) }
    }
    fn halve(n: integer) -> s: integer {
        until((k: n, steps: 0), k <= 1) { (k / 2, steps + 1) }.steps
    } where {
        claim none: !(n <= 1) || s == 0;
        examples { (8) -> 3, (9) -> 3, (1) -> 0, (-4) -> 0 }
    }
}";

fn errors(text: &str) -> Vec<String> {
    check(&parse(text).unwrap_or_else(|e| panic!("{e}")))
        .iter()
        .map(|e| e.to_string())
        .collect()
}

fn refused(text: &str) -> String {
    parse(text)
        .map(|_| "parsed".to_string())
        .unwrap_or_else(|e| e.to_string())
}

fn agreed(text: &str) -> Vec<Agreed> {
    let model = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![]);
    agree(&model, &Settings::default()).0
}

/// A function `f(xs: [integer]) -> t: integer` with this body.
fn body(b: &str) -> String {
    format!("subject S {{\n    fn f(xs: [integer]) -> t: integer {{ {b} }}\n}}")
}

#[test]
fn a_loop_reads_back_and_its_examples_hold() {
    let model = parse(STATS).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![]);
    let printed = print(&model);
    assert!(
        printed.contains("xs.fold(sum: 0) { x -> sum + x }"),
        "{printed}"
    );
    assert!(
        printed.contains("xs.fold((sum: 0, n: 0)) { x -> (sum + x, n + 1) }"),
        "{printed}"
    );
    assert!(
        printed.contains("until((k: n, steps: 0), k <= 1) { (k / 2, steps + 1) }.steps"),
        "{printed}"
    );
    assert!(
        printed.contains("-> c: (sum: integer, n: integer)"),
        "{printed}"
    );
    assert!(printed.contains("([1, 2]) -> (sum: 3, n: 2)"), "{printed}");
    assert_eq!(parse(&printed).unwrap(), model);
    let out = agreed(STATS);
    let examples: Vec<&Agreed> = out
        .iter()
        .filter(|a| a.against.starts_with("example"))
        .collect();
    assert_eq!(examples.len(), 14, "{out:?}");
    assert!(
        examples.iter().all(|a| a.agreement == Agreement::Holds),
        "{out:?}"
    );
    // The claims hold at every point tried, none broken.
    assert!(
        !out.iter()
            .any(|a| matches!(a.agreement, Agreement::Fails { .. })),
        "{out:?}"
    );
}

#[test]
fn a_wrong_example_of_a_loop_is_found() {
    for (right, wrong) in [
        ("([1, 2, 3]) -> 6", "([1, 2, 3]) -> 5"),
        ("([1, 2]) -> (sum: 3, n: 2)", "([1, 2]) -> (sum: 3, n: 1)"),
        ("(8) -> 3", "(8) -> 4"),
        ("(1) -> 0", "(1) -> 1"),
    ] {
        let out = agreed(&STATS.replace(right, wrong));
        assert!(
            out.iter()
                .any(|a| matches!(a.agreement, Agreement::Fails { .. })),
            "{wrong}: {out:?}"
        );
    }
}

#[test]
fn a_broken_claim_of_a_loop_is_found() {
    let out = agreed(&STATS.replace("c.n == xs.len()", "c.n == xs.len() + 1"));
    assert!(
        out.iter().any(
            |a| a.against.contains("counted") && matches!(a.agreement, Agreement::Fails { .. })
        ),
        "{out:?}"
    );
}

#[test]
fn the_block_gives_what_the_loop_carries() {
    let out = errors(&body(
        "xs.fold((sum: 0, n: 0)) { x -> (sum + x, true) }.sum",
    ));
    assert_eq!(out.len(), 1, "{out:?}");
    assert!(
        out[0].starts_with("error[M005]: the body is not of the type of `t`"),
        "{out:?}"
    );
    assert!(
        out[0].contains("`(sum + x, true)` is (integer, bool)")
            && out[0].contains("its block gives the next values of what it carries, of the types `integer, integer`, in place"),
        "{out:?}"
    );
    // Three values for two.
    let out = errors(&body("xs.fold((sum: 0, n: 0)) { x -> (sum, n, x) }.sum"));
    assert!(out[0].starts_with("error[M005]"), "{out:?}");
    // `done` is a bool.
    let out = errors(&body("until(k: 1, k) { k + 1 }"));
    assert!(out[0].contains("`done` is a bool"), "{out:?}");
    // A sequence, not a text.
    let out = errors(
        "subject S {\n    fn f(s: [~char]) -> t: integer { s.fold(n: 0) { c -> n + 1 } }\n}",
    );
    assert!(out[0].contains("reads a sequence `xs`"), "{out:?}");
}

#[test]
fn one_value_carried_is_bare() {
    // No `.sum`: the loop gives the value itself.
    let out = errors(&body("xs.fold(sum: 0) { x -> sum + x }.sum"));
    assert!(
        out[0].contains("`.sum` reads a field of a record that has one"),
        "{out:?}"
    );
    // `(sum: 0)` is `sum: 0`: no tuple of one value.
    let model = parse(&body("xs.fold((sum: 0)) { x -> sum + x }")).unwrap();
    assert!(
        print(&model).contains("xs.fold(sum: 0) { x -> sum + x }"),
        "{}",
        print(&model)
    );
    assert_eq!(check(&model), vec![]);
}

#[test]
fn a_tuple_with_labels_is_read_by_label_or_place() {
    let text = "subject S {
    fn f(a: integer) -> b: bool { let p = (x: a, y: true); p.x == p.0 && p.y == p.1 } where {
        examples { (3) -> true }
    }
}";
    let out = agreed(text);
    assert!(
        out.iter().all(|a| a.agreement == Agreement::Holds),
        "{out:?}"
    );
    // No place past its last value.
    let out = errors(&text.replace("p.y == p.1", "p.y == p.2"));
    assert!(out[0].starts_with("error[M005]"), "{out:?}");
    // One label each, none twice.
    assert!(
        refused(&body("xs.fold((sum: 0, sum: 0)) { x -> (sum, sum) }.sum"))
            .contains("error[P003]: the label `sum` is given twice")
    );
    assert!(
        refused("subject S { fn f(a: (x: integer, bool)) -> b: bool { true } }")
            .contains("a label for each value, or none")
    );
}

#[test]
fn a_label_hides_a_name_in_the_block_only() {
    // `sum: sum` starts from the argument; inside, `sum` is what is carried.
    let text = "subject S {
    fn f(xs: [integer], sum: integer) -> t: integer { xs.fold(sum: sum) { x -> sum + x } } where {
        examples { ([1, 2], 10) -> 13 }
    }
}";
    let out = agreed(text);
    assert!(
        out.iter().all(|a| a.agreement == Agreement::Holds),
        "{out:?}"
    );
}

#[test]
fn until_stops_before_its_block_when_done_holds() {
    // The block would divide by zero: never run, as `done` holds at once.
    let text = "subject S {
    fn f(a: integer) -> b: integer { until(k: a, true) { k / 0 } } where {
        examples { (3) -> 3 }
    }
}";
    let out = agreed(text);
    assert!(
        out.iter().all(|a| a.agreement == Agreement::Holds),
        "{out:?}"
    );
    // Run once, it has no value: the example cannot hold.
    let out = agreed(&text.replace("until(k: a, true)", "until(k: a, k == 0)"));
    assert!(
        !out.iter().any(|a| a.agreement == Agreement::Holds),
        "{out:?}"
    );
}

#[test]
fn a_loop_past_the_step_budget_says_so() {
    let text = "subject S {
    fn f(a: integer) -> b: integer { until(k: a, false) { k + 1 } } where {
        examples { (1) -> 1 }
    }
}";
    let out = agreed(text);
    assert!(
        out.iter().any(|a| matches!(&a.agreement, Agreement::Wrong { code, why } if code.to_string() == "A016" && why.contains("the step budget, 1000000 turns of `until`, reached"))),
        "{out:?}"
    );
}

#[test]
fn bounds_never_take_a_loop_for_its_start() {
    // By bounds, a loop is any value of what it carries: a claim true
    // at the start only is broken at a point, never held.
    let text = "subject S {
    fn total(xs: [integer]) -> t: integer { xs.fold(sum: 0) { x -> sum + x } } where {
        claim zero: t == 0;
    }
    fn first(n: integer) -> s: integer { until(k: 0, k >= n) { k + 1 } } where {
        claim start: s == 0;
    }
}";
    let out = agreed(text);
    for claim in ["claim `zero`", "claim `start`"] {
        assert!(
            out.iter()
                .any(|a| a.against == claim && matches!(a.agreement, Agreement::Fails { .. })),
            "{claim}: {out:?}"
        );
    }
}

#[test]
fn a_loop_may_have_no_value_by_bounds() {
    // Its block divides by zero at once: by bounds, a loop may give no
    // value, so `s == s` is not held for every `a`.
    let text = "subject S {
    fn f(a: integer) -> s: integer { until(k: a, k == 0) { k / 0 } } where {
        claim same: s == s;
    }
}";
    let out = agreed(text);
    assert!(
        out.iter()
            .any(|a| a.against == "claim `same`" && a.agreement != Agreement::Holds),
        "{out:?}"
    );
}

#[test]
fn one_value_carried_keeps_its_type() {
    let out = errors(&body("xs.fold(sum: 0) { x -> true }"));
    assert!(out[0].starts_with("error[M005]"), "{out:?}");
    let out = errors(&body("until(k: 0, k > 3) { k > 1 }"));
    assert!(out[0].starts_with("error[M005]"), "{out:?}");
}

#[test]
fn bounds_of_a_loop_cover_what_it_gives() {
    use mitra_lang::expression::eval;
    use mitra_lang::simulation::{covers, Bounds, Exact, Ranges, Type, Value};
    use num_bigint::BigInt;
    use std::collections::BTreeMap;
    for body in [
        "until(k: a, k == 0) { k / 0 }",
        "until(k: a, k <= 0) { k - 1 }",
        "[a, 2].fold(sum: 0) { x -> sum + x / a }",
        "[a, a].fold((s: 0, n: 0)) { x -> (s + x, n + 1) }.n",
    ] {
        let text = format!("subject S {{ fn f(a: integer) -> s: integer {{ {body} }} }}");
        let model = parse(&text).unwrap_or_else(|e| panic!("{e}"));
        let e = model.parts[0].functions[0].body.clone().unwrap();
        for a in [-2, 0, 3] {
            let v = Value::Int(BigInt::from(a));
            let exact = eval(
                &Exact::default(),
                &BTreeMap::from([("a".to_string(), v.clone())]),
                &e,
            );
            let bounds = eval(
                &Ranges::default(),
                &BTreeMap::from([("a".to_string(), Bounds::exactly(&v))]),
                &e,
            );
            assert!(
                covers(&Type::Int, &bounds, &exact),
                "{body} at {a}: {bounds:?} vs {exact:?}"
            );
        }
    }
}
