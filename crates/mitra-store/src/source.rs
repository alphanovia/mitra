//! Model files, read from the disk and handed to `mitra-lang` as named
//! texts, with the modules they import.

use std::path::{Path, PathBuf};

use mitra_lang::{Handed, Module, Modules};

/// The text of a file.
// mitra: implements files::read
pub fn read(path: &str) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))
}

/// One model file, with what it imports from beside it.
// mitra: implements files::file
pub fn file(path: &str) -> Result<Modules, String> {
    files(&[PathBuf::from(path)])
}

/// The files of one model, the first one saying where imports are read:
/// `import a::b::name;` reads `a/b.mitra` beside it.
pub fn files(paths: &[PathBuf]) -> Result<Modules, String> {
    let mut read_files = Vec::new();
    for p in paths {
        let name = p.display().to_string();
        let text = read(&name)?;
        read_files.push(Module { name, text });
    }
    let dir = paths
        .first()
        .and_then(|p| p.parent())
        .filter(|d| !d.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    let mut handed = Handed::default();
    let mut todo: Vec<String> = read_files
        .iter()
        .flat_map(|f| mitra_lang::parse::imports(&f.text))
        .collect();
    while let Some(name) = todo.pop() {
        if handed.modules.contains_key(&name) || handed.missing.contains_key(&name) {
            continue;
        }
        let mut file = dir.to_path_buf();
        file.extend(name.split("::"));
        file.set_extension("mitra");
        match std::fs::read_to_string(&file) {
            Ok(text) => {
                todo.extend(mitra_lang::parse::imports(&text));
                handed.modules.insert(name, text);
            }
            // Said when the import is read, at its line.
            Err(e) => {
                handed
                    .missing
                    .insert(name, format!("{}: {e}", file.display()));
            }
        }
    }
    Ok(Modules::new(read_files, handed))
}
