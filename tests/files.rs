//! Red team of a model in several files (finding 3, step 1b): `use
//! std::core::Option;` reads Mitra's own standard model, shipped in the
//! binary; a part a file uses and does not declare is read from
//! `<part>.mitra` beside it; a part no file declares is P010. A part
//! read from another file is printed there, not in the model.

use mitra_core::check::check;
use mitra_core::parse::{load, parse, Loaded};
use mitra_core::print::print;
use std::collections::BTreeMap;

const OVER: &str = "subject calc {
    use std::core::Option;
    fn over(x: Real, y: Real) -> z: Option<Real> {
        if y == 0.0 { None } else { Some(x / y) }
    }
}";

/// `load` over a folder of named texts.
fn folder(model: &str, files: &[(&str, &str)]) -> Result<mitra_core::model::Model, Loaded> {
    let files: BTreeMap<String, String> = files
        .iter()
        .map(|(n, t)| (n.to_string(), t.to_string()))
        .collect();
    load(model, &|name: &str| files.get(name).cloned())
}

#[test]
fn std_core_is_read_from_the_binary() {
    let model = parse(OVER).unwrap();
    assert_eq!(check(&model), vec![]);
    let core = model.parts.iter().find(|p| p.name == "core").unwrap();
    assert_eq!(core.file.as_deref(), Some("std/core.mitra"));
    let enums: Vec<&str> = core.enums.iter().map(|e| e.name.as_str()).collect();
    assert_eq!(enums, ["Option", "Result"]);
    // Without the use, no part declares `Option`: M003 says where it is.
    let bare = OVER.replace("use std::core::Option;", "");
    let errors: Vec<String> = check(&parse(&bare).unwrap())
        .iter()
        .map(|e| e.to_string())
        .collect();
    assert_eq!(errors[0], "error[M003]: the type of `z` names `Option`, which no part of the model declares: it is in Mitra's standard model, `use std::core::Option;`\n --> calc::over::z");
    assert_eq!(
        errors[1],
        "error[M017]: `Some` names no function of the model\n --> calc::over"
    );
    // Read by another part, not this one: M018 says the use to write.
    let other = format!("{bare}\nsubject other {{ use std::core::Result; }}");
    let errors: Vec<String> = check(&parse(&other).unwrap())
        .iter()
        .map(|e| e.to_string())
        .collect();
    assert!(
        errors
            .iter()
            .any(|e| e.contains("`core::Option` is of another part: `use std::core::Option;`")),
        "{errors:?}"
    );
}

#[test]
fn std_has_only_its_parts() {
    let e = parse(&OVER.replace("std::core", "std::kore")).unwrap_err();
    assert_eq!(e.code.to_string(), "P010");
    assert!(e.message.contains("no part `std::kore`"), "{}", e.message);
    assert_eq!((e.at.line, e.at.column), (2, 9));
}

#[test]
fn a_part_is_read_from_its_file() {
    let model = "subject calc {
    use geometry::Point;
    fn mid(p: Point) -> m: Real { (p.x + p.y) / 2.0 }
}";
    let geometry = "subject geometry {
    use std::core::Option;
    record Point { x: Real, y: Real }
    fn first(p: Point) -> o: Option<Real> { Some(p.x) }
}";
    let m = folder(model, &[("geometry.mitra", geometry)]).unwrap();
    assert_eq!(check(&m), vec![]);
    let files: Vec<(String, Option<String>)> = m
        .parts
        .iter()
        .map(|p| (p.name.clone(), p.file.clone()))
        .collect();
    assert_eq!(
        files,
        [
            ("calc".to_string(), None),
            ("geometry".to_string(), Some("geometry.mitra".to_string())),
            ("core".to_string(), Some("std/core.mitra".to_string())),
        ]
    );
    // No file: P010 where the use is.
    let e = folder(model, &[]).unwrap_err();
    assert_eq!(
        (e.file.as_str(), e.error.code.to_string()),
        ("", "P010".to_string())
    );
    assert_eq!((e.error.at.line, e.error.at.column), (2, 9));
    // A file that does not declare its part.
    let e = folder(model, &[("geometry.mitra", "subject shapes { }")]).unwrap_err();
    assert!(
        e.error
            .message
            .contains("`geometry.mitra` declares no part `geometry`"),
        "{}",
        e.error.message
    );
    // An error inside the file is given with that file.
    let e = folder(
        model,
        &[("geometry.mitra", "subject geometry { record Point { x: } }")],
    )
    .unwrap_err();
    assert_eq!(
        (e.file.as_str(), e.error.code.to_string()),
        ("geometry.mitra", "P003".to_string())
    );
    assert!(e
        .error
        .render(&e.file, &e.source)
        .contains("geometry.mitra:1:"));
}

#[test]
fn files_that_use_each_other_are_read_once() {
    let a = "subject a { use b::g; fn f(x: Int) -> y: Int { g(x) } }";
    let b = "subject b { use a::f; fn g(x: Int) -> y: Int { x } fn h(x: Int) -> y: Int { f(x) } }";
    let m = folder(a, &[("b.mitra", b)]).unwrap();
    assert_eq!(m.parts.len(), 2);
}

#[test]
fn a_part_from_another_file_is_printed_there() {
    let printed = print(&parse(OVER).unwrap());
    assert!(printed.contains("use std::core::Option;"), "{printed}");
    assert!(!printed.contains("enum Option"), "{printed}");
    assert_eq!(parse(&printed).unwrap(), parse(OVER).unwrap());
}

/// A part of the model named `std` is read by `use std::x;`, two names;
/// Mitra's standard model by three, `use std::core::Option;`.
#[test]
fn a_part_named_std_stays_the_model_s() {
    let text = "environment std { fn clock() -> t: Int; }
subject s {
    use std::clock;
    use std::core::Option;
    fn f(x: Int) -> y: Option<Int> { Some(clock() + x) }
}";
    let m = parse(text).unwrap();
    assert_eq!(check(&m), vec![]);
    let names: Vec<&str> = m.parts.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["std", "s", "core"]);
    assert_eq!(parse(&print(&m)).unwrap(), m);
}
