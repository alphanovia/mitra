//! Typed expressions with no built-in type (`docs/design/core2.md`).
//!
//! A *standard* is a language: its types, its operations, the names each
//! argument of an operation binds, and the type each operation gives. A
//! *reading* is a meaning of a standard: its values, which values have
//! which type, and what each operation gives on values. One expression
//! has one type and may have several readings, such as exact values and
//! the bounds they lie in.
//!
//! This file knows no type, no value and no operation. A constant is an
//! operation that takes nothing, and `let` an operation whose second
//! argument binds a name.
//!
//! An expression is a binding tree (Harper, *Practical Foundations for
//! Programming Languages*, ch. 1): a name, or an operation on arguments,
//! each an expression and the names the operation binds in it. A name
//! stands for what the nearest argument around it that binds it is given,
//! or else for the input of that name.
//!
//! The typing judgement `Γ ⊢ e : T` is `check(standard, Γ, e) == Some(T)`,
//! where `Γ, x : U` is `Γ` with `x` now of type `U`:
//!
//! ```text
//!   Γ(x) = T
//!   ───────────────
//!   Γ ⊢ Name(x) : T
//!
//!   each i: argument i binds [x₁ … xₖ], binds(op, i, [T₁ … Tᵢ₋₁]) = [U₁ … Uₖ],
//!           Γ, x₁ : U₁, …, xₖ : Uₖ ⊢ eᵢ : Tᵢ             gives(op, [T₁ … Tₙ]) = T
//!   ─────────────────────────────────────────────────────────────────────────
//!   Γ ⊢ Apply(op, [e₁ … eₙ]) : T
//! ```
//!
//! So `let x = e₁; e₂` is `Apply(Let, [e₁, x.e₂])` in a standard where
//! `binds(Let, 1, [T₁]) = [T₁]` and `gives(Let, [T₁, T₂]) = T₂`.
//!
//! Every argument that binds no name is evaluated first, left to right.
//! An operation none of whose arguments binds a name is read by `apply`,
//! on what each gave. Any other is read by `bind`, which also runs each
//! argument that binds names, at values of those names, as often as it
//! needs.
//!
//! What a reading must keep, for every operation (agreement): when
//! `gives(op, [T₁ … Tₙ]) = Some(T)` as above, and each argument gives a
//! value of its type `Tᵢ` or an operation's fault whenever its names
//! stand for values of their types `Uⱼ` or operations' faults, then the
//! operation gives a value of type `T` or an operation's fault, and
//! `bind` runs an argument only at such values of its names. Neither
//! `apply` nor `bind` panics.
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
//! `T` stands for. For every operation, typed as above: when each
//! argument gives in `C` an outcome in γ of what it gives in `A`, at
//! values of its names in `C` in γ of those in `A`, what the operation
//! gives in `C` is in γ of what it gives in `A`. Then by induction, as
//! for preservation: when `check(s, Γ, e) == Some(T)`, the inputs' values
//! in `A` fit `Γ` and each input's value in `C` is in γ of its value in
//! `A` at its type, `eval(C, …, e)` is in `γ_T(eval(A, …, e))`.
//!
//! Substitution: `substitute(e₂, x, e₁)` writes `e₁` in place of each `x`
//! free in `e₂`, and renames first any name an argument of `e₂` binds
//! that would catch a name of `e₁`. When `check(s, Γ, e₁) == Some(T₁)`
//! and `check(s, Γ + x : T₁, e₂) == Some(T₂)`, `substitute(e₂, x, e₁)`
//! checks to `T₂` in `Γ`. On every reading, when `eval(r, values, e₁) ==
//! Ok(v)`, `eval(r, values + x = v, e₂)` is `eval(r, values,
//! substitute(e₂, x, e₁))`. Whether a standard's `let` gives what writing
//! out its value gives is up to the standard (`docs/design/core2.md`, S6).
//!
//! `check` and `eval` never panic, on any expression and any values.
//! They recurse once per level of the expression, as do `substitute` and
//! the derived `Clone`, `PartialEq` and drop, and an argument that binds
//! names runs inside its reading's `bind`. So depth is bounded by the
//! stack: on a thread of 2 MiB, 1,000 levels in a debug build and 2,000
//! in a release build hold for all of them, for the standards here, where
//! a simulation `all` counts as three. Time and memory are not bounded by
//! the size of an expression: an operation may run an argument many
//! times, and each `let` may square a number.

use std::collections::{BTreeMap, BTreeSet};

/// An expression: a name, or an operation on arguments.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression<Op> {
    /// What the nearest argument around it that binds it is given, or
    /// else the input of that name.
    Name(String),
    /// An operation on its arguments.
    Apply(Op, Vec<Arg<Op>>),
}

/// An argument of an operation: an expression, and the names the
/// operation binds in it, in scope in `body` only. Most bind none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Arg<Op> {
    pub binds: Vec<String>,
    pub body: Expression<Op>,
}

/// A language: its types and its operations.
pub trait Standard {
    type Type: Clone;
    type Op;
    /// The types of the names argument `i` of `op` binds, in order, when
    /// the arguments before it have the types `before`. None, unless the
    /// standard says otherwise.
    fn binds(&self, op: &Self::Op, i: usize, before: &[Self::Type]) -> Vec<Self::Type> {
        let _ = (op, i, before);
        Vec::new()
    }
    /// The type `op` gives when its arguments have these types, in
    /// order. `None` when it does not take them.
    fn gives(&self, op: &Self::Op, takes: &[Self::Type]) -> Option<Self::Type>;
}

/// Why an expression has no value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Fault<F> {
    /// An operation has no value here, by its own reading: a division
    /// by zero, for one.
    Op(F),
    /// Evaluation went wrong: a name has no value, or an operation got
    /// values or arguments it does not take. A checked expression on
    /// values that fit its inputs never goes wrong.
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
/// them. Its methods are functions: the same arguments give the same
/// answer, so a reading keeps no state that changes one.
pub trait Reading {
    type Standard: Standard;
    type Value: Clone;
    type Fault: Clone;
    /// `value` is a value of type `of`, by the reading's own test.
    fn has_type(&self, value: &Self::Value, of: &TypeOf<Self>) -> bool;
    /// What `op` gives when none of its arguments binds a name. Each
    /// argument is computed first, and is what its expression gave, fault
    /// included: an operation that does not need one, as `if` does not
    /// need the branch it does not take, may give a value anyway. One
    /// that needs them all reads them with `strict`.
    fn apply(&self, op: &OpOf<Self>, args: &[OutcomeOf<Self>]) -> OutcomeOf<Self>;
    /// What `op` gives when an argument binds names. `args` is as for
    /// `apply`, but an argument that binds names has no outcome of its
    /// own: `Stuck` stands in its place. `run(i, values)` is what argument
    /// `i` gives with its names standing for `values`, in order. The
    /// operation runs it as often as it needs: `let` once, at the value
    /// it binds. `Stuck`, unless the reading says otherwise.
    #[allow(clippy::type_complexity)]
    fn bind(
        &self,
        op: &OpOf<Self>,
        args: &[OutcomeOf<Self>],
        run: &mut dyn FnMut(usize, &[OutcomeOf<Self>]) -> OutcomeOf<Self>,
    ) -> OutcomeOf<Self> {
        let _ = (op, args, run);
        Err(Fault::Stuck)
    }
    /// Whether argument `i` of `op` is needed, given what the arguments
    /// before it gave: one that is not is never computed, and `Stuck`
    /// stands in its place. Only an argument `apply` would drop may be
    /// left out, as `if` drops the branch it does not take, so this
    /// changes the time a reading takes, never what it gives. Every one,
    /// unless the reading says otherwise.
    fn needed(&self, op: &OpOf<Self>, i: usize, before: &[OutcomeOf<Self>]) -> bool {
        let _ = (op, i, before);
        true
    }
}

/// What each name in scope stands for: its bindings, the nearest last.
type Scope<'e, X> = BTreeMap<&'e str, Vec<X>>;

/// What the nearest binding of `name` stands for.
fn nearest<'s, X>(scope: &'s Scope<'_, X>, name: &str) -> Option<&'s X> {
    scope.get(name).and_then(|xs| xs.last())
}

/// Each of `names` now stands for the next of `xs`.
fn enter<'e, X>(scope: &mut Scope<'e, X>, names: &'e [String], xs: impl IntoIterator<Item = X>) {
    for (name, x) in names.iter().zip(xs) {
        scope.entry(name.as_str()).or_default().push(x);
    }
}

/// Takes back one binding of each of `names`.
fn leave<X>(scope: &mut Scope<'_, X>, names: &[String]) {
    for name in names {
        if let Some(xs) = scope.get_mut(name.as_str()) {
            xs.pop();
        }
    }
}

/// The type of `e` when its inputs have the types of `inputs`. `None`
/// when it has none: a name neither bound nor in `inputs`, an argument
/// that binds other names than its operation binds there, or an
/// operation that does not take the types it is given.
pub fn check<S: Standard>(
    standard: &S,
    inputs: &BTreeMap<String, S::Type>,
    e: &Expression<S::Op>,
) -> Option<S::Type> {
    check_in(standard, inputs, &mut Scope::new(), e)
}

fn check_in<'e, S: Standard>(
    standard: &S,
    inputs: &BTreeMap<String, S::Type>,
    scope: &mut Scope<'e, S::Type>,
    e: &'e Expression<S::Op>,
) -> Option<S::Type> {
    match e {
        Expression::Name(name) => nearest(scope, name).or_else(|| inputs.get(name)).cloned(),
        Expression::Apply(op, args) => {
            let mut takes = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                let types = standard.binds(op, i, &takes);
                if types.len() != arg.binds.len() {
                    return None;
                }
                enter(scope, &arg.binds, types);
                let t = check_in(standard, inputs, scope, &arg.body);
                leave(scope, &arg.binds);
                takes.push(t?);
            }
            standard.gives(op, &takes)
        }
    }
}

/// What `e` gives at these values of its inputs. Every argument that
/// binds no name is evaluated, left to right, and the operation is given
/// all of them; an operation with an argument that binds names also
/// runs it.
pub fn eval<R: Reading>(
    reading: &R,
    values: &BTreeMap<String, R::Value>,
    e: &Expression<OpOf<R>>,
) -> OutcomeOf<R> {
    eval_in(reading, values, &mut Scope::new(), e)
}

fn eval_in<'e, R: Reading>(
    reading: &R,
    values: &BTreeMap<String, R::Value>,
    scope: &mut Scope<'e, OutcomeOf<R>>,
    e: &'e Expression<OpOf<R>>,
) -> OutcomeOf<R> {
    match e {
        Expression::Name(name) => read::<R>(values, scope, name),
        Expression::Apply(op, args) => {
            let mut outcomes = Vec::with_capacity(args.len());
            for (i, arg) in args.iter().enumerate() {
                let needed = arg.binds.is_empty() && reading.needed(op, i, &outcomes);
                outcomes.push(match needed {
                    true => eval_in(reading, values, scope, &arg.body),
                    false => Err(Fault::Stuck),
                });
            }
            match args.iter().all(|arg| arg.binds.is_empty()) {
                true => reading.apply(op, &outcomes),
                false => bind(reading, values, scope, op, args, &outcomes),
            }
        }
    }
}

/// What a name stands for: its nearest binding, or else the input.
#[inline(never)]
fn read<R: Reading>(
    values: &BTreeMap<String, R::Value>,
    scope: &Scope<'_, OutcomeOf<R>>,
    name: &str,
) -> OutcomeOf<R> {
    match nearest(scope, name) {
        Some(outcome) => outcome.clone(),
        None => values.get(name).cloned().ok_or(Fault::Stuck),
    }
}

/// An operation with an argument that binds names, given `outcomes`:
/// the reading runs that argument.
#[inline(never)]
fn bind<'e, R: Reading>(
    reading: &R,
    values: &BTreeMap<String, R::Value>,
    scope: &mut Scope<'e, OutcomeOf<R>>,
    op: &OpOf<R>,
    args: &'e [Arg<OpOf<R>>],
    outcomes: &[OutcomeOf<R>],
) -> OutcomeOf<R> {
    let mut run = |i: usize, given: &[OutcomeOf<R>]| match args.get(i) {
        Some(arg) if arg.binds.len() == given.len() => {
            enter(scope, &arg.binds, given.iter().cloned());
            let gives = eval_in(reading, values, scope, &arg.body);
            leave(scope, &arg.binds);
            gives
        }
        _ => Err(Fault::Stuck),
    };
    reading.bind(op, outcomes, &mut run)
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

/// `e` with `with` written in place of each `name` free in it. Where an
/// argument binds a name free in `with`, and `name` is free in it, that
/// name is first renamed, adding `'` until it is new, so that no name of
/// `with` is caught.
pub fn substitute<Op: Clone>(
    e: &Expression<Op>,
    name: &str,
    with: &Expression<Op>,
) -> Expression<Op> {
    let Expression::Apply(op, args) = e else {
        return match e {
            Expression::Name(n) if n == name => with.clone(),
            other => other.clone(),
        };
    };
    let mut written = Vec::with_capacity(args.len());
    for arg in args {
        written.push(substitute_in(arg, name, with));
    }
    Expression::Apply(op.clone(), written)
}

/// `substitute` in one argument.
fn substitute_in<Op: Clone>(arg: &Arg<Op>, name: &str, with: &Expression<Op>) -> Arg<Op> {
    if arg.binds.iter().any(|b| b == name) || !free(&arg.body).contains(name) {
        return arg.clone();
    }
    let caught = free(with);
    let mut binds = arg.binds.clone();
    // The body once a name it binds is renamed.
    let mut renamed: Option<Expression<Op>> = None;
    // The last of two equal names is the one read: rename it first.
    for k in (0..binds.len()).rev() {
        if caught.contains(&binds[k]) {
            let body = renamed.as_ref().unwrap_or(&arg.body);
            let used = free(body);
            let mut new = binds[k].clone();
            while caught.contains(&new) || used.contains(&new) || binds.contains(&new) {
                new.push('\'');
            }
            renamed = Some(substitute(body, &binds[k], &Expression::Name(new.clone())));
            binds[k] = new;
        }
    }
    let body = substitute(renamed.as_ref().unwrap_or(&arg.body), name, with);
    Arg { binds, body }
}

/// The names free in `e`: those no argument around them binds.
fn free<Op>(e: &Expression<Op>) -> BTreeSet<String> {
    let mut found = BTreeSet::new();
    free_in(e, &mut Scope::new(), &mut found);
    found
}

fn free_in<'e, Op>(e: &'e Expression<Op>, bound: &mut Scope<'e, ()>, found: &mut BTreeSet<String>) {
    match e {
        Expression::Name(n) if nearest(bound, n).is_none() => {
            found.insert(n.clone());
        }
        Expression::Name(_) => {}
        Expression::Apply(_, args) => {
            for arg in args {
                enter(bound, &arg.binds, std::iter::repeat(()));
                free_in(&arg.body, bound, found);
                leave(bound, &arg.binds);
            }
        }
    }
}
