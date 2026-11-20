//! The value of an expression of the model: literals, operators, calls of
//! specs, and the outputs observed for the subjects it calls.

mod counting;

pub(crate) use counting::{computable, subject_calls};

use crate::model::*;
use crate::types::set::{Value, MAP, SET};
use crate::types::tuples::Tuples;
use crate::Error;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, HashMap};

pub(crate) type Env = HashMap<String, Value>;

/// How a call of a subject gets its value.
#[derive(Clone, Copy)]
pub(crate) enum Subjects<'a> {
    /// From observed outputs.
    Observed,
    /// From these premises, used as definitions: a premise
    /// `g(pattern) matches e` replaces `g(x)` by `e` on its domain.
    Assumed(&'a [&'a Claim]),
}

/// Calls nested deeper than this are given up (no value).
pub(crate) const MAX_DEPTH: usize = 256;

pub(crate) struct Eval<'m> {
    pub(crate) model: &'m Model,
    /// Observed outputs: (subject, inputs) -> outputs.
    pub(crate) observed: BTreeMap<(String, Vec<Value>), Vec<Value>>,
    pub(crate) depth: Cell<usize>,
    /// The observed outputs read so far: (subject, inputs).
    pub(crate) read: RefCell<Vec<(String, Vec<Value>)>>,
    /// The lines of the measurements read so far.
    pub(crate) measured: RefCell<Vec<usize>>,
    /// What was counted for the measures that have no value yet.
    pub(crate) notes: RefCell<Vec<String>>,
    /// Vouched claims `f(a, b) == e`, read as "the output of `f` is `e`".
    pub(crate) rules: Vec<Rule>,
    /// The calls being worked out through a rule: a rule is not used to
    /// find the output it is asked for.
    pub(crate) through: RefCell<Vec<(String, Vec<Value>)>>,
    /// The claim being judged: its own vouch gives it nothing.
    pub(crate) claim: RefCell<String>,
    /// The assumptions that gave an output since last cleared.
    pub(crate) leaned: RefCell<Vec<String>>,
    /// The claims that read `correct(…)` and are vouched for: each forces
    /// the judgements that would make it false.
    pub(crate) relations: Vec<Relation>,
    /// A judgement taken as given while a forced one is worked out.
    pub(crate) supposed: RefCell<Vec<((String, Vec<Value>), bool)>>,
}

/// A claim someone vouched for that gives the output of a subject from
/// other outputs: `add(a, b) == add(b, a)`.
pub(crate) struct Rule {
    /// The claim vouched for.
    pub(crate) claim: String,
    /// The claim is an assumption on a boundary: nobody checked it.
    pub(crate) assumed: bool,
    pub(crate) func: String,
    /// At each input of the subject: the name bound there, with its set.
    pub(crate) params: Vec<(String, Option<crate::Set>)>,
    /// Or the value written there, by position.
    pub(crate) fixed: Vec<Option<Expr>>,
    pub(crate) gives: Expr,
    /// The evidence that vouches for the claim.
    pub(crate) line: usize,
}

/// The rules of a model: property claims of the shape `f(x, y) == e`,
/// each input a different name the claim binds, vouched for on the whole
/// claim and nowhere denied.
pub(crate) fn rules(model: &Model) -> Vec<Rule> {
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
pub(crate) struct Relation {
    pub(crate) claim: String,
    /// The evidence that vouches for it.
    pub(crate) line: usize,
}

pub(crate) fn relations(model: &Model) -> Vec<Relation> {
    let mut out = Vec::new();
    for c in &model.claims {
        let Target::Property(e) = &c.target else {
            continue;
        };
        let mut reads = false;
        crate::model::walk(e, &mut |e| reads |= matches!(e, Expr::Correct(..)));
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

impl<'m> Eval<'m> {
    /// Nothing observed, no claim vouched for: what the model alone says.
    pub(crate) fn bare(model: &'m Model) -> Eval<'m> {
        Eval {
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
        }
    }
}

impl Eval<'_> {
    /// Every value an expression can take. Specs give one value; a subject
    /// gives each value it has (none when unknown).
    pub(crate) fn eval(
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

    pub(crate) fn call(
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
                    crate::report::print::call(f, args)
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

    pub(crate) fn call_inner(
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
                        crate::report::print::call(&func.name, args)
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
    pub(crate) fn target(
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
    pub(crate) fn compare(
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
}

/// Every combination of one value per position.
pub(crate) fn product(lists: &[Vec<Value>]) -> Vec<Vec<Value>> {
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

/// The value of an expression that reads no code, with these names
/// bound. `None` when it reads an output, or has no single value.
pub(crate) fn computed(model: &Model, e: &Expr, names: &[(String, Value)]) -> Option<Value> {
    let ev = Eval::bare(model);
    let env: Env = names.iter().cloned().collect();
    let got = ev.eval(e, &env, 0, Subjects::Observed).ok()?;
    match &got[..] {
        [v] => Some(v.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod tests;
