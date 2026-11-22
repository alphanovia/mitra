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
//! The typing judgement `Γ ⊢ e : T` is `check(standard, Γ, e) == Some(T)`,
//! where `Γ, x : T₁` is `Γ` with `x` now of type `T₁`:
//!
//! ```text
//!   Γ(x) = T                      Γ ⊢ eᵢ : Tᵢ (each i)    gives(op, [T₁ … Tₙ]) = T
//!   ─────────────────             ───────────────────────────────────────────────
//!   Γ ⊢ Name(x) : T               Γ ⊢ Apply(op, [e₁ … eₙ]) : T
//!
//!   Γ ⊢ e₁ : T₁    Γ, x : T₁ ⊢ e₂ : T₂
//!   ──────────────────────────────────
//!   Γ ⊢ Let(x, e₁, e₂) : T₂
//! ```
//!
//! `Let(x, e₁, e₂)` is written as Rust's `let x = e₁; e₂`. It evaluates
//! `e₁` once and reads `e₂` with `x` standing for its outcome, fault
//! included. Unlike Rust, a fault of `e₁` is not raised at the `let`: it
//! counts only where `x` is read, as if `e₁` were written there. So a
//! `Let` never changes what an expression gives; it shares (sharing):
//! `eval` of `Let(x, e₁, e₂)` is `eval` of `e₂` with `e₁` in place of
//! each `x` it binds.
//!
//! What a reading must keep, for every operation (agreement): when
//! `gives(op, [T₁ … Tₙ]) = Some(T)` and each argument is a value of its
//! type `Tᵢ` or an operation's fault, `apply` gives a value of type `T`
//! or an operation's fault. `apply` never panics.
//!
//! What follows, by induction on `e` (preservation and progress): when
//! `check(s, Γ, e) == Some(T)` and `fits(r, Γ, values)`, then
//! `eval(r, values, e)` is `Ok(v)` with `r.has_type(&v, &T)`, or
//! `Err(Fault::Op(_))`. It is never `Err(Fault::Stuck)`. The induction
//! keeps every name in scope standing for a value of its type or an
//! operation's fault.
//!
//! When one reading bounds another (covering): a reading `A` bounds a
//! reading `C` of the same standard by γ, where `γ_T(a)` is the set of
//! outcomes of `C`, faults included, that an outcome `a` of `A` of type
//! `T` stands for. For every operation with `gives(op, [T₁ … Tₙ]) =
//! Some(T)`, and arguments where each `aᵢ` is a value of type `Tᵢ` or an
//! operation's fault and each `cᵢ` is in `γ_Tᵢ(aᵢ)`, `C.apply(op, c)` is
//! in `γ_T(A.apply(op, a))`. Then by induction, as for preservation:
//! when `check(s, Γ, e) == Some(T)`, the inputs' values in `A` fit `Γ`
//! and each input's value in `C` is in `γ` of its value in `A` at its
//! type, `eval(C, …, e)` is in `γ_T(eval(A, …, e))`.
//!
//! `check` and `eval` never panic, on any expression and any values.
//! They recurse once per level of the expression, except along a chain
//! of `Let`s, each in the body of the one before, which takes no stack.
//! The derived `Clone`, `PartialEq` and drop recurse on every level. So
//! depth is bounded by the stack: on a thread of 2 MiB, 1,500 levels in
//! a debug build and 3,000 in a release build hold for all of them, for
//! the standards here. Time and memory are not bounded by the size of
//! an expression: each `Let` may square a number.

use std::collections::BTreeMap;

/// An expression.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression<Op> {
    /// A name: what the nearest enclosing `Let` of it binds, or else an
    /// input.
    Name(String),
    /// An operation on what other expressions give.
    Apply(Op, Vec<Expression<Op>>),
    /// `let x = e₁; e₂`: `e₂`, where `x` stands for what `e₁` gives.
    Let(String, Box<Expression<Op>>, Box<Expression<Op>>),
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
#[derive(Clone, Debug, PartialEq, Eq)]
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
/// them. `has_type` and `apply` are functions: the same arguments give
/// the same answer, so a reading keeps no state that changes one.
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

/// What the names bound by the enclosing `Let`s stand for: for each
/// name, its bindings, the nearest last.
type Bound<'e, X> = BTreeMap<&'e str, Vec<X>>;

/// What the nearest `Let` of `name` binds it to.
fn nearest<'b, X>(bound: &'b Bound<'_, X>, name: &str) -> Option<&'b X> {
    bound.get(name).and_then(|xs| xs.last())
}

/// Takes back one binding of each of `names`.
fn unbind<X>(bound: &mut Bound<'_, X>, names: &[&str]) {
    for name in names {
        if let Some(xs) = bound.get_mut(*name) {
            xs.pop();
        }
    }
}

/// The type of `e` when its inputs have the types of `inputs`. `None`
/// when it has none: a name neither bound nor in `inputs`, or an
/// operation that does not take the types it is given.
pub fn check<S: Standard>(
    standard: &S,
    inputs: &BTreeMap<String, S::Type>,
    e: &Expression<S::Op>,
) -> Option<S::Type> {
    check_in(standard, inputs, &mut BTreeMap::new(), e)
}

fn check_in<'e, S: Standard>(
    standard: &S,
    inputs: &BTreeMap<String, S::Type>,
    bound: &mut Bound<'e, S::Type>,
    e: &'e Expression<S::Op>,
) -> Option<S::Type> {
    match e {
        Expression::Name(name) => nearest(bound, name).or_else(|| inputs.get(name)).cloned(),
        Expression::Apply(op, args) => {
            let mut takes = Vec::with_capacity(args.len());
            for arg in args {
                takes.push(check_in(standard, inputs, bound, arg)?);
            }
            standard.gives(op, &takes)
        }
        Expression::Let(..) => check_lets(standard, inputs, bound, e),
    }
}

/// `check_in` of a `Let`, and of each `Let` in the body of the one
/// before, in a loop: a chain of them takes no stack.
#[inline(never)]
fn check_lets<'e, S: Standard>(
    standard: &S,
    inputs: &BTreeMap<String, S::Type>,
    bound: &mut Bound<'e, S::Type>,
    mut e: &'e Expression<S::Op>,
) -> Option<S::Type> {
    let mut names = Vec::new();
    let gives = 'scope: {
        while let Expression::Let(name, value, body) = e {
            let Some(t) = check_in(standard, inputs, bound, value) else {
                break 'scope None;
            };
            bound.entry(name.as_str()).or_default().push(t);
            names.push(name.as_str());
            e = body;
        }
        check_in(standard, inputs, bound, e)
    };
    unbind(bound, &names);
    gives
}

/// What `e` gives at these values of its inputs. Every argument is
/// evaluated, left to right, and the operation is given all of them.
/// The value of a `Let` is evaluated once, before its body.
pub fn eval<R: Reading>(
    reading: &R,
    values: &BTreeMap<String, R::Value>,
    e: &Expression<OpOf<R>>,
) -> OutcomeOf<R> {
    eval_in(reading, values, &mut BTreeMap::new(), e)
}

fn eval_in<'e, R: Reading>(
    reading: &R,
    values: &BTreeMap<String, R::Value>,
    bound: &mut Bound<'e, OutcomeOf<R>>,
    e: &'e Expression<OpOf<R>>,
) -> OutcomeOf<R> {
    match e {
        Expression::Name(name) => read::<R>(values, bound, name),
        Expression::Apply(op, args) => {
            let mut outcomes = Vec::with_capacity(args.len());
            for arg in args {
                outcomes.push(eval_in(reading, values, bound, arg));
            }
            reading.apply(op, &outcomes)
        }
        Expression::Let(..) => eval_lets(reading, values, bound, e),
    }
}

/// What a name stands for: the nearest `Let` of it, or else the input.
#[inline(never)]
fn read<R: Reading>(
    values: &BTreeMap<String, R::Value>,
    bound: &Bound<'_, OutcomeOf<R>>,
    name: &str,
) -> OutcomeOf<R> {
    match nearest(bound, name) {
        Some(outcome) => outcome.clone(),
        None => values.get(name).cloned().ok_or(Fault::Stuck),
    }
}

/// `eval_in` of a `Let`, and of each `Let` in the body of the one
/// before, in a loop: a chain of them takes no stack.
#[inline(never)]
fn eval_lets<'e, R: Reading>(
    reading: &R,
    values: &BTreeMap<String, R::Value>,
    bound: &mut Bound<'e, OutcomeOf<R>>,
    mut e: &'e Expression<OpOf<R>>,
) -> OutcomeOf<R> {
    let mut names = Vec::new();
    while let Expression::Let(name, value, body) = e {
        let outcome = eval_in(reading, values, bound, value);
        bound.entry(name.as_str()).or_default().push(outcome);
        names.push(name.as_str());
        e = body;
    }
    let gives = eval_in(reading, values, bound, e);
    unbind(bound, &names);
    gives
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
