//! The printer is the parser's inverse: for every golden file,
//! parse → print → parse gives the same AST (line numbers aside).

use mitra_core::*;
use std::{fs, path::Path};

pub fn model_without_lines(mut m: Model) -> Model {
    m.records.iter_mut().for_each(|x| x.line = 0);
    m.enums.iter_mut().for_each(|x| x.line = 0);
    // The names of sets are for reports: printing may list fewer.
    m.aliases.clear();
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
        let inner = |m: &str| mitra_core::surface::model(m).unwrap();
        if path.file_name().unwrap() == "model.mitra" {
            let src = inner(&src).0;
            let parsed = parse_model(&src).unwrap();
            let printed = print_model(&parsed);
            let m = model_without_lines(parsed);
            let again = parse_model(&printed)
                .unwrap_or_else(|e| panic!("{}: {e:?}\n{printed}", path.display()));
            assert_eq!(
                model_without_lines(again),
                m,
                "{}\n{printed}",
                path.display()
            );
        } else {
            // Evidence is read with its model: a name alone is a variant.
            let text = fs::read_to_string(path.with_file_name("model.mitra")).unwrap();
            let (text, names) = inner(&text);
            let model = parse_model(&text).unwrap();
            let src = mitra_core::surface::evidence_alone(&src, &names).unwrap();
            let e = evidence_without_lines(parse_evidence_for(&src, &model).unwrap());
            let printed = print_evidence(&e, None);
            let again = parse_evidence_for(&printed, &model)
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
