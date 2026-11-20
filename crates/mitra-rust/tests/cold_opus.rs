//! Tests written from the model in `model/` alone.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use mitra_lang::ledger::results::{Outcome, Tied};
use mitra_lang::Reader;
use mitra_rust::report::libtest;
use mitra_rust::source::{fn_below, is_tie, mitra_line};
use mitra_rust::Rust;

static COUNTER: AtomicUsize = AtomicUsize::new(0);

/// A fresh, empty folder of its own.
fn fresh(tag: &str) -> PathBuf {
    let n = COUNTER.fetch_add(1, Ordering::SeqCst);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "mitra-rust-cold-{tag}-{}-{n}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn put(root: &Path, rel: &str, text: &str) {
    let p = root.join(rel);
    fs::create_dir_all(p.parent().unwrap()).unwrap();
    fs::write(p, text).unwrap();
}

fn out(name: &str, passed: bool) -> Outcome {
    Outcome {
        name: name.to_string(),
        passed,
    }
}

fn model(text: &str) -> mitra_lang::Model {
    match mitra_lang::load(text, "") {
        Ok(m) => m,
        Err(_) => panic!("the model does not load: {text}"),
    }
}

// ---------------------------------------------------------------- libtest

// mitra: evidence for reader::libtest written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_libtest_empty_report_has_no_results() {
    assert_eq!(libtest(""), Vec::<Outcome>::new());
}

// mitra: evidence for reader::libtest::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_libtest_keeps_order_and_skips_noise() {
    let text = "\nrunning 4 tests\n\
                test z::last_first ... FAILED\n\
                test m::skipped ... ignored\n\
                test a::first_last ... ok\n\
                \n\
                failures:\n\
                test result: FAILED. 1 passed; 1 failed; 1 ignored";
    assert_eq!(
        libtest(text),
        vec![out("last_first", false), out("first_last", true)]
    );
}

// mitra: evidence for reader::libtest::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_libtest_name_is_last_segment_of_deep_path() {
    assert_eq!(
        libtest("test outer::inner::deeper::t_deep ... FAILED"),
        vec![out("t_deep", false)]
    );
}

// mitra: evidence for reader::libtest::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_libtest_summary_line_ok_is_no_test() {
    // A line starting with `test` that is the summary, even when it says ok.
    assert_eq!(
        libtest("test result: ok. 3 passed; 0 failed; 0 ignored"),
        Vec::<Outcome>::new()
    );
}

// mitra: evidence for reader::libtest::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_libtest_crlf_lines() {
    assert_eq!(
        libtest("test a ... ok\r\ntest b ... FAILED\r\n"),
        vec![out("a", true), out("b", false)]
    );
}

// mitra: evidence for reader::libtest::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_libtest_failed_with_time() {
    assert_eq!(
        libtest("test slowpoke ... FAILED <1.250s>"),
        vec![out("slowpoke", false)]
    );
}

// mitra: evidence for reader::Rust::every_result_is_read written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_results_reads_every_test_same_as_libtest() {
    let text = "test x::one ... ok\ntest two ... FAILED\ntest three ... ok";
    let got = Rust.results(text);
    assert_eq!(
        got,
        vec![out("one", true), out("two", false), out("three", true)]
    );
    assert_eq!(got, libtest(text));
    // The second call gives the same.
    assert_eq!(Rust.results(text), got);
}

// ---------------------------------------------------------------- mitra_line

// mitra: evidence for source::mitra_line::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_mitra_line_tab_indented_and_no_space() {
    assert_eq!(
        mitra_line("\t\t// mitra: implements a::f"),
        Some("implements a::f")
    );
    assert_eq!(
        mitra_line("//mitra: implements a::f"),
        Some("implements a::f")
    );
}

// mitra: evidence for source::mitra_line::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_mitra_line_empty_after_colon() {
    assert_eq!(mitra_line("// mitra:"), Some(""));
}

// mitra: evidence for source::mitra_line::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_mitra_line_not_a_comment_or_wrong_word() {
    assert_eq!(mitra_line("mitra: implements a::f"), None);
    assert_eq!(mitra_line("// Mitra: implements a::f"), None);
    assert_eq!(mitra_line("// mitra implements a::f"), None);
    assert_eq!(mitra_line(""), None);
    assert_eq!(mitra_line("/ mitra: x"), None);
}

// ---------------------------------------------------------------- fn_below

// mitra: evidence for source::fn_below::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_fn_below_several_attributes_and_indent() {
    assert_eq!(
        fn_below("  #[test]\n  #[should_panic(expected = \"x\")]\n  fn t_cold() {}"),
        Some("t_cold")
    );
    assert_eq!(fn_below("#[test] fn same_line() {}"), Some("same_line"));
}

// mitra: evidence for source::fn_below::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_fn_below_word_starting_with_fn_is_no_fn() {
    assert_eq!(fn_below("fnord() {}"), None);
    assert_eq!(fn_below("let fn_x = 1;\nfn f() {}"), None);
    assert_eq!(fn_below("mod m;\nfn f() {}"), None);
}

// mitra: evidence for source::fn_below::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_fn_below_name_ends_at_generics_or_paren() {
    assert_eq!(fn_below("fn name_1<'a>(x: &'a str) {}"), Some("name_1"));
    assert_eq!(fn_below("pub(super) fn g(){}"), Some("g"));
}

// ---------------------------------------------------------------- is_tie

// mitra: evidence for source::is_tie::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_is_tie_only_the_two_forms() {
    assert!(is_tie("evidence for m::c"));
    assert!(is_tie("implements reader::Rust::tie"));
    assert!(!is_tie("claim c : correct(f(1))"));
    assert!(!is_tie("Implements a::f"));
    assert!(!is_tie("evidence a::c"));
    assert!(!is_tie(""));
    assert!(!is_tie("  then 1"));
}

// ---------------------------------------------------------------- Rust::tests

const LIB: &str = "pub fn add(a: i64) -> i64 { a }\n\
\n\
#[cfg(test)]\n\
mod tests {\n\
    // mitra: evidence for m::c\n\
    #[test]\n\
    fn t_one() {}\n\
}\n";

const INTEG: &str = "/// mitra: evidence for m::d given a = 1 gives 2\n\
#[test]\n\
fn t_two() {}\n";

// mitra: evidence for reader::Rust::tests::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_tests_finds_ties_in_src_and_tests_with_place_and_pin() {
    let root = fresh("tests-both");
    put(&root, "src/lib.rs", LIB);
    put(&root, "tests/integ.rs", INTEG);
    let mut got = Rust.tests(&root).expect("tests are read");
    got.sort_by(|a, b| a.test.cmp(&b.test));
    assert_eq!(got.len(), 2, "{got:?}");
    assert_eq!(got[0].test, "t_one");
    assert_eq!(got[0].claim, "m::c");
    assert_eq!(got[0].pinned, "");
    assert!(got[0].place.contains("lib.rs"), "{}", got[0].place);
    assert!(got[0].place.ends_with(":5"), "{}", got[0].place);
    assert_eq!(got[1].test, "t_two");
    assert_eq!(got[1].claim, "m::d");
    assert_eq!(got[1].pinned, "given a = 1 gives 2");
    assert!(got[1].place.contains("integ.rs"), "{}", got[1].place);
    assert!(got[1].place.ends_with(":1"), "{}", got[1].place);
    let _ = fs::remove_dir_all(&root);
}

// mitra: evidence for code::reads_what_is_there written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_tests_reads_nested_folders() {
    let root = fresh("tests-nested");
    put(
        &root,
        "src/a/b/deep.rs",
        "// mitra: evidence for m::deep\n#[test]\nfn t_deep() {}\n",
    );
    put(
        &root,
        "tests/sub/mod.rs",
        "// mitra: evidence for m::sub\n#[test]\nfn t_sub() {}\n",
    );
    let got = Rust.tests(&root).expect("tests are read");
    let mut names: Vec<&str> = got.iter().map(|t| t.test.as_str()).collect();
    names.sort();
    assert_eq!(names, ["t_deep", "t_sub"]);
    let _ = fs::remove_dir_all(&root);
}

// mitra: evidence for reader::Rust::tests::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_tests_only_rust_files_under_src_and_tests() {
    let root = fresh("tests-only-rs");
    let tie = "// mitra: evidence for m::c\n#[test]\nfn t_x() {}\n";
    put(&root, "src/notes.txt", tie);
    put(&root, "src/lib.rs.bak", tie);
    put(&root, "benches/b.rs", tie);
    put(&root, "build.rs", tie);
    put(&root, "examples/e.rs", tie);
    put(&root, "src/lib.rs", "pub fn f() {}\n");
    let got = Rust.tests(&root).expect("tests are read");
    assert_eq!(got, Vec::<Tied>::new());
    let _ = fs::remove_dir_all(&root);
}

// mitra: evidence for reader::Rust::tests::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_tests_implements_is_not_a_tied_test() {
    let root = fresh("tests-implements");
    put(
        &root,
        "src/lib.rs",
        "// mitra: implements m::f\npub fn f() {}\n",
    );
    assert_eq!(
        Rust.tests(&root).expect("tests are read"),
        Vec::<Tied>::new()
    );
    let _ = fs::remove_dir_all(&root);
}

// mitra: evidence for source::finds_ties written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_tests_tie_inside_string_or_block_comment_is_none() {
    let root = fresh("tests-in-string");
    let text = "pub const S: &str = r#\"\n\
// mitra: evidence for m::in_string\n\
fn not_a_test() {}\n\
\"#;\n\
/*\n\
// mitra: evidence for m::in_comment\n\
fn nor_this() {}\n\
*/\n\
// mitra: evidence for m::real\n\
#[test]\n\
fn t_real() {}\n";
    put(&root, "src/lib.rs", text);
    let got = Rust.tests(&root).expect("tests are read");
    assert_eq!(got.len(), 1, "{got:?}");
    assert_eq!(got[0].test, "t_real");
    assert_eq!(got[0].claim, "m::real");
    let _ = fs::remove_dir_all(&root);
}

// mitra: evidence for reader::Rust::reads_only written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_tests_and_tie_leave_files_untouched_and_repeat() {
    let root = fresh("reads-only");
    let code = "// mitra: implements m::f\npub fn f(a: i64) -> i64 { a }\n";
    put(&root, "src/lib.rs", code);
    put(&root, "tests/integ.rs", INTEG);
    let list = |root: &Path| {
        let mut all = Vec::new();
        for d in ["src", "tests"] {
            for e in fs::read_dir(root.join(d)).unwrap() {
                all.push(e.unwrap().path());
            }
        }
        all.sort();
        all
    };
    let before = list(&root);
    let first = Rust.tests(&root).expect("tests are read");
    let second = Rust.tests(&root).expect("tests are read again");
    assert_eq!(first, second);
    let m = model("subject m { fn f(a: Int) -> Int; }");
    let a = Rust.tie(&root, &m, None).expect("tie");
    let b = Rust.tie(&root, &m, None).expect("tie again");
    assert_eq!(a.evidence, b.evidence);
    assert_eq!(list(&root), before);
    assert_eq!(fs::read_to_string(root.join("src/lib.rs")).unwrap(), code);
    assert_eq!(
        fs::read_to_string(root.join("tests/integ.rs")).unwrap(),
        INTEG
    );
    let _ = fs::remove_dir_all(&root);
}

// ---------------------------------------------------------------- Rust::tie

// mitra: evidence for reader::Rust::tie::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_tie_agreeing_function_holds_with_rev() {
    let root = fresh("tie-agree");
    put(
        &root,
        "src/lib.rs",
        "// mitra: implements m::f\npub fn f(a: String) -> String { a }\n",
    );
    let m = model("subject m { fn f(a: String) -> String; }");
    let found = Rust.tie(&root, &m, Some("abc123")).expect("tie");
    assert_eq!(found.agree, 1);
    assert_eq!(found.differ, 0);
    assert_eq!(found.evidence.len(), 1, "{:?}", found.evidence);
    let e = &found.evidence[0];
    assert!(e.contains("for m::f::tied holds"), "{e}");
    assert!(e.contains("rev \"abc123\""), "{e}");
    assert!(found.untied.is_empty(), "{:?}", found.untied);
    let _ = fs::remove_dir_all(&root);
}

// mitra: evidence for reader::Rust::compares_shape written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_tie_wrong_inputs_fail_without_rev() {
    let root = fresh("tie-differ");
    put(
        &root,
        "src/lib.rs",
        "// mitra: implements m::f\npub fn f(a: String, b: String) -> String { a }\n",
    );
    let m = model("subject m { fn f(a: String) -> String; }");
    let found = Rust.tie(&root, &m, None).expect("tie");
    assert_eq!(found.agree, 0);
    assert_eq!(found.differ, 1);
    let e = found
        .evidence
        .iter()
        .find(|e| e.contains("m::f::tied"))
        .expect("evidence on the tie");
    assert!(e.contains("fails"), "{e}");
    assert!(!e.contains(" rev "), "{e}");
    let _ = fs::remove_dir_all(&root);
}

// mitra: evidence for reader::Rust::compares_shape written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_tie_wrong_result_type_fails() {
    let root = fresh("tie-result");
    put(
        &root,
        "src/lib.rs",
        "// mitra: implements m::f\npub fn f(a: String) -> bool { true }\n",
    );
    let m = model("subject m { fn f(a: String) -> String; }");
    let found = Rust.tie(&root, &m, None).expect("tie");
    assert_eq!(found.differ, 1);
    assert_eq!(found.agree, 0);
    let _ = fs::remove_dir_all(&root);
}

// mitra: evidence for reader::Rust::compares_shape written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_tie_record_missing_field_fails() {
    let root = fresh("tie-record");
    put(
        &root,
        "src/lib.rs",
        "// mitra: implements m::R\npub struct R {\n    pub a: String,\n}\n",
    );
    let m = model("subject m { record R { a: String, b: String, } }");
    let found = Rust.tie(&root, &m, None).expect("tie");
    assert_eq!(found.differ, 1, "{:?}", found.evidence);
    let e = found
        .evidence
        .iter()
        .find(|e| e.contains("m::R::tied"))
        .expect("evidence on the record");
    assert!(e.contains("fails"), "{e}");
    let _ = fs::remove_dir_all(&root);
}

// mitra: evidence for reader::Rust::compares_calls written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_tie_missing_used_call_fails() {
    let root = fresh("tie-uses");
    put(
        &root,
        "src/lib.rs",
        "// mitra: implements m::g\npub fn g() -> String { String::new() }\n\n\
         // mitra: implements m::f\npub fn f() -> String { String::new() }\n",
    );
    let m = model("subject m { fn g() -> String; fn f() -> String uses g; }");
    let found = Rust.tie(&root, &m, None).expect("tie");
    let fails: Vec<&String> = found
        .evidence
        .iter()
        .filter(|e| e.contains("m::f::uses") && e.contains("fails"))
        .collect();
    assert_eq!(fails.len(), 1, "{:?}", found.evidence);
    let _ = fs::remove_dir_all(&root);
}

// mitra: evidence for reader::Rust::compares_calls written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_tie_unstated_call_outside_subject_fails() {
    let root = fresh("tie-unstated");
    put(
        &root,
        "src/a.rs",
        "// mitra: implements other::helper\npub fn helper() -> String { String::new() }\n",
    );
    put(
        &root,
        "src/lib.rs",
        "pub mod a;\n// mitra: implements m::f\npub fn f() -> String { a::helper() }\n",
    );
    let m = model("subject other { fn helper() -> String; } subject m { fn f() -> String; }");
    let found = Rust.tie(&root, &m, None).expect("tie");
    assert!(
        found
            .evidence
            .iter()
            .any(|e| e.contains("m::f::uses") && e.contains("fails")),
        "{:?}",
        found.evidence
    );
    let _ = fs::remove_dir_all(&root);
}

// mitra: evidence for reader::Rust::tie::correct written "agent:anthropic/opus/cold-w-rust" cold
#[test]
fn cold_tie_lists_untied_functions() {
    let root = fresh("tie-untied");
    put(
        &root,
        "src/lib.rs",
        "// mitra: implements m::f\npub fn f(a: String) -> String { a }\n",
    );
    let m = model("subject m { fn f(a: String) -> String; fn g() -> String; }");
    let found = Rust.tie(&root, &m, None).expect("tie");
    assert_eq!(found.untied, vec!["m::g".to_string()]);
    let _ = fs::remove_dir_all(&root);
}
