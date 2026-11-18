//! From evidence to verdicts. Every piece of evidence counts, whatever its
//! bias: the bias only ranks what is most plausibly wrong when the model
//! and the evidence can't all be true.

use crate::ast::*;
use crate::set::Value;
use crate::tuples::Tuples;
use crate::{Error, Input};
use std::cell::Cell;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Logic {
    /// Evidence covers every input, directly or through claims that hold.
    Holds,
    /// The same, if the claims in `Verdict::assuming` hold: they are
    /// unverified.
    Assuming,
    /// Some inputs are missed.
    Unverified,
    /// Evidence refutes the claim.
    Fails,
}

impl Logic {
    pub fn as_str(self) -> &'static str {
        match self {
            Logic::Holds => "holds",
            Logic::Assuming => "holds assuming",
            Logic::Unverified => "unverified",
            Logic::Fails => "fails",
        }
    }
}

/// The status of the implication behind a `from`: premises ⇒ claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Implication {
    /// Substituting the premises into the claim gives its target on every
    /// input: the implication holds by computation.
    Checked,
    /// Mitra could not compute it (no shape, an infinite domain, or a
    /// `correct` claim). It is taken as stated by the author.
    Unchecked,
    /// Substituting the premises gives a different result on some input: the
    /// premises imply something else, so the derivation is not used.
    Refuted,
}

impl Implication {
    pub fn as_str(self) -> &'static str {
        match self {
            Implication::Checked => "checked",
            Implication::Unchecked => "unchecked",
            Implication::Refuted => "refuted",
        }
    }
}

/// The verdict on one claim.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub claim: String,
    pub logic: Logic,
    /// With `Logic::Assuming`: the unverified claims the verdict rests on.
    pub assuming: Vec<String>,
    /// The claim's inputs that evidence covers directly.
    pub tested: Tuples,
    /// The inputs that follow from the premises of `by derive`.
    pub derived: Tuples,
    /// With `by derive`: the implication is not refuted and no premise
    /// fails.
    pub follows: bool,
    /// What a `uniform` claim on the subject adds, if there is one.
    pub assumed: Option<Assumed>,
    /// Why no test stands for other inputs, when the claim calls subjects
    /// and a `uniform` claim can't be used.
    pub unsorted: Option<String>,
    /// The inputs nothing covers.
    pub missed: Tuples,
    /// The claim's inputs where it is known to fail and not contradicted.
    pub refuted: Tuples,
    /// The inputs the claim is about.
    pub domain: Tuples,
    /// For a claim with `from`.
    pub implication: Option<Implication>,
}

/// What a `uniform` claim adds to another claim on the same subject: an
/// input that passed stands for its whole class.
#[derive(Debug, Clone, PartialEq)]
pub struct Assumed {
    /// The `uniform` claim, and its spec, whose values are the classes.
    /// Without a spec there is one class, of every input.
    pub by: String,
    pub spec: Option<String>,
    /// For a claim that calls subjects: the calls its inputs are sorted
    /// by, one class each, and the other `uniform` claims relied on.
    pub calls: Vec<String>,
    pub with: Vec<String>,
    /// Those of the `uniform` claims that no evidence vouches for.
    pub unvouched: Vec<String>,
    /// Evidence covers the `uniform` claim: then a failure stands for
    /// its class too.
    pub vouched: bool,
    /// The `uniform` claim does not fail: then `set` counts, under the
    /// assumption that it holds when it is not vouched for.
    pub usable: bool,
    /// A failure over a set too large to sort into classes: no class is
    /// relied on.
    pub blind: bool,
    /// The classes left out because one of their inputs has both a pass
    /// and a failure.
    pub tainted: Vec<Value>,
    /// The classes with an input that passed.
    pub classes: Vec<Value>,
    /// The classes the claim needs that have none.
    pub untested: Vec<Value>,
    /// The classes with an input that failed.
    pub failing: Vec<Value>,
    /// The inputs of the claim's domain that the classes stand for, when
    /// Mitra can write them as a set: all of them on a finite domain; on an
    /// infinite one, the rest of the domain once no class is untested.
    pub set: Tuples,
}

/// Knowledge that can't all be true: a `uniform` claim, and a pass and a
/// failure in one of its classes.
#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    pub claim: String,
    pub spec: Option<String>,
    pub class: Value,
    /// For a claim that calls subjects: that claim, the calls its inputs
    /// are sorted by, and the other `uniform` claims relied on.
    pub on: Option<String>,
    pub calls: Vec<String>,
    pub with: Vec<String>,
    /// The input, its evidence line and the evidence as text.
    pub pass: (Vec<Value>, usize, String),
    pub fail: (Vec<Value>, usize, String),
    /// Every passing and every failing input of the class, with its line.
    pub passes: Vec<(Vec<Value>, usize)>,
    pub fails: Vec<(Vec<Value>, usize)>,
}

/// A subject that uses, directly or through others, a subject with a
/// failing claim: the place to look first.
#[derive(Debug, Clone, PartialEq)]
pub struct Suspect {
    pub subject: String,
    pub uses: String,
    /// The failing claims on `uses`.
    pub failing: Vec<String>,
}

/// A pass and a failure on the same input: they can't both be true.
#[derive(Debug, Clone, PartialEq)]
pub struct Contradiction {
    pub subject: String,
    /// The inputs both are about, as text.
    pub inputs: String,
    /// The evidence line and the evidence as text.
    pub pass: (usize, String),
    pub fail: (usize, String),
}

/// A failing subject whose `only uses` claim holds, while nothing fails
/// in what it uses: the fault is its own.
#[derive(Debug, Clone, PartialEq)]
pub struct Root {
    pub subject: String,
    /// The `only uses` claim.
    pub by: String,
    pub used: Vec<String>,
    /// Every claim on the subjects used holds. Otherwise the fault may
    /// also be where no claim covers them.
    pub covered: bool,
}

impl Root {
    pub fn text(&self) -> String {
        let used: Vec<String> = self.used.iter().map(|g| format!("`{g}`")).collect();
        format!(
            "`{}` fails and only uses {}, where nothing fails: the fault is in `{}` itself{}",
            self.subject,
            used.join(", "),
            self.subject,
            if self.covered {
                ""
            } else {
                ", or where no claim covers them"
            }
        )
    }
}

/// Evidence Mitra cannot use as given.
#[derive(Debug, Clone, PartialEq)]
pub struct Flag {
    pub line: usize,
    pub message: String,
}

/// Something that may be wrong.
#[derive(Debug, Clone, PartialEq)]
pub enum Piece {
    /// A ledger entry, by its line.
    Evidence(usize),
    /// The code of a subject.
    Code(String),
    /// A claim: an assumption, or a claim on specs alone.
    Claim(String),
}

/// How plausibly a piece is the wrong one.
#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    pub piece: Piece,
    pub belief: f64,
    pub plausibility: f64,
}

/// Why the candidates of a problem are not ranked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unranked {
    /// Too many pieces with a bias strictly between 0 and 1, counting
    /// those trusted at 1 when they can't all be right.
    TooMany,
}

/// An error: a claim that fails, or pieces that can't all be true.
#[derive(Debug, Clone, PartialEq)]
pub struct Problem {
    pub kind: Kind,
    /// The evidence failing the model: line and text.
    pub evidence: Vec<(usize, String)>,
    /// Sets of pieces that can't all be right.
    pub clauses: Vec<Vec<Piece>>,
    /// The pieces, most plausibly wrong first.
    pub candidates: Result<Vec<Candidate>, Unranked>,
    /// How much the biases say that nothing is wrong.
    pub conflict: f64,
}

/// What is not possible.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// Claims on a subject or a spec that evidence refutes. Several claims
    /// failing on the same evidence share one error.
    Fails { claims: Vec<String> },
    /// A `uniform` or `uses` claim that evidence denies.
    Denied { claim: String },
    /// Two pieces of evidence that disagree on the same inputs.
    Contradiction { subject: String, inputs: String },
    /// A pass and a failure in one class of a `uniform` claim.
    Conflict {
        claim: String,
        spec: Option<String>,
        class: Value,
        calls: Vec<String>,
    },
}

impl Kind {
    /// The stable code of this kind of error.
    pub fn code(&self) -> &'static str {
        match self {
            Kind::Fails { .. } => "M001",
            Kind::Denied { .. } => "M002",
            Kind::Contradiction { .. } => "M003",
            Kind::Conflict { .. } => "M004",
        }
    }
}

/// How much of an error is shown.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Display {
    /// The most candidates listed per error.
    pub max: usize,
    /// Candidates under this plausibility are hidden.
    pub threshold: f64,
}

impl Default for Display {
    fn default() -> Self {
        Display {
            max: 5,
            threshold: 0.05,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub verdicts: Vec<Verdict>,
    pub problems: Vec<Problem>,
    pub flags: Vec<Flag>,
    pub contradictions: Vec<Contradiction>,
    pub conflicts: Vec<Conflict>,
    pub suspects: Vec<Suspect>,
    pub roots: Vec<Root>,
    /// The model and the ledger the report is about, to print it.
    pub model: Model,
}

type Env = HashMap<String, Value>;

/// How a call of a subject gets its value.
#[derive(Clone, Copy)]
enum Subjects<'a> {
    /// From observed outputs.
    Observed,
    /// From these premises, used as definitions: a premise
    /// `g(pattern) matches e` replaces `g(x)` by `e` on its domain.
    Assumed(&'a [&'a Claim]),
}

/// Calls nested deeper than this are given up (no value).
const MAX_DEPTH: usize = 256;

struct Eval<'m> {
    model: &'m Model,
    /// Observed outputs: (subject, inputs) -> outputs.
    observed: BTreeMap<(String, Vec<Value>), Vec<Value>>,
    depth: Cell<usize>,
}

impl Eval<'_> {
    /// Every value an expression can take. Specs give one value; a subject
    /// gives each value it has (none when unknown).
    fn eval(
        &self,
        e: &Expr,
        env: &Env,
        line: usize,
        subjects: Subjects,
    ) -> Result<Vec<Value>, Error> {
        let err = |m: String| Err(Error::new(line, m));
        Ok(match e {
            Expr::Lit(v) => vec![v.clone()],
            Expr::Var(x) => match env.get(x) {
                Some(v) => vec![v.clone()],
                None => return err(format!("unknown name `{x}`")),
            },
            Expr::Call(f, args) => {
                let args = args
                    .iter()
                    .map(|a| self.eval(a, env, line, subjects))
                    .collect::<Result<Vec<_>, _>>()?;
                let mut out = Vec::new();
                for tuple in product(&args) {
                    out.extend(self.call(f, &tuple, line, subjects)?);
                }
                out
            }
            Expr::Len(a) => {
                let mut out = Vec::new();
                for v in self.eval(a, env, line, subjects)? {
                    match v {
                        Value::List(vs) => out.push(Value::Int(vs.len() as i64)),
                        v => return err(format!("`len` needs a list, got {v}")),
                    }
                }
                out
            }
            Expr::Index(a, i) => {
                let (xs, is) = (
                    self.eval(a, env, line, subjects)?,
                    self.eval(i, env, line, subjects)?,
                );
                let mut out = Vec::new();
                for x in &xs {
                    for i in &is {
                        let at = match (x, i) {
                            (Value::List(vs), Value::Int(i)) => {
                                usize::try_from(*i).ok().and_then(|i| vs.get(i))
                            }
                            _ => None,
                        };
                        match at {
                            Some(v) => out.push(v.clone()),
                            None => return err(format!("`{x}[{i}]` is not defined")),
                        }
                    }
                }
                out
            }
            Expr::Tuple(args) | Expr::Record(_, args) | Expr::List(args) => {
                let args = args
                    .iter()
                    .map(|a| self.eval(a, env, line, subjects))
                    .collect::<Result<Vec<_>, _>>()?;
                let mut out = Vec::new();
                for parts in product(&args) {
                    let Expr::Record(name, _) = e else {
                        out.push(match e {
                            Expr::List(_) => Value::List(parts),
                            _ => Value::Tuple(parts),
                        });
                        continue;
                    };
                    let v = Value::Record(name.clone(), parts);
                    let Some(r) = self.model.record(name) else {
                        return err(format!("unknown record `{name}`"));
                    };
                    if !r.set().contains(&v) {
                        return err(format!("`{v}` is not a `{name}`: {}", r.set()));
                    }
                    out.push(v);
                }
                out
            }
            Expr::Field(a, field) => {
                let mut out = Vec::new();
                for v in self.eval(a, env, line, subjects)? {
                    let part = match &v {
                        Value::Tuple(vs) => field.parse::<usize>().ok().and_then(|i| vs.get(i)),
                        Value::Record(name, vs) => self
                            .model
                            .record(name)
                            .and_then(|r| r.fields.iter().position(|f| &f.0 == field))
                            .and_then(|i| vs.get(i)),
                        _ => None,
                    };
                    match part {
                        Some(p) => out.push(p.clone()),
                        None => return err(format!("`{v}` has no `.{field}`")),
                    }
                }
                out
            }
            Expr::Unary(op, a) => {
                let mut out = Vec::new();
                for v in self.eval(a, env, line, subjects)? {
                    match unary(op, &v) {
                        Some(r) => out.push(r),
                        None => return err(format!("`{op}({v})` is not defined")),
                    }
                }
                out
            }
            Expr::Binary(op, a, b) => {
                let (xs, ys) = (
                    self.eval(a, env, line, subjects)?,
                    self.eval(b, env, line, subjects)?,
                );
                let mut out = Vec::new();
                for x in &xs {
                    for y in &ys {
                        match binary(op, x, y) {
                            Some(v) => out.push(v),
                            None => return err(format!("`{x} {op} {y}` is not defined")),
                        }
                    }
                }
                out
            }
            Expr::If(c, t, f) => {
                let mut out = Vec::new();
                for v in self.eval(c, env, line, subjects)? {
                    match v {
                        Value::Bool(true) => out.extend(self.eval(t, env, line, subjects)?),
                        Value::Bool(false) => out.extend(self.eval(f, env, line, subjects)?),
                        v => return err(format!("`if` needs a Bool, got {v}")),
                    }
                }
                out
            }
            Expr::Let(x, v, body) => {
                let mut out = Vec::new();
                for v in self.eval(v, env, line, subjects)? {
                    let mut env = env.clone();
                    env.insert(x.clone(), v);
                    out.extend(self.eval(body, &env, line, subjects)?);
                }
                out
            }
        })
    }

    fn call(
        &self,
        f: &str,
        args: &[Value],
        line: usize,
        subjects: Subjects,
    ) -> Result<Vec<Value>, Error> {
        let Some(func) = self.model.function(f) else {
            return Err(Error::new(line, format!("unknown function `{f}`")));
        };
        if args.len() != func.params.len() {
            return Err(Error::new(
                line,
                format!(
                    "`{f}` takes {} input(s), got {}",
                    func.params.len(),
                    args.len()
                ),
            ));
        }
        if !func.domain().contains(args) {
            return Err(Error::new(
                line,
                format!(
                    "`{}` is outside the domain of `{f}`",
                    crate::print::call(f, args)
                ),
            ));
        }
        if self.depth.get() >= MAX_DEPTH {
            return Ok(vec![]);
        }
        self.depth.set(self.depth.get() + 1);
        let out = self.call_inner(func, args, subjects);
        self.depth.set(self.depth.get() - 1);
        out
    }

    fn call_inner(
        &self,
        func: &Function,
        args: &[Value],
        subjects: Subjects,
    ) -> Result<Vec<Value>, Error> {
        if let Some(body) = &func.body {
            let env: Env = func
                .params
                .iter()
                .map(|p| p.0.clone())
                .zip(args.iter().cloned())
                .collect();
            let out = self.eval(body, &env, func.line, Subjects::Observed)?;
            if let Some(v) = out.iter().find(|v| !func.range.contains(v)) {
                return Err(Error::new(
                    func.line,
                    format!(
                        "`{}` = {v} is outside its result type",
                        crate::print::call(&func.name, args)
                    ),
                ));
            }
            return Ok(out);
        }
        match subjects {
            Subjects::Observed => Ok(self
                .observed
                .get(&(func.name.clone(), args.to_vec()))
                .cloned()
                .unwrap_or_default()),
            Subjects::Assumed(premises) => {
                let rule = premises.iter().find(|p| {
                    p.func == func.name
                        && matches!(p.target, Target::Matches(_))
                        && self.model.covers(&p.func, &p.pattern).contains(args)
                });
                match rule {
                    Some(p) => {
                        let Target::Matches(e) = &p.target else {
                            unreachable!()
                        };
                        self.target(p, e, args, subjects)
                    }
                    None => Ok(vec![]),
                }
            }
        }
    }

    /// The values of a `matches` target on input `x`. A bare function name
    /// is that function applied to `x`.
    fn target(
        &self,
        c: &Claim,
        target: &Expr,
        x: &[Value],
        subjects: Subjects,
    ) -> Result<Vec<Value>, Error> {
        let bound = |v: &String| c.pattern.iter().any(|p| p.var.as_ref() == Some(v));
        match target {
            Expr::Var(g) if !bound(g) && self.model.function(g).is_some() => {
                self.call(g, x, c.line, subjects)
            }
            e => {
                let env: Env = c
                    .pattern
                    .iter()
                    .zip(x)
                    .filter_map(|(p, v)| Some((p.var.clone()?, v.clone())))
                    .collect();
                self.eval(e, &env, c.line, subjects)
            }
        }
    }

    /// Whether the claim is right, wrong (or both) on `x`, or `None` when a
    /// side has no value.
    fn compare(
        &self,
        c: &Claim,
        target: &Expr,
        x: &[Value],
        subjects: Subjects,
    ) -> Result<Option<(bool, bool)>, Error> {
        let lhs = self.call(&c.func, x, c.line, subjects)?;
        let rhs = self.target(c, target, x, subjects)?;
        if lhs.is_empty() || rhs.is_empty() {
            return Ok(None);
        }
        let right = lhs.iter().any(|l| rhs.contains(l));
        let wrong = lhs.iter().any(|l| rhs.iter().any(|r| r != l));
        Ok(Some((right, wrong)))
    }

    /// Check `premises ⇒ c` by substitution on every input of a finite
    /// domain.
    fn implication(
        &self,
        c: &Claim,
        domain: &Tuples,
        premises: &[&Claim],
    ) -> Result<Implication, Error> {
        let (Target::Matches(target), Some(inputs)) = (&c.target, domain.values()) else {
            return Ok(Implication::Unchecked);
        };
        let mut unknown = false;
        for x in inputs {
            match self.compare(c, target, &x, Subjects::Assumed(premises))? {
                Some((_, true)) => return Ok(Implication::Refuted),
                Some(_) => {}
                None => unknown = true,
            }
        }
        Ok(if unknown {
            Implication::Unchecked
        } else {
            Implication::Checked
        })
    }
}

/// Every combination of one value per position.
fn product(lists: &[Vec<Value>]) -> Vec<Vec<Value>> {
    lists.iter().fold(vec![vec![]], |acc, vs| {
        acc.iter()
            .flat_map(|t| {
                vs.iter()
                    .map(move |v| [t.clone(), vec![v.clone()]].concat())
            })
            .collect()
    })
}

fn unary(op: &str, v: &Value) -> Option<Value> {
    match (op, v) {
        ("!", Value::Bool(b)) => Some(Value::Bool(!b)),
        ("-", Value::Int(i)) => i.checked_neg().map(Value::Int),
        ("-", Value::Float(x)) => Some(Value::Float(-x)),
        _ => None,
    }
}

fn binary(op: &str, x: &Value, y: &Value) -> Option<Value> {
    use Value::*;
    Some(match (op, x, y) {
        ("==", x, y) => Bool(x == y),
        ("!=", x, y) => Bool(x != y),
        ("&&", Bool(a), Bool(b)) => Bool(*a && *b),
        ("||", Bool(a), Bool(b)) => Bool(*a || *b),
        ("<", Int(a), Int(b)) => Bool(a < b),
        ("<=", Int(a), Int(b)) => Bool(a <= b),
        (">", Int(a), Int(b)) => Bool(a > b),
        (">=", Int(a), Int(b)) => Bool(a >= b),
        ("+", Int(a), Int(b)) => Int(a.checked_add(*b)?),
        ("-", Int(a), Int(b)) => Int(a.checked_sub(*b)?),
        ("*", Int(a), Int(b)) => Int(a.checked_mul(*b)?),
        // Floats follow IEEE 754, except `==` and `!=` above, which compare
        // values: `NaN == NaN` is true.
        ("<", Float(a), Float(b)) => Bool(a < b),
        ("<=", Float(a), Float(b)) => Bool(a <= b),
        (">", Float(a), Float(b)) => Bool(a > b),
        (">=", Float(a), Float(b)) => Bool(a >= b),
        ("+", Float(a), Float(b)) => Float(a + b),
        ("-", Float(a), Float(b)) => Float(a - b),
        ("*", Float(a), Float(b)) => Float(a * b),
        _ => return None,
    })
}

/// Evidence over at most this many inputs is also read input by input, to
/// find the class of each (see `uniform`).
pub(crate) const POINTS: f64 = 10_000.0;

/// The class of every input, for a `uniform` claim without a spec.
const ONE_CLASS: Value = Value::Bool(true);

/// `class "rest" of `div_class``, `every input`, or for a claim that calls
/// subjects `classes ("neg", "pos") of `f(x)`, `f(-x)``.
pub fn class_text(classes: &[Value], spec: &Option<String>, calls: &[String]) -> String {
    let word = if classes.len() == 1 && calls.len() < 2 {
        "class"
    } else {
        "classes"
    };
    let raw = |c: &Value| match c {
        Value::Str(s) if calls.len() > 1 => format!("({s})"),
        Value::Str(s) if !calls.is_empty() => s.clone(),
        c => c.to_string(),
    };
    let names: Vec<String> = classes.iter().map(raw).collect();
    let calls: Vec<String> = calls.iter().map(|c| format!("`{c}`")).collect();
    match (spec, calls.is_empty()) {
        (_, false) => format!("{word} {} of {}", names.join(", "), calls.join(", ")),
        (Some(spec), true) => format!("{word} {} of `{spec}`", names.join(", ")),
        (None, true) => "every input".to_string(),
    }
}

/// How the inputs of a claim are sorted into classes.
enum Sorted {
    /// No `uniform` claim applies.
    No,
    /// The claim calls subjects, and this keeps their `uniform` claims
    /// from being used.
    Unsorted(String),
    By(Sorting),
}

/// One class per call of a subject, the claim's own call first.
struct Sorting {
    /// The `uniform` claims relied on, the one of the claim's subject first.
    claims: Vec<usize>,
    parts: Vec<Part>,
    /// The calls that have classes, as text. Empty for a claim that calls
    /// no subject but its own.
    calls: Vec<String>,
}

struct Part {
    spec: Option<String>,
    /// The arguments of the call; `None`: the claim's inputs.
    args: Option<Vec<Expr>>,
    /// The line of the `uniform` claim.
    line: usize,
}

/// An argument Mitra can compute from the claim's inputs alone.
fn computable(e: &Expr, bound: &[&String], model: &Model) -> bool {
    let all = |es: &[&Expr]| es.iter().all(|e| computable(e, bound, model));
    match e {
        Expr::Lit(_) => true,
        Expr::Var(x) => bound.contains(&x),
        Expr::Unary(_, a) | Expr::Field(a, _) | Expr::Len(a) => all(&[a]),
        Expr::Tuple(args) | Expr::Record(_, args) | Expr::List(args) => {
            args.iter().all(|a| computable(a, bound, model))
        }
        Expr::Binary(_, a, b) | Expr::Index(a, b) => all(&[a, b]),
        Expr::Call(f, args) => {
            model.function(f).is_some_and(|f| f.is_spec())
                && args.iter().all(|a| computable(a, bound, model))
        }
        Expr::If(..) | Expr::Let(..) => false,
    }
}

/// The calls of subjects in an expression, with their arguments.
fn subject_calls<'e>(e: &'e Expr, model: &Model, out: &mut Vec<(&'e String, &'e [Expr])>) {
    match e {
        Expr::Lit(_) | Expr::Var(_) => {}
        Expr::Call(f, args) => {
            if model.function(f).is_some_and(|f| !f.is_spec()) {
                out.push((f, args));
            }
            args.iter().for_each(|a| subject_calls(a, model, out));
        }
        Expr::Unary(_, a) | Expr::Field(a, _) | Expr::Len(a) => subject_calls(a, model, out),
        Expr::Tuple(args) | Expr::Record(_, args) | Expr::List(args) => {
            args.iter().for_each(|a| subject_calls(a, model, out))
        }
        Expr::Binary(_, a, b) | Expr::Let(_, a, b) | Expr::Index(a, b) => {
            subject_calls(a, model, out);
            subject_calls(b, model, out);
        }
        Expr::If(a, b, c) => {
            subject_calls(a, model, out);
            subject_calls(b, model, out);
            subject_calls(c, model, out);
        }
    }
}

/// How claim `i` is sorted into classes by the `uniform` claims.
fn sorted(model: &Model, i: usize) -> Sorted {
    let c = &model.claims[i];
    if c.target.is_structural() {
        return Sorted::No;
    }
    let uniform = |f: &str| {
        model
            .claims
            .iter()
            .position(|u| u.func == f && matches!(u.target, Target::Uniform(_)) && u.name != c.name)
    };
    let bound: Vec<&String> = c.pattern.iter().filter_map(|p| p.var.as_ref()).collect();
    // The claim's own call, then the calls of its other side.
    let mut found: Vec<(&String, Option<&[Expr]>)> = vec![(&c.func, None)];
    if let Target::Matches(e) = &c.target {
        match e {
            Expr::Var(g) if !bound.contains(&g) => {
                if model.function(g).is_some_and(|g| !g.is_spec()) {
                    found.push((g, None));
                }
            }
            e => {
                let mut calls = Vec::new();
                subject_calls(e, model, &mut calls);
                found.extend(calls.into_iter().map(|(g, args)| (g, Some(args))));
            }
        }
    }
    let lacking: Vec<&String> = found
        .iter()
        .map(|(g, _)| *g)
        .filter(|g| uniform(g).is_none())
        .collect();
    if lacking.len() == found.len() || (found.len() == 1 && !lacking.is_empty()) {
        return Sorted::No;
    }
    if let Some(g) = lacking.first() {
        return Sorted::Unsorted(format!(
            "`{g}` has no `uniform` claim, so no test stands for other inputs"
        ));
    }
    let params = model
        .function(&c.func)
        .map(|f| &f.params[..])
        .unwrap_or(&[]);
    let own = format!("{}{}", c.func, crate::print::pattern(&c.pattern, params).1);
    let (mut claims, mut parts, mut calls) = (Vec::new(), Vec::new(), Vec::new());
    for (g, args) in &found {
        if args.is_some_and(|a| !a.iter().all(|a| computable(a, &bound, model))) {
            return Sorted::Unsorted(format!(
                "a call of `{g}` takes more than the claim's inputs, so no test stands for other inputs"
            ));
        }
        let u = uniform(g).expect("not lacking");
        let Target::Uniform(spec) = &model.claims[u].target else {
            unreachable!()
        };
        if !claims.contains(&u) {
            claims.push(u);
        }
        if spec.is_some() && found.len() > 1 {
            calls.push(match args {
                Some(args) => crate::print::expr(&Expr::Call(g.to_string(), args.to_vec())),
                None if *g == &c.func => own.clone(),
                None => own.replacen(c.func.as_str(), g, 1),
            });
        }
        parts.push(Part {
            spec: spec.clone(),
            args: args.map(|a| a.to_vec()),
            line: model.claims[u].line,
        });
    }
    Sorted::By(Sorting {
        claims,
        parts,
        calls,
    })
}

/// One judged input: the input, whether it passed, the evidence line.
type Point = (Vec<Value>, bool, usize);

/// What the evidence says about a claim, before derivations and classes.
struct Direct {
    domain: Tuples,
    right: Tuples,
    wrong: Tuples,
    /// Single inputs judged for this claim, contradictions left out.
    points: Vec<Point>,
    /// A failure over a set too large to read input by input: its classes
    /// are unknown.
    blind: bool,
    /// Inputs with both a pass and a failure: their classes are not relied
    /// on.
    tainted: Vec<Vec<Value>>,
    /// Each pass or failure: its inputs, its side and its evidence line.
    sources: Vec<(Tuples, bool, usize)>,
}

/// The inputs of `set` one by one, when there are few enough.
fn listed(set: &Tuples) -> Vec<Vec<Value>> {
    match set.size_bound() {
        Some(n) if n <= POINTS => set.values().unwrap_or_default(),
        _ => Vec::new(),
    }
}

pub fn evaluate(model: &Model) -> Result<Report, (Input, Error)> {
    let mut observed: BTreeMap<(String, Vec<Value>), Vec<Value>> = BTreeMap::new();
    let mut flags = Vec::new();
    for e in model.evidence.iter() {
        let Some(f) = model.function(&e.about) else {
            continue;
        };
        if f.is_spec() {
            flags.push(Flag {
                line: e.line,
                message: format!(
                    "`{}` is a spec, true by definition: evidence does not apply to it",
                    f.name
                ),
            });
        } else if let Judgment::Output(out) = &e.judgment {
            let input = single(&e.pattern).expect("checked");
            observed
                .entry((f.name.clone(), input))
                .or_default()
                .push(out.clone());
        }
    }
    let ev = Eval {
        model,
        observed,
        depth: Cell::new(0),
    };
    let model_err = |e| (Input::Model, e);

    let shown = |line: usize| {
        let e = model.evidence.iter().find(|e| e.line == line);
        e.map(|e| {
            crate::print::print_evidence(std::slice::from_ref(e), Some(model))
                .trim_end()
                .to_string()
        })
        .unwrap_or_default()
    };

    // 1. What the evidence says about each claim.
    let mut contradictions: Vec<Contradiction> = Vec::new();
    let clash = |all: &mut Vec<Contradiction>, subject: &Function, both: &Tuples, a, b| {
        let k = Contradiction {
            subject: subject.name.clone(),
            inputs: both.show(&subject.domain()),
            pass: (a, shown(a)),
            fail: (b, shown(b)),
        };
        // The same two statements on other lines add nothing.
        let known = |o: &Contradiction| {
            (o.pass.0, o.fail.0) == (a, b)
                || (&o.subject, &o.inputs, &o.pass.1, &o.fail.1)
                    == (&k.subject, &k.inputs, &k.pass.1, &k.fail.1)
        };
        if !all.iter().any(known) {
            all.push(k);
        }
    };
    // Whatever the claims: an input judged correct and not correct, or
    // seen with two outputs.
    let mut split: Vec<(String, Vec<Value>)> = Vec::new();
    for f in model.functions.iter().filter(|f| !f.is_spec()) {
        let about = |e: &&Evidence| e.about == f.name;
        let of = |want: bool| {
            let want = Judgment::Correct(want);
            let on = model.evidence.iter().filter(about);
            on.filter(move |e| e.judgment == want)
        };
        for a in of(true) {
            for b in of(false) {
                let both = model
                    .covers(&f.name, &a.pattern)
                    .intersect(&model.covers(&f.name, &b.pattern));
                if !both.is_empty() {
                    clash(&mut contradictions, f, &both, a.line, b.line);
                }
            }
        }
        let outputs: Vec<(Vec<Value>, &Value, usize)> = model
            .evidence
            .iter()
            .filter(about)
            .filter_map(|e| match &e.judgment {
                Judgment::Output(v) => Some((single(&e.pattern)?, v, e.line)),
                _ => None,
            })
            .collect();
        for (n, (x, v, a)) in outputs.iter().enumerate() {
            if outputs[..n].iter().any(|o| &o.0 == x) {
                continue;
            }
            for (_, _, b) in outputs.iter().filter(|o| &o.0 == x && o.1 != *v) {
                clash(&mut contradictions, f, &Tuples::point(x), *a, *b);
                split.push((f.name.clone(), x.clone()));
            }
        }
    }
    let mut direct = Vec::new();
    for c in &model.claims {
        let func = model.function(&c.func).expect("checked");
        let domain = model.covers(&c.func, &c.pattern);
        let arity = domain.arity();
        // `right` and `wrong` over all inputs of the function, then cut to
        // the claim's domain; `points` keeps the inputs outside it too.
        let (mut right, mut wrong) = (Tuples::empty(arity), Tuples::empty(arity));
        let mut points: Vec<Point> = Vec::new();
        let mut blind = false;
        let mut tainted = Vec::new();
        // Each pass or failure with its inputs and evidence line, to find
        // those that can't both be true.
        let mut sources: Vec<(Tuples, bool, usize)> = Vec::new();
        for e in model.evidence.iter() {
            let ok = match &e.judgment {
                Judgment::Holds(ok) if e.about == c.name => *ok,
                Judgment::Correct(ok)
                    if e.about == c.func && c.target == Target::Correct && !func.is_spec() =>
                {
                    *ok
                }
                _ => continue,
            };
            let mut set = model.covers(&c.func, &e.pattern);
            if matches!(e.judgment, Judgment::Holds(_)) {
                // Evidence on a claim says nothing outside the claim.
                set = set.intersect(&domain);
            }
            let inputs = listed(&set);
            blind |= !ok && inputs.is_empty() && !set.is_empty();
            points.extend(inputs.into_iter().map(|x| (x, ok, e.line)));
            sources.push((set.clone(), ok, e.line));
            let side = if ok { &mut right } else { &mut wrong };
            *side = side.union(&set);
        }
        if let Target::Matches(target) = &c.target {
            // Every input of a finite domain; otherwise every observed input.
            let inputs = domain.values().unwrap_or_else(|| {
                ev.observed
                    .keys()
                    .map(|(_, x)| x.clone())
                    .filter(|x| x.len() == arity && domain.contains(x))
                    .collect()
            });
            // For classes, also the observed inputs outside the domain that
            // the same statement reaches: those that fit the claim's values.
            let reach: Pattern = c
                .pattern
                .iter()
                .map(|p| Position {
                    var: p.var.clone(),
                    set: p.set.clone().filter(|_| p.var.is_none()),
                })
                .collect();
            let reach = model.covers(&c.func, &reach);
            let outside: Vec<Vec<Value>> = ev
                .observed
                .keys()
                .filter(|(f, x)| f == &c.func && reach.contains(x) && !domain.contains(x))
                .map(|(_, x)| x.clone())
                .collect();
            for x in inputs.into_iter().chain(outside) {
                if split.contains(&(c.func.clone(), x.clone())) {
                    // Two outputs for one input: neither is relied on.
                    tainted.push(x);
                    continue;
                }
                let inside = domain.contains(&x);
                if let Some((ok, ko)) = ev
                    .compare(c, target, &x, Subjects::Observed)
                    .map_err(model_err)?
                {
                    let lines: Vec<usize> = model
                        .evidence
                        .iter()
                        .filter(|e| {
                            e.about == c.func
                                && matches!(e.judgment, Judgment::Output(_))
                                && single(&e.pattern).as_ref() == Some(&x)
                        })
                        .map(|e| e.line)
                        .collect();
                    let line = lines.first().copied().unwrap_or(0);
                    let point = Tuples::point(&x);
                    if ok && ko {
                        // The other side has two outputs, reported there.
                        tainted.push(x);
                        continue;
                    }
                    // Every line that records the output is a source.
                    sources.push((point.clone(), ok, line));
                    for l in lines.iter().skip(1) {
                        sources.push((point.clone(), ok, *l));
                    }
                    if ok {
                        if inside {
                            right = right.union(&point);
                        }
                        points.push((x.clone(), true, line));
                    }
                    if ko {
                        if inside {
                            wrong = wrong.union(&point);
                        }
                        points.push((x, false, line));
                    }
                }
            }
        }
        points.retain(|(x, ok, _)| !(if *ok { &wrong } else { &right }).contains(x));
        // A pass and a failure on the same input can't both be true.
        for (pass, _, a) in sources.iter().filter(|s| s.1) {
            for (fail, _, b) in sources.iter().filter(|s| !s.1) {
                let both = pass.intersect(fail);
                if both.is_empty() {
                    continue;
                }
                let inputs = listed(&both);
                blind |= inputs.is_empty();
                tainted.extend(inputs);
                clash(&mut contradictions, func, &both, *a, *b);
            }
        }
        points.retain(|(x, _, _)| !tainted.contains(x));
        direct.push(Direct {
            tainted,
            right: right.intersect(&domain),
            wrong: wrong.intersect(&domain),
            domain,
            points,
            blind,
            sources,
        });
    }
    // Inputs judged `correct` or not, per subject, whatever the claims.
    let judged = |subject: &str| -> Vec<Point> {
        let about =
            |e: &&Evidence| e.about == subject && matches!(e.judgment, Judgment::Correct(_));
        let side = |want: bool| {
            let arity = model.function(subject).expect("checked").params.len();
            model
                .evidence
                .iter()
                .filter(about)
                .filter(|e| e.judgment == Judgment::Correct(want))
                .fold(Tuples::empty(arity), |s, e| {
                    s.union(&model.covers(subject, &e.pattern))
                })
        };
        let (right, wrong) = (side(true), side(false));
        let mut points = Vec::new();
        for e in model.evidence.iter().filter(about) {
            let ok = e.judgment == Judgment::Correct(true);
            for x in listed(&model.covers(subject, &e.pattern)) {
                if !(if ok { &wrong } else { &right }).contains(&x) {
                    points.push((x, ok, e.line));
                }
            }
        }
        points
    };
    let class_of =
        |spec: &Option<String>, x: &[Value], line: usize| -> Result<Value, (Input, Error)> {
            let Some(spec) = spec else {
                return Ok(ONE_CLASS);
            };
            let out = ev
                .call(spec, x, line, Subjects::Observed)
                .map_err(model_err)?;
            Ok(out.into_iter().next().expect("a spec has a value"))
        };
    // Claims that say the same thing of a subject, on whatever domain:
    // they share their judged inputs.
    let same = |i: usize, j: usize| {
        let (c, d) = (&model.claims[i], &model.claims[j]);
        c.func == d.func && c.target == d.target
    };
    let points_of = |i: usize| -> Vec<Point> {
        let mut points = match model.claims[i].target {
            Target::Correct => judged(&model.claims[i].func),
            _ => Vec::new(),
        };
        for j in (0..direct.len()).filter(|j| same(i, *j)) {
            points.extend(direct[j].points.iter().cloned());
        }
        // A contradicted input is neither a pass nor a failure, whichever
        // claim the contradiction was found on.
        let tainted =
            |x: &Vec<Value>| (0..direct.len()).any(|j| same(i, j) && direct[j].tainted.contains(x));
        points.retain(|p| !tainted(&p.0));
        points
    };
    let blind = |i: usize| (0..direct.len()).any(|j| same(i, j) && direct[j].blind);
    // How each claim's inputs are sorted into classes: one class per call
    // of a subject, each by that subject's `uniform` claim.
    let sorting: Vec<Sorted> = (0..model.claims.len()).map(|i| sorted(model, i)).collect();
    let sig = |i: usize, s: &Sorting, x: &[Value]| -> Result<Value, (Input, Error)> {
        if let [only] = &s.parts[..] {
            return class_of(&only.spec, x, only.line);
        }
        let c = &model.claims[i];
        let env: Env = c
            .pattern
            .iter()
            .zip(x)
            .filter_map(|(p, v)| Some((p.var.clone()?, v.clone())))
            .collect();
        let mut names = Vec::new();
        for part in s.parts.iter().filter(|p| p.spec.is_some()) {
            let inputs = match &part.args {
                None => x.to_vec(),
                Some(args) => {
                    let mut inputs = Vec::new();
                    for a in args {
                        let v = ev
                            .eval(a, &env, c.line, Subjects::Observed)
                            .map_err(model_err)?;
                        inputs.extend(v.into_iter().next());
                    }
                    inputs
                }
            };
            names.push(class_of(&part.spec, &inputs, part.line)?.to_string());
        }
        Ok(match names.is_empty() {
            true => ONE_CLASS,
            false => Value::Str(names.join(", ")),
        })
    };
    // Every class a sorting can give.
    let every = |s: &Sorting| -> Vec<Value> {
        let range = |p: &Part| -> Vec<Value> {
            let g = p.spec.as_ref().expect("filtered");
            let all = model.function(g).expect("checked").range.values();
            all.unwrap_or_default()
        };
        if let [only] = &s.parts[..] {
            return match &only.spec {
                Some(_) => range(only),
                None => vec![ONE_CLASS],
            };
        }
        let mut all: Vec<Vec<String>> = vec![Vec::new()];
        for p in s.parts.iter().filter(|p| p.spec.is_some()) {
            let values = range(p);
            all = all
                .iter()
                .flat_map(|a| {
                    values
                        .iter()
                        .map(|v| [a.clone(), vec![v.to_string()]].concat())
                })
                .collect();
        }
        all.into_iter()
            .map(|names| match names.is_empty() {
                true => ONE_CLASS,
                false => Value::Str(names.join(", ")),
            })
            .collect()
    };

    // 2. Uniformity: a class with a pass and a failure is a conflict.
    let mut conflicts = Vec::new();
    for u in &model.claims {
        let Target::Uniform(spec) = &u.target else {
            continue;
        };
        // `correct` evidence without a claim to carry it.
        let stated = |c: &Claim| c.func == u.func && c.target == Target::Correct;
        let mut groups = Vec::new();
        if !model.claims.iter().any(stated) {
            groups.push((None, judged(&u.func)));
        }
        for (i, c) in model.claims.iter().enumerate() {
            let first = (0..i).all(|j| !same(i, j));
            if c.func == u.func && first {
                if let Sorted::By(s) = &sorting[i] {
                    groups.push((Some((i, s)), points_of(i)));
                }
            }
        }
        for (by, group) in groups {
            let mut seen: BTreeMap<Value, [Vec<&Point>; 2]> = BTreeMap::new();
            for p in &group {
                let class = match by {
                    Some((i, s)) => sig(i, s, &p.0)?,
                    None => class_of(spec, &p.0, u.line)?,
                };
                seen.entry(class).or_default()[p.1 as usize].push(p);
            }
            // A claim that calls subjects has its own classes.
            let wide = by.filter(|(_, s)| s.parts.len() > 1);
            let calls = wide.map(|(_, s)| s.calls.clone()).unwrap_or_default();
            let on = wide.map(|(i, _)| model.claims[i].name.clone());
            let with: Vec<String> = wide
                .map(|(_, s)| &s.claims[1..])
                .unwrap_or_default()
                .iter()
                .map(|u| model.claims[*u].name.clone())
                .collect();
            for (class, [fails, passes]) in seen {
                let (Some(fail), Some(pass)) = (fails.first(), passes.first()) else {
                    continue;
                };
                let lines = |side: &[&Point]| -> Vec<(Vec<Value>, usize)> {
                    side.iter().map(|p| (p.0.clone(), p.2)).collect()
                };
                if conflicts
                    .iter()
                    .any(|k: &Conflict| k.claim == u.name && k.class == class && k.on == on)
                {
                    continue;
                }
                conflicts.push(Conflict {
                    claim: u.name.clone(),
                    spec: spec.clone().filter(|_| wide.is_none()),
                    on: on.clone(),
                    calls: calls.clone(),
                    with: with.clone(),
                    class,
                    pass: (pass.0.clone(), pass.2, shown(pass.2)),
                    fail: (fail.0.clone(), fail.2, shown(fail.2)),
                    passes: lines(&passes),
                    fails: lines(&fails),
                });
            }
        }
    }
    let broken = |name: &str| conflicts.iter().any(|k| k.claim == name);
    // A structural claim can be relied on while nothing speaks against it,
    // and is vouched for once evidence covers it.
    let usable = |i: usize| !broken(&model.claims[i].name) && direct[i].wrong.is_empty();
    let vouched = |i: usize| usable(i) && direct[i].right.same(&direct[i].domain);
    let vouched_one = vouched;

    // 3. Verdicts.
    let mut logic_of: HashMap<&str, Logic> = HashMap::new();
    let mut assuming_of: HashMap<&str, Vec<String>> = HashMap::new();
    let mut verdicts = Vec::new();
    // Per claim, the evidence lines of failures that refute it through a
    // class.
    let mut through: Vec<(Vec<usize>, Vec<usize>)> = Vec::new();
    for (i, c) in model.claims.iter().enumerate() {
        let Direct {
            domain,
            right,
            wrong,
            ..
        } = &direct[i];
        let mut assuming: Vec<String> = Vec::new();
        let mut assume = |name: &String| {
            if !assuming.contains(name) {
                assuming.push(name.clone());
            }
        };
        // A derivation: when the implication is not refuted and no premise
        // fails, the claim follows on its whole domain, assuming the
        // premises that are not verified. A direct counterexample still
        // beats a derivation.
        let implication = if c.from.is_empty() {
            None
        } else {
            let premises: Vec<&Claim> = c
                .from
                .iter()
                .map(|p| model.claim(p).expect("checked"))
                .collect();
            Some(ev.implication(c, domain, &premises).map_err(model_err)?)
        };
        let follows = implication.is_some_and(|i| i != Implication::Refuted)
            && c.from.iter().all(|p| logic_of[p.as_str()] != Logic::Fails);
        // An input both right and wrong is a contradiction: it counts on
        // neither side.
        let mut refuted = wrong.minus(right);
        let tested = right.minus(wrong);
        let derived = if follows {
            domain.minus(wrong).minus(&tested)
        } else {
            Tuples::empty(domain.arity())
        };
        if !derived.is_empty() {
            for p in &c.from {
                match logic_of[p.as_str()] {
                    Logic::Assuming => assuming_of[p.as_str()].iter().for_each(&mut assume),
                    Logic::Unverified => assume(p),
                    _ => {}
                }
            }
        }
        let covered = tested.union(&derived);
        // `uniform` claims on the subjects it calls: a passing input stands
        // for its class.
        let mut assumed = None;
        let mut via = Vec::new();
        let mut unsorted = None;
        let mut relied = Vec::new();
        if let Sorted::Unsorted(why) = &sorting[i] {
            unsorted = Some(why.clone());
        }
        if let Sorted::By(s) = &sorting[i] {
            relied = s.claims.clone();
            let u = s.claims[0];
            let class_of = |_: &(), x: &[Value], _: usize| sig(i, s, x);
            let spec = &();
            let line = model.claims[u].line;
            let (mut classes, mut failing) = (Vec::new(), Vec::new());
            let points = points_of(i);
            for p in &points {
                let class = class_of(spec, &p.0, line)?;
                let side = if p.1 { &mut classes } else { &mut failing };
                if !side.contains(&class) {
                    side.push(class);
                }
            }
            // A class with a contradicted input is not relied on.
            let mut tainted = Vec::new();
            for j in (0..direct.len()).filter(|j| same(i, *j)) {
                for x in &direct[j].tainted {
                    let class = class_of(spec, x, line)?;
                    classes.retain(|k| k != &class);
                    if !tainted.contains(&class) {
                        tainted.push(class);
                    }
                }
            }
            classes.sort();
            let blind = blind(i);
            if blind {
                // A failure somewhere unknown: no class can be relied on.
                classes.clear();
            }
            let usable = s.claims.iter().all(|u| usable(*u));
            let vouched = s.claims.iter().all(|u| vouched(*u));
            let open = domain.minus(&covered).minus(wrong);
            let (mut set, mut untested) = (Tuples::empty(domain.arity()), Vec::new());
            if open.size_bound().is_some_and(|n| n <= POINTS) {
                // Only the classes this domain meets count.
                let passing = std::mem::take(&mut classes);
                let mut hit = Vec::new();
                for x in open.values().unwrap_or_default() {
                    let class = class_of(spec, &x, line)?;
                    if vouched && failing.contains(&class) {
                        // A failure stands for its class too.
                        refuted = refuted.union(&Tuples::point(&x));
                        if !hit.contains(&class) {
                            hit.push(class);
                        }
                    } else if passing.contains(&class) {
                        set = set.union(&Tuples::point(&x));
                        if !classes.contains(&class) {
                            classes.push(class);
                        }
                    } else if !untested.contains(&class) {
                        untested.push(class);
                    }
                }
                for p in points.iter().filter(|p| !p.1) {
                    if hit.contains(&class_of(spec, &p.0, line)?) && !via.contains(&p.2) {
                        via.push(p.2);
                    }
                }
            } else {
                untested = every(s);
                untested.retain(|v| !classes.contains(v));
                if untested.is_empty() {
                    set = open;
                }
            }
            untested.sort();
            classes.sort();
            if usable && !set.is_empty() {
                for w in s.claims.iter().filter(|w| !vouched_one(**w)) {
                    assume(&model.claims[*w].name);
                }
            }
            let wide = s.parts.len() > 1;
            let Target::Uniform(spec) = &model.claims[u].target else {
                unreachable!()
            };
            assumed = Some(Assumed {
                blind,
                tainted,
                by: model.claims[u].name.clone(),
                spec: spec.clone().filter(|_| !wide),
                calls: s.calls.clone(),
                unvouched: s
                    .claims
                    .iter()
                    .filter(|w| !vouched_one(**w))
                    .map(|w| model.claims[*w].name.clone())
                    .collect(),
                with: s.claims[1..]
                    .iter()
                    .map(|w| model.claims[*w].name.clone())
                    .collect(),
                vouched,
                usable,
                classes,
                untested,
                failing,
                set,
            });
        }
        through.push((via, relied));
        let with_assumed = match &assumed {
            Some(a) if a.usable => covered.union(&a.set),
            _ => covered.clone(),
        };
        let mut logic = if !refuted.is_empty() {
            Logic::Fails
        } else if !with_assumed.same(domain) {
            Logic::Unverified
        } else if assuming.is_empty() {
            Logic::Holds
        } else {
            Logic::Assuming
        };
        if broken(&c.name) {
            logic = Logic::Fails;
            for k in conflicts.iter().filter(|k| k.claim == c.name) {
                for x in [&k.pass.0, &k.fail.0] {
                    refuted = refuted.union(&Tuples::point(x));
                }
            }
        }
        if logic != Logic::Assuming {
            assuming.clear();
        }
        logic_of.insert(&c.name, logic);
        assuming_of.insert(&c.name, assuming.clone());
        verdicts.push(Verdict {
            claim: c.name.clone(),
            logic,
            assuming,
            missed: domain.minus(&with_assumed).minus(&refuted),
            tested,
            derived,
            follows,
            refuted,
            assumed,
            unsorted,
            domain: domain.clone(),
            implication,
        });
    }

    // What a subject uses, directly or not, by the `uses` claims that do
    // not fail.
    let uses_of = |f: &String| -> Vec<&String> {
        let mut used: Vec<&String> = Vec::new();
        let mut todo = vec![f];
        while let Some(g) = todo.pop() {
            for c in model.claims.iter().filter(|c| &c.func == g) {
                let Target::Uses { used: list, .. } = &c.target else {
                    continue;
                };
                if logic_of[c.name.as_str()] == Logic::Fails {
                    continue;
                }
                for h in list {
                    if h != f && !used.contains(&h) {
                        used.push(h);
                        todo.push(h);
                    }
                }
            }
        }
        used.sort();
        used
    };

    // 3b. Errors: each failing claim, and each set of pieces that can't
    // all be true, with the pieces that may be the wrong one.
    let is_subject = |f: &str| model.function(f).is_some_and(|f| !f.is_spec());
    let mut problems: Vec<Problem> = Vec::new();
    let quoted = |line: usize| (line, shown(line));
    for (i, c) in model.claims.iter().enumerate() {
        let v = &verdicts[i];
        if v.logic != Logic::Fails || broken(&c.name) {
            continue;
        }
        let mut lines: Vec<usize> = Vec::new();
        for j in (0..direct.len()).filter(|j| same(i, *j)) {
            for (set, ok, line) in &direct[j].sources {
                if !ok && !set.intersect(&v.refuted).is_empty() {
                    lines.push(*line);
                }
            }
        }
        lines.sort();
        lines.dedup();
        lines.retain(|l| *l > 0);
        // A failure that refutes the claim through its class does so with
        // the evidence vouching for the uniform claim.
        let (via, by) = &through[i];
        let via: Vec<usize> = via.iter().filter(|l| !lines.contains(l)).copied().collect();
        let mut vouches: Vec<usize> = Vec::new();
        for u in by.iter().filter(|_| !via.is_empty()) {
            for (_, ok, line) in &direct[*u].sources {
                if *ok && !vouches.contains(line) {
                    vouches.push(*line);
                }
            }
        }
        // What may be wrong on the model's side: the code of the subject,
        // of what it uses and of the subjects the claim calls, or the
        // claim itself.
        let mut side: Vec<Piece> = Vec::new();
        if c.target.is_structural() || !is_subject(&c.func) {
            side.push(Piece::Claim(c.name.clone()));
        } else {
            side.push(Piece::Code(c.func.clone()));
            for g in uses_of(&c.func) {
                side.push(Piece::Code(g.clone()));
            }
        }
        if let Target::Matches(e) = &c.target {
            for g in calls(e).into_iter().filter(|g| is_subject(g)) {
                if !side.contains(&Piece::Code(g.clone())) {
                    side.push(Piece::Code(g));
                }
            }
        }
        let with = |l: &usize| [side.clone(), vec![Piece::Evidence(*l)]].concat();
        let mut clauses: Vec<Vec<Piece>> = lines.iter().map(with).collect();
        for l in &via {
            for w in &vouches {
                clauses.push([with(l), vec![Piece::Evidence(*w)]].concat());
            }
        }
        if clauses.is_empty() {
            clauses.push(side.clone());
        }
        lines.extend(via);
        lines.extend(vouches);
        lines.sort();
        lines.dedup();
        let denied = c.target.is_structural();
        let merged = problems.iter_mut().find_map(|p| match &mut p.kind {
            Kind::Fails { claims } if !denied && p.clauses == clauses => Some(claims),
            _ => None,
        });
        if let Some(claims) = merged {
            claims.push(c.name.clone());
            continue;
        }
        let name = c.name.clone();
        problems.push(Problem {
            kind: if denied {
                Kind::Denied { claim: name }
            } else {
                Kind::Fails { claims: vec![name] }
            },
            evidence: lines.into_iter().map(quoted).collect(),
            clauses,
            candidates: Ok(Vec::new()),
            conflict: 0.0,
        });
    }
    for k in &contradictions {
        problems.push(Problem {
            kind: Kind::Contradiction {
                subject: k.subject.clone(),
                inputs: k.inputs.clone(),
            },
            evidence: vec![k.pass.clone(), k.fail.clone()],
            clauses: vec![vec![Piece::Evidence(k.pass.0), Piece::Evidence(k.fail.0)]],
            candidates: Ok(Vec::new()),
            conflict: 0.0,
        });
    }
    for k in &conflicts {
        let u = model.claims.iter().position(|c| c.name == k.claim);
        let u = u.expect("conflicts name claims");
        // The claim stands through the evidence vouching for it: a vouch
        // is a piece where it covers both inputs.
        let mut clauses: Vec<Vec<Piece>> = Vec::new();
        let mut lines: Vec<usize> = Vec::new();
        for (x, pass) in &k.passes {
            for (y, fail) in &k.fails {
                let rest = [Piece::Evidence(*pass), Piece::Evidence(*fail)];
                let mut vouched = false;
                for (set, ok, line) in &direct[u].sources {
                    if *ok && set.contains(x) && set.contains(y) {
                        vouched = true;
                        clauses.push([vec![Piece::Evidence(*line)], rest.to_vec()].concat());
                        lines.push(*line);
                    }
                }
                if !vouched {
                    clauses.push([vec![Piece::Claim(k.claim.clone())], rest.to_vec()].concat());
                }
            }
        }
        // The other `uniform` claims it rests on: each by what vouches for
        // it, or the claim itself.
        for w in &k.with {
            let w = model.claims.iter().position(|c| &c.name == w);
            let w = w.expect("conflicts name claims");
            let mut pieces: Vec<Piece> = Vec::new();
            for (_, ok, line) in &direct[w].sources {
                if *ok && !pieces.contains(&Piece::Evidence(*line)) {
                    pieces.push(Piece::Evidence(*line));
                    lines.push(*line);
                }
            }
            if pieces.is_empty() {
                pieces.push(Piece::Claim(model.claims[w].name.clone()));
            }
            clauses = clauses
                .iter()
                .flat_map(|c| pieces.iter().map(|p| [c.clone(), vec![p.clone()]].concat()))
                .collect();
        }
        clauses.sort_by_key(|c| format!("{c:?}"));
        clauses.dedup();
        lines.sort();
        lines.dedup();
        let mut evidence: Vec<(usize, String)> = lines.into_iter().map(quoted).collect();
        for (_, l) in k.passes.iter().chain(&k.fails) {
            if !evidence.iter().any(|e| e.0 == *l) {
                evidence.push(quoted(*l));
            }
        }
        problems.push(Problem {
            kind: Kind::Conflict {
                claim: k.claim.clone(),
                spec: k.spec.clone(),
                class: k.class.clone(),
                calls: k.calls.clone(),
            },
            evidence,
            clauses,
            candidates: Ok(Vec::new()),
            conflict: 0.0,
        });
    }
    rank(model, &mut problems);
    // The same observation recorded twice gives the same error twice.
    let mut once: Vec<Problem> = Vec::new();
    for p in problems {
        if !once
            .iter()
            .any(|o| (&o.kind, &o.clauses) == (&p.kind, &p.clauses))
        {
            once.push(p);
        }
    }
    let problems = once;

    // 4. Suspects: what a subject uses, directly or not, that fails.
    let failing = |subject: &str| -> Vec<String> {
        let fails = |c: &&Claim| {
            c.func == subject
                && !matches!(c.target, Target::Uses { .. })
                && logic_of[c.name.as_str()] == Logic::Fails
        };
        model
            .claims
            .iter()
            .filter(fails)
            .map(|c| c.name.clone())
            .collect()
    };
    let mut suspects = Vec::new();
    for f in model.functions.iter().filter(|f| !f.is_spec()) {
        for g in uses_of(&f.name) {
            let failing = failing(g);
            if !failing.is_empty() {
                suspects.push(Suspect {
                    subject: f.name.clone(),
                    uses: g.clone(),
                    failing,
                });
            }
        }
    }
    // 5. Root causes: a failing subject that only uses subjects where
    // nothing fails.
    let mut roots: Vec<Root> = Vec::new();
    for (i, c) in model.claims.iter().enumerate() {
        let Target::Uses { used, only: true } = &c.target else {
            continue;
        };
        let suspect = suspects.iter().any(|s: &Suspect| s.subject == c.func);
        // The subject itself must fail, not only a claim on its structure.
        let fails = model.claims.iter().any(|d| {
            d.func == c.func
                && !d.target.is_structural()
                && logic_of[d.name.as_str()] == Logic::Fails
        });
        let named = roots.iter().any(|r| r.subject == c.func);
        if !fails || suspect || named || !vouched(i) {
            continue;
        }
        // Every other `uses` claim on the subject that is not denied counts
        // too, vouched for or not.
        let mut all: Vec<&String> = Vec::new();
        for d in model.claims.iter().filter(|d| d.func == c.func) {
            if let Target::Uses { used, .. } = &d.target {
                if logic_of[d.name.as_str()] != Logic::Fails {
                    all.extend(used);
                }
            }
        }
        // A failure in what it uses that no claim carries: not a root cause.
        if all.iter().any(|g| judged(g).iter().any(|p| !p.1)) {
            continue;
        }
        // Every claim on what it uses holds: the fault can only be here.
        let covered = all.iter().all(|g| {
            let mut on = model.claims.iter().filter(|d| &&d.func == g);
            let mut on = on.by_ref().filter(|d| !d.target.is_structural());
            let mut any = false;
            on.all(|d| {
                any = true;
                logic_of[d.name.as_str()] == Logic::Holds
            }) && any
        });
        roots.push(Root {
            subject: c.func.clone(),
            by: c.name.clone(),
            used: used.clone(),
            covered,
        });
    }
    Ok(Report {
        roots,
        model: model.clone(),
        problems,
        verdicts,
        flags,
        contradictions,
        conflicts,
        suspects,
    })
}

/// What a failed `f x = v` test refutes: its output was not `v`, so each
/// `=` claim on `f` whose other side is `v` at `x` is wrong there.
pub fn failed_output(model: &Model, test: &Test) -> Result<Vec<Evidence>, Error> {
    let e = &test.judgment;
    let (Judgment::Output(v), Some(x)) = (&e.judgment, single(&e.pattern)) else {
        return Ok(Vec::new());
    };
    let ev = Eval {
        model,
        observed: BTreeMap::new(),
        depth: Cell::new(0),
    };
    let mut out = Vec::new();
    for c in model.claims.iter().filter(|c| c.func == e.about) {
        let Target::Matches(target) = &c.target else {
            continue;
        };
        if !model.covers(&c.func, &c.pattern).contains(&x) {
            continue;
        }
        if ev.target(c, target, &x, Subjects::Observed)? == [v.clone()] {
            out.push(Evidence {
                about: c.name.clone(),
                judgment: Judgment::Holds(false),
                ..e.clone()
            });
        }
    }
    Ok(out)
}

/// Fill in, for each problem, how plausibly each of its pieces is the
/// wrong one. Problems that share a piece are weighed together.
fn rank(model: &Model, problems: &mut [Problem]) {
    // The same observation recorded twice is one piece: same judgment,
    // inputs, source and revision. Its bias is the highest given.
    let same = |a: &Evidence, b: &Evidence| {
        (&a.about, &a.pattern, &a.judgment, &a.source, &a.rev)
            == (&b.about, &b.pattern, &b.judgment, &b.source, &b.rev)
    };
    let entry = |line: usize| model.evidence.iter().find(|e| e.line == line);
    let canon = |p: &Piece| match p {
        Piece::Evidence(line) => {
            let first = entry(*line).and_then(|e| model.evidence.iter().find(|o| same(o, e)));
            Piece::Evidence(first.map_or(*line, |o| o.line))
        }
        other => other.clone(),
    };
    let bias = |p: &Piece| match p {
        Piece::Evidence(line) => entry(*line).map(|e| {
            let twins = model.evidence.iter().filter(|o| same(o, e));
            twins.map(|o| o.confidence).fold(0.0, f64::max)
        }),
        _ => None,
    };
    for p in problems.iter_mut() {
        for clause in &mut p.clauses {
            *clause = clause.iter().map(canon).collect();
            clause.dedup();
        }
    }
    type Weighed = Result<(Vec<Candidate>, f64), Unranked>;
    let weigh = |members: &[usize], problems: &[Problem]| -> Weighed {
        let mut pieces: Vec<Piece> = Vec::new();
        let mut clauses: Vec<Vec<usize>> = Vec::new();
        for m in members {
            for clause in &problems[*m].clauses {
                let mut ids = Vec::new();
                for piece in clause {
                    let known = pieces.iter().position(|q| q == piece);
                    ids.push(known.unwrap_or_else(|| {
                        pieces.push(piece.clone());
                        pieces.len() - 1
                    }));
                }
                clauses.push(ids);
            }
        }
        let biases: Vec<Option<f64>> = pieces.iter().map(bias).collect();
        match mitra_ds::culprits::rank(&biases, &clauses) {
            Ok(r) => {
                let candidate = |(i, piece): (usize, Piece)| Candidate {
                    piece,
                    belief: r.belief[i],
                    plausibility: r.plausibility[i],
                };
                let all = pieces.into_iter().enumerate().map(candidate).collect();
                Ok((all, r.conflict))
            }
            Err(_) => Err(Unranked::TooMany),
        }
    };
    let n = problems.len();
    // Groups of problems that share a piece.
    let mut group: Vec<usize> = (0..n).collect();
    let shares = |a: &Problem, b: &Problem| {
        let mut of_b = b.clauses.iter().flatten();
        of_b.any(|x| a.clauses.iter().flatten().any(|y| x == y))
    };
    for i in 0..n {
        for j in 0..i {
            if group[i] != group[j] && shares(&problems[i], &problems[j]) {
                let (from, to) = (group[i], group[j]);
                group
                    .iter_mut()
                    .filter(|g| **g == from)
                    .for_each(|g| *g = to);
            }
        }
    }
    let mut done = vec![false; n];
    for i in 0..n {
        if done[i] {
            continue;
        }
        let members: Vec<usize> = (0..n).filter(|j| group[*j] == group[i]).collect();
        let together = weigh(&members, problems);
        for m in &members {
            done[*m] = true;
            let result = together.clone();
            let p = &mut problems[*m];
            match result {
                Ok((all, conflict)) => {
                    let mut mine: Vec<Candidate> = Vec::new();
                    for piece in p.clauses.iter().flatten() {
                        if !mine.iter().any(|c| &c.piece == piece) {
                            mine.extend(all.iter().find(|c| &c.piece == piece).cloned());
                        }
                    }
                    let key = |c: &Candidate| (c.plausibility, c.belief);
                    mine.sort_by(|a, b| key(b).partial_cmp(&key(a)).expect("no NaN"));
                    p.candidates = Ok(mine);
                    p.conflict = conflict;
                }
                Err(e) => p.candidates = Err(e),
            }
        }
    }
}

/// A set as text, or its size when the text would be too long to read.
pub(crate) fn brief(set: &Tuples, domain: &Tuples) -> String {
    if set.is_empty() {
        return "nothing".to_string();
    }
    let text = set.show(domain);
    if text.len() <= WIDE {
        return text;
    }
    // The bound counts an input twice when two boxes overlap: list a
    // small set to count it exactly.
    match set.size_bound() {
        Some(n) if n <= POINTS => format!("{} inputs", set.count().unwrap_or(n)),
        Some(n) => format!("{n} inputs"),
        None => "a set too long to print".to_string(),
    }
}

/// A set printed longer than this is summed up.
pub(crate) const WIDE: usize = 160;

/// The most evidence lines an error quotes.
pub(crate) const QUOTED: usize = 5;

impl Report {
    /// The claims that are unverified, or hold only under an assumption.
    pub fn warnings(&self) -> usize {
        let warns = |v: &&Verdict| matches!(v.logic, Logic::Unverified | Logic::Assuming);
        self.verdicts.iter().filter(warns).count() + self.flags.len()
    }
}
