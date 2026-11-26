//! The simulation standard (`docs/design/core2.md`): truths, whole numbers,
//! reals and records, read two ways. `Exact` runs the model on inputs
//! and gives values. `Ranges` runs it on bounds of the inputs and gives
//! bounds every exact value lies in (Cousot and Cousot, 1977).
//!
//! Two operations bind a name: `let` and `all`. In both readings `let`
//! shares (S6): `let x = a; b` gives what `b` with `a` written in place
//! of `x` gives, so a fault of `a` counts only where `x` is read.
//! `(lo..hi).all(|x| p)` reads `p` at each number from `lo` to `hi - 1`
//! in turn, as `&&` does. By bounds, when both ends are one number each
//! it does the same; else it reads `p` once, with `x` anywhere the range
//! may reach, and allows true when the range may be empty.
//!
//! Numbers are of the world: a whole number is any integer and a real
//! is any ratio of two, both exact and unbounded. Nothing overflows;
//! the one fault is a division by zero.
//!
//! `Ranges` bounds `Exact` (the covering of `expression`), with γ given by
//! `covers`: for every checked expression, bounds that fit the inputs'
//! types and exact values in γ of them, the exact outcome is in γ of
//! the bounded one.
//!
//! A call names a function of the model by its path, `sun::light()`.
//! The standard and both readings hold the same `Calls`: what each
//! function takes and gives, and its body, or rows of what it gives at
//! points when it has none. `Exact` runs the body, or reads a row; with
//! neither, a call has no value (`Undefined::Unknown`). `Ranges` runs the
//! body, or else allows any value of the type, or none. Calls never go
//! round: `Calls::with` refuses a function that calls itself, however
//! far. A call takes the values of its arguments, as Rust's does: a
//! fault in one is the call's.
//!
//! A type parameter, `T` or `T: Number`, is a type a contract names but
//! does not fix. A contract is checked once with it; no value is of it.

use crate::expression::{check, eval, strict, Expression, Fault, Outcome, Reading, Standard};
use crate::graph::components;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// A ratio of two integers, in lowest terms, its lower part above zero.
/// Made only by `new`, `integer` and the operations, so there is no
/// other kind.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Rational(BigRational);

impl Rational {
    /// `numer / denom`. `None` when `denom` is zero.
    pub fn new(numer: impl Into<BigInt>, denom: impl Into<BigInt>) -> Option<Rational> {
        let denom = denom.into();
        match denom.is_zero() {
            true => None,
            false => Some(Rational(BigRational::new(numer.into(), denom))),
        }
    }
    pub fn integer(n: impl Into<BigInt>) -> Rational {
        Rational(BigRational::from_integer(n.into()))
    }
    pub fn numer(&self) -> &BigInt {
        self.0.numer()
    }
    pub fn denom(&self) -> &BigInt {
        self.0.denom()
    }
    pub fn is_integer(&self) -> bool {
        self.0.is_integer()
    }
    pub fn plus(&self, other: &Rational) -> Rational {
        Rational(&self.0 + &other.0)
    }
    pub fn minus(&self, other: &Rational) -> Rational {
        Rational(&self.0 - &other.0)
    }
    pub fn times(&self, other: &Rational) -> Rational {
        Rational(&self.0 * &other.0)
    }
    /// `None` when `other` is zero.
    pub fn over(&self, other: &Rational) -> Option<Rational> {
        match other.0.is_zero() {
            true => None,
            false => Some(Rational(&self.0 / &other.0)),
        }
    }
    /// The greatest whole number at most this one.
    fn floor(&self) -> Rational {
        Rational(self.0.floor())
    }
    /// The least whole number at least this one.
    fn ceil(&self) -> Rational {
        Rational(self.0.ceil())
    }
    /// The whole number nearest zero of those between zero and this one:
    /// Rust's rounding of `/` on integers.
    fn toward_zero(&self) -> Rational {
        Rational(self.0.trunc())
    }
    /// This one, when whole.
    fn to_integer(&self) -> Option<BigInt> {
        self.is_integer().then(|| self.0.to_integer())
    }
    pub fn abs(&self) -> Rational {
        Rational(self.0.abs())
    }
    pub fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

/// A record: its name and its fields, each a name and a type. Known by
/// all of it: two records of one name and other fields are two types.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Record {
    pub name: String,
    /// The types its parameters are given, `Pair<Int, Real>`: already
    /// written in its fields. Empty for a record with none.
    pub args: Vec<Type>,
    pub fields: Vec<(String, Type)>,
}

/// An enum, Rust's: a value that is one of its variants, each with the
/// values it holds, `enum Option<T> { None, Some(T) }`. Known by its
/// name and the types its parameters are given: `Option<Int>` and
/// `Option<Real>` are two types. One that holds itself (a tree) is cut
/// where it holds itself: there it has no variant written, and is the
/// same type, by name (trees, the user's go 12:33).
#[derive(Clone, Debug)]
pub struct Enum {
    pub name: String,
    /// The types its parameters are given, already written in its
    /// variants.
    pub args: Vec<Type>,
    /// Each variant: its name and the types of what it holds, in order.
    pub variants: Vec<(String, Vec<Type>)>,
}

impl PartialEq for Enum {
    fn eq(&self, other: &Enum) -> bool {
        self.name == other.name && self.args == other.args
    }
}

impl Eq for Enum {}

impl PartialOrd for Enum {
    fn partial_cmp(&self, other: &Enum) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Enum {
    fn cmp(&self, other: &Enum) -> std::cmp::Ordering {
        (&self.name, &self.args).cmp(&(&other.name, &other.args))
    }
}

impl std::hash::Hash for Enum {
    fn hash<H: std::hash::Hasher>(&self, h: &mut H) {
        self.name.hash(h);
        self.args.hash(h);
    }
}

impl Enum {
    /// Cut where it holds itself: no variant written here; read it from
    /// the enum of that name around it.
    pub fn cut(&self) -> bool {
        self.variants.is_empty()
    }

    /// The place of the variant named `name`.
    pub fn variant(&self, name: &str) -> Option<usize> {
        self.variants.iter().position(|(n, _)| n == name)
    }
}

/// The type of a tuple `(a, b, …)`: a record with no name, its fields
/// named `0`, `1`, … in order, so that `t.0` reads the first. No
/// declared record has no name.
pub fn tuple(of: Vec<Type>) -> Type {
    let fields = of.into_iter().enumerate();
    let fields = fields.map(|(i, t)| (i.to_string(), t)).collect();
    Type::Record(Record {
        name: String::new(),
        args: vec![],
        fields,
    })
}

impl Record {
    /// The place and type of the first field named `name`.
    pub fn field(&self, name: &str) -> Option<(usize, &Type)> {
        let place = self.fields.iter().position(|(n, _)| n == name);
        // `.1` of a labelled tuple: by place.
        let place = match place {
            Some(p) => p,
            None if self.name.is_empty() => name.parse().ok().filter(|p| *p < self.fields.len())?,
            None => return None,
        };
        Some((place, &self.fields[place].1))
    }
    /// A tuple whose values have labels, `(sum: integer, n: integer)`.
    pub fn labelled(&self) -> bool {
        self.name.is_empty()
            && !self.fields.is_empty()
            && self.fields.iter().all(|(n, _)| n.parse::<usize>().is_err())
    }
}

/// What a loop carries, as the names its block binds: each value of a
/// labelled tuple, else the one value.
pub fn carried(start: &Type) -> Vec<Type> {
    match start {
        Type::Record(r) if r.labelled() => r.fields.iter().map(|(_, t)| t.clone()).collect(),
        t => vec![t.clone()],
    }
}

/// The next values a loop's block gives fit what it carries: the same
/// type, or the values of a labelled tuple, by place, in a plain one.
pub fn next_fits(start: &Type, next: &Type) -> bool {
    match (start, next) {
        (Type::Record(r), Type::Record(n)) if r.labelled() && n.name.is_empty() => {
            r.fields.len() == n.fields.len()
                && r.fields
                    .iter()
                    .zip(&n.fields)
                    .all(|((_, a), (_, b))| a == b)
        }
        _ => start == next,
    }
}

/// What a value of the simulation can be. A record is not a name: a
/// record named "rational" is not `rational`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    Truth,
    Int,
    Real,
    Record(Record),
    Enum(Enum),
    /// A sequence of values of one type, `[T]`: Rust's slice form. Its
    /// length is its own; it is not changed, only read (finding 3, step 2).
    Seq(Box<Type>),
    /// An encoded sequence, `[~T]`: its storage is bytes, which decode
    /// to values of `T`. Text is `[~char]` (UTF-8). Its length counts
    /// bytes, a place is a `byte`; `.decoded()` gives the `[T]`
    /// (finding 3, step 3; your "go", 2026-11-24).
    Encoded(Box<Type>),
    /// One grapheme, what a reader sees as one letter: opaque, with `==`
    /// and `<` in the order of its bytes. No value is written yet.
    Char,
    /// One byte of an encoded sequence: opaque, `==` only.
    Byte,
    /// A type a contract, a record or an enum names but does not fix, with its bound: `T`, or
    /// `T: Number`, a whole number or a real. No value is of it.
    Param {
        name: String,
        bound: Bound,
    },
}

/// What a type parameter promises: nothing but `==`, or a number, with
/// `+`, `-`, `*`, `<`, `<=` and `within`. Rust's `T` and `T: Number`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Bound {
    Any,
    Number,
}

impl Bound {
    /// `t` keeps the promise: any type, or `Int`, `Real` or a parameter
    /// bound by `Number`, for `Number`.
    pub fn admits(&self, t: &Type) -> bool {
        match self {
            Bound::Any => true,
            Bound::Number => number(t),
        }
    }
}

impl Type {
    /// `self` with each parameter `given` names replaced by its type, in
    /// records too. By name only: whether the type is one the bound
    /// admits (`Bound::admits`) is the caller's to ask.
    pub fn given(&self, given: &BTreeMap<String, Type>) -> Type {
        match self {
            Type::Param { name, .. } => given.get(name).cloned().unwrap_or_else(|| self.clone()),
            Type::Record(r) => Type::Record(Record {
                name: r.name.clone(),
                args: r.args.iter().map(|t| t.given(given)).collect(),
                fields: r
                    .fields
                    .iter()
                    .map(|(n, t)| (n.clone(), t.given(given)))
                    .collect(),
            }),
            Type::Enum(e) => Type::Enum(Enum {
                name: e.name.clone(),
                args: e.args.iter().map(|t| t.given(given)).collect(),
                variants: e
                    .variants
                    .iter()
                    .map(|(n, ts)| (n.clone(), ts.iter().map(|t| t.given(given)).collect()))
                    .collect(),
            }),
            Type::Seq(t) => Type::Seq(Box::new(t.given(given))),
            Type::Encoded(t) => Type::Encoded(Box::new(t.given(given))),
            _ => self.clone(),
        }
    }
    /// It names a type parameter, in a record too.
    pub fn open(&self) -> bool {
        match self {
            Type::Param { .. } => true,
            Type::Record(r) => r.fields.iter().any(|(_, t)| t.open()),
            Type::Enum(e) => e.variants.iter().any(|(_, ts)| ts.iter().any(Type::open)),
            Type::Seq(t) | Type::Encoded(t) => t.open(),
            _ => false,
        }
    }
    /// Learns what each parameter of `self` stands for, from `actual`,
    /// a type of the same shape: `Option<T>` against `Option<Int>`
    /// learns `T` is `Int`. False when the two differ elsewhere, or a
    /// parameter would stand for two types.
    pub fn learn(&self, actual: &Type, learnt: &mut BTreeMap<String, Type>) -> bool {
        match (self, actual) {
            (Type::Param { name, .. }, t) => match learnt.get(name) {
                Some(known) => known == t,
                None => {
                    learnt.insert(name.clone(), t.clone());
                    true
                }
            },
            (Type::Record(a), Type::Record(b)) => {
                a.name == b.name
                    && a.fields.len() == b.fields.len()
                    && a.fields
                        .iter()
                        .zip(&b.fields)
                        .all(|((n, x), (m, y))| n == m && x.learn(y, learnt))
            }
            (Type::Enum(a), Type::Enum(b)) => {
                a.name == b.name
                    && a.variants.len() == b.variants.len()
                    && a.variants
                        .iter()
                        .zip(&b.variants)
                        .all(|((n, xs), (m, ys))| {
                            n == m
                                && xs.len() == ys.len()
                                && xs.iter().zip(ys).all(|(x, y)| x.learn(y, learnt))
                        })
            }
            (Type::Seq(a), Type::Seq(b)) | (Type::Encoded(a), Type::Encoded(b)) => {
                a.learn(b, learnt)
            }
            (a, b) => a == b,
        }
    }
}

/// An exact value. Its order is only for keeping values in sets: it
/// is not the order of numbers between kinds.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Truth(bool),
    Int(BigInt),
    Real(Rational),
    /// A record and the value of each field, in order.
    Record(Record, Vec<Value>),
    /// A record known in part, `Location { depth: 2, .. }`: each field's
    /// value, `None` where it is not known, at least one. Made by
    /// `Value::partial`. Reading a field not known has no value
    /// (`Undefined::Unknown`), nor has `==` that only it decides.
    Partial(Record, Vec<Option<Value>>),
    /// A variant of an enum, by its place, and what it holds.
    Variant(Enum, usize, Vec<Value>),
    /// A sequence: the type of what it holds, and each value in order.
    Seq(Type, Vec<Value>),
    /// Text, `[~char]`: its bytes, UTF-8.
    Text(String),
    /// One byte of an encoded sequence.
    Byte(u8),
}

impl Value {
    /// A record of these fields: known in part when one is `None`.
    pub fn partial(r: Record, fields: Vec<Option<Value>>) -> Value {
        match fields.iter().all(Option::is_some) {
            true => Value::Record(r, fields.into_iter().flatten().collect()),
            false => Value::Partial(r, fields),
        }
    }
}

/// A call of `f` at `args`, each bound to a name of its own: a value
/// known in part has no expression that gives it.
pub fn call_at(f: &str, args: &[Value]) -> (Expression<Op>, BTreeMap<String, Value>) {
    let names: Vec<String> = (0..args.len()).map(|i| format!("#{i}")).collect();
    let call = Expression::Apply(
        Op::Call(f.to_string()),
        names
            .iter()
            .map(|n| crate::expression::Arg {
                binds: vec![],
                body: Expression::Name(n.clone()),
            })
            .collect(),
    );
    (call, names.into_iter().zip(args.iter().cloned()).collect())
}

/// Whether `a` and `b` are one value: `Err` with the path of a field not
/// known, `R::f`, when only fields not known could tell; false when a
/// known field differs, whatever the others.
pub fn same(a: &Value, b: &Value) -> Result<bool, String> {
    alike(a, b, false)
}

/// `same`, of a value and what an example or a measurement says of it:
/// the fields a record with `..` does not state are not compared, since
/// the model says nothing of them (finding 19).
pub fn matches(got: &Value, said: &Value) -> Result<bool, String> {
    alike(got, said, true)
}

fn alike(a: &Value, b: &Value, rest_free: bool) -> Result<bool, String> {
    let same = |x: &Value, y: &Value| alike(x, y, rest_free);
    let fields = |v: &Value| -> Option<(Record, Vec<Option<Value>>)> {
        match v {
            Value::Record(r, vs) => Some((r.clone(), vs.iter().cloned().map(Some).collect())),
            Value::Partial(r, vs) => Some((r.clone(), vs.clone())),
            _ => None,
        }
    };
    let held = match (a, b) {
        (Value::Variant(e, i, xs), Value::Variant(f, j, ys)) => {
            Some((e == f && i == j && xs.len() == ys.len(), xs, ys))
        }
        (Value::Seq(t, xs), Value::Seq(u, ys)) => Some((t == u && xs.len() == ys.len(), xs, ys)),
        _ => None,
    };
    if let Some((alike, xs, ys)) = held {
        if !alike {
            return Ok(false);
        }
        let mut unknown = None;
        for (x, y) in xs.iter().zip(ys) {
            match same(x, y) {
                Ok(true) => {}
                Ok(false) => return Ok(false),
                Err(g) => _ = unknown.get_or_insert(g),
            }
        }
        return match unknown {
            Some(g) => Err(g),
            None => Ok(true),
        };
    }
    let (Some((r, xs)), Some((t, ys))) = (fields(a), fields(b)) else {
        return Ok(a == b);
    };
    if r != t || xs.len() != ys.len() {
        return Ok(false);
    }
    let mut unknown = None;
    for (i, (x, y)) in xs.iter().zip(&ys).enumerate() {
        let field = || format!("{}::{}", r.name, r.fields[i].0);
        if rest_free && r.fields[i].0 == crate::model::REST {
            continue;
        }
        match (x, y) {
            (Some(x), Some(y)) => match same(x, y) {
                Ok(true) => {}
                Ok(false) => return Ok(false),
                Err(g) => _ = unknown.get_or_insert(g),
            },
            _ => _ = unknown.get_or_insert_with(field),
        }
    }
    match unknown {
        Some(g) => Err(g),
        None => Ok(true),
    }
}

/// Why an operation has no value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Undefined {
    DivisionByZero,
    /// A call of this function has no value here: it has no body, and no
    /// row gives one at these arguments.
    Unknown(String),
    /// A budget was reached before the value was: what it was, and where.
    Budget(String),
    /// A value outside the range of its type, at a call: what, and where.
    Outside(String),
    /// `xs[i]` with no value at `i`: below zero, or at or past its length.
    Index(String),
    /// An `until` whose carried values came back to ones it had already
    /// carried, every one known: it can never end (finding 39 G). The
    /// function whose body holds it, once known, the values, and the
    /// turns between the two times.
    EndlessRepeat {
        inside: Option<String>,
        state: String,
        turns: usize,
    },
}

impl Undefined {
    /// What an endless loop says: where it is, the state that came back.
    /// `here`, the function read, when no body it calls holds the loop.
    pub fn endless(&self, here: Option<&str>) -> Option<String> {
        let Undefined::EndlessRepeat {
            inside,
            state,
            turns,
        } = self
        else {
            return None;
        };
        let place = match inside.as_deref().or(here) {
            Some(f) => format!("the `until` in `{f}`"),
            None => "an `until`".to_string(),
        };
        let turns = match turns {
            1 => "1 turn".to_string(),
            n => format!("{n} turns"),
        };
        Some(format!(
            "{place} never ends: what it carries, {state}, came back after {turns}"
        ))
    }
}

/// An endless `until` met in the body of `f`, said to be there, when
/// no deeper body held it.
fn held_in(f: &str, gives: Exactly) -> Exactly {
    match gives {
        Err(Fault::Op(Undefined::EndlessRepeat {
            inside: None,
            state,
            turns,
        })) => Err(Fault::Op(Undefined::EndlessRepeat {
            inside: Some(f.to_string()),
            state,
            turns,
        })),
        gives => gives,
    }
}

/// Whether `v` is known all through: no record known only in part.
pub fn known(v: &Value) -> bool {
    match v {
        Value::Partial(..) => false,
        Value::Record(_, vs) | Value::Variant(_, _, vs) | Value::Seq(_, vs) => vs.iter().all(known),
        _ => true,
    }
}

/// Whole numbers from `low` to `high`, both in: the type `0..=9`, an
/// `Int` that is checked (finding 11). Rust writes the same range of
/// numbers `0..=9`; as a type it is Ada's `range 0 .. 9`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Range {
    pub low: BigInt,
    pub high: BigInt,
}

impl Range {
    /// `value` is a whole number in the range.
    pub fn holds(&self, value: &Value) -> bool {
        matches!(value, Value::Int(n) if &self.low <= n && n <= &self.high)
    }
    /// The bounds of its numbers; `None` when it has none.
    pub fn bounds(&self) -> Option<Bounds> {
        Bounds::between(
            Numbers::Int,
            Rational::integer(self.low.clone()),
            Rational::integer(self.high.clone()),
        )
    }
    /// `b` cut to the range: `None` when no number of `b` is in it; and
    /// whether some of `b` is outside it.
    fn cut(&self, b: &Bounds) -> Option<(Bounds, bool)> {
        let Span::Number(Numbers::Int, low, high) = &b.span else {
            return Some((b.clone(), true));
        };
        let (rl, rh) = (
            Rational::integer(self.low.clone()),
            Rational::integer(self.high.clone()),
        );
        let outside = End::low(low) < End::At(rl.clone()) || End::At(rh.clone()) < End::high(high);
        let low = low.clone().map_or(rl.clone(), |l| l.max(rl));
        let high = high.clone().map_or(rh.clone(), |h| h.min(rh));
        (low <= high).then(|| {
            let span = Span::Number(Numbers::Int, Some(low), Some(high));
            (
                Bounds {
                    span,
                    may_fault: b.may_fault,
                },
                outside,
            )
        })
    }
}

impl std::fmt::Display for Range {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}..={}", self.low, self.high)
    }
}

/// How far one computation through calls may go, each a setting: the
/// most calls from outside one call, and the most levels of the bodies
/// active at once. A reading that reaches one gives `Undefined::Budget`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Budget {
    pub steps: usize,
    pub depth: usize,
}

/// A million calls; 1,000 levels, what `expression` says a thread of
/// 2 MiB holds in a debug build.
impl Default for Budget {
    fn default() -> Budget {
        Budget {
            steps: 100_000,
            depth: 1_000,
        }
    }
}

/// The calls of one function `Ranges` has read: arguments, and outcome.
type Seen = Vec<(Vec<Bounds>, Bounded)>;

/// What the readings remember of calls: a call's outcome at its
/// arguments, each body's height, and the count of steps and levels of
/// the computation under way. It never changes an answer, so two are
/// always equal, and a copy starts empty.
#[derive(Default)]
struct Memo {
    exact: std::sync::Mutex<BTreeMap<(String, Vec<Value>), Exactly>>,
    ranges: std::sync::Mutex<BTreeMap<String, Seen>>,
    heights: std::sync::Mutex<BTreeMap<String, usize>>,
    depth: std::sync::atomic::AtomicUsize,
    steps: std::sync::atomic::AtomicUsize,
    /// The promises `Ranges` read since `Calls::read` last took them.
    read: std::sync::Mutex<BTreeSet<String>>,
    /// The functions whose promises `Ranges` is reading now: a call of
    /// one of them inside its own promise is only its type.
    promising: std::sync::Mutex<BTreeSet<String>>,
    /// The calls `Ranges` is reading now, by function and bounds: one
    /// of them again inside itself (a tree's part read as any tree)
    /// gives any value, which may fault, not a computation that only a
    /// budget ends.
    reading: std::sync::Mutex<Vec<(String, Vec<Bounds>)>>,
}

impl Clone for Memo {
    fn clone(&self) -> Memo {
        Memo::default()
    }
}

impl PartialEq for Memo {
    fn eq(&self, _: &Memo) -> bool {
        true
    }
}

impl Eq for Memo {}

impl std::fmt::Debug for Memo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("Memo")
    }
}

fn plain_arg(body: Expression<Op>) -> crate::expression::Arg<Op> {
    crate::expression::Arg {
        binds: vec![],
        body,
    }
}

/// The levels of `e`.
fn height<Op>(e: &Expression<Op>) -> usize {
    match e {
        Expression::Name(_) => 1,
        Expression::Apply(_, args) => 1 + args.iter().map(|a| height(&a.body)).max().unwrap_or(0),
    }
}

/// One call under way: its levels counted until it ends.
struct Active<'m> {
    memo: &'m Memo,
    levels: usize,
}

impl Drop for Active<'_> {
    fn drop(&mut self) {
        self.memo
            .depth
            .fetch_sub(self.levels, std::sync::atomic::Ordering::SeqCst);
    }
}

/// The operations, in Rust's form.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    /// Constants: `true`, `3`, `1/3`.
    Truth(bool),
    Int(BigInt),
    Real(Rational),
    /// `!a`, `a && b`, `a || b`: `&&` and `||` drop what `b` gives,
    /// fault included, when `a` decides. `Exact` does not compute `b`
    /// then; `Ranges` computes both.
    Not,
    And,
    Or,
    /// `a + b`, `a - b`, `a * b` on two `Int` or two `Real`.
    Add,
    Sub,
    Mul,
    /// `a / b` on two `Int`, rounded toward zero as Rust does (`-7 / 2`
    /// is -3), or on two `Real`, exactly. No value when `b` is zero.
    Div,
    /// `a % b` on two `Int`: what `a / b` leaves, of the sign of `a`, as
    /// Rust does (`-7 % 2` is -1). No value when `b` is zero.
    Rem,
    /// `a < b`, `a <= b` on two `Int` or two `Real`.
    Less,
    AtMost,
    /// `a == b` on two values of one type.
    Equal,
    /// `within(a, b, tol)`: `|a - b| <= tol`, on three `Int` or three
    /// `Real`. Rust would write `(a - b).abs() <= tol`.
    Within,
    /// `if c { a } else { b }`: gives what the branch `c` takes gives,
    /// and drops the other, fault included. `Exact` computes only the
    /// branch taken; `Ranges` both.
    If,
    /// `R { x: a, y: b }`, the fields in the record's order.
    Make(Record),
    /// `r.x`: the first field named `x`.
    Field(String),
    /// `let x = a; b`: `b`, with `x` standing for what `a` gives. The
    /// second argument binds `x`.
    Let,
    /// `(lo..hi).all(|x| p)`: `p` holds for every whole number `x` from
    /// `lo` up to `hi`, `hi` left out. The third argument binds `x`, an
    /// `Int`, and is read for each number in turn, as `&&` reads.
    All,
    /// `sun::light(a, b)`: the function at this path, at these arguments.
    Call(String),
    /// `Some(a)`: the variant at this place of the enum, holding what its
    /// arguments give.
    Variant(Enum, usize),
    /// `match e { None => a, Some(x) => b }`: what the arm of the variant
    /// `e` gives gives, the others dropped. The first argument is `e`;
    /// then one arm per variant, in the enum's order, each binding what
    /// its variant holds. Rust's `match`, every variant covered.
    Match(Enum),
    /// `[a, b, c]`: a sequence of what its arguments give, each of this
    /// type; `[]` holds none.
    Seq(Type),
    /// `xs.len()`: how many values a sequence holds, an `Int`.
    Len,
    /// `xs[i]`: the value at place `i` of a sequence, from 0. No value
    /// below 0, or at its length or past it, as Rust's `xs[i]` panics.
    /// Of text, the byte at place `i`.
    Index,
    /// `"abc"`: text, encoded.
    Text(String),
    /// `a ++ b`: the values of `a`, then those of `b`; of text, its bytes.
    Join,
    /// `a.starts_with(b)`: the first values of `a` are those of `b`; of
    /// text, its first bytes.
    StartsWith,
    /// `xs.decoded()`: the sequence an encoded one decodes to; a plain
    /// one is its own. Text waits for the grapheme decoder.
    Decoded,
    /// `xs.fold((sum: 0, n: 0)) { x -> (sum + x, n + 1) }`: the values
    /// carried, from the second argument, through each value of the
    /// sequence `xs` in turn. The third argument binds the value, then
    /// each label of the carried tuple (or the one value carried), and
    /// gives the next values, by place. Gives the last carried.
    Fold,
    /// `until((h: s, got: g), done) { (h2, g2) }`: from the first
    /// argument, the next values while `done` does not hold. The second
    /// and third arguments bind the labels. Gives the first carried
    /// where `done` holds; each turn is a step of the budget.
    Until,
}

/// A function a call can name: what it takes, each by name, what it
/// gives, and its body; or, with no body, rows of what it gives at
/// points (its examples, observations). With neither it is opaque.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Callee {
    /// With no body: what its claims promise of what it gives, read by
    /// bounds as bounds on it. `None` for none.
    pub promises: Option<Promises>,
    pub takes: Vec<(String, Type)>,
    pub gives: Type,
    pub body: Option<Expression<Op>>,
    pub rows: Vec<(Vec<Value>, Value)>,
    /// The ranges of what it takes and gives: a call with a value
    /// outside one has none.
    pub ranges: Ranged,
}

/// The ranges of a function's types: of what it takes, by name, and of
/// what it gives, with its name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ranged {
    pub takes: BTreeMap<String, Range>,
    pub gives: Option<(String, Range)>,
}

/// What a function with no body promises: its claims, each by its path,
/// over what it takes and `gives`, the name of what it gives.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Promises {
    pub gives: String,
    pub claims: Vec<(String, Expression<Op>)>,
}

/// A row `given f(args) -> v`: each argument written, or `None` for `_`.
type GivenRow = (Vec<Option<Value>>, Value);

/// The functions calls can name, each by its path. Only `with` makes
/// one with functions, and it checks them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Calls {
    callees: BTreeMap<String, Callee>,
    /// What a reading is given, by path, at points: a value of the world
    /// a measurement or an example of the model saw, of a function with
    /// no body. Read before its rows, and never compared with them here.
    /// An argument `None` is any value (`_`, finding 30).
    given: BTreeMap<String, Vec<GivenRow>>,
    /// How far a computation through calls may go.
    budget: Budget,
    memo: Box<Memo>,
}

/// The inputs every call the body of the function at `path` makes to
/// itself takes a part of, at that input's place, taken apart by a
/// `match`: structural recursion on any of them ends (trees, the user's
/// go 12:33). Lean and Coq take recursion on this guard, one input for
/// all the calls. Every input when it calls itself nowhere.
pub fn decreasing(takes: &[(String, Type)], body: &Expression<Op>, path: &str) -> BTreeSet<usize> {
    /// A name in scope: an input at its place, a part of one, or other.
    #[derive(Clone, Copy, PartialEq)]
    enum Is {
        Input(usize),
        Part(usize),
        Other,
    }
    type Scope = Vec<(String, Is)>;
    fn is(scope: &Scope, e: &Expression<Op>) -> Is {
        match e {
            Expression::Name(n) => scope
                .iter()
                .rev()
                .find(|(m, _)| m == n)
                .map_or(Is::Other, |(_, k)| *k),
            _ => Is::Other,
        }
    }
    fn walk(e: &Expression<Op>, scope: &mut Scope, path: &str, left: &mut BTreeSet<usize>) {
        let Expression::Apply(op, args) = e else {
            return;
        };
        if matches!(op, Op::Call(g) if g == path) {
            left.retain(|&k| {
                args.get(k)
                    .is_some_and(|a| a.binds.is_empty() && is(scope, &a.body) == Is::Part(k))
            });
        }
        // What a match takes apart from an input, or from a part of
        // one, is a part of that input.
        let taken = match (op, args.first()) {
            (Op::Match(_), Some(a)) => match is(scope, &a.body) {
                Is::Input(k) | Is::Part(k) => Is::Part(k),
                Is::Other => Is::Other,
            },
            _ => Is::Other,
        };
        for (i, a) in args.iter().enumerate() {
            let depth = scope.len();
            let bound = match (op, i) {
                (Op::Match(_), i) if i >= 1 => taken,
                _ => Is::Other,
            };
            scope.extend(a.binds.iter().map(|b| (b.clone(), bound)));
            walk(&a.body, scope, path, left);
            scope.truncate(depth);
        }
    }
    let mut scope: Scope = takes
        .iter()
        .enumerate()
        .map(|(k, (n, _))| (n.clone(), Is::Input(k)))
        .collect();
    let mut left: BTreeSet<usize> = (0..takes.len()).collect();
    walk(body, &mut scope, path, &mut left);
    left
}

/// Its calls to itself all end: structural recursion on one input.
pub fn structural(takes: &[(String, Type)], body: &Expression<Op>, path: &str) -> bool {
    !decreasing(takes, body, path).is_empty()
}

impl Calls {
    /// These functions, with this budget.
    pub fn budget(mut self, budget: Budget) -> Calls {
        self.budget = budget;
        self.memo = Box::default();
        self
    }
    /// The promises `Ranges` read as bounds since this was last asked,
    /// by path: what a result by bounds is given.
    pub fn read(&self) -> BTreeSet<String> {
        std::mem::take(&mut *self.memo.read.lock().unwrap_or_else(|e| e.into_inner()))
    }
    /// Enter a call of `f`: `Err` when it would pass the budget. The
    /// first call from outside starts the count of steps.
    fn enter(&self, f: &str) -> Result<Active<'_>, Fault<Undefined>> {
        use std::sync::atomic::Ordering::SeqCst;
        let levels = {
            let mut heights = self.memo.heights.lock().unwrap_or_else(|e| e.into_inner());
            *heights.entry(f.to_string()).or_insert_with(|| {
                self.callees
                    .get(f)
                    .and_then(|c| c.body.as_ref())
                    .map_or(1, height)
            })
        };
        let before = self.memo.depth.fetch_add(levels, SeqCst);
        let active = Active {
            memo: &self.memo,
            levels,
        };
        if before == 0 {
            self.memo.steps.store(0, SeqCst);
        }
        let steps = self.memo.steps.fetch_add(1, SeqCst) + 1;
        if before + levels > self.budget.depth {
            return Err(Fault::Op(Undefined::Budget(format!(
                "the depth budget, {} levels of bodies through calls, reached at `{f}`",
                self.budget.depth
            ))));
        }
        if steps > self.budget.steps {
            return Err(Fault::Op(Undefined::Budget(format!(
                "the step budget, {} calls, reached at `{f}`",
                self.budget.steps
            ))));
        }
        Ok(active)
    }
    /// These functions, or `Err` with the paths of those refused: a
    /// function that calls itself, however far; two arguments of one
    /// name; a body that does not check to what it gives, from what it
    /// takes; a row that is not of its types; a body or a row while its
    /// types name a parameter, which no value is of; and a function that
    /// calls one refused or unknown; two rows at one point that give
    /// different values (a function gives one value at a point). Beside
    /// a body, rows are kept and never read (`agree` compares them with
    /// it, A002).
    pub fn with(callees: BTreeMap<String, Callee>) -> Result<Calls, BTreeSet<String>> {
        let names: Vec<&String> = callees.keys().collect();
        let place: BTreeMap<&String, usize> =
            names.iter().enumerate().map(|(i, n)| (*n, i)).collect();
        let mut out = vec![Vec::new(); names.len()];
        let mut refused = BTreeSet::new();
        for (i, callee) in callees.values().enumerate() {
            // A function the library lacks is refused by its type, below.
            for called in callee.body.iter().flat_map(called) {
                out[i].extend(place.get(&called).copied());
            }
        }
        // On a cycle: in a component of two or more, or calling itself.
        let component = components(&out);
        let mut size = vec![0usize; names.len()];
        for &c in &component {
            size[c] += 1;
        }
        for (i, name) in names.iter().enumerate() {
            let callee = &callees[*name];
            let own = out[i].contains(&i)
                && !callee
                    .body
                    .as_ref()
                    .is_some_and(|b| structural(&callee.takes, b, name));
            if size[component[i]] > 1 || own {
                refused.insert((*name).clone());
            }
        }
        let standard = Simulation {
            calls: Arc::new(Calls {
                callees: callees.clone(),
                ..Calls::default()
            }),
        };
        let exact = Exact::default();
        for (name, callee) in &callees {
            let open = callee.gives.open() || callee.takes.iter().any(|(_, t)| t.open());
            let inputs: BTreeMap<String, Type> = callee.takes.iter().cloned().collect();
            let distinct = inputs.len() == callee.takes.len();
            let body = callee.body.as_ref().is_none_or(|body| {
                !refused.contains(name)
                    && check(&standard, &inputs, body) == Some(callee.gives.clone())
            });
            let row = |(at, gives): &(Vec<Value>, Value)| {
                at.len() == callee.takes.len()
                    && at
                        .iter()
                        .zip(&callee.takes)
                        .all(|(v, (_, t))| exact.has_type(v, t))
                    && exact.has_type(gives, &callee.gives)
            };
            let bare = callee.body.is_none() && callee.rows.is_empty();
            let one_value = callee.rows.iter().enumerate().all(|(i, (at, gives))| {
                callee.rows[i + 1..]
                    .iter()
                    .all(|(other, g)| other != at || g == gives)
            });
            if !distinct || !body || !callee.rows.iter().all(row) || !one_value || (open && !bare) {
                refused.insert(name.clone());
            }
        }
        // And whatever calls one refused, however far: one walk back
        // along the calls.
        let mut back = vec![Vec::new(); names.len()];
        for (from, to) in out.iter().enumerate() {
            for &to in to {
                back[to].push(from);
            }
        }
        let mut walk: Vec<usize> = (0..names.len())
            .filter(|&i| refused.contains(names[i]))
            .collect();
        while let Some(node) = walk.pop() {
            for &from in &back[node] {
                if refused.insert(names[from].clone()) {
                    walk.push(from);
                }
            }
        }
        match refused.is_empty() {
            true => Ok(Calls {
                callees,
                ..Calls::default()
            }),
            false => Err(refused),
        }
    }
    /// These functions, with `gives` given for `path` at `at`: read
    /// first, before its rows. Only a function with no body takes a
    /// value so, of its types (none of them a parameter): else `Err`,
    /// with the functions as they were. A body is what the function
    /// gives, and `Ranges` reads it, not what was given.
    pub fn given(self, path: &str, at: Vec<Value>, gives: Value) -> Result<Calls, Calls> {
        self.given_any(path, at.into_iter().map(Some).collect(), gives)
    }
    /// `given`, an argument `None` standing for any value (`_`): a call
    /// at arguments some row gives with each written is read first, then
    /// one with `_`, the latest given first.
    pub fn given_any(
        mut self,
        path: &str,
        at: Vec<Option<Value>>,
        gives: Value,
    ) -> Result<Calls, Calls> {
        let exact = Exact::default();
        let fits = self.callees.get(path).is_some_and(|c| {
            c.body.is_none()
                && !c.gives.open()
                && c.takes.iter().all(|(_, t)| !t.open())
                && at.len() == c.takes.len()
                && at
                    .iter()
                    .zip(&c.takes)
                    .all(|(v, (_, t))| v.as_ref().is_none_or(|v| exact.has_type(v, t)))
                && exact.has_type(&gives, &c.gives)
        });
        if !fits {
            return Err(self);
        }
        self.memo = Box::default();
        self.given
            .entry(path.to_string())
            .or_default()
            .insert(0, (at, gives));
        Ok(self)
    }
    /// The function at `path`.
    pub fn get(&self, path: &str) -> Option<&Callee> {
        self.callees.get(path)
    }
    /// Every function, by path.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Callee)> {
        self.callees.iter()
    }
}

/// The paths a body calls, each once.
fn called(body: &Expression<Op>) -> BTreeSet<String> {
    let (mut found, mut todo) = (BTreeSet::new(), vec![body]);
    while let Some(e) = todo.pop() {
        if let Expression::Apply(op, args) = e {
            if let Op::Call(f) = op {
                found.insert(f.clone());
            }
            todo.extend(args.iter().map(|a| &a.body));
        }
    }
    found
}

/// The simulation standard: its types, the type of each operation, and
/// the functions a call can name.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Simulation {
    pub calls: Arc<Calls>,
}

/// Text and letters: ordered by their bytes, as Rust orders `str`.
fn ordered(t: &Type) -> bool {
    matches!(t, Type::Char) || matches!(t, Type::Encoded(c) if **c == Type::Char)
}

fn number(t: &Type) -> bool {
    matches!(
        t,
        Type::Int
            | Type::Real
            | Type::Param {
                bound: Bound::Number,
                ..
            }
    )
}

impl Standard for Simulation {
    type Type = Type;
    type Op = Op;
    fn binds(&self, op: &Op, i: usize, before: &[Type]) -> Vec<Type> {
        match (op, i, before) {
            (Op::Let, 1, [t]) => vec![t.clone()],
            (Op::All, 2, _) => vec![Type::Int],
            // Of a text too, so that its block types and check says
            // where the loop goes wrong: `gives` takes a sequence only.
            (Op::Fold, 2, [Type::Seq(t) | Type::Encoded(t), start]) => {
                let mut binds = vec![(**t).clone()];
                binds.extend(carried(start));
                binds
            }
            (Op::Until, 1 | 2, [start, ..]) => carried(start),
            (Op::Match(e), i, _) if i >= 1 => e
                .variants
                .get(i - 1)
                .map(|(_, ts)| ts.clone())
                .unwrap_or_default(),
            _ => vec![],
        }
    }
    fn gives(&self, op: &Op, takes: &[Type]) -> Option<Type> {
        use Type::{Int, Real, Truth};
        Some(match (op, takes) {
            (Op::Truth(_), []) => Truth,
            (Op::Int(_), []) => Int,
            (Op::Real(_), []) => Real,
            (Op::Not, [Truth]) | (Op::And | Op::Or, [Truth, Truth]) => Truth,
            (Op::Add | Op::Sub | Op::Mul, [a, b]) if number(a) && a == b => a.clone(),
            (Op::Div, [Real, Real]) => Real,
            (Op::Div | Op::Rem, [Int, Int]) => Int,
            (Op::Less | Op::AtMost, [a, b]) if number(a) && a == b => Truth,
            (Op::Equal, [a, b]) if a == b => Truth,
            (Op::Within, [a, b, c]) if number(a) && a == b && b == c => Truth,
            (Op::If, [Truth, a, b]) if a == b => a.clone(),
            (Op::Make(r), fields) if fits_fields(fields, r, |t, f| t == f) => {
                Type::Record(r.clone())
            }
            (Op::Field(name), [Type::Record(r)]) => r.field(name)?.1.clone(),
            (Op::Let, [_, t]) => t.clone(),
            (Op::All, [Int, Int, Truth]) => Truth,
            (Op::Variant(e, i), holds)
                if e.variants
                    .get(*i)
                    .is_some_and(|(_, ts)| ts.as_slice() == holds) =>
            {
                Type::Enum(e.clone())
            }
            (Op::Match(e), [Type::Enum(t), arms @ ..])
                if e == t
                    && !arms.is_empty()
                    && arms.len() == e.variants.len()
                    && arms.iter().all(|a| a == &arms[0]) =>
            {
                arms[0].clone()
            }
            (Op::Seq(t), holds) if holds.iter().all(|h| h == t) => Type::Seq(Box::new(t.clone())),
            (Op::Len, [Type::Seq(_) | Type::Encoded(_)]) => Int,
            (Op::Index, [Type::Seq(t), Int]) => (**t).clone(),
            (Op::Index, [Type::Encoded(_), Int]) => Type::Byte,
            (Op::Text(_), []) => Type::Encoded(Box::new(Type::Char)),
            (Op::Join, [a @ (Type::Seq(_) | Type::Encoded(_)), b]) if a == b => a.clone(),
            (Op::StartsWith, [Type::Seq(_) | Type::Encoded(_), _]) if takes[0] == takes[1] => Truth,
            (Op::Decoded, [a @ Type::Seq(_)]) => a.clone(),
            (Op::Fold, [Type::Seq(_), start, next]) if next_fits(start, next) => start.clone(),
            (Op::Until, [start, Truth, next]) if next_fits(start, next) => start.clone(),
            (Op::Less | Op::AtMost, [a, b]) if ordered(a) && a == b => Truth,
            (Op::Call(f), takes) => {
                let callee = self.calls.get(f)?;
                let fits = takes.len() == callee.takes.len()
                    && takes.iter().zip(&callee.takes).all(|(t, (_, u))| t == u);
                return fits.then(|| callee.gives.clone());
            }
            _ => return None,
        })
    }
}

/// There are as many of `these` as `r` has fields, and each is right
/// for its field's type.
fn fits_fields<X>(these: &[X], r: &Record, right: impl Fn(&X, &Type) -> bool) -> bool {
    these.len() == r.fields.len() && these.iter().zip(&r.fields).all(|(x, (_, f))| right(x, f))
}

/// The exact reading: values.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Exact {
    pub calls: Arc<Calls>,
    /// When kept, each call to a function with no body, in the order
    /// the run made it (BDD step 2): what the run did to the world.
    pub log: Log,
}

/// The calls a run made, in order: to functions with no body, and to
/// those with one (finding 14). Kept only when asked, `Log::kept()`; any
/// two are equal, as it says nothing of the reading.
#[derive(Clone, Debug, Default)]
pub struct Log(Option<Arc<std::sync::Mutex<Vec<Logged>>>>);

/// One call of a log, and whether the function has a body.
type Logged = (Made, bool);

/// A call made: the path of the function, and its arguments.
pub type Made = (String, Vec<Value>);

impl PartialEq for Log {
    fn eq(&self, _: &Log) -> bool {
        true
    }
}
impl Eq for Log {}

impl Log {
    /// A log that keeps the calls.
    pub fn kept() -> Log {
        Log(Some(Arc::default()))
    }
    /// The calls to functions with no body kept so far, in order; none
    /// when not kept.
    pub fn calls(&self) -> Vec<(String, Vec<Value>)> {
        self.made(false)
    }
    /// The calls to functions with a body kept so far, in order: the
    /// way the model's run went (finding 14).
    pub fn bodies(&self) -> Vec<(String, Vec<Value>)> {
        self.made(true)
    }
    fn made(&self, body: bool) -> Vec<(String, Vec<Value>)> {
        match &self.0 {
            Some(l) => l
                .lock()
                .unwrap_or_else(|e| e.into_inner())
                .iter()
                .filter(|(_, b)| *b == body)
                .map(|(m, _)| m.clone())
                .collect(),
            None => vec![],
        }
    }
    fn push(&self, f: &str, args: &[Value], body: bool) {
        if let Some(l) = &self.0 {
            l.lock()
                .unwrap_or_else(|e| e.into_inner())
                .push(((f.to_string(), args.to_vec()), body));
        }
    }
}

impl Exact {
    /// A call of `f` at these values: its body run on them; or else what
    /// the reading was given there, else its row at them.
    fn call(&self, f: &str, args: &[Value]) -> Exactly {
        let callee = self.calls.get(f).ok_or(Fault::Stuck)?;
        if args.len() != callee.takes.len() {
            return Err(Fault::Stuck);
        }
        for ((n, _), v) in callee.takes.iter().zip(args) {
            match callee.ranges.takes.get(n) {
                Some(range) if !range.holds(v) => {
                    return Err(Fault::Op(Undefined::Outside(format!(
                        "`{f}` takes `{n}` in {range}, given {}",
                        crate::print::value(v)
                    ))))
                }
                _ => {}
            }
        }
        if let Some(body) = &callee.body {
            self.log.push(f, args, true);
            let gives = self.body(f, callee, body, args);
            return match (&gives, &callee.ranges.gives) {
                (Ok(v), Some((q, range))) if !range.holds(v) => {
                    Err(Fault::Op(Undefined::Outside(format!(
                        "`{f}` gives `{q}` in {range}, and its body gives {}",
                        crate::print::value(v)
                    ))))
                }
                _ => gives,
            };
        }
        self.log.push(f, args, false);
        let given: Vec<_> = self.calls.given.get(f).into_iter().flatten().collect();
        let written = |at: &[Option<Value>]| {
            at.len() == args.len() && at.iter().zip(args).all(|(a, v)| a.as_ref() == Some(v))
        };
        let any = |at: &[Option<Value>]| {
            at.len() == args.len()
                && at
                    .iter()
                    .zip(args)
                    .all(|(a, v)| a.as_ref().is_none_or(|a| a == v))
        };
        let found = given.iter().find(|(at, _)| written(at));
        if let Some((_, gives)) = found.or_else(|| given.iter().find(|(at, _)| any(at))) {
            return Ok(gives.clone());
        }
        match callee.rows.iter().find(|(at, _)| at.as_slice() == args) {
            Some((_, gives)) => Ok(gives.clone()),
            // `()` is the one value of its type: a call made for what it
            // does gives it, with no row (BDD step 2).
            None if callee.gives == tuple(vec![]) => Ok(Value::Record(
                Record {
                    name: String::new(),
                    args: vec![],
                    fields: vec![],
                },
                vec![],
            )),
            None => Err(Fault::Op(Undefined::Unknown(f.to_string()))),
        }
    }
    /// A call's body read at `args`, remembered.
    fn body(&self, f: &str, callee: &Callee, body: &Expression<Op>, args: &[Value]) -> Exactly {
        {
            let key = (f.to_string(), args.to_vec());
            let memo = &self.calls.memo.exact;
            // A run whose calls are kept runs each body: one remembered
            // would make its calls unseen.
            let kept = self.log.0.is_some();
            if let Some(known) = memo.lock().unwrap_or_else(|e| e.into_inner()).get(&key) {
                if !kept {
                    return known.clone();
                }
            }
            let active = self.calls.enter(f)?;
            let names = callee.takes.iter().map(|(n, _)| n.clone());
            let gives = held_in(f, eval(self, &names.zip(args.iter().cloned()).collect(), body));
            drop(active);
            // A budget reached depends on the way here: not remembered.
            if !kept && !matches!(gives, Err(Fault::Op(Undefined::Budget(_)))) {
                memo.lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .insert(key, gives.clone());
            }
            gives
        }
    }
}

type Exactly = Outcome<Value, Undefined>;

/// What a truth decides, or why it does not.
fn decides(test: &Exactly) -> Result<bool, Fault<Undefined>> {
    match test {
        Ok(Value::Truth(b)) => Ok(*b),
        Ok(_) => Err(Fault::Stuck),
        Err(fault) => Err(fault.clone()),
    }
}

/// `xs.fold(start) { x -> next }`, exactly.
#[inline(never)]
fn fold_exactly(
    xs: &Exactly,
    start: &Exactly,
    run: &mut dyn FnMut(usize, &[Exactly]) -> Exactly,
) -> Exactly {
    let Value::Seq(_, holds) = xs.clone()? else {
        return Err(Fault::Stuck);
    };
    let mut carried = start.clone()?;
    for x in holds {
        let mut binds = vec![Ok(x)];
        binds.extend(unpack(&carried));
        carried = repack(&carried, run(2, &binds)?);
    }
    Ok(carried)
}

/// `until(start, done) { next }`, exactly, at most `steps` turns.
#[inline(never)]
fn until_exactly(
    start: &Exactly,
    steps: usize,
    run: &mut dyn FnMut(usize, &[Exactly]) -> Exactly,
) -> Exactly {
    let mut carried = start.clone()?;
    let mut turns = 0;
    // Brent's cycle finding: a state kept, put by again at each power
    // of two turns; one carried again can only go round again (finding
    // 39 G). No memory past one state, and a cycle is seen within twice
    // the turns to its end.
    let (mut kept, mut power, mut since): (Option<Value>, usize, usize) = (None, 1, 0);
    loop {
        if decides(&run(1, &unpack(&carried)))? {
            return Ok(carried);
        }
        if !known(&carried) {
            kept = None;
        } else if kept.as_ref() == Some(&carried) {
            return Err(Fault::Op(Undefined::EndlessRepeat {
                inside: None,
                state: crate::print::value(&carried),
                turns: since,
            }));
        } else if kept.is_none() || since == power {
            kept = Some(carried.clone());
            power *= 2;
            since = 0;
        }
        since += 1;
        turns += 1;
        if turns > steps {
            return Err(Fault::Op(Undefined::Budget(format!(
                "the step budget, {steps} turns of `until`, reached"
            ))));
        }
        carried = repack(&carried, run(2, &unpack(&carried))?);
    }
}

/// The values a loop's block is given: each of a labelled tuple, else
/// the one value carried.
fn unpack(carried: &Value) -> Vec<Exactly> {
    match carried {
        Value::Record(r, values) if r.labelled() => values.iter().cloned().map(Ok).collect(),
        v => vec![Ok(v.clone())],
    }
}

/// What a loop carries next: the block's values, by place, under the
/// labels of what it carried.
fn repack(carried: &Value, next: Value) -> Value {
    match (carried, next) {
        (Value::Record(r, _), Value::Record(n, values)) if r.labelled() && n.name.is_empty() => {
            Value::Record(r.clone(), values)
        }
        (_, next) => next,
    }
}

/// `then`, when it is a truth.
fn truth(then: &Exactly) -> Exactly {
    decides(then).map(Value::Truth)
}

impl Reading for Exact {
    type Standard = Simulation;
    type Value = Value;
    type Fault = Undefined;

    fn has_type(&self, value: &Value, of: &Type) -> bool {
        match (value, of) {
            (Value::Truth(_), Type::Truth)
            | (Value::Int(_), Type::Int)
            | (Value::Real(_), Type::Real) => true,
            (Value::Record(r, values), Type::Record(t)) => {
                r == t && fits_fields(values, t, |v, f| self.has_type(v, f))
            }
            // Known in part: each field it knows of its type, one not.
            (Value::Partial(r, values), Type::Record(t)) => {
                r == t
                    && values.iter().any(Option::is_none)
                    && fits_fields(values, t, |v, f| {
                        v.as_ref().is_none_or(|v| self.has_type(v, f))
                    })
            }
            (Value::Variant(e, i, holds), Type::Enum(t)) => {
                // Where a tree holds itself, the value's own variants.
                let t = if t.cut() { e } else { t };
                e == t
                    && t.variants.get(*i).is_some_and(|(_, ts)| {
                        ts.len() == holds.len()
                            && holds.iter().zip(ts).all(|(v, t)| self.has_type(v, t))
                    })
            }
            (Value::Seq(t, holds), Type::Seq(u)) => {
                t == &**u && holds.iter().all(|v| self.has_type(v, t))
            }
            (Value::Text(_), Type::Encoded(t)) => **t == Type::Char,
            (Value::Byte(_), Type::Byte) => true,
            _ => false,
        }
    }

    fn apply(&self, op: &Op, args: &[Exactly]) -> Exactly {
        // Two small frames, so that the large one is not held while a
        // callee's body runs.
        match op {
            Op::Call(f) => self.call_strict(f, args),
            _ => self.plain(op, args),
        }
    }

    fn bind(
        &self,
        op: &Op,
        args: &[Exactly],
        run: &mut dyn FnMut(usize, &[Exactly]) -> Exactly,
    ) -> Exactly {
        match (op, args) {
            (Op::Let, [value, _]) => run(1, std::slice::from_ref(value)),
            (Op::All, [lo, hi, _]) => all_exactly(lo, hi, run),
            // Loops in frames of their own: this one is on the path of
            // every level.
            (Op::Fold, [xs, start, _]) => fold_exactly(xs, start, run),
            (Op::Until, [start, _, _]) => until_exactly(start, self.calls.budget.steps, run),
            (Op::Match(_), [scrutinee, ..]) => match scrutinee {
                Ok(Value::Variant(_, i, holds)) => {
                    let holds: Vec<Exactly> = holds.iter().cloned().map(Ok).collect();
                    run(1 + i, &holds)
                }
                Ok(_) => Err(Fault::Stuck),
                Err(fault) => Err(fault.clone()),
            },
            _ => Err(Fault::Stuck),
        }
    }

    /// `if`, `&&` and `||` compute only the side they take: none when
    /// the test has no value, which is then what they give.
    fn needed(&self, op: &Op, i: usize, before: &[Exactly]) -> bool {
        let test = || match before.first() {
            Some(Ok(Value::Truth(b))) => Some(*b),
            _ => None,
        };
        match (op, i) {
            (Op::If, 1) | (Op::And, 1) => test() == Some(true),
            (Op::If, 2) | (Op::Or, 1) => test() == Some(false),
            // `match`: the arm of its variant only.
            (Op::Match(_), i) if i >= 1 => {
                matches!(before.first(), Some(Ok(Value::Variant(_, k, _))) if i == 1 + k)
            }
            _ => true,
        }
    }
}

impl Exact {
    #[inline(never)]
    fn call_strict(&self, f: &str, args: &[Exactly]) -> Exactly {
        self.call(f, &strict(args)?)
    }
    /// Every operation but a call.
    #[inline(never)]
    fn plain(&self, op: &Op, args: &[Exactly]) -> Exactly {
        match (op, args) {
            (Op::If, [test, then, otherwise]) => {
                return match decides(test)? {
                    true => then.clone(),
                    false => otherwise.clone(),
                }
            }
            // A record with `..` made from another known in part, `R { a:
            // 1, ..r }`: its `..` not known, it is known in part too.
            (Op::Make(r), fields)
                if fields.len() == r.fields.len()
                    && fields.iter().zip(&r.fields).any(|(f, _)| f.is_err())
                    && fields.iter().zip(&r.fields).all(|(f, (n, _))| match f {
                        Ok(_) => true,
                        Err(Fault::Op(Undefined::Unknown(_))) => n == crate::model::REST,
                        Err(_) => false,
                    }) =>
            {
                let values: Vec<Option<Value>> = fields.iter().map(|f| f.clone().ok()).collect();
                let typed = values
                    .iter()
                    .zip(&r.fields)
                    .all(|(v, (_, t))| v.as_ref().is_none_or(|v| self.has_type(v, t)));
                return match typed {
                    true => Ok(Value::partial(r.clone(), values)),
                    false => Err(Fault::Stuck),
                };
            }
            // Arms that bind nothing: the one of its variant was computed.
            (Op::Match(_), [scrutinee, arms @ ..]) => {
                return match scrutinee {
                    Ok(Value::Variant(_, i, _)) => {
                        arms.get(*i).cloned().unwrap_or(Err(Fault::Stuck))
                    }
                    Ok(_) => Err(Fault::Stuck),
                    Err(fault) => Err(fault.clone()),
                }
            }
            (Op::And, [a, b]) => {
                return match decides(a)? {
                    true => truth(b),
                    false => Ok(Value::Truth(false)),
                }
            }
            (Op::Or, [a, b]) => {
                return match decides(a)? {
                    true => Ok(Value::Truth(true)),
                    false => truth(b),
                }
            }
            _ => {}
        }
        let args = strict(args)?;
        use Value::{Int, Real, Truth};
        Ok(match (op, args.as_slice()) {
            (Op::Truth(b), []) => Truth(*b),
            (Op::Int(n), []) => Int(n.clone()),
            (Op::Real(q), []) => Real(q.clone()),
            (Op::Not, [Truth(a)]) => Truth(!a),
            (Op::Add, [Int(a), Int(b)]) => Int(a + b),
            (Op::Sub, [Int(a), Int(b)]) => Int(a - b),
            (Op::Mul, [Int(a), Int(b)]) => Int(a * b),
            (Op::Add, [Real(a), Real(b)]) => Real(a.plus(b)),
            (Op::Sub, [Real(a), Real(b)]) => Real(a.minus(b)),
            (Op::Mul, [Real(a), Real(b)]) => Real(a.times(b)),
            (Op::Div, [Real(a), Real(b)]) => match a.over(b) {
                Some(q) => Real(q),
                None => return Err(Fault::Op(Undefined::DivisionByZero)),
            },
            // `BigInt` rounds toward zero, and its remainder takes the
            // sign of `a`: as Rust's `i64`.
            (Op::Div | Op::Rem, [Int(_), Int(b)]) if b.is_zero() => {
                return Err(Fault::Op(Undefined::DivisionByZero))
            }
            (Op::Div, [Int(a), Int(b)]) => Int(a / b),
            (Op::Rem, [Int(a), Int(b)]) => Int(a % b),
            (Op::Less, [Int(a), Int(b)]) => Truth(a < b),
            (Op::Less, [Real(a), Real(b)]) => Truth(a < b),
            (Op::AtMost, [Int(a), Int(b)]) => Truth(a <= b),
            (Op::AtMost, [Real(a), Real(b)]) => Truth(a <= b),
            (Op::Equal, [a, b]) => match same(a, b) {
                Ok(yes) => Truth(yes),
                Err(g) => return Err(Fault::Op(Undefined::Unknown(g))),
            },
            (Op::Within, [Int(a), Int(b), Int(tol)]) => Truth((a - b).abs() <= *tol),
            (Op::Within, [Real(a), Real(b), Real(tol)]) => Truth(a.minus(b).abs() <= *tol),
            (Op::Make(r), fields) if fits_fields(fields, r, |v, f| self.has_type(v, f)) => {
                Value::Record(r.clone(), fields.to_vec())
            }
            (Op::Variant(e, i), holds)
                if e.variants.get(*i).is_some_and(|(_, ts)| {
                    ts.len() == holds.len()
                        && holds.iter().zip(ts).all(|(v, t)| self.has_type(v, t))
                }) =>
            {
                Value::Variant(e.clone(), *i, holds.to_vec())
            }
            (Op::Seq(t), holds) if holds.iter().all(|v| self.has_type(v, t)) => {
                Value::Seq(t.clone(), holds.to_vec())
            }
            (Op::Len, [Value::Seq(_, holds)]) => Int(BigInt::from(holds.len())),
            // Text: its bytes, as Rust's `str`.
            (Op::Text(s), []) => Value::Text(s.clone()),
            (Op::Len, [Value::Text(s)]) => Int(BigInt::from(s.len())),
            (Op::Index, [Value::Text(s), Int(i)]) => {
                match usize::try_from(i).ok().and_then(|k| s.as_bytes().get(k)) {
                    Some(b) => Value::Byte(*b),
                    None => {
                        return Err(Fault::Op(Undefined::Index(format!(
                            "the index is {i}, and the text holds {} bytes",
                            s.len()
                        ))))
                    }
                }
            }
            (Op::Join, [Value::Text(a), Value::Text(b)]) => Value::Text(format!("{a}{b}")),
            (Op::Join, [Value::Seq(t, a), Value::Seq(u, b)]) if t == u => {
                Value::Seq(t.clone(), [a.clone(), b.clone()].concat())
            }
            (Op::StartsWith, [Value::Text(a), Value::Text(b)]) => Truth(a.starts_with(b.as_str())),
            (Op::StartsWith, [Value::Seq(t, a), Value::Seq(u, b)]) if t == u => {
                if b.len() > a.len() {
                    Truth(false)
                } else {
                    let mut all = true;
                    for (x, y) in a.iter().zip(b) {
                        match same(x, y) {
                            Ok(true) => {}
                            Ok(false) => {
                                all = false;
                                break;
                            }
                            Err(g) => return Err(Fault::Op(Undefined::Unknown(g))),
                        }
                    }
                    Truth(all)
                }
            }
            (Op::Decoded, [v @ Value::Seq(..)]) => v.clone(),
            (Op::Less, [Value::Text(a), Value::Text(b)]) => Truth(a.as_bytes() < b.as_bytes()),
            (Op::AtMost, [Value::Text(a), Value::Text(b)]) => Truth(a.as_bytes() <= b.as_bytes()),
            (Op::Index, [Value::Seq(_, holds), Int(i)]) => {
                match usize::try_from(i).ok().and_then(|k| holds.get(k)) {
                    Some(v) => v.clone(),
                    None => {
                        return Err(Fault::Op(Undefined::Index(format!(
                            "the index is {i}, and the sequence holds {}",
                            holds.len()
                        ))))
                    }
                }
            }
            (Op::Field(name), [Value::Record(r, values)]) => {
                match r.field(name).and_then(|(place, _)| values.get(place)) {
                    Some(v) => v.clone(),
                    None => return Err(Fault::Stuck),
                }
            }
            (Op::Field(name), [Value::Partial(r, values)]) => {
                match r.field(name).and_then(|(place, _)| values.get(place)) {
                    Some(Some(v)) => v.clone(),
                    Some(None) => {
                        return Err(Fault::Op(Undefined::Unknown(format!("{}::{name}", r.name))))
                    }
                    None => return Err(Fault::Stuck),
                }
            }
            _ => return Err(Fault::Stuck),
        })
    }
}

/// `(lo..hi).all(|x| p)`: `p` at each number in turn, until one is not
/// true.
#[inline(never)]
fn all_exactly(
    lo: &Exactly,
    hi: &Exactly,
    run: &mut dyn FnMut(usize, &[Exactly]) -> Exactly,
) -> Exactly {
    let (Value::Int(mut x), Value::Int(hi)) = (lo.clone()?, hi.clone()?) else {
        return Err(Fault::Stuck);
    };
    while x < hi {
        if !decides(&run(2, &[Ok(Value::Int(x.clone()))]))? {
            return Ok(Value::Truth(false));
        }
        x += 1;
    }
    Ok(Value::Truth(true))
}

/// Bounds a value lies in, and whether computing it may have no value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Bounds {
    pub span: Span,
    pub may_fault: bool,
}

/// The numbers a span holds: `/` rounds on one and not the other.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Numbers {
    Int,
    Real,
}

/// Where a value lies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Span {
    /// The truths it may be: at least one.
    Truth { yes: bool, no: bool },
    /// A number of one type from the low end to the high end, both in.
    /// An end that is `None` is no end: the number may be as low, or as
    /// high, as its type goes. For an `Int`, each end is whole.
    Number(Numbers, Option<Rational>, Option<Rational>),
    /// A record, each field in its span.
    Record(Record, Vec<Span>),
    /// An enum: for each variant, `None` when it may not be, else the
    /// spans of what it holds. At least one may be.
    Variant(Enum, Vec<Option<Vec<Span>>>),
    /// A sequence of values of this type: its length, a span of `Int`
    /// from zero up; a span every value it holds lies in; and the spans
    /// of its first places, each surely held (no more of them than its
    /// least length), so that a sequence known at a point is known
    /// exactly.
    Seq(Type, Box<Span>, Box<Span>, Vec<Span>),
    /// Text: the span of its length in bytes, and the text itself when
    /// it is known.
    Text(Box<Span>, Option<String>),
    /// A byte, when it is known.
    Byte(Option<u8>),
    /// A letter: no value of it is written yet, so any.
    Char,
    /// The fields a record with `..` does not state: any, never equal
    /// or different surely.
    Rest,
}

/// A loop by bounds: any value of what it carries, perhaps none.
#[inline(never)]
fn loop_bounds(xs: Option<&Bounded>, start: &Bounded) -> Bounded {
    if let Some(xs) = xs {
        xs.clone()?;
    }
    let start = start.clone()?;
    Ok(Bounds {
        may_fault: true,
        ..Bounds::any(&type_of(&start.span))
    })
}

/// The type of the values a span holds.
fn type_of(span: &Span) -> Type {
    match span {
        Span::Truth { .. } => Type::Truth,
        Span::Number(Numbers::Int, ..) => Type::Int,
        Span::Number(Numbers::Real, ..) => Type::Real,
        Span::Record(r, _) => Type::Record(r.clone()),
        Span::Variant(e, _) => Type::Enum(e.clone()),
        Span::Seq(t, ..) => Type::Seq(Box::new(t.clone())),
        Span::Text(..) => Type::Encoded(Box::new(Type::Char)),
        Span::Byte(_) => Type::Byte,
        Span::Char => Type::Char,
        Span::Rest => crate::model::rest_type(),
    }
}

/// γ: `value` lies in `span`.
pub fn lies_in(value: &Value, span: &Span) -> bool {
    match (value, span) {
        (Value::Truth(b), Span::Truth { yes, no }) => match b {
            true => *yes,
            false => *no,
        },
        (Value::Int(n), Span::Number(Numbers::Int, low, high)) => {
            inside(&Rational::integer(n.clone()), low, high)
        }
        (Value::Real(q), Span::Number(Numbers::Real, low, high)) => inside(q, low, high),
        (Value::Record(r, values), Span::Record(s, spans)) => {
            r == s
                && values.len() == spans.len()
                && values.iter().zip(spans).all(|(v, s)| lies_in(v, s))
        }
        (Value::Partial(r, values), Span::Record(s, spans)) => {
            r == s
                && values.len() == spans.len()
                && values
                    .iter()
                    .zip(spans)
                    .all(|(v, s)| v.as_ref().is_none_or(|v| lies_in(v, s)))
        }
        // Any tree where a tree holds itself.
        (Value::Variant(e, _, _), Span::Variant(f, _)) if f.cut() => e == f,
        (Value::Variant(e, i, holds), Span::Variant(f, may)) => {
            e == f
                && may.get(*i).is_some_and(|m| {
                    m.as_ref().is_some_and(|spans| {
                        spans.len() == holds.len()
                            && holds.iter().zip(spans).all(|(v, s)| lies_in(v, s))
                    })
                })
        }
        (Value::Text(s), Span::Text(length, known)) => {
            lies_in(&Value::Int(BigInt::from(s.len())), length)
                && known.as_ref().is_none_or(|k| k == s)
        }
        (Value::Byte(b), Span::Byte(known)) => known.is_none_or(|k| k == *b),
        (Value::Seq(t, holds), Span::Seq(u, length, each, first)) => {
            t == u
                && lies_in(&Value::Int(BigInt::from(holds.len())), length)
                && holds.iter().all(|v| lies_in(v, each))
                && first.len() <= holds.len()
                && holds.iter().zip(first).all(|(v, s)| lies_in(v, s))
        }
        _ => false,
    }
}

/// γ on outcomes of type `of`: what `exact` gave is one of those
/// `bounds` stand for. A value must be of the type and lie in the span;
/// a fault is allowed where the bounds may fault, or are a fault.
pub fn covers(of: &Type, bounds: &Outcome<Bounds, Undefined>, exact: &Exactly) -> bool {
    match (bounds, exact) {
        // Bounds read past a budget say nothing, and claim nothing.
        (Err(Fault::Op(Undefined::Budget(_))), _) => true,
        (Err(Fault::Op(_)), Err(Fault::Op(_))) => true,
        // A value of the world `Exact` does not know: any bounds hold it.
        (Ok(_), Err(Fault::Op(Undefined::Unknown(_)))) => true,
        (Ok(b), Err(Fault::Op(_))) => b.may_fault,
        (Ok(b), Ok(v)) => Exact::default().has_type(v, of) && lies_in(v, &b.span),
        _ => false,
    }
}

impl Bounds {
    /// Exactly `value`.
    pub fn exactly(value: &Value) -> Bounds {
        fn span(v: &Value) -> Span {
            match v {
                Value::Truth(b) => Span::Truth { yes: *b, no: !*b },
                Value::Int(n) => point(Numbers::Int, Rational::integer(n.clone())),
                Value::Real(q) => point(Numbers::Real, q.clone()),
                Value::Record(r, values) => {
                    Span::Record(r.clone(), values.iter().map(span).collect())
                }
                Value::Variant(e, i, holds) => Span::Variant(
                    e.clone(),
                    (0..e.variants.len())
                        .map(|k| (k == *i).then(|| holds.iter().map(span).collect()))
                        .collect(),
                ),
                Value::Text(s) => Span::Text(
                    Box::new(point(
                        Numbers::Int,
                        Rational::integer(BigInt::from(s.len())),
                    )),
                    Some(s.clone()),
                ),
                Value::Byte(b) => Span::Byte(Some(*b)),
                Value::Seq(t, holds) => held(t, holds.iter().map(span).collect())
                    .unwrap_or_else(|| Bounds::any(&Type::Seq(Box::new(t.clone()))).span),
                // A field not known: any value of its type.
                Value::Partial(r, values) => Span::Record(
                    r.clone(),
                    values
                        .iter()
                        .zip(&r.fields)
                        .map(|(v, (_, t))| match v {
                            Some(v) => span(v),
                            None => Bounds::any(t).span,
                        })
                        .collect(),
                ),
            }
        }
        Bounds {
            span: span(value),
            may_fault: false,
        }
    }
    /// Any value of the type.
    pub fn any(of: &Type) -> Bounds {
        fn span(t: &Type) -> Span {
            match t {
                Type::Truth => Span::Truth {
                    yes: true,
                    no: true,
                },
                // No value is of a parameter: its span is never read.
                Type::Int => Span::Number(Numbers::Int, None, None),
                Type::Real | Type::Param { .. } => Span::Number(Numbers::Real, None, None),
                Type::Record(r) if r.name == crate::model::REST => Span::Rest,
                Type::Record(r) => {
                    Span::Record(r.clone(), r.fields.iter().map(|(_, f)| span(f)).collect())
                }
                Type::Enum(e) => Span::Variant(
                    e.clone(),
                    e.variants
                        .iter()
                        .map(|(_, ts)| Some(ts.iter().map(span).collect()))
                        .collect(),
                ),
                Type::Seq(t) => Span::Seq(
                    (**t).clone(),
                    Box::new(Span::Number(Numbers::Int, Some(Rational::integer(0)), None)),
                    Box::new(span(t)),
                    vec![],
                ),
                Type::Encoded(_) => Span::Text(
                    Box::new(Span::Number(Numbers::Int, Some(Rational::integer(0)), None)),
                    None,
                ),
                Type::Byte => Span::Byte(None),
                Type::Char => Span::Char,
            }
        }
        Bounds {
            span: span(of),
            may_fault: false,
        }
    }
    /// A number from `low` to `high`. `None` when `low` is above `high`.
    pub fn between(of: Numbers, low: Rational, high: Rational) -> Option<Bounds> {
        (low <= high).then_some(Bounds {
            span: Span::Number(of, Some(low), Some(high)),
            may_fault: false,
        })
    }
    /// The truth every exact value is, when they are all one, with no
    /// fault.
    pub fn surely(&self) -> Option<bool> {
        match (&self.span, self.may_fault) {
            (Span::Truth { yes, no }, false) if yes != no => Some(*yes),
            _ => None,
        }
    }
}

fn point(of: Numbers, q: Rational) -> Span {
    Span::Number(of, Some(q.clone()), Some(q))
}

/// `q` is from `low` to `high`, a missing end no end.
fn inside(q: &Rational, low: &Option<Rational>, high: &Option<Rational>) -> bool {
    low.as_ref().is_none_or(|l| l <= q) && high.as_ref().is_none_or(|h| q <= h)
}

/// An end of a range, or past every number on one side.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
enum End {
    Below,
    At(Rational),
    Above,
}

impl End {
    fn low(e: &Option<Rational>) -> End {
        e.clone().map_or(End::Below, End::At)
    }
    fn high(e: &Option<Rational>) -> End {
        e.clone().map_or(End::Above, End::At)
    }
    /// The end as a span keeps it: no end when past every number.
    fn finite(self) -> Option<Rational> {
        match self {
            End::At(q) => Some(q),
            _ => None,
        }
    }
    fn sign(&self) -> std::cmp::Ordering {
        match self {
            End::Below => std::cmp::Ordering::Less,
            End::At(q) => q.cmp(&Rational::integer(0)),
            End::Above => std::cmp::Ordering::Greater,
        }
    }
    /// The product of two ends: zero times no end is zero, as every
    /// number of a range is finite.
    fn times(&self, other: &End) -> End {
        use std::cmp::Ordering::{Equal, Greater, Less};
        match (self, other) {
            (End::At(a), End::At(b)) => End::At(a.times(b)),
            _ => match (self.sign(), other.sign()) {
                (Equal, _) | (_, Equal) => End::At(Rational::integer(0)),
                (Greater, Greater) | (Less, Less) => End::Above,
                _ => End::Below,
            },
        }
    }
    fn abs(&self) -> End {
        match self {
            End::At(q) => End::At(q.abs()),
            _ => End::Above,
        }
    }
}

/// `span` is right for a value of type `of`.
fn fits(span: &Span, of: &Type) -> bool {
    match (span, of) {
        (Span::Truth { yes, no }, Type::Truth) => *yes || *no,
        (Span::Number(numbers, low, high), Type::Int | Type::Real) => {
            let whole = |e: &Option<Rational>| e.as_ref().is_none_or(|q| q.is_integer());
            End::low(low) <= End::high(high)
                && match numbers {
                    Numbers::Int => of == &Type::Int && whole(low) && whole(high),
                    Numbers::Real => of == &Type::Real,
                }
        }
        (Span::Record(r, spans), Type::Record(t)) => r == t && fits_fields(spans, t, fits),
        (Span::Variant(e, _), Type::Enum(t)) if e.cut() => e == t,
        (Span::Variant(e, may), Type::Enum(t)) => {
            let t = if t.cut() { e } else { t };
            e == t
                && may.len() == t.variants.len()
                && may.iter().any(Option::is_some)
                && may.iter().zip(&t.variants).all(|(m, (_, ts))| {
                    m.as_ref().is_none_or(|spans| {
                        spans.len() == ts.len() && spans.iter().zip(ts).all(|(s, t)| fits(s, t))
                    })
                })
        }
        (Span::Text(length, known), Type::Encoded(c)) => {
            **c == Type::Char
                && fits(length, &Type::Int)
                && matches!(&**length, Span::Number(_, Some(low), _) if low >= &Rational::integer(0))
                && known
                    .as_ref()
                    .is_none_or(|k| lies_in(&Value::Int(BigInt::from(k.len())), length))
        }
        (Span::Byte(_), Type::Byte) | (Span::Char, Type::Char) => true,
        (Span::Rest, Type::Record(r)) => r.name == crate::model::REST,
        (Span::Seq(t, length, each, first), Type::Seq(u)) => {
            t == &**u
                && fits(length, &Type::Int)
                && matches!(&**length, Span::Number(_, Some(low), _)
                    if low >= &Rational::integer(0) && low >= &Rational::integer(BigInt::from(first.len())))
                && fits(each, t)
                && first.iter().all(|s| fits(s, t))
        }
        _ => false,
    }
}

/// The reading by bounds.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ranges {
    pub calls: Arc<Calls>,
}

impl Ranges {
    /// A call of `f` on these bounds: its body run on them, or else any
    /// value of its type, which may be none; none at all when its type
    /// names a parameter. It may fault where an argument may.
    fn call(&self, f: &str, args: &[Bounds]) -> Bounded {
        let callee = self.calls.get(f).ok_or(Fault::Stuck)?;
        if args.len() != callee.takes.len() {
            return Err(Fault::Stuck);
        }
        // What it takes, cut to its ranges: outside, no value.
        let mut outside = false;
        let mut cut = Vec::with_capacity(args.len());
        for ((n, _), b) in callee.takes.iter().zip(args) {
            match callee.ranges.takes.get(n) {
                Some(range) => match range.cut(b) {
                    Some((b, out)) => {
                        outside |= out;
                        cut.push(b);
                    }
                    None => {
                        return Err(Fault::Op(Undefined::Outside(format!(
                            "`{f}` takes `{n}` in {range}, given none of it"
                        ))))
                    }
                },
                None => cut.push(b.clone()),
            }
        }
        let args = cut.as_slice();
        let gives = match &callee.body {
            Some(body) => {
                let memo = &self.calls.memo.ranges;
                let known = memo
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .get(f)
                    .and_then(|seen| {
                        seen.iter()
                            .find(|(at, _)| at.as_slice() == args)
                            .map(|(_, g)| g.clone())
                    });
                let reading = || {
                    self.calls
                        .memo
                        .reading
                        .lock()
                        .unwrap_or_else(|e| e.into_inner())
                };
                let again = reading()
                    .iter()
                    .any(|(g, at)| g == f && at.as_slice() == args);
                match known {
                    Some(gives) => gives,
                    None if again => Ok(Bounds {
                        may_fault: true,
                        ..Bounds::any(&callee.gives)
                    }),
                    None => {
                        let active = self.calls.enter(f)?;
                        reading().push((f.to_string(), args.to_vec()));
                        let names = callee.takes.iter().map(|(n, _)| n.clone());
                        let gives = eval(self, &names.zip(args.iter().cloned()).collect(), body);
                        reading().pop();
                        drop(active);
                        if !matches!(gives, Err(Fault::Op(Undefined::Budget(_)))) {
                            memo.lock()
                                .unwrap_or_else(|e| e.into_inner())
                                .entry(f.to_string())
                                .or_default()
                                .push((args.to_vec(), gives.clone()));
                        }
                        gives
                    }
                }
            }
            None if callee.gives.open() => Err(Fault::Op(Undefined::Unknown(f.to_string()))),
            // The world gives a value: not knowing it is no fault.
            None => Ok(self.promised(f, callee, args)),
        };
        // What it gives, cut to its range: by its body, outside is no
        // value; the world's value is in it (check and evidence say so).
        let gives = match (gives, &callee.ranges.gives) {
            (Ok(b), Some((_, range))) => match range.cut(&b) {
                Some((cut, out)) => Ok(Bounds {
                    may_fault: cut.may_fault || (out && callee.body.is_some()),
                    ..cut
                }),
                None if callee.body.is_some() => Err(Fault::Op(Undefined::Outside(format!(
                    "`{f}` gives in {range}, and its body gives none of it"
                )))),
                None => Ok(range.bounds().unwrap_or(b)),
            },
            (gives, _) => gives,
        };
        let may_fault = outside || args.iter().any(|b| b.may_fault);
        gives.map(|b| Bounds {
            may_fault: b.may_fault || may_fault,
            ..b
        })
    }
}

type Bounded = Outcome<Bounds, Undefined>;

/// The smallest span holding both. `None` for two of different kinds.
fn hull(a: &Span, b: &Span) -> Option<Span> {
    Some(match (a, b) {
        (Span::Truth { yes: a, no: b }, Span::Truth { yes: c, no: d }) => Span::Truth {
            yes: *a || *c,
            no: *b || *d,
        },
        (Span::Number(a, al, ah), Span::Number(b, bl, bh)) if a == b => Span::Number(
            *a,
            End::low(al).min(End::low(bl)).finite(),
            End::high(ah).max(End::high(bh)).finite(),
        ),
        (Span::Record(r, xs), Span::Record(s, ys)) if r == s && xs.len() == ys.len() => {
            let fields = xs.iter().zip(ys).map(|(x, y)| hull(x, y));
            Span::Record(r.clone(), fields.collect::<Option<_>>()?)
        }
        // Any tree, joined with a tree, is any tree.
        (Span::Variant(e, _), Span::Variant(f, _)) if e == f && (e.cut() || f.cut()) => {
            Span::Variant(if e.cut() { e.clone() } else { f.clone() }, vec![])
        }
        (Span::Variant(e, xs), Span::Variant(f, ys)) if e == f && xs.len() == ys.len() => {
            let mut may = Vec::with_capacity(xs.len());
            for (x, y) in xs.iter().zip(ys) {
                may.push(match (x, y) {
                    (None, None) => None,
                    (Some(x), None) | (None, Some(x)) => Some(x.clone()),
                    (Some(x), Some(y)) if x.len() == y.len() => Some(
                        x.iter()
                            .zip(y)
                            .map(|(a, b)| hull(a, b))
                            .collect::<Option<_>>()?,
                    ),
                    _ => return None,
                });
            }
            Span::Variant(e.clone(), may)
        }
        (Span::Text(al, a), Span::Text(bl, b)) => {
            Span::Text(Box::new(hull(al, bl)?), a.clone().filter(|_| a == b))
        }
        (Span::Byte(a), Span::Byte(b)) => Span::Byte(a.filter(|_| a == b)),
        (Span::Char, Span::Char) => Span::Char,
        (Span::Rest, Span::Rest) => Span::Rest,
        // The first places both surely hold, each joined.
        (Span::Seq(t, al, ae, af), Span::Seq(u, bl, be, bf)) if t == u => Span::Seq(
            t.clone(),
            Box::new(hull(al, bl)?),
            Box::new(hull(ae, be)?),
            af.iter()
                .zip(bf)
                .map(|(a, b)| hull(a, b))
                .collect::<Option<_>>()?,
        ),
        _ => return None,
    })
}

/// What either may give.
fn join(a: Bounded, b: Bounded) -> Bounded {
    match (a, b) {
        (Err(Fault::Stuck), _) | (_, Err(Fault::Stuck)) => Err(Fault::Stuck),
        // A side read past its budget tells nothing of what it gives:
        // said, not taken for a fault (finding 24).
        (Err(Fault::Op(Undefined::Budget(why))), _)
        | (_, Err(Fault::Op(Undefined::Budget(why)))) => Err(Fault::Op(Undefined::Budget(why))),
        (Err(fault), Err(_)) => Err(fault),
        (Err(_), Ok(b)) | (Ok(b), Err(_)) => Ok(Bounds {
            may_fault: true,
            ..b
        }),
        (Ok(a), Ok(b)) => match hull(&a.span, &b.span) {
            Some(span) => Ok(Bounds {
                span,
                may_fault: a.may_fault || b.may_fault,
            }),
            None => Err(Fault::Stuck),
        },
    }
}

/// What a `match` on `scrutinee` may give: what the arm of each variant
/// it may be gives, joined, and a fault if it may fault.
fn arms(
    whole: &Enum,
    scrutinee: &Bounded,
    arm: &mut dyn FnMut(usize, &[Span]) -> Bounded,
) -> Bounded {
    let (may, may_fault) = match scrutinee {
        // Any tree where a tree holds itself: any variant of the enum the
        // `match` reads.
        Ok(Bounds {
            span: Span::Variant(e, _),
            may_fault,
        }) if e.cut() => match Bounds::any(&Type::Enum(whole.clone())).span {
            Span::Variant(_, may) => (may, *may_fault),
            _ => return Err(Fault::Stuck),
        },
        Ok(Bounds {
            span: Span::Variant(_, may),
            may_fault,
        }) => (may.clone(), *may_fault),
        Ok(_) => return Err(Fault::Stuck),
        Err(fault) => return Err(fault.clone()),
    };
    let mut gives: Option<Bounded> = None;
    for (k, holds) in may.iter().enumerate() {
        if let Some(holds) = holds {
            let this = arm(k, holds);
            gives = Some(match gives {
                None => this,
                Some(before) => join(before, this),
            });
        }
    }
    match gives {
        None => Err(Fault::Stuck),
        Some(Ok(b)) => Ok(Bounds {
            may_fault: b.may_fault || may_fault,
            ..b
        }),
        Some(fault) => fault,
    }
}

/// What `if test { then } else { otherwise }` may give: what each
/// branch the test may take gives, and a fault if the test may fault.
fn choose(test: &Bounded, then: Bounded, otherwise: Bounded) -> Bounded {
    let (yes, no, may_fault) = match test {
        Ok(Bounds {
            span: Span::Truth { yes, no },
            may_fault,
        }) => (*yes, *no, *may_fault),
        Ok(_) => return Err(Fault::Stuck),
        Err(fault) => return Err(fault.clone()),
    };
    let taken = match (yes, no) {
        (true, true) => join(then, otherwise),
        (true, false) => then,
        (false, true) => otherwise,
        (false, false) => return Err(Fault::Stuck),
    };
    match taken {
        Ok(b) => Ok(Bounds {
            may_fault: b.may_fault || may_fault,
            ..b
        }),
        other => other,
    }
}

fn said(b: bool) -> Bounded {
    Ok(Bounds::exactly(&Value::Truth(b)))
}

/// A range of numbers: its low end and its high end, `None` for none.
type Ends = (Option<Rational>, Option<Rational>);

fn ends(low: &Option<Rational>, high: &Option<Rational>) -> (End, End) {
    (End::low(low), End::high(high))
}

/// `+`, `-`, `*` on two ranges: the least and the most of what the ends
/// give. An end that is no end gives none.
fn arith(op: &Op, of: Numbers, a: &Ends, b: &Ends) -> Span {
    let ((al, ah), (bl, bh)) = (ends(&a.0, &a.1), ends(&b.0, &b.1));
    let at = |x: &End, y: &End, f: fn(&Rational, &Rational) -> Rational| match (x, y) {
        (End::At(x), End::At(y)) => Some(f(x, y)),
        _ => None,
    };
    match op {
        Op::Add => Span::Number(
            of,
            at(&al, &bl, Rational::plus),
            at(&ah, &bh, Rational::plus),
        ),
        Op::Sub => Span::Number(
            of,
            at(&al, &bh, Rational::minus),
            at(&ah, &bl, Rational::minus),
        ),
        _ => {
            let all = [al.times(&bl), al.times(&bh), ah.times(&bl), ah.times(&bh)];
            let low = all.iter().min().cloned().and_then(End::finite);
            let high = all.iter().max().cloned().and_then(End::finite);
            Span::Number(of, low, high)
        }
    }
}

/// `a / b` on two ranges: a fault when `b` is zero; any number, and a
/// possible fault, when `b` may be zero; else the least and the most of
/// what the ends give: a finite number over no end is near zero, and no
/// end over no end any number.
fn divide(of: Numbers, a: &Ends, b: &Ends, may_fault: bool) -> Bounded {
    let zero = End::At(Rational::integer(0));
    let ((al, ah), (bl, bh)) = (ends(&a.0, &a.1), ends(&b.0, &b.1));
    if bl == zero && bh == zero {
        return Err(Fault::Op(Undefined::DivisionByZero));
    }
    if !(zero < bl || bh < zero) {
        return Ok(Bounds {
            span: Span::Number(of, None, None),
            may_fault: true,
        });
    }
    let over = |x: &End, y: &End| -> Option<End> {
        Some(match (x, y) {
            (End::At(x), End::At(y)) => End::At(x.over(y)?),
            (End::At(_), _) => zero.clone(),
            // No end over a number: no end, of the sign of the two.
            (_, End::At(y)) => x.times(&End::At(Rational::integer(
                y.cmp(&Rational::integer(0)) as i64
            ))),
            _ => return None,
        })
    };
    let all = [
        over(&al, &bl),
        over(&al, &bh),
        over(&ah, &bl),
        over(&ah, &bh),
    ];
    let (low, high) = match all.into_iter().collect::<Option<Vec<End>>>() {
        Some(all) => (
            all.iter().min().cloned().and_then(End::finite),
            all.iter().max().cloned().and_then(End::finite),
        ),
        None => (None, None),
    };
    // On `Int`, the quotient rounded toward zero: rounding keeps order,
    // so the ends rounded are the ends.
    let (low, high) = match of {
        Numbers::Int => (low.map(|q| q.toward_zero()), high.map(|q| q.toward_zero())),
        Numbers::Real => (low, high),
    };
    Ok(Bounds {
        span: Span::Number(of, low, high),
        may_fault,
    })
}

/// `a % b` on two ranges of `Int`: a fault when `b` is zero; else of the
/// sign of `a`, no farther from zero than `a`, and nearer than `b`. A
/// possible fault when `b` may be zero.
fn remainder(a: &Ends, b: &Ends, may_fault: bool) -> Bounded {
    let zero = Rational::integer(0);
    let one = Rational::integer(1);
    let is = |e: &Option<Rational>, q: &Rational| e.as_ref() == Some(q);
    if is(&b.0, &zero) && is(&b.1, &zero) {
        return Err(Fault::Op(Undefined::DivisionByZero));
    }
    let may_be_zero =
        End::low(&b.0) <= End::At(zero.clone()) && End::At(zero.clone()) <= End::high(&b.1);
    // At two points, the remainder itself.
    if let ((Some(al), Some(ah)), (Some(bl), Some(bh))) = (a, b) {
        if al == ah && bl == bh {
            let r = al.to_integer().zip(bl.to_integer()).map(|(x, y)| x % y);
            if let Some(r) = r {
                return Ok(Bounds {
                    span: point(Numbers::Int, Rational::integer(r)),
                    may_fault,
                });
            }
        }
    }
    // The most |b| - 1 may be: none when b has no end.
    let most = match b {
        (Some(l), Some(h)) => Some(l.abs().max(h.abs()).minus(&one)),
        _ => None,
    };
    let low = match &a.0 {
        Some(l) if *l >= zero => Some(zero.clone()),
        l => match (l, &most) {
            (Some(l), Some(m)) => Some(l.clone().max(zero.minus(m))),
            (Some(l), None) => Some(l.clone()),
            (None, Some(m)) => Some(zero.minus(m)),
            (None, None) => None,
        },
    };
    let high = match &a.1 {
        Some(h) if *h <= zero => Some(zero.clone()),
        h => match (h, &most) {
            (Some(h), Some(m)) => Some(h.clone().min(m.clone())),
            (Some(h), None) => Some(h.clone()),
            (None, Some(m)) => Some(m.clone()),
            (None, None) => None,
        },
    };
    Ok(Bounds {
        span: Span::Number(Numbers::Int, low, high),
        may_fault: may_fault || may_be_zero,
    })
}

/// `a < b`, or `a <= b` when `or_equal`: whether it may be true, and
/// may be false.
fn less(a: &Ends, b: &Ends, or_equal: bool) -> Span {
    let ((al, ah), (bl, bh)) = (ends(&a.0, &a.1), ends(&b.0, &b.1));
    let (yes, no) = match or_equal {
        true => (al <= bh, ah > bl),
        false => (al < bh, ah >= bl),
    };
    Span::Truth { yes, no }
}

/// Whether two values in these spans may be equal, and may differ.
/// `None` for spans of different kinds.
fn compare(a: &Span, b: &Span) -> Option<(bool, bool)> {
    Some(match (a, b) {
        (Span::Truth { yes: ay, no: an }, Span::Truth { yes: by, no: bn }) => {
            ((*ay && *by) || (*an && *bn), (*ay && *bn) || (*an && *by))
        }
        (Span::Number(a, al, ah), Span::Number(b, bl, bh)) if a == b => {
            let ((al, ah), (bl, bh)) = (ends(al, ah), ends(bl, bh));
            let one = |l: &End, h: &End| matches!(l, End::At(_)) && l == h;
            (
                al <= bh && bl <= ah,
                !(one(&al, &ah) && one(&bl, &bh) && al == bl),
            )
        }
        (Span::Record(r, xs), Span::Record(s, ys)) if r == s && xs.len() == ys.len() => {
            let (mut equal, mut differ) = (true, false);
            for (x, y) in xs.iter().zip(ys) {
                let (e, d) = compare(x, y)?;
                equal &= e;
                differ |= d;
            }
            (equal, differ)
        }
        // Equal only in one variant both may be, each value it holds
        // equal; different in two variants, or in what one holds.
        // Any tree may be equal to a tree, or not.
        (Span::Variant(e, _), Span::Variant(f, _)) if e == f && (e.cut() || f.cut()) => {
            (true, true)
        }
        (Span::Variant(e, xs), Span::Variant(f, ys)) if e == f && xs.len() == ys.len() => {
            let may: Vec<usize> = (0..xs.len()).filter(|k| xs[*k].is_some()).collect();
            let other: Vec<usize> = (0..ys.len()).filter(|k| ys[*k].is_some()).collect();
            let (mut equal, mut differ) = (false, may.iter().any(|k| other.iter().any(|j| j != k)));
            for k in &may {
                if let (Some(x), Some(y)) = (&xs[*k], &ys[*k]) {
                    if x.len() != y.len() {
                        return None;
                    }
                    let (mut all_equal, mut one_differs) = (true, false);
                    for (a, b) in x.iter().zip(y) {
                        let (e, d) = compare(a, b)?;
                        all_equal &= e;
                        one_differs |= d;
                    }
                    equal |= all_equal;
                    differ |= one_differs;
                }
            }
            (equal, differ)
        }
        // Text: decided when both are known; else equal only at one
        // length both may have.
        (Span::Text(al, a), Span::Text(bl, b)) => match (a, b) {
            (Some(a), Some(b)) => (a == b, a != b),
            _ => (compare(al, bl)?.0, true),
        },
        (Span::Byte(a), Span::Byte(b)) => match (a, b) {
            (Some(a), Some(b)) => (a == b, a != b),
            _ => (true, true),
        },
        (Span::Char, Span::Char) | (Span::Rest, Span::Rest) => (true, true),
        // Equal only at one length both may have, each value equal; they
        // differ in length, or in a value, unless both surely hold none.
        // Two of one length, each place known: place by place.
        (Span::Seq(t, al, _, af), Span::Seq(u, bl, _, bf))
            if t == u
                && one_length(al) == Some(af.len())
                && one_length(bl) == Some(af.len())
                && bf.len() == af.len() =>
        {
            let (mut equal, mut differ) = (true, false);
            for (x, y) in af.iter().zip(bf) {
                let (e, d) = compare(x, y)?;
                equal &= e;
                differ |= d;
            }
            (equal, differ)
        }
        (Span::Seq(t, al, ae, _), Span::Seq(u, bl, be, _)) if t == u => {
            let (same_length, other_length) = compare(al, bl)?;
            let (one_equal, one_differs) = compare(ae, be)?;
            let zero = Rational::integer(0);
            let may_be_empty =
                |l: &Span| matches!(l, Span::Number(_, Some(low), _) if low == &zero);
            let surely_empty = |l: &Span| matches!(l, Span::Number(_, Some(low), Some(high)) if low == &zero && high == &zero);
            (
                same_length && (one_equal || (may_be_empty(al) && may_be_empty(bl))),
                other_length || (one_differs && !(surely_empty(al) && surely_empty(bl))),
            )
        }
        _ => return None,
    })
}

/// `|a - b| <= tol`: whether it may be true, and may be false.
fn within(a: &Ends, b: &Ends, tol: &Ends) -> Span {
    let zero = End::At(Rational::integer(0));
    let Span::Number(_, low, high) = arith(&Op::Sub, Numbers::Real, a, b) else {
        return Span::Truth {
            yes: true,
            no: true,
        };
    };
    let (low, high) = ends(&low, &high);
    let (tl, th) = ends(&tol.0, &tol.1);
    let (near, far) = if low > zero {
        (low, high.abs())
    } else if high < zero {
        (high.abs(), low.abs())
    } else {
        (zero, low.abs().max(high.abs()))
    };
    Span::Truth {
        yes: near <= th,
        no: far > tl,
    }
}

/// `xs[i]` on bounds: any value `xs` holds, which may be none when `i`
/// may be below zero or at its length or past it; none at all when `i`
/// surely is.
fn index(
    length: &Span,
    each: &Span,
    first: &[Span],
    at: (&Option<Rational>, &Option<Rational>),
    may_fault: bool,
    text: bool,
) -> Bounded {
    let Span::Number(_, low, high) = length else {
        return Err(Fault::Stuck);
    };
    let zero = End::At(Rational::integer(0));
    let (il, ih) = ends(at.0, at.1);
    let (ll, lh) = ends(low, high);
    // The highest place a value may be at, and the lowest it surely is.
    let one = Rational::integer(1);
    let top = match &lh {
        End::At(h) => End::At(h.minus(&one)),
        other => other.clone(),
    };
    let surely_held = match &ll {
        End::At(l) => End::At(l.minus(&one)),
        other => other.clone(),
    };
    if ih < zero || il > top {
        // At one index and one length, said as `Exact` says it.
        let why = match (&il, &ih, &ll, &lh) {
            (End::At(i), End::At(j), End::At(n), End::At(m)) if i == j && n == m => match text {
                true => format!(
                    "the index is {}, and the text holds {} bytes",
                    i.numer(),
                    n.numer()
                ),
                false => format!(
                    "the index is {}, and the sequence holds {}",
                    i.numer(),
                    n.numer()
                ),
            },
            _ => "the index lies outside every place the sequence may have".to_string(),
        };
        return Err(Fault::Op(Undefined::Index(why)));
    }
    // At one place surely held: what is known of it.
    let span = match (&il, &ih) {
        (End::At(i), End::At(j)) if i == j => usize::try_from(i.numer())
            .ok()
            .and_then(|k| first.get(k))
            .unwrap_or(each)
            .clone(),
        _ => each.clone(),
    };
    Ok(Bounds {
        span,
        may_fault: may_fault || il < zero || ih > surely_held,
    })
}

/// The length of a join: the two lengths added.
fn join_lengths(a: &Span, b: &Span) -> Result<Span, Fault<Undefined>> {
    match (a, b) {
        (Span::Number(Numbers::Int, al, ah), Span::Number(Numbers::Int, bl, bh)) => Ok(arith(
            &Op::Add,
            Numbers::Int,
            &(al.clone(), ah.clone()),
            &(bl.clone(), bh.clone()),
        )),
        _ => Err(Fault::Stuck),
    }
}

/// A truth, said.
fn said_span(b: bool) -> Span {
    Span::Truth { yes: b, no: !b }
}

/// The prefix's length may be at most the whole's: else it surely does
/// not start the whole.
fn compare_lengths_may_fit(whole: &Span, prefix: &Span) -> bool {
    match (whole, prefix) {
        (Span::Number(_, _, wh), Span::Number(_, pl, _)) => End::low(pl) <= End::high(wh),
        _ => true,
    }
}

/// A sequence holding values in these spans, each place known: of one
/// length, `None` when two spans are of different kinds.
fn held(t: &Type, holds: Vec<Span>) -> Option<Span> {
    let mut each: Option<Span> = None;
    for s in &holds {
        each = Some(match each {
            None => s.clone(),
            Some(e) => hull(&e, s)?,
        });
    }
    Some(Span::Seq(
        t.clone(),
        Box::new(point(
            Numbers::Int,
            Rational::integer(BigInt::from(holds.len())),
        )),
        Box::new(each.unwrap_or_else(|| Bounds::any(t).span)),
        holds,
    ))
}

/// The length of a sequence when it is one number, and fits a `usize`.
fn one_length(length: &Span) -> Option<usize> {
    match length {
        Span::Number(_, Some(l), Some(h)) if l == h => usize::try_from(l.numer()).ok(),
        _ => None,
    }
}

impl Reading for Ranges {
    type Standard = Simulation;
    type Value = Bounds;
    type Fault = Undefined;

    fn has_type(&self, value: &Bounds, of: &Type) -> bool {
        fits(&value.span, of)
    }

    fn apply(&self, op: &Op, args: &[Bounded]) -> Bounded {
        // Two small frames, so that the large one is not held while a
        // callee's body runs.
        match op {
            Op::Call(f) => self.call_strict(f, args),
            _ => self.plain(op, args),
        }
    }

    fn bind(
        &self,
        op: &Op,
        args: &[Bounded],
        run: &mut dyn FnMut(usize, &[Bounded]) -> Bounded,
    ) -> Bounded {
        match (op, args) {
            (Op::Let, [value, _]) => run(1, std::slice::from_ref(value)),
            (Op::All, [lo, hi, _]) => all(lo, hi, run),
            // A loop by bounds: any value of what it carries, perhaps no
            // value (a fault inside, or the budget).
            (Op::Fold, [xs, start, _]) => loop_bounds(Some(xs), start),
            (Op::Until, [start, _, _]) => loop_bounds(None, start),
            (Op::Match(whole), [scrutinee, ..]) => match_bounds(whole, scrutinee, run),
            _ => Err(Fault::Stuck),
        }
    }
}

/// `match` by bounds, in a frame of its own: `bind` is on the path of
/// every level.
#[inline(never)]
fn match_bounds(
    whole: &Enum,
    scrutinee: &Bounded,
    run: &mut dyn FnMut(usize, &[Bounded]) -> Bounded,
) -> Bounded {
    let mut arm = |k: usize, holds: &[Span]| {
        let holds: Vec<Bounded> = holds
            .iter()
            .map(|s| {
                Ok(Bounds {
                    span: s.clone(),
                    may_fault: false,
                })
            })
            .collect();
        run(1 + k, &holds)
    };
    arms(whole, scrutinee, &mut arm)
}

impl Ranges {
    /// What a function with no body gives, by bounds: any value of its
    /// type, cut by what its claims promise, read at these arguments.
    /// Promises that keep nothing cut nothing (agree says so, A004).
    fn promised(&self, f: &str, callee: &Callee, args: &[Bounds]) -> Bounds {
        let any = Bounds::any(&callee.gives);
        let Some(p) = callee.promises.as_ref().filter(|p| !p.claims.is_empty()) else {
            return any;
        };
        // A promise may call the function again: there, only its type.
        let promising = || {
            self.calls
                .memo
                .promising
                .lock()
                .unwrap_or_else(|e| e.into_inner())
        };
        if !promising().insert(f.to_string()) {
            return any;
        }
        let gives = self.promised_by(f, callee, p, args, any);
        promising().remove(f);
        gives
    }

    fn promised_by(
        &self,
        f: &str,
        callee: &Callee,
        p: &Promises,
        args: &[Bounds],
        any: Bounds,
    ) -> Bounds {
        let Ok(_active) = self.calls.enter(f) else {
            return any;
        };
        let mut over = callee.takes.clone();
        over.push((p.gives.clone(), callee.gives.clone()));
        let mut bounds: BTreeMap<String, Bounds> = callee
            .takes
            .iter()
            .map(|(n, _)| n.clone())
            .zip(args.iter().cloned())
            .collect();
        bounds.insert(p.gives.clone(), any.clone());
        let all = p
            .claims
            .iter()
            .map(|(_, c)| c.clone())
            .reduce(|a, b| Expression::Apply(Op::And, vec![plain_arg(a), plain_arg(b)]))
            .expect("not empty");
        match self.kept(&over, &bounds, &all) {
            Some(kept) => {
                let gives = kept.get(&p.gives).cloned().unwrap_or(any);
                if gives != Bounds::any(&callee.gives) {
                    let mut read = self
                        .calls
                        .memo
                        .read
                        .lock()
                        .unwrap_or_else(|e| e.into_inner());
                    read.extend(p.claims.iter().map(|(path, _)| path.clone()));
                }
                gives
            }
            None => any,
        }
    }

    /// The bounds of each name in `over` that `e` keeps, when `e` is
    /// true: each comparison of a name with what bounds give of the other
    /// side, among the `&&` at the top of `e`, cuts that name's span, one
    /// name at a time. `None` when a span is cut to nothing: `e` is true
    /// nowhere in `bounds`. Strict on `Int`, not on `Real`, where the end
    /// stays in.
    pub fn kept(
        &self,
        over: &[(String, Type)],
        bounds: &BTreeMap<String, Bounds>,
        e: &Expression<Op>,
    ) -> Option<BTreeMap<String, Bounds>> {
        let mut kept = bounds.clone();
        let mut todo = vec![e];
        while let Some(e) = todo.pop() {
            let Expression::Apply(op, args) = e else {
                continue;
            };
            if args.iter().any(|a| !a.binds.is_empty()) {
                continue;
            }
            let (a, b) = match (op, args.as_slice()) {
                (Op::And, [a, b]) => {
                    todo.extend([&a.body, &b.body]);
                    continue;
                }
                (Op::Less | Op::AtMost | Op::Equal, [a, b]) => (&a.body, &b.body),
                _ => continue,
            };
            // `n < x` cuts n's high end, `x < n` its low end.
            for (name, other, n_low) in [(a, b, false), (b, a, true)] {
                let Expression::Name(n) = name else { continue };
                let Some(Bounds {
                    span: Span::Number(numbers, low, high),
                    may_fault,
                }) = kept.get(n).cloned()
                else {
                    continue;
                };
                let whole =
                    numbers == Numbers::Int || over.iter().any(|(m, t)| m == n && t == &Type::Int);
                let Ok(Bounds {
                    span: Span::Number(_, ol, oh),
                    ..
                }) = eval(self, &kept, other)
                else {
                    continue;
                };
                let strict = whole && matches!(op, Op::Less);
                let one = Rational::integer(1);
                let (mut low, mut high) = (low, high);
                if n_low || matches!(op, Op::Equal) {
                    // x <= n: n is at least the least x; x < n on Int, one more.
                    if let Some(l) = &ol {
                        let l = match whole {
                            true if strict => l.floor().plus(&one),
                            true => l.ceil(),
                            false => l.clone(),
                        };
                        low = Some(low.map_or(l.clone(), |x| x.max(l)));
                    }
                }
                if !n_low || matches!(op, Op::Equal) {
                    if let Some(h) = &oh {
                        let h = match whole {
                            true if strict => h.ceil().minus(&one),
                            true => h.floor(),
                            false => h.clone(),
                        };
                        high = Some(high.map_or(h.clone(), |x| x.min(h)));
                    }
                }
                if let (Some(l), Some(h)) = (&low, &high) {
                    if l > h {
                        return None;
                    }
                }
                kept.insert(
                    n.clone(),
                    Bounds {
                        span: Span::Number(numbers, low, high),
                        may_fault,
                    },
                );
            }
        }
        Some(kept)
    }
}

impl Ranges {
    #[inline(never)]
    fn call_strict(&self, f: &str, args: &[Bounded]) -> Bounded {
        self.call(f, &strict(args)?)
    }
    /// Every operation but a call.
    #[inline(never)]
    fn plain(&self, op: &Op, args: &[Bounded]) -> Bounded {
        match (op, args) {
            (Op::If, [test, then, otherwise]) => {
                return choose(test, then.clone(), otherwise.clone())
            }
            (Op::And, [a, b]) => return choose(a, b.clone(), said(false)),
            // Arms that bind nothing: each computed, the possible joined.
            (Op::Match(whole), [scrutinee, given @ ..]) => {
                return arms(whole, scrutinee, &mut |k, _| {
                    given.get(k).cloned().unwrap_or(Err(Fault::Stuck))
                })
            }
            (Op::Or, [a, b]) => return choose(a, said(true), b.clone()),
            _ => {}
        }
        let args = strict(args)?;
        let may_fault = args.iter().any(|b| b.may_fault);
        let spans: Vec<&Span> = args.iter().map(|b| &b.span).collect();
        let n = |s: &Span| match s {
            Span::Number(of, low, high) => Some((*of, (low.clone(), high.clone()))),
            _ => None,
        };
        let typed: Vec<(Numbers, Ends)> = spans.iter().filter_map(|s| n(s)).collect();
        // Numbers of one type only, as the types say.
        let of = typed.first().map_or(Numbers::Real, |(of, _)| *of);
        let numbers: Vec<Ends> =
            if typed.len() == spans.len() && typed.iter().all(|(o, _)| *o == of) {
                typed.into_iter().map(|(_, e)| e).collect()
            } else {
                vec![]
            };
        let span = match (op, spans.as_slice()) {
            (Op::Truth(_) | Op::Int(_) | Op::Real(_), []) => {
                let exact = Exact::default().apply(op, &[])?;
                Bounds::exactly(&exact).span
            }
            (Op::Not, [Span::Truth { yes, no }]) => Span::Truth { yes: *no, no: *yes },
            (Op::Add | Op::Sub | Op::Mul, [_, _]) if numbers.len() == 2 => {
                arith(op, of, &numbers[0], &numbers[1])
            }
            (Op::Div, [_, _]) if numbers.len() == 2 => {
                return divide(of, &numbers[0], &numbers[1], may_fault)
            }
            (Op::Rem, [_, _]) if numbers.len() == 2 && of == Numbers::Int => {
                return remainder(&numbers[0], &numbers[1], may_fault)
            }
            (Op::Less, [_, _]) if numbers.len() == 2 => less(&numbers[0], &numbers[1], false),
            (Op::AtMost, [_, _]) if numbers.len() == 2 => less(&numbers[0], &numbers[1], true),
            (Op::Equal, [a, b]) => match compare(a, b) {
                Some((yes, no)) => Span::Truth { yes, no },
                None => return Err(Fault::Stuck),
            },
            (Op::Within, [_, _, _]) if numbers.len() == 3 => {
                within(&numbers[0], &numbers[1], &numbers[2])
            }
            (Op::Make(r), fields) if fits_fields(fields, r, |s, f| fits(s, f)) => {
                Span::Record(r.clone(), fields.iter().map(|s| (*s).clone()).collect())
            }
            (Op::Variant(e, i), holds)
                if e.variants.get(*i).is_some_and(|(_, ts)| {
                    ts.len() == holds.len() && holds.iter().zip(ts).all(|(s, t)| fits(s, t))
                }) =>
            {
                Span::Variant(
                    e.clone(),
                    (0..e.variants.len())
                        .map(|k| (k == *i).then(|| holds.iter().map(|s| (*s).clone()).collect()))
                        .collect(),
                )
            }
            (Op::Field(name), [Span::Record(r, spans)]) => {
                match r.field(name).and_then(|(place, _)| spans.get(place)) {
                    Some(s) => s.clone(),
                    None => return Err(Fault::Stuck),
                }
            }
            (Op::Seq(t), holds) if holds.iter().all(|s| fits(s, t)) => {
                held(t, holds.iter().map(|s| (*s).clone()).collect()).ok_or(Fault::Stuck)?
            }
            (Op::Len, [Span::Seq(_, length, _, _)]) => (**length).clone(),
            (
                Op::Index,
                [Span::Seq(_, length, each, first), Span::Number(Numbers::Int, il, ih)],
            ) => return index(length, each, first, (il, ih), may_fault, false),
            // Text: what is known of it; its bytes when it is known.
            (Op::Text(_), []) => {
                let exact = Exact::default().apply(op, &[])?;
                Bounds::exactly(&exact).span
            }
            (Op::Len, [Span::Text(length, _)]) => (**length).clone(),
            (Op::Index, [Span::Text(length, known), Span::Number(Numbers::Int, il, ih)]) => {
                let bytes: Vec<Span> = known
                    .iter()
                    .flat_map(|k| k.bytes().map(|b| Span::Byte(Some(b))))
                    .collect();
                return index(length, &Span::Byte(None), &bytes, (il, ih), may_fault, true);
            }
            (Op::Join, [Span::Text(al, a), Span::Text(bl, b)]) => {
                let length = join_lengths(al, bl)?;
                let known = match (a, b) {
                    (Some(a), Some(b)) => Some(format!("{a}{b}")),
                    _ => None,
                };
                Span::Text(Box::new(length), known)
            }
            (Op::Join, [Span::Seq(t, al, ae, af), Span::Seq(u, bl, be, bf)]) if t == u => {
                let length = join_lengths(al, bl)?;
                // The places surely held: all of `a`'s, then `b`'s when
                // `a` is of one length.
                let first = match one_length(al) {
                    Some(n) if n == af.len() => [af.clone(), bf.clone()].concat(),
                    _ => af.clone(),
                };
                // An empty side adds nothing to what each place may be.
                let each = match (one_length(al), one_length(bl)) {
                    (Some(0), _) => (**be).clone(),
                    (_, Some(0)) => (**ae).clone(),
                    _ => hull(ae, be).ok_or(Fault::Stuck)?,
                };
                Span::Seq(t.clone(), Box::new(length), Box::new(each), first)
            }
            (Op::StartsWith, [Span::Text(al, a), Span::Text(bl, b)]) => match (a, b) {
                (Some(a), Some(b)) => said_span(a.starts_with(b.as_str())),
                (_, Some(b)) if b.is_empty() => said_span(true),
                _ => Span::Truth {
                    yes: compare_lengths_may_fit(al, bl),
                    no: true,
                },
            },
            (Op::StartsWith, [Span::Seq(t, al, _, af), Span::Seq(u, bl, _, bf)]) if t == u => {
                match (one_length(al), one_length(bl)) {
                    (_, Some(0)) => said_span(true),
                    (Some(n), Some(m)) if n == af.len() && m == bf.len() => {
                        if m > n {
                            said_span(false)
                        } else {
                            let (mut yes, mut no) = (true, false);
                            for (x, y) in af.iter().zip(bf) {
                                let (e, d) = compare(x, y).ok_or(Fault::Stuck)?;
                                yes &= e;
                                no |= d;
                            }
                            Span::Truth { yes, no }
                        }
                    }
                    _ => Span::Truth {
                        yes: compare_lengths_may_fit(al, bl),
                        no: true,
                    },
                }
            }
            (Op::Less | Op::AtMost, [Span::Text(_, a), Span::Text(_, b)]) => match (a, b) {
                (Some(a), Some(b)) => said_span(match op {
                    Op::Less => a.as_bytes() < b.as_bytes(),
                    _ => a.as_bytes() <= b.as_bytes(),
                }),
                _ => Span::Truth {
                    yes: true,
                    no: true,
                },
            },
            (Op::Less | Op::AtMost, [Span::Char, Span::Char]) => Span::Truth {
                yes: true,
                no: true,
            },
            (Op::Decoded, [s @ Span::Seq(..)]) => (*s).clone(),
            _ => return Err(Fault::Stuck),
        };
        Ok(Bounds { span, may_fault })
    }
}

/// `(lo..hi).all(|x| p)` on bounds. When `lo` and `hi` are each one
/// number, `p` is read at each number in turn, as `&&` reads it, so
/// nothing is lost on points. Else `p` is read once, with `x` anywhere
/// the range may reach, and the range may be empty.
#[inline(never)]
fn all(lo: &Bounded, hi: &Bounded, run: &mut dyn FnMut(usize, &[Bounded]) -> Bounded) -> Bounded {
    let (lo, hi) = match (lo, hi) {
        (Ok(lo), Ok(hi)) => (lo, hi),
        (Err(fault), _) | (_, Err(fault)) => return Err(fault.clone()),
    };
    let gives = match (&lo.span, &hi.span) {
        (Span::Number(_, Some(ll), Some(lh)), Span::Number(_, Some(hl), Some(hh)))
            if ll == lh && hl == hh =>
        {
            all_points(ll, hl, run)
        }
        (Span::Number(_, ll, lh), Span::Number(_, hl, hh)) => {
            all_once(&(ll.clone(), lh.clone()), &(hl.clone(), hh.clone()), run)
        }
        _ => Err(Fault::Stuck),
    };
    gives.map(|b| Bounds {
        may_fault: b.may_fault || lo.may_fault || hi.may_fault,
        ..b
    })
}

/// `p` at each number from `lo` up to `hi`, as `&&` reads it, until the
/// outcome can no longer be true: no number after can change it.
#[inline(never)]
fn all_points(
    lo: &Rational,
    hi: &Rational,
    run: &mut dyn FnMut(usize, &[Bounded]) -> Bounded,
) -> Bounded {
    let (mut x, mut so_far) = (lo.clone(), said(true));
    let may_be_true = |b: &Bounded| {
        matches!(
            b,
            Ok(Bounds {
                span: Span::Truth { yes: true, .. },
                ..
            })
        )
    };
    while &x < hi && may_be_true(&so_far) {
        so_far = and_at(so_far, &x, run);
        x = x.plus(&Rational::integer(1));
    }
    so_far
}

/// `so_far && p`, `p` read with `x` at one number.
#[inline(never)]
fn and_at(
    so_far: Bounded,
    x: &Rational,
    run: &mut dyn FnMut(usize, &[Bounded]) -> Bounded,
) -> Bounded {
    let p = read_at(point(Numbers::Int, x.clone()), run);
    choose(&so_far, p, said(false))
}

/// `p` read with `x` in `span`.
#[inline(never)]
fn read_at(span: Span, run: &mut dyn FnMut(usize, &[Bounded]) -> Bounded) -> Bounded {
    run(
        2,
        &[Ok(Bounds {
            span,
            may_fault: false,
        })],
    )
}

/// `p` once, with `x` anywhere from the least `lo` to the most `hi`, `hi`
/// left out. True when the range may be empty.
#[inline(never)]
fn all_once(low: &Ends, high: &Ends, run: &mut dyn FnMut(usize, &[Bounded]) -> Bounded) -> Bounded {
    let top = high.1.as_ref().map(|hh| hh.minus(&Rational::integer(1)));
    if let (Some(ll), Some(top)) = (&low.0, &top) {
        if top < ll {
            return said(true);
        }
    }
    let may_be_empty = match (&low.1, &high.0) {
        (Some(lh), Some(hl)) => lh >= hl,
        _ => true,
    };
    match read_at(Span::Number(Numbers::Int, low.0.clone(), top), run) {
        Ok(Bounds {
            span: Span::Truth { yes, no },
            may_fault,
        }) => Ok(Bounds {
            span: Span::Truth {
                yes: yes || may_be_empty,
                no,
            },
            may_fault,
        }),
        Ok(_) => Err(Fault::Stuck),
        Err(Fault::Op(_)) if may_be_empty => Ok(Bounds {
            may_fault: true,
            ..Bounds::exactly(&Value::Truth(true))
        }),
        Err(fault) => Err(fault),
    }
}
