//! The claims of `model/function.mitra`, and the golden of `double`:
//! the same toy as `examples/double.rs`.

use mitra_core::claim::Claim;
use mitra_core::function::*;
use mitra_core::model::*;
use mitra_core::standard::{add, at_least, mul, number, truth, whole, Exact, Table};
use mitra_core::value::Expression::{Apply, Given, Input};
use mitra_core::value::{eval, Operation};
use mitra_core::verdict::{overall, suspects, verdict_of, State};

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
    let mut m = Model::empty(S);
    for (at, gave) in runs {
        m = noted(&m, f, test_of(f, *at, *gave).unwrap()).unwrap();
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
    assert_eq!(names, ["double::correct", "double::grows"]);
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

// mitra: evidence for functions::a_test_is_about_an_instance
#[test]
fn a_test_is_about_an_instance() {
    let f = double(true);
    let at_three = Piece::named("double::correct @ n = Whole(3)");
    let pass = test_of(&f, 3, 6).unwrap();
    assert_eq!((pass.value, pass.about.clone()), (yes(), at_three.clone()));
    let fail = test_of(&f, 7, 15).unwrap();
    assert_eq!(fail.value, no());
    assert_eq!(fail.about, Piece::named("double::correct @ n = Whole(7)"));
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
    // Noted through the tie: the test is about the tie, the instance
    // and the tie's completeness together.
    let m = noted(&Model::empty(S), &f, pass).unwrap();
    assert_eq!(m.ties, [tie(&f).unwrap()]);
    assert_eq!(
        m.related[0].content.pieces,
        [
            Piece::named("double::correct"),
            at_three.clone(),
            Piece::named("double::uniform")
        ]
    );
    let through = &m.related[1].content;
    assert_eq!(m.evidence[0].about, through.pieces[0]);
    assert_eq!(
        parts(&m, through),
        [
            Piece::named("double::tie"),
            at_three,
            Piece::named("complete: double::tie")
        ]
    );
}

fn shown(m: &Model, x: &str) -> String {
    mitra_core::verdict::shown(&verdict_of(m, &Piece::named(x)), S.weight)
}
const AT_3: &str = "double::correct @ n = Whole(3)";
const AT_7: &str = "double::correct @ n = Whole(7)";

/// The golden. One test at 3 passes. Without `uniform`, nothing leads
/// from that instance to `double::correct`: it is unknown, and so is
/// the verdict. With `uniform` reviewed it is true, and as sure as
/// that one review.
// mitra: evidence for tests::correct_only_through_uniform
#[test]
fn correct_only_through_uniform() {
    let without = after(&double(false), &[(3, 6)], false);
    assert_eq!(shown(&without, "double::correct"), "unknown");
    assert_eq!(shown(&without, AT_3), "true 0.91");
    assert_eq!(overall(&without), State::Unknown);

    let unreviewed = after(&double(true), &[(3, 6)], false);
    assert_eq!(shown(&unreviewed, "double::correct"), "unknown");

    let with = after(&double(true), &[(3, 6)], true);
    assert_eq!(overall(&with), State::True);
    assert_eq!(shown(&with, "double::correct"), "true 0.91");
    assert_eq!(shown(&with, "double::uniform"), "true 0.91");
    assert_eq!(shown(&with, "double::tie"), "true 0.91");
    assert_eq!(shown(&with, "complete: double::tie"), "true 0.93");

    // A function with no tie is still tested, and nothing is read of
    // a tie.
    let free = Function {
        at: None,
        ..double(true)
    };
    let untied = after(&free, &[(3, 6)], true);
    assert!(untied.ties.is_empty());
    assert_eq!(shown(&untied, "double::correct"), "true 0.91");
}

/// More passing tests make the tie and each instance surer. The claim
/// stays as sure as the one review of `uniform`: every way to it goes
/// through that review.
// mitra: evidence for tests::more_tests_are_surer
#[test]
fn more_tests_are_surer() {
    let four = after(&double(true), &[(1, 2), (2, 4), (3, 6), (4, 8)], true);
    assert_eq!(overall(&four), State::True);
    assert_eq!(shown(&four, "double::tie"), "true 0.97");
    assert_eq!(shown(&four, AT_3), "true 0.95");
    assert_eq!(shown(&four, "double::correct"), "true 0.91");
    // A second review of `uniform` by another voice, and the claim is
    // surer.
    let mut twice = four.clone();
    twice.evidence.push(Evidence {
        piece: Piece::named("second review of uniform"),
        about: Piece::named("double::uniform"),
        value: yes(),
        by: Voice::named("human:reviewer"),
    });
    assert_eq!(shown(&twice, "double::correct"), "true 0.95");
}

/// With no `uniform`, a run at 7 that gives 15 refutes: the tie and
/// its completeness are true by the test at 3, so the claim is false
/// at 7, and false. Nothing is in conflict.
// mitra: evidence for tests::a_bug_refutes_correct
#[test]
fn a_bug_refutes_correct() {
    let m = after(&double(false), &[(3, 6), (7, 15)], false);
    assert_eq!(shown(&m, AT_3), "true 0.91");
    assert_eq!(shown(&m, AT_7), "false 0.91");
    assert_eq!(shown(&m, "double::correct"), "false 0.91");
    assert_eq!(overall(&m), State::False);
    assert!(suspects(&m).is_empty());
    // With no passing test the tie is not known true: the failure
    // names three suspects and refutes nothing yet.
    let alone = after(&double(false), &[(7, 15)], false);
    assert_eq!(shown(&alone, "double::correct"), "unknown");
    assert_eq!(suspects(&alone)[0].len(), 3);
}

/// With `uniform` reviewed, the same run is a conflict: by `uniform`
/// the claim is true at 7, and the run says it is not. What to give
/// up is listed, the lightest first: the run at 7 or the review, one
/// report each; the passing tests only all together.
// mitra: evidence for tests::a_bug_under_uniform_is_a_conflict
#[test]
fn a_bug_under_uniform_is_a_conflict() {
    let m = after(
        &double(true),
        &[(1, 2), (2, 4), (3, 6), (4, 8), (7, 15)],
        true,
    );
    assert_eq!(overall(&m), State::Conflict);
    assert_eq!(shown(&m, "double::correct"), "conflict");
    assert_eq!(shown(&m, "double::uniform"), "true 0.91");
    // Only what is contested is marked: the claim and the claim at 7.
    assert_eq!(shown(&m, AT_7), "conflict");
    assert_eq!(shown(&m, AT_3), "true 0.95");
    assert_eq!(shown(&m, "double::tie"), "true 0.97");
    let found = suspects(&m);
    let names =
        |group: &Vec<Piece>| -> Vec<String> { group.iter().map(|p| p.name.clone()).collect() };
    assert_eq!(found.len(), 3);
    assert_eq!(
        names(&found[0]),
        [format!("{AT_7} by machine:cargo test at 7")]
    );
    assert_eq!(names(&found[1]), ["review of uniform"]);
    assert_eq!(found[2].len(), 4);
    // Giving up the review leaves a refuted claim and no conflict.
    let mut given_up = m.clone();
    given_up
        .evidence
        .retain(|e| e.piece.name != "review of uniform");
    assert_eq!(overall(&given_up), State::False);
    assert_eq!(shown(&given_up, "double::correct"), "false 0.91");
}
