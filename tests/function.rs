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
        ["double::correct", "double::deterministic", "double::grows"]
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
    // Noted through the tie, with no `uniform`: the test is about the
    // tie, the instance and the tie's completeness together.
    let plain = double(false);
    let m = noted(&Model::empty(S), &plain, pass.clone()).unwrap();
    assert_eq!(m.ties, [tie(&plain).unwrap()]);
    let through = &m.related[0].content;
    assert_eq!(m.evidence[0].about, through.pieces[0]);
    assert_eq!(
        parts(&m, through),
        [
            Piece::named("double::tie"),
            at_three,
            Piece::named("complete: double::tie")
        ]
    );
    // With `uniform`, the same test is also evidence about
    // `double::correct` through the tie.
    let m = noted(&Model::empty(S), &f, pass).unwrap();
    assert_eq!(m.evidence.len(), 2);
    let general = m
        .related
        .iter()
        .find(|said| said.content.pieces[0] == m.evidence[0].about)
        .unwrap();
    assert_eq!(
        parts(&m, &general.content),
        [
            Piece::named("double::tie"),
            Piece::named("double::correct"),
            Piece::named("complete: double::tie")
        ]
    );
}

/// The golden. One test at 3 passes. Without `uniform`, nothing leads
/// from that instance to `double::correct`: no belief at all. With
/// `uniform`, the test counts for `double::correct` as far as
/// `uniform` is believed.
// mitra: evidence for tests::correct_only_through_uniform
#[test]
fn correct_only_through_uniform() {
    let at_three = "double::correct @ n = Whole(3)";
    let without = after(&double(false), &[(3, 6)], false);
    let o = opinion(&without, &Piece::named("double::correct"));
    assert_eq!((o.r, o.s), (0.0, 0.0));
    assert!(near(read(&without, at_three), 0.91));

    // `uniform` at one half halves the count of the test; with a
    // weight of 0.2 the claim still reads high on it.
    let unreviewed = after(&double(true), &[(3, 6)], false);
    assert!(near(read(&unreviewed, "double::correct"), 0.85));
    let with = after(&double(true), &[(3, 6)], true);
    assert!(near(read(&with, "double::correct"), 0.90));
    assert!(near(read(&with, "double::uniform"), 0.91));
    let count = |m: &Model| opinion(m, &Piece::named("double::correct")).r;
    assert!(count(&unreviewed) < count(&with));

    // A function with no tie is still tested, and nothing is read of
    // a tie.
    let free = Function {
        at: None,
        ..double(true)
    };
    let untied = after(&free, &[(3, 6)], true);
    assert!(untied.ties.is_empty());
    assert!(near(read(&untied, "double::correct"), 0.90));
}

/// More passing tests believe the claim more.
// mitra: evidence for tests::more_tests_read_higher
#[test]
fn more_tests_read_higher() {
    let one = after(&double(true), &[(3, 6)], true);
    let four = after(&double(true), &[(1, 2), (2, 4), (3, 6), (4, 8)], true);
    assert!(near(read(&one, "double::correct"), 0.90));
    assert!(near(read(&four, "double::correct"), 0.97));
}

/// The golden, then a run at 7 gives 15. The instance at 7 reads low.
///
/// FINDING, recorded and not wanted: `double::correct` is not refuted.
/// The no about it is one count, shared with the tie and its
/// completeness, against the yes of the passing tests: 0.80 after one
/// passing test and 0.94 after four.
// mitra: evidence for tests::a_bug_lowers_correct
#[test]
fn a_bug_lowers_correct() {
    let at_seven = Piece::named("double::correct @ n = Whole(7)");
    let one = after(&double(true), &[(3, 6), (7, 15)], true);
    assert!(near(read(&one, "double::correct"), 0.80));
    assert!(near(read(&one, &at_seven.name), 0.25));
    assert!(near(read(&one, "double::uniform"), 0.91));
    assert!(near(read(&one, "double::tie"), 0.82));
    assert!(conflicts(&one)
        .iter()
        .any(|group| group.contains(&Piece::named("double::correct"))));

    let four = after(
        &double(true),
        &[(1, 2), (2, 4), (3, 6), (4, 8), (7, 15)],
        true,
    );
    assert!(near(read(&four, "double::correct"), 0.94));
    assert!(near(read(&four, &at_seven.name), 0.19));
    assert!(near(read(&four, "double::tie"), 0.95));
    let before = after(&double(true), &[(1, 2), (2, 4), (3, 6), (4, 8)], true);
    assert!(read(&four, "double::correct") < read(&before, "double::correct"));

    // With no tie the no is not shared: one pass and one failure read
    // one half.
    let free = Function {
        at: None,
        ..double(true)
    };
    let untied = after(&free, &[(3, 6), (7, 15)], true);
    assert!(near(read(&untied, "double::correct"), 0.50));
}
