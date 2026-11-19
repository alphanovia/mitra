use std::process::Command;

const EXAMPLE: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../examples/reworked/readme"
);

const FLIP: &str = "subject s {
    fn flip(a: bool) -> bool;
    claim flip_correct(a: bool) { correct(flip(a)) }
    claim flip_on_true() { correct(flip(true)) }
}
";

#[test]
fn eval_writes_the_result_file() {
    let out = std::env::temp_dir().join(format!("mitra-cli-test-{}.txt", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_mitra"))
        // From the case's folder, so that the result names the two files
        // as the expected one does.
        .current_dir(EXAMPLE)
        .args(["eval", "model.mitra", "evidence.mitra", "-o"])
        .arg(&out)
        .status()
        .unwrap();
    // The case has a failing claim: `eval` exits with 1 on errors.
    assert_eq!(status.code(), Some(1));
    let expected = std::fs::read_to_string(format!("{EXAMPLE}/expected.txt")).unwrap();
    assert_eq!(std::fs::read_to_string(&out).unwrap(), expected);
    let _ = std::fs::remove_file(out);
}

#[test]
fn bad_evidence_names_the_file_and_line() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("ev.mitra"), "\nevidence e { for flop holds }\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mitra"))
        .args(["eval", &format!("{EXAMPLE}/model.mitra")])
        .arg(dir.join("ev.mitra"))
        .output()
        .unwrap();
    // Evidence on a piece the model does not have is set aside and
    // shown; evidence that does not read stops the command.
    assert_eq!(out.status.code(), Some(0));
    let said = String::from_utf8(out.stdout).unwrap();
    assert!(said.contains("warning[M107]"), "{said}");
    assert!(
        said.contains("ev.mitra:2  evidence e { for flop holds }"),
        "{said}"
    );
    assert!(
        said.contains("The model has no `flop`. It judges nothing."),
        "{said}"
    );
    std::fs::write(dir.join("ev.mitra"), "\nevidence e { flop }\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mitra"))
        .args(["eval", &format!("{EXAMPLE}/model.mitra")])
        .arg(dir.join("ev.mitra"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(err.contains("ev.mitra: line 2: "), "{err}");
    let _ = std::fs::remove_dir_all(dir);
}

/// The model alone: no evidence file is needed to check it.
#[test]
fn the_evidence_file_is_optional() {
    let model = format!("{EXAMPLE}/model.mitra");
    let dir = std::env::temp_dir();
    let (code, out, err) = mitra(&dir, &["check", &model]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.starts_with("ok: "), "{out}");
    let (code, out, _) = mitra(&dir, &["eval", &model]);
    assert_eq!(code, Some(0));
    assert!(out.contains("unverified"), "{out}");
    assert!(!out.contains("fails"), "{out}");
}

/// The syntax of before the rework is refused, with what to write.
#[test]
fn the_earlier_syntax_is_refused() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-earlier-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("m.mitra"),
        "record R { a: bool }\nopaque f(a: Bool) -> Bool\n",
    )
    .unwrap();
    std::fs::write(dir.join("e.mitra"), "").unwrap();
    let (code, _, err) = mitra(&dir, &["eval", "m.mitra", "e.mitra"]);
    assert_eq!(code, Some(2));
    assert!(
        err.contains("m.mitra: line 2: this syntax is no longer read: `opaque f(a: T) -> U` is written `fn f(a: T) -> U;`"),
        "{err}"
    );
    std::fs::write(dir.join("m.mitra"), FLIP).unwrap();
    std::fs::write(
        dir.join("e.mitra"),
        "evidence correct(s::flip(true)) at 1\n",
    )
    .unwrap();
    let (code, _, err) = mitra(&dir, &["eval", "m.mitra", "e.mitra"]);
    assert_eq!(code, Some(2));
    assert!(
        err.contains("e.mitra: line 1: expected an `evidence` block"),
        "{err}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

/// `mitra` run in `dir`, with its exit code, stdout and stderr.
fn mitra(dir: &std::path::Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_mitra"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    let s = |b: Vec<u8>| String::from_utf8(b).unwrap();
    (out.status.code(), s(out.stdout), s(out.stderr))
}

/// A new store holding `model`.
fn store(name: &str, model: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("mitra-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    std::fs::write(dir.join(".mitra/model.mitra"), model).unwrap();
    dir
}

#[test]
fn store_init_add_check_eval() {
    let dir = store("store", FLIP);
    assert_eq!(mitra(&dir, &["init"]).0, Some(2), "init never overwrites");
    // From a subfolder: the store is found above.
    let sub = dir.join("sub");
    let good = "evidence r { for flip given a = true holds }";
    let (code, out, err) = mitra(
        &sub,
        &["add", good, "--source", "review:ana", "--rev", "3f2a"],
    );
    assert_eq!(
        (code, out.starts_with("added 1 evidence")),
        (Some(0), true),
        "{err}"
    );
    // Rejected evidence leaves the store unchanged.
    let before = std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap();
    let (code, _, err) = mitra(&sub, &["add", "evidence r { for flop holds }"]);
    assert_eq!(code, Some(2));
    assert!(err.contains("`flop`"), "{err}");
    assert_eq!(
        std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap(),
        before
    );
    assert!(
        before.ends_with(
            "evidence r { for flip given a = true holds source \"review:ana\" rev \"3f2a\" seen \"47d8e0c71f5a\" }\n"
        ),
        "{before}"
    );
    // Evidence on an unknown function is refused, and blamed on the new text.
    let two = "evidence a { for flip given a = true gives false }\nevidence b { for flop holds }";
    let (code, _, err) = mitra(&sub, &["add", two]);
    assert_eq!(code, Some(2));
    assert!(err.contains("evidence to add: line 2: "), "{err}");
    // --source twice, or in both the option and the text, is refused.
    assert_eq!(
        mitra(&sub, &["add", good, "--source", "a", "--source", "b"]).0,
        Some(2)
    );
    let said = "evidence r { for flip given a = true holds source \"a\" }";
    let (_, _, err) = mitra(&sub, &["add", said, "--source", "b"]);
    assert!(err.contains("--source is given twice"), "{err}");
    assert_eq!(mitra(&sub, &["eval", "-o", "x.mitra"]).0, Some(2));
    mitra(
        &sub,
        &["add", "evidence w { for flip given a in {false} fails }"],
    );
    let (code, out, _) = mitra(&sub, &["check"]);
    assert_eq!(
        (code, out.as_str()),
        (Some(0), "ok: 1 functions, 3 claims, 2 evidence\n")
    );
    let (code, out, err) = mitra(&sub, &["eval"]);
    assert!(
        out.contains(
            "s::flip_correct: fails\n  fails on        `s::flip(false)`\n  \
             passes on       `s::flip(true)`\ns::flip_on_true: holds\n"
        ),
        "{out}"
    );
    assert_eq!(
        (code, err.as_str()),
        (Some(1), "mitra: 1 errors, 1 warnings\n")
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn ingest_turns_test_results_into_evidence() {
    let model = format!("{FLIP}subject t {{ claim on_false() {{ correct(s::flip(false)) }} }}\n");
    let dir = store("ingest", &model);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let tests = "// mitra: evidence for flip_on_true\n#[test]\nfn t_true() {}\n\
                // mitra: evidence for t::on_false\n#[test]\nfn t_false() {}\n";
    std::fs::write(dir.join("tests/t.rs"), tests).unwrap();
    let report = "running 4 tests\n\
                  test tests::t_true ... ok\n\
                  test tests::t_false ... FAILED\n\
                  test t_all ... FAILED\n\
                  test t_other ... ok\n";
    std::fs::write(dir.join("report.txt"), report).unwrap();
    let (code, out, err) = mitra(&dir, &["ingest", "report.txt", "--rev", "3f2a"]);
    assert_eq!(code, Some(0), "{err}");
    assert_eq!(
        out,
        "4 results: 2 passed, 2 failed; 2 evidence written\n\
         FAILED, and tied to no claim, so no verdict shows it: t_all\n\
         tied to no claim: t_other\n"
    );
    let evidence = std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap();
    assert!(
        evidence.ends_with(
            "evidence test { for s::flip_on_true holds source \"test:t_true\" rev \"3f2a\" }\n\
             evidence test { for t::on_false fails source \"test:t_false\" rev \"3f2a\" }\n"
        ),
        "{evidence}"
    );
    let (_, out, _) = mitra(&dir, &["eval"]);
    assert!(out.contains("\ns::flip_correct: fails\n"), "{out}");
    assert!(
        out.contains(".mitra/evidence.mitra:3  evidence test { for t::on_false fails"),
        "{out}"
    );
    // A comment that names no claim stops the ingest.
    std::fs::write(dir.join("tests/t.rs"), tests.replace("t::on_false", "nope")).unwrap();
    let (code, _, err) = mitra(&dir, &["ingest", "report.txt"]);
    assert_eq!(code, Some(2));
    assert!(
        err.contains("t.rs:4: the model has no claim `nope`"),
        "{err}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_broken_store_is_blamed_not_the_new_evidence() {
    let dir = store("broken", FLIP);
    let ev = dir.join(".mitra/evidence.mitra");
    let broken = std::fs::read_to_string(&ev).unwrap() + "evidence x { for flip nope }\n";
    std::fs::write(&ev, &broken).unwrap();
    let (code, _, err) = mitra(
        &dir,
        &["add", "evidence r { for flip given a = true holds }"],
    );
    assert_eq!(code, Some(2));
    assert!(err.contains("evidence.mitra: line 2: "), "{err}");
    assert!(!err.contains("evidence to add"), "{err}");
    assert_eq!(std::fs::read_to_string(&ev).unwrap(), broken);
    let _ = std::fs::remove_dir_all(dir);
}

/// Dogfood: each store of Mitra (its claims, and evidence from its own test
/// runs, recorded by scripts/dogfood.sh) gives the expected verdicts.
#[test]
fn mitra_stores_evaluate() {
    // Not while scripts/breaks.py replaces the evidence.
    if std::env::var_os("MITRA_BREAKS").is_some() {
        return;
    }
    for name in ["mitra-lang", "mitra-store", "mitra-rust", "mitra-cli"] {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join(name);
        let (code, out, err) = mitra(&dir, &["eval"]);
        assert_eq!(code, Some(0), "{name}: {err}");
        let expected = std::fs::read_to_string(dir.join(".mitra/expected.txt")).unwrap();
        assert_eq!(out, expected, "{name}");
    }
}

/// Naming a store's two files from another folder reads the same model.
#[test]
fn mitra_store_evaluates_by_path() {
    if std::env::var_os("MITRA_BREAKS").is_some() {
        return;
    }
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let files = [
        "crates/mitra-lang/.mitra/model.mitra",
        "crates/mitra-lang/.mitra/evidence.mitra",
    ];
    let (code, out, err) = mitra(&root, &["eval", files[0], files[1]]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.contains("model::Model::tied: holds"), "{out}");
}

#[test]
fn rev_leaves_out_stale_evidence() {
    let dir = store("rev", FLIP);
    let on = |v: &str| format!("evidence t {{ for flip given a = {v} holds }}");
    mitra(&dir, &["add", &on("true"), "--rev", "old"]);
    mitra(&dir, &["add", &on("false"), "--rev", "new"]);
    // Evidence with no revision is stale under --rev too: else this old
    // pass would outlive any newer failure.
    mitra(&dir, &["add", &on("true")]);
    let (_, all, _) = mitra(&dir, &["eval"]);
    assert!(all.contains("\ns::flip_correct: holds\n"), "{all}");
    let (code, out, err) = mitra(&dir, &["eval", "--rev", "new"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        err,
        "mitra: 2 evidence left out: not recorded at revision new\n\
         mitra: 0 errors, 3 warnings\n"
    );
    assert!(
        out.contains(
            "s::flip_correct: unverified\n  passes on       `s::flip(false)`\n  \
             no evidence on  `s::flip(true)`\n\
             s::flip_on_true: unverified\n  no evidence on  `s::flip(true)`\n"
        ) && out.contains(
            "warning[M101]: a claim is not verified\n  \
             --> .mitra/model.mitra:3  claim flip_correct(a: bool) { correct(flip(a)) }\n\
             \n  \
             No evidence on `s::flip(true)`.\n  \
             help: `mitra next` lists the evidence that would settle it\n"
        ),
        "{out}"
    );
    let (_, out, _) = mitra(&dir, &["next", "--rev", "new"]);
    assert!(out.contains("s::flip_correct: missing {true}\n"), "{out}");
    assert_eq!(mitra(&dir, &["eval", "--rev"]).0, Some(2));
    assert_eq!(mitra(&dir, &["eval", "--rev", ""]).0, Some(2));
    assert_eq!(mitra(&dir, &["eval", "--rev", "--rev"]).0, Some(2));
    assert_eq!(mitra(&dir, &["add", &on("true"), "--rev", ""]).0, Some(2));
    let _ = std::fs::remove_dir_all(dir);
}

/// A comment in the code only ties it to the model: a model written in
/// comments is refused, with the file and line.
#[test]
fn comments_hold_only_ties() {
    let dir = store("rust", FLIP);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    let code = "// mitra: implements s::flip\npub fn flip(a: bool) -> bool { !a }\n";
    std::fs::write(dir.join("src/lib.rs"), code).unwrap();
    let (code, out, err) = mitra(&dir, &["check"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.starts_with("ok: 1 functions, 3 claims"), "{out}");
    let old = "/// mitra: claim flip_ok(a: bool) { correct(flip(a)) }\npub fn flip(a: bool) -> bool { !a }\n";
    std::fs::write(dir.join("src/lib.rs"), old).unwrap();
    let (code, _, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(2));
    assert!(
        err.contains("lib.rs: line 1: a model in `mitra:` comments is no longer read"),
        "{err}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

/// A failed test of `f(x) == v` on a function that returns `bool` gives
/// the other output. Otherwise it refutes the specs that give `v` at `x`.
#[test]
fn a_failed_bool_output_test_gives_the_other_output() {
    let model = "subject s {
    fn flip(a: bool) -> bool;
    fn inc(x: Int) -> Int;
    fn dec(x: Int) -> Int { x - 1 }
    claim t_flip() { flip(true) == true }
    claim t_inc() { inc(1) == 2 }
    claim t_dec() { dec(5) == 4 }
}
";
    let dir = store("boolout", model);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let code =
        "// mitra: evidence for t_flip\nfn a() {}\n// mitra: evidence for t_inc\nfn b() {}\n\
                // mitra: evidence for t_dec\nfn c() {}\n";
    std::fs::write(dir.join("tests/t.rs"), code).unwrap();
    std::fs::write(
        dir.join("report.txt"),
        "test a ... FAILED\ntest b ... FAILED\ntest c ... FAILED\n",
    )
    .unwrap();
    let (code, out, err) = mitra(&dir, &["ingest", "report.txt"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.contains("3 evidence written"), "{out}");
    let (_, out, _) = mitra(&dir, &["eval"]);
    // `dec(5)` was not 4, which is what the spec says it is.
    assert!(
        out.contains("s::dec::spec: fails\n  fails on        `s::dec(5)`\n"),
        "{out}"
    );
    assert!(out.contains("s::t_flip: fails"), "{out}");
    let _ = std::fs::remove_dir_all(dir);
}

/// Mistakes made in the first trial: the message says what to write.
#[test]
fn add_says_what_to_write_instead() {
    let model = "subject s { fn add(l: Int, r: Int) -> Int;\n\
                 claim add_ok(l: Int, r: Int) { correct(add(l, r)) } }\n";
    let dir = store("hints", model);
    let cases: [(&[&str], &str); 3] = [
        (
            &["add", "evidence", "add_ok", "at", "1", "--source", "me"],
            "mitra add 'evidence add_ok at 1'",
        ),
        (
            &["add", "evidence add_ok at 1"],
            "evidence is written `evidence name { for f given a = 1 holds }`",
        ),
        (
            &["add", "evidence r { for add_ok gives 3 }"],
            "`s::add_ok` is a claim: `holds` or `fails`",
        ),
    ];
    for (args, hint) in cases {
        let (code, _, err) = mitra(&dir, args);
        assert_eq!(code, Some(2), "{args:?}");
        assert!(err.contains(hint), "{args:?}: {err}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A model aside from the code, tied by `// mitra: implements` comments:
/// `mitra tie` compares the two and records what it finds as evidence.
#[test]
fn tie_compares_the_code_with_the_model() {
    let from = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/tie/bank");
    let dir = std::env::temp_dir().join(format!("mitra-cli-tie-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    for f in [".mitra/model.mitra", "src/lib.rs"] {
        std::fs::copy(from.join(f), dir.join(f)).unwrap();
    }
    let (code, out, err) = mitra(&dir, &["tie", "--rev", "abc"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(
        out.starts_with("2 agree, 2 differ; 4 evidence written"),
        "{out}"
    );
    assert!(
        out.contains("input `amount` in the code is `n` in the model"),
        "{out}"
    );
    assert!(out.contains("`withdraw` also calls `deposit`"), "{out}");
    assert!(out.contains("no code is tied to: bank::close"), "{out}");
    let (code, out, _) = mitra(&dir, &["eval"]);
    assert_eq!(code, Some(1));
    assert!(out.contains("bank::deposit::tied: holds"), "{out}");
    assert!(out.contains("bank::withdraw::tied: fails"), "{out}");
    assert!(
        out.contains("The model and the code of `bank::withdraw` do not agree"),
        "{out}"
    );
    // A test tied to a claim by a comment is evidence for that claim.
    let report = "test deposit_adds ... ok\ntest other ... FAILED\n";
    std::fs::write(dir.join("report.txt"), report).unwrap();
    let (code, out, err) = mitra(&dir, &["ingest", "report.txt", "--rev", "abc"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.contains("1 evidence written"), "{out}");
    assert!(
        out.contains("tied to no claim, so no verdict shows it: other"),
        "{out}"
    );
    let (_, out, _) = mitra(&dir, &["eval"]);
    assert!(out.contains("deposit_adds: holds"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A record is tied to its struct: the fields the model states are
/// there, and no other unless the record says `..`.
#[test]
fn tie_compares_a_record_with_its_struct() {
    let from = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/tie/records");
    let dir = std::env::temp_dir().join(format!("mitra-cli-records-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    for f in [".mitra/model.mitra", "src/lib.rs"] {
        std::fs::copy(from.join(f), dir.join(f)).unwrap();
    }
    let (code, out, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.starts_with("5 agree, 1 differ"), "{out}");
    assert!(
        out.contains("field `y` is `String` in the code, `Int` in the model"),
        "{out}"
    );
    // A field the record does not state is forgotten, and said.
    let ledger = std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap();
    assert!(ledger.contains("erased 1: field(b)"), "{ledger}");
    let (code, out, _) = mitra(&dir, &["eval"]);
    assert_eq!(code, Some(1));
    for want in [
        "check::Model::tied: holds",
        "check::Claim::tied: holds",
        "check::Point::tied: fails",
    ] {
        assert!(out.contains(want), "{out}");
    }
    assert!(
        out.contains("The record `check::Point` and its struct do not agree"),
        "{out}"
    );
    assert!(!out.contains("::prose"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// An enum is tied to the enum of the code: the same variants, each
/// holding as many values, of types that fit.
#[test]
fn tie_compares_an_enum_with_its_enum() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-enums-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    let model = "subject shapes {\n\
                 enum Shape { Dot, Circle(radius: f64), Box(w: f64, h: f64) }\n\
                 enum Side { Left, Right }\n\
                 enum Unit { Mm(n: Int), Inch(n: Int) }\n}\n";
    let code = "// mitra: implements shapes::Shape\n\
                pub enum Shape { Dot, Circle(f64), Box { w: f64, h: f64 } }\n\
                // mitra: implements shapes::Side\n\
                pub enum Side { Left, Right, Up }\n\
                // mitra: implements shapes::Unit\n\
                pub enum Unit { Mm(i64), Inch(String) }\n";
    std::fs::write(dir.join(".mitra/model.mitra"), model).unwrap();
    std::fs::write(dir.join("src/lib.rs"), code).unwrap();
    let (code, out, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.starts_with("1 agree, 2 differ"), "{out}");
    assert!(
        out.contains("the enum has a variant `Up` the model does not state"),
        "{out}"
    );
    assert!(
        out.contains("`n` of variant `Inch` is `String` in the code, `Int` in the model"),
        "{out}"
    );
    let (_, out, _) = mitra(&dir, &["eval"]);
    for want in ["shapes::Shape::tied: holds", "shapes::Side::tied: fails"] {
        assert!(out.contains(want), "{out}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A method of `impl Trait for Type` is tied as the function the
/// contract gives its implementer; `impl Trait` in the code fits an
/// `impl Contract` of the model.
#[test]
fn tie_compares_an_implementer_with_its_impl() {
    let dir = std::env::temp_dir().join(format!("mitra-tie-contract-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    let model = "contract Shape { fn area(&self) -> Int; fn scale(&self, by: Int) -> Self; }\n\
                 subject shapes {\n\
                 record Square { side: Int }\n\
                 record Dot;\n\
                 impl Shape for Square { }\n\
                 impl Shape for Dot { }\n\
                 fn total(s: impl Shape) -> Int uses Shape::area;\n}\n";
    let code = "pub trait Shape { fn area(&self) -> i64; fn scale(&self, by: i64) -> Self; }\n\
                // mitra: implements shapes::Square\n\
                pub struct Square { side: i64 }\n\
                pub struct Dot;\n\
                impl Shape for Square {\n\
                // mitra: implements shapes::Square::area\n\
                fn area(&self) -> i64 { self.side * self.side }\n\
                // mitra: implements shapes::Square::scale\n\
                fn scale(&self, by: i64) -> Self { Square { side: self.side * by } }\n\
                }\n\
                impl Shape for Dot {\n\
                // mitra: implements shapes::Dot::area\n\
                fn area(&self) -> String { String::new() }\n\
                fn scale(&self, by: i64) -> Self { Dot }\n\
                }\n\
                // mitra: implements shapes::total\n\
                pub fn total(s: impl Shape) -> i64 { s.area() }\n";
    std::fs::write(dir.join(".mitra/model.mitra"), model).unwrap();
    std::fs::write(dir.join("src/lib.rs"), code).unwrap();
    let (code, out, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.starts_with("5 agree, 1 differ"), "{out}");
    // A function of the contract is called by its name: `uses Shape::area`.
    let ledger = std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap();
    assert!(ledger.contains("for shapes::total::uses holds"), "{ledger}");
    assert!(out.contains("shapes::Dot::area"), "{out}");
    // What the contract gives an implementer and no code is tied to.
    let (_, out, _) = mitra(&dir, &["conform"]);
    assert!(out.contains("shapes::Dot::scale"), "{out}");
    assert!(!out.contains("Shape::area\n"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Type parameters and bounds are compared with the code: a record
/// with a parameter with its struct, an input bound by a contract with
/// the traits that bound it.
#[test]
fn tie_compares_bounds_and_type_parameters() {
    let dir = std::env::temp_dir().join(format!("mitra-tie-generics-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    let model = "import std::rust::{Option};\n\
                 contract Link { fn send(&self, bytes: List<Int>) -> Bool; }\n\
                 subject sync {\n\
                 fn push<L: Link>(l: L, data: List<Int>) -> Bool uses Link::send;\n\
                 fn pull<L>(l: &L) -> List<Int> where L: Link;\n\
                 fn more<L: Link, M>(l: L, m: M) -> Bool;\n\
                 fn both(a: impl Link, b: impl Link, c: impl Link) -> Bool;\n\
                 record Stack<T> { items: List<T> }\n\
                 record Pair<A, B> { first: A }\n\
                 impl<T> Stack<T> { fn top(&self) -> Option<T>;\n\
                 fn swap<U: Link>(&self, u: U) -> Stack<U>; }\n}\n";
    let code = "pub trait Link { fn send(&self, bytes: Vec<i64>) -> bool; }\n\
                // mitra: implements sync::push\n\
                pub fn push<L: Link + Clone>(l: L, data: Vec<i64>) -> bool { l.send(data) }\n\
                // mitra: implements sync::pull\n\
                pub fn pull<L>(l: &L) -> Vec<i64> where L: Clone { vec![] }\n\
                // mitra: implements sync::more\n\
                pub fn more<L: Link + Ord, M: Send + 'static>(l: L, m: M) -> bool { true }\n\
                // mitra: implements sync::both\n\
                pub fn both<C>(a: impl Link, b: &dyn Link, c: C) -> bool\n\
                where C: crate::Link { true }\n\
                // mitra: implements sync::Stack\n\
                pub struct Stack<T> { items: Vec<T> }\n\
                // mitra: implements sync::Pair\n\
                pub struct Pair<A, B> { first: A, second: B }\n\
                impl<T> Stack<T> {\n\
                // mitra: implements sync::Stack::top\n\
                pub fn top(&self) -> Option<T> { None }\n\
                // mitra: implements sync::Stack::swap\n\
                pub fn swap<U>(&self, u: U) -> Stack<U> { Stack { items: vec![] } }\n\
                }\n";
    std::fs::write(dir.join(".mitra/model.mitra"), model).unwrap();
    std::fs::write(dir.join("src/lib.rs"), code).unwrap();
    let (code, out, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.starts_with("7 agree, 2 differ"), "{out}");
    for want in [
        "sync::pull::tied: input `l` is bound by `Link` in the model, by `Clone` in the code",
        "sync::Stack::swap::tied: input `u` is bound by `Link` in the model, by nothing in the code",
            ] {
        assert!(out.contains(want), "{out}");
    }
    // A contract is nobody's code.
    assert!(!out.contains("no code is tied to"), "{out}");
    // A bound of the code alone is a detail of the code: said, no drift.
    let ledger = std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap();
    assert!(
        ledger.contains("for sync::push::tied holds")
            && ledger.contains("bound(l: Clone)")
            && ledger.contains("for sync::more::tied holds trust 0.96")
            && ledger.contains("bound(l: Ord), bound(m: Send)")
            && ledger.contains("for sync::Pair::tied holds trust 0.95")
            && ledger.contains("erased 1: field(second)"),
        "{ledger}"
    );
    // The interface is written with the parameters and the bounds.
    assert_eq!(mitra(&dir, &["build"]).0, Some(0));
    let interface = std::fs::read_to_string(dir.join(".mitra/build/interface.mitra")).unwrap();
    for want in [
        "fn push<L: Link>(l: L, data: List<Int>) -> Bool;",
        "fn pull<L>(l: &L) -> List<Int> where L: Link;",
        "fn both(a: impl Link, b: impl Link, c: impl Link) -> Bool;",
        "record Stack<T> { items: List<T> }",
        "fn swap<T, U: Link>(self: sync::Stack<T>, u: U) -> sync::Stack<U>;",
    ] {
        assert!(interface.contains(want), "no `{want}` in\n{interface}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// `mitra explain <claim>`: the claim as written, as read, and its verdict.
#[test]
fn explain_a_claim() {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/reworked/shapes");
    let files = ["model.mitra", "evidence.mitra"];
    let (code, out, err) = mitra(&dir, &["explain", "mul_grows", files[0], files[1]]);
    assert_eq!(code, Some(0), "{err}");
    assert!(
        out.contains("written   claim mul_grows = only_when(mul, 0..=2, |a, b| b > 0);"),
        "{out}"
    );
    assert!(out.contains("!(b > 0) || calc::mul(a, b) >= a"), "{out}");
    assert!(out.contains("calc::mul_grows: unverified"), "{out}");
    let (code, _, err) = mitra(&dir, &["explain", "nope", files[0], files[1]]);
    assert_eq!(code, Some(2));
    assert!(err.contains("no such claim"), "{err}");
    assert_eq!(mitra(&dir, &["explain", "M001"]).0, Some(0));
}

/// `mitra plan`: what each claim owes, from the model alone.
#[test]
fn plan_lists_the_evidence_owed() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/reworked/plan");
    let (code, out, err) = mitra(&dir, &["plan", "model.mitra", "evidence.mitra"]);
    assert_eq!(code, Some(0), "{err}");
    // One point per class, a pinned point with what the spec expects, and
    // a library claim point by point.
    assert!(
        out.contains("any one point of { (a, b) | b == 0 }"),
        "{out}"
    );
    assert!(out.contains("calc::div(6, 2), expected 3"), "{out}");
    assert!(out.contains("calc::add(0, 1), expected 1"), "{out}");
    assert!(
        out.contains("calc::add_commutes owed:\n  calc::add(0, 0)\n"),
        "{out}"
    );
    assert!(
        out.contains("a review that `calc::div` behaves the same"),
        "{out}"
    );
}

/// `import` reads the file beside the model, and `std` from the tool.
#[test]
fn import_reads_beside_the_model() {
    let dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/reworked/imports");
    let (code, out, err) = mitra(&dir, &["eval", "model.mitra", "evidence.mitra"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(
        out.contains("plane::origin_is_neutral: unverified"),
        "{out}"
    );
    let (code, out, _) = mitra(&dir, &["explain", "std::claims"]);
    assert_eq!(code, Some(0));
    assert!(out.contains("claim commutative(f, T)"), "{out}");
}

/// `mitra plan` with `represented`: one point per set.
#[test]
fn plan_owes_one_point_per_represented_set() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/reworked/represented");
    let (code, out, err) = mitra(&dir, &["plan", "model.mitra", "evidence.mitra"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(
        out.contains("calc::div::spec owed:\n  any one point of { (a, b) | b == 0 }\n"),
        "{out}"
    );
    assert!(
        out.contains("a review that one input of 0..=9 stands for the others"),
        "{out}"
    );
}

/// A method of an `impl` of the model is tied to the method of the
/// code: `self` in any form, `Self` the type of the `impl`.
#[test]
fn tie_compares_a_method_with_its_code() {
    let from = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/tie/methods");
    let dir = std::env::temp_dir().join(format!("mitra-cli-methods-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    for f in [".mitra/model.mitra", "src/lib.rs"] {
        std::fs::copy(from.join(f), dir.join(f)).unwrap();
    }
    let (code, out, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.starts_with("4 agree, 1 differ"), "{out}");
    let why = "bank::Account::close::tied: the result is `Account` in the code, `Int` in the model";
    assert!(out.contains(why), "{out}");
}

/// A tied test that states no input, on a claim with inputs, only
/// `passes`: it settles nothing, and `ingest` says so. A comment that
/// pins what the test checks is written as it stands.
#[test]
fn ingest_pins_what_a_test_checks() {
    let dir = store("pins", FLIP);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let tests = "// mitra: evidence for s::flip_correct\n#[test]\nfn t_sweep() {}\n\
                // mitra: evidence for s::flip_correct sampled 9\n#[test]\nfn t_drawn() {}\n\
                // mitra: evidence for s::flip sampled 4\n#[test]\nfn t_drawn_bad() {}\n\
                // mitra: evidence for s::flip given a = true gives false\n\
                // mitra: evidence for s::flip_correct given a = true\n#[test]\nfn t_pinned() {}\n\
                // mitra: evidence for s::flip given a = false gives true\n#[test]\nfn t_broken() {}\n";
    std::fs::write(dir.join("tests/t.rs"), tests).unwrap();
    let report = "test t_sweep ... ok\ntest t_pinned ... ok\ntest t_broken ... FAILED\n\
                  test t_drawn ... ok\ntest t_drawn_bad ... FAILED\n";
    std::fs::write(dir.join("report.txt"), report).unwrap();
    let (code, out, err) = mitra(&dir, &["ingest", "report.txt"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(
        out.contains("warning: 1 tied test(s) state no input"),
        "{out}"
    );
    assert!(
        out.contains("tests/t.rs:1: t_sweep on `s::flip_correct`"),
        "{out}"
    );
    let evidence = std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap();
    for want in [
        "for s::flip_correct passes source \"test:t_sweep\"",
        "for s::flip_correct sampled 9 source \"test:t_drawn\"",
        "for s::flip sampled 4 fails source \"test:t_drawn_bad\"",
        "for s::flip given a = true gives false source \"test:t_pinned\"",
        "for s::flip given a = true holds source \"test:t_pinned\"",
        "for s::flip_correct given a = true holds source \"test:t_pinned\"",
        "for s::flip given a = false fails source \"test:t_broken\"",
    ] {
        assert!(evidence.contains(want), "{evidence}");
    }
    let (_, out, _) = mitra(&dir, &["eval"]);
    let want = "s::flip_correct: fails\n  fails on        `s::flip(false)`\n  \
                fails on        one of 4 drawn from its inputs, not stated (test:t_drawn_bad)\n  \
                passes on       `s::flip(true)`\n  \
                sampled on      9 drawn from its inputs (test:t_drawn), trust 0.01 at tolerance 0.001\n  \
                passes on       inputs not stated (test:t_sweep)\n";
    assert!(out.contains(want), "{out}");
    let _ = std::fs::remove_dir_all(dir);
}

/// A function that states no `uses` calls nothing outside its subject:
/// the reader says so when the text settles it, and only then.
#[test]
fn tie_reports_a_call_the_model_does_not_state() {
    let model = "subject a { fn f(x: Int) -> Int; fn h(x: Int) -> Int; }\n\
                 subject b { record R { n: Int } impl R { fn size(&self) -> Int; }\n\
                 fn g(x: Int) -> Int; fn k(x: Int) -> Int; }";
    let dir = store("implied", model);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    let code = "// mitra: implements a::f\npub fn f(x: i64) -> i64 { g(x) + h(x) }\n\
                // mitra: implements a::h\npub fn h(x: i64) -> i64 {\n    let k = |y: i64| y;\n    \
                k(x) + x.to_string().size() as i64\n}\n\
                // mitra: implements b::g\npub fn g(x: i64) -> i64 { x }\n\
                // mitra: implements b::k\npub fn k(x: i64) -> i64 { x }\n\
                // mitra: implements b::R\npub struct R { pub n: i64 }\n\
                impl R {\n    // mitra: implements b::R::size\n    pub fn size(&self) -> i64 { self.n }\n}\n";
    std::fs::write(dir.join("src/lib.rs"), code).unwrap();
    let (code, out, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(0), "{err}");
    let why = "a::f::uses: `f` calls `g`, which is `b::g`, and the model states no `uses`";
    assert!(out.contains(why), "{out}");
    // A `let` of the body and a method through a dot settle nothing.
    assert!(!out.contains("a::h::uses"), "{out}");
    let (_, out, _) = mitra(&dir, &["eval"]);
    assert!(out.contains("a::f::uses: fails"), "{out}");
    let _ = std::fs::remove_dir_all(dir);
}

/// The ledger over time: a run of the tests replaces the last one, what a
/// person said stays until `mitra remove`, and both say what left.
#[test]
fn the_ledger_is_kept_not_only_grown() {
    let dir = store("kept", FLIP);
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let tests = "// mitra: evidence for flip_on_true\n#[test]\nfn t_true() {}\n\
                // mitra: evidence for flip given a = false gives true\n#[test]\nfn t_false() {}\n";
    std::fs::write(dir.join("tests/t.rs"), tests).unwrap();
    std::fs::write(
        dir.join("r1.txt"),
        "test t_true ... ok\ntest t_false ... ok\n",
    )
    .unwrap();
    let (_, out, err) = mitra(&dir, &["ingest", "r1.txt", "--rev", "r1"]);
    assert!(
        out.starts_with("2 results: 2 passed, 0 failed; 2 evidence written\n"),
        "{out}{err}"
    );
    let (_, _, err) = mitra(
        &dir,
        &[
            "add",
            "evidence r { for flip_correct holds }",
            "--source",
            "review:ana",
        ],
    );
    assert_eq!(err, "");
    // The second run no longer has `t_false`.
    std::fs::write(dir.join("r2.txt"), "test t_true ... ok\n").unwrap();
    let (_, out, _) = mitra(&dir, &["ingest", "r2.txt", "--rev", "r2"]);
    assert!(
        out.starts_with(
            "1 results: 1 passed, 0 failed; 1 evidence written, in place of 2 test results of rev r1\n\
             no longer observed, removed (1):\n  evidence test { for flip given a = false gives true source \"test:t_false\" rev \"r1\" }\n"
        ),
        "{out}"
    );
    let ledger = std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap();
    assert!(
        !ledger.contains("\"r1\"") && ledger.contains("review:ana"),
        "{ledger}"
    );
    // The claim is reworded: the review is stale, shown, and judges nothing.
    let model = FLIP.replace(
        "claim flip_correct(a: bool) { correct(flip(a)) }",
        "claim flip_correct(a: bool) { flip(flip(a)) == a }",
    );
    std::fs::write(dir.join(".mitra/model.mitra"), model).unwrap();
    let (_, out, _) = mitra(&dir, &["eval"]);
    assert!(out.contains("s::flip_correct: unverified"), "{out}");
    assert!(
        out.contains("warning[M108]: evidence said before its piece changed"),
        "{out}"
    );
    let (code, out, _) = mitra(&dir, &["remove", "--stale"]);
    assert_eq!(code, Some(0));
    assert!(out.starts_with("removed 1 evidence from "), "{out}");
    assert!(
        out.contains("\n  evidence r { for flip_correct holds source \"review:ana\" seen \""),
        "{out}"
    );
    let (_, out, _) = mitra(&dir, &["eval"]);
    assert!(!out.contains("M108"), "{out}");
    let (code, _, _) = mitra(&dir, &["remove"]);
    assert_eq!(code, Some(2));
    let _ = std::fs::remove_dir_all(dir);
}

const CALC: &str = "subject calc {
    ensure adds = \"Adds two numbers.\";
    fn add(a: Int, b: Int) -> Int;
    claim small() { add(1, 2) == 3 }
}
";

/// The four levels, each running the ones before it and stopping at the
/// first that fails.
#[test]
fn the_four_levels() {
    let dir = store("levels", CALC);
    // build: the model alone. Nothing on what is owed.
    let (code, out, err) = mitra(&dir, &["build"]);
    assert_eq!((code, err.as_str()), (Some(0), ""), "{out}");
    assert!(out.starts_with("build: ok, model "), "{out}");
    assert!(
        out.contains("functions 1, claims 1, promises 1, requires 0, blocks 1, warnings 0\n"),
        "{out}"
    );
    assert!(!out.contains("M101") && !out.contains(" owed:"), "{out}");
    let lock = std::fs::read_to_string(dir.join(".mitra/build/model.lock")).unwrap();
    assert!(lock.starts_with("mitra build 1\nmodel "), "{lock}");
    assert!(lock.contains("\nimplied calc::add::correct "), "{lock}");
    let interface = std::fs::read_to_string(dir.join(".mitra/build/interface.mitra")).unwrap();
    assert!(
        interface.ends_with("// mitra: implements calc::add\nfn add(a: Int, b: Int) -> Int;\n"),
        "{interface}"
    );
    let owed = std::fs::read_to_string(dir.join(".mitra/build/owed.txt")).unwrap();
    assert!(owed.contains("calc::small owed:"), "{owed}");
    // conform: no code yet, so the model is not tied: drift, and assess
    // stops there.
    let (code, out, _) = mitra(&dir, &["conform"]);
    assert_eq!(code, Some(1), "{out}");
    assert!(out.contains("no code is tied to: calc::add\nconform: drift: 0 ties agree, 0 differ, 1 pieces untied"), "{out}");
    let (code, out, _) = mitra(&dir, &["assess"]);
    assert_eq!(code, Some(1));
    assert!(
        out.contains("conform: drift") && !out.contains("assess:"),
        "{out}"
    );
    // With the code and a tied test.
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    let code =
        "// mitra: implements calc::add\npub fn add(a: i64, b: i64) -> i64 {\n    a + b\n}\n";
    std::fs::write(dir.join("src/lib.rs"), code).unwrap();
    let test = "// mitra: evidence for calc::small\n#[test]\nfn small() {}\n";
    std::fs::write(dir.join("tests/t.rs"), test).unwrap();
    let (code, out, _) = mitra(&dir, &["conform"]);
    assert_eq!(code, Some(0), "{out}");
    assert!(
        out.contains(
            "conform: ok: 1 ties agree, 0 differ, 0 pieces untied; 1 tests tied, 0 pin an input\n"
        ),
        "{out}"
    );
    assert!(
        out.contains(
            "  5 lines of model over 3 lines of code, in 1 blocks with 1 promises and 0 requires\n"
        ),
        "{out}"
    );
    // assess: the report, then who settles what is left.
    std::fs::write(dir.join("r.txt"), "test small ... ok\n").unwrap();
    let (code, out, _) = mitra(&dir, &["assess", "r.txt", "--rev", "r1"]);
    assert_eq!(code, Some(0), "{out}");
    assert!(
        out.contains("1 results: 1 passed, 0 failed; 1 evidence written\n"),
        "{out}"
    );
    assert!(out.contains("calc::small: holds\n"), "{out}");
    let tail = "  for a person to review or decide (2):\n    \
                calc::add::deterministic: a reviewer's word that `calc::add` gives one output per input\n    \
                calc::adds::captured: a reviewer's word that the claims say what the text says\n  \
                to hand over (tests to write, code to tie) (1):\n    calc::add::correct:";
    assert!(out.contains(tail), "{out}");
    // diagnose: an observation that agrees, then one that does not.
    let ledger = std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap();
    let agrees = "evidence seen { for calc::add given a = 5, b = 5 gives 10 }";
    let (code, out, _) = mitra(&dir, &["diagnose", agrees]);
    assert_eq!(code, Some(0), "{out}");
    assert!(out.contains("assess: 0 errors"), "{out}");
    assert!(
        out.contains("diagnose: the observation contradicts nothing"),
        "{out}"
    );
    let bug = "evidence bug { for calc::add given a = 1, b = 2 gives 4 }";
    let (code, out, _) = mitra(&dir, &["diagnose", bug]);
    assert_eq!(code, Some(1), "{out}");
    assert!(
        out.contains("diagnose: the observation contradicts what the model and its ledger say\n"),
        "{out}"
    );
    assert!(
        out.contains(" -> fails\n") && out.contains("error[M"),
        "{out}"
    );
    assert!(
        out.contains(&format!("  to file it: mitra add '{bug}' --source <who>\n")),
        "{out}"
    );
    // Neither observation was written: only the ties, read again.
    let after = std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap();
    assert!(
        !after.contains("gives") && ledger.lines().count() == after.lines().count(),
        "{after}"
    );
    let (code, _, err) = mitra(&dir, &["diagnose", "evidence bug { for calc::nope holds }"]);
    assert_eq!(code, Some(2));
    assert!(
        err.contains("the observation: line 1: the model has no function or claim `calc::nope`"),
        "{err}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

/// A model that contradicts itself does not build, and nothing is written.
#[test]
fn a_model_that_contradicts_itself_does_not_build() {
    let model =
        "subject s { fn double(n: 0..=2) -> 0..=4 { n + n } claim odd() { double(1) == 3 } }\n";
    let dir = store("nobuild", model);
    let (code, out, _) = mitra(&dir, &["build"]);
    assert_eq!(code, Some(1), "{out}");
    assert!(
        out.contains("error[M007]")
            && out.ends_with("build: 1 errors in the model, nothing written\n"),
        "{out}"
    );
    assert!(!dir.join(".mitra/build").exists());
    let (code, out, _) = mitra(&dir, &["conform"]);
    assert_eq!(code, Some(1));
    assert!(!out.contains("conform:"), "{out}");
    let _ = std::fs::remove_dir_all(dir);
}

/// What the code adds to hold or hand over a value is no drift: it is
/// said with the tie. A different result is one.
#[test]
fn tie_takes_boilerplate_as_no_drift() {
    let dir = std::env::temp_dir().join(format!("mitra-tie-held-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    let model = "subject s {\n\
                 record Path;\n\
                 fn a(p: Path, name: String) -> Path;\n\
                 fn e(xs: List<Int>, label: String) -> String;\n\
                 fn g(x: Int) -> Int;\n}\n";
    let code = "// mitra: implements s::Path\npub struct Path;\n\
                // mitra: implements s::a\n\
                pub fn a<'a>(p: &'a Path, name: Cow<'a, str>) -> PathBuf { todo!() }\n\
                // mitra: implements s::e\n\
                pub fn e(xs: impl Iterator<Item = i64>, label: impl Into<String>) -> &'static str { \"\" }\n\
                // mitra: implements s::g\n\
                pub fn g(x: i64) -> Option<i64> { None }\n";
    std::fs::write(dir.join(".mitra/model.mitra"), model).unwrap();
    std::fs::write(dir.join("src/lib.rs"), code).unwrap();
    let (code, out, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.starts_with("3 agree, 1 differ"), "{out}");
    assert!(
        out.contains("s::g::tied: the result is `Option<i64>` in the code"),
        "{out}"
    );
    let ledger = std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap();
    assert!(
        ledger.contains("for s::a::tied holds trust 0.98")
            && ledger
                .contains("erased 4: reference(p), reference(name), owned(name), owned(result)"),
        "{ledger}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The settings of a store: read from `.mitra/settings.toml`, written
/// in force by `build`, and what the trust of a tie and the weight of a
/// sample come from.
#[test]
fn settings_set_the_trust_of_a_tie_and_of_a_sample() {
    let dir = std::env::temp_dir().join(format!("mitra-settings-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    let model = "subject s {\n\
                 fn abs(x: -5..=5) -> 0..=5;\n\
                 claim never_negative(x) { abs(x) >= 0 }\n}\n";
    let code = "// mitra: implements s::abs\npub fn abs(x: &i64) -> i64 { x.abs() }\n";
    std::fs::write(dir.join(".mitra/model.mitra"), model).unwrap();
    std::fs::write(dir.join("src/lib.rs"), code).unwrap();
    let ledger = dir.join(".mitra/evidence.mitra");
    let sample = "evidence t { for s::never_negative sampled 1000 source \"review:me\" }\n";
    std::fs::write(&ledger, sample).unwrap();
    // The defaults.
    let (_, out, _) = mitra(&dir, &["eval"]);
    let want = "s::never_negative: holds assuming [s::never_negative::sample]\n  \
                sampled on      1000 drawn from its inputs (review:me), trust 0.63 at tolerance 0.001\n";
    assert!(out.contains(want), "{out}");
    assert_eq!(mitra(&dir, &["tie"]).0, Some(0));
    let read = || std::fs::read_to_string(&ledger).unwrap();
    let tie = "for s::abs::tied holds trust 0.884 source \"reader:rust src/lib.rs:1; \
               erased 4: reference(x), range(x: -5..=5 for &i64), range(result: 0..=5 for i64), width\"";
    assert!(read().contains(tie), "{}", read());
    assert_eq!(mitra(&dir, &["build"]).0, Some(0));
    let written = std::fs::read_to_string(dir.join(".mitra/build/settings.toml")).unwrap();
    assert!(
        written.starts_with(
            "[sample]\ntolerance = 0.001\ncounts = true\n\n[erasure]\nreference = 0\n"
        ),
        "{written}"
    );
    // A file: another tolerance, a sample that does not count, a cost.
    let settings = dir.join(".mitra/settings.toml");
    let text = "[sample]\ntolerance = 0.01\ncounts = false\n\n[erasure]\nrange = 0\nwidth = 0.5\n";
    std::fs::write(&settings, text).unwrap();
    let (_, out, _) = mitra(&dir, &["eval"]);
    let want = "s::never_negative: unverified\n  \
                sampled on      1000 drawn from its inputs (review:me), trust 1 at tolerance 0.01\n";
    assert!(out.contains(want), "{out}");
    assert_eq!(mitra(&dir, &["tie"]).0, Some(0));
    assert!(
        read().contains("for s::abs::tied holds trust 0.5 source"),
        "{}",
        read()
    );
    assert_eq!(mitra(&dir, &["build"]).0, Some(0));
    let written = std::fs::read_to_string(dir.join(".mitra/build/settings.toml")).unwrap();
    assert!(
        written.contains("width = 0.5\n") && written.contains("field = 0.05\n"),
        "{written}"
    );
    // What is not a setting is refused, and so is an erasure the reader
    // does not have.
    std::fs::write(&settings, "[sample]\ntolerence = 0.01\n").unwrap();
    let (code, _, err) = mitra(&dir, &["eval"]);
    assert_eq!(code, Some(2));
    assert!(
        err.contains("settings.toml: line 2: no setting `tolerence` in `[sample]`"),
        "{err}"
    );
    std::fs::write(&settings, "[erasure]\nwidht = 0.1\n").unwrap();
    let (code, _, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(2));
    assert!(
        err.contains("no erasure `widht` in the profile of this reader"),
        "{err}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
