//! The validation harness: each folder in tests/golden holds a claim model,
//! an evidence list and the result Mitra must produce for them. Cases are
//! grouped in subfolders (one per seed claim, e.g. `not/`); any folder that
//! holds a `claims.mitra` is a case.

use std::{
    fs,
    path::{Path, PathBuf},
};

#[test]
fn golden() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden");
    let mut dirs = Vec::new();
    collect(&root, &mut dirs);
    dirs.sort();
    assert!(!dirs.is_empty(), "no golden cases found");
    for dir in dirs {
        let read = |f: &str| fs::read_to_string(dir.join(f)).unwrap();
        let report = mitra_core::run(
            &read("claims.mitra"),
            &read("evidence.mitra"),
            &Default::default(),
        )
        .unwrap_or_else(|(_, e)| panic!("{}: {e}", dir.display()));
        assert_eq!(report.to_json(), read("expected.json"), "{}", dir.display());
    }
}

fn collect(dir: &Path, out: &mut Vec<PathBuf>) {
    if dir.join("claims.mitra").exists() {
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
