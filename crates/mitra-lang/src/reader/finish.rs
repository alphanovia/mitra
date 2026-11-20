//! The end of a read: what was declared becomes the model.

use crate::reader::prelude::*;

impl P {
    /// The model, once every declaration is read.
    pub(crate) fn finish(mut self) -> R<(Model, Names)> {
        let mut m = std::mem::take(&mut self.m);
        // A property uses every name it binds, and no other.
        for (i, bound) in std::mem::take(&mut self.properties) {
            // `f(x, y) == e` with one name per input is the claim that the
            // output of `f` is `e`.
            let record = |f: &String| m.records.iter().any(|r| &r.name == f);
            if let Target::Property(Expr::Binary("==", call, e)) = &m.claims[i].target {
                if let Expr::Call(f, args) = &**call {
                    let names: Vec<&String> = args
                        .iter()
                        .filter_map(|a| match a {
                            Expr::Var(x) => Some(x),
                            _ => None,
                        })
                        .collect();
                    let each_once = names.len() == args.len()
                        && names.len() == bound.len()
                        && bound.iter().all(|b| names.contains(&&b.0));
                    if each_once && !record(f) {
                        let pattern = names
                            .iter()
                            .map(|x| Position {
                                var: Some(x.to_string()),
                                set: bound.iter().find(|b| &&b.0 == x).unwrap().1.clone(),
                            })
                            .collect();
                        let (func, target) = (f.clone(), Target::Matches((**e).clone()));
                        let c = &mut m.claims[i];
                        (c.func, c.pattern, c.target) = (func, pattern, target);
                        continue;
                    }
                }
            }
            let c = &m.claims[i];
            let Target::Property(e) = &c.target else {
                unreachable!()
            };
            let mut used = Vec::new();
            let variant = |x: &String| {
                m.functions.iter().any(|f| &f.name == x)
                    || m.records
                        .iter()
                        .any(|r| &r.name == x && r.fields.is_empty())
            };
            if let Some(x) = free(e, &mut Vec::new(), &mut used)
                .into_iter()
                .find(|x| !bound.iter().any(|b| &b.0 == x) && !variant(x))
            {
                let msg = format!("`{x}` is not known here: it is no input of the claim");
                return Err(Error::new(c.line, msg));
            }
            if let Some((x, _, line)) = bound.iter().find(|b| !used.contains(&b.0)) {
                return Err(Error::new(*line, format!("the input `{x}` is not used")));
            }
        }
        m.recursive = std::mem::take(&mut self.recursive);
        m.aliases = std::mem::take(&mut self.aliases);
        m.aliases.sort();
        m.aliases.dedup();
        // `Name(a, b)` is a record when `Name` is one, wherever it is declared.
        let names: Vec<String> = m.records.iter().map(|r| r.name.clone()).collect();
        for f in &mut m.functions {
            if let Some(body) = &mut f.body {
                records_in(body, &names);
            }
        }
        for c in &mut m.claims {
            if let Target::Matches(e) | Target::Property(e) = &mut c.target {
                records_in(e, &names);
            }
        }
        // No value of a record with unstated parts can be written.
        let built = |e: &Expr| -> Option<String> {
            let mut found = None;
            crate::model::walk(e, &mut |x| {
                if let Expr::Record(n, _) = x {
                    if m.records.iter().any(|r| &r.name == n && r.open) {
                        found = Some(n.clone());
                    }
                }
            });
            found
        };
        for f in &m.functions {
            if let Some(n) = f.body.as_ref().and_then(&built) {
                return Err(Error::new(f.line, unwritten(&n)));
            }
        }
        for c in &m.claims {
            if let Target::Matches(e) | Target::Property(e) = &c.target {
                if let Some(n) = built(e) {
                    return Err(Error::new(c.line, unwritten(&n)));
                }
            }
        }
        // The code of a subject is claimed correct: `f::correct`, unless
        // the model has its own claim on `correct(f(…))`, or a body, whose
        // spec says what correct is.
        let mut implied = Vec::new();
        for f in m.functions.iter().filter(|f| !f.is_spec()) {
            let in_subject = !self.names.boundaries.contains(&f.name);
            // A point (`correct(f(1, 2))`, what a test pins) narrows nothing.
            let own = |c: &Claim| {
                let inputs = c.pattern.iter().any(|p| p.var.is_some());
                c.func == f.name && c.target == Target::Correct && inputs
            };
            let specified = self.names.specified.contains_key(&f.name);
            if !in_subject || !self.names.opaque.contains(&f.name) || specified {
                continue;
            }
            if m.claims.iter().any(own) {
                continue;
            }
            let inputs: Vec<String> = f.params.iter().map(|p| p.0.clone()).collect();
            let pattern = inputs.iter().map(|x| Position {
                var: Some(x.clone()),
                set: None,
            });
            implied.push(Claim {
                name: format!("{}::correct", f.name),
                func: f.name.clone(),
                pattern: pattern.collect(),
                target: Target::Correct,
                from: Vec::new(),
                auto: false,
                proved: false,
                trust: None,
                line: f.line,
            });
            let name = format!("{}::correct", f.name);
            self.names
                .claims
                .entry(name)
                .or_insert((inputs, Vec::new()));
        }
        for c in implied {
            // Beside the function: before the first claim written after it.
            let at = m.claims.iter().position(|d| d.line > c.line);
            m.claims.insert(at.unwrap_or(m.claims.len()), c);
        }
        // The code of a subject is claimed deterministic, the same input
        // always giving the same output: `f::deterministic`, unless the
        // model has its own claim on `deterministic(f)`.
        let mut stated = self.names.determinism.clone();
        for f in self.names.opaque.clone() {
            let own = stated.iter().any(|d| d.0 == f);
            let line = m.functions.iter().find(|g| g.name == f).map(|g| g.line);
            if let (false, false, Some(line)) = (own, self.names.boundaries.contains(&f), line) {
                stated.push((f.clone(), format!("{f}::deterministic"), true, line));
            }
        }
        for (f, name, same, line) in stated {
            let func = format!("{name}::prose");
            m.functions.push(Function {
                name: func.clone(),
                params: Vec::new(),
                range: Set::bool(),
                body: None,
                written: None,
                line,
            });
            let claim = Claim {
                name: name.clone(),
                func: func.clone(),
                pattern: Vec::new(),
                target: Target::Correct,
                from: Vec::new(),
                auto: false,
                proved: false,
                trust: self.names.trust.get(&name).copied(),
                line,
            };
            let at = m.claims.iter().position(|d| d.line > line);
            m.claims.insert(at.unwrap_or(m.claims.len()), claim);
            self.names
                .claims
                .entry(name.clone())
                .or_insert((Vec::new(), Vec::new()));
            let sig = Sig {
                params: Vec::new(),
                deep: vec![Set::bool()],
            };
            self.names.fns.insert(func, sig);
            m.determinism.push((name, f, same));
        }
        self.requirements(&mut m)?;
        Ok((m, self.names))
    }
}
