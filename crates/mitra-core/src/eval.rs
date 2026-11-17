//! From evidence to verdicts. Only evidence at confidence 0 or 1 exists so
//! far: evidence at 1 is fully believed, evidence at 0 is ignored.

use crate::ast::*;
use crate::set::{Set, Value};
use crate::{Error, Input};
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

/// The verdict on one claim under one policy.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub claim: String,
    pub policy: String,
    /// What the fully believed evidence and the specs prove, by strict logic.
    pub logic: Logic,
    /// The claim's inputs where it is known to hold and not contradicted.
    pub covered: Set,
    /// The claim's inputs where it is known to fail and not contradicted.
    pub refuted: Set,
    /// The inputs the claim is about.
    pub domain: Set,
    /// How much the policy believes the claim, from 0 to 1.
    pub belief: f64,
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

struct Eval<'m> {
    model: &'m Model,
    /// Fully believed observed outputs: (subject, input) -> outputs.
    observed: BTreeMap<(String, Value), Vec<Value>>,
}

impl Eval<'_> {
    /// Every value an expression can take. Specs give one value; a subject
    /// gives each output observed for that input, so none when unobserved.
    fn eval(&self, e: &Expr, env: &Env, line: usize) -> Result<Vec<Value>, Error> {
        let err = |m: String| Err(Error::new(line, m));
        Ok(match e {
            Expr::Lit(v) => vec![v.clone()],
            Expr::Var(x) => match env.get(x) {
                Some(v) => vec![v.clone()],
                None => return err(format!("unknown name `{x}`")),
            },
            Expr::Call(f, a) => {
                let mut out = Vec::new();
                for arg in self.eval(a, env, line)? {
                    out.extend(self.call(f, &arg, line)?);
                }
                out
            }
            Expr::Unary(op, a) => self
                .eval(a, env, line)?
                .iter()
                .map(|v| unary(op, v))
                .collect::<Option<_>>()
                .ok_or_else(|| Error::new(line, format!("`{op}` does not apply here")))?,
            Expr::Binary(op, a, b) => {
                let (xs, ys) = (self.eval(a, env, line)?, self.eval(b, env, line)?);
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
                for v in self.eval(c, env, line)? {
                    match v {
                        Value::Bool(true) => out.extend(self.eval(t, env, line)?),
                        Value::Bool(false) => out.extend(self.eval(f, env, line)?),
                        v => return err(format!("`if` needs a bool, got {v}")),
                    }
                }
                out
            }
            Expr::Let(x, v, body) => {
                let mut out = Vec::new();
                for v in self.eval(v, env, line)? {
                    let mut env = env.clone();
                    env.insert(x.clone(), v);
                    out.extend(self.eval(body, &env, line)?);
                }
                out
            }
        })
    }

    fn call(&self, f: &str, arg: &Value, line: usize) -> Result<Vec<Value>, Error> {
        let Some(func) = self.model.function(f) else {
            return Err(Error::new(line, format!("unknown function `{f}`")));
        };
        if !func.domain.contains(arg) {
            return Err(Error::new(
                line,
                format!("{arg} is outside the domain of `{f}`"),
            ));
        }
        let Some(body) = &func.body else {
            return Ok(self
                .observed
                .get(&(f.to_string(), arg.clone()))
                .cloned()
                .unwrap_or_default());
        };
        let env = Env::from([(func.param.clone(), arg.clone())]);
        let out = self.eval(body, &env, func.line)?;
        if let Some(v) = out.iter().find(|v| !func.range.contains(v)) {
            return Err(Error::new(
                func.line,
                format!("`{f}({arg})` = {v} is outside its result type"),
            ));
        }
        Ok(out)
    }

    /// Whether a `matches` claim is right, wrong (or both) on one input.
    fn judge(&self, c: &Claim, target: &Expr, x: &Value) -> Result<(bool, bool), Error> {
        let lhs = self.call(&c.func, x, c.line)?;
        let rhs = match target {
            Expr::Var(f)
                if Some(f) != c.pattern.var.as_ref() && self.model.function(f).is_some() =>
            {
                self.call(f, x, c.line)?
            }
            e => {
                let env: Env = c
                    .pattern
                    .var
                    .iter()
                    .map(|v| (v.clone(), x.clone()))
                    .collect();
                self.eval(e, &env, c.line)?
            }
        };
        let mut seen = (false, false);
        for l in &lhs {
            for r in &rhs {
                if l == r {
                    seen.0 = true;
                } else {
                    seen.1 = true;
                }
            }
        }
        Ok(seen)
    }
}

fn unary(op: &str, v: &Value) -> Option<Value> {
    match (op, v) {
        ("!", Value::Bool(b)) => Some(Value::Bool(!b)),
        ("-", Value::Int(i)) => i.checked_neg().map(Value::Int),
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
        _ => return None,
    })
}

pub fn evaluate(model: &Model) -> Result<Report, (Input, Error)> {
    let believed = |e: &&Evidence| e.confidence == 1.0;
    let mut observed: BTreeMap<(String, Value), Vec<Value>> = BTreeMap::new();
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
            let input = e
                .pattern
                .set
                .as_ref()
                .and_then(|s| s.single())
                .expect("checked");
            observed
                .entry((f.name.clone(), input))
                .or_default()
                .push(out.clone());
        }
    }
    let ev = Eval { model, observed };
    // Every input observed for some subject: the points where a claim on an
    // infinite domain can be judged.
    let observed_inputs = ev
        .observed
        .keys()
        .fold(Set::empty(), |s, (_, x)| s.union(&Set::point(x)));

    let mut logic_of: HashMap<&str, Logic> = HashMap::new();
    let mut verdicts = Vec::new();
    for c in &model.claims {
        let func = model.function(&c.func).expect("checked");
        let domain = model.covers(&c.func, &c.pattern);
        let (mut correct, mut wrong) = (Set::empty(), Set::empty());
        let mut add = |ok: bool, s: &Set| {
            let side = if ok { &mut correct } else { &mut wrong };
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
            let inputs = domain.values().unwrap_or_else(|| {
                observed_inputs
                    .intersect(&domain)
                    .values()
                    .unwrap_or_default()
            });
            for x in inputs {
                let (ok, ko) = ev.judge(c, target, &x).map_err(|e| (Input::Model, e))?;
                let point = Set::point(&x);
                if ok {
                    add(true, &point);
                }
                if ko {
                    add(false, &point);
                }
            }
        }
        // A derivation: when every premise holds, so does this claim. A
        // premise that fails only removes the support.
        let derived =
            !c.from.is_empty() && c.from.iter().all(|p| logic_of[p.as_str()] == Logic::Holds);
        // An input both right and wrong is a contradiction: it counts on
        // neither side. A direct counterexample still beats a derivation.
        let refuted = wrong.minus(&correct);
        let supported = if derived { domain.clone() } else { correct };
        let covered = supported.minus(&wrong);
        let logic = if !refuted.is_empty() {
            Logic::Fails
        } else if covered == domain {
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
                    let measure = |s: &Set| measure(model, &c.func, profile, s);
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
            });
        }
    }
    Ok(Report { verdicts, flags })
}

/// The share of a function's inputs that `s` represents under a profile.
///
/// Each region gets its share of the total weight. Inside a finite region
/// every value weighs the same. Inside an infinite region a set counts only
/// if it covers the whole region: any finite sample of it measures 0.
fn measure(model: &Model, func: &str, profile: &str, s: &Set) -> f64 {
    let domain = &model.function(func).expect("checked").domain;
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
                Logic::Fails => format!("refuted: {}", v.refuted),
                _ => format!("covered: {}", v.covered),
            };
            out += &format!(
                "verdict {} [{}] = {{ logic: {}, {}, domain: {}, belief: {} }}\n",
                v.claim,
                v.policy,
                v.logic.as_str(),
                set,
                v.domain,
                num(v.belief)
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
