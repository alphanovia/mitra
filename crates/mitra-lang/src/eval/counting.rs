//! What an expression says over many inputs: measures and counts, the
//! inputs a property is tried on, forced judgements, implications.

use super::*;
use crate::verdicts::Implication;

/// What the observations say of a measure.
pub(crate) struct Count {
    /// The value, when every input is known.
    pub(crate) exact: Option<f64>,
    /// The least and the most it can be, given what is known.
    pub(crate) bounds: (f64, f64),
    /// What was counted, in words.
    pub(crate) text: String,
}

/// An expression with the values of `env` written in place of its names.
pub(crate) fn close(e: &Expr, env: &Env, hidden: &mut Vec<String>) -> Expr {
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

/// An argument Mitra can compute from the claim's inputs alone.
pub(crate) fn computable(e: &Expr, bound: &[&String], model: &Model) -> bool {
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
pub(crate) fn subject_calls<'e>(
    e: &'e Expr,
    model: &Model,
    out: &mut Vec<(&'e String, &'e [Expr])>,
) {
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

impl Eval<'_> {
    /// The value of a measure: as measured when a measurement says so,
    /// otherwise counted from the observations when they cover every
    /// input. No value when neither does.
    pub(crate) fn measure(&self, e: &Expr, env: &Env, line: usize) -> Result<Vec<Value>, Error> {
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
    pub(crate) fn count(&self, measure: &Expr, line: usize) -> Result<Count, Error> {
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
    pub(crate) fn candidates(&self, c: &Claim, e: &Expr, domain: &Tuples) -> Vec<Vec<Value>> {
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
    pub(crate) fn judged(&self, f: &str, x: &[Value], line: usize) -> Result<Vec<Value>, Error> {
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
    pub(crate) fn forced(
        &self,
        f: &str,
        x: &[Value],
        _line: usize,
    ) -> Result<Option<(bool, usize)>, Error> {
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
    pub(crate) fn counter(
        &self,
        c: &Claim,
        domain: &Tuples,
        premises: &[&Claim],
    ) -> Option<Vec<Value>> {
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
    pub(crate) fn implication(
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
