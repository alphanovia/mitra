//! The binary end to end: what it prints and how it exits.
use std::process::Command;

fn semver(args: &[&str]) -> (String, Option<i32>) {
    let out = Command::new(env!("CARGO_BIN_EXE_semver")).args(args).output().unwrap();
    (String::from_utf8(out.stdout).unwrap(), out.status.code())
}

// mitra: measures Cli::run (["bump", "minor", "1.4.2"]) -> Ok("1.5.0")
#[test]
fn prints_bumped_version() {
    assert_eq!(semver(&["bump", "minor", "1.4.2"]), ("1.5.0\n".to_string(), Some(0)));
}

// mitra: measures Cli::run (["bump", "minor"]) -> Err(Failure::Usage)
#[test]
fn usage_prints_nothing_and_fails() {
    let (out, code) = semver(&["bump", "minor"]);
    assert_eq!(out, "");
    assert_ne!(code, Some(0));
}

// mitra: measures Cli::run (["bump", "minor", "1.4"]) -> Err(Failure::BadVersion)
#[test]
fn bad_version_prints_nothing_and_fails() {
    let (out, code) = semver(&["bump", "minor", "1.4"]);
    assert_eq!(out, "");
    assert_ne!(code, Some(0));
}

#[test]
fn overflow_prints_nothing_and_fails() {
    let (out, code) = semver(&["bump", "major", "18446744073709551615.0.0"]);
    assert_eq!(out, "");
    assert_ne!(code, Some(0));
}
