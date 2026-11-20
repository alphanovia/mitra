//! The side of trust (`model/model.mitra`). It reads nothing of the
//! other side.
//!
//! - [`place`]: a location in the system;
//! - [`piece`]: something said, right or wrong, and the opinion of it;
//! - the three kinds of piece: [`evidence`], [`statement`], [`tie`];
//! - here, `model`: all the pieces put together, what is believed of
//!   each from the whole, and which cannot all be right.

pub mod evidence;
pub mod piece;
pub mod place;
pub mod statement;
pub mod tie;

pub use evidence::{apart, conflict, fuse, moved, Evidence, Voice};
pub use piece::{against, no, nothing, piece_id, projected, reaches, yes, Opinion, Piece};
pub use place::{within, Place};
pub use statement::Statement;
pub use tie::{concerned, places, Tie};

use std::collections::BTreeSet;

/// How pieces stand to each other. One relation can hold many pieces.
// mitra: implements model::Relation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    /// The first is all of the others together: believed as their
    /// product; a yes about it flows down to each, a no is shared.
    All,
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
///
/// A relation is itself said: a statement whose content is the
/// relation. What is computed on it counts as far as that statement is
/// believed; one believed at 1 is plain structure.
// mitra: implements model::Model
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Model {
    pub evidence: Vec<Evidence>,
    pub related: Vec<Statement<Related>>,
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
/// itself: it is then taken with no evidence, or as a part of nothing.
#[derive(Default)]
struct Walk {
    judged: Vec<Piece>,
    resting: Vec<Piece>,
    wholes: Vec<Piece>,
}

/// What is believed of `x`, from the whole model.
// mitra: implements model::opinion
pub fn opinion(model: &Model, x: &Piece) -> Opinion {
    let mut walk = Walk::default();
    // Evidence moves b, d and u; the base rate stays that of the piece.
    Opinion {
        a: base(model, x),
        ..believed(model, x, &mut walk)
    }
}

/// `x` is believed enough for the policy.
// mitra: implements model::trusted
pub fn trusted(model: &Model, x: &Piece, policy: f64) -> bool {
    reaches(opinion(model, x), policy)
}

/// What `x` stands on, taken together, as far as the relations that
/// lead there are believed.
fn believed(model: &Model, x: &Piece, walk: &mut Walk) -> Opinion {
    let (on, trust) = support(model, x, walk);
    moved(nothing(trust), together(model, &on, walk))
}

/// The relations of one kind, each with how far its statement is
/// believed.
fn related<'a>(model: &'a Model, how: Relation, walk: &mut Walk) -> Vec<(&'a Related, f64)> {
    let mut found = Vec::new();
    for said in model.related.iter().filter(|said| said.content.how == how) {
        found.push((&said.content, projected(direct(model, &said.piece, walk))));
    }
    found
}

fn believing(b: f64) -> Opinion {
    Opinion {
        b,
        d: 0.0,
        u: 1.0 - b,
        a: 0.0,
    }
}
fn disbelieving(d: f64) -> Opinion {
    Opinion {
        b: 0.0,
        d,
        u: 1.0 - d,
        a: 0.0,
    }
}

/// Everything said of `x`, each with its voice: by the evidence about
/// it, and by the evidence about each whole it is a part of.
///
/// - A yes flows down: what believes a whole believes each part as much.
/// - A no is shared: what disbelieves a whole disbelieves a part as far
///   as the other parts are believed on other evidence.
///
/// Without `blame`, only what speaks for `x`.
fn heard(model: &Model, x: &Piece, walk: &mut Walk, blame: bool) -> Vec<(Opinion, Voice)> {
    let mut said = Vec::new();
    for e in model.evidence.iter().filter(|e| &e.about == x) {
        let says = moved(believed(model, &e.piece, walk), e.value);
        said.push((if blame { says } else { believing(says.b) }, e.by.clone()));
    }
    for (r, trust) in related(model, Relation::All, walk) {
        if r.pieces.len() < 2 || !r.pieces[1..].contains(x) {
            continue;
        }
        let whole = &r.pieces[0];
        if walk.wholes.contains(whole) {
            continue;
        }
        walk.wholes.push(whole.clone());
        for (says, by) in heard(model, whole, walk, blame) {
            if trust * says.b > 0.0 {
                said.push((believing(trust * says.b), by.clone()));
            }
            if blame && says.d > 0.0 {
                let mut others = 1.0;
                for part in r.pieces[1..].iter().filter(|part| *part != x) {
                    others *= fused(heard(model, part, walk, false)).map_or(0.0, |o| o.b);
                }
                if trust * says.d * others > 0.0 {
                    said.push((disbelieving(trust * says.d * others), by));
                }
            }
        }
        walk.wholes.pop();
    }
    said
}

/// Several opinions on one piece, each as independent as its voice is
/// from the nearest one already heard.
fn fused(said: Vec<(Opinion, Voice)>) -> Option<Opinion> {
    let mut so_far: Option<Opinion> = None;
    let mut voices: Vec<Voice> = Vec::new();
    for (says, by) in said {
        let mut i = 1.0;
        for voice in &voices {
            i = f64::min(i, apart(voice, &by));
        }
        so_far = Some(match so_far {
            None => says,
            Some(o) => fuse(o, says, i),
        });
        voices.push(by);
    }
    so_far
}

/// What is said of `x` itself, fused.
fn direct(model: &Model, x: &Piece, walk: &mut Walk) -> Opinion {
    let a = base(model, x);
    if walk.judged.contains(x) {
        return nothing(a);
    }
    walk.judged.push(x.clone());
    let said = heard(model, x, walk, true);
    walk.judged.pop();
    Opinion {
        a,
        ..fused(said).unwrap_or(nothing(a))
    }
}

/// The pieces `x` stands on, each taken on what is said of it: itself,
/// or its parts, whichever is more certain. A set, so a piece reached by
/// two paths is there once. With it, how far the relations that lead to
/// those pieces are believed.
fn support(model: &Model, x: &Piece, walk: &mut Walk) -> (BTreeSet<Piece>, f64) {
    let mut best = (BTreeSet::from([x.clone()]), 1.0);
    if walk.resting.contains(x) {
        return best;
    }
    walk.resting.push(x.clone());
    for (r, trust) in related(model, Relation::All, walk) {
        if r.pieces.first() != Some(x) {
            continue;
        }
        let mut on = (BTreeSet::new(), trust);
        for part in &r.pieces[1..] {
            let (pieces, through) = support(model, part, walk);
            on.0.extend(pieces);
            on.1 *= through;
        }
        let new = moved(nothing(on.1), together(model, &on.0, walk));
        let old = moved(nothing(best.1), together(model, &best.0, walk));
        if new.u < old.u {
            best = on;
        }
    }
    walk.resting.pop();
    best
}

/// All of these pieces, each on what is said of it.
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

/// The pieces that cannot all be right, the least believed first:
/// the parts of a whole and the evidence that says no about it, the
/// evidence for and against one piece, and the pieces of an `Exclude`
/// that are each believed.
// mitra: implements model::conflicts
pub fn conflicts(model: &Model) -> Vec<Vec<Piece>> {
    let mut out = Vec::new();
    for said in model
        .related
        .iter()
        .filter(|said| said.content.how == Relation::All)
    {
        let (whole, parts) = (
            said.content.pieces.first(),
            said.content.pieces.iter().skip(1),
        );
        for e in model.evidence.iter().filter(|e| Some(&e.about) == whole) {
            if moved(opinion(model, &e.piece), e.value).d > 0.0 {
                let mut suspects: Vec<Piece> = parts.clone().cloned().collect();
                suspects.push(e.piece.clone());
                // The relation is a suspect when it can be doubted.
                if projected(opinion(model, &said.piece)) < 1.0 {
                    suspects.push(said.piece.clone());
                }
                out.push(ordered(model, suspects));
            }
        }
    }
    for said in model
        .related
        .iter()
        .filter(|said| said.content.how == Relation::Exclude)
    {
        let r = &said.content;
        if projected(opinion(model, &said.piece)) > 0.0
            && r.pieces.iter().all(|p| opinion(model, p).b > 0.0)
        {
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
