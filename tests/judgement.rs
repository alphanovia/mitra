//! The judgement standard (`docs/core2.md`): S2 to S4, J1 (every rule
//! preserves truth, checked on every valuation), J2 (states are read
//! from rests) and J3 (rests are the smallest, and all) on random
//! derivations, and the proposition walked through.

mod common;

use common::{apply, inline, let_in, Rng};
use mitra_core::expression::{check, eval, fits, Arg, Expression, Fault, Reading};
use mitra_core::judgement::{Definition, Judgement, Op, Opinion, Prop, State, Taking, Type, Value};
use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

type E = Expression<Op>;

fn is_let(op: &Op) -> bool {
    matches!(op, Op::Let)
}
fn input(name: &str) -> E {
    Expression::Name(name.to_string())
}

/// A random model: three named propositions and their negations, one to three
/// rules over them (a rule may be about an earlier one), each written in
/// place or known by a name, and a leaf opinion of every proposition
/// from random takings.
struct World {
    rng: Rng,
    props: Vec<Prop>,
    /// Each rule, its pres and its posts, as the world makes them.
    rules: Vec<(Prop, Vec<Prop>, Vec<Prop>)>,
    defined: BTreeMap<String, Definition>,
    judgement: Judgement,
    names: BTreeMap<Prop, String>,
    inputs: BTreeMap<String, Type>,
    values: BTreeMap<String, Value>,
    /// The names `let` binds in scope, the nearest last.
    bound: Vec<(String, Type)>,
}

impl World {
    fn new(seed: u64) -> World {
        let mut rng = Rng::new(seed);
        let atoms: Vec<Prop> = ["a", "b", "c"].map(Prop::named).to_vec();
        let mut props: Vec<Prop> = atoms
            .iter()
            .cloned()
            .chain(atoms.iter().map(Prop::not))
            .collect();
        let (mut rules, mut defined) = (Vec::new(), BTreeMap::new());
        for i in 0..1 + rng.below(3) {
            let pool: Vec<Prop> = props
                .iter()
                .chain(rules.iter().map(|(t, _, _)| t))
                .cloned()
                .collect();
            let pre: Vec<Prop> = (0..rng.below(3)).map(|_| rng.pick(&pool).clone()).collect();
            let post: Vec<Prop> = (0..1 + rng.below(2))
                .map(|_| rng.pick(&pool).clone())
                .collect();
            let t = match rng.coin() {
                true => Prop::Implies {
                    pre: pre.clone(),
                    subject: format!("s{i}"),
                    post: post.clone(),
                },
                false => {
                    let (name, post) = (format!("r{i}"), post.clone());
                    let pre = pre.clone();
                    defined.insert(name.clone(), Definition { pre, post });
                    Prop::Named(name)
                }
            };
            rules.push((t, pre, post));
        }
        props.extend(rules.iter().flat_map(|(t, _, _)| [t.clone(), t.not()]));
        let judgement = Judgement::with(defined.clone()).unwrap();
        let mut w = World {
            rng,
            props: props.clone(),
            rules,
            defined,
            judgement,
            names: BTreeMap::new(),
            inputs: BTreeMap::new(),
            values: BTreeMap::new(),
            bound: Vec::new(),
        };
        for (i, prop) in props.iter().enumerate() {
            let name = format!("o{i}");
            let takings = (0..w.rng.below(4)).map(|_| w.taking(prop)).collect();
            let opinion = Opinion::taken(prop.clone(), takings).unwrap();
            w.names.insert(prop.clone(), name.clone());
            w.inputs.insert(name.clone(), Type::Opinion(prop.clone()));
            w.values.insert(name, Value::Opinion(opinion));
        }
        for name in ["s0", "s1"] {
            let state = *w
                .rng
                .pick(&[State::Unknown, State::True, State::False, State::Conflict]);
            w.inputs.insert(name.to_string(), Type::State);
            w.values.insert(name.to_string(), Value::State(state));
        }
        w
    }
    fn taking(&mut self, prop: &Prop) -> Taking {
        Taking {
            prop: prop.clone(),
            value: self.rng.below(3) > 0,
            count: 1 + self.rng.below(3) as u64,
            by: self.rng.pick(&["v1", "v2"]).to_string(),
        }
    }
    fn prop(&mut self) -> Prop {
        self.rng.pick(&self.props.clone()).clone()
    }
    /// The premises of the rule `t`, `t` first, and its posts.
    fn rule(&self, t: &Prop) -> (Vec<Prop>, Vec<Prop>) {
        let (_, pre, post) = self.rules.iter().find(|(r, _, _)| r == t).unwrap();
        ([vec![t.clone()], pre.clone()].concat(), post.clone())
    }
    /// A name in scope of type `t`: an input, or one a `let` binds.
    fn name(&mut self, t: &Type) -> E {
        let mut visible = self.inputs.clone();
        visible.extend(self.bound.iter().cloned());
        let names: Vec<String> = visible
            .into_iter()
            .filter(|(_, it)| it == t)
            .map(|(n, _)| n)
            .collect();
        Expression::Name(self.rng.pick(&names).clone())
    }
    /// `let n = <value of type t>; <body>`, where `n` may shadow a name of
    /// the same type, or another let.
    fn let_in(
        &mut self,
        t: Type,
        value: impl FnOnce(&mut Self) -> E,
        body: impl FnOnce(&mut Self) -> E,
    ) -> E {
        let same = match &t {
            Type::Opinion(y) => self.names[y].clone(),
            Type::State => "s0".to_string(),
        };
        let name = self
            .rng
            .pick(&[same, "l0".to_string(), "l1".to_string()])
            .clone();
        let value = value(self);
        self.bound.push((name.clone(), t));
        let body = body(self);
        self.bound.pop();
        let_in(Op::Let, &name, value, body)
    }
    /// A random derivation of an opinion of `x`.
    fn opinion(&mut self, x: &Prop, depth: u32) -> E {
        if depth == 0 || self.rng.below(4) == 0 {
            return self.name(&Type::Opinion(x.clone()));
        }
        let d = depth - 1;
        if self.rng.below(6) == 0 {
            let y = match self.rng.coin() {
                true => x.clone(),
                false => self.prop(),
            };
            let x = x.clone();
            return self.let_in(
                Type::Opinion(y.clone()),
                |w| w.opinion(&y, d),
                |w| w.opinion(&x, d),
            );
        }
        let mut ways = vec![Op::Join, Op::Not, Op::Reductio { of: x.clone() }];
        for (t, _, _) in &self.rules {
            let (premises, posts) = self.rule(t);
            for (post, q) in posts.iter().enumerate() {
                if q == x {
                    ways.extend(vec![Op::Ponens { t: t.clone(), post }; 2]);
                }
                for (blame, p) in premises.iter().enumerate() {
                    if p == x {
                        ways.extend(vec![
                            Op::Tollens {
                                t: t.clone(),
                                post,
                                blame
                            };
                            2
                        ]);
                    }
                }
            }
        }
        let op = self.rng.pick(&ways).clone();
        let args = match &op {
            Op::Join => vec![self.opinion(x, d), self.opinion(x, d)],
            Op::Not => vec![self.opinion(&x.not(), d)],
            Op::Reductio { .. } => {
                // Where a conflict may hold a taking of x: x itself, a post
                // of an implication x is a premise of, or any proposition.
                let posts: Vec<Prop> = self
                    .rules
                    .iter()
                    .filter(|(t, pre, _)| t == x || pre.contains(x))
                    .flat_map(|(_, _, post)| post.clone())
                    .collect();
                let y = match self.rng.below(3) {
                    0 => x.clone(),
                    1 if !posts.is_empty() => self.rng.pick(&posts).clone(),
                    _ => self.prop(),
                };
                vec![self.opinion(&y, d)]
            }
            Op::Ponens { t, .. } => self.rule(t).0.iter().map(|p| self.opinion(p, d)).collect(),
            Op::Tollens { t, post, blame } => {
                let (mut from, posts) = self.rule(t);
                from.remove(*blame);
                let q = posts[*post].clone();
                [q].iter()
                    .chain(&from)
                    .map(|p| self.opinion(p, d))
                    .collect()
            }
            _ => unreachable!(),
        };
        apply(op, args)
    }
    /// A random expression of a state.
    fn state(&mut self, depth: u32) -> E {
        if depth == 0 || self.rng.below(4) == 0 {
            return self.name(&Type::State);
        }
        let d = depth - 1;
        match self.rng.below(5) {
            4 => self.let_in(Type::State, |w| w.state(d), |w| w.state(d)),
            0 => {
                let y = self.prop();
                apply(Op::StateOf, vec![self.opinion(&y, d)])
            }
            1 => apply(Op::Not, vec![self.state(d)]),
            2 => apply(Op::And, vec![self.state(d), self.state(d)]),
            _ => apply(Op::Join, vec![self.state(d), self.state(d)]),
        }
    }
    /// A random type and an expression of it.
    fn typed(&mut self, depth: u32) -> (Type, E) {
        match self.rng.below(4) {
            0 => (Type::State, self.state(depth)),
            _ => {
                let x = self.prop();
                (Type::Opinion(x.clone()), self.opinion(&x, depth))
            }
        }
    }
    /// Any operation, with any parameters.
    fn wild_op(&mut self) -> Op {
        let t = self.prop();
        let (post, blame) = (self.rng.below(4), self.rng.below(5));
        let all = [
            Op::Not,
            Op::And,
            Op::Join,
            Op::StateOf,
            Op::Ponens { t: t.clone(), post },
            Op::Tollens {
                t: t.clone(),
                post,
                blame,
            },
            Op::Reductio { of: t },
            Op::Let,
        ];
        self.rng.pick(&all).clone()
    }
    /// Any expression, typed or not.
    fn wild(&mut self, depth: u32) -> E {
        if depth == 0 || self.rng.below(4) == 0 {
            return Expression::Name(format!("o{}", self.rng.below(self.props.len() + 2)));
        }
        if self.rng.below(6) == 0 {
            let name = format!("o{}", self.rng.below(self.props.len() + 2));
            let (value, body) = (self.wild(depth - 1), self.wild(depth - 1));
            return let_in(Op::Let, &name, value, body);
        }
        let op = self.wild_op();
        let names = self.props.len() + 2;
        let args = (0..self.rng.below(5))
            .map(|_| {
                let binds = (0..[0, 0, 0, 1, 2][self.rng.below(5)])
                    .map(|_| format!("o{}", self.rng.below(names)))
                    .collect();
                Arg {
                    binds,
                    body: self.wild(depth - 1),
                }
            })
            .collect();
        Expression::Apply(op, args)
    }
}

/// The meaning of propositions, for J1: a valuation gives a truth to every
/// named proposition and every implication, and a rule `t` that is true
/// makes its posts true when its pres are. A rule is an implication, or
/// a name `defined` defines.
struct Meaning {
    vars: Vec<Prop>,
    rules: Vec<(Prop, Vec<Prop>, Vec<Prop>)>,
}

type Rules = BTreeSet<(Prop, Vec<Prop>, Vec<Prop>)>;

impl Meaning {
    fn of(props: &[Prop], defined: &BTreeMap<String, Definition>) -> Meaning {
        let collect = |p: &Prop, vars: &mut BTreeSet<Prop>, rules: &mut Rules| {
            let mut todo = vec![p.clone()];
            while let Some(p) = todo.pop() {
                let rule = match &p {
                    Prop::Named(n) => defined.get(n).map(|d| (d.pre.clone(), d.post.clone())),
                    Prop::Not(x) => {
                        todo.push((**x).clone());
                        continue;
                    }
                    Prop::Implies { pre, post, .. } => Some((pre.clone(), post.clone())),
                };
                if vars.insert(p.clone()) {
                    if let Some((pre, post)) = rule {
                        todo.extend(pre.iter().chain(&post).cloned());
                        rules.insert((p, pre, post));
                    }
                }
            }
        };
        let (mut vars, mut rules) = (BTreeSet::new(), Rules::new());
        props.iter().for_each(|p| collect(p, &mut vars, &mut rules));
        Meaning {
            vars: vars.into_iter().collect(),
            rules: rules.into_iter().collect(),
        }
    }
    fn truth(&self, p: &Prop, bits: u64) -> bool {
        match p {
            Prop::Not(x) => !self.truth(x, bits),
            _ => {
                let place = self
                    .vars
                    .iter()
                    .position(|v| v == p)
                    .expect("a prop outside the world");
                bits >> place & 1 == 1
            }
        }
    }
    /// Every rule holds as it says.
    fn lawful(&self, bits: u64) -> bool {
        self.rules.iter().all(|(t, pre, post)| {
            let applies = self.truth(t, bits) && pre.iter().all(|p| self.truth(p, bits));
            !applies || post.iter().all(|q| self.truth(q, bits))
        })
    }
    /// `set` entails that `x` is `value`: in every lawful valuation where
    /// every taking of `set` holds.
    fn entails(&self, set: &BTreeSet<Taking>, x: &Prop, value: bool) -> bool {
        (0..1u64 << self.vars.len()).all(|bits| {
            let holds =
                self.lawful(bits) && set.iter().all(|t| self.truth(&t.prop, bits) == t.value);
            !holds || self.truth(x, bits) == value
        })
    }
    fn sound(&self, o: &Opinion) -> bool {
        o.rests_for()
            .iter()
            .all(|s| self.entails(s, o.about(), true))
            && o.rests_against()
                .iter()
                .all(|s| self.entails(s, o.about(), false))
    }
}

fn subexpressions(e: &E) -> Vec<&E> {
    let mut all = vec![e];
    if let Expression::Apply(_, args) = e {
        args.iter()
            .for_each(|a| all.extend(subexpressions(&a.body)));
    }
    all
}

fn opinion(v: Result<Value, Fault<std::convert::Infallible>>) -> Opinion {
    match v {
        Ok(Value::Opinion(o)) => o,
        other => panic!("not an opinion: {other:?}"),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3000))]

    /// S1, S2, S4: a checked expression gives a value of its type, never
    /// stuck, and the same each time. Judgement has no fault.
    #[test]
    fn judging_preserves_progresses_and_is_determined(seed in any::<u64>(), depth in 1u32..6) {
        let mut w = World::new(seed);
        let (t, e) = w.typed(depth);
        let j = w.judgement.clone();
        prop_assert_eq!(check(&j, &w.inputs, &e), Some(t.clone()));
        prop_assert!(fits(&j, &w.inputs, &w.values));
        let out = eval(&j, &w.values, &e);
        prop_assert!(matches!(&out, Ok(v) if j.has_type(v, &t)), "{:?} gave {:?}", e, out);
        prop_assert_eq!(out, eval(&j, &w.values, &e));
    }

    /// J1: every set every opinion rests on, in every step of a random
    /// derivation, entails what it supports.
    #[test]
    fn every_rule_preserves_truth(seed in any::<u64>(), depth in 1u32..6) {
        let mut w = World::new(seed);
        let (_, e) = w.typed(depth);
        let meaning = Meaning::of(&w.props, &w.defined);
        let j = w.judgement.clone();
        let written = inline(&e, &is_let);
        for sub in subexpressions(&written) {
            if let Ok(Value::Opinion(o)) = eval(&j, &w.values, sub) {
                prop_assert!(meaning.sound(&o), "unsound: {:?}\nfrom {:?}", o, sub);
            }
        }
    }

    /// J2: the state of `!x` and of a join is read from the states.
    /// Join is a semilattice.
    #[test]
    fn states_follow_from_rests(seed in any::<u64>(), depth in 0u32..5) {
        let mut w = World::new(seed);
        let x = w.prop();
        let j = w.judgement.clone();
        let (e1, e2, e3) = (w.opinion(&x, depth), w.opinion(&x, depth), w.opinion(&x, depth));
        let (a, b, c) = (opinion(eval(&j, &w.values, &e1)), opinion(eval(&j, &w.values, &e2)), opinion(eval(&j, &w.values, &e3)));
        let o = |v: &Opinion| Ok(Value::Opinion(v.clone()));
        let not_a = opinion(j.apply(&Op::Not, &[o(&a)]));
        prop_assert_eq!(not_a.state(), !a.state());
        prop_assert_eq!(not_a.about(), &x.not());
        let ab = opinion(j.apply(&Op::Join, &[o(&a), o(&b)]));
        prop_assert_eq!(ab.state(), a.state().join(b.state()));
        prop_assert_eq!(&ab, &opinion(j.apply(&Op::Join, &[o(&b), o(&a)])));
        prop_assert_eq!(&a, &opinion(j.apply(&Op::Join, &[o(&a), o(&a)])));
        let ab_c = opinion(j.apply(&Op::Join, &[o(&ab), o(&c)]));
        let bc = opinion(j.apply(&Op::Join, &[o(&b), o(&c)]));
        prop_assert_eq!(ab_c, opinion(j.apply(&Op::Join, &[o(&a), o(&bc)])));
        prop_assert_eq!(a.state(), State::told(!a.rests_for().is_empty(), !a.rests_against().is_empty()));
    }

    /// J3: rests are the smallest sets of takings a derivation reads, and
    /// all of them. No set holds another. Keeping only the takings in a
    /// set `S`, a proposition still follows true (false) only when one of its
    /// rests for (against) lies in `S`, unless a reductio lies below
    /// another (`a_reductio_below_another`); and, with no reductio,
    /// whenever one does. A reductio's rest leaves out the taking it
    /// finds wrong, which its derivation still reads.
    #[test]
    fn rests_are_the_smallest_and_all(seed in any::<u64>(), depth in 1u32..6) {
        let mut w = World::new(seed);
        let x = w.prop();
        let e = w.opinion(&x, depth);
        let j = w.judgement.clone();
        let written = inline(&e, &is_let);
        for sub in subexpressions(&written) {
            if let Ok(Value::Opinion(o)) = eval(&j, &w.values, sub) {
                for rests in [o.rests_for(), o.rests_against()] {
                    let held = rests.iter().any(|s| rests.iter().any(|r| r != s && r.is_subset(s)));
                    prop_assert!(!held, "a rest holds another: {:?}\nfrom {:?}", o, sub);
                }
            }
        }
        let whole = opinion(eval(&j, &w.values, &e));
        let takings: Vec<Taking> = w.values.values().flat_map(|v| match v {
            Value::Opinion(o) => o.rests_for().iter().chain(o.rests_against()).flatten().cloned().collect(),
            Value::State(_) => vec![],
        }).collect();
        let reductio = |s: &&E| matches!(s, Expression::Apply(Op::Reductio { .. }, _));
        let exact = !subexpressions(&written).iter().any(reductio);
        let nested = subexpressions(&written).iter().any(|s| match s {
            Expression::Apply(Op::Reductio { .. }, args) => args.iter().any(|a| subexpressions(&a.body).iter().any(reductio)),
            _ => false,
        });
        for _ in 0..8 {
            let kept: BTreeSet<Taking> = takings.iter().filter(|_| w.rng.coin()).cloned().collect();
            let values: BTreeMap<String, Value> = w.values.iter().map(|(name, v)| (name.clone(), match v {
                Value::Opinion(o) => {
                    let left = kept.iter().filter(|t| &t.prop == o.about()).cloned().collect();
                    Value::Opinion(Opinion::taken(o.about().clone(), left).unwrap())
                }
                state => state.clone(),
            })).collect();
            let part = opinion(eval(&j, &values, &e));
            for (some, all) in [(part.rests_for(), whole.rests_for()), (part.rests_against(), whole.rests_against())] {
                let within = all.iter().any(|r| r.is_subset(&kept));
                prop_assert!(nested || some.is_empty() || within, "follows from {:?} but no rest of {:?} lies in it", kept, all);
                prop_assert!(!exact || !within || !some.is_empty(), "a rest of {:?} lies in {:?}, yet nothing follows", all, kept);
            }
        }
    }

    /// S6: a let shares. Judging gives what the derivation with every let
    /// written out gives, and that checks to the same type, checked or
    /// not.
    #[test]
    fn a_let_shares(seed in any::<u64>(), depth in 1u32..6) {
        let mut w = World::new(seed);
        let (t, e) = w.typed(depth);
        let written = inline(&e, &is_let);
        let j = w.judgement.clone();
        prop_assert_eq!(check(&j, &w.inputs, &written), Some(t));
        prop_assert_eq!(eval(&j, &w.values, &e), eval(&j, &w.values, &written));
        let wild = w.wild(depth);
        prop_assert_eq!(eval(&j, &w.values, &wild), eval(&j, &w.values, &inline(&wild, &is_let)));
    }

    /// S3: no expression makes `check` or `eval` panic.
    #[test]
    fn nothing_panics(seed in any::<u64>(), depth in 0u32..6) {
        let mut w = World::new(seed);
        let e = w.wild(depth);
        let j = w.judgement.clone();
        let _ = check(&j, &w.inputs, &e);
        let _ = eval(&j, &w.values, &e);
    }
}

/// Belnap's four values: `and`, `not` and `join`, every case.
#[test]
fn belnap() {
    use State::*;
    let all = [Unknown, True, False, Conflict];
    for a in all {
        assert_eq!(!!a, a);
        assert_eq!(a.join(Unknown), a);
        assert_eq!(a.and(True), a);
        for b in all {
            assert_eq!(a.and(b), b.and(a));
            assert_eq!(a.join(b), b.join(a));
            // De Morgan, with `or` from the sides: told true when either is,
            // told false when both are.
            let or = |x: State, y: State| {
                let ((xt, xf), (yt, yf)) = (x.sides(), y.sides());
                State::told(xt || yt, xf && yf)
            };
            assert_eq!(!a.and(b), or(!a, !b));
            // `and` is the meet of the truth order, `join` of knowledge.
            assert_eq!(a.and(a.join(b)) == a, a.join(b).and(a) == a);
        }
    }
    assert_eq!(True.and(False), False);
    assert_eq!(Unknown.and(False), False);
    assert_eq!(Unknown.and(True), Unknown);
    assert_eq!(Conflict.and(Unknown), False);
    assert_eq!(True.join(False), Conflict);
}

fn take(x: &Prop, value: bool, count: u64, by: &str) -> Taking {
    Taking {
        prop: x.clone(),
        value,
        count,
        by: by.to_string(),
    }
}

/// The proposition `t = {p} s {q}` walked through: each of `p`, `t` and `q`
/// has a state and an opinion; ponens, tollens and reductio.
#[test]
fn a_proposition_walked_through() {
    let (p, q) = (Prop::named("p"), Prop::named("q"));
    let t = Prop::Implies {
        pre: vec![p.clone()],
        subject: "s".to_string(),
        post: vec![q.clone()],
    };
    let (test, review, run) = (
        take(&p, true, 3, "test"),
        take(&t, true, 1, "review"),
        take(&q, false, 2, "run"),
    );
    let j = Judgement::default();
    let inputs = BTreeMap::from([
        ("p".to_string(), Type::Opinion(p.clone())),
        ("t".to_string(), Type::Opinion(t.clone())),
        ("q".to_string(), Type::Opinion(q.clone())),
    ]);
    let values: BTreeMap<String, Value> = [
        ("p", Opinion::taken(p.clone(), vec![test.clone()])),
        ("t", Opinion::taken(t.clone(), vec![review.clone()])),
        ("q", Opinion::taken(q.clone(), vec![run.clone()])),
    ]
    .into_iter()
    .map(|(n, o)| (n.to_string(), Value::Opinion(o.unwrap())))
    .collect();
    let run_e = |e: &E| {
        assert!(check(&j, &inputs, e).is_some(), "does not check: {e:?}");
        opinion(eval(&j, &values, e))
    };
    let set = |ts: &[&Taking]| BTreeSet::from_iter(ts.iter().map(|t| (*t).clone()));

    // Ponens: q follows true from the test of p and the review of t.
    let ponens = apply(
        Op::Ponens {
            t: t.clone(),
            post: 0,
        },
        vec![input("t"), input("p")],
    );
    let q_derived = run_e(&ponens);
    assert_eq!(q_derived.state(), State::True);
    assert_eq!(
        q_derived.rests_for(),
        &BTreeSet::from([set(&[&test, &review])])
    );
    assert!(q_derived.rests_against().is_empty());

    // The run says q is false: q is in conflict.
    let q_all = apply(Op::Join, vec![ponens.clone(), input("q")]);
    assert_eq!(run_e(&q_all).state(), State::Conflict);

    // Tollens blames p, given t; or t, given p.
    let blame_p = apply(
        Op::Tollens {
            t: t.clone(),
            post: 0,
            blame: 1,
        },
        vec![input("q"), input("t")],
    );
    assert_eq!(
        run_e(&blame_p).rests_against(),
        &BTreeSet::from([set(&[&run, &review])])
    );
    let blame_t = apply(
        Op::Tollens {
            t: t.clone(),
            post: 0,
            blame: 0,
        },
        vec![input("q"), input("p")],
    );
    assert_eq!(
        run_e(&blame_t).rests_against(),
        &BTreeSet::from([set(&[&run, &test])])
    );

    // Reductio: the conflict on q says the test of p is wrong if the
    // review and the run hold.
    let reductio = apply(Op::Reductio { of: p.clone() }, vec![q_all]);
    let against_p = run_e(&reductio);
    assert_eq!(
        against_p.rests_against(),
        &BTreeSet::from([set(&[&review, &run])])
    );
    assert_eq!(against_p.state(), State::False);
    let p_all = apply(Op::Join, vec![input("p"), reductio]);
    assert_eq!(run_e(&p_all).state(), State::Conflict);
    let of_p = apply(Op::StateOf, vec![p_all]);
    assert_eq!(eval(&j, &values, &of_p), Ok(Value::State(State::Conflict)));
}

/// The type of a rule is the rule: ponens on the wrong premise, or on
/// another implication of the same subject, does not check.
#[test]
fn rules_check_their_propositions() {
    let (p, q, r) = (Prop::named("p"), Prop::named("q"), Prop::named("r"));
    let t = Prop::Implies {
        pre: vec![p.clone()],
        subject: "s".to_string(),
        post: vec![q.clone()],
    };
    let other = Prop::Implies {
        pre: vec![p.clone()],
        subject: "s".to_string(),
        post: vec![r.clone()],
    };
    let j = Judgement::default();
    let inputs: BTreeMap<String, Type> = [("p", &p), ("q", &q), ("t", &t), ("other", &other)]
        .into_iter()
        .map(|(n, x)| (n.to_string(), Type::Opinion(x.clone())))
        .collect();
    let ponens = |args: &[&str]| {
        apply(
            Op::Ponens {
                t: t.clone(),
                post: 0,
            },
            args.iter().map(|a| input(a)).collect(),
        )
    };
    assert_eq!(
        check(&j, &inputs, &ponens(&["t", "p"])),
        Some(Type::Opinion(q.clone()))
    );
    assert_eq!(check(&j, &inputs, &ponens(&["t", "q"])), None);
    assert_eq!(check(&j, &inputs, &ponens(&["other", "p"])), None);
    assert_eq!(check(&j, &inputs, &ponens(&["p", "t"])), None);
    let out_of_range = apply(
        Op::Ponens {
            t: t.clone(),
            post: 1,
        },
        vec![input("t"), input("p")],
    );
    assert_eq!(check(&j, &inputs, &out_of_range), None);
    let join = apply(Op::Join, vec![input("p"), input("q")]);
    assert_eq!(check(&j, &inputs, &join), None);
}

/// foundation2's red team: `{} d {q}` gave `q` from nothing. Here `t`
/// is a premise: untaken, it gives nothing; taken false, nothing either.
#[test]
fn nothing_follows_from_nothing() {
    let q = Prop::named("q");
    let t = Prop::Implies {
        pre: vec![],
        subject: "d".to_string(),
        post: vec![q.clone()],
    };
    let j = Judgement::default();
    let ponens = apply(
        Op::Ponens {
            t: t.clone(),
            post: 0,
        },
        vec![input("t")],
    );
    for takings in [vec![], vec![take(&t, false, 100, "x")]] {
        let values = BTreeMap::from([(
            "t".to_string(),
            Value::Opinion(Opinion::taken(t.clone(), takings).unwrap()),
        )]);
        assert_eq!(opinion(eval(&j, &values, &ponens)).state(), State::Unknown);
    }
}

/// Counts never decide a state: a taking of count 0 holds a proposition as
/// firmly, for the logic, as one of count 10. Two takings of p are two
/// ways for it; through t, every way needs the review.
#[test]
fn counts_never_decide() {
    let (p, q) = (Prop::named("p"), Prop::named("q"));
    let t = Prop::Implies {
        pre: vec![p.clone()],
        subject: "s".to_string(),
        post: vec![q.clone()],
    };
    let ponens = apply(
        Op::Ponens {
            t: t.clone(),
            post: 0,
        },
        vec![input("t"), input("p")],
    );
    let (v1, v2) = (take(&p, true, 3, "v1"), take(&p, true, 5, "v2"));
    let at = |count_t: u64| -> BTreeMap<String, Value> {
        BTreeMap::from([
            (
                "t".to_string(),
                Value::Opinion(
                    Opinion::taken(t.clone(), vec![take(&t, true, count_t, "review")]).unwrap(),
                ),
            ),
            (
                "p".to_string(),
                Value::Opinion(Opinion::taken(p.clone(), vec![v1.clone(), v2.clone()]).unwrap()),
            ),
        ])
    };
    let j = Judgement::default();
    let states: Vec<State> = [0, 1, 10]
        .into_iter()
        .map(|count| opinion(eval(&j, &at(count), &ponens)).state())
        .collect();
    assert_eq!(states, vec![State::True; 3]);
    let review = take(&t, true, 1, "review");
    let q_from = opinion(eval(&j, &at(1), &ponens));
    assert_eq!(
        q_from.rests_for(),
        &BTreeSet::from([
            BTreeSet::from([review.clone(), v1]),
            BTreeSet::from([review, v2])
        ])
    );
}

/// An implication that cannot hold is refuted by logic alone: its
/// opinion follows false from no taking.
#[test]
fn an_impossible_implication_is_refuted() {
    let a = Prop::named("a");
    let t = Prop::Implies {
        pre: vec![],
        subject: "s".to_string(),
        post: vec![a.clone(), a.not()],
    };
    let j = Judgement::default();
    let values = BTreeMap::from([(
        "t".to_string(),
        Value::Opinion(Opinion::taken(t.clone(), vec![take(&t, true, 1, "review")]).unwrap()),
    )]);
    let yes_a = apply(
        Op::Ponens {
            t: t.clone(),
            post: 0,
        },
        vec![input("t")],
    );
    let no_a = apply(
        Op::Not,
        vec![apply(
            Op::Ponens {
                t: t.clone(),
                post: 1,
            },
            vec![input("t")],
        )],
    );
    let both = apply(Op::Join, vec![yes_a, no_a]);
    assert_eq!(opinion(eval(&j, &values, &both)).state(), State::Conflict);
    let refuted = opinion(eval(
        &j,
        &values,
        &apply(Op::Reductio { of: t.clone() }, vec![both]),
    ));
    assert_eq!(refuted.rests_against(), &BTreeSet::from([BTreeSet::new()]));
    assert_eq!(refuted.state(), State::False);
    assert!(Meaning::of(&[t], &BTreeMap::new()).sound(&refuted));
}

/// A leaf opinion is of the takings of its own proposition only.
#[test]
fn a_leaf_is_its_own_takings() {
    let (p, q) = (Prop::named("p"), Prop::named("q"));
    assert!(Opinion::taken(p.clone(), vec![take(&q, true, 1, "v")]).is_none());
    let o = Opinion::taken(p.clone(), vec![]).unwrap();
    assert_eq!(o.state(), State::Unknown);
    assert!(!Judgement::default().has_type(&Value::Opinion(o), &Type::Opinion(q)));
}

/// Red team, round 1: one ponens over twelve premises, each taken by two
/// voices, rests on 4,096 sets. That is what rests are, and it must come
/// quickly: premises that share no taking are not compared.
#[test]
fn a_wide_ponens_is_quick() {
    let pre: Vec<Prop> = (0..11).map(|i| Prop::named(&format!("p{i}"))).collect();
    let t = Prop::Implies {
        pre,
        subject: "s".to_string(),
        post: vec![Prop::named("q")],
    };
    let premises = Judgement::default().premises(&t).unwrap();
    let (mut inputs, mut values, mut args) = (BTreeMap::new(), BTreeMap::new(), vec![]);
    for (i, x) in premises.iter().enumerate() {
        let name = format!("x{i}");
        let takings = vec![take(x, true, 1, "u"), take(x, true, 1, "v")];
        inputs.insert(name.clone(), Type::Opinion(x.clone()));
        values.insert(
            name.clone(),
            Value::Opinion(Opinion::taken(x.clone(), takings).unwrap()),
        );
        args.push(input(&name));
    }
    let e = apply(
        Op::Ponens {
            t: t.clone(),
            post: 0,
        },
        args,
    );
    assert!(check(&Judgement::default(), &inputs, &e).is_some());
    let start = std::time::Instant::now();
    let q = opinion(eval(&Judgement::default(), &values, &e));
    assert_eq!(q.rests_for().len(), 1 << 12);
    assert!(
        start.elapsed() < std::time::Duration::from_secs(2),
        "{:?}",
        start.elapsed()
    );
}

/// Red team, round 1: a leaf taken 20,000 times. Its opinion and the
/// rules over it neither recurse per taking nor slow down.
#[test]
fn a_much_taken_leaf() {
    let (p, q) = (Prop::named("p"), Prop::named("q"));
    let t = Prop::Implies {
        pre: vec![p.clone()],
        subject: "s".to_string(),
        post: vec![q.clone()],
    };
    let many = (0..20_000)
        .map(|i| take(&p, true, 1, &format!("run{i}")))
        .collect();
    let values = BTreeMap::from([
        (
            "p".to_string(),
            Value::Opinion(Opinion::taken(p.clone(), many).unwrap()),
        ),
        (
            "t".to_string(),
            Value::Opinion(Opinion::taken(t.clone(), vec![take(&t, true, 1, "review")]).unwrap()),
        ),
    ]);
    let e = apply(
        Op::Join,
        vec![
            input("p"),
            apply(Op::Not, vec![apply(Op::Not, vec![input("p")])]),
        ],
    );
    let joined = std::thread::spawn(move || {
        let ponens = apply(Op::Ponens { t, post: 0 }, vec![input("t"), input("p")]);
        (
            opinion(eval(&Judgement::default(), &values, &e)),
            opinion(eval(&Judgement::default(), &values, &ponens)),
        )
    })
    .join()
    .unwrap();
    assert_eq!(joined.0.rests_for().len(), 20_000);
    assert_eq!(joined.1.rests_for().len(), 20_000);
}

/// Red team, round 2: below another reductio, a taking added can take a
/// conclusion away. The inner reductio keeps only its smallest blame,
/// and the outer one needed a larger one, which held a taking of `q`.
/// What is lost rested on takings that cannot all hold.
#[test]
fn a_reductio_below_another() {
    let (z, q, w) = (Prop::named("z"), Prop::named("q"), Prop::named("w"));
    let t7 = Prop::Implies {
        pre: vec![z.not()],
        subject: "s7".to_string(),
        post: vec![w.not()],
    };
    let t9 = Prop::Implies {
        pre: vec![z.not(), q.clone()],
        subject: "s9".to_string(),
        post: vec![z.not()],
    };
    let (u, u2, v) = (
        take(&z, false, 1, "u"),
        take(&z, false, 1, "u2"),
        take(&z, false, 1, "v"),
    );
    let (qt, t7t, t9t, wt) = (
        take(&q, true, 1, "qt"),
        take(&t7, true, 1, "t7"),
        take(&t9, true, 1, "t9"),
        take(&w, true, 1, "wt"),
    );
    let leaves = [
        ("zu", &z, &u),
        ("zu2", &z, &u2),
        ("zv", &z, &v),
        ("q", &q, &qt),
        ("t7", &t7, &t7t),
        ("t9", &t9, &t9t),
        ("w", &w, &wt),
    ];
    let values = |without: Option<&Taking>| -> BTreeMap<String, Value> {
        leaves
            .iter()
            .map(|(name, x, taking)| {
                let takings = match Some(*taking) == without {
                    true => vec![],
                    false => vec![(*taking).clone()],
                };
                let o = Opinion::taken((*x).clone(), takings).unwrap();
                (name.to_string(), Value::Opinion(o))
            })
            .collect()
    };
    let not_z = apply(
        Op::Join,
        vec![
            apply(Op::Not, vec![input("zu")]),
            apply(
                Op::Ponens {
                    t: t9.clone(),
                    post: 0,
                },
                vec![input("t9"), apply(Op::Not, vec![input("zu2")]), input("q")],
            ),
        ],
    );
    let not_w = apply(
        Op::Ponens {
            t: t7.clone(),
            post: 0,
        },
        vec![input("t7"), not_z],
    );
    let w_both = apply(Op::Join, vec![input("w"), apply(Op::Not, vec![not_w])]);
    let blame_z = apply(Op::Reductio { of: z.clone() }, vec![w_both]);
    let z_both = apply(Op::Join, vec![blame_z.clone(), input("zv")]);
    let e = apply(Op::Reductio { of: q.clone() }, vec![z_both]);
    let inputs = leaves
        .iter()
        .map(|(name, x, _)| (name.to_string(), Type::Opinion((*x).clone())))
        .collect();
    assert_eq!(
        check(&Judgement::default(), &inputs, &e),
        Some(Type::Opinion(q.clone()))
    );
    let all = opinion(eval(&Judgement::default(), &values(None), &e));
    let fewer = opinion(eval(&Judgement::default(), &values(Some(&u)), &e));
    // The inner reductio: z follows from {wt, t7} (blaming u) and from
    // {wt, t7, t9, qt} (blaming u2); only the smaller stays.
    let inner = opinion(eval(&Judgement::default(), &values(None), &blame_z));
    let smaller = BTreeSet::from([wt.clone(), t7t.clone()]);
    assert_eq!(inner.rests_for(), &BTreeSet::from([smaller]));
    assert_eq!(all.state(), State::Unknown);
    let lost = BTreeSet::from([wt.clone(), t7t.clone(), t9t, v.clone()]);
    assert_eq!(fewer.rests_against(), &BTreeSet::from([lost.clone()]));
    let meaning = Meaning::of(&[z, q.clone(), w, t7], &BTreeMap::new());
    assert!(meaning.entails(&lost, &q, false));
    let cannot = BTreeSet::from([wt, t7t, v]);
    assert!(meaning.entails(&cannot, &q, true) && meaning.entails(&cannot, &q, false));
}

/// S3's bound on depth: 1,000 levels, every other one a `let`, checked
/// and judged on a thread of 2 MiB.
#[test]
fn a_thousand_levels() {
    let p = Prop::named("p");
    let t = Prop::Implies {
        pre: vec![p.clone()],
        subject: "s".to_string(),
        post: vec![p.clone()],
    };
    let deep = std::thread::Builder::new()
        .stack_size(2 << 20)
        .spawn(move || {
            // S3: 1,000 levels in a debug build, 2,000 in a release build.
            let levels = if cfg!(debug_assertions) { 1000 } else { 2000 };
            let mut e = input("p");
            for level in 0..levels {
                e = match level % 2 {
                    0 => apply(
                        Op::Ponens {
                            t: t.clone(),
                            post: 0,
                        },
                        vec![input("t"), e],
                    ),
                    _ => let_in(Op::Let, "o", e, input("o")),
                };
            }
            let inputs = BTreeMap::from([
                ("p".to_string(), Type::Opinion(p.clone())),
                ("t".to_string(), Type::Opinion(t.clone())),
            ]);
            let values = BTreeMap::from([
                (
                    "p".to_string(),
                    Value::Opinion(
                        Opinion::taken(p.clone(), vec![take(&p, true, 1, "run")]).unwrap(),
                    ),
                ),
                (
                    "t".to_string(),
                    Value::Opinion(
                        Opinion::taken(t.clone(), vec![take(&t, true, 1, "review")]).unwrap(),
                    ),
                ),
            ]);
            let ty = check(&Judgement::default(), &inputs, &e);
            (
                ty,
                opinion(eval(&Judgement::default(), &values, &e)).state(),
            )
        })
        .unwrap()
        .join()
        .unwrap();
    assert_eq!(deep, (Some(Type::Opinion(Prop::named("p"))), State::True));
}

/// Red team, round 2: a premise whose one rest holds the takings of every
/// other premise keeps a ponens over sixteen premises, two ways each, to
/// one rest, whatever the order of its premises.
#[test]
fn a_premise_that_holds_the_others() {
    let ps: Vec<Prop> = (0..16).map(|i| Prop::named(&format!("p{i}"))).collect();
    let z = Prop::named("z");
    let tz = Prop::Implies {
        pre: [ps.clone(), ps.clone()].concat(),
        subject: "sz".to_string(),
        post: vec![z.clone()],
    };
    let t = Prop::Implies {
        pre: [ps.clone(), vec![z]].concat(),
        subject: "s".to_string(),
        post: vec![Prop::named("q")],
    };
    let mut values = BTreeMap::new();
    let mut leaf = |name: String, x: &Prop, by: &[&str]| {
        let takings = by.iter().map(|b| take(x, true, 1, b)).collect();
        let o = Opinion::taken(x.clone(), takings).unwrap();
        values.insert(name.clone(), Value::Opinion(o));
        input(&name)
    };
    let mut z_args = vec![leaf("tz".to_string(), &tz, &["a"])];
    for (i, x) in ps.iter().enumerate() {
        z_args.push(leaf(format!("a{i}"), x, &["a"]));
    }
    for (i, x) in ps.iter().enumerate() {
        z_args.push(leaf(format!("b{i}"), x, &["b"]));
    }
    let mut args = vec![leaf("t".to_string(), &t, &["a"])];
    for (i, x) in ps.iter().enumerate() {
        args.push(leaf(format!("ab{i}"), x, &["a", "b"]));
    }
    args.push(apply(Op::Ponens { t: tz, post: 0 }, z_args));
    let e = apply(Op::Ponens { t, post: 0 }, args);
    let start = std::time::Instant::now();
    let q = opinion(eval(&Judgement::default(), &values, &e));
    assert_eq!(q.rests_for().len(), 1);
    assert!(
        start.elapsed() < std::time::Duration::from_secs(2),
        "{:?}",
        start.elapsed()
    );
}

/// Judgement that knows these rules: a name, its pres and its posts.
fn knowing(rules: &[(&str, &[&Prop], &[&Prop])]) -> Result<Judgement, BTreeSet<String>> {
    let cloned = |ps: &[&Prop]| ps.iter().map(|p| (*p).clone()).collect();
    let rules = rules.iter().map(|(name, pre, post)| {
        let (pre, post) = (cloned(pre), cloned(post));
        (name.to_string(), Definition { pre, post })
    });
    Judgement::with(rules.collect())
}

/// The rule `r = {p} r {q}`, known by its name: modus ponens gives `q`,
/// and modus tollens blames `r` itself, so a run against `q` reaches
/// the review that took the rule. A name judgement does not know is no
/// rule.
#[test]
fn a_rule_known_by_its_name() {
    let (p, q, r) = (Prop::named("p"), Prop::named("q"), Prop::named("r"));
    let j = knowing(&[("r", &[&p], &[&q])]).unwrap();
    assert_eq!(j.premises(&r), Some(vec![r.clone(), p.clone()]));
    assert_eq!(j.posts(&r), Some(&[q.clone()][..]));
    assert_eq!(j.premises(&p), None);
    assert_eq!(j.premises(&r.not()), None);
    let (test, review, run) = (
        take(&p, true, 3, "test"),
        take(&r, true, 1, "review"),
        take(&q, false, 2, "run"),
    );
    let inputs = BTreeMap::from([
        ("p".to_string(), Type::Opinion(p.clone())),
        ("r".to_string(), Type::Opinion(r.clone())),
        ("q".to_string(), Type::Opinion(q.clone())),
    ]);
    let values: BTreeMap<String, Value> = [("p", &test), ("r", &review), ("q", &run)]
        .into_iter()
        .map(|(name, t)| {
            let o = Opinion::taken(t.prop.clone(), vec![t.clone()]).unwrap();
            (name.to_string(), Value::Opinion(o))
        })
        .collect();
    let ponens = apply(
        Op::Ponens {
            t: r.clone(),
            post: 0,
        },
        vec![input("r"), input("p")],
    );
    assert_eq!(check(&j, &inputs, &ponens), Some(Type::Opinion(q.clone())));
    let given = opinion(eval(&j, &values, &ponens));
    assert_eq!(given.state(), State::True);
    let rest = BTreeSet::from([review.clone(), test.clone()]);
    assert_eq!(given.rests_for(), &BTreeSet::from([rest]));
    let tollens = apply(
        Op::Tollens {
            t: r.clone(),
            post: 0,
            blame: 0,
        },
        vec![input("q"), input("p")],
    );
    assert_eq!(check(&j, &inputs, &tollens), Some(Type::Opinion(r.clone())));
    let blamed = opinion(eval(&j, &values, &tollens));
    assert_eq!(blamed.state(), State::False);
    let rest = BTreeSet::from([run.clone(), test.clone()]);
    assert_eq!(blamed.rests_against(), &BTreeSet::from([rest]));
    let defined = BTreeMap::from([(
        "r".to_string(),
        Definition {
            pre: vec![p.clone()],
            post: vec![q.clone()],
        },
    )]);
    let meaning = Meaning::of(&[p.clone(), q.clone(), r.clone()], &defined);
    assert!(meaning.sound(&given) && meaning.sound(&blamed));
    // Not known: no type, and stuck if run anyway.
    let none = Judgement::default();
    assert_eq!(check(&none, &inputs, &ponens), None);
    assert_eq!(eval(&none, &values, &ponens), Err(Fault::Stuck));
    let on_p = apply(
        Op::Ponens {
            t: p.clone(),
            post: 0,
        },
        vec![input("p")],
    );
    assert_eq!(check(&j, &inputs, &on_p), None);
    assert_eq!(eval(&j, &values, &on_p), Err(Fault::Stuck));
    // Nor is a negation, or a name judgement does not know.
    for t in [r.not(), Prop::named("s")] {
        let e = apply(Op::Ponens { t, post: 0 }, vec![input("r"), input("p")]);
        assert_eq!(check(&j, &inputs, &e), None);
        assert_eq!(eval(&j, &values, &e), Err(Fault::Stuck));
    }
    // A blame past the premises has no type.
    let past = Op::Tollens {
        t: r.clone(),
        post: 0,
        blame: 2,
    };
    let past = apply(past, vec![input("q"), input("p")]);
    assert_eq!(check(&j, &inputs, &past), None);
}

/// No rule is defined through itself: not by naming itself, nor
/// through other rules, a negation or an implication written in place,
/// in its pres or its posts. Only the names on a cycle are refused, not
/// those that name one.
#[test]
fn no_rule_is_defined_through_itself() {
    let named = Prop::named;
    let (p, q) = (named("p"), named("q"));
    let (a, b, c, d, e) = (named("a"), named("b"), named("c"), named("d"), named("e"));
    let refused = |names: &[&str]| Err(names.iter().map(|n| n.to_string()).collect());
    assert_eq!(knowing(&[("a", &[&a], &[&q])]), refused(&["a"]));
    assert_eq!(knowing(&[("a", &[&p], &[&a])]), refused(&["a"]));
    let not_a = a.not();
    let both = knowing(&[
        ("a", &[&b], &[&q]),
        ("b", &[&not_a], &[&q]),
        ("e", &[&a], &[&q]),
    ]);
    assert_eq!(both, refused(&["a", "b"]));
    let inside = Prop::Implies {
        pre: vec![d.clone()],
        subject: "s".to_string(),
        post: vec![q.clone()],
    };
    let through = knowing(&[("c", &[&p], &[&inside]), ("d", &[&c], &[&q])]);
    assert_eq!(through, refused(&["c", "d"]));
    // Names that only name rules, or are no rule, are fine.
    let fine = knowing(&[
        ("c", &[&d, &d], &[&q]),
        ("d", &[&e, &p], &[&q]),
        ("e", &[&p], &[&q]),
    ]);
    assert!(fine.is_ok());
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3000))]

    /// `with` refuses exactly the names on a cycle, on random tables,
    /// against reachability worked out by brute force: a name reaches
    /// the names its pres and posts mention, through negations and
    /// implications written in place, but not their subjects.
    #[test]
    fn with_refuses_exactly_the_names_on_a_cycle(seed in any::<u64>()) {
        let mut rng = Rng::new(seed);
        let n = 1 + rng.below(7);
        let name = |i: usize| format!("n{i}");
        let (mut edges, mut rules) = (vec![BTreeSet::new(); n], BTreeMap::new());
        for (i, out) in edges.iter_mut().enumerate() {
            let (mut pre, mut post) = (Vec::new(), Vec::new());
            for _ in 0..rng.below(3) {
                // `n` and `n + 1` name no rule.
                let to = rng.below(n + 2);
                if to < n {
                    out.insert(to);
                }
                let x = match rng.below(4) {
                    0 => Prop::named(&name(to)).not(),
                    1 => Prop::Implies {
                        pre: vec![Prop::named(&name(to))],
                        subject: name(rng.below(n)),
                        post: vec![Prop::named("q")],
                    },
                    _ => Prop::named(&name(to)),
                };
                match rng.coin() {
                    true => pre.push(x),
                    false => post.push(x),
                }
            }
            rules.insert(name(i), Definition { pre, post });
        }
        // `i` reaches itself in one step or more.
        let back = |i: usize| {
            let (mut seen, mut todo) = (BTreeSet::new(), vec![i]);
            while let Some(k) = todo.pop() {
                for &to in &edges[k] {
                    if seen.insert(to) {
                        todo.push(to);
                    }
                }
            }
            seen.contains(&i)
        };
        let cyclic: BTreeSet<String> = (0..n).filter(|&i| back(i)).map(name).collect();
        match Judgement::with(rules) {
            Ok(_) => prop_assert!(cyclic.is_empty()),
            Err(refused) => prop_assert_eq!(refused, cyclic),
        }
    }
}

/// A chain of 100,000 rules, each named by the next, is known at once,
/// and closed on itself every name on it is refused, without running
/// out of the call stack.
#[test]
fn a_long_chain_of_names() {
    let n = 100_000;
    let name = |i: usize| format!("r{i}");
    let chain = |closed: bool| {
        let rules = (0..n).map(|i| {
            let before = match (i, closed) {
                (0, false) => Prop::named("p"),
                (0, true) => Prop::named(&name(n - 1)),
                (i, _) => Prop::named(&name(i - 1)),
            };
            let post = vec![Prop::named("q")];
            (
                name(i),
                Definition {
                    pre: vec![before],
                    post,
                },
            )
        });
        Judgement::with(rules.collect())
    };
    let start = std::time::Instant::now();
    assert!(chain(false).is_ok());
    assert_eq!(chain(true).unwrap_err().len(), n);
    assert!(start.elapsed() < std::time::Duration::from_secs(5));
}
