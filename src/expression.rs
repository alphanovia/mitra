//! Typed expressions with no built-in type (`docs/core2.md`).
//!
//! A *standard* is a language: its types and its operations, and the
//! type each operation gives. A *reading* is a meaning of a standard:
//! its values, which values have which type, and what each operation
//! gives on values. One expression has one type and may have several
//! readings, such as exact values and the bounds they lie in.
//!
//! This file knows no type, no value and no operation. A constant is an
//! operation that takes nothing.
//!
//! The typing judgement `Γ ⊢ e : T` is `check(standard, Γ, e) == Some(T)`:
//!
//! ```text
//!   Γ(x) = T                      Γ ⊢ eᵢ : Tᵢ (each i)    gives(op, [T₁ … Tₙ]) = T
//!   ─────────────────             ───────────────────────────────────────────────
//!   Γ ⊢ Input(x) : T              Γ ⊢ Apply(op, [e₁ … eₙ]) : T
//! ```
//!
//! What a reading must keep, for every operation (agreement): when
//! `gives(op, [T₁ … Tₙ]) = Some(T)` and each argument is a value of its
//! type `Tᵢ` or an operation's fault, `apply` gives a value of type `T`
//! or an operation's fault. `apply` never panics.
//!
//! What follows, by induction on `e` (preservation and progress): when
//! `check(s, Γ, e) == Some(T)` and `fits(r, Γ, values)`, then
//! `eval(r, values, e)` is `Ok(v)` with `r.has_type(&v, &T)`, or
//! `Err(Fault::Op(_))`. It is never `Err(Fault::Stuck)`.
//!
//! `check` and `eval` never panic, on any expression and any values.
//! They recurse once per level of the expression, so the depth of an
//! expression is bounded by the stack.

use std::collections::BTreeMap;

/// An expression: a named input, or an operation on what other
/// expressions give.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Expression<Op> {
    Input(String),
    Apply(Op, Vec<Expression<Op>>),
}

/// A language: its types and its operations.
pub trait Standard {
    type Type: Clone;
    type Op;
    /// The type `op` gives when it takes arguments of these types, in
    /// order. `None` when it does not take them.
    fn gives(&self, op: &Self::Op, takes: &[Self::Type]) -> Option<Self::Type>;
}

/// Why an expression has no value.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Fault<F> {
    /// An operation has no value here, by its own reading: a division
    /// by zero, for one.
    Op(F),
    /// Evaluation went wrong: an input has no value, or an operation got
    /// values it does not take. A checked expression on values that fit
    /// its inputs never goes wrong.
    Stuck,
}

/// What evaluating gives: a value, or why there is none.
pub type Outcome<V, F> = Result<V, Fault<F>>;

/// The types of a reading's standard.
pub type TypeOf<R> = <<R as Reading>::Standard as Standard>::Type;
/// The operations of a reading's standard.
pub type OpOf<R> = <<R as Reading>::Standard as Standard>::Op;
/// What a reading gives.
pub type OutcomeOf<R> = Outcome<<R as Reading>::Value, <R as Reading>::Fault>;

/// A meaning of a standard: its values and what operations give on
/// them.
pub trait Reading {
    type Standard: Standard;
    type Value: Clone;
    type Fault: Clone;
    /// `value` is a value of type `of`. Read from what the value is,
    /// not from a name it carries.
    fn has_type(&self, value: &Self::Value, of: &TypeOf<Self>) -> bool;
    /// What `op` gives on these arguments. Each argument is what its
    /// expression gave, fault included: an operation that does not need
    /// an argument, as `if` does not need the branch it does not take,
    /// may give a value anyway. Every other operation is `strict`.
    fn apply(&self, op: &OpOf<Self>, args: &[OutcomeOf<Self>]) -> OutcomeOf<Self>;
}

/// The type of `e` when its inputs have the types of `inputs`. `None`
/// when it has none: an input not in `inputs`, or an operation that does
/// not take the types it is given.
pub fn check<S: Standard>(
    standard: &S,
    inputs: &BTreeMap<String, S::Type>,
    e: &Expression<S::Op>,
) -> Option<S::Type> {
    match e {
        Expression::Input(name) => inputs.get(name).cloned(),
        Expression::Apply(op, args) => {
            let takes = args
                .iter()
                .map(|arg| check(standard, inputs, arg))
                .collect::<Option<Vec<_>>>()?;
            standard.gives(op, &takes)
        }
    }
}

/// What `e` gives at these values of its inputs. Every argument is
/// evaluated, left to right, and the operation is given all of them.
pub fn eval<R: Reading>(
    reading: &R,
    values: &BTreeMap<String, R::Value>,
    e: &Expression<OpOf<R>>,
) -> OutcomeOf<R> {
    match e {
        Expression::Input(name) => values.get(name).cloned().ok_or(Fault::Stuck),
        Expression::Apply(op, args) => {
            let args: Vec<_> = args.iter().map(|arg| eval(reading, values, arg)).collect();
            reading.apply(op, &args)
        }
    }
}

/// Every input of `inputs` has a value of its type in `values`.
pub fn fits<R: Reading>(
    reading: &R,
    inputs: &BTreeMap<String, TypeOf<R>>,
    values: &BTreeMap<String, R::Value>,
) -> bool {
    inputs.iter().all(|(name, of)| {
        values
            .get(name)
            .is_some_and(|value| reading.has_type(value, of))
    })
}

/// The values of all the arguments, or the first fault among them, left
/// to right: what an operation that needs all its arguments reads.
pub fn strict<V: Clone, F: Clone>(args: &[Outcome<V, F>]) -> Result<Vec<V>, Fault<F>> {
    args.iter().cloned().collect()
}
