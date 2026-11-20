//! The names of types in a text before it is read: which records and
//! enums a block declares, and each name put under its block.

use crate::reader::prelude::*;

/// One file, read twice: the second reading knows every function of it.
/// The records, enums and variants declared inside `subject name { … }`
/// or `boundary name { … }`, as `name::Record`, and those declared
/// outside any block.
pub(crate) fn types_of(toks: &[Tok]) -> (BTreeSet<String>, BTreeSet<String>) {
    let (mut scoped, mut top) = (BTreeSet::new(), BTreeSet::new());
    let (mut depth, mut scope): (usize, Option<String>) = (0, None);
    let mut i = 0;
    while i < toks.len() {
        let t = &toks[i];
        let next = toks.get(i + 1).filter(|n| n.kind == Kind::Name);
        let mut add = |name: &str, scope: &Option<String>| match scope {
            Some(s) => scoped.insert(format!("{s}::{name}")),
            None => top.insert(name.to_string()),
        };
        match (t.text.as_str(), &t.kind, next) {
            ("{", Kind::Punct, _) => depth += 1,
            ("}", Kind::Punct, _) => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    scope = None;
                }
            }
            ("subject" | "boundary", Kind::Name, Some(n)) if depth == 0 && n.text != "fn" => {
                scope = Some(n.text.clone())
            }
            ("record", Kind::Name, Some(n)) if depth <= 1 => {
                add(&n.text, &scope);
            }
            ("enum", Kind::Name, Some(n)) if depth <= 1 => {
                add(&n.text, &scope);
                // Its variants: each name at the first level of its braces.
                let mut j = i + 2;
                while j < toks.len() && toks[j].text != "{" && toks[j].text != ";" {
                    j += 1;
                }
                let (mut inner, mut first) = (0usize, true);
                while j < toks.len() {
                    match toks[j].text.as_str() {
                        "{" | "(" | "<" => inner += 1,
                        "}" | ")" | ">" => inner = inner.saturating_sub(1),
                        "," if inner == 1 => first = true,
                        _ if inner == 1 && first && toks[j].kind == Kind::Name => {
                            add(&toks[j].text, &scope);
                            first = false;
                        }
                        _ => {}
                    }
                    if inner == 0 {
                        break;
                    }
                    j += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    (scoped, top)
}

/// `a :: X` as one name, where `a::X` is a type or a variant of a block.
pub(crate) fn merge_types(toks: Vec<Tok>, scoped: &BTreeSet<String>) -> Vec<Tok> {
    let mut out: Vec<Tok> = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        let joined = match (toks.get(i + 1), toks.get(i + 2)) {
            (Some(c), Some(x)) if toks[i].kind == Kind::Name && c.text == "::" => {
                Some(format!("{}::{}", toks[i].text, x.text)).filter(|f| scoped.contains(f))
            }
            _ => None,
        };
        match joined {
            Some(text) => {
                out.push(Tok {
                    kind: Kind::Name,
                    text,
                    line: toks[i].line,
                });
                i += 3;
            }
            None => {
                out.push(toks[i].clone());
                i += 1;
            }
        }
    }
    out
}

/// Give each type and variant of a block its full name: inside the
/// block by its short name, elsewhere by its path, or by its short name
/// where one block has it and nothing outside any block does.
pub(crate) fn scope_types(
    toks: Vec<Tok>,
    scoped: &BTreeSet<String>,
    top: &BTreeSet<String>,
    evidence: bool,
) -> Vec<Tok> {
    if scoped.is_empty() {
        return toks;
    }
    let mut toks = merge_types(toks, scoped);
    let (mut depth, mut scope): (usize, Option<String>) = (0, None);
    for i in 0..toks.len() {
        let text = toks[i].text.clone();
        match text.as_str() {
            "{" if toks[i].kind == Kind::Punct => depth += 1,
            "}" if toks[i].kind == Kind::Punct => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    scope = None;
                }
            }
            "subject" | "boundary" if depth == 0 && toks[i].kind == Kind::Name => {
                let n = toks
                    .get(i + 1)
                    .filter(|n| n.kind == Kind::Name && n.text != "fn");
                scope = n.map(|n| n.text.clone());
            }
            // Evidence `for block::f`: short names are those of the block.
            "for" if evidence && toks.get(i + 2).is_some_and(|c| c.text == "::") => {
                scope = toks.get(i + 1).map(|n| n.text.clone());
            }
            _ => {}
        }
        if toks[i].kind != Kind::Name || text.contains("::") {
            continue;
        }
        let before = i.checked_sub(1).map(|j| toks[j].text.as_str());
        let after = toks.get(i + 1).map(|t| t.text.as_str());
        // A field read, the rest of a path, the head of one, a label.
        if matches!(before, Some("." | "::")) || matches!(after, Some("::" | ":")) {
            continue;
        }
        let own = scope.as_ref().map(|s| format!("{s}::{text}"));
        if let Some(full) = own.filter(|f| scoped.contains(f)) {
            toks[i].text = full;
            continue;
        }
        if top.contains(&text) {
            continue;
        }
        let ends = format!("::{text}");
        let mut with = scoped.iter().filter(|f| f.ends_with(&ends));
        if let (Some(full), None) = (with.next(), with.next()) {
            toks[i].text = full.clone();
        }
    }
    toks
}

/// The `>` that closes the `<` at `open`.
pub(crate) fn angle(toks: &[Tok], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (j, t) in toks.iter().enumerate().skip(open) {
        match (t.kind == Kind::Punct, t.text.as_str()) {
            (true, "<") => depth += 1,
            (true, ">") => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            (true, "{" | "}" | ";") => return None,
            _ => {}
        }
    }
    None
}

/// The names of the type parameters written between `<` and `>`:
/// `T, U: Link` gives `T` and `U`.
pub(crate) fn param_names(params: &[Tok]) -> Vec<String> {
    let (mut names, mut depth, mut first) = (Vec::new(), 0i32, true);
    for t in params {
        let punct = t.kind == Kind::Punct;
        match t.text.as_str() {
            "<" | "(" if punct => depth += 1,
            ">" | ")" if punct => depth -= 1,
            "," if punct && depth == 0 => {
                first = true;
                continue;
            }
            _ if first && t.kind == Kind::Name => names.push(t.text.clone()),
            _ => {}
        }
        first = false;
    }
    names
}

/// The types written between `<` and `>`, one list of tokens each.
pub(crate) fn type_args(args: &[Tok]) -> Vec<Vec<Tok>> {
    let (mut all, mut depth) = (vec![Vec::new()], 0i32);
    for t in args {
        let punct = t.kind == Kind::Punct;
        match t.text.as_str() {
            "<" | "(" if punct => depth += 1,
            ">" | ")" if punct => depth -= 1,
            "," if punct && depth == 0 => {
                all.push(Vec::new());
                continue;
            }
            _ => {}
        }
        all.last_mut().expect("one").push(t.clone());
    }
    all.retain(|a| !a.is_empty());
    all
}

/// A `fn` or a `claim`, from its keyword and name, given type
/// parameters: `fn f(…)` is `fn f<T>(…)`, `fn f<U>(…)` is `fn f<T, U>(…)`.
pub(crate) fn with_params(item: &[Tok], params: &[Tok]) -> Vec<Tok> {
    if params.is_empty() || item.len() < 2 {
        return item.to_vec();
    }
    let at = |text: &str| Tok {
        kind: Kind::Punct,
        text: text.into(),
        line: item[1].line,
    };
    let mut out = item[..2].to_vec();
    out.push(at("<"));
    out.extend(params.iter().cloned());
    match item.get(2).is_some_and(|t| t.text == "<") {
        true => {
            out.push(at(","));
            out.extend(item[3..].iter().cloned());
        }
        false => {
            out.push(at(">"));
            out.extend(item[2..].iter().cloned());
        }
    }
    out
}
