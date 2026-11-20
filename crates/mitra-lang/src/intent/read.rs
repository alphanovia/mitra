//! Reading intent: what each `ensure` rests on, what a `require`
//! needs, and what the prose of a promise is.

use crate::reader::prelude::*;

impl P {
    /// Each `ensure` as a claim that rests on the claims of its block, on
    /// what the block requires and, in a subject, on `name::captured`.
    pub(crate) fn requirements(&mut self, m: &mut Model) -> R<()> {
        let ensures = std::mem::take(&mut self.ensures);
        let named = |path: &str| {
            let promise = |e: &&(String, String, Prose, usize, usize, Option<f64>)| {
                !matches!(e.2, Prose::Need(_))
            };
            let ends = |q: &String| q == path || q.ends_with(&format!("::{path}"));
            let mut exact = ensures.iter().filter(promise).map(|e| &e.0);
            exact.find(|q| *q == path).or_else(|| {
                ensures
                    .iter()
                    .filter(promise)
                    .map(|e| &e.0)
                    .find(|q| ends(q))
            })
        };
        // What each one waits for: a `require` for what it names, an
        // `ensure` for every `require` of its block.
        let mut waits: Vec<Vec<String>> = Vec::new();
        for (name, _, kind, block, line, _) in &ensures {
            waits.push(match kind {
                Prose::Need(path) => match named(path) {
                    Some(q) => vec![q.clone()],
                    None => {
                        let msg =
                            format!("`require {name} = {path}`: no `ensure` is named `{path}`");
                        return Err(Error::new(*line, msg));
                    }
                },
                Prose::Wanted => Vec::new(),
                Prose::Promise | Prose::World => {
                    let of = ensures.iter().filter(|e| e.3 == *block);
                    let needs = of.filter(|e| matches!(e.2, Prose::Need(_) | Prose::Wanted));
                    let mut waits: Vec<String> = needs.map(|e| e.0.clone()).collect();
                    // A promise of a contract: the same of each implementer.
                    let fills = self.contracts.fills.iter().filter(|f| &f.0 == name);
                    waits.extend(fills.map(|f| f.1.clone()));
                    waits
                }
            });
        }
        let mut order: Vec<usize> = Vec::new();
        while order.len() < ensures.len() {
            let ready = (0..ensures.len()).find(|i| {
                let done = |n: &String| order.iter().any(|j| &ensures[*j].0 == n);
                !order.contains(i) && waits[*i].iter().all(done)
            });
            let Some(i) = ready else {
                let (name, _, _, _, line, _) =
                    &ensures[(0..ensures.len()).find(|i| !order.contains(i)).unwrap()];
                let msg = format!("`{name}` requires itself, through what it requires");
                return Err(Error::new(*line, msg));
            };
            order.push(i);
        }
        let all: Vec<String> = ensures.iter().map(|e| e.0.clone()).collect();
        for i in order {
            let (name, prose, kind, block, line, trust) = ensures[i].clone();
            if m.claims.iter().any(|c| c.name == name) || m.functions.iter().any(|f| f.name == name)
            {
                return Err(Error::new(line, format!("`{name}` is declared twice")));
            }
            // Its claims: those of the block that say something of results.
            let exists = |n: &String| {
                m.claims
                    .iter()
                    .any(|c| &c.name == n && !c.target.is_structural())
            };
            let mut claims: Vec<String> = Vec::new();
            for (c, b) in &self.owned {
                if *b == block && exists(c) && !claims.contains(c) && !all.contains(c) {
                    claims.push(c.clone());
                }
            }
            let requires = waits[i].clone();
            // A promise of the world rests on the block's assumptions, as
            // one of a subject on its claims; with none, on its own word.
            let world = kind == Prose::World;
            if matches!(kind, Prose::Need(_) | Prose::Wanted) {
                claims.clear();
            }
            let boundary = match kind {
                Prose::Promise => false,
                Prose::World => claims.is_empty(),
                Prose::Need(_) | Prose::Wanted => true,
            };
            let mut state = |m: &mut Model, name: &str, from: Vec<String>, trust: Option<f64>| {
                m.functions.push(Function {
                    name: format!("{name}::prose"),
                    params: Vec::new(),
                    range: Set::bool(),
                    body: None,
                    written: None,
                    line,
                });
                m.claims.push(Claim {
                    name: name.to_string(),
                    func: format!("{name}::prose"),
                    pattern: Vec::new(),
                    target: Target::Correct,
                    from,
                    auto: false,
                    proved: false,
                    trust,
                    line,
                });
                if let Some(t) = trust {
                    self.names.trust.insert(name.to_string(), t);
                }
                self.names
                    .claims
                    .entry(name.to_string())
                    .or_insert((Vec::new(), Vec::new()));
                let sig = Sig {
                    params: Vec::new(),
                    deep: vec![Set::bool()],
                };
                self.names.fns.insert(format!("{name}::prose"), sig);
            };
            let mut from = claims.clone();
            from.extend(requires.clone());
            let filled = self.contracts.fills.iter().any(|f| f.0 == name);
            if matches!(kind, Prose::Need(_)) || (filled && boundary) {
                // It holds as far as the promise it names; that of a
                // contract, as far as its implementers keep it.
                state(m, &name, requires.clone(), trust);
            } else if boundary {
                // A promise of the world: assumed, as far as it is not observed.
                state(m, &name, requires.clone(), trust);
                self.declare(&name, "assumed", trust.unwrap_or(1.0));
            } else {
                let captured = format!("{name}::captured");
                state(m, &captured, Vec::new(), None);
                from.push(captured);
                state(m, &name, from, trust);
            }
            m.ensures.push(Ensure {
                name,
                prose,
                boundary,
                world,
                needs: match kind {
                    Prose::Need(_) => requires.first().cloned(),
                    _ => None,
                },
                claims,
                requires,
                line,
            });
        }
        self.reliance(m, &ensures, &waits);
        Ok(())
    }

    /// Intent against calls: a `require` on a block nothing here uses, and
    /// a `uses` on a block that makes promises, none of them required
    /// here. Only blocks that state an `ensure` or a `require` are held
    /// to the second.
    pub(crate) fn reliance(
        &self,
        m: &mut Model,
        ensures: &[(String, String, Prose, usize, usize, Option<f64>)],
        waits: &[Vec<String>],
    ) {
        // The block of each function, and of each promise.
        let home = |f: &str| {
            let own = format!("{f}::correct");
            self.owned.iter().find(|o| o.0 == own).map(|o| o.1)
        };
        let made = |name: &str| ensures.iter().find(|e| e.0 == name).map(|e| e.3);
        // (function, what it uses, the block of that), across blocks.
        let mut calls: Vec<(&Claim, &String, usize, usize)> = Vec::new();
        for c in &m.claims {
            let Target::Uses { used, .. } = &c.target else {
                continue;
            };
            let Some(from) = home(&c.func) else { continue };
            for u in used {
                match home(u) {
                    Some(to) if to != from => calls.push((c, u, from, to)),
                    _ => {}
                }
            }
        }
        // What a block leans on, beyond `uses`: the calls of its bodies
        // and of its claims, then all of it through other blocks.
        let mut leans: Vec<(usize, usize)> = calls.iter().map(|c| (c.2, c.3)).collect();
        let code = |g: &str| {
            let g = g.strip_suffix("::spec").or(g.strip_suffix("::body"));
            g.map(str::to_string)
        };
        let mut reach = |from: Option<usize>, e: &Expr| {
            for g in crate::model::calls(e) {
                let to = home(&code(&g).unwrap_or(g));
                if let (Some(from), Some(to)) = (from, to.filter(|to| Some(*to) != from)) {
                    leans.push((from, to));
                }
            }
        };
        for f in &m.functions {
            if let Some(body) = &f.body {
                let name = code(&f.name).unwrap_or(f.name.clone());
                reach(home(&name), body);
            }
        }
        for c in &m.claims {
            let from = self.owned.iter().find(|o| o.0 == c.name).map(|o| o.1);
            if let Target::Property(e) | Target::Matches(e) = &c.target {
                reach(from, e);
            }
        }
        loop {
            let more: Vec<(usize, usize)> = leans
                .iter()
                .flat_map(|a| leans.iter().filter(|b| b.0 == a.1).map(|b| (a.0, b.1)))
                .filter(|e| e.0 != e.1 && !leans.contains(e))
                .collect();
            if more.is_empty() {
                break;
            }
            leans.extend(more);
            leans.sort();
            leans.dedup();
        }
        // A type of the other block, in a signature or a field here.
        let typed = |block: usize, here: Option<&str>, there: &str| {
            let there = format!("{there}::");
            let names = |s: &Set| s.to_string().contains(&there);
            let sig = |f: &Function| f.params.iter().any(|p| names(&p.1)) || names(&f.range);
            let own = |f: &&Function| home(&code(&f.name).unwrap_or(f.name.clone())) == Some(block);
            let mine = |r: &&Record| here.is_some_and(|h| r.name.starts_with(&format!("{h}::")));
            m.functions.iter().filter(own).any(sig)
                || (m.records.iter().filter(mine)).any(|r| r.fields.iter().any(|f| names(&f.1)))
        };
        let scope = |name: &str| name.rsplit_once("::").map(|n| n.0.to_string());
        let mut lints = Vec::new();
        for (i, (name, _, kind, block, line, _)) in ensures.iter().enumerate() {
            let Prose::Need(_) = kind else { continue };
            let promise = &waits[i][0];
            let to = made(promise).expect("named");
            let shares = scope(promise).is_some_and(|t| typed(*block, scope(name).as_deref(), &t));
            // `x: impl C` in a signature beside it leans on the contract.
            let pair = scope(name).zip(scope(promise));
            let leans_on = |p: (String, String)| {
                let mut all = self.contracts.leans.iter().chain(&self.leaning);
                all.any(|l| l.0 == p.0 && l.1 == p.1)
            };
            let takes = pair.is_some_and(leans_on);
            if !leans.contains(&(*block, to)) && !shares && !takes {
                lints.push(Lint {
                    code: "M105",
                    line: *line,
                    message: format!(
                        "`{name}` needs `{promise}`, and nothing declared beside it leans on \
                         what is declared beside `{promise}`: no `uses`, no call, no type."
                    ),
                    help: "say which function `uses` it, or remove the `require`".to_string(),
                });
            }
        }
        let mut seen: Vec<(&str, usize)> = Vec::new();
        for (c, used, from, to) in &calls {
            let states = ensures.iter().any(|e| e.3 == *from);
            let promises: Vec<&String> = ensures
                .iter()
                .filter(|e| e.3 == *to && matches!(e.2, Prose::Promise | Prose::World))
                .map(|e| &e.0)
                .collect();
            let required = ensures.iter().enumerate().any(|(i, e)| {
                e.3 == *from && matches!(e.2, Prose::Need(_)) && made(&waits[i][0]) == Some(*to)
            });
            if !states || promises.is_empty() || required || seen.contains(&(&c.func, *to)) {
                continue;
            }
            seen.push((&c.func, *to));
            let all: Vec<String> = promises.iter().map(|p| format!("`{p}`")).collect();
            lints.push(Lint {
                code: "M106",
                line: c.line,
                message: format!(
                    "`{}` uses `{used}`, and no `require` beside it names a promise made \
                     beside `{used}` ({}).",
                    c.func,
                    all.join(", ")
                ),
                help: format!(
                    "write `require <name> = {};` beside `{}`",
                    promises[0], c.func
                ),
            });
        }
        // `x: impl C` in a block that states its intent, and no `require`
        // beside it names a promise of the contract.
        let mut told: Vec<(&String, &String)> = Vec::new();
        for (block, contract, line) in self.contracts.leans.iter().chain(&self.leaning) {
            let inside = |e: &&(String, String, Prose, usize, usize, Option<f64>)| {
                scope(&e.0).as_ref() == Some(block)
            };
            let promises: Vec<&String> = ensures
                .iter()
                .filter(|e| scope(&e.0).as_ref() == Some(contract))
                .filter(|e| matches!(e.2, Prose::Promise | Prose::World))
                .map(|e| &e.0)
                .collect();
            let required = ensures.iter().enumerate().any(|(i, e)| {
                inside(&e) && matches!(e.2, Prose::Need(_)) && promises.contains(&&waits[i][0])
            });
            let states = ensures.iter().any(|e| inside(&e));
            // Already said, of a function that `uses` the contract.
            let to = promises.first().and_then(|p| made(p));
            let own = format!("{block}::");
            let said = (seen.iter()).any(|s| Some(s.1) == to && s.0.starts_with(&own));
            if !states || promises.is_empty() || required || said {
                continue;
            }
            if told.contains(&(block, contract)) {
                continue;
            }
            told.push((block, contract));
            let all: Vec<String> = promises.iter().map(|p| format!("`{p}`")).collect();
            lints.push(Lint {
                code: "M106",
                line: *line,
                message: format!(
                    "`{block}` takes a value bound by `{contract}`, and no `require` in it \
                     names a promise of `{contract}` ({}).",
                    all.join(", ")
                ),
                help: format!("write `require <name> = {};` in `{block}`", promises[0]),
            });
        }
        m.lints = lints;
    }
}

/// What a line of prose is: a promise of a subject, a promise of the
/// world (assumed), or the need of a promise made elsewhere.
#[derive(Clone, PartialEq)]
pub(crate) enum Prose {
    Promise,
    World,
    Need(String),
    /// `require name = "prose";`: needed, with nobody to promise it yet.
    Wanted,
}
