//! The two sides never read each other, and only the seam reads both.

use std::fs;

fn sources(of: &str) -> String {
    let mut text = fs::read_to_string(format!("src/{of}.rs")).unwrap();
    if let Ok(folder) = fs::read_dir(format!("src/{of}")) {
        for file in folder {
            text.push_str(&fs::read_to_string(file.unwrap().path()).unwrap());
        }
    }
    text
}

fn code(text: &str) -> String {
    let lines = text.lines().filter(|l| !l.trim_start().starts_with("//"));
    lines.collect::<Vec<_>>().join("\n")
}

#[test]
fn a_piece_knows_no_value_and_a_value_no_piece() {
    let (model, value) = (code(&sources("model")), code(&sources("value")));
    for word in ["value::", "claim::", "Expression", "eval("] {
        assert!(!model.contains(word), "src/model names `{word}`");
    }
    for word in ["piece", "model::", "claim::", "Opinion", "Taking"] {
        assert!(!value.contains(word), "src/value names `{word}`");
    }
}

/// Nothing is built in: the core of values never reads a quantity and
/// knows no standard operation. The standard table reads values only.
#[test]
fn the_core_of_values_reads_no_quantity() {
    // No kind of quantity is named in the core of values, and no
    // reading of them.
    let value = code(&sources("value"));
    for word in ["Exact", "Bounds", "standard", "bounds", "Truth(", "Whole("] {
        assert!(!value.contains(word), "src/value names `{word}`");
    }
    let standard = code(&sources("standard"));
    let bounds = code(&sources("bounds"));
    for word in ["model::", "claim::", "Opinion", "Taking"] {
        assert!(!standard.contains(word), "src/standard names `{word}`");
        assert!(!bounds.contains(word), "src/bounds names `{word}`");
    }
    assert!(!standard.contains("bounds"), "src/standard reads bounds");
}

#[test]
fn the_core_reads_no_disk_and_runs_nothing() {
    for of in ["model", "value", "standard", "bounds", "claim"] {
        let text = code(&sources(of));
        for word in ["std::fs", "std::net", "std::process", "std::env"] {
            assert!(!text.contains(word), "src/{of} names `{word}`");
        }
    }
}

#[test]
fn each_model_file_has_its_code() {
    for of in ["model", "value", "standard", "bounds", "claim"] {
        assert!(fs::metadata(format!("model/{of}.mitra")).is_ok());
        assert!(fs::metadata(format!("src/{of}.rs")).is_ok());
    }
}
