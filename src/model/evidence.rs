//! How an opinion moves: a piece of evidence says an opinion about
//! another piece, and counts as far as it is itself believed.

use super::piece::{projected, Opinion, Piece};

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

/// What evidence says, as far as it is believed: its counts, scaled.
/// A yes stays a yes and a no stays a no, however little is believed.
// mitra: implements evidence::moved
pub fn moved(believed: Opinion, said: Opinion) -> Opinion {
    let p = projected(believed);
    Opinion {
        r: p * said.r,
        s: p * said.s,
        a: said.a,
    }
}

/// Two opinions on one piece, at independence `i`. At 1 the counts add
/// up. At 0 it is one voice, counted once: its strongest yes and its
/// strongest no.
// mitra: implements evidence::fuse
pub fn fuse(x: Opinion, y: Opinion, i: f64) -> Opinion {
    Opinion {
        r: f64::max(x.r, y.r) + i * f64::min(x.r, y.r),
        s: f64::max(x.s, y.s) + i * f64::min(x.s, y.s),
        a: x.a,
    }
}

/// How far two opinions on one piece contradict each other.
// mitra: implements evidence::conflict
pub fn conflict(x: Opinion, y: Opinion) -> f64 {
    x.r * y.s + x.s * y.r
}
