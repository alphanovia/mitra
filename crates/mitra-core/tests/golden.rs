//! The validation harness. Each case folder holds `model.mitra`,
//! `evidence.mitra` and the hand-written `expected.txt` Mitra must produce.
//! Cases live in `golden/<subject>/<policy>/<case>/`; `golden/<subject>/
//! noise.mitra` holds evidence at confidence 0 for that subject.

use std::{
    fs,
    path::{Path, PathBuf},
};

fn cases() -> Vec<PathBuf> {
    fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
        if dir.join("model.mitra").exists() {
            out.push(dir.to_path_buf());
            return;
        }
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                collect(&path, out);
            }
        }
    }
    let mut out = Vec::new();
    collect(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden"),
        &mut out,
    );
    out.sort();
    assert!(!out.is_empty(), "no golden cases found");
    out
}

fn eval(dir: &Path, extra_evidence: &str) -> String {
    let read = |f: &str| fs::read_to_string(dir.join(f)).unwrap();
    let evidence = read("evidence.mitra") + extra_evidence;
    mitra_core::run(&read("model.mitra"), &evidence)
        .unwrap_or_else(|(_, e)| panic!("{}: {e}", dir.display()))
        .to_text()
}

#[test]
fn golden() {
    for dir in cases() {
        let expected = fs::read_to_string(dir.join("expected.txt")).unwrap();
        assert_eq!(eval(&dir, ""), expected, "{}", dir.display());
    }
}

/// Evidence at confidence 0 brings nothing: adding the subject's
/// `noise.mitra` to any case never changes its result.
#[test]
fn distrusted_evidence_changes_nothing() {
    for dir in cases() {
        let noise = dir
            .ancestors()
            .find_map(|d| fs::read_to_string(d.join("noise.mitra")).ok());
        let noise = noise.unwrap_or_else(|| panic!("no noise.mitra above {}", dir.display()));
        assert_eq!(
            eval(&dir, &format!("\n{noise}")),
            eval(&dir, ""),
            "{}",
            dir.display()
        );
    }
}
