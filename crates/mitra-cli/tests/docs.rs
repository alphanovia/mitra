//! The public docs show only Mitra that checks. Each ```mitra block of
//! README.md, docs/getting-started.md and docs/reference/*.md is either
//! lines of an example file (examples/), in order and together,
//! comments and blank lines aside, or a whole model on its own. Every
//! file a block is taken from passes `mitra check` (exit 0: well formed;
//! the findings it shows on purpose are allowed), and every block on its
//! own passes with no finding.

use std::path::{Path, PathBuf};
use std::process::Command;

const ROOT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");

fn files(dir: &Path, ext: &str, out: &mut Vec<PathBuf>) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    for entry in entries {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n != "target") {
                files(&path, ext, out);
            }
        } else if path.extension().is_some_and(|e| e == ext) {
            out.push(path);
        }
    }
}

/// The lines that carry meaning: trimmed, with no blank or comment line.
fn lines(text: &str) -> Vec<String> {
    text.lines()
        .map(|l| without_comment(l).trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with("//"))
        .collect()
}

/// A line without its `// …` comment, a `//` in a string kept.
fn without_comment(line: &str) -> &str {
    let mut in_string = false;
    let bytes = line.as_bytes();
    for i in 0..bytes.len() {
        match bytes[i] {
            b'"' => in_string = !in_string,
            b'/' if !in_string && bytes.get(i + 1) == Some(&b'/') => return &line[..i],
            _ => {}
        }
    }
    line
}

fn blocks(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut current: Option<String> = None;
    for line in text.lines() {
        match &mut current {
            None if line.trim() == "```mitra" => current = Some(String::new()),
            Some(block) if line.trim() == "```" => {
                out.push(std::mem::take(block));
                current = None;
            }
            Some(block) => {
                block.push_str(line);
                block.push('\n');
            }
            None => {}
        }
    }
    assert!(current.is_none(), "a ```mitra block is not closed");
    out
}

fn checks(path: &Path, clean: bool) -> Result<(), String> {
    let out = Command::new(env!("CARGO_BIN_EXE_mitra"))
        .arg("check")
        .arg(path)
        .output()
        .unwrap();
    let said = String::from_utf8_lossy(&out.stdout);
    if out.status.success() && (!clean || said.contains(", 0 findings")) {
        Ok(())
    } else {
        Err(String::from_utf8_lossy(&out.stdout).to_string()
            + &String::from_utf8_lossy(&out.stderr))
    }
}

#[test]
fn every_example_in_the_docs_checks() {
    let root = Path::new(ROOT);
    let mut docs = vec![root.join("README.md"), root.join("docs/getting-started.md")];
    files(&root.join("docs/reference"), "md", &mut docs);
    let mut models = Vec::new();
    files(&root.join("examples"), "mitra", &mut models);
    let models: Vec<(PathBuf, Vec<String>)> = models
        .into_iter()
        .map(|p| {
            let text = std::fs::read_to_string(&p).unwrap();
            (p, lines(&text))
        })
        .collect();

    let scratch = std::env::temp_dir().join(format!("mitra-docs-{}", std::process::id()));
    std::fs::create_dir_all(&scratch).unwrap();
    let mut used: Vec<PathBuf> = Vec::new();
    let mut count = 0;
    for doc in &docs {
        let text = std::fs::read_to_string(doc).unwrap();
        for (i, block) in blocks(&text).into_iter().enumerate() {
            count += 1;
            let wanted = lines(&block);
            assert!(!wanted.is_empty(), "{}: block {i} is empty", doc.display());
            let from = models
                .iter()
                .find(|(_, have)| have.windows(wanted.len()).any(|w| w == wanted.as_slice()));
            match from {
                Some((path, _)) => {
                    if !used.contains(path) {
                        used.push(path.clone());
                    }
                }
                None => {
                    let alone = scratch.join(format!("block{count}.mitra"));
                    std::fs::write(&alone, &block).unwrap();
                    if let Err(e) = checks(&alone, true) {
                        panic!(
                            "{}: block {i} is in no example file, and does not check on its own with no finding:\n{block}\n{e}",
                            doc.display()
                        );
                    }
                }
            }
        }
    }
    for path in &used {
        if let Err(e) = checks(path, false) {
            panic!(
                "{} is shown in the docs and does not check:\n{e}",
                path.display()
            );
        }
    }
    assert!(
        count > 20,
        "only {count} blocks found: is the fence still ```mitra?"
    );
}
