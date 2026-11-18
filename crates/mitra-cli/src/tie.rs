//! `mitra tie`: the reader of Rust code. A comment `// mitra: implements
//! subject::name` above a `fn` ties it to a function of the model. The
//! reader compares the two (inputs, result, calls) and says what it found
//! as plain evidence: the core never depends on it, and a reviewer can
//! bring the same evidence by hand.

use std::path::Path;

use crate::source::{fn_below, mitra_line, rust_files, scan};
use mitra_core::{Model, Target};

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
        return format!("List({})", as_model(item));
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
        false => format!("{name}({})", args.join(", ")),
    }
}

/// Whether a type of the model can be the model of this Rust type. The
/// model may be narrower (a range, a list of values): then nothing is
/// compared.
fn fits(model: &str, rust: &str) -> bool {
    let narrow = ["..", "{", "|", "\\", "&", "Value"];
    if narrow.iter().any(|n| model.contains(n)) {
        return true;
    }
    let ints = [
        "UInt8", "UInt16", "UInt32", "Int8", "Int16", "Int32", "Int64",
    ];
    let mut plain = model.to_string();
    for i in ints {
        plain = plain.replace(i, "Int");
    }
    let tight = |s: &str| s.replace(' ', "");
    let (plain, rust) = (tight(&plain), tight(&as_model(rust)));
    // A record prints with its fields, `Mass(Float64, Float64, Float64)`:
    // the name is what is compared.
    let head = |s: &str| s.split('(').next().unwrap_or(s).to_string();
    let named = !rust.is_empty() && !rust.contains('(') && !plain.starts_with('(');
    plain == rust || (named && head(&plain) == rust)
}

/// The ties of one Rust file.
fn read(path: &str, text: &str, out: &mut Vec<Code>) -> Result<(), String> {
    let lines: Vec<&str> = text.lines().collect();
    let scanned = scan(text);
    for i in 0..lines.len() {
        let tie = mitra_line(lines[i]).and_then(|m| m.strip_prefix("implements "));
        let Some(implements) = tie.filter(|_| scanned[i].0) else {
            continue;
        };
        let place = format!("{path}:{}", i + 1);
        // The code below: past the other comments of the block.
        let code: Vec<&str> = scanned[i + 1..].iter().map(|l| l.1.as_str()).collect();
        let code = code.join("\n");
        let Some(name) = fn_below(&code) else {
            return Err(format!("{place}: `implements` must sit above a `fn`"));
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
        let params = pieces(inside).into_iter().map(|p| {
            let (name, ty) = match p.split_once(':') {
                Some((n, t)) => (n.trim().to_string(), Some(t.trim().to_string())),
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
            result: result.to_string(),
            body: body.to_string(),
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
pub struct Found {
    pub evidence: Vec<String>,
    pub notes: Vec<String>,
    pub agree: usize,
    pub differ: usize,
}

/// Compare the code under `root` with the model.
/// A test tied to a claim: `// mitra: evidence for math::div_small` above
/// its `fn`. A pass is evidence that the claim holds, a failure that it
/// does not.
pub struct Tied {
    /// The name of the test function.
    pub test: String,
    pub claim: String,
    /// The file and line of the comment.
    pub place: String,
}

/// Every test of `src/` and `tests/` tied to a claim.
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
            out.push(Tied {
                test: test.to_string(),
                claim: claim.trim().to_string(),
                place,
            });
        }
    }
    Ok(out)
}

pub fn tie(root: &Path, model: &Model, rev: Option<&str>) -> Result<Found, String> {
    let mut files = Vec::new();
    for d in ["src", "tests"] {
        rust_files(&root.join(d), &mut files);
    }
    files.sort();
    let (mut code, mut tests) = (Vec::new(), String::new());
    for f in files {
        let shown = f.strip_prefix(root).unwrap_or(&f).display().to_string();
        let text = std::fs::read_to_string(&f).map_err(|e| format!("{shown}: {e}"))?;
        read(&shown, &text, &mut code)?;
        tests += &text;
    }
    let mut found = Found {
        evidence: Vec::new(),
        notes: Vec::new(),
        agree: 0,
        differ: 0,
    };
    let attrs = match rev {
        Some(r) => format!(", rev \"{r}\""),
        None => String::new(),
    };
    let clean = |why: &str| why.replace(['"', '\\'], "'");
    let say = |found: &mut Found, claim: &str, place: &str, why: Option<String>| match why {
        None => {
            found.agree += 1;
            let line = format!("@[source \"reader:rust {place}\"{attrs}] evidence {claim} at 1");
            found.evidence.push(line);
        }
        Some(why) => {
            found.differ += 1;
            found.notes.push(format!("{place}: {claim}: {why}"));
            let source = format!("reader:rust {place}: {}", clean(&why));
            let line = format!("@[source \"{source}\"{attrs}] evidence not {claim} at 1");
            found.evidence.push(line);
        }
    };
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
            if name != mname && name != "self" {
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
        say(&mut found, &format!("{}::tied", f.name), &c.place, why);
        // Calls.
        let uses = model.claims.iter().find_map(|k| match &k.target {
            Target::Uses { used, only } if k.func == f.name => Some((k, used, *only)),
            _ => None,
        });
        let Some((claim, used, only)) = uses else {
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
                if calls(&c.body, &d.name) && !code.iter().any(|e| same(&e)) && why.is_none() {
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
        |f: &&mitra_core::Function| !f.is_spec() && !code.iter().any(|c| c.implements == f.name);
    let names: Vec<&str> = model
        .functions
        .iter()
        .filter(untied)
        .map(|f| f.name.as_str())
        .collect();
    if !names.is_empty() {
        found
            .notes
            .push(format!("no code is tied to: {}", names.join(", ")));
    }
    for t in &model.tests {
        if !calls(&tests, &format!("fn {}", t.name)) {
            found
                .notes
                .push(format!("test `{}`: no `fn {}` in the code", t.name, t.name));
        }
    }
    Ok(found)
}
