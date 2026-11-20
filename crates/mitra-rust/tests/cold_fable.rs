//! Tests written from the model in `model/`, without knowledge of the code.

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use mitra_lang::results::{Outcome, Tied};
use mitra_lang::Reader;
use mitra_rust::report::libtest;
use mitra_rust::source::{fn_below, is_tie, mitra_line};
use mitra_rust::Rust;

static N: AtomicUsize = AtomicUsize::new(0);

/// A fresh folder with the given files (path relative to it, text).
fn project(files: &[(&str, &str)]) -> PathBuf {
    let n = N.fetch_add(1, Ordering::SeqCst);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "mitra-rust-cold-{}-{}-{}",
        std::process::id(),
        nanos,
        n
    ));
    std::fs::create_dir_all(&root).unwrap();
    for (path, text) in files {
        let p = root.join(path);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, text).unwrap();
    }
    root
}

fn done(root: &Path) {
    let _ = std::fs::remove_dir_all(root);
}

fn outcome(name: &str, passed: bool) -> Outcome {
    Outcome {
        name: name.into(),
        passed,
    }
}

fn model(text: &str) -> mitra_lang::Model {
    match mitra_lang::load(text, "") {
        Ok(m) => m,
        Err(_) => panic!("the model of the test does not load: {text}"),
    }
}

// ---------------------------------------------------------------- libtest

// mitra: evidence for reader::libtest written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_libtest_empty_report_has_no_results() {
    assert_eq!(libtest(""), Vec::<Outcome>::new());
    assert_eq!(libtest("\n\n"), Vec::<Outcome>::new());
}

// mitra: evidence for reader::libtest::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_libtest_deep_path_gives_last_segment() {
    assert_eq!(
        libtest("test a::b::tests::t_deep ... FAILED"),
        [outcome("t_deep", false)]
    );
}

// mitra: evidence for reader::libtest::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_libtest_keeps_order_and_skips_ignored_between() {
    let text = "running 4 tests\n\
                test m::z_last ... FAILED\n\
                test m::skipped ... ignored\n\
                test m::a_first ... ok\n\
                test m::mid ... ok <0.120s>\n\
                \n\
                test result: FAILED. 2 passed; 1 failed; 1 ignored";
    assert_eq!(
        libtest(text),
        [
            outcome("z_last", false),
            outcome("a_first", true),
            outcome("mid", true)
        ]
    );
}

// mitra: evidence for reader::libtest::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_libtest_same_name_twice_is_two_results() {
    assert_eq!(
        libtest("test a::t ... ok\ntest b::t ... FAILED\n"),
        [outcome("t", true), outcome("t", false)]
    );
}

// mitra: evidence for reader::libtest::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_libtest_second_call_gives_the_same() {
    let text = "test a ... ok\ntest b ... FAILED\ntest c ... ignored";
    let first = libtest(text);
    assert_eq!(first, libtest(text));
    assert_eq!(first, [outcome("a", true), outcome("b", false)]);
}

// mitra: evidence for reader::libtest::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_results_of_the_reader_are_libtest() {
    let text =
        "running 2 tests\ntest tests::t_small ... ok\ntest t_big ... FAILED\ntest s ... ignored";
    assert_eq!(Rust.results(text), libtest(text));
    assert_eq!(
        Rust.results(text),
        [outcome("t_small", true), outcome("t_big", false)]
    );
}

// ------------------------------------------------------------- mitra_line

// mitra: evidence for source::mitra_line::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_line_indented_doc_comment() {
    assert_eq!(
        mitra_line("\t    /// mitra: implements a::f"),
        Some("implements a::f")
    );
}

// mitra: evidence for source::mitra_line::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_line_not_a_comment_is_none() {
    assert_eq!(mitra_line("mitra: implements a::f"), None);
    assert_eq!(mitra_line(""), None);
    assert_eq!(mitra_line("// implements a::f"), None);
    assert_eq!(mitra_line("fn mitra() {}"), None);
}

// mitra: evidence for source::mitra_line::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_line_nothing_after_mitra_is_empty_text() {
    assert_eq!(mitra_line("// mitra:"), Some(""));
}

// --------------------------------------------------------------- fn_below

// mitra: evidence for source::fn_below::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_below_whole_name_is_given() {
    assert_eq!(
        fn_below("pub fn long_name_2(a: u8) -> u8 { a }"),
        Some("long_name_2")
    );
    assert_eq!(fn_below("fn g<'a, T: Clone>(x: &'a T) {}"), Some("g"));
}

// mitra: evidence for source::fn_below::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_below_first_fn_of_two() {
    assert_eq!(fn_below("fn first() {}\nfn second() {}"), Some("first"));
}

// mitra: evidence for source::fn_below::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_below_a_word_that_starts_with_fn_is_no_fn() {
    assert_eq!(fn_below("fnord();"), None);
    assert_eq!(fn_below("fn_like(1);\nfn f() {}"), None);
}

// ----------------------------------------------------------------- is_tie

// mitra: evidence for source::is_tie::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_is_tie_only_for_the_two_forms() {
    assert!(is_tie("implements reader::Rust::tie"));
    assert!(is_tie("evidence for t_line_doc"));
    assert!(!is_tie(""));
    assert!(!is_tie("claim c : correct(f(1))"));
    assert!(!is_tie("ensure p = \"it implements a::f\";"));
    assert!(!is_tie("fn f() -> Bool; // evidence for a::c"));
}

// ------------------------------------------------------------ Rust::tests

fn tied(found: &[Tied]) -> Vec<(String, String)> {
    let mut v: Vec<(String, String)> = found
        .iter()
        .map(|t| (t.claim.clone(), t.test.clone()))
        .collect();
    v.sort();
    v
}

// mitra: evidence for source::finds_ties written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_tests_gives_claim_test_and_place() {
    let root = project(&[(
        "src/lib.rs",
        "pub fn f() {}\n\n// mitra: evidence for a::c\n#[test]\nfn t_c() {\n    f();\n}\n",
    )]);
    let found = Rust.tests(&root).unwrap();
    done(&root);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].claim, "a::c");
    assert_eq!(found[0].test, "t_c");
    assert_eq!(found[0].pinned, "");
    assert!(
        found[0].place.contains("lib.rs") && found[0].place.ends_with(":3"),
        "place is {}",
        found[0].place
    );
}

// mitra: evidence for code::reads_what_is_there written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_tests_reads_every_rust_file_under_src_and_tests_only() {
    let t = |claim: &str, name: &str| {
        format!("// mitra: evidence for {claim}\n#[test]\nfn {name}() {{}}\n")
    };
    let root = project(&[
        ("src/lib.rs", &t("a::one", "t_one")),
        ("src/deep/er/mod.rs", &t("a::two", "t_two")),
        ("tests/it.rs", &t("a::three", "t_three")),
        ("tests/sub/more.rs", &t("a::four", "t_four")),
        ("src/notes.txt", &t("a::txt", "t_txt")),
        ("benches/b.rs", &t("a::bench", "t_bench")),
        ("other.rs", &t("a::root", "t_root")),
    ]);
    let found = Rust.tests(&root).unwrap();
    done(&root);
    let want: Vec<(String, String)> = [
        ("a::four", "t_four"),
        ("a::one", "t_one"),
        ("a::three", "t_three"),
        ("a::two", "t_two"),
    ]
    .iter()
    .map(|(c, n)| (c.to_string(), n.to_string()))
    .collect();
    assert_eq!(tied(&found), want);
}

// mitra: evidence for reader::Rust::tests::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_tests_two_ties_in_one_file_each_with_its_own_fn() {
    let root = project(&[(
        "tests/it.rs",
        "/// mitra: evidence for a::first\n#[test]\nfn t_first() {}\n\n    // mitra: evidence for a::second\n    #[test]\n    fn t_second() {}\n",
    )]);
    let found = Rust.tests(&root).unwrap();
    done(&root);
    assert_eq!(found.len(), 2);
    assert_eq!(
        (found[0].claim.as_str(), found[0].test.as_str()),
        ("a::first", "t_first")
    );
    assert_eq!(
        (found[1].claim.as_str(), found[1].test.as_str()),
        ("a::second", "t_second")
    );
    assert!(
        found[0].place.ends_with(":1"),
        "place is {}",
        found[0].place
    );
    assert!(
        found[1].place.ends_with(":5"),
        "place is {}",
        found[1].place
    );
}

// mitra: evidence for reader::Rust::tests::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_tests_keeps_what_the_comment_pins() {
    let root = project(&[(
        "src/lib.rs",
        "// mitra: evidence for a::c given x = 1 gives 2\n#[test]\nfn t_pinned() {}\n",
    )]);
    let found = Rust.tests(&root).unwrap();
    done(&root);
    assert_eq!(found.len(), 1);
    assert_eq!(found[0].claim, "a::c");
    assert_eq!(found[0].test, "t_pinned");
    assert_eq!(found[0].pinned, "given x = 1 gives 2");
}

// mitra: evidence for reader::Rust::tests::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_tests_leaves_out_implements_ties() {
    let root = project(&[(
        "src/lib.rs",
        "// mitra: implements a::f\npub fn f() {}\n\n// mitra: evidence for a::c\n#[test]\nfn t_c() {}\n",
    )]);
    let found = Rust.tests(&root).unwrap();
    done(&root);
    assert_eq!(tied(&found), [("a::c".to_string(), "t_c".to_string())]);
}

// mitra: evidence for reader::Rust::tests::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_tests_of_a_project_without_ties_is_empty() {
    let root = project(&[("src/lib.rs", "pub fn f() {}\n// a comment\n")]);
    assert_eq!(Rust.tests(&root).unwrap().len(), 0);
    done(&root);
    let bare = project(&[]);
    assert_eq!(Rust.tests(&bare).unwrap().len(), 0);
    done(&bare);
}

// mitra: evidence for reader::Rust::tests::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_tests_second_call_same_and_files_untouched() {
    let text = "// mitra: evidence for a::c\n#[test]\nfn t_c() {}\n";
    let root = project(&[("src/lib.rs", text), ("tests/x.rs", text)]);
    let first = Rust.tests(&root).unwrap();
    let second = Rust.tests(&root).unwrap();
    assert_eq!(first, second);
    assert_eq!(first.len(), 2);
    assert_eq!(
        std::fs::read_to_string(root.join("src/lib.rs")).unwrap(),
        text
    );
    assert_eq!(
        std::fs::read_to_string(root.join("tests/x.rs")).unwrap(),
        text
    );
    let mut names: Vec<String> = std::fs::read_dir(&root)
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    names.sort();
    done(&root);
    assert_eq!(names, ["src", "tests"]);
}

// mitra: evidence for source::finds_ties written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_tests_tie_without_a_fn_below_is_not_a_tied_test() {
    let root = project(&[(
        "src/lib.rs",
        "// mitra: evidence for a::c\npub struct S;\n\nfn later() {}\n",
    )]);
    let found = Rust.tests(&root);
    done(&root);
    // The name below the comment is no function: nothing may be tied to
    // `later`, which is not below the comment.
    match found {
        Err(_) => {}
        Ok(v) => assert!(v.is_empty(), "tied to {:?}", v),
    }
}

// ------------------------------------------------------------- Rust::tie

const MODEL: &str = "subject a {\n    fn f(x: Int) -> Int;\n}\n";

// mitra: evidence for reader::Rust::tie::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_tie_agrees_when_code_has_the_models_shape() {
    let root = project(&[(
        "src/lib.rs",
        "// mitra: implements a::f\npub fn f(x: i64) -> i64 {\n    x\n}\n",
    )]);
    let found = Rust.tie(&root, &model(MODEL), None).unwrap();
    done(&root);
    assert_eq!(
        (found.agree, found.differ),
        (1, 0),
        "notes: {:?}",
        found.notes
    );
    assert_eq!(found.evidence.len(), 1);
    assert!(found.evidence[0].contains("a::f"));
}

// mitra: evidence for reader::Rust::tie::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_tie_differs_when_inputs_are_not_the_models() {
    let root = project(&[(
        "src/lib.rs",
        "// mitra: implements a::f\npub fn f(x: i64, y: i64) -> i64 {\n    x + y\n}\n",
    )]);
    let found = Rust.tie(&root, &model(MODEL), None).unwrap();
    done(&root);
    assert_eq!((found.agree, found.differ), (0, 1));
    assert!(!found.notes.is_empty());
}

// mitra: evidence for reader::Rust::tie::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_tie_differs_when_the_result_is_not_the_models() {
    let root = project(&[(
        "src/lib.rs",
        "// mitra: implements a::f\npub fn f(x: i64) -> String {\n    x.to_string()\n}\n",
    )]);
    let found = Rust.tie(&root, &model(MODEL), None).unwrap();
    done(&root);
    assert_eq!((found.agree, found.differ), (0, 1));
}

// mitra: evidence for code::is_the_revision_named written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_tie_records_the_revision_the_caller_names_and_no_other() {
    let root = project(&[(
        "src/lib.rs",
        "// mitra: implements a::f\npub fn f(x: i64) -> i64 {\n    x\n}\n",
    )]);
    let m = model(MODEL);
    let with = Rust.tie(&root, &m, Some("abc1234")).unwrap();
    let without = Rust.tie(&root, &m, None).unwrap();
    done(&root);
    assert_eq!(with.evidence.len(), 1);
    assert!(with.evidence[0].contains("abc1234"), "{}", with.evidence[0]);
    assert_eq!(without.evidence.len(), 1);
    assert!(!without.evidence[0].contains("abc1234"));
    assert!(
        !without.evidence[0].contains("rev"),
        "{}",
        without.evidence[0]
    );
}

// mitra: evidence for reader::Rust::tie::correct written "agent:anthropic/fable-5.2/cold-w-rust" cold
#[test]
fn cold_tie_second_call_same_and_files_untouched() {
    let text = "// mitra: implements a::f\npub fn f(x: i64) -> i64 {\n    x\n}\n";
    let root = project(&[("src/lib.rs", text)]);
    let m = model(MODEL);
    let first = Rust.tie(&root, &m, Some("r1")).unwrap();
    let second = Rust.tie(&root, &m, Some("r1")).unwrap();
    assert_eq!(first.evidence, second.evidence);
    assert_eq!((first.agree, first.differ), (second.agree, second.differ));
    assert_eq!(
        std::fs::read_to_string(root.join("src/lib.rs")).unwrap(),
        text
    );
    let count = std::fs::read_dir(root.join("src")).unwrap().count();
    done(&root);
    assert_eq!(count, 1);
}
