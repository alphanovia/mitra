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

use crate::expression::{strict, Fault, Outcome, Reading, Standard};
use crate::graph::components;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;

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
            }
        }
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
/// rules, so every set it holds is one the proposition follows from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Opinion {
    about: Prop,
    rests_for: Rests,
    rests_against: Rests,
}

impl Opinion {
    /// What these takings of `about` say of it: each taking it true is
    /// a set it follows true from, each taking it false one it follows
    /// false from. `None` when a taking is of another proposition.
    pub fn taken(about: Prop, takings: Vec<Taking>) -> Option<Opinion> {
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
            Prop::Not(_) => None,
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
    /// What a rule takes and gives, read from `t`: the propositions of
    /// its arguments, in order, and the proposition it gives an opinion
    /// of. `None` for an operation that is not a rule, or a rule that
    /// does not apply to `t`.
    fn rule(&self, op: &Op) -> Option<(Vec<Prop>, Prop)> {
        match op {
            Op::Ponens { t, post } => Some((self.premises(t)?, self.posts(t)?.get(*post)?.clone())),
            Op::Tollens { t, post, blame } => {
                let mut premises = self.premises(t)?;
                let q = self.posts(t)?.get(*post)?.clone();
                if *blame >= premises.len() {
                    return None;
                }
                let blamed = premises.remove(*blame);
                Some(([vec![q], premises].concat(), blamed))
            }
            _ => None,
        }
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
            (Op::Reductio { of }, [Opinion(_)]) => Some(Opinion(of.clone())),
            (Op::Ponens { .. } | Op::Tollens { .. }, takes) => {
                let (from, gives) = self.rule(op)?;
                let matches = takes.len() == from.len()
                    && takes
                        .iter()
                        .zip(&from)
                        .all(|(t, p)| t == &Opinion(p.clone()));
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
            (Op::Reductio { of }, [O(x)]) => reductio(of, x),
            (Op::Ponens { .. } | Op::Tollens { .. }, args) => {
                let (from, gives) = self.rule(op).ok_or(Fault::Stuck)?;
                if args.len() != from.len() {
                    return Err(Fault::Stuck);
                }
                let mut opinions = Vec::with_capacity(args.len());
                for (arg, prop) in args.iter().zip(&from) {
                    match arg {
                        O(x) if &x.about == prop => opinions.push(x),
                        _ => return Err(Fault::Stuck),
                    }
                }
                match op {
                    Op::Ponens { .. } => {
                        let sides: Vec<_> = opinions.iter().map(|x| &x.rests_for).collect();
                        Opinion {
                            about: gives,
                            rests_for: all_of(&sides),
                            rests_against: Rests::new(),
                        }
                    }
                    _ => {
                        let (q, others) = opinions.split_first().ok_or(Fault::Stuck)?;
                        let sides: Vec<_> = [&q.rests_against]
                            .into_iter()
                            .chain(others.iter().map(|x| &x.rests_for))
                            .collect();
                        Opinion {
                            about: gives,
                            rests_for: Rests::new(),
                            rests_against: all_of(&sides),
                        }
                    }
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
