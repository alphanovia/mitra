//! What the cold tests of `cold_fable*.rs` share: fresh folders and stores.
#![allow(dead_code)]

use mitra_store::Store;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

pub static N: AtomicUsize = AtomicUsize::new(0);

/// A fresh folder of this test alone.
pub fn fresh(name: &str) -> PathBuf {
    let n = N.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!(
        "mitra-store-cold-{name}-{}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root.canonicalize().unwrap()
}

pub fn path(p: &Path) -> String {
    p.display().to_string()
}

/// A store made by hand in `root`: a model and an empty ledger.
pub fn store_in(root: &Path, model: &str) -> Store {
    let dir = root.join("model");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("model.mitra"), model).unwrap();
    fs::write(dir.join("evidence.mitra"), "").unwrap();
    Store::of(&path(&dir.join("model.mitra"))).expect("a model.mitra in a .mitra folder")
}

pub fn ledger(s: &Store) -> String {
    fs::read_to_string(s.evidence_path()).unwrap()
}

pub const CALC: &str = "subject calc {\n    fn add(a: Int, b: Int) -> Int;\n    fn neg(a: Int) -> Int;\n    claim commutes(a: Int, b: Int) { add(a, b) == add(b, a) }\n}\n";

pub fn pinned(test: &str, a: i64, rev: &str) -> String {
    format!(
        "evidence t {{ for calc::add given a = {a}, b = 2 gives {} source \"test:{test}\" rev \"{rev}\" }}",
        a + 2
    )
}

pub fn who(s: &str) -> Option<String> {
    Some(s.to_string())
}
