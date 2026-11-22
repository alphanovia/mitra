//! The foundation test model: the speed of a fall, from a value to a
//! verdict. A measurement is evaluated, a voice takes what it gave,
//! and the calculus judges every piece. The same toy as
//! `examples/falling.rs`.

use mitra_core::atom::*;
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
/// What falls: a part of the model, with one function, `fall`: from a
/// height `h`, a speed `v`. It is only declared: nature does it.
fn part(object: &str) -> Part {
    Part {
        name: object.to_string(),
        kind: Kind::Subject,
        records: Vec::new(),
        functions: vec![Function {
            name: fall(object),
            inputs: vec![("h".to_string(), real())],
            output: ("v".to_string(), real()),
            body: None,
        }],
    }
}
fn fall(object: &str) -> String {
    format!("{object}::fall")
}
/// Where it is dropped.
fn tower(object: &str) -> Location {
    Location::at(&["lab", "tower", object])
}
/// The tie: `fall` is the drop in the tower, and the model says all
/// that matters of it there.
fn tie(object: &str) -> Piece<Tie> {
    let about = Tie {
        of: fall(object),
        location: tower(object),
    };
    leaf(&format!("{object}::fall is the drop in the tower"), about)
}
/// `speed`: from 0 to 100 m and 0 to 100 m/s, v² = 2·g·h within
/// 2 m²/s². g is 9.81 m/s².
fn speed(object: &str) -> Piece<Claim> {
    let seen = times(input("v"), input("v"));
    let about = Claim {
        of: fall(object),
        domain: vec![
            ("h".to_string(), metres(0, 100)),
            ("v".to_string(), metres(0, 100)),
        ],
        expression: Apply(within(&real()), vec![seen, fallen(), Given(r(2, 1))]),
    };
    leaf(&format!("{object}::speed"), about)
}
/// `slow`: from 20 m or less, 2·g·h stays under 400.
fn slow(object: &str) -> Piece<Claim> {
    let about = Claim {
        of: fall(object),
        domain: vec![("h".to_string(), metres(0, 20))],
        expression: Apply(less(&real()), vec![fallen(), Given(r(400, 1))]),
    };
    leaf(&format!("{object}::slow"), about)
}
/// What the part needs, said by the architect: nothing but the height
/// matters.
fn only_height(object: &str) -> Piece<Require> {
    let about = Require {
        part: object.to_string(),
        text: "nothing but the height matters".to_string(),
    };
    leaf(&format!("{object}::only the height matters"), about)
}
fn ensure(
    name: &str,
    object: &str,
    text: &str,
    pre: Vec<SomePiece>,
    post: Vec<SomePiece>,
) -> Piece<Ensure> {
    Piece {
        name: format!("{object}::{name}"),
        pre,
        about: Ensure {
            part: object.to_string(),
            text: text.to_string(),
        },
        post,
    }
}
/// What the part promises: given that only the height matters, it
/// falls at `speed`.
fn falls(object: &str) -> Piece<Ensure> {
    let (given, then) = (
        vec![erased(&only_height(object))],
        vec![erased(&speed(object))],
    );
    ensure("falls", object, "it falls at `speed`", given, then)
}
/// A height and a speed measured in the tower, in hundredths.
fn at(object: &str, h: i64, v: i64) -> Measurement<Exact> {
    Measurement {
        function: fall(object),
        at: tower(object),
        values: vec![("h".to_string(), r(h, 100)), ("v".to_string(), r(v, 100))],
    }
}
fn timer(h: i64) -> Voice {
    Voice::named(&format!("machine:timer {h}"))
}
/// The piece of evidence of `speed` on one measurement.
fn seen(object: &str, h: i64, v: i64) -> SomePiece {
    erased(&evidence_of(&speed(object), &at(object, h, v), &timer(h)))
}
/// What a reviewer says: these measured falls represent every fall.
fn represents(object: &str, measured: &[(i64, i64)]) -> Piece<Ensure> {
    let each = measured.iter().map(|(h, v)| seen(object, *h, *v));
    let text = "the measured falls represent every fall";
    ensure(
        "represents",
        object,
        text,
        each.collect(),
        vec![erased(&speed(object))],
    )
}
/// A second claim, resting on the first: given `speed`, `slow`.
fn stays_slow(object: &str) -> Piece<Ensure> {
    let (given, then) = (vec![erased(&speed(object))], vec![erased(&slow(object))]);
    ensure(
        "stays slow",
        object,
        "`slow` follows from `speed`",
        given,
        then,
    )
}
fn take<T: Keyed>(m: &mut Model, x: &Piece<T>, by: &str) {
    m.taken.push(Taking {
        piece: erased(x),
        value: true,
        count: 1.0,
        by: Voice::named(by),
        form: Form::Review,
    });
}
/// The model of one object after these measurements, each by its own
/// timer. Who tied takes the tie. With `reviewed`, the architect
/// takes what the part needs and a reviewer takes that the measured
/// falls represent all.
fn after(object: &str, measured: &[(i64, i64)], reviewed: bool) -> Model {
    let mut m = Model::empty(S);
    m.pieces = vec![erased(&falls(object)), erased(&stays_slow(object))];
    // `slow` follows from `speed` by arithmetic: a deduction.
    m.bias.push((erased(&stays_slow(object)), 1.0));
    for (h, v) in measured {
        m = measure(&m, object, *h, *v);
    }
    take(&mut m, &tie(object), "agent:modeller");
    if reviewed {
        take(&mut m, &only_height(object), "agent:architect");
        take(&mut m, &represents(object, measured), "agent:reviewer");
    }
    m
}
/// One more measurement, evaluated on `speed` and taken by its timer.
fn measure(m: &Model, object: &str, h: i64, v: i64) -> Model {
    let (table, here) = (Table::default(), tie(object));
    let run = at(object, h, v);
    observe(m, &speed(object), &run, &table, Some(&here), 1.0, &timer(h)).unwrap()
}

const STONE: [(i64, i64); 3] = [(500, 990), (1000, 1400), (2000, 1980)];
const FEATHER: [(i64, i64); 2] = [(500, 200), (1000, 210)];

fn says<T>(m: &Model, x: &Piece<T>) -> String {
    shown(&opinion(m, x))
}

/// The part is specified as far as its author chose: `fall` is only
/// declared, and with claims about it it is partly specified.
// mitra: evidence for part::specified_is_a_spectrum
#[test]
fn the_stone_is_partly_specified() {
    let stone = part("stone");
    assert_eq!(function_level(&stone.functions[0], 0), Level::Opaque);
    assert_eq!(function_level(&stone.functions[0], 2), Level::Partial);
    assert_eq!(part_level(&[Level::Partial]), Level::Partial);
}

/// The type of what a piece is about gives its keyword.
// mitra: evidence for piece::the_type_gives_the_keyword
#[test]
fn each_piece_has_its_keyword() {
    assert_eq!(erased(&speed("stone")).about, "claim");
    assert_eq!(erased(&only_height("stone")).about, "require");
    assert_eq!(erased(&falls("stone")).about, "ensure");
    assert_eq!(erased(&tie("stone")).about, "tie");
    assert_eq!(seen("stone", 500, 990).about, "evidence");
    // Before `falls` is a require, after it a claim: any kind.
    assert_eq!(falls("stone").pre[0].about, "require");
    assert_eq!(falls("stone").post[0].about, "claim");
}

/// Evaluate: a value. A stone 5 m down at 9.9 m/s gives true, a
/// feather at 2 m/s gives false, and a measurement of something else
/// gives nothing.
// mitra: evidence for claims::evaluating_gives_a_truth
#[test]
fn a_measurement_is_evaluated() {
    let table = Table::default();
    assert!(reads(&speed("stone")));
    let on = |h, v| evaluate(&speed("stone"), &at("stone", h, v), &table);
    assert_eq!((on(500, 990), on(500, 200)), (Some(true), Some(false)));
    assert_eq!(
        evaluate(&speed("stone"), &at("feather", 500, 990), &table),
        None
    );
}

/// Take: the only crossing. The evidence carries what evaluating gave
/// and what the timer took, as a measurement, and the claim itself is
/// not touched.
// mitra: evidence for claims::a_voice_takes_what_was_evaluated
#[test]
fn a_voice_takes_what_was_evaluated() {
    let m = after("stone", &STONE[..1], false);
    let here = opinion(&m, &seen("stone", 500, 990));
    assert_eq!(here.evaluated(), State::True);
    assert_eq!(here.taken(), State::True);
    assert_eq!(here.judged(), State::True);
    assert_eq!(m.taken[0].form, Form::Measurement);
    let claim = opinion(&m, &speed("stone"));
    assert_eq!(
        (claim.evaluated(), claim.taken(), claim.judged()),
        (State::Unknown, State::Unknown, State::Unknown)
    );
    assert_eq!(verdict(&m), State::Unknown);
}

/// Judge: from three point tests to the claim, through the piece that
/// says they represent every fall, and on into a second claim.
///
/// - `speed` holds, less sure than the tests: the piece that says
///   they represent it is an induction.
/// - `falls` holds: what it promises is true.
/// - `slow` holds: `speed` is injected in `stays slow`.
// mitra: evidence for calculus::a_true_piece_passes_its_state_on
#[test]
fn a_point_test_reaches_the_claim_and_the_next_claim() {
    let m = after("stone", &STONE, true);
    assert_eq!(verdict(&m), State::True);
    assert_eq!(says(&m, &seen("stone", 500, 990)), "true 0.92");
    assert_eq!(says(&m, &speed("stone")), "true 0.86");
    assert_eq!(says(&m, &falls("stone")), "true 0.86");
    assert_eq!(says(&m, &slow("stone")), "true 0.86");
    let all = represents("stone", &STONE);
    assert_eq!(step(&m, &all), Some(Inference::Induction));
    assert_eq!(step(&m, &stays_slow("stone")), Some(Inference::Deduction));
    assert!(suspects(&m).is_empty());
    // `slow` rests on the three measurements and on `represents`.
    let under = rests(&m, &slow("stone"));
    assert_eq!(under.len(), 1);
    assert_eq!(under[0].len(), 4);
    assert!(under[0].contains(&erased(&all)));
    // Nothing is taken of `speed` or `slow`: they hold, judged only.
    assert_eq!(opinion(&m, &slow("stone")).taken(), State::Unknown);
}

/// Without the review the tests stay point tests: the claim is
/// unknown, and so is all that rests on it.
#[test]
fn point_tests_alone_do_not_reach_the_claim() {
    let m = after("stone", &STONE, false);
    assert_eq!(says(&m, &seen("stone", 500, 990)), "true 0.92");
    assert_eq!(says(&m, &speed("stone")), "unknown");
    assert_eq!(says(&m, &slow("stone")), "unknown");
    assert_eq!(verdict(&m), State::Unknown);
}

/// The feather: each measurement is false, so `speed` is refuted
/// there. `falls` promised it under what the architect requires,
/// which is taken: `falls` is refuted.
// mitra: evidence for calculus::a_false_piece_after_refutes
#[test]
fn the_feather_refutes_its_claim() {
    let mut m = after("feather", &FEATHER, false);
    take(&mut m, &only_height("feather"), "agent:architect");
    assert_eq!(says(&m, &speed("feather")), "false 0.92");
    assert_eq!(says(&m, &falls("feather")), "false 0.92");
    assert_eq!(verdict(&m), State::False);
    // With the require not taken, nothing is refuted: `falls` and the
    // require cannot both be true.
    let open = after("feather", &FEATHER, false);
    assert_eq!(says(&open, &falls("feather")), "unknown");
    let both = vec![erased(&falls("feather")), erased(&only_height("feather"))];
    assert_eq!(suspects(&open), [both]);
    // With the tie not taken either, the claim is not refuted: the
    // claim and the tie cannot both be true.
    let mut untied = open.clone();
    untied.taken.retain(|t| t.piece.about != "tie");
    assert_eq!(says(&untied, &speed("feather")), "unknown");
    let either = vec![erased(&tie("feather")), erased(&speed("feather"))];
    assert!(suspects(&untied).contains(&either));
}

/// One bad measurement of the stone after the review: `speed` follows
/// true from the three and the review, and false from the fourth.
/// That is a conflict, and it names what to give up.
// mitra: evidence for calculus::a_conflict_names_what_to_give_up
#[test]
fn a_bad_measurement_is_a_conflict() {
    let m = measure(&after("stone", &STONE, true), "stone", 3000, 990);
    assert_eq!(says(&m, &speed("stone")), "conflict");
    assert_eq!(verdict(&m), State::Conflict);
    let found = suspects(&m);
    assert!(found.contains(&vec![seen("stone", 3000, 990)]));
    assert!(found.contains(&vec![erased(&represents("stone", &STONE))]));
    assert!(found.contains(&vec![erased(&tie("stone"))]));
    // `slow` rests on `speed`, so it is contested with it.
    assert_eq!(says(&m, &slow("stone")), "conflict");
    // What the architect requires is not contested.
    assert_eq!(says(&m, &only_height("stone")), "true 0.92");
}

/// A measurement made somewhere else than the tie says is not taken
/// through that tie.
// mitra: evidence for claims::a_tie_is_of_the_claim_and_the_location
#[test]
fn a_measurement_elsewhere_is_not_taken_through_the_tie() {
    let (table, by) = (Table::default(), timer(500));
    let mut run = at("stone", 500, 990);
    run.at = Location::at(&["field"]);
    let m = Model::empty(S);
    let here = tie("stone");
    assert_eq!(
        observe(&m, &speed("stone"), &run, &table, Some(&here), 1.0, &by),
        None
    );
    assert!(observe(&m, &speed("stone"), &run, &table, None, 1.0, &by).is_some());
}

/// The domain is a guard: a run outside it is outside the contract.
/// From 25 m the expression of `slow` is false, and nothing is said.
// mitra: evidence for claims::a_run_outside_the_domain_says_nothing
#[test]
fn a_run_outside_the_domain_says_nothing() {
    let table = Table::default();
    let on = |h| evaluate(&slow("stone"), &at("stone", h, 0), &table);
    assert_eq!((on(1000), on(2500)), (Some(true), None));
    let m = Model::empty(S);
    let run = at("stone", 2500, 0);
    assert_eq!(
        observe(&m, &slow("stone"), &run, &table, None, 1.0, &timer(2500)),
        None
    );
}

/// A proof and not a test: `slow` read by bounds over its whole
/// domain, every height from 0 to 20 m. The voice takes the claim
/// itself, as an evaluation, and nobody writes a piece for it.
// mitra: evidence for claims::a_reading_over_the_domain_is_a_proof
#[test]
fn a_reading_over_the_whole_domain_is_a_proof() {
    let by = Voice::named("machine:bounds");
    let m = prove(&Model::empty(S), &slow("stone"), &Ranges, 10.0, &by).unwrap();
    let all = opinion(&m, &slow("stone"));
    assert_eq!((all.evaluated(), all.taken()), (State::True, State::True));
    assert_eq!(shown(&all), "true 0.99");
    assert_eq!(m.taken[0].form, Form::Evaluation);
    // `speed` is not true everywhere in its domain, nor false: no proof.
    assert_eq!(prove(&m, &speed("stone"), &Ranges, 10.0, &by), None);
    // A set inside the domain is one piece of evidence, not the claim.
    let lower = Measurement {
        function: fall("feather"),
        at: tower("feather"),
        values: vec![("h".to_string(), metres(0, 10))],
    };
    let claim = slow("feather");
    let some = observe_over(&m, &claim, &lower, &Ranges, None, 10.0, &by).unwrap();
    assert_eq!(says(&some, &claim), "unknown");
    assert_eq!(says(&some, &evidence_of(&claim, &lower, &by)), "true 0.99");
}

/// The models the soundness checks run on.
fn cases() -> Vec<Model> {
    let mut feather = after("feather", &FEATHER, false);
    take(&mut feather, &only_height("feather"), "agent:architect");
    vec![
        after("stone", &STONE, false),
        after("stone", &STONE, true),
        measure(&after("stone", &STONE, true), "stone", 3000, 990),
        after("feather", &FEATHER, false),
        feather,
    ]
}
/// The model with only the takings of these pieces.
fn only(m: &Model, keep: &[SomePiece]) -> Model {
    let mut out = m.clone();
    out.taken.retain(|t| keep.contains(&t.piece));
    out
}
/// The model without the takings of these pieces.
fn without(m: &Model, drop: &[SomePiece]) -> Model {
    let mut out = m.clone();
    out.taken.retain(|t| !drop.contains(&t.piece));
    out
}

/// Sound, one: no state without support. A piece judged true or false
/// rests on something, and an unknown piece or one in conflict has no
/// confidence.
// mitra: evidence for calculus::a_state_keeps_what_it_was_drawn_from
#[test]
fn no_state_without_support() {
    for m in cases() {
        for (x, o) in judge(&m).pieces {
            match o.judged() {
                State::True | State::False => {
                    assert!(!o.rests().is_empty(), "{} rests on nothing", x.name);
                    assert!(o.confidence().is_some());
                }
                _ => assert!(o.rests().is_empty() && o.confidence().is_none()),
            }
        }
    }
}

/// Sound, two: what a state rests on is enough. With only the takings
/// of one of the sets a piece rests on, the piece is judged the same.
#[test]
fn what_a_state_rests_on_is_enough() {
    for m in cases() {
        for (x, o) in judge(&m).pieces {
            for set in o.rests() {
                let alone = opinion(&only(&m, set), &x);
                assert_eq!(alone.judged(), o.judged(), "{} from {set:?}", x.name);
            }
        }
    }
}

/// Sound, three: the order changes nothing. The pieces and the
/// takings in reverse give the same opinion of every piece and the
/// same suspects.
// mitra: evidence for calculus::states_spread_to_the_end
#[test]
fn the_order_changes_nothing() {
    for m in cases() {
        let mut back = m.clone();
        back.pieces.reverse();
        back.taken.reverse();
        for (x, o) in judge(&m).pieces {
            let other = opinion(&back, &x);
            assert_eq!(other.judged(), o.judged(), "{}", x.name);
            assert_eq!(other.confidence(), o.confidence(), "{}", x.name);
        }
        let (one, two) = (suspects(&m), suspects(&back));
        assert_eq!(one.len(), two.len());
        assert!(one.iter().all(|group| two.contains(group)));
    }
}

/// Sound, four: every way out of a conflict is a way out. Giving up
/// the takings of any one listed set leaves no piece in conflict, and
/// a conflict always lists at least one.
// mitra: evidence for calculus::a_conflict_reaches_what_rests_on_it
#[test]
fn giving_up_what_is_named_ends_the_conflict() {
    let m = measure(&after("stone", &STONE, true), "stone", 3000, 990);
    assert_eq!(verdict(&m), State::Conflict);
    let found = suspects(&m);
    assert!(!found.is_empty());
    for group in &found {
        assert_ne!(verdict(&without(&m, group)), State::Conflict, "{group:?}");
    }
    // And only a conflict or something refuted lists suspects.
    for m in cases() {
        if verdict(&m) == State::True || verdict(&m) == State::Unknown {
            assert!(suspects(&m).is_empty());
        }
    }
}

/// Sound, five: a conflict stays where it is. A second part with its
/// own measurements in the same model keeps its states when the first
/// is contested.
#[test]
fn a_conflict_does_not_leak() {
    let mut m = measure(&after("stone", &STONE, true), "stone", 3000, 990);
    let other = after("ball", &STONE, true);
    m.pieces.extend(other.pieces.clone());
    m.taken.extend(other.taken.clone());
    m.evaluated.extend(other.evaluated.clone());
    m.bias.extend(other.bias.clone());
    assert_eq!(says(&m, &speed("stone")), "conflict");
    assert_eq!(says(&m, &speed("ball")), says(&other, &speed("ball")));
    assert_eq!(says(&m, &slow("ball")), "true 0.86");
}

/// Sound, six: nothing is believed that nobody took. With no taking,
/// every piece is unknown.
#[test]
fn nothing_taken_nothing_judged() {
    for mut m in cases() {
        m.taken.clear();
        assert!(judge(&m)
            .pieces
            .iter()
            .all(|(_, o)| o.judged() == State::Unknown));
        assert_eq!(verdict(&m), State::Unknown);
    }
}
