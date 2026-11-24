//! The project on disk: its root, and its `.mitra` files, as texts for
//! `mitra_lang::parse::load`.

use mitra_lang::parse::Project;
use std::path::{Path, PathBuf};

/// A file's text, or why it cannot be read.
pub fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}

/// The project's root: the nearest folder up from the model's holding
/// `mitra.toml`, else the root of its git work tree, else the model's
/// folder (the user's choice, 12:21).
pub fn root(model: &Path) -> PathBuf {
    let full = std::fs::canonicalize(model).unwrap_or_else(|_| model.to_path_buf());
    let here = full
        .parent()
        .map(Path::to_path_buf)
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let up = |marker: &str| {
        here.ancestors()
            .find(|d| d.join(marker).exists())
            .map(Path::to_path_buf)
    };
    up("mitra.toml")
        .or_else(|| up(".git"))
        .unwrap_or(here.clone())
}

/// Every `.mitra` file under `root`, by its path from it, but what the
/// root's `.gitignore` leaves out and hidden folders; and the model's
/// own path, when it is under the root.
pub fn project(root: &Path, model: &Path) -> Project {
    let ignored = ignored(root);
    let mut files = vec![];
    let mut todo = vec![root.to_path_buf()];
    while let Some(dir) = todo.pop() {
        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };
        let mut entries: Vec<PathBuf> = entries.flatten().map(|e| e.path()).collect();
        entries.sort();
        for path in entries {
            let Ok(rel) = path.strip_prefix(root) else {
                continue;
            };
            let rel = rel.to_string_lossy().replace('\\', "/");
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_default();
            if name.starts_with('.') || ignored.iter().any(|p| p.ignores(&rel, &name)) {
                continue;
            }
            if path.is_dir() {
                todo.push(path);
            } else if name.ends_with(".mitra") {
                if let Ok(text) = std::fs::read_to_string(&path) {
                    files.push((rel, text));
                }
            }
        }
    }
    let model = std::fs::canonicalize(model).ok().and_then(|m| {
        let root = std::fs::canonicalize(root).ok()?;
        let rel = m.strip_prefix(root).ok()?;
        Some(rel.to_string_lossy().replace('\\', "/"))
    });
    Project { files, model }
}

/// A line of `.gitignore`, as far as Mitra reads one: a name (`target`),
/// a path from the root (`/build`, `docs/out`), each with `*` for any
/// letters; a trailing `/` is dropped. Negation (`!`) and `**` are not
/// read: such a line ignores nothing.
struct Ignore {
    pattern: String,
    rooted: bool,
}

impl Ignore {
    fn ignores(&self, rel: &str, name: &str) -> bool {
        match self.rooted {
            true => glob(&self.pattern, rel),
            false => glob(&self.pattern, name),
        }
    }
}

fn ignored(root: &Path) -> Vec<Ignore> {
    let text = std::fs::read_to_string(root.join(".gitignore")).unwrap_or_default();
    text.lines()
        .map(str::trim)
        .filter(|l| {
            !l.is_empty() && !l.starts_with('#') && !l.starts_with('!') && !l.contains("**")
        })
        .map(|l| {
            let l = l.trim_end_matches('/');
            let rooted = l.contains('/');
            Ignore {
                pattern: l.trim_start_matches('/').to_string(),
                rooted,
            }
        })
        .collect()
}

/// `text` is matched by `pattern`, where `*` stands for any letters but `/`.
fn glob(pattern: &str, text: &str) -> bool {
    match pattern.split_once('*') {
        None => pattern == text,
        Some((head, tail)) => {
            let Some(rest) = text.strip_prefix(head) else {
                return false;
            };
            (0..=rest.len())
                .filter(|i| rest.is_char_boundary(*i) && !rest[..*i].contains('/'))
                .any(|i| glob(tail, &rest[i..]))
        }
    }
}
