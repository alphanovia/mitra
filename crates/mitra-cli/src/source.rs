//! The model's text: the `.mitra` files of a store.
//!
//! In the Rust files next to it, a comment line `// mitra: …` only ties
//! the code right below it to the model: `implements <function>` above a
//! `fn`, `evidence for <claim>` above a test.

use std::path::{Path, PathBuf};

use mitra_rust::source::{is_tie, mitra_line, rust_files, scan};

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
        // `import a::b::name;` reads `a/b.mitra` beside the model.
        let dir = Path::new(path)
            .parent()
            .filter(|d| !d.as_os_str().is_empty());
        mitra_lang::parse::imports_from(Some(dir.unwrap_or(Path::new("."))));
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

    /// Check the `mitra:` comments of every `.rs` file under `dirs`: each
    /// is a tie.
    pub fn add_rust(&mut self, dirs: &[PathBuf]) -> Result<(), String> {
        let mut files = Vec::new();
        for d in dirs {
            rust_files(d, &mut files);
        }
        files.sort();
        for f in files {
            let shown = f.display().to_string();
            let text = std::fs::read_to_string(&f).map_err(|e| format!("{shown}: {e}"))?;
            // The model is kept aside: a comment only ties code to it.
            let scanned = scan(&text);
            let other = |(i, l): (usize, &str)| {
                let model = mitra_line(l).is_some_and(|m| !is_tie(m));
                (scanned[i].0 && model).then_some(i + 1)
            };
            if let Some(n) = text.lines().enumerate().find_map(other) {
                return Err(format!(
                    "{shown}: line {n}: a model in `mitra:` comments is no longer read: \
                     write it in `.mitra/model.mitra`; a comment is `// mitra: implements \
                     <function>` or `// mitra: evidence for <claim>`"
                ));
            }
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

    pub fn locate(&self, err: &mitra_lang::Error) -> String {
        match self.lines.get(err.line.wrapping_sub(1)) {
            Some((path, n)) => format!("{path}: line {n}: {}", err.message),
            None => format!("{}: {}", self.first, err.message),
        }
    }
}
