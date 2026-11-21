//! The speed of a fall: a claim computed with the standard table, on
//! what was measured, read in the model. The same toy as
//! `examples/falling.rs`.

use mitra_core::bounds::{between, Bounds, Ranges};
use mitra_core::claim::{bounded, instance, observe, reads, recorded, Claim};
use mitra_core::model::*;
use mitra_core::standard::{less, mul, ratio, real, within, Exact, Table};
use mitra_core::verdict::{overall, state, suspects, verdict_of, State};

type Value = mitra_core::value::Value<Exact>;
use mitra_core::value::Expression::{Apply, Given, Input};

type Expression = mitra_core::value::Expression<Exact>;

/// The settings these are recorded at: a statement and a tie start at
/// one half, a measurement at 0.9, and no voice is doubted. Nothing is
/// written piece by piece.
const S: Settings = Settings {
    weight: 0.2,
    statement: 0.5,
    evidence: 0.9,
    tie: 0.5,
    completeness: 0.6,
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

/// v² = 2·g·h within 2 m²/s². g is 9.81 m/s².
fn speed() -> Statement<Claim> {
    let fallen = times(times(Given(r(2, 1)), Given(r(981, 100))), input("h"));
    let seen = times(input("v"), input("v"));
    Statement {
        piece: Piece::named("speed"),
        content: Claim {
            expression: Apply(within(&real()), vec![seen, fallen, Given(r(2, 1))]),
        },
    }
}
fn tie_of(object: &str) -> Tie {
    Tie {
        piece: Piece::named(&format!("{object} applies")),
        statements: vec![Piece::named("speed")],
        place: Place::at(&[object]),
    }
}
/// A height and a speed, in hundredths.
fn at(h: i64, v: i64) -> [(String, Value); 2] {
    [("h".to_string(), r(h, 100)), ("v".to_string(), r(v, 100))]
}
/// The model after these measurements, each believed at 0.9 before
/// anything and each by its own voice.
fn after(measurements: &[(&str, i64, i64)]) -> Model {
    after_with(measurements, false)
}
/// The piece that says the measured falls stand for all falls.
fn uniform() -> Piece {
    Piece::named("speed::uniform")
}
fn after_with(measurements: &[(&str, i64, i64)], reviewed: bool) -> Model {
    let mut m = Model::empty(S);
    for (object, h, v) in measurements {
        let by = Voice::named(&format!("machine:timer {object} {h}"));
        let e = observe(&speed(), &at(*h, *v), &by, &Table::default()).unwrap();
        m = recorded(&m, &speed(), e, Some(&tie_of(object)), Some(&uniform())).unwrap();
    }
    if reviewed {
        m.evidence.push(Evidence {
            piece: Piece::named("review of uniform"),
            about: uniform(),
            value: yes(),
            by: Voice::named("agent:reviewer"),
        });
    }
    m
}

const STONE: [(&str, i64, i64); 3] = [
    ("stone", 500, 990),
    ("stone", 1000, 1400),
    ("stone", 2000, 1980),
];
const FEATHER: [(&str, i64, i64); 2] = [("feather", 500, 200), ("feather", 1000, 210)];

/// The claim reads: its expression gives a truth.
#[test]
fn the_speed_of_a_fall_is_a_claim() {
    assert!(reads(&speed()));
}

/// A stone 5 m down at 9.9 m/s: 98.01 against 98.1, within 2. A
/// feather at 2 m/s: 4 against 98.1.
#[test]
fn a_measurement_says_yes_or_no() {
    let me = Voice::named("me");
    let table = Table::default();
    let stone = observe(&speed(), &at(500, 990), &me, &table).unwrap();
    let feather = observe(&speed(), &at(500, 200), &me, &table).unwrap();
    assert_eq!((stone.value, feather.value), (yes(), no()));
}

/// What was measured on the stone is about the claim at that height
/// and speed. Recorded through the stone's tie, it is about the tie,
/// that instance and the tie's completeness together; a tie that does
/// not list the claim records nothing.
// mitra: evidence for claims::an_observation_is_about_an_instance
// mitra: evidence for claims::a_tie_that_does_not_list_it_records_nothing
#[test]
fn a_measurement_is_about_an_instance() {
    let me = Voice::named("me");
    let e = observe(&speed(), &at(500, 990), &me, &Table::default()).unwrap();
    let here = instance(&speed(), &at(500, 990));
    assert_eq!(e.about, here);
    let m = recorded(
        &Model::empty(S),
        &speed(),
        e.clone(),
        Some(&tie_of("stone")),
        None,
    )
    .unwrap();
    assert_eq!(
        parts(&m, &m.related[1].content),
        [
            Piece::named("stone applies"),
            here,
            Piece::named("complete: stone applies")
        ]
    );
    assert_eq!(m.evidence[0].about, m.related[1].content.pieces[0]);
    let other = Tie {
        statements: vec![Piece::named("floats")],
        ..tie_of("balloon")
    };
    assert_eq!(
        recorded(&Model::empty(S), &speed(), e, Some(&other), None),
        None
    );
}

fn shown(m: &Model, x: &str) -> String {
    mitra_core::verdict::shown(&verdict_of(m, &Piece::named(x)), S.weight)
}

/// Three measurements of the stone, `uniform` reviewed: the claim, its
/// instances, the tie and its completeness are true. The claim is as
/// sure as the one review of `uniform`; the tie as three measurements.
#[test]
fn the_stone_holds_the_speed() {
    let m = after_with(&STONE, true);
    assert_eq!(overall(&m), State::True);
    assert_eq!(shown(&m, "speed"), "true 0.91");
    assert_eq!(shown(&m, "stone applies"), "true 0.97");
    assert_eq!(shown(&m, "complete: stone applies"), "true 0.97");
    for (_, h, v) in STONE {
        let here = instance(&speed(), &at(h, v));
        assert_eq!(shown(&m, &here.name), "true 0.95");
    }
    assert!(suspects(&m).is_empty());
    // Without the review nothing leads to the claim.
    let unreviewed = after(&STONE);
    assert_eq!(state(&unreviewed, &Piece::named("speed")), State::Unknown);
    assert_eq!(overall(&unreviewed), State::Unknown);
}

/// The feather after the stone: the claim stands through the stone, so
/// it is true at the feather's heights too. Each of the feather's
/// measurements is then false with the claim true there: the feather's
/// tie and its completeness cannot both be true. Nothing is in
/// conflict, and the verdict is false.
#[test]
fn the_feather_is_blamed_on_its_tie_and_its_completeness() {
    let mut all = STONE.to_vec();
    all.extend(FEATHER);
    let m = after_with(&all, true);
    assert_eq!(shown(&m, "speed"), "true 0.91");
    assert_eq!(shown(&m, "stone applies"), "true 0.97");
    let here = instance(&speed(), &at(500, 200));
    assert_eq!(shown(&m, &here.name), "true 0.91");
    assert_eq!(shown(&m, "feather applies"), "unknown");
    assert_eq!(shown(&m, "complete: feather applies"), "unknown");
    assert_eq!(overall(&m), State::False);
    assert_eq!(
        suspects(&m),
        [[
            Piece::named("feather applies"),
            Piece::named("complete: feather applies")
        ]]
    );
}

/// The feather alone: nothing holds the claim up. Each measurement is
/// false and three things cannot all be true: the tie, the claim at
/// that height and speed, and the tie's completeness.
#[test]
fn the_feather_alone_leaves_the_speed_unknown() {
    let m = after_with(&FEATHER, true);
    assert_eq!(shown(&m, "speed"), "unknown");
    assert_eq!(overall(&m), State::False);
    let found = suspects(&m);
    assert_eq!(found.len(), 2);
    assert_eq!(found[0].len(), 3);
    assert_eq!(found[0][0], Piece::named("feather applies"));
}

/// A height and a speed, each known within a margin, in thousandths.
fn about(h: (i64, i64), v: (i64, i64)) -> [(String, mitra_core::value::Value<Bounds>); 2] {
    let range = |(low, high): (i64, i64)| between(&real(), (low, 1000), (high, 1000)).unwrap();
    [("h".to_string(), range(h)), ("v".to_string(), range(v))]
}
/// What a reading over a whole range is worth here, in observations.
const RANGE: f64 = 10.0;

/// The stone at 5 m and 9.9 m/s, each known to a hundredth: the claim
/// is true wherever the real height and speed are in those margins. A
/// feather is false everywhere in its own. Margins of a metre say
/// nothing.
// mitra: evidence for claims::a_sure_claim_says_its_count
#[test]
fn a_measurement_with_its_margin_is_read_by_bounds() {
    let me = Voice::named("me");
    let read = |h, v| bounded(&speed(), &about(h, v), RANGE, &me, &Ranges).map(|e| e.value);
    let sure = read((4990, 5010), (9890, 9910)).unwrap();
    assert_eq!((sure.r, sure.s), (RANGE, 0.0));
    let never = read((4990, 5010), (1990, 2010)).unwrap();
    assert_eq!((never.r, never.s), (0.0, RANGE));
    assert_eq!(read((4000, 6000), (9890, 9910)), None);
}

/// One reading over the margins says more than the point in their
/// middle: it stands for every point of them.
#[test]
fn a_range_says_more_than_a_point() {
    let me = Voice::named("machine:timer");
    let tie = tie_of("stone");
    let with = |e: Evidence| {
        recorded(&Model::empty(S), &speed(), e, Some(&tie), Some(&uniform())).unwrap()
    };
    let point = with(observe(&speed(), &at(500, 990), &me, &Table::default()).unwrap());
    let ranged = bounded(
        &speed(),
        &about((4990, 5010), (9890, 9910)),
        RANGE,
        &me,
        &Ranges,
    );
    let range = with(ranged.unwrap());
    let over = instance(&speed(), &about((4990, 5010), (9890, 9910)));
    let here = instance(&speed(), &at(500, 990));
    assert_eq!(shown(&point, &here.name), "true 0.91");
    assert_eq!(shown(&range, &over.name), "true 0.99");
}

/// A claim over a whole domain: from 0 to 20 m, the square of the
/// speed stays under 400. Read by bounds it is true for every height
/// at once; computed at 5 m it is true at 5 m.
#[test]
fn a_claim_is_read_over_its_whole_domain() {
    let fallen = times(times(Given(r(2, 1)), Given(r(981, 100))), input("h"));
    let slow = Statement {
        piece: Piece::named("slow enough"),
        content: Claim {
            expression: Apply(less(&real()), vec![fallen, Given(r(400, 1))]),
        },
    };
    let me = Voice::named("machine:mitra");
    let heights = |low, high| {
        [(
            "h".to_string(),
            between(&real(), (low, 1), (high, 1)).unwrap(),
        )]
    };
    let all = bounded(&slow, &heights(0, 20), RANGE, &me, &Ranges).unwrap();
    assert_eq!((all.value.r, all.value.s), (RANGE, 0.0));
    // Up to 21 m it is not true everywhere, nor false everywhere.
    assert_eq!(bounded(&slow, &heights(0, 21), RANGE, &me, &Ranges), None);
    let one = observe(&slow, &[("h".to_string(), r(5, 1))], &me, &Table::default()).unwrap();
    assert_eq!(one.value, yes());
}
