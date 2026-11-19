//! The store on a real disk: each test works in a folder of its own.

use mitra_store::Store;
use std::fs;
use std::path::PathBuf;

const MODEL: &str = "subject fn f(a: Int) -> Int;\n";

/// A new store in a new folder, with `model` as its model.
fn store(name: &str, model: &str) -> (PathBuf, Store) {
    let root = std::env::temp_dir().join(format!("mitra-store-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    let dir = Store::init(&root).unwrap();
    fs::write(dir.join("model.mitra"), model).unwrap();
    let path = dir.join("model.mitra").display().to_string();
    (root, Store::of(&path).unwrap())
}

// mitra: evidence for store::a_model_file_names_its_store
#[test]
fn a_model_file_names_its_store() {
    assert!(Store::of("a/.mitra/model.mitra").is_some());
}

// mitra: evidence for store::another_file_is_no_store
#[test]
fn another_file_is_no_store() {
    assert!(Store::of("a/model.mitra").is_none());
}

/// whole_folder: every `.mitra` file but the ledger, `model.mitra` first.
#[test]
fn the_model_is_the_whole_folder() {
    let (root, s) = store("folder", MODEL);
    fs::write(root.join(".mitra/more.mitra"), "subject fn g() -> Int;\n").unwrap();
    let m = s.source().unwrap();
    let names: Vec<&str> = m.files().iter().map(|f| f.name.as_str()).collect();
    assert_eq!(names.len(), 2);
    assert!(names[0].ends_with("model.mitra") && names[1].ends_with("more.mitra"));
    assert_eq!(s.model().unwrap().functions.len(), 4);
    let _ = fs::remove_dir_all(root);
}

/// never_overwrites.
#[test]
fn a_store_is_not_created_twice() {
    let (root, _) = store("twice", MODEL);
    assert!(Store::init(&root).is_err());
    let _ = fs::remove_dir_all(root);
}

/// reads_back, keeps_who_and_when: what `add` writes is what is read.
#[test]
fn written_evidence_reads_back() {
    let (root, s) = store("back", MODEL);
    let (who, rev) = (Some("review:me".to_string()), Some("abc".to_string()));
    let added = s.add("evidence r { for f given a = 1 gives 2 }", who, rev);
    assert_eq!(added, Ok(1));
    let text = fs::read_to_string(s.evidence_path()).unwrap();
    assert!(text.contains("for f given a = 1 gives 2"), "{text}");
    let m = s.model().unwrap();
    assert_eq!(m.evidence.len(), 1);
    assert_eq!(m.evidence[0].source.as_deref(), Some("review:me"));
    assert_eq!(m.evidence[0].rev.as_deref(), Some("abc"));
    let _ = fs::remove_dir_all(root);
}

/// sound_or_untouched: evidence the model refuses leaves the file as it was.
#[test]
fn refused_evidence_is_not_written() {
    let (root, s) = store("refused", MODEL);
    let before = fs::read_to_string(s.evidence_path()).unwrap();
    assert!(s
        .add("evidence r { for nothing holds }", None, None)
        .is_err());
    assert_eq!(fs::read_to_string(s.evidence_path()).unwrap(), before);
    let _ = fs::remove_dir_all(root);
}

/// imports_beside, missing_is_said_at_the_import.
#[test]
fn an_import_is_read_beside_the_model() {
    let model = "import lib::util::g;\nsubject fn f() -> Int { g() }\n";
    let (root, s) = store("import", model);
    let e = s.model().unwrap_err();
    assert!(
        e.contains("model.mitra: line 1: no module `lib::util`"),
        "{e}"
    );
    // A folder beside the model: its files are read only when imported.
    fs::create_dir_all(root.join(".mitra/lib")).unwrap();
    fs::write(root.join(".mitra/lib/util.mitra"), "fn g() -> Int { 1 }\n").unwrap();
    assert!(s.model().is_ok(), "{:?}", s.model().err());
    let _ = fs::remove_dir_all(root);
}
