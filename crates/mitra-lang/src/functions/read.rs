//! Reading a function: its signature, its inputs, its body, what it
//! uses.

use crate::reader::prelude::*;

impl P {
    /// `trust 0.8`, when written here.
    pub(crate) fn trust(&mut self) -> R<Option<f64>> {
        if !(self.is("trust") && self.kind_at(1) == Some(Kind::Num)) {
            return Ok(None);
        }
        self.pos += 2;
        match self.toks[self.pos - 1].text.parse::<f64>() {
            Ok(t) if (0.0..=1.0).contains(&t) => Ok(Some(t)),
            _ => self.err("trust is a number from 0 to 1"),
        }
    }

    /// A claim of the model being built.
    pub(crate) fn state(
        &mut self,
        name: &str,
        func: &str,
        pattern: Pattern,
        target: Target,
        line: usize,
    ) {
        if self.block.is_some() {
            self.owned.push((name.to_string(), self.blocks));
        }
        self.m.claims.push(Claim {
            name: name.to_string(),
            func: func.to_string(),
            pattern,
            target,
            from: Vec::new(),
            auto: false,
            proved: false,
            trust: None,
            line,
        });
        self.names
            .claims
            .entry(name.to_string())
            .or_insert((Vec::new(), Vec::new()));
    }

    /// A function of the model being built.
    pub(crate) fn define(
        &mut self,
        name: &str,
        sig: &Sig,
        range: Set,
        body: Option<Expr>,
        line: usize,
    ) {
        let function = Function {
            name: name.to_string(),
            params: sig.params.clone(),
            range,
            body,
            written: None,
            line,
        };
        self.m.deep.insert(name.to_string(), sig.deep.clone());
        self.functions.push(function.clone());
        self.m.functions.push(function);
    }

    /// What the model itself brings as evidence on one of its claims.
    pub(crate) fn declare(&mut self, claim: &str, source: &str, trust: f64) {
        self.names.declared.push(Evidence {
            about: claim.to_string(),
            pattern: Vec::new(),
            judgment: Judgment::Holds(true),
            confidence: trust,
            source: Some(source.to_string()),
            rev: None,
            seen: None,
            line: 0,
            act: None,
            by: None,
            written: None,
            cold: false,
            of: None,
        });
    }

    pub(crate) fn function(&mut self, line: usize) -> R<()> {
        let name = self.ident()?;
        let q = self.qualified(&name);
        // fn f<T>(x: T) -> T: each type parameter is any value.
        let head = self.pos;
        let type_names = self.type_params(&name)?;
        let inputs_at = self.pos;
        let mut bounds = std::mem::take(&mut self.bounds);
        for t in &type_names {
            self.generics.push((t.clone(), Set::any()));
        }
        let read = self
            .both(|p| p.binders("(", ")"))
            .and_then(|(params, deep)| {
                let range = match self.eat("->") {
                    true => self.set_both()?,
                    false => (Set::tuple(Vec::new()), Set::tuple(Vec::new())),
                };
                Ok((params, deep, range))
            });
        let generic = !std::mem::take(&mut self.generics).is_empty();
        let (params, mut deep, (range, deep_range)) = read?;
        // An input can be read in a body: not a word an expression starts with.
        let taken = |p: &&(String, Set)| KEYWORDS.contains(&&*p.0) && !DECLARING.contains(&&*p.0);
        if let Some((word, _)) = params.iter().find(taken) {
            let msg = format!("`{word}` is a keyword: as an input, write `r#{word}`");
            return Err(Error::new(line, msg));
        }
        // where L: Link, M: A + B
        if self.eat("where") {
            loop {
                let t = self.ident()?;
                if !type_names.contains(&t) {
                    self.pos -= 1;
                    return self.err(format!("`{name}` has no type parameter `{t}`"));
                }
                self.expect(":")?;
                let bound = self.bounded_by()?;
                bounds.push((t, bound));
                if !self.eat(",") {
                    break;
                }
            }
        }
        let written = generic.then(|| tidy(&join(&self.toks[head..self.pos])));
        // What each input is bound by: its type is a bound parameter,
        // or it was written `impl Contract`.
        let mut bound: Vec<(String, Vec<String>)> = Vec::new();
        for (t, contracts) in &bounds {
            for (i, tok) in self.toks[head..self.pos].iter().enumerate() {
                let (mut j, at) = (head + i, |j: usize| self.toks[j].text.as_str());
                let alone = &tok.text == t
                    && tok.kind == Kind::Name
                    && [",", ")"].contains(&at(j + 1))
                    && j > head + 2;
                while alone && ["&", "mut"].contains(&at(j - 1)) {
                    j -= 1;
                }
                if alone && at(j - 1) == ":" {
                    bound.push((at(j - 2).to_string(), contracts.clone()));
                }
            }
            if let Some(block) = &self.scope {
                for c in contracts {
                    self.leaning.push((block.clone(), c.clone(), line));
                }
            }
        }
        let last = name.rsplit("::").next().unwrap_or(&name);
        let at = self.toks[head - 1].line;
        for (_, _, input, contracts) in
            (self.contracts.sugar.iter()).filter(|s| s.0 == at && s.1 == last)
        {
            bound.push((input.clone(), contracts.clone()));
        }
        self.m.bounds.retain(|b| b.function != q);
        for (input, contracts) in &bound {
            self.m.bounds.push(Bounded {
                function: q.clone(),
                input: input.clone(),
                contracts: contracts.clone(),
            });
        }
        // Inside the function a type parameter is opaque: a record with
        // nothing stated, known by the contracts that bound it. The
        // inputs are read once more with it; a caller still gives any
        // value.
        let opaque = |t: &str| Set::record(&format!("<{q}::{t}>"), vec![Set::any()]);
        let mut inside = params.clone();
        let mut known: Vec<(Set, Vec<String>)> = Vec::new();
        if generic {
            let (end, met, direct) = (self.pos, self.met, self.direct);
            self.pos = inputs_at;
            for t in &type_names {
                let set = opaque(t);
                self.aliases.push((set.to_string(), t.clone()));
                let by = bounds.iter().filter(|b| &b.0 == t);
                known.push((set.clone(), by.flat_map(|b| b.1.clone()).collect()));
                self.generics.push((t.clone(), set));
            }
            let read = self.binders("(", ")");
            self.generics.clear();
            (self.pos, self.met, self.direct) = (end, met, direct);
            inside = read?;
        }
        // `x: impl Link`: the same, with no name.
        for (input, contracts) in &bound {
            let at = params.iter().position(|p| &p.0 == input);
            let Some(at) = at.filter(|at| params[*at].1 == Set::any()) else {
                continue;
            };
            if inside[at].1 != Set::any() {
                continue;
            }
            let written = format!("impl {}", contracts.join(" + "));
            let set = opaque(&format!("{input}: {written}"));
            self.aliases.push((set.to_string(), written));
            known.push((set.clone(), contracts.clone()));
            inside[at].1 = set;
        }
        self.m.generics.retain(|g| g.function != q);
        if inside != params {
            self.m.generics.push(Generic {
                function: q.clone(),
                params: inside.clone(),
                known: known.clone(),
            });
        }
        deep.push(deep_range);
        let sig = Sig { params, deep };
        // `uses a, b` or `uses only a, b`: what it calls.
        let mut uses = None;
        if self.eat("uses") {
            let only = self.eat("only");
            let mut used = vec![self.path()?];
            while self.eat(",") {
                used.push(self.path()?);
            }
            let used: Vec<String> = used.iter().map(|u| self.resolve(u)).collect();
            uses = Some((only, used, self.trust()?.unwrap_or(1.0)));
        }
        // `trust 0.9` on a helper: how far its definition is trusted.
        if self.block.is_none() {
            if let Some(t) = self.trust()? {
                self.m.helper_trust.push((q.clone(), t));
            }
        }
        self.names.fns.insert(q.clone(), sig.clone());
        if self.block.is_some() {
            self.names.opaque.insert(q.clone());
            self.owned.push((format!("{q}::correct"), self.blocks));
        }
        if self.block.as_deref() == Some("boundary") {
            self.names.boundaries.insert(q.clone());
        }
        if self.is(";") && self.block.is_none() {
            let msg = format!(
                "`fn {name}` has no body: write `subject fn {name}(…)` if it is your code, \
                 `boundary fn {name}(…)` if it is the world. A plain `fn` is a helper \
                 of the model and needs a body"
            );
            return Err(Error::new(line, msg));
        }
        if self.eat(";") {
            self.define(&q, &sig, range, None, line);
        } else {
            let in_subject = self.block.is_some();
            let body_name = match in_subject {
                true => format!("{q}::body"),
                false => q.clone(),
            };
            self.locals = sig.params.iter().map(|p| p.0.clone()).collect();
            self.typed = inside;
            self.opaque = known;
            // In a subject, the body is the spec of the code: it calls
            // itself as a spec.
            self.own = Some((name.clone(), body_name.clone()));
            let body = self.block();
            self.own = None;
            self.locals.clear();
            self.opaque.clear();
            let body = body?;
            // No `uses` written: what the body calls of the code and of
            // the world, and nothing else, is implied.
            if in_subject && uses.is_none() {
                let mut used: Vec<String> = Vec::new();
                for g in crate::model::calls(&body) {
                    let code = self.known.opaque.contains(&g) || self.names.opaque.contains(&g);
                    if code && g != q && !used.contains(&g) {
                        used.push(g);
                    }
                }
                if !used.is_empty() {
                    uses = Some((true, used, 1.0));
                }
            }
            if in_subject {
                let spec = format!("{q}::spec");
                self.names.specified.insert(q.clone(), body_name.clone());
                self.define(&q, &sig, range.clone(), None, line);
                self.define(&body_name, &sig, range, Some(body), line);
                let names = sig.params.iter().map(|p| &p.0);
                let pattern = names
                    .clone()
                    .map(|x| Position {
                        var: Some(x.clone()),
                        set: None,
                    })
                    .collect();
                let args = names.map(|x| Expr::Var(x.clone())).collect();
                let target = Target::Matches(Expr::Call(body_name, args));
                self.state(&spec, &q, pattern, target, line);
            } else {
                self.define(&q, &sig, range, Some(body), line);
            }
        }
        if let Some(f) = self.m.functions.iter_mut().rev().find(|f| f.name == q) {
            f.written = written.clone();
        }
        if let Some(f) = self.functions.iter_mut().rev().find(|f| f.name == q) {
            f.written = written;
        }
        if let Some((only, used, trust)) = uses {
            let claim = format!("{q}::uses");
            self.state(&claim, &q, Vec::new(), Target::Uses { used, only }, line);
            self.declare(&claim, "declared", trust);
        }
        Ok(())
    }

    /// The inputs of the function `f`, as known where the parser is.
    pub(crate) fn sig(&self, f: &str, inputs: usize) -> R<Sig> {
        let known = self.known.fns.get(f).or(self.names.fns.get(f)).cloned();
        match known {
            Some(sig) => Ok(sig),
            // The first reading: declared further down.
            None if self.first => Ok(Sig {
                params: vec![(String::new(), Set::any()); inputs],
                deep: vec![Set::any(); inputs],
            }),
            None => self.err(format!("the model has no function `{f}`")),
        }
    }

    /// `(a: T, b)`: the inputs of a claim. The type may be left out: the
    /// one of the function the input is passed to. Plain, then deep.
    pub(crate) fn inputs(&mut self, close: &str, into: &mut Vec<(Bound, Option<Set>)>) -> R<()> {
        while !self.is(close) {
            let line = self.line();
            let x = self.local()?;
            if into.iter().any(|i| i.0 .0 == x) {
                return Err(Error::new(line, format!("`{x}` is an input twice")));
            }
            let (set, deep) = match self.eat(":") {
                true => {
                    // `o: &Mass`, as Rust writes it: a borrowed value is
                    // the same value here.
                    if self.eat("&") {
                        self.eat("mut");
                    }
                    let (set, deep) = self.set_both()?;
                    (Some(set), Some(deep))
                }
                false => (None, None),
            };
            self.locals.push(x.clone());
            self.typed
                .push((x.clone(), set.clone().unwrap_or(Set::any())));
            into.push(((x, set, line), deep));
            if !self.eat(",") {
                break;
            }
        }
        Ok(())
    }
}
