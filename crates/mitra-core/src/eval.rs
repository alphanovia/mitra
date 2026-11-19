//! From evidence to verdicts. Every piece of evidence counts, whatever its
//! bias: the bias only ranks what is most plausibly wrong when the model
//! and the evidence can't all be true.

use crate::ast::*;
use crate::set::{Value, MAP, SET};
use crate::tuples::Tuples;
use crate::{Error, Input};
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap};

///
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
    // mitra: implements eval::as_str
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
    /// The inputs a tested one stands for, with the `represented` claim
    /// that says so.
    pub stood: Vec<(String, Tuples)>,
    /// An input where the specs of the functions the claim calls say it
    /// is false: the model contradicts itself there.
    pub against: Option<Vec<Value>>,
    pub claim: String,
    pub logic: Logic,
    /// With `Logic::Assuming`: the unverified claims the verdict rests on.
    pub assuming: Vec<String>,
    /// The claim's inputs that evidence covers directly.
    pub tested: Tuples,
    /// The inputs that follow from the premises of the claim.
    pub derived: Tuples,
    /// With premises: the implication is not refuted and no premise
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
///
#[derive(Debug, Clone, PartialEq)]
pub enum Piece {
    /// A ledger entry, by its line.
    Evidence(usize),
    /// The code of a subject.
    Code(String),
    /// A claim as stated: the statement itself may be the wrong piece.
    Claim(String),
    /// The signature of a function: its inputs and its result type.
    Signature(String),
}

/// How plausibly a piece is the wrong one.
///
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
    /// A measurement that the observations, or another measurement of the
    /// same measure, can't agree with.
    Measure { measure: String, counted: String },
    /// A pass and a failure in one class of a `uniform` claim.
    Conflict {
        claim: String,
        spec: Option<String>,
        class: Value,
        calls: Vec<String>,
    },
    /// Evidence that does not fit the signature of its function.
    Misfit { function: String, why: String },
    /// A claim that the specs of the functions it calls contradict.
    Against {
        claim: String,
        specs: Vec<String>,
        input: String,
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
            Kind::Measure { .. } => "M005",
            Kind::Misfit { .. } => "M006",
            Kind::Against { .. } => "M007",
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
    /// The observed outputs read so far: (subject, inputs).
    read: RefCell<Vec<(String, Vec<Value>)>>,
    /// The lines of the measurements read so far.
    measured: RefCell<Vec<usize>>,
    /// What was counted for the measures that have no value yet.
    notes: RefCell<Vec<String>>,
    /// Vouched claims `f(a, b) == e`, read as "the output of `f` is `e`".
    rules: Vec<Rule>,
    /// The calls being worked out through a rule: a rule is not used to
    /// find the output it is asked for.
    through: RefCell<Vec<(String, Vec<Value>)>>,
    /// The claim being judged: its own vouch gives it nothing.
    claim: RefCell<String>,
    /// The assumptions that gave an output since last cleared.
    leaned: RefCell<Vec<String>>,
    /// The claims that read `correct(…)` and are vouched for: each forces
    /// the judgements that would make it false.
    relations: Vec<Relation>,
    /// A judgement taken as given while a forced one is worked out.
    supposed: RefCell<Vec<((String, Vec<Value>), bool)>>,
}

/// A claim someone vouched for that gives the output of a subject from
/// other outputs: `add(a, b) == add(b, a)`.
struct Rule {
    /// The claim vouched for.
    claim: String,
    /// The claim is an assumption on a boundary: nobody checked it.
    assumed: bool,
    func: String,
    /// At each input of the subject: the name bound there, with its set.
    params: Vec<(String, Option<crate::Set>)>,
    /// Or the value written there, by position.
    fixed: Vec<Option<Expr>>,
    gives: Expr,
    /// The evidence that vouches for the claim.
    line: usize,
}

/// The rules of a model: property claims of the shape `f(x, y) == e`,
/// each input a different name the claim binds, vouched for on the whole
/// claim and nowhere denied.
fn rules(model: &Model) -> Vec<Rule> {
    let mut out = Vec::new();
    for c in &model.claims {
        let on = |ok: bool| {
            let whole = |e: &&Evidence| e.pattern.iter().all(|p| p.set.is_none());
            let about = model.evidence.iter().filter(|e| e.about == c.name);
            about
                .filter(whole)
                .find(|e| e.judgment == Judgment::Holds(ok))
        };
        let denied = model
            .evidence
            .iter()
            .any(|e| e.about == c.name && e.judgment == Judgment::Holds(false));
        let Some(vouch) = on(true).filter(|_| !denied) else {
            continue;
        };
        // `f(x, y) = e` over the whole of the claim's inputs.
        if let Target::Matches(e) = &c.target {
            let params: Vec<(String, Option<crate::Set>)> = c
                .pattern
                .iter()
                .filter_map(|p| Some((p.var.clone()?, p.set.clone())))
                .collect();
            if params.len() == c.pattern.len() {
                out.push(Rule {
                    claim: c.name.clone(),
                    assumed: vouch.source.as_deref() == Some("assumed"),
                    func: c.func.clone(),
                    fixed: vec![None; params.len()],
                    params,
                    gives: e.clone(),
                    line: vouch.line,
                });
            }
            continue;
        }
        let Target::Property(Expr::Binary("==", l, r)) = &c.target else {
            continue;
        };
        for (call, gives) in [(l, r), (r, l)] {
            let Expr::Call(f, args) = &**call else {
                continue;
            };
            if !model.function(f).is_some_and(|f| !f.is_spec()) {
                continue;
            }
            let mut params: Vec<(String, Option<crate::Set>)> = Vec::new();
            let mut fixed = Vec::new();
            let mut names = 0;
            for a in args {
                let bound = |x: &String| c.pattern.iter().find(|p| p.var.as_ref() == Some(x));
                match a {
                    Expr::Var(x) if !params.iter().any(|p| &p.0 == x) => match bound(x) {
                        Some(p) => {
                            params.push((x.clone(), p.set.clone()));
                            fixed.push(None);
                            names += 1;
                        }
                        None => break,
                    },
                    Expr::Var(_) => break,
                    // A value written in place: `fetch("a") == "ok"`.
                    e => {
                        params.push((String::new(), None));
                        fixed.push(Some(e.clone()));
                    }
                }
            }
            if params.len() == args.len() && names == c.pattern.len() {
                out.push(Rule {
                    claim: c.name.clone(),
                    assumed: vouch.source.as_deref() == Some("assumed"),
                    func: f.clone(),
                    fixed,
                    params,
                    gives: (**gives).clone(),
                    line: vouch.line,
                });
            }
        }
    }
    out
}

/// A vouched claim that reads `correct(…)`.
struct Relation {
    claim: String,
    /// The evidence that vouches for it.
    line: usize,
}

fn relations(model: &Model) -> Vec<Relation> {
    let mut out = Vec::new();
    for c in &model.claims {
        let Target::Property(e) = &c.target else {
            continue;
        };
        let mut reads = false;
        crate::ast::walk(e, &mut |e| reads |= matches!(e, Expr::Correct(..)));
        let about = |e: &&Evidence| e.about == c.name && e.pattern.iter().all(|p| p.set.is_none());
        let says = |ok: bool| {
            let mut on = model.evidence.iter().filter(about);
            on.find(|e| e.judgment == Judgment::Holds(ok))
        };
        if let (true, Some(vouch), None) = (reads, says(true), says(false)) {
            out.push(Relation {
                claim: c.name.clone(),
                line: vouch.line,
            });
        }
    }
    out
}

/// What the observations say of a measure.
struct Count {
    /// The value, when every input is known.
    exact: Option<f64>,
    /// The least and the most it can be, given what is known.
    bounds: (f64, f64),
    /// What was counted, in words.
    text: String,
}

/// An expression with the values of `env` written in place of its names.
fn close(e: &Expr, env: &Env, hidden: &mut Vec<String>) -> Expr {
    let b = |e: &Expr, hidden: &mut Vec<String>| Box::new(close(e, env, hidden));
    let all = |es: &[Expr], hidden: &mut Vec<String>| -> Vec<Expr> {
        es.iter().map(|e| close(e, env, hidden)).collect()
    };
    let under = |x: &[String], e: &Expr, hidden: &mut Vec<String>| {
        let n = hidden.len();
        hidden.extend(x.iter().cloned());
        let out = close(e, env, hidden);
        hidden.truncate(n);
        out
    };
    match e {
        Expr::Var(x) if !hidden.contains(x) => match env.get(x) {
            Some(v) => Expr::Lit(v.clone()),
            None => e.clone(),
        },
        Expr::Lit(_) | Expr::Var(_) | Expr::Score(..) => e.clone(),
        Expr::Call(f, args) => Expr::Call(f.clone(), all(args, hidden)),
        Expr::Correct(f, args) => Expr::Correct(f.clone(), all(args, hidden)),
        Expr::Tuple(args) => Expr::Tuple(all(args, hidden)),
        Expr::Record(n, args) => Expr::Record(n.clone(), all(args, hidden)),
        Expr::List(args) => Expr::List(all(args, hidden)),
        Expr::Unary(op, a) => Expr::Unary(op, b(a, hidden)),
        Expr::Field(a, f) => Expr::Field(b(a, hidden), f.clone()),
        Expr::Len(a) => Expr::Len(b(a, hidden)),
        Expr::Binary(op, x, y) => Expr::Binary(op, b(x, hidden), b(y, hidden)),
        Expr::Index(x, y) => Expr::Index(b(x, hidden), b(y, hidden)),
        Expr::If(c, t, f) => Expr::If(b(c, hidden), b(t, hidden), b(f, hidden)),
        Expr::Let(x, v, body) => {
            let v = b(v, hidden);
            Expr::Let(
                x.clone(),
                v,
                Box::new(under(std::slice::from_ref(x), body, hidden)),
            )
        }
        Expr::Match(a, arms) => {
            let a = b(a, hidden);
            let arms = arms
                .iter()
                .map(|arm| Arm {
                    variant: arm.variant.clone(),
                    body: match &arm.variant {
                        Some((_, names)) => under(names, &arm.body, hidden),
                        None => close(&arm.body, env, hidden),
                    },
                })
                .collect();
            Expr::Match(a, arms)
        }
        Expr::Each(kind, x, list, body) => Expr::Each(
            kind,
            x.clone(),
            b(list, hidden),
            Box::new(under(std::slice::from_ref(x), body, hidden)),
        ),
        Expr::Rate(x, set, cond) => Expr::Rate(
            x.clone(),
            set.clone(),
            Box::new(under(std::slice::from_ref(x), cond, hidden)),
        ),
    }
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
                // A variant without fields.
                None if self.model.record(x).is_some_and(|r| r.fields.is_empty()) => {
                    vec![Value::Record(x.clone(), Vec::new())]
                }
                // A function, as a value: its name.
                None if self.model.declared_function(x).is_some() => {
                    vec![Value::Record(x.clone(), Vec::new())]
                }
                None => return err(format!("unknown name `{x}`")),
            },
            Expr::Match(a, arms) => {
                let mut out = Vec::new();
                for v in self.eval(a, env, line, subjects)? {
                    let fits = |arm: &&Arm| match (&arm.variant, &v) {
                        (None, _) => true,
                        (Some((name, fields)), Value::Record(n, parts)) => {
                            name == n && fields.len() == parts.len()
                        }
                        _ => false,
                    };
                    let Some(arm) = arms.iter().find(fits) else {
                        return err(format!("`match` has no arm for {v}"));
                    };
                    let mut env = env.clone();
                    if let (Some((_, fields)), Value::Record(_, parts)) = (&arm.variant, &v) {
                        for (x, p) in fields.iter().zip(parts) {
                            env.insert(x.clone(), p.clone());
                        }
                    }
                    out.extend(self.eval(&arm.body, &env, line, subjects)?);
                }
                out
            }
            Expr::Call(f, args) => {
                let args = args
                    .iter()
                    .map(|a| self.eval(a, env, line, subjects))
                    .collect::<Result<Vec<_>, _>>()?;
                // A name that holds a function: the function it holds.
                let f = match env.get(f) {
                    Some(Value::Record(g, none)) if none.is_empty() => g,
                    Some(v) => return err(format!("`{f}` is {v}, not a function")),
                    None => f,
                };
                let mut out = Vec::new();
                for tuple in product(&args) {
                    out.extend(self.call(f, &tuple, line, subjects)?);
                }
                out
            }
            Expr::Correct(f, args) => {
                // Under premises nothing is judged: a judgement is evidence.
                if matches!(subjects, Subjects::Assumed(_)) {
                    return Ok(Vec::new());
                }
                let mut x = Vec::new();
                for a in args {
                    match &self.eval(a, env, line, subjects)?[..] {
                        [v] => x.push(v.clone()),
                        _ => return Ok(Vec::new()),
                    }
                }
                self.judged(f, &x, line)?
            }
            Expr::Rate(..) | Expr::Score(..) => self.measure(e, env, line)?,
            Expr::Len(a) => {
                let mut out = Vec::new();
                for v in self.eval(a, env, line, subjects)? {
                    let n = match &v {
                        Value::List(vs) => vs.len(),
                        v if v.as_set().is_some() => v.as_set().expect("checked").len(),
                        v if v.as_map().is_some() => v.as_map().expect("checked").len(),
                        v => return err(format!("`len` needs a list, a set or a map, got {v}")),
                    };
                    out.push(Value::Int(n as i64));
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
                            // `m[k]`: the value at a key of a map.
                            (m, k) => m
                                .as_map()
                                .and_then(|m| m.into_iter().find(|(key, _)| *key == k))
                                .map(|(_, v)| v),
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
                    // A set or a map: its list, in order.
                    if let ([Value::List(items)], true) = (&parts[..], name == SET) {
                        out.push(Value::set_of(items.clone()));
                        continue;
                    }
                    if let ([Value::List(pairs)], true) = (&parts[..], name == MAP) {
                        let pairs = pairs.iter().filter_map(|p| match p {
                            Value::Tuple(kv) => Some((kv[0].clone(), kv[1].clone())),
                            _ => None,
                        });
                        match Value::map_of(pairs.collect()) {
                            Some(m) => out.push(m),
                            None => return err("a key comes twice in a map".to_string()),
                        }
                        continue;
                    }
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
                let xs = self.eval(a, env, line, subjects)?;
                // As in Rust, `&&` and `||` stop at the first operand
                // that settles them.
                let settles = |x: &Value| {
                    matches!(
                        (op.to_string().as_str(), x),
                        ("&&", Value::Bool(false)) | ("||", Value::Bool(true))
                    )
                };
                if xs.is_empty() && ["&&", "||"].contains(&op.to_string().as_str()) {
                    // An unknown side: the other one may settle it alone.
                    let ys = self.eval(b, env, line, subjects).unwrap_or_default();
                    let mut out: Vec<Value> = ys.into_iter().filter(settles).collect();
                    out.dedup();
                    return Ok(out);
                }
                if xs.iter().all(settles) {
                    return Ok(xs);
                }
                let ys = self.eval(b, env, line, subjects)?;
                let mut out = Vec::new();
                for x in &xs {
                    if settles(x) {
                        out.push(x.clone());
                        continue;
                    }
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
            Expr::Each(kind, x, list, body) => {
                let mut out = Vec::new();
                for whole in self.eval(list, env, line, subjects)? {
                    let (items, set) = match (&whole, whole.as_set()) {
                        (_, Some(items)) => (items.to_vec(), true),
                        (Value::List(items), _) => (items.clone(), false),
                        _ => {
                            return err(format!("`{kind}` goes over a list or a set, got {whole}"))
                        }
                    };
                    let again = |items: Vec<Value>| match set {
                        true => Value::set_of(items),
                        false => Value::List(items),
                    };
                    let mut env = env.clone();
                    if let ("fold", Expr::Let(acc, start, step)) = (*kind, &**body) {
                        // Every value the result can take, item after item.
                        let mut states = self.eval(start, &env, line, subjects)?;
                        for item in &items {
                            env.insert(x.clone(), item.clone());
                            let mut next = Vec::new();
                            for s in states {
                                env.insert(acc.clone(), s);
                                next.extend(self.eval(step, &env, line, subjects)?);
                            }
                            next.sort();
                            next.dedup();
                            states = next;
                        }
                        out.extend(states);
                        continue;
                    }
                    let mut each = Vec::new();
                    for item in &items {
                        env.insert(x.clone(), item.clone());
                        each.push(self.eval(body, &env, line, subjects)?);
                    }
                    for results in product(&each) {
                        let bools: Option<Vec<bool>> = results
                            .iter()
                            .map(|r| match r {
                                Value::Bool(b) => Some(*b),
                                _ => None,
                            })
                            .collect();
                        out.push(match (*kind, bools) {
                            ("map", _) => again(results),
                            ("all", Some(bs)) => Value::Bool(bs.iter().all(|b| *b)),
                            ("any", Some(bs)) => Value::Bool(bs.iter().any(|b| *b)),
                            ("filter", Some(bs)) => {
                                let kept = items.iter().zip(bs).filter(|(_, b)| *b);
                                again(kept.map(|(v, _)| v.clone()).collect())
                            }
                            _ => return err(format!("`{kind}` needs a Bool for each element")),
                        });
                    }
                }
                out.sort();
                out.dedup();
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
        let deep = |(i, v)| self.model.conforms(&func.name, i, v);
        if !func.domain().contains(args) || !args.iter().enumerate().all(deep) {
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
            let fits = |v: &Value| {
                func.range.contains(v) && self.model.conforms(&func.name, args.len(), v)
            };
            if let Some(v) = out.iter().find(|v| !fits(v)) {
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
            Subjects::Observed => {
                let key = (func.name.clone(), args.to_vec());
                let out = self.observed.get(&key).cloned().unwrap_or_default();
                if !out.is_empty() {
                    self.read.borrow_mut().push(key);
                    return Ok(out);
                }
                // Not observed: a vouched claim may give it from outputs
                // that were.
                if self.through.borrow().contains(&key) {
                    return Ok(out);
                }
                let own = self.claim.borrow().clone();
                for rule in self
                    .rules
                    .iter()
                    .filter(|r| r.func == func.name && r.claim != own)
                {
                    let fits = |(p, v): (&(String, Option<crate::Set>), &Value)| {
                        p.1.as_ref().is_none_or(|s| s.contains(v))
                    };
                    if rule.params.len() != args.len() || !rule.params.iter().zip(args).all(fits) {
                        continue;
                    }
                    let same = |(e, v): (&Option<Expr>, &Value)| match e {
                        None => true,
                        Some(e) => {
                            let got = self.eval(e, &Env::new(), func.line, Subjects::Observed);
                            got.is_ok_and(|got| got == [v.clone()])
                        }
                    };
                    self.through.borrow_mut().push(key.clone());
                    let same = rule.fixed.iter().zip(args).all(same);
                    self.through.borrow_mut().pop();
                    if !same {
                        continue;
                    }
                    let names = rule.params.iter().map(|p| p.0.clone());
                    let env: Env = names
                        .zip(args.iter().cloned())
                        .filter(|p| !p.0.is_empty())
                        .collect();
                    self.through.borrow_mut().push(key.clone());
                    let given = self.eval(&rule.gives, &env, func.line, Subjects::Observed);
                    self.through.borrow_mut().pop();
                    let given = given.unwrap_or_default();
                    let ok = |v: &Value| {
                        func.range.contains(v) && self.model.conforms(&func.name, args.len(), v)
                    };
                    if !given.is_empty() && given.iter().all(ok) {
                        self.measured.borrow_mut().push(rule.line);
                        if rule.assumed {
                            self.leaned.borrow_mut().push(rule.claim.clone());
                        }
                        return Ok(given);
                    }
                }
                Ok(out)
            }
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
        if matches!(c.target, Target::Property(_)) {
            // True, false (or both, when a subject has two outputs).
            let env: Env = c
                .pattern
                .iter()
                .zip(x)
                .filter_map(|(p, v)| Some((p.var.clone()?, v.clone())))
                .collect();
            let values = self.eval(target, &env, c.line, subjects)?;
            if let Some(v) = values.iter().find(|v| !matches!(v, Value::Bool(_))) {
                let msg = format!("`{}` must be true or false, got {v}", c.name);
                return Err(Error::new(c.line, msg));
            }
            let (yes, no) = (Value::Bool(true), Value::Bool(false));
            return Ok(match values.is_empty() {
                true => None,
                false => Some((values.contains(&yes), values.contains(&no))),
            });
        }
        let lhs = self.call(&c.func, x, c.line, subjects)?;
        let rhs = self.target(c, target, x, subjects)?;
        if lhs.is_empty() || rhs.is_empty() {
            return Ok(None);
        }
        let right = lhs.iter().any(|l| rhs.contains(l));
        let wrong = lhs.iter().any(|l| rhs.iter().any(|r| r != l));
        Ok(Some((right, wrong)))
    }

    /// The value of a measure: as measured when a measurement says so,
    /// otherwise counted from the observations when they cover every
    /// input. No value when neither does.
    fn measure(&self, e: &Expr, env: &Env, line: usize) -> Result<Vec<Value>, Error> {
        let closed = close(e, env, &mut Vec::new());
        let key = crate::print::expr(&closed);
        let mut values: Vec<Value> = Vec::new();
        for ev in self.model.evidence.iter().filter(|ev| ev.about == key) {
            if let Judgment::Measured(_, v) = &ev.judgment {
                self.measured.borrow_mut().push(ev.line);
                if !values.contains(&Value::Float(*v)) {
                    values.push(Value::Float(*v));
                }
            }
        }
        if !values.is_empty() {
            // A measurement that another one, or the observations,
            // contradict gives no value until that is settled.
            let (read, lines) = (self.read.take(), self.measured.take());
            let (lo, hi) = self.count(&closed, line)?.bounds;
            self.read.replace(read);
            let fits =
                |v: &Value| matches!(v, Value::Float(v) if *v >= lo - 1e-9 && *v <= hi + 1e-9);
            if values.len() > 1 || !values.iter().all(fits) {
                self.notes.borrow_mut().push(format!(
                    "`{key}` has no value: its measurement is contradicted"
                ));
                return Ok(vec![]);
            }
            self.measured.replace(lines);
            return Ok(values);
        }
        let count = self.count(&closed, line)?;
        if let Some(v) = count.exact {
            return Ok(vec![Value::Float(v)]);
        }
        self.notes
            .borrow_mut()
            .push(format!("`{key}` has no value yet: {}", count.text));
        Ok(vec![])
    }

    /// What the observations say of a measure with no name left to bind.
    fn count(&self, measure: &Expr, line: usize) -> Result<Count, Error> {
        let unknown = |text: &str| Count {
            exact: None,
            bounds: (0.0, 1.0),
            text: text.to_string(),
        };
        const INFINITE: &str = "only a measurement can give it, the set is too large to count";
        match measure {
            Expr::Rate(x, set, cond) => {
                let Some(inputs) = set.values() else {
                    return Ok(unknown(INFINITE));
                };
                if inputs.is_empty() {
                    return Err(Error::new(line, "a rate over an empty set"));
                }
                let (mut yes, mut no) = (0usize, 0usize);
                for v in &inputs {
                    let env: Env = [(x.clone(), v.clone())].into_iter().collect();
                    let out = self.eval(cond, &env, line, Subjects::Observed)?;
                    match &out[..] {
                        [] => {}
                        all if all.iter().all(|v| v == &Value::Bool(true)) => yes += 1,
                        all if all.iter().all(|v| v == &Value::Bool(false)) => no += 1,
                        all if all.iter().all(|v| matches!(v, Value::Bool(_))) => {}
                        [v, ..] => {
                            let msg = format!("a rate counts true or false, got {v}");
                            return Err(Error::new(line, msg));
                        }
                    }
                }
                let n = inputs.len();
                Ok(Count {
                    exact: (yes + no == n).then_some(yes as f64 / n as f64),
                    bounds: (yes as f64 / n as f64, (n - no) as f64 / n as f64),
                    text: format!("{} of {n} inputs counted, {yes} true", yes + no),
                })
            }
            Expr::Score(kind, f) => {
                let func = self.model.function(f).expect("checked");
                let Some(inputs) = func.domain().values() else {
                    return Ok(unknown(INFINITE));
                };
                // (answer, truth): the truth is the answer when it was
                // judged correct, the other one when judged not correct.
                let (mut tp, mut fp, mut tn, mut fneg, mut known) = (0, 0, 0, 0, 0);
                for x in &inputs {
                    let answers = self.observed.get(&(f.clone(), x.clone()));
                    let judged: Vec<(bool, usize)> = self
                        .model
                        .evidence
                        .iter()
                        .filter(|e| &e.about == f && self.model.covers(f, &e.pattern).contains(x))
                        .filter_map(|e| match e.judgment {
                            Judgment::Correct(ok) => Some((ok, e.line)),
                            _ => None,
                        })
                        .collect();
                    let lines: Vec<usize> = judged.iter().map(|j| j.1).collect();
                    let judged: Vec<bool> = judged.iter().map(|j| j.0).collect();
                    let (Some([Value::Bool(answer)]), Some(ok)) =
                        (answers.map(|a| &a[..]), judged.first())
                    else {
                        continue;
                    };
                    if judged.iter().any(|j| j != ok) {
                        continue;
                    }
                    known += 1;
                    self.read.borrow_mut().push((f.clone(), x.clone()));
                    self.measured.borrow_mut().extend(lines);
                    match (*answer, *ok) {
                        (true, true) => tp += 1,
                        (true, false) => fp += 1,
                        (false, true) => tn += 1,
                        (false, false) => fneg += 1,
                    }
                }
                let (top, bottom) = match *kind {
                    "precision" => (tp, tp + fp),
                    "recall" => (tp, tp + fneg),
                    _ => (fp, fp + tn),
                };
                let n = inputs.len();
                let exact = (known == n && bottom > 0).then(|| top as f64 / bottom as f64);
                Ok(Count {
                    exact,
                    bounds: exact.map_or((0.0, 1.0), |v| (v, v)),
                    text: format!("{known} of {n} inputs have an answer and a judgment"),
                })
            }
            _ => Ok(unknown("not a measure")),
        }
    }

    /// The inputs of a property that the observed outputs speak of: every
    /// way to give its names the observed inputs of the calls whose
    /// arguments are names or values.
    fn candidates(&self, c: &Claim, e: &Expr, domain: &Tuples) -> Vec<Vec<Value>> {
        const MOST: usize = 4096;
        let names: Vec<&String> = c.pattern.iter().filter_map(|p| p.var.as_ref()).collect();
        let mut calls = Vec::new();
        subject_calls(e, self.model, &mut calls);
        let plain = |a: &Expr| match a {
            Expr::Var(x) => names.contains(&x),
            Expr::Lit(_) => true,
            _ => false,
        };
        let mut partial: Vec<BTreeMap<&String, &Value>> = vec![BTreeMap::new()];
        for (g, args) in calls.iter().filter(|(_, args)| args.iter().all(plain)) {
            let mut next = Vec::new();
            for known in &partial {
                let mut any = false;
                for ((_, input), _) in self.observed.iter().filter(|((f, _), _)| &f == g) {
                    let mut all = known.clone();
                    let fits = args.iter().zip(input).all(|(a, v)| match a {
                        Expr::Var(x) => *all.entry(x).or_insert(v) == v,
                        Expr::Lit(l) => l == v,
                        _ => false,
                    });
                    if fits && next.len() < MOST {
                        any = true;
                        next.push(all);
                    }
                }
                if !any && next.len() < MOST {
                    next.push(known.clone());
                }
            }
            next.sort();
            next.dedup();
            partial = next;
        }
        let whole = |known: &BTreeMap<&String, &Value>| -> Option<Vec<Value>> {
            names
                .iter()
                .map(|x| known.get(x).cloned().cloned())
                .collect()
        };
        let mut out: Vec<Vec<Value>> = partial.iter().filter_map(whole).collect();
        out.retain(|x| domain.contains(x));
        out.sort();
        out.dedup();
        out
    }

    /// `correct(f(x))` as a value: what the evidence says of that call.
    /// True, false, both when the evidence disagrees, nothing when no
    /// evidence says and no vouched claim settles it.
    fn judged(&self, f: &str, x: &[Value], line: usize) -> Result<Vec<Value>, Error> {
        let key = (f.to_string(), x.to_vec());
        if let Some(s) = self.supposed.borrow().iter().find(|s| s.0 == key) {
            return Ok(vec![Value::Bool(s.1)]);
        }
        let mut out = Vec::new();
        let mut say = |ok: bool| {
            if !out.contains(&Value::Bool(ok)) {
                out.push(Value::Bool(ok));
            }
        };
        for e in self.model.evidence.iter().filter(|e| e.about == f) {
            if let Judgment::Correct(ok) = e.judgment {
                if self.model.covers(f, &e.pattern).contains(x) {
                    self.measured.borrow_mut().push(e.line);
                    say(ok);
                }
            }
        }
        // An observed output is right when it is what a spec gives.
        if let Some(seen) = self.observed.get(&key) {
            for c in self.model.claims.iter().filter(|c| c.func == f) {
                let Target::Matches(target) = &c.target else {
                    continue;
                };
                if !self.model.covers(f, &c.pattern).contains(x) {
                    continue;
                }
                let expected = self.target(c, target, x, Subjects::Observed)?;
                if expected.is_empty() {
                    continue;
                }
                self.read.borrow_mut().push(key.clone());
                seen.iter().for_each(|v| say(expected.contains(v)));
            }
        }
        if out.is_empty() {
            if let Some((ok, vouch)) = self.forced(f, x, line)? {
                self.measured.borrow_mut().push(vouch);
                out.push(Value::Bool(ok));
            }
        }
        Ok(out)
    }

    /// The judgement a vouched claim forces on a call nothing judges:
    /// the one value of `correct(f(x))` that does not make the claim
    /// false on some input, given what is known. With the evidence line
    /// of the vouch.
    fn forced(&self, f: &str, x: &[Value], _line: usize) -> Result<Option<(bool, usize)>, Error> {
        // One at a time: what is forced rests on evidence, not on what
        // another forced value would give.
        if !self.through.borrow().is_empty() || !self.supposed.borrow().is_empty() {
            return Ok(None);
        }
        let key = (f.to_string(), x.to_vec());
        for r in self
            .relations
            .iter()
            .filter(|r| r.claim != *self.claim.borrow())
        {
            let c = self.model.claim(&r.claim).expect("checked");
            let Target::Property(body) = &c.target else {
                continue;
            };
            let names: Vec<Option<&String>> = c.pattern.iter().map(|p| p.var.as_ref()).collect();
            let Some(inputs) = self.model.covers(&c.func, &c.pattern).values() else {
                continue;
            };
            // Each place the claim reads `correct(f(…))` on names alone:
            // the inputs of the claim where that call is this one.
            let mut places: Vec<Vec<(usize, Value)>> = Vec::new();
            crate::ast::walk(body, &mut |e| {
                let Expr::Correct(g, args) = e else { return };
                if g != f || args.len() != x.len() {
                    return;
                }
                let mut fixed = Vec::new();
                for (a, v) in args.iter().zip(x) {
                    let at = match a {
                        Expr::Var(n) => names.iter().position(|m| *m == Some(n)),
                        _ => None,
                    };
                    match at {
                        Some(i) => fixed.push((i, v.clone())),
                        None => return,
                    }
                }
                places.push(fixed);
            });
            for fixed in places {
                let fits = |t: &Vec<Value>| fixed.iter().all(|(i, v)| &t[*i] == v);
                for t in inputs.iter().filter(|t| fits(t)) {
                    for ok in [true, false] {
                        self.supposed.borrow_mut().push((key.clone(), ok));
                        let got = self.compare(c, body, t, Subjects::Observed);
                        self.supposed.borrow_mut().pop();
                        // False for sure with this value: it is the other.
                        if let Ok(Some((false, true))) = got {
                            return Ok(Some((!ok, r.line)));
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    /// An input where the premises say `c` is false.
    fn counter(&self, c: &Claim, domain: &Tuples, premises: &[&Claim]) -> Option<Vec<Value>> {
        let (Target::Matches(target) | Target::Property(target), Some(inputs)) =
            (&c.target, domain.values())
        else {
            return None;
        };
        inputs.into_iter().find(|x| {
            let got = self.compare(c, target, x, Subjects::Assumed(premises));
            matches!(got, Ok(Some((_, true))))
        })
    }

    /// Check `premises ⇒ c` by substitution on every input of a finite
    /// domain.
    fn implication(
        &self,
        c: &Claim,
        domain: &Tuples,
        premises: &[&Claim],
    ) -> Result<Implication, Error> {
        let (Target::Matches(target) | Target::Property(target), Some(inputs)) =
            (&c.target, domain.values())
        else {
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

// mitra: implements eval::unary
pub(crate) fn unary(op: &str, v: &Value) -> Option<Value> {
    match (op, v) {
        ("!", Value::Bool(b)) => Some(Value::Bool(!b)),
        ("-", Value::Int(i)) => i.checked_neg().map(Value::Int),
        ("-", Value::Float(x)) => Some(Value::Float(-x)),
        // The keys of a map, as a set.
        ("keys", m) => {
            let keys = m.as_map()?.into_iter().map(|(k, _)| k.clone());
            Some(Value::set_of(keys.collect()))
        }
        _ => None,
    }
}

// mitra: implements eval::binary
pub(crate) fn binary(op: &str, x: &Value, y: &Value) -> Option<Value> {
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
        // As in Rust: towards zero; by zero is not defined.
        ("/", Int(a), Int(b)) => Int(a.checked_div(*b)?),
        // Floats follow IEEE 754, except `==` and `!=` above, which compare
        // values: `NaN == NaN` is true.
        ("<", Float(a), Float(b)) => Bool(a < b),
        ("<=", Float(a), Float(b)) => Bool(a <= b),
        (">", Float(a), Float(b)) => Bool(a > b),
        (">=", Float(a), Float(b)) => Bool(a >= b),
        ("+", Float(a), Float(b)) => Float(a + b),
        ("-", Float(a), Float(b)) => Float(a - b),
        ("*", Float(a), Float(b)) => Float(a * b),
        ("/", Float(a), Float(b)) => Float(a / b),
        // `x in c`: an element of a list or a set, a key of a map.
        ("in", x, List(vs)) => Bool(vs.contains(x)),
        ("in", x, c) if c.as_set().is_some() => Bool(c.as_set()?.contains(x)),
        ("in", x, c) => Bool(c.as_map()?.iter().any(|(k, _)| *k == x)),
        // Sets: union, intersection, difference.
        ("|", a, b) if a.as_set().is_some() => Value::set_of([a.as_set()?, b.as_set()?].concat()),
        ("&", a, b) if a.as_set().is_some() => {
            let b = b.as_set()?;
            Value::set_of(
                a.as_set()?
                    .iter()
                    .filter(|x| b.contains(x))
                    .cloned()
                    .collect(),
            )
        }
        ("\\", a, b) if a.as_set().is_some() => {
            let b = b.as_set()?;
            Value::set_of(
                a.as_set()?
                    .iter()
                    .filter(|x| !b.contains(x))
                    .cloned()
                    .collect(),
            )
        }
        // Maps: `m | n` is `m` with the entries of `n`, which win;
        // `m \ s` is `m` without the keys in the set `s`.
        ("|", a, b) => {
            let (a, b) = (a.as_map()?, b.as_map()?);
            let kept = a.iter().filter(|(k, _)| !b.iter().any(|(j, _)| j == k));
            let all = kept
                .chain(b.iter())
                .map(|(k, v)| ((*k).clone(), (*v).clone()));
            Value::map_of(all.collect())?
        }
        ("\\", a, b) => {
            let (a, b) = (a.as_map()?, b.as_set()?);
            let kept = a.iter().filter(|(k, _)| !b.contains(k));
            Value::map_of(kept.map(|(k, v)| ((*k).clone(), (*v).clone())).collect())?
        }
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
        // A judgement is read from the evidence, not computed.
        Expr::Correct(..) => false,
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
        Expr::If(..)
        | Expr::Let(..)
        | Expr::Match(..)
        | Expr::Rate(..)
        | Expr::Score(..)
        | Expr::Each(..) => false,
    }
}

/// The calls of subjects in an expression, with their arguments.
fn subject_calls<'e>(e: &'e Expr, model: &Model, out: &mut Vec<(&'e String, &'e [Expr])>) {
    match e {
        // A measure is not a call: no class stands for what it counts.
        Expr::Lit(_) | Expr::Var(_) | Expr::Rate(..) | Expr::Score(..) => {}
        Expr::Correct(_, args) => args.iter().for_each(|a| subject_calls(a, model, out)),
        Expr::Call(f, args) => {
            if model.function(f).is_some_and(|f| !f.is_spec()) {
                out.push((f, args));
            }
            args.iter().for_each(|a| subject_calls(a, model, out));
        }
        Expr::Match(a, arms) => {
            subject_calls(a, model, out);
            arms.iter()
                .for_each(|arm| subject_calls(&arm.body, model, out));
        }
        Expr::Each(_, _, a, b) => {
            subject_calls(a, model, out);
            subject_calls(b, model, out);
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
    if let (Target::Property(e), None) = (&c.target, uniform(&c.func)) {
        // A property has no call of its own: every call has its class,
        // unless a `uniform` claim sorts the inputs of the property itself.
        let mut calls = Vec::new();
        subject_calls(e, model, &mut calls);
        found = calls.into_iter().map(|(g, a)| (g, Some(a))).collect();
        if found.is_empty() {
            return Sorted::No;
        }
    }
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
    /// For a property: each failing input with the lines of every output
    /// it was computed from. They are wrong together or not at all.
    joint: Vec<(Vec<Value>, Vec<usize>)>,
    /// For a claim on a measure that has no value yet: what was counted.
    counted: Option<String>,
    /// The assumptions whose word gave an output the claim read.
    leans: Vec<String>,
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
        read: RefCell::new(Vec::new()),
        measured: RefCell::new(Vec::new()),
        notes: RefCell::new(Vec::new()),
        rules: rules(model),
        through: RefCell::new(Vec::new()),
        claim: RefCell::new(String::new()),
        leaned: RefCell::new(Vec::new()),
        relations: relations(model),
        supposed: RefCell::new(Vec::new()),
    };
    let model_err = |e| (Input::Model, e);

    let shown = |line: usize| {
        let written = model.written.as_ref();
        if let Some(text) = written.and_then(|w| crate::parse::written(&w.1, line)) {
            return text;
        }
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
            // A boundary may answer differently each time.
            if outputs[..n].iter().any(|o| &o.0 == x) || model.boundaries.contains(&f.name) {
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
        ev.claim.replace(c.name.clone());
        ev.leaned.borrow_mut().clear();
        let func = model.function(&c.func).expect("checked");
        let mut domain = model.covers(&c.func, &c.pattern);
        // `when` on the inputs alone: the inputs it
        // refuses are not in the claim.
        let when = format!("{}::when", c.name);
        if let (Some(all), true) = (domain.values(), model.function(&when).is_some()) {
            for x in all {
                let kept = ev.call(&when, &x, c.line, Subjects::Observed);
                if kept.map_err(model_err)? != [Value::Bool(true)] {
                    domain = domain.minus(&Tuples::point(&x));
                }
            }
        }
        let arity = domain.arity();
        // `right` and `wrong` over all inputs of the function, then cut to
        // the claim's domain; `points` keeps the inputs outside it too.
        let (mut right, mut wrong) = (Tuples::empty(arity), Tuples::empty(arity));
        let mut points: Vec<Point> = Vec::new();
        let mut blind = false;
        let mut tainted = Vec::new();
        // For a property: each failing input with every output it read.
        let mut joint: Vec<(Vec<Value>, Vec<usize>)> = Vec::new();
        let mut counted: Option<String> = None;
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
        if let Target::Matches(target) | Target::Property(target) = &c.target {
            let property = matches!(c.target, Target::Property(_));
            // Every input of a finite domain; otherwise every observed input.
            let inputs = domain.values().unwrap_or_else(|| match property {
                true => ev.candidates(c, target, &domain),
                false => ev
                    .observed
                    .keys()
                    .map(|(_, x)| x.clone())
                    .filter(|x| x.len() == arity && domain.contains(x))
                    .collect(),
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
                ev.read.borrow_mut().clear();
                ev.measured.borrow_mut().clear();
                ev.notes.borrow_mut().clear();
                let compared = match ev.compare(c, target, &x, Subjects::Observed) {
                    // Outside the domain the statement need not be defined.
                    Err(_) if !inside => continue,
                    r => r.map_err(model_err)?,
                };
                // A measure without a value: what was counted so far.
                if let (None, Some(note)) = (compared, ev.notes.take().into_iter().next()) {
                    counted.get_or_insert(note);
                }
                // The outputs a property was computed from.
                let mut read = ev.read.take();
                read.sort();
                read.dedup();
                if property && read.iter().any(|r| split.contains(r)) {
                    tainted.push(x);
                    continue;
                }
                if let Some((ok, ko)) = compared {
                    let own = [(c.func.clone(), x.clone())];
                    // A matched output that a vouched claim gave: what it
                    // was worked out from, as for a property.
                    let property = property || !ev.measured.borrow().is_empty();
                    let lines: Vec<usize> = model
                        .evidence
                        .iter()
                        .filter(|e| {
                            let from = if property { &read[..] } else { &own[..] };
                            matches!(e.judgment, Judgment::Output(_))
                                && from.iter().any(|(g, x)| {
                                    &e.about == g && single(&e.pattern).as_ref() == Some(x)
                                })
                        })
                        .map(|e| e.line)
                        .collect();
                    let mut lines = lines;
                    if property {
                        lines.extend(ev.measured.take());
                        lines.sort();
                        lines.dedup();
                    }
                    if property && ko && !ok {
                        joint.push((x.clone(), lines.clone()));
                    }
                    let line = lines.first().copied().unwrap_or(0);
                    let point = Tuples::point(&x);
                    // True of one answer of a boundary and not of another:
                    // the claim is about every answer.
                    let answers = read.iter().any(|(g, _)| model.boundaries.contains(g));
                    let ok = ok && !(ko && answers);
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
        let mut leans = ev.leaned.take();
        leans.sort();
        leans.dedup();
        direct.push(Direct {
            leans,
            counted,
            joint,
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
            // The inputs of the one call: the claim's own, or its arguments.
            let Some(args) = &only.args else {
                return class_of(&only.spec, x, only.line);
            };
            let c = &model.claims[i];
            let names = c.pattern.iter().zip(x);
            let env: Env = names
                .filter_map(|(p, v)| Some((p.var.clone()?, v.clone())))
                .collect();
            let mut inputs = Vec::new();
            for a in args {
                let v = ev.eval(a, &env, c.line, Subjects::Observed);
                inputs.extend(v.map_err(model_err)?.into_iter().next());
            }
            // An argument that reads an output not observed: no class yet.
            if inputs.len() != args.len() {
                return class_of(&only.spec, x, only.line);
            }
            return class_of(&only.spec, &inputs, only.line);
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
        for (i, sorted) in sorting.iter().enumerate() {
            let first = (0..i).all(|j| !same(i, j));
            // A claim counts for the `uniform` claim of its subject; a
            // property, for the one of the first subject it calls.
            if let (Sorted::By(s), true) = (sorted, first) {
                if model.claims[s.claims[0]].name == u.name {
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

    // `represented` claims: each with its set and, when one of its
    // inputs passes a claim and another fails it, the two evidence lines.
    type Stand = (usize, Tuples, Option<(usize, usize)>);
    let mut stands: Vec<Stand> = Vec::new();
    for (r, c) in model.claims.iter().enumerate() {
        let Some(set) = represented(model, c) else {
            continue;
        };
        let mut split = None;
        for (j, d) in model.claims.iter().enumerate() {
            if d.func != c.func || d.target.is_structural() {
                continue;
            }
            let inside = |ok: bool| {
                let on = direct[j].points.iter();
                on.filter(move |p| p.1 == ok).find(|p| set.contains(&p.0))
            };
            if let (Some(pass), Some(fail)) = (inside(true), inside(false)) {
                split.get_or_insert((pass.2, fail.2));
            }
        }
        stands.push((r, set, split));
    }
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
        } else if c.proved {
            Some(Implication::Checked)
        } else {
            let premises: Vec<&Claim> = c
                .from
                .iter()
                .map(|p| model.claim(p).expect("checked"))
                .collect();
            Some(ev.implication(c, domain, &premises).map_err(model_err)?)
        };
        // Premises nobody wrote count only where every input was checked.
        let enough = |i: Implication| match c.auto {
            true => i == Implication::Checked,
            false => i != Implication::Refuted,
        };
        let follows = implication.is_some_and(enough)
            && c.from.iter().all(|p| logic_of[p.as_str()] != Logic::Fails);
        // An input both right and wrong is a contradiction: it counts on
        // neither side.
        let mut refuted = wrong.minus(right);
        // The specs of what it calls say it is false on an input: the
        // model contradicts itself, whatever the evidence.
        let mut against = None;
        if c.auto && implication == Some(Implication::Refuted) {
            let premises: Vec<&Claim> = c
                .from
                .iter()
                .map(|p| model.claim(p).expect("checked"))
                .collect();
            against = ev.counter(c, domain, &premises);
            if let Some(x) = &against {
                refuted = refuted.union(&Tuples::point(x));
            }
        }
        let tested = right.minus(wrong).minus(&refuted);
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
        // An assumption on a boundary gave an output: the claim holds as
        // far as the assumption does.
        for a in &direct[i].leans {
            if logic_of.get(a.as_str()) != Some(&Logic::Holds) {
                assume(a);
            }
        }
        // An assumption itself: nobody checked it where no evidence other
        // than its own word covers it.
        let word = |l: usize| {
            let e = model.evidence.iter().find(|e| e.line == l);
            e.is_some_and(|e| e.source.as_deref() == Some("assumed"))
        };
        let about = |e: &&Evidence| e.about == c.name && word(e.line);
        if model.evidence.iter().any(|e| about(&e)) {
            let mut backed = Tuples::empty(domain.arity());
            for s in direct[i].sources.iter().filter(|s| s.1 && !word(s.2)) {
                backed = backed.union(&s.0);
            }
            if !backed.intersect(domain).same(domain) {
                assume(&c.name);
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
        if unsorted.is_none() {
            unsorted = direct[i].counted.clone();
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
        // One tested input of a `represented` set stands for the set.
        let mut stood = Vec::new();
        let mut with_assumed = with_assumed;
        let own = !c.target.is_structural() && c.func != c.name;
        let subject = matches!(c.target, Target::Correct | Target::Matches(_));
        for (r, set, split) in stands.iter().filter(|_| own && subject) {
            let by = &model.claims[*r];
            let about = |e: &&Evidence| e.about == by.name;
            let says = |ok: bool| {
                let mut on = model.evidence.iter().filter(about);
                on.any(|e| e.judgment == Judgment::Holds(ok))
            };
            let inside = set.intersect(domain);
            let tested_there = !inside.intersect(&tested).is_empty();
            let fails_there = !inside.intersect(&refuted).is_empty();
            if by.func != c.func || split.is_some() || says(false) || !tested_there || fails_there {
                continue;
            }
            let more = inside.minus(&with_assumed);
            if more.is_empty() {
                continue;
            }
            with_assumed = with_assumed.union(&more);
            stood.push((by.name.clone(), more));
            if !says(true) && !assuming.contains(&by.name) {
                assuming.push(by.name.clone());
            }
        }
        // A vouched claim that reads `correct(…)` forces the judgement of
        // the inputs nothing judges.
        if own && subject && !ev.relations.is_empty() {
            let left = domain.minus(&with_assumed).minus(&refuted);
            let few = left.size_bound().is_some_and(|n| n <= 4096.0);
            ev.claim.replace(c.name.clone());
            for x in left.values().filter(|_| few).unwrap_or_default() {
                let Some((true, line)) = ev.forced(&c.func, &x, c.line).map_err(model_err)? else {
                    continue;
                };
                let by = ev
                    .relations
                    .iter()
                    .find(|r| r.line == line)
                    .expect("its own");
                let point = Tuples::point(&x);
                with_assumed = with_assumed.union(&point);
                match stood
                    .iter_mut()
                    .find(|s: &&mut (String, Tuples)| s.0 == by.claim)
                {
                    Some(s) => s.1 = s.1.union(&point),
                    None => stood.push((by.claim.clone(), point)),
                }
            }
        }
        let split = stands.iter().find(|s| s.0 == i).and_then(|s| s.2);
        let mut logic = if !refuted.is_empty() || split.is_some() {
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
            stood,
            against,
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
        if let Some((_, _, Some((pass, fail)))) = stands.iter().find(|s| s.0 == i) {
            // The two inputs did not behave the same: the claim is denied.
            let mut pieces = vec![Piece::Evidence(*pass), Piece::Evidence(*fail)];
            let vouches = model.evidence.iter().filter(|e| e.about == c.name);
            pieces.extend(vouches.map(|e| Piece::Evidence(e.line)));
            pieces.push(Piece::Claim(c.name.clone()));
            problems.push(Problem {
                kind: Kind::Denied {
                    claim: c.name.clone(),
                },
                evidence: vec![quoted(*pass), quoted(*fail)],
                clauses: vec![pieces],
                candidates: Ok(Vec::new()),
                conflict: 0.0,
            });
            continue;
        }
        if let Some(x) = &v.against {
            let mut pieces = vec![Piece::Claim(c.name.clone())];
            pieces.extend(c.from.iter().map(|p| Piece::Claim(p.clone())));
            problems.push(Problem {
                kind: Kind::Against {
                    claim: c.name.clone(),
                    specs: c.from.clone(),
                    input: brief(&Tuples::point(x), &v.domain),
                },
                evidence: Vec::new(),
                clauses: vec![pieces],
                candidates: Ok(Vec::new()),
                conflict: 0.0,
            });
            if direct[i].wrong.is_empty() {
                continue;
            }
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
        if let Target::Property(_) = &c.target {
            for g in model.subjects_of(c) {
                for h in std::iter::once(&g).chain(uses_of(&g)) {
                    if !side.contains(&Piece::Code(h.clone())) {
                        side.push(Piece::Code(h.clone()));
                    }
                }
            }
        } else if c.target.is_structural() || !is_subject(&c.func) {
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
        // A boundary has no code of ours to be wrong: it answers as it
        // does.
        side.retain(|p| !matches!(p, Piece::Code(g) if model.boundaries.contains(g)));
        // The statement of the claim is a piece like the others. For a
        // tie: what the model says of the function, or the code.
        if c.target == Target::Tied {
            side.push(Piece::Signature(c.func.clone()));
            side.push(Piece::Code(c.func.clone()));
        } else {
            side.push(Piece::Claim(c.name.clone()));
        }
        let with = |l: &usize| [side.clone(), vec![Piece::Evidence(*l)]].concat();
        let mut clauses: Vec<Vec<Piece>> = lines.iter().map(with).collect();
        if let Target::Property(_) = &c.target {
            // The outputs a failing input read are wrong together.
            clauses.clear();
            for (x, read) in &direct[i].joint {
                if v.refuted.contains(x) {
                    let read = read.iter().map(|l| Piece::Evidence(*l));
                    let clause: Vec<Piece> = side.iter().cloned().chain(read).collect();
                    if !clauses.contains(&clause) {
                        clauses.push(clause);
                    }
                }
            }
            // A claim refuted by evidence on the claim itself.
            let stated = |l: &&usize| {
                let on = model.evidence.iter().find(|e| e.line == **l);
                on.is_some_and(|e| matches!(e.judgment, Judgment::Holds(_)))
            };
            clauses.extend(lines.iter().filter(stated).map(with));
        }
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
        // Claims refuted by the same pieces share one error: the same
        // clauses, the statements of the claims apart.
        let apart = |clauses: &[Vec<Piece>]| -> Vec<Vec<Piece>> {
            let stated = |p: &&Piece| !matches!(p, Piece::Claim(_));
            let each = clauses.iter();
            each.map(|c| c.iter().filter(stated).cloned().collect())
                .collect()
        };
        let merged = problems.iter_mut().find(|p| {
            matches!(p.kind, Kind::Fails { .. }) && !denied && apart(&p.clauses) == apart(&clauses)
        });
        if let Some(p) = merged {
            if let Kind::Fails { claims } = &mut p.kind {
                claims.push(c.name.clone());
            }
            p.clauses.extend(clauses);
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
    for (e, why) in &model.misfits {
        let written = model.written.as_ref();
        let text = written.and_then(|w| crate::parse::written(&w.1, e.line));
        let text = text.unwrap_or_else(|| {
            let old = crate::print::print_evidence(std::slice::from_ref(e), Some(model));
            old.trim_end().to_string()
        });
        problems.push(Problem {
            kind: Kind::Misfit {
                function: e.about.clone(),
                why: why.clone(),
            },
            evidence: vec![(e.line, text)],
            clauses: vec![vec![
                Piece::Evidence(e.line),
                Piece::Signature(e.about.clone()),
            ]],
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
    // A measurement against another one of the same measure, and against
    // what the observations count.
    let measurements: Vec<(&Evidence, &Expr, f64)> = model
        .evidence
        .iter()
        .filter_map(|e| match &e.judgment {
            Judgment::Measured(m, v) => Some((e, m, *v)),
            _ => None,
        })
        .collect();
    for (n, (e, measure, v)) in measurements.iter().enumerate() {
        for (o, _, w) in measurements[..n].iter().filter(|o| o.0.about == e.about) {
            if w != v {
                problems.push(Problem {
                    kind: Kind::Measure {
                        measure: e.about.clone(),
                        counted: String::new(),
                    },
                    evidence: vec![quoted(o.line), quoted(e.line)],
                    clauses: vec![vec![Piece::Evidence(o.line), Piece::Evidence(e.line)]],
                    candidates: Ok(Vec::new()),
                    conflict: 0.0,
                });
            }
        }
        ev.read.borrow_mut().clear();
        ev.measured.borrow_mut().clear();
        let count = ev
            .count(measure, e.line)
            .map_err(|x| (Input::Evidence, x))?;
        let (lo, hi) = count.bounds;
        if *v < lo - 1e-9 || *v > hi + 1e-9 {
            // The outputs that were counted, with the measurement.
            let read = ev.read.take();
            let mut lines: Vec<usize> = model
                .evidence
                .iter()
                .filter(|o| {
                    matches!(o.judgment, Judgment::Output(_))
                        && read
                            .iter()
                            .any(|(g, x)| &o.about == g && single(&o.pattern).as_ref() == Some(x))
                })
                .map(|o| o.line)
                .collect();
            lines.push(e.line);
            lines.extend(ev.measured.take());
            lines.sort();
            lines.dedup();
            problems.push(Problem {
                kind: Kind::Measure {
                    measure: e.about.clone(),
                    counted: count.text.clone(),
                },
                clauses: vec![lines.iter().map(|l| Piece::Evidence(*l)).collect()],
                evidence: lines.into_iter().map(quoted).collect(),
                candidates: Ok(Vec::new()),
                conflict: 0.0,
            });
        }
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
            model.subjects_of(c).iter().any(|g| g == subject)
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
            model.subjects_of(d).contains(&c.func)
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

/// The set of a `represented` claim: the inputs where its spec is true.
/// `None` when Mitra can't list them.
pub(crate) fn represented(model: &Model, c: &Claim) -> Option<Tuples> {
    let Target::Represented(spec) = &c.target else {
        return None;
    };
    let g = model.function(spec)?;
    let domain = g.domain();
    if g.body == Some(Expr::Lit(Value::Bool(true))) {
        return Some(domain);
    }
    let mut set = Tuples::empty(domain.arity());
    for x in domain.values()? {
        let names: Vec<(String, Value)> = g
            .params
            .iter()
            .map(|p| p.0.clone())
            .zip(x.iter().cloned())
            .collect();
        if computed(model, g.body.as_ref()?, &names)? == Value::Bool(true) {
            set = set.union(&Tuples::point(&x));
        }
    }
    Some(set)
}

/// The value of an expression that reads no code, with these names
/// bound. `None` when it reads an output, or has no single value.
pub(crate) fn computed(model: &Model, e: &Expr, names: &[(String, Value)]) -> Option<Value> {
    let ev = Eval {
        model,
        observed: BTreeMap::new(),
        depth: Cell::new(0),
        read: RefCell::new(Vec::new()),
        measured: RefCell::new(Vec::new()),
        notes: RefCell::new(Vec::new()),
        rules: Vec::new(),
        through: RefCell::new(Vec::new()),
        claim: RefCell::new(String::new()),
        leaned: RefCell::new(Vec::new()),
        relations: Vec::new(),
        supposed: RefCell::new(Vec::new()),
    };
    let env: Env = names.iter().cloned().collect();
    let got = ev.eval(e, &env, 0, Subjects::Observed).ok()?;
    match &got[..] {
        [v] => Some(v.clone()),
        _ => None,
    }
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
    let set_aside = model.misfits.iter().map(|m| &m.0);
    let entry = |line: usize| {
        model
            .evidence
            .iter()
            .chain(set_aside.clone())
            .find(|e| e.line == line)
    };
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
            twins.map(|o| o.confidence).fold(e.confidence, f64::max)
        }),
        Piece::Claim(c) => Some(model.claim(c).and_then(|c| c.trust).unwrap_or(1.0)),
        // What the model states is trusted like evidence: 1 unless said.
        Piece::Signature(_) => Some(1.0),
        Piece::Code(_) => None,
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
        let warns =
            |v: &&Verdict| matches!(v.logic, Logic::Unverified | Logic::Assuming) && !v.own_word();
        self.verdicts.iter().filter(warns).count() + self.flags.len()
    }
}

impl Verdict {
    /// An assumption on a boundary that rests on nothing but itself: the
    /// model says so on purpose.
    pub fn own_word(&self) -> bool {
        self.logic == Logic::Assuming && self.assuming == [self.claim.clone()]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // One input per test, for Mitra: each `mitra: test` line gives the
    // output the test checks.

    // mitra: evidence for t_holds
    #[test]
    fn t_holds() {
        assert_eq!(Logic::Holds.as_str(), "holds");
    }

    // mitra: evidence for t_assuming
    #[test]
    fn t_assuming() {
        assert_eq!(Logic::Assuming.as_str(), "holds assuming");
    }

    // mitra: evidence for t_unverified
    #[test]
    fn t_unverified() {
        assert_eq!(Logic::Unverified.as_str(), "unverified");
    }

    // mitra: evidence for t_fails
    #[test]
    fn t_fails() {
        assert_eq!(Logic::Fails.as_str(), "fails");
    }

    // mitra: evidence for t_plus
    #[test]
    fn t_plus() {
        assert_eq!(
            binary("+", &Value::Int(2), &Value::Int(3)),
            Some(Value::Int(5))
        );
    }

    // mitra: evidence for t_plus_swapped
    #[test]
    fn t_plus_swapped() {
        assert_eq!(
            binary("+", &Value::Int(3), &Value::Int(2)),
            Some(Value::Int(5))
        );
    }

    // mitra: evidence for t_plus_overflow
    #[test]
    fn t_plus_overflow() {
        assert_eq!(
            binary("+", &Value::Int(9223372036854775807), &Value::Int(1)),
            None
        );
    }

    // mitra: evidence for t_plus_overflow_swapped
    #[test]
    fn t_plus_overflow_swapped() {
        assert_eq!(
            binary("+", &Value::Int(1), &Value::Int(9223372036854775807)),
            None
        );
    }

    // mitra: evidence for t_plus_mixed
    #[test]
    fn t_plus_mixed() {
        assert_eq!(binary("+", &Value::Int(2), &Value::Bool(true)), None);
    }

    // mitra: evidence for t_plus_mixed_swapped
    #[test]
    fn t_plus_mixed_swapped() {
        assert_eq!(binary("+", &Value::Bool(true), &Value::Int(2)), None);
    }

    // mitra: evidence for t_plus_floats
    #[test]
    fn t_plus_floats() {
        assert_eq!(
            binary("+", &Value::Float(0.5), &Value::Float(0.25)),
            Some(Value::Float(0.75))
        );
    }

    // mitra: evidence for t_plus_floats_swapped
    #[test]
    fn t_plus_floats_swapped() {
        assert_eq!(
            binary("+", &Value::Float(0.25), &Value::Float(0.5)),
            Some(Value::Float(0.75))
        );
    }

    // mitra: evidence for t_minus
    #[test]
    fn t_minus() {
        assert_eq!(
            binary("-", &Value::Int(2), &Value::Int(3)),
            Some(Value::Int(-1))
        );
    }

    // mitra: evidence for t_plus_negated
    #[test]
    fn t_plus_negated() {
        assert_eq!(
            binary("+", &Value::Int(2), &Value::Int(-3)),
            Some(Value::Int(-1))
        );
    }

    // mitra: evidence for t_times
    #[test]
    fn t_times() {
        assert_eq!(
            binary("*", &Value::Int(4), &Value::Int(-3)),
            Some(Value::Int(-12))
        );
    }

    // mitra: evidence for t_times_swapped
    #[test]
    fn t_times_swapped() {
        assert_eq!(
            binary("*", &Value::Int(-3), &Value::Int(4)),
            Some(Value::Int(-12))
        );
    }

    // mitra: evidence for t_divide
    #[test]
    fn t_divide() {
        assert_eq!(
            binary("/", &Value::Int(-12), &Value::Int(-3)),
            Some(Value::Int(4))
        );
    }

    // mitra: evidence for t_divide_towards_zero
    #[test]
    fn t_divide_towards_zero() {
        assert_eq!(
            binary("/", &Value::Int(-7), &Value::Int(2)),
            Some(Value::Int(-3))
        );
    }

    // mitra: evidence for t_divide_by_zero
    #[test]
    fn t_divide_by_zero() {
        assert_eq!(binary("/", &Value::Int(1), &Value::Int(0)), None);
    }

    // mitra: evidence for t_divide_floats
    #[test]
    fn t_divide_floats() {
        assert_eq!(
            binary("/", &Value::Float(1.0), &Value::Float(4.0)),
            Some(Value::Float(0.25))
        );
    }

    // mitra: evidence for t_equal
    #[test]
    fn t_equal() {
        assert_eq!(
            binary("==", &Value::Int(2), &Value::Int(2)),
            Some(Value::Bool(true))
        );
    }

    // mitra: evidence for t_different
    #[test]
    fn t_different() {
        assert_eq!(
            binary("!=", &Value::Int(2), &Value::Int(2)),
            Some(Value::Bool(false))
        );
    }

    // mitra: evidence for t_equal_mixed
    #[test]
    fn t_equal_mixed() {
        assert_eq!(
            binary("==", &Value::Int(2), &Value::Str("2".into())),
            Some(Value::Bool(false))
        );
    }

    // mitra: evidence for t_different_mixed
    #[test]
    fn t_different_mixed() {
        assert_eq!(
            binary("!=", &Value::Int(2), &Value::Str("2".into())),
            Some(Value::Bool(true))
        );
    }

    // mitra: evidence for t_less
    #[test]
    fn t_less() {
        assert_eq!(
            binary("<", &Value::Int(2), &Value::Int(3)),
            Some(Value::Bool(true))
        );
    }

    // mitra: evidence for t_and
    #[test]
    fn t_and() {
        assert_eq!(
            binary("&&", &Value::Bool(true), &Value::Bool(false)),
            Some(Value::Bool(false))
        );
    }

    // mitra: evidence for t_or
    #[test]
    fn t_or() {
        assert_eq!(
            binary("||", &Value::Bool(true), &Value::Bool(false)),
            Some(Value::Bool(true))
        );
    }

    // mitra: evidence for t_unknown_operator
    #[test]
    fn t_unknown_operator() {
        assert_eq!(binary("%", &Value::Int(2), &Value::Int(3)), None);
    }

    // mitra: evidence for t_not_true
    #[test]
    fn t_not_true() {
        assert_eq!(unary("!", &Value::Bool(true)), Some(Value::Bool(false)));
    }

    // mitra: evidence for t_not_false
    #[test]
    fn t_not_false() {
        assert_eq!(unary("!", &Value::Bool(false)), Some(Value::Bool(true)));
    }

    // mitra: evidence for t_negate
    #[test]
    fn t_negate() {
        assert_eq!(unary("-", &Value::Int(3)), Some(Value::Int(-3)));
    }

    // mitra: evidence for t_negate_back
    #[test]
    fn t_negate_back() {
        assert_eq!(unary("-", &Value::Int(-3)), Some(Value::Int(3)));
    }

    // mitra: evidence for t_negate_float
    #[test]
    fn t_negate_float() {
        assert_eq!(unary("-", &Value::Float(0.5)), Some(Value::Float(-0.5)));
    }

    // mitra: evidence for t_negate_string
    #[test]
    fn t_negate_string() {
        assert_eq!(unary("-", &Value::Str("a".into())), None);
    }

    fn ints(xs: &[i64]) -> Value {
        Value::set_of(xs.iter().map(|x| Value::Int(*x)).collect())
    }

    fn text(s: &str) -> Value {
        Value::Str(s.into())
    }

    fn map(pairs: &[(&str, i64)]) -> Value {
        let pairs = pairs.iter().map(|(k, v)| (text(k), Value::Int(*v)));
        Value::map_of(pairs.collect()).unwrap()
    }

    // mitra: evidence for t_in_set
    #[test]
    fn t_in_set() {
        assert_eq!(
            binary("in", &Value::Int(2), &ints(&[1, 2])),
            Some(Value::Bool(true))
        );
    }

    // mitra: evidence for t_in_list
    #[test]
    fn t_in_list() {
        assert_eq!(
            binary(
                "in",
                &Value::Int(3),
                &Value::List(vec![Value::Int(1), Value::Int(2)])
            ),
            Some(Value::Bool(false))
        );
    }

    // mitra: evidence for t_in_map
    #[test]
    fn t_in_map() {
        assert_eq!(
            binary("in", &text("a"), &map(&[("a", 1)])),
            Some(Value::Bool(true))
        );
    }

    // mitra: evidence for t_in_number
    #[test]
    fn t_in_number() {
        assert_eq!(binary("in", &Value::Int(1), &Value::Int(2)), None);
    }

    // mitra: evidence for t_union
    #[test]
    fn t_union() {
        assert_eq!(
            binary("|", &ints(&[1, 2]), &ints(&[2, 3])),
            Some(ints(&[1, 2, 3]))
        );
    }

    // mitra: evidence for t_union_swapped
    #[test]
    fn t_union_swapped() {
        assert_eq!(
            binary("|", &ints(&[2, 3]), &ints(&[1, 2])),
            Some(ints(&[1, 2, 3]))
        );
    }

    // mitra: evidence for t_intersection
    #[test]
    fn t_intersection() {
        assert_eq!(
            binary("&", &ints(&[1, 2]), &ints(&[2, 3])),
            Some(ints(&[2]))
        );
    }

    // mitra: evidence for t_apart
    #[test]
    fn t_apart() {
        assert_eq!(binary("&", &ints(&[1]), &ints(&[2, 3])), Some(ints(&[])));
    }

    // mitra: evidence for t_difference
    #[test]
    fn t_difference() {
        assert_eq!(
            binary("\\", &ints(&[1, 2]), &ints(&[2, 3])),
            Some(ints(&[1]))
        );
    }

    // mitra: evidence for t_map_update
    #[test]
    fn t_map_update() {
        assert_eq!(
            binary("|", &map(&[("a", 1), ("b", 2)]), &map(&[("a", 5)])),
            Some(map(&[("a", 5), ("b", 2)]))
        );
    }

    // mitra: evidence for t_map_remove
    #[test]
    fn t_map_remove() {
        assert_eq!(
            binary(
                "\\",
                &map(&[("a", 1), ("b", 2)]),
                &Value::set_of(vec![text("a"), text("z")])
            ),
            Some(map(&[("b", 2)]))
        );
    }

    // mitra: evidence for t_set_with_map
    #[test]
    fn t_set_with_map() {
        assert_eq!(binary("|", &ints(&[1]), &map(&[("a", 1)])), None);
    }

    // mitra: evidence for t_keys
    #[test]
    fn t_keys() {
        assert_eq!(
            unary("keys", &map(&[("a", 1), ("b", 2)])),
            Some(Value::set_of(vec![text("a"), text("b")]))
        );
    }

    // mitra: evidence for t_keys_of_a_set
    #[test]
    fn t_keys_of_a_set() {
        assert_eq!(unary("keys", &ints(&[1])), None);
    }
}
