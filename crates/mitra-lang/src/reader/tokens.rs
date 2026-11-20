//! The tokens of a text: words, numbers, texts, punctuation, each with
//! its line. The keywords of the language.

use crate::reader::prelude::*;

/// The tokens of a text, with each name a `type` gave to a collection
/// replaced by the collection's: `Vec` by `List` after `import
/// std::rust::{Vec};`.
pub(crate) fn lex(src: &str) -> Result<Vec<Tok>, Error> {
    let mut toks = lex_raw(src)?;
    let mut renames = types_named(&toks);
    for (i, t) in toks.iter().enumerate() {
        if t.kind != Kind::Name
            || t.text != "import"
            || toks.get(i + 1).is_none_or(|n| n.text != "std")
        {
            continue;
        }
        let Some(part) = toks.get(i + 3).filter(|n| n.kind == Kind::Name) else {
            continue;
        };
        let Some(module) = library(&format!("std::{}", part.text)) else {
            continue;
        };
        let end = toks[i..]
            .iter()
            .position(|n| n.text == ";")
            .map_or(toks.len(), |e| i + e);
        for (name, named) in types_named(&lex_raw(module)?) {
            if toks[i + 4..end].iter().any(|n| n.text == name) {
                renames.insert(name, named);
            }
        }
    }
    if renames.is_empty() {
        return Ok(toks);
    }
    // Not where the name is declared or imported.
    let mut inside = false;
    for i in 0..toks.len() {
        if toks[i].kind == Kind::Name && (toks[i].text == "import" || toks[i].text == "type") {
            inside = true;
        } else if toks[i].text == ";" {
            inside = false;
        } else if !inside && toks[i].kind == Kind::Name {
            if let Some(named) = renames.get(&toks[i].text) {
                toks[i].text = named.clone();
            }
        }
    }
    Ok(toks)
}

/// `type Vec<T> = List<T>;` at the top of a text: `Vec` names `List`.
pub(crate) fn types_named(toks: &[Tok]) -> BTreeMap<String, String> {
    let (mut out, mut depth) = (BTreeMap::new(), 0);
    for (i, t) in toks.iter().enumerate() {
        if t.kind == Kind::Punct {
            depth += (t.text == "{") as i32 - (t.text == "}") as i32;
        }
        if depth != 0 || t.kind != Kind::Name || t.text != "type" {
            continue;
        }
        let equals = toks[i..]
            .iter()
            .position(|n| n.text == "=" || n.text == ";");
        if let (Some(name), Some(e)) = (toks.get(i + 1), equals) {
            if let Some(named) = toks.get(i + e + 1).filter(|n| n.kind == Kind::Name) {
                out.insert(name.text.clone(), named.text.clone());
            }
        }
    }
    out
}

/// The declarations at the top of a file that start an item.
pub const ITEMS: [&str; 13] = [
    "subject", "boundary", "record", "enum", "fn", "claim", "assume", "test", "import", "ensure",
    "require", "contract", "type",
];

#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum Kind {
    #[default]
    Name,
    Num,
    Str,
    Punct,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct Tok {
    pub(crate) kind: Kind,
    pub(crate) text: String,
    pub(crate) line: usize,
}

/// Longest first.
pub(crate) const PUNCT: [&str; 34] = [
    "<=>", "..=", "::", "=>", "->", "==", "!=", "<=", ">=", "&&", "||", "..", "(", ")", "{", "}",
    "[", "]", ",", ";", ":", "=", "<", ">", "+", "-", "*", "/", "!", "|", "&", "\\", ".", "@",
];

fn lex_raw(src: &str) -> Result<Vec<Tok>, Error> {
    let mut toks = Vec::new();
    let mut line = 1;
    let mut rest = src;
    let mut push = |kind, text: &str, line| {
        toks.push(Tok {
            kind,
            text: text.to_string(),
            line,
        })
    };
    while let Some(c) = rest.chars().next() {
        if c == '\n' {
            line += 1;
        }
        if c.is_whitespace() {
            rest = &rest[c.len_utf8()..];
        } else if rest.starts_with("//") {
            rest = rest.find('\n').map_or("", |n| &rest[n..]);
        } else if c == '"' {
            let mut end = None;
            let mut escaped = false;
            for (i, c) in rest[1..].char_indices() {
                match c {
                    '\n' => break,
                    '"' if !escaped => {
                        end = Some(i);
                        break;
                    }
                    _ if escaped && !"nt\"\\".contains(c) => {
                        let msg = "in a string, `\\` starts `\\n`, `\\t`, `\\\"` or `\\\\`";
                        return Err(Error::new(line, msg));
                    }
                    _ => escaped = c == '\\' && !escaped,
                }
            }
            let Some(end) = end else {
                return Err(Error::new(line, "unterminated string"));
            };
            push(Kind::Str, &rest[..end + 2], line);
            rest = &rest[end + 2..];
        } else if c.is_ascii_digit() {
            let digits = |s: &str| s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
            let mut end = digits(rest);
            if let Some(frac) = rest[end..].strip_prefix('.').map(digits).filter(|&f| f > 0) {
                end += 1 + frac;
            }
            push(Kind::Num, &rest[..end], line);
            rest = &rest[end..];
        } else if c.is_ascii_alphabetic() || c == '_' {
            // `r#match`, Rust's raw identifier: a name, whatever the word.
            let word = |c: char| c.is_ascii_alphabetic() || c == '_';
            let raw = rest.strip_prefix("r#").filter(|r| r.starts_with(word));
            let from = if raw.is_some() { 2 } else { 0 };
            let end = rest[from..]
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .map_or(rest.len(), |n| n + from);
            push(Kind::Name, &rest[..end], line);
            rest = &rest[end..];
        } else if let Some(p) = PUNCT.iter().find(|p| rest.starts_with(**p)) {
            push(Kind::Punct, p, line);
            rest = &rest[p.len()..];
        } else {
            return Err(Error::new(line, format!("unexpected character `{c}`")));
        }
    }
    Ok(toks)
}

/// The first line written in the syntax of before the rework (`opaque`, `def`, `:=`,
/// `claim c :`, `-- comment`), which is no longer read, and what to
/// write instead.
pub fn earlier(model: &str) -> Option<(usize, &'static str)> {
    let named = |l: &str, k: &str| {
        let rest = l.strip_prefix(k).map(str::trim_start)?;
        let name = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))?;
        Some(rest[name..].trim_start().starts_with(':') && name > 0)
    };
    let hint = |l: &str| {
        let l = l.trim_start();
        if l.starts_with("opaque ") {
            Some("`opaque f(a: T) -> U` is written `fn f(a: T) -> U;` inside `subject name { … }`")
        } else if l.starts_with("def ") {
            Some("`def f(a: T) -> U := e` is written `fn f(a: T) -> U { e }`")
        } else if named(l, "claim ") == Some(true) {
            Some("`claim c : forall a, e` is written `claim c(a: T) { e }`")
        } else if named(l, "test ") == Some(true) {
            Some("a test is a comment above the real test: `// mitra: evidence for <claim>`")
        } else if l.starts_with("--") {
            Some("a comment starts with `//`")
        } else if l.starts_with("@[")
            || (l.starts_with("evidence ") && l.contains(" at ") && !l.contains('{'))
        {
            Some("evidence is written `evidence name { for f given a = 1 holds }`")
        } else if l.contains(":=") {
            Some("`:=` is not used: a body is written in braces")
        } else {
            None
        }
    };
    let mut lines = model.lines().enumerate();
    lines.find_map(|(i, l)| Some((i + 1, hint(l)?)))
}

/// Reserved words: never names.
pub const KEYWORDS: [&str; 36] = [
    "Set",
    "Map",
    "keys",
    "enum",
    "match",
    "rate",
    "precision",
    "recall",
    "false_positive_rate",
    "List",
    "len",
    "record",
    "fn",
    "claim",
    "evidence",
    "correct",
    "if",
    "else",
    "let",
    "true",
    "false",
    "in",
    "Bool",
    "Int",
    "Int8",
    "Int16",
    "Int32",
    "Int64",
    "UInt8",
    "UInt16",
    "UInt32",
    "Float64",
    "String",
    "Any",
    "_",
    "Real",
];

/// The reserved words that only start a declaration: nothing else
/// begins with them, so each can also name an input.
pub(crate) const DECLARING: [&str; 5] = ["record", "fn", "claim", "evidence", "enum"];

/// The text of a string as written between quotes.
pub(crate) fn unquote(raw: &str) -> String {
    let (mut out, mut chars) = (String::new(), raw[1..raw.len() - 1].chars());
    while let Some(c) = chars.next() {
        out.push(match c {
            '\\' => match chars.next() {
                Some('n') => '\n',
                Some('t') => '\t',
                Some(c) => c,
                None => '\\',
            },
            c => c,
        });
    }
    out
}

/// Tokens as one line of text.
pub(crate) fn join(toks: &[Tok]) -> String {
    let each: Vec<&str> = toks.iter().map(|t| t.text.as_str()).collect();
    each.join(" ")
}

/// Tokens joined by spaces, set as a person writes types: `Stack < T >`
/// is `Stack<T>`.
pub(crate) fn tidy(spaced: &str) -> String {
    let mut out = spaced.to_string();
    for (from, to) in [
        (" ,", ","),
        (" :", ":"),
        ("< ", "<"),
        (" <", "<"),
        (" >", ">"),
        ("( ", "("),
        (" (", "("),
        (" )", ")"),
        ("& ", "&"),
        ("-> (", "-> ("),
    ] {
        out = out.replace(from, to);
    }
    out.replace("->(", "-> (")
}
