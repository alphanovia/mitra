//! All the pieces put together: what is believed of each from the
//! whole, and which cannot all be right.

use super::evidence::{apart, conflict, fuse, moved, Evidence, Voice};
use super::{nothing, projected, reaches, Opinion, Piece};
use std::collections::BTreeSet;

/// How pieces stand to each other. One relation can hold many pieces.
// mitra: implements model::Relation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    /// The first holds when all the others do.
    RestsOn,
    /// They cannot all be right.
    Exclude,
}

// mitra: implements model::Related
#[derive(Clone, Debug, PartialEq)]
pub struct Related {
    pub how: Relation,
    pub pieces: Vec<Piece>,
}

/// All that is said of a system: the evidence, the relations between
/// pieces, and the base rate of each piece that has one. A piece with
/// none has 0: nothing is assumed.
// mitra: implements model::Model
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Model {
    pub evidence: Vec<Evidence>,
    pub related: Vec<Related>,
    pub base: Vec<(Piece, f64)>,
}

/// Both of two pieces: believed as the product.
// mitra: implements model::all
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

/// What to expect of a piece with no evidence: its base rate, or 0.
// mitra: implements model::base
pub fn base(model: &Model, x: &Piece) -> f64 {
    let own = model.base.iter().find(|(p, _)| p == x);
    own.map_or(0.0, |(_, a)| *a)
}

/// The pieces being worked out, to stop at one that leads back to
/// itself: it is then taken with no evidence, or as resting on nothing.
#[derive(Default)]
struct Walk {
    judged: Vec<Piece>,
    resting: Vec<Piece>,
}

/// What is believed of `x`, from the whole model.
// mitra: implements model::opinion
pub fn opinion(model: &Model, x: &Piece) -> Opinion {
    let mut walk = Walk::default();
    let on = support(model, x, &mut walk);
    // Evidence moves b, d and u; the base rate stays that of the piece.
    Opinion {
        a: base(model, x),
        ..together(model, &on, &mut walk)
    }
}

/// `x` is believed enough for the policy.
// mitra: implements model::trusted
pub fn trusted(model: &Model, x: &Piece, policy: f64) -> bool {
    reaches(opinion(model, x), policy)
}

/// What the evidence about `x` says of it, and nothing else.
fn direct(model: &Model, x: &Piece, walk: &mut Walk) -> Opinion {
    let a = base(model, x);
    if walk.judged.contains(x) {
        return nothing(a);
    }
    walk.judged.push(x.clone());
    let mut so_far: Option<Opinion> = None;
    let mut voices: Vec<&Voice> = Vec::new();
    for e in model.evidence.iter().filter(|e| &e.about == x) {
        let on = support(model, &e.piece, walk);
        let says = moved(together(model, &on, walk), e.value);
        // As independent as its voice is from the nearest one heard.
        let mut i = 1.0;
        for voice in &voices {
            i = f64::min(i, apart(voice, &e.by));
        }
        so_far = Some(match so_far {
            None => says,
            Some(o) => fuse(o, says, i),
        });
        voices.push(&e.by);
    }
    walk.judged.pop();
    Opinion {
        a,
        ..so_far.unwrap_or(nothing(a))
    }
}

/// The pieces `x` stands on, each taken on its own evidence: itself, or
/// the pieces it rests on, whichever is believed more. A set, so a piece
/// reached by two paths is there once.
fn support(model: &Model, x: &Piece, walk: &mut Walk) -> BTreeSet<Piece> {
    let mut best = BTreeSet::from([x.clone()]);
    if walk.resting.contains(x) {
        return best;
    }
    walk.resting.push(x.clone());
    let rests = model.related.iter().filter(|r| r.how == Relation::RestsOn);
    for r in rests.filter(|r| r.pieces.first() == Some(x)) {
        let mut on = BTreeSet::new();
        for part in &r.pieces[1..] {
            on.extend(support(model, part, walk));
        }
        if projected(together(model, &on, walk)) > projected(together(model, &best, walk)) {
            best = on;
        }
    }
    walk.resting.pop();
    best
}

/// All of these pieces, each on its own evidence.
fn together(model: &Model, on: &BTreeSet<Piece>, walk: &mut Walk) -> Opinion {
    let mut whole: Option<Opinion> = None;
    for part in on {
        let own = direct(model, part, walk);
        whole = Some(match whole {
            None => own,
            Some(o) => all(o, own),
        });
    }
    whole.unwrap_or(nothing(0.0))
}

/// The pieces that cannot all be right and are each believed, the least
/// believed first: the pieces of an `Exclude`, or the evidence for and
/// against one piece.
// mitra: implements model::conflicts
pub fn conflicts(model: &Model) -> Vec<Vec<Piece>> {
    let mut out = Vec::new();
    let excluded = model.related.iter().filter(|r| r.how == Relation::Exclude);
    for r in excluded {
        if r.pieces.iter().all(|p| opinion(model, p).b > 0.0) {
            out.push(ordered(model, r.pieces.clone()));
        }
    }
    let mut seen = BTreeSet::new();
    for e in &model.evidence {
        if !seen.insert(&e.about) {
            continue;
        }
        let about: Vec<_> = model
            .evidence
            .iter()
            .filter(|o| o.about == e.about)
            .collect();
        let said = |o: &&Evidence| moved(opinion(model, &o.piece), o.value);
        let clash = about
            .iter()
            .any(|x| about.iter().any(|y| conflict(said(x), said(y)) > 0.0));
        if clash {
            out.push(ordered(
                model,
                about.iter().map(|o| o.piece.clone()).collect(),
            ));
        }
    }
    out
}

fn ordered(model: &Model, mut pieces: Vec<Piece>) -> Vec<Piece> {
    let p = |x: &Piece| projected(opinion(model, x));
    pieces.sort_by(|x, y| p(x).total_cmp(&p(y)));
    pieces
}
