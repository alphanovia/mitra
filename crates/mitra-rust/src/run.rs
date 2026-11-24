//! Running the tests, and finding the Rust files to read.

use std::path::{Path, PathBuf};

/// The tests run at `root` by `command` (`[test] command`, `cargo test`
/// unless said), all it prints, its own lines and the tests' in the
/// order printed. A run that built no test binary is an error: cargo's
/// last lines say why.
pub fn cargo_test(root: &Path, command: &[String]) -> Result<String, String> {
    let shown = command.join(" ");
    let file = std::env::temp_dir().join(format!("mitra-cargo-test-{}.txt", std::process::id()));
    let out = std::fs::File::create(&file)
        .map_err(|e| format!("cannot write {}: {e}", file.display()))?;
    let err = out
        .try_clone()
        .map_err(|e| format!("cannot write {}: {e}", file.display()))?;
    let status = std::process::Command::new(&command[0])
        .args(&command[1..])
        .current_dir(root)
        .stdout(out)
        .stderr(err)
        .status()
        .map_err(|e| format!("cannot run `{shown}`: {e}"))?;
    let text = std::fs::read_to_string(&file)
        .map_err(|e| format!("cannot read {}: {e}", file.display()))?;
    let _ = std::fs::remove_file(&file);
    let ran = text.lines().any(|l| l.trim_start().starts_with("Running "));
    if !status.success() && !ran {
        let last: Vec<&str> = text.lines().rev().take(20).collect();
        let last: Vec<&str> = last.into_iter().rev().collect();
        return Err(format!(
            "`{shown}` in {} built no test: {status}\n{}",
            root.display(),
            last.join("\n")
        ));
    }
    Ok(text)
}

/// Every `.rs` file under `dir`, in order.
pub fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if dir.is_file() {
        out.push(dir.to_path_buf());
        return Ok(());
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            rust_files(&p, out)?;
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
    Ok(())
}
