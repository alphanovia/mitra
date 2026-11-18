//! The printer is the parser's inverse: for every golden file,
//! parse → print → parse gives the same AST (line numbers aside).

use mitra_core::*;
use std::{fs, path::Path};

pub fn model_without_lines(mut m: Model) -> Model {
    m.records.iter_mut().for_each(|x| x.line = 0);
    m.functions.iter_mut().for_each(|x| x.line = 0);
    m.claims.iter_mut().for_each(|x| x.line = 0);
    m.tests.iter_mut().for_each(|x| {
        x.line = 0;
        x.judgment.line = 0;
    });
    m
}

fn evidence_without_lines(mut e: Vec<Evidence>) -> Vec<Evidence> {
    e.iter_mut().for_each(|x| x.line = 0);
    e
}

fn files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            files(&path, out);
        } else if path.extension().is_some_and(|e| e == "mitra") {
            out.push(path);
        }
    }
}

#[test]
fn golden_files_round_trip() {
    let mut paths = Vec::new();
    files(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden"),
        &mut paths,
    );
    assert!(paths.len() > 20);
    for path in paths {
        let src = fs::read_to_string(&path).unwrap();
        if path.file_name().unwrap() == "model.mitra" {
            let m = model_without_lines(parse_model(&src).unwrap());
            let printed = print_model(&m);
            let again = parse_model(&printed)
                .unwrap_or_else(|e| panic!("{}: {e:?}\n{printed}", path.display()));
            assert_eq!(
                model_without_lines(again),
                m,
                "{}\n{printed}",
                path.display()
            );
        } else {
            let e = evidence_without_lines(parse_evidence(&src).unwrap());
            let printed = print_evidence(&e, None);
            let again = parse_evidence(&printed)
                .unwrap_or_else(|e| panic!("{}: {e:?}\n{printed}", path.display()));
            assert_eq!(
                evidence_without_lines(again),
                e,
                "{}\n{printed}",
                path.display()
            );
        }
    }
}
