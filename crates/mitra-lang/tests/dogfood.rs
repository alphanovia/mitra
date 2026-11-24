//! The dogfood: tests that measure core2 against its own model,
//! the dogfood model, `src/*.mitra` beside the code (`src/bin/mitra.mitra`
//! the whole). Each `// mitra: measures` comment is a row
//! of the model; the test checks every value of it. Rows are exact, the
//! code's f64 is compared within `NEAR`.

use mitra_lang::judgement::{Prop, State, Taking};
use mitra_lang::numbers::{fused, Belief, Settings};
use mitra_lang::simulation::Rational;

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

// mitra: measures simulation::over (1/2, 1/4) -> Some(2.0)
// mitra: measures simulation::over (1/2, 0.0) -> None
#[test]
fn dogfood_over() {
    assert_eq!(q(1, 2).over(&q(1, 4)), Some(q(2, 1)));
    assert_eq!(q(1, 2).over(&q(0, 1)), None);
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

// --------------------------------------------------------------- location

fn at(names: &[&str]) -> mitra_lang::location::Location {
    mitra_lang::location::Location {
        path: names.iter().map(|n| n.to_string()).collect(),
    }
}

// Text now has a value (finding 3, step 3): `within` is measured.
// mitra: measures location::within (Location { path: ["lab", "tower"] }, Location { path: ["lab"] }) -> true
// mitra: measures location::within (Location { path: ["lab"] }, Location { path: ["lab", "tower"] }) -> false
// mitra: measures location::within (Location { path: ["été"] }, Location { path: ["ét"] }) -> false
// mitra: measures location::within (Location { path: [] }, Location { path: [] }) -> true
#[test]
fn dogfood_within() {
    use mitra_lang::location::within;
    assert!(within(&at(&["lab", "tower"]), &at(&["lab"])));
    assert!(!within(&at(&["lab"]), &at(&["lab", "tower"])));
    assert!(!within(&at(&["été"]), &at(&["ét"])));
    assert!(within(&at(&[]), &at(&[])));
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
    is(
        fused(&[taking(true, 1), taking(false, 1)], &s),
        0.5,
        0.5,
        0.0,
        0.5,
    );
}

// ------------------------------------------- judgement, every point

/// The claims of `judgement::and`, `join` and `negate` in src/judgement.mitra,
/// at every one of the 16 (or 4) points: the store reviews each claim
/// by this test's voice.
#[test]
fn dogfood_state_claims_everywhere() {
    let all = [State::Unknown, State::True, State::False, State::Conflict];
    for x in all {
        let (xt, xf) = x.sides();
        let (nt, nf) = (!x).sides();
        // negate::swaps
        assert!(nt == xf && nf == xt);
        for y in all {
            let (yt, yf) = y.sides();
            let (zt, zf) = x.and(y).sides();
            // and::false_wins, and::true_needs_both
            assert_eq!(xf || yf, zf);
            assert!(!zt || (xt && yt));
            let (jt, jf) = x.join(y).sides();
            // join::keeps
            assert!((!xt || jt) && (!xf || jf) && (!yt || jt) && (!yf || jf));
        }
    }
}

// ---------------------------------------------- settings, their defaults

// mitra: measures agree::agree_defaults () -> AgreeSettings { points: 64, splits: 4, budget: Budget { steps: 1000000, depth: 1000 }, length: 3, height: 3 }
#[test]
fn dogfood_agree_defaults() {
    let s = mitra_lang::agree::Settings::default();
    assert_eq!((s.points, s.splits, s.length, s.height), (64, 4, 3, 3));
    assert_eq!((s.budget.steps, s.budget.depth), (1_000_000, 1_000));
}

// mitra: measures parse::parse_defaults () -> ParseSettings { depth: 2000 }
#[test]
fn dogfood_parse_defaults() {
    assert_eq!(mitra_lang::parse::Settings::default().depth, 2000);
}

// mitra: measures explain::explain_defaults () -> ExplainSettings { rounds: 64, cases: 1024, rests: 1024, budget: Budget { steps: 1000000, depth: 1000 } }
#[test]
fn dogfood_explain_defaults() {
    let s = mitra_lang::explain::Settings::default();
    assert_eq!((s.rounds, s.cases, s.rests), (64, 1024, 1024));
    assert_eq!((s.budget.steps, s.budget.depth), (1_000_000, 1_000));
}

// mitra: measures simulation::budget_defaults () -> Budget { steps: 1000000, depth: 1000 }
#[test]
fn dogfood_budget_defaults() {
    let b = mitra_lang::simulation::Budget::default();
    assert_eq!((b.steps, b.depth), (1_000_000, 1_000));
}

// mitra: measures blame::blame_defaults () -> BlameSettings { diagnoses: 10, tries: 100000 }
#[test]
fn dogfood_blame_defaults() {
    let s = mitra_lang::blame::Settings::default();
    assert_eq!((s.diagnoses, s.tries), (10, 100_000));
}
