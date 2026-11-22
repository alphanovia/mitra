//! The judgement standard (`docs/core2.md`): S2 to S4, J1 (every rule
//! preserves truth, checked on every valuation) and J2 (states are read
//! from rests) on random derivations, and the piece walked through.

mod common;

use common::Rng;
use mitra_core::expression::{check, eval, fits, Expression, Fault, Reading};
use mitra_core::judgement::{Judgement, Op, Opinion, Piece, State, Taking, Type, Value};
use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

type E = Expression<Op>;

fn apply(op: Op, args: Vec<E>) -> E {
    Expression::Apply(op, args)
}
fn input(name: &str) -> E {
    Expression::Input(name.to_string())
}

/// A random model: three named pieces and their negations, one to three
/// implications over them (an implication may be about an earlier one),
/// and a leaf opinion of every piece from random takings.
struct World {
    rng: Rng,
    pieces: Vec<Piece>,
    steps: Vec<Piece>,
    names: BTreeMap<Piece, String>,
    inputs: BTreeMap<String, Type>,
    values: BTreeMap<String, Value>,
}

impl World {
    fn new(seed: u64) -> World {
        let mut rng = Rng::new(seed);
        let atoms: Vec<Piece> = ["a", "b", "c"].map(Piece::named).to_vec();
        let mut pieces: Vec<Piece> = atoms
            .iter()
            .cloned()
            .chain(atoms.iter().map(Piece::not))
            .collect();
        let mut steps: Vec<Piece> = Vec::new();
        for i in 0..1 + rng.below(3) {
            let pool: Vec<Piece> = pieces.iter().chain(&steps).cloned().collect();
            let pre = (0..rng.below(3)).map(|_| rng.pick(&pool).clone()).collect();
            let post = (0..1 + rng.below(2))
                .map(|_| rng.pick(&pool).clone())
                .collect();
            steps.push(Piece::Implies {
                pre,
                subject: format!("s{i}"),
                post,
            });
        }
        pieces.extend(steps.iter().flat_map(|t| [t.clone(), t.not()]));
        let mut w = World {
            rng,
            pieces: pieces.clone(),
            steps,
            names: BTreeMap::new(),
            inputs: BTreeMap::new(),
            values: BTreeMap::new(),
        };
        for (i, piece) in pieces.iter().enumerate() {
            let name = format!("o{i}");
            let takings = (0..w.rng.below(4)).map(|_| w.taking(piece)).collect();
            let opinion = Opinion::taken(piece.clone(), takings).unwrap();
            w.names.insert(piece.clone(), name.clone());
            w.inputs.insert(name.clone(), Type::Opinion(piece.clone()));
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
    fn taking(&mut self, piece: &Piece) -> Taking {
        Taking {
            piece: piece.clone(),
            value: self.rng.below(3) > 0,
            count: 1 + self.rng.below(3) as u64,
            by: self.rng.pick(&["v1", "v2"]).to_string(),
        }
    }
    fn piece(&mut self) -> Piece {
        self.rng.pick(&self.pieces.clone()).clone()
    }
    /// A random derivation of an opinion of `x`.
    fn opinion(&mut self, x: &Piece, depth: u32) -> E {
        if depth == 0 || self.rng.below(4) == 0 {
            return Expression::Input(self.names[x].clone());
        }
        let d = depth - 1;
        let mut ways = vec![Op::Join, Op::Not, Op::Reductio { of: x.clone() }];
        for t in &self.steps {
            let (premises, posts) = (t.premises().unwrap(), t.posts().unwrap());
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
                // of an implication x is a premise of, or any piece.
                let posts: Vec<Piece> = self
                    .steps
                    .iter()
                    .filter(|t| t.premises().unwrap().contains(x))
                    .flat_map(|t| t.posts().unwrap().to_vec())
                    .collect();
                let y = match self.rng.below(3) {
                    0 => x.clone(),
                    1 if !posts.is_empty() => self.rng.pick(&posts).clone(),
                    _ => self.piece(),
                };
                vec![self.opinion(&y, d)]
            }
            Op::Ponens { t, .. } => t
                .premises()
                .unwrap()
                .iter()
                .map(|p| self.opinion(p, d))
                .collect(),
            Op::Tollens { t, post, blame } => {
                let mut from = t.premises().unwrap();
                from.remove(*blame);
                let q = t.posts().unwrap()[*post].clone();
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
            return input(self.rng.pick(&["s0", "s1"]));
        }
        let d = depth - 1;
        match self.rng.below(4) {
            0 => {
                let y = self.piece();
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
                let x = self.piece();
                (Type::Opinion(x.clone()), self.opinion(&x, depth))
            }
        }
    }
    /// Any operation, with any parameters.
    fn wild_op(&mut self) -> Op {
        let t = self.piece();
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
        ];
        self.rng.pick(&all).clone()
    }
    /// Any expression, typed or not.
    fn wild(&mut self, depth: u32) -> E {
        if depth == 0 || self.rng.below(4) == 0 {
            return Expression::Input(format!("o{}", self.rng.below(self.pieces.len() + 2)));
        }
        let op = self.wild_op();
        let args = (0..self.rng.below(5))
            .map(|_| self.wild(depth - 1))
            .collect();
        apply(op, args)
    }
}

/// The meaning of pieces, for J1: a valuation gives a truth to every
/// named piece and every implication, and an implication `t` that is
/// true makes its posts true when its pres are.
struct Meaning {
    vars: Vec<Piece>,
}

impl Meaning {
    fn of(pieces: &[Piece]) -> Meaning {
        fn collect(p: &Piece, vars: &mut BTreeSet<Piece>) {
            match p {
                Piece::Named(_) => {
                    vars.insert(p.clone());
                }
                Piece::Not(x) => collect(x, vars),
                Piece::Implies { pre, post, .. } => {
                    vars.insert(p.clone());
                    pre.iter().chain(post).for_each(|q| collect(q, vars));
                }
            }
        }
        let mut vars = BTreeSet::new();
        pieces.iter().for_each(|p| collect(p, &mut vars));
        Meaning {
            vars: vars.into_iter().collect(),
        }
    }
    fn truth(&self, p: &Piece, bits: u64) -> bool {
        match p {
            Piece::Not(x) => !self.truth(x, bits),
            _ => {
                let place = self
                    .vars
                    .iter()
                    .position(|v| v == p)
                    .expect("a piece outside the world");
                bits >> place & 1 == 1
            }
        }
    }
    /// Every implication holds as it says.
    fn lawful(&self, bits: u64) -> bool {
        self.vars.iter().all(|t| match t {
            Piece::Implies { pre, post, .. } => {
                let applies = self.truth(t, bits) && pre.iter().all(|p| self.truth(p, bits));
                !applies || post.iter().all(|q| self.truth(q, bits))
            }
            _ => true,
        })
    }
    /// `set` entails that `x` is `value`: in every lawful valuation where
    /// every taking of `set` holds.
    fn entails(&self, set: &BTreeSet<Taking>, x: &Piece, value: bool) -> bool {
        (0..1u64 << self.vars.len()).all(|bits| {
            let holds =
                self.lawful(bits) && set.iter().all(|t| self.truth(&t.piece, bits) == t.value);
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
        args.iter().for_each(|a| all.extend(subexpressions(a)));
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
        let j = Judgement::default();
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
        let meaning = Meaning::of(&w.pieces);
        let j = Judgement::default();
        for sub in subexpressions(&e) {
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
        let x = w.piece();
        let j = Judgement::default();
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

    /// S3: no expression makes `check` or `eval` panic.
    #[test]
    fn nothing_panics(seed in any::<u64>(), depth in 0u32..6) {
        let mut w = World::new(seed);
        let e = w.wild(depth);
        let j = Judgement::default();
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

fn take(x: &Piece, value: bool, count: u64, by: &str) -> Taking {
    Taking {
        piece: x.clone(),
        value,
        count,
        by: by.to_string(),
    }
}

/// The piece `t = {p} s {q}` walked through: each of `p`, `t` and `q`
/// has a state and an opinion; ponens, tollens and reductio.
#[test]
fn a_piece_walked_through() {
    let (p, q) = (Piece::named("p"), Piece::named("q"));
    let t = Piece::Implies {
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
    assert_eq!((q_derived.yes(), q_derived.no()), (Some(1), Some(0)));

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
fn rules_check_their_pieces() {
    let (p, q, r) = (Piece::named("p"), Piece::named("q"), Piece::named("r"));
    let t = Piece::Implies {
        pre: vec![p.clone()],
        subject: "s".to_string(),
        post: vec![q.clone()],
    };
    let other = Piece::Implies {
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
    let q = Piece::named("q");
    let t = Piece::Implies {
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

/// Counts never decide a state, and the bias is read by no rule.
#[test]
fn numbers_never_decide() {
    let (p, q) = (Piece::named("p"), Piece::named("q"));
    let t = Piece::Implies {
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
                Value::Opinion(
                    Opinion::taken(
                        p.clone(),
                        vec![take(&p, true, 3, "v1"), take(&p, true, 5, "v2")],
                    )
                    .unwrap(),
                ),
            ),
        ])
    };
    let plain = Judgement::default();
    let biased = Judgement {
        bias: BTreeMap::from([(q.clone(), 0.01), (t.clone(), 0.99)]),
    };
    for count in [0, 1, 10] {
        let (a, b) = (
            eval(&plain, &at(count), &ponens),
            eval(&biased, &at(count), &ponens),
        );
        assert_eq!(a, b);
        assert_eq!(opinion(a).state(), State::True);
    }
    // Two takings of p add: 3 + 5. Through t, the least to give up.
    let p_alone = opinion(eval(&plain, &at(1), &input("p")));
    assert_eq!(p_alone.yes(), Some(8));
    assert_eq!(opinion(eval(&plain, &at(1), &ponens)).yes(), Some(1));
    assert_eq!(opinion(eval(&plain, &at(10), &ponens)).yes(), Some(8));
}

/// An implication that cannot hold is refuted by logic alone: its
/// opinion follows false from no taking.
#[test]
fn an_impossible_implication_is_refuted() {
    let a = Piece::named("a");
    let t = Piece::Implies {
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
    assert_eq!((refuted.state(), refuted.no()), (State::False, None));
    assert!(Meaning::of(&[t]).sound(&refuted));
}

/// A leaf opinion is of the takings of its own piece only.
#[test]
fn a_leaf_is_its_own_takings() {
    let (p, q) = (Piece::named("p"), Piece::named("q"));
    assert!(Opinion::taken(p.clone(), vec![take(&q, true, 1, "v")]).is_none());
    let o = Opinion::taken(p.clone(), vec![]).unwrap();
    assert_eq!(
        (o.state(), o.yes(), o.no()),
        (State::Unknown, Some(0), Some(0))
    );
    assert!(!Judgement::default().has_type(&Value::Opinion(o), &Type::Opinion(q)));
}
