//! Where a function with type parameters was seen: the types its
//! parameters stood for in each piece of evidence on it. Evidence at an
//! instance is a point; this only says which instances have points.

use crate::ast::{Function, Model};
use crate::set::Value;
use std::collections::{BTreeMap, BTreeSet};

type Bound = BTreeMap<String, BTreeSet<String>>;

/// `text` cut at each `,` outside any bracket.
fn pieces(text: &str) -> Vec<String> {
    let (mut all, mut depth, mut now) = (Vec::new(), 0, String::new());
    for (i, c) in text.char_indices() {
        let arrow = c == '>' && text[..i].ends_with('-');
        match c {
            '(' | '<' | '[' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            '>' if !arrow => depth -= 1,
            _ => {}
        }
        match c == ',' && depth == 0 {
            true => all.push(std::mem::take(&mut now)),
            false => now.push(c),
        }
    }
    all.push(now);
    let trimmed = all.iter().map(|p| p.trim().to_string());
    trimmed.filter(|p| !p.is_empty()).collect()
}

/// What is between the bracket `open` at the start of `text` and the
/// one that closes it.
fn within(text: &str, open: char, close: char) -> Option<&str> {
    let mut depth = 0;
    for (i, c) in text.char_indices() {
        let arrow = c == '>' && text[..i].ends_with('-');
        if c == open {
            depth += 1;
        } else if c == close && !arrow {
            depth -= 1;
            if depth == 0 {
                return Some(&text[open.len_utf8()..i]);
            }
        }
    }
    None
}

/// The type of a value, as a model writes it.
fn type_of(m: &Model, v: &Value) -> String {
    let each = |vs: &[Value]| -> Vec<String> {
        let all: BTreeSet<String> = vs.iter().map(|v| type_of(m, v)).collect();
        all.into_iter().collect()
    };
    match v {
        Value::Bool(_) => "Bool".to_string(),
        Value::Int(_) => "Int".to_string(),
        Value::Float(_) => "Float64".to_string(),
        Value::Str(_) => "String".to_string(),
        Value::Tuple(vs) => {
            let parts: Vec<String> = vs.iter().map(|v| type_of(m, v)).collect();
            format!("({})", parts.join(", "))
        }
        Value::List(vs) if vs.is_empty() => "List<_>".to_string(),
        Value::List(vs) => format!("List<{}>", each(vs).join(" | ")),
        Value::Record(name, _) => {
            // A variant is of its enum.
            let of = m.records.iter().find(|r| &r.name == name);
            let of = of.and_then(|r| r.of.clone()).unwrap_or(name.clone());
            of.rsplit("::").next().unwrap_or(&of).to_string()
        }
    }
}

/// `ty` with each whole word of `from` written as the text beside it.
fn given(ty: &str, from: &[(String, String)]) -> String {
    let (mut out, mut word) = (String::new(), String::new());
    let flush = |word: &mut String, out: &mut String| {
        let to = from.iter().find(|f| &f.0 == word);
        *out += to.map_or(word.as_str(), |f| f.1.as_str());
        word.clear();
    };
    for c in ty.chars() {
        match c.is_alphanumeric() || c == '_' {
            true => word.push(c),
            false => {
                flush(&mut word, &mut out);
                out.push(c);
            }
        }
    }
    flush(&mut word, &mut out);
    out
}

/// What the type parameters `names` stand for, where a value of type
/// `ty` is `v`.
fn unify(m: &Model, names: &[String], ty: &str, v: &Value, into: &mut Bound) {
    let ty = ty.trim().trim_start_matches('&').trim();
    let ty = ty.strip_prefix("mut ").unwrap_or(ty).trim();
    if names.iter().any(|n| n == ty) {
        into.entry(ty.to_string())
            .or_default()
            .insert(type_of(m, v));
        return;
    }
    if let (Some(parts), Value::Tuple(vs)) =
        (within(ty, '(', ')').filter(|_| ty.starts_with('(')), v)
    {
        for (t, v) in pieces(parts).iter().zip(vs) {
            unify(m, names, t, v, into);
        }
        return;
    }
    let Some(at) = ty.find('<') else { return };
    let (head, args) = (ty[..at].trim(), within(&ty[at..], '<', '>').map(pieces));
    let (head, args) = (
        head.rsplit("::").next().unwrap_or(head),
        args.unwrap_or_default(),
    );
    match (head, v) {
        ("List" | "Set", Value::List(vs)) if args.len() == 1 => {
            for v in vs {
                unify(m, names, &args[0], v, into);
            }
        }
        ("Set", Value::Record(_, vs)) if args.len() == 1 => {
            for v in vs {
                unify(m, names, &format!("List<{}>", args[0]), v, into);
            }
        }
        (_, Value::Record(variant, vs)) => {
            // An enum with parameters: the variant's fields, each
            // parameter written as the type given for it.
            let last = |n: &str| n.rsplit("::").next().unwrap_or(n).to_string();
            let Some(e) = m.enums.iter().find(|e| last(&e.name) == head) else {
                return;
            };
            let from: Vec<(String, String)> = e.params.iter().cloned().zip(args).collect();
            let words: Vec<&str> = e.body.split_whitespace().collect();
            let body = crate::parse::tidy(&words.join(" "));
            for variant_text in pieces(&body) {
                let name = variant_text.split('(').next().unwrap_or("").trim();
                if last(name) != last(variant) {
                    continue;
                }
                let Some(fields) = variant_text
                    .find('(')
                    .and_then(|i| within(&variant_text[i..], '(', ')'))
                else {
                    continue;
                };
                for (field, v) in pieces(fields).iter().zip(vs) {
                    let ty = field.split_once(':').map_or(field.as_str(), |f| f.1);
                    unify(m, names, &given(ty, &from), v, into);
                }
            }
        }
        _ => {}
    }
}

/// The instances the evidence on `f` was observed at, each as
/// `T = Int` or `A = Int, B = String`. None for a function without
/// type parameters, or where an input does not say.
pub fn seen(m: &Model, f: &Function) -> Vec<String> {
    let Some(written) = f.written.as_deref().filter(|w| w.starts_with('<')) else {
        return Vec::new();
    };
    let Some(params) = within(written, '<', '>') else {
        return Vec::new();
    };
    let name = |p: &String| p.split(':').next().unwrap_or(p).trim().to_string();
    let names: Vec<String> = pieces(params).iter().map(name).collect();
    let rest = &written[params.len() + 2..];
    let Some(inputs) = within(rest.trim_start(), '(', ')') else {
        return Vec::new();
    };
    let types: Vec<String> = pieces(inputs)
        .iter()
        .map(|p| p.split_once(':').map_or(String::new(), |t| t.1.to_string()))
        .collect();
    let mut all = BTreeSet::new();
    for e in m.evidence.iter().filter(|e| e.about == f.name) {
        let values: Option<Vec<Value>> = (e.pattern.iter())
            .map(|p| p.set.as_ref().and_then(|s| s.single()))
            .collect();
        let Some(values) = values.filter(|v| v.len() == types.len()) else {
            continue;
        };
        let mut bound = Bound::new();
        for (t, v) in types.iter().zip(&values) {
            unify(m, &names, t, v, &mut bound);
        }
        let each: Vec<String> = names
            .iter()
            .filter_map(|n| {
                let types: Vec<&str> = bound.get(n)?.iter().map(String::as_str).collect();
                Some(format!("{n} = {}", types.join(" | ")))
            })
            .collect();
        if !each.is_empty() {
            all.insert(each.join(", "));
        }
    }
    all.into_iter().collect()
}
