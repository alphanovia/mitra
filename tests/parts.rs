//! Parts as types (your design, 2026-11-24 09:18-10:08, "go" 10:21): a
//! part's own record, methods taking `self`, `impl C;` in a part, a part
//! reopened, parts nested in a part, and the data shorthand.

use mitra_core::agree::{agree, Agreement, Settings};
use mitra_core::check::check;
use mitra_core::parse::parse;
use mitra_core::print::print;

pub const ASSISTANT: &str = include_str!("../models/assistant.mitra");

fn errors(text: &str) -> Vec<String> {
    check(&parse(text).unwrap_or_else(|e| panic!("{e}")))
        .iter()
        .map(|e| e.to_string())
        .collect()
}

fn refused(text: &str) -> String {
    parse(text)
        .map(|_| "parsed".to_string())
        .unwrap_or_else(|e| e.to_string())
}

#[test]
fn the_assistant_is_well_formed_and_reads_back() {
    let model = parse(ASSISTANT).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![]);
    let printed = print(&model);
    let again = parse(&printed).unwrap_or_else(|e| panic!("{e}\n{printed}"));
    assert_eq!(model, again, "{printed}");
    assert_eq!(print(&again), printed);
    // Its example of the whole holds.
    let (agreed, _) = agree(&model, &Settings::default());
    assert!(
        agreed
            .iter()
            .all(|a| !matches!(a.agreement, Agreement::Fails { .. })),
        "{agreed:?}"
    );
}

#[test]
fn a_method_is_called_on_its_value() {
    let text = "subject Counter {
    record { n: integer }
    fn up(self, step: integer) -> c: Counter { Counter { n: self.n + step } }
    fn twice(c: Counter) -> d: Counter { c.up(1).up(1) } where {
        claim two: d.n == c.n + 2;
    }
}";
    let model = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![]);
    let printed = print(&model);
    assert!(printed.contains("c.up(1).up(1)"), "{printed}");
    assert!(printed.contains("fn up(self, step: integer)"), "{printed}");
    assert_eq!(parse(&printed).unwrap(), model);
    let (agreed, _) = agree(&model, &Settings::default());
    assert!(
        !agreed
            .iter()
            .any(|a| matches!(a.agreement, Agreement::Fails { .. })),
        "{agreed:?}"
    );
    // No such method: P013, which says what a method is.
    let out = refused(&text.replace("c.up(1).up(1)", "c.down(1)"));
    assert!(
        out.contains("error[P013]: `.down(…)` calls a method, and `Counter` has no method `down`"),
        "{out}"
    );
    // A field read with parentheses is no method either.
    let out = refused(&text.replace("c.up(1).up(1)", "Counter { n: c.n() }"));
    assert!(out.contains("error[P013]"), "{out}");
}

#[test]
fn a_method_is_seen_where_its_part_is() {
    // A method of another part needs no `use`; a static function does.
    let text = "subject Tool {
    record { name: [~char] }
    fn call(self, input: [~char]) -> out: [~char] { self.name ++ input }
    fn named(n: [~char]) -> t: Tool { Tool { name: n } }
}
subject Harness {
    fn run(t: Tool) -> o: [~char] { t.call(\"go\") }
    fn make(n: [~char]) -> t: Tool { Tool::named(n) }
}";
    let out = errors(text);
    assert_eq!(
        out,
        ["error[M018]: `Tool::named` is of another part: `use Tool::named;`\n --> Harness::make"]
    );
}

#[test]
fn a_data_part_is_written_short() {
    let text = "environment record Reply { text: [~char], stop: bool }
subject enum Step { Answer([~char]), Stop }
subject Harness {
    fn stopped(r: Reply) -> s: Reply { Reply { stop: true, ..r } } where {
        claim same: s.text == r.text && s.stop;
    }
    fn first(s: Step) -> b: bool { match s { Answer(_) => true, Stop => false } }
}";
    let model = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![]);
    let printed = print(&model);
    assert!(
        printed.starts_with("environment record Reply { text: [~char], stop: bool }"),
        "{printed}"
    );
    assert!(
        printed.contains("subject enum Step { Answer([~char]), Stop }"),
        "{printed}"
    );
    // The update form: each other field read from `r`.
    assert!(
        printed.contains("Reply { text: r.text, stop: true }"),
        "{printed}"
    );
    assert_eq!(parse(&printed).unwrap(), model);
}

#[test]
fn a_part_is_reopened_of_its_kind() {
    let text = "subject Tool { record { name: [~char] } }
subject Tool { fn shout(self) -> s: [~char] { self.name ++ \"!\" } }";
    let model = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(model.parts.len(), 1);
    assert_eq!(model.parts[0].functions.len(), 1);
    assert_eq!(check(&model), vec![]);
    let out = refused(&text.replacen("subject Tool { fn", "environment Tool { fn", 1));
    assert!(
        out.contains("error[P012]: `Tool` is a subject, reopened here as an environment: a part keeps its kind"),
        "{out}"
    );
    // Reopened by its path in the part holding it.
    let text = "subject A { subject Tool { record { name: [~char] } } }
subject A::Tool { fn get(self) -> s: [~char] { self.name } }";
    let model = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![]);
    assert_eq!(
        model
            .parts
            .iter()
            .find(|p| p.name == "Tool")
            .unwrap()
            .functions
            .len(),
        1
    );
    // A part keeps one record of its own.
    let out = refused("subject T { record { a: bool } record { b: bool } }");
    assert!(
        out.contains("`T` holds one record of its own, given twice"),
        "{out}"
    );
}

#[test]
fn a_nested_part_is_named_by_its_path_or_its_name() {
    let text = "subject A {
    ensure whole: \"whole\" from (B::b);
    subject B {
        ensure b: \"b\";
        fn f(x: integer) -> y: integer { x }
    }
}
subject C {
    use A::B::f;
    fn g(x: integer) -> y: integer { f(x) }
}";
    let model = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![]);
    // The parts are written inside the part holding them.
    let printed = print(&model);
    assert!(
        printed.contains("subject A {\n    ensure whole"),
        "{printed}"
    );
    assert!(printed.contains("\n    subject B {\n"), "{printed}");
    assert_eq!(parse(&printed).unwrap(), model);
    let out = refused(&text.replace("use A::B::f;", "use C::B::f;"));
    assert!(
        out.contains("error[P003]: `B` is no part inside `C`"),
        "{out}"
    );
}

#[test]
fn a_part_fulfils_a_contract_with_its_methods() {
    let text = "contract Named {
    fn name(self) -> n: [~char];
}
subject Tool {
    record { label: [~char] }
    fn name(self) -> n: [~char] { self.label }
    impl Named;
}";
    let model = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![]);
    let printed = print(&model);
    assert!(printed.contains("    impl Named;\n"), "{printed}");
    assert!(
        printed.contains("contract Named {\n    fn name(self)"),
        "{printed}"
    );
    assert_eq!(parse(&printed).unwrap(), model);
    // A method that does not fit: M020.
    let out = errors(&text.replace(
        "fn name(self) -> n: [~char] { self.label }",
        "fn name(self) -> n: bool { true }",
    ));
    assert!(out.iter().any(|e| e.starts_with("error[M020]")), "{out:?}");
}

#[test]
fn a_part_holding_parts_rests_on_their_ensures() {
    let text = "subject A {
    require given: \"given\";
    ensure whole: \"whole\" from (B::b, given);
    subject B {
        require inner: \"inner\";
        ensure b: \"b\";
    }
}";
    assert_eq!(errors(text), Vec::<String>::new());
    // The full path of an inner part is its name.
    assert_eq!(
        errors(&text.replace("from (B::b,", "from (A::B::b,")),
        Vec::<String>::new()
    );
    let m029 = "holds parts, and its ensures rest only on its parts' ensures and its own requires";
    // Not on its own ensures, nor on its parts' requires.
    for wrong in ["from (A::whole, given)", "from (B::inner, given)"] {
        let out = errors(&text.replace("from (B::b, given)", wrong));
        assert!(out.iter().any(|e| e.contains(m029)), "{wrong}: {out:?}");
    }
    // An end-to-end example belongs to the part it is written in.
    let model = parse(
        "subject A {\n    subject B { fn f(x: integer) -> y: integer { x } }\n    examples { (B::f::x = 1) -> B::f::y = 1 }\n}",
    )
    .unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(model.examples[0].within.as_deref(), Some("A"));
}

#[test]
fn self_comes_first_in_a_part_with_a_record() {
    let out = errors(
        "subject T {\n    record { n: integer }\n    fn g(x: integer, self) -> y: integer { x }\n}",
    );
    assert_eq!(
        out,
        ["error[M030]: `self` is taken by `g` in place 1: a method takes `self` first, and nothing else is so named\n --> T::g"]
    );
}

#[test]
#[ignore]
fn show() {
    println!("{}", print(&parse(ASSISTANT).unwrap()));
}
