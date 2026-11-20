//! Tests written from the model of mitra-store (`.mitra/*.mitra`) alone.
//! Each test works in a fresh folder of its own under the temp dir.

use mitra_store::source;
use mitra_store::store::Which;
use mitra_store::Store;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

static COUNTER: AtomicUsize = AtomicUsize::new(0);
// Only the test of `nearest` changes the current folder; it holds this.
static CWD: Mutex<()> = Mutex::new(());

/// A fresh empty folder with a unique name.
fn fresh(name: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "mitra-cold-{name}-{}-{n}-{nanos}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).unwrap();
    root
}

const CALC: &str = "subject calc {\n    fn add(a: Int, b: Int) -> Int;\n    fn neg(a: Int) -> Int;\n    claim comm(a: Int, b: Int) { add(a, b) == add(b, a) }\n}\n";

/// A store made by hand (not by `init`): `.mitra/model.mitra` and an
/// evidence file with the given texts.
fn store_with(name: &str, model: &str, evidence: &str) -> (PathBuf, Store) {
    let root = fresh(name);
    let dir = root.join(".mitra");
    fs::create_dir_all(&dir).unwrap();
    fs::write(dir.join("model.mitra"), model).unwrap();
    fs::write(dir.join("evidence.mitra"), evidence).unwrap();
    let s = Store::of(&dir.join("model.mitra").display().to_string()).unwrap();
    (root, s)
}

fn ledger(s: &Store) -> String {
    fs::read_to_string(s.evidence_path()).unwrap()
}

fn test_ev(n: &str, a: i64, rev: &str) -> String {
    format!(
        "evidence t {{ for calc::add given a = {a}, b = 0 gives {a} source \"test:{n}\" rev \"{rev}\" }}"
    )
}

// ---------------------------------------------------------------- store::of

// mitra: evidence for store::a_model_file_names_its_store written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_of_model_in_mitra_folder_is_a_store() {
    assert!(Store::of("a/.mitra/model.mitra").is_some());
    assert!(Store::of("/x/y/.mitra/model.mitra").is_some());
}

// mitra: evidence for store::another_file_is_no_store written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_of_other_files_are_no_store() {
    assert!(Store::of("a/model.mitra").is_none());
    assert!(Store::of("model.mitra").is_none());
    assert!(Store::of("a/.mitra/more.mitra").is_none());
    assert!(Store::of("a/.mitra/evidence.mitra").is_none());
    assert!(Store::of("a/mitra/model.mitra").is_none());
    assert!(Store::of("a/.mitra/model.mitra/x").is_none());
}

// mitra: evidence for store::Store::model_path::correct written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_paths_of_a_store_are_in_its_folder() {
    let s = Store::of("/some/where/.mitra/model.mitra").unwrap();
    assert_eq!(
        Path::new(&s.model_path()),
        Path::new("/some/where/.mitra/model.mitra")
    );
    assert_eq!(
        Path::new(&s.evidence_path()),
        Path::new("/some/where/.mitra/evidence.mitra")
    );
    // Nothing of it is read: the folder does not exist and that is fine.
    assert!(!Path::new("/some/where/.mitra").exists());
}

// ------------------------------------------------------- store::whole_folder

// mitra: evidence for store::whole_folder written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_source_is_every_mitra_file_model_first_ledger_aside() {
    let (root, s) = store_with(
        "whole",
        "subject m { fn f() -> Int; }\n",
        "evidence e { for m::f holds }\n",
    );
    let dir = root.join(".mitra");
    // Names that sort before `model.mitra`, and after.
    fs::write(dir.join("aaa.mitra"), "subject a { fn g() -> Int; }\n").unwrap();
    fs::write(dir.join("zzz.mitra"), "subject z { fn h() -> Int; }\n").unwrap();
    fs::write(dir.join("notes.txt"), "not model\n").unwrap();
    fs::write(dir.join("settings.toml"), "").unwrap();
    let m = s.source().unwrap();
    let names: Vec<String> = m
        .files()
        .iter()
        .map(|f| {
            Path::new(&f.name)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        })
        .collect();
    assert_eq!(names.len(), 3, "{names:?}");
    assert_eq!(names[0], "model.mitra", "{names:?}");
    assert!(names.contains(&"aaa.mitra".to_string()), "{names:?}");
    assert!(names.contains(&"zzz.mitra".to_string()), "{names:?}");
    assert!(!m.text().contains("evidence e"), "ledger handed as model");
    assert!(!m.text().contains("not model"));
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for files::handed_unchanged written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_model_files_are_handed_under_their_path_unchanged() {
    let model = "// first   \n\nsubject m { fn f() -> Int; }   \n\n\n// no newline at end";
    let more = "subject n {\r\n  fn g() -> Int;\r\n}\r\n";
    let (root, s) = store_with("unchanged", model, "");
    fs::write(root.join(".mitra/more.mitra"), more).unwrap();
    let m = s.source().unwrap();
    let by_name = |n: &str| {
        m.files()
            .iter()
            .find(|f| f.name.ends_with(n))
            .unwrap_or_else(|| panic!("{n} not handed"))
    };
    assert_eq!(by_name("model.mitra").text, model);
    assert_eq!(by_name("more.mitra").text, more);
    assert!(Path::new(&by_name("more.mitra").name).exists());
    // A single file through `files::file` is handed to the byte too.
    let one = source::file(&s.model_path()).unwrap();
    assert_eq!(one.files().len(), 1);
    assert_eq!(one.files()[0].text, model);
    assert_eq!(one.files()[0].name, s.model_path());
    let _ = fs::remove_dir_all(root);
}

// ---------------------------------------------------------------- files::read

// mitra: evidence for files::read::correct written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_read_gives_the_text_or_why_with_the_path() {
    let root = fresh("read");
    let p = root.join("x.mitra");
    let text = "line one\n\n  two\t\n";
    fs::write(&p, text).unwrap();
    let path = p.display().to_string();
    assert_eq!(source::read(&path), Ok(text.to_string()));
    // Read twice, same text.
    assert_eq!(source::read(&path), source::read(&path));
    let missing = root.join("nope.mitra").display().to_string();
    let e = source::read(&missing).unwrap_err();
    assert!(e.contains(&missing), "{e}");
    let _ = fs::remove_dir_all(root);
}

// ---------------------------------------------------------------- files::file

// mitra: evidence for files::imports_beside written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_imports_are_read_beside_the_first_file_and_in_turn() {
    let root = fresh("imports");
    fs::create_dir_all(root.join("lib")).unwrap();
    let a = "import lib::b::g;\nsubject a { fn f() -> Int; }\n";
    let b = "subject b { fn g() -> Int; }\n";
    fs::write(root.join("lib/a.mitra"), a).unwrap();
    fs::write(root.join("lib/b.mitra"), b).unwrap();
    let model = "import lib::a::f;\nsubject m { fn h() -> Int; }\n";
    fs::write(root.join("model.mitra"), model).unwrap();
    let m = source::file(&root.join("model.mitra").display().to_string()).unwrap();
    assert_eq!(m.handed.modules.get("lib::a").map(String::as_str), Some(a));
    assert_eq!(m.handed.modules.get("lib::b").map(String::as_str), Some(b));
    assert!(m.handed.missing.is_empty(), "{:?}", m.handed.missing);
    // Only the model file itself is a file of the model.
    assert_eq!(m.files().len(), 1);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for files::missing_is_said_at_the_import written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_missing_import_does_not_stop_reading() {
    let root = fresh("missing");
    let model = "import gone::here::x;\nsubject m { fn f() -> Int; }\n";
    let p = root.join("model.mitra");
    fs::write(&p, model).unwrap();
    let m = source::file(&p.display().to_string()).expect("a missing import stops the reading");
    let why = m
        .handed
        .missing
        .get("gone::here")
        .expect("why is not handed");
    assert!(why.contains("here.mitra"), "{why}");
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for files::missing_is_said_at_the_import written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_missing_import_is_said_at_its_line() {
    let model = "// one\n// two\nimport gone::here::x;\nsubject m { fn f() -> Int; }\n";
    let (root, s) = store_with("missing-line", model, "");
    let e = s.model().unwrap_err();
    assert!(e.contains("model.mitra: line 3"), "{e}");
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for files::stores_beside written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_other_store_beside_is_handed_whole_ledger_aside() {
    let root = fresh("beside");
    let other = root.join("other/.mitra");
    let here = root.join("here/.mitra");
    fs::create_dir_all(&other).unwrap();
    fs::create_dir_all(&here).unwrap();
    fs::write(other.join("model.mitra"), "// OTHER-MODEL\n").unwrap();
    fs::write(
        other.join("block.mitra"),
        "subject block { record R { n: Int } }\n// OTHER-BLOCK\n",
    )
    .unwrap();
    fs::write(other.join("evidence.mitra"), "// OTHER-LEDGER\n").unwrap();
    fs::write(
        here.join("model.mitra"),
        "import other::block::{R};\nsubject s { fn f(r: block::R) -> Int; }\n",
    )
    .unwrap();
    fs::write(here.join("evidence.mitra"), "").unwrap();
    let s = Store::of(&here.join("model.mitra").display().to_string()).unwrap();
    let m = s.source().unwrap();
    let handed = m
        .handed
        .stores
        .get("other")
        .expect("other store not handed");
    assert!(handed.contains("OTHER-MODEL"), "{handed}");
    assert!(handed.contains("OTHER-BLOCK"), "{handed}");
    assert!(!handed.contains("OTHER-LEDGER"), "{handed}");
    let _ = fs::remove_dir_all(root);
}

// ------------------------------------------------------------ store::settings

// mitra: evidence for store::settings_beside_the_model written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_no_settings_file_is_the_defaults() {
    let (root, s) = store_with("settings-none", CALC, "");
    let got = s.settings().unwrap();
    assert_eq!(got, mitra_lang::Settings::default());
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for store::Store::settings::correct written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_settings_are_read_and_handed_with_the_model() {
    let (root, s) = store_with("settings-read", CALC, "");
    fs::write(
        root.join(".mitra/settings.toml"),
        "[sample]\ntolerance = 0.25\n",
    )
    .unwrap();
    assert_eq!(s.settings().unwrap().tolerance, 0.25);
    assert_eq!(s.source().unwrap().handed.settings.tolerance, 0.25);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for store::settings_beside_the_model written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_settings_that_do_not_read_are_an_error_with_the_path() {
    let (root, s) = store_with("settings-bad", CALC, "");
    let path = root.join(".mitra/settings.toml");
    fs::write(&path, "[nothing_like_this]\nx = 1\n").unwrap();
    let e = s.settings().unwrap_err();
    assert!(e.contains("settings.toml"), "{e}");
    // Handed with the model: the model does not read either.
    assert!(s.source().is_err());
    // A settings "file" that is a folder does not read either.
    fs::remove_file(&path).unwrap();
    fs::create_dir(&path).unwrap();
    let e = s.settings().unwrap_err();
    assert!(e.contains("settings.toml"), "{e}");
    let _ = fs::remove_dir_all(root);
}

// ---------------------------------------------------------- store::write_build

// mitra: evidence for store::Store::write_build::correct written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_write_build_writes_each_file_under_build() {
    let (root, s) = store_with("build", CALC, "");
    let files = vec![
        ("a.rs".to_string(), "fn a() {}\n".to_string()),
        ("b.txt".to_string(), "bee".to_string()),
    ];
    let dir = s.write_build(&files).unwrap();
    let expect = root.join(".mitra/build");
    assert_eq!(
        Path::new(&dir).canonicalize().unwrap(),
        expect.canonicalize().unwrap()
    );
    assert_eq!(
        fs::read_to_string(expect.join("a.rs")).unwrap(),
        "fn a() {}\n"
    );
    assert_eq!(fs::read_to_string(expect.join("b.txt")).unwrap(), "bee");
    // The second build: in place of the first, not edited onto it.
    s.write_build(&[("a.rs".to_string(), "x".to_string())])
        .unwrap();
    assert_eq!(fs::read_to_string(expect.join("a.rs")).unwrap(), "x");
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for store::build_is_written_again written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_a_build_is_never_read_back_as_model() {
    let (root, s) = store_with("build-unread", CALC, "");
    s.write_build(&[(
        "built.mitra".to_string(),
        "this is not a model at all {{{".to_string(),
    )])
    .unwrap();
    let m = s.source().unwrap();
    assert!(m.files().iter().all(|f| !f.name.contains(".mitra/build")));
    assert!(s.model().is_ok(), "{:?}", s.model().err());
    let _ = fs::remove_dir_all(root);
}

// ------------------------------------------------------ store::never_overwrites

// mitra: evidence for store::never_overwrites written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_init_creates_once_and_never_overwrites() {
    let root = fresh("init");
    let dir = Store::init(&root).unwrap();
    assert!(dir.join("model.mitra").is_file());
    assert!(dir.join("evidence.mitra").is_file());
    fs::write(dir.join("model.mitra"), "MINE").unwrap();
    fs::write(dir.join("evidence.mitra"), "LEDGER").unwrap();
    assert!(Store::init(&root).is_err());
    assert_eq!(fs::read_to_string(dir.join("model.mitra")).unwrap(), "MINE");
    assert_eq!(
        fs::read_to_string(dir.join("evidence.mitra")).unwrap(),
        "LEDGER"
    );
    // A `.mitra` that is a file is something there too: left alone.
    let other = fresh("init-file");
    fs::write(other.join(".mitra"), "a file").unwrap();
    assert!(Store::init(&other).is_err());
    assert_eq!(fs::read_to_string(other.join(".mitra")).unwrap(), "a file");
    let _ = fs::remove_dir_all(root);
    let _ = fs::remove_dir_all(other);
}

// ---------------------------------------------------------------- store::nearest

// mitra: evidence for store::nearest written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_find_uses_the_nearest_store_at_or_above() {
    let _g = CWD.lock().unwrap_or_else(|e| e.into_inner());
    let root = fresh("nearest").canonicalize().unwrap();
    Store::init(&root).unwrap();
    let inner = root.join("a");
    fs::create_dir_all(inner.join("b/c")).unwrap();
    Store::init(&inner).unwrap();
    let was = std::env::current_dir().unwrap();
    let found = |at: &Path| {
        std::env::set_current_dir(at).unwrap();
        let r = Store::find().map(|s| s.root().canonicalize().unwrap());
        std::env::set_current_dir(&was).unwrap();
        r
    };
    assert_eq!(found(&inner.join("b/c")), Ok(inner.clone()));
    assert_eq!(found(&inner), Ok(inner.clone()));
    assert_eq!(found(&root), Ok(root.clone()));
    let _ = fs::remove_dir_all(root);
}

// ------------------------------------------------------------------ ledger::add

// mitra: evidence for ledger::keeps_who_and_when written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_add_counts_and_stamps_who_and_when() {
    let (root, s) = store_with("add", CALC, "");
    let n = s.add(
        "evidence x { for calc::add given a = 1, b = 2 gives 3 }\nevidence y { for calc::neg given a = 1 gives -1 }",
        Some("review:ana".to_string()),
        Some("rev-77".to_string()),
    );
    assert_eq!(n, Ok(2));
    let m = s.model().unwrap();
    assert_eq!(m.evidence.len(), 2);
    for e in &m.evidence {
        assert_eq!(e.source.as_deref(), Some("review:ana"));
        assert_eq!(e.rev.as_deref(), Some("rev-77"));
    }
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for ledger::Store::add::correct written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_add_without_who_or_when_stamps_neither() {
    let (root, s) = store_with("add-none", CALC, "");
    assert_eq!(
        s.add(
            "evidence x { for calc::add given a = 1, b = 2 gives 3 }",
            None,
            None
        ),
        Ok(1)
    );
    let m = s.model().unwrap();
    assert_eq!(m.evidence.len(), 1);
    // No who and no revision were given: none is written.
    assert_eq!(m.evidence[0].rev, None);
    let l = ledger(&s);
    assert!(!l.contains("source \"") && !l.contains("rev \""), "{l}");
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for ledger::sound_or_untouched written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_add_error_writes_nothing() {
    let start = "// head\nevidence x { for calc::add given a = 1, b = 2 gives 3 }\n";
    let (root, s) = store_with("add-err", CALC, start);
    // A piece the model does not have.
    assert!(s
        .add("evidence y { for calc::nope holds }", None, None)
        .is_err());
    assert_eq!(ledger(&s), start);
    // Nothing at all.
    assert!(s.add("", Some("review:a".into()), None).is_err());
    assert_eq!(ledger(&s), start);
    // A who that cannot be written as given.
    assert!(s
        .add(
            "evidence y { for calc::neg given a = 1 gives -1 }",
            Some("review:\"a\"".into()),
            None
        )
        .is_err());
    assert_eq!(ledger(&s), start);
    let _ = fs::remove_dir_all(root);
}

// --------------------------------------------------------------- ledger::append

// mitra: evidence for ledger::Store::append::correct written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_append_adds_after_and_keeps_what_was_there() {
    // A ledger with no newline at its end.
    let start = "// head\nevidence x { for calc::add given a = 1, b = 2 gives 3 }";
    let (root, s) = store_with("append", CALC, start);
    let new = "evidence y { for calc::neg given a = 1 gives -1 }";
    s.append(new).unwrap();
    let after = ledger(&s);
    assert!(after.starts_with(start), "{after}");
    let rest = &after[start.len()..];
    // The new block is on a line of its own, after the old ones.
    assert!(rest.lines().any(|l| l.trim() == new), "{after}");
    assert_eq!(s.model().unwrap().evidence.len(), 2);
    // The second call adds after the first.
    let third = "evidence z { for calc::add given a = 2, b = 2 gives 4 }";
    s.append(third).unwrap();
    let last = ledger(&s);
    assert!(last.starts_with(&after), "{last}");
    assert!(last.find(new).unwrap() < last.find(third).unwrap());
    assert_eq!(s.model().unwrap().evidence.len(), 3);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for ledger::reads_back written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_appended_evidence_reads_back_exactly() {
    let (root, s) = store_with("readback", CALC, "");
    let new = "evidence y { for calc::neg given a = 3 gives -3 source \"test:n\" rev \"r9\" }";
    s.append(new).unwrap();
    assert_eq!(ledger(&s).trim(), new);
    let m = s.model().unwrap();
    assert_eq!(m.evidence.len(), 1);
    assert_eq!(m.evidence[0].source.as_deref(), Some("test:n"));
    assert_eq!(m.evidence[0].rev.as_deref(), Some("r9"));
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for ledger::sound_or_untouched written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_append_on_a_store_that_does_not_read_writes_nothing() {
    let start = "evidence x { for calc::add given a = 1, b = 2 gives 3 }\n";
    let (root, s) = store_with("append-broken", CALC, start);
    // The model no longer reads.
    fs::write(
        s.model_path(),
        "subject calc { fn add(a: Int, b: Int) -> Int; ",
    )
    .unwrap();
    assert!(s
        .append("evidence y { for calc::add given a = 2, b = 2 gives 4 }")
        .is_err());
    assert_eq!(ledger(&s), start);
    // Bad new evidence on a sound store: nothing written either.
    fs::write(s.model_path(), CALC).unwrap();
    assert!(s.append("evidence y { this is not evidence").is_err());
    assert_eq!(ledger(&s), start);
    let _ = fs::remove_dir_all(root);
}

// ------------------------------------------------------------ keeping::replace

// mitra: evidence for keeping::machine_is_a_snapshot written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_replace_takes_out_what_the_machine_wrote_and_says_so() {
    let (root, s) = store_with("replace", CALC, "// head\n");
    let first = format!(
        "{}\n{}\n{}",
        test_ev("one", 1, "r2"),
        test_ev("two", 2, "r1"),
        test_ev("three", 3, "r2")
    );
    let r = s.replace("test:", &first).unwrap();
    assert_eq!((r.before, r.written), (0, 3));
    assert!(r.revs.is_empty() && r.gone.is_empty(), "{r:?}");
    // Next run: only `one`, on a new revision.
    let next = test_ev("one", 1, "r3");
    let r = s.replace("test:", &next).unwrap();
    assert_eq!(r.before, 3);
    assert_eq!(r.written, 1);
    // The revisions they were observed on, each once.
    assert_eq!(r.revs, vec!["r1".to_string(), "r2".to_string()]);
    // Those with nothing in their place, as written.
    let mut gone = r.gone.clone();
    gone.sort();
    let mut want = vec![test_ev("two", 2, "r1"), test_ev("three", 3, "r2")];
    want.sort();
    assert_eq!(gone, want);
    let l = ledger(&s);
    assert!(l.starts_with("// head\n"), "{l}");
    assert!(l.contains(&next), "{l}");
    assert!(!l.contains("r1") && !l.contains("r2"), "{l}");
    assert_eq!(s.model().unwrap().evidence.len(), 1);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::human_is_never_dropped_silently written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_replace_leaves_what_a_person_and_other_machines_said() {
    let person = "evidence p { for calc::comm holds source \"review:ana\" }";
    let reader = "evidence q { for calc::neg given a = 1 gives -1 source \"reader:rust\" }";
    let start = format!("{person}\n{reader}\n{}\n", test_ev("one", 1, "r1"));
    let (root, s) = store_with("replace-keep", CALC, &start);
    let r = s.replace("test:", "").unwrap();
    assert_eq!((r.before, r.written), (1, 0));
    assert_eq!(r.gone, vec![test_ev("one", 1, "r1")]);
    let l = ledger(&s);
    assert!(l.contains(person), "{l}");
    assert!(l.contains(reader), "{l}");
    assert!(!l.contains("test:one"), "{l}");
    // Second empty run: nothing left to take, nothing changes.
    let r = s.replace("test:", "").unwrap();
    assert_eq!((r.before, r.written, r.gone.len()), (0, 0, 0));
    assert_eq!(ledger(&s), l);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::sound_or_untouched written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_replace_error_leaves_the_ledger_as_it_was() {
    let start = format!("// head\n{}\n", test_ev("one", 1, "r1"));
    let (root, s) = store_with("replace-err", CALC, &start);
    let bad = "evidence t { for calc::nope holds source \"test:x\" }";
    assert!(s.replace("test:", bad).is_err());
    assert_eq!(ledger(&s), start);
    let _ = fs::remove_dir_all(root);
}

// ------------------------------------------------------------- keeping::remove

// mitra: evidence for keeping::nothing_leaves_unshown written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_remove_by_source_gives_back_what_left_as_written() {
    let a = "evidence p { for calc::comm holds source \"review:ana\" }";
    let b = "evidence q { for calc::add given a = 1, b = 1 gives 2 source \"review:bob\" }";
    let c = "evidence r { for calc::neg given a = 1 gives -1 source \"review:ana\" }";
    let start = format!("// head\n{a}\n{b}\n{c}\n");
    let (root, s) = store_with("remove-src", CALC, &start);
    let gone = s.remove(&Which::Source("review:ana".into())).unwrap();
    assert_eq!(gone, vec![a.to_string(), c.to_string()]);
    let l = ledger(&s);
    assert!(l.contains(b), "{l}");
    assert!(!l.contains("review:ana"), "{l}");
    assert!(l.starts_with("// head\n"), "{l}");
    // A second call has nothing to take.
    assert_eq!(s.remove(&Which::Source("review:ana".into())), Ok(vec![]));
    assert_eq!(ledger(&s), l);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::Store::remove::correct written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_remove_by_piece_takes_only_that_piece() {
    let a = "evidence p { for calc::comm holds source \"review:ana\" }";
    let b = "evidence q { for calc::add given a = 1, b = 1 gives 2 source \"review:bob\" }";
    let start = format!("{a}\n{b}\n");
    let (root, s) = store_with("remove-piece", CALC, &start);
    let gone = s.remove(&Which::Piece("calc::add".into())).unwrap();
    assert_eq!(gone, vec![b.to_string()]);
    let l = ledger(&s);
    assert!(l.contains(a) && !l.contains(b), "{l}");
    // Nothing on a piece with no evidence: nothing changes.
    assert_eq!(s.remove(&Which::Piece("calc::neg".into())), Ok(vec![]));
    assert_eq!(ledger(&s), l);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::removal_is_explicit written "agent:anthropic/opus/cold-w-store" cold
#[test]
fn cold_orphaned_person_word_stays_until_removed() {
    let person = "evidence p { for calc::neg given a = 1 gives -1 source \"review:ana\" }";
    let (root, s) = store_with("orphan", CALC, &format!("{person}\n"));
    // The piece is gone from the model.
    fs::write(
        s.model_path(),
        "subject calc {\n    fn add(a: Int, b: Int) -> Int;\n    claim comm(a: Int, b: Int) { add(a, b) == add(b, a) }\n}\n",
    )
    .unwrap();
    // A run of a machine leaves it.
    s.replace("test:", &test_ev("one", 1, "r1")).unwrap();
    assert!(ledger(&s).contains(person));
    let m = s.model().unwrap();
    assert_eq!(m.aside.len(), 1, "{:?}", m.aside);
    // Stale picks nothing orphaned; Orphaned picks it and lists it.
    assert_eq!(s.remove(&Which::Stale(None)), Ok(vec![]));
    assert_eq!(s.remove(&Which::Orphaned), Ok(vec![person.to_string()]));
    assert!(!ledger(&s).contains("review:ana"));
    let _ = fs::remove_dir_all(root);
}
