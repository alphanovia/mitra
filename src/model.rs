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
pub use piece::{
    against, no, nothing, piece_id, projected, reaches, uncertainty, weight, with_weight, yes,
    Opinion, Piece,
};
pub use place::{within, Place};
pub use statement::Statement;
pub use tie::{concerned, places, Tie};

use std::collections::{BTreeMap, BTreeSet};

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

/// An opinion as the two parameters of a Beta: yes and no, each with
/// its part of the weight of the base rate.
fn beta(o: Opinion) -> (f64, f64) {
    (o.r + o.a * weight(), o.s + (1.0 - o.a) * weight())
}

/// The mean of a Beta and the mean of its square.
fn moments((alpha, beta): (f64, f64)) -> (f64, f64) {
    let n = alpha + beta;
    (alpha / n, alpha * (alpha + 1.0) / (n * (n + 1.0)))
}

/// Both of two pieces. Each is how often it is right, a Beta; the two
/// taken as independent, how often both are right is their product,
/// and the answer is the Beta with the mean and the variance of that
/// product. Its base rate is the product of the two.
// mitra: implements model::all
pub fn all(x: Opinion, y: Opinion) -> Opinion {
    let a = x.a * y.a;
    let (bx, by) = (beta(x), beta(y));
    if bx.0 == 0.0 || by.0 == 0.0 {
        // One is surely wrong: so is the whole.
        return Opinion {
            r: 0.0,
            s: f64::max(x.s, y.s),
            a,
        };
    }
    if bx.1 == 0.0 {
        // One is surely right: the whole is the other.
        return Opinion { a, ..y };
    }
    if by.1 == 0.0 {
        return Opinion { a, ..x };
    }
    let ((m1, q1), (m2, q2)) = (moments(bx), moments(by));
    let mean = m1 * m2;
    let variance = q1 * q2 - mean * mean;
    // The counts of the Beta that fits, and never fewer than the base
    // rate alone would give.
    let mut n = mean * (1.0 - mean) / variance - 1.0;
    n = f64::max(n, weight() * a / mean);
    n = f64::max(n, weight() * (1.0 - a) / (1.0 - mean));
    Opinion {
        r: f64::max(0.0, mean * n - a * weight()),
        s: f64::max(0.0, (1.0 - mean) * n - (1.0 - a) * weight()),
        a,
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
    memo: Memo,
}

/// What one walk has already worked out, not to work it out again: how
/// far each relation is believed (before any no, and after), what each
/// piece read as before any no, and the failures of the model.
#[derive(Default)]
struct Memo {
    trust: BTreeMap<(Piece, bool), f64>,
    prior: BTreeMap<Piece, Opinion>,
    failures: Option<Vec<(usize, usize)>>,
}

/// What is said of `x` itself before any no is shared.
fn prior(model: &Model, x: &Piece, walk: &mut Walk) -> Opinion {
    if let Some(o) = walk.memo.prior.get(x) {
        return *o;
    }
    let mut before = Walk {
        before: true,
        memo: std::mem::take(&mut walk.memo),
        ..Walk::default()
    };
    let o = direct(model, x, &mut before);
    walk.memo = before.memo;
    walk.memo.prior.insert(x.clone(), o);
    o
}

/// What is believed of `x`, from the whole model.
// mitra: implements model::opinion
pub fn opinion(model: &Model, x: &Piece) -> Opinion {
    let mut walk = Walk::default();
    believed(model, x, &mut walk)
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
        let key = (said.piece.clone(), walk.before);
        let trust = match walk.memo.trust.get(&key) {
            Some(trust) => *trust,
            None => {
                let trust = projected(direct(model, &said.piece, walk));
                walk.memo.trust.insert(key, trust);
                trust
            }
        };
        found.push((&said.content, trust));
    }
    found
}

fn believing(r: f64) -> Opinion {
    Opinion { r, s: 0.0, a: 0.0 }
}
fn disbelieving(s: f64) -> Opinion {
    Opinion { r: 0.0, s, a: 0.0 }
}

/// Everything said of `x`, each with its voice: by the evidence about
/// it, by the evidence about each whole it is a part of, and by each
/// failure it is one of the suspects of.
///
/// - A yes flows down: what believes a whole believes each part as far
///   as the relation is believed.
/// - A no is shared: see `blamed`.
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
            if trust * says.r > 0.0 {
                said.push((believing(trust * says.r), by));
            }
        }
        walk.wholes.pop();
    }
    if !walk.before {
        for (relation, e) in failures(model, walk) {
            let no = blamed(model, relation, e, x, walk) * e.value.s;
            if no > 0.0 {
                said.push((disbelieving(no), e.by.clone()));
            }
        }
    }
    said
}

/// Each failure of a model: evidence that says no about a whole, with
/// the relation that makes it a whole.
fn failures<'a>(model: &'a Model, walk: &mut Walk) -> Vec<(&'a Statement<Related>, &'a Evidence)> {
    if walk.memo.failures.is_none() {
        let mut found = Vec::new();
        for (r, relation) in model.related.iter().enumerate() {
            if relation.content.how != Relation::All {
                continue;
            }
            let whole = relation.content.pieces.first();
            for (k, e) in model.evidence.iter().enumerate() {
                if Some(&e.about) == whole && e.value.s > 0.0 {
                    found.push((r, k));
                }
            }
        }
        walk.memo.failures = Some(found);
    }
    let found = walk.memo.failures.iter().flatten();
    found
        .map(|(r, k)| (&model.related[*r], &model.evidence[*k]))
        .collect()
}

/// The suspects of a failure: the parts of the whole, the evidence that
/// says no, and the relation when it can be doubted. They cannot all be
/// right. A relation believed at 1 is structure, and not a suspect.
fn suspects(
    model: &Model,
    relation: &Statement<Related>,
    e: &Evidence,
    walk: &mut Walk,
) -> Vec<Piece> {
    let mut group: Vec<Piece> = relation.content.pieces.iter().skip(1).cloned().collect();
    group.push(e.piece.clone());
    if projected(prior(model, &relation.piece, walk)) < 1.0 {
        group.push(relation.piece.clone());
    }
    group
}

/// The count of no that one failure puts on `x`. A failure is seeing
/// that the suspects are not all right. Each suspect is how often it
/// is right, a Beta, from what was said of it before any no; with `q`
/// the chance that all the others are right, Bayes gives what to
/// expect of `x` afterwards:
///
/// ```text
/// mean after = (mean - q · mean of the square) / (1 - q · mean)
/// ```
///
/// and `x` gets the count of no that brings it to that mean. Its count
/// of yes is not touched. When every suspect was sure, the no is
/// shared equally.
fn blamed(
    model: &Model,
    relation: &Statement<Related>,
    e: &Evidence,
    x: &Piece,
    walk: &mut Walk,
) -> f64 {
    let parts = &relation.content.pieces;
    if x != &e.piece && x != &relation.piece && !parts.iter().skip(1).any(|part| part == x) {
        return 0.0;
    }
    let group = suspects(model, relation, e, walk);
    let Some(at) = group.iter().position(|suspect| suspect == x) else {
        return 0.0;
    };
    let mut read = Vec::new();
    for suspect in &group {
        read.push(beta(prior(model, suspect, walk)));
    }
    let mut others = 1.0;
    for (k, suspect) in read.iter().enumerate() {
        if k != at {
            others *= moments(*suspect).0;
        }
    }
    let (alpha, beta) = read[at];
    let (mean, square) = moments(read[at]);
    if alpha == 0.0 {
        return 0.0; // surely wrong already
    }
    if beta == 0.0 {
        // Surely right: the others take it, unless they are sure too.
        return match others >= 1.0 {
            true => 1.0 / group.len() as f64,
            false => 0.0,
        };
    }
    let after = (mean - others * square) / (1.0 - others * mean);
    alpha / after - alpha / mean
}

/// Several opinions on one piece. One voice counts once; different
/// voices add up.
fn fused(said: Vec<(Opinion, Voice)>) -> Option<Opinion> {
    let mut voices: Vec<(Voice, Opinion)> = Vec::new();
    for (says, by) in said {
        match voices
            .iter_mut()
            .find(|(voice, _)| apart(voice, &by) == 0.0)
        {
            Some((_, o)) => *o = fuse(*o, says, 0.0),
            None => voices.push((by, says)),
        }
    }
    let mut each = voices.into_iter().map(|(_, o)| o);
    let first = each.next()?;
    Some(each.fold(first, |o, says| fuse(o, says, 1.0)))
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

/// The pieces `x` stands on: its parts when a relation says it is a
/// whole, and then their parts in turn, or itself when what is said of
/// it directly leaves less uncertain. A set, so a piece reached by two
/// paths is there once. With it, how far the relations that lead to
/// those pieces are believed.
fn support(model: &Model, x: &Piece, walk: &mut Walk) -> (BTreeSet<Piece>, f64) {
    let mut best = (BTreeSet::from([x.clone()]), 1.0);
    if walk.resting.contains(x) {
        return best;
    }
    walk.resting.push(x.clone());
    for (r, trust) in related(model, Relation::All, walk) {
        if r.pieces.first() != Some(x) || r.pieces.len() < 2 || trust <= 0.0 {
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
        if uncertainty(new) <= uncertainty(old) {
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
    let mut walk = Walk::default();
    for (relation, e) in failures(model, &mut walk) {
        // Evidence that was not believed before it failed accuses nothing.
        if projected(prior(model, &e.piece, &mut walk)) > 0.0 {
            let group = suspects(model, relation, e, &mut walk);
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
            && r.pieces.iter().all(|p| opinion(model, p).r > 0.0)
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
