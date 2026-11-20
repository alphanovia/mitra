//! Step 2: classes. A `uniform` claim sorts the inputs of a subject into
//! classes where one input stands for the others; a class with a pass and
//! a failure is a conflict.

use super::evidence::Point;
use super::*;
use crate::eval::{computable, subject_calls, Env, Subjects};
use std::collections::BTreeMap;

/// Evidence over at most this many inputs is also read input by input, to
/// find the class of each (see `uniform`).
pub(crate) const POINTS: f64 = 10_000.0;

/// The class of every input, for a `uniform` claim without a spec.
pub(crate) const ONE_CLASS: Value = Value::Bool(true);

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
pub(crate) enum Sorted {
    /// No `uniform` claim applies.
    No,
    /// The claim calls subjects, and this keeps their `uniform` claims
    /// from being used.
    Unsorted(String),
    By(Sorting),
}

/// One class per call of a subject, the claim's own call first.
pub(crate) struct Sorting {
    /// The `uniform` claims relied on, the one of the claim's subject first.
    pub(crate) claims: Vec<usize>,
    pub(crate) parts: Vec<Part>,
    /// The calls that have classes, as text. Empty for a claim that calls
    /// no subject but its own.
    pub(crate) calls: Vec<String>,
}

pub(crate) struct Part {
    pub(crate) spec: Option<String>,
    /// The arguments of the call; `None`: the claim's inputs.
    pub(crate) args: Option<Vec<Expr>>,
    /// The line of the `uniform` claim.
    pub(crate) line: usize,
}

/// How claim `i` is sorted into classes by the `uniform` claims.
pub(crate) fn sorted(model: &Model, i: usize) -> Sorted {
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

impl Judge<'_> {
    /// The class of an input by a spec; one class when there is no spec.
    fn class_of(&self, spec: &Option<String>, x: &[Value], line: usize) -> Result<Value, Fail> {
        let Some(spec) = spec else {
            return Ok(ONE_CLASS);
        };
        let out = self
            .ev
            .call(spec, x, line, Subjects::Observed)
            .map_err(model_err)?;
        Ok(out.into_iter().next().expect("a spec has a value"))
    }

    /// Claims that say the same thing of a subject, on whatever domain:
    /// they share their judged inputs.
    pub(super) fn same(&self, i: usize, j: usize) -> bool {
        let (c, d) = (&self.model.claims[i], &self.model.claims[j]);
        c.func == d.func && c.target == d.target
    }

    /// The inputs judged for a claim, with those of the claims that say
    /// the same.
    pub(super) fn points_of(&self, i: usize) -> Vec<Point> {
        let model = self.model;
        let mut points = match model.claims[i].target {
            Target::Correct => self.judged(&model.claims[i].func),
            _ => Vec::new(),
        };
        for j in (0..self.direct.len()).filter(|j| self.same(i, *j)) {
            points.extend(self.direct[j].points.iter().cloned());
        }
        // A contradicted input is neither a pass nor a failure, whichever
        // claim the contradiction was found on.
        let tainted = |x: &Vec<Value>| {
            (0..self.direct.len()).any(|j| self.same(i, j) && self.direct[j].tainted.contains(x))
        };
        points.retain(|p| !tainted(&p.0));
        points
    }

    /// A failure somewhere unknown: no class can be relied on.
    pub(super) fn blind(&self, i: usize) -> bool {
        (0..self.direct.len()).any(|j| self.same(i, j) && self.direct[j].blind)
    }

    /// The class of an input of claim `i` under a sorting.
    pub(super) fn sig(&self, i: usize, s: &Sorting, x: &[Value]) -> Result<Value, Fail> {
        let (model, ev) = (self.model, &self.ev);
        if let [only] = &s.parts[..] {
            // The inputs of the one call: the claim's own, or its arguments.
            let Some(args) = &only.args else {
                return self.class_of(&only.spec, x, only.line);
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
                return self.class_of(&only.spec, x, only.line);
            }
            return self.class_of(&only.spec, &inputs, only.line);
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
            names.push(self.class_of(&part.spec, &inputs, part.line)?.to_string());
        }
        Ok(match names.is_empty() {
            true => ONE_CLASS,
            false => Value::Str(names.join(", ")),
        })
    }

    /// Every class a sorting can give.
    pub(super) fn every(&self, s: &Sorting) -> Vec<Value> {
        let model = self.model;
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
    }

    /// Each class of a `uniform` claim that has a pass and a failure.
    pub(super) fn conflicts(&self) -> Result<Vec<Conflict>, Fail> {
        let model = self.model;
        let mut conflicts: Vec<Conflict> = Vec::new();
        for u in &model.claims {
            let Target::Uniform(spec) = &u.target else {
                continue;
            };
            // `correct` evidence without a claim to carry it.
            let stated = |c: &Claim| c.func == u.func && c.target == Target::Correct;
            let mut groups = Vec::new();
            if !model.claims.iter().any(stated) {
                groups.push((None, self.judged(&u.func)));
            }
            for (i, sorted) in self.sorting.iter().enumerate() {
                let first = (0..i).all(|j| !self.same(i, j));
                // A claim counts for the `uniform` claim of its subject; a
                // property, for the one of the first subject it calls.
                if let (Sorted::By(s), true) = (sorted, first) {
                    if model.claims[s.claims[0]].name == u.name {
                        groups.push((Some((i, s)), self.points_of(i)));
                    }
                }
            }
            for (by, group) in groups {
                let mut seen: BTreeMap<Value, [Vec<&Point>; 2]> = BTreeMap::new();
                for p in &group {
                    let class = match by {
                        Some((i, s)) => self.sig(i, s, &p.0)?,
                        None => self.class_of(spec, &p.0, u.line)?,
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
                        pass: (pass.0.clone(), pass.2, self.shown(pass.2)),
                        fail: (fail.0.clone(), fail.2, self.shown(fail.2)),
                        passes: lines(&passes),
                        fails: lines(&fails),
                    });
                }
            }
        }
        Ok(conflicts)
    }

    pub(super) fn broken(&self, name: &str) -> bool {
        self.conflicts.iter().any(|k| k.claim == name)
    }

    /// A structural claim can be relied on while nothing speaks against
    /// it, and is vouched for once evidence covers it.
    pub(super) fn usable(&self, i: usize) -> bool {
        !self.broken(&self.model.claims[i].name) && self.direct[i].wrong.is_empty()
    }

    pub(super) fn vouched(&self, i: usize) -> bool {
        self.usable(i) && self.direct[i].right.same(&self.direct[i].domain)
    }
}
