//! Tests written from the model of mitra-store (`model/*.mitra`) alone:
//! each one says what a wrong implementation would get wrong.

use mitra_store::store::Which;
use mitra_store::{source, Store};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

static N: AtomicUsize = AtomicUsize::new(0);

/// A fresh folder of this test alone.
fn fresh(name: &str) -> PathBuf {
    let n = N.fetch_add(1, Ordering::SeqCst);
    let root = std::env::temp_dir().join(format!(
        "mitra-store-cold-{name}-{}-{n}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root.canonicalize().unwrap()
}

fn path(p: &Path) -> String {
    p.display().to_string()
}

/// A store made by hand in `root`: a model and an empty ledger.
fn store_in(root: &Path, model: &str) -> Store {
    let dir = root.join("model");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("model.mitra"), model).unwrap();
    fs::write(dir.join("evidence.mitra"), "").unwrap();
    Store::of(&path(&dir.join("model.mitra"))).expect("a model.mitra in a .mitra folder")
}

fn ledger(s: &Store) -> String {
    fs::read_to_string(s.evidence_path()).unwrap()
}

const CALC: &str = "subject calc {\n    fn add(a: Int, b: Int) -> Int;\n    fn neg(a: Int) -> Int;\n    claim commutes(a: Int, b: Int) { add(a, b) == add(b, a) }\n}\n";

fn pinned(test: &str, a: i64, rev: &str) -> String {
    format!(
        "evidence t {{ for calc::add given a = {a}, b = 2 gives {} source \"test:{test}\" rev \"{rev}\" }}",
        a + 2
    )
}

fn who(s: &str) -> Option<String> {
    Some(s.to_string())
}

// ---------------------------------------------------------------- files

// mitra: evidence for files::read::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_read_gives_the_whole_text() {
    let root = fresh("read");
    let text = "first line\n\n  second, with é and 語\nno newline at the end";
    fs::write(root.join("a.mitra"), text).unwrap();
    assert_eq!(
        source::read(&path(&root.join("a.mitra"))),
        Ok(text.to_string())
    );
    fs::write(root.join("empty.mitra"), "").unwrap();
    assert_eq!(
        source::read(&path(&root.join("empty.mitra"))),
        Ok(String::new())
    );
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for files::read::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_read_of_no_file_says_why_with_its_path() {
    let root = fresh("read-none");
    let p = path(&root.join("not-there.mitra"));
    let e = source::read(&p).unwrap_err();
    assert!(e.contains(&p), "{e}");
    assert!(e.len() > p.len(), "the path alone says not why: {e}");
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for files::handed_unchanged written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_file_is_handed_under_its_path_unchanged() {
    let root = fresh("handed");
    let text = "// a comment kept\n\nsubject fn f(a: Int) -> Int;   \n\n\n// no newline at the end";
    let p = path(&root.join("one.mitra"));
    fs::write(&p, text).unwrap();
    let m = source::file(&p).unwrap();
    assert_eq!(m.files().len(), 1);
    assert_eq!(m.files()[0].name, p);
    assert_eq!(m.files()[0].text, text);
    assert_eq!(m.text(), text);
    assert!(m.handed.modules.is_empty() && m.handed.missing.is_empty());
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for files::file::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_file_that_is_not_there_is_an_error_with_its_path() {
    let root = fresh("file-none");
    let p = path(&root.join("nope.mitra"));
    let e = source::file(&p).map(|_| ()).unwrap_err();
    assert!(e.contains(&p), "{e}");
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for files::imports_beside written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_import_is_answered_by_the_file_beside() {
    let root = fresh("import");
    let lib = "fn g() -> Int { 1 }\n// kept as written";
    fs::create_dir_all(root.join("a")).unwrap();
    fs::write(root.join("a/b.mitra"), lib).unwrap();
    // A file with the name of the item, or flat, is not the answer.
    fs::write(root.join("b.mitra"), "fn g() -> Int { 2 }\n").unwrap();
    let p = path(&root.join("first.mitra"));
    fs::write(&p, "import a::b::g;\nsubject fn f() -> Int { g() }\n").unwrap();
    let m = source::file(&p).unwrap();
    assert_eq!(m.handed.modules.get("a::b").map(String::as_str), Some(lib));
    assert_eq!(m.handed.modules.len(), 1, "{:?}", m.handed.modules);
    assert!(m.handed.missing.is_empty(), "{:?}", m.handed.missing);
    // The file itself is still the only model file.
    assert_eq!(m.files().len(), 1);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for files::imports_beside written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_import_of_an_import_is_read_beside_the_first_file() {
    let root = fresh("import-turn");
    fs::create_dir_all(root.join("a")).unwrap();
    fs::create_dir_all(root.join("c")).unwrap();
    // a/b.mitra imports c::d: answered by c/d.mitra beside the first
    // model file, not by a/c/d.mitra.
    fs::write(
        root.join("a/b.mitra"),
        "import c::d::h;\nfn g() -> Int { h() }\n",
    )
    .unwrap();
    let deep = "fn h() -> Int { 7 }\n";
    fs::write(root.join("c/d.mitra"), deep).unwrap();
    let p = path(&root.join("first.mitra"));
    fs::write(&p, "import a::b::g;\nsubject fn f() -> Int { g() }\n").unwrap();
    let m = source::file(&p).unwrap();
    assert!(m.handed.modules.contains_key("a::b"));
    assert_eq!(m.handed.modules.get("c::d").map(String::as_str), Some(deep));
    assert!(m.handed.missing.is_empty(), "{:?}", m.handed.missing);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for files::missing_is_said_at_the_import written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_missing_import_does_not_stop_the_reading() {
    let root = fresh("missing");
    fs::create_dir_all(root.join("here")).unwrap();
    let there = "fn k() -> Int { 3 }\n";
    fs::write(root.join("here/is.mitra"), there).unwrap();
    let text = "// one\n// two\nimport gone::away::g;\nimport here::is::k;\nsubject fn f() -> Int { k() }\n";
    let p = path(&root.join("first.mitra"));
    fs::write(&p, text).unwrap();
    let m = source::file(&p).expect("a missing import is no error of the reading");
    // The rest is read all the same.
    assert_eq!(m.files()[0].text, text);
    assert_eq!(
        m.handed.modules.get("here::is").map(String::as_str),
        Some(there)
    );
    // Why is handed over, naming the file that was looked for.
    let why = m.handed.missing.get("gone::away").expect("why is handed");
    let looked = path(&root.join("gone").join("away.mitra"));
    assert!(why.contains(&looked), "{why}");
    assert!(!m.handed.modules.contains_key("gone::away"));
    // And said at the line of the import.
    let (_, err) = mitra_lang::load_modules(&m, "").map(|_| ()).unwrap_err();
    assert_eq!(err.line, 3, "{}", err.message);
    assert!(
        m.locate(&err).contains("first.mitra: line 3"),
        "{}",
        m.locate(&err)
    );
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for files::stores_beside written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_import_with_no_file_is_answered_by_the_store_beside() {
    let root = fresh("beside");
    let other = root.join("other/model");
    fs::create_dir_all(&other).unwrap();
    fs::write(other.join("model.mitra"), "// the index of other\n").unwrap();
    fs::write(
        other.join("block.mitra"),
        "subject block { record R { n: Int } }\n",
    )
    .unwrap();
    fs::write(
        other.join("zeta.mitra"),
        "subject zeta { record Z { n: Int } }\n",
    )
    .unwrap();
    fs::write(other.join("evidence.mitra"), "// LEDGER-OF-OTHER\n").unwrap();
    fs::write(other.join("notes.txt"), "NOT-A-MODEL-FILE\n").unwrap();
    fs::create_dir_all(root.join("this")).unwrap();
    let s = store_in(
        &root.join("this"),
        "import other::block::{R};\nsubject s { fn f(r: block::R) -> Int; }\n",
    );
    for m in [s.source().unwrap(), source::file(&s.model_path()).unwrap()] {
        let whole = m.handed.stores.get("other").expect("the store beside");
        assert!(whole.contains("// the index of other"), "{whole}");
        assert!(whole.contains("record R { n: Int }"), "{whole}");
        // The whole model, not only the block asked for.
        assert!(whole.contains("record Z { n: Int }"), "{whole}");
        assert!(!whole.contains("LEDGER-OF-OTHER"), "{whole}");
        assert!(!whole.contains("NOT-A-MODEL-FILE"), "{whole}");
    }
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for files::stores_beside written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_file_beside_the_model_answers_before_another_store() {
    let root = fresh("beside-file");
    let other = root.join("other/model");
    fs::create_dir_all(&other).unwrap();
    fs::write(
        other.join("model.mitra"),
        "subject block { record R { n: Int } }\n",
    )
    .unwrap();
    fs::create_dir_all(root.join("this")).unwrap();
    let s = store_in(
        &root.join("this"),
        "import other::block::{R};\nsubject s { fn f() -> Int; }\n",
    );
    // There is such a file beside the model: the store `other` is not asked.
    let beside = "record R { m: Int }\n";
    fs::create_dir_all(root.join("this/model/other")).unwrap();
    fs::write(root.join("this/model/other/block.mitra"), beside).unwrap();
    let m = s.source().unwrap();
    assert_eq!(
        m.handed.modules.get("other::block").map(String::as_str),
        Some(beside)
    );
    assert!(m.handed.stores.is_empty(), "{:?}", m.handed.stores);
    assert!(m.handed.missing.is_empty(), "{:?}", m.handed.missing);
    let _ = fs::remove_dir_all(root);
}

// ---------------------------------------------------------------- store

// mitra: evidence for store::Store::of::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_of_takes_only_a_model_mitra_in_a_dot_mitra_folder() {
    assert!(Store::of("a/model/model.mitra").is_some());
    assert!(Store::of("model/model.mitra").is_some());
    assert!(Store::of("/x/y/model/model.mitra").is_some());
    assert!(Store::of("a/model.mitra").is_none());
    assert!(Store::of("model.mitra").is_none());
    assert!(Store::of("a/model/other.mitra").is_none());
    assert!(Store::of("a/model/evidence.mitra").is_none());
    assert!(Store::of("a/model").is_none());
    assert!(Store::of("a/model/sub/model.mitra").is_none());
    assert!(Store::of("a/mitra/model.mitra").is_none());
    assert!(Store::of("a/model/xmodel.mitra").is_none());
    assert!(Store::of("").is_none());
}

// mitra: evidence for store::Store::model_path::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_model_path_is_the_file_the_store_was_named_by() {
    let s = Store::of("a/model/model.mitra").unwrap();
    assert_eq!(Path::new(&s.model_path()), Path::new("a/model/model.mitra"));
    // Asked twice, the same.
    assert_eq!(s.model_path(), s.model_path());
}

// mitra: evidence for store::Store::evidence_path::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_evidence_path_is_evidence_mitra_in_the_same_folder() {
    let s = Store::of("a/b/model/model.mitra").unwrap();
    assert_eq!(
        Path::new(&s.evidence_path()),
        Path::new("a/b/model/evidence.mitra")
    );
    // Nothing of the store is read to say so: there is no such folder.
    assert!(!Path::new("a/b/model").exists());
}

// mitra: evidence for store::whole_folder written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_source_is_every_mitra_file_with_the_model_first() {
    let root = fresh("whole");
    let model = "subject fn f(a: Int) -> Int;\n";
    let s = store_in(&root, model);
    let dir = root.join("model");
    // Names before and after `model` in the alphabet, and names that only
    // end like the model and the ledger.
    let early = "subject aaa { fn a1() -> Int; }\n";
    fs::write(dir.join("aaa.mitra"), early).unwrap();
    fs::write(dir.join("zzz.mitra"), "subject zzz { fn z1() -> Int; }\n").unwrap();
    fs::write(
        dir.join("mymodel.mitra"),
        "subject mm { fn m1() -> Int; }\n",
    )
    .unwrap();
    fs::write(
        dir.join("my_evidence.mitra"),
        "subject me { fn e1() -> Int; }\n",
    )
    .unwrap();
    // Not model: the ledger, and files that are no `.mitra` file.
    fs::write(dir.join("evidence.mitra"), "// THE-LEDGER\n").unwrap();
    fs::write(dir.join("notes.txt"), "NOTES\n").unwrap();
    fs::write(dir.join("old.mitra.bak"), "BACKUP\n").unwrap();
    let m = s.source().unwrap();
    let names: Vec<String> = m
        .files()
        .iter()
        .map(|f| {
            Path::new(&f.name)
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string()
        })
        .collect();
    assert_eq!(names[0], "model.mitra", "{names:?}");
    let mut rest = names[1..].to_vec();
    rest.sort();
    assert_eq!(
        rest,
        [
            "aaa.mitra",
            "my_evidence.mitra",
            "mymodel.mitra",
            "zzz.mitra"
        ]
    );
    // Each under its path, with its text.
    assert_eq!(m.files()[0].name, s.model_path());
    assert_eq!(m.files()[0].text, model);
    let aaa = m
        .files()
        .iter()
        .find(|f| f.name.ends_with("aaa.mitra"))
        .unwrap();
    assert_eq!(aaa.name, path(&dir.join("aaa.mitra")));
    assert_eq!(aaa.text, early);
    assert!(!m.text().contains("THE-LEDGER") && !m.text().contains("NOTES"));
    assert!(!m.text().contains("BACKUP"));
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for store::Store::source::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_source_of_a_folder_that_is_not_there_is_an_error() {
    let root = fresh("no-folder");
    let s = Store::of(&path(&root.join("nowhere/model/model.mitra"))).unwrap();
    assert!(s.source().is_err());
    // A folder with no model.mitra has no model to hand first.
    fs::create_dir_all(root.join("half/model")).unwrap();
    fs::write(
        root.join("half/model/more.mitra"),
        "subject fn g() -> Int;\n",
    )
    .unwrap();
    let s = Store::of(&path(&root.join("half/model/model.mitra"))).unwrap();
    let e = s.source().map(|_| ()).unwrap_err();
    assert!(e.contains("model.mitra"), "{e}");
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for store::settings_beside_the_model written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_no_settings_file_is_the_defaults() {
    let root = fresh("settings-none");
    let s = store_in(&root, "subject fn f() -> Int;\n");
    assert_eq!(s.settings(), Ok(mitra_lang::Settings::default()));
    assert_eq!(
        s.source().unwrap().handed.settings,
        mitra_lang::Settings::default()
    );
    // Asking made no file.
    assert!(!root.join("model/settings.toml").exists());
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for store::Store::settings::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_settings_are_read_from_the_file_and_handed_with_the_model() {
    let root = fresh("settings");
    let s = store_in(&root, "subject fn f() -> Int;\n");
    let text = "[sample]\ntolerance = 0.25\ncounts = true\n";
    fs::write(root.join("model/settings.toml"), text).unwrap();
    // A settings file elsewhere is not the store's.
    fs::write(root.join("settings.toml"), "[sample]\ntolerance = 0.5\n").unwrap();
    let read = s.settings().unwrap();
    assert_eq!(read.tolerance, 0.25);
    assert!(read.counts);
    assert_ne!(read, mitra_lang::Settings::default());
    assert_eq!(s.source().unwrap().handed.settings, read);
    // Read again when the file says something else.
    fs::write(
        root.join("model/settings.toml"),
        "[sample]\ntolerance = 0.125\n",
    )
    .unwrap();
    assert_eq!(s.settings().unwrap().tolerance, 0.125);
    assert!(!s.settings().unwrap().counts);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for store::settings_beside_the_model written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_settings_that_do_not_read_are_an_error_with_their_path() {
    let root = fresh("settings-bad");
    let s = store_in(&root, "subject fn f() -> Int;\n");
    let file = root.join("model/settings.toml");
    fs::write(&file, "[sample]\nthis is no setting\n").unwrap();
    let e = s.settings().unwrap_err();
    assert!(e.contains(&path(&file)), "{e}");
    // Handed with the model: the model is not handed with defaults instead.
    let e = s.source().map(|_| ()).unwrap_err();
    assert!(e.contains(&path(&file)), "{e}");
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for store::build_is_written_again written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_build_files_are_written_by_name_under_build() {
    let root = fresh("build");
    let s = store_in(&root, "subject fn f() -> Int;\n");
    let files = vec![
        ("one.lock".to_string(), "ONE\nno newline".to_string()),
        ("two.txt".to_string(), String::new()),
    ];
    let dir = s.write_build(&files).unwrap();
    assert_eq!(
        Path::new(&dir).canonicalize().unwrap(),
        root.join(".mitra/build")
    );
    assert_eq!(
        fs::read_to_string(root.join(".mitra/build/one.lock")).unwrap(),
        "ONE\nno newline"
    );
    assert_eq!(
        fs::read_to_string(root.join(".mitra/build/two.txt")).unwrap(),
        ""
    );
    // The model and the ledger are as they were.
    assert_eq!(
        fs::read_to_string(s.model_path()).unwrap(),
        "subject fn f() -> Int;\n"
    );
    assert_eq!(ledger(&s), "");
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for store::build_is_written_again written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_build_is_never_read_as_model() {
    let root = fresh("build-unread");
    let s = store_in(&root, "subject fn f() -> Int;\n");
    let before: Vec<_> = s.source().unwrap().files().to_vec();
    s.write_build(&[(
        "model.mitra".to_string(),
        "THIS IS NO MODEL {{{\n".to_string(),
    )])
    .unwrap();
    let after = s.source().unwrap();
    assert_eq!(after.files(), &before[..]);
    assert!(!after.text().contains("THIS IS NO MODEL"));
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for store::never_overwrites written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_init_where_there_is_a_store_changes_nothing() {
    let root = fresh("init-twice");
    let s = store_in(&root, "subject fn mine() -> Int;\n");
    fs::write(s.evidence_path(), "// my ledger\n").unwrap();
    assert!(Store::init(&root).is_err());
    assert_eq!(
        fs::read_to_string(s.model_path()).unwrap(),
        "subject fn mine() -> Int;\n"
    );
    assert_eq!(ledger(&s), "// my ledger\n");
    let names: Vec<_> = fs::read_dir(root.join("model")).unwrap().collect();
    assert_eq!(names.len(), 2, "nothing was added to the folder");
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for store::never_overwrites written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_init_where_there_is_none_makes_a_store_once() {
    let root = fresh("init");
    let dir = Store::init(&root).unwrap();
    assert_eq!(dir.canonicalize().unwrap(), root.join("model"));
    let s = Store::of(&path(&dir.join("model.mitra"))).unwrap();
    // A store that reads: a model and a ledger.
    assert!(Path::new(&s.model_path()).is_file());
    assert!(Path::new(&s.evidence_path()).is_file());
    assert!(s.model().is_ok(), "{:?}", s.model().err());
    // The second time there is one.
    fs::write(s.model_path(), "subject fn mine() -> Int;\n").unwrap();
    assert!(Store::init(&root).is_err());
    assert_eq!(
        fs::read_to_string(s.model_path()).unwrap(),
        "subject fn mine() -> Int;\n"
    );
    let _ = fs::remove_dir_all(root);
}

// The only test that moves the current folder: every other test names
// its files in full.
// mitra: evidence for store::nearest written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_find_uses_the_nearest_store_at_or_above() {
    let root = fresh("nearest");
    store_in(&root, "subject fn far() -> Int;\n");
    store_in(&root.join("in"), "subject fn near() -> Int;\n");
    fs::create_dir_all(root.join("in/deep/er")).unwrap();
    fs::create_dir_all(root.join("side/way")).unwrap();
    let back = std::env::current_dir().unwrap();
    let found = |at: &Path| {
        std::env::set_current_dir(at).unwrap();
        let s = Store::find();
        std::env::set_current_dir(&back).unwrap();
        let s = s.unwrap_or_else(|e| panic!("{}: {e}", at.display()));
        fs::read_to_string(s.model_path()).unwrap()
    };
    // Above, with a farther one above that.
    assert_eq!(
        found(&root.join("in/deep/er")),
        "subject fn near() -> Int;\n"
    );
    // At.
    assert_eq!(found(&root.join("in")), "subject fn near() -> Int;\n");
    assert_eq!(found(&root), "subject fn far() -> Int;\n");
    // A store in a folder beside is not above.
    assert_eq!(found(&root.join("side/way")), "subject fn far() -> Int;\n");
    let _ = fs::remove_dir_all(root);
}

// --------------------------------------------------------------- ledger

// mitra: evidence for ledger::reads_back written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_append_adds_at_the_end_and_reads_back() {
    let root = fresh("append");
    let s = store_in(&root, CALC);
    let head = "// the head of the ledger, kept";
    fs::write(s.evidence_path(), head).unwrap();
    let (one, two) = (pinned("one", 1, "r1"), pinned("two", 5, "r1"));
    assert_eq!(s.append(&one), Ok(()));
    let first = ledger(&s);
    assert!(first.starts_with(head), "{first}");
    assert_eq!(first.matches(&one).count(), 1, "{first}");
    assert_eq!(s.append(&two), Ok(()));
    let second = ledger(&s);
    // What was there is untouched, the new text is after it.
    assert!(second.starts_with(&first), "{second}");
    assert_eq!(second[first.len()..].trim(), two);
    // Each entry on lines of its own, and read as two.
    let lines: Vec<&str> = second.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines, [head, one.as_str(), two.as_str()]);
    assert_eq!(s.model().unwrap().evidence.len(), 2);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for ledger::Store::add::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_add_gives_how_many_pieces_were_added() {
    let root = fresh("add-count");
    let s = store_in(&root, CALC);
    let two = format!("{}\n{}", pinned("one", 1, "r1"), pinned("two", 5, "r1"));
    assert_eq!(s.add(&two, None, None), Ok(2));
    assert_eq!(s.model().unwrap().evidence.len(), 2);
    // The second call counts what it added, not what is there.
    assert_eq!(s.add(&pinned("three", 9, "r1"), None, None), Ok(1));
    assert_eq!(s.model().unwrap().evidence.len(), 3);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for ledger::keeps_who_and_when written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_add_writes_who_and_which_revision_into_each_entry() {
    let root = fresh("add-who");
    let s = store_in(&root, CALC);
    let text = "evidence r { for calc::commutes holds }\nevidence q { for calc::neg holds }";
    assert_eq!(s.add(text, who("review:ana"), who("abc123")), Ok(2));
    // A later entry by someone else, with no revision.
    assert_eq!(
        s.add(
            "evidence r { for calc::add holds }",
            who("review:bob"),
            None
        ),
        Ok(1)
    );
    let m = s.model().unwrap();
    let got: Vec<(&str, Option<&str>, Option<&str>)> = m
        .evidence
        .iter()
        .map(|e| (e.about.as_str(), e.source.as_deref(), e.rev.as_deref()))
        .collect();
    assert_eq!(got.len(), 3, "{got:?}");
    assert!(
        got.contains(&("calc::commutes", Some("review:ana"), Some("abc123"))),
        "{got:?}"
    );
    assert!(
        got.contains(&("calc::neg", Some("review:ana"), Some("abc123"))),
        "{got:?}"
    );
    let bob = got.iter().find(|g| g.0 == "calc::add").unwrap();
    assert_eq!((bob.1, bob.2), (Some("review:bob"), None));
    // It is in the file, not only in what was read.
    let text = ledger(&s);
    assert_eq!(text.matches("review:ana").count(), 2, "{text}");
    assert_eq!(text.matches("abc123").count(), 2, "{text}");
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for ledger::sound_or_untouched written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_refused_evidence_writes_nothing() {
    let root = fresh("refused");
    let s = store_in(&root, CALC);
    s.add(&pinned("one", 1, "r1"), None, None).unwrap();
    let before = ledger(&s);
    // A piece the model does not have, after one it has: neither is written.
    let mixed = format!(
        "{}\nevidence t {{ for calc::nope given a = 1 gives 2 source \"test:x\" }}",
        pinned("two", 5, "r1")
    );
    assert!(s.add(&mixed, None, None).is_err());
    assert_eq!(ledger(&s), before);
    // Text that does not read.
    assert!(s.append("evidence t { for calc::add given").is_err());
    assert_eq!(ledger(&s), before);
    assert!(s
        .add("evidence t { for calc::add given", who("review:ana"), None)
        .is_err());
    assert_eq!(ledger(&s), before);
    // The store still takes what is sound.
    assert_eq!(s.add(&pinned("two", 5, "r1"), None, None), Ok(1));
    let _ = fs::remove_dir_all(root);
}

// New evidence on a piece the model does not have is refused (mitra-lang,
// `ledger::orphaned_judges_nothing`); `append` is a door of its own for
// new evidence, and checks the store with it before it writes.

// mitra: evidence for ledger::Store::append::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_append_to_a_store_that_does_not_read_writes_nothing() {
    let root = fresh("unsound");
    let s = store_in(&root, CALC);
    s.append(&pinned("one", 1, "r1")).unwrap();
    let before = ledger(&s);
    // The model no longer reads.
    fs::write(
        s.model_path(),
        "subject calc { fn add(a: Int, b: Int) -> ;;; {{{\n",
    )
    .unwrap();
    assert!(s.append(&pinned("two", 5, "r1")).is_err());
    assert_eq!(ledger(&s), before);
    assert!(s.add(&pinned("two", 5, "r1"), None, None).is_err());
    assert_eq!(ledger(&s), before);
    // The model reads again, the ledger does not.
    fs::write(s.model_path(), CALC).unwrap();
    let broken = format!("{before}evidence t {{ for calc::add given\n");
    fs::write(s.evidence_path(), &broken).unwrap();
    assert!(s.append(&pinned("two", 5, "r1")).is_err());
    assert_eq!(ledger(&s), broken);
    let _ = fs::remove_dir_all(root);
}

// -------------------------------------------------------------- keeping

// mitra: evidence for keeping::Store::replace::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_replace_takes_out_the_entries_of_the_prefix_and_writes_the_new() {
    let root = fresh("replace");
    let s = store_in(&root, CALC);
    let reader =
        "evidence t { for calc::neg given a = 1 gives -1 source \"reader:rust\" rev \"r0\" }";
    s.add(&pinned("one", 1, "r1"), None, None).unwrap();
    s.add(reader, None, None).unwrap();
    s.add(&pinned("two", 5, "r2"), None, None).unwrap();
    // `one` is observed again on another revision, `two` is not, `three` is new.
    let new = format!("{}\n{}", pinned("one", 1, "r3"), pinned("three", 9, "r3"));
    let r = s.replace("test:", &new).unwrap();
    assert_eq!(r.before, 2);
    assert_eq!(r.written, 2);
    let mut revs = r.revs.clone();
    revs.sort();
    assert_eq!(revs, ["r1", "r2"]);
    // Only `two` has nothing in its place; it is given as it was written.
    assert_eq!(r.gone, [pinned("two", 5, "r2")]);
    let text = ledger(&s);
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 3, "{text}");
    assert!(
        lines.contains(&reader),
        "another machine is left alone: {text}"
    );
    assert!(lines.contains(&pinned("one", 1, "r3").as_str()), "{text}");
    assert!(lines.contains(&pinned("three", 9, "r3").as_str()), "{text}");
    assert!(
        !text.contains("\"r1\"") && !text.contains("\"r2\""),
        "{text}"
    );
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::machine_is_a_snapshot written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_replace_with_nothing_new_says_everything_it_took_out() {
    let root = fresh("replace-empty");
    let s = store_in(&root, CALC);
    // The first run of a machine replaces nothing.
    let first = s
        .replace(
            "test:",
            &format!("{}\n{}", pinned("one", 1, "r1"), pinned("two", 5, "r1")),
        )
        .unwrap();
    assert_eq!((first.before, first.written), (0, 2));
    assert!(first.revs.is_empty() && first.gone.is_empty(), "{first:?}");
    assert_eq!(s.model().unwrap().evidence.len(), 2);
    // A run that observed nothing: all of it leaves, and all of it is shown.
    let r = s.replace("test:", "").unwrap();
    assert_eq!((r.before, r.written), (2, 0));
    assert_eq!(r.revs, ["r1"]);
    let mut gone = r.gone.clone();
    gone.sort();
    let mut all = vec![pinned("one", 1, "r1"), pinned("two", 5, "r1")];
    all.sort();
    assert_eq!(gone, all);
    assert!(s.model().unwrap().evidence.is_empty());
    assert!(!ledger(&s).contains("test:"));
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::human_is_never_dropped_silently written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_a_run_of_a_machine_never_takes_out_what_a_person_said() {
    let root = fresh("human");
    let s = store_in(&root, CALC);
    s.add(
        "evidence r { for calc::commutes holds }",
        who("review:ana"),
        None,
    )
    .unwrap();
    s.add(
        "evidence r { for calc::neg holds }",
        who("review:bob"),
        None,
    )
    .unwrap();
    // A source that only looks like the machine's: `testing:` is not `test:`.
    s.add(
        "evidence r { for calc::add holds }",
        who("testing:carl"),
        None,
    )
    .unwrap();
    s.add(&pinned("one", 1, "r1"), None, None).unwrap();
    let said: Vec<String> = ledger(&s)
        .lines()
        .filter(|l| l.contains("review:") || l.contains("testing:"))
        .map(str::to_string)
        .collect();
    assert_eq!(said.len(), 3);
    // The claim is reworded and `neg` is gone: what was said of them is
    // stale and orphaned, and stays through the next run of the machine.
    let reworded = CALC
        .replace("add(a, b) == add(b, a)", "add(b, a) == add(a, b)")
        .replace("    fn neg(a: Int) -> Int;\n", "");
    assert_ne!(reworded, CALC);
    fs::write(s.model_path(), reworded).unwrap();
    let r = s.replace("test:", &pinned("one", 1, "r2")).unwrap();
    assert_eq!((r.before, r.written), (1, 1));
    assert!(r.gone.is_empty(), "{:?}", r.gone);
    let text = ledger(&s);
    for line in &said {
        assert!(text.lines().any(|l| l == line), "{line} is not in\n{text}");
    }
    // Listed as orphaned or stale, counting for nothing.
    let m = s.model().unwrap();
    let aside: Vec<(&str, bool)> = m
        .aside
        .iter()
        .map(|a| (a.about.as_str(), a.why == mitra_lang::Why::Orphaned))
        .collect();
    assert!(aside.contains(&("calc::neg", true)), "{aside:?}");
    assert!(aside.contains(&("calc::commutes", false)), "{aside:?}");
    assert!(m
        .evidence
        .iter()
        .all(|e| e.source.as_deref() != Some("review:ana")));
    assert!(m
        .evidence
        .iter()
        .all(|e| e.source.as_deref() != Some("review:bob")));
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::sound_or_untouched written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_refused_replace_and_remove_leave_the_ledger_as_it_was() {
    let root = fresh("keep-sound");
    let s = store_in(&root, CALC);
    s.add(
        "evidence r { for calc::commutes holds }",
        who("review:ana"),
        None,
    )
    .unwrap();
    s.replace("test:", &pinned("one", 1, "r1")).unwrap();
    let before = ledger(&s);
    // New evidence on a piece the model does not have.
    let bad = "evidence t { for calc::nope given a = 1 gives 2 source \"test:x\" }";
    assert!(s.replace("test:", bad).is_err());
    assert_eq!(ledger(&s), before);
    // New evidence that does not read.
    assert!(s
        .replace("test:", "evidence t { for calc::add given")
        .is_err());
    assert_eq!(ledger(&s), before);
    // A store whose model no longer reads.
    fs::write(
        s.model_path(),
        "subject calc { fn add(a: Int, b: Int) -> ;;; {{{\n",
    )
    .unwrap();
    assert!(s.replace("test:", "").is_err());
    assert_eq!(ledger(&s), before);
    assert!(s.replace("test:", &pinned("one", 1, "r2")).is_err());
    assert_eq!(ledger(&s), before);
    assert!(s.remove(&Which::Source("review:ana".into())).is_err());
    assert_eq!(ledger(&s), before);
    assert!(s.remove(&Which::Piece("calc::add".into())).is_err());
    assert_eq!(ledger(&s), before);
    // No file is left beside the ledger by the refused writes.
    let mut there: Vec<String> = fs::read_dir(root.join("model"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    there.sort();
    assert_eq!(there, ["evidence.mitra", "model.mitra"]);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::Store::remove::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_remove_by_source_and_by_piece_gives_back_what_left_as_written() {
    let root = fresh("remove");
    let s = store_in(&root, CALC);
    s.add(
        "evidence r { for calc::commutes holds }",
        who("review:ana"),
        who("r1"),
    )
    .unwrap();
    s.add(
        "evidence r { for calc::neg holds }",
        who("review:ana"),
        who("r1"),
    )
    .unwrap();
    s.add(
        "evidence r { for calc::neg holds }",
        who("review:bob"),
        None,
    )
    .unwrap();
    s.add(&pinned("one", 1, "r1"), None, None).unwrap();
    let lines = |s: &Store| -> Vec<String> {
        ledger(s)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.trim().to_string())
            .collect()
    };
    let all = lines(&s);
    assert_eq!(all.len(), 4);
    // Nobody of that name: nothing leaves, nothing is rewritten. A source
    // is taken whole, not by how it starts.
    let before = ledger(&s);
    assert_eq!(s.remove(&Which::Source("review:nobody".into())), Ok(vec![]));
    assert_eq!(s.remove(&Which::Source("review:".into())), Ok(vec![]));
    assert_eq!(s.remove(&Which::Source("review:an".into())), Ok(vec![]));
    assert_eq!(s.remove(&Which::Orphaned), Ok(vec![]));
    assert_eq!(s.remove(&Which::Stale(None)), Ok(vec![]));
    assert_eq!(ledger(&s), before);
    // By piece: both said of `neg`, whoever said it, in the order written.
    let gone = s.remove(&Which::Piece("calc::neg".into())).unwrap();
    assert_eq!(gone, [all[1].clone(), all[2].clone()]);
    assert_eq!(lines(&s), [all[0].clone(), all[3].clone()]);
    // By source: what is left of ana.
    let gone = s.remove(&Which::Source("review:ana".into())).unwrap();
    assert_eq!(gone, [all[0].clone()]);
    assert_eq!(lines(&s), [all[3].clone()]);
    // The second time there is nothing to take.
    assert_eq!(s.remove(&Which::Source("review:ana".into())), Ok(vec![]));
    assert_eq!(lines(&s), [all[3].clone()]);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::removal_is_explicit written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_remove_orphaned_and_stale_take_only_those() {
    let root = fresh("remove-aside");
    let s = store_in(&root, CALC);
    s.add(
        "evidence r { for calc::commutes holds }",
        who("review:ana"),
        None,
    )
    .unwrap();
    s.add(
        "evidence r { for calc::neg holds }",
        who("review:bob"),
        None,
    )
    .unwrap();
    s.add(
        "evidence r { for calc::add holds }",
        who("review:carl"),
        None,
    )
    .unwrap();
    let all: Vec<String> = ledger(&s)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect();
    assert_eq!(all.len(), 3);
    let reworded = CALC
        .replace("add(a, b) == add(b, a)", "add(b, a) == add(a, b)")
        .replace("    fn neg(a: Int) -> Int;\n", "");
    fs::write(s.model_path(), reworded).unwrap();
    // Stale on a piece that is not stale: nothing.
    assert_eq!(
        s.remove(&Which::Stale(Some("calc::add".into()))),
        Ok(vec![])
    );
    // Orphaned takes what was said of `neg`, and not the stale one.
    assert_eq!(s.remove(&Which::Orphaned), Ok(vec![all[1].clone()]));
    let text = ledger(&s);
    assert!(text.contains(&all[0]) && text.contains(&all[2]), "{text}");
    // Stale takes what was said of the reworded claim, and not what still
    // judges.
    assert_eq!(s.remove(&Which::Stale(None)), Ok(vec![all[0].clone()]));
    let left: Vec<String> = ledger(&s)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect();
    assert_eq!(left, [all[2].clone()]);
    let m = s.model().unwrap();
    assert!(m.aside.is_empty());
    assert_eq!(m.evidence.len(), 1);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::nothing_leaves_unshown written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_every_entry_that_leaves_is_given_back() {
    let root = fresh("unshown");
    let s = store_in(&root, CALC);
    // An entry over several lines, as a person may write it in the file.
    let long = "evidence t {\n    for calc::add given a = 7, b = 2 gives 9\n    source \"test:long\" rev \"r1\"\n}";
    s.append(long).unwrap();
    s.add(&pinned("one", 1, "r1"), None, None).unwrap();
    s.add(
        "evidence r { for calc::neg holds }",
        who("review:bob"),
        None,
    )
    .unwrap();
    let count = |s: &Store| s.model().unwrap().evidence.len();
    assert_eq!(count(&s), 3);
    // Taken out by a machine: both are shown, the long one whole.
    let r = s.replace("test:", "").unwrap();
    assert_eq!(r.before, 2);
    assert_eq!(r.gone.len(), 2, "{:?}", r.gone);
    assert!(r.gone.contains(&long.to_string()), "{:?}", r.gone);
    assert!(r.gone.contains(&pinned("one", 1, "r1")), "{:?}", r.gone);
    assert_eq!(count(&s), 1);
    // Taken out by `remove`: as many are given back as left.
    let bob = ledger(&s).trim().to_string();
    let gone = s.remove(&Which::Piece("calc::neg".into())).unwrap();
    assert_eq!(gone, [bob]);
    assert_eq!(count(&s), 0);
    assert_eq!(ledger(&s).trim(), "");
    let _ = fs::remove_dir_all(root);
}
