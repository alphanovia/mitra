//! The shape of a tied record or enum against the Rust struct or enum
//! it is tied to (finding 19; the user's go, 2026-11-24 19:38).
//!
//! - Fields match by name, in any order. A field on one side only is
//!   R011, unless the record ends with `..`: then the struct may hold
//!   fields the model does not state.
//! - Types are checked, never converted: every value of the Rust field
//!   must be a value of the model's type. One that is not is R012.
//! - Variants of an enum match by name, and what each holds by order
//!   and type.
//! - A Rust form Mitra cannot write yet (a generic struct, a tuple
//!   struct, a variant with named fields, `f32`/`f64`, a map or a set)
//!   is R013.
//!
//! Each tie checked is reviewed by the reader's voice, `machine:mitra/
//! reader`: it holds when the shapes agree, and fails when they do not,
//! so blame can say the tie is wrong.

use mitra_lang::catalogue::{R011, R012, R013};
use mitra_lang::evidence::Review;
use mitra_lang::model::own;
use mitra_lang::model::{Model, Path, Tie, REST};
use mitra_lang::simulation::Type;
use mitra_lang::store::{finding, ReaderFinding};
use std::collections::{BTreeMap, BTreeSet};

/// The reader's voice, for the reviews of shapes.
pub const VOICE: &str = "machine:mitra/reader";

/// What the model declares that a tie may point at.
enum Declared<'m> {
    Record(&'m [(String, Type)]),
    Enum(&'m [(String, Vec<Type>)]),
}

/// A Rust struct or enum, as read.
#[derive(Debug, PartialEq)]
enum Shape {
    /// `struct S { a: T }`; `struct S;` has no field.
    Struct(Vec<(String, String)>),
    /// `enum E { A, B(T) }`: each variant and the types it holds.
    Enum(Vec<(String, Vec<String>)>),
}

/// Each tie of a record or an enum to a Rust struct or enum, checked:
/// a review by the reader's voice for each, and a finding for each way
/// the shapes differ. `files` are each Rust file's path and text.
pub fn shapes(
    model: &Model,
    ties: &[Tie],
    files: &[(String, String)],
) -> (Vec<Review>, Vec<ReaderFinding>) {
    let (mut reviews, mut findings) = (vec![], vec![]);
    let names = rust_names(model, ties);
    let mut seen = BTreeSet::new();
    for tie in ties {
        let Some(declared) = declared(model, &tie.item) else {
            continue;
        };
        let Some(k) = tie.at.path.iter().position(|p| p.ends_with(".rs")) else {
            continue;
        };
        let file = tie.at.path[..=k].join("/");
        let Some((_, source)) = files.iter().find(|(f, _)| *f == file) else {
            continue;
        };
        let Some(name) = tie.at.path.get(k + 1..).and_then(|n| n.last()) else {
            continue;
        };
        if !seen.insert((tie.item.clone(), tie.at.clone())) {
            continue;
        }
        let item = tie.item.to_string();
        let before = findings.len();
        match read_shape(source, name) {
            Err(Missing::None) => {
                // A part tied to an `impl` or a `mod` ties its functions.
                if tie.item.names.len() == 1 {
                    continue;
                }
                let message = format!(
                    "`{item}` is tied to `{}`, and `{file}` declares no struct or enum `{name}`",
                    tie.at.path.join("/")
                );
                findings.push(finding(R011, &file, 0, message));
            }
            Err(Missing::Form(line, why)) => findings.push(finding(R013, &file, line, why)),
            Ok((line, shape)) => {
                let mut said =
                    |code, message: String| findings.push(finding(code, &file, line, message));
                compare(&item, name, &declared, &shape, &names, &mut said);
            }
        }
        // A form Mitra cannot write says nothing of the tie either way.
        let new = &findings[before..];
        if new.iter().any(|f| f.code == R013) {
            continue;
        }
        reviews.push(Review {
            prop: tie.item.clone(),
            from: None,
            holds: new.is_empty(),
            by: VOICE.to_string(),
            seeing: vec![],
        });
    }
    // One voice reviews an item once: where two ties of one item were
    // checked, it holds only if both do.
    let mut by_item: BTreeMap<Path, Review> = BTreeMap::new();
    for r in reviews {
        by_item
            .entry(r.prop.clone())
            .and_modify(|first| first.holds &= r.holds)
            .or_insert(r);
    }
    (by_item.into_values().collect(), findings)
}

/// The record or enum `item` names: `[P]`, the part's own record or
/// enum; `[P, R]`, the record or enum `R` of part `P`.
fn declared<'m>(model: &'m Model, item: &Path) -> Option<Declared<'m>> {
    let (part, name) = match &item.names[..] {
        [p] => (p, p),
        [p, n] => (p, n),
        _ => return None,
    };
    let part = model.parts.iter().find(|p| p.name == *part)?;
    // A nested data part's record is named by the part's whole path, and
    // tied at its own name: `agree::Settings::Settings`.
    let named = |n: &str| n == name.as_str() || own(n) == name.as_str();
    if let Some(r) = part.records.iter().find(|r| named(&r.name)) {
        return r.fields.as_deref().map(Declared::Record);
    }
    part.enums
        .iter()
        .find(|e| named(&e.name))
        .map(|e| Declared::Enum(&e.variants))
}

/// For each record and enum of the model, the Rust names it is tied to
/// (the last name of each tie's place), and its own.
fn rust_names(model: &Model, ties: &[Tie]) -> BTreeMap<String, BTreeSet<String>> {
    let mut names: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for part in &model.parts {
        for r in part
            .records
            .iter()
            .map(|r| &r.name)
            .chain(part.enums.iter().map(|e| &e.name))
        {
            names.entry(r.clone()).or_default().insert(r.clone());
        }
    }
    for tie in ties {
        // The record a tie names: its part's own, by the part's whole path.
        let name = match &tie.item.names[..] {
            [p, n] if own(p) == n => p,
            names => match names.last() {
                Some(n) => n,
                None => continue,
            },
        };
        if let Some(rust) = tie.at.path.last() {
            names.entry(name.clone()).or_default().insert(rust.clone());
        }
    }
    names
}

enum Missing {
    None,
    Form(usize, String),
}

/// The struct or enum `name` of `source`, its line (from 1) and shape.
fn read_shape(source: &str, name: &str) -> Result<(usize, Shape), Missing> {
    let text = without_comments(source);
    let mut found = vec![];
    for (i, line) in text.lines().enumerate() {
        let words: Vec<&str> = line
            .split(|c: char| !(c.is_alphanumeric() || c == '_'))
            .filter(|w| !w.is_empty())
            .collect();
        let at = words
            .windows(2)
            .position(|w| (w[0] == "struct" || w[0] == "enum") && w[1] == name);
        if let Some(at) = at {
            let before_pub = words[..at]
                .iter()
                .all(|w| ["pub", "crate", "super", "in", "self"].contains(w));
            if before_pub {
                found.push((i, words[at] == "struct"));
            }
        }
    }
    let [(i, is_struct)] = found[..] else {
        return match found.len() {
            0 => Err(Missing::None),
            n => Err(Missing::Form(
                found[0].0 + 1,
                format!("{n} structs or enums are named `{name}` in this file: Mitra cannot tell which the tie names"),
            )),
        };
    };
    let line = i + 1;
    let rest: String = text.lines().skip(i).collect::<Vec<_>>().join("\n");
    let start = rest.find(name).map(|k| k + name.len()).unwrap_or(0);
    let after = rest[start..].trim_start();
    let kind = if is_struct { "struct" } else { "enum" };
    // Lifetimes only, `struct Text<'a>`: no type parameter.
    let after = match after
        .strip_prefix('<')
        .and_then(|a| inside(a, '<', '>').map(|p| (a, p)))
    {
        Some((a, params)) if split_top(&params).iter().all(|p| p.starts_with('\'')) => {
            a[params.len() + 1..].trim_start()
        }
        _ => after,
    };
    if after.starts_with('<') {
        return Err(Missing::Form(
            line,
            format!(
                "{kind} `{name}` has type parameters: Mitra has no form for a generic {kind} yet"
            ),
        ));
    }
    if after.starts_with(';') {
        return Ok((line, Shape::Struct(vec![])));
    }
    if after.starts_with('(') {
        return Err(Missing::Form(
            line,
            format!("struct `{name}` is a tuple struct: Mitra has no form for one yet; give its fields names"),
        ));
    }
    let after = after.strip_prefix("where").map_or(after, |_| "");
    let Some(body) = after.strip_prefix('{').and_then(|b| inside(b, '{', '}')) else {
        return Err(Missing::Form(
            line,
            format!("{kind} `{name}`: Mitra cannot read its body"),
        ));
    };
    let body = without_attributes(&body);
    if is_struct {
        let mut fields = vec![];
        for field in split_top(&body) {
            let field = without_visibility(&field);
            let Some((n, t)) = field.split_once(':') else {
                return Err(Missing::Form(
                    line,
                    format!("struct `{name}`: `{field}` is no field Mitra can read"),
                ));
            };
            fields.push((n.trim().to_string(), t.trim().to_string()));
        }
        return Ok((line, Shape::Struct(fields)));
    }
    let mut variants = vec![];
    for variant in split_top(&body) {
        let variant = variant.split('=').next().unwrap_or("").trim().to_string();
        if let Some(k) = variant.find('{') {
            let v = variant[..k].trim();
            return Err(Missing::Form(
                line,
                format!("variant `{v}` of enum `{name}` has named fields: Mitra has no form for one yet; it holds values in order, `{v}(T, U)`"),
            ));
        }
        match variant.split_once('(') {
            Some((v, holds)) => {
                let holds = inside(holds, '(', ')').unwrap_or_default();
                variants.push((v.trim().to_string(), split_top(&holds)));
            }
            None => variants.push((variant, vec![])),
        }
    }
    Ok((line, Shape::Enum(variants)))
}

/// Compare what the model declares with what the code has; `said`
/// takes each difference.
fn compare(
    item: &str,
    rust: &str,
    declared: &Declared,
    shape: &Shape,
    names: &BTreeMap<String, BTreeSet<String>>,
    said: &mut impl FnMut(mitra_lang::catalogue::Code, String),
) {
    match (declared, shape) {
        (Declared::Record(fields), Shape::Struct(code)) => {
            let open = fields.last().is_some_and(|(n, _)| n == REST);
            for (n, t) in fields.iter().filter(|(n, _)| n != REST) {
                match code.iter().find(|(c, _)| c == n) {
                    None => said(
                        R011,
                        format!("field `{n}` of the model's `{item}` is not in struct `{rust}`"),
                    ),
                    Some((_, rt)) => check(
                        &format!("field `{n}` of struct `{rust}`"),
                        rt,
                        t,
                        names,
                        said,
                    ),
                }
            }
            if !open {
                for (c, _) in code
                    .iter()
                    .filter(|(c, _)| !fields.iter().any(|(n, _)| n == c))
                {
                    // `by`, `from`: a field no model can name yet (finding 12c).
                    if mitra_lang::parse::KEYWORDS.contains(&c.as_str()) {
                        said(
                            R013,
                            format!("field `{c}` of struct `{rust}` is named by a keyword of Mitra, so no field of the model can be named so yet: end the record with `..`, the fields the model does not state"),
                        );
                        continue;
                    }
                    said(
                        R011,
                        format!("field `{c}` of struct `{rust}` is not in the model's `{item}`: model it, or end the record with `..`, the fields the model does not state"),
                    );
                }
            }
        }
        (Declared::Enum(variants), Shape::Enum(code)) => {
            for (v, holds) in variants.iter() {
                match code.iter().find(|(c, _)| c == v) {
                    None => said(R011, format!("variant `{v}` of the model's `{item}` is not in enum `{rust}`")),
                    Some((_, rust_holds)) if rust_holds.len() != holds.len() => said(
                        R012,
                        format!("variant `{v}` holds {} values in the model's `{item}`, and {} in enum `{rust}`", holds.len(), rust_holds.len()),
                    ),
                    Some((_, rust_holds)) => {
                        for (i, (rt, t)) in rust_holds.iter().zip(holds).enumerate() {
                            check(&format!("value {i} of variant `{v}` of enum `{rust}`"), rt, t, names, said);
                        }
                    }
                }
            }
            for (c, _) in code
                .iter()
                .filter(|(c, _)| !variants.iter().any(|(v, _)| v == c))
            {
                said(
                    R011,
                    format!("variant `{c}` of enum `{rust}` is not in the model's `{item}`"),
                );
            }
        }
        (Declared::Record(_), Shape::Enum(_)) => said(
            R012,
            format!("the model's `{item}` is a record, and `{rust}` an enum"),
        ),
        (Declared::Enum(_), Shape::Struct(_)) => said(
            R012,
            format!("the model's `{item}` is an enum, and `{rust}` a struct"),
        ),
    }
}

/// `rust`, a Rust type as written, holds only values of `t`: else R012,
/// or R013 for a form Mitra has none of.
fn check(
    what: &str,
    rust: &str,
    t: &Type,
    names: &BTreeMap<String, BTreeSet<String>>,
    said: &mut impl FnMut(mitra_lang::catalogue::Code, String),
) {
    if let Err((code, why)) = fits(rust, t, names) {
        said(code, format!("{what} is `{rust}`: {why}"));
    }
}

type Misfit = (mitra_lang::catalogue::Code, String);

fn fits(rust: &str, t: &Type, names: &BTreeMap<String, BTreeSet<String>>) -> Result<(), Misfit> {
    let rust = rust.trim();
    let shown = mitra_lang::print::type_(t);
    let wrong = || Err((R012, format!("not every value of it is of `{shown}`")));
    // `&T`, `&'a mut T`: the value is the `T`.
    if let Some(r) = rust.strip_prefix('&') {
        let r = r.trim_start();
        let r = match r.strip_prefix('\'') {
            Some(l) => l.split_once(' ').map_or("", |(_, r)| r),
            None => r,
        };
        let r = r.strip_prefix("mut ").unwrap_or(r);
        return fits(r, t, names);
    }
    if let Some(inner) = rust.strip_prefix('(') {
        let holds = split_top(&inside(inner, '(', ')').unwrap_or_default());
        let Type::Record(r) = t else { return wrong() };
        if !r.name.is_empty() || r.fields.len() != holds.len() {
            return wrong();
        }
        for (h, (_, t)) in holds.iter().zip(&r.fields) {
            fits(h, t, names)?;
        }
        return Ok(());
    }
    if let Some(inner) = rust.strip_prefix('[') {
        let inner = inside(inner, '[', ']').unwrap_or_default();
        let element = split_semicolon(&inner);
        return match t {
            Type::Seq(e) => fits(&element, e, names),
            _ => wrong(),
        };
    }
    let (head, args) = match rust.find('<') {
        Some(k) => (
            &rust[..k],
            split_top(&inside(&rust[k + 1..], '<', '>').unwrap_or_default()),
        ),
        None => (rust, vec![]),
    };
    let head = head.rsplit("::").next().unwrap_or(head).trim();
    let args: Vec<String> = args.into_iter().filter(|a| !a.starts_with('\'')).collect();
    let arg = |i: usize| args.get(i).cloned().unwrap_or_default();
    let ints = [
        "i8", "i16", "i32", "i64", "i128", "isize", "u8", "u16", "u32", "u64", "u128", "usize",
        "BigInt", "BigUint",
    ];
    match (head, t) {
        ("Box" | "Rc" | "Arc", _) => fits(&arg(0), t, names),
        (h, _)
            if h.starts_with("dyn ")
                || h.starts_with("impl ")
                || h.starts_with("fn(")
                || h.starts_with("fn (") =>
        {
            Err((
                R013,
                "Mitra has no form for a function or a trait object yet".to_string(),
            ))
        }
        ("f32" | "f64", _) => Err((
            R013,
            "Mitra has no form for a float yet: `rational` is exact, and nothing converts"
                .to_string(),
        )),
        ("HashMap" | "BTreeMap" | "HashSet" | "BTreeSet" | "IndexMap" | "IndexSet", _) => Err((
            R013,
            "Mitra has no form for a map or a set yet: `[T]` keeps order and repeats".to_string(),
        )),
        ("bool", Type::Truth) | ("char", Type::Char) | ("u8", Type::Byte) => Ok(()),
        (h, Type::Int) if ints.contains(&h) => Ok(()),
        ("Rational" | "BigRational" | "Ratio", Type::Real) => Ok(()),
        ("String" | "str", Type::Encoded(e)) if **e == Type::Char => Ok(()),
        ("Cow", Type::Encoded(e)) if **e == Type::Char && arg(0) == "str" => Ok(()),
        ("Vec" | "VecDeque", Type::Seq(e)) => fits(&arg(0), e, names),
        // What `Some`, `Ok` and `Err` hold, as the model's variants say.
        ("Option", Type::Enum(e)) if e.name == "Option" => match held(e, "Some") {
            Some(a) => fits(&arg(0), a, names),
            None => wrong(),
        },
        ("Result", Type::Enum(e)) if e.name == "Result" => match (held(e, "Ok"), held(e, "Err")) {
            (Some(ok), Some(err)) => {
                fits(&arg(0), ok, names)?;
                fits(&arg(1), err, names)
            }
            _ => wrong(),
        },
        (h, Type::Record(r)) if names.get(&r.name).is_some_and(|n| n.contains(h)) => Ok(()),
        (h, Type::Enum(e)) if names.get(&e.name).is_some_and(|n| n.contains(h)) => Ok(()),
        (_, Type::Param { .. }) => Err((
            R013,
            "Mitra has no form for a generic record's field yet".to_string(),
        )),
        _ => wrong(),
    }
}

/// The one value variant `v` of `e` holds.
fn held<'e>(e: &'e mitra_lang::simulation::Enum, v: &str) -> Option<&'e Type> {
    match e.variants.iter().find(|(n, _)| n == v) {
        Some((_, holds)) if holds.len() == 1 => holds.first(),
        _ => None,
    }
}

/// `source` with its `//` comments blanked, lines kept.
fn without_comments(source: &str) -> String {
    // `/* … */`, its lines kept.
    let mut plain = String::new();
    let mut rest = source;
    while let Some(k) = rest.find("/*") {
        plain.push_str(&rest[..k]);
        let end = rest[k..].find("*/").map_or(rest.len(), |e| k + e + 2);
        plain.extend(rest[k..end].chars().filter(|c| *c == '\n'));
        rest = &rest[end..];
    }
    plain.push_str(rest);
    plain
        .lines()
        .map(|l| match l.find("//") {
            Some(k) if !l[..k].contains('"') => &l[..k],
            _ => l,
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// `#[…]` attributes taken out.
fn without_attributes(s: &str) -> String {
    let mut out = String::new();
    let mut rest = s;
    while let Some(k) = rest.find("#[") {
        out.push_str(&rest[..k]);
        let after = &rest[k + 2..];
        let end = inside(after, '[', ']').map_or(after.len(), |i| i.len() + 1);
        rest = &after[end.min(after.len())..];
    }
    out.push_str(rest);
    out
}

/// `pub`, `pub(crate)`, `pub(in a::b)` taken off a field.
fn without_visibility(field: &str) -> String {
    let field = field.trim();
    match field.strip_prefix("pub") {
        Some(r) if r.starts_with('(') => r.split_once(')').map_or(r, |(_, r)| r).trim().to_string(),
        Some(r) if r.starts_with(char::is_whitespace) => r.trim().to_string(),
        _ => field.to_string(),
    }
}

/// What lies between an opened `open` and its matching `close`.
fn inside(s: &str, open: char, close: char) -> Option<String> {
    let mut depth = 1;
    for (k, c) in s.char_indices() {
        if c == open {
            depth += 1;
        } else if c == close {
            depth -= 1;
            if depth == 0 {
                return Some(s[..k].to_string());
            }
        }
    }
    None
}

/// `s` split at its commas outside `<>`, `()`, `[]` and `{}`, each
/// trimmed, empty ones dropped.
fn split_top(s: &str) -> Vec<String> {
    let (mut out, mut depth, mut start) = (vec![], 0i32, 0);
    let mut prev = ' ';
    for (k, c) in s.char_indices() {
        match c {
            '<' | '(' | '[' | '{' => depth += 1,
            '>' if prev != '-' => depth -= 1,
            ')' | ']' | '}' => depth -= 1,
            ',' if depth == 0 => {
                out.push(s[start..k].trim().to_string());
                start = k + 1;
            }
            _ => {}
        }
        prev = c;
    }
    out.push(s[start..].trim().to_string());
    out.into_iter().filter(|x| !x.is_empty()).collect()
}

/// `T; N` is `T`; `T` is `T`.
fn split_semicolon(s: &str) -> String {
    let mut depth = 0i32;
    for (k, c) in s.char_indices() {
        match c {
            '<' | '(' | '[' => depth += 1,
            '>' | ')' | ']' => depth -= 1,
            ';' if depth == 0 => return s[..k].trim().to_string(),
            _ => {}
        }
    }
    s.trim().to_string()
}
