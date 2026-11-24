//! Red team of the dogfood model (`core2.mitra`): rows that say what
//! the CODE gives, where the model says otherwise, and probes of the
//! reader on its own repository. Every test here passes; each `// mitra:
//! measures` row is what the code does, so a contradiction in `mitra
//! explain` is the model being wrong (or the reader misreading).

use mitra_core::judgement::{Prop, State, Taking};
use mitra_core::numbers::{fused, Belief, Settings};
use mitra_core::simulation::Rational;
use std::collections::BTreeMap;

const NEAR: f64 = 1e-12;

fn taking(value: bool, count: u64, by: &str) -> Taking {
    Taking {
        prop: Prop::Named("p".to_string()),
        value,
        count,
        by: by.to_string(),
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

fn with(prior_weight: f64, base_rate: f64) -> Settings {
    Settings {
        prior_weight,
        base_rate,
        ..Settings::default()
    }
}

// ------------------------------------------- numbers::fused, model wrong

// A negative prior weight is read as 0 (Settings::sane). The model's
// body takes w as given: n = 1 - 2 = -1, so it gives "nothing known".
// mitra: measures numbers::fused (1.0, 0.0, -2.0, 1/2) -> Belief { belief: 1.0, disbelief: 0.0, uncertainty: 0.0, expected: 1.0 }
#[test]
fn rt_fused_negative_prior() {
    is(
        fused(&[taking(true, 1, "x")], &with(-2.0, 0.5)),
        1.0,
        0.0,
        0.0,
        1.0,
    );
}

// A base rate above 1 is clamped to 1. The model gives expected = a = 2,
// and its own claim `nothing_known` (expected == a) is false of the code.
// mitra: measures numbers::fused (0.0, 0.0, 2.0, 2.0) -> Belief { belief: 0.0, disbelief: 0.0, uncertainty: 1.0, expected: 1.0 }
#[test]
fn rt_fused_base_rate_above_one() {
    is(fused(&[], &with(2.0, 2.0)), 0.0, 0.0, 1.0, 1.0);
}

// A base rate below 0 is clamped to 0.
// mitra: measures numbers::fused (0.0, 0.0, 2.0, -1.0) -> Belief { belief: 0.0, disbelief: 0.0, uncertainty: 1.0, expected: 0.0 }
#[test]
fn rt_fused_base_rate_below_zero() {
    is(fused(&[], &with(2.0, -1.0)), 0.0, 0.0, 1.0, 0.0);
}

// A voice weighted -1 counts 0 runs (sane clamps voice weights), so one
// count by it is r = -1 in the model's terms ("weighted runs"), r = 0 in
// the code. The model gives b = -1, u = 2, expected 0.
// mitra: measures numbers::fused (-1.0, 0.0, 2.0, 1/2) -> Belief { belief: 0.0, disbelief: 0.0, uncertainty: 1.0, expected: 1/2 }
#[test]
fn rt_fused_negative_voice() {
    let s = Settings {
        voices: BTreeMap::from([("human:neg".to_string(), -1.0)]),
        ..Settings::default()
    };
    is(
        fused(&[taking(true, 1, "human:neg")], &s),
        0.0,
        0.0,
        1.0,
        0.5,
    );
}

// Control: where the model and the code agree although w < 0 (n > 0,
// and the conflict branch of `opinion` renormalises).
// mitra: measures numbers::fused (3.0, 1.0, -1.0, 1/2) -> Belief { belief: 3/4, disbelief: 1/4, uncertainty: 0.0, expected: 3/4 }
#[test]
fn rt_fused_negative_prior_agrees() {
    let b = fused(
        &[taking(true, 3, "x"), taking(false, 1, "x")],
        &with(-1.0, 0.5),
    );
    is(b, 0.75, 0.25, 0.0, 0.75);
}

// NaN settings (NaN base rate is read as 1/2, NaN weights as 0) cannot be
// written as a row: the language has no NaN. Checked here, unmeasured.
#[test]
fn rt_fused_nan_settings_unmeasurable() {
    is(fused(&[], &with(f64::NAN, f64::NAN)), 0.0, 0.0, 1.0, 0.5);
    is(
        fused(&[taking(true, 1, "x")], &with(f64::NAN, 0.5)),
        1.0,
        0.0,
        0.0,
        1.0,
    );
}

// Rational::over gives None at 0; the model's division faults. No row can
// say "None" or "fault" as a result: unmeasurable.
#[test]
fn rt_over_zero_unmeasurable() {
    assert_eq!(Rational::integer(1).over(&Rational::integer(0)), None);
    assert_eq!(Rational::new(1, 0), None);
}

// ------------------------------------------------------- reader probes

// A test in a `mod`: cargo names it `inner::rt_in_mod`, as the reader does.
mod inner {
    use mitra_core::judgement::State;
    // mitra: measures judgement::sides (State { t: true, f: true }) -> (true, true)
    #[test]
    fn rt_in_mod() {
        assert_eq!(State::Conflict.sides(), (true, true));
    }
}

// An ignored test that measures: R005 expected.
// mitra: measures judgement::told (false, false) -> State { t: false, f: false }
#[test]
#[ignore]
fn rt_ignored() {
    assert_eq!(State::told(false, false), State::Unknown);
}

// A one-line `#[test] fn`: the reader skips the whole line as an
// attribute, so this row is credited to the NEXT test, which measures
// something else entirely.
// mitra: measures judgement::join (State { t: false, f: false }, State { t: false, f: false }) -> State { t: false, f: false }
#[rustfmt::skip]
#[test] fn rt_one_line() { assert_eq!(State::Unknown.join(State::Unknown), State::Unknown); }

#[test]
fn rt_after_one_line() {
    // Tests nothing about `join`.
    assert_eq!(1 + 1, 2);
}

// A row inside a string literal, the string's last line looking like an
// attribute: the reader takes the row and credits it to the next test.
#[allow(dead_code)]
const NOT_A_ROW: &str = r"
// mitra: measures judgement::negate (State { t: false, f: false }) -> State { t: false, f: false }
#[just text in a string]";

#[test]
fn rt_after_string() {
    // Tests nothing about `negate`.
    assert!(NOT_A_ROW.contains("mitra"));
}
