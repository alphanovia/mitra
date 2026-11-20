//! Something said, right or wrong, and the opinion of it.

/// Something said, right or wrong. Known by its name only.
// mitra: implements piece::Piece
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Piece {
    pub name: String,
}

impl Piece {
    pub fn named(name: &str) -> Piece {
        Piece {
            name: name.to_string(),
        }
    }
}

// mitra: implements piece::piece_id
pub fn piece_id(x: &Piece) -> &str {
    &x.name
}

/// What is believed of a piece. `b + d + u == 1`; `a` is the base rate:
/// what to expect with no evidence.
// mitra: implements piece::Opinion
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opinion {
    pub b: f64,
    pub d: f64,
    pub u: f64,
    pub a: f64,
}

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

/// The number an opinion reads as.
// mitra: implements piece::projected
pub fn projected(o: Opinion) -> f64 {
    o.b + o.a * o.u
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
