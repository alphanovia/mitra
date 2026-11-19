//! The store on a real disk: each test works in a folder of its own.
//!
//! A test that needs a disk pins no input: it is tied to the claim that
//! the function it exercises is correct, where a pass only `passes` and
//! a failure shows.

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
// mitra: evidence for store::Store::source::correct
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
// mitra: evidence for ledger::Store::add::correct
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
// mitra: evidence for ledger::Store::append::correct
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
// mitra: evidence for files::file::correct
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

// mitra: evidence for files::file::correct
#[test]
fn another_store_is_read_beside_this_one() {
    let (root, _) = store("beside", "");
    let (a, b) = (root.join("one_a/.mitra"), root.join("two/.mitra"));
    fs::create_dir_all(&a).unwrap();
    fs::create_dir_all(&b).unwrap();
    fs::write(a.join("model.mitra"), "// index\n").unwrap();
    let there = "subject kept { ensure whole = \"All of it.\"; record R { n: Int } }\n";
    fs::write(a.join("more.mitra"), there).unwrap();
    fs::write(
        a.join("evidence.mitra"),
        "evidence e { for nothing holds }\n",
    )
    .unwrap();
    let here = "import one_a::kept::{R, whole};\nsubject s { require w = kept::whole; fn f(r: kept::R) -> Int; }\n";
    fs::write(b.join("model.mitra"), here).unwrap();
    fs::write(b.join("evidence.mitra"), "").unwrap();
    let path = b.join("model.mitra").display().to_string();
    let source = Store::of(&path).unwrap().source().unwrap();
    // The whole model of the other store, without its ledger.
    let handed = &source.handed.stores["one_a"];
    assert!(handed.contains("record R") && !handed.contains("for nothing"));
    let model = mitra_lang::load_modules(&source, "")
        .map_err(|e| e.1.message)
        .unwrap();
    let from = model.block_of("kept::R").and_then(|b| b.from.clone());
    assert_eq!(from.as_deref(), Some("one_a"));
    // `one-a` answers `one_a` too.
    fs::rename(root.join("one_a"), root.join("one-a")).unwrap();
    let source = Store::of(&path).unwrap().source().unwrap();
    assert!(source.handed.stores.contains_key("one_a"));
    let _ = fs::remove_dir_all(root);
}

const CALC: &str = "subject calc {\n    fn add(a: Int, b: Int) -> Int;\n    claim commutes(a: Int, b: Int) { add(a, b) == add(b, a) }\n}\n";

/// machine_is_a_snapshot, nothing_leaves_unshown.
// mitra: evidence for keeping::Store::replace::correct
#[test]
fn a_run_replaces_what_its_machine_said() {
    let (root, s) = store("replace", CALC);
    let test = |n: &str, rev: &str| {
        format!("evidence test {{ for calc::add given a = 1, b = 2 gives 3 source \"test:{n}\" rev \"{rev}\" }}")
    };
    s.add(
        "evidence r { for calc::commutes holds }",
        Some("review:ana".into()),
        None,
    )
    .unwrap();
    let first = s
        .replace(
            "test:",
            &format!("{}\n{}", test("one", "r1"), test("two", "r1")),
        )
        .unwrap();
    assert_eq!((first.before, first.written, first.gone.len()), (0, 2, 0));
    // The next run: `one` is observed again, `two` is not.
    let next = s.replace("test:", &test("one", "r2")).unwrap();
    assert_eq!((next.before, next.written), (2, 1));
    assert_eq!(next.revs, ["r1"]);
    assert_eq!(next.gone, [test("two", "r1")]);
    let ledger = fs::read_to_string(s.evidence_path()).unwrap();
    assert!(
        !ledger.contains("r1") && ledger.contains(&test("one", "r2")),
        "{ledger}"
    );
    // What a person said is still there, and so is the head of the file.
    assert!(ledger.contains("source \"review:ana\""), "{ledger}");
    assert!(ledger.starts_with("// Evidence"), "{ledger}");
    // The reader of the code is another machine.
    assert_eq!(s.replace("reader:", "").unwrap().before, 0);
    assert_eq!(fs::read_to_string(s.evidence_path()).unwrap(), ledger);
    let _ = fs::remove_dir_all(root);
}

/// sound_or_untouched.
// mitra: evidence for keeping::Store::replace::correct
#[test]
fn a_refused_run_leaves_the_ledger() {
    let (root, s) = store("refused-run", CALC);
    let good = "evidence test { for calc::add given a = 1, b = 2 gives 3 source \"test:one\" }";
    s.replace("test:", good).unwrap();
    let before = fs::read_to_string(s.evidence_path()).unwrap();
    let e = s.replace(
        "test:",
        "evidence test { for calc::nope holds source \"test:x\" }",
    );
    assert!(e.unwrap_err().contains("no function or claim `calc::nope`"));
    assert_eq!(fs::read_to_string(s.evidence_path()).unwrap(), before);
    let _ = fs::remove_dir_all(root);
}

/// human_is_never_dropped_silently, removal_is_explicit.
// mitra: evidence for keeping::Store::remove::correct
#[test]
fn what_a_person_said_leaves_only_when_removed() {
    use mitra_store::store::Which;
    let (root, s) = store("remove", CALC);
    s.add(
        "evidence r { for calc::commutes holds }",
        Some("review:ana".into()),
        None,
    )
    .unwrap();
    s.add(
        "evidence b { for calc::commutes given a = 1, b = 2 fails }",
        Some("ticket:7".into()),
        None,
    )
    .unwrap();
    s.add(
        "evidence o { for calc::add holds }",
        Some("review:bob".into()),
        None,
    )
    .unwrap();
    // The claim is reworded and the function renamed: all three stay.
    let reworded = CALC
        .replace("add(a, b) == add(b, a)", "sum(b, a) == sum(a, b)")
        .replace("add(", "sum(");
    fs::write(s.model_path(), reworded).unwrap();
    s.replace("test:", "").unwrap();
    let model = s.model().unwrap();
    assert_eq!(model.aside.len(), 3, "{:?}", model.aside);
    assert!(model
        .evidence
        .iter()
        .all(|e| e.source.as_deref() != Some("review:ana")));
    // Removal lists what goes.
    let gone = s.remove(&Which::Orphaned).unwrap();
    assert_eq!(gone.len(), 1);
    assert!(
        gone[0].contains("for calc::add holds source \"review:bob\""),
        "{gone:?}"
    );
    let gone = s.remove(&Which::Source("ticket:7".into())).unwrap();
    assert!(gone[0].contains("fails source \"ticket:7\""), "{gone:?}");
    assert!(s
        .remove(&Which::Stale(Some("calc::add".into())))
        .unwrap()
        .is_empty());
    assert_eq!(
        s.remove(&Which::Stale(Some("commutes".into())))
            .unwrap()
            .len(),
        1
    );
    assert!(s
        .remove(&Which::Piece("commutes".into()))
        .unwrap()
        .is_empty());
    assert!(s.model().unwrap().aside.is_empty());
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for ledger::Store::add::correct
#[test]
fn what_a_person_says_keeps_the_fingerprint_of_its_piece() {
    let (root, s) = store("seen", CALC);
    s.add(
        "evidence r { for calc::commutes holds }",
        Some("review:ana".into()),
        None,
    )
    .unwrap();
    s.add(
        "evidence t { for calc::add given a = 1, b = 2 gives 3 source \"test:x\" }",
        None,
        None,
    )
    .unwrap();
    let ledger = fs::read_to_string(s.evidence_path()).unwrap();
    let lines: Vec<&str> = ledger.lines().collect();
    assert!(lines[1].contains(" seen \""), "{ledger}");
    assert!(!lines[2].contains("seen"), "{ledger}");
    let _ = fs::remove_dir_all(root);
}

/// build_is_written_again.
// mitra: evidence for store::Store::write_build::correct
#[test]
fn a_build_is_written_again() {
    let (root, s) = store("build", MODEL);
    let file = |text: &str| [("model.lock".to_string(), text.to_string())];
    let dir = s.write_build(&file("one\n")).unwrap();
    assert!(dir.ends_with(".mitra/build"), "{dir}");
    s.write_build(&file("two\n")).unwrap();
    let read = fs::read_to_string(root.join(".mitra/build/model.lock")).unwrap();
    assert_eq!(read, "two\n");
    // A build is no model file: the store reads as before.
    assert!(s.model().is_ok());
    let _ = fs::remove_dir_all(root);
}
