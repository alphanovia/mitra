//! The crossing (`model/claim.mitra`): evaluate, take, judge.
//!
//! A claim is a piece whose content is an expression. Evaluating it
//! at inputs gives a truth, which is a value. A voice takes that
//! truth, and from there the calculus judges. Nothing else crosses: a
//! value never depends on a piece, and an opinion never reads a value.

use crate::bounds::{lifted, surely, Bounds};
use crate::model::{leaf, Inference, Model, Piece, Taking, Voice};
use crate::standard::{truth, Exact};
use crate::value::{check, eval, Applies, Expression, Value};

/// A piece and the expression that says it. The expression gives a
/// truth, and the claim says it is true at every input.
// mitra: implements claims::Claim
#[derive(Clone, Debug, PartialEq)]
pub struct Claim {
    pub piece: Piece,
    pub expression: Expression<Exact>,
}

/// The expression gives a truth.
// mitra: implements claims::reads
pub fn reads(c: &Claim) -> bool {
    check(&c.expression) == Some(truth())
}

/// What the claim's expression gives at these inputs: true, false, or
/// nothing where it has no value.
// mitra: implements claims::evaluate
pub fn evaluate(
    c: &Claim,
    inputs: &[(String, Value<Exact>)],
    with: &impl Applies<Exact>,
) -> Option<bool> {
    let seen = eval(&c.expression, inputs, with)?;
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
    c: &Claim,
    ranges: &[(String, Value<Bounds>)],
    with: &impl Applies<Bounds>,
) -> Option<bool> {
    surely(&eval(&lifted(&c.expression)?, ranges, with)?)
}

/// The claim at these inputs: a piece of its own, named after the
/// claim and the inputs. The inputs are exact values, or the ranges
/// they lie in.
// mitra: implements claims::instance
pub fn instance<Q: std::fmt::Debug>(c: &Claim, inputs: &[(String, Value<Q>)]) -> Piece {
    let each = inputs
        .iter()
        .map(|(name, v)| format!("{name} = {:?}", v.quantity));
    let at = each.collect::<Vec<_>>().join(", ");
    leaf(&format!("{} @ {at}", c.piece.name), &c.piece.subject)
}

/// The step from a claim to one instance of it: the claim true, the
/// instance is true; the instance false, the claim is false. It holds
/// by what it is, so it is given to the model and never judged.
// mitra: implements claims::instantiation
pub fn instantiation(c: &Claim, here: &Piece) -> Piece {
    Piece {
        name: format!("instance: {}", here.name),
        pre: vec![c.piece.clone()],
        subject: c.piece.subject.clone(),
        post: vec![here.clone()],
        by: Inference::Deduction,
    }
}

/// The crossing, whole: the claim is evaluated at these inputs, and
/// the voice takes its instance as what that gave, `count` times
/// over. Nothing where the claim has no value there. The model gets
/// the instance, what evaluating gave, the taking, and the step from
/// the claim to the instance.
// mitra: implements claims::observe
pub fn observe(
    model: &Model,
    c: &Claim,
    inputs: &[(String, Value<Exact>)],
    with: &impl Applies<Exact>,
    count: f64,
    by: &Voice,
) -> Option<Model> {
    let seen = evaluate(c, inputs, with)?;
    Some(crossed(model, c, instance(c, inputs), seen, count, by))
}

/// The same crossing over ranges: one instance for every point in
/// them.
// mitra: implements claims::observe_over
pub fn observe_over(
    model: &Model,
    c: &Claim,
    ranges: &[(String, Value<Bounds>)],
    with: &impl Applies<Bounds>,
    count: f64,
    by: &Voice,
) -> Option<Model> {
    let seen = evaluate_over(c, ranges, with)?;
    Some(crossed(model, c, instance(c, ranges), seen, count, by))
}

fn crossed(model: &Model, c: &Claim, here: Piece, seen: bool, count: f64, by: &Voice) -> Model {
    let mut out = model.clone();
    let step = instantiation(c, &here);
    if !out.given.iter().any(|old| old.name == step.name) {
        out.given.push(step);
    }
    out.evaluated.push((here.clone(), seen));
    out.taken.push(Taking {
        piece: here,
        value: seen,
        count,
        by: by.clone(),
    });
    out
}
