//! The foundation test model: the speed of a fall, from a value to a
//! verdict. A measurement is evaluated, a voice takes what it gave,
//! and the calculus judges every piece. The same toy as
//! `examples/falling.rs`.

use mitra_core::bounds::{between, Bounds, Ranges};
use mitra_core::claim::*;
use mitra_core::model::*;
use mitra_core::standard::{less, mul, ratio, real, within, Exact, Table};

type Value = mitra_core::value::Value<Exact>;
use mitra_core::value::Expression::{Apply, Given, Input};

type Expression = mitra_core::value::Expression<Exact>;

const S: Settings = Settings {
    weight: 0.2,
    bias: 0.5,
    reliability: 1.0,
};

fn r(n: i64, d: i64) -> Value {
    ratio(n, d).unwrap()
}
fn input(name: &str) -> Expression {
    Input(name.to_string(), real())
}
fn times(x: Expression, y: Expression) -> Expression {
    Apply(mul(&real()), vec![x, y])
}
fn fallen() -> Expression {
    times(times(Given(r(2, 1)), Given(r(981, 100))), input("h"))
}

/// From `low` to `high`, in whole metres or metres per second.
fn metres(low: i64, high: i64) -> mitra_core::value::Value<Bounds> {
    between(&real(), (low, 1), (high, 1)).unwrap()
}
/// What falls: a part of the model.
fn part(object: &str) -> Part {
    Part::subject(object)
}
/// Where it was dropped, and that this is the part the model means.
fn tie(object: &str) -> Tie {
    Tie {
        piece: leaf(&format!("{object}::is the one dropped"), &part(object)),
        location: Location::at(&["lab", "tower", object]),
    }
}
/// `speed`: from 0 to 100 m and 0 to 100 m/s, v² = 2·g·h within
/// 2 m²/s². g is 9.81 m/s².
fn speed(object: &str) -> Claim {
    let seen = times(input("v"), input("v"));
    Claim {
        piece: leaf(&format!("{object}::speed"), &part(object)),
        domain: vec![
            ("h".to_string(), metres(0, 100)),
            ("v".to_string(), metres(0, 100)),
        ],
        expression: Apply(within(&real()), vec![seen, fallen(), Given(r(2, 1))]),
    }
}
/// `slow`: from 20 m or less, 2·g·h stays under 400.
fn slow(object: &str) -> Claim {
    Claim {
        piece: leaf(&format!("{object}::slow"), &part(object)),
        domain: vec![("h".to_string(), metres(0, 20))],
        expression: Apply(less(&real()), vec![fallen(), Given(r(400, 1))]),
    }
}
/// The hypothesis of the architect: nothing but the height matters.
fn only_height(object: &str) -> Piece {
    leaf(&format!("{object}::only the height matters"), &part(object))
}
fn piece(name: &str, object: &str, pre: &[Piece], post: &[Piece]) -> Piece {
    Piece {
        name: format!("{object}::{name}"),
        pre: pre.to_vec(),
        subject: part(object),
        post: post.to_vec(),
    }
}
/// The claim of the model: given that only the height matters, the
/// object falls at `speed`.
fn falls(object: &str) -> Piece {
    let then = [speed(object).piece];
    piece("falls", object, &[only_height(object)], &then)
}
/// What a reviewer says: these measured falls represent every fall.
fn represents(object: &str, measured: &[(i64, i64)]) -> Piece {
    let each = measured
        .iter()
        .map(|(h, v)| instance(&speed(object), &at(*h, *v)));
    let seen: Vec<Piece> = each.collect();
    let all = [speed(object).piece];
    piece("represents", object, &seen, &all)
}
/// A second claim, resting on the first: given `speed`, `slow`.
fn stays_slow(object: &str) -> Piece {
    let (given, then) = ([speed(object).piece], [slow(object).piece]);
    piece("stays slow", object, &given, &then)
}

/// A height and a speed, in hundredths.
fn at(h: i64, v: i64) -> [(String, Value); 2] {
    [("h".to_string(), r(h, 100)), ("v".to_string(), r(v, 100))]
}
fn take(m: &mut Model, x: Piece, by: &str) {
    m.taken.push(Taking {
        piece: x,
        value: true,
        count: 1.0,
        by: Voice::named(by),
    });
}
/// The model of one object after these measurements, each by its own
/// timer. With `reviewed`, the architect takes the hypothesis and a
/// reviewer takes that the measured falls represent all.
fn after(object: &str, measured: &[(i64, i64)], reviewed: bool) -> Model {
    let mut m = Model::empty(S);
    m.pieces = vec![falls(object), stays_slow(object)];
    // `slow` follows from `speed` by arithmetic: a deduction.
    m.bias.push((stays_slow(object), 1.0));
    for (h, v) in measured {
        let by = Voice::named(&format!("machine:timer {h}"));
        m = observe(
            &m,
            &speed(object),
            &at(*h, *v),
            &Table::default(),
            Some(&tie(object)),
            1.0,
            &by,
        )
        .unwrap();
    }
    take(&mut m, tie(object).piece, "agent:modeller");
    if reviewed {
        take(&mut m, only_height(object), "agent:architect");
        take(&mut m, represents(object, measured), "agent:reviewer");
    }
    m
}

const STONE: [(i64, i64); 3] = [(500, 990), (1000, 1400), (2000, 1980)];
const FEATHER: [(i64, i64); 2] = [(500, 200), (1000, 210)];

fn says(m: &Model, x: &Piece) -> String {
    shown(&opinion(m, x))
}

/// Evaluate: a value. A stone 5 m down at 9.9 m/s gives true, a
/// feather at 2 m/s gives false, and no speed gives nothing.
// mitra: evidence for claims::evaluating_gives_a_truth
#[test]
fn a_measurement_is_evaluated() {
    let table = Table::default();
    assert!(reads(&speed("stone")));
    assert_eq!(evaluate(&speed("stone"), &at(500, 990), &table), Some(true));
    assert_eq!(
        evaluate(&speed("stone"), &at(500, 200), &table),
        Some(false)
    );
    assert_eq!(evaluate(&speed("stone"), &at(500, 990)[..1], &table), None);
}

/// Take: the only crossing. The instance carries what evaluating gave
/// and what the timer took, and the claim itself is not touched.
// mitra: evidence for claims::a_voice_takes_what_was_evaluated
#[test]
fn a_voice_takes_what_was_evaluated() {
    let m = after("stone", &STONE[..1], false);
    let here = opinion(&m, &instance(&speed("stone"), &at(500, 990)));
    assert_eq!(here.evaluated(), State::True);
    assert_eq!(here.taken(), State::True);
    assert_eq!(here.judged(), State::True);
    let claim = opinion(&m, &speed("stone").piece);
    assert_eq!(
        (claim.evaluated(), claim.taken(), claim.judged()),
        (State::Unknown, State::Unknown, State::Unknown)
    );
    assert_eq!(verdict(&m), State::Unknown);
}

/// Judge: from three point tests to the claim, through the piece that
/// says they represent every fall, and on into a second claim.
///
/// - `speed` is true, less sure than the tests: the piece that says
///   they represent it is an induction.
/// - `falls` holds: what it promises is true.
/// - `slow` is true: `speed` is injected in `stays slow`.
// mitra: evidence for calculus::a_true_piece_passes_its_state_on
#[test]
fn a_point_test_reaches_the_claim_and_the_next_claim() {
    let m = after("stone", &STONE, true);
    assert_eq!(verdict(&m), State::True);
    // The instances are as sure as their measurement. `speed` comes
    // from them by an induction, so it is less sure, and so is all
    // that rests on it.
    let here = instance(&speed("stone"), &at(500, 990));
    assert_eq!(says(&m, &here), "true 0.92");
    assert_eq!(says(&m, &speed("stone").piece), "true 0.86");
    assert_eq!(says(&m, &falls("stone")), "true 0.86");
    assert_eq!(says(&m, &slow("stone").piece), "true 0.86");
    assert_eq!(
        step(&m, &represents("stone", &STONE)),
        Some(Inference::Induction)
    );
    assert_eq!(step(&m, &stays_slow("stone")), Some(Inference::Deduction));
    assert!(suspects(&m).is_empty());
    // `slow` rests on the three measurements and on `represents`.
    let under = rests(&m, &slow("stone").piece);
    assert_eq!(under.len(), 1);
    assert_eq!(under[0].len(), 4);
    assert!(under[0].contains(&represents("stone", &STONE)));
    // Nothing is taken of `speed` or `slow`: they are judged only.
    assert_eq!(opinion(&m, &slow("stone").piece).taken(), State::Unknown);
}

/// Without the review the tests stay point tests: the claim is
/// unknown, and so is all that rests on it.
#[test]
fn point_tests_alone_do_not_reach_the_claim() {
    let m = after("stone", &STONE, false);
    let here = instance(&speed("stone"), &at(500, 990));
    assert_eq!(says(&m, &here), "true 0.92");
    assert_eq!(says(&m, &speed("stone").piece), "unknown");
    assert_eq!(says(&m, &slow("stone").piece), "unknown");
    assert_eq!(verdict(&m), State::Unknown);
}

/// The feather: each measurement is false, so `speed` is false there,
/// surer with two measurements than with one. `falls` promised it
/// under the architect's hypothesis, which is taken: `falls` is false.
// mitra: evidence for calculus::a_false_piece_after_refutes
#[test]
fn the_feather_refutes_its_claim() {
    let mut m = after("feather", &FEATHER, false);
    take(&mut m, only_height("feather"), "agent:architect");
    assert_eq!(says(&m, &speed("feather").piece), "false 0.92");
    assert_eq!(says(&m, &falls("feather")), "false 0.92");
    assert_eq!(verdict(&m), State::False);
    // With the hypothesis not taken, nothing is refuted: `falls` and
    // the hypothesis cannot both be true.
    let open = after("feather", &FEATHER, false);
    assert_eq!(says(&open, &falls("feather")), "unknown");
    assert_eq!(
        suspects(&open),
        [vec![falls("feather"), only_height("feather")]]
    );
}

/// One bad measurement of the stone after the review: `speed` follows
/// true from the three and the review, and false from the fourth.
/// That is a conflict, and it names what to give up.
// mitra: evidence for calculus::a_conflict_names_what_to_give_up
#[test]
fn a_bad_measurement_is_a_conflict() {
    let mut m = after("stone", &STONE, true);
    let by = Voice::named("machine:timer 3000");
    m = observe(
        &m,
        &speed("stone"),
        &at(3000, 990),
        &Table::default(),
        Some(&tie("stone")),
        1.0,
        &by,
    )
    .unwrap();
    assert_eq!(says(&m, &speed("stone").piece), "conflict");
    assert_eq!(verdict(&m), State::Conflict);
    let found = suspects(&m);
    let bad = instance(&speed("stone"), &at(3000, 990));
    assert!(found.contains(&vec![bad]));
    assert!(found.contains(&vec![represents("stone", &STONE)]));
    // `slow` rests on `speed`, so it is contested with it.
    assert_eq!(says(&m, &slow("stone").piece), "conflict");
    // The hypothesis of the architect is not contested.
    assert_eq!(says(&m, &only_height("stone")), "true 0.92");
}

/// The domain is a guard: a run outside it is outside the contract.
/// From 25 m the expression of `slow` is false, and nothing is said.
// mitra: evidence for claims::a_run_outside_the_domain_says_nothing
#[test]
fn a_run_outside_the_domain_says_nothing() {
    let table = Table::default();
    let from = |h: i64| [("h".to_string(), r(h, 1))];
    assert_eq!(evaluate(&slow("stone"), &from(10), &table), Some(true));
    assert_eq!(evaluate(&slow("stone"), &from(25), &table), None);
    let by = Voice::named("machine:test");
    let m = Model::empty(S);
    assert_eq!(
        observe(&m, &slow("stone"), &from(25), &table, None, 1.0, &by),
        None
    );
}

/// A proof and not a test: `slow` read by bounds over its whole
/// domain, every height from 0 to 20 m. The voice takes the claim
/// itself, and nobody writes a piece for it.
// mitra: evidence for claims::a_reading_over_the_domain_is_a_proof
#[test]
fn a_reading_over_the_whole_domain_is_a_proof() {
    let by = Voice::named("machine:bounds");
    let m = prove(&Model::empty(S), &slow("stone"), &Ranges, 10.0, &by).unwrap();
    let all = opinion(&m, &slow("stone").piece);
    assert_eq!((all.evaluated(), all.taken()), (State::True, State::True));
    assert_eq!(shown(&all), "true 0.99");
    // `speed` is not true everywhere in its domain, nor false: no proof.
    assert_eq!(prove(&m, &speed("stone"), &Ranges, 10.0, &by), None);
    // A range inside the domain is one instance, not the claim.
    let part_of = [("h".to_string(), metres(0, 10))];
    let some = observe_over(&m, &slow("feather"), &part_of, &Ranges, None, 10.0, &by).unwrap();
    assert_eq!(says(&some, &slow("feather").piece), "unknown");
    assert_eq!(
        says(&some, &instance(&slow("feather"), &part_of)),
        "true 0.99"
    );
}
