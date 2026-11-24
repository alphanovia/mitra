//! The crates beside Mitra's own, `crates/mitra` (the test macros) and
//! `crates/mitra-derive` (`derive(Observe)`): their tests run with ours.

#[test]
fn the_mitra_test_crate_passes_its_tests() {
    let root = env!("CARGO_MANIFEST_DIR");
    let out = std::process::Command::new("cargo")
        .args(["test", "--quiet"])
        .current_dir(format!("{root}/crates/mitra"))
        .output()
        .unwrap();
    let text =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    assert!(out.status.success(), "{text}");
    assert!(text.contains("4 passed"), "{text}");
}
