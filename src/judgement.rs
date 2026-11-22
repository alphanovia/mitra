//! The judgement standard (`docs/core2.md`): what is believed of
//! propositions, and the rules that pass it on. Judging runs the model on
//! takings and gives states.
//!
//! A proposition is `p`, a premise; `t`, the implication `{p} s {q}`, itself
//! a premise that can be blamed; or `q`, a conclusion. A leaf is a proposition
//! nothing concludes: it is held only by being taken. The type of an
//! opinion names its proposition, so the type of a rule is the rule:
//! `Ponens` takes opinions of `t, p₁ … pₙ` and gives one of `q`.
//!
//! An implication is written in place, or known by a name: judgement
//! holds the definition of each name it knows as a rule, once, however
//! many propositions name it. No definition reaches itself through the
//! names it uses.
//!
//! Every rule preserves truth. Numbers live only on takings: no rule
//! reads or makes one. The state of a proposition is read from what it rests
//! on, never from numbers.
//!
//! `let` names a derivation to use it twice. It shares (S6): `let x = a;
//! b` gives what `b` with `a` written in place of `x` gives.
//!
//! A proposition may be about points: `correct(f, x)`, under `all x in
//! S` or `any x in S`. Types are then dependent, syntactically: the type
//! of an opinion names its proposition, points and bound names
//! included, and two are equal only when written the same. A rule at a
//! point writes the point in place of the name, so the types of its
//! arguments say which point. Judgement reads a set with the simulation
//! standard: `Exact` says whether a point is in it, and lists the points
//! of a set of whole numbers.

use crate::expression::{check, eval, strict, Expression, Fault, Outcome, Reading, Standard};
use crate::graph::components;
use crate::simulation::{self, Exact, Simulation};
use num_bigint::BigInt;
use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;
use std::hash::{Hash, Hasher};

/// A proposition about the system.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Prop {
    /// A proposition known by its name. A name judgement defines is a
    /// rule: the implication its definition says.
    Named(String),
    /// `!x`: `x` is false.
    Not(Box<Prop>),
    /// `t = {pre} subject {post}`: when every proposition of `pre` holds,
    /// `subject` makes every proposition of `post` hold.
    Implies {
        pre: Vec<Prop>,
        subject: String,
        post: Vec<Prop>,
    },
    /// `correct(f, at)`: the function `f`, known by its path, gives at
    /// `at` what it should. Judgement does not know `f`; the model does.
    Correct { function: String, at: Term },
    /// `all x in S: body`: `body` holds at every point of `S`, `x`
    /// standing for that point.
    All {
        name: String,
        over: Set,
        body: Box<Prop>,
    },
    /// `any x in S: body`: `body` holds at some point of `S`.
    Any {
        name: String,
        over: Set,
        body: Box<Prop>,
    },
}

/// Where a function is said correct: at a point, the values of its
/// arguments in order, as in a row of its examples; or at the name an
/// all or an any around it binds to each point of its set.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Term {
    Point(Vec<simulation::Value>),
    Var(String),
}

/// A set of points: one number per range, each in its range, `lo..hi`,
/// `hi` left out as in Rust, such that `such`, when there is one, is
/// true at the point, by `Exact`, each number read by the name of its
/// range. A fault is not true. A set names nothing outside itself.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Set {
    pub ranges: Vec<(String, simulation::Value, simulation::Value)>,
    pub such: Option<Expression<simulation::Op>>,
}

impl Set {
    /// Its names are distinct, each range is of two `Int` or of two
    /// `Real`, and `such` is a truth when each name is a number of its
    /// range.
    pub fn well_formed(&self) -> bool {
        use simulation::{Type as T, Value as V};
        let mut inputs = BTreeMap::new();
        for (name, lo, hi) in &self.ranges {
            let kind = match (lo, hi) {
                (V::Int(_), V::Int(_)) => T::Int,
                (V::Real(_), V::Real(_)) => T::Real,
                _ => return false,
            };
            if inputs.insert(name.clone(), kind).is_some() {
                return false;
            }
        }
        self.such
            .as_ref()
            .is_none_or(|such| check(&Simulation, &inputs, such) == Some(T::Truth))
    }
    /// `point` is in the set: one value per range, each in its range,
    /// and `such` true there.
    pub fn contains(&self, point: &[simulation::Value]) -> bool {
        use simulation::Value as V;
        let inside = |((_, lo, hi), v): (&(String, V, V), &V)| match (lo, hi, v) {
            (V::Int(lo), V::Int(hi), V::Int(v)) => lo <= v && v < hi,
            (V::Real(lo), V::Real(hi), V::Real(v)) => lo <= v && v < hi,
            _ => false,
        };
        point.len() == self.ranges.len()
            && self.ranges.iter().zip(point).all(inside)
            && self.such_at(point)
    }
    /// `such` is true at `point`, or there is none.
    fn such_at(&self, point: &[simulation::Value]) -> bool {
        let Some(such) = &self.such else {
            return true;
        };
        let values = self.ranges.iter().zip(point);
        let values = values
            .map(|((name, _, _), v)| (name.clone(), v.clone()))
            .collect();
        eval(&Exact, &values, such) == Ok(simulation::Value::Truth(true))
    }
    /// Every point of the set, in increasing order, the first number
    /// first, as Rust orders tuples. `None` when a range is not of whole
    /// numbers. It reads every point of the ranges, in the set or not.
    pub fn points(&self) -> Option<impl Iterator<Item = Vec<simulation::Value>> + '_> {
        use simulation::Value as V;
        let mut ends: Vec<(BigInt, BigInt)> = Vec::with_capacity(self.ranges.len());
        for (_, lo, hi) in &self.ranges {
            let (V::Int(lo), V::Int(hi)) = (lo, hi) else {
                return None;
            };
            ends.push((lo.clone(), hi.clone()));
        }
        let empty = ends.iter().any(|(lo, hi)| lo >= hi);
        let mut next = (!empty).then(|| ends.iter().map(|(lo, _)| lo.clone()).collect::<Vec<_>>());
        let every = std::iter::from_fn(move || {
            let point = next.take()?;
            // The last number moves first; past its end it starts again,
            // and the one before moves.
            let mut after = point.clone();
            for i in (0..after.len()).rev() {
                after[i] += 1;
                if after[i] < ends[i].1 {
                    next = Some(after);
                    break;
                }
                after[i] = ends[i].0.clone();
            }
            Some(point.into_iter().map(V::Int).collect::<Vec<_>>())
        });
        Some(every.filter(move |point| self.such_at(point)))
    }
}

/// Sets in a fixed order, the same as their equality, so that they can
/// be kept in sets: by their ranges, then by `such`, read part by part.
impl Ord for Set {
    fn cmp(&self, other: &Set) -> Ordering {
        let such = match (&self.such, &other.such) {
            (Some(a), Some(b)) => order(a, b),
            (a, b) => a.is_some().cmp(&b.is_some()),
        };
        self.ranges.cmp(&other.ranges).then(such)
    }
}

impl PartialOrd for Set {
    fn partial_cmp(&self, other: &Set) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Equal sets have equal ranges.
impl Hash for Set {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.ranges.hash(state);
    }
}

/// Two expressions in a fixed order, the same as their equality: the
/// first part where they differ decides, read in order, each operation
/// before its arguments. On a stack of its own.
fn order<Op: Ord>(a: &Expression<Op>, b: &Expression<Op>) -> Ordering {
    let mut todo = vec![(a, b)];
    while let Some(pair) = todo.pop() {
        let here = match pair {
            (Expression::Name(x), Expression::Name(y)) => x.cmp(y),
            (Expression::Name(_), Expression::Apply(..)) => Ordering::Less,
            (Expression::Apply(..), Expression::Name(_)) => Ordering::Greater,
            (Expression::Apply(f, xs), Expression::Apply(g, ys)) => {
                todo.extend(xs.iter().zip(ys).rev().map(|(x, y)| (&x.body, &y.body)));
                let binds = |args: &'_ [crate::expression::Arg<Op>]| {
                    args.iter().map(|a| a.binds.clone()).collect::<Vec<_>>()
                };
                f.cmp(g)
                    .then(xs.len().cmp(&ys.len()))
                    .then_with(|| binds(xs).cmp(&binds(ys)))
            }
        };
        if here != Ordering::Equal {
            return here;
        }
    }
    Ordering::Equal
}

impl Prop {
    pub fn named(name: &str) -> Prop {
        Prop::Named(name.to_string())
    }
    /// `!self`: the negation of a `Not(x)` is `x`, of any other proposition
    /// its `Not`.
    pub fn not(&self) -> Prop {
        match self {
            Prop::Not(x) => (**x).clone(),
            x => Prop::Not(Box::new(x.clone())),
        }
    }
    /// Every name in `self`, into `found`. On a stack of its own, so
    /// that a deep proposition does not run out of the call stack.
    fn names<'a>(&'a self, found: &mut BTreeSet<&'a str>) {
        let mut todo = vec![self];
        while let Some(p) = todo.pop() {
            match p {
                Prop::Named(n) => _ = found.insert(n),
                Prop::Not(x) => todo.push(x),
                Prop::Implies { pre, post, .. } => todo.extend(pre.iter().chain(post)),
                Prop::Correct { .. } => {}
                Prop::All { body, .. } | Prop::Any { body, .. } => todo.push(body),
            }
        }
    }
    /// A proposition judgement reasons about: each name a term uses is
    /// bound by an all or an any around it, and each set is well formed.
    pub fn well_formed(&self) -> bool {
        self.closed(&mut Vec::new())
    }
    fn closed<'a>(&'a self, bound: &mut Vec<&'a str>) -> bool {
        match self {
            Prop::Named(_) => true,
            Prop::Not(x) => x.closed(bound),
            Prop::Implies { pre, post, .. } => pre.iter().chain(post).all(|p| p.closed(bound)),
            Prop::Correct { at, .. } => match at {
                Term::Point(_) => true,
                Term::Var(x) => bound.contains(&x.as_str()),
            },
            Prop::All { name, over, body } | Prop::Any { name, over, body } => {
                bound.push(name);
                let closed = over.well_formed() && body.closed(bound);
                bound.pop();
                closed
            }
        }
    }
    /// The body of an all or an any at `point`, a point of its set: the
    /// body with `point` in place of the name it binds. `None` when
    /// `self` is neither, or `point` is not in its set.
    pub fn at(&self, point: &[simulation::Value]) -> Option<Prop> {
        match self {
            Prop::All { name, over, body } | Prop::Any { name, over, body }
                if over.contains(point) =>
            {
                Some(body.put(name, point))
            }
            _ => None,
        }
    }
    /// `self` with `point` in place of each `name` free in it. A point
    /// holds no name, so nothing is caught.
    fn put(&self, name: &str, point: &[simulation::Value]) -> Prop {
        let put = |p: &Prop| p.put(name, point);
        match self {
            Prop::Named(_) => self.clone(),
            Prop::Not(x) => Prop::Not(Box::new(put(x))),
            Prop::Implies { pre, subject, post } => Prop::Implies {
                pre: pre.iter().map(put).collect(),
                subject: subject.clone(),
                post: post.iter().map(put).collect(),
            },
            Prop::Correct {
                function,
                at: Term::Var(x),
            } if x == name => Prop::Correct {
                function: function.clone(),
                at: Term::Point(point.to_vec()),
            },
            Prop::Correct { .. } => self.clone(),
            // An all or an any that binds `name` again hides it.
            Prop::All { name: x, .. } | Prop::Any { name: x, .. } if x == name => self.clone(),
            Prop::All {
                name: x,
                over,
                body,
            } => Prop::All {
                name: x.clone(),
                over: over.clone(),
                body: Box::new(put(body)),
            },
            Prop::Any {
                name: x,
                over,
                body,
            } => Prop::Any {
                name: x.clone(),
                over: over.clone(),
                body: Box::new(put(body)),
            },
        }
    }
    /// The body of an all or an any at each point of its set, in order,
    /// when it has `n` points. `None` when `self` is neither, its set is
    /// not of whole numbers, or it has more or fewer points: listing
    /// stops at `n + 1`.
    fn cases(&self, n: usize) -> Option<Vec<Prop>> {
        let (Prop::All { name, over, body } | Prop::Any { name, over, body }) = self else {
            return None;
        };
        let points = over.points()?.take(n.saturating_add(1));
        let bodies: Vec<Prop> = points.map(|point| body.put(name, &point)).collect();
        (bodies.len() == n).then_some(bodies)
    }
}

/// What a rule known by its name says: when every proposition of `pre`
/// holds, every proposition of `post` does.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Definition {
    pub pre: Vec<Prop>,
    pub post: Vec<Prop>,
}

/// What is known of a proposition (Belnap): nothing, true, false, or both.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum State {
    Unknown,
    True,
    False,
    Conflict,
}

impl State {
    /// The state of a proposition told true or not, told false or not.
    pub fn told(told_true: bool, told_false: bool) -> State {
        match (told_true, told_false) {
            (false, false) => State::Unknown,
            (true, false) => State::True,
            (false, true) => State::False,
            (true, true) => State::Conflict,
        }
    }
    /// Told true, told false.
    pub fn sides(self) -> (bool, bool) {
        (
            matches!(self, State::True | State::Conflict),
            matches!(self, State::False | State::Conflict),
        )
    }
    /// Both hold: told true when both are, told false when either is.
    pub fn and(self, other: State) -> State {
        let ((a, b), (c, d)) = (self.sides(), other.sides());
        State::told(a && c, b || d)
    }
    /// All that either tells.
    pub fn join(self, other: State) -> State {
        let ((a, b), (c, d)) = (self.sides(), other.sides());
        State::told(a || c, b || d)
    }
}

/// `!state`: told true when told false, and the other way.
impl std::ops::Not for State {
    type Output = State;
    fn not(self) -> State {
        let (t, f) = self.sides();
        State::told(f, t)
    }
}

/// A voice takes a proposition as true or as false, `count` times as firmly
/// as one test.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Taking {
    pub prop: Prop,
    pub value: bool,
    pub count: u64,
    pub by: String,
}

/// Sets of takings, none holding another.
pub type Rests = BTreeSet<BTreeSet<Taking>>;

/// What is believed of one proposition: the smallest sets of takings it
/// follows true from, and false from. Made only by `taken` and by the
/// rules, so every set it holds is one the proposition follows from,
/// under the rules the judgement that made it knows by name. It does not
/// say which: read it only with that judgement.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opinion {
    about: Prop,
    rests_for: Rests,
    rests_against: Rests,
}

impl Opinion {
    /// What these takings of `about` say of it: each taking it true is
    /// a set it follows true from, each taking it false one it follows
    /// false from. `None` when a taking is of another proposition, or
    /// `about` is not well formed.
    pub fn taken(about: Prop, takings: Vec<Taking>) -> Option<Opinion> {
        if !about.well_formed() {
            return None;
        }
        let mut opinion = Opinion {
            about,
            rests_for: Rests::new(),
            rests_against: Rests::new(),
        };
        for taking in takings {
            if taking.prop != opinion.about {
                return None;
            }
            let side = match taking.value {
                true => &mut opinion.rests_for,
                false => &mut opinion.rests_against,
            };
            side.insert(BTreeSet::from([taking]));
        }
        Some(opinion)
    }
    pub fn about(&self) -> &Prop {
        &self.about
    }
    pub fn rests_for(&self) -> &Rests {
        &self.rests_for
    }
    pub fn rests_against(&self) -> &Rests {
        &self.rests_against
    }
    /// Read from the rests: told true when it follows true from some
    /// set, told false when it follows false from some set.
    pub fn state(&self) -> State {
        State::told(!self.rests_for.is_empty(), !self.rests_against.is_empty())
    }
}

/// Only the smallest: every set that holds another goes.
fn smallest(sets: impl IntoIterator<Item = BTreeSet<Taking>>) -> Rests {
    let mut all: Vec<BTreeSet<Taking>> = sets.into_iter().collect::<Rests>().into_iter().collect();
    all.sort_by_key(BTreeSet::len);
    // A kept set within `s` has its least taking in `s`: look only there.
    let mut by_least: BTreeMap<&Taking, Vec<usize>> = BTreeMap::new();
    let mut keep = vec![false; all.len()];
    for (i, s) in all.iter().enumerate() {
        let Some(least) = s.first() else {
            return Rests::from([BTreeSet::new()]);
        };
        let within = |x| {
            by_least
                .get(x)
                .is_some_and(|ks: &Vec<usize>| ks.iter().any(|&k| all[k].is_subset(s)))
        };
        if !s.iter().any(within) {
            by_least.entry(least).or_default().push(i);
            keep[i] = true;
        }
    }
    all.into_iter()
        .zip(keep)
        .filter_map(|(s, k)| k.then_some(s))
        .collect()
}

/// Every way to have all of `sides`: one set of each, together. Sides
/// with no taking in common already give only the smallest.
fn all_of(sides: &[&Rests]) -> Rests {
    // Sides with fewest sets first: a side that holds the others' takings
    // then keeps the ways few.
    let mut sides = sides.to_vec();
    sides.sort_by_key(|side| side.len());
    let mut ways = Rests::from([BTreeSet::new()]);
    let mut seen: BTreeSet<&Taking> = BTreeSet::new();
    for side in sides {
        let takings: BTreeSet<&Taking> = side.iter().flatten().collect();
        let shared = takings.iter().any(|t| seen.contains(t));
        let next = ways
            .iter()
            .flat_map(|way| side.iter().map(move |s| way.union(s).cloned().collect()));
        ways = match shared {
            true => smallest(next),
            false => next.collect(),
        };
        seen.extend(takings);
    }
    ways
}

/// The types: a state, or an opinion of a proposition.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    State,
    Opinion(Prop),
}

/// A value: a state, or an opinion.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    State(State),
    Opinion(Opinion),
}

/// The rules. The premises of an implication `t` are `t`, then its pres.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Op {
    /// `!x` on a state, or on an opinion of `x`, giving one of `!x`.
    Not,
    /// On two states.
    And,
    /// All that both tell: two states, or two opinions of one proposition.
    Join,
    /// The state of an opinion.
    StateOf,
    /// Modus ponens. From every premise of `t`, in order, its post
    /// number `post`.
    Ponens { t: Prop, post: usize },
    /// Modus tollens. From post number `post` of `t`, then every premise
    /// of `t` but number `blame`, in order: premise `blame` is false.
    /// `blame` 0 blames `t`.
    Tollens { t: Prop, post: usize, blame: usize },
    /// Reductio. Where an opinion follows both true and false, the
    /// takings it follows from cannot all hold: whatever else they rest
    /// on says a taking of `of` is wrong.
    Reductio { of: Prop },
    /// `let x = a; b`: `b`, with `x` standing for what `a` gives. The
    /// second argument binds `x`.
    Let,
    /// From an opinion of `q`, an all or an any, one of its body at
    /// `at`, a point of its set: an all true makes it true, an any false
    /// makes it false.
    Instance { q: Prop, at: Vec<simulation::Value> },
    /// From an opinion of the body of `q` at `at`, a point of its set,
    /// one of `q`: true there makes an any true, false there makes an
    /// all false.
    Witness { q: Prop, at: Vec<simulation::Value> },
    /// From an opinion of the body of `q` at each point of its set, in
    /// the order `Set::points` lists them, one of `q`: all the points
    /// tell. An all is true when every point is, and false when one is;
    /// an any is true when one is, and false when every one is.
    Cases { q: Prop },
}

/// The judgement standard and its reading. It knows rules by name: the
/// definition of each, held once. `Judgement::default()` knows none.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Judgement {
    rules: BTreeMap<String, Definition>,
}

impl Judgement {
    /// Judgement that knows these rules, each by its name. `Err` with
    /// the names defined through themselves: the definition of each
    /// names it, or names a rule whose definition does, and so on. What
    /// such a name means would rest on itself.
    pub fn with(rules: BTreeMap<String, Definition>) -> Result<Judgement, BTreeSet<String>> {
        let number: BTreeMap<&str, usize> = rules
            .keys()
            .enumerate()
            .map(|(i, n)| (n.as_str(), i))
            .collect();
        let out: Vec<Vec<usize>> = rules
            .values()
            .map(|d| {
                let mut names = BTreeSet::new();
                d.pre
                    .iter()
                    .chain(&d.post)
                    .for_each(|p| p.names(&mut names));
                names
                    .iter()
                    .filter_map(|n| number.get(n).copied())
                    .collect()
            })
            .collect();
        // On a cycle: a name the definition of which names one of its
        // own component, itself included.
        let component = components(&out);
        let circular: BTreeSet<String> = rules
            .keys()
            .zip(&out)
            .enumerate()
            .filter(|(i, (_, to))| to.iter().any(|&t| component[t] == component[*i]))
            .map(|(_, (name, _))| name.clone())
            .collect();
        match circular.is_empty() {
            true => Ok(Judgement { rules }),
            false => Err(circular),
        }
    }
    /// The pres and posts of the rule `t`: its own when it is written in
    /// place, its definition's when it is a name judgement knows.
    fn unfold<'a>(&'a self, t: &'a Prop) -> Option<(&'a [Prop], &'a [Prop])> {
        match t {
            Prop::Implies { pre, post, .. } => Some((pre, post)),
            Prop::Named(n) => self.rules.get(n).map(|d| (&d.pre[..], &d.post[..])),
            _ => None,
        }
    }
    /// The premises of the rule `t`: `t` itself, then its pres. `None`
    /// for a proposition that is no rule: a name judgement does not
    /// know, or a negation.
    pub fn premises(&self, t: &Prop) -> Option<Vec<Prop>> {
        let (pre, _) = self.unfold(t)?;
        Some([std::slice::from_ref(t), pre].concat())
    }
    /// The posts of the rule `t`. `None` for a proposition that is no
    /// rule.
    pub fn posts<'a>(&'a self, t: &'a Prop) -> Option<&'a [Prop]> {
        Some(self.unfold(t)?.1)
    }
    /// What a rule takes and gives, read from its parameters, when it
    /// is given `n` arguments: the propositions of its arguments, in
    /// order, and the proposition it gives an opinion of. `None` for an
    /// operation that is not a rule, a rule that does not apply, or one
    /// that would give or take an opinion of a proposition that is not
    /// well formed.
    fn rule(&self, op: &Op, n: usize) -> Option<(Vec<Prop>, Prop)> {
        let (from, gives) = match op {
            Op::Ponens { t, post } => (self.premises(t)?, self.posts(t)?.get(*post)?.clone()),
            Op::Tollens { t, post, blame } => {
                let mut premises = self.premises(t)?;
                let q = self.posts(t)?.get(*post)?.clone();
                if *blame >= premises.len() {
                    return None;
                }
                let blamed = premises.remove(*blame);
                ([vec![q], premises].concat(), blamed)
            }
            Op::Instance { q, at } => (vec![q.clone()], q.at(at)?),
            Op::Witness { q, at } => (vec![q.at(at)?], q.clone()),
            Op::Cases { q } => (q.cases(n)?, q.clone()),
            _ => return None,
        };
        let formed = gives.well_formed() && from.iter().all(Prop::well_formed);
        (from.len() == n && formed).then_some((from, gives))
    }
}

impl Standard for Judgement {
    type Type = Type;
    type Op = Op;
    fn binds(&self, op: &Op, i: usize, before: &[Type]) -> Vec<Type> {
        match (op, i, before) {
            (Op::Let, 1, [t]) => vec![t.clone()],
            _ => vec![],
        }
    }
    fn gives(&self, op: &Op, takes: &[Type]) -> Option<Type> {
        use Type::{Opinion, State};
        match (op, takes) {
            (Op::Not, [State]) | (Op::And | Op::Join, [State, State]) => Some(State),
            (Op::StateOf, [Opinion(_)]) => Some(State),
            (Op::Not, [Opinion(x)]) => Some(Opinion(x.not())),
            (Op::Join, [Opinion(x), Opinion(y)]) if x == y => Some(Opinion(x.clone())),
            (Op::Reductio { of }, [Opinion(_)]) if of.well_formed() => Some(Opinion(of.clone())),
            (
                Op::Ponens { .. }
                | Op::Tollens { .. }
                | Op::Instance { .. }
                | Op::Witness { .. }
                | Op::Cases { .. },
                takes,
            ) => {
                let (from, gives) = self.rule(op, takes.len())?;
                let matches = takes
                    .iter()
                    .zip(&from)
                    .all(|(t, p)| matches!(t, Opinion(x) if x == p));
                matches.then_some(Opinion(gives))
            }
            (Op::Let, [_, t]) => Some(t.clone()),
            _ => None,
        }
    }
}

type Judged = Outcome<Value, Infallible>;

impl Reading for Judgement {
    type Standard = Judgement;
    type Value = Value;
    type Fault = Infallible;

    fn has_type(&self, value: &Value, of: &Type) -> bool {
        match (value, of) {
            (Value::State(_), Type::State) => true,
            (Value::Opinion(o), Type::Opinion(x)) => &o.about == x,
            _ => false,
        }
    }

    fn apply(&self, op: &Op, args: &[Judged]) -> Judged {
        let args = strict(args)?;
        use Value::{Opinion as O, State as S};
        let opinion = match (op, args.as_slice()) {
            (Op::Not, [S(a)]) => return Ok(S(!*a)),
            (Op::And, [S(a), S(b)]) => return Ok(S(a.and(*b))),
            (Op::Join, [S(a), S(b)]) => return Ok(S(a.join(*b))),
            (Op::StateOf, [O(x)]) => return Ok(S(x.state())),
            (Op::Not, [O(x)]) => Opinion {
                about: x.about.not(),
                rests_for: x.rests_against.clone(),
                rests_against: x.rests_for.clone(),
            },
            (Op::Join, [O(x), O(y)]) if x.about == y.about => Opinion {
                about: x.about.clone(),
                rests_for: smallest(x.rests_for.iter().chain(&y.rests_for).cloned()),
                rests_against: smallest(x.rests_against.iter().chain(&y.rests_against).cloned()),
            },
            (Op::Reductio { of }, [O(x)]) if of.well_formed() => reductio(of, x),
            (
                Op::Ponens { .. }
                | Op::Tollens { .. }
                | Op::Instance { .. }
                | Op::Witness { .. }
                | Op::Cases { .. },
                args,
            ) => {
                let (from, gives) = self.rule(op, args.len()).ok_or(Fault::Stuck)?;
                let mut opinions = Vec::with_capacity(args.len());
                for (arg, prop) in args.iter().zip(&from) {
                    match arg {
                        O(x) if &x.about == prop => opinions.push(x),
                        _ => return Err(Fault::Stuck),
                    }
                }
                let (rests_for, rests_against) = match op {
                    Op::Ponens { .. } => {
                        let sides: Vec<_> = opinions.iter().map(|x| &x.rests_for).collect();
                        (all_of(&sides), Rests::new())
                    }
                    Op::Tollens { .. } => {
                        let (q, others) = opinions.split_first().ok_or(Fault::Stuck)?;
                        let sides: Vec<_> = [&q.rests_against]
                            .into_iter()
                            .chain(others.iter().map(|x| &x.rests_for))
                            .collect();
                        (Rests::new(), all_of(&sides))
                    }
                    Op::Instance { q, .. } | Op::Witness { q, .. } | Op::Cases { q } => {
                        quantified(op, matches!(q, Prop::All { .. }), &opinions)
                    }
                    _ => return Err(Fault::Stuck),
                };
                Opinion {
                    about: gives,
                    rests_for,
                    rests_against,
                }
            }
            _ => return Err(Fault::Stuck),
        };
        Ok(O(opinion))
    }

    fn bind(
        &self,
        op: &Op,
        args: &[Judged],
        run: &mut dyn FnMut(usize, &[Judged]) -> Judged,
    ) -> Judged {
        match (op, args) {
            (Op::Let, [value, _]) => run(1, std::slice::from_ref(value)),
            _ => Err(Fault::Stuck),
        }
    }
}

/// The rests of what `op` gives on these opinions, for an all when
/// `all`, else an any. Down from `q`: an instance is true when an all
/// is, false when an any is. Up from one point: a witness makes an all
/// false, an any true. Up from every point: the cases tell all of both.
fn quantified(op: &Op, all: bool, opinions: &[&Opinion]) -> (Rests, Rests) {
    let one = |side: fn(&Opinion) -> &Rests| {
        smallest(opinions.iter().flat_map(|x| side(x).iter().cloned()))
    };
    let every = |side: fn(&Opinion) -> &Rests| {
        all_of(&opinions.iter().map(|x| side(x)).collect::<Vec<_>>())
    };
    let holds: fn(&Opinion) -> &Rests = Opinion::rests_for;
    let fails: fn(&Opinion) -> &Rests = Opinion::rests_against;
    match (op, all) {
        (Op::Instance { .. }, true) | (Op::Witness { .. }, false) => (one(holds), Rests::new()),
        (Op::Instance { .. }, false) | (Op::Witness { .. }, true) => (Rests::new(), one(fails)),
        (_, true) => (every(holds), one(fails)),
        (_, false) => (one(holds), every(fails)),
    }
}

/// Each way `x` follows true, with each way it follows false, is a set
/// of takings that cannot all hold. For each taking of `of` in it, the
/// rest of the set says that taking is wrong.
fn reductio(of: &Prop, x: &Opinion) -> Opinion {
    let (mut wrong_true, mut wrong_false) = (Rests::new(), Rests::new());
    for yes in &x.rests_for {
        for no in &x.rests_against {
            let cannot: BTreeSet<Taking> = yes.union(no).cloned().collect();
            for taking in cannot.iter().filter(|t| &t.prop == of) {
                let mut rest = cannot.clone();
                rest.remove(taking);
                match taking.value {
                    true => wrong_true.insert(rest),
                    false => wrong_false.insert(rest),
                };
            }
        }
    }
    Opinion {
        about: of.clone(),
        rests_for: smallest(wrong_false),
        rests_against: smallest(wrong_true),
    }
}
