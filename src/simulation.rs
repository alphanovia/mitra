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

use crate::expression::{strict, Fault, Outcome, Reading, Standard};
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};

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
}

/// The simulation standard: its types and the type of each operation.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Simulation;

fn number(t: &Type) -> bool {
    matches!(t, Type::Int | Type::Real)
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
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Exact;

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
    /// A number from `low` to `high`, both in, or `None`: any number of
    /// its type. For an `Int`, both ends are whole.
    Number(Option<(Rational, Rational)>),
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
        (Value::Int(_) | Value::Real(_), Span::Number(None)) => true,
        (Value::Int(n), Span::Number(Some((low, high)))) => {
            let n = Rational::integer(n.clone());
            low <= &n && &n <= high
        }
        (Value::Real(q), Span::Number(Some((low, high)))) => low <= q && q <= high,
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
        (Ok(b), Ok(v)) => Exact.has_type(v, of) && lies_in(v, &b.span),
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
                Type::Int | Type::Real => Span::Number(None),
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
            span: Span::Number(Some((low, high))),
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
    Span::Number(Some((q.clone(), q)))
}

/// `span` is right for a value of type `of`.
fn fits(span: &Span, of: &Type) -> bool {
    match (span, of) {
        (Span::Truth { yes, no }, Type::Truth) => *yes || *no,
        (Span::Number(None), Type::Int | Type::Real) => true,
        (Span::Number(Some((low, high))), Type::Real) => low <= high,
        (Span::Number(Some((low, high))), Type::Int) => {
            low <= high && low.is_integer() && high.is_integer()
        }
        (Span::Record(r, spans), Type::Record(t)) => r == t && fits_fields(spans, t, fits),
        _ => false,
    }
}

/// The reading by bounds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ranges;

type Bounded = Outcome<Bounds, Undefined>;

/// The smallest span holding both. `None` for two of different kinds.
fn hull(a: &Span, b: &Span) -> Option<Span> {
    Some(match (a, b) {
        (Span::Truth { yes: a, no: b }, Span::Truth { yes: c, no: d }) => Span::Truth {
            yes: *a || *c,
            no: *b || *d,
        },
        (Span::Number(Some((al, ah))), Span::Number(Some((bl, bh)))) => {
            Span::Number(Some((al.min(bl).clone(), ah.max(bh).clone())))
        }
        (Span::Number(_), Span::Number(_)) => Span::Number(None),
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

type Ends = Option<(Rational, Rational)>;

/// `+`, `-`, `*` on two ranges: the least and the most of what the ends
/// give.
fn arith(op: &Op, a: &Ends, b: &Ends) -> Ends {
    let ((al, ah), (bl, bh)) = (a.as_ref()?, b.as_ref()?);
    Some(match op {
        Op::Add => (al.plus(bl), ah.plus(bh)),
        Op::Sub => (al.minus(bh), ah.minus(bl)),
        _ => {
            let all = [al.times(bl), al.times(bh), ah.times(bl), ah.times(bh)];
            let low = all.iter().min()?.clone();
            let high = all.iter().max()?.clone();
            (low, high)
        }
    })
}

/// `a / b` on two ranges: a fault when `b` is zero; any number, and a
/// possible fault, when `b` may be zero; else the least and the most of
/// what the ends give.
fn divide(a: &Ends, b: &Ends, may_fault: bool) -> Bounded {
    let zero = Rational::integer(0);
    let (bl, bh) = match b {
        Some((low, high)) if low.is_zero() && high.is_zero() => {
            return Err(Fault::Op(Undefined::DivisionByZero))
        }
        Some((low, high)) if &zero < low || high < &zero => (low, high),
        _ => {
            return Ok(Bounds {
                span: Span::Number(None),
                may_fault: true,
            })
        }
    };
    let ends = a.as_ref().and_then(|(al, ah)| {
        let all = [al.over(bl)?, al.over(bh)?, ah.over(bl)?, ah.over(bh)?];
        Some((all.iter().min()?.clone(), all.iter().max()?.clone()))
    });
    Ok(Bounds {
        span: Span::Number(ends),
        may_fault,
    })
}

/// `a < b`, or `a <= b` when `or_equal`: whether it may be true, and
/// may be false.
fn less(a: &Ends, b: &Ends, or_equal: bool) -> Span {
    let (yes, no) = match (a, b) {
        (Some((al, ah)), Some((bl, bh))) => match or_equal {
            true => (al <= bh, ah > bl),
            false => (al < bh, ah >= bl),
        },
        _ => (true, true),
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
        (Span::Number(Some((al, ah))), Span::Number(Some((bl, bh)))) => {
            (al <= bh && bl <= ah, !(al == ah && bl == bh && al == bl))
        }
        (Span::Number(_), Span::Number(_)) => (true, true),
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
    let zero = Rational::integer(0);
    let (yes, no) = match (a, b, tol) {
        (Some((al, ah)), Some((bl, bh)), Some((tl, th))) => {
            let (low, high) = (al.minus(bh), ah.minus(bl));
            let (near, far) = if low > zero {
                (low, high)
            } else if high < zero {
                (high.abs(), low.abs())
            } else {
                (zero, low.abs().max(high.abs()))
            };
            (&near <= th, &far > tl)
        }
        _ => (true, true),
    };
    Span::Truth { yes, no }
}

impl Reading for Ranges {
    type Standard = Simulation;
    type Value = Bounds;
    type Fault = Undefined;

    fn has_type(&self, value: &Bounds, of: &Type) -> bool {
        fits(&value.span, of)
    }

    fn apply(&self, op: &Op, args: &[Bounded]) -> Bounded {
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
        use Span::Number as N;
        let span = match (op, spans.as_slice()) {
            (Op::Truth(_) | Op::Int(_) | Op::Real(_), []) => {
                let exact = Exact.apply(op, &[])?;
                Bounds::exactly(&exact).span
            }
            (Op::Not, [Span::Truth { yes, no }]) => Span::Truth { yes: *no, no: *yes },
            (Op::Add | Op::Sub | Op::Mul, [N(a), N(b)]) => N(arith(op, a, b)),
            (Op::Div, [N(a), N(b)]) => return divide(a, b, may_fault),
            (Op::Less, [N(a), N(b)]) => less(a, b, false),
            (Op::AtMost, [N(a), N(b)]) => less(a, b, true),
            (Op::Equal, [a, b]) => match compare(a, b) {
                Some((yes, no)) => Span::Truth { yes, no },
                None => return Err(Fault::Stuck),
            },
            (Op::Within, [N(a), N(b), N(tol)]) => within(a, b, tol),
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
        (Span::Number(Some((ll, lh))), Span::Number(Some((hl, hh)))) if ll == lh && hl == hh => {
            all_points(ll, hl, run)
        }
        (Span::Number(low), Span::Number(high)) => all_once(low, high, run),
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
    let p = read_at(Span::Number(Some((x.clone(), x.clone()))), run);
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
    let reach = match (low, high) {
        (Some((ll, _)), Some((_, hh))) => {
            let top = hh.minus(&Rational::integer(1));
            if &top < ll {
                return said(true);
            }
            Some((ll.clone(), top))
        }
        _ => None,
    };
    let may_be_empty = match (low, high) {
        (Some((_, lh)), Some((hl, _))) => lh >= hl,
        _ => true,
    };
    match read_at(Span::Number(reach), run) {
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
