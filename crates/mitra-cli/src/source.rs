//! The model's text: a `.mitra` file, plus, in a store, the `mitra:`
//! comments of the Rust files next to it.
//!
//! In Rust, a comment line `/// mitra: …` or `// mitra: …` holds one line
//! of model. Consecutive such lines form a block, which belongs to the code
//! right below it: an `opaque f` or a `test t` in a block must sit above
//! `fn f` or `fn t`, so the model and the code stay aligned.

use std::path::{Path, PathBuf};

/// Model text gathered from several files, with where each line came from.
pub struct Source {
    pub text: String,
    /// For each line of `text`: its file and line.
    lines: Vec<(String, usize)>,
    first: String,
}

impl Source {
    pub fn file(path: &str) -> Result<Source, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let mut s = Source {
            text: String::new(),
            lines: Vec::new(),
            first: path.to_string(),
        };
        s.push_lines(
            path,
            text.lines()
                .enumerate()
                .map(|(i, l)| (i + 1, l.to_string())),
        );
        Ok(s)
    }

    /// Add another model file after the first.
    pub fn add_file(&mut self, path: &str) -> Result<(), String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
        let lines = text.lines().enumerate();
        self.push_lines(path, lines.map(|(i, l)| (i + 1, l.to_string())));
        Ok(())
    }

    fn push_lines(&mut self, path: &str, lines: impl Iterator<Item = (usize, String)>) {
        for (n, l) in lines {
            self.text += &l;
            self.text.push('\n');
            self.lines.push((path.to_string(), n));
        }
    }

    /// Add the `mitra:` lines of every `.rs` file under `dirs`, in path
    /// order.
    pub fn add_rust(&mut self, dirs: &[PathBuf]) -> Result<(), String> {
        let mut files = Vec::new();
        for d in dirs {
            rust_files(d, &mut files);
        }
        files.sort();
        for f in files {
            let shown = f.display().to_string();
            let text = std::fs::read_to_string(&f).map_err(|e| format!("{shown}: {e}"))?;
            let lines = extract(&text).map_err(|(n, m)| format!("{shown}: line {n}: {m}"))?;
            self.push_lines(&shown, lines.into_iter());
        }
        Ok(())
    }

    /// An error in `text`, at its own file and line. An error past the
    /// last line names the first file.
    /// A line of the model, as `file:line`.
    pub fn place(&self, line: usize) -> String {
        match self.lines.get(line.wrapping_sub(1)) {
            Some((path, n)) => format!("{path}:{n}"),
            None => self.first.clone(),
        }
    }

    pub fn locate(&self, err: &mitra_core::Error) -> String {
        match self.lines.get(err.line.wrapping_sub(1)) {
            Some((path, n)) => format!("{path}: line {n}: {}", err.message),
            None => format!("{}: {}", self.first, err.message),
        }
    }
}

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

/// The words of a model line, outside its strings and its `--` comment.
// mitra: implements source::words
fn words(line: &str) -> Vec<&str> {
    // The pieces of the line outside its strings; in a string, `\` takes
    // the next character with it.
    let (mut code, mut start) = (Vec::new(), Some(0));
    let mut chars = line.char_indices();
    while let Some((i, ch)) = chars.next() {
        match (start, ch) {
            (Some(s), '"') => {
                code.push(&line[s..i]);
                start = None;
            }
            (Some(s), '-') if line[i..].starts_with("--") => {
                code.push(&line[s..i]);
                start = None;
                break;
            }
            (None, '\\') => drop(chars.next()),
            (None, '"') => start = Some(i + 1),
            _ => {}
        }
    }
    code.extend(start.map(|s| &line[s..]));
    code.into_iter()
        .flat_map(|part| part.split(|ch: char| !(ch.is_alphanumeric() || ch == '_')))
        .filter(|w| !w.is_empty())
        .collect()
}

/// The model lines of a Rust file, numbered. A model line is a whole-line
/// comment in code, not text inside a string or a block comment. Checks
/// that each `opaque f` or `test t` starts its line and sits above `fn f`
/// or `fn t`.
// mitra: implements source::extract
pub fn extract(text: &str) -> Result<Vec<(usize, String)>, (usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let scanned = scan(text);
    let is_model = |i: usize| {
        scanned[i].0 && mitra_line(lines[i]).is_some_and(|m| !m.starts_with("implements "))
    };
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if !is_model(i) {
            i += 1;
            continue;
        }
        // A block: consecutive comment lines, some of them model lines.
        let mut names = Vec::new();
        while i < lines.len() && scanned[i].0 && lines[i].trim_start().starts_with("//") {
            // A tie to a model kept aside is not a line of model.
            let tie = mitra_line(lines[i]).is_some_and(|m| m.starts_with("implements "));
            if let Some(m) = mitra_line(lines[i]).filter(|_| !tie) {
                let w = words(m);
                for (at, word) in w.iter().enumerate() {
                    if !matches!(*word, "opaque" | "test") {
                        continue;
                    }
                    if at > 0 {
                        return Err((i + 1, format!("`{word}` must start its line")));
                    }
                    if let Some(name) = w.get(1) {
                        names.push((i + 1, *word, name.to_string()));
                    }
                }
                out.push((i + 1, m.to_string()));
            }
            i += 1;
        }
        if names.is_empty() {
            continue;
        }
        // The code the block belongs to.
        let code: Vec<&str> = scanned[i..].iter().map(|l| l.1.as_str()).collect();
        let code = code.join("\n");
        let below = fn_below(&code);
        for (n, kw, name) in names {
            if below != Some(name.as_str()) {
                let found = below.map_or("no function".to_string(), |b| format!("`fn {b}`"));
                return Err((
                    n,
                    format!("`{kw} {name}` must sit above `fn {name}`, found {found}"),
                ));
            }
        }
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    // One input per test, for Mitra: each `mitra: test` line gives the
    // output the test checks.

    #[test]
    fn t_line_doc() {
        assert_eq!(
            mitra_line("/// mitra: claim c : correct(f(1))"),
            Some("claim c : correct(f(1))")
        );
    }

    #[test]
    fn t_line_plain() {
        assert_eq!(
            mitra_line("    // mitra: claim c : correct(f(1))"),
            Some("claim c : correct(f(1))")
        );
    }

    #[test]
    fn t_line_keeps_indent() {
        assert_eq!(mitra_line("// mitra:   then 1"), Some("  then 1"));
    }

    #[test]
    fn t_line_comment() {
        assert_eq!(mitra_line("// a comment about mitra: no"), None);
    }

    #[test]
    fn t_line_code() {
        assert_eq!(
            mitra_line("let x = 1; // mitra: claim c : correct(f(1))"),
            None
        );
    }

    #[test]
    fn t_below_plain() {
        assert_eq!(fn_below("fn f() {}"), Some("f"));
    }

    #[test]
    fn t_below_qualified() {
        assert_eq!(fn_below("pub(crate) const unsafe fn f() {}"), Some("f"));
    }

    #[test]
    fn t_below_attribute() {
        assert_eq!(
            fn_below("#[cfg_attr(test, allow(unused))]\npub async fn f<T>() {}"),
            Some("f")
        );
    }

    #[test]
    fn t_below_struct() {
        assert_eq!(fn_below("pub struct S;\nfn f() {}"), None);
    }

    #[test]
    fn t_below_nothing() {
        assert_eq!(fn_below(""), None);
    }

    #[test]
    fn t_words_plain() {
        assert_eq!(
            words("opaque f(a: Int) -> Int"),
            ["opaque", "f", "a", "Int", "Int"]
        );
    }

    #[test]
    fn t_words_comment() {
        assert_eq!(words("claim c -- a test of opaque things"), ["claim", "c"]);
    }

    #[test]
    fn t_words_string() {
        assert_eq!(
            words("claim c : correct(g(\"test -- \\\" opaque\")) -- x"),
            ["claim", "c", "correct", "g"]
        );
    }

    #[test]
    fn t_extract_one() {
        assert_eq!(
            extract("// mitra: claim c : correct(f(1))"),
            Ok(vec![(1, "claim c : correct(f(1))".to_string())])
        );
    }

    #[test]
    fn t_extract_none() {
        assert_eq!(extract("// a comment about mitra: no"), Ok(vec![]));
    }

    #[test]
    fn t_extract_aligned() {
        assert_eq!(
            extract("// mitra: opaque f(a: Bool) -> Bool\n#[inline] pub fn f() {}"),
            Ok(vec![(1, "opaque f(a: Bool) -> Bool".to_string())])
        );
    }

    #[test]
    fn t_extract_misaligned() {
        assert_eq!(
            extract("// mitra: opaque add(a: Int) -> Int\nfn sub() {}"),
            Err((
                1,
                "`opaque add` must sit above `fn add`, found `fn sub`".to_string()
            ))
        );
    }

    #[test]
    fn t_extract_alone() {
        assert_eq!(
            extract("// mitra: test t : correct(f(1))"),
            Err((
                1,
                "`test t` must sit above `fn t`, found no function".to_string()
            ))
        );
    }

    #[test]
    fn t_extract_in_a_string() {
        assert_eq!(
            extract("const A: &str = \"\n// mitra: claim a : correct(f(1))\n\";"),
            Ok(vec![])
        );
    }

    #[test]
    fn reads_blocks_and_checks_alignment() {
        let text = "/// Adds.\n\
                    /// mitra: opaque add(a: Int, b: Int) -> Int\n\
                    /// mitra: claim add_ok : forall a, forall b, correct(add(a, b))\n\
                    #[inline]\n\
                    pub fn add(a: i64, b: i64) -> i64 { a + b }\n\
                    // mitra: test t_add : correct(add(1, 2))\n\
                    #[test]\n\
                    fn t_add() {}\n";
        let got = extract(text).unwrap();
        assert_eq!(
            got,
            vec![
                (2, "opaque add(a: Int, b: Int) -> Int".to_string()),
                (
                    3,
                    "claim add_ok : forall a, forall b, correct(add(a, b))".to_string()
                ),
                (6, "test t_add : correct(add(1, 2))".to_string()),
            ]
        );
        let err = extract("// mitra: opaque add(a: Int) -> Int\nfn sub() {}\n").unwrap_err();
        assert_eq!(err.0, 1);
        assert!(err.1.contains("found `fn sub`"), "{}", err.1);
        let err = extract("// mitra: test t (a : Int) : Int\n").unwrap_err();
        assert!(err.1.contains("no function"), "{}", err.1);
    }

    #[test]
    fn is_not_fooled_by_strings_comments_and_attributes() {
        // Not model lines: inside a string, a raw string, a block comment.
        let text = "const A: &str = \"\n// mitra: claim a : correct(f(1))\n\";\n\
                    const B: &str = r#\"\n// mitra: claim b : correct(f(1))\n\"#;\n\
                    /*\n// mitra: claim c : correct(f(1))\n*/\n\
                    let q = '\"'; // mitra: not at the line's start\n";
        assert_eq!(extract(text).unwrap(), vec![]);
        // The function below is the first item, whatever follows it.
        let above = "// mitra: opaque f(a: Bool) -> Bool\n";
        for wrong in [
            "#[inline] pub fn g() {}\npub fn f() {}\n",
            "const X: &str = \"see fn f below\";\n",
            "pub struct S;\nfn f() {}\n",
        ] {
            assert!(extract(&format!("{above}{wrong}")).is_err(), "{wrong}");
        }
        for right in [
            "#[inline] pub fn f() {}\n",
            "#[cfg_attr(\n  test,\n  allow(unused)\n)]\n/* note */\npub(crate) const unsafe fn f() {}\n",
            "\n// a comment\npub async fn f<T>() {}\n",
        ] {
            assert!(extract(&format!("{above}{right}")).is_ok(), "{right}");
        }
        // `opaque` and `test` start their line; comments and strings are free.
        let hidden = "// mitra: claim c : correct(g(true)) opaque g(a: Bool) -> Bool\nstruct S;\n";
        assert!(extract(hidden)
            .unwrap_err()
            .1
            .contains("must start its line"));
        let free = "// mitra: -- a test of opaque things\n\
                    // mitra: claim c : correct(g(\"test\"))\nstruct S;\n";
        assert_eq!(extract(free).unwrap().len(), 2);
    }
}
