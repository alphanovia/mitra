//! Reading contracts and type parameters: `contract C {{ }}`, `<T: C>`,
//! a contract brought from another store.

use crate::reader::prelude::*;

impl P {
    /// `<T, U>` after a name.
    pub(crate) fn type_params(&mut self, of: &str) -> R<Vec<String>> {
        let mut names = Vec::new();
        self.bounds.clear();
        if !self.eat("<") {
            return Ok(names);
        }
        loop {
            let t = self.ident()?;
            if names.contains(&t) {
                return self.err(format!("`{of}` has two parameters `{t}`"));
            }
            if self.eat(":") {
                let bound = self.bounded_by()?;
                self.bounds.push((t.clone(), bound));
            }
            names.push(t);
            if !self.eat(",") {
                self.expect(">")?;
                return Ok(names);
            }
        }
    }

    /// `A + B<X>` after `T:`: each names a contract.
    pub(crate) fn bounded_by(&mut self) -> R<Vec<String>> {
        let mut all = Vec::new();
        loop {
            let name = self.ident()?;
            if !self.contracts.names.contains(&name) {
                self.pos -= 1;
                return self.err(format!(
                    "`{name}` is not a contract: a bound names a contract"
                ));
            }
            all.push(name);
            if self.eat("<") {
                self.type_tokens(">")?;
                while self.eat(",") {
                    self.type_tokens(">")?;
                }
                self.expect(">")?;
            }
            if !self.eat("+") {
                return Ok(all);
            }
        }
    }

    /// No bound on a `record` or an `enum`: a bound says what a function
    /// leans on.
    pub(crate) fn unbounded(&mut self, of: &str) -> R<()> {
        match self.bounds.first() {
            Some((t, _)) => self.err(format!(
                "`{t}: …` on `{of}`: a bound is written on a function, an `impl` or a contract"
            )),
            None => Ok(()),
        }
    }

    /// The tokens of a type, up to `,` or `close` outside any bracket.
    pub(crate) fn type_tokens(&mut self, close: &str) -> R<Vec<Tok>> {
        let (start, mut depth) = (self.pos, 0);
        loop {
            let Some(t) = self.toks.get(self.pos).filter(|t| t.kind != Kind::Str) else {
                return self.err(format!("expected `{close}`"));
            };
            let t = t.text.as_str();
            if depth == 0 && (t == "," || t == close) {
                return Ok(self.toks[start..self.pos].to_vec());
            }
            depth += ["(", "<", "{", "["].contains(&t) as i32;
            depth -= [")", ">", "}", "]"].contains(&t) as i32;
            self.pos += 1;
        }
    }
}

/// What the contracts of a file come to, once read as blocks.
#[derive(Debug, Clone, Default)]
pub(crate) struct Contracts {
    pub(crate) names: BTreeSet<String>,
    /// A promise of a contract, and the same promise of one implementer.
    pub(crate) fills: Vec<(String, String)>,
    /// A block, a contract a signature of it names, and the line.
    pub(crate) leans: Vec<(String, String, usize)>,
    /// The functions of a contract that take `self`.
    pub(crate) methods: BTreeSet<String>,
    /// The contracts brought from another store, each with its store.
    pub(crate) brought: BTreeMap<String, String>,
    /// `x: impl C`: the line of the name of its function, that name, the
    /// input and the contracts.
    pub(crate) sugar: Vec<(usize, String, String, Vec<String>)>,
}

/// `contract C { … }`, `impl C for T { … }` and `x: impl C`, read as what
/// they stand for. A contract is a block of its own: its functions take
/// any value as `self`. An implementer is given each function and each
/// promise of the contract, in a group of its own after its block, so
/// its promises rest on what it says and on nothing else. `impl C` as a
/// type is any value. `contract C<K> { … }` gives each function the
/// parameter; `impl C<Int> for T` gives `T` the functions with `Int`.
pub(crate) fn contracts(toks: Vec<Tok>) -> Result<(Vec<Tok>, Contracts), Error> {
    let word = |t: &Tok, w: &str| t.kind == Kind::Name && t.text == w;
    let punct = |t: &Tok, w: &str| t.kind == Kind::Punct && t.text == w;
    let mut c = Contracts::default();
    for (i, t) in toks.iter().enumerate() {
        let named = toks.get(i + 1).filter(|n| n.kind == Kind::Name);
        let opens = toks
            .get(i + 2)
            .is_some_and(|b| b.text == "{" || b.text == "<");
        if let (true, Some(n), true) = (word(t, "contract"), named, opens) {
            c.names.insert(n.text.clone());
        }
    }
    let toks = impl_types(toks, &mut c)?;
    let is = |i: usize, w: &str| {
        toks.get(i)
            .is_some_and(|t| t.text == w && t.kind != Kind::Str)
    };
    let named = |i: usize| toks.get(i).filter(|t| t.kind == Kind::Name);
    // What each contract holds: its type parameters, its signatures, its
    // promises, its claims.
    struct Held {
        params: Vec<Tok>,
        fns: Vec<Vec<Tok>>,
        ensures: Vec<(String, Vec<Tok>)>,
        claims: Vec<Vec<Tok>>,
    }
    let mut held: BTreeMap<String, Held> = BTreeMap::new();
    let mut spans: Vec<(usize, usize, String)> = Vec::new();
    let (mut depth, mut i) = (0usize, 0);
    while i < toks.len() {
        let t = &toks[i];
        if punct(t, "{") {
            depth += 1;
        } else if punct(t, "}") {
            depth = depth.saturating_sub(1);
        } else if word(t, "contract") && depth == 0 && named(i + 1).is_some() {
            let name = toks[i + 1].text.clone();
            let mut open = i + 2;
            let mut params = Vec::new();
            if is(open, "<") {
                let msg = format!("the `<` of `contract {name}` is not closed");
                let end = angle(&toks, open).ok_or(Error::new(t.line, msg))?;
                params = toks[open + 1..end].to_vec();
                open = end + 1;
            }
            if !is(open, "{") {
                i += 1;
                continue;
            }
            let Some(end) = closing(&toks, open) else {
                return Err(Error::new(
                    t.line,
                    format!("`contract {name}` is not closed"),
                ));
            };
            if held.contains_key(&name) {
                return Err(Error::new(
                    t.line,
                    format!("`contract {name}` is declared twice"),
                ));
            }
            let mut h = Held {
                params,
                fns: Vec::new(),
                ensures: Vec::new(),
                claims: Vec::new(),
            };
            let mut j = open + 1;
            while j < end {
                let start = j;
                let kw = toks[j].text.clone();
                let only = "a contract holds `fn` signatures, `ensure` lines and claims";
                match kw.as_str() {
                    "fn" | "ensure" if toks[j].kind == Kind::Name => {
                        while j < end && !punct(&toks[j], ";") {
                            if punct(&toks[j], "{") {
                                let msg = match kw.as_str() {
                                    "fn" => "a function of a contract has no body: \
                                             no default method"
                                        .to_string(),
                                    _ => only.to_string(),
                                };
                                return Err(Error::new(toks[j].line, msg));
                            }
                            j += 1;
                        }
                        j += 1;
                        let item = toks[start..j.min(end)].to_vec();
                        match kw.as_str() {
                            "fn" => h.fns.push(item),
                            _ => {
                                let n = named(start + 1).map(|n| n.text.clone());
                                h.ensures.push((n.unwrap_or_default(), item));
                            }
                        }
                    }
                    "claim" if toks[j].kind == Kind::Name => {
                        // To its `;`, or to the `}` of its braces.
                        while j < end && !punct(&toks[j], ";") && !punct(&toks[j], "{") {
                            j += 1;
                        }
                        if j < end && punct(&toks[j], "{") {
                            j = closing(&toks[..end], j).unwrap_or(end - 1);
                        }
                        j += 1;
                        h.claims.push(toks[start..j.min(end)].to_vec());
                    }
                    _ => return Err(Error::new(toks[j].line, format!("`{kw}`: {only}"))),
                }
            }
            spans.push((i, end, name.clone()));
            held.insert(name, h);
            i = end + 1;
            continue;
        }
        i += 1;
    }
    if held.is_empty() && !toks.iter().any(|t| word(t, "for")) {
        return Ok((toks, c));
    }
    let any = |at: &Tok| Tok {
        kind: Kind::Name,
        text: "Any".into(),
        line: at.line,
    };
    let mut out: Vec<Tok> = Vec::new();
    // The groups to write after the block being read.
    let mut after: Vec<Tok> = Vec::new();
    let mut filled: Vec<(String, String)> = Vec::new();
    let (mut depth, mut scope): (usize, Option<(String, String)>) = (0, None);
    let mut i = 0;
    while i < toks.len() {
        let t = &toks[i];
        if let Some((_, end, name)) = spans.iter().find(|s| s.0 == i) {
            // The contract: a block of the world, `self` any value.
            let h = &held[name];
            out.push(Tok {
                text: "boundary".into(),
                ..t.clone()
            });
            out.push(toks[i + 1].clone());
            out.push(Tok {
                kind: Kind::Punct,
                text: "{".into(),
                line: t.line,
            });
            for f in &h.fns {
                let f = &with_params(f, &h.params);
                let open = f.iter().position(|a| punct(a, "(")).unwrap_or(f.len());
                let mut k = 0;
                while k < f.len() {
                    // `(self`, `(&self`, `(&mut self`: an input `self: Any`.
                    let mut s = k + 1;
                    s += usize::from(f.get(s).is_some_and(|a| a.text == "&"));
                    s += usize::from(f.get(s).is_some_and(|a| a.text == "mut"));
                    let own = k == open
                        && f.get(s).is_some_and(|a| word(a, "self"))
                        && f.get(s + 1).is_none_or(|a| a.text != ":");
                    if own {
                        out.push(f[k].clone());
                        out.push(f[s].clone());
                        out.push(Tok {
                            kind: Kind::Punct,
                            text: ":".into(),
                            line: f[s].line,
                        });
                        out.push(any(&f[s]));
                        c.methods.insert(format!("{name}::{}", f[1].text));
                        k = s + 1;
                        continue;
                    }
                    out.push(match word(&f[k], "Self") {
                        true => any(&f[k]),
                        false => f[k].clone(),
                    });
                    k += 1;
                }
            }
            for (_, e) in &h.ensures {
                out.extend(e.iter().cloned());
            }
            out.push(toks[*end].clone());
            i = end + 1;
            continue;
        }
        // impl C for T { … }, impl<P> C<A> for T<P> { … }
        let brace = (i..toks.len()).find(|&j| ["{", "}", ";"].contains(&toks[j].text.as_str()));
        let fills = word(t, "impl")
            && brace.is_some_and(|b| punct(&toks[b], "{") && (i..b).any(|j| word(&toks[j], "for")));
        if punct(t, "{") {
            depth += 1;
        } else if punct(t, "}") {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                scope = None;
                out.push(t.clone());
                out.append(&mut after);
                i += 1;
                continue;
            }
        } else if (word(t, "subject") || word(t, "boundary")) && depth == 0 && is(i + 2, "{") {
            scope = Some((t.text.clone(), toks[i + 1].text.clone()));
        } else if fills {
            let form = "expected `impl Contract for Type { … }`";
            let open = brace.expect("found");
            let mut j = i + 1;
            let mut params: &[Tok] = &[];
            if is(j, "<") {
                let end = angle(&toks, j).ok_or(Error::new(t.line, form))?;
                params = &toks[j + 1..end];
                j = end + 1;
            }
            let of = named(j).ok_or(Error::new(t.line, form))?;
            j += 1;
            let mut args: Vec<Vec<Tok>> = Vec::new();
            if is(j, "<") {
                let end = angle(&toks, j).ok_or(Error::new(t.line, form))?;
                args = type_args(&toks[j + 1..end]);
                j = end + 1;
            }
            if !is(j, "for") {
                return Err(Error::new(t.line, form));
            }
            let by = named(j + 1).ok_or(Error::new(t.line, form))?;
            let target = &toks[j + 2..open];
            let Some(h) = held.get(&of.text) else {
                let msg = format!("`{}` is not a contract: no `contract {0} {{ … }}`", of.text);
                return Err(Error::new(t.line, msg));
            };
            let Some((side, block)) = scope.clone().filter(|_| depth == 1) else {
                let msg = format!(
                    "`impl {} for {}` is written in a `subject {{ … }}` or a `boundary {{ … }}`",
                    of.text, by.text
                );
                return Err(Error::new(t.line, msg));
            };
            let wanted = param_names(&h.params);
            if wanted.len() != args.len() {
                let msg = format!(
                    "`{}` takes {} type(s), `impl {0}<…> for {}` gives {}",
                    of.text,
                    wanted.len(),
                    by.text,
                    args.len()
                );
                return Err(Error::new(t.line, msg));
            }
            let end =
                closing(&toks, open).ok_or(Error::new(t.line, "this `impl` is not closed"))?;
            // Its braces: what decomposes the promises for this
            // implementer, what it requires, and a function of the
            // contract written again to say what it `uses` or computes.
            let mut again: Vec<(String, Vec<Tok>)> = Vec::new();
            let mut rest: Vec<Tok> = Vec::new();
            let mut k = open + 1;
            while k < end {
                let b = &toks[k];
                let mut stop = k;
                while stop < end && !punct(&toks[stop], ";") && !punct(&toks[stop], "{") {
                    stop += 1;
                }
                if stop < end && punct(&toks[stop], "{") {
                    stop = closing(&toks[..end], stop).unwrap_or(end - 1);
                }
                let item = &toks[k..(stop + 1).min(end)];
                match b.text.as_str() {
                    "claim" | "assume" | "require" if b.kind == Kind::Name => {
                        rest.extend(item.iter().cloned())
                    }
                    "fn" if b.kind == Kind::Name => {
                        let name = item.get(1).map_or("", |n| n.text.as_str());
                        let mut sig = Vec::new();
                        if let Some(f) = h.fns.iter().find(|f| f[1].text == name) {
                            given_types(f, &wanted, &args, t.line, &mut sig);
                        }
                        sig.pop();
                        let same = !sig.is_empty()
                            && item.len() >= sig.len()
                            && sig.iter().zip(item).all(|(a, b)| a.text == b.text);
                        if !same {
                            let msg = format!(
                                "`fn {name}` in `impl {} for {}` is not a function of the \
                                 contract, written as the contract writes it",
                                of.text, by.text
                            );
                            return Err(Error::new(b.line, msg));
                        }
                        again.push((name.to_string(), item.to_vec()));
                    }
                    _ => {
                        let msg = format!(
                            "`{}`: an `impl {} for {}` holds only what decomposes the promises \
                             of the contract: `claim` in a subject, `assume` in a boundary, \
                             `require`, or a function of the contract with what it `uses`",
                            b.text, of.text, by.text
                        );
                        return Err(Error::new(b.line, msg));
                    }
                }
                k = stop + 1;
            }
            let pair = (format!("{block}::{}", by.text), of.text.clone());
            if filled.contains(&pair) {
                let msg = format!("`{}` fills `{}` twice", pair.0, pair.1);
                return Err(Error::new(t.line, msg));
            }
            filled.push(pair);
            let at = |text: &str, kind: Kind| Tok {
                kind,
                text: text.into(),
                line: t.line,
            };
            let given =
                |item: &[Tok], into: &mut Vec<Tok>| given_types(item, &wanted, &args, t.line, into);
            after.push(at(&side, Kind::Name));
            after.push(at(&block, Kind::Name));
            after.push(at("{", Kind::Punct));
            after.push(at("impl", Kind::Name));
            if !params.is_empty() {
                after.push(at("<", Kind::Punct));
                after.extend(params.iter().cloned());
                after.push(at(">", Kind::Punct));
            }
            after.push(by.clone());
            after.extend(target.iter().cloned());
            after.push(at("{", Kind::Punct));
            for f in &h.fns {
                match again.iter().find(|a| a.0 == f[1].text) {
                    Some((_, written)) => after.extend(written.iter().cloned()),
                    None => given(f, &mut after),
                }
            }
            for (n, e) in &h.ensures {
                given(e, &mut after);
                let fill = format!("{block}::{}::{n}", by.text);
                c.fills.push((format!("{}::{n}", of.text), fill));
            }
            let said = match side.as_str() {
                "boundary" => "assume",
                _ => "claim",
            };
            for claim in &h.claims {
                after.push(at(said, Kind::Name));
                given(&claim[1..], &mut after);
            }
            after.extend(rest);
            after.push(at("}", Kind::Punct));
            after.push(at("}", Kind::Punct));
            i = end + 1;
            continue;
        }
        out.push(t.clone());
        i += 1;
    }
    out.append(&mut after);
    Ok((out, c))
}

/// `import store::Contract;` or `import store::{A, B};`: each contract
/// of the model of another store, written here in place of the line.
/// Also gives each contract brought with its store.
pub(crate) fn bring_contracts(
    toks: Vec<Tok>,
    handed: &Handed,
) -> Result<(Vec<Tok>, BTreeMap<String, String>), Error> {
    let mut brought: BTreeMap<String, String> = BTreeMap::new();
    // The types this text has: its own, and those it brings by name.
    let mut known: BTreeSet<String> = types_of(&toks).0;
    for (path, names) in imports_of(&toks).iter().filter(|i| i.0.len() == 2) {
        known.extend(names.iter().map(|n| format!("{}::{n}", path[1])));
    }
    // The contracts go after the last `import`: the types they name are
    // brought above them.
    let mut last = 0;
    let mut level = 0;
    for (k, t) in toks.iter().enumerate() {
        if t.kind == Kind::Punct {
            level += (t.text == "{") as i32 - (t.text == "}") as i32;
        }
        if level == 0 && t.kind == Kind::Name && t.text == "import" {
            let end = (k..toks.len()).find(|&j| toks[j].text == ";");
            last = end.map_or(toks.len(), |j| j + 1);
        }
    }
    let mut held: Vec<Tok> = Vec::new();
    let (mut out, mut depth, mut i) = (Vec::new(), 0, 0);
    while i < toks.len() {
        if i == last {
            out.append(&mut held);
        }
        let t = &toks[i];
        if t.kind == Kind::Punct {
            depth += (t.text == "{") as i32 - (t.text == "}") as i32;
        }
        let is = |j: usize, w: &str| toks.get(j).is_some_and(|t| t.text == w);
        let store = toks.get(i + 1).filter(|s| s.kind == Kind::Name);
        let store = store.filter(|s| handed.stores.contains_key(&s.text));
        let store = store.filter(|s| !handed.modules.contains_key(&s.text));
        let line = depth == 0 && t.kind == Kind::Name && t.text == "import" && is(i + 2, "::");
        // One name then `;`, or names in braces: not `store::block::…`.
        let (Some(store), true) = (store, line && !is(i + 4, "::")) else {
            out.push(t.clone());
            i += 1;
            continue;
        };
        let mut j = i + 3;
        let mut names = Vec::new();
        while toks.get(j).is_some_and(|n| n.text != ";") {
            if toks[j].kind == Kind::Name {
                names.push(toks[j].text.clone());
            }
            j += 1;
        }
        let there = lex(&handed.stores[&store.text]).unwrap_or_default();
        for name in names {
            match brought.get(&name) {
                Some(from) if from == &store.text => continue,
                Some(from) => {
                    let msg = format!("`{name}` is brought from `{from}` already");
                    return Err(Error::new(t.line, msg));
                }
                None => {}
            }
            let mut level = 0;
            let mut start = None;
            for (k, a) in there.iter().enumerate() {
                let named = there.get(k + 1).is_some_and(|n| n.text == name);
                if level == 0 && a.kind == Kind::Name && a.text == "contract" && named {
                    start = Some(k);
                    break;
                }
                if a.kind == Kind::Punct {
                    level += (a.text == "{") as i32 - (a.text == "}") as i32;
                }
            }
            let open = start.and_then(|k| (k..there.len()).find(|&b| there[b].text == "{"));
            let Some((start, end)) = start.zip(open.and_then(|b| closing(&there, b))) else {
                let msg = format!(
                    "the model of `{}` has no `contract {name}`: `import store::Name;` brings \
                     a contract, `import store::block::{{…}};` records and promises",
                    store.text
                );
                return Err(Error::new(t.line, msg));
            };
            // A type of a block of that store which is not brought here
            // is any value: `results::Outcome` without its import.
            let block = |w: &&[Tok]| {
                let word =
                    w[0].kind == Kind::Name && ["subject", "boundary"].contains(&&*w[0].text);
                word && w[1].kind == Kind::Name && w[2].text == "{"
            };
            let blocks: BTreeSet<&str> = (there.windows(3).filter(block))
                .map(|w| w[1].text.as_str())
                .collect();
            let mut k = start;
            while k <= end {
                let a = &there[k];
                let path = a.kind == Kind::Name
                    && blocks.contains(a.text.as_str())
                    && there.get(k + 1).is_some_and(|c| c.text == "::")
                    && there.get(k + 2).is_some_and(|n| n.kind == Kind::Name);
                if path && !known.contains(&format!("{}::{}", a.text, there[k + 2].text)) {
                    held.push(Tok {
                        kind: Kind::Name,
                        text: "Any".into(),
                        line: t.line,
                    });
                    k += 3;
                    continue;
                }
                held.push(Tok {
                    line: t.line,
                    ..a.clone()
                });
                k += 1;
            }
            brought.insert(name, store.text.clone());
        }
        i = j + 1;
    }
    if last >= toks.len() {
        out.append(&mut held);
    }
    Ok((out, brought))
}
