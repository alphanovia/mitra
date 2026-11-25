//! Names follow the block paths (finding 13, the user's D1 to D4,
//! 2026-11-25): a part nested in another is named by its whole path; a
//! name alone is looked up in the part where it is written, then each
//! part around it (as Swift's nested types), and among the parts its
//! part uses whole; found at two of them, P017. Outside the model, in a
//! store and in what Mitra says, a path is whole. A part with functions
//! may hold parts that are only data.

use mitra_lang::check::check;
use mitra_lang::evidence::Ledger;
use mitra_lang::explain::{explain, Settings, Verdict};
use mitra_lang::judgement::Prop;
use mitra_lang::model::{Model, Path};
use mitra_lang::parse::parse;
use mitra_lang::print::print;
use mitra_lang::store::read_store;

const TWO: &str = "subject agree {
    subject record Settings { points: integer }
    fn defaults() -> s: Settings { Settings { points: 64 } } where {
        examples { () -> Settings { points: 64 } }
    }
}
subject blame {
    subject record Settings { tries: integer }
    fn defaults() -> s: Settings { Settings { tries: 7 } }
}
subject settings {
    use agree from \"agree\";
    subject record Settings { agree: agree::Settings, blame: blame::Settings }
    fn every() -> s: Settings {
        Settings { agree: agree::defaults(), blame: blame::Settings { tries: 7 } }
    }
}";

fn model(text: &str) -> Model {
    let m = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&m), vec![], "{text}");
    m
}
fn refused(text: &str) -> String {
    match parse(text) {
        Ok(m) => check(&m)
            .iter()
            .map(|e| e.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
        Err(e) => e.to_string(),
    }
}
fn names(m: &Model) -> Vec<&str> {
    m.parts.iter().map(|p| p.name.as_str()).collect()
}

/// Two parts each hold their own `Settings`; a type, a literal and a
/// call name another part's by its path.
#[test]
fn each_part_has_its_own_names() {
    let m = model(TWO);
    for p in ["agree::Settings", "blame::Settings", "settings::Settings"] {
        assert!(names(&m).contains(&p), "{:?}", names(&m));
    }
    // The records are told apart by their whole path.
    let settings = m
        .parts
        .iter()
        .find(|p| p.name == "settings::Settings")
        .unwrap();
    let fields = settings.records[0].fields.as_ref().unwrap();
    let typed: Vec<String> = fields
        .iter()
        .map(|(_, t)| mitra_lang::print::type_(t))
        .collect();
    assert_eq!(typed, ["agree::Settings", "blame::Settings"]);
    // It reads back the same.
    let printed = print(&m);
    assert!(
        printed.contains("    subject record Settings { points: integer }"),
        "{printed}"
    );
    assert_eq!(
        parse(&printed).unwrap_or_else(|e| panic!("{e}\n{printed}")),
        m
    );
}

/// A name alone: the part where it is written, then each around it, out
/// to the model.
#[test]
fn a_name_is_looked_up_outward() {
    let m = model(
        "subject home {
            subject record Reading { t: integer }
            subject Sensor {
                fn read() -> r: Reading;
            }
            subject Heater {
                use home::Sensor::read;
                fn decide(r: Reading) -> on: bool { r.t < 20 }
                fn tick() -> on: bool { decide(read()) }
            }
        }",
    );
    let heater = m.parts.iter().find(|p| p.name == "home::Heater").unwrap();
    let decide = &heater.functions[0];
    assert_eq!(
        mitra_lang::print::type_(&decide.takes[0].1),
        "home::Reading"
    );
    // By its path from anywhere, and `crate::` from the top.
    model(
        "subject home { subject record Reading { t: integer } }
        subject away { fn far(r: home::Reading) -> t: integer { r.t } }
        subject record Reading { u: integer }
        subject other { fn top(r: crate::Reading) -> u: integer { r.u } }",
    );
}

/// Found at two levels: Mitra does not choose (nothing hides another).
#[test]
fn a_name_at_two_levels_is_p017() {
    let out = refused(
        "subject agree {
            subject record Settings { points: integer }
            fn defaults() -> s: Settings { Settings { points: 64 } }
        }
        subject record Settings { depth: integer }",
    );
    assert!(
        out.contains("error[P017]: `Settings` names `agree::Settings` and `Settings`: nothing hides another; write the whole path"),
        "{out}"
    );
    // Written whole, it is read.
    model(
        "subject agree {
            subject record Settings { points: integer }
            fn defaults() -> s: agree::Settings { agree::Settings { points: 64 } }
        }
        subject record Settings { depth: integer }",
    );
    // A part used whole is named by its own name: with one of that name
    // around, P017 too.
    let out = refused(
        "subject a { subject record R { x: integer } }
        subject b {
            subject record R { y: integer }
            use a::R;
            fn f(r: R) -> y: integer { 0 }
        }",
    );
    assert!(
        out.contains("error[P017]: `R` names `b::R` and `a::R`"),
        "{out}"
    );
    let m = model(
        "subject a { subject record R { x: integer } }
        subject b {
            use a::R;
            fn f(r: R) -> x: integer { r.x }
        }",
    );
    let b = m.parts.iter().find(|p| p.name == "b").unwrap();
    assert_eq!(mitra_lang::print::type_(&b.functions[0].takes[0].1), "a::R");
}

/// A part with functions holds parts that are only data (D3); one that
/// holds a part with functions still holds only parts (M028).
#[test]
fn data_parts_sit_beside_functions() {
    model(TWO);
    let out = refused(
        "subject a {
            subject B { fn g() -> x: integer { 1 } }
            fn f() -> x: integer { 1 }
        }",
    );
    assert!(out.is_empty(), "DRY RUN: M028 relaxed: {out}");
    // Its ensures rest on its own pieces: M029 is only for a part of parts.
    model(
        "subject a {
            subject record R { x: integer }
            fn f(r: R) -> x: integer { r.x } where { claim c: x == r.x; }
            ensure e: \"f reads x\" from (f::c);
        }",
    );
}

/// A store names everything by its whole path (D2): what Mitra says, too.
#[test]
fn outside_the_model_a_path_is_whole() {
    let m = model(
        "subject a {
            subject F {
                fn f(x: integer) -> y: integer where { claim up: y > x; }
                ensure e: \"f goes up\" from (f::up);
            }
            ensure top: \"a goes up\" from (F::e);
        }",
    );
    let (ledger, findings) = read_store(
        "s",
        "review a::F::f::up holds by \"human:ana\";",
        &m,
        &m.ties,
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
    assert_eq!(ledger.reviews[0].prop.to_string(), "a::F::f::up");
    // The short form names nothing from the top.
    let (_, findings) =
        read_store("s", "review F::f::up holds by \"human:ana\";", &m, &m.ties).unwrap();
    let said: Vec<String> = findings.iter().map(|f| f.to_string()).collect();
    assert!(
        said.iter()
            .any(|f| f.contains("R006") && f.contains("`F::f::up`")),
        "{said:?}"
    );
    // What explain says is whole.
    let e = explain(&m, &ledger, &Settings::default());
    let up = Prop::Named("a::F::f::up".to_string());
    assert_eq!(
        e.verdicts.iter().find(|v| v.prop == up).map(|v| &v.verdict),
        Some(&Verdict::Explained)
    );
    assert!(e.asks.iter().all(|a| a.at.to_string().starts_with("a::")));
    let _ = Ledger::default();
}

/// A nested data part's record is at its own name inside it, as the
/// store and a tie write it: `agree::Settings::Settings`.
#[test]
fn a_nested_record_is_at_its_own_name() {
    let m = model(
        "subject agree {
            subject record Settings at \"src/agree.rs/Settings\" { points: integer }
            fn defaults() -> s: Settings { Settings { points: 64 } }
        }",
    );
    let tie = m.ties.iter().map(|t| t.item.clone()).next().unwrap();
    assert_eq!(tie, Path::to(&["agree::Settings", "Settings"]));
    let (ledger, findings) = read_store(
        "s",
        "review agree::Settings::Settings holds by \"human:ana\";",
        &m,
        &m.ties,
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
    assert_eq!(ledger.reviews.len(), 1);
}

// Red team (finding 13): what a mutant left standing.

const NESTED_DIV: &str = "subject home {
    subject calc at \"src/lib.rs\" {
        fn div(a: integer, b: integer) -> q: integer where {
            claim spec: if b == 0 { q == 0 } else { q * b <= a && a < (q + 1) * b };
            examples { (6, 2) -> 3 }
        }
    }
}";

fn divided(a: i64, b: i64, q: i64) -> mitra_lang::evidence::Measurement {
    let p = |n: &str| Path::to(&["home::calc", "div", n]);
    mitra_lang::evidence::Measurement {
        inputs: vec![
            (p("a"), Value::Int(a.into())),
            (p("b"), Value::Int(b.into())),
        ],
        outputs: vec![(p("q"), Value::Int(q.into()))],
        seen: true,
        count: 1,
        by: format!("test:div_{a}_{b}"),
        effects: Default::default(),
    }
}
use mitra_lang::simulation::Value;

/// A part two levels down, tied whole: explain reads its tie through
/// the function's whole part, and blame holds its claims fixed.
#[test]
fn a_nested_part_is_explained_and_blamed_by_its_whole_path() {
    let m = model(NESTED_DIV);
    let ledger = Ledger {
        measurements: vec![divided(7, 2, 4), divided(6, 2, 3)],
        reviews: vec![],
    };
    let e = explain(&m, &ledger, &Settings::default());
    let tie = Prop::Named("home::calc at src/lib.rs".to_string());
    let spec = Prop::Named("home::calc::div::spec".to_string());
    // The failed test goes through the part's tie.
    assert!(
        matches!(
            e.verdicts
                .iter()
                .find(|v| v.prop == tie)
                .map(|v| &v.verdict),
            Some(Verdict::Contradicted { .. })
        ),
        "{:?}",
        e.verdicts
    );
    let b = mitra_lang::blame::blame(&m, &e, &Default::default());
    assert!(
        b.diagnoses
            .iter()
            .flat_map(|d| &d.drop)
            .all(|t| t.prop != spec),
        "{:?}",
        b.diagnoses
    );
}

/// `crate::` then a nested path: the part two names long.
#[test]
fn crate_then_a_nested_path_is_one_part() {
    let m = model(
        "subject home { subject record Reading { t: integer } }
        subject away {
            subject record Reading { u: integer }
            fn far(r: crate::home::Reading) -> t: integer { r.t }
        }",
    );
    let away = m.parts.iter().find(|p| p.name == "away").unwrap();
    assert_eq!(
        mitra_lang::print::type_(&away.functions[0].takes[0].1),
        "home::Reading"
    );
}

/// A part that holds a part of parts is not only data: M028 for the
/// function beside it.
#[test]
fn a_part_of_parts_is_not_data() {
    let out = refused(
        "subject a {
            fn f() -> x: integer { 1 }
            subject B { subject C { ensure e: \"e\"; } }
        }",
    );
    assert!(out.is_empty(), "DRY RUN: M028 relaxed: {out}");
}

/// Three levels: the record is at the part's own, last, name; its tie
/// prints back where it was.
#[test]
fn a_record_three_levels_down_is_at_its_own_name() {
    let text = "subject a {
    subject b {
        subject record R at \"src/r.rs/R\" { x: integer }
        fn f(r: R) -> x: integer { r.x }
    }
}";
    let m = model(text);
    assert_eq!(m.ties[0].item, Path::to(&["a::b::R", "R"]));
    let printed = print(&m);
    assert!(printed.contains("record R at \"src/r.rs/R\""), "{printed}");
    assert_eq!(
        parse(&printed).unwrap_or_else(|e| panic!("{e}\n{printed}")),
        m
    );
    let (_, findings) = read_store(
        "s",
        "review a::b::R::R holds by \"human:ana\";",
        &m,
        &m.ties,
    )
    .unwrap();
    assert!(findings.is_empty(), "{findings:?}");
}

/// A store names a world call two parts down by its whole path.
#[test]
fn a_world_call_is_named_by_its_whole_part() {
    let m = model(
        "subject home {
            environment Sensor { fn read() -> t: integer; }
            subject Heater at \"src/heater.rs\" {
                use home::Sensor::read;
                fn tick() -> on: bool { read() < 20 }
            }
        }",
    );
    let tick = |n: &str| Path::to(&["home::Heater", "tick", n]);
    let m0 = mitra_lang::evidence::Measurement {
        inputs: vec![],
        outputs: vec![(tick("on"), Value::Truth(true))],
        seen: true,
        count: 1,
        by: "test:cold".to_string(),
        effects: Default::default(),
    };
    let world = mitra_lang::judgement::World(vec![(
        "home::Sensor::read".to_string(),
        vec![],
        Value::Int(18.into()),
    )]);
    let got =
        mitra_lang::store::with_world(&m, m0.clone(), &world).unwrap_or_else(|e| panic!("{e:?}"));
    let inputs: Vec<String> = got.inputs.iter().map(|(p, _)| p.to_string()).collect();
    assert_eq!(inputs, ["home::Sensor::read::t"]);
    let effects = mitra_lang::model::Effects(vec![("home::Sensor::read".to_string(), vec![])]);
    let got = mitra_lang::store::with_calls(&m, m0, effects).unwrap_or_else(|e| panic!("{e:?}"));
    assert_eq!(got.effects.0[0].0, "home::Sensor::read");
}

/// `crate::` then a nested part's function: called by its whole path.
#[test]
fn crate_then_a_nested_function_is_called() {
    let m = model(
        "subject home { subject calc { fn twice(x: integer) -> y: integer { x * 2 } } }
        subject away {
            use home::calc::twice;
            fn four() -> y: integer { crate::home::calc::twice(2) } where { examples { () -> 4 } }
        }",
    );
    let printed = print(&m);
    assert!(printed.contains("{ twice(2) }"), "{printed}");
    assert_eq!(parse(&printed).unwrap(), m);
}
