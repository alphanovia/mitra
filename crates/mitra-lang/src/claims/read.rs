//! Reading a claim: its inputs and statement, the shapes `equivalence`
//! and `represented`, an instance of a claim with parameters.

use crate::reader::prelude::*;

impl P {
    pub(crate) fn claim(&mut self, line: usize, assumed: bool) -> R<()> {
        let name = self.ident()?;
        let named_at = self.toks[self.pos - 1].line;
        let q = self.qualified(&name);
        // claim c = deterministic(f);  claim c = !deterministic(f);
        let not = self.is("=") && self.peek_at(1) == "!";
        if self.is("=") && self.peek_at(1 + usize::from(not)) == "deterministic" {
            self.pos += 2 + usize::from(not);
            self.expect("(")?;
            let f = self.path()?;
            let f = self.resolve(&f);
            self.expect(")")?;
            if let Some(t) = self.trust()? {
                self.names.trust.insert(q.clone(), t);
            }
            self.eat(";");
            if !self.first && !self.known.opaque.contains(&f) {
                let msg = format!("`{f}` is no function of a subject or a boundary");
                return Err(Error::new(line, msg));
            }
            if self.block.is_some() {
                self.owned.push((q.clone(), self.blocks));
            }
            self.names.determinism.push((f, q, !not, line));
            return Ok(());
        }
        if self.eat("=") {
            // claim c = uniform(f), uniform(f, |a, b| class), uniform(f, classes)
            let form = self.path()?;
            // The two shapes Mitra works out on whole sets: see `equivalence`.
            let shape = std::mem::take(&mut self.shape);
            if form == "represented" && shape {
                return self.represented(line, &q);
            }
            if form != "uniform" || !shape {
                return self.instance(line, assumed, &name, &form);
            }
            self.expect("(")?;
            let f = self.path()?;
            let f = self.resolve(&f);
            let mut by = None;
            if self.eat(",") {
                if self.eat("|") {
                    let names = self.bars()?;
                    let mut sig = self.sig(&f, names.len())?;
                    if names.len() != sig.params.len() {
                        let msg = format!(
                            "`{f}` takes {} input(s), got {}",
                            sig.params.len(),
                            names.len()
                        );
                        return self.err(msg);
                    }
                    self.locals = names.clone();
                    // `|a| -> Type { class }`; without a type, a Bool.
                    // The class is computed: a function in it is its spec.
                    self.own = Some((String::new(), String::new()));
                    let read = match self.eat("->") {
                        true => self.set_both().and_then(|c| Ok((c, self.block()?))),
                        false => self.expr().map(|e| ((Set::bool(), Set::bool()), e)),
                    };
                    self.own = None;
                    self.locals.clear();
                    let ((class, deep), body) = read?;
                    for (p, x) in sig.params.iter_mut().zip(names) {
                        p.0 = x;
                    }
                    sig.deep.truncate(sig.params.len());
                    sig.deep.push(deep);
                    let case = format!("{q}::case");
                    self.define(&case, &sig, class, Some(body), line);
                    by = Some(case);
                } else {
                    let g = self.path()?;
                    by = Some(self.resolve(&g));
                }
            }
            self.expect(")")?;
            self.eat(";");
            self.state(&q, &f, Vec::new(), Target::Uniform(by), line);
            return Ok(());
        }
        // claim c(f, T) { given a: T then f(a) == a }: a claim with
        // parameters, used as `claim d = c(g, Int);`.
        if self.is("(") {
            let mut i = 1;
            let mut params = Vec::new();
            while self.kind_at(i) == Some(Kind::Name) && [",", ")"].contains(&self.peek_at(i + 1)) {
                params.push(self.peek_at(i).to_string());
                i += 2;
            }
            let closed = i > 1 && self.peek_at(i - 1) == ")";
            if closed && self.peek_at(i) == "{" && self.peek_at(i + 1) == "given" {
                self.pos += i;
                let start = self.pos;
                let mut depth = 0;
                loop {
                    if self.pos >= self.toks.len() {
                        return self.err("expected `}`");
                    }
                    depth += self.is("{") as i32 - self.is("}") as i32;
                    self.pos += 1;
                    if depth == 0 {
                        break;
                    }
                }
                let body = self.toks[start..self.pos].to_vec();
                self.eat(";");
                self.templates
                    .insert(format!("{q}/{}", params.len()), (params, body));
                return Ok(());
            }
        }
        // claim c(a: T) when cond { body }, or
        // claim c { given a: T  when cond  then body }
        let mut inputs = Vec::new();
        self.locals.clear();
        self.typed.clear();
        self.opaque.clear();
        if self.eat("(") {
            self.inputs(")", &mut inputs)?;
            self.expect(")")?;
        }
        // `x: impl Link`: opaque in the claim, known by its contract.
        let last = name.rsplit("::").next().unwrap_or(&name).to_string();
        let sugar = self.contracts.clone();
        let mut inside = Vec::new();
        for (_, _, input, contracts) in
            (sugar.sugar.iter()).filter(|s| s.0 == named_at && s.1 == last)
        {
            let written = format!("impl {}", contracts.join(" + "));
            let set = Set::record(&format!("<{q}::{input}: {written}>"), vec![Set::any()]);
            self.aliases.push((set.to_string(), written));
            for t in self.typed.iter_mut().filter(|t| &t.0 == input) {
                t.1 = set.clone();
            }
            self.opaque.push((set.clone(), contracts.clone()));
            inside.push((input.clone(), set));
        }
        self.m.generics.retain(|g| g.function != q);
        if !inside.is_empty() {
            self.m.generics.push(Generic {
                function: q.clone(),
                params: inside,
                known: self.opaque.clone(),
            });
        }
        let mut when = None;
        self.called.clear();
        let trust = self.trust()?;
        if let Some(t) = trust.filter(|_| !assumed) {
            self.names.trust.insert(q.clone(), t);
        }
        self.measures.clear();
        self.impure = false;
        if self.eat("when") {
            when = Some(self.expr()?);
        }
        self.expect("{")?;
        if when.is_none() && self.equivalence(line, &name)? {
            return Ok(());
        }
        let contract = self.is("given") || self.is("when") || self.is("then");
        if self.eat("given") {
            self.inputs("", &mut inputs)?;
        }
        if contract && self.eat("when") {
            when = Some(self.expr()?);
        }
        // A condition on the inputs alone takes inputs out of the claim.
        let typed = !inputs.is_empty() && inputs.iter().all(|i| i.1.is_some());
        if let Some(w) = when.as_ref().filter(|_| !self.impure && typed) {
            let each = inputs
                .iter()
                .map(|((x, set, _), _)| (x.clone(), set.clone()));
            let mut sig = Sig {
                params: each.map(|(x, set)| (x, set.expect("typed"))).collect(),
                deep: inputs.iter().filter_map(|i| i.1.clone()).collect(),
            };
            sig.deep.push(Set::bool());
            self.define(
                &format!("{q}::when"),
                &sig,
                Set::bool(),
                Some(w.clone()),
                line,
            );
        }
        if contract {
            self.expect("then")?;
        }
        let bound: Vec<Bound> = inputs.into_iter().map(|i| i.0).collect();
        // `correct(f(a))` alone is the claim on the code; inside a larger
        // expression it is a value.
        let alone = self.is("correct") && self.peek_at(1) == "(" && {
            let (mut i, mut depth) = (1, 0);
            loop {
                depth += (self.peek_at(i) == "(") as i32 - (self.peek_at(i) == ")") as i32;
                i += 1;
                if depth == 0 || self.pos + i >= self.toks.len() {
                    break;
                }
            }
            self.peek_at(i) == "}"
        };
        if alone {
            // correct(f(a, b)): judged by people or by tests, not computed.
            self.pos += 2;
            let f = self.path()?;
            let f = self.resolve(&f);
            let pattern = self.args(&bound)?;
            if !self.parens {
                return self.err(format!("`correct({f})` needs the inputs of `{f}`"));
            }
            self.expect(")")?;
            if when.is_some() {
                return self.err(
                    "`correct(…)` under `when` is not supported yet: narrow the type of an input",
                );
            }
            self.state(&q, &f, pattern, Target::Correct, line);
        } else {
            let body = self.sequence()?;
            let body = match when {
                Some(w) => bin("||", Expr::Unary("!", Box::new(w)), body),
                None => body,
            };
            let pattern = bound
                .iter()
                .map(|(x, set, _)| Position {
                    var: Some(x.clone()),
                    set: set.clone(),
                })
                .collect();
            self.properties.push((self.m.claims.len(), bound.clone()));
            self.state(&q, &q, pattern, Target::Property(body), line);
        }
        self.expect("}")?;
        self.locals.clear();
        if assumed {
            self.declare(&q, "assumed", trust.unwrap_or(1.0));
        }
        let inputs = bound.into_iter().map(|b| b.0).collect();
        let mut called = std::mem::take(&mut self.called);
        called.sort();
        called.dedup();
        self.names.derived.insert(q.clone(), called);
        let measures = std::mem::take(&mut self.measures);
        self.names.claims.insert(q, (inputs, measures));
        Ok(())
    }

    /// A claim that says the same judgement holds on every input of a set,
    /// or of each class:
    ///
    /// `given x: S, y: S then correct(f(x)) <=> correct(f(y))`, or
    /// `given x, y [when k(x) == k(y)] then correct(f(x)) <=> correct(f(y))`.
    ///
    /// Read as that set or those classes, whatever the claim is called:
    /// Mitra then works on sets, not input by input. Just after `{`.
    pub(crate) fn equivalence(&mut self, line: usize, name: &str) -> R<bool> {
        let (start, mut end, mut depth) = (self.pos, self.pos, 1);
        while depth > 0 {
            let Some(t) = self.toks.get(end) else {
                return Ok(false);
            };
            depth += (t.text == "{") as i32 - (t.text == "}") as i32;
            end += 1;
        }
        let body: Vec<Tok> = self.toks[start..end - 1].to_vec();
        let text = |t: &[Tok]| {
            t.iter()
                .map(|t| t.text.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        };
        // The first of these words outside any bracket.
        let top = |toks: &[Tok], words: &[&str]| {
            let mut depth = 0;
            toks.iter().position(|t| {
                let at = depth == 0 && words.contains(&t.text.as_str()) && t.kind != Kind::Str;
                depth += ["(", "[", "{"].contains(&t.text.as_str()) as i32;
                depth -= [")", "]", "}"].contains(&t.text.as_str()) as i32;
                at
            })
        };
        if body.first().is_none_or(|t| t.text != "given") {
            return Ok(false);
        }
        let Some(then) = top(&body, &["then"]) else {
            return Ok(false);
        };
        // then correct(f(x)) <=> correct(f(y))
        let said = &body[then + 1..];
        let Some(op) = top(said, &["<=>", "=="]) else {
            return Ok(false);
        };
        // `correct ( f ( x ) )`: the function, then the name.
        let side = |toks: &[Tok]| -> Option<(Vec<Tok>, String)> {
            let n = toks.len();
            let fits = n >= 7
                && text(&toks[..2]) == "correct ("
                && toks[n - 4].text == "("
                && toks[n - 3].kind == Kind::Name
                && text(&toks[n - 2..]) == ") )";
            fits.then(|| (toks[2..n - 4].to_vec(), toks[n - 3].text.clone()))
        };
        let (Some((f, x)), Some((g, y))) = (side(&said[..op]), side(&said[op + 1..])) else {
            return Ok(false);
        };
        if text(&f) != text(&g) || x == y {
            return Ok(false);
        }
        // given x[: S], y[: S]  [when k(x) == k(y)]
        let when = top(&body[..then], &["when"]);
        let given = &body[1..when.unwrap_or(then)];
        let Some(comma) = top(given, &[","]) else {
            return Ok(false);
        };
        let input = |toks: &[Tok]| -> Option<(String, Vec<Tok>)> {
            let (first, rest) = toks.split_first()?;
            match rest.split_first() {
                None => Some((first.text.clone(), Vec::new())),
                Some((colon, set)) if colon.text == ":" && !set.is_empty() => {
                    Some((first.text.clone(), set.to_vec()))
                }
                _ => None,
            }
        };
        let (Some((a, set)), Some((b, other))) =
            (input(&given[..comma]), input(&given[comma + 1..]))
        else {
            return Ok(false);
        };
        if (&a, &b) != (&x, &y) || text(&set) != text(&other) {
            return Ok(false);
        }
        let mut key = Vec::new();
        if let Some(w) = when {
            let cond = &body[w + 1..then];
            let Some(eq) = top(cond, &["=="]) else {
                return Ok(false);
            };
            let on = |toks: &[Tok], x: &str| -> Option<Vec<Tok>> {
                let n = toks.len().checked_sub(3)?;
                (text(&toks[n..]) == format!("( {x} )")).then(|| toks[..n].to_vec())
            };
            let (Some(k), Some(l)) = (on(&cond[..eq], &x), on(&cond[eq + 1..], &y)) else {
                return Ok(false);
            };
            if text(&k) != text(&l) || k.is_empty() || !set.is_empty() {
                return Ok(false);
            }
            // An inline function comes in parentheses.
            key = match k.len() > 2 && k[0].text == "(" && k[1].text == "|" {
                true => k[1..k.len() - 1].to_vec(),
                false => k,
            };
        }
        let tok = |text: &str, kind: Kind| Tok {
            kind,
            text: text.to_string(),
            line,
        };
        let form = if set.is_empty() {
            "uniform"
        } else {
            "represented"
        };
        let mut toks = vec![
            tok(name, Kind::Name),
            tok("=", Kind::Punct),
            tok(form, Kind::Name),
            tok("(", Kind::Punct),
        ];
        toks.extend(f);
        for more in [set, key].into_iter().filter(|m| !m.is_empty()) {
            toks.push(tok(",", Kind::Punct));
            toks.extend(more);
        }
        toks.push(tok(")", Kind::Punct));
        let rest = std::mem::replace(&mut self.toks, toks);
        self.pos = 0;
        self.shape = true;
        let done = self.claim(line, false);
        self.shape = false;
        (self.toks, self.pos) = (rest, end);
        done.map(|_| true)
    }

    /// `claim c = represented(f, { (a, b) | b != 0 });` or, with a set per
    /// input, `represented(f, (0..=9, 1..=9))`: one tested input of the
    /// set stands for all of it.
    pub(crate) fn represented(&mut self, line: usize, q: &str) -> R<()> {
        self.expect("(")?;
        let f = self.path()?;
        let f = self.resolve(&f);
        self.expect(",")?;
        let known = self.sig(&f, 0)?;
        let arity = |n: usize, p: &Self| match n == known.params.len() || p.first {
            true => Ok(()),
            false => p.err(format!(
                "`{f}` takes {} input(s), got {n}",
                known.params.len()
            )),
        };
        let comprehension = self.is("{")
            && (self.peek_at(2) == "|"
                || (self.peek_at(1) == "(" && self.kind_at(2) == Some(Kind::Name)));
        let any = || std::iter::repeat(Set::any());
        let (sig, body) = if comprehension {
            // { (a, b) | cond } over the inputs of f.
            self.pos += 1;
            let mut names = Vec::new();
            if self.eat("(") {
                while !self.eat(")") {
                    names.push(self.ident()?);
                    if !self.eat(",") {
                        self.expect(")")?;
                        break;
                    }
                }
            } else {
                names.push(self.ident()?);
            }
            self.expect("|")?;
            arity(names.len(), self)?;
            self.locals = names.clone();
            self.own = Some((String::new(), String::new()));
            let cond = self.expr();
            self.own = None;
            self.locals.clear();
            self.expect("}")?;
            let n = names.len();
            let types = known.params.iter().map(|p| p.1.clone()).chain(any());
            let deep = known.deep.iter().take(known.params.len()).cloned();
            let sig = Sig {
                params: names.into_iter().zip(types).collect(),
                deep: deep.chain(any()).take(n).collect(),
            };
            (sig, cond?)
        } else {
            // A set per input.
            let mut sets = Vec::new();
            let open = self.eat("(");
            while !(open && self.eat(")")) {
                sets.push(self.set_both()?);
                if !open {
                    break;
                }
                if !self.eat(",") {
                    self.expect(")")?;
                    break;
                }
            }
            arity(sets.len(), self)?;
            let names = known
                .params
                .iter()
                .map(|p| p.0.clone())
                .chain((0..).map(|i| format!("x{i}")));
            let sig = Sig {
                params: names.zip(sets.iter().map(|s| s.0.clone())).collect(),
                deep: sets.into_iter().map(|s| s.1).collect(),
            };
            (sig, Expr::Lit(Value::Bool(true)))
        };
        self.expect(")")?;
        if let Some(t) = self.trust()? {
            self.names.trust.insert(q.to_string(), t);
        }
        self.eat(";");
        let set = format!("{q}::set");
        let mut sig = sig;
        sig.deep.push(Set::bool());
        self.define(&set, &sig, Set::bool(), Some(body), line);
        self.names.represented.push((q.to_string(), f, set, line));
        self.names
            .claims
            .insert(q.to_string(), (Vec::new(), Vec::new()));
        Ok(())
    }

    /// `claim c = shape(f, 0..=9, |a| a > 0) trust 0.9;`: the claim `shape`
    /// with its parameters replaced by what is written here.
    pub(crate) fn instance(&mut self, line: usize, assumed: bool, name: &str, form: &str) -> R<()> {
        self.expect("(")?;
        let mut args: Vec<Vec<Tok>> = vec![Vec::new()];
        let (mut depth, mut bars) = (0, false);
        loop {
            if self.pos >= self.toks.len() {
                return self.err("expected `)`");
            }
            let t = self.toks[self.pos].clone();
            self.pos += 1;
            let last = args.last_mut().expect("one at least");
            match t.text.as_str() {
                // The inputs of an inline function: `|a, b|`.
                "|" if depth == 0 && (last.is_empty() || bars) => bars = !bars,
                "(" | "[" | "{" => depth += 1,
                ")" if depth == 0 => break,
                ")" | "]" | "}" => depth -= 1,
                "," if depth == 0 && !bars => {
                    args.push(Vec::new());
                    continue;
                }
                _ => {}
            }
            last.push(t);
        }
        if args.len() == 1 && args[0].is_empty() {
            args.clear();
        }
        let wanted = [self.qualified(form), form.to_string()];
        let found = wanted
            .iter()
            .find_map(|n| self.templates.get(&format!("{n}/{}", args.len())).cloned());
        let Some((params, body)) = found else {
            let named = |k: &&String| wanted.iter().any(|n| k.starts_with(&format!("{n}/")));
            let takes: Vec<&str> = self
                .templates
                .keys()
                .filter(named)
                .filter_map(|k| k.rsplit('/').next())
                .collect();
            let msg = match takes.is_empty() {
                true => format!(
                    "no claim with parameters `{form}` above: write \
                     `claim {form}(f, …) {{ given … then … }}`, or import it"
                ),
                false => format!(
                    "`{form}` takes {} parameter(s), got {}",
                    takes.join(" or "),
                    args.len()
                ),
            };
            return Err(Error::new(line, msg));
        };
        let tok = |text: &str, kind: Kind| Tok {
            kind,
            text: text.to_string(),
            line,
        };
        let mut toks = vec![tok(name, Kind::Name)];
        // `trust 0.9` after the parameters.
        while !self.is(";") && self.pos < self.toks.len() && self.toks[self.pos].line == line {
            toks.push(self.toks[self.pos].clone());
            self.pos += 1;
        }
        self.eat(";");
        for t in body {
            let at = params
                .iter()
                .position(|p| t.kind == Kind::Name && p == &t.text);
            match at {
                Some(i) if args[i].first().is_some_and(|a| a.text == "|") => {
                    toks.push(tok("(", Kind::Punct));
                    toks.extend(args[i].iter().cloned());
                    toks.push(tok(")", Kind::Punct));
                }
                Some(i) => toks.extend(args[i].iter().cloned()),
                None => toks.push(t),
            }
        }
        for t in &mut toks {
            t.line = line;
        }
        let rest = std::mem::replace(&mut self.toks, toks);
        let pos = std::mem::replace(&mut self.pos, 0);
        let done = self.claim(line, assumed);
        let all = self.pos == self.toks.len();
        let extra = self.err::<()>(format!("unexpected `{}` in `{form}`", self.peek()));
        (self.toks, self.pos) = (rest, pos);
        done?;
        match all {
            true => Ok(()),
            false => extra,
        }
    }
}
