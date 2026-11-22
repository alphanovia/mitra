//! Tactics: functions that write proofs (`docs/core2.md`). A tactic is
//! trusted with nothing: what it writes is a proof like any other, and
//! judgement checks it, as Lean's kernel checks what its tactics write.
//! A checked proof is a taking by the checker, whatever wrote it.

use crate::expression::{Arg, Expression};
use crate::judgement::{Op, Prop};
use crate::simulation::Value;

/// A proof of `q`, an all or an any over a set of whole numbers, by its
/// cases: for each point of the set, in the order `Set::points` lists
/// them, `case(goal, point)` writes a proof of `goal`, the body of `q`
/// at that point. `None` when `q` is neither, or its set is not of
/// whole numbers. There are as many cases as points: a large set is
/// taken or refuted instead.
pub fn cases(
    q: &Prop,
    mut case: impl FnMut(&Prop, &[Value]) -> Expression<Op>,
) -> Option<Expression<Op>> {
    let (Prop::All { over, .. } | Prop::Any { over, .. }) = q else {
        return None;
    };
    let mut args = Vec::new();
    for point in over.points()? {
        let goal = q.at(&point)?;
        let body = case(&goal, &point);
        args.push(Arg {
            binds: vec![],
            body,
        });
    }
    Some(Expression::Apply(Op::Cases { q: q.clone() }, args))
}
