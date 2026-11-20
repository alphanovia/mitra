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
/// With `before`, what was believed before any no was shared.
#[derive(Default)]
struct Walk {
    judged: Vec<Piece>,
    resting: Vec<Piece>,
    wholes: Vec<Piece>,
    before: bool,
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
/// it, by the evidence about each whole it is a part of, and by each
/// failure it is one of the suspects of.
///
/// - A yes flows down: what believes a whole believes each part as far
///   as the relation is believed.
/// - A no is shared: see `share`.
fn heard(model: &Model, x: &Piece, walk: &mut Walk) -> Vec<(Opinion, Voice)> {
    let mut said = Vec::new();
    for e in model.evidence.iter().filter(|e| &e.about == x) {
        said.push((
            moved(believed(model, &e.piece, walk), e.value),
            e.by.clone(),
        ));
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
        for (says, by) in heard(model, whole, walk) {
            if trust * says.b > 0.0 {
                said.push((believing(trust * says.b), by));
            }
        }
        walk.wholes.pop();
    }
    if !walk.before {
        for (relation, e) in failures(model) {
            let d = share(model, relation, e, x) * e.value.d;
            if d > 0.0 {
                said.push((disbelieving(d), e.by.clone()));
            }
        }
    }
    said
}

/// Each failure of a model: evidence that says no about a whole, with
/// the relation that makes it a whole.
fn failures(model: &Model) -> Vec<(&Statement<Related>, &Evidence)> {
    let mut found = Vec::new();
    let wholes = model
        .related
        .iter()
        .filter(|said| said.content.how == Relation::All);
    for relation in wholes {
        let whole = relation.content.pieces.first();
        let about = model.evidence.iter().filter(|e| Some(&e.about) == whole);
        for e in about.filter(|e| e.value.d > 0.0) {
            found.push((relation, e));
        }
    }
    found
}

/// The suspects of a failure: the parts of the whole, the evidence that
/// says no, and the relation when it can be doubted. They cannot all be
/// right. A relation believed at 1 is structure, and not a suspect.
fn suspects(model: &Model, relation: &Statement<Related>, e: &Evidence) -> Vec<Piece> {
    let mut group: Vec<Piece> = relation.content.pieces.iter().skip(1).cloned().collect();
    group.push(e.piece.clone());
    let mut before = Walk {
        before: true,
        ..Walk::default()
    };
    if projected(direct(model, &relation.piece, &mut before)) < 1.0 {
        group.push(relation.piece.clone());
    }
    group
}

/// The share of a failure that falls on `x`: how likely it is that `x`
/// alone is wrong, given that the suspects are not all right, each
/// taken at what it read as before the failure.
///
/// ```text
/// share(x) = (1 - p(x)) · product of p(others) / (1 - product of p(all))
/// ```
///
/// When every suspect read as 1, the shares are equal.
fn share(model: &Model, relation: &Statement<Related>, e: &Evidence, x: &Piece) -> f64 {
    let group = suspects(model, relation, e);
    let Some(at) = group.iter().position(|suspect| suspect == x) else {
        return 0.0;
    };
    let mut before = Walk {
        before: true,
        wholes: relation
            .content
            .pieces
            .first()
            .cloned()
            .into_iter()
            .collect(),
        ..Walk::default()
    };
    let mut read = Vec::new();
    for suspect in &group {
        read.push(projected(direct(model, suspect, &mut before)));
    }
    let all: f64 = read.iter().product();
    if all >= 1.0 {
        return 1.0 / group.len() as f64;
    }
    let mut others = 1.0;
    for (k, p) in read.iter().enumerate() {
        if k != at {
            others *= p;
        }
    }
    (1.0 - read[at]) * others / (1.0 - all)
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
    let said = heard(model, x, walk);
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
    for (relation, e) in failures(model) {
        // Evidence that was not believed before it failed accuses nothing.
        let mut before = Walk {
            before: true,
            ..Walk::default()
        };
        if projected(direct(model, &e.piece, &mut before)) > 0.0 {
            let group = suspects(model, relation, e);
            out.push(ordered(model, group));
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
