//! The claims of `model/function.mitra`, and the golden of `double`:
//! the same toy as `examples/double.rs`.

use mitra_core::claim::Claim;
use mitra_core::function::*;
use mitra_core::model::*;
use mitra_core::standard::{add, at_least, mul, number, truth, whole, Exact, Table};
use mitra_core::value::Expression::{Apply, Given, Input};
use mitra_core::value::{eval, Operation};

type Expression = mitra_core::value::Expression<Exact>;

/// The settings these are recorded at.
const S: Settings = Settings {
    weight: 0.2,
    statement: 0.5,
    evidence: 0.9,
    tie: 0.5,
    completeness: 0.6,
    reliability: 1.0,
};

fn n() -> Expression {
    Input("n".to_string(), whole())
}
fn int_to_int(name: &str) -> Operation {
    Operation {
        name: name.to_string(),
        takes: vec![whole()],
        gives: whole(),
    }
}
/// `fn double(n: Int) -> Int { n * 2 }`, tied to its code.
fn double(uniform: bool) -> Function {
    Function {
        operation: int_to_int("double"),
        inputs: vec!["n".to_string()],
        body: Some(Apply(mul(&whole()), vec![n(), Given(number(2))])),
        uniform,
        claims: vec![],
        at: Some(Place::at(&["src/lib.rs", "double"])),
    }
}
fn run(f: &Function, at: i64, gave: i64) -> Run {
    Run {
        operation: f.operation.clone(),
        args: vec![number(at)],
        result: number(gave),
    }
}
fn test_of(f: &Function, at: i64, gave: i64) -> Option<Evidence> {
    let table = Table::default();
    let known = Known {
        table: &table,
        helpers: &[],
        runs: &[],
    };
    let by = Voice::named(&format!("machine:cargo test at {at}"));
    tested(f, &run(f, at, gave), &by, &known)
}
/// The model after these runs, with `uniform` reviewed or not.
fn after(f: &Function, runs: &[(i64, i64)], reviewed: bool) -> Model {
    let mut m = placed(&Model::empty(S), f);
    for (at, gave) in runs {
        m.evidence.push(test_of(f, *at, *gave).unwrap());
    }
    if let (true, Some(hypothesis)) = (reviewed, uniformity(f)) {
        m.evidence.push(Evidence {
            piece: Piece::named("review of uniform"),
            about: hypothesis,
            value: yes(),
            by: Voice::named("agent:reviewer"),
        });
    }
    m
}
fn read(m: &Model, x: &str) -> f64 {
    projected(opinion(m, &Piece::named(x)), S.weight)
}
fn near(x: f64, y: f64) -> bool {
    (x - y).abs() < 0.005
}

// mitra: evidence for functions::a_body_implies_correct
#[test]
fn a_body_implies_correct() {
    let f = double(false);
    let general = correctness(&f).unwrap();
    assert_eq!(general.piece, Piece::named("double::correct"));
    assert!(mitra_core::claim::reads(&general));
    let bare = Function { body: None, ..f };
    assert_eq!(correctness(&bare), None);
}

// mitra: evidence for functions::uniform_only_when_asked
#[test]
fn uniform_is_asked_for() {
    assert_eq!(uniformity(&double(false)), None);
    assert_eq!(
        uniformity(&double(true)),
        Some(Piece::named("double::uniform"))
    );
}

// mitra: evidence for functions::the_tie_lists_the_statements
#[test]
fn the_tie_lists_the_statements() {
    let at_least_n = Statement {
        piece: Piece::named("double::grows"),
        content: Claim {
            expression: Apply(at_least(&whole()), vec![called(&double(false)), n()]),
        },
    };
    let f = Function {
        claims: vec![at_least_n],
        ..double(false)
    };
    let names: Vec<String> = tie(&f)
        .unwrap()
        .statements
        .into_iter()
        .map(|p| p.name)
        .collect();
    assert_eq!(
        names,
        ["double::seen", "double::deterministic", "double::grows"]
    );
}

// mitra: evidence for functions::a_claim_reads_the_signature_only
#[test]
fn a_claim_reads_the_signature_only() {
    let f = double(false);
    let about = |name: &str, e: Expression| Statement {
        piece: Piece::named(name),
        content: Claim { expression: e },
    };
    let grows = Apply(at_least(&whole()), vec![called(&f), n()]);
    assert!(in_scope(&f, &about("double::grows", grows.clone())));
    assert!(!in_scope(&f, &about("triple::grows", grows)));
    let other = Apply(
        at_least(&whole()),
        vec![called(&f), Input("m".to_string(), whole())],
    );
    assert!(!in_scope(&f, &about("double::other", other)));
    let retyped = Apply(
        at_least(&whole()),
        vec![called(&f), Input("n".to_string(), truth())],
    );
    assert!(!in_scope(&f, &about("double::retyped", retyped)));
}

// mitra: evidence for functions::an_untied_function_is_named
#[test]
fn untied_is_owed() {
    let free = Function {
        at: None,
        ..double(true)
    };
    assert_eq!(untied(&[double(true), free.clone()]), ["double"]);
    assert_eq!(tie(&free), None);
    // Still supportable: a passing test says yes to `double::seen`.
    let e = test_of(&free, 3, 6).unwrap();
    assert_eq!(e.about, Piece::named("double::seen"));
    let m = Model {
        evidence: vec![e],
        ..placed(&Model::empty(S), &free)
    };
    assert!(m.ties.is_empty());
    assert!(read(&m, "double::seen") > 0.9);
}

/// A helper is a definition: `fn twice(n) { n + n }` is applied by its
/// body, and nothing is believed of it.
// mitra: evidence for functions::a_helper_is_its_body
#[test]
fn a_helper_is_its_body() {
    let twice = Helper {
        operation: int_to_int("twice"),
        inputs: vec!["n".to_string()],
        body: Apply(add(&whole()), vec![n(), n()]),
    };
    let table = Table::default();
    let helpers = [twice.clone()];
    let known = Known {
        table: &table,
        helpers: &helpers,
        runs: &[],
    };
    let call = Apply(twice.operation.clone(), vec![Given(number(21))]);
    assert_eq!(eval(&call, &[], &known), Some(number(42)));
    assert_eq!(eval(&call, &[], &table), None);
    // A function whose body calls the helper is tested through it.
    let f = Function {
        body: Some(Apply(twice.operation, vec![n()])),
        ..double(false)
    };
    let by = Voice::named("machine:cargo");
    let pass = tested(&f, &run(&f, 3, 6), &by, &known).unwrap();
    assert_eq!(pass.value, yes());
}

// mitra: evidence for functions::a_pass_is_about_what_was_seen
// mitra: evidence for functions::a_failure_is_about_correct
#[test]
fn a_test_is_a_measurement() {
    let f = double(true);
    let pass = test_of(&f, 3, 6).unwrap();
    assert_eq!(
        (pass.value, pass.about),
        (yes(), Piece::named("whole: double::tie"))
    );
    let fail = test_of(&f, 7, 15).unwrap();
    assert_eq!(
        (fail.value, fail.about),
        (no(), Piece::named("double::correct"))
    );
    // A run of something else says nothing.
    let other = Run {
        operation: int_to_int("triple"),
        ..run(&f, 3, 6)
    };
    let table = Table::default();
    let known = Known {
        table: &table,
        helpers: &[],
        runs: &[],
    };
    assert_eq!(
        tested(&f, &other, &Voice::named("machine:cargo"), &known),
        None
    );
}

/// The golden. One test at 3 passes. Without `uniform`, nothing leads
/// from it to `double::correct`: no belief at all. With `uniform`
/// reviewed, `double::correct` is believed as all of what was seen and
/// `uniform` together.
// mitra: evidence for tests::correct_only_through_uniform
#[test]
fn correct_only_through_uniform() {
    let without = after(&double(false), &[(3, 6)], false);
    let o = opinion(&without, &Piece::named("double::correct"));
    assert_eq!((o.r, o.s), (0.0, 0.0));
    assert!(near(read(&without, "double::seen"), 0.91));

    let unreviewed = after(&double(true), &[(3, 6)], false);
    assert!(near(read(&unreviewed, "double::correct"), 0.35));

    let with = after(&double(true), &[(3, 6)], true);
    assert!(near(read(&with, "double::correct"), 0.63));
    assert!(near(read(&with, "double::uniform"), 0.91));
    assert!(read(&with, "double::correct") < read(&with, "double::uniform"));

    // Four tests: more is seen, and the claim reads closer to `uniform`.
    let four = after(&double(true), &[(1, 2), (2, 4), (3, 6), (4, 8)], true);
    assert!(near(read(&four, "double::correct"), 0.82));
}

/// The golden, then a run at 7 gives 15. After one passing test the no
/// is shared equally between `uniform`, what was seen and the tie.
/// After four, what was seen stands and `uniform` takes the blame.
// mitra: evidence for tests::a_bug_blames_uniform
#[test]
fn a_bug_blames_uniform() {
    let one = after(&double(true), &[(3, 6), (7, 15)], true);
    assert!(near(read(&one, "double::uniform"), 0.85));
    assert!(near(read(&one, "double::seen"), 0.85));
    let suspects = &conflicts(&one)[0];
    assert_eq!(suspects[1], Piece::named("double::uniform"));

    let four = after(
        &double(true),
        &[(1, 2), (2, 4), (3, 6), (4, 8), (7, 15)],
        true,
    );
    assert!(near(read(&four, "double::uniform"), 0.79));
    assert!(near(read(&four, "double::seen"), 0.96));
    assert!(near(read(&four, "double::tie"), 0.96));
    let before = after(&double(true), &[(1, 2), (2, 4), (3, 6), (4, 8)], true);
    assert!(read(&four, "double::correct") < read(&before, "double::correct"));
}
