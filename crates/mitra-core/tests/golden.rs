//! The validation harness. Each case folder holds `model.mitra`,
//! `evidence.mitra` and the hand-written `expected.txt` Mitra must produce.
//! Cases live in `golden/<subject>/<case>/`.

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
    mitra_core::core_run(&read("model.mitra"), &evidence)
        .unwrap_or_else(|(_, e)| panic!("{}: {e}", dir.display()))
        .to_text()
}

#[test]
fn golden() {
    for dir in cases() {
        if std::env::var("MITRA_RECORD").is_ok() {
            fs::write(dir.join("expected.txt"), eval(&dir, "")).unwrap();
        }
        let expected = fs::read_to_string(dir.join("expected.txt")).unwrap();
        assert_eq!(eval(&dir, ""), expected, "{}", dir.display());
    }
}
