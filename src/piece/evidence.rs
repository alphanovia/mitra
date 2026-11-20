//! How an opinion moves: a piece of evidence says an opinion about
//! another piece, and counts as far as it is itself believed.

use super::{projected, Opinion, Piece};

/// Who says it.
// mitra: implements evidence::Voice
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Voice {
    pub name: String,
}

impl Voice {
    pub fn named(name: &str) -> Voice {
        Voice {
            name: name.to_string(),
        }
    }
}

/// A piece that says `value` about the piece `about`. `by` is its voice.
// mitra: implements evidence::Evidence
#[derive(Clone, Debug, PartialEq)]
pub struct Evidence {
    pub piece: Piece,
    pub about: Piece,
    pub value: Opinion,
    pub by: Voice,
}

/// How independent two voices are, from 0 (one voice) to 1.
///
/// Only the two ends for now: a voice is its name, not read further.
// mitra: implements evidence::apart
pub fn apart(a: &Voice, b: &Voice) -> f64 {
    match a == b {
        true => 0.0,
        false => 1.0,
    }
}

/// What evidence says, as far as it is believed.
// mitra: implements evidence::moved
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
// mitra: implements evidence::fuse
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

/// How far two opinions on one piece contradict each other.
// mitra: implements evidence::conflict
pub fn conflict(x: Opinion, y: Opinion) -> f64 {
    x.b * y.d + x.d * y.b
}
