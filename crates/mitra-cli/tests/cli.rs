use std::process::Command;

const GOLDEN: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/golden/not/05_correct_on_true_wrong_on_false"
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
    assert!(status.success());
    let expected = std::fs::read_to_string(format!("{GOLDEN}/expected.txt")).unwrap();
    assert_eq!(std::fs::read_to_string(&out).unwrap(), expected);
    let _ = std::fs::remove_file(out);
}

#[test]
fn bad_evidence_names_the_file_and_line() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-bad-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(
        dir.join("ev.mitra"),
        "\nevidence not(true, false) correct @ 1\n",
    )
    .unwrap();
    let out = Command::new(env!("CARGO_BIN_EXE_mitra"))
        .args(["eval", &format!("{GOLDEN}/model.mitra")])
        .arg(dir.join("ev.mitra"))
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let err = String::from_utf8(out.stderr).unwrap();
    assert!(
        err.contains("ev.mitra: line 2: `not` takes 1 input(s), got 2"),
        "{err}"
    );
    let _ = std::fs::remove_dir_all(dir);
}
