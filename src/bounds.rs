//! A second reading of the standard operations (`model/bounds.mitra`):
//! each quantity is the bounds a value lies in, with a flag for an
//! operation that may have no value somewhere in them.
//!
//! After Cousot and Cousot, 1977, "Abstract Interpretation: A Unified
//! Lattice Model for Static Analysis of Programs by Construction or
//! Approximation of Fixpoints": the same expression computed on a
//! simpler kind of quantity that covers every exact one. Here the
//! simpler kind is an interval, and there is no loop, so no fixpoint.

use crate::standard::{is, real, truth, Exact};
use crate::value::{Applies, Expression, Operation, Type, Value};

/// What a value may be: nothing at all, some of the two truths, or a
/// number from `low` to `high`, both included. A number is a ratio of
/// two whole numbers, the lower one above 0.
// mitra: implements bounds::Span
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Span {
    Empty,
    Truths { yes: bool, no: bool },
    Numbers { low: (i64, i64), high: (i64, i64) },
}

/// The bounds a value lies in. `may_fault`: somewhere in the bounds of
/// what it was computed from, an operation has no value.
// mitra: implements bounds::Bounds
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Bounds {
    pub within: Span,
    pub may_fault: bool,
}

type Wide = (i128, i128);

fn wide((n, d): (i64, i64)) -> Wide {
    (n as i128, d as i128)
}
/// A ratio in lowest terms, or none when a part does not fit 64 bits.
fn fit((n, d): Wide) -> Option<(i64, i64)> {
    if d == 0 {
        return None;
    }
    let (mut a, mut b) = (n.unsigned_abs(), d.unsigned_abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    let by = a.max(1) as i128 * d.signum();
    Some((i64::try_from(n / by).ok()?, i64::try_from(d / by).ok()?))
}
fn less(a: Wide, b: Wide) -> bool {
    a.0 * b.1 < b.0 * a.1
}
fn add(a: Wide, b: Wide) -> Wide {
    (a.0 * b.1 + b.0 * a.1, a.1 * b.1)
}
fn sub(a: Wide, b: Wide) -> Wide {
    (a.0 * b.1 - b.0 * a.1, a.1 * b.1)
}
fn mul(a: Wide, b: Wide) -> Wide {
    (a.0 * b.0, a.1 * b.1)
}
fn least(all: &[Wide]) -> Wide {
    let mut out = all[0];
    for x in all {
        if less(*x, out) {
            out = *x;
        }
    }
    out
}
fn most(all: &[Wide]) -> Wide {
    let mut out = all[0];
    for x in all {
        if less(out, *x) {
            out = *x;
        }
    }
    out
}

/// A number of this type known to lie from `low` to `high`. None when
/// the bounds are not in order, or not whole for a whole type.
// mitra: implements bounds::between
pub fn between(of: &Type, low: (i64, i64), high: (i64, i64)) -> Option<Value<Bounds>> {
    if low.1 <= 0 || high.1 <= 0 || less(wide(high), wide(low)) {
        return None;
    }
    let (low, high) = (fit(wide(low))?, fit(wide(high))?);
    if of != &real() && (low.1 != 1 || high.1 != 1) {
        return None;
    }
    Some(Value {
        quantity: Bounds {
            within: Span::Numbers { low, high },
            may_fault: false,
        },
        of: of.clone(),
    })
}

/// An exact value as bounds: the one point it is. None for what has
/// no bounds here, a record.
// mitra: implements bounds::point
pub fn point(v: &crate::value::Value<Exact>) -> Option<Value<Bounds>> {
    let within = match &v.quantity {
        Exact::Truth(x) => Span::Truths { yes: *x, no: !*x },
        Exact::Whole(n) => Span::Numbers {
            low: (*n, 1),
            high: (*n, 1),
        },
        Exact::Many(both) if v.of == real() => match both.as_slice() {
            [Exact::Whole(n), Exact::Whole(d)] => Span::Numbers {
                low: (*n, *d),
                high: (*n, *d),
            },
            _ => return None,
        },
        Exact::Many(_) => return None,
    };
    Some(Value {
        quantity: Bounds {
            within,
            may_fault: false,
        },
        of: v.of.clone(),
    })
}

/// The same expression, to be read by bounds: each value given in it
/// becomes the point it is. None when one of them has no bounds.
// mitra: implements bounds::lifted
pub fn lifted(e: &Expression<Exact>) -> Option<Expression<Bounds>> {
    Some(match e {
        Expression::Given(v) => Expression::Given(point(v)?),
        Expression::Input(name, of) => Expression::Input(name.clone(), of.clone()),
        Expression::Apply(op, args) => {
            let mut lifted_args = Vec::with_capacity(args.len());
            for arg in args {
                lifted_args.push(lifted(arg)?);
            }
            Expression::Apply(op.clone(), lifted_args)
        }
    })
}

/// What a truth read by bounds surely is: true everywhere in the
/// bounds, false everywhere, or none when it may be either, may have
/// no value, or is not a truth.
// mitra: implements bounds::surely
pub fn surely(v: &Value<Bounds>) -> Option<bool> {
    match (&v.quantity.within, v.quantity.may_fault, v.of == truth()) {
        (
            Span::Truths {
                yes: true,
                no: false,
            },
            false,
            true,
        ) => Some(true),
        (
            Span::Truths {
                yes: false,
                no: true,
            },
            false,
            true,
        ) => Some(false),
        _ => None,
    }
}

/// What applies the standard operations on bounds. It lacks some: the
/// reading is then unknown, which `apply` says with none.
// mitra: implements bounds::Ranges
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Ranges;

fn numbers(v: &Value<Bounds>) -> Option<(Wide, Wide)> {
    match v.quantity.within {
        Span::Numbers { low, high } => Some((wide(low), wide(high))),
        _ => None,
    }
}
fn truths(v: &Value<Bounds>) -> Option<(bool, bool)> {
    match v.quantity.within {
        Span::Truths { yes, no } => Some((yes, no)),
        _ => None,
    }
}
/// The smaller span that holds both. None for two of different kinds.
fn joined(a: &Span, b: &Span) -> Option<Span> {
    Some(match (a, b) {
        (Span::Empty, other) | (other, Span::Empty) => other.clone(),
        (Span::Truths { yes: a, no: b }, Span::Truths { yes: c, no: d }) => Span::Truths {
            yes: *a || *c,
            no: *b || *d,
        },
        (Span::Numbers { low: a, high: b }, Span::Numbers { low: c, high: d }) => Span::Numbers {
            low: fit(least(&[wide(*a), wide(*c)]))?,
            high: fit(most(&[wide(*b), wide(*d)]))?,
        },
        _ => return None,
    })
}

impl Applies<Bounds> for Ranges {
    // mitra: implements bounds::Ranges::apply
    fn apply(&self, op: &Operation, args: &[Option<Value<Bounds>>]) -> Option<Bounds> {
        const T: &str = "Truth";
        const I: &str = "Int";
        const R: &str = "Real";
        const I8: &str = "Int8";
        // `if` reads the branches its test may take, and no other.
        if let ("if", [Some(test), yes, no]) = (op.name.as_str(), args) {
            let (may_yes, may_no) = match test.quantity.within {
                Span::Empty => (false, false),
                _ => truths(test)?,
            };
            let mut out = Bounds {
                within: Span::Empty,
                may_fault: test.quantity.may_fault || !(may_yes || may_no),
            };
            for (taken, branch) in [(may_yes, yes), (may_no, no)] {
                if taken {
                    let branch = branch.as_ref()?;
                    if !is(op, &[T, &branch.of.name, &branch.of.name], &branch.of.name) {
                        return None;
                    }
                    out.within = joined(&out.within, &branch.quantity.within)?;
                    out.may_fault |= branch.quantity.may_fault;
                }
            }
            return Some(out);
        }
        let present: Option<Vec<&Value<Bounds>>> = args.iter().map(Option::as_ref).collect();
        let args = present?;
        let may_fault = args.iter().any(|v| v.quantity.may_fault);
        // Nothing in, nothing out: what it was computed from never has
        // a value.
        if args.iter().any(|v| v.quantity.within == Span::Empty) {
            return Some(Bounds {
                within: Span::Empty,
                may_fault: true,
            });
        }
        let said = |yes: bool, no: bool| {
            Some(Bounds {
                within: Span::Truths { yes, no },
                may_fault,
            })
        };
        let from = |low: Wide, high: Wide, fault: bool| {
            Some(Bounds {
                within: Span::Numbers {
                    low: fit(low)?,
                    high: fit(high)?,
                },
                may_fault: may_fault || fault,
            })
        };
        match (op.name.as_str(), args.as_slice()) {
            ("!", [x]) if is(op, &[T], T) => {
                let (yes, no) = truths(x)?;
                said(no, yes)
            }
            ("&&", [x, y]) if is(op, &[T, T], T) => {
                let ((a, b), (c, d)) = (truths(x)?, truths(y)?);
                said(a && c, b || d)
            }
            ("||", [x, y]) if is(op, &[T, T], T) => {
                let ((a, b), (c, d)) = (truths(x)?, truths(y)?);
                said(a || c, b && d)
            }
            ("<" | "<=" | ">" | ">=", [x, y]) if is(op, &[I, I], T) || is(op, &[R, R], T) => {
                let ((xl, xh), (yl, yh)) = (numbers(x)?, numbers(y)?);
                match op.name.as_str() {
                    "<" => said(less(xl, yh), !less(xh, yl)),
                    "<=" => said(!less(yh, xl), less(yl, xh)),
                    ">" => said(less(yl, xh), !less(yh, xl)),
                    _ => said(!less(xh, yl), less(xl, yh)),
                }
            }
            ("+" | "-" | "*", [x, y]) if is(op, &[I, I], I) || is(op, &[R, R], R) => {
                let ((xl, xh), (yl, yh)) = (numbers(x)?, numbers(y)?);
                match op.name.as_str() {
                    "+" => from(add(xl, yl), add(xh, yh), false),
                    "-" => from(sub(xl, yh), sub(xh, yl), false),
                    _ => {
                        let all = [mul(xl, yl), mul(xl, yh), mul(xh, yl), mul(xh, yh)];
                        from(least(&all), most(&all), false)
                    }
                }
            }
            ("within", [x, y, by]) if is(op, &[I, I, I], T) || is(op, &[R, R, R], T) => {
                let ((xl, xh), (yl, yh), (tl, th)) = (numbers(x)?, numbers(y)?, numbers(by)?);
                // The distance lies between `near` and `far`.
                let (low, high) = (sub(xl, yh), sub(xh, yl));
                let size = |d: Wide| (d.0.abs(), d.1);
                let zero = (0, 1);
                let far = most(&[size(low), size(high)]);
                let near = match less(zero, low) || less(high, zero) {
                    true => least(&[size(low), size(high)]),
                    false => zero,
                };
                said(!less(th, near), less(tl, far))
            }
            ("widening_add" | "wrapping_add" | "checked_add", [x, y])
                if is(op, &[I8, I8], "Int16") || is(op, &[I8, I8], I8) =>
            {
                let ((xl, xh), (yl, yh)) = (numbers(x)?, numbers(y)?);
                let (low, high) = (add(xl, yl), add(xh, yh));
                let (least8, most8) = ((-128, 1), (127, 1));
                let outside = less(low, least8) || less(most8, high);
                match (op.name.as_str(), op.gives.name.as_str()) {
                    ("widening_add", "Int16") => from(low, high, false),
                    // Wrapped, it may be anywhere in the type.
                    ("wrapping_add", "Int8") if outside => from(least8, most8, false),
                    ("wrapping_add", "Int8") => from(low, high, false),
                    ("checked_add", "Int8") if less(high, least8) || less(most8, low) => {
                        Some(Bounds {
                            within: Span::Empty,
                            may_fault: true,
                        })
                    }
                    ("checked_add", "Int8") => {
                        from(most(&[low, least8]), least(&[high, most8]), outside)
                    }
                    _ => None,
                }
            }
            _ => None,
        }
    }
}

/// The whole type as bounds: every whole number of 8 bits, signed.
// mitra: implements bounds::any_int8
pub fn any_int8() -> Value<Bounds> {
    Value {
        quantity: Bounds {
            within: Span::Numbers {
                low: (-128, 1),
                high: (127, 1),
            },
            may_fault: false,
        },
        of: crate::standard::int8(),
    }
}
