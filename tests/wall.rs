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
    for word in ["value::", "claim::", "Expression", "eval"] {
        assert!(!model.contains(word), "src/model names `{word}`");
    }
    for word in [
        "piece",
        "model::",
        "claim::",
        "Opinion",
        "Evidence",
        "Statement",
    ] {
        assert!(!value.contains(word), "src/value names `{word}`");
    }
}

/// Inside the model: a place and a piece read nothing, and each kind of
/// piece reads only what its subject requires.
#[test]
fn each_subject_reads_only_what_it_requires() {
    let reads = |of: &str| code(&fs::read_to_string(format!("src/model/{of}.rs")).unwrap());
    let allowed: [(&str, &[&str]); 5] = [
        ("place", &[]),
        ("piece", &[]),
        ("evidence", &["piece"]),
        ("statement", &["piece"]),
        ("tie", &["piece", "place"]),
    ];
    for (of, may) in allowed {
        let text = reads(of);
        assert!(
            !text.contains("crate::"),
            "src/model/{of}.rs reads outside the model"
        );
        for read in text.split("super::").skip(1) {
            let other: String = read.chars().take_while(|c| c.is_alphanumeric()).collect();
            assert!(
                may.contains(&other.as_str()),
                "src/model/{of}.rs reads `{other}`"
            );
        }
    }
}

#[test]
fn the_core_reads_no_disk_and_runs_nothing() {
    for of in ["model", "value", "claim"] {
        let text = code(&sources(of));
        for word in ["std::fs", "std::net", "std::process", "std::env"] {
            assert!(!text.contains(word), "src/{of} names `{word}`");
        }
    }
}

#[test]
fn each_model_file_has_its_code() {
    for of in ["model", "value", "claim"] {
        assert!(fs::metadata(format!("model/{of}.mitra")).is_ok());
        assert!(fs::metadata(format!("src/{of}.rs")).is_ok());
    }
}
