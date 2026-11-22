//! The core (`docs/core2.md`) on a third, tiny standard: counting
//! numbers, where `pred` of zero has no value and `if_zero` reads only
//! the branch it takes.

mod common;

use common::Rng;
use mitra_core::expression::{check, eval, fits, strict, Expression, Fault, Reading, Standard};
use proptest::prelude::*;
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
    Expression::Input("n".to_string())
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

fn random(rng: &mut Rng, depth: u32) -> E {
    if depth == 0 || rng.below(4) == 0 {
        return match rng.coin() {
            true => n(),
            false => zero(),
        };
    }
    match rng.below(3) {
        0 => op(Op::Succ, vec![random(rng, depth - 1)]),
        1 => op(Op::Pred, vec![random(rng, depth - 1)]),
        _ => op(Op::IfZero, (0..3).map(|_| random(rng, depth - 1)).collect()),
    }
}

proptest! {
    /// S2 and S4 on the third standard.
    #[test]
    fn preservation_progress_determinism(seed in any::<u64>(), v in any::<u64>(), depth in 0u32..8) {
        let e = random(&mut Rng::new(seed), depth);
        let values = BTreeMap::from([("n".to_string(), v)]);
        prop_assert_eq!(check(&Counting, &nat(), &e), Some(Ty::Nat));
        let out = eval(&Counting, &values, &e);
        prop_assert!(!matches!(out, Err(Fault::Stuck)));
        prop_assert_eq!(out, eval(&Counting, &values, &e));
    }
}
