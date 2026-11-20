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
    let (piece, value) = (code(&sources("piece")), code(&sources("value")));
    for word in ["value::", "claim::", "Expression", "eval"] {
        assert!(!piece.contains(word), "src/piece names `{word}`");
    }
    for word in ["piece", "claim::", "Opinion", "Evidence"] {
        assert!(!value.contains(word), "src/value names `{word}`");
    }
}

#[test]
fn the_core_reads_no_disk_and_runs_nothing() {
    for of in ["piece", "value", "claim"] {
        let text = code(&sources(of));
        for word in ["std::fs", "std::net", "std::process", "std::env"] {
            assert!(!text.contains(word), "src/{of} names `{word}`");
        }
    }
}

#[test]
fn each_model_file_has_its_code() {
    for of in ["piece", "value", "claim"] {
        assert!(fs::metadata(format!("model/{of}.mitra")).is_ok());
        assert!(fs::metadata(format!("src/{of}.rs")).is_ok());
    }
}
