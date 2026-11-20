//! A signature of the code, in the model's words.

use super::*;

/// A `fn` of the code, as far as text tells.
pub(super) struct Code {
    /// The function of the model it says it implements.
    pub(super) implements: String,
    pub(super) place: String,
    pub(super) name: String,
    /// Each parameter with its type; `self` has none.
    pub(super) params: Vec<(String, Option<String>)>,
    pub(super) result: String,
    pub(super) body: String,
    /// The free functions its file defines: a call by one of these names
    /// is a call to the one beside it.
    pub(super) beside: Vec<String>,
    /// Each type parameter with the traits that bound it: its own
    /// `<L: Link>` and `where`, and those of the `impl` it is in.
    pub(super) bounds: Vec<(String, Vec<String>)>,
}

/// The traits of `Link + Store<K, V> + 'a`, by name.
pub(super) fn traits(bound: &str) -> Vec<String> {
    let name = |t: &str| {
        let t = t.trim().split('<').next().unwrap_or("").trim();
        t.rsplit("::").next().unwrap_or(t).to_string()
    };
    let mut depth = 0;
    let split = |c: char| {
        depth += (c == '<') as i32 - (c == '>') as i32;
        c == '+' && depth == 0
    };
    let all = bound.split(split).map(name);
    all.filter(|t| !t.is_empty() && !t.starts_with('\''))
        .collect()
}

/// The bounds written in `L: Link, M` (between `<` and `>`, or after
/// `where`), added to `into`.
pub(super) fn bounds_of(text: &str, into: &mut Vec<(String, Vec<String>)>) {
    for piece in pieces(text) {
        // A lifetime and a `const N: usize` are no type.
        if piece.starts_with('\'') || piece.starts_with("const ") {
            continue;
        }
        match piece.split_once(':') {
            Some((name, bound)) => into.push((name.trim().to_string(), traits(bound))),
            None => into.push((piece.trim().to_string(), Vec::new())),
        }
    }
}

/// The bounds of a `fn` or `impl` head, the text from its keyword to its
/// `{` or `;`: its `<…>` and its `where`.
pub(super) fn head_bounds(head: &str, into: &mut Vec<(String, Vec<String>)>) {
    let (list, clause) = match head.split_once(" where ") {
        Some((l, c)) => (l, c),
        None => (head, ""),
    };
    // The first `<` before any `(`: the parameters.
    let open = list
        .find('<')
        .filter(|i| list.find('(').is_none_or(|p| *i < p));
    if let Some(inside) = open.and_then(|i| balanced(&list[i + 1..], '<', '>')) {
        bounds_of(inside, into);
    }
    bounds_of(clause, into);
}

/// A `struct` or an `enum` of the code tied to a record of the model.
pub(super) struct Shape {
    pub(super) implements: String,
    pub(super) place: String,
    /// `struct` or `enum`.
    pub(super) kind: String,
    /// Each named field with its type; none for an enum, a tuple struct
    /// or a unit struct.
    pub(super) fields: Option<Vec<(String, String)>>,
    /// For an enum: each variant with the types it holds, in order.
    pub(super) variants: Option<Vec<(String, Vec<String>)>>,
}

/// The variants of an enum, from the text after its name: `A`, `A(T, U)`
/// and `A { x: T }`, each with the types it holds.
pub(super) fn variants(after: &str) -> Option<Vec<(String, Vec<String>)>> {
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
pub(super) fn shape_below(code: &str) -> Option<(&str, &str)> {
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
pub(super) fn fields(after: &str) -> Option<Vec<(String, String)>> {
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
pub(super) fn balanced(s: &str, open: char, close: char) -> Option<&str> {
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
pub(super) fn pieces(s: &str) -> Vec<String> {
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
pub(super) fn as_model(ty: &str) -> String {
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
        "PathBuf" => "Path",
        // How a value is held or handed over is the code's own: the model
        // has no memory and no ownership.
        "Box" | "Rc" | "Arc" | "Into" | "AsRef" => return as_model(args),
        "Cow" => return as_model(pieces(args).last().map_or("", |p| p)),
        "Iterator" | "IntoIterator" => {
            let item = args.trim().strip_prefix("Item").map(|a| a.trim_start());
            let item = item.and_then(|a| a.strip_prefix('=')).unwrap_or(args);
            return format!("List<{}>", as_model(item));
        }
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
pub(super) fn fits(model: &str, rust: &str) -> bool {
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
pub(super) fn implemented(line: &str) -> Option<String> {
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
pub(super) fn words(text: &str, from: &str, to: &str) -> String {
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
pub(super) fn read(
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
        let head = rest[..end].replace('\n', " ");
        let head = head.split(" where ").next().unwrap_or("");
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
        let mut bounds = Vec::new();
        let whole = after.find(['{', ';']).unwrap_or(after.len());
        // `->` would close a `<`: the bounds hold none.
        let spaced = after[..whole].replace("->", "==").replace('\n', " ");
        head_bounds(&spaced, &mut bounds);
        let around = lines[..i].iter().rev().find(|l| implemented(l).is_some());
        if let Some(line) = around {
            head_bounds(line.trim_end_matches('{').trim(), &mut bounds);
        }
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
            bounds,
        });
    }
    Ok(())
}

/// Whether `body` calls a function named `name`.
pub(super) fn calls(body: &str, name: &str) -> bool {
    let word = |c: char| c.is_alphanumeric() || c == '_';
    body.match_indices(name).any(|(i, _)| {
        let before = body[..i].chars().next_back();
        let after = body[i + name.len()..].trim_start();
        // `name(` or `name::<T>(`, not a longer name.
        !before.is_some_and(word) && (after.starts_with('(') || after.starts_with("::<"))
    })
}
