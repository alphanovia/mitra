//! Quantifiers over every kind of set, by the red team of round 6: a
//! set's points are what `contains` holds, in Rust's order of tuples;
//! sets are ordered as they are equal; random and ill-typed expressions
//! over any sets never panic and make no opinion of a proposition that is
//! not well formed; and J1 holds over sets of several ranges, of none,
//! and with conditions, by the test's own listing and substitution.

mod common;

use common::{apply, let_in, Rng};
use mitra_core::expression::{check, eval, Arg, Expression};
use mitra_core::judgement::{Judgement, Op, Opinion, Prop, Set, Taking, Term, Type, Value};
use mitra_core::simulation::{self, Exact, Rational, Value as V};
use proptest::prelude::*;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};

type E = Expression<Op>;
type Sim = Expression<simulation::Op>;

fn int(n: i64) -> V {
    V::Int(n.into())
}
fn real(n: i64, d: i64) -> V {
    V::Real(Rational::new(n, d).unwrap())
}
fn sim(op: simulation::Op, args: Vec<Sim>) -> Sim {
    apply(op, args)
}
fn word(x: &str) -> Sim {
    Expression::Name(x.to_string())
}
fn input(x: &str) -> E {
    Expression::Name(x.to_string())
}
fn correct(f: &str, at: Term) -> Prop {
    Prop::Correct {
        function: f.to_string(),
        at,
    }
}
fn var(x: &str) -> Term {
    Term::Var(x.to_string())
}
fn all_in(x: &str, over: &Set, body: Prop) -> Prop {
    Prop::All {
        name: x.to_string(),
        over: over.clone(),
        body: Box::new(body),
    }
}
fn any_in(x: &str, over: &Set, body: Prop) -> Prop {
    Prop::Any {
        name: x.to_string(),
        over: over.clone(),
        body: Box::new(body),
    }
}
fn take(p: &Prop, value: bool, by: &str) -> Taking {
    Taking {
        prop: p.clone(),
        value,
        count: 1,
        by: by.to_string(),
    }
}
fn opinion_of(p: &Prop, takings: Vec<Taking>) -> Value {
    Value::Opinion(Opinion::taken(p.clone(), takings).unwrap())
}

/// A random simulation expression over the names `a`, `b`, `c`, `z`:
/// well typed or not, with lets, alls, faults.
fn such(rng: &mut Rng, depth: u32) -> Sim {
    use simulation::Op as O;
    let names = ["a", "b", "c", "z"];
    if depth == 0 || rng.below(4) == 0 {
        return match rng.below(6) {
            0 | 1 => word(rng.pick(&names)),
            2 => sim(O::Truth(rng.coin()), vec![]),
            3 => sim(O::Real(Rational::integer(rng.below(3) as i64)), vec![]),
            _ => sim(O::Int((rng.below(4) as i64 - 1).into()), vec![]),
        };
    }
    let d = depth - 1;
    let s = |rng: &mut Rng| such(rng, d);
    match rng.below(13) {
        0 => sim(O::Not, vec![s(rng)]),
        1 => sim(O::Equal, vec![s(rng), s(rng)]),
        2 => sim(O::Less, vec![s(rng), s(rng)]),
        3 => sim(O::And, vec![s(rng), s(rng)]),
        4 => sim(O::Or, vec![s(rng), s(rng)]),
        5 => sim(O::Add, vec![s(rng), s(rng)]),
        6 => sim(O::Div, vec![s(rng), s(rng)]),
        7 => sim(O::If, vec![s(rng), s(rng), s(rng)]),
        8 => {
            let x = rng.pick(&names).to_string();
            let (v, b) = (s(rng), s(rng));
            let_in(O::Let, &x, v, b)
        }
        9 => {
            // (lo..hi).all(|x| p)
            let x = rng.pick(&names).to_string();
            let (lo, hi, p) = (s(rng), s(rng), s(rng));
            Expression::Apply(
                O::All,
                vec![
                    Arg {
                        binds: vec![],
                        body: lo,
                    },
                    Arg {
                        binds: vec![],
                        body: hi,
                    },
                    Arg {
                        binds: vec![x],
                        body: p,
                    },
                ],
            )
        }
        10 => sim(O::AtMost, vec![s(rng), s(rng)]),
        11 => sim(O::Not, vec![s(rng), s(rng)]), // wrong arity
        _ => sim(O::Sub, vec![s(rng), s(rng)]),
    }
}

/// A random set of whole numbers: zero to three ranges, small ends
/// (some empty, some reversed), names maybe twice, maybe a condition.
fn int_set(rng: &mut Rng) -> Set {
    let names = ["a", "b", "c"];
    let ranges = (0..rng.below(4))
        .map(|_| {
            let lo = rng.below(5) as i64 - 2;
            let hi = rng.below(6) as i64 - 2;
            (rng.pick(&names).to_string(), int(lo), int(hi))
        })
        .collect();
    let such = match rng.below(3) {
        0 => None,
        _ => Some(such(rng, 3)),
    };
    Set { ranges, such }
}

/// Every point of the box of `s`, by an odometer of the test's own, and
/// whether `such` is true there by `Exact`.
fn brute(s: &Set) -> Vec<Vec<V>> {
    let mut ends = vec![];
    for (_, lo, hi) in &s.ranges {
        let (V::Int(lo), V::Int(hi)) = (lo, hi) else {
            panic!("not whole")
        };
        let (lo, hi) = (i64::try_from(lo).unwrap(), i64::try_from(hi).unwrap());
        ends.push((lo, hi));
    }
    let mut out = vec![vec![]];
    for (lo, hi) in ends {
        let mut next = vec![];
        for p in &out {
            for v in lo..hi {
                let mut q = p.clone();
                q.push(int(v));
                next.push(q);
            }
        }
        out = next;
    }
    out.into_iter()
        .filter(|p| match &s.such {
            None => true,
            Some(e) => {
                let values: BTreeMap<String, V> = s
                    .ranges
                    .iter()
                    .zip(p)
                    .map(|((n, _, _), v)| (n.clone(), v.clone()))
                    .collect();
                eval(&Exact::default(), &values, e) == Ok(V::Truth(true))
            }
        })
        .collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4000))]

    /// `points` lists exactly the points `contains` holds, in Rust's
    /// order of tuples, each once.
    #[test]
    fn points_are_what_contains_holds(seed in any::<u64>()) {
        let mut rng = Rng::new(seed);
        let s = int_set(&mut rng);
        let listed: Vec<Vec<V>> = s.points().unwrap().collect();
        prop_assert_eq!(&listed, &brute(&s), "{:?}", s);
        for w in listed.windows(2) {
            prop_assert!(w[0] < w[1]);
        }
        for p in &listed {
            prop_assert!(s.contains(p));
        }
    }
}

/// A random value for a range end: mostly numbers, sometimes not.
fn end(rng: &mut Rng) -> V {
    match rng.below(8) {
        0 => V::Truth(rng.coin()),
        1 | 2 => real(rng.below(5) as i64 - 2, 1 + rng.below(2) as i64),
        _ => int(rng.below(4) as i64 - 1),
    }
}

/// A random set of any kind, small, for comparing.
fn any_set(rng: &mut Rng) -> Set {
    let names = ["a", "b"];
    let ranges = (0..rng.below(3))
        .map(|_| (rng.pick(&names).to_string(), end(rng), end(rng)))
        .collect();
    let such = match rng.below(3) {
        0 => None,
        _ => Some(such(rng, 2)),
    };
    Set { ranges, such }
}

fn any_prop(rng: &mut Rng, sets: &[Set], depth: u32) -> Prop {
    if depth == 0 || rng.below(3) == 0 {
        return match rng.below(3) {
            0 => Prop::named(rng.pick(&["p", "q"])),
            1 => correct("f", var(rng.pick(&["x", "y"]))),
            _ => correct("f", Term::Point(vec![end(rng)])),
        };
    }
    let d = depth - 1;
    match rng.below(4) {
        0 => Prop::Not(Box::new(any_prop(rng, sets, d))),
        1 => all_in(
            rng.pick(&["x", "y"]),
            rng.pick(sets),
            any_prop(rng, sets, d),
        ),
        2 => any_in(
            rng.pick(&["x", "y"]),
            rng.pick(sets),
            any_prop(rng, sets, d),
        ),
        _ => Prop::Implies {
            pre: vec![any_prop(rng, sets, d)],
            subject: "s".to_string(),
            post: vec![any_prop(rng, sets, d)],
        },
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4000))]

    /// Order agrees with equality on sets and propositions of every
    /// kind, is antisymmetric and transitive; hash agrees with equality.
    #[test]
    fn order_agrees_with_equality(seed in any::<u64>()) {
        use std::hash::{DefaultHasher, Hash, Hasher};
        fn hash<T: Hash>(x: &T) -> u64 {
            let mut h = DefaultHasher::new();
            x.hash(&mut h);
            h.finish()
        }
        let mut rng = Rng::new(seed);
        let sets: Vec<Set> = (0..3).map(|_| any_set(&mut rng)).collect();
        // Copies, so that equal ones are frequent.
        let mut sets2 = sets.clone();
        sets2.push(sets[0].clone());
        let props: Vec<Prop> = (0..4).map(|_| any_prop(&mut rng, &sets2, 3)).collect();
        for x in &sets2 {
            for y in &sets2 {
                prop_assert_eq!(x.cmp(y) == Ordering::Equal, x == y);
                prop_assert_eq!(x.cmp(y), y.cmp(x).reverse());
                prop_assert_eq!(x.partial_cmp(y), Some(x.cmp(y)));
                prop_assert!(x != y || hash(x) == hash(y));
                for z in &sets2 {
                    prop_assert!(!(x <= y && y <= z) || x <= z);
                }
            }
        }
        for x in &props {
            for y in &props {
                prop_assert_eq!(x.cmp(y) == Ordering::Equal, x == y);
                prop_assert_eq!(x.cmp(y), y.cmp(x).reverse());
                prop_assert!(x != y || hash(x) == hash(y));
                for z in &props {
                    prop_assert!(!(x <= y && y <= z) || x <= z);
                }
            }
        }
    }
}

/// A random point: of any length, any kind of value.
fn any_point(rng: &mut Rng) -> Vec<V> {
    let rec = simulation::Record {
        name: "R".to_string(),
        fields: vec![("x".to_string(), simulation::Type::Int)],
    };
    (0..rng.below(4))
        .map(|_| match rng.below(10) {
            0 => V::Truth(rng.coin()),
            1 => V::Record(rec.clone(), vec![int(0)]),
            2 => real(rng.below(5) as i64 - 2, 1 + rng.below(2) as i64),
            _ => int(rng.below(5) as i64 - 2),
        })
        .collect()
}

/// A random judgement expression over the inputs `o0` .. `o{n-1}`, with
/// rules on the propositions of `pool` and `wild`.
fn any_expr(rng: &mut Rng, pool: &[Prop], wild: &[Prop], n: usize, depth: u32) -> E {
    if depth == 0 || rng.below(3) == 0 {
        return input(&format!("o{}", rng.below(n + 1)));
    }
    let q = match rng.coin() {
        true => rng.pick(pool).clone(),
        false => rng.pick(wild).clone(),
    };
    let at = match rng.below(3) {
        0 => any_point(rng),
        _ => vec![int(rng.below(4) as i64 - 1)],
    };
    let point = apply(Op::Point(at), vec![]);
    let op = match rng.below(9) {
        0 => Op::Instance { q },
        1 => Op::Witness { q },
        2 | 3 => Op::Cases { q },
        4 => Op::Not,
        5 => Op::Join,
        6 => Op::Reductio { of: q },
        7 => Op::Ponens {
            t: q,
            post: rng.below(2),
        },
        _ => Op::Tollens {
            t: q,
            post: rng.below(2),
            blame: rng.below(3),
        },
    };
    let mut args: Vec<E> = (0..rng.below(5))
        .map(|_| any_expr(rng, pool, wild, n, depth - 1))
        .collect();
    if rng.below(3) > 0 {
        args.insert(rng.below(args.len() + 1), point);
    }
    apply(op, args)
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4000))]

    /// No panic; a checked expression on inputs that fit gives a value of
    /// its type; every opinion made is of a well formed proposition.
    #[test]
    fn wild_quantifiers(seed in any::<u64>(), depth in 1u32..4) {
        let mut rng = Rng::new(seed);
        // Well formed sets, of whole numbers, small; and any sets.
        let good: Vec<Set> = (0..2).map(|_| {
            let mut s = int_set(&mut rng);
            while !s.well_formed() { s = int_set(&mut rng); }
            s
        }).collect();
        let mut sets: Vec<Set> = good.clone();
        sets.extend((0..2).map(|_| any_set(&mut rng)));
        // A pool of propositions with their instances, and wild ones.
        let mut pool: Vec<Prop> = vec![];
        for _ in 0..3 {
            let q = any_prop(&mut rng, &good, 2);
            pool.push(q.clone());
            let listed: Vec<Vec<V>> = match &q {
                Prop::All { over, .. } | Prop::Any { over, .. } => over.points().map(|ps| ps.take(6).collect()).unwrap_or_default(),
                _ => vec![],
            };
            for p in listed {
                if let Some(i) = q.at(&p) { pool.push(i); }
            }
        }
        let wild: Vec<Prop> = (0..4).map(|_| any_prop(&mut rng, &sets, 3)).collect();
        let mut inputs = BTreeMap::new();
        let mut values = BTreeMap::new();
        let mut i = 0;
        for p in pool.iter().chain(&wild) {
            if !p.well_formed() { continue; }
            let takings = (0..rng.below(3)).map(|_| take(p, rng.coin(), rng.pick(&["u", "v"]))).collect();
            let name = format!("o{i}");
            inputs.insert(name.clone(), Type::Opinion(p.clone()));
            values.insert(name, opinion_of(p, takings));
            i += 1;
        }
        let j = Judgement::default();
        let e = any_expr(&mut rng, &pool, &wild, i, depth);
        let t = check(&j, &inputs, &e);
        let out = eval(&j, &values, &e);
        if let Some(t) = &t {
            use mitra_core::expression::Reading;
            prop_assert!(matches!(&out, Ok(v) if j.has_type(v, t)), "{:?}\n{:?}", e, out);
        }
        if let Ok(Value::Opinion(o)) = &out {
            prop_assert!(o.about().well_formed(), "{:?}", o);
        }
    }
}

/// The test's own substitution: `p` with the point `v` for each `x` free.
fn subst(p: &Prop, x: &str, v: &[V]) -> Prop {
    let s = |q: &Prop| subst(q, x, v);
    match p {
        Prop::Correct {
            function,
            at: Term::Var(y),
        } if y == x => correct(function, Term::Point(v.to_vec())),
        Prop::Named(_) | Prop::Correct { .. } => p.clone(),
        Prop::Not(q) => Prop::Not(Box::new(s(q))),
        Prop::Implies { pre, subject, post } => Prop::Implies {
            pre: pre.iter().map(s).collect(),
            subject: subject.clone(),
            post: post.iter().map(s).collect(),
        },
        Prop::All { name, .. } | Prop::Any { name, .. } if name == x => p.clone(),
        Prop::All { name, over, body } => all_in(name, over, s(body)),
        Prop::Any { name, over, body } => any_in(name, over, s(body)),
    }
}

/// A small set of whole numbers, well formed: at most four points in its
/// box, zero to two ranges, maybe a condition with lets and alls.
fn small_set(rng: &mut Rng) -> Set {
    loop {
        let s = int_set(rng);
        let size: i64 = s
            .ranges
            .iter()
            .map(|(_, lo, hi)| match (lo, hi) {
                (V::Int(lo), V::Int(hi)) => {
                    (i64::try_from(hi).unwrap() - i64::try_from(lo).unwrap()).max(0)
                }
                _ => 9,
            })
            .product();
        if s.well_formed() && size <= 4 && s.ranges.len() <= 2 {
            return s;
        }
    }
}

/// The meaning, with points listed by `brute` and the test's own
/// substitution: atoms, and implications as atoms with their rule.
struct Rich {
    vars: Vec<Prop>,
    rules: Vec<(Prop, Vec<Prop>, Vec<Prop>)>,
}

impl Rich {
    fn of(props: &[Prop]) -> Rich {
        let (mut vars, mut rules) = (BTreeSet::new(), BTreeSet::new());
        let mut todo: Vec<Prop> = props.to_vec();
        while let Some(p) = todo.pop() {
            match &p {
                Prop::Not(x) => todo.push((**x).clone()),
                Prop::All { name, over, body } | Prop::Any { name, over, body } => {
                    for v in brute(over) {
                        todo.push(subst(body, name, &v));
                    }
                }
                Prop::Implies { pre, post, .. } => {
                    if vars.insert(p.clone()) {
                        todo.extend(pre.iter().chain(post).cloned());
                        rules.insert((p.clone(), pre.clone(), post.clone()));
                    }
                }
                _ => {
                    vars.insert(p.clone());
                }
            }
        }
        Rich {
            vars: vars.into_iter().collect(),
            rules: rules.into_iter().collect(),
        }
    }
    fn truth(&self, p: &Prop, bits: u64) -> bool {
        match p {
            Prop::Not(x) => !self.truth(x, bits),
            Prop::All { name, over, body } => brute(over)
                .iter()
                .all(|v| self.truth(&subst(body, name, v), bits)),
            Prop::Any { name, over, body } => brute(over)
                .iter()
                .any(|v| self.truth(&subst(body, name, v), bits)),
            _ => {
                let place = self.vars.iter().position(|v| v == p).expect("outside");
                bits >> place & 1 == 1
            }
        }
    }
    fn entails(&self, set: &BTreeSet<Taking>, x: &Prop, value: bool) -> bool {
        (0..1u64 << self.vars.len()).all(|bits| {
            let lawful = self.rules.iter().all(|(t, pre, post)| {
                !(self.truth(t, bits) && pre.iter().all(|p| self.truth(p, bits)))
                    || post.iter().all(|q| self.truth(q, bits))
            });
            let holds = lawful && set.iter().all(|t| self.truth(&t.prop, bits) == t.value);
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

/// A derivation of an opinion of `x`, goal first, from the inputs named
/// in `names`, by every rule that can give it.
fn derive(rng: &mut Rng, x: &Prop, pool: &[Prop], names: &BTreeMap<Prop, String>, depth: u32) -> E {
    if depth == 0 || rng.below(4) == 0 {
        if let Some(n) = names.get(x) {
            return input(n);
        }
    }
    let d = depth.saturating_sub(1);
    let mut ways: Vec<E> = vec![];
    // Down from a quantified proposition of the pool.
    for q in pool {
        if let Prop::All { name, over, body } | Prop::Any { name, over, body } = q {
            for v in brute(over) {
                if &subst(body, name, &v) == x {
                    let point = apply(Op::Point(v.clone()), vec![]);
                    ways.push(apply(
                        Op::Instance { q: q.clone() },
                        vec![derive(rng, q, pool, names, d), point],
                    ));
                }
            }
        }
    }
    if let Prop::All { name, over, body } | Prop::Any { name, over, body } = x {
        let points = brute(over);
        if let Some(v) = points.get(rng.below(points.len().max(1))) {
            let at = subst(body, name, v);
            let point = apply(Op::Point(v.clone()), vec![]);
            ways.push(apply(
                Op::Witness { q: x.clone() },
                vec![derive(rng, &at, pool, names, d), point],
            ));
        }
        let cases = points
            .iter()
            .map(|v| derive(rng, &subst(body, name, v), pool, names, d))
            .collect();
        ways.push(apply(Op::Cases { q: x.clone() }, cases));
    }
    if let Prop::Implies { .. } = x {
        // nothing more: an implication is taken or instanced
    }
    // Ponens on an implication of the pool whose post is x.
    for t in pool {
        if let Prop::Implies { pre, post, .. } = t {
            for (k, p) in post.iter().enumerate() {
                if p == x {
                    let mut args = vec![derive(rng, t, pool, names, d)];
                    args.extend(pre.iter().map(|p| derive(rng, p, pool, names, d)));
                    ways.push(apply(
                        Op::Ponens {
                            t: t.clone(),
                            post: k,
                        },
                        args,
                    ));
                }
            }
            // Tollens blaming x, a premise of t.
            let premises: Vec<Prop> = [vec![t.clone()], pre.clone()].concat();
            for (b, p) in premises.iter().enumerate() {
                if p == x && !post.is_empty() {
                    let k = rng.below(post.len());
                    let mut args = vec![derive(rng, &post[k], pool, names, d)];
                    for (i, p) in premises.iter().enumerate() {
                        if i != b {
                            args.push(derive(rng, p, pool, names, d));
                        }
                    }
                    ways.push(apply(
                        Op::Tollens {
                            t: t.clone(),
                            post: k,
                            blame: b,
                        },
                        args,
                    ));
                }
            }
        }
    }
    if names.contains_key(&x.not()) && rng.coin() {
        ways.push(apply(Op::Not, vec![derive(rng, &x.not(), pool, names, d)]));
    }
    match ways.len() {
        0 => input(names.get(x).map(String::as_str).unwrap_or("none")),
        n => ways.swap_remove(rng.below(n)),
    }
}

/// A quantified proposition over small sets, a name bound again or not,
/// a rule under it or not.
fn small_q(rng: &mut Rng, sets: &[Set], depth: u32) -> Prop {
    let x = *rng.pick(&["x", "y"]);
    let other = if x == "x" { "y" } else { "x" };
    let body = match (depth, rng.below(5)) {
        (0, _) | (_, 0) => correct("f", var(x)),
        (_, 1) => Prop::Not(Box::new(correct("f", var(x)))),
        (_, 2) => Prop::Implies {
            pre: vec![correct("f", var(x))],
            subject: "k".to_string(),
            post: vec![correct("g", var(x))],
        },
        (_, 3) => small_q(rng, sets, depth - 1),
        _ => {
            let inner = Prop::Implies {
                pre: vec![correct("f", var(x))],
                subject: "k".to_string(),
                post: vec![correct("g", var(other))],
            };
            match rng.coin() {
                true => all_in(other, rng.pick(sets), inner),
                false => any_in(other, rng.pick(sets), inner),
            }
        }
    };
    match rng.coin() {
        true => all_in(x, rng.pick(sets), body),
        false => any_in(x, rng.pick(sets), body),
    }
}

/// Every instance of every quantifier in `p`, by the test's own means.
fn instances(p: &Prop, found: &mut Vec<Prop>) {
    if found.contains(p) {
        return;
    }
    found.push(p.clone());
    match p {
        Prop::Not(x) => instances(x, found),
        Prop::Implies { pre, post, .. } => pre.iter().chain(post).for_each(|q| instances(q, found)),
        Prop::All { name, over, body } | Prop::Any { name, over, body } => {
            for v in brute(over) {
                instances(&subst(body, name, &v), found);
            }
        }
        _ => {}
    }
}

/// Counts of what a derivation holds: quantifier rules, and the sets
/// they read with several ranges or none.
#[derive(Default, Debug)]
struct Seen {
    cases: usize,
    rules: usize,
    wide: usize,
    rangeless: usize,
    conditioned: usize,
    skipped: usize,
}

fn count(e: &E, seen: &mut Seen) {
    if let Expression::Apply(op, args) = e {
        if let Op::Ponens { t, .. } | Op::Tollens { t, .. } = op {
            if matches!(t, Prop::Implies { pre, .. } if pre.iter().any(|p| matches!(p, Prop::Correct { at: Term::Point(_), .. })))
            {
                seen.skipped += 1_000_000;
            }
        }
        if let Op::Instance { q, .. } | Op::Witness { q, .. } | Op::Cases { q } = op {
            seen.rules += 1;
            if let Prop::All { over, .. } | Prop::Any { over, .. } = q {
                seen.wide += (over.ranges.len() > 1) as usize;
                seen.rangeless += over.ranges.is_empty() as usize;
                seen.conditioned += over.such.is_some() as usize;
            }
        }
        args.iter().for_each(|a| count(&a.body, seen));
    }
}

fn j1_rich_case(seed: u64, depth: u32, seen: &mut Seen) -> Result<(), String> {
    let mut rng = Rng::new(seed);
    let sets: Vec<Set> = (0..3).map(|_| small_set(&mut rng)).collect();
    let mut pool = vec![];
    for _ in 0..1 + rng.below(2) {
        let q = small_q(&mut rng, &sets, 1);
        instances(&q, &mut pool);
    }
    let rich = Rich::of(&pool);
    if rich.vars.len() > 14 {
        seen.skipped += 1;
        return Ok(());
    }
    let mut props = pool.clone();
    props.extend(pool.iter().map(Prop::not));
    let (mut inputs, mut values, mut names) = (BTreeMap::new(), BTreeMap::new(), BTreeMap::new());
    for (i, p) in props.iter().enumerate() {
        let n = format!("o{i}");
        let takings = (0..rng.below(3))
            .map(|_| take(p, rng.below(3) > 0, rng.pick(&["u", "v"])))
            .collect();
        inputs.insert(n.clone(), Type::Opinion(p.clone()));
        values.insert(n.clone(), opinion_of(p, takings));
        names.insert(p.clone(), n);
    }
    let j = Judgement::default();
    let x = rng.pick(&pool).clone();
    let e = derive(&mut rng, &x, &pool, &names, depth);
    seen.cases += 1;
    count(&e, seen);
    if check(&j, &inputs, &e) != Some(Type::Opinion(x.clone())) {
        return Err(format!("does not check: {e:?}"));
    }
    match eval(&j, &values, &e) {
        Ok(Value::Opinion(o)) if rich.sound(&o) => Ok(()),
        other => Err(format!("unsound or stuck: {other:?}\nfrom {e:?}")),
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(1500))]

    /// J1 on sets the suite never uses: several ranges, no range, lets
    /// and alls in `such`, every derivation checked and sound by the
    /// test's own points and substitution.
    #[test]
    fn j1_rich(seed in any::<u64>(), depth in 1u32..4) {
        let r = j1_rich_case(seed, depth, &mut Seen::default());
        prop_assert!(r.is_ok(), "{}", r.unwrap_err());
    }
}

#[test]
fn j1_rich_counts() {
    let mut seen = Seen::default();
    for seed in 0..3000u64 {
        j1_rich_case(seed, 1 + (seed % 3) as u32, &mut seen).unwrap();
    }
    println!("{seen:?}");
}
