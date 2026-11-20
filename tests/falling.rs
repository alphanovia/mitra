//! The speed of a fall: a claim computed with the standard table, on
//! what was measured, read in the model. The same toy as
//! `examples/falling.rs`.

use mitra_core::bounds::{between, Bounds, Ranges};
use mitra_core::claim::{aimed, bounded, measured, observe, reads, Claim};
use mitra_core::model::*;
use mitra_core::standard::{less, mul, ratio, real, within, Exact, Table};

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
    let mut m = Model::empty(S);
    for (object, h, v) in measurements {
        let tie = tie_of(object);
        if !m.ties.contains(&tie) {
            m.ties.push(tie.clone());
        }
        let by = Voice::named(&format!("machine:timer {object} {h}"));
        let e = measured(&speed(), &tie, &at(*h, *v), &by, &Table::default()).unwrap();
        m.evidence.push(e);
    }
    m
}
fn read(m: &Model, x: &str) -> f64 {
    projected(opinion(m, &Piece::named(x)), S.weight)
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

/// What was measured on the stone is about the whole of the stone's
/// tie, and a tie that does not list the statement is not measured.
// mitra: evidence for claims::measured_is_about_the_whole_of_the_tie
// mitra: evidence for claims::a_tie_that_does_not_list_it_is_not_measured
#[test]
fn a_measurement_is_about_the_whole_of_its_tie() {
    let me = Voice::named("me");
    let e = measured(
        &speed(),
        &tie_of("stone"),
        &at(500, 990),
        &me,
        &Table::default(),
    )
    .unwrap();
    assert_eq!(e.about, whole(&tie_of("stone")));
    let other = Tie {
        statements: vec![Piece::named("floats")],
        ..tie_of("balloon")
    };
    assert_eq!(
        measured(&speed(), &other, &at(500, 990), &me, &Table::default()),
        None
    );
}

/// Three measurements of the stone: the statement, the tie and its
/// completeness are all believed.
#[test]
fn the_stone_believes_the_speed() {
    let m = after(&STONE);
    for x in ["speed", "stone applies", "complete: stone applies"] {
        assert!(read(&m, x) > 0.95, "{x}: {}", read(&m, x));
    }
}

/// The feather after the stone: the statement stands through the
/// stone, and the no goes to the feather's tie and to its completeness.
#[test]
fn the_feather_is_blamed_on_its_tie_and_its_completeness() {
    let mut all = STONE.to_vec();
    all.extend(FEATHER);
    let m = after(&all);
    assert!(read(&m, "speed") > 0.95, "{}", read(&m, "speed"));
    assert!(read(&m, "stone applies") > 0.95);
    assert!(
        read(&m, "feather applies") < 0.3,
        "{}",
        read(&m, "feather applies")
    );
    assert!(read(&m, "complete: feather applies") < 0.45);
    let suspects = conflicts(&m);
    assert!(suspects
        .iter()
        .any(|group| group.contains(&Piece::named("feather applies"))));
}

/// The feather alone: nothing holds the statement up, and it takes its
/// share with the tie.
#[test]
fn the_feather_alone_doubts_the_speed() {
    let m = after(&FEATHER);
    assert!(read(&m, "speed") < 0.4, "{}", read(&m, "speed"));
    assert!((read(&m, "speed") - read(&m, "feather applies")).abs() < 1e-9);
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
    let with = |e: Evidence| Model {
        ties: vec![tie.clone()],
        evidence: vec![e],
        ..Model::empty(S)
    };
    let point = with(measured(&speed(), &tie, &at(500, 990), &me, &Table::default()).unwrap());
    let ranged = bounded(
        &speed(),
        &about((4990, 5010), (9890, 9910)),
        RANGE,
        &me,
        &Ranges,
    );
    let range = with(aimed(ranged.unwrap(), &tie).unwrap());
    assert!(read(&range, "speed") > read(&point, "speed"));
    let unsure = |m: &Model| uncertainty(opinion(m, &Piece::named("speed")), S.weight);
    assert!(unsure(&range) < unsure(&point) / 5.0);
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
