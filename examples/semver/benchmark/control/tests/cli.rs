use std::process::Command;

fn run(args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_semver")).args(args).output().unwrap();
    (
        out.status.code(),
        String::from_utf8(out.stdout).unwrap(),
        String::from_utf8(out.stderr).unwrap(),
    )
}

#[test]
fn bump_minor_example() {
    assert_eq!(run(&["bump", "minor", "1.4.2"]), (Some(0), "1.5.0\n".into(), String::new()));
}

#[test]
fn bump_each_level() {
    assert_eq!(run(&["bump", "major", "1.4.2"]).1, "2.0.0\n");
    assert_eq!(run(&["bump", "patch", "1.4.2"]).1, "1.4.3\n");
    assert_eq!(run(&["bump", "patch", "1.4.2-alpha.1"]).1, "1.4.3\n");
}

#[test]
fn bad_input_prints_nothing_and_fails() {
    let cases: &[&[&str]] = &[
        &[],
        &["bump"],
        &["bump", "minor"],
        &["bump", "huge", "1.2.3"],
        &["bump", "minor", "1.2"],
        &["bump", "minor", "a.b.c"],
        &["bump", "minor", "01.2.3"],
        &["bump", "minor", "1.2.3-"],
        &["bump", "major", "18446744073709551615.0.0"],
        &["bump", "minor", "1.2.3", "extra"],
    ];
    for args in cases {
        let (code, out, err) = run(args);
        assert_ne!(code, Some(0), "{args:?}");
        assert!(code.is_some(), "{args:?}");
        assert_eq!(out, "", "{args:?}");
        assert!(err.starts_with("error:"), "{args:?}: {err}");
    }
}
