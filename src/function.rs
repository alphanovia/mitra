//! Functions (`model/function.mitra`): an operation the system's code
//! implements, the tie to that code, and the statements about it.
//!
//! A function is a group, not a new kind of piece. A helper is only a
//! definition: it joins the operations that can be applied, and
//! nothing is believed of it.

use crate::claim::{observe, Claim};
use crate::model::{
    whole, yes, Evidence, Model, Piece, Place, Related, Relation, Statement, Tie, Voice,
};
use crate::standard::{equal, Exact, Table};
use crate::value::{eval, Applies, Expression, Operation, Value};

/// An operation the code implements: its name and signature, the
/// names of its inputs, and what the model says of it.
///
/// - `body`: what it should give, as an expression of its inputs: the
///   spec. It implies the statement `f::correct`.
/// - `uniform`: the model takes one input as standing for all: the
///   statement `f::uniform`, asked for and not implied.
/// - `claims`: other statements about it, named `f::…`, that speak of
///   its inputs and of nothing else.
/// - `at`: where its code is. With none it is untied: owed work.
// mitra: implements functions::Function
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub operation: Operation,
    pub inputs: Vec<String>,
    pub body: Option<Expression<Exact>>,
    pub uniform: bool,
    pub claims: Vec<Statement<Claim>>,
    pub at: Option<Place>,
}

/// A definition: an operation with a body, and nothing else. It has no
/// tie and no statement.
// mitra: implements functions::Helper
#[derive(Clone, Debug, PartialEq)]
pub struct Helper {
    pub operation: Operation,
    pub inputs: Vec<String>,
    pub body: Expression<Exact>,
}

/// One run of the code: what an operation gave on these values.
// mitra: implements functions::Run
#[derive(Clone, Debug, PartialEq)]
pub struct Run {
    pub operation: Operation,
    pub args: Vec<Value<Exact>>,
    pub result: Value<Exact>,
}

fn named(f: &Function, part: &str) -> Piece {
    Piece::named(&format!("{}::{part}", f.operation.name))
}

/// The function applied to its inputs.
// mitra: implements functions::called
pub fn called(f: &Function) -> Expression<Exact> {
    let inputs = f.inputs.iter().zip(&f.operation.takes);
    let args = inputs.map(|(name, of)| Expression::Input(name.clone(), of.clone()));
    Expression::Apply(f.operation.clone(), args.collect())
}

/// `f::correct`: on every input, the code gives what the body gives.
/// None for a function with no body.
// mitra: implements functions::correctness
pub fn correctness(f: &Function) -> Option<Statement<Claim>> {
    let same = equal(&f.operation.gives);
    Some(Statement {
        piece: named(f, "correct"),
        content: Claim {
            expression: Expression::Apply(same, vec![called(f), f.body.clone()?]),
        },
    })
}

/// `f::seen`: where the code was run, it gave what the model says.
/// This is what a passing test says yes to.
// mitra: implements functions::seen
pub fn seen(f: &Function) -> Piece {
    named(f, "seen")
}
/// `f::deterministic`: the same inputs always give the same output.
// mitra: implements functions::determinism
pub fn determinism(f: &Function) -> Piece {
    named(f, "deterministic")
}
/// `f::uniform`: the code is right on every input if it is right on
/// one. None when the model does not ask for it.
// mitra: implements functions::uniformity
pub fn uniformity(f: &Function) -> Option<Piece> {
    f.uniform.then(|| named(f, "uniform"))
}

/// A statement is one of this function's: named `f::…`, and every
/// input it reads is an input of the function, of the same type.
// mitra: implements functions::in_scope
pub fn in_scope(f: &Function, s: &Statement<Claim>) -> bool {
    fn signature_only(e: &Expression<Exact>, f: &Function) -> bool {
        match e {
            Expression::Given(_) => true,
            Expression::Input(name, of) => {
                let mine = f.inputs.iter().zip(&f.operation.takes);
                mine.into_iter().any(|(n, t)| n == name && t == of)
            }
            Expression::Apply(_, args) => args.iter().all(|arg| signature_only(arg, f)),
        }
    }
    let prefix = format!("{}::", f.operation.name);
    s.piece.name.starts_with(&prefix) && signature_only(&s.content.expression, f)
}

/// The tie of a function to its code: it lists `f::seen`,
/// `f::deterministic` and the function's other statements. None for a
/// function with no place.
// mitra: implements functions::tie
pub fn tie(f: &Function) -> Option<Tie> {
    let mut statements = vec![seen(f), determinism(f)];
    statements.extend(f.claims.iter().map(|s| s.piece.clone()));
    Some(Tie {
        piece: named(f, "tie"),
        statements,
        place: f.at.clone()?,
    })
}

/// The functions with no tie: what the model says of them cannot be
/// measured on the code yet.
// mitra: implements functions::untied
pub fn untied(functions: &[Function]) -> Vec<String> {
    let none = functions.iter().filter(|f| f.at.is_none());
    none.map(|f| f.operation.name.clone()).collect()
}

/// What a passing test of the function says yes to: the whole of its
/// tie, or `f::seen` alone when it has no tie.
// mitra: implements functions::ground
pub fn ground(f: &Function) -> Piece {
    match tie(f) {
        Some(t) => whole(&t),
        None => seen(f),
    }
}

/// The model with the function in it: its tie, and when it has a body
/// and asks for `uniform`, the relation that `f::correct` is all of
/// what was seen and `f::uniform`. Without `uniform`, nothing leads
/// from what was seen to `f::correct`.
// mitra: implements functions::placed
pub fn placed(model: &Model, f: &Function) -> Model {
    let mut out = model.clone();
    if let Some(t) = tie(f) {
        if !out.ties.contains(&t) {
            out.ties.push(t);
        }
    }
    if let (Some(general), Some(hypothesis)) = (correctness(f), uniformity(f)) {
        let relation = Piece::named(&format!("all: {}", general.piece.name));
        if !out.related.iter().any(|said| said.piece == relation) {
            out.related.push(Statement {
                piece: relation.clone(),
                content: Related {
                    how: Relation::All,
                    pieces: vec![general.piece, ground(f), hypothesis],
                },
            });
            out.base.push((relation, 1.0));
        }
    }
    out
}

/// What applies operations once helpers are defined and the code was
/// run: an operation that was run gives what the run gave, a helper
/// gives what its body gives, and the table applies the rest.
// mitra: implements functions::Known
#[derive(Clone, Debug)]
pub struct Known<'a> {
    pub table: &'a Table,
    pub helpers: &'a [Helper],
    pub runs: &'a [Run],
}

impl Applies<Exact> for Known<'_> {
    // mitra: implements functions::Known::apply
    fn apply(&self, op: &Operation, args: &[Option<Value<Exact>>]) -> Option<Exact> {
        let all: Option<Vec<Value<Exact>>> = args.iter().cloned().collect();
        if let Some(values) = &all {
            let same = |run: &&Run| &run.operation == op && &run.args == values;
            if let Some(run) = self.runs.iter().find(same) {
                return Some(run.result.quantity.clone());
            }
            if let Some(helper) = self.helpers.iter().find(|h| &h.operation == op) {
                let names = helper.inputs.iter().cloned();
                let inputs: Vec<(String, Value<Exact>)> =
                    names.zip(values.iter().cloned()).collect();
                return Some(eval(&helper.body, &inputs, self)?.quantity);
            }
        }
        self.table.apply(op, args)
    }
}

/// What one run of the code says, read against the body. Where the
/// run gave what the body gives, it is a yes about what was seen: the
/// whole of the function's tie. Where it did not, it is a no about
/// `f::correct` itself: one input where the code is wrong refutes it
/// whatever else is assumed. Nothing for a function with no body, a
/// run of another operation, or a body with no value there.
// mitra: implements functions::tested
pub fn tested(f: &Function, run: &Run, by: &Voice, with: &Known) -> Option<Evidence> {
    if run.operation != f.operation || run.args.len() != f.inputs.len() {
        return None;
    }
    let inputs: Vec<(String, Value<Exact>)> = f
        .inputs
        .iter()
        .cloned()
        .zip(run.args.iter().cloned())
        .collect();
    let runs = [run.clone()];
    let known = Known {
        table: with.table,
        helpers: with.helpers,
        runs: &runs,
    };
    let e = observe(&correctness(f)?, &inputs, by, &known)?;
    Some(match e.value == yes() {
        true => Evidence {
            about: ground(f),
            ..e
        },
        false => e,
    })
}
