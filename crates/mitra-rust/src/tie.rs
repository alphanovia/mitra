//! `mitra tie`: the reader of Rust code. A comment `// mitra: implements
//! subject::name` above a `fn` ties it to a function of the model. The
//! reader compares the two (inputs, result, calls) and says what it found
//! as plain evidence: the core never depends on it, and a reviewer can
//! bring the same evidence by hand.

use std::path::Path;

use crate::source::{fn_below, mitra_line, rust_files, scan};
use mitra_lang::{Model, Target};

/// A `fn` of the code, as far as text tells.
struct Code {
    /// The function of the model it says it implements.
    implements: String,
    place: String,
    name: String,
    /// Each parameter with its type; `self` has none.
    params: Vec<(String, Option<String>)>,
    result: String,
    body: String,
    /// The free functions its file defines: a call by one of these names
    /// is a call to the one beside it.
    beside: Vec<String>,
}

/// A `struct` or an `enum` of the code tied to a record of the model.
struct Shape {
    implements: String,
    place: String,
    /// `struct` or `enum`.
    kind: String,
    /// Each named field with its type; none for an enum, a tuple struct
    /// or a unit struct.
    fields: Option<Vec<(String, String)>>,
    /// For an enum: each variant with the types it holds, in order.
    variants: Option<Vec<(String, Vec<String>)>>,
}

/// The variants of an enum, from the text after its name: `A`, `A(T, U)`
/// and `A { x: T }`, each with the types it holds.
fn variants(after: &str) -> Option<Vec<(String, Vec<String>)>> {
    let open = after.find('{')?;
    let inside = balanced(&after[open + 1..], '{', '}')?;
    let mut out = Vec::new();
    for p in pieces(inside) {
        let mut p = p.trim();
        while let Some(r) = p.strip_prefix('#') {
            p = r[r.find(']')? + 1..].trim_start();
        }
        let end = p.find(|c: char| !(c.is_alphanumeric() || c == '_'));
        let (name, rest) = p.split_at(end.unwrap_or(p.len()));
        let rest = rest.trim_start();
        let held = if let Some(r) = rest.strip_prefix('(') {
            pieces(balanced(r, '(', ')')?)
        } else if let Some(r) = rest.strip_prefix('{') {
            let named = pieces(balanced(r, '{', '}')?);
            let ty = |f: &String| f.split_once(':').map(|t| t.1.trim().to_string());
            named.iter().map(ty).collect::<Option<Vec<String>>>()?
        } else {
            Vec::new()
        };
        out.push((name.to_string(), held));
    }
    Some(out)
}

/// The `struct` or `enum` at the start of `code`, past attributes and
/// `pub`: its kind, and the text after its name.
fn shape_below(code: &str) -> Option<(&str, &str)> {
    let mut rest = code.trim_start();
    loop {
        if let Some(r) = rest.strip_prefix('#') {
            rest = r[r.find(']')? + 1..].trim_start();
        } else if let Some(r) = rest.strip_prefix("pub(") {
            rest = r[r.find(')')? + 1..].trim_start();
        } else if let Some(r) = rest.strip_prefix("pub ") {
            rest = r.trim_start();
        } else {
            break;
        }
    }
    for kind in ["struct", "enum"] {
        if let Some(r) = rest.strip_prefix(kind).filter(|r| r.starts_with(' ')) {
            let r = r.trim_start();
            let end = r.find(|c: char| !(c.is_alphanumeric() || c == '_'));
            return Some((kind, &r[end.unwrap_or(r.len())..]));
        }
    }
    None
}

/// The named fields of a struct, from the text after its name.
fn fields(after: &str) -> Option<Vec<(String, String)>> {
    let open = after.find(['{', ';', '('])?;
    let inside = balanced(after[open..].strip_prefix('{')?, '{', '}')?;
    let mut out = Vec::new();
    for p in pieces(inside) {
        let mut p = p.trim();
        while let Some(r) = p.strip_prefix('#') {
            p = r[r.find(']')? + 1..].trim_start();
        }
        if let Some(r) = p.strip_prefix("pub(") {
            p = r[r.find(')')? + 1..].trim_start();
        }
        let p = p.strip_prefix("pub ").unwrap_or(p);
        let (name, ty) = p.split_once(':')?;
        out.push((name.trim().to_string(), ty.trim().to_string()));
    }
    Some(out)
}

/// The piece of `s` up to the bracket that closes the one opened before it.
fn balanced(s: &str, open: char, close: char) -> Option<&str> {
    let mut depth = 1;
    for (i, c) in s.char_indices() {
        // `->` closes nothing.
        let arrow = c == '>' && s[..i].ends_with('-');
        depth += (c == open) as i32;
        depth -= (c == close && !arrow) as i32;
        if depth == 0 {
            return Some(&s[..i]);
        }
    }
    None
}

/// The pieces of `s` between commas outside any bracket.
fn pieces(s: &str) -> Vec<String> {
    let (mut out, mut depth, mut now) = (Vec::new(), 0, String::new());
    for (i, c) in s.char_indices() {
        let arrow = c == '>' && s[..i].ends_with('-');
        match c {
            '(' | '[' | '<' | '{' => depth += 1,
            ')' | ']' | '}' => depth -= 1,
            '>' if !arrow => depth -= 1,
            _ => {}
        }
        if c == ',' && depth == 0 {
            out.push(std::mem::take(&mut now));
        } else {
            now.push(c);
        }
    }
    out.push(now);
    let trimmed = out.iter().map(|p| p.trim().to_string());
    trimmed.filter(|p| !p.is_empty()).collect()
}

/// A Rust type as the model writes it.
fn as_model(ty: &str) -> String {
    let mut t = ty.trim();
    loop {
        let before = t;
        for prefix in ["&", "mut ", "dyn ", "impl "] {
            t = t.strip_prefix(prefix).unwrap_or(t).trim_start();
        }
        if let Some(rest) = t.strip_prefix('\'') {
            // A lifetime.
            let end = rest.find(|c: char| !(c.is_alphanumeric() || c == '_'));
            t = rest[end.unwrap_or(rest.len())..].trim_start();
        }
        if t == before {
            break;
        }
    }
    if let Some(inner) = t.strip_prefix('(').and_then(|r| r.strip_suffix(')')) {
        let each: Vec<String> = pieces(inner).iter().map(|p| as_model(p)).collect();
        return format!("({})", each.join(", "));
    }
    if let Some(inner) = t.strip_prefix('[').and_then(|r| r.strip_suffix(']')) {
        let item = inner.split(';').next().unwrap_or(inner);
        return format!("List<{}>", as_model(item));
    }
    let (name, args) = match t.find('<') {
        Some(i) => (&t[..i], t[i + 1..].strip_suffix('>').unwrap_or(&t[i + 1..])),
        None => (t, ""),
    };
    let name = name.rsplit("::").next().unwrap_or(name).trim();
    let name = match name {
        "bool" => "Bool",
        "f64" | "f32" => "Float64",
        "i8" | "i16" | "i32" | "i64" | "isize" | "u8" | "u16" | "u32" | "u64" | "usize" => "Int",
        "str" | "String" => "String",
        "Vec" | "VecDeque" => "List",
        "HashSet" | "BTreeSet" => "Set",
        "HashMap" | "BTreeMap" => "Map",
        "Box" | "Rc" | "Arc" => return as_model(args),
        other => other,
    };
    let args: Vec<String> = pieces(args).iter().map(|p| as_model(p)).collect();
    match args.is_empty() {
        true => name.to_string(),
        false => format!("{name}<{}>", args.join(", ")),
    }
}

/// Whether a type of the model can be the model of this Rust type. The
/// model may be narrower (a range, a list of values): then nothing is
/// compared.
fn fits(model: &str, rust: &str) -> bool {
    let narrow = ["..", "{", "|", "\\", "&", "Any"];
    if narrow.iter().any(|n| model.contains(n)) {
        return true;
    }
    let ints = [
        "UInt8", "UInt16", "UInt32", "Int8", "Int16", "Int32", "Int64",
    ];
    // A type of a block is `block::Name` in the model, `Name` in the code.
    let mut plain = String::new();
    for (i, part) in model.split("::").enumerate() {
        let part = match i + 1 < model.split("::").count() {
            true => part.trim_end_matches(|c: char| c.is_alphanumeric() || c == '_'),
            false => part,
        };
        plain += part;
    }
    for i in ints {
        plain = plain.replace(i, "Int");
    }
    let tight = |s: &str| s.replace(' ', "");
    let (plain, rust) = (tight(&plain), tight(&as_model(rust)));
    // A record prints with its fields, `Mass(Float64, Float64, Float64)`:
    // the name is what is compared.
    let head = |s: &str| s.split('(').next().unwrap_or(s).to_string();
    let named = !rust.is_empty() && !rust.contains(['(', '<']) && !plain.starts_with('(');
    plain == rust || (named && head(&plain) == rust)
}

/// The type an `impl` line is for: `impl<T> Trait for Name<T> {` is `Name`.
fn implemented(line: &str) -> Option<String> {
    let rest = line.strip_prefix("impl")?;
    let rest = match rest.strip_prefix('<') {
        Some(r) => &r[balanced(r, '<', '>')?.len() + 1..],
        None => rest,
    };
    let rest = rest.strip_prefix(' ')?;
    let ty = rest.split(" for ").last()?.trim();
    let end = ty.find(|c: char| !(c.is_alphanumeric() || c == '_' || c == ':'));
    let name = ty[..end.unwrap_or(ty.len())].rsplit("::").next()?;
    Some(name.to_string()).filter(|n| !n.is_empty())
}

/// `text` with the whole word `from` written `to`.
fn words(text: &str, from: &str, to: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    let part = |c: char| c.is_alphanumeric() || c == '_';
    while let Some(i) = rest.find(from) {
        let before = rest[..i].chars().next_back().is_some_and(part);
        let after = rest[i + from.len()..].chars().next().is_some_and(part);
        out += &rest[..i];
        out += if before || after { from } else { to };
        rest = &rest[i + from.len()..];
    }
    out + rest
}

/// The ties of one Rust file.
fn read(
    path: &str,
    text: &str,
    out: &mut Vec<Code>,
    shapes: &mut Vec<Shape>,
) -> Result<(), String> {
    let lines: Vec<&str> = text.lines().collect();
    let scanned = scan(text);
    // `fn name` at the start of a line, whatever stands before `fn`.
    let free = |l: &&str| {
        let head = l.split("fn ").next().unwrap_or("");
        let plain = head
            .split_whitespace()
            .all(|w| w.starts_with("pub") || w == "const");
        let name = l[head.len()..].strip_prefix("fn ")?;
        let end = name.find(|c: char| !c.is_alphanumeric() && c != '_')?;
        (plain && !l.starts_with(' ')).then(|| name[..end].to_string())
    };
    let beside: Vec<String> = lines.iter().filter_map(|l| free(&l)).collect();
    for i in 0..lines.len() {
        let tie = mitra_line(lines[i]).and_then(|m| m.strip_prefix("implements "));
        let Some(implements) = tie.filter(|_| scanned[i].0) else {
            continue;
        };
        let place = format!("{path}:{}", i + 1);
        // The code below: past the other comments of the block.
        let code: Vec<&str> = scanned[i + 1..].iter().map(|l| l.1.as_str()).collect();
        let code = code.join("\n");
        if let Some((kind, after)) = shape_below(&code) {
            shapes.push(Shape {
                implements: implements.trim().to_string(),
                place,
                kind: kind.to_string(),
                fields: fields(after).filter(|_| kind == "struct"),
                variants: variants(after).filter(|_| kind == "enum"),
            });
            continue;
        }
        let Some(name) = fn_below(&code) else {
            let msg = "`implements` must sit above a `fn`, a `struct` or an `enum`";
            return Err(format!("{place}: {msg}"));
        };
        let after = &code[code.find(&format!("fn {name}")).unwrap_or(0)..];
        let open = after
            .find('(')
            .ok_or(format!("{place}: no inputs after `fn {name}`"))?;
        let inside = balanced(&after[open + 1..], '(', ')');
        let inside = inside.ok_or(format!("{place}: unclosed `(` after `fn {name}`"))?;
        let rest = &after[open + 1 + inside.len() + 1..];
        let end = rest.find(['{', ';']).unwrap_or(rest.len());
        let head = rest[..end].split(" where ").next().unwrap_or("");
        let result = head.trim().strip_prefix("->").map_or("()", str::trim);
        let body = match rest[end..].strip_prefix('{') {
            Some(b) => balanced(b, '{', '}').unwrap_or(b),
            None => "",
        };
        // `Self` is the type of the `impl` the function is in.
        let of = lines[..i].iter().rev().find_map(|l| implemented(l));
        let own = |ty: &str| match &of {
            Some(of) => words(ty, "Self", of),
            None => ty.to_string(),
        };
        let result = own(result);
        let params = pieces(inside).into_iter().map(|p| {
            let (name, ty) = match p.split_once(':') {
                Some((n, t)) => (n.trim().to_string(), Some(own(t.trim()))),
                None => ("self".to_string(), None),
            };
            let name = name.strip_prefix("mut ").unwrap_or(&name).trim();
            (name.trim_start_matches('_').to_string(), ty)
        });
        out.push(Code {
            implements: implements.trim().to_string(),
            place,
            name: name.to_string(),
            params: params.collect(),
            result,
            body: body.to_string(),
            beside: beside.clone(),
        });
    }
    Ok(())
}

/// Whether `body` calls a function named `name`.
fn calls(body: &str, name: &str) -> bool {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    body.match_indices(name).any(|(i, _)| {
        let before = body[..i].chars().next_back();
        let after = body[i + name.len()..].trim_start();
        // `name(` or `name::<T>(`, not a longer name.
        !before.is_some_and(word) && (after.starts_with('(') || after.starts_with("::<"))
    })
}

/// What the reader found: evidence lines, and what it prints.
// mitra: implements ties::Found
pub struct Found {
    pub evidence: Vec<String>,
    pub notes: Vec<String>,
    pub agree: usize,
    pub differ: usize,
}

/// Compare the code under `root` with the model.
pub use mitra_lang::results::Tied;

/// Every test of `src/` and `tests/` tied to a claim.
// mitra: implements ties::tests
pub fn tests(root: &Path) -> Result<Vec<Tied>, String> {
    let mut files = Vec::new();
    for d in ["src", "tests"] {
        rust_files(&root.join(d), &mut files);
    }
    files.sort();
    let mut out = Vec::new();
    for f in files {
        let shown = f.strip_prefix(root).unwrap_or(&f).display().to_string();
        let text = std::fs::read_to_string(&f).map_err(|e| format!("{shown}: {e}"))?;
        let lines: Vec<&str> = text.lines().collect();
        let scanned = scan(&text);
        for i in 0..lines.len() {
            let tie = mitra_line(lines[i]).and_then(|m| m.strip_prefix("evidence for "));
            let Some(claim) = tie.filter(|_| scanned[i].0) else {
                continue;
            };
            let place = format!("{shown}:{}", i + 1);
            let code: Vec<&str> = scanned[i + 1..].iter().map(|l| l.1.as_str()).collect();
            let code = code.join("\n");
            let Some(test) = fn_below(&code) else {
                return Err(format!(
                    "{place}: `evidence for` must sit above a test `fn`"
                ));
            };
            let claim = claim.trim();
            let (claim, pinned) = claim.split_once(char::is_whitespace).unwrap_or((claim, ""));
            out.push(Tied {
                test: test.to_string(),
                claim: claim.to_string(),
                pinned: pinned.trim().to_string(),
                place,
            });
        }
    }
    Ok(out)
}

// mitra: implements ties::tie
pub fn tie(root: &Path, model: &Model, rev: Option<&str>) -> Result<Found, String> {
    let mut files = Vec::new();
    for d in ["src", "tests"] {
        rust_files(&root.join(d), &mut files);
    }
    files.sort();
    let (mut code, mut tests) = (Vec::new(), String::new());
    let mut shapes = Vec::new();
    for f in files {
        let shown = f.strip_prefix(root).unwrap_or(&f).display().to_string();
        let text = std::fs::read_to_string(&f).map_err(|e| format!("{shown}: {e}"))?;
        read(&shown, &text, &mut code, &mut shapes)?;
        tests += &text;
    }
    let mut found = Found {
        evidence: Vec::new(),
        notes: Vec::new(),
        agree: 0,
        differ: 0,
    };
    let attrs = match rev {
        Some(r) => format!(" rev \"{r}\""),
        None => String::new(),
    };
    let clean = |why: &str| why.replace(['"', '\\'], "'");
    let say = |found: &mut Found, claim: &str, place: &str, why: Option<String>| match why {
        None => {
            found.agree += 1;
            let source = format!("reader:rust {place}");
            found.evidence.push(format!(
                "evidence tie {{ for {claim} holds source \"{source}\"{attrs} }}"
            ));
        }
        Some(why) => {
            found.differ += 1;
            found.notes.push(format!("{place}: {claim}: {why}"));
            let source = format!("reader:rust {place}: {}", clean(&why));
            found.evidence.push(format!(
                "evidence tie {{ for {claim} fails source \"{source}\"{attrs} }}"
            ));
        }
    };
    // Records: every field the model states is in the struct, with a type
    // that fits; without `..` the struct has no other field.
    for s in &shapes {
        // An enum of the model: the same variants, each holding as many
        // values, of types that fit.
        if let Some(e) = model.enums.iter().find(|e| e.name == s.implements) {
            if shapes
                .iter()
                .filter(|d| d.implements == s.implements)
                .count()
                > 1
            {
                let msg = format!("{}: `{}` is implemented twice", s.place, s.implements);
                return Err(msg);
            }
            let last = |n: &str| n.rsplit("::").next().unwrap_or(n).to_string();
            let mut why = None;
            match &s.variants {
                None => why = Some(format!("the code is a `{}`, not an enum", s.kind)),
                Some(_) if !e.params.is_empty() => {
                    why = Some("an enum with type parameters is not compared".to_string())
                }
                Some(code) => {
                    for v in &e.variants {
                        let of = |r: &&mitra_lang::Record| {
                            r.name == *v && r.of.as_deref() == Some(e.name.as_str())
                        };
                        let Some(r) = model.records.iter().find(of) else {
                            continue;
                        };
                        let stated: Vec<&(String, mitra_lang::Set)> =
                            r.fields.iter().filter(|f| f.0 != "..").collect();
                        let name = last(v);
                        match code.iter().find(|c| c.0 == name) {
                            None => why = Some(format!("the enum has no variant `{name}`")),
                            Some((_, held)) if held.len() != stated.len() => {
                                why = Some(format!(
                                "variant `{name}` holds {} value(s) in the code, {} in the model",
                                held.len(),
                                stated.len()
                            ))
                            }
                            Some((_, held)) => {
                                for ((field, set), ty) in stated.iter().zip(held) {
                                    let mty = model.named(&set.to_string());
                                    if !fits(&mty, ty) && why.is_none() {
                                        why = Some(format!(
                                            "`{field}` of variant `{name}` is `{ty}` in the code, \
                                             `{mty}` in the model"
                                        ));
                                    }
                                }
                            }
                        }
                        if why.is_some() {
                            break;
                        }
                    }
                    let known =
                        |c: &&(String, Vec<String>)| e.variants.iter().any(|v| last(v) == c.0);
                    if let Some((name, _)) =
                        code.iter().find(|c| !known(c)).filter(|_| why.is_none())
                    {
                        why = Some(format!(
                            "the enum has a variant `{name}` the model does not state"
                        ));
                    }
                }
            }
            say(&mut found, &format!("{}::tied", e.name), &s.place, why);
            continue;
        }
        let record = model.records.iter().find(|r| r.name == s.implements);
        let Some(r) = record.filter(|r| r.of.is_none()) else {
            let msg = format!("{}: the model has no record `{}`", s.place, s.implements);
            return Err(msg);
        };
        if shapes
            .iter()
            .filter(|d| d.implements == s.implements)
            .count()
            > 1
        {
            let msg = format!("{}: `{}` is implemented twice", s.place, s.implements);
            return Err(msg);
        }
        let stated: Vec<&(String, mitra_lang::Set)> =
            r.fields.iter().filter(|f| f.0 != "..").collect();
        let mut why = None;
        match &s.fields {
            // Nothing of it is stated: any struct or enum will do.
            _ if stated.is_empty() && r.open => {}
            None => why = Some(format!("the code has no named fields (`{}`)", s.kind)),
            Some(fields) => {
                for (name, set) in &stated {
                    let mty = model.named(&set.to_string());
                    let bare = |n: &str| n.strip_prefix("r#").unwrap_or(n).to_string();
                    match fields.iter().find(|f| bare(&f.0) == bare(name)) {
                        None => why = Some(format!("the struct has no field `{name}`")),
                        Some((_, ty)) if !fits(&mty, ty) => {
                            why = Some(format!(
                                "field `{name}` is `{ty}` in the code, `{mty}` in the model"
                            ))
                        }
                        Some(_) => {}
                    }
                    if why.is_some() {
                        break;
                    }
                }
                let bare = |n: &str| n.strip_prefix("r#").unwrap_or(n).to_string();
                let extra = fields
                    .iter()
                    .find(|f| !stated.iter().any(|m| bare(&m.0) == bare(&f.0)));
                if let Some((name, _)) = extra.filter(|_| why.is_none() && !r.open) {
                    why = Some(format!(
                        "the struct has a field `{name}` the record does not state, and no `..`"
                    ));
                }
            }
        }
        say(&mut found, &format!("{}::tied", r.name), &s.place, why);
    }
    for c in &code {
        let Some(f) = model
            .declared_function(&c.implements)
            .filter(|f| !f.is_spec())
        else {
            let msg = format!("{}: the model has no function `{}`", c.place, c.implements);
            return Err(msg);
        };
        if code.iter().filter(|d| d.implements == c.implements).count() > 1 {
            return Err(format!(
                "{}: `{}` is implemented twice",
                c.place, c.implements
            ));
        }
        // Inputs and result.
        let mut why = None;
        if c.params.len() != f.params.len() {
            let (n, m) = (c.params.len(), f.params.len());
            why = Some(format!("{n} input(s) in the code, {m} in the model"));
        }
        for ((name, ty), (mname, mty)) in c.params.iter().zip(&f.params) {
            let mty = model.named(&mty.to_string());
            if why.is_some() {
                break;
            }
            // `r#match` and `match` are one name.
            let bare = |n: &str| n.strip_prefix("r#").unwrap_or(n).to_string();
            if bare(name) != bare(mname) && name != "self" {
                why = Some(format!(
                    "input `{name}` in the code is `{mname}` in the model"
                ));
            } else if let Some(ty) = ty.as_ref().filter(|ty| !fits(&mty, ty)) {
                why = Some(format!(
                    "input `{name}` is `{ty}` in the code, `{mty}` in the model"
                ));
            }
        }
        let range = model.named(&f.range.to_string());
        if why.is_none() && !fits(&range, &c.result) {
            let r = &c.result;
            why = Some(format!(
                "the result is `{r}` in the code, `{range}` in the model"
            ));
        }
        // What agreeing does not compare: said with the tie, so that it
        // can be doubted by name.
        let narrow = |m: &str| {
            ["..", "{", "|", "\\", "&", "Any"]
                .iter()
                .any(|n| m.contains(n))
        };
        let mut taken: Vec<String> = Vec::new();
        let mut numbers = range.contains("Int") || range.contains("Float");
        for ((name, ty), (_, mty)) in c.params.iter().zip(&f.params) {
            let mty = model.named(&mty.to_string());
            numbers |= mty.contains("Int") || mty.contains("Float") || mty.contains("..");
            if let Some(ty) = ty.as_ref().filter(|_| narrow(&mty)) {
                taken.push(format!("{name} is {mty} in the model and {ty} in the code"));
            }
        }
        if narrow(&range) {
            let r = &c.result;
            taken.push(format!(
                "the result is {range} in the model and {r} in the code"
            ));
        }
        if numbers {
            taken.push("the numbers of the model computed as the code computes".to_string());
        }
        let place = match (why.is_none(), taken.is_empty()) {
            (true, false) => format!("{}; not compared: {}", c.place, clean(&taken.join(", "))),
            _ => c.place.clone(),
        };
        say(&mut found, &format!("{}::tied", f.name), &place, why);
        // Calls.
        let uses = model.claims.iter().find_map(|k| match &k.target {
            Target::Uses { used, only } if k.func == f.name => Some((k, used, *only)),
            _ => None,
        });
        let Some((claim, used, only)) = uses else {
            // No `uses`, written or implied by a body: it calls nothing
            // outside its own subject.
            let block = |name: &str| name.split_once("::").map_or("", |p| p.0).to_string();
            let file = |place: &str| place.rsplit_once(':').map_or("", |p| p.0).to_string() + ":";
            let home = block(&f.name);
            let own = |d: &Code| {
                code.iter()
                    .any(|e| e.name == d.name && block(&e.implements) == home)
            };
            // Only what the text settles: a method through a dot may be
            // any type's, and a name the file defines is its own.
            // The type a function is written in, `Modules` for
            // `modules::Modules::new`: it is then called as `Modules::new(`,
            // with or without `self`.
            let of_type = |d: &Code| {
                let of = d.implements.rsplit("::").nth(1).unwrap_or("");
                of.starts_with(char::is_uppercase).then(|| of.to_string())
            };
            let plainly = |d: &Code| match of_type(d) {
                Some(of) => c.body.contains(&format!("{of}::{}(", d.name)),
                None => {
                    // Its own: a function of the file, or a `let` of the body.
                    let bound = c.body.contains(&format!("let {} =", d.name));
                    let beside = c.beside.contains(&d.name);
                    let local = bound || (beside && !d.place.starts_with(&file(&c.place)));
                    !local && calls(&c.body.replace(&format!(".{}", d.name), ""), &d.name)
                }
            };
            let outside = code
                .iter()
                .find(|d| block(&d.implements) != home && !own(d) && plainly(d));
            if let Some(d) = outside {
                let why = format!(
                    "`{}` calls `{}`, which is `{}`, and the model states no `uses`",
                    c.name, d.name, d.implements
                );
                say(
                    &mut found,
                    &format!("{}::uses", f.name),
                    &c.place,
                    Some(why),
                );
            }
            continue;
        };
        let code_of = |g: &str| code.iter().find(|d| d.implements == g);
        let mut why = None;
        let mut checked = false;
        for g in used {
            let Some(d) = code_of(g) else { continue };
            checked = true;
            if !calls(&c.body, &d.name) {
                why = Some(format!("`{}` does not call `{}`", c.name, d.name));
            }
        }
        if only {
            checked = true;
            for d in code.iter().filter(|d| d.implements != f.name) {
                // Another function of the same name may be the one called.
                let same = |e: &&Code| e.name == d.name && used.contains(&e.implements);
                // A function written in a type is called as `Type::name(`:
                // through a dot it may be any type's.
                let of = d.implements.rsplit("::").nth(1).unwrap_or("");
                let called = match of.starts_with(char::is_uppercase) {
                    true => c.body.contains(&format!("{of}::{}(", d.name)),
                    false => calls(&c.body, &d.name),
                };
                if called && !code.iter().any(|e| same(&e)) && why.is_none() {
                    let (n, g) = (&c.name, &d.implements);
                    why = Some(format!("`{n}` also calls `{}`, which is `{g}`", d.name));
                }
            }
        }
        if checked {
            say(&mut found, &claim.name, &c.place, why);
        }
    }
    // What has no tie.
    let untied =
        |f: &&mitra_lang::Function| !f.is_spec() && !code.iter().any(|c| c.implements == f.name);
    // The text of a requirement is no code.
    let untied = |f: &&mitra_lang::Function| untied(f) && !f.name.ends_with("::prose");
    let names: Vec<&str> = model
        .functions
        .iter()
        .filter(untied)
        .map(|f| f.name.as_str())
        .collect();
    let mut names: Vec<&str> = names;
    let loose =
        |r: &&mitra_lang::Record| r.of.is_none() && !shapes.iter().any(|s| s.implements == r.name);
    // What another store declares is tied there.
    let here = |n: &&str| model.block_of(n).is_none_or(|b| b.from.is_none());
    names.extend(model.records.iter().filter(loose).map(|r| r.name.as_str()));
    let loose = |e: &&mitra_lang::Enum| {
        e.params.is_empty() && !shapes.iter().any(|s| s.implements == e.name)
    };
    names.extend(model.enums.iter().filter(loose).map(|e| e.name.as_str()));
    let names: Vec<&str> = names.into_iter().filter(here).collect();
    if !names.is_empty() {
        found
            .notes
            .push(format!("no code is tied to: {}", names.join(", ")));
    }
    Ok(found)
}
