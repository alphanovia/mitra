//! The crossing (`model/claim.mitra`): evaluate, take, judge.
//!
//! A claim is a piece about a record or a function, said by an
//! expression. Evaluating it on a measurement gives a truth, which is
//! a value. A voice takes that truth as evidence, and from there the
//! calculus judges. Nothing else crosses: a value never depends on a
//! piece, and an opinion never reads a value.

use crate::atom::Measurement;
use crate::bounds::{lifted, point, surely, Bounds, Span};
use crate::model::{
    erased, leaf, within, Form, Keyed, Location, Model, Piece, SomePiece, Taking, Tie, Voice,
};
use crate::standard::{truth, Exact};
use crate::value::{check, eval, Applies, Expression, Value};

/// What a claim is about and how it is said.
///
/// - `of`: the record or the function it is about.
/// - `domain`: for each input it names, the bounds that input lies
///   in. It is what stands before the claim, as a guard.
/// - `expression`: gives a truth. The claim says it is true at every
///   input in the domain.
// mitra: implements claims::Claim
#[derive(Clone, Debug, PartialEq)]
pub struct Claim {
    pub of: String,
    pub domain: Vec<(String, Value<Bounds>)>,
    pub expression: Expression<Exact>,
}

/// What a piece of evidence is about: a measurement, and who made it.
// mitra: implements claims::Evidence
#[derive(Clone, Debug, PartialEq)]
pub struct Evidence<Q> {
    pub measurement: Measurement<Q>,
    pub by: Voice,
}

/// The step from a claim to one piece of evidence of it.
// mitra: implements claims::Instance
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Instance;

impl Keyed for Claim {
    fn keyword(&self) -> String {
        "claim".to_string()
    }
}
impl<Q> Keyed for Evidence<Q> {
    fn keyword(&self) -> String {
        "evidence".to_string()
    }
}
impl Keyed for Instance {
    fn keyword(&self) -> String {
        "instance".to_string()
    }
}

/// The expression gives a truth.
// mitra: implements claims::reads
pub fn reads(c: &Piece<Claim>) -> bool {
    check(&c.about.expression) == Some(truth())
}

/// The value lies in the bounds. None where it has no bounds.
fn lies(v: &Value<Exact>, bounds: &Value<Bounds>) -> Option<bool> {
    let at = point(v)?;
    Some(match (&at.quantity.within, &bounds.quantity.within) {
        (Span::Numbers { low: x, .. }, Span::Numbers { low, high }) => {
            let under = |a: &(i64, i64), b: &(i64, i64)| {
                a.0 as i128 * b.1 as i128 <= b.0 as i128 * a.1 as i128
            };
            at.of == bounds.of && under(low, x) && under(x, high)
        }
        (Span::Truths { yes, no }, Span::Truths { yes: y, no: n }) => (!yes || *y) && (!no || *n),
        _ => false,
    })
}

/// The measurement is of what the claim is about, and in its domain:
/// each input the domain names was measured, and lies in its bounds.
// mitra: implements claims::guarded
pub fn guarded(c: &Piece<Claim>, m: &Measurement<Exact>) -> bool {
    m.function == c.about.of
        && c.about.domain.iter().all(|(name, bounds)| {
            let given = m.values.iter().find(|(input, _)| input == name);
            given.and_then(|(_, v)| lies(v, bounds)) == Some(true)
        })
}

/// What the claim's expression gives on a measurement at a point:
/// true, false, or nothing. Nothing where the measurement is of
/// something else or outside the domain: the run is outside the
/// contract. Nothing where the expression has no value.
// mitra: implements claims::evaluate
pub fn evaluate(
    c: &Piece<Claim>,
    m: &Measurement<Exact>,
    with: &impl Applies<Exact>,
) -> Option<bool> {
    if !guarded(c, m) {
        return None;
    }
    let seen = eval(&c.about.expression, &m.values, with)?;
    match (seen.of == truth(), seen.quantity) {
        (true, Exact::Truth(value)) => Some(value),
        _ => None,
    }
}

/// What the claim's expression gives over these ranges: true where it
/// is true everywhere in them, false where it is false everywhere,
/// nothing where it may be either or the reading is unknown.
// mitra: implements claims::evaluate_over
pub fn evaluate_over(
    c: &Piece<Claim>,
    ranges: &[(String, Value<Bounds>)],
    with: &impl Applies<Bounds>,
) -> Option<bool> {
    surely(&eval(&lifted(&c.about.expression)?, ranges, with)?)
}

/// The piece of evidence for a claim on a measurement: a leaf named
/// after the claim and what was measured, so the same claim on the
/// same measurement is one piece.
// mitra: implements claims::evidence_of
pub fn evidence_of<Q: std::fmt::Debug + Clone>(
    c: &Piece<Claim>,
    m: &Measurement<Q>,
    by: &Voice,
) -> Piece<Evidence<Q>> {
    let each = m
        .values
        .iter()
        .map(|(name, v)| format!("{name} = {:?}", v.quantity));
    let at = each.collect::<Vec<_>>().join(", ");
    let about = Evidence {
        measurement: m.clone(),
        by: by.clone(),
    };
    leaf(&format!("{} @ {at}", c.name), about)
}

/// The step from a claim to one piece of evidence of it: the claim
/// true, the evidence is true; the evidence false, the claim is
/// false. With a tie, the measurement was made at the tie's location,
/// and the tie stands before the evidence with the claim: evidence
/// that is false there says the claim and the tie cannot both be
/// true. The step is a deduction: the model gets it with a bias of 1.
// mitra: implements claims::instantiation
pub fn instantiation(c: &Piece<Claim>, e: &SomePiece, at: Option<&Piece<Tie>>) -> Piece<Instance> {
    let mut pre = vec![erased(c)];
    pre.extend(at.map(erased));
    Piece {
        name: format!("instance: {}", e.name),
        pre,
        about: Instance,
        post: vec![e.clone()],
    }
}

/// The crossing, whole: the claim is evaluated on the measurement,
/// and the voice takes the evidence as what that gave, `count` times
/// over. The model gets the evidence, what evaluating gave, the
/// taking, and the step from the claim to the evidence.
///
/// Nothing where the claim has no value there. Nothing where the tie
/// is of something else than the claim, or the measurement was not
/// made at the tie's location.
// mitra: implements claims::observe
pub fn observe(
    model: &Model,
    c: &Piece<Claim>,
    m: &Measurement<Exact>,
    with: &impl Applies<Exact>,
    at: Option<&Piece<Tie>>,
    count: f64,
    by: &Voice,
) -> Option<Model> {
    let seen = evaluate(c, m, with)?;
    let e = erased(&evidence_of(c, m, by));
    crossed(
        model,
        c,
        e,
        seen,
        (&m.at, at),
        (count, by, Form::Measurement),
    )
}

/// The same crossing over a set: one piece of evidence for every
/// point in the ranges of the measurement. It is an evaluation and
/// not a run.
// mitra: implements claims::observe_over
pub fn observe_over(
    model: &Model,
    c: &Piece<Claim>,
    m: &Measurement<Bounds>,
    with: &impl Applies<Bounds>,
    at: Option<&Piece<Tie>>,
    count: f64,
    by: &Voice,
) -> Option<Model> {
    if m.function != c.about.of {
        return None;
    }
    let seen = evaluate_over(c, &m.values, with)?;
    let e = erased(&evidence_of(c, m, by));
    crossed(
        model,
        c,
        e,
        seen,
        (&m.at, at),
        (count, by, Form::Evaluation),
    )
}

/// A proof and not a test: the claim is evaluated over its whole
/// domain, and the voice takes the claim itself as what that gave.
/// Nothing where the reading is not sure everywhere in the domain.
// mitra: implements claims::prove
pub fn prove(
    model: &Model,
    c: &Piece<Claim>,
    with: &impl Applies<Bounds>,
    count: f64,
    by: &Voice,
) -> Option<Model> {
    let seen = evaluate_over(c, &c.about.domain, with)?;
    let mut out = model.clone();
    out.evaluated.push((erased(c), seen));
    out.taken.push(Taking {
        piece: erased(c),
        value: seen,
        count,
        by: by.clone(),
        form: Form::Evaluation,
    });
    Some(out)
}

fn crossed(
    model: &Model,
    c: &Piece<Claim>,
    e: SomePiece,
    seen: bool,
    (place, at): (&Location, Option<&Piece<Tie>>),
    (count, by, form): (f64, &Voice, Form),
) -> Option<Model> {
    let mut out = model.clone();
    if let Some(tie) = at {
        if tie.about.of != c.about.of || !within(place, &tie.about.location) {
            return None;
        }
    }
    let step = erased(&instantiation(c, &e, at));
    if !out.pieces.iter().any(|old| old.name == step.name) {
        out.bias.push((step.clone(), 1.0));
        out.pieces.push(step);
    }
    out.evaluated.push((e.clone(), seen));
    out.taken.push(Taking {
        piece: e,
        value: seen,
        count,
        by: by.clone(),
        form,
    });
    Some(out)
}
