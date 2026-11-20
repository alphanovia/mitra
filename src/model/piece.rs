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

/// The weight of what is expected before any evidence, in counts. A
/// setting of the whole system, not of one opinion. Small, so that the
/// first observation nearly settles a piece: one sure yes on one half
/// reads 0.92.
// mitra: implements piece::weight
pub fn weight() -> f64 {
    0.2
}

/// What is believed of a piece, as counts: `r` of yes, `s` of no. `a`
/// is the base rate: what to expect with no evidence.
// mitra: implements piece::Opinion
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Opinion {
    pub r: f64,
    pub s: f64,
    pub a: f64,
}

/// No evidence: the base rate alone.
// mitra: implements piece::nothing
pub fn nothing(a: f64) -> Opinion {
    Opinion { r: 0.0, s: 0.0, a }
}

/// One yes.
// mitra: implements piece::yes
pub fn yes() -> Opinion {
    Opinion {
        r: 1.0,
        s: 0.0,
        a: 1.0,
    }
}

/// One no.
// mitra: implements piece::no
pub fn no() -> Opinion {
    Opinion {
        r: 0.0,
        s: 1.0,
        a: 0.0,
    }
}

/// The number an opinion reads as: the share of yes, the base rate
/// counted `weight()` times.
// mitra: implements piece::projected
pub fn projected(o: Opinion) -> f64 {
    (o.r + o.a * weight()) / (o.r + o.s + weight())
}

/// How much of an opinion is not evidence: 1 with none, towards 0 as
/// the counts grow.
// mitra: implements piece::uncertainty
pub fn uncertainty(o: Opinion) -> f64 {
    weight() / (o.r + o.s + weight())
}

/// Believed enough for the policy, and never with no yes at all.
// mitra: implements piece::reaches
pub fn reaches(o: Opinion, policy: f64) -> bool {
    o.r > 0.0 && projected(o) >= policy
}

/// The evidence against is at least as strong as the evidence for.
// mitra: implements piece::against
pub fn against(o: Opinion) -> bool {
    o.s > 0.0 && o.s >= o.r
}
