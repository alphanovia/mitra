//! From everything known to the opinion of one piece.

use super::calculus::{all, conflict, fuse, moved, nothing, projected, reaches};
use super::{apart, Known, Opinion, Piece, Relation};
use std::collections::BTreeSet;

impl Known {
    fn base(&self, x: &Piece) -> f64 {
        let own = self.base.iter().find(|(p, _)| p == x);
        own.map_or(0.0, |(_, a)| *a)
    }
}

/// The pieces being worked out, to stop at one that leads back to
/// itself: it is then taken with no evidence, or as resting on nothing.
#[derive(Default)]
struct Walk {
    judged: Vec<Piece>,
    resting: Vec<Piece>,
}

/// What is believed of `x`, from all that is known.
// mitra: implements piece::opinion
pub fn opinion(known: &Known, x: &Piece) -> Opinion {
    let mut walk = Walk::default();
    let on = support(known, x, &mut walk);
    together(known, &on, &mut walk)
}

/// `x` is believed enough for the policy.
// mitra: implements piece::trusted
pub fn trusted(known: &Known, x: &Piece, policy: f64) -> bool {
    reaches(opinion(known, x), policy)
}

/// What the evidence about `x` says of it, and nothing else.
fn direct(known: &Known, x: &Piece, walk: &mut Walk) -> Opinion {
    let a = known.base(x);
    if walk.judged.contains(x) {
        return nothing(a);
    }
    walk.judged.push(x.clone());
    let mut so_far: Option<Opinion> = None;
    let mut voices: Vec<&str> = Vec::new();
    for e in known.evidence.iter().filter(|e| &e.about == x) {
        let on = support(known, &e.piece, walk);
        let says = moved(together(known, &on, walk), e.value);
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
fn support(known: &Known, x: &Piece, walk: &mut Walk) -> BTreeSet<Piece> {
    let mut best = BTreeSet::from([x.clone()]);
    if walk.resting.contains(x) {
        return best;
    }
    walk.resting.push(x.clone());
    let rests = known.related.iter().filter(|r| r.how == Relation::RestsOn);
    for r in rests.filter(|r| r.pieces.first() == Some(x)) {
        let mut on = BTreeSet::new();
        for part in &r.pieces[1..] {
            on.extend(support(known, part, walk));
        }
        if projected(together(known, &on, walk)) > projected(together(known, &best, walk)) {
            best = on;
        }
    }
    walk.resting.pop();
    best
}

/// All of these pieces, each on its own evidence.
fn together(known: &Known, on: &BTreeSet<Piece>, walk: &mut Walk) -> Opinion {
    let mut each = on.iter().map(|p| direct(known, p, walk));
    let first = each.next().unwrap_or(nothing(0.0));
    each.fold(first, all)
}

/// The pieces that cannot all be right and are each believed, the least
/// believed first: the pieces of an `Exclude`, or the evidence for and
/// against one piece.
// mitra: implements piece::conflicts
pub fn conflicts(known: &Known) -> Vec<Vec<Piece>> {
    let mut out = Vec::new();
    let excluded = known.related.iter().filter(|r| r.how == Relation::Exclude);
    for r in excluded {
        if r.pieces.iter().all(|p| opinion(known, p).b > 0.0) {
            out.push(ordered(known, r.pieces.clone()));
        }
    }
    let mut seen = BTreeSet::new();
    for e in &known.evidence {
        if !seen.insert(&e.about) {
            continue;
        }
        let about: Vec<_> = known
            .evidence
            .iter()
            .filter(|o| o.about == e.about)
            .collect();
        let said = |o: &&super::Evidence| moved(opinion(known, &o.piece), o.value);
        let clash = about
            .iter()
            .any(|x| about.iter().any(|y| conflict(said(x), said(y)) > 0.0));
        if clash {
            out.push(ordered(
                known,
                about.iter().map(|o| o.piece.clone()).collect(),
            ));
        }
    }
    out
}

fn ordered(known: &Known, mut pieces: Vec<Piece>) -> Vec<Piece> {
    let p = |x: &Piece| projected(opinion(known, x));
    pieces.sort_by(|x, y| p(x).total_cmp(&p(y)));
    pieces
}
