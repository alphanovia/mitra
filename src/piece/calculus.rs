//! The opinions themselves: plain arithmetic, no piece in sight.

use super::Opinion;

/// No evidence: the base rate alone.
// mitra: implements piece::nothing
pub fn nothing(a: f64) -> Opinion {
    Opinion {
        b: 0.0,
        d: 0.0,
        u: 1.0,
        a,
    }
}

// mitra: implements piece::yes
pub fn yes() -> Opinion {
    Opinion {
        b: 1.0,
        d: 0.0,
        u: 0.0,
        a: 1.0,
    }
}

// mitra: implements piece::no
pub fn no() -> Opinion {
    Opinion {
        b: 0.0,
        d: 1.0,
        u: 0.0,
        a: 0.0,
    }
}

/// The number shown.
// mitra: implements piece::projected
pub fn projected(o: Opinion) -> f64 {
    o.b + o.a * o.u
}

/// What evidence says, as far as it is believed.
// mitra: implements piece::moved
pub fn moved(believed: Opinion, said: Opinion) -> Opinion {
    let p = projected(believed);
    Opinion {
        b: p * said.b,
        d: p * said.d,
        u: 1.0 - p * (said.b + said.d),
        a: said.a,
    }
}

/// Two opinions on one piece, at independence `i`. At 1 they add up, at
/// 0 they are averaged. Both certain: their average.
// mitra: implements piece::fuse
pub fn fuse(x: Opinion, y: Opinion, i: f64) -> Opinion {
    if x.u == 0.0 && y.u == 0.0 {
        return Opinion {
            b: (x.b + y.b) / 2.0,
            d: (x.d + y.d) / 2.0,
            u: 0.0,
            a: x.a,
        };
    }
    let k = x.u + y.u - i * x.u * y.u;
    Opinion {
        b: (x.b * y.u + y.b * x.u) / k,
        d: (x.d * y.u + y.d * x.u) / k,
        u: (2.0 - i) * x.u * y.u / k,
        a: x.a,
    }
}

/// Both of two pieces: believed as the product.
// mitra: implements piece::all
pub fn all(x: Opinion, y: Opinion) -> Opinion {
    let b = x.b * y.b;
    let d = x.d + y.d - x.d * y.d;
    Opinion {
        b,
        d,
        u: 1.0 - b - d,
        a: x.a * y.a,
    }
}

/// Believed enough for the policy, and never with no belief at all.
// mitra: implements piece::reaches
pub fn reaches(o: Opinion, policy: f64) -> bool {
    o.b > 0.0 && projected(o) >= policy
}

/// The evidence against is at least as strong as the evidence for.
// mitra: implements piece::against
pub fn against(o: Opinion) -> bool {
    o.d > 0.0 && o.d >= o.b
}

/// How far two opinions on one piece contradict each other.
// mitra: implements piece::conflict
pub fn conflict(x: Opinion, y: Opinion) -> f64 {
    x.b * y.d + x.d * y.b
}
