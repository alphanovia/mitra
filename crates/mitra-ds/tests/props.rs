//! mitra-ds's tests. Each one above which a `mitra: test` line sits stands
//! for the evidence that line gives (see the claims in `src/lib.rs`). In
//! the model, a mass is the record `Mass(b, d, u)`, as in Rust.

use mitra_ds::Mass;

/// xorshift64, fixed seed: case n is always the same pair of masses. Even
/// cases pair single pieces of evidence (`judged`); odd cases pair general
/// masses, with belief on both sides, as combining several pieces gives.
fn case(n: u64) -> (Mass, Mass) {
    let mut s = (n + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
    let mut next = || {
        s ^= s << 13;
        s ^= s >> 7;
        s ^= s << 17;
        (s >> 11) as f64 / (1u64 << 53) as f64
    };
    let mut mass = || {
        if n.is_multiple_of(2) {
            let correct = next() < 0.5;
            Mass::judged(correct, next()).unwrap()
        } else {
            let (x, y) = (next(), next());
            let (lo, hi) = (x.min(y), x.max(y));
            Mass {
                b: lo,
                d: hi - lo,
                u: 1.0 - hi,
            }
        }
    };
    (mass(), mass())
}

fn mass(b: f64, d: f64, u: f64) -> Mass {
    Mass { b, d, u }
}

const SURE: Mass = Mass {
    b: 1.0,
    d: 0.0,
    u: 0.0,
};
const SURE_WRONG: Mass = Mass {
    b: 0.0,
    d: 1.0,
    u: 0.0,
};

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() <= 1e-9
}

fn same(m: Mass, n: Mass) -> bool {
    close(m.b, n.b) && close(m.d, n.d) && close(m.u, n.u)
}

/// Yager: a valid mass, commutative, the conflict in unknown.
fn yager_ok(m1: Mass, m2: Mass) -> bool {
    let (Some((m, k)), Some((n, k2))) = (m1.yager(&m2), m2.yager(&m1)) else {
        return false;
    };
    m.is_valid() && close(k, k2) && same(m, n) && m.u >= k - 1e-9
}

/// Dempster: a valid mass, commutative, and Yager's result renormalised.
fn dempster_ok(m1: Mass, m2: Mass) -> bool {
    let k = m1.conflict(&m2);
    match (m1.dempster(&m2), m2.dempster(&m1)) {
        (Some(m), Some(n)) => {
            let Some((y, _)) = m1.yager(&m2) else {
                return false;
            };
            let renormalised = Mass {
                b: y.b / (1.0 - k),
                d: y.d / (1.0 - k),
                u: (y.u - k) / (1.0 - k),
            };
            m.is_valid() && same(m, n) && same(m, renormalised)
        }
        (None, None) => close(k, 1.0),
        _ => false,
    }
}

// Not in the model: Mitra can't record evidence from sampled inputs yet
// (review list, gap G3). Same for `t_dempster_random`.
#[test]
fn t_yager_random() {
    assert!((0..1000).all(|n| {
        let (a, b) = case(n);
        yager_ok(a, b)
    }));
}

#[test]
fn t_dempster_random() {
    assert!((0..1000).all(|n| {
        let (a, b) = case(n);
        dempster_ok(a, b)
    }));
}

// The extremes of Mitra's golden sets, one input each.

// mitra: test t_yager_agree : yager(Mass(1.0, 0.0, 0.0), Mass(1.0, 0.0, 0.0)) = Some((Mass(1.0, 0.0, 0.0), 0.0))
#[test]
fn t_yager_agree() {
    assert_eq!(SURE.yager(&SURE), Some((SURE, 0.0)));
}

// mitra: test t_yager_contradict : yager(Mass(1.0, 0.0, 0.0), Mass(0.0, 1.0, 0.0)) = Some((Mass(0.0, 0.0, 1.0), 1.0))
#[test]
fn t_yager_contradict() {
    assert_eq!(SURE.yager(&SURE_WRONG), Some((Mass::VACUOUS, 1.0)));
}

// mitra: test t_yager_contradict_swapped : yager(Mass(0.0, 1.0, 0.0), Mass(1.0, 0.0, 0.0)) = Some((Mass(0.0, 0.0, 1.0), 1.0))
#[test]
fn t_yager_contradict_swapped() {
    assert_eq!(SURE_WRONG.yager(&SURE), Some((Mass::VACUOUS, 1.0)));
}

// mitra: test t_yager_with_nothing : yager(Mass(1.0, 0.0, 0.0), Mass(0.0, 0.0, 1.0)) = Some((Mass(1.0, 0.0, 0.0), 0.0))
#[test]
fn t_yager_with_nothing() {
    assert_eq!(SURE.yager(&Mass::VACUOUS), Some((SURE, 0.0)));
}

// mitra: test t_yager_nothing_first : yager(Mass(0.0, 0.0, 1.0), Mass(1.0, 0.0, 0.0)) = Some((Mass(1.0, 0.0, 0.0), 0.0))
#[test]
fn t_yager_nothing_first() {
    assert_eq!(Mass::VACUOUS.yager(&SURE), Some((SURE, 0.0)));
}

// mitra: test t_yager_nothing : yager(Mass(0.0, 0.0, 1.0), Mass(0.0, 0.0, 1.0)) = Some((Mass(0.0, 0.0, 1.0), 0.0))
#[test]
fn t_yager_nothing() {
    assert_eq!(
        Mass::VACUOUS.yager(&Mass::VACUOUS),
        Some((Mass::VACUOUS, 0.0))
    );
}

// mitra: test t_dempster_agree : dempster(Mass(1.0, 0.0, 0.0), Mass(1.0, 0.0, 0.0)) = Some(Mass(1.0, 0.0, 0.0))
#[test]
fn t_dempster_agree() {
    assert_eq!(SURE.dempster(&SURE), Some(SURE));
}

// mitra: test t_dempster_contradict : dempster(Mass(1.0, 0.0, 0.0), Mass(0.0, 1.0, 0.0)) = None
#[test]
fn t_dempster_contradict() {
    assert_eq!(SURE.dempster(&SURE_WRONG), None, "total conflict");
}

// mitra: test t_dempster_contradict_swapped : dempster(Mass(0.0, 1.0, 0.0), Mass(1.0, 0.0, 0.0)) = None
#[test]
fn t_dempster_contradict_swapped() {
    assert_eq!(SURE_WRONG.dempster(&SURE), None, "total conflict");
}

// mitra: test t_dempster_with_nothing : dempster(Mass(1.0, 0.0, 0.0), Mass(0.0, 0.0, 1.0)) = Some(Mass(1.0, 0.0, 0.0))
#[test]
fn t_dempster_with_nothing() {
    assert_eq!(SURE.dempster(&Mass::VACUOUS), Some(SURE));
}

// mitra: test t_dempster_nothing_first : dempster(Mass(0.0, 0.0, 1.0), Mass(1.0, 0.0, 0.0)) = Some(Mass(1.0, 0.0, 0.0))
#[test]
fn t_dempster_nothing_first() {
    assert_eq!(Mass::VACUOUS.dempster(&SURE), Some(SURE));
}

// mitra: test t_dempster_nothing : dempster(Mass(0.0, 0.0, 1.0), Mass(0.0, 0.0, 1.0)) = Some(Mass(0.0, 0.0, 1.0))
#[test]
fn t_dempster_nothing() {
    assert_eq!(Mass::VACUOUS.dempster(&Mass::VACUOUS), Some(Mass::VACUOUS));
}

// `judged`, one input per test: a failed test then names its input.

// mitra: test t_judged_correct : judged(true, 0.25) = Some(Mass(0.25, 0.0, 0.75))
#[test]
fn t_judged_correct() {
    assert_eq!(Mass::judged(true, 0.25), Some(mass(0.25, 0.0, 0.75)));
}

// mitra: test t_judged_wrong : judged(false, 0.25) = Some(Mass(0.0, 0.25, 0.75))
#[test]
fn t_judged_wrong() {
    assert_eq!(Mass::judged(false, 0.25), Some(mass(0.0, 0.25, 0.75)));
}

// mitra: test t_judged_at_zero : judged(true, 0.0) = Some(Mass(0.0, 0.0, 1.0))
#[test]
fn t_judged_at_zero() {
    assert_eq!(Mass::judged(true, 0.0), Some(Mass::VACUOUS));
}

// mitra: test t_judged_at_one : judged(false, 1.0) = Some(Mass(0.0, 1.0, 0.0))
#[test]
fn t_judged_at_one() {
    assert_eq!(Mass::judged(false, 1.0), Some(SURE_WRONG));
}

// mitra: test t_judged_rejects_above : judged(true, 1.5) = None
#[test]
fn t_judged_rejects_above() {
    assert_eq!(Mass::judged(true, 1.5), None);
}

// mitra: test t_judged_rejects_below : judged(false, -0.5) = None
#[test]
fn t_judged_rejects_below() {
    assert_eq!(Mass::judged(false, -0.5), None);
}

// Not in the model: Mitra has no NaN literal (gap G4).
#[test]
fn t_judged_rejects_nan() {
    assert_eq!(Mass::judged(true, f64::NAN), None);
}

// Invalid masses, as the first argument of `yager` and the second of
// `dempster`: a negative part, or parts that do not sum to 1.

// mitra: test t_yager_rejects_negative : yager(Mass(-0.5, 0.5, 1.0), Mass(0.0, 0.0, 1.0)) = None
#[test]
fn t_yager_rejects_negative() {
    assert_eq!(mass(-0.5, 0.5, 1.0).yager(&Mass::VACUOUS), None);
}

// mitra: test t_yager_rejects_sum : yager(Mass(0.5, 0.5, 0.5), Mass(0.0, 0.0, 1.0)) = None
#[test]
fn t_yager_rejects_sum() {
    assert_eq!(mass(0.5, 0.5, 0.5).yager(&Mass::VACUOUS), None);
}

// `mitra next` asked for these two: the classes "second invalid" of
// `yager` and "first invalid" of `dempster` had no test.

// mitra: test t_yager_rejects_second : yager(Mass(0.0, 0.0, 1.0), Mass(0.5, 0.5, 0.5)) = None
#[test]
fn t_yager_rejects_second() {
    assert_eq!(Mass::VACUOUS.yager(&mass(0.5, 0.5, 0.5)), None);
}

// mitra: test t_dempster_rejects_first : dempster(Mass(-0.5, 0.5, 1.0), Mass(0.0, 0.0, 1.0)) = None
#[test]
fn t_dempster_rejects_first() {
    assert_eq!(mass(-0.5, 0.5, 1.0).dempster(&Mass::VACUOUS), None);
}

// mitra: test t_dempster_rejects_negative : dempster(Mass(0.0, 0.0, 1.0), Mass(-0.5, 0.5, 1.0)) = None
#[test]
fn t_dempster_rejects_negative() {
    assert_eq!(Mass::VACUOUS.dempster(&mass(-0.5, 0.5, 1.0)), None);
}

// mitra: test t_dempster_rejects_sum : dempster(Mass(0.0, 0.0, 1.0), Mass(0.5, 0.5, 0.5)) = None
#[test]
fn t_dempster_rejects_sum() {
    assert_eq!(Mass::VACUOUS.dempster(&mass(0.5, 0.5, 0.5)), None);
}

// Not in the model: no NaN literal (gap G4).
#[test]
fn t_rejects_nan_mass() {
    let m = mass(f64::NAN, 0.0, 1.0);
    assert_eq!(m.yager(&Mass::VACUOUS), None);
    assert_eq!(Mass::VACUOUS.dempster(&m), None);
}

// mitra: test t_valid_yes : is_valid(Mass(0.5, 0.0, 0.5)) = true
#[test]
fn t_valid_yes() {
    assert!(mass(0.5, 0.0, 0.5).is_valid());
}

// Within EPSILON of a sum of 1.
// mitra: test t_valid_slack : is_valid(Mass(0.5, 0.0, 0.5000000001)) = true
#[test]
fn t_valid_slack() {
    assert!(mass(0.5, 0.0, 0.5000000001).is_valid());
}

// mitra: test t_valid_negative : is_valid(Mass(-0.5, 0.5, 1.0)) = false
#[test]
fn t_valid_negative() {
    assert!(!mass(-0.5, 0.5, 1.0).is_valid());
}

// mitra: test t_valid_sum : is_valid(Mass(0.5, 0.5, 0.5)) = false
#[test]
fn t_valid_sum() {
    assert!(!mass(0.5, 0.5, 0.5).is_valid());
}

// mitra: test t_conflict_total : conflict(Mass(1.0, 0.0, 0.0), Mass(0.0, 1.0, 0.0)) = 1.0
#[test]
fn t_conflict_total() {
    assert_eq!(SURE.conflict(&SURE_WRONG), 1.0);
}

// mitra: test t_conflict_mixed : conflict(Mass(0.5, 0.25, 0.25), Mass(0.25, 0.5, 0.25)) = 0.3125
#[test]
fn t_conflict_mixed() {
    assert_eq!(
        mass(0.5, 0.25, 0.25).conflict(&mass(0.25, 0.5, 0.25)),
        0.3125
    );
}

// `mitra next` asked for these: the claims that relate `yager` and
// `dempster` to `conflict` need the conflict on the inputs the rules are
// tested on.

// mitra: test t_conflict_none : conflict(Mass(1.0, 0.0, 0.0), Mass(1.0, 0.0, 0.0)) = 0.0
#[test]
fn t_conflict_none() {
    assert_eq!(SURE.conflict(&SURE), 0.0);
}

// mitra: test t_conflict_invalid_first : conflict(Mass(-0.5, 0.5, 1.0), Mass(0.0, 0.0, 1.0)) = 0.0
#[test]
fn t_conflict_invalid_first() {
    assert_eq!(mass(-0.5, 0.5, 1.0).conflict(&Mass::VACUOUS), 0.0);
}

// mitra: test t_conflict_invalid_second : conflict(Mass(0.0, 0.0, 1.0), Mass(0.5, 0.5, 0.5)) = 0.0
#[test]
fn t_conflict_invalid_second() {
    assert_eq!(Mass::VACUOUS.conflict(&mass(0.5, 0.5, 0.5)), 0.0);
}
