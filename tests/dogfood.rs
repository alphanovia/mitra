//! The dogfood: tests that measure core2 against its own model,
//! `core2.mitra` at the root. Each `// mitra: measures` comment is a row
//! of the model; the test checks every value of it. Rows are exact, the
//! code's f64 is compared within `NEAR`.

use mitra_core::judgement::{Prop, State, Taking};
use mitra_core::numbers::{fused, Belief, Settings};
use mitra_core::simulation::Rational;

/// How close an f64 must be to the exact value of a row.
const NEAR: f64 = 1e-12;

fn q(n: i64, d: i64) -> Rational {
    Rational::new(n, d).unwrap()
}

fn state(t: bool, f: bool) -> State {
    State::told(t, f)
}

// ------------------------------------------------------------- simulation

// mitra: measures simulation::plus (1/2, 1/3) -> 5/6
#[test]
fn dogfood_plus() {
    assert_eq!(q(1, 2).plus(&q(1, 3)), q(5, 6));
}

// mitra: measures simulation::minus (1/2, 1/3) -> 1/6
#[test]
fn dogfood_minus() {
    assert_eq!(q(1, 2).minus(&q(1, 3)), q(1, 6));
}

// mitra: measures simulation::times (2/3, 3/4) -> 1/2
#[test]
fn dogfood_times() {
    assert_eq!(q(2, 3).times(&q(3, 4)), q(1, 2));
}

// mitra: measures simulation::over (1/2, 1/4) -> 2.0
#[test]
fn dogfood_over() {
    assert_eq!(q(1, 2).over(&q(1, 4)), Some(q(2, 1)));
}

// mitra: measures simulation::abs (-3/2) -> 3/2
// mitra: measures simulation::abs (3/2) -> 3/2
// mitra: measures simulation::abs (0.0) -> 0.0
#[test]
fn dogfood_abs() {
    assert_eq!(q(-3, 2).abs(), q(3, 2));
    assert_eq!(q(3, 2).abs(), q(3, 2));
    assert_eq!(q(0, 1).abs(), q(0, 1));
}

// -------------------------------------------------------------- judgement

// mitra: measures judgement::told (true, false) -> State { t: true, f: false }
// mitra: measures judgement::told (true, true) -> State { t: true, f: true }
#[test]
fn dogfood_told() {
    assert_eq!(State::told(true, false), State::True);
    assert_eq!(State::told(true, true), State::Conflict);
}

// mitra: measures judgement::sides (State { t: false, f: true }) -> (false, true)
#[test]
fn dogfood_sides() {
    assert_eq!(State::False.sides(), (false, true));
}

// mitra: measures judgement::and (State { t: true, f: false }, State { t: true, f: true }) -> State { t: true, f: true }
// mitra: measures judgement::and (State { t: true, f: false }, State { t: false, f: false }) -> State { t: false, f: false }
#[test]
fn dogfood_and() {
    assert_eq!(state(true, false).and(state(true, true)), State::Conflict);
    assert_eq!(state(true, false).and(state(false, false)), State::Unknown);
}

// mitra: measures judgement::join (State { t: true, f: false }, State { t: false, f: true }) -> State { t: true, f: true }
#[test]
fn dogfood_join() {
    assert_eq!(State::True.join(State::False), State::Conflict);
}

// mitra: measures judgement::negate (State { t: true, f: false }) -> State { t: false, f: true }
#[test]
fn dogfood_negate() {
    assert_eq!(!State::True, State::False);
}

// ---------------------------------------------------------------- numbers

fn taking(value: bool, count: u64) -> Taking {
    Taking {
        prop: Prop::Named("p".to_string()),
        value,
        count,
        by: "test:dogfood".to_string(),
    }
}

fn is(b: Belief, belief: f64, disbelief: f64, uncertainty: f64, expected: f64) {
    for (got, want) in [
        (b.belief, belief),
        (b.disbelief, disbelief),
        (b.uncertainty, uncertainty),
        (b.expected, expected),
    ] {
        assert!((got - want).abs() < NEAR, "{b:?}");
    }
}

// mitra: measures numbers::fused (97.0, 3.0, 2.0, 1/2) -> Belief { belief: 97/102, disbelief: 1/34, uncertainty: 1/51, expected: 49/51 }
#[test]
fn dogfood_fused() {
    let b = fused(&[taking(true, 97), taking(false, 3)], &Settings::default());
    is(b, 97.0 / 102.0, 1.0 / 34.0, 1.0 / 51.0, 49.0 / 51.0);
}

// mitra: measures numbers::fused (0.0, 0.0, 2.0, 1/2) -> Belief { belief: 0.0, disbelief: 0.0, uncertainty: 1.0, expected: 1/2 }
#[test]
fn dogfood_fused_nothing() {
    is(fused(&[], &Settings::default()), 0.0, 0.0, 1.0, 0.5);
}

// mitra: measures numbers::fused (1.0, 1.0, 0.0, 1/4) -> Belief { belief: 1/2, disbelief: 1/2, uncertainty: 0.0, expected: 1/2 }
#[test]
fn dogfood_fused_no_prior() {
    let s = Settings {
        prior_weight: 0.0,
        base_rate: 0.25,
        ..Settings::default()
    };
    is(fused(&[taking(true, 1), taking(false, 1)], &s), 0.5, 0.5, 0.0, 0.5);
}
