//! Every file of code, tests and model can be read whole: none is
//! longer than the ceiling (docs/design/layout.md).

use std::path::{Path, PathBuf};

/// Lines. At 40 characters a line, about 11,000 tokens.
const CEILING: usize = 1000;

/// The files not yet cut. The list only shrinks.
const NOT_YET: [&str; 8] = [
    "mitra-lang/src/eval.rs",
    "mitra-lang/src/ast.rs",
    "mitra-lang/src/report.rs",
    "mitra-lang/src/set.rs",
    "mitra-lang/tests/surface.rs",
    "mitra-rust/src/tie.rs",
    "mitra-store/tests/cold_fable.rs",
    "mitra-cli/tests/cli.rs",
];

fn files(dir: &Path, out: &mut Vec<PathBuf>) {
    for e in std::fs::read_dir(dir).into_iter().flatten().flatten() {
        let p = e.path();
        if p.is_dir() {
            files(&p, out);
        } else if p.extension().is_some_and(|x| x == "rs" || x == "mitra") {
            out.push(p);
        }
    }
}

#[test]
fn every_file_can_be_read_whole() {
    let crates = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let mut all = Vec::new();
    for c in std::fs::read_dir(crates).unwrap().flatten() {
        for part in ["src", "tests", "model", "std"] {
            files(&c.path().join(part), &mut all);
        }
    }
    assert!(all.len() > 50, "the crates were not found");
    let mut long = Vec::new();
    for f in &all {
        let name = f.strip_prefix(crates).unwrap().display().to_string();
        // The ledger is written by machines.
        if name.ends_with("evidence.mitra") {
            continue;
        }
        let lines = std::fs::read_to_string(f).unwrap().lines().count();
        let waived = NOT_YET.contains(&name.as_str());
        if lines > CEILING && !waived {
            long.push(format!("{name}: {lines} lines"));
        }
        assert!(
            !waived || lines > CEILING,
            "{name} is cut: take it off the list"
        );
    }
    assert!(long.is_empty(), "over {CEILING} lines: {long:?}");
}
