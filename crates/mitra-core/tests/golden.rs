//! The validation harness: each folder in tests/golden holds a claim model,
//! an evidence list and the result Mitra must produce for them.

use std::{fs, path::Path};

#[test]
fn golden() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden");
    let mut dirs: Vec<_> = fs::read_dir(&root)
        .unwrap()
        .map(|d| d.unwrap().path())
        .collect();
    dirs.sort();
    assert!(!dirs.is_empty());
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
