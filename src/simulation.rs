//! The simulation standard (`docs/core2.md`): truths, whole numbers,
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
    pub fields: Vec<(String, Type)>,
}

/// The type of a tuple `(a, b, …)`: a record with no name, its fields
/// named `0`, `1`, … in order, so that `t.0` reads the first. No
/// declared record has no name.
pub fn tuple(of: Vec<Type>) -> Type {
    let fields = of.into_iter().enumerate();
    let fields = fields.map(|(i, t)| (i.to_string(), t)).collect();
    Type::Record(Record {
        name: String::new(),
        fields,
    })
}

impl Record {
    /// The place and type of the first field named `name`.
    pub fn field(&self, name: &str) -> Option<(usize, &Type)> {
        let place = self.fields.iter().position(|(n, _)| n == name)?;
        Some((place, &self.fields[place].1))
    }
}

/// What a value of the simulation can be. A record is not a name: a
/// record named "Real" is not `Real`.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Type {
    Truth,
    Int,
    Real,
    Record(Record),
    /// A type a contract names but does not fix, with its bound: `T`, or
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
                fields: r
                    .fields
                    .iter()
                    .map(|(n, t)| (n.clone(), t.given(given)))
                    .collect(),
            }),
            _ => self.clone(),
        }
    }
    /// It names a type parameter, in a record too.
    pub fn open(&self) -> bool {
        match self {
            Type::Param { .. } => true,
            Type::Record(r) => r.fields.iter().any(|(_, t)| t.open()),
            _ => false,
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
}

/// Why an operation has no value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Undefined {
    DivisionByZero,
    /// A call of this function has no value here: it has no body, and no
    /// row gives one at these arguments.
    Unknown(String),
}

/// The operations, in Rust's form.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Op {
    /// Constants: `true`, `3`, `1/3`.
    Truth(bool),
    Int(BigInt),
    Real(Rational),
    /// `!a`, `a && b`, `a || b`: `&&` and `||` drop what `b` gives,
    /// fault included, when `a` decides. `b` is computed all the same.
    Not,
    And,
    Or,
    /// `a + b`, `a - b`, `a * b` on two `Int` or two `Real`.
    Add,
    Sub,
    Mul,
    /// `a / b` on two `Real`. No value when `b` is zero.
    Div,
    /// `a < b`, `a <= b` on two `Int` or two `Real`.
    Less,
    AtMost,
    /// `a == b` on two values of one type.
    Equal,
    /// `within(a, b, tol)`: `|a - b| <= tol`, on three `Int` or three
    /// `Real`. Rust would write `(a - b).abs() <= tol`.
    Within,
    /// `if c { a } else { b }`: gives what the branch `c` takes gives,
    /// and drops the other, fault included. Both are computed.
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
}

/// A function a call can name: what it takes, each by name, what it
/// gives, and its body; or, with no body, rows of what it gives at
/// points (its examples, observations). With neither it is opaque.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Callee {
    pub takes: Vec<(String, Type)>,
    pub gives: Type,
    pub body: Option<Expression<Op>>,
    pub rows: Vec<(Vec<Value>, Value)>,
}

/// The functions calls can name, each by its path. Only `with` makes
/// one with functions, and it checks them.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Calls {
    callees: BTreeMap<String, Callee>,
    /// What a reading is given, by path, at points: a value of the world
    /// a measurement or an example of the model saw, of a function with
    /// no body. Read before its rows, and never compared with them here.
    given: BTreeMap<String, Vec<(Vec<Value>, Value)>>,
}

impl Calls {
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
            if size[component[i]] > 1 || out[i].contains(&i) {
                refused.insert((*name).clone());
            }
        }
        let standard = Simulation {
            calls: Arc::new(Calls {
                callees: callees.clone(),
                given: BTreeMap::new(),
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
                given: BTreeMap::new(),
            }),
            false => Err(refused),
        }
    }
    /// These functions, with `gives` given for `path` at `at`: read
    /// first, before its rows. Only a function with no body takes a
    /// value so, of its types (none of them a parameter): else `Err`,
    /// with the functions as they were. A body is what the function
    /// gives, and `Ranges` reads it, not what was given.
    pub fn given(mut self, path: &str, at: Vec<Value>, gives: Value) -> Result<Calls, Calls> {
        let exact = Exact::default();
        let fits = self.callees.get(path).is_some_and(|c| {
            c.body.is_none()
                && !c.gives.open()
                && c.takes.iter().all(|(_, t)| !t.open())
                && at.len() == c.takes.len()
                && at
                    .iter()
                    .zip(&c.takes)
                    .all(|(v, (_, t))| exact.has_type(v, t))
                && exact.has_type(&gives, &c.gives)
        });
        if !fits {
            return Err(self);
        }
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
}

impl Exact {
    /// A call of `f` at these values: its body run on them; or else what
    /// the reading was given there, else its row at them.
    fn call(&self, f: &str, args: &[Value]) -> Exactly {
        let callee = self.calls.get(f).ok_or(Fault::Stuck)?;
        if args.len() != callee.takes.len() {
            return Err(Fault::Stuck);
        }
        if let Some(body) = &callee.body {
            let names = callee.takes.iter().map(|(n, _)| n.clone());
            return eval(self, &names.zip(args.iter().cloned()).collect(), body);
        }
        let given = self.calls.given.get(f).into_iter().flatten();
        if let Some((_, gives)) = given.into_iter().find(|(at, _)| at.as_slice() == args) {
            return Ok(gives.clone());
        }
        match callee.rows.iter().find(|(at, _)| at.as_slice() == args) {
            Some((_, gives)) => Ok(gives.clone()),
            None => Err(Fault::Op(Undefined::Unknown(f.to_string()))),
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
            _ => Err(Fault::Stuck),
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
            (Op::Less, [Int(a), Int(b)]) => Truth(a < b),
            (Op::Less, [Real(a), Real(b)]) => Truth(a < b),
            (Op::AtMost, [Int(a), Int(b)]) => Truth(a <= b),
            (Op::AtMost, [Real(a), Real(b)]) => Truth(a <= b),
            (Op::Equal, [a, b]) => Truth(a == b),
            (Op::Within, [Int(a), Int(b), Int(tol)]) => Truth((a - b).abs() <= *tol),
            (Op::Within, [Real(a), Real(b), Real(tol)]) => Truth(a.minus(b).abs() <= *tol),
            (Op::Make(r), fields) if fits_fields(fields, r, |v, f| self.has_type(v, f)) => {
                Value::Record(r.clone(), fields.to_vec())
            }
            (Op::Field(name), [Value::Record(r, values)]) => {
                match r.field(name).and_then(|(place, _)| values.get(place)) {
                    Some(v) => v.clone(),
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

/// Where a value lies.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Span {
    /// The truths it may be: at least one.
    Truth { yes: bool, no: bool },
    /// A number from the low end to the high end, both in. An end that
    /// is `None` is no end: the number may be as low, or as high, as its
    /// type goes. For an `Int`, each end is whole.
    Number(Option<Rational>, Option<Rational>),
    /// A record, each field in its span.
    Record(Record, Vec<Span>),
}

/// γ: `value` lies in `span`.
pub fn lies_in(value: &Value, span: &Span) -> bool {
    match (value, span) {
        (Value::Truth(b), Span::Truth { yes, no }) => match b {
            true => *yes,
            false => *no,
        },
        (Value::Int(n), Span::Number(low, high)) => {
            inside(&Rational::integer(n.clone()), low, high)
        }
        (Value::Real(q), Span::Number(low, high)) => inside(q, low, high),
        (Value::Record(r, values), Span::Record(s, spans)) => {
            r == s
                && values.len() == spans.len()
                && values.iter().zip(spans).all(|(v, s)| lies_in(v, s))
        }
        _ => false,
    }
}

/// γ on outcomes of type `of`: what `exact` gave is one of those
/// `bounds` stand for. A value must be of the type and lie in the span;
/// a fault is allowed where the bounds may fault, or are a fault.
pub fn covers(of: &Type, bounds: &Outcome<Bounds, Undefined>, exact: &Exactly) -> bool {
    match (bounds, exact) {
        (Err(Fault::Op(_)), Err(Fault::Op(_))) => true,
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
                Value::Int(n) => point(Rational::integer(n.clone())),
                Value::Real(q) => point(q.clone()),
                Value::Record(r, values) => {
                    Span::Record(r.clone(), values.iter().map(span).collect())
                }
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
                Type::Int | Type::Real | Type::Param { .. } => Span::Number(None, None),
                Type::Record(r) => {
                    Span::Record(r.clone(), r.fields.iter().map(|(_, f)| span(f)).collect())
                }
            }
        }
        Bounds {
            span: span(of),
            may_fault: false,
        }
    }
    /// A number from `low` to `high`. `None` when `low` is above `high`.
    pub fn between(low: Rational, high: Rational) -> Option<Bounds> {
        (low <= high).then_some(Bounds {
            span: Span::Number(Some(low), Some(high)),
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

fn point(q: Rational) -> Span {
    Span::Number(Some(q.clone()), Some(q))
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
        (Span::Number(low, high), Type::Int | Type::Real) => {
            let whole = |e: &Option<Rational>| e.as_ref().is_none_or(|q| q.is_integer());
            End::low(low) <= End::high(high) && (of == &Type::Real || (whole(low) && whole(high)))
        }
        (Span::Record(r, spans), Type::Record(t)) => r == t && fits_fields(spans, t, fits),
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
        let gives = match &callee.body {
            Some(body) => {
                let names = callee.takes.iter().map(|(n, _)| n.clone());
                eval(self, &names.zip(args.iter().cloned()).collect(), body)
            }
            None if callee.gives.open() => Err(Fault::Op(Undefined::Unknown(f.to_string()))),
            None => Ok(Bounds {
                may_fault: true,
                ..Bounds::any(&callee.gives)
            }),
        };
        let may_fault = args.iter().any(|b| b.may_fault);
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
        (Span::Number(al, ah), Span::Number(bl, bh)) => Span::Number(
            End::low(al).min(End::low(bl)).finite(),
            End::high(ah).max(End::high(bh)).finite(),
        ),
        (Span::Record(r, xs), Span::Record(s, ys)) if r == s && xs.len() == ys.len() => {
            let fields = xs.iter().zip(ys).map(|(x, y)| hull(x, y));
            Span::Record(r.clone(), fields.collect::<Option<_>>()?)
        }
        _ => return None,
    })
}

/// What either may give.
fn join(a: Bounded, b: Bounded) -> Bounded {
    match (a, b) {
        (Err(Fault::Stuck), _) | (_, Err(Fault::Stuck)) => Err(Fault::Stuck),
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
fn arith(op: &Op, a: &Ends, b: &Ends) -> Span {
    let ((al, ah), (bl, bh)) = (ends(&a.0, &a.1), ends(&b.0, &b.1));
    let at = |x: &End, y: &End, f: fn(&Rational, &Rational) -> Rational| match (x, y) {
        (End::At(x), End::At(y)) => Some(f(x, y)),
        _ => None,
    };
    match op {
        Op::Add => Span::Number(at(&al, &bl, Rational::plus), at(&ah, &bh, Rational::plus)),
        Op::Sub => Span::Number(at(&al, &bh, Rational::minus), at(&ah, &bl, Rational::minus)),
        _ => {
            let all = [al.times(&bl), al.times(&bh), ah.times(&bl), ah.times(&bh)];
            let low = all.iter().min().cloned().and_then(End::finite);
            let high = all.iter().max().cloned().and_then(End::finite);
            Span::Number(low, high)
        }
    }
}

/// `a / b` on two ranges: a fault when `b` is zero; any number, and a
/// possible fault, when `b` may be zero; else the least and the most of
/// what the ends give: a finite number over no end is near zero, and no
/// end over no end any number.
fn divide(a: &Ends, b: &Ends, may_fault: bool) -> Bounded {
    let zero = End::At(Rational::integer(0));
    let ((al, ah), (bl, bh)) = (ends(&a.0, &a.1), ends(&b.0, &b.1));
    if bl == zero && bh == zero {
        return Err(Fault::Op(Undefined::DivisionByZero));
    }
    if !(zero < bl || bh < zero) {
        return Ok(Bounds {
            span: Span::Number(None, None),
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
    let span = match all.into_iter().collect::<Option<Vec<End>>>() {
        Some(all) => Span::Number(
            all.iter().min().cloned().and_then(End::finite),
            all.iter().max().cloned().and_then(End::finite),
        ),
        None => Span::Number(None, None),
    };
    Ok(Bounds { span, may_fault })
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
        (Span::Number(al, ah), Span::Number(bl, bh)) => {
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
        _ => return None,
    })
}

/// `|a - b| <= tol`: whether it may be true, and may be false.
fn within(a: &Ends, b: &Ends, tol: &Ends) -> Span {
    let zero = End::At(Rational::integer(0));
    let Span::Number(low, high) = arith(&Op::Sub, a, b) else {
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
            _ => Err(Fault::Stuck),
        }
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
            (Op::Or, [a, b]) => return choose(a, said(true), b.clone()),
            _ => {}
        }
        let args = strict(args)?;
        let may_fault = args.iter().any(|b| b.may_fault);
        let spans: Vec<&Span> = args.iter().map(|b| &b.span).collect();
        let n = |s: &Span| match s {
            Span::Number(low, high) => Some((low.clone(), high.clone())),
            _ => None,
        };
        let numbers: Vec<Ends> = spans.iter().filter_map(|s| n(s)).collect();
        let numbers = if numbers.len() == spans.len() {
            numbers
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
                arith(op, &numbers[0], &numbers[1])
            }
            (Op::Div, [_, _]) if numbers.len() == 2 => {
                return divide(&numbers[0], &numbers[1], may_fault)
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
            (Op::Field(name), [Span::Record(r, spans)]) => {
                match r.field(name).and_then(|(place, _)| spans.get(place)) {
                    Some(s) => s.clone(),
                    None => return Err(Fault::Stuck),
                }
            }
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
        (Span::Number(Some(ll), Some(lh)), Span::Number(Some(hl), Some(hh)))
            if ll == lh && hl == hh =>
        {
            all_points(ll, hl, run)
        }
        (Span::Number(ll, lh), Span::Number(hl, hh)) => {
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
    let p = read_at(point(x.clone()), run);
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
    match read_at(Span::Number(low.0.clone(), top), run) {
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
