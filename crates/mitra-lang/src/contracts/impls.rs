//! Reading `impl`: the methods of a record, and a contract filled for a
//! type.

use crate::reader::prelude::*;

/// `impl R { … }` as Rust writes it, read as what it stands for: a `fn`
/// or a `claim` named `R::name`, `self` (also `&self`, `&mut self`) an
/// input `self: R`, and `Self` the record. `impl<T> R<T> { … }` gives
/// each function the parameter, and `self` is an `R<T>`. Also gives the
/// methods, those that take `self`, by their full name.
pub(crate) fn impls(toks: Vec<Tok>) -> Result<(Vec<Tok>, BTreeSet<String>), Error> {
    // Inside an `impl`: its record, the depth of its braces, its type
    // parameters as written and by name, and the record's own `<…>`.
    struct Of {
        record: String,
        depth: usize,
        params: Vec<Tok>,
        names: Vec<String>,
        args: Vec<Tok>,
    }
    let (mut out, mut methods) = (Vec::new(), BTreeSet::new());
    let (mut depth, mut scope): (usize, Option<String>) = (0, None);
    let mut of: Option<Of> = None;
    // In a claim of an `impl<T>`, up to this token: `T` is any value.
    let mut plain = 0;
    let mut i = 0;
    let name = |t: Option<&Tok>| t.filter(|t| t.kind == Kind::Name).map(|t| t.text.clone());
    let any = |t: &Tok| Tok {
        kind: Kind::Name,
        text: "Any".into(),
        line: t.line,
    };
    while i < toks.len() {
        let t = &toks[i];
        let word = t.kind == Kind::Name;
        let punct = t.kind == Kind::Punct;
        match (t.text.as_str(), &of) {
            ("subject" | "boundary", None) if word && depth == 0 => {
                scope = name(toks.get(i + 1)).filter(|n| n != "fn");
            }
            ("impl", None) if word && depth <= 1 => {
                let form = "an `impl` is `impl R { … }`, `impl<T> R<T> { … }` or \
                            `impl Contract for R { … }`";
                let is = |j: usize, w: &str| toks.get(j).is_some_and(|t| t.text == w);
                let mut j = i + 1;
                let mut params = Vec::new();
                if is(j, "<") {
                    let end = angle(&toks, j).ok_or(Error::new(t.line, form))?;
                    params = toks[j + 1..end].to_vec();
                    j = end + 1;
                }
                let Some(record) = name(toks.get(j)) else {
                    return Err(Error::new(t.line, "expected a record after `impl`"));
                };
                j += 1;
                let mut args = Vec::new();
                if is(j, "<") {
                    let end = angle(&toks, j).ok_or(Error::new(t.line, form))?;
                    args = toks[j..=end].to_vec();
                    j = end + 1;
                }
                if !is(j, "{") {
                    return Err(Error::new(t.line, form));
                }
                let names = param_names(&params);
                of = Some(Of {
                    record,
                    depth,
                    params,
                    names,
                    args,
                });
                depth += 1;
                i = j + 1;
                continue;
            }
            ("impl", Some(_)) if word => {
                return Err(Error::new(t.line, "an `impl` inside another"));
            }
            ("{", _) if punct => depth += 1,
            ("}", _) if punct => {
                depth = depth.saturating_sub(1);
                if of.as_ref().is_some_and(|o| o.depth == depth) {
                    of = None;
                    i += 1;
                    continue;
                }
                if depth == 0 {
                    scope = None;
                }
            }
            ("Self", Some(o)) if word => {
                out.push(Tok {
                    text: o.record.clone(),
                    ..t.clone()
                });
                for a in &o.args {
                    let open = i < plain && a.kind == Kind::Name && o.names.contains(&a.text);
                    out.push(if open { any(a) } else { a.clone() });
                }
                i += 1;
                continue;
            }
            (p, Some(o)) if word && i < plain && o.names.iter().any(|n| n == p) => {
                out.push(any(t));
                i += 1;
                continue;
            }
            ("fn" | "claim" | "assume" | "ensure" | "require", Some(o))
                if word && depth == o.depth + 1 =>
            {
                let Some(n) = name(toks.get(i + 1)) else {
                    out.push(t.clone());
                    i += 1;
                    continue;
                };
                let (r, func) = (&o.record, t.text == "fn");
                if !func && !o.names.is_empty() {
                    // To its `;`, or to the `}` of its braces.
                    let mut end = i;
                    while toks
                        .get(end)
                        .is_some_and(|e| e.text != ";" && e.text != "{")
                    {
                        end += 1;
                    }
                    if toks.get(end).is_some_and(|e| e.text == "{") {
                        end = closing(&toks, end).unwrap_or(end);
                    }
                    plain = end;
                }
                let full = format!("{r}::{n}");
                out.push(t.clone());
                out.push(Tok {
                    text: full.clone(),
                    ..toks[i + 1].clone()
                });
                i += 2;
                // Its own `<U>` after the parameters of the `impl`.
                if func && !o.params.is_empty() {
                    let at = |text: &str| Tok {
                        kind: Kind::Punct,
                        text: text.into(),
                        line: t.line,
                    };
                    out.push(at("<"));
                    out.extend(o.params.iter().cloned());
                    let own = toks
                        .get(i)
                        .filter(|t| t.text == "<")
                        .map(|_| angle(&toks, i));
                    match own {
                        Some(Some(end)) => {
                            out.push(at(","));
                            out.extend(toks[i + 1..=end].iter().cloned());
                            i = end + 1;
                        }
                        _ => out.push(at(">")),
                    }
                } else if func && toks.get(i).is_some_and(|t| t.text == "<") {
                    if let Some(end) = angle(&toks, i) {
                        out.extend(toks[i..=end].iter().cloned());
                        i = end + 1;
                    }
                }
                // `(self`, `(&self`, `(&mut self`: an input `self: R`.
                let mut j = i + 1;
                if toks.get(i).is_some_and(|o| o.text == "(") {
                    j += usize::from(toks.get(j).is_some_and(|a| a.text == "&"));
                    j += usize::from(toks.get(j).is_some_and(|m| m.text == "mut"));
                    let typed = toks.get(j + 1).is_some_and(|c| c.text == ":");
                    if toks.get(j).is_some_and(|s| s.text == "self") && !typed {
                        let at = toks[j].clone();
                        out.push(toks[i].clone());
                        out.push(at.clone());
                        out.push(Tok {
                            kind: Kind::Punct,
                            text: ":".into(),
                            line: at.line,
                        });
                        out.push(Tok {
                            text: r.clone(),
                            ..at
                        });
                        for a in &o.args {
                            let open = !func && a.kind == Kind::Name && o.names.contains(&a.text);
                            out.push(if open { any(a) } else { a.clone() });
                        }
                        i = j + 1;
                        if func {
                            methods.insert(match &scope {
                                Some(s) => format!("{s}::{full}"),
                                None => full,
                            });
                        }
                    }
                }
                continue;
            }
            _ => {}
        }
        out.push(t.clone());
        i += 1;
    }
    Ok((out, methods))
}

/// The tokens from `open`, a `{`, to the `}` that closes it: the index
/// of that `}`.
pub(crate) fn closing(toks: &[Tok], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (j, t) in toks.iter().enumerate().skip(open) {
        match (t.kind == Kind::Punct, t.text.as_str()) {
            (true, "{") => depth += 1,
            (true, "}") => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
    }
    None
}

/// `x: impl C`, also `impl C<A>` and `impl C + D`, as the type of an
/// input: any value. What it names is kept beside, with the block it is
/// written in.
pub(crate) fn impl_types(toks: Vec<Tok>, c: &mut Contracts) -> Result<Vec<Tok>, Error> {
    let word = |t: &Tok, w: &str| t.kind == Kind::Name && t.text == w;
    let mut out: Vec<Tok> = Vec::new();
    let (mut depth, mut scope): (usize, Option<String>) = (0, None);
    // The last `fn` read: its name and the line of it.
    let mut func: Option<(String, usize)> = None;
    let mut i = 0;
    while i < toks.len() {
        let t = &toks[i];
        let next = toks.get(i + 1).filter(|n| n.kind == Kind::Name);
        if t.kind == Kind::Punct && t.text == "{" {
            depth += 1;
        } else if t.kind == Kind::Punct && t.text == "}" {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                scope = None;
            }
        } else if depth == 0
            && ["subject", "boundary", "contract"]
                .iter()
                .any(|w| word(t, w))
        {
            scope = next.map(|n| n.text.clone()).filter(|n| n != "fn");
        } else if let (true, Some(n)) = (["fn", "claim", "assume"].iter().any(|w| word(t, w)), next)
        {
            func = Some((n.text.clone(), n.line));
        } else if let (true, Some(n)) =
            (word(t, "impl"), next.filter(|n| c.names.contains(&n.text)))
        {
            // After `:` (and `&`, `&mut`) it is a type; before `for` or
            // `{` it is an implementer.
            let mut b = i;
            while b > 0 && ["&", "mut"].contains(&toks[b - 1].text.as_str()) {
                b -= 1;
            }
            let typed = b >= 2 && toks[b - 1].text == ":" && toks[b - 2].kind == Kind::Name;
            if !typed {
                if toks.get(i + 2).is_some_and(|b| b.text == "{") {
                    let msg = format!(
                        "`{}` is a contract: a type fills it with `impl {0} for Type {{ … }}`",
                        n.text
                    );
                    return Err(Error::new(t.line, msg));
                }
                out.push(t.clone());
                i += 1;
                continue;
            }
            let (mut all, mut j) = (Vec::new(), i + 1);
            loop {
                let Some(name) = toks.get(j).filter(|n| n.kind == Kind::Name) else {
                    return Err(Error::new(t.line, "expected a contract after `+`"));
                };
                if !c.names.contains(&name.text) {
                    let msg = format!(
                        "`{}` is not a contract: a bound names a contract",
                        name.text
                    );
                    return Err(Error::new(name.line, msg));
                }
                all.push(name.text.clone());
                j += 1;
                if toks.get(j).is_some_and(|a| a.text == "<") {
                    let end = angle(&toks, j);
                    j = end.ok_or(Error::new(t.line, "this `<` is not closed"))? + 1;
                }
                if !toks.get(j).is_some_and(|p| p.text == "+") {
                    break;
                }
                j += 1;
            }
            if let Some(block) = &scope {
                for name in &all {
                    c.leans.push((block.clone(), name.clone(), t.line));
                }
            }
            if let Some((f, line)) = &func {
                c.sugar
                    .push((*line, f.clone(), toks[b - 2].text.clone(), all));
            }
            out.push(Tok {
                kind: Kind::Name,
                text: "Any".into(),
                line: t.line,
            });
            i = j;
            continue;
        }
        out.push(t.clone());
        i += 1;
    }
    Ok(out)
}

/// `item` on `line`, each type parameter of `wanted` written as the
/// type of `args` given for it.
pub(crate) fn given_types(
    item: &[Tok],
    wanted: &[String],
    args: &[Vec<Tok>],
    line: usize,
    into: &mut Vec<Tok>,
) {
    for f in item {
        let n = wanted
            .iter()
            .position(|w| f.kind == Kind::Name && w == &f.text);
        let each: &[Tok] = match n {
            Some(n) => &args[n],
            None => std::slice::from_ref(f),
        };
        into.extend(each.iter().map(|f| Tok { line, ..f.clone() }));
    }
}
