//! The judgement standard (`docs/core2.md`): S2 to S4, J1 (every rule
//! preserves truth, checked on every valuation), J2 (states are read
//! from rests) and J3 (rests are the smallest, and all) on random
//! derivations, half of them over points, and the proposition walked
//! through.

mod common;

use common::{apply, inline, let_in, Rng};
use mitra_core::expression::{check, eval, fits, Arg, Expression, Fault, Reading};
use mitra_core::judgement::{
    Definition, Judgement, Op, Opinion, Point, Prop, Set, State, Taking, Term, Type, Value,
};
use mitra_core::simulation;
use proptest::prelude::*;
use std::collections::{BTreeMap, BTreeSet};

type E = Expression<Op>;
type Sim = Expression<simulation::Op>;

fn int(n: i64) -> simulation::Value {
    simulation::Value::Int(n.into())
}
fn sim(op: simulation::Op, args: Vec<Sim>) -> Sim {
    apply(op, args)
}
fn word(name: &str) -> Sim {
    Expression::Name(name.to_string())
}
/// `name` in `lo..hi`, such that `such`.
fn set(name: &str, lo: i64, hi: i64, such: Option<Sim>) -> Set {
    Set {
        ranges: vec![(name.to_string(), int(lo), int(hi))],
        such,
    }
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
fn point(n: i64) -> Term {
    Term::Point(vec![int(n)])
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

/// The test's own substitution, to check the kernel's: `p` with the point
/// `[v]` in place of each `name` free in it.
fn subst(p: &Prop, name: &str, v: i64) -> Prop {
    subst_at(p, name, &point(v))
}
/// `p` with `at` in place of each `name` free in it.
fn subst_at(p: &Prop, name: &str, at: &Term) -> Prop {
    let s = |q: &Prop| subst_at(q, name, at);
    match p {
        Prop::Correct {
            function,
            at: Term::Var(x),
        } if x == name => correct(function, at.clone()),
        Prop::Named(_) | Prop::Correct { .. } => p.clone(),
        Prop::Not(x) => Prop::Not(Box::new(s(x))),
        Prop::Implies { pre, subject, post } => Prop::Implies {
            pre: pre.iter().map(s).collect(),
            subject: subject.clone(),
            post: post.iter().map(s).collect(),
        },
        Prop::All { name: x, .. } | Prop::Any { name: x, .. } if x == name => p.clone(),
        Prop::All {
            name: x,
            over,
            body,
        } => all_in(x, over, s(body)),
        Prop::Any {
            name: x,
            over,
            body,
        } => any_in(x, over, s(body)),
    }
}

/// The sets of the worlds over points, each with its points listed by
/// hand: `0..3`, `0..3` but 1, `2..3`, and `0..0`, which is empty.
fn sets() -> Vec<(Set, Vec<i64>)> {
    let not_one = sim(
        simulation::Op::Not,
        vec![sim(
            simulation::Op::Equal,
            vec![word("n"), sim(simulation::Op::Int(1.into()), vec![])],
        )],
    );
    vec![
        (set("n", 0, 3, None), vec![0, 1, 2]),
        (set("n", 0, 3, Some(not_one)), vec![0, 2]),
        (set("n", 2, 3, None), vec![2]),
        (set("n", 0, 0, None), vec![]),
    ]
}

/// Every point of `a` is in `b`, as the kernel decides it: by how they
/// are written, the range of `a` within that of `b`, and the condition of
/// `b` none or that of `a`. The test's sets have one range, named `n`.
fn within(a: &Set, b: &Set) -> bool {
    let bounds = |s: &Set| match &s.ranges[..] {
        [(_, simulation::Value::Int(lo), simulation::Value::Int(hi))] => (lo.clone(), hi.clone()),
        _ => unreachable!(),
    };
    let ((lo, hi), (low, high)) = (bounds(a), bounds(b));
    low <= lo && hi <= high && (b.such.is_none() || a.such == b.such)
}
/// The point `intro` binds for `q`.
fn arbitrary(q: &Prop) -> Term {
    let (Prop::All { name, over, .. } | Prop::Any { name, over, .. }) = q else {
        unreachable!()
    };
    Term::Arbitrary {
        name: name.clone(),
        over: over.clone(),
    }
}
/// The point `[v]`, written in a proof.
fn literal(v: i64) -> E {
    apply(Op::Point(vec![int(v)]), vec![])
}

/// The points of `over`, as listed by hand in `sets`.
fn listed<'a>(sets: &'a [(Set, Vec<i64>)], over: &Set) -> &'a [i64] {
    &sets
        .iter()
        .find(|(s, _)| s == over)
        .expect("a set outside the world")
        .1
}

/// A random proposition over points, of `f` at `x`: `correct`, its
/// negation, a rule from it to `a`, or an all or an any around one,
/// binding `x` again or `y`.
fn over_points(rng: &mut Rng, sets: &[(Set, Vec<i64>)], depth: u32) -> Prop {
    let quantified = |rng: &mut Rng, body: Prop| {
        let over = &rng.pick(sets).0;
        match rng.coin() {
            true => all_in("x", over, body),
            false => any_in("x", over, body),
        }
    };
    let body = match (depth, rng.below(5)) {
        (0, _) | (_, 0) => correct("f", var("x")),
        (_, 1) => Prop::Not(Box::new(correct("f", var("x")))),
        (_, 2) => Prop::Implies {
            pre: vec![correct("f", var("x"))],
            subject: "k".to_string(),
            post: vec![Prop::named("a")],
        },
        (_, 3) => {
            // An inner all or any binding `x` again, which hides the outer.
            over_points(rng, sets, depth - 1)
        }
        _ => {
            // An inner one binding `y`, its body still of the outer `x`.
            let inner = Prop::Implies {
                pre: vec![correct("f", var("x"))],
                subject: "k".to_string(),
                post: vec![correct("f", var("y"))],
            };
            let over = &rng.pick(sets).0;
            match rng.coin() {
                true => all_in("y", over, inner),
                false => any_in("y", over, inner),
            }
        }
    };
    quantified(rng, body)
}

/// `p` and every instance of every all and any in it, as the test lists
/// them, each once.
fn with_instances(p: &Prop, sets: &[(Set, Vec<i64>)], found: &mut Vec<Prop>) {
    if found.contains(p) {
        return;
    }
    found.push(p.clone());
    match p {
        Prop::Not(x) => with_instances(x, sets, found),
        Prop::Implies { pre, post, .. } => pre
            .iter()
            .chain(post)
            .for_each(|q| with_instances(q, sets, found)),
        Prop::All { name, over, body } | Prop::Any { name, over, body } => {
            for &v in listed(sets, over) {
                with_instances(&subst(body, name, v), sets, found);
            }
        }
        _ => {}
    }
}

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
    /// The sets, each with its points, in the worlds over points.
    sets: Vec<(Set, Vec<i64>)>,
    /// Each rule, its pres and its posts, as the world makes them.
    rules: Vec<(Prop, Vec<Prop>, Vec<Prop>)>,
    defined: BTreeMap<String, Definition>,
    judgement: Judgement,
    names: BTreeMap<Prop, String>,
    inputs: BTreeMap<String, Type>,
    values: BTreeMap<String, Value>,
    /// The names `let` and `intro` bind in scope, the nearest last.
    bound: Vec<(String, Type)>,
    /// What an `intro` in scope makes known at the point it binds: the
    /// instances there of each all and any around it, and theirs.
    scope: Vec<Prop>,
}

impl World {
    /// Half the worlds are over points: `a`, `f` correct at 0, 1 and 2,
    /// one or two alls and anys over them, each with its instances, and
    /// fewer rules, so that valuations stay few.
    fn new(seed: u64) -> World {
        let mut rng = Rng::new(seed);
        let points = rng.coin();
        World::made(rng, points)
    }
    /// A world over points.
    fn over_points(seed: u64) -> World {
        let mut rng = Rng::new(seed);
        rng.coin();
        World::made(rng, true)
    }
    fn made(mut rng: Rng, points: bool) -> World {
        let sets = match points {
            true => sets(),
            false => vec![],
        };
        let atoms: Vec<Prop> = match points {
            false => ["a", "b", "c"].map(Prop::named).to_vec(),
            true => {
                let mut atoms = vec![Prop::named("a")];
                atoms.extend((0..3).map(|i| correct("f", point(i))));
                for _ in 0..1 + rng.below(2) {
                    let q = over_points(&mut rng, &sets, 1);
                    with_instances(&q, &sets, &mut atoms);
                }
                atoms
            }
        };
        let mut props: Vec<Prop> = atoms
            .iter()
            .cloned()
            .chain(atoms.iter().map(Prop::not))
            .collect();
        let (mut rules, mut defined) = (Vec::new(), BTreeMap::new());
        // The rules a world over points holds: each instance of a rule
        // under an all or an any, written in place.
        for p in &atoms {
            if let Prop::Implies { pre, post, .. } = p {
                rules.push((p.clone(), pre.clone(), post.clone()));
            }
        }
        for i in 0..1 + rng.below([3, 2][points as usize]) {
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
        // Each proposition once: an instance may be the negation of another.
        let mut seen = BTreeSet::new();
        props.retain(|p| seen.insert(p.clone()));
        let mut w = World {
            rng,
            props: props.clone(),
            sets,
            rules,
            defined,
            judgement,
            names: BTreeMap::new(),
            inputs: BTreeMap::new(),
            values: BTreeMap::new(),
            bound: Vec::new(),
            scope: Vec::new(),
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
    /// A proposition of the world; over points, an all or an any one
    /// time in three.
    fn prop(&mut self) -> Prop {
        let quantified: Vec<Prop> = self
            .props
            .iter()
            .filter(|p| matches!(p, Prop::All { .. } | Prop::Any { .. }))
            .cloned()
            .collect();
        if !quantified.is_empty() && self.rng.below(3) == 0 {
            return self.rng.pick(&quantified).clone();
        }
        self.rng.pick(&self.props.clone()).clone()
    }
    /// The premises of the rule `t`, `t` first, and its posts.
    fn rule(&self, t: &Prop) -> (Vec<Prop>, Vec<Prop>) {
        let (_, pre, post) = self.rules.iter().find(|(r, _, _)| r == t).unwrap();
        ([vec![t.clone()], pre.clone()].concat(), post.clone())
    }
    /// The names in scope, each with its type: the inputs, and those a
    /// `let` or an `intro` binds, the nearest hiding the others.
    fn visible(&self) -> BTreeMap<String, Type> {
        let mut visible = self.inputs.clone();
        visible.extend(self.bound.iter().cloned());
        visible
    }
    /// Some name in scope is of type `t`.
    fn named(&self, t: &Type) -> bool {
        self.visible().values().any(|it| it == t)
    }
    /// A name in scope of type `t`.
    fn name(&mut self, t: &Type) -> E {
        let names: Vec<String> = self
            .visible()
            .into_iter()
            .filter(|(_, it)| it == t)
            .map(|(n, _)| n)
            .collect();
        Expression::Name(self.rng.pick(&names).clone())
    }
    /// The points in scope that an `intro` binds, each with its name.
    fn points(&self) -> Vec<(String, Term)> {
        self.visible()
            .into_iter()
            .filter_map(|(n, t)| match t {
                Type::Point(at @ Term::Arbitrary { .. }) => Some((n, at)),
                _ => None,
            })
            .collect()
    }
    /// Every all and any in reach: the world's, and those an `intro` in
    /// scope makes known.
    fn quantified(&self) -> Vec<Prop> {
        self.props
            .iter()
            .chain(&self.scope)
            .filter(|p| matches!(p, Prop::All { .. } | Prop::Any { .. }))
            .cloned()
            .collect()
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
            Type::Opinion(y) => self.names.get(y).cloned().unwrap_or("l0".to_string()),
            Type::State => "s0".to_string(),
            Type::Point(_) => "p0".to_string(),
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
        let named = self.named(&Type::Opinion(x.clone()));
        if named && (depth == 0 || self.rng.below(4) == 0) {
            return self.name(&Type::Opinion(x.clone()));
        }
        if depth == 0 {
            return self.at_hand(x);
        }
        let d = depth - 1;
        // Inside an intro, a let of what holds around it, a third of the
        // time: so that the point is a witness there too.
        let around = !self.points().is_empty() && self.rng.below(3) == 0;
        if around || self.rng.below(6) == 0 {
            let quantified: Vec<Prop> = self
                .props
                .iter()
                .filter(|p| matches!(p, Prop::All { .. } | Prop::Any { .. }))
                .cloned()
                .collect();
            let y = match (around && !quantified.is_empty(), self.rng.coin()) {
                (true, _) => self.rng.pick(&quantified).clone(),
                (false, true) => x.clone(),
                (false, false) => self.prop(),
            };
            let x = x.clone();
            return self.let_in(
                Type::Opinion(y.clone()),
                |w| w.opinion(&y, d),
                |w| w.opinion(&x, d),
            );
        }
        // Each way, with the point it takes, written and as a term.
        let mut ways: Vec<(Op, Option<(E, Term)>)> = vec![
            (Op::Join, None),
            (Op::Not, None),
            (Op::Reductio { of: x.clone() }, None),
        ];
        // Down to x from an all or an any it is an instance of, at a
        // point of its set or at the point an intro binds; up to x, an
        // all or an any, from one point, from every point, or from any.
        let mut quantified = self.quantified();
        if matches!(x, Prop::All { .. } | Prop::Any { .. }) && !quantified.contains(x) {
            quantified.push(x.clone());
        }
        for q in &quantified {
            let (Prop::All { name, over, body } | Prop::Any { name, over, body }) = q else {
                unreachable!()
            };
            for &v in listed(&self.sets, over) {
                let at = Some((literal(v), point(v)));
                if &subst(body, name, v) == x {
                    ways.extend(vec![(Op::Instance { q: q.clone() }, at.clone()); 4]);
                }
                if q == x {
                    ways.extend(vec![(Op::Witness { q: q.clone() }, at); 3]);
                }
            }
            for (p, arb) in self.points() {
                let Term::Arbitrary { over: s, .. } = &arb else {
                    unreachable!()
                };
                if !within(s, over) {
                    continue;
                }
                let at = Some((input(&p), arb.clone()));
                if &subst_at(body, name, &arb) == x {
                    ways.extend(vec![(Op::Instance { q: q.clone() }, at.clone()); 4]);
                }
                if q == x {
                    ways.extend(vec![(Op::Witness { q: q.clone() }, at); 6]);
                }
            }
            if q == x {
                ways.extend(vec![(Op::Cases { q: q.clone() }, None); 6]);
                ways.extend(vec![(Op::Intro { q: q.clone() }, None); 8]);
            }
        }
        for (t, _, _) in &self.rules {
            let (premises, posts) = self.rule(t);
            for (post, q) in posts.iter().enumerate() {
                if q == x {
                    ways.extend(vec![(Op::Ponens { t: t.clone(), post }, None); 2]);
                }
                for (blame, p) in premises.iter().enumerate() {
                    if p == x {
                        let tollens = Op::Tollens {
                            t: t.clone(),
                            post,
                            blame,
                        };
                        ways.extend(vec![(tollens, None); 2]);
                    }
                }
            }
        }
        let (op, at) = self.rng.pick(&ways).clone();
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
            Op::Instance { q } => {
                let (point, _) = at.unwrap();
                vec![self.opinion(&q.clone(), d), point]
            }
            Op::Witness { q } => {
                let (point, term) = at.unwrap();
                let instance = self.instance(q, &term);
                vec![self.opinion(&instance, d), point]
            }
            Op::Intro { q } => return self.intro(&q.clone(), d),
            Op::Cases { q } => {
                let (Prop::All { name, over, body } | Prop::Any { name, over, body }) = q else {
                    unreachable!()
                };
                let cases: Vec<Prop> = listed(&self.sets, over)
                    .iter()
                    .map(|&v| subst(body, name, v))
                    .collect();
                cases.iter().map(|p| self.opinion(p, d)).collect()
            }
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
    /// The instance of `q` at `at`, by the test's own substitution.
    fn instance(&self, q: &Prop, at: &Term) -> Prop {
        let (Prop::All { name, body, .. } | Prop::Any { name, body, .. }) = q else {
            unreachable!()
        };
        subst_at(body, name, at)
    }
    /// `intro(q, |p| …)`: a derivation of the body of `q` at the point
    /// the intro binds, which knows there the instances of every all and
    /// any whose set holds the intro's, and the rules among them.
    fn intro(&mut self, q: &Prop, d: u32) -> E {
        let arb = arbitrary(q);
        let goal = self.instance(q, &arb);
        let Term::Arbitrary { over, .. } = &arb else {
            unreachable!()
        };
        let mut found = Vec::new();
        for around in self.quantified() {
            let (Prop::All { name, over: s, body } | Prop::Any { name, over: s, body }) = &around
            else {
                unreachable!()
            };
            if within(over, s) {
                with_instances(&subst_at(body, name, &arb), &self.sets, &mut found);
            }
        }
        with_instances(&goal, &self.sets, &mut found);
        let (scope, rules) = (self.scope.len(), self.rules.len());
        for p in &found {
            if let Prop::Implies { pre, post, .. } = p {
                self.rules.push((p.clone(), pre.clone(), post.clone()));
            }
        }
        self.scope.extend(found);
        let name = self.rng.pick(&["p0", "p1"]).to_string();
        self.bound.push((name.clone(), Type::Point(arb)));
        let body = self.opinion(&goal, d);
        self.bound.pop();
        self.scope.truncate(scope);
        self.rules.truncate(rules);
        Expression::Apply(
            Op::Intro { q: q.clone() },
            vec![Arg {
                binds: vec![name],
                body,
            }],
        )
    }
    /// An opinion of `x`, which no name has, with no depth left: an
    /// instance of it at a point an intro binds, or else a reductio from
    /// an opinion of the world, which finds nothing.
    fn at_hand(&mut self, x: &Prop) -> E {
        let mut ways = Vec::new();
        for q in &self.props {
            let (Prop::All { name, over, body } | Prop::Any { name, over, body }) = q else {
                continue;
            };
            for (p, arb) in self.points() {
                let Term::Arbitrary { over: s, .. } = &arb else {
                    unreachable!()
                };
                if within(s, over) && &subst_at(body, name, &arb) == x {
                    ways.push((q.clone(), p));
                }
            }
        }
        if !ways.is_empty() && self.rng.below(4) > 0 {
            let (q, p) = self.rng.pick(&ways).clone();
            let whole = self.name(&Type::Opinion(q.clone()));
            return apply(Op::Instance { q }, vec![whole, input(&p)]);
        }
        let y = self.prop();
        let other = self.name(&Type::Opinion(y));
        apply(Op::Reductio { of: x.clone() }, vec![other])
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
        let at = self
            .rng
            .pick(&[
                vec![],
                vec![int(0)],
                vec![int(1)],
                vec![int(2)],
                vec![int(7)],
                vec![int(0), int(1)],
            ])
            .clone();
        let all = [
            Op::Point(at),
            Op::Instance { q: t.clone() },
            Op::Witness { q: t.clone() },
            Op::Cases { q: t.clone() },
            Op::Intro { q: t.clone() },
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
    sets: Vec<(Set, Vec<i64>)>,
}

type Rules = BTreeSet<(Prop, Vec<Prop>, Vec<Prop>)>;

impl Meaning {
    fn of(
        props: &[Prop],
        defined: &BTreeMap<String, Definition>,
        sets: &[(Set, Vec<i64>)],
    ) -> Meaning {
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
                    Prop::Correct { .. } => None,
                    // An all or an any is no variable: its truth is read
                    // from its instances.
                    Prop::All { name, over, body } | Prop::Any { name, over, body } => {
                        let at = listed(sets, over).iter();
                        todo.extend(at.map(|&v| subst(body, name, v)));
                        continue;
                    }
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
            sets: sets.to_vec(),
        }
    }
    fn truth(&self, p: &Prop, bits: u64) -> bool {
        match p {
            Prop::Not(x) => !self.truth(x, bits),
            Prop::All { name, over, body } => listed(&self.sets, over)
                .iter()
                .all(|&v| self.truth(&subst(body, name, v), bits)),
            Prop::Any { name, over, body } => listed(&self.sets, over)
                .iter()
                .any(|&v| self.truth(&subst(body, name, v), bits)),
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
        let meaning = Meaning::of(&w.props, &w.defined, &w.sets);
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
            Value::State(_) | Value::Point(_) => vec![],
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
                prop_assert!(nested || some.is_empty() || within, "follows from {:?} but no rest of {:?} lies in it\nderivation {:?}\npart {:?}", kept, all, e, part);
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

proptest! {
    // Valuations over points are many: fewer cases, each a deeper search.
    #![proptest_config(ProptestConfig::with_cases(1000))]

    /// J1 over points: every set every opinion rests on, in a random
    /// derivation of an all, an any or one of their instances, entails
    /// what it supports, under the truth of an all and an any read from
    /// their points, listed by hand.
    #[test]
    fn quantifiers_preserve_truth(seed in any::<u64>(), depth in 1u32..4) {
        let mut w = World::over_points(seed);
        let x = w.prop();
        let e = w.opinion(&x, depth);
        let j = w.judgement.clone();
        prop_assert_eq!(check(&j, &w.inputs, &e), Some(Type::Opinion(x.clone())));
        let meaning = Meaning::of(&w.props, &w.defined, &w.sets);
        let written = inline(&e, &is_let);
        for sub in subexpressions(&written) {
            if let Ok(Value::Opinion(o)) = eval(&j, &w.values, sub) {
                prop_assert!(meaning.sound(&o), "unsound: {:?}\nfrom {:?}", o, sub);
            }
        }
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
    assert!(Meaning::of(&[t], &BTreeMap::new(), &[]).sound(&refuted));
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
    let meaning = Meaning::of(&[z, q.clone(), w, t7], &BTreeMap::new(), &[]);
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
    let meaning = Meaning::of(&[p.clone(), q.clone(), r.clone()], &defined, &[]);
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
    // Through the body of an all, and of an any.
    let s = set("n", 0, 3, None);
    let every = all_in("x", &s, Prop::Not(Box::new(e.clone())));
    assert_eq!(knowing(&[("e", &[&every], &[&q])]), refused(&["e"]));
    let some = any_in("x", &s, inside.clone());
    assert_eq!(knowing(&[("d", &[&p], &[&some])]), refused(&["d"]));
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

/// Opinions taken of `props`, each by its name, and what checks and runs
/// on them: `Some` opinion when the derivation checks, `None` when it
/// does not, in which case it does not run either.
struct Bench {
    judgement: Judgement,
    inputs: BTreeMap<String, Type>,
    values: BTreeMap<String, Value>,
}

impl Bench {
    fn of(judgement: Judgement, taken: &[(&str, &Prop, Vec<Taking>)]) -> Bench {
        let mut bench = Bench {
            judgement,
            inputs: BTreeMap::new(),
            values: BTreeMap::new(),
        };
        for (name, p, takings) in taken {
            let o = Opinion::taken((*p).clone(), takings.clone()).unwrap();
            bench
                .inputs
                .insert(name.to_string(), Type::Opinion((*p).clone()));
            bench.values.insert(name.to_string(), Value::Opinion(o));
        }
        bench
    }
    fn run(&self, e: &E) -> Option<Opinion> {
        let out = eval(&self.judgement, &self.values, e);
        match check(&self.judgement, &self.inputs, e) {
            Some(_) => Some(opinion(out)),
            None => {
                assert_eq!(out, Err(Fault::Stuck), "runs but does not check: {e:?}");
                None
            }
        }
    }
}

fn rests(sets: &[&[&Taking]]) -> BTreeSet<BTreeSet<Taking>> {
    let set = |ts: &&[&Taking]| ts.iter().map(|t| (*t).clone()).collect();
    sets.iter().map(set).collect()
}

/// A set: its points in order, what is in it, and what is not well
/// formed.
#[test]
fn a_set_and_its_points() {
    use simulation::Op as O;
    let lit = |n: i64| sim(O::Int(n.into()), vec![]);
    let real = |n: i64, d: i64| simulation::Value::Real(simulation::Rational::new(n, d).unwrap());
    let differ = sim(O::Not, vec![sim(O::Equal, vec![word("a"), word("b")])]);
    let s = Set {
        ranges: vec![("a".into(), int(0), int(3)), ("b".into(), int(0), int(2))],
        such: Some(differ),
    };
    assert!(s.well_formed());
    let points: Vec<Vec<simulation::Value>> = s.points().unwrap().collect();
    let expected = [[0, 1], [1, 0], [2, 0], [2, 1]].map(|p| p.map(int).to_vec());
    assert_eq!(points, expected);
    assert!(s.contains(&[int(2), int(1)]));
    for out in [
        vec![int(1), int(1)],
        vec![int(3), int(0)],
        vec![int(-1), int(0)],
        vec![int(0)],
        vec![int(0), int(1), int(0)],
        vec![real(1, 1), int(0)],
    ] {
        assert!(!s.contains(&out), "{out:?} is not in it");
    }
    // Reals: in or not, never listed.
    let r = Set {
        ranges: vec![("v".into(), real(0, 1), real(1, 1))],
        such: None,
    };
    assert!(r.well_formed() && r.contains(&[real(1, 2)]));
    assert!(!r.contains(&[real(1, 1)]) && !r.contains(&[int(0)]));
    assert!(r.points().is_none());
    // A fault is not true: `1 / v <= 4` has no value at 0.
    let one = sim(O::Real(simulation::Rational::integer(1)), vec![]);
    let four = sim(O::Real(simulation::Rational::integer(4)), vec![]);
    let inverse = sim(O::AtMost, vec![sim(O::Div, vec![one, word("v")]), four]);
    let r = Set {
        ranges: vec![("v".into(), real(-1, 1), real(1, 1))],
        such: Some(inverse),
    };
    assert!(r.well_formed() && r.contains(&[real(1, 2)]) && !r.contains(&[real(0, 1)]));
    // No point in an empty range; one point, of no number, with no range.
    assert_eq!(set("n", 3, 3, None).points().unwrap().count(), 0);
    assert_eq!(set("n", 5, 2, None).points().unwrap().count(), 0);
    let none = Set {
        ranges: vec![],
        such: None,
    };
    assert_eq!(none.points().unwrap().collect::<Vec<_>>(), [vec![]]);
    // Not well formed: a range of two kinds, or of truths, a name twice,
    // `such` not a truth, or naming what is no range.
    let (yes, no) = (
        simulation::Value::Truth(true),
        simulation::Value::Truth(false),
    );
    for bad in [
        vec![("a".into(), int(0), real(1, 1))],
        vec![("a".into(), no, yes)],
        vec![("a".into(), int(0), int(1)), ("a".into(), int(0), int(1))],
    ] {
        let bad = Set {
            ranges: bad,
            such: None,
        };
        assert!(!bad.well_formed(), "{bad:?}");
    }
    for such in [lit(1), sim(O::Equal, vec![word("z"), lit(0)])] {
        let bad = set("a", 0, 1, Some(such));
        assert!(!bad.well_formed(), "{bad:?}");
    }
}

/// `all x in 0..3: correct(f, x)` and its any, each rule on each side,
/// by hand: down to a point, up from one point, up from every point.
#[test]
fn quantifiers_walked_through() {
    let s = set("n", 0, 3, None);
    let (every, some) = (
        all_in("x", &s, correct("f", var("x"))),
        any_in("x", &s, correct("f", var("x"))),
    );
    let f = |n: i64| correct("f", point(n));
    let (t0, t1, t2) = (
        take(&f(0), true, 1, "test"),
        take(&f(1), false, 1, "test"),
        take(&f(2), true, 1, "test"),
    );
    let (review, none) = (
        take(&every, true, 1, "review"),
        take(&some, false, 1, "review"),
    );
    let b = Bench::of(
        Judgement::default(),
        &[
            ("o0", &f(0), vec![t0.clone()]),
            ("o1", &f(1), vec![t1.clone()]),
            ("o2", &f(2), vec![t2.clone()]),
            ("every", &every, vec![review.clone()]),
            ("some", &some, vec![none.clone()]),
        ],
    );
    let instance = |q: &Prop, n, x| apply(Op::Instance { q: q.clone() }, vec![input(x), literal(n)]);
    let witness = |q: &Prop, n, x| apply(Op::Witness { q: q.clone() }, vec![input(x), literal(n)]);
    let cases = |q: &Prop, xs: &[&str]| {
        apply(
            Op::Cases { q: q.clone() },
            xs.iter().map(|x| input(x)).collect(),
        )
    };

    // Down: the review of the all says f is correct at 1; the review of
    // the any, false, says f is wrong at 1. Neither says the other side.
    let down = b.run(&instance(&every, 1, "every")).unwrap();
    assert_eq!(
        (down.about(), down.rests_for()),
        (&f(1), &rests(&[&[&review]]))
    );
    assert!(down.rests_against().is_empty());
    let down = b.run(&instance(&some, 1, "some")).unwrap();
    assert_eq!(
        (down.about(), down.rests_against()),
        (&f(1), &rests(&[&[&none]]))
    );
    assert!(down.rests_for().is_empty());

    // Up from one point: f wrong at 1 makes the all false; f correct at
    // 0 makes the any true. Not the other way.
    let up = b.run(&witness(&every, 1, "o1")).unwrap();
    assert_eq!(up.rests_against(), &rests(&[&[&t1]]));
    assert!(up.rests_for().is_empty());
    let up = b.run(&witness(&every, 0, "o0")).unwrap();
    assert!(up.rests_for().is_empty() && up.rests_against().is_empty());
    let up = b.run(&witness(&some, 0, "o0")).unwrap();
    assert_eq!(up.rests_for(), &rests(&[&[&t0]]));
    assert!(up.rests_against().is_empty());

    // Up from every point: the all is false by 1, the any true by 0 or by 2.
    let all3 = b.run(&cases(&every, &["o0", "o1", "o2"])).unwrap();
    assert!(all3.rests_for().is_empty());
    assert_eq!(all3.rests_against(), &rests(&[&[&t1]]));
    let any3 = b.run(&cases(&some, &["o0", "o1", "o2"])).unwrap();
    assert_eq!(any3.rests_for(), &rests(&[&[&t0], &[&t2]]));
    assert!(any3.rests_against().is_empty());
    // With f correct at 1 too, by the review's instance: the all is true
    // from the two tests and the review.
    let mixed = Expression::Apply(
        Op::Cases { q: every.clone() },
        [input("o0"), instance(&every, 1, "every"), input("o2")]
            .map(|body| Arg {
                binds: vec![],
                body,
            })
            .to_vec(),
    );
    let mixed = b.run(&mixed).unwrap();
    assert_eq!(mixed.rests_for(), &rests(&[&[&t0, &review, &t2]]));

    // Not a point of the set, not every point, or not in order: nothing.
    assert!(b.run(&instance(&every, 3, "every")).is_none());
    assert!(b.run(&witness(&every, 5, "o1")).is_none());
    assert!(b.run(&witness(&every, 0, "o1")).is_none());
    assert!(b.run(&cases(&every, &["o0", "o1"])).is_none());
    assert!(b.run(&cases(&every, &["o0", "o2", "o1"])).is_none());
    assert!(b.run(&cases(&every, &["o0", "o1", "o2", "o2"])).is_none());
    // Nor on what is no all and no any.
    let a = Prop::named("a");
    let nowhere = apply(Op::Point(vec![]), vec![]);
    let not_q = Op::Instance { q: f(0) };
    assert!(b.run(&apply(not_q, vec![input("o0"), nowhere])).is_none());
    // A point is the second argument, given once, of as many numbers as
    // the set has ranges; given by a let, it is the same point.
    let one = |args: Vec<E>| apply(Op::Instance { q: every.clone() }, args);
    assert!(b.run(&one(vec![input("every")])).is_none());
    assert!(b.run(&one(vec![literal(1), input("every")])).is_none());
    assert!(b.run(&one(vec![input("every"), literal(1), literal(1)])).is_none());
    let pair = apply(Op::Point(vec![int(1), int(1)]), vec![]);
    assert!(b.run(&one(vec![input("every"), pair])).is_none());
    let shared = let_in(Op::Let, "p", literal(1), one(vec![input("every"), input("p")]));
    assert_eq!(b.run(&shared), b.run(&instance(&every, 1, "every")));
    let not_q = Op::Cases { q: a };
    assert!(b.run(&apply(not_q, vec![])).is_none());

    // Over an empty set: an all is true from nothing, an any false.
    let empty = set("n", 0, 0, None);
    let (every, some) = (
        all_in("x", &empty, correct("f", var("x"))),
        any_in("x", &empty, correct("f", var("x"))),
    );
    let vacuous = b.run(&cases(&every, &[])).unwrap();
    assert_eq!(vacuous.rests_for(), &rests(&[&[]]));
    assert_eq!(vacuous.state(), State::True);
    let vacuous = b.run(&cases(&some, &[])).unwrap();
    assert_eq!(vacuous.rests_against(), &rests(&[&[]]));
    assert_eq!(vacuous.state(), State::False);
}

/// `intro`, by hand: one proof at the point it binds, any point of the
/// set, proves an all or refutes an any. The point is known only inside
/// that proof, only for sets within those it reads, and nothing taken
/// or named can be about it.
#[test]
fn intro_walked_through() {
    let s = set("n", 0, 3, None);
    let empty = set("n", 0, 0, None);
    let (every, some_not) = (
        all_in("x", &s, correct("f", var("x"))),
        any_in("x", &s, Prop::Not(Box::new(correct("f", var("x"))))),
    );
    let review = take(&every, true, 1, "review");
    let mut b = Bench::of(Judgement::default(), &[("every", &every, vec![review.clone()])]);
    b.inputs.insert("one".into(), Type::Point(point(1)));
    b.values.insert("one".into(), Value::Point(Point::of(vec![int(1)])));
    let at_p = |q: &Prop| apply(Op::Instance { q: q.clone() }, vec![input("every"), input("p")]);
    let intro = |q: &Prop, body: E| {
        Expression::Apply(
            Op::Intro { q: q.clone() },
            vec![Arg {
                binds: vec!["p".into()],
                body,
            }],
        )
    };
    // All true from the body true at the point: f correct everywhere,
    // again, over the set or one within it, the empty set included.
    for over in [&s, &set("n", 1, 3, None), &empty] {
        let q = all_in("y", over, correct("f", var("y")));
        let o = b.run(&intro(&q, at_p(&every))).unwrap();
        assert_eq!((o.about(), o.rests_for()), (&q, &rests(&[&[&review]])));
        assert!(o.rests_against().is_empty());
    }
    // Any false from the body false at the point: f wrong nowhere.
    let o = b.run(&intro(&some_not, apply(Op::Not, vec![at_p(&every)]))).unwrap();
    assert_eq!(o.rests_against(), &rests(&[&[&review]]));
    assert!(o.rests_for().is_empty());
    // An all false at the point, or an any true there, says nothing of
    // the whole: the point is any point, not one that was tested.
    let wrong = all_in("x", &s, Prop::Not(Box::new(correct("f", var("x")))));
    let o = b.run(&intro(&wrong, apply(Op::Not, vec![at_p(&every)]))).unwrap();
    assert!(o.rests_for().is_empty() && o.rests_against().is_empty());
    // A set not within the one read, or a body of another proposition:
    // nothing. Nor outside the intro, where `p` is unbound.
    let wider = all_in("x", &set("n", 0, 4, None), correct("f", var("x")));
    assert!(b.run(&intro(&wider, at_p(&every))).is_none());
    let other = all_in("x", &s, correct("g", var("x")));
    assert!(b.run(&intro(&other, at_p(&every))).is_none());
    assert!(b.run(&at_p(&every)).is_none());
    // A set with a condition holds the any point of a set without it
    // only if written the same: 0..3 is not within 0..3 where n != 1.
    let (not_one, _) = sets().swap_remove(1);
    let on_some = all_in("x", &not_one, correct("f", var("x")));
    let c = Bench::of(Judgement::default(), &[("some", &on_some, vec![take(&on_some, true, 1, "review")])]);
    let from_some = apply(Op::Instance { q: on_some.clone() }, vec![input("some"), input("p")]);
    assert!(c.run(&intro(&every, from_some.clone())).is_none());
    let o = c.run(&intro(&all_in("y", &not_one, correct("f", var("y"))), from_some)).unwrap();
    assert_eq!(o.state(), State::True);
    // The point given as an input is a point written in place, of its
    // own type only.
    let at_one = apply(Op::Instance { q: every.clone() }, vec![input("every"), input("one")]);
    assert_eq!(b.run(&at_one).unwrap().about(), &correct("f", point(1)));
    let two = Value::Point(Point::of(vec![int(2)]));
    assert!(b.judgement.has_type(&b.values["one"], &Type::Point(point(1))));
    assert!(!b.judgement.has_type(&two, &Type::Point(point(1))));
    assert!(!b.judgement.has_type(&two, &Type::Point(arbitrary(&every))));
    // The any point of a set that is not well formed is not well formed.
    let bad = Set {
        ranges: vec![("n".into(), int(0), simulation::Value::Truth(true))],
        such: None,
    };
    let nowhere = Term::Arbitrary {
        name: "x".into(),
        over: bad,
    };
    assert!(!correct("f", nowhere).well_formed());
    // Nothing taken, and no rule known by name, is about the point an
    // intro binds.
    let arb = Term::Arbitrary {
        name: "x".into(),
        over: s.clone(),
    };
    assert!(Opinion::taken(correct("f", arb.clone()), vec![]).is_none());
    let defined = BTreeMap::from([(
        "r".to_string(),
        Definition {
            pre: vec![correct("f", arb.clone())],
            post: vec![Prop::named("a")],
        },
    )]);
    let j = Judgement::with(defined).unwrap();
    assert!(j.premises(&Prop::named("r")).is_none());
    // Inside an intro, the point may be taken back to its whole, or be
    // a witness: what follows is about any point of the set, so only
    // what the intro proves leaves it.
    let up = apply(Op::Witness { q: any_in("z", &s, correct("f", var("z"))) }, vec![at_p(&every), input("p")]);
    let q = all_in("y", &s, correct("f", var("y")));
    let inside = let_in(Op::Let, "w", up, at_p(&every));
    let o = b.run(&intro(&q, inside)).unwrap();
    assert_eq!(o.rests_for(), &rests(&[&[&review]]));
}

/// Judgement reasons only about propositions that are well formed: a
/// name a term uses is bound around it, and each set is well formed. It
/// takes no other, and no rule gives or takes one.
#[test]
fn only_well_formed_propositions() {
    let s = set("n", 0, 3, None);
    let open = correct("f", var("x"));
    assert!(!open.well_formed());
    assert!(Opinion::taken(open.clone(), vec![]).is_none());
    assert!(!all_in("x", &s, correct("f", var("y"))).well_formed());
    assert!(!Prop::Not(Box::new(open.clone())).well_formed());
    let twice = Set {
        ranges: vec![("a".into(), int(0), int(1)), ("a".into(), int(0), int(1))],
        such: None,
    };
    assert!(!all_in("x", &twice, Prop::named("a")).well_formed());
    // Bound by an all around another.
    assert!(all_in("x", &s, any_in("y", &s, correct("f", var("x")))).well_formed());
    assert!(any_in("x", &s, all_in("y", &s, open.clone())).well_formed());

    // A rule that gives what is not well formed gives nothing.
    let (a, r) = (Prop::named("a"), Prop::named("r"));
    let gives_open = Definition {
        pre: vec![a.clone()],
        post: vec![open.clone()],
    };
    let j = Judgement::with(BTreeMap::from([("r".to_string(), gives_open)])).unwrap();
    let b = Bench::of(
        j,
        &[
            ("a", &a, vec![take(&a, true, 1, "test")]),
            ("r", &r, vec![take(&r, true, 1, "review")]),
        ],
    );
    let ponens = apply(
        Op::Ponens {
            t: r.clone(),
            post: 0,
        },
        vec![input("r"), input("a")],
    );
    assert!(b.run(&ponens).is_none());
    let reductio = apply(Op::Reductio { of: open.clone() }, vec![input("a")]);
    assert!(b.run(&reductio).is_none());
    let reductio = apply(Op::Reductio { of: a.clone() }, vec![input("a")]);
    assert!(b.run(&reductio).is_some());
}

/// An all or an any that binds a name again hides the outer one, and a
/// point is written only where the name is free.
#[test]
fn a_name_bound_again_is_hidden() {
    let (s, t) = (set("n", 0, 3, None), set("n", 2, 3, None));
    let hidden = all_in("x", &s, any_in("x", &t, correct("f", var("x"))));
    assert_eq!(
        hidden.at(&[int(0)]),
        Some(any_in("x", &t, correct("f", var("x"))))
    );
    let outer = all_in("x", &s, any_in("y", &t, correct("f", var("x"))));
    assert_eq!(
        outer.at(&[int(0)]),
        Some(any_in("y", &t, correct("f", point(0))))
    );
    let rule = all_in(
        "x",
        &s,
        Prop::Implies {
            pre: vec![correct("f", var("x"))],
            subject: "k".to_string(),
            post: vec![Prop::Not(Box::new(correct("g", var("x"))))],
        },
    );
    let at_2 = Prop::Implies {
        pre: vec![correct("f", point(2))],
        subject: "k".to_string(),
        post: vec![Prop::Not(Box::new(correct("g", point(2))))],
    };
    assert_eq!(rule.at(&[int(2)]), Some(at_2));
    assert_eq!(rule.at(&[int(3)]), None);
    assert_eq!(correct("f", point(0)).at(&[int(0)]), None);
}

/// A random `such`, small so that two are often equal: `a`, `b`, 0 to
/// 2, `!`, `==`, `<`, `&&`, and a `let` binding `a` or `c`.
fn such(rng: &mut Rng, depth: u32) -> Sim {
    use simulation::Op as O;
    if depth == 0 || rng.below(3) == 0 {
        return match rng.below(3) {
            0 => word(rng.pick(&["a", "b"])),
            _ => sim(O::Int((rng.below(3) as i64).into()), vec![]),
        };
    }
    let d = depth - 1;
    match rng.below(5) {
        0 => sim(O::Not, vec![such(rng, d)]),
        1 => sim(O::Equal, vec![such(rng, d), such(rng, d)]),
        2 => sim(O::Less, vec![such(rng, d), such(rng, d)]),
        3 => sim(O::And, vec![such(rng, d), such(rng, d)]),
        _ => {
            let name = rng.pick(&["a", "c"]).to_string();
            let (value, body) = (such(rng, d), such(rng, d));
            let_in(O::Let, &name, value, body)
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3000))]

    /// Sets are ordered as they are equal, so that propositions about
    /// them can be kept in sets: `cmp` is `Equal` exactly when `==`
    /// holds, turns round with its sides, and is transitive; equal sets
    /// hash alike.
    #[test]
    fn sets_are_ordered_as_they_are_equal(seed in any::<u64>()) {
        use std::cmp::Ordering;
        use std::hash::{DefaultHasher, Hash, Hasher};
        let mut rng = Rng::new(seed);
        let mut make = || Set {
            ranges: vec![("a".to_string(), int(0), int(1 + rng.below(2) as i64))],
            such: match rng.below(4) {
                0 => None,
                _ => Some(such(&mut rng, 2)),
            },
        };
        let three = [make(), make(), make()];
        let hash = |x: &Set| {
            let mut h = DefaultHasher::new();
            x.hash(&mut h);
            h.finish()
        };
        // Two that differ only in the name a `let` binds.
        let (one, two) = (sim(simulation::Op::Int(1.into()), vec![]), sim(simulation::Op::Int(2.into()), vec![]));
        let (by_a, by_c) = (
            let_in(simulation::Op::Let, "a", one.clone(), two.clone()),
            let_in(simulation::Op::Let, "c", one, two),
        );
        let (x, y) = (set("a", 0, 1, Some(by_a)), set("a", 0, 1, Some(by_c)));
        prop_assert!(x != y && x.cmp(&y) != Ordering::Equal);
        for x in &three {
            for y in &three {
                prop_assert_eq!(x.cmp(y) == Ordering::Equal, x == y);
                prop_assert_eq!(x.cmp(y), y.cmp(x).reverse());
                prop_assert!(x != y || hash(x) == hash(y));
                for z in &three {
                    prop_assert!(!(x <= y && y <= z) || x <= z);
                }
            }
        }
    }
}
