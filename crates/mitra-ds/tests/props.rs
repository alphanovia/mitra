//! The tests named in `.mitra/model.mitra`. Each one checks the properties
//! listed there on a range of numbered cases.

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

/// Cases 1000 to 1003: the extremes of Mitra's golden sets.
fn extreme(n: u64) -> (Mass, Mass) {
    let m = |ok, c| Mass::judged(ok, c).unwrap();
    match n {
        1000 => (m(true, 1.0), m(true, 1.0)),  // agreement at 1
        1001 => (m(true, 1.0), m(false, 1.0)), // contradiction at 1
        1002 => (m(true, 1.0), m(false, 0.0)), // evidence at 0 brings nothing
        1003 => (m(true, 0.0), m(false, 0.0)), // no evidence
        _ => unreachable!(),
    }
}

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

#[test]
fn t_yager_extremes() {
    let y = |n| {
        let (a, b) = extreme(n);
        let m = a.yager(&b).unwrap().0;
        assert_eq!(Some(m), b.yager(&a).map(|r| r.0), "commutative");
        m
    };
    assert_eq!(
        y(1000),
        Mass {
            b: 1.0,
            d: 0.0,
            u: 0.0
        }
    );
    assert_eq!(
        y(1001),
        Mass::VACUOUS,
        "a contradiction at 1 leaves the input unknown"
    );
    assert_eq!(
        y(1002),
        Mass {
            b: 1.0,
            d: 0.0,
            u: 0.0
        }
    );
    assert_eq!(y(1003), Mass::VACUOUS);
}

#[test]
fn t_dempster_extremes() {
    let d = |n| {
        let (a, b) = extreme(n);
        let m = a.dempster(&b);
        assert_eq!(m, b.dempster(&a), "commutative");
        m
    };
    assert_eq!(
        d(1000),
        Some(Mass {
            b: 1.0,
            d: 0.0,
            u: 0.0
        })
    );
    assert_eq!(
        d(1001),
        None,
        "total conflict: Dempster's rule is undefined"
    );
    assert_eq!(
        d(1002),
        Some(Mass {
            b: 1.0,
            d: 0.0,
            u: 0.0
        })
    );
    assert_eq!(d(1003), Some(Mass::VACUOUS));
}

#[test]
fn t_judged_rejects_bad_confidence() {
    assert_eq!(Mass::judged(true, 1.5), None);
    assert_eq!(Mass::judged(true, f64::NAN), None);
}

#[test]
fn t_rejects_invalid_masses() {
    let bad = [
        Mass {
            b: f64::NAN,
            d: 0.0,
            u: 1.0,
        },
        Mass {
            b: -0.5,
            d: 0.5,
            u: 1.0,
        },
        Mass {
            b: 0.5,
            d: 0.5,
            u: 0.5,
        },
    ];
    for m in bad {
        assert_eq!(m.yager(&Mass::VACUOUS), None);
        assert_eq!(Mass::VACUOUS.dempster(&m), None);
    }
}
