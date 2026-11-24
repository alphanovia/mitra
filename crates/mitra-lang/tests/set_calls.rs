//! A set's condition calling the model's functions (finding 12b, dry run
//! of option B): a function with a body is read as its definition; one
//! with no body, or one that reaches such, is M023 with help; a point
//! the condition does not read to its end leaves the all with nothing
//! from its points.

use mitra_lang::check::check;
use mitra_lang::evidence::Ledger;
use mitra_lang::explain::{explain, Settings, Verdict};
use mitra_lang::judgement::Prop;
use mitra_lang::parse::parse;
use mitra_lang::simulation::Value;

const MODEL: &str = "subject s {
    fn f(a: integer) -> b: integer { a + 1 }
    fn g(a: integer) -> b: integer at \"src/lib.rs/g\" { a } where {
        claim c: 0 <= b;
        rule r: g::c |- all p in (a: 0..10) where f(a) < 5: correct(g, p);
    }
}";

fn the_all(m: &mitra_lang::model::Model) -> Prop {
    let f = &m.parts[0].functions[1];
    match &f.rules[0].conclusion {
        mitra_lang::model::Term::Prop(p) => p.clone(),
        _ => panic!("a proposition"),
    }
}

#[test]
fn the_set_is_read_by_the_model_function() {
    let m = parse(MODEL).unwrap();
    assert_eq!(check(&m), vec![]);
    let Prop::All { over, .. } = the_all(&m) else {
        panic!("an all")
    };
    let points: Vec<Vec<Value>> = over.points().unwrap().flatten().collect();
    let want: Vec<Vec<Value>> = (0..4).map(|a| vec![Value::Int(a.into())]).collect();
    assert_eq!(points, want);
    assert_eq!(over.undecided(), None);
}

#[test]
fn four_tests_explain_the_all_three_do_not() {
    let m = parse(MODEL).unwrap();
    let verdict = |n: i64| {
        let store: String = (0..n)
            .map(|a| format!("measure s::g ({a}) -> {a} by \"test:g{a}\";\n"))
            .collect();
        let ledger = mitra_lang::store::read_store("s", &store, &m, &m.ties)
            .unwrap()
            .0;
        let e = explain(&m, &ledger, &Settings::default());
        let q = the_all(&m);
        e.verdicts
            .iter()
            .find(|v| v.prop == q)
            .map(|v| v.verdict.clone())
            .unwrap()
    };
    assert_eq!(verdict(4), Verdict::Explained);
    assert_eq!(verdict(3), Verdict::Unsupported);
}

#[test]
fn a_function_with_no_body_is_refused() {
    let text = MODEL.replace("{ a + 1 }", ";");
    let errors: Vec<String> = check(&parse(&text).unwrap())
        .iter()
        .map(|e| e.to_string())
        .collect();
    assert_eq!(errors, ["error[M023]: the conclusion: the condition of a set calls only functions with a body, read as their definition: `s::f` has no body; write the bound by hand, or give it a body\n --> s::g::r"]);
    // Through a call.
    let text = MODEL.replace(
        "fn f(a: integer) -> b: integer { a + 1 }",
        "fn f(a: integer) -> b: integer { h(a) }\n    fn h(a: integer) -> b: integer;",
    );
    let errors: Vec<String> = check(&parse(&text).unwrap())
        .iter()
        .map(|e| e.to_string())
        .collect();
    assert_eq!(errors, ["error[M023]: the conclusion: the condition of a set calls only functions with a body, read as their definition: `s::f` calls `s::h`, which has no body; write the bound by hand, or give it a body\n --> s::g::r"]);
}

/// Built by hand past check: a point not read to its end is no point of
/// the listing, and explain says so (X009) and gives the all nothing.
#[test]
fn a_point_not_decided_is_said() {
    let text = MODEL.replace("{ a + 1 }", ";");
    let m = parse(&text).unwrap();
    let Prop::All { over, .. } = the_all(&m) else {
        panic!("an all")
    };
    assert_eq!(over.undecided(), Some(vec![Value::Int(0.into())]));
    assert!(over.points().unwrap().any(|p| p.is_none()));
    let e = explain(&m, &Ledger::default(), &Settings::default());
    assert!(e
        .findings
        .iter()
        .any(|f| f.code.to_string() == "X009" && f.message.contains("not read to its end")));
}
