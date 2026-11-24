//! The command line (`crates/mitra-cli/src/main.rs`), on the example project
//! `examples/div`: main's div, its tie and tests in Rust, a `cargo test`
//! report, and a store with one measurement taken wrong and one review.

use std::process::Command;

fn mitra(args: &[&str]) -> (i32, String) {
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let out = Command::new(env!("CARGO_BIN_EXE_mitra"))
        .current_dir(root)
        .args(args)
        .output()
        .unwrap();
    let text = String::from_utf8(out.stdout).unwrap() + &String::from_utf8(out.stderr).unwrap();
    (out.status.code().unwrap_or(-1), text)
}

const EVIDENCE: [&str; 6] = [
    "--code",
    "examples/div/src",
    "--report",
    "examples/div/report.txt",
    "--store",
    "examples/div/evidence.mitra",
];

#[test]
fn check_says_the_model_agrees() {
    let (code, out) = mitra(&["check", "examples/div/model.mitra"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("check: well formed; agree: 9 comparisons, 9 hold, 0 findings"),
        "{out}"
    );
}

#[test]
fn explain_gives_a_verdict_on_each_proposition() {
    let mut args = vec!["explain", "examples/div/model.mitra"];
    args.extend(EVIDENCE);
    let (code, out) = mitra(&args);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("evidence: 4 measurements, 1 reviews, 1 ties"),
        "{out}"
    );
    assert!(out.contains("explained    correct(calc::div, (6, 2)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::div_by_two)"), "{out}");
    assert!(out.contains("contradicted calc::div::spec "), "{out}");
    assert!(out.contains("explain: 20 propositions, 10 explained, 6 contradicted, 4 unsupported; 0 findings, 0 hints, 1 reviews asked"), "{out}");
}

#[test]
fn blame_doubts_the_measurement_taken_wrong_first() {
    let mut args = vec!["blame", "examples/div/model.mitra"];
    args.extend(EVIDENCE);
    let (code, out) = mitra(&args);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains(
            "diagnosis 1 (doubt 0.33): drop measurements::3 (human:ana, saw calc::div::q = 4)"
        ),
        "{out}"
    );
    assert!(out.contains("blame: 3 conflicts, 3 diagnoses"), "{out}");
}

#[test]
fn a_model_not_well_formed_stops_with_its_errors() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let bad = dir.join("bad.mitra");
    std::fs::write(
        &bad,
        "subject s { fn f(a: integer) -> b: integer { true } }",
    )
    .unwrap();
    let (code, out) = mitra(&["check", bad.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("error[M005]"), "{out}");
    std::fs::write(&bad, "subject s { fn f(a: integer) -> b: integer }").unwrap();
    let (code, out) = mitra(&["check", bad.to_str().unwrap()]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("error[P003]"), "{out}");
    let (code, out) = mitra(&["judge", "x"]);
    assert_eq!(code, 2, "{out}");
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn points_decide_main_s_div_on_every_pair() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-points-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let div = dir.join("div.mitra");
    std::fs::write(
        &div,
        "subject calc {
    fn div(a: 0..=9, b: 0..=9) -> q: 0..=9 { if b == 0 { 0 } else { a / b } } where {
        claim spec: if b == 0 { q == 0 } else { q * b <= a && a < (q + 1) * b };
    }
}",
    )
    .unwrap();
    let d = div.to_str().unwrap();
    // 10 x 10 = 100 pairs: more than the 64 points agree tries by default.
    let (code, out) = mitra(&["check", d]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("finding[A007]: claim `spec`: unknown"),
        "{out}"
    );
    let (code, out) = mitra(&["check", d, "--points", "100"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("3 comparisons, 3 hold, 0 findings"), "{out}");
    let (code, out) = mitra(&["check", d, "--points", "99"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("finding[A007]"), "{out}");
    for (args, says) in [
        (
            vec!["check", d, "--points", "0"],
            "takes a whole number above 0",
        ),
        (
            vec!["check", d, "--points", "x"],
            "takes a whole number above 0",
        ),
        (vec!["check", d, "--points"], "needs a value"),
        (
            vec!["check", d, "--points", "9", "--points", "9"],
            "given twice",
        ),
        (vec!["explain", d, "--points", "9"], "is for `check`"),
    ] {
        let (code, out) = mitra(&args);
        assert_eq!(code, 2, "{args:?}: {out}");
        assert!(out.contains(says), "{args:?}: {out}");
    }
    std::fs::remove_dir_all(&dir).unwrap();
}

/// `mitra` run in `dir`.
fn mitra_in(dir: &std::path::Path, args: &[&str]) -> (i32, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_mitra"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    let text = String::from_utf8(out.stdout).unwrap() + &String::from_utf8(out.stderr).unwrap();
    (out.status.code().unwrap_or(-1), text)
}

/// The project is the folder of `mitra.toml`, up from where `mitra` runs:
/// a part is read from any file of it, here in another folder.
#[test]
fn a_part_is_read_from_the_project() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-files-{}", std::process::id()));
    let src = dir.join("src");
    let shapes = dir.join("shapes");
    std::fs::create_dir_all(&src).unwrap();
    std::fs::create_dir_all(&shapes).unwrap();
    std::fs::create_dir_all(dir.join("target")).unwrap();
    std::fs::write(dir.join("mitra.toml"), "").unwrap();
    std::fs::write(dir.join(".gitignore"), "target/\n").unwrap();
    std::fs::write(
        src.join("calc.mitra"),
        "subject Calc {\n    use Point from \"shapes\";\n    use Option from \"std\";\n    fn mid(p: Point) -> m: Option<rational> { Some((p.x + p.y) / 2.0) }\n}\n",
    )
    .unwrap();
    let (code, out) = mitra_in(&src, &["check", "calc.mitra"]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("error[P010]: no part `Point` from \"shapes\""),
        "{out}"
    );
    assert!(out.contains("calc.mitra:2:9"), "{out}");
    std::fs::write(
        shapes.join("point.mitra"),
        "subject record Point { x: rational, y: }\n",
    )
    .unwrap();
    let (code, out) = mitra_in(&src, &["check", "calc.mitra"]);
    assert_eq!(code, 1, "{out}");
    assert!(out.contains("shapes/point.mitra:1:"), "{out}");
    std::fs::write(
        shapes.join("point.mitra"),
        "subject record Point { x: rational, y: rational }\n",
    )
    .unwrap();
    let (code, out) = mitra_in(&src, &["check", "calc.mitra"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("check: well formed"), "{out}");
    // The root is found up from the model, wherever `mitra` runs.
    let calc = src.join("calc.mitra");
    let (code, out) = mitra_in(&std::env::temp_dir(), &["check", calc.to_str().unwrap()]);
    assert_eq!(code, 0, "{out}");
    // A folder `.gitignore` leaves out is no part of the project: with
    // no `from`, `Point` is still of one folder.
    std::fs::write(
        dir.join("target/point.mitra"),
        "subject record Point { z: rational }\n",
    )
    .unwrap();
    let calc = std::fs::read_to_string(src.join("calc.mitra")).unwrap();
    let calc = calc.replace("use Point from \"shapes\";", "use Point;");
    std::fs::write(src.join("calc.mitra"), calc).unwrap();
    let (code, out) = mitra_in(&src, &["check", "calc.mitra"]);
    assert_eq!(code, 0, "{out}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// `mitra.toml` at the root gives each layer its settings: here agree's
/// points, which `--points` still overrides; what it cannot take stops
/// with P015, in the file, exit 1.
#[test]
fn mitra_toml_sets_the_settings() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-toml-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("div.mitra"),
        "subject calc {
    fn div(a: 0..=9, b: 0..=9) -> q: 0..=9 { if b == 0 { 0 } else { a / b } } where {
        claim spec: if b == 0 { q == 0 } else { q * b <= a && a < (q + 1) * b };
    }
}",
    )
    .unwrap();
    std::fs::write(dir.join("mitra.toml"), "[agree]\npoints = 100\n").unwrap();
    let (code, out) = mitra_in(&dir, &["check", "div.mitra"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("3 comparisons, 3 hold, 0 findings"), "{out}");
    let (code, out) = mitra_in(&dir, &["check", "div.mitra", "--points", "99"]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("finding[A007]"), "{out}");
    // What a statement is expected to be with no evidence: explain's numbers.
    let (_, out) = mitra_in(&dir, &["explain", "div.mitra"]);
    assert!(out.contains("unsupported  calc::div::spec 0.50 "), "{out}");
    std::fs::write(dir.join("mitra.toml"), "[numbers]\nbase_rate = 0.25\n").unwrap();
    let (_, out) = mitra_in(&dir, &["explain", "div.mitra"]);
    assert!(out.contains("unsupported  calc::div::spec 0.25 "), "{out}");
    std::fs::write(dir.join("mitra.toml"), "[agree]\npionts = 100\n").unwrap();
    let (code, out) = mitra_in(&dir, &["check", "div.mitra"]);
    assert_eq!(code, 1, "{out}");
    assert!(
        out.contains("error[P015]: no setting `pionts` in `[agree]`"),
        "{out}"
    );
    assert!(out.contains("mitra.toml:2:1"), "{out}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// The dogfood model, beside the code it models, from the root's
/// `mitra.toml`: every module's file is read through `use … from`.
#[test]
fn the_dogfood_model_is_whole_and_checks() {
    let (code, out) = mitra(&["check", "crates/mitra-cli/src/main.mitra"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("check: well formed; agree: 24 comparisons, 16 hold, 8 findings"),
        "{out}"
    ); // Its ties (`src/simulation.rs/…`) are read from the root, not from
       // `crates/mitra-cli/src/`: each once, none twice by two spellings.
    let (code, out) = mitra(&[
        "explain",
        "crates/mitra-cli/src/main.mitra",
        "--code",
        "crates",
        "--store",
        "core2.store.mitra",
        "--no-test",
    ]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.lines().any(|l| l.starts_with("evidence: 0 measurements,") && l.ends_with(" reviews, 48 ties")),
        "{out}"
    );
}

/// With no `--report`, explain runs `cargo test` at the project's root,
/// says so, and reads what it printed: `examples/div`'s three tests.
/// `--no-test` runs none; with `--report`, it is a mistake.
#[test]
fn explain_runs_cargo_test_when_no_report_is_given() {
    let (code, out) = mitra(&[
        "explain",
        "examples/div/model.mitra",
        "--code",
        "examples/div/src",
        "--store",
        "examples/div/evidence.mitra",
    ]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("ran `cargo test --no-fail-fast -- --show-output` in ")
            && out.contains("examples/div: 3 tests, 3 passed, 0 failed, 0 ignored"),
        "{out}"
    );
    assert!(
        out.contains("evidence: 4 measurements, 1 reviews, 1 ties"),
        "{out}"
    );
    let (_, out) = mitra(&[
        "explain",
        "examples/div/model.mitra",
        "--code",
        "examples/div/src",
        "--no-test",
    ]);
    assert!(
        out.starts_with("`--no-test`: the tests in the code that measure ran in no report"),
        "{out}"
    );
    let (code, out) = mitra(&[
        "explain",
        "examples/div/model.mitra",
        "--no-test",
        "--report",
        "examples/div/report.txt",
    ]);
    assert_eq!(code, 2, "{out}");
    assert!(out.contains("`--no-test` and `--report` together"), "{out}");
    // A report whose binary stopped says so.
    let report = std::env::temp_dir().join(format!("mitra-stopped-{}.txt", std::process::id()));
    std::fs::write(&report, "     Running unittests src/lib.rs (target/debug/deps/div-1)\n\nrunning 3 tests\ntest tests::div_by_two ... ok\n").unwrap();
    let (_, out) = mitra(&[
        "explain",
        "examples/div/model.mitra",
        "--code",
        "examples/div/src",
        "--report",
        report.to_str().unwrap(),
    ]);
    assert!(
        out.contains("finding[R009]: the test binary `div/src/lib.rs` stopped before its result"),
        "{out}"
    );
    std::fs::remove_file(&report).unwrap();
}

/// `examples/heater`: tests that print rows with `mitra::check_eq!`,
/// read through the tie, and reviews of a link and of words a test
/// shows. Its test failing, the review that sees it is dropped (R008)
/// and asked for again; the row still measures what the system gave.
#[test]
fn rows_and_reviews_seeing_a_test() {
    let (code, out) = mitra(&[
        "explain",
        "examples/heater/model.mitra",
        "--code",
        "examples/heater/src",
        "--store",
        "examples/heater/evidence.mitra",
    ]);
    assert_eq!(code, 0, "{out}");
    assert!(out.contains("examples/heater: 3 tests, 3 passed"), "{out}");
    assert!(!out.contains("finding[R"), "{out}");
    assert!(
        out.contains("evidence: 3 measurements, 3 reviews, 2 ties"),
        "{out}"
    );
    assert!(out.contains("explained    correct(Heater::decide, (Heater { target: 20, on: false }, 18)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::heats_below_the_target)"), "{out}");
    assert!(out.ends_with(", 0 reviews asked\n"), "{out}");
    // A copy whose test of `steady` fails.
    let root = concat!(env!("CARGO_MANIFEST_DIR"), "/../..");
    let dir = std::env::temp_dir().join(format!("mitra-heater-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("src")).unwrap();
    for f in ["model.mitra", "evidence.mitra", "mitra.toml", "src/lib.rs"] {
        std::fs::copy(format!("{root}/examples/heater/{f}"), dir.join(f)).unwrap();
    }
    let cargo = std::fs::read_to_string(format!("{root}/examples/heater/Cargo.toml"))
        .unwrap()
        .replace("../../crates/mitra", &format!("{root}/crates/mitra"));
    std::fs::write(dir.join("Cargo.toml"), cargo).unwrap();
    let lib = std::fs::read_to_string(dir.join("src/lib.rs"))
        .unwrap()
        .replace("18), once);", "18), Heater { target: 21, ..once });");
    std::fs::write(dir.join("src/lib.rs"), lib).unwrap();
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_mitra"))
        .current_dir(&dir)
        .env("CARGO_TARGET_DIR", format!("{root}/target/heater-copy"))
        .args([
            "explain",
            "model.mitra",
            "--code",
            "src",
            "--store",
            "evidence.mitra",
        ])
        .output()
        .unwrap();
    let out = String::from_utf8(out.stdout).unwrap();
    assert!(
        out.contains(": 3 tests, 2 passed, 1 failed, 0 ignored"),
        "{out}"
    );
    assert!(out.contains("finding[R008]: the test `tests::decides_the_same_twice` failed: the review of `Heater::steady` by `human:ana` counts only while it passes"), "{out}");
    assert!(
        out.contains("evidence: 3 measurements, 2 reviews, 2 ties"),
        "{out}"
    );
    assert!(
        out.contains("ask[X012]: only words say `Heater::steady`"),
        "{out}"
    );
    std::fs::remove_dir_all(&dir).unwrap();
}
