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
    pub fn locate(&self, err: &mitra_core::Error) -> String {
        match self.lines.get(err.line.wrapping_sub(1)) {
            Some((path, n)) => format!("{path}: line {n}: {}", err.message),
            None => format!("{}: {}", self.first, err.message),
        }
    }
}

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for e in entries.flatten() {
        let p = e.path();
        if p.is_dir() {
            rust_files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
}

/// The text after `mitra:` in a `///` or `//` comment line.
fn mitra_line(line: &str) -> Option<&str> {
    let rest = line.trim_start();
    let rest = rest
        .strip_prefix("///")
        .or_else(|| rest.strip_prefix("//"))?;
    let rest = rest.trim_start().strip_prefix("mitra:")?;
    Some(rest.strip_prefix(' ').unwrap_or(rest))
}

/// The name `fn name` declares on a code line, if any.
fn fn_name(line: &str) -> Option<&str> {
    let i = line.find("fn ")?;
    if i > 0 && !line[..i].ends_with(' ') {
        return None;
    }
    let rest = &line[i + 3..];
    let end = rest
        .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
        .unwrap_or(rest.len());
    Some(&rest[..end]).filter(|n| !n.is_empty())
}

/// The model lines of a Rust file, numbered. Checks that each `opaque f`
/// or `test t` sits above `fn f` or `fn t`.
pub fn extract(text: &str) -> Result<Vec<(usize, String)>, (usize, String)> {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if mitra_line(lines[i]).is_none() {
            i += 1;
            continue;
        }
        // A block of consecutive `mitra:` lines (other comments may sit
        // between them).
        let mut names = Vec::new();
        while i < lines.len() {
            let l = lines[i].trim_start();
            if let Some(m) = mitra_line(l) {
                let mut words = m.split_whitespace();
                if let (Some(kw @ ("opaque" | "test")), Some(name)) = (words.next(), words.next()) {
                    names.push((i + 1, kw, name.to_string()));
                }
                out.push((i + 1, m.to_string()));
            } else if !l.starts_with("//") {
                break;
            }
            i += 1;
        }
        if names.is_empty() {
            continue;
        }
        // The code the block belongs to: past attributes and comments.
        let code = lines[i..]
            .iter()
            .map(|l| l.trim_start())
            .find(|l| !l.is_empty() && !l.starts_with("#[") && !l.starts_with("//"));
        let below = code.and_then(fn_name);
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

    #[test]
    fn reads_blocks_and_checks_alignment() {
        let text = "/// Adds.\n\
                    /// mitra: opaque add (a b : Int) : Int\n\
                    /// mitra: claim add_ok : forall a, forall b, correct (add a b)\n\
                    #[inline]\n\
                    pub fn add(a: i64, b: i64) -> i64 { a + b }\n\
                    // mitra: test t_add : correct (add 1 2)\n\
                    #[test]\n\
                    fn t_add() {}\n";
        let got = extract(text).unwrap();
        assert_eq!(
            got,
            vec![
                (2, "opaque add (a b : Int) : Int".to_string()),
                (
                    3,
                    "claim add_ok : forall a, forall b, correct (add a b)".to_string()
                ),
                (6, "test t_add : correct (add 1 2)".to_string()),
            ]
        );
        let err = extract("// mitra: opaque add (a : Int) : Int\nfn sub() {}\n").unwrap_err();
        assert_eq!(err.0, 1);
        assert!(err.1.contains("found `fn sub`"), "{}", err.1);
        let err = extract("// mitra: test t (a : Int) : Int\n").unwrap_err();
        assert!(err.1.contains("no function"), "{}", err.1);
    }
}
