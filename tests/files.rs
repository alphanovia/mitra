//! A model in several files: `use Part::name from "path";` (the user's
//! design, 2026-11-24 11:11-11:19). A part a file uses and does not
//! declare is read from the project's files that declare it, all those
//! of one folder (a folder is a name space); `from` keeps the files and
//! folders whose path ends with it. Mitra's standard model is the folder
//! `std`, shipped in the binary. None: P010; two folders: P014.

use mitra_core::check::check;
use mitra_core::model::Model;
use mitra_core::parse::{load, parse, Loaded, Project};
use mitra_core::print::print;

const OVER: &str = "subject Calc {
    use Option from \"std\";
    fn over(x: rational, y: rational) -> z: Option<rational> {
        if y == 0.0 { None } else { Some(x / y) }
    }
}";

/// `load` over a project of these files, by their paths from its root.
fn project(model: &str, files: &[(&str, &str)]) -> Result<Model, Loaded> {
    let files = files
        .iter()
        .map(|(n, t)| (n.to_string(), t.to_string()))
        .collect();
    load(model, &Project { files, model: None }, &Default::default())
}

fn errors(model: &Model) -> Vec<String> {
    check(model).iter().map(|e| e.to_string()).collect()
}

#[test]
fn std_is_read_from_the_binary() {
    let model = parse(OVER).unwrap();
    assert_eq!(check(&model), vec![]);
    let option = model.parts.iter().find(|p| p.name == "Option").unwrap();
    assert_eq!(option.file.as_deref(), Some("std/option.mitra"));
    assert_eq!(option.enums[0].name, "Option");
    // `from` may be left out: one folder declares `Option`.
    let bare = parse(&OVER.replace("use Option from \"std\";", "use Option;")).unwrap();
    assert_eq!(check(&bare), vec![]);
    // Without the use, no part declares `Option`: M003 says where it is.
    let none = parse(&OVER.replace("use Option from \"std\";", "")).unwrap();
    let out = errors(&none);
    assert_eq!(out[0], "error[M003]: the type of `z` names `Option`, which no part of the model declares: it is in Mitra's standard model, `use Option from \"std\";`\n --> Calc::over::z");
    // A use reads it for the whole model: a part is seen everywhere.
    let other = format!(
        "{}\nsubject Other {{ use Result from \"std\"; use Option from \"std\"; }}",
        OVER.replace("use Option from \"std\";", "")
    );
    assert_eq!(check(&parse(&other).unwrap()), vec![]);
}

#[test]
fn std_has_only_its_parts() {
    let e = project(
        &OVER.replace("use Option from \"std\";", "use Opshun from \"std\";"),
        &[],
    )
    .unwrap_err();
    assert_eq!(e.error.code.to_string(), "P010");
    assert!(
        e.error.message.contains("no part `Opshun` from \"std\": no file or folder of the project whose path ends with \"std\" declares it"),
        "{}",
        e.error.message
    );
    assert_eq!((e.error.at.line, e.error.at.column), (2, 9));
    // The retired form is no form: `std` is a part's name like any.
    let e = project(
        &OVER.replace("use Option from \"std\";", "use std::core::Option;"),
        &[],
    )
    .unwrap_err();
    assert!(
        e.error.message.contains("no part `std`"),
        "{}",
        e.error.message
    );
}

#[test]
fn a_part_is_read_from_the_file_that_declares_it() {
    let model = "subject Calc {
    use Geometry::first;
    fn mid(p: Point) -> m: rational { (p.x + p.y) / 2.0 }
}";
    let geometry = "subject record Point { x: rational, y: rational }
subject Geometry {
    use Option from \"std\";
    fn first(p: Point) -> o: Option<rational> { Some(p.x) }
}";
    let m = project(
        model,
        &[
            ("shapes/geometry.mitra", geometry),
            ("notes/other.mitra", "subject Other { }"),
        ],
    )
    .unwrap();
    assert_eq!(check(&m), vec![]);
    let files: Vec<(&str, Option<&str>)> = m
        .parts
        .iter()
        .map(|p| (p.name.as_str(), p.file.as_deref()))
        .collect();
    assert_eq!(
        files,
        [
            ("Calc", None),
            ("Point", Some("shapes/geometry.mitra")),
            ("Geometry", Some("shapes/geometry.mitra")),
            ("Option", Some("std/option.mitra")),
        ]
    );
    // No file: P010 where the use is.
    let e = project(model, &[]).unwrap_err();
    assert_eq!(
        (e.file.as_str(), e.error.code.to_string()),
        ("", "P010".to_string())
    );
    assert_eq!(
        e.error.message,
        "no part `Geometry`: no file of the project declares it"
    );
    assert_eq!((e.error.at.line, e.error.at.column), (2, 9));
    // An error inside a file is given with that file.
    let e = project(
        model,
        &[(
            "shapes/geometry.mitra",
            "subject record Point { x: }\nsubject Geometry { }",
        )],
    )
    .unwrap_err();
    assert_eq!(
        (e.file.as_str(), e.error.code.to_string()),
        ("shapes/geometry.mitra", "P003".to_string())
    );
    assert!(e
        .error
        .render(&e.file, &e.source)
        .contains("shapes/geometry.mitra:1:"));
}

#[test]
fn a_folder_is_a_name_space() {
    let model = "subject Agent {
    use Store::get;
    fn f(k: integer) -> v: integer { get(k) }
}";
    let memory = "subject Store { fn get(k: integer) -> v: integer { k } }";
    let cache = "subject Store { fn get(k: integer) -> v: integer { 0 } }";
    let files = [("memory/store.mitra", memory), ("cache/store.mitra", cache)];
    // Two folders: P014, naming both, and the `from` to write.
    let e = project(model, &files).unwrap_err();
    assert_eq!(e.error.code.to_string(), "P014");
    assert_eq!(
        e.error.message,
        "`Store` is declared in 2 folders: memory/store.mitra, cache/store.mitra; name the one you mean: `use Store from \"memory/store\";`"
    );
    // `from` tells them apart, by the end of a file's or folder's path.
    for from in ["memory", "memory/store"] {
        let text = model.replace(
            "use Store::get;",
            &format!("use Store::get from \"{from}\";"),
        );
        let m = project(&text, &files).unwrap_or_else(|e| panic!("{from}: {e:?}"));
        let store = m.parts.iter().find(|p| p.name == "Store").unwrap();
        assert_eq!(store.file.as_deref(), Some("memory/store.mitra"), "{from}");
    }
    // "store" ends both paths: still two.
    let text = model.replace("use Store::get;", "use Store::get from \"store\";");
    assert_eq!(
        project(&text, &files).unwrap_err().error.code.to_string(),
        "P014"
    );
    // By whole names: "ore" ends no name.
    let text = model.replace("use Store::get;", "use Store::get from \"ore\";");
    assert_eq!(
        project(&text, &files).unwrap_err().error.code.to_string(),
        "P010"
    );
    // A part reopened in two files of one folder is read whole.
    let more = "subject Store { fn put(k: integer) -> v: integer { k } }";
    let text = model.replace("use Store::get;", "use Store::{get, put};");
    let m = project(
        &text,
        &[("memory/store.mitra", memory), ("memory/more.mitra", more)],
    )
    .unwrap();
    assert_eq!(check(&m), vec![]);
    let store = m.parts.iter().find(|p| p.name == "Store").unwrap();
    assert_eq!(store.functions.len(), 2);
}

#[test]
fn with_no_project_a_part_not_found_is_left_to_check() {
    // `parse` reads std only: a part used and declared nowhere is M016's.
    let m = parse("subject A { use Geometry::f; }").unwrap();
    assert_eq!(
        errors(&m),
        ["error[M016]: `use Geometry` names no other part\n --> A"]
    );
}

#[test]
fn only_a_file_that_declares_a_used_part_is_read() {
    // A draft that does not parse and a store sit in the project; they
    // declare nothing the model uses, so they are not read.
    let model =
        "subject Calc { use Geometry::first; fn f(x: rational) -> y: rational { first(x) } }";
    let geometry = "subject Geometry { fn first(x: rational) -> y: rational { x } }";
    let draft = "subject Draft { use Geometry::first; fn g( }";
    let store = "review 1 { by: \"human:ana\" }";
    let m = project(
        model,
        &[
            ("drafts/draft.mitra", draft),
            ("evidence.mitra", store),
            ("shapes/geometry.mitra", geometry),
        ],
    )
    .unwrap();
    assert_eq!(check(&m), vec![]);
    assert_eq!(m.parts.len(), 2);
    // An environment, and a data part, are found by their words too.
    let model = "subject Calc { use Clock::now; use Tick; fn f(t: Tick) -> y: integer { now() } }";
    let clock =
        "environment Clock { fn now() -> t: integer; }\nenvironment record Tick { n: integer }";
    let m = project(
        model,
        &[("drafts/draft.mitra", draft), ("io/clock.mitra", clock)],
    )
    .unwrap();
    assert_eq!(check(&m), vec![]);
    let model =
        "subject Calc { use Geometry::first; fn f(x: rational) -> y: rational { first(x) } }";
    // The draft is read, and its error given, once it declares the part.
    let draft = "subject Geometry { fn g( }";
    let e = project(
        model,
        &[
            ("drafts/draft.mitra", draft),
            ("shapes/geometry.mitra", geometry),
        ],
    )
    .unwrap_err();
    assert_eq!(e.error.code.to_string(), "P014");
}

#[test]
fn files_that_use_each_other_are_read_once() {
    let a = "subject A { use B::g; fn f(x: integer) -> y: integer { g(x) } }";
    let b = "subject B { use A::f; fn g(x: integer) -> y: integer { x } fn h(x: integer) -> y: integer { f(x) } }";
    // `B` uses `A`, which the model declares: read once.
    let m = project(a, &[("b.mitra", b), ("a.mitra", a)]).unwrap();
    assert_eq!(m.parts.len(), 2);
    let files = vec![
        ("a.mitra".to_string(), a.to_string()),
        ("b.mitra".to_string(), b.to_string()),
    ];
    let m = load(
        a,
        &Project {
            files,
            model: Some("a.mitra".to_string()),
        },
        &Default::default(),
    )
    .unwrap();
    assert_eq!(m.parts.len(), 2);
}

#[test]
fn a_part_from_another_file_is_printed_there() {
    let printed = print(&parse(OVER).unwrap());
    assert!(printed.contains("use Option from \"std\";"), "{printed}");
    assert!(!printed.contains("enum Option"), "{printed}");
    assert_eq!(parse(&printed).unwrap(), parse(OVER).unwrap());
}

/// `std` is not reserved: a part of the model so named is read by
/// `use std::clock;`, Mitra's standard model by `from "std"`.
#[test]
fn a_part_named_std_stays_the_model_s() {
    let text = "environment std { fn clock() -> t: integer; }
subject S {
    use std::clock;
    use Option from \"std\";
    fn f(x: integer) -> y: Option<integer> { Some(clock() + x) }
}";
    let m = parse(text).unwrap();
    assert_eq!(check(&m), vec![]);
    let names: Vec<&str> = m.parts.iter().map(|p| p.name.as_str()).collect();
    assert_eq!(names, ["std", "S", "Option"]);
    assert_eq!(parse(&print(&m)).unwrap(), m);
}
