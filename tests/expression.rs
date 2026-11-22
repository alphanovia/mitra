//! The core (`docs/core2.md`) on a third, tiny standard: counting
//! numbers, where `pred` of zero has no value and `if_zero` reads only
//! the branch it takes. And `let`: its scope, shadowing, a fault in what
//! it binds, and sharing (S6).

mod common;

use common::{inline, let_in, Rng};
use mitra_core::expression::{check, eval, fits, strict, Expression, Fault, Reading, Standard};
use proptest::prelude::*;
use std::cell::Cell;
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq)]
enum Ty {
    Nat,
}

#[derive(Clone, Debug)]
enum Op {
    Zero,
    Succ,
    Pred,
    IfZero,
}

struct Counting;

impl Standard for Counting {
    type Type = Ty;
    type Op = Op;
    fn gives(&self, op: &Op, takes: &[Ty]) -> Option<Ty> {
        match (op, takes) {
            (Op::Zero, [])
            | (Op::Succ | Op::Pred, [Ty::Nat])
            | (Op::IfZero, [Ty::Nat, Ty::Nat, Ty::Nat]) => Some(Ty::Nat),
            _ => None,
        }
    }
}

impl Reading for Counting {
    type Standard = Counting;
    type Value = u64;
    type Fault = &'static str;
    fn has_type(&self, _: &u64, _: &Ty) -> bool {
        true
    }
    fn apply(
        &self,
        op: &Op,
        args: &[Result<u64, Fault<&'static str>>],
    ) -> Result<u64, Fault<&'static str>> {
        if let (Op::IfZero, [n, zero, other]) = (op, args) {
            return match n.clone()? {
                0 => zero.clone(),
                _ => other.clone(),
            };
        }
        match (op, strict(args)?.as_slice()) {
            (Op::Zero, []) => Ok(0),
            (Op::Succ, [n]) => n.checked_add(1).ok_or(Fault::Op("too big")),
            (Op::Pred, [n]) => n.checked_sub(1).ok_or(Fault::Op("pred of zero")),
            _ => Err(Fault::Stuck),
        }
    }
}

type E = Expression<Op>;

fn op(op: Op, args: Vec<E>) -> E {
    Expression::Apply(op, args)
}
fn n() -> E {
    Expression::Name("n".to_string())
}
fn zero() -> E {
    op(Op::Zero, vec![])
}
fn nat() -> BTreeMap<String, Ty> {
    BTreeMap::from([("n".to_string(), Ty::Nat)])
}

#[test]
fn checking() {
    assert_eq!(
        check(&Counting, &nat(), &op(Op::Succ, vec![n()])),
        Some(Ty::Nat)
    );
    assert_eq!(check(&Counting, &BTreeMap::new(), &n()), None);
    assert_eq!(check(&Counting, &nat(), &op(Op::Succ, vec![])), None);
    assert_eq!(
        check(&Counting, &nat(), &op(Op::Succ, vec![n(), n()])),
        None
    );
}

#[test]
fn evaluating() {
    let at = |v: u64| BTreeMap::from([("n".to_string(), v)]);
    assert_eq!(eval(&Counting, &at(4), &op(Op::Pred, vec![n()])), Ok(3));
    assert_eq!(
        eval(&Counting, &at(0), &op(Op::Pred, vec![n()])),
        Err(Fault::Op("pred of zero"))
    );
    assert_eq!(eval(&Counting, &BTreeMap::new(), &n()), Err(Fault::Stuck));
    // The branch not taken may fault.
    let guarded = op(Op::IfZero, vec![n(), zero(), op(Op::Pred, vec![n()])]);
    assert_eq!(eval(&Counting, &at(0), &guarded), Ok(0));
    // A strict operation gives the first fault, left to right.
    let first: Vec<Result<u64, Fault<&str>>> = vec![
        Ok(1),
        Err(Fault::Op("a")),
        Err(Fault::Stuck),
        Err(Fault::Op("b")),
    ];
    assert_eq!(strict(&first), Err(Fault::Op("a")));
    assert_eq!(
        eval(&Counting, &at(u64::MAX), &op(Op::Succ, vec![n()])),
        Err(Fault::Op("too big"))
    );
}

#[test]
fn fitting() {
    assert!(fits(
        &Counting,
        &nat(),
        &BTreeMap::from([("n".to_string(), 1), ("m".to_string(), 2)])
    ));
    assert!(!fits(&Counting, &nat(), &BTreeMap::new()));
    assert!(fits(&Counting, &BTreeMap::new(), &BTreeMap::new()));
}

fn name(x: &str) -> E {
    Expression::Name(x.to_string())
}

/// A random expression, its names among `n` and those `let` binds in
/// scope: `x` and `y`, and `n` again, shadowing the input.
fn random(rng: &mut Rng, depth: u32, scope: &mut Vec<&'static str>) -> E {
    if depth == 0 || rng.below(4) == 0 {
        return match rng.below(3) {
            0 if !scope.is_empty() => name(rng.pick(scope)),
            0 | 1 => n(),
            _ => zero(),
        };
    }
    let d = depth - 1;
    match rng.below(4) {
        0 => op(Op::Succ, vec![random(rng, d, scope)]),
        1 => op(Op::Pred, vec![random(rng, d, scope)]),
        2 => op(Op::IfZero, (0..3).map(|_| random(rng, d, scope)).collect()),
        _ => {
            let x = *rng.pick(&["x", "y", "n"]);
            let value = random(rng, d, scope);
            scope.push(x);
            let body = random(rng, d, scope);
            scope.pop();
            let_in(x, value, body)
        }
    }
}

#[test]
fn letting() {
    let at = |v: u64| BTreeMap::from([("n".to_string(), v)]);
    let succ = |e: E| op(Op::Succ, vec![e]);
    let pred = |e: E| op(Op::Pred, vec![e]);
    // let x = n + 1; x + 1
    let e = let_in("x", succ(n()), succ(name("x")));
    assert_eq!(check(&Counting, &nat(), &e), Some(Ty::Nat));
    assert_eq!(eval(&Counting, &at(1), &e), Ok(3));
    // Shadowing: let x = n + 1; let x = x + 1; x
    let e = let_in("x", succ(n()), let_in("x", succ(name("x")), name("x")));
    assert_eq!(eval(&Counting, &at(1), &e), Ok(3));
    // An input shadowed: let n = 0; n + 1
    assert_eq!(
        eval(&Counting, &at(5), &let_in("n", zero(), succ(n()))),
        Ok(1)
    );
    // Lexical scope: let x = n; let n = 0; x is the input n.
    let e = let_in("x", n(), let_in("n", zero(), name("x")));
    assert_eq!(eval(&Counting, &at(5), &e), Ok(5));
    // A name is bound in the body only.
    let e = op(
        Op::IfZero,
        vec![let_in("x", zero(), name("x")), name("x"), name("x")],
    );
    assert_eq!(check(&Counting, &nat(), &e), None);
    assert_eq!(eval(&Counting, &at(5), &e), Err(Fault::Stuck));
    let e = let_in("x", name("x"), zero());
    assert_eq!(check(&Counting, &nat(), &e), None);
    // What a let binds checks even when it is not read, as in Rust.
    let e = let_in("x", op(Op::Succ, vec![]), zero());
    assert_eq!(check(&Counting, &nat(), &e), None);
    // A fault in what a let binds counts where it is read only: unlike
    // Rust, `let x = 0 - 1; 0` has a value.
    assert_eq!(
        eval(&Counting, &at(5), &let_in("x", pred(zero()), zero())),
        Ok(0)
    );
    assert_eq!(
        eval(
            &Counting,
            &at(5),
            &let_in("x", pred(zero()), succ(name("x")))
        ),
        Err(Fault::Op("pred of zero"))
    );
    // let x = n - 1; if n == 0 { 0 } else { x }
    let guarded = let_in("x", pred(n()), op(Op::IfZero, vec![n(), zero(), name("x")]));
    assert_eq!(eval(&Counting, &at(0), &guarded), Ok(0));
    assert_eq!(eval(&Counting, &at(3), &guarded), Ok(2));
}

/// Counting, with a count of the operations applied.
struct Tally(Cell<usize>);

impl Reading for Tally {
    type Standard = Counting;
    type Value = u64;
    type Fault = &'static str;
    fn has_type(&self, v: &u64, t: &Ty) -> bool {
        Counting.has_type(v, t)
    }
    fn apply(
        &self,
        op: &Op,
        args: &[Result<u64, Fault<&'static str>>],
    ) -> Result<u64, Fault<&'static str>> {
        self.0.set(self.0.get() + 1);
        Counting.apply(op, args)
    }
}

/// What a let gives: what it binds is evaluated once, however often it
/// is read.
#[test]
fn a_let_is_evaluated_once() {
    let twice = op(Op::Succ, vec![op(Op::Succ, vec![n()])]);
    let e = let_in(
        "x",
        twice,
        op(Op::IfZero, vec![name("x"), name("x"), name("x")]),
    );
    let at = BTreeMap::from([("n".to_string(), 1)]);
    let (shared, written) = (Tally(Cell::new(0)), Tally(Cell::new(0)));
    assert_eq!(eval(&shared, &at, &e), Ok(3));
    assert_eq!(eval(&written, &at, &inline(&e)), Ok(3));
    assert_eq!((shared.0.get(), written.0.get()), (3, 7));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3000))]

    /// S2 and S4 on the third standard.
    #[test]
    fn preservation_progress_determinism(seed in any::<u64>(), v in any::<u64>(), depth in 0u32..8) {
        let e = random(&mut Rng::new(seed), depth, &mut vec![]);
        let values = BTreeMap::from([("n".to_string(), v)]);
        prop_assert_eq!(check(&Counting, &nat(), &e), Some(Ty::Nat));
        let out = eval(&Counting, &values, &e);
        prop_assert!(!matches!(out, Err(Fault::Stuck)));
        prop_assert_eq!(out, eval(&Counting, &values, &e));
    }

    /// S6: a let shares. It gives what the expression with every let
    /// written out gives, and it checks only if that does.
    #[test]
    fn a_let_shares(seed in any::<u64>(), v in 0u64..4, depth in 0u32..8) {
        let e = random(&mut Rng::new(seed), depth, &mut vec![]);
        let values = BTreeMap::from([("n".to_string(), v)]);
        let written = inline(&e);
        prop_assert_eq!(check(&Counting, &nat(), &written), Some(Ty::Nat));
        prop_assert_eq!(eval(&Counting, &values, &e), eval(&Counting, &values, &written));
    }
}
