//! Reading an expression.

use crate::reader::prelude::*;

impl P {
    /// The arguments of a call in a claim or evidence: bound names and
    /// values, as a pattern. Every bound name is used exactly once.
    pub(crate) fn args(&mut self, bound: &[(String, Option<Set>, usize)]) -> R<Pattern> {
        let mut pattern = Vec::new();
        // `(a, 1, -2)`; no parentheses when there are no arguments.
        self.parens = self.eat("(");
        let mut more = self.parens && !self.eat(")");
        while more {
            // A name followed by `(` is a record; a name that no `forall`
            // binds is a variant without fields, when one is declared.
            let variant = |x: &String| {
                // A variant without fields, or a function, as a value.
                !bound.iter().any(|b| &b.0 == x)
                    && (self.functions.iter().any(|f| &f.name == x)
                        || self
                            .records
                            .iter()
                            .any(|r| &r.name == x && r.fields.is_empty()))
            };
            let record =
                self.is_name() && (self.peek_at(1) == "(" || variant(&self.peek().to_string()));
            let value = matches!(self.kind(), Some(Kind::Num | Kind::Str))
                || record
                || self.is("-")
                || self.is("(")
                || self.is("[")
                || self.is("{")
                || self.is_keyword("true")
                || self.is_keyword("false");
            let pos = if value {
                Position {
                    var: None,
                    set: Some(Set::point(&self.value()?)),
                }
            } else {
                let line = self.line();
                let x = self.ident()?;
                let at = |m: String| Err(Error::new(line, m));
                let Some((_, set, _)) = bound.iter().find(|b| b.0 == x) else {
                    return at(format!("`{x}` is not an input of the claim"));
                };
                if pattern
                    .iter()
                    .any(|p: &Position| p.var.as_ref() == Some(&x))
                {
                    return at(format!("`{x}` is used twice: one name per input"));
                }
                Position {
                    var: Some(x),
                    set: set.clone(),
                }
            };
            pattern.push(pos);
            more = self.eat(",");
            if !more {
                self.expect(")")?;
            }
        }
        if let Some((x, _, line)) = bound
            .iter()
            .find(|b| !pattern.iter().any(|p| p.var.as_ref() == Some(&b.0)))
        {
            return Err(Error::new(*line, format!("the input `{x}` is not used")));
        }
        Ok(pattern)
    }

    pub(crate) fn expr(&mut self) -> R<Expr> {
        let mut a = self.or()?;
        while self.eat("<=>") {
            a = bin("==", a, self.or()?);
        }
        Ok(a)
    }

    pub(crate) fn or(&mut self) -> R<Expr> {
        let mut a = self.and()?;
        while self.eat("||") {
            a = bin("||", a, self.and()?);
        }
        Ok(a)
    }

    pub(crate) fn and(&mut self) -> R<Expr> {
        let mut a = self.compare()?;
        while self.eat("&&") {
            a = bin("&&", a, self.compare()?);
        }
        Ok(a)
    }

    pub(crate) fn compare(&mut self) -> R<Expr> {
        let mut a = self.sum()?;
        loop {
            if self.eat("is") {
                // `x is Err`: of that variant.
                let v = self.name()?;
                let Some(n) = self.variants.get(&v) else {
                    return self.err(format!("`is` needs a variant of an enum, got `{v}`"));
                };
                let fields: Vec<String> = (0..*n).map(|i| format!("is_{i}")).collect();
                let arm = |variant, b| Arm {
                    variant,
                    body: Expr::Lit(Value::Bool(b)),
                };
                let arms = vec![arm(Some((v, fields)), true), arm(None, false)];
                a = Expr::Match(Box::new(a), arms);
                continue;
            }
            const OPS: [&str; 7] = ["==", "!=", "<=", ">=", "<", ">", "in"];
            let Some(op) = OPS.iter().find(|op| self.is(op)) else {
                return Ok(a);
            };
            self.pos += 1;
            a = bin(op, a, self.sum()?);
        }
    }

    pub(crate) fn sum(&mut self) -> R<Expr> {
        let mut a = self.product()?;
        loop {
            // `|` and `\` on sets and maps bind like `+`, `&` like `*`.
            const OPS: [&str; 4] = ["+", "-", "|", "\\"];
            let Some(op) = OPS.iter().find(|op| self.is(op)) else {
                return Ok(a);
            };
            self.pos += 1;
            a = bin(op, a, self.product()?);
        }
    }

    pub(crate) fn product(&mut self) -> R<Expr> {
        let mut a = self.unary()?;
        loop {
            const OPS: [&str; 3] = ["*", "/", "&"];
            let Some(op) = OPS.iter().find(|op| self.is(op)) else {
                return Ok(a);
            };
            self.pos += 1;
            a = bin(op, a, self.unary()?);
        }
    }

    pub(crate) fn unary(&mut self) -> R<Expr> {
        if self.eat("!") {
            return Ok(Expr::Unary("!", Box::new(self.unary()?)));
        }
        // `-` right before digits is a negative literal, down to i64::MIN.
        let negative = self.eat("-");
        let digits = self.kind() == Some(Kind::Num)
            && (self.peek().contains('.') || !self.peek().trim_start_matches('0').is_empty());
        if negative && !digits {
            return Ok(Expr::Unary("-", Box::new(self.unary()?)));
        }
        let mut a = match negative {
            true => Expr::Lit(self.numeral(true)?),
            false => self.primary()?,
        };
        // `.field` on a record, `.0` on a tuple, `[i]` on a list.
        loop {
            if self.eat("[") {
                let i = self.expr()?;
                self.expect("]")?;
                a = Expr::Index(Box::new(a), Box::new(i));
            } else if self.eat(".") {
                let position = self.kind() == Some(Kind::Num) && !self.peek().contains('.');
                if !position && self.kind() != Some(Kind::Name) {
                    return self.err(
                        "expected a field name, or a position such as `.0`; \
                         for a position inside a position write `(t.0).1`",
                    );
                }
                let field = self.peek().to_string();
                self.pos += 1;
                // `c.full_name(x)` is `Claim::full_name(c, x)`.
                if !position && self.eat("(") {
                    let f = self.method(&a, &field)?;
                    let mut args = vec![a];
                    args.extend(self.exprs(")")?);
                    a = Expr::Call(f, args);
                    continue;
                }
                a = Expr::Field(Box::new(a), field);
            } else {
                return Ok(a);
            }
        }
    }

    /// The method `name` called on `on`: the one whose `self` the type
    /// of `on` can be. Where that type is not known, the only method of
    /// that name.
    pub(crate) fn method(&mut self, on: &Expr, name: &str) -> R<String> {
        let tail = format!("::{name}");
        let named: Vec<String> = (self.methods.iter())
            .filter(|m| m.ends_with(&tail))
            .cloned()
            .collect();
        let back = |p: &mut P, msg: String| {
            p.pos -= 2;
            p.err(msg)
        };
        if named.is_empty() {
            let msg =
                format!("no method `{name}`: a method is a `fn {name}(self, …)` in an `impl`");
            return back(self, msg);
        }
        // The first reading knows no function yet: any will do.
        let kind = crate::types::of(&self.seen, on, &mut self.typed.clone()).ok();
        let kind = kind.filter(|k| !Set::any().is_subset(k) && !self.first);
        let takes = |m: &String| {
            let own = self.seen.functions.iter().find(|f| &f.name == m);
            own.and_then(|f| f.params.first()).map(|p| p.1.clone())
        };
        let full = match (&kind, &named[..]) {
            (Some(k), _) => {
                let fits = |m: &&String| takes(m).is_some_and(|t| !t.intersect(k).is_empty());
                let mut fit: Vec<&String> = named.iter().filter(fits).collect();
                // An opaque type has the methods of its contracts only.
                if let Some((_, by)) = self.opaque.iter().find(|o| &o.0 == k) {
                    let of = |m: &&String| {
                        m.split("::")
                            .next()
                            .is_some_and(|c| by.iter().any(|b| b == c))
                    };
                    fit.retain(|m| self.contracts.methods.contains(*m) && of(m));
                }
                // Its own method before that of a contract, which any
                // value has.
                if fit.iter().any(|m| !self.contracts.methods.contains(*m)) {
                    fit.retain(|m| !self.contracts.methods.contains(*m));
                }
                match fit[..] {
                    [one] => one.clone(),
                    [] => {
                        let text = crate::print::expr(on);
                        let shown = k.to_string();
                        let kind = match self.aliases.iter().find(|a| a.0 == shown) {
                            Some(a) => a.1.clone(),
                            None => self.seen.named(&shown),
                        };
                        let msg = format!("`{text}` is `{kind}`, which has no method `{name}`");
                        return back(self, msg);
                    }
                    _ => {
                        let each: Vec<String> = fit.iter().map(|n| format!("`{n}`")).collect();
                        let msg = format!(
                            "`{name}` is a method of several types it can be: write one of {} as a function",
                            each.join(", ")
                        );
                        return back(self, msg);
                    }
                }
            }
            (None, [one]) => one.clone(),
            (None, _) if self.first => named[0].clone(),
            // Any value: the method of the contract, where only one has it.
            (None, _)
                if named
                    .iter()
                    .filter(|m| self.contracts.methods.contains(*m))
                    .count()
                    == 1 =>
            {
                let of = named.iter().find(|m| self.contracts.methods.contains(*m));
                of.expect("counted").clone()
            }
            (None, _) => {
                let each: Vec<String> = named.iter().map(|n| format!("`{n}`")).collect();
                let msg = format!(
                    "`{name}` is a method of several records: write one of {} as a function",
                    each.join(", ")
                );
                return back(self, msg);
            }
        };
        let full = self.resolve(&full);
        self.impure |= self.known.opaque.contains(&full);
        if self.own.is_none() && self.known.specified.contains_key(&full) {
            self.called.push(format!("{full}::spec"));
        }
        Ok(full)
    }

    pub(crate) fn exprs(&mut self, close: &str) -> R<Vec<Expr>> {
        let mut out = Vec::new();
        while !self.eat(close) {
            out.push(self.expr()?);
            if !self.eat(",") {
                self.expect(close)?;
                break;
            }
        }
        Ok(out)
    }

    /// `{ let x = e; … result }`
    pub(crate) fn block(&mut self) -> R<Expr> {
        self.expect("{")?;
        let out = self.sequence()?;
        self.expect("}")?;
        Ok(out)
    }

    /// `let x = e; … result`, without the braces.
    pub(crate) fn sequence(&mut self) -> R<Expr> {
        if !self.eat("let") {
            return self.expr();
        }
        let x = self.local()?;
        self.expect("=")?;
        let v = self.expr()?;
        self.expect(";")?;
        self.locals.push(x.clone());
        // What it is bound to says what its methods are.
        let kind = crate::types::of(&self.seen, &v, &mut self.typed.clone());
        self.typed.push((x.clone(), kind.unwrap_or(Set::any())));
        let body = self.sequence();
        self.typed.pop();
        self.locals.pop();
        Ok(Expr::Let(x, Box::new(v), Box::new(body?)))
    }

    /// `|a, b|`, after the first `|`.
    pub(crate) fn bars(&mut self) -> R<Vec<String>> {
        let mut names = Vec::new();
        while !self.eat("|") {
            names.push(self.local()?);
            if !self.eat(",") {
                self.expect("|")?;
                break;
            }
        }
        Ok(names)
    }

    pub(crate) fn primary(&mut self) -> R<Expr> {
        match self.kind() {
            Some(Kind::Num) => return Ok(Expr::Lit(self.numeral(false)?)),
            Some(Kind::Str) => {
                self.pos += 1;
                return Ok(Expr::Lit(Value::Str(unquote(
                    &self.toks[self.pos - 1].text,
                ))));
            }
            None => return self.err("expected an expression"),
            _ => {}
        }
        if self.is("(") && self.peek_at(1) == "|" {
            // (|a, b| a + b)(1, 2): an inline function, on its inputs.
            self.pos += 2;
            let names = self.bars()?;
            let n = self.locals.len();
            self.locals.extend(names.iter().cloned());
            let typed = self.typed.len();
            self.typed
                .extend(names.iter().map(|x| (x.clone(), Set::any())));
            let body = self.expr();
            self.locals.truncate(n);
            self.typed.truncate(typed);
            let mut body = body?;
            self.expect(")")?;
            if !self.eat("(") {
                return self.err("an inline function is given its inputs here: `(|a| a + 1)(2)`");
            }
            let args = self.exprs(")")?;
            if args.len() != names.len() {
                let msg = format!(
                    "the function takes {} input(s), got {}",
                    names.len(),
                    args.len()
                );
                return self.err(msg);
            }
            // The inputs are worked out before any takes its name.
            let pairs: Vec<(String, Expr)> = names
                .iter()
                .cloned()
                .zip(args)
                .filter(|(x, a)| a != &Expr::Var(x.clone()))
                .collect();
            let reads = |a: &Expr| free(a, &mut Vec::new(), &mut Vec::new());
            let mixed = pairs
                .iter()
                .any(|(_, a)| reads(a).iter().any(|x| names.contains(x)));
            let mut held = Vec::new();
            for (x, a) in pairs.into_iter().rev() {
                if mixed {
                    self.fresh += 1;
                    let h = format!("held_{}", self.fresh);
                    body = Expr::Let(x, Box::new(Expr::Var(h.clone())), Box::new(body));
                    held.push((h, a));
                } else {
                    body = Expr::Let(x, Box::new(a), Box::new(body));
                }
            }
            for (h, a) in held {
                body = Expr::Let(h, Box::new(a), Box::new(body));
            }
            return Ok(body);
        }
        if self.eat("(") {
            // `(e)` groups; `(a, b)` is a tuple.
            let mut parts = self.exprs(")")?;
            return Ok(match parts.len() {
                1 => parts.remove(0),
                _ => Expr::Tuple(parts),
            });
        }
        if self.eat("[") {
            return Ok(Expr::List(self.exprs("]")?));
        }
        if self.eat("{") {
            // A set `{a, b}` or a map `{k: v}`; `{}` and `{:}` are empty.
            // Both are records of one list, put in order when evaluated.
            let of = |name: &str, items| Expr::Record(name.into(), vec![Expr::List(items)]);
            if self.eat(":") {
                self.expect("}")?;
                return Ok(of(MAP, Vec::new()));
            }
            if self.is_name() && self.peek_at(1) == "in" {
                // { x in xs | cond }: the elements where cond holds.
                let x = self.name()?;
                self.pos += 1;
                // `|` joins two sets: one written here goes in parentheses.
                let from = self.product()?;
                self.expect("|")?;
                self.locals.push(x.clone());
                self.typed.push((x.clone(), Set::any()));
                let cond = self.expr();
                self.locals.pop();
                self.typed.pop();
                self.expect("}")?;
                return Ok(Expr::Each("filter", x, Box::new(from), Box::new(cond?)));
            }
            let (mut items, mut map) = (Vec::new(), false);
            while !self.eat("}") {
                let item = self.expr()?;
                map |= items.is_empty() && self.is(":");
                items.push(match map {
                    true => {
                        self.expect(":")?;
                        Expr::Tuple(vec![item, self.expr()?])
                    }
                    false => item,
                });
                if !self.eat(",") {
                    self.expect("}")?;
                    break;
                }
            }
            return Ok(of(if map { MAP } else { SET }, items));
        }
        if self.is("let") {
            return self.sequence();
        }
        if self.eat("if") {
            let c = self.expr()?;
            let t = self.block()?;
            if !self.eat("else") {
                return self.err("`if` needs an `else`: it gives a value");
            }
            let e = match self.is("if") {
                true => self.primary()?,
                false => self.block()?,
            };
            return Ok(Expr::If(Box::new(c), Box::new(t), Box::new(e)));
        }
        if self.eat("match") {
            // match e { None => a, Some(v) => b, _ => c }
            let e = self.expr()?;
            self.expect("{")?;
            let mut arms = Vec::new();
            while !self.eat("}") {
                let n = self.locals.len();
                let typed = self.typed.len();
                let variant = match self.eat("_") {
                    true => None,
                    false => {
                        let v = self.local()?;
                        let mut fields = Vec::new();
                        if self.eat("(") {
                            while !self.eat(")") {
                                // `_` for a field nobody reads.
                                let x = match self.eat("_") {
                                    true => {
                                        self.fresh += 1;
                                        format!("unread_{}", self.fresh)
                                    }
                                    false => self.local()?,
                                };
                                fields.push(x);
                                if !self.eat(",") {
                                    self.expect(")")?;
                                    break;
                                }
                            }
                        }
                        self.locals.extend(fields.iter().cloned());
                        // A field of a variant has the type the variant gives it.
                        let of = self.seen.records.iter().find(|r| r.name == v);
                        let kind = |i: usize| of.and_then(|r| r.fields.get(i)).map(|f| f.1.clone());
                        let typed = fields.iter().enumerate();
                        let typed: Vec<(String, Set)> = typed
                            .map(|(i, x)| (x.clone(), kind(i).unwrap_or(Set::any())))
                            .collect();
                        self.typed.extend(typed);
                        Some((v, fields))
                    }
                };
                self.expect("=>")?;
                let body = match self.is("{") && self.peek_at(1) == "let" {
                    true => self.block(),
                    false => self.expr(),
                };
                self.locals.truncate(n);
                self.typed.truncate(typed);
                arms.push(Arm {
                    variant,
                    body: body?,
                });
                if !self.eat(",") {
                    self.expect("}")?;
                    break;
                }
            }
            return Ok(Expr::Match(Box::new(e), arms));
        }
        if self.is("correct") && self.peek_at(1) == "(" {
            // correct(f(a, b)) as a value: what the evidence says of the call.
            self.pos += 2;
            let f = self.path()?;
            let f = self.resolve(&f);
            self.expect("(")?;
            let args = self.exprs(")")?;
            self.expect(")")?;
            self.impure = true;
            return Ok(Expr::Correct(f, args));
        }
        if self.eat("true") {
            return Ok(Expr::Lit(Value::Bool(true)));
        }
        if self.eat("false") {
            return Ok(Expr::Lit(Value::Bool(false)));
        }
        if self.is("rate") && self.peek_at(1) == "(" {
            // rate(x in Type, cond): the share of the type where cond holds.
            self.pos += 2;
            let x = self.local()?;
            self.expect("in")?;
            let set = self.set()?;
            self.expect(",")?;
            self.locals.push(x.clone());
            self.typed.push((x.clone(), Set::any()));
            let cond = self.expr();
            self.locals.pop();
            self.typed.pop();
            self.expect(")")?;
            let measure = Expr::Rate(x, set, Box::new(cond?));
            self.measures.push(measure.clone());
            self.impure = true;
            return Ok(measure);
        }
        if let Some(kind) = SCORES.iter().find(|k| self.is(k) && self.peek_at(1) == "(") {
            self.pos += 2;
            let f = self.path()?;
            self.expect(")")?;
            self.impure = true;
            let measure = Expr::Score(kind, self.resolve(&f));
            self.measures.push(measure.clone());
            return Ok(measure);
        }
        let each = EACH.iter().find(|k| self.is(k));
        if let Some(kind) = each.filter(|_| self.peek_at(1) == "(" && self.peek_at(3) == "in") {
            // all(x in xs, cond): over each element.
            self.pos += 2;
            let x = self.local()?;
            self.expect("in")?;
            let list = self.sum()?;
            self.expect(",")?;
            self.locals.push(x.clone());
            self.typed.push((x.clone(), Set::any()));
            let body = self.sequence();
            self.locals.pop();
            self.typed.pop();
            self.expect(")")?;
            let body = body?;
            if *kind == "fold" && !matches!(body, Expr::Let(..)) {
                return self
                    .err("`fold` starts from a value: `fold(x in xs, let sum = 0; sum + x)`");
            }
            return Ok(Expr::Each(kind, x, Box::new(list), Box::new(body)));
        }
        for (word, len) in [("len", true), ("keys", false)] {
            if self.is(word) && self.peek_at(1) == "(" {
                self.pos += 2;
                let e = Box::new(self.expr()?);
                self.expect(")")?;
                return Ok(match len {
                    true => Expr::Len(e),
                    false => Expr::Unary("keys", e),
                });
            }
        }
        let input =
            DECLARING.contains(&self.peek()) && self.locals.iter().any(|l| l == self.peek());
        if !self.is_name() && !input {
            return self.err("expected an expression");
        }
        let name = self.path()?;
        let local = self.locals.contains(&name);
        let name = match local {
            true => name,
            false => self.resolve(&name),
        };
        self.impure |= !local && self.known.opaque.contains(&name);
        if !local && self.own.is_none() && self.known.specified.contains_key(&name) {
            self.called.push(format!("{name}::spec"));
        }
        if self.eat("(") {
            return Ok(Expr::Call(name, self.exprs(")")?));
        }
        Ok(Expr::Var(name))
    }
}
