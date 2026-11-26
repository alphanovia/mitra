//! Points in a world (finding 16): BDD rows `given … when … then …` in
//! the store and in a function's examples. What the environment gave a
//! run is part of the point judged: two worlds are two points; an all
//! over what a function takes holds in every world, so a point in a
//! world is an instance or a witness of it, never a case.

use mitra_lang::check::check;
use mitra_lang::explain::{explain, Settings, Verdict};
use mitra_lang::expression::{check as typed, eval, Arg, Expression};
use mitra_lang::judgement::{Judgement, Op, Opinion, Prop, Taking, Term, Type, Value, World};
use mitra_lang::model::Model;
use mitra_lang::parse::{parse, parse_prop};
use mitra_lang::print::{print, print_prop};
use mitra_lang::simulation::{Rational, Value as V};
use mitra_lang::store::{print_store, read_store};
use std::collections::BTreeMap;

const MODEL: &str = "environment sun {
    fn light() -> w: rational at \"lab/sun\";
    fn hour(d: integer) -> h: integer at \"lab/clock\";
}
subject stone {
    use sun::light;
    use sun::hour;
    fn heat(h: rational) -> e: rational at \"src/lib.rs/heat\" { h * light() } where {
        claim c: 0.0 <= h * h;
        rule r: heat::c |- all x in (h: 0.0..10.0): correct(heat, x);
    }
    fn cool(h: rational) -> e: rational at \"src/lib.rs/cool\" { h }
}";

fn real(n: i64) -> V {
    V::Real(Rational::integer(n))
}

fn model() -> Model {
    let m = parse(MODEL).unwrap();
    assert_eq!(check(&m), vec![]);
    m
}

fn at(h: i64, light: i64) -> Prop {
    Prop::Correct {
        function: "stone::heat".into(),
        at: Term::Given {
            point: vec![real(h)],
            world: World(vec![("sun::light".into(), vec![], real(light))]),
        },
    }
}

fn verdicts(store: &str) -> (Vec<(Prop, Verdict)>, Vec<String>) {
    let m = model();
    let (ledger, findings) = read_store("s", store, &m, &m.ties).unwrap();
    assert_eq!(findings, vec![], "{store}");
    let e = explain(&m, &ledger, &Settings::default());
    let v = e
        .verdicts
        .iter()
        .map(|v| (v.prop.clone(), v.verdict.clone()))
        .collect();
    (v, e.findings.iter().map(|f| f.to_string()).collect())
}

fn of(v: &[(Prop, Verdict)], p: &Prop) -> Verdict {
    v.iter()
        .find(|(q, _)| q == p)
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| panic!("no verdict on {}", print_prop(p)))
}

#[test]
fn two_worlds_are_two_points() {
    let (v, findings) = verdicts(
        "measure given sun::light() -> 2.0 when stone::heat(5.0) then 10.0 by \"test:monday\";
measure given sun::light() -> 3.0 when stone::heat(5.0) then 15.0 by \"test:tuesday\";",
    );
    assert_eq!(findings, Vec::<String>::new());
    assert_eq!(of(&v, &at(5, 2)), Verdict::Explained);
    assert_eq!(of(&v, &at(5, 3)), Verdict::Explained);
    // Every world is not two: the all is not explained by cases.
    let all = v
        .iter()
        .find(|(p, _)| matches!(p, Prop::All { .. }))
        .unwrap();
    assert_eq!(all.1, Verdict::Unsupported);
}

/// A wrong value in one world is a witness against the all, and leaves
/// the other world as it was.
#[test]
fn one_world_wrong_is_a_witness() {
    let (v, _) = verdicts(
        "measure given sun::light() -> 2.0 when stone::heat(5.0) then 11.0 by \"test:monday\";
measure given sun::light() -> 3.0 when stone::heat(5.0) then 15.0 by \"test:tuesday\";",
    );
    assert!(matches!(of(&v, &at(5, 2)), Verdict::Contradicted { .. }));
    assert_eq!(of(&v, &at(5, 3)), Verdict::Explained);
    let all = v
        .iter()
        .find(|(p, _)| matches!(p, Prop::All { .. }))
        .unwrap();
    assert!(matches!(all.1, Verdict::Contradicted { .. }));
}

/// A function that reads no world keeps today's point, in either form.
#[test]
fn no_world_no_change() {
    let plain = Prop::Correct {
        function: "stone::cool".into(),
        at: Term::Point(vec![real(4)]),
    };
    for store in [
        "measure stone::cool (4.0) -> 4.0 by \"test:a\";",
        "measure when stone::cool(4.0) then 4.0 by \"test:a\";",
    ] {
        let (v, _) = verdicts(store);
        assert_eq!(of(&v, &plain), Verdict::Explained);
    }
    // Given what it does not read: X005 says so, and the point is plain.
    let (v, findings) =
        verdicts("measure given sun::light() -> 2.0 when stone::cool(4.0) then 4.0 by \"test:a\";");
    assert_eq!(of(&v, &plain), Verdict::Explained);
    assert!(
        findings
            .iter()
            .any(|f| f.starts_with("finding[X005]: `sun::light::w`")),
        "{findings:?}"
    );
}

/// The store reads back what it prints, worlds included.
#[test]
fn the_store_reads_back_what_it_prints() {
    let m = model();
    let text = "measure given sun::light() -> 2.0, sun::hour(1) -> 9 when stone::heat(5.0) then 10.0 count 2 by \"test:monday\";
measure stone::cool (4.0) -> 4.0 failed by \"test:a\";
";
    let (ledger, findings) = read_store("s", text, &m, &m.ties).unwrap();
    assert_eq!(findings, vec![]);
    assert_eq!(print_store(&ledger), text);
}

/// What the store refuses in a `given`, each R006 with why.
#[test]
fn a_given_the_store_refuses() {
    let m = model();
    let cases = [
        ("stone::cool(1.0) -> 1.0", "has a body"),
        ("sun::moon() -> 1.0", "is no function of the model"),
        ("sun::light() -> 1", "is not of `sun::light`"),
        ("sun::light() -> 1.0, sun::light() -> 2.0", "is given twice"),
    ];
    for (given, why) in cases {
        let text = format!("measure given {given} when stone::heat(5.0) then 10.0 by \"test:a\";");
        let (ledger, findings) = read_store("s", &text, &m, &m.ties).unwrap();
        assert!(ledger.measurements.is_empty(), "{given}");
        assert_eq!(findings.len(), 1, "{given}");
        let shown = findings[0].to_string();
        assert!(
            shown.contains("R006") && shown.contains(why),
            "{given}: {shown}"
        );
    }
    // `given` with no `when` is not the store's.
    assert!(read_store(
        "s",
        "measure given sun::light() -> 1.0 stone::heat (5.0) -> 5.0 by \"test:a\";",
        &m,
        &m.ties
    )
    .is_err());
}

/// Examples in a world: compared with the body in that world; two
/// worlds may give two values at one point (no M024), one world may not.
#[test]
fn examples_in_a_world() {
    let with = |rows: &str| -> (Vec<String>, Vec<String>) {
        let text = MODEL.replace(
            "fn cool(h: rational) -> e: rational at \"src/lib.rs/cool\" { h }",
            &format!("fn cool(h: rational) -> e: rational at \"src/lib.rs/cool\" {{ h }}\n    fn warm(h: rational) -> e: rational {{ h * light() }} where {{\n        examples {{ {rows} }}\n    }}"),
        );
        let m = parse(&text).unwrap();
        let errors = check(&m).iter().map(|e| e.to_string()).collect();
        let agreed = mitra_lang::agree::agree(&m, &Default::default());
        let findings = agreed.1.iter().map(|f| f.to_string()).collect();
        (errors, findings)
    };
    let (errors, findings) = with(
        "given sun::light() -> 2.0 when (5.0) then 10.0, given light() -> 3.0 when (5.0) then 15.0",
    );
    assert_eq!((errors, findings), (vec![], vec![]));
    let (_, findings) = with("given sun::light() -> 2.0 when (5.0) then 11.0");
    assert!(
        findings
            .iter()
            .any(|f| f.starts_with("finding[A002]: example 0")),
        "{findings:?}"
    );
    let (errors, _) = with("given sun::light() -> 2.0 when (5.0) then 10.0, given sun::light() -> 2.0 when (5.0) then 11.0");
    assert!(
        errors.iter().any(|e| e.starts_with("error[M024]")),
        "{errors:?}"
    );
    // A later pair in one world, after a row of another world at that point.
    let (errors, _) = with("given sun::light() -> 1.0 when (5.0) then 5.0, given sun::light() -> 2.0 when (5.0) then 10.0, given sun::light() -> 2.0 when (5.0) then 11.0");
    assert!(
        errors
            .iter()
            .any(|e| e.starts_with("error[M024]: examples 1 and 2")),
        "{errors:?}"
    );
    let (errors, _) = with("given stone::cool(1.0) -> 1.0 when (5.0) then 10.0");
    assert!(
        errors
            .iter()
            .any(|e| e.starts_with("error[M022]: example 0: `stone::cool(1.0) -> 1.0`")),
        "{errors:?}"
    );
    // The model prints the rows as written, and reads them back.
    let text = MODEL.replace("{ h }", "{ h }\n    fn warm(h: rational) -> e: rational { h * light() } where {\n        examples { given sun::light() -> 2.0 when (5.0) then 10.0 }\n    }");
    let m = parse(&text).unwrap();
    assert_eq!(parse(&print(&m)).unwrap(), m);
    assert!(print(&m).contains("given sun::light() -> 2.0 when (5.0) then 10.0"));
}

/// A proposition at a point in a world reads back from its text.
#[test]
fn a_point_in_a_world_reads_back() {
    let m = model();
    let p = at(5, 2);
    let text = print_prop(&p);
    assert_eq!(
        text,
        "correct(stone::heat, (5.0) given sun::light() -> 2.0)"
    );
    assert_eq!(parse_prop(&text, &m).unwrap(), p);
}

/// The kernel: a point in a world is an instance and a witness of an
/// all, never one of its cases.
#[test]
fn the_kernel_reads_a_world_as_an_instance_never_a_case() {
    let m = model();
    let all = match &m.parts[1].functions[0].rules[0].conclusion {
        mitra_lang::model::Term::Prop(p) => p.clone(),
        _ => panic!(),
    };
    let take = |p: &Prop, value: bool| {
        Opinion::taken(
            p.clone(),
            vec![Taking {
                prop: p.clone(),
                value,
                count: 1,
                by: "test:t".into(),
            }],
        )
        .unwrap()
    };
    let run = |e: &Expression<Op>, taken: &[(&str, Opinion)]| -> Option<Opinion> {
        let mut inputs = BTreeMap::new();
        let mut values = BTreeMap::new();
        for (n, o) in taken {
            inputs.insert(n.to_string(), Type::Opinion(o.about().clone()));
            values.insert(n.to_string(), Value::Opinion(o.clone()));
        }
        let j = Judgement::default();
        typed(&j, &inputs, e)?;
        match eval(&j, &values, e) {
            Ok(Value::Opinion(o)) => Some(o),
            _ => None,
        }
    };
    let arg = |body| Arg {
        binds: vec![],
        body,
    };
    let world = World(vec![("sun::light".into(), vec![], real(2))]);
    let point = || {
        arg(Expression::Apply(
            Op::Given(vec![real(5)], world.clone()),
            vec![],
        ))
    };
    let name = |n: &str| arg(Expression::Name(n.into()));
    // Instance: the all, at a point in a world.
    let o = run(
        &Expression::Apply(Op::Instance { q: all.clone() }, vec![name("q"), point()]),
        &[("q", take(&all, true))],
    )
    .unwrap();
    assert_eq!(o.about(), &at(5, 2));
    // Witness: wrong in a world, the all is wrong.
    let o = run(
        &Expression::Apply(Op::Witness { q: all.clone() }, vec![name("x"), point()]),
        &[("x", take(&at(5, 2), false))],
    )
    .unwrap();
    assert_eq!(o.about(), &all);
    assert!(!o.rests_against().is_empty());
    // Never a case: a set over reals is not listed, and an all over
    // whole numbers is not shown from a point in a world.
    let ints = parse_prop("all x in (h: 0..1): correct(stone::heat, x)", &m).unwrap();
    let in_world = Prop::Correct {
        function: "stone::heat".into(),
        at: Term::Given {
            point: vec![V::Int(0.into())],
            world: world.clone(),
        },
    };
    let cases = Expression::Apply(Op::Cases { q: ints }, vec![name("x")]);
    assert!(run(&cases, &[("x", take(&in_world, true))]).is_none());
}

/// An example of a function with no body, in a world, is what it gave
/// there only: not a row a caller reads.
#[test]
fn an_example_in_a_world_is_no_row() {
    let text = MODEL.replace(
        "fn cool(h: rational) -> e: rational at \"src/lib.rs/cool\" { h }",
        "fn cool(h: rational) -> e: rational at \"src/lib.rs/cool\" { h }
    fn lamp(h: rational) -> e: rational where {
        examples { given sun::light() -> 2.0 when (5.0) then 10.0 }
    }
    fn uses(h: rational) -> e: rational { lamp(h) } where {
        examples { (5.0) -> 99.0 }
    }",
    );
    let m = parse(&text).unwrap();
    assert_eq!(check(&m), vec![]);
    let (_, findings) = mitra_lang::agree::agree(&m, &Default::default());
    let shown: Vec<String> = findings.iter().map(|f| f.to_string()).collect();
    assert!(
        !shown.iter().any(|f| f.starts_with("finding[A002]")),
        "{shown:?}"
    );
}

/// `_` for any arguments (finding 30): one row answers every call of the
/// function, a row with written arguments wins where it fits.
#[test]
fn any_arguments() {
    let with = |rows: &str| -> (Vec<String>, Vec<String>) {
        let text = MODEL.replace(
            "fn cool(h: rational) -> e: rational at \"src/lib.rs/cool\" { h }",
            &format!("fn cool(h: rational) -> e: rational at \"src/lib.rs/cool\" {{ h }}\n    fn warm(h: integer) -> e: integer {{ h * hour(1) + hour(2) }} where {{\n        examples {{ {rows} }}\n    }}"),
        );
        let m = parse(&text).unwrap();
        let errors = check(&m).iter().map(|e| e.to_string()).collect();
        let agreed = mitra_lang::agree::agree(&m, &Default::default());
        (errors, agreed.1.iter().map(|f| f.to_string()).collect())
    };
    // Any hour is 3: 5 * 3 + 3.
    assert_eq!(
        with("given sun::hour(_) -> 3 when (5) then 18"),
        (vec![], vec![])
    );
    // A written row wins: hour(2) is 4, any other 3: 5 * 3 + 4.
    assert_eq!(
        with("given sun::hour(_) -> 3, sun::hour(2) -> 4 when (5) then 19"),
        (vec![], vec![])
    );
    // And whatever the order it is written in.
    assert_eq!(
        with("given sun::hour(2) -> 4, sun::hour(_) -> 3 when (5) then 19"),
        (vec![], vec![])
    );
    let (_, findings) = with("given sun::hour(_) -> 3 when (5) then 19");
    assert!(
        findings
            .iter()
            .any(|f| f.starts_with("finding[A002]: example 0")),
        "{findings:?}"
    );
    // `_` still takes the count of arguments, and a written one its type.
    let (errors, _) = with("given sun::hour(_, _) -> 3 when (5) then 18");
    assert!(
        errors
            .iter()
            .any(|e| e.starts_with("error[M022]: example 0: `sun::hour(_, _) -> 3`")),
        "{errors:?}"
    );
    let (errors, _) = with("given sun::hour(true) -> 3 when (5) then 18");
    assert!(
        errors.iter().any(|e| e.starts_with("error[M022]")),
        "{errors:?}"
    );
    // Printed as written, read back.
    let text = MODEL.replace("{ h }", "{ h }\n    fn warm(h: integer) -> e: integer { h * hour(1) } where {\n        examples { given sun::hour(_) -> 3 when (5) then 15 }\n    }");
    let m = parse(&text).unwrap();
    assert!(print(&m).contains("given sun::hour(_) -> 3 when (5) then 15"));
    assert_eq!(parse(&print(&m)).unwrap(), m);
}

const WHOLE: &str = "environment sun {
    fn light() -> w: rational;
    fn hour(d: integer) -> h: integer;
}
subject stone {
    use sun::light;
    fn heat(h: rational) -> e: rational { h * light() + h * light() }
}";

fn whole(rows: &str) -> (Vec<String>, Vec<String>) {
    let m = parse(&format!("{WHOLE}\nexamples {{ {rows} }}")).unwrap();
    let errors = check(&m).iter().map(|e| e.to_string()).collect();
    let agreed = mitra_lang::agree::agree(&m, &Default::default());
    (errors, agreed.1.iter().map(|f| f.to_string()).collect())
}

/// A row of the model, end to end: `given` the environment, `when` a
/// function of the model is called, `then` what it gives.
#[test]
fn rows_of_the_model() {
    assert_eq!(
        whole("given sun::light() -> 2.0 when stone::heat(5.0) then 20.0"),
        (vec![], vec![])
    );
    // A bare name is the one function of that name.
    assert_eq!(
        whole("given light() -> 2.0 when stone::heat(5.0) then 20.0"),
        (vec![], vec![])
    );
    let (_, findings) = whole("given sun::light() -> 2.0 when stone::heat(5.0) then 21.0");
    assert!(
        findings
            .iter()
            .any(|f| f.starts_with("finding[A006]: `stone::heat::e`")),
        "{findings:?}"
    );
    // `given` in front of the ports form too.
    assert_eq!(
        whole("given sun::light() -> 2.0 (stone::heat::h = 5.0) -> stone::heat::e = 20.0"),
        (vec![], vec![])
    );
    // A `when` that names no function of the model.
    let (errors, _) = whole("when stone::boil(5.0) then 20.0");
    assert!(
        errors.iter().any(|e| e.starts_with("error[M021]")),
        "{errors:?}"
    );
    // A `given` that is not what the environment gives.
    let (errors, _) = whole("given sun::light() -> true when stone::heat(5.0) then 20.0");
    assert!(
        errors.iter().any(|e| e.starts_with("error[M022]")),
        "{errors:?}"
    );
    // A given row no run reads: the run never calls it (finding 39 B).
    let (_, findings) =
        whole("given sun::light() -> 2.0, sun::hour(_) -> 3 when stone::heat(5.0) then 20.0");
    assert!(
        findings.iter().any(|f| f.starts_with("finding[A015]: the example: `sun::hour(_)` is given but never called")),
        "{findings:?}"
    );
    // Printed as written, read back.
    let m = parse(&format!(
        "{WHOLE}\nexamples {{ given sun::hour(_) -> 3, sun::light() -> 2.0 when stone::heat(5.0) then 20.0 }}"
    ))
    .unwrap();
    let printed = print(&m);
    assert_eq!(parse(&printed).unwrap(), m, "{printed}");
}

/// A given value is read by the type its function gives: a bare variant
/// of its enum, as an input of that type would be.
#[test]
fn a_given_value_is_read_by_its_type() {
    let m = parse(
        "environment enum Weather { Sun, Rain }
environment sky {
    fn now() -> w: Weather;
}
subject walk {
    use sky::now;
    fn go(n: integer) -> g: bool { match now() { Weather::Sun => true, Weather::Rain => false } }
}
examples {
    given sky::now() -> Rain when walk::go(1) then false,
    given now() -> Sun when walk::go(2) then true
}",
    )
    .unwrap();
    assert_eq!(check(&m), vec![]);
    let (_, findings) = mitra_lang::agree::agree(&m, &Default::default());
    assert_eq!(
        findings.iter().map(|f| f.to_string()).collect::<Vec<_>>(),
        Vec::<String>::new()
    );
}

/// One answer for each call in a row: two for the same arguments are
/// M022, in a function's examples and in the model's.
#[test]
fn a_call_given_twice_in_a_row() {
    let (errors, _) =
        whole("given sun::light() -> 2.0, sun::light() -> 3.0 when stone::heat(5.0) then 10.0");
    assert!(
        errors
            .iter()
            .any(|e| e.starts_with("error[M022]: `sun::light()` is given twice in one row")),
        "{errors:?}"
    );
    let (errors, _) = whole("given sun::hour(_) -> 1, sun::hour(_) -> 2, sun::light() -> 2.0 when stone::heat(5.0) then 20.0");
    assert!(
        errors
            .iter()
            .any(|e| e.starts_with("error[M022]: `sun::hour(_)` is given twice in one row")),
        "{errors:?}"
    );
    // Two arguments, two calls: no error.
    let (errors, _) = whole("given sun::hour(1) -> 1, sun::hour(2) -> 2, sun::hour(_) -> 3, sun::light() -> 2.0 when stone::heat(5.0) then 20.0");
    assert!(!errors.iter().any(|e| e.contains("twice")), "{errors:?}");
}
