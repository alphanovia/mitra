//! Finding 39 G (the user's go, 2026-11-26): an `until` that comes back
//! to values it had already carried can never end. The simulation
//! remembers what it carried, stops at once with no value, and says so:
//! A013 in agree, X016 in explain. A value known only in part is never
//! taken for one seen; a loop whose values never repeat still runs to
//! the step budget.

use mitra_lang::agree::{agree, Agreement, Settings};
use mitra_lang::check::check;
use mitra_lang::expression::{eval, Fault};
use mitra_lang::model::calls;
use mitra_lang::parse::parse;
use mitra_lang::simulation::{Budget, Exact, Record, Type, Undefined, Value};
use num_bigint::BigInt;
use std::collections::BTreeMap;
use std::sync::Arc;

fn findings(text: &str, settings: &Settings) -> Vec<String> {
    let m = parse(text).unwrap();
    assert_eq!(check(&m), vec![], "{text}");
    agree(&m, settings).1.iter().map(|f| f.to_string()).collect()
}

fn small() -> Settings {
    Settings {
        budget: Budget {
            steps: 1_000,
            ..Budget::default()
        },
        ..Settings::default()
    }
}

/// The halving mistake: the block gives back what it was given, so the
/// first turn carries the start again. Stopped at once, the claim and
/// the example unknown, with the loop, the input and the values said.
#[test]
fn a_loop_that_comes_back_stops_at_once() {
    let text = "subject s {\n    fn halve(n: integer) -> k: integer {\n        until(k: n, k <= 1) { k }\n    } where {\n        claim small: k <= 1;\n        examples { (4) -> 1 }\n    }\n}";
    // Within a budget of 1 turn: the repeat is seen before the budget.
    let one = Settings {
        budget: Budget {
            steps: 1,
            ..Budget::default()
        },
        ..Settings::default()
    };
    for settings in [Settings::default(), one] {
        let found = findings(text, &settings);
        assert_eq!(
            found,
            [
                "finding[A013]: claim `small`: unknown, at n = 4, the `until` in `s::halve` never ends: what it carries, 4, came back after 1 turn\n --> s::halve",
                "finding[A013]: example 0: unknown, at n = 4, the `until` in `s::halve` never ends: what it carries, 4, came back after 1 turn\n --> s::halve",
            ]
        );
    }
    // Where the condition holds at the start, the block is not run.
    let ends = text.replace("(4) -> 1", "(1) -> 1");
    let found = findings(&ends, &small());
    assert!(
        found.iter().all(|f| !f.contains("example 0")),
        "{found:?}"
    );
}

/// Values that come back after more than one turn, of a labelled tuple:
/// the turns between the two times are said.
#[test]
fn a_cycle_of_two_turns_is_said() {
    let text = "subject s {\n    fn swap(n: integer) -> k: integer {\n        until((a: n, b: 0), a == 7) { (b, a) }.a\n    } where {\n        examples { (3) -> 7 }\n    }\n}";
    assert_eq!(
        findings(text, &Settings::default()),
        ["finding[A013]: example 0: unknown, at n = 3, the `until` in `s::swap` never ends: what it carries, (a: 3, b: 0), came back after 2 turns\n --> s::swap"]
    );
}

/// A cycle the start is not on, seen once the loop is in it.
#[test]
fn a_cycle_after_some_turns_is_seen() {
    let text = "subject s {\n    fn settle(n: integer) -> k: integer {\n        until(k: n, k == 99) { if k <= 0 { 1 } else { k - 1 } }\n    } where {\n        examples { (5) -> 99 }\n    }\n}";
    assert_eq!(
        findings(text, &small()),
        ["finding[A013]: example 0: unknown, at n = 5, the `until` in `s::settle` never ends: what it carries, 1, came back after 2 turns\n --> s::settle"]
    );
}

/// A loop in a function called: the finding names the function whose
/// body holds it, not the one compared.
#[test]
fn the_loop_is_named_where_it_is() {
    let text = "subject s {\n    fn halve(n: integer) -> k: integer {\n        until(k: n, k <= 1) { k }\n    }\n    fn twice(n: integer) -> k: integer { halve(n) + halve(n) } where {\n        examples { (4) -> 2 }\n    }\n}";
    let found = findings(text, &Settings::default());
    assert_eq!(
        found,
        ["finding[A013]: example 0: unknown, at n = 4, the `until` in `s::halve` never ends: what it carries, 4, came back after 1 turn\n --> s::twice"]
    );
}

/// A counter hides the repeat: the loop runs to the budget, and is A007,
/// as before (the known limit).
#[test]
fn a_counter_runs_to_the_budget() {
    let text = "subject s {\n    fn count(n: integer) -> c: integer {\n        until((k: n, steps: 0), k <= 1) { (k, steps + 1) }.steps\n    } where {\n        examples { (4) -> 2 }\n    }\n}";
    assert_eq!(
        findings(text, &small()),
        ["finding[A007]: example 0: unknown, the step budget, 1000 turns of `until`, reached\n --> s::count"]
    );
    // With the step: it ends, and the example holds.
    let fixed = text.replace("(k, steps + 1)", "(k / 2, steps + 1)");
    assert_eq!(findings(&fixed, &small()), Vec::<String>::new());
}

/// A value known in part is never equal to one seen: the loop runs to
/// its budget, never said to be endless on what is not known.
#[test]
fn a_value_known_in_part_never_repeats() {
    let text = "subject S {\n    record { a: integer, b: integer }\n    fn keep(r: S) -> t: S {\n        until(t: r, t.a == 9) { t }\n    }\n}";
    let model = parse(text).unwrap();
    assert_eq!(check(&model), vec![], "{text}");
    let budget = Budget {
        steps: 50,
        ..Budget::default()
    };
    let exact = Exact {
        calls: Arc::new(calls(&model).unwrap().budget(budget)),
        log: Default::default(),
    };
    let body = model.parts[0].functions[0].body.clone().unwrap();
    let Type::Record(r) = &model.parts[0].functions[0].takes[0].1 else {
        panic!()
    };
    let r: Record = r.clone();
    let int = |n: i64| Value::Int(BigInt::from(n));
    let whole = Value::Record(r.clone(), vec![int(1), int(2)]);
    let part = Value::partial(r, vec![Some(int(1)), None]);
    let at = |v: &Value| BTreeMap::from([("r".to_string(), v.clone())]);
    assert!(matches!(
        eval(&exact, &at(&whole), &body),
        Err(Fault::Op(Undefined::EndlessRepeat { turns: 1, .. }))
    ));
    assert!(matches!(
        eval(&exact, &at(&part), &body),
        Err(Fault::Op(Undefined::Budget(_)))
    ));
    // Nor is a tuple that holds one.
    let text = "subject S {\n    record { a: integer, b: integer }\n    fn keep(r: S) -> t: S {\n        until((t: r, n: 0), t.a == 9) { (t, n) }.t\n    }\n}";
    let model = parse(text).unwrap();
    assert_eq!(check(&model), vec![], "{text}");
    let exact = Exact {
        calls: Arc::new(calls(&model).unwrap().budget(budget)),
        log: Default::default(),
    };
    let body = model.parts[0].functions[0].body.clone().unwrap();
    assert!(matches!(
        eval(&exact, &at(&whole), &body),
        Err(Fault::Op(Undefined::EndlessRepeat { turns: 1, .. }))
    ));
    assert!(matches!(
        eval(&exact, &at(&part), &body),
        Err(Fault::Op(Undefined::Budget(_)))
    ));
}

/// A claim that calls a function whose loop never ends, read at an
/// example's value: unknown and said, not false.
#[test]
fn a_claim_through_an_endless_loop_is_said() {
    let text = "subject s {\n    fn halve(n: integer) -> k: integer {\n        until(k: n, k <= 1) { k }\n    }\n    fn id(n: integer) -> m: integer where {\n        claim c: halve(m) <= 1;\n        examples { (4) -> 4 }\n    }\n}";
    let found = findings(text, &Settings::default());
    assert!(
        found.iter().any(|f| f.starts_with("finding[A013]: example 0 against claim `c`: unknown, at n = 4, the `until` in `s::halve` never ends")),
        "{found:?}"
    );
    assert!(!found.iter().any(|f| f.starts_with("finding[A003]")), "{found:?}");
}

/// What agree says of an endless loop is kept as such, never a fault:
/// a body with no value elsewhere still fails.
#[test]
fn an_endless_loop_is_not_a_failure() {
    let text = "subject s {\n    fn halve(n: integer) -> k: integer {\n        until(k: n, k <= 1) { k }\n    } where {\n        claim small: k <= 1;\n    }\n}";
    let m = parse(text).unwrap();
    let (agreed, _) = agree(&m, &Settings::default());
    assert!(agreed
        .iter()
        .any(|a| matches!(a.agreement, Agreement::Endless { .. })));
    assert!(!agreed
        .iter()
        .any(|a| matches!(a.agreement, Agreement::Fails { .. })));
}
