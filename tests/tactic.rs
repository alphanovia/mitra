//! Tactics write proofs and judgement checks them (`docs/core2.md`).
//! The measure of power, on `div(a, b)` with `a` and `b` in `0..10`:
//! how many takings it needs to reach "div is correct everywhere".

mod common;

use common::{apply, let_in};
use mitra_core::expression::{check, eval, Expression};
use mitra_core::judgement::{Judgement, Op, Opinion, Prop, Set, State, Taking, Term, Type, Value};
use mitra_core::simulation::{self, Value as V};
use mitra_core::tactic::{cases, intro};
use std::collections::{BTreeMap, BTreeSet};

type E = Expression<Op>;
type Sim = Expression<simulation::Op>;

fn int(n: i64) -> V {
    V::Int(n.into())
}
fn name<Op>(x: &str) -> Expression<Op> {
    Expression::Name(x.to_string())
}
fn sim(op: simulation::Op, args: Vec<Sim>) -> Sim {
    apply(op, args)
}
/// `(a, b)` in `0..10` × `0..10`, such that `such`.
fn grid(such: Option<Sim>) -> Set {
    Set {
        ranges: vec![("a".into(), int(0), int(10)), ("b".into(), int(0), int(10))],
        such,
    }
}
/// `b == 0`.
fn b_zero() -> Sim {
    let zero = sim(simulation::Op::Int(0.into()), vec![]);
    sim(simulation::Op::Equal, vec![name("b"), zero])
}
fn correct(at: Term) -> Prop {
    Prop::Correct {
        function: "div".to_string(),
        at,
    }
}
/// `div` is correct at every point of `over`.
fn everywhere(over: Set) -> Prop {
    Prop::All {
        name: "x".to_string(),
        over,
        body: Box::new(correct(Term::Var("x".to_string()))),
    }
}
fn point(a: i64, b: i64) -> Vec<V> {
    vec![int(a), int(b)]
}
/// The point `p`, written in a proof.
fn literal(p: &[V]) -> E {
    apply(Op::Point(p.to_vec()), vec![])
}
fn take(p: &Prop, value: bool, by: &str) -> Taking {
    Taking {
        prop: p.clone(),
        value,
        count: 1,
        by: by.to_string(),
    }
}
/// The name of the input that holds the test at `p`.
fn test_at(p: &[V]) -> String {
    let [V::Int(a), V::Int(b)] = p else {
        unreachable!()
    };
    format!("t{a}_{b}")
}

/// Inputs and their opinions, each taken by the takings given.
#[derive(Default)]
struct Ledger {
    inputs: BTreeMap<String, Type>,
    values: BTreeMap<String, Value>,
}

impl Ledger {
    fn take(&mut self, name: &str, p: &Prop, takings: Vec<Taking>) {
        let o = Opinion::taken(p.clone(), takings).unwrap();
        self.inputs
            .insert(name.to_string(), Type::Opinion(p.clone()));
        self.values.insert(name.to_string(), Value::Opinion(o));
    }
    /// What the proof gives, checked first.
    fn prove(&self, proof: &E, of: &Prop) -> Opinion {
        let j = Judgement::default();
        assert_eq!(
            check(&j, &self.inputs, proof),
            Some(Type::Opinion(of.clone()))
        );
        match eval(&j, &self.values, proof) {
            Ok(Value::Opinion(o)) => o,
            other => panic!("not an opinion: {other:?}"),
        }
    }
}

/// Exhaustive: one test at each of the 100 points, and `cases` over
/// them. "div is correct everywhere" rests on all 100.
#[test]
fn every_point_tested() {
    let q = everywhere(grid(None));
    let mut ledger = Ledger::default();
    for p in grid(None).points().unwrap() {
        let at = correct(Term::Point(p.clone()));
        ledger.take(&test_at(&p), &at, vec![take(&at, true, "machine:test")]);
    }
    let proof = cases(&q, |goal, p| {
        assert_eq!(goal, &correct(Term::Point(p.to_vec())));
        name(&test_at(p))
    })
    .unwrap();
    let o = ledger.prove(&proof, &q);
    assert_eq!(o.state(), State::True);
    let ways: Vec<usize> = o.rests_for().iter().map(BTreeSet::len).collect();
    assert_eq!(ways, [100]);
}

/// The two classes of div, `b != 0` and `b == 0`, each a rule a reviewer
/// takes: correct at one point of the class, correct on all of it
/// (main's "represented"). One test per class.
struct Represented {
    rest: Prop,
    zero: Prop,
    on_rest: Prop,
    on_zero: Prop,
}

fn represented() -> Represented {
    let not_zero = sim(simulation::Op::Not, vec![b_zero()]);
    let (on_rest, on_zero) = (
        everywhere(grid(Some(not_zero))),
        everywhere(grid(Some(b_zero()))),
    );
    let class = |at: Vec<V>, all: &Prop| Prop::Implies {
        pre: vec![correct(Term::Point(at))],
        subject: "div".to_string(),
        post: vec![all.clone()],
    };
    Represented {
        rest: class(point(6, 2), &on_rest),
        zero: class(point(3, 0), &on_zero),
        on_rest,
        on_zero,
    }
}

/// Represented: two reviews and two tests. `cases` over the 100 points
/// takes each point from its class, so "div is correct everywhere" rests
/// on those 4 takings.
#[test]
fn two_classes_two_tests() {
    let r = represented();
    let q = everywhere(grid(None));
    let (t62, t30) = (
        correct(Term::Point(point(6, 2))),
        correct(Term::Point(point(3, 0))),
    );
    let takings = [
        take(&r.rest, true, "human:reviewer"),
        take(&r.zero, true, "human:reviewer"),
        take(&t62, true, "machine:test"),
        take(&t30, true, "machine:test"),
    ];
    let mut ledger = Ledger::default();
    ledger.take("rest", &r.rest, vec![takings[0].clone()]);
    ledger.take("zero", &r.zero, vec![takings[1].clone()]);
    ledger.take("t6_2", &t62, vec![takings[2].clone()]);
    ledger.take("t3_0", &t30, vec![takings[3].clone()]);
    let ponens = |t: &Prop, args: [&str; 2]| {
        apply(
            Op::Ponens {
                t: t.clone(),
                post: 0,
            },
            args.map(name).to_vec(),
        )
    };
    let by_class = cases(&q, |_, p| {
        let (class, held) = match p[1] == int(0) {
            true => (&r.on_zero, "z"),
            false => (&r.on_rest, "r"),
        };
        apply(
            Op::Instance { q: class.clone() },
            vec![name(held), literal(p)],
        )
    })
    .unwrap();
    let proof = let_in(
        Op::Let,
        "r",
        ponens(&r.rest, ["rest", "t6_2"]),
        let_in(Op::Let, "z", ponens(&r.zero, ["zero", "t3_0"]), by_class),
    );
    let o = ledger.prove(&proof, &q);
    assert_eq!(o.state(), State::True);
    assert_eq!(o.rests_for(), &BTreeSet::from([BTreeSet::from(takings)]));
}

/// A second test in a class that fails refutes the class's review: the
/// review is wrong, given both tests, which the refutation names.
#[test]
fn a_failing_test_refutes_its_class() {
    let r = represented();
    let (t62, t73) = (
        correct(Term::Point(point(6, 2))),
        correct(Term::Point(point(7, 3))),
    );
    let (pass, fail) = (
        take(&t62, true, "machine:test"),
        take(&t73, false, "machine:test"),
    );
    let mut ledger = Ledger::default();
    ledger.take("t6_2", &t62, vec![pass.clone()]);
    ledger.take("t7_3", &t73, vec![fail.clone()]);
    let witness = Op::Witness {
        q: r.on_rest.clone(),
    };
    let tollens = Op::Tollens {
        t: r.rest.clone(),
        post: 0,
        blame: 0,
    };
    let proof = apply(
        tollens,
        vec![
            apply(witness, vec![name("t7_3"), literal(&point(7, 3))]),
            name("t6_2"),
        ],
    );
    let o = ledger.prove(&proof, &r.rest);
    assert_eq!(o.state(), State::False);
    assert_eq!(
        o.rests_against(),
        &BTreeSet::from([BTreeSet::from([pass, fail])])
    );
}

/// What a tactic writes is checked like any proof: a case that proves
/// another point does not check. A set of reals has no cases.
#[test]
fn a_tactic_is_trusted_with_nothing() {
    let q = everywhere(grid(None));
    let mut ledger = Ledger::default();
    for p in grid(None).points().unwrap() {
        let at = correct(Term::Point(p.clone()));
        ledger.take(&test_at(&p), &at, vec![take(&at, true, "machine:test")]);
    }
    let shifted = cases(&q, |_, p| {
        let [V::Int(a), V::Int(b)] = p else {
            unreachable!()
        };
        name(&format!("t{}_{}", b, a))
    })
    .unwrap();
    assert_eq!(check(&Judgement::default(), &ledger.inputs, &shifted), None);
    let one = simulation::Rational::integer(1);
    let reals = Set {
        ranges: vec![("v".into(), V::Real(one.minus(&one)), V::Real(one))],
        such: None,
    };
    assert!(cases(&everywhere(reals), |_, _| name("t0_0")).is_none());
    assert!(cases(&correct(Term::Point(point(0, 0))), |_, _| name("t0_0")).is_none());
}

/// A large set has too many points for cases: its all is taken, by a
/// review, and refuted by one failing test.
#[test]
fn a_large_set_is_taken_or_refuted() {
    let big = Set {
        ranges: vec![
            ("a".into(), int(0), int(1 << 40)),
            ("b".into(), int(1), int(1 << 40)),
        ],
        such: None,
    };
    let q = everywhere(big);
    let (at, fail_at) = (point(123_456_789, 42), point(5, 7));
    let (review, fail) = (
        take(&q, true, "human:reviewer"),
        take(
            &correct(Term::Point(fail_at.clone())),
            false,
            "machine:test",
        ),
    );
    let mut ledger = Ledger::default();
    ledger.take("review", &q, vec![review.clone()]);
    ledger.take(
        "fail",
        &correct(Term::Point(fail_at.clone())),
        vec![fail.clone()],
    );
    let down = apply(
        Op::Instance { q: q.clone() },
        vec![name("review"), literal(&at)],
    );
    let o = ledger.prove(&down, &correct(Term::Point(at)));
    assert_eq!(o.rests_for(), &BTreeSet::from([BTreeSet::from([review])]));
    let up = apply(
        Op::Witness { q: q.clone() },
        vec![name("fail"), literal(&fail_at)],
    );
    let o = ledger.prove(&up, &q);
    assert_eq!(o.rests_against(), &BTreeSet::from([BTreeSet::from([fail])]));
    // Out of the set: b is 0.
    let out = apply(
        Op::Instance { q },
        vec![name("review"), literal(&point(1, 0))],
    );
    assert_eq!(check(&Judgement::default(), &ledger.inputs, &out), None);
}

/// One proof for every point of a large set, by `intro`: div is correct
/// on all of it (a review), and quot is wherever div is (a rule a
/// reviewer takes). Quot is then correct on all of it, from those two
/// takings, in one step: no point is listed.
#[test]
fn one_proof_for_every_point() {
    let big = |b_from: i64| Set {
        ranges: vec![
            ("a".into(), int(0), int(1 << 40)),
            ("b".into(), int(b_from), int(1 << 40)),
        ],
        such: None,
    };
    let x = || Term::Var("x".to_string());
    let quot = |at: Term| Prop::Correct {
        function: "quot".to_string(),
        at,
    };
    let follows = |at: Term| Prop::Implies {
        pre: vec![correct(at.clone())],
        subject: "quot".to_string(),
        post: vec![quot(at)],
    };
    let on = |over: Set, body: Prop| Prop::All {
        name: "x".to_string(),
        over,
        body: Box::new(body),
    };
    let (div_all, chain) = (everywhere(big(1)), on(big(1), follows(x())));
    let (review, rule) = (
        take(&div_all, true, "human:reviewer"),
        take(&chain, true, "human:reviewer"),
    );
    let mut ledger = Ledger::default();
    ledger.take("div", &div_all, vec![review.clone()]);
    ledger.take("chain", &chain, vec![rule.clone()]);
    let by = |at: &Term| {
        let instance = |q: &Prop, held: &str| {
            apply(Op::Instance { q: q.clone() }, vec![name(held), name("p")])
        };
        apply(
            Op::Ponens {
                t: follows(at.clone()),
                post: 0,
            },
            vec![instance(&chain, "chain"), instance(&div_all, "div")],
        )
    };
    let goal = on(big(1), quot(x()));
    let proof = intro(&goal, "p", |g, at| {
        assert_eq!(g, &quot(at.clone()));
        by(at)
    })
    .unwrap();
    let o = ledger.prove(&proof, &goal);
    assert_eq!(
        o.rests_for(),
        &BTreeSet::from([BTreeSet::from([review.clone(), rule])])
    );
    // Over a set wider than what is known, b from 0: div is not known at
    // b == 0, so the instances do not check.
    let wider = on(big(0), quot(x()));
    let proof = intro(&wider, "p", |_, at| by(at)).unwrap();
    assert_eq!(check(&Judgement::default(), &ledger.inputs, &proof), None);
    // An any, false at any point, is false: no point where div is wrong.
    let wrong = Prop::Any {
        name: "x".to_string(),
        over: big(1),
        body: Box::new(Prop::Not(Box::new(correct(x())))),
    };
    let proof = intro(&wrong, "p", |_, _| {
        let at_p = apply(
            Op::Instance { q: div_all.clone() },
            vec![name("div"), name("p")],
        );
        apply(Op::Not, vec![at_p])
    })
    .unwrap();
    let o = ledger.prove(&proof, &wrong);
    assert_eq!(o.state(), State::False);
    assert_eq!(
        o.rests_against(),
        &BTreeSet::from([BTreeSet::from([review])])
    );
    // The point is known only inside the proof.
    let outside = apply(
        Op::Instance { q: div_all.clone() },
        vec![name("div"), name("p")],
    );
    assert_eq!(check(&Judgement::default(), &ledger.inputs, &outside), None);
    assert!(intro(&correct(x()), "p", |_, _| name("div")).is_none());
}
