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

pub use evidence::{apart, conflict, fuse, held, moved, reliable, Evidence, Reliable, Voice};
pub use piece::{
    against, belief, disbelief, interval, no, nothing, piece_id, projected, reaches, uncertainty,
    yes, Opinion, Piece,
};
pub use place::{within, Place};
pub use statement::Statement;
pub use tie::{complete, concerned, places, whole, Complete, Tie};

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
/// pieces, the base rate of each piece that has one, the ties, and the
/// settings it is read under. A piece with no base rate, written or
/// set, has 0. The settings are given: the core holds no value for
/// them.
///
/// A relation is itself said: a statement whose content is the
/// relation. What is computed on it counts as far as that statement is
/// believed; one believed at 1 is plain structure.
///
/// Each tie implies its completeness, a part of every whole the tie is
/// a part of, and its whole: the tie and its statements together,
/// which is what an observation at the place is about.
// mitra: implements model::Model
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub evidence: Vec<Evidence>,
    pub related: Vec<Statement<Related>>,
    pub base: Vec<(Piece, f64)>,
    pub ties: Vec<Tie>,
    pub settings: Settings,
}

/// What a model is read under, apart from what it says: the weight of
/// a base rate, in counts, and the base rate of each kind of piece
/// when nobody wrote one. The core holds no value for any of them.
// mitra: implements model::Settings
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// The weight of a base rate, in counts.
    pub weight: f64,
    /// The base rate of a statement: any piece that is none of the
    /// kinds below.
    pub statement: f64,
    /// The base rate of a piece of evidence.
    pub evidence: f64,
    /// The base rate of a tie.
    pub tie: f64,
    /// The base rate of the completeness of a tie.
    pub completeness: f64,
    /// The base rate of the reliability of a voice. A voice sure to be
    /// reliable, at 1, is not judged: its word counts whole.
    pub reliability: f64,
}

impl Model {
    /// Nothing said yet, under the two settings.
    pub fn empty(settings: Settings) -> Model {
        Model {
            evidence: Vec::new(),
            related: Vec::new(),
            base: Vec::new(),
            ties: Vec::new(),
            settings,
        }
    }
}

/// The relation a tie implies: its whole is all of the tie and its
/// statements. It is structure, believed at 1.
fn implied(t: &Tie) -> Statement<Related> {
    let about = whole(t);
    let mut pieces = vec![about.clone(), t.piece.clone()];
    pieces.extend(t.statements.iter().cloned());
    Statement {
        piece: Piece::named(&format!("all: {}", about.name)),
        content: Related {
            how: Relation::All,
            pieces,
        },
    }
}

/// The model with the relation of each tie written out.
// mitra: implements model::written
pub fn written(model: &Model) -> Model {
    let mut out = model.clone();
    for t in &model.ties {
        let relation = implied(t);
        if !out.related.iter().any(|said| said.piece == relation.piece) {
            out.related.push(relation);
        }
    }
    out
}

/// The parts of a whole: the pieces the relation names after the
/// first, and the completeness of each tie among them.
// mitra: implements model::parts
pub fn parts(model: &Model, r: &Related) -> Vec<Piece> {
    let mut found: Vec<Piece> = r.pieces.iter().skip(1).cloned().collect();
    for t in &model.ties {
        let implied = complete(t).piece;
        if r.pieces[1..].contains(&t.piece) && !found.contains(&implied) {
            found.push(implied);
        }
    }
    found
}

/// An opinion as the two parameters of a Beta: yes and no, each with
/// its part of the weight of the base rate.
fn beta(o: Opinion, w: f64) -> (f64, f64) {
    (o.r + o.a * w, o.s + (1.0 - o.a) * w)
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
pub fn all(x: Opinion, y: Opinion, w: f64) -> Opinion {
    let a = x.a * y.a;
    let (bx, by) = (beta(x, w), beta(y, w));
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
    n = f64::max(n, w * a / mean);
    n = f64::max(n, w * (1.0 - a) / (1.0 - mean));
    Opinion {
        r: f64::max(0.0, mean * n - a * w),
        s: f64::max(0.0, (1.0 - mean) * n - (1.0 - a) * w),
        a,
    }
}

/// What to expect of a piece with no evidence: its base rate. With
/// none written, it is the setting for its kind: the completeness of a
/// tie, a tie, the reliability of a voice, a piece of evidence, and a
/// statement for any other. The relation a tie implies is structure,
/// at 1.
// mitra: implements model::base
pub fn base(model: &Model, x: &Piece) -> f64 {
    if let Some((_, a)) = model.base.iter().find(|(p, _)| p == x) {
        return *a;
    }
    let set = &model.settings;
    if model.ties.iter().any(|t| &complete(t).piece == x) {
        return set.completeness;
    }
    if model.ties.iter().any(|t| &implied(t).piece == x) {
        return 1.0;
    }
    if model.ties.iter().any(|t| &t.piece == x) {
        return set.tie;
    }
    if model.evidence.iter().any(|e| &reliable(&e.by).piece == x) {
        return set.reliability;
    }
    match model.evidence.iter().any(|e| &e.piece == x) {
        true => set.evidence,
        false => set.statement,
    }
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

/// What is believed of `x`, from the whole model: the voices are
/// judged first, their word counts as far as it holds, and the rest is
/// read from that.
// mitra: implements model::opinion
pub fn opinion(model: &Model, x: &Piece) -> Opinion {
    let model = &written(model);
    let voices = judged(model);
    match voices.iter().find(|(piece, _)| piece == x) {
        None => read(&settled(model, &voices), x),
        Some((_, holds)) => {
            // What was said of it, and the count that brings it to
            // what it was judged at.
            let o = read(model, x);
            let (n, w) = (o.r + o.s, model.settings.weight);
            let yes = (holds * (n + w) - o.r - o.a * w) / (1.0 - holds);
            let no = (o.r + o.a * w) / holds - n - w;
            match (*holds >= projected(o, w), *holds < 1.0, *holds > 0.0) {
                (true, true, _) => Opinion { r: o.r + yes, ..o },
                (false, _, true) => Opinion { s: o.s + no, ..o },
                _ => o,
            }
        }
    }
}

/// What is believed of `x` in a model whose voices are not judged.
fn read(model: &Model, x: &Piece) -> Opinion {
    let mut walk = Walk::default();
    believed(model, x, &mut walk)
}

/// The voices that can be doubted: those whose reliability has a base
/// rate written, or set under 1. With each, the sources it rests on: the parts of its
/// reliability, when a relation says it is a whole.
fn doubted(model: &Model) -> Vec<(Voice, Piece, Vec<Piece>)> {
    let mut found: Vec<(Voice, Piece, Vec<Piece>)> = Vec::new();
    for e in &model.evidence {
        let piece = reliable(&e.by).piece;
        // Written, or set under 1: a voice sure to be reliable is not
        // judged.
        let known = model.base.iter().any(|(p, _)| p == &piece) || model.settings.reliability < 1.0;
        if !known || found.iter().any(|(voice, _, _)| voice == &e.by) {
            continue;
        }
        let mut sources = Vec::new();
        for said in &model.related {
            let r = &said.content;
            if r.how == Relation::All && r.pieces.first() == Some(&piece) {
                sources.extend(parts(model, r));
            }
        }
        found.push((e.by.clone(), piece, sources));
    }
    found
}

/// How far the word of each doubted voice holds, and how far each
/// source is believed, with the piece of each.
///
/// A voice is reliable or not: if not, its word is nothing. Each is
/// judged by every whole of the model. With `p` the chance that the
/// parts of a whole are all right before anything is seen of it,
///
/// ```text
/// L = p · (every failing observation is wrong) + (1 - p) · (every passing one is wrong)
/// ```
///
/// is read once with the word of the voice taken whole and once
/// without it; the ratio of the two, over all the wholes, moves the
/// odds of the voice. Voices on one source are weighed together. The
/// parts are then read again with the voices as judged, and so on
/// until nothing moves: expectation and maximisation, in turn.
// mitra: implements model::judged
pub fn judged(model: &Model) -> Vec<(Piece, f64)> {
    let model = &written(model);
    let voices = doubted(model);
    if voices.is_empty() {
        return Vec::new();
    }
    let w = model.settings.weight;
    let prior = |x: &Piece| projected(read(model, x), w);
    let own: Vec<f64> = voices.iter().map(|(_, piece, _)| prior(piece)).collect();
    let mut sources: Vec<(Piece, f64)> = Vec::new();
    for (_, _, on) in &voices {
        for source in on {
            if !sources.iter().any(|(piece, _)| piece == source) {
                sources.push((source.clone(), prior(source)));
            }
        }
    }
    let mut trusted_source: Vec<f64> = sources.iter().map(|(_, p)| *p).collect();
    let through = |k: usize, at: &[f64]| -> f64 {
        let on = &voices[k].2;
        let found = |s: &Piece| sources.iter().position(|(piece, _)| piece == s);
        on.iter().filter_map(found).map(|i| at[i]).product()
    };
    let mut holds: Vec<f64> = Vec::new();
    for (k, mine) in own.iter().enumerate() {
        holds.push(mine * through(k, &trusted_source));
    }
    // Each whole that was observed: the model before anything is seen
    // of it and before any no, and how likely its observations are
    // all wrong.
    let mut wholes = Vec::new();
    for said in &model.related {
        let whole = &said.content.pieces[0];
        let seen: Vec<&Evidence> = model
            .evidence
            .iter()
            .filter(|e| &e.about == whole)
            .collect();
        let is_a_voice = voices.iter().any(|(_, piece, _)| piece == whole);
        if seen.is_empty() || is_a_voice || said.content.how != Relation::All {
            continue;
        }
        let mut calm = model.clone();
        calm.evidence
            .retain(|e| &e.about != whole && e.value.s == 0.0);
        let (mut passes_wrong, mut failures_wrong) = (1.0, 1.0);
        for e in seen {
            let right = projected(read(&calm, &e.piece), w);
            match e.value.s > 0.0 {
                true => failures_wrong *= 1.0 - right,
                false => passes_wrong *= 1.0 - right,
            }
        }
        wholes.push((&said.content, calm, passes_wrong, failures_wrong));
    }
    for _ in 0..200 {
        let mut ratio = vec![1.0; voices.len()];
        for (k, said) in ratio.iter_mut().enumerate() {
            for (relation, calm, passes_wrong, failures_wrong) in &wholes {
                let likely = |mine: f64| {
                    let mut at = holds.clone();
                    at[k] = mine;
                    let world = heard_at(calm, &voices, &at);
                    let mut all_right = 1.0;
                    for part in parts(&world, relation) {
                        all_right *= projected(read(&world, &part), w);
                    }
                    all_right * failures_wrong + (1.0 - all_right) * passes_wrong
                };
                let (with, without) = (likely(1.0), likely(0.0));
                if with > 0.0 && without > 0.0 {
                    *said *= with / without;
                }
            }
        }
        // A voice with ratio L, reliable with chance a, is as likely
        // as a·L + 1 - a against 1.
        let weighed = |k: usize| own[k] * ratio[k] + 1.0 - own[k];
        for (i, (source, before)) in sources.iter().enumerate() {
            let mut for_it = *before;
            for (k, (_, _, on)) in voices.iter().enumerate() {
                if on.contains(source) {
                    for_it *= weighed(k);
                }
            }
            trusted_source[i] = for_it / (for_it + 1.0 - before);
        }
        let mut moved_by: f64 = 0.0;
        for k in 0..voices.len() {
            let now = own[k] * ratio[k] / weighed(k) * through(k, &trusted_source);
            moved_by = moved_by.max((now - holds[k]).abs());
            holds[k] = now;
        }
        if moved_by < 1e-9 {
            break;
        }
    }
    let mut out: Vec<(Piece, f64)> = Vec::new();
    for (k, (_, piece, _)) in voices.iter().enumerate() {
        out.push((piece.clone(), holds[k]));
    }
    for (i, (source, _)) in sources.iter().enumerate() {
        out.push((source.clone(), trusted_source[i]));
    }
    out
}

/// The model with the word of each voice counted as far as it holds.
fn heard_at(model: &Model, voices: &[(Voice, Piece, Vec<Piece>)], at: &[f64]) -> Model {
    let mut out = model.clone();
    out.evidence.clear();
    for e in &model.evidence {
        match voices.iter().position(|(voice, _, _)| voice == &e.by) {
            None => out.evidence.push(e.clone()),
            Some(k) if at[k] > 0.0 => out.evidence.push(Evidence {
                value: held(at[k], e.value, model.settings.weight),
                ..e.clone()
            }),
            Some(_) => {}
        }
    }
    out
}

/// The model once its voices are judged: their word counted as far as
/// it holds, and no voice left to doubt. What is believed of a piece
/// is read from it.
// mitra: implements model::settled
pub fn settled(model: &Model, judged: &[(Piece, f64)]) -> Model {
    let model = &written(model);
    let voices = doubted(model);
    let at: Vec<f64> = voices
        .iter()
        .map(|(_, piece, _)| judged.iter().find(|(p, _)| p == piece).map_or(1.0, |j| j.1))
        .collect();
    let mut out = heard_at(model, &voices, &at);
    out.base
        .retain(|(piece, _)| !voices.iter().any(|(_, p, _)| p == piece));
    out.settings.reliability = 1.0;
    out
}

/// `x` is believed enough for the policy.
// mitra: implements model::trusted
pub fn trusted(model: &Model, x: &Piece, policy: f64) -> bool {
    reaches(opinion(model, x), policy, model.settings.weight)
}

/// What `x` stands on, taken together, as far as the relations that
/// lead there are believed.
fn believed(model: &Model, x: &Piece, walk: &mut Walk) -> Opinion {
    let (on, trust) = support(model, x, walk);
    moved(
        nothing(trust),
        together(model, &on, walk),
        model.settings.weight,
    )
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
                let trust = projected(direct(model, &said.piece, walk), model.settings.weight);
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
            moved(
                believed(model, &e.piece, walk),
                e.value,
                model.settings.weight,
            ),
            e.by.clone(),
        ));
    }
    for (r, trust) in related(model, Relation::All, walk) {
        if r.pieces.len() < 2 || !parts(model, r).contains(x) {
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

/// The suspects of a failure: the parts of the whole, and the parts of
/// each part that is itself a whole, down to what stands on nothing;
/// the evidence that says no; and each relation on the way that can be
/// doubted. They cannot all be right. A relation believed at 1 is
/// structure, and not a suspect. A piece reached by two paths is one
/// suspect.
fn suspects(
    model: &Model,
    relation: &Statement<Related>,
    e: &Evidence,
    walk: &mut Walk,
) -> Vec<Piece> {
    let mut group: Vec<Piece> = Vec::new();
    let mut doubted: Vec<Piece> = Vec::new();
    let mut seen = vec![relation.content.pieces[0].clone()];
    let mut open = vec![relation];
    while let Some(said) = open.pop() {
        if projected(prior(model, &said.piece, walk), model.settings.weight) < 1.0 {
            doubted.push(said.piece.clone());
        }
        for part in parts(model, &said.content) {
            if seen.contains(&part) {
                continue;
            }
            seen.push(part.clone());
            let inner = model.related.iter().find(|r| {
                r.content.how == Relation::All && r.content.pieces.first() == Some(&part)
            });
            match inner {
                Some(r) if r.content.pieces.len() > 1 => open.push(r),
                _ => group.push(part),
            }
        }
    }
    group.push(e.piece.clone());
    group.extend(doubted);
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
    let group = suspects(model, relation, e, walk);
    let Some(at) = group.iter().position(|suspect| suspect == x) else {
        return 0.0;
    };
    let mut read = Vec::new();
    for suspect in &group {
        read.push(beta(prior(model, suspect, walk), model.settings.weight));
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
        for part in &parts(model, r) {
            let (pieces, through) = support(model, part, walk);
            on.0.extend(pieces);
            on.1 *= through;
        }
        let new = moved(
            nothing(on.1),
            together(model, &on.0, walk),
            model.settings.weight,
        );
        let old = moved(
            nothing(best.1),
            together(model, &best.0, walk),
            model.settings.weight,
        );
        if uncertainty(new, model.settings.weight) <= uncertainty(old, model.settings.weight) {
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
            Some(o) => all(o, own, model.settings.weight),
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
    let model = &settled(model, &judged(model));
    let mut out = Vec::new();
    let mut walk = Walk::default();
    for (relation, e) in failures(model, &mut walk) {
        // Evidence that was not believed before it failed accuses nothing.
        if projected(prior(model, &e.piece, &mut walk), model.settings.weight) > 0.0 {
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
        if projected(read(model, &said.piece), model.settings.weight) > 0.0
            && r.pieces.iter().all(|p| read(model, p).r > 0.0)
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
        let said = |o: &&Evidence| moved(read(model, &o.piece), o.value, model.settings.weight);
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
    let p = |x: &Piece| projected(read(model, x), model.settings.weight);
    pieces.sort_by(|x, y| p(x).total_cmp(&p(y)));
    pieces
}
