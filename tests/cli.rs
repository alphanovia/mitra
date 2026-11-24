//! The command line (`src/bin/mitra.rs`), on the example project
//! `examples/div`: main's div, its tie and tests in Rust, a `cargo test`
//! report, and a store with one measurement taken wrong and one review.

use std::process::Command;

fn mitra(args: &[&str]) -> (i32, String) {
    let root = env!("CARGO_MANIFEST_DIR");
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
    assert!(out.contains("explain: 20 propositions, 10 explained, 6 contradicted, 4 unsupported; 0 findings, 0 hints"), "{out}");
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
