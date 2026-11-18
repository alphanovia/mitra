use std::process::Command;

const GOLDEN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/golden/flip/05_correct_on_true_wrong_on_false"
);

#[test]
fn eval_writes_the_result_file() {
    let out = std::env::temp_dir().join(format!("mitra-cli-test-{}.txt", std::process::id()));
    let status = Command::new(env!("CARGO_BIN_EXE_mitra"))
        .args([
            "eval",
            &format!("{GOLDEN}/model.mitra"),
            &format!("{GOLDEN}/evidence.mitra"),
            "-o",
        ])
        .arg(&out)
        .status()
        .unwrap();
    // The case has a failing claim: `eval` exits with 1 on errors.
    assert_eq!(status.code(), Some(1));
    let expected = std::fs::read_to_string(format!("{GOLDEN}/expected.txt")).unwrap();
    // The result names the files as given; the expected one, by their names.
    let result = std::fs::read_to_string(&out).unwrap();
    let dir = GOLDEN.strip_prefix(concat!(env!("CARGO_MANIFEST_DIR"), "/"));
    assert_eq!(result.replace(&format!("{}/", dir.unwrap()), ""), expected);
    let _ = std::fs::remove_file(out);
}

#[test]
fn bad_evidence_names_the_file_and_line() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("ev.mitra"), "\nevidence correct (flip 7) at 1\n").unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mitra"))
        .args(["eval", &format!("{GOLDEN}/model.mitra")])
        .arg(dir.join("ev.mitra"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(
        err.contains("ev.mitra: line 2: inputs outside the domain of `flip`"),
        "{err}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

fn expected() -> String {
    std::fs::read_to_string(format!("{GOLDEN}/expected.txt")).unwrap()
}

/// The verdicts of a result: what comes before the first blank line. The
/// errors name files and lines, which differ from store to store.
fn verdicts(result: &str) -> &str {
    result.split("\n\n").next().unwrap()
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

#[test]
fn store_init_add_check_eval() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-store-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    assert_eq!(mitra(&dir, &["init"]).0, Some(2), "init never overwrites");
    let model = std::fs::read_to_string(format!("{GOLDEN}/model.mitra")).unwrap();
    std::fs::write(dir.join(".mitra/model.mitra"), model).unwrap();
    // From a subfolder: the store is found above.
    let sub = dir.join("sub");
    let (code, out, _) = mitra(
        &sub,
        &[
            "add",
            "evidence correct (flip true) at 1",
            "--source",
            "review:ana",
            "--rev",
            "3f2a",
        ],
    );
    assert_eq!((code, out.starts_with("added 1 evidence")), (Some(0), true));
    // Rejected evidence leaves the store unchanged.
    let before = std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap();
    let (code, _, err) = mitra(&sub, &["add", "evidence correct (flip 7) at 1"]);
    assert_eq!(code, Some(2));
    assert!(err.contains("inputs outside the domain of `flip`"), "{err}");
    assert_eq!(
        std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap(),
        before
    );
    assert!(
        before.ends_with(
            "@[source \"review:ana\", rev \"3f2a\"] evidence correct (flip true) at 1\n"
        ),
        "{before}"
    );
    // An output outside the result type is refused, and blamed on the new text.
    let (code, _, err) = mitra(
        &sub,
        &[
            "add",
            "evidence flip true = false at 1\nevidence flip false = 3 at 1",
        ],
    );
    assert_eq!(code, Some(2));
    assert!(
        err.contains("evidence to add: line 2: output 3 is outside the result type of `flip`"),
        "{err}"
    );
    // --source twice, or in both the option and the text, is refused.
    assert_eq!(
        mitra(
            &sub,
            &[
                "add",
                "evidence flip true = false at 1",
                "--source",
                "a",
                "--source",
                "b"
            ]
        )
        .0,
        Some(2)
    );
    let (_, _, err) = mitra(
        &sub,
        &[
            "add",
            "@[source \"a\"] evidence flip true = false at 1",
            "--source",
            "b",
        ],
    );
    assert!(err.contains("--source is given twice"), "{err}");
    assert_eq!(mitra(&sub, &["eval", "-o", "x.mitra"]).0, Some(2));
    mitra(
        &sub,
        &[
            "add",
            "evidence forall a in {false}, not correct (flip a) at 1",
        ],
    );
    let (code, out, _) = mitra(&sub, &["check"]);
    assert_eq!(
        (code, out.as_str()),
        (Some(0), "ok: 1 functions, 2 claims, 0 tests, 2 evidence\n")
    );
    let (code, out, err) = mitra(&sub, &["eval"]);
    assert_eq!(verdicts(&out), verdicts(&expected()));
    assert_eq!(
        (code, err.as_str()),
        (Some(1), "mitra: 1 errors, 0 warnings\n")
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn ingest_turns_test_results_into_evidence() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-ingest-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    mitra(&dir, &["init"]);
    let model = std::fs::read_to_string(format!("{GOLDEN}/model.mitra")).unwrap();
    let tests = "test t_true : correct (flip true)\n\
                 test t_false : correct (flip false)\n\
                 test t_all : forall a, correct (flip a)\n";
    std::fs::write(dir.join(".mitra/model.mitra"), format!("{model}\n{tests}")).unwrap();
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
        "4 results: 2 passed, 2 failed; 2 evidence added\n\
         test t_all failed: no evidence, since it is about a set of inputs, and Mitra can't tell which one was wrong\n\
         no `test` declared for: t_other\n"
    );
    let evidence = std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap();
    assert!(
        evidence.ends_with(
            "@[source \"test:t_true\", rev \"3f2a\"] evidence correct (flip true) at 1\n\
         @[source \"test:t_false\", rev \"3f2a\"] evidence not correct (flip false) at 1\n"
        ),
        "{evidence}"
    );
    let (_, out, _) = mitra(&dir, &["eval"]);
    assert_eq!(verdicts(&out), verdicts(&expected()));
    assert!(
        out.contains("fails     .mitra/evidence.mitra:3  @[source \"test:t_false\", rev \"3f2a\"]"),
        "{out}"
    );
    let _ = std::fs::remove_dir_all(dir);
}

#[test]
fn a_broken_store_is_blamed_not_the_new_evidence() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-broken-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    mitra(&dir, &["init"]);
    let model = std::fs::read_to_string(format!("{GOLDEN}/model.mitra")).unwrap();
    std::fs::write(dir.join(".mitra/model.mitra"), model).unwrap();
    let ev = dir.join(".mitra/evidence.mitra");
    let broken = std::fs::read_to_string(&ev).unwrap() + "evidence nope 1 = 2 at 1\n";
    std::fs::write(&ev, &broken).unwrap();
    let (code, _, err) = mitra(&dir, &["add", "evidence flip true = false at 1"]);
    assert_eq!(code, Some(2));
    assert!(
        err.contains("evidence.mitra: line 2: `nope` is not a subject"),
        "{err}"
    );
    assert_eq!(std::fs::read_to_string(&ev).unwrap(), broken);
    let _ = std::fs::remove_dir_all(dir);
}

/// Dogfood: mitra-ds's store (its claims, and evidence from its own test
/// runs, recorded by scripts/dogfood-ds.sh) gives the expected verdicts.
#[test]
fn mitra_ds_store_evaluates() {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../mitra-ds");
    let (code, out, err) = mitra(&dir, &["eval"]);
    assert_eq!(code, Some(0), "{err}");
    let expected = std::fs::read_to_string(dir.join(".mitra/expected.txt")).unwrap();
    assert_eq!(out, expected);
}

#[test]
fn rev_leaves_out_stale_evidence() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-rev-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    mitra(&dir, &["init"]);
    let model = std::fs::read_to_string(format!("{GOLDEN}/model.mitra")).unwrap();
    std::fs::write(dir.join(".mitra/model.mitra"), model).unwrap();
    mitra(
        &dir,
        &["add", "evidence correct (flip true) at 1", "--rev", "old"],
    );
    mitra(
        &dir,
        &["add", "evidence correct (flip false) at 1", "--rev", "new"],
    );
    // Evidence with no revision is stale under --rev too: else this old
    // pass would outlive any newer failure.
    mitra(&dir, &["add", "evidence correct (flip true) at 1"]);
    let (_, all, _) = mitra(&dir, &["eval"]);
    assert!(all.starts_with("flip_correct: holds\n"), "{all}");
    let (code, out, err) = mitra(&dir, &["eval", "--rev", "new"]);
    assert_eq!(code, Some(0));
    assert_eq!(
        err,
        "mitra: 2 evidence left out: not recorded at revision new\n\
         mitra: 0 errors, 2 warnings\n"
    );
    assert!(out.starts_with(
        "flip_correct: unverified\n  passes on       `flip false`\n  no evidence on  `flip true`\n\
         flip_on_true: unverified\n  no evidence on  `flip true`\n\
         \n\
         warning[M101]: a claim is not verified\n  \
         --> .mitra/model.mitra:4  claim flip_correct : forall a, correct (flip a)\n\
         \n  \
         No evidence on `flip true`.\n  \
         help: `mitra next` lists the evidence that would settle it\n"
    ));
    let (_, out, _) = mitra(&dir, &["next", "--rev", "new"]);
    assert!(out.starts_with("flip_correct: missing {true}\n"), "{out}");
    assert_eq!(mitra(&dir, &["eval", "--rev"]).0, Some(2));
    assert_eq!(mitra(&dir, &["eval", "--rev", ""]).0, Some(2));
    assert_eq!(mitra(&dir, &["eval", "--rev", "--rev"]).0, Some(2));
    assert_eq!(
        mitra(
            &dir,
            &["add", "evidence correct (flip true) at 1", "--rev", ""]
        )
        .0,
        Some(2)
    );
    let _ = std::fs::remove_dir_all(dir);
}

/// The store's model includes the `mitra:` comments of `src/` and
/// `tests/`; errors there name the Rust file and line, and a declaration
/// above the wrong function is refused.
#[test]
fn store_reads_rust_comments() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-rust-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    std::fs::create_dir_all(dir.join("tests")).unwrap();
    mitra(&dir, &["init"]);
    std::fs::write(
        dir.join("src/lib.rs"),
        "/// Negates.\n\
         /// mitra: opaque flip (a : Bool) : Bool\n\
         /// mitra: claim flip_correct : forall a, correct (flip a)\n\
         pub fn flip(a: bool) -> bool { !a }\n",
    )
    .unwrap();
    let test = "// mitra: test t_true : correct (flip true)\n#[test]\nfn t_true() {}\n";
    std::fs::write(dir.join("tests/t.rs"), test).unwrap();
    let (code, out, err) = mitra(&dir, &["check"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(
        out.starts_with("ok: 1 functions, 1 claims, 1 tests"),
        "{out}"
    );
    let report = dir.join("report.txt");
    std::fs::write(&report, "test t_true ... ok\n").unwrap();
    let (code, out, err) = mitra(&dir, &["ingest", report.to_str().unwrap()]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.contains("1 evidence added"), "{out}");

    std::fs::write(
        dir.join("tests/t.rs"),
        test.replace("fn t_true", "fn t_other"),
    )
    .unwrap();
    let (code, _, err) = mitra(&dir, &["check"]);
    assert_eq!(code, Some(2));
    assert!(
        err.contains("t.rs: line 1: `test t_true` must sit above `fn t_true`, found `fn t_other`"),
        "{err}"
    );
    std::fs::write(
        dir.join("tests/t.rs"),
        "// mitra: test t_true : correct (nope true)\n#[test]\nfn t_true() {}\n",
    )
    .unwrap();
    let (code, _, err) = mitra(&dir, &["check"]);
    assert_eq!(code, Some(2));
    assert!(err.contains("t.rs: line 1: "), "{err}");
    let _ = std::fs::remove_dir_all(dir);
}

/// A failed `f x = v` test on a subject that returns `Bool` gives the
/// other output. Otherwise it refutes the `=` claims that give `v` at `x`,
/// and gives nothing when there is none.
#[test]
fn a_failed_bool_output_test_gives_the_other_output() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-boolout-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    mitra(&dir, &["init"]);
    let model = "opaque flip (a : Bool) : Bool\nopaque inc (x : Int) : Int\n\
                 opaque dec (x : Int) : Int\ndef pred (x : Int) : Int := x - 1\n\
                 claim dec_spec : forall x, dec x = pred x\n\
                 test t_flip : flip true = true\ntest t_inc : inc 1 = 2\n\
                 test t_dec : dec 5 = 4\n";
    std::fs::write(dir.join(".mitra/model.mitra"), model).unwrap();
    std::fs::write(
        dir.join("report.txt"),
        "test t_flip ... FAILED\ntest t_inc ... FAILED\ntest t_dec ... FAILED\n",
    )
    .unwrap();
    let (code, out, err) = mitra(&dir, &["ingest", "report.txt"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.contains("2 evidence added"), "{out}");
    assert!(out.contains("test t_inc failed: no evidence"), "{out}");
    let evidence = std::fs::read_to_string(dir.join(".mitra/evidence.mitra")).unwrap();
    // `dec 5` was not 4, which is what `dec_spec` says it is.
    assert!(
        evidence.ends_with(
            "evidence flip true = false at 1\n\
             @[source \"test:t_dec\"] evidence not dec_spec 5 at 1\n"
        ),
        "{evidence}"
    );
    let (_, out, _) = mitra(&dir, &["eval"]);
    assert!(
        out.contains("dec_spec: fails\n  fails on        `dec 5`\n"),
        "{out}"
    );
    let _ = std::fs::remove_dir_all(dir);
}
