//! Red team of calls in a set's condition (finding 12b). The kernel
//! trusts that two equal sets carry the same definitions; these tests
//! check that a set carrying none, or reading past its budget, fails
//! closed: no point listed, no instance, no witness, no proof by cases,
//! and never an all explained.

use mitra_lang::check::check;
use mitra_lang::evidence::Ledger;
use mitra_lang::explain::{explain, Settings, Verdict};
use mitra_lang::expression::{check as typed, eval, Arg, Expression};
use mitra_lang::judgement::{Definitions, Judgement, Op, Opinion, Prop, Taking, Term, Type, Value};
use mitra_lang::model::{define_sets, Model};
use mitra_lang::parse::{parse, parse_prop};
use mitra_lang::simulation::{Budget, Value as V};
use std::collections::BTreeMap;

const MODEL: &str = "subject s {
    fn k(a: integer) -> b: integer { a * a }
    fn f(a: integer) -> b: integer { k(a) + 1 }
    fn g(a: integer) -> b: integer at \"src/lib.rs/g\" { a } where {
        claim c: 0 <= b;
        rule r: g::c |- all p in (a: 0..10) where f(a) < 5: correct(g, p);
        rule w: g::c |- any p in (a: 0..10) where f(a) < 5: correct(g, p);
    }
}";

fn rule(m: &Model, i: usize) -> Prop {
    match &m.parts[0].functions[2].rules[i].conclusion {
        mitra_lang::model::Term::Prop(p) => p.clone(),
        _ => panic!("a proposition"),
    }
}

fn strip(p: &Prop) -> Prop {
    let mut q = p.clone();
    q.define(&Definitions::default());
    q
}

fn int(n: i64) -> V {
    V::Int(n.into())
}

fn correct_at(n: i64) -> Prop {
    Prop::Correct {
        function: "s::g".to_string(),
        at: Term::Point(vec![int(n)]),
    }
}

fn arg(body: Expression<Op>) -> Arg<Op> {
    Arg {
        binds: vec![],
        body,
    }
}

/// A proof by the kernel, from opinions named `o0`, `o1`, ...
fn run(proof: &Expression<Op>, taken: &[(&str, Prop)]) -> Option<Opinion> {
    let (mut inputs, mut values) = (BTreeMap::new(), BTreeMap::new());
    for (name, p) in taken {
        let t = Taking {
            prop: p.clone(),
            value: true,
            count: 1,
            by: format!("test:{name}"),
        };
        let o = Opinion::taken(p.clone(), vec![t])?;
        inputs.insert(name.to_string(), Type::Opinion(p.clone()));
        values.insert(name.to_string(), Value::Opinion(o));
    }
    let j = Judgement::default();
    typed(&j, &inputs, proof)?;
    match eval(&j, &values, proof) {
        Ok(Value::Opinion(o)) => Some(o),
        _ => None,
    }
}

/// The trust point: a set with no definitions equals the model's, and
/// then every reading of it fails closed.
#[test]
fn a_set_without_its_definitions_fails_closed() {
    let m = parse(MODEL).unwrap();
    assert_eq!(check(&m), vec![]);
    let all = rule(&m, 0);
    let bare = strip(&all);
    assert_eq!(all, bare);
    assert!(all.well_formed());
    assert!(!bare.well_formed());
    let (Prop::All { over, .. }, Prop::All { over: o2, .. }) = (&all, &bare) else {
        panic!()
    };
    assert_eq!(over.points().unwrap().flatten().count(), 2);
    assert!(o2.points().unwrap().all(|p| p.is_none()));
    assert!(!o2.contains(&[int(0)]));
    assert!(bare.case_bodies(10).is_none());
    // The kernel: a proof by cases, an instance, a witness, over the bare
    // set, give nothing; over the model's, they do.
    let taken = [("o0", correct_at(0)), ("o1", correct_at(1))];
    let cases = |q: &Prop| {
        Expression::Apply(
            Op::Cases { q: q.clone() },
            vec![
                arg(Expression::Name("o0".into())),
                arg(Expression::Name("o1".into())),
            ],
        )
    };
    assert!(run(&cases(&all), &taken).is_some());
    assert!(run(&cases(&bare), &taken).is_none());
    let any = rule(&m, 1);
    let witness = |q: &Prop| {
        Expression::Apply(
            Op::Witness { q: q.clone() },
            vec![
                arg(Expression::Name("o0".into())),
                arg(Expression::Apply(Op::Point(vec![int(0)]), vec![])),
            ],
        )
    };
    assert!(run(&witness(&any), &taken).is_some());
    assert!(run(&witness(&strip(&any)), &taken).is_none());
}

/// A proposition read from a store carries the model's definitions, the
/// very ones, budget included.
#[test]
fn a_store_proposition_reads_its_set_as_the_model_does() {
    let mut m = parse(MODEL).unwrap();
    define_sets(&mut m, Budget { steps: 1, depth: 1 });
    let read = parse_prop(
        "all p in (a: 0..10) where s::f(a) < 5: correct(s::g, p)",
        &m,
    )
    .unwrap();
    assert_eq!(read, rule(&m, 0));
    let Prop::All { over, .. } = &read else {
        panic!()
    };
    assert_eq!(over.undecided(), Some(vec![int(0)]));
}

/// A budget reached at a point leaves the all unsupported, even with a
/// test at every point of the set, and X009 says where.
#[test]
fn a_budget_reached_gives_nothing() {
    let store: String = (0..2)
        .map(|a| format!("measure s::g ({a}) -> {a} by \"test:g{a}\";\n"))
        .collect();
    let verdict = |budget: Budget| {
        let mut m = parse(MODEL).unwrap();
        define_sets(&mut m, budget);
        let ledger = mitra_lang::store::read_store("s", &store, &m, &m.ties)
            .unwrap()
            .0;
        let e = explain(&m, &ledger, &Settings::default());
        let all = rule(&m, 0);
        let v = e.verdicts.iter().find(|v| v.prop == all).unwrap();
        let said = e.findings.iter().any(|f| {
            f.code.to_string() == "X009" && f.message.contains("not read to its end at (0)")
        });
        (v.verdict.clone(), said)
    };
    assert_eq!(verdict(Budget::default()), (Verdict::Explained, false));
    assert_eq!(
        verdict(Budget { steps: 1, depth: 1 }),
        (Verdict::Unsupported, true)
    );
}

/// Explain never says an all over a set it could not list is explained,
/// whatever the budget, and agrees with listing when it can.
#[test]
fn every_budget_is_sound() {
    for steps in 0..6 {
        for depth in 0..4 {
            let mut m = parse(MODEL).unwrap();
            define_sets(&mut m, Budget { steps, depth });
            let all = rule(&m, 0);
            let Prop::All { over, .. } = &all else {
                panic!()
            };
            let decided = over.undecided().is_none();
            let store =
                "measure s::g (0) -> 0 by \"test:g0\";\nmeasure s::g (1) -> 1 by \"test:g1\";\n";
            let ledger = mitra_lang::store::read_store("s", store, &m, &m.ties)
                .unwrap()
                .0;
            let e = explain(&m, &ledger, &Settings::default());
            let v = &e.verdicts.iter().find(|v| v.prop == all).unwrap().verdict;
            assert_eq!(
                *v == Verdict::Explained,
                decided,
                "steps {steps}, depth {depth}"
            );
        }
    }
    let _ = Ledger::default();
}

/// A set some of whose points are decided and others not: 0 and 1 by
/// the condition alone, the rest through `h`, which has no body (built
/// past check, which refuses it).
const HALF: &str = "subject s {
    fn h(a: integer) -> b: integer;
    fn g(a: integer) -> b: integer at \"src/lib.rs/g\" { a } where {
        claim c: 0 <= b;
        rule r: g::c |- all p in (a: 0..4) where if a < 2 { true } else { h(a) < 5 }: correct(g, p);
        rule e: g::c |- all p in (a: 5..5) where h(a) < 5: correct(g, p);
    }
}";

fn half_rule(m: &Model, i: usize) -> Prop {
    match &m.parts[0].functions[1].rules[i].conclusion {
        mitra_lang::model::Term::Prop(p) => p.clone(),
        _ => panic!("a proposition"),
    }
}

/// Two points decided and two not: no proof by cases from the two, by
/// the kernel, by the tactic or by explain, with a test at each.
#[test]
fn some_points_decided_are_not_every_point() {
    let m = parse(HALF).unwrap();
    let all = half_rule(&m, 0);
    let Prop::All { over, .. } = &all else {
        panic!()
    };
    let listed: Vec<Option<Vec<V>>> = over.points().unwrap().collect();
    assert_eq!(listed, [Some(vec![int(0)]), Some(vec![int(1)]), None, None]);
    assert_eq!(over.undecided(), Some(vec![int(2)]));
    let taken = [("o0", correct_at(0)), ("o1", correct_at(1))];
    let cases = Expression::Apply(
        Op::Cases { q: all.clone() },
        vec![
            arg(Expression::Name("o0".into())),
            arg(Expression::Name("o1".into())),
        ],
    );
    assert!(run(&cases, &taken).is_none());
    assert!(all.case_bodies(10).is_none());
    assert!(mitra_lang::tactic::cases(&all, |_, _| Expression::Name("o0".into())).is_none());
    let store = "measure s::g (0) -> 0 by \"test:g0\";\nmeasure s::g (1) -> 1 by \"test:g1\";\n";
    let ledger = mitra_lang::store::read_store("s", store, &m, &m.ties)
        .unwrap()
        .0;
    let e = explain(&m, &ledger, &Settings::default());
    let v = &e.verdicts.iter().find(|v| v.prop == all).unwrap().verdict;
    assert_eq!(*v, Verdict::Unsupported);
    // An empty range reads no point, so none is undecided.
    let Prop::All { over, .. } = half_rule(&m, 1) else {
        panic!()
    };
    assert_eq!(over.undecided(), None);
}

/// Every set a proposition or a proof names is given the definitions:
/// inside an all, under a `!`, at the point an intro binds, and in each
/// argument of a proof.
#[test]
fn define_reaches_every_set() {
    let m = parse(MODEL).unwrap();
    let all = rule(&m, 0);
    let defs = all.defs().unwrap();
    let Prop::All { name, over, body } = strip(&all) else {
        panic!()
    };
    let inner = Prop::All {
        name: "q".into(),
        over: over.clone(),
        body: body.clone(),
    };
    let mut nested = Prop::Not(Box::new(Prop::All {
        name,
        over: over.clone(),
        body: Box::new(inner),
    }));
    assert!(nested.defs().is_none());
    nested.define(&defs);
    let Prop::Not(outer) = &nested else { panic!() };
    let Prop::All { body: inner, .. } = &**outer else {
        panic!()
    };
    assert!(inner.defs().is_some());
    let Prop::All {
        over: inner_set, ..
    } = &**inner
    else {
        panic!()
    };
    assert!(inner_set.defs.0.is_some());
    // The point an intro binds.
    let mut at = Prop::Correct {
        function: "s::g".into(),
        at: Term::Arbitrary {
            name: "p".into(),
            over: over.clone(),
        },
    };
    assert!(at.defs().is_none());
    at.define(&defs);
    assert!(at.defs().is_some());
    // A proof: an instance under a `!`.
    let mut proof = Expression::Apply(
        Op::Not,
        vec![arg(Expression::Apply(
            Op::Instance { q: strip(&all) },
            vec![arg(Expression::Name("o".into()))],
        ))],
    );
    mitra_lang::judgement::define_proof(&mut proof, &defs);
    let Expression::Apply(_, args) = &proof else {
        panic!()
    };
    let Expression::Apply(Op::Instance { q }, _) = &args[0].body else {
        panic!()
    };
    assert!(q.defs().is_some());
}

/// A store's proposition lists the set's points as the model's does.
#[test]
fn a_store_proposition_lists_its_points() {
    let m = parse(MODEL).unwrap();
    let read = parse_prop(
        "all p in (a: 0..10) where s::f(a) < 5: correct(s::g, p)",
        &m,
    )
    .unwrap();
    let Prop::All { over, .. } = &read else {
        panic!()
    };
    assert_eq!(over.points().unwrap().flatten().count(), 2);
    assert!(read.well_formed());
}
