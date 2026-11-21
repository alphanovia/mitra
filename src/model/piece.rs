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
/// counted `w` times. `w` is the weight of what is expected before any
/// evidence, in counts: a setting of the whole system, given by the
/// model, never chosen here.
// mitra: implements piece::projected
pub fn projected(o: Opinion, w: f64) -> f64 {
    (o.r + o.a * w) / (o.r + o.s + w)
}
