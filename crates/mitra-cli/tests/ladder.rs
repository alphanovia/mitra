//! The ladder (`ladder/README.md`): models of growing difficulty, each
//! with its code, its ledger, its expected report and its seeded breaks.
//! Here each rung is read through the levels in a copy, and must say
//! what `model/expected.txt` says. `scripts/rung.sh` records it again.

use std::path::{Path, PathBuf};
use std::process::Command;

fn ladder() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../ladder")
}

fn rungs() -> Vec<PathBuf> {
    let mut all: Vec<PathBuf> = std::fs::read_dir(ladder())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.join("model/model.mitra").exists())
        .collect();
    all.sort();
    all
}

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap().path();
        let name = e.file_name().unwrap();
        if name == ".mitra" {
            continue;
        }
        match e.is_dir() {
            true => copy(&e, &to.join(name)),
            false => drop(std::fs::copy(&e, to.join(name)).unwrap()),
        }
    }
}

/// `mitra assess` on a rung, with the ledger as committed: the model
/// builds, the code conforms, and the report is the expected one.
#[test]
fn each_rung_says_what_is_expected() {
    // The breaks benchmark replaces the evidence: nothing to compare.
    if std::env::var("MITRA_BREAKS").is_ok() {
        return;
    }
    assert!(rungs().len() >= 2);
    for rung in rungs() {
        let name = rung.file_name().unwrap().to_string_lossy().to_string();
        let dir = std::env::temp_dir().join(format!("mitra-ladder-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        copy(&rung, &dir);
        let out = Command::new(env!("CARGO_BIN_EXE_mitra"))
            .arg("assess")
            .current_dir(&dir)
            .output()
            .unwrap();
        let said = String::from_utf8_lossy(&out.stdout).to_string();
        assert!(said.contains("\nconform: ok: "), "{name}: {said}");
        let expected = std::fs::read_to_string(rung.join("model/expected.txt")).unwrap();
        assert_eq!(said, expected, "{name}: run scripts/rung.sh ladder/{name}");
        let _ = std::fs::remove_dir_all(&dir);
    }
}

/// A rung has the five parts, and its breaks name text that is in its
/// code exactly once.
#[test]
fn each_rung_has_its_five_parts() {
    for rung in rungs() {
        for part in ["src/lib.rs", "model/evidence.mitra", "model/expected.txt", "breaks.txt"] {
            assert!(rung.join(part).exists(), "{}: no {part}", rung.display());
        }
        let breaks = std::fs::read_to_string(rung.join("breaks.txt")).unwrap();
        for block in breaks.split("\n==\n") {
            let parts: Vec<&str> = block.trim_matches('\n').split("\n--\n").collect();
            assert!(parts.len() == 3 || parts.len() == 4, "{block}");
            let (name, file) = parts[0].split_once('\n').unwrap();
            let code = std::fs::read_to_string(ladder().join("..").join(file)).unwrap();
            assert_eq!(code.matches(parts[1]).count(), 1, "{name}");
        }
    }
}
