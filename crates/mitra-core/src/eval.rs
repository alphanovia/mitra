//! From evidence to verdicts. Only evidence at confidence 0 or 1 exists so
//! far: evidence at 1 is fully believed, evidence at 0 is ignored.

use crate::ast::*;
use crate::set::{Set, Value};
use crate::tuples::Tuples;
use crate::{Error, Input};
use std::cell::Cell;
use std::collections::{BTreeMap, HashMap};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Logic {
    Holds,
    Fails,
    Uncertain,
}

impl Logic {
    pub fn as_str(self) -> &'static str {
        match self {
            Logic::Holds => "holds",
            Logic::Fails => "fails",
            Logic::Uncertain => "uncertain",
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

/// The verdict on one claim under one policy.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub claim: String,
    pub policy: String,
    /// What the fully believed evidence and the specs prove, by strict logic.
    pub logic: Logic,
    /// The claim's inputs where it is known to hold and not contradicted.
    pub covered: Tuples,
    /// The claim's inputs where it is known to fail and not contradicted.
    pub refuted: Tuples,
    /// The inputs the claim is about.
    pub domain: Tuples,
    /// How much the policy believes the claim, from 0 to 1.
    pub belief: f64,
    /// For a claim with `from`.
    pub implication: Option<Implication>,
}

/// Evidence Mitra cannot use as given.
#[derive(Debug, Clone, PartialEq)]
pub struct Flag {
    pub line: usize,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub verdicts: Vec<Verdict>,
    pub flags: Vec<Flag>,
}

type Env = HashMap<String, Value>;

/// How a call of a subject gets its value.
#[derive(Clone, Copy)]
enum Subjects<'a> {
    /// From fully believed observed outputs.
    Observed,
    /// From these premises, used as definitions: a premise
    /// `g(pattern) matches e` replaces `g(x)` by `e` on its domain.
    Assumed(&'a [&'a Claim]),
}

/// Calls nested deeper than this are given up (no value).
const MAX_DEPTH: usize = 256;

struct Eval<'m> {
    model: &'m Model,
    /// Fully believed observed outputs: (subject, inputs) -> outputs.
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
            Expr::Unary(op, a) => {
                let mut out = Vec::new();
                for v in self.eval(a, env, line, subjects)? {
                    match unary(op, &v) {
                        Some(r) => out.push(r),
                        None => return err(format!("`{op}{v}` is not defined")),
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
                format!("{} is outside the domain of `{f}`", show_tuple(args)),
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
                        "`{}{}` = {v} is outside its result type",
                        func.name,
                        show_tuple(args)
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

fn show_tuple(t: &[Value]) -> String {
    format!(
        "({})",
        t.iter()
            .map(|v| v.to_string())
            .collect::<Vec<_>>()
            .join(", ")
    )
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

pub fn evaluate(model: &Model) -> Result<Report, (Input, Error)> {
    let believed = |e: &&Evidence| e.confidence == 1.0;
    let mut observed: BTreeMap<(String, Vec<Value>), Vec<Value>> = BTreeMap::new();
    let mut flags = Vec::new();
    for e in model.evidence.iter().filter(believed) {
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

    let mut logic_of: HashMap<&str, Logic> = HashMap::new();
    let mut verdicts = Vec::new();
    for c in &model.claims {
        let func = model.function(&c.func).expect("checked");
        let domain = model.covers(&c.func, &c.pattern);
        let arity = domain.arity();
        let (mut right, mut wrong) = (Tuples::empty(arity), Tuples::empty(arity));
        let mut add = |ok: bool, s: &Tuples| {
            let side = if ok { &mut right } else { &mut wrong };
            *side = side.union(&s.intersect(&domain));
        };
        for e in model.evidence.iter().filter(believed) {
            let set = |e: &Evidence| model.covers(&c.func, &e.pattern);
            match &e.judgment {
                Judgment::Holds(ok) if e.about == c.name => add(*ok, &set(e)),
                Judgment::Correct(ok)
                    if e.about == c.func && c.target == Target::Correct && !func.is_spec() =>
                {
                    add(*ok, &set(e))
                }
                _ => {}
            }
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
            for x in inputs {
                if let Some((ok, ko)) = ev
                    .compare(c, target, &x, Subjects::Observed)
                    .map_err(model_err)?
                {
                    let point = Tuples::point(&x);
                    if ok {
                        add(true, &point);
                    }
                    if ko {
                        add(false, &point);
                    }
                }
            }
        }
        // A derivation: when the implication is not refuted and every premise
        // holds, so does this claim. A failing premise only removes support.
        let implication = if c.from.is_empty() {
            None
        } else {
            let premises: Vec<&Claim> = c
                .from
                .iter()
                .map(|p| model.claim(p).expect("checked"))
                .collect();
            Some(ev.implication(c, &domain, &premises).map_err(model_err)?)
        };
        let derived = implication.is_some_and(|i| i != Implication::Refuted)
            && c.from.iter().all(|p| logic_of[p.as_str()] == Logic::Holds);
        // An input both right and wrong is a contradiction: it counts on
        // neither side. A direct counterexample still beats a derivation.
        let refuted = wrong.minus(&right);
        let supported = if derived { domain.clone() } else { right };
        let covered = supported.minus(&wrong);
        let logic = if !refuted.is_empty() {
            Logic::Fails
        } else if covered.same(&domain) {
            Logic::Holds
        } else {
            Logic::Uncertain
        };
        logic_of.insert(&c.name, logic);
        for p in &model.policies {
            let belief = match (&p.kind, logic) {
                (_, Logic::Fails) => 0.0,
                (PolicyKind::Strict, l) => (l == Logic::Holds) as u8 as f64,
                (PolicyKind::Coverage(profile), l) => {
                    let measure = |s: &Tuples| measure(model, &c.func, profile, s);
                    let whole = measure(&domain);
                    if whole > 0.0 {
                        measure(&covered) / whole
                    } else {
                        (l == Logic::Holds) as u8 as f64
                    }
                }
            };
            verdicts.push(Verdict {
                claim: c.name.clone(),
                policy: p.name.clone(),
                logic,
                covered: covered.clone(),
                refuted: refuted.clone(),
                domain: domain.clone(),
                belief,
                implication,
            });
        }
    }
    Ok(Report { verdicts, flags })
}

/// The share of a function's inputs that `s` represents under a profile.
///
/// One parameter: each region gets its share of the total weight. Inside a
/// finite region every value weighs the same. Inside an infinite region a
/// set counts only if it covers the whole region: any finite sample of it
/// measures 0. Several parameters: `uniform` over the tuples, with the same
/// rule for finite and infinite.
fn measure(model: &Model, func: &str, profile: &str, s: &Tuples) -> f64 {
    let f = model.function(func).expect("checked");
    if f.params.len() != 1 {
        let domain = f.domain();
        return match domain.count() {
            Some(n) if n > 0.0 => s.intersect(&domain).count().unwrap() / n,
            _ => domain.is_subset(s) as u8 as f64,
        };
    }
    let (domain, s) = (&f.params[0].1, s.as_set());
    let regions: Vec<(Option<Set>, f64)> = model
        .profiles
        .iter()
        .find(|p| p.name == profile && p.subject == func)
        .map(|p| p.regions.clone())
        .unwrap_or_else(|| vec![(None, 1.0)]); // `uniform`, or a profile for another function
    let total: f64 = regions.iter().map(|r| r.1).sum();
    let mut seen = Set::empty();
    let mut m = 0.0;
    for (region, w) in regions {
        let region = region.unwrap_or_else(|| domain.clone()).minus(&seen);
        seen = seen.union(&region);
        let inside = s.intersect(&region);
        let share = match region.count() {
            _ if region.is_empty() => 0.0,
            Some(n) => inside.count().unwrap() / n,
            None => (inside == region) as u8 as f64,
        };
        m += w / total * share;
    }
    m
}

impl Report {
    /// The result file: one line per claim and policy, then one line per
    /// flagged piece of evidence.
    pub fn to_text(&self) -> String {
        let mut out = String::new();
        for v in &self.verdicts {
            let set = match v.logic {
                Logic::Fails => format!("refuted := {}", v.refuted.show(&v.domain)),
                _ => format!("covered := {}", v.covered.show(&v.domain)),
            };
            let implication = v
                .implication
                .map(|i| format!(", implication := .{}", i.as_str()))
                .unwrap_or_default();
            out += &format!(
                "{} [{}] := {{ logic := .{}, {}, domain := {}, belief := {}{} }}\n",
                v.claim,
                v.policy,
                v.logic.as_str(),
                set,
                v.domain.show(&v.domain),
                num(v.belief),
                implication
            );
        }
        for f in &self.flags {
            out += &format!("contested evidence line {}: {}\n", f.line, f.message);
        }
        out
    }
}

fn num(x: f64) -> String {
    let s = format!("{:.4}", x);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
