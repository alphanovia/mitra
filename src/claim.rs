//! The seam (`model/claim.mitra`): here the content of a statement is
//! filled in. Computing a claim at an input makes evidence about that
//! instance of it.
//!
//! The only way across: a value never depends on a piece, and an
//! opinion is only moved by evidence.

use crate::bounds::{lifted, surely, Bounds};
use crate::model::{no, yes, Evidence, Model, Piece, Related, Relation, Statement, Tie, Voice};
use crate::standard::{truth, Exact};
use crate::value::{check, eval, Applies, Expression, Value};

/// What a claim says: an expression that gives a truth. A claim is a
/// statement with this content.
// mitra: implements claims::Claim
#[derive(Clone, Debug, PartialEq)]
pub struct Claim {
    pub expression: Expression<Exact>,
}

/// The expression gives a truth.
// mitra: implements claims::reads
pub fn reads(s: &Statement<Claim>) -> bool {
    check(&s.content.expression) == Some(truth())
}

/// The claim at these inputs: "at these inputs, what the claim says is
/// so". A piece derived from the claim, named after it and the inputs.
/// The inputs are exact values, or the ranges they lie in.
// mitra: implements claims::instance
pub fn instance<Q: std::fmt::Debug>(s: &Statement<Claim>, inputs: &[(String, Value<Q>)]) -> Piece {
    let each = inputs
        .iter()
        .map(|(name, v)| format!("{name} = {:?}", v.quantity));
    Piece {
        name: format!("{} @ {}", s.piece.name, each.collect::<Vec<_>>().join(", ")),
    }
}

/// What computing the claim at these inputs says of that instance:
/// yes where it is true, no where it is false, nothing where it has no
/// value. The evidence is named after the instance and the voice, so
/// the same observation twice is one piece.
// mitra: implements claims::observe
pub fn observe(
    s: &Statement<Claim>,
    inputs: &[(String, Value<Exact>)],
    by: &Voice,
    with: &impl Applies<Exact>,
) -> Option<Evidence> {
    let seen = eval(&s.content.expression, inputs, with)?;
    let value = match (seen.of == truth(), seen.quantity) {
        (true, Exact::Truth(true)) => yes(),
        (true, Exact::Truth(false)) => no(),
        _ => return None,
    };
    let about = instance(s, inputs);
    Some(Evidence {
        piece: Piece {
            name: format!("{} by {}", about.name, by.name),
        },
        about,
        value,
        by: by.clone(),
    })
}

/// What reading the claim by bounds says of its instance over these
/// ranges: `count` yes where it is true everywhere in them, `count` no
/// where it is false everywhere, and nothing where it may be either,
/// may have no value, or the reading is unknown. `count` is how many
/// observations a whole range is worth: it is given, and nothing here
/// holds a value for it.
// mitra: implements claims::bounded
pub fn bounded(
    s: &Statement<Claim>,
    ranges: &[(String, Value<Bounds>)],
    count: f64,
    by: &Voice,
    with: &impl Applies<Bounds>,
) -> Option<Evidence> {
    let seen = eval(&lifted(&s.content.expression)?, ranges, with)?;
    let value = match surely(&seen)? {
        true => crate::model::Opinion { r: count, ..yes() },
        false => crate::model::Opinion { s: count, ..no() },
    };
    let about = instance(s, ranges);
    Some(Evidence {
        piece: Piece {
            name: format!("{} by {}", about.name, by.name),
        },
        about,
        value,
        by: by.clone(),
    })
}

/// The model with an observation of a claim in it. `e` is evidence
/// about an instance of `s`, from `observe` or `bounded`.
///
/// - With a tie that lists the claim, the evidence is about the tie's
///   whole at that instance: the tie, the instance and the tie's
///   completeness, all together. So a no is shared between them.
///   Nothing for a tie that does not list the claim.
/// - With no tie, the evidence is about the instance alone.
/// - With `uniform`, the claim is all of its observed instances and
///   that piece. Without it nothing leads from an instance to the
///   claim.
// mitra: implements claims::recorded
pub fn recorded(
    model: &Model,
    s: &Statement<Claim>,
    e: Evidence,
    at: Option<&Tie>,
    uniform: Option<&Piece>,
) -> Option<Model> {
    let mut out = model.clone();
    let structure = |out: &mut Model, whole: &Piece, parts: Vec<Piece>| {
        let name = Piece::named(&format!("all: {}", whole.name));
        let mut pieces = vec![whole.clone()];
        pieces.extend(parts);
        out.related.push(Statement {
            piece: name.clone(),
            content: Related {
                how: Relation::All,
                pieces,
            },
        });
        out.base.push((name, 1.0));
    };
    let the_instance = e.about.clone();
    if let Some(hypothesis) = uniform {
        let name = format!("all: {}", s.piece.name);
        match out.related.iter_mut().find(|said| said.piece.name == name) {
            Some(said) if said.content.pieces.contains(&the_instance) => {}
            Some(said) => {
                let last = said.content.pieces.len() - 1;
                said.content.pieces.insert(last, the_instance.clone());
            }
            None => structure(
                &mut out,
                &s.piece,
                vec![the_instance.clone(), hypothesis.clone()],
            ),
        }
    }
    match at {
        None => out.evidence.push(e),
        Some(t) => {
            if !t.statements.contains(&s.piece) {
                return None;
            }
            if !out.ties.contains(t) {
                out.ties.push(t.clone());
            }
            let whole = Piece::named(&format!("{} at {}", the_instance.name, t.piece.name));
            if !out
                .related
                .iter()
                .any(|said| said.content.pieces[0] == whole)
            {
                structure(&mut out, &whole, vec![t.piece.clone(), the_instance]);
            }
            out.evidence.push(Evidence { about: whole, ..e });
        }
    }
    Some(out)
}
