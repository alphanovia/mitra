//! The comments of a Rust file that speak to Mitra, and the code below
//! them, as far as text tells.

use std::path::{Path, PathBuf};

pub fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        // Symbolic links are not followed: they may leave the project or loop.
        let Ok(kind) = e.file_type() else { continue };
        let p = e.path();
        if kind.is_dir() {
            rust_files(&p, out);
        } else if kind.is_file() && p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// The text after `mitra:` in a `///` or `//` comment line.
// mitra: implements source::mitra_line
pub fn mitra_line(line: &str) -> Option<&str> {
    let rest = line.trim_start();
    let rest = rest
        .strip_prefix("///")
        .or_else(|| rest.strip_prefix("//"))?;
    let rest = rest.trim_start().strip_prefix("mitra:")?;
    Some(rest.strip_prefix(' ').unwrap_or(rest))
}

/// Where a Rust line starts: in code, or inside a block comment or a
/// string that began on an earlier line.
#[derive(Clone, Copy, PartialEq)]
enum At {
    Code,
    Comment(usize),
    Str,
    Raw(usize),
}

/// For each line: whether it starts in code, and its code, with comments
/// removed and strings emptied.
pub fn scan(text: &str) -> Vec<(bool, String)> {
    let mut at = At::Code;
    let mut out = Vec::new();
    for line in text.lines() {
        let c: Vec<char> = line.chars().collect();
        let (starts_in_code, mut code, mut i) = (at == At::Code, String::new(), 0);
        while i < c.len() {
            let two = |a, b| c[i] == a && c.get(i + 1) == Some(&b);
            let ident = |p: char| p.is_alphanumeric() || p == '_';
            match at {
                At::Code if two('/', '/') => break,
                At::Code if two('/', '*') => (at, i) = (At::Comment(1), i + 1),
                At::Code if c[i] == '"' => {
                    code.push_str("\"\"");
                    at = At::Str;
                }
                At::Code if c[i] == 'r' && !code.ends_with(ident) => {
                    let hashes = c[i + 1..].iter().take_while(|x| **x == '#').count();
                    if c.get(i + 1 + hashes) == Some(&'"') {
                        code.push_str("\"\"");
                        (at, i) = (At::Raw(hashes), i + 1 + hashes);
                    } else {
                        code.push('r');
                    }
                }
                // A character literal, as opposed to a lifetime.
                At::Code if c[i] == '\'' && c.get(i + 1) == Some(&'\\') => {
                    i += 2;
                    while i < c.len() && c[i] != '\'' {
                        i += 1;
                    }
                }
                At::Code if c[i] == '\'' && c.get(i + 2) == Some(&'\'') => i += 2,
                At::Code => code.push(c[i]),
                At::Comment(n) if two('/', '*') => (at, i) = (At::Comment(n + 1), i + 1),
                At::Comment(n) if two('*', '/') => {
                    at = if n == 1 { At::Code } else { At::Comment(n - 1) };
                    i += 1;
                }
                At::Comment(_) => {}
                At::Str if c[i] == '\\' => i += 1,
                At::Str if c[i] == '"' => at = At::Code,
                At::Str => {}
                At::Raw(n) => {
                    let closes = c[i + 1..].iter().take(n).filter(|x| **x == '#').count() == n;
                    if c[i] == '"' && closes {
                        (at, i) = (At::Code, i + n);
                    }
                }
            }
            i += 1;
        }
        out.push((starts_in_code, code));
    }
    out
}

/// The name of the function that `code` starts with, past its attributes
/// and qualifiers (`pub`, `const`, `async`, `unsafe`, `extern`).
// mitra: implements source::fn_below
pub fn fn_below(code: &str) -> Option<&str> {
    let ident_end = |s: &str| {
        s.find(|ch: char| !(ch.is_alphanumeric() || ch == '_'))
            .unwrap_or(s.len())
    };
    let mut rest = code.trim_start();
    while let Some(r) = rest.strip_prefix('#') {
        let r = r.strip_prefix('!').unwrap_or(r).trim_start();
        let mut depth = 0;
        let end = r.char_indices().find_map(|(i, ch)| {
            match ch {
                '[' => depth += 1,
                ']' => depth -= 1,
                _ => {}
            }
            (depth == 0).then_some(i)
        })?;
        rest = r[end + 1..].trim_start();
    }
    loop {
        let (word, after) = rest.split_at(ident_end(rest));
        let spaced = after.starts_with(char::is_whitespace);
        let after = after.trim_start();
        match word {
            "pub" if after.starts_with('(') => rest = after[after.find(')')? + 1..].trim_start(),
            "extern" => rest = after.strip_prefix("\"\"").unwrap_or(after).trim_start(),
            "pub" | "const" | "async" | "unsafe" | "default" => rest = after,
            "fn" if spaced => {
                let name = after.strip_prefix("r#").unwrap_or(after);
                return Some(&name[..ident_end(name)]).filter(|n| !n.is_empty());
            }
            _ => return None,
        }
    }
}

/// A comment that ties code to a model kept aside: not a line of model.
/// `implements f` above a function, `evidence for c` above a test.
// mitra: implements source::is_tie
pub fn is_tie(m: &str) -> bool {
    m.starts_with("implements ") || m.starts_with("evidence for ")
}

#[cfg(test)]
mod tests {
    use super::*;

    // One input per test, for Mitra.

    // mitra: evidence for t_line_doc
    #[test]
    fn t_line_doc() {
        assert_eq!(
            mitra_line("/// mitra: claim c : correct(f(1))"),
            Some("claim c : correct(f(1))")
        );
    }

    // mitra: evidence for t_line_plain
    #[test]
    fn t_line_plain() {
        assert_eq!(
            mitra_line("    // mitra: claim c : correct(f(1))"),
            Some("claim c : correct(f(1))")
        );
    }

    // mitra: evidence for t_line_keeps_indent
    #[test]
    fn t_line_keeps_indent() {
        assert_eq!(mitra_line("// mitra:   then 1"), Some("  then 1"));
    }

    // mitra: evidence for t_line_comment
    #[test]
    fn t_line_comment() {
        assert_eq!(mitra_line("// a comment about mitra: no"), None);
    }

    // mitra: evidence for t_line_code
    #[test]
    fn t_line_code() {
        assert_eq!(
            mitra_line("let x = 1; // mitra: claim c : correct(f(1))"),
            None
        );
    }

    // mitra: evidence for t_below_plain
    #[test]
    fn t_below_plain() {
        assert_eq!(fn_below("fn f() {}"), Some("f"));
    }

    // mitra: evidence for t_below_qualified
    #[test]
    fn t_below_qualified() {
        assert_eq!(fn_below("pub(crate) const unsafe fn f() {}"), Some("f"));
    }

    // mitra: evidence for t_below_attribute
    #[test]
    fn t_below_attribute() {
        assert_eq!(
            fn_below("#[cfg_attr(test, allow(unused))]\npub async fn f<T>() {}"),
            Some("f")
        );
    }

    // mitra: evidence for t_below_struct
    #[test]
    fn t_below_struct() {
        assert_eq!(fn_below("pub struct S;\nfn f() {}"), None);
    }

    // mitra: evidence for t_below_nothing
    #[test]
    fn t_below_nothing() {
        assert_eq!(fn_below(""), None);
    }

    // mitra: evidence for source::implements_is_a_tie
    #[test]
    fn implements_is_a_tie() {
        assert!(is_tie("implements a::f"));
    }

    // mitra: evidence for source::evidence_for_is_a_tie
    #[test]
    fn evidence_for_is_a_tie() {
        assert!(is_tie("evidence for a::c given x = 1 gives 2"));
    }

    // mitra: evidence for source::a_line_of_model_is_no_tie
    #[test]
    fn a_line_of_model_is_no_tie() {
        assert!(!is_tie("subject fn f() -> Bool;"));
    }
}

/// A `mitra:` comment that is no tie: refused, since the model is kept
/// aside and a comment only ties code to it.
pub fn stray(shown: &str, text: &str) -> Result<(), String> {
    let scanned = scan(text);
    let other = |(i, l): (usize, &str)| {
        let model = mitra_line(l).is_some_and(|m| !is_tie(m));
        (scanned[i].0 && model).then_some(i + 1)
    };
    match text.lines().enumerate().find_map(other) {
        Some(n) => Err(format!(
            "{shown}: line {n}: a model in `mitra:` comments is no longer read: \
             write it in `.mitra/model.mitra`; a comment is `// mitra: implements \
             <function>` or `// mitra: evidence for <claim>`"
        )),
        None => Ok(()),
    }
}
