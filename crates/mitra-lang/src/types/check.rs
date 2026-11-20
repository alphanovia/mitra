//! The checks of a model as a whole: names, references, and patterns that
//! stay inside domains.

use crate::model::*;
use crate::types::set::{Set, Value};
use crate::{Error, Input};

/// Every call in an expression, with its number of arguments.
pub(super) fn arities(e: &Expr) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    walk(e, &mut |e| {
        if let Expr::Call(f, args) = e {
            out.push((f.clone(), args.len()));
        }
    });
    out
}

/// The parameter types of the functions that `x` is passed to as it is.
pub(super) fn passed(e: &Expr, x: &str, model: &Model, out: &mut Vec<(String, Set)>) {
    walk(e, &mut |e| {
        let Expr::Call(f, args) = e else { return };
        let Some(f) = model.declared_function(f) else {
            return;
        };
        for (a, (_, ty)) in args.iter().zip(&f.params) {
            if matches!(a, Expr::Var(v) if v == x) {
                out.push((f.name.clone(), ty.clone()));
            }
        }
    });
}

/// A name in scope: the parameter it is a part of, and whether it is a
/// strict piece of it.
pub(super) type Part = Option<(usize, bool)>;

/// For each call of `name` inside `e`: whether each argument is a strict
/// piece of the parameter at the same place.
pub(super) fn descents(
    e: &Expr,
    name: &str,
    scope: &mut Vec<(String, Part)>,
    out: &mut Vec<Vec<bool>>,
) {
    fn part(e: &Expr, scope: &[(String, Part)]) -> Part {
        match e {
            Expr::Var(x) => scope.iter().rev().find(|s| &s.0 == x)?.1,
            Expr::Field(a, _) | Expr::Index(a, _) => part(a, scope).map(|(i, _)| (i, true)),
            _ => None,
        }
    }
    let piece = |e: &Expr, scope: &[(String, Part)]| part(e, scope).map(|(i, _)| (i, true));
    match e {
        Expr::Lit(_) | Expr::Var(_) | Expr::Score(..) => {}
        Expr::Call(f, args) => {
            if f == name {
                let strict = |(i, a)| part(a, scope) == Some((i, true));
                out.push(args.iter().enumerate().map(strict).collect());
            }
            args.iter().for_each(|a| descents(a, name, scope, out));
        }
        Expr::Let(x, a, b) => {
            descents(a, name, scope, out);
            scope.push((x.clone(), part(a, scope)));
            descents(b, name, scope, out);
            scope.pop();
        }
        Expr::Match(a, arms) => {
            descents(a, name, scope, out);
            let inside = piece(a, scope);
            for arm in arms {
                let n = scope.len();
                let names = arm.variant.iter().flat_map(|v| v.1.clone());
                scope.extend(names.map(|x| (x, inside)));
                descents(&arm.body, name, scope, out);
                scope.truncate(n);
            }
        }
        Expr::Each(_, x, list, body) => {
            descents(list, name, scope, out);
            scope.push((x.clone(), piece(list, scope)));
            descents(body, name, scope, out);
            scope.pop();
        }
        Expr::Rate(x, _, a) => {
            scope.push((x.clone(), None));
            descents(a, name, scope, out);
            scope.pop();
        }
        Expr::Unary(_, a) | Expr::Field(a, _) | Expr::Len(a) => descents(a, name, scope, out),
        Expr::Tuple(args) | Expr::Correct(_, args) | Expr::Record(_, args) | Expr::List(args) => {
            args.iter().for_each(|a| descents(a, name, scope, out))
        }
        Expr::Binary(_, a, b) | Expr::Index(a, b) => {
            descents(a, name, scope, out);
            descents(b, name, scope, out);
        }
        Expr::If(a, b, c) => {
            descents(a, name, scope, out);
            descents(b, name, scope, out);
            descents(c, name, scope, out);
        }
    }
}

impl Model {
    /// Check names, references and that patterns stay inside domains.
    // mitra: implements types::Model::check
    pub fn check(mut self) -> Result<Model, (Input, Error)> {
        let m = |line, msg: String| (Input::Model, Error::new(line, msg));
        let mut names: Vec<(&String, usize)> =
            self.records.iter().map(|r| (&r.name, r.line)).collect();
        // A record with type parameters is an enum of one variant, of
        // the same name.
        let generic_record = |e: &&Enum| e.variants.len() == 1 && e.variants[0] == e.name;
        let enums = self.enums.iter().filter(|e| !generic_record(e));
        names.extend(enums.map(|e| (&e.name, e.line)));
        for r in &self.records {
            for (i, (field, _)) in r.fields.iter().enumerate() {
                if r.fields[..i].iter().any(|f| &f.0 == field) {
                    let msg = format!("`{}` has two fields `{field}`", r.name);
                    return Err(m(r.line, msg));
                }
            }
        }
        names.extend(self.functions.iter().map(|f| (&f.name, f.line)));
        names.extend(self.claims.iter().map(|c| (&c.name, c.line)));
        for (i, (n, line)) in names.iter().enumerate() {
            if names[..i].iter().any(|(o, _)| o == n) {
                return Err(m(*line, format!("`{n}` is defined twice")));
            }
        }
        for (i, f) in self.functions.iter().enumerate() {
            if let Some(body) = &f.body {
                // A spec may call the specs defined before it, and itself
                // on a piece of its input: it always ends.
                let earlier: Vec<&str> = self.functions[..=i]
                    .iter()
                    .filter(|g| g.is_spec())
                    .map(|g| g.name.as_str())
                    .collect();
                let mut own = Vec::new();
                let whole = |(i, p): (usize, &(String, Set))| (p.0.clone(), Some((i, false)));
                let mut scope = f.params.iter().enumerate().map(whole).collect();
                descents(body, &f.name, &mut scope, &mut own);
                let ends = (0..f.params.len()).any(|i| own.iter().all(|c| c.get(i) == Some(&true)));
                if !own.is_empty() && !ends {
                    return Err(m(
                        f.line,
                        format!(
                            "`{0}` calls itself, and must do so on a piece of one of its \
                             inputs, always the same one: a name bound by `match`, a field, \
                             or an element (`all(x in xs, …)`)",
                            f.name
                        ),
                    ));
                }
                // A name the spec binds may hold a function.
                let mut bound: Vec<String> = f.params.iter().map(|p| p.0.clone()).collect();
                walk(body, &mut |e| match e {
                    Expr::Let(x, ..) | Expr::Each(_, x, ..) | Expr::Rate(x, ..) => {
                        bound.push(x.clone())
                    }
                    Expr::Match(_, arms) => {
                        let names = arms.iter().filter_map(|a| a.variant.as_ref());
                        bound.extend(names.flat_map(|v| v.1.clone()));
                    }
                    _ => {}
                });
                // A body may call the code of a subject: it is then
                // computed where that code was observed, and reasoned
                // about from the claims on it.
                let code = |c: &String| {
                    let opaque = self.functions.iter().any(|g| &g.name == c && !g.is_spec());
                    opaque && !self.boundaries.contains(c)
                };
                if let Some(bad) = calls(body)
                    .into_iter()
                    .find(|c| !earlier.contains(&c.as_str()) && !bound.contains(c) && !code(c))
                {
                    return Err(m(
                        f.line,
                        format!("`{bad}` is not a spec defined above `{}`", f.name),
                    ));
                }
            }
        }
        // An arm of a `match` names a variant, with a name for each field.
        let bodies = self
            .functions
            .iter()
            .filter_map(|f| Some((f.body.as_ref()?, f.line)));
        let claimed = self.claims.iter().filter_map(|c| match &c.target {
            Target::Matches(e) | Target::Property(e) => Some((e, c.line)),
            _ => None,
        });
        for (e, line) in bodies.chain(claimed) {
            for (name, fields) in arms(e) {
                let Some(r) = self
                    .records
                    .iter()
                    .find(|r| r.name == name && r.of.is_some())
                else {
                    return Err(m(line, format!("`{name}` is not a variant of an enum")));
                };
                if r.fields.len() != fields.len() {
                    let msg = format!(
                        "`{name}` has {} field(s), the arm names {}",
                        r.fields.len(),
                        fields.len()
                    );
                    return Err(m(line, msg));
                }
            }
        }
        let (deep, recursive) = (self.deep.clone(), self.recursive.clone());
        let conforms = |f: &str, i: usize, v: &Value| {
            let deep = deep.get(f).and_then(|d| d.get(i));
            deep.is_none_or(|s| s.conforms(v, &recursive))
        };
        let outside = |func: &Function, p: &Pattern| {
            p.iter()
                .zip(&func.params)
                .enumerate()
                .any(|(i, (pos, (_, ty)))| {
                    pos.set.as_ref().is_some_and(|s| {
                        !s.is_subset(ty) || s.single().is_some_and(|v| !conforms(&func.name, i, &v))
                    })
                })
        };
        let fits = |func: &Function, p: &Pattern, line: usize, input: Input| {
            let err = |msg: String| Err((input, Error::new(line, msg)));
            if p.len() != func.params.len() {
                let n = func.params.len();
                return err(format!(
                    "`{}` takes {n} input(s), got {}",
                    func.name,
                    p.len()
                ));
            }
            let outside = outside(func, p);
            if outside {
                return err(format!("inputs outside the domain of `{}`", func.name));
            }
            Ok(())
        };
        // A property is about the names it binds: its hidden function takes
        // them as inputs, each with the set given or the type of the
        // parameters it is passed to.
        let mut properties = Vec::new();
        for c in &self.claims {
            let Target::Property(e) = &c.target else {
                continue;
            };
            for (g, n) in arities(e) {
                // A bound name may hold a function.
                if c.pattern.iter().any(|p| p.var.as_ref() == Some(&g)) {
                    continue;
                }
                match self.declared_function(&g) {
                    None => return Err(m(c.line, format!("unknown function `{g}`"))),
                    Some(f) if f.params.len() != n => {
                        let k = f.params.len();
                        return Err(m(c.line, format!("`{g}` takes {k} input(s), got {n}")));
                    }
                    _ => {}
                }
            }
            let mut scored = Vec::new();
            walk(e, &mut |e| {
                if let Expr::Score(kind, f) = e {
                    scored.push((*kind, f.clone()));
                }
            });
            for (kind, f) in scored {
                self.scored(kind, &f).map_err(|msg| m(c.line, msg))?;
            }
            let mut params = Vec::new();
            for p in &c.pattern {
                let x = p.var.as_ref().expect("a property binds names");
                let mut types = Vec::new();
                passed(e, x, &self, &mut types);
                let set = match (&p.set, types.first()) {
                    (Some(s), _) => {
                        if let Some((g, _)) = types.iter().find(|t| !s.is_subset(&t.1)) {
                            let msg = format!("`{x}` goes outside the inputs of `{g}`");
                            return Err(m(c.line, msg));
                        }
                        s.clone()
                    }
                    (None, Some(_)) => types.iter().fold(Set::any(), |a, t| a.intersect(&t.1)),
                    (None, None) => {
                        let msg = format!(
                            "`{x}` is not passed to a function as it is: give its set, `{x}: S`"
                        );
                        return Err(m(c.line, msg));
                    }
                };
                params.push((x.clone(), set));
            }
            properties.push(Function {
                name: c.name.clone(),
                params,
                range: Set::bool(),
                body: None,
                written: None,
                line: c.line,
            });
        }
        self.properties = properties;
        // `uniform f by g` is about every input of `f`.
        for i in 0..self.claims.len() {
            let c = &self.claims[i];
            if !c.target.is_structural() {
                continue;
            }
            let Some(f) = self.function(&c.func) else {
                return Err(m(c.line, format!("unknown function `{}`", c.func)));
            };
            let whole = Position {
                var: None,
                set: None,
            };
            let arity = f.params.len();
            if f.is_spec() {
                let msg = format!(
                    "`{}` is a spec: `uniform` and `uses` are for the functions of the code",
                    c.func
                );
                return Err(m(c.line, msg));
            }
            let g = match &c.target {
                Target::Uniform(g) => g,
                Target::Uses { used, .. } => {
                    for (n, g) in used.iter().enumerate() {
                        if used[..n].contains(g) {
                            return Err(m(c.line, format!("`{g}` is listed twice")));
                        }
                        if !self.function(g).is_some_and(|g| !g.is_spec()) {
                            return Err(m(
                                c.line,
                                format!("`{g}` is not a function of the code: `uses` lists those"),
                            ));
                        }
                        if g == &c.func {
                            return Err(m(c.line, format!("`{g}` can't use itself")));
                        }
                    }
                    self.claims[i].pattern = vec![whole; arity];
                    continue;
                }
                _ => {
                    self.claims[i].pattern = vec![whole; arity];
                    continue;
                }
            };
            self.spec_of_classes(f, g.as_deref())
                .map_err(|msg| m(c.line, msg))?;
            let twice = |d: &Claim| d.func == c.func && matches!(d.target, Target::Uniform(_));
            if self.claims[..i].iter().any(twice) {
                return Err(m(
                    c.line,
                    format!("`{}` already has a `uniform` claim", c.func),
                ));
            }
            self.claims[i].pattern = vec![whole; arity];
        }
        for (i, c) in self.claims.iter().enumerate() {
            let Some(f) = self.function(&c.func) else {
                return Err(m(c.line, format!("unknown function `{}`", c.func)));
            };
            fits(f, &c.pattern, c.line, Input::Model)?;
            if let Target::Matches(e) = &c.target {
                let bound = |v: &String| c.pattern.iter().any(|p| p.var.as_ref() == Some(v));
                let fun_ref = matches!(e, Expr::Var(v) if self.function(v).is_some() && !bound(v));
                for name in calls(e).into_iter().chain(fun_ref.then(|| match e {
                    Expr::Var(v) => v.clone(),
                    _ => unreachable!(),
                })) {
                    // A bound name may hold a function.
                    if self.declared_function(&name).is_none() && !bound(&name) {
                        return Err(m(c.line, format!("unknown function `{name}`")));
                    }
                }
            } else if c.target == Target::Correct && f.is_spec() {
                return Err(m(
                    c.line,
                    format!(
                        "`{}` is a spec: `correct` is for the functions of the code; write `{}(…) == …`",
                        c.func, c.func
                    ),
                ));
            }
            for p in &c.from {
                let structural = |d: &&Claim| &d.name == p && d.target.is_structural();
                if self.claims.iter().any(|d| structural(&d)) {
                    return Err(m(
                        c.line,
                        format!("`{p}` is a `uniform` or `uses` claim: it can't be a premise"),
                    ));
                }
                if !self.claims[..i].iter().any(|d| &d.name == p) {
                    return Err(m(
                        c.line,
                        format!("`{p}` is not a claim defined above `{}`", c.name),
                    ));
                }
            }
        }
        // Evidence on a claim without `given`: on its whole domain.
        let ranges: Vec<String> = self
            .functions
            .iter()
            .map(|f| self.named(&f.range.to_string()))
            .collect();
        let arities: Vec<(String, usize)> = self
            .claims
            .iter()
            .filter_map(|c| Some((c.name.clone(), self.function(&c.func)?.params.len())))
            .collect();
        for e in self.evidence.iter_mut() {
            let arity = arities.iter().find(|a| a.0 == e.about).map(|a| a.1);
            if let (Judgment::Holds(_), true, Some(n)) = (&e.judgment, e.pattern.is_empty(), arity)
            {
                let whole = Position {
                    var: None,
                    set: None,
                };
                e.pattern = vec![whole; n];
            }
        }
        // Evidence that does not fit a signature: set aside, with why.
        let mut misfits: Vec<(usize, String)> = Vec::new();
        let evidence = self.evidence.iter().map(|e| (e, Input::Evidence));
        for (e, input) in evidence {
            let ev = |line, msg: String| (input, Error::new(line, msg));
            match (
                &e.judgment,
                self.declared_function(&e.about),
                self.claim(&e.about),
            ) {
                (Judgment::Holds(_), _, Some(c)) => {
                    fits(self.function(&c.func).unwrap(), &e.pattern, e.line, input)?
                }
                // A test that states no input: nothing to compare.
                (Judgment::Passed | Judgment::Sampled { .. }, _, _) => {}
                (Judgment::Measured(measure, value), _, _) => {
                    let mut bad = None;
                    if !(0.0..=1.0).contains(value) {
                        bad = Some(format!("a measure goes from 0 to 1, got {value}"));
                    }
                    walk(measure, &mut |e| match e {
                        Expr::Call(f, _) if self.declared_function(f).is_none() => {
                            bad = Some(format!("unknown function `{f}`"));
                        }
                        Expr::Score(kind, f) => bad = bad.take().or(self.scored(kind, f).err()),
                        _ => {}
                    });
                    if let Some(msg) = bad {
                        return Err(ev(e.line, msg));
                    }
                }
                (Judgment::Holds(_), _, None) => {
                    return Err(ev(
                        e.line,
                        format!(
                            "`{}` is not a claim (or an output is missing: `= value`)",
                            e.about
                        ),
                    ))
                }
                (_, Some(f), _) => {
                    let kept = input == Input::Evidence && e.pattern.len() == f.params.len();
                    if kept && outside(f, &e.pattern) {
                        let msg = format!("its inputs are outside the inputs of `{}`", f.name);
                        misfits.push((e.line, msg));
                        continue;
                    }
                    fits(f, &e.pattern, e.line, input)?;
                    if matches!(e.judgment, Judgment::Output(_)) && single(&e.pattern).is_none() {
                        return Err(ev(
                            e.line,
                            "an observed output needs a single input value".into(),
                        ));
                    }
                    if let Judgment::Output(v) = &e.judgment {
                        let fit = f.range.contains(v) && self.conforms(&f.name, f.params.len(), v);
                        if !fit && kept && single(&e.pattern).is_some() {
                            let i = self.functions.iter().position(|g| g.name == f.name);
                            let msg = format!(
                                "its output {v} is outside the result type of `{}`: {}",
                                f.name,
                                ranges[i.unwrap()]
                            );
                            misfits.push((e.line, msg));
                            continue;
                        }
                        if !fit {
                            return Err(ev(
                                e.line,
                                format!(
                                    "output {v} is outside the result type of `{}`: {}",
                                    f.name,
                                    ranges[self
                                        .functions
                                        .iter()
                                        .position(|g| g.name == f.name)
                                        .unwrap()]
                                ),
                            ));
                        }
                    }
                }
                (_, None, _) if self.claim(&e.about).is_some() => {
                    let c = &e.about;
                    let inputs = match self.claim(c).is_some_and(|c| c.target.is_structural()) {
                        true => String::new(),
                        false => format!(", or `for {c} given … holds` for some inputs"),
                    };
                    return Err(ev(
                        e.line,
                        format!(
                            "`{c}` is a claim, not a function: an output is given for a function\n  \
                             help: to vouch for the claim, write `evidence r {{ for {c} holds }}`{inputs}"
                        ),
                    ));
                }
                (_, None, _) => {
                    return Err(ev(
                        e.line,
                        format!("`{}` is not a function of the model", e.about),
                    ))
                }
            }
        }
        for (line, why) in misfits {
            let at = self.evidence.iter().position(|e| e.line == line);
            let e = self.evidence.remove(at.expect("its line"));
            self.misfits.push((e, why));
        }
        crate::types::check(&self).map_err(|e| (Input::Model, e))?;
        Ok(self)
    }
}
