//! The core (`docs/core2.md`) on two tiny standards. Counting: numbers
//! where `pred` of zero has no value, `if_zero` reads only the branch it
//! takes, and two binders, `let` and a `let` of a pair. Picking: numbers
//! where `choose(a, b)` is `a` or `b`, read as the set of what may come
//! out, where `let` chooses once.

mod common;

use common::{apply, inline, let_in, Rng};
use mitra_core::expression::{
    check, eval, fits, strict, substitute, Arg, Expression, Fault, Outcome, Reading, Standard,
};
use proptest::prelude::*;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::convert::Infallible;

#[derive(Clone, Debug, PartialEq)]
enum Ty {
    Nat,
}

#[derive(Clone, Debug, PartialEq)]
enum Op {
    Zero,
    Succ,
    Pred,
    IfZero,
    /// `let x = a; b`: the second argument binds `x`.
    Let,
    /// `let (x, y) = (a, b); c`: the third argument binds `x` and `y`.
    Pair,
}

struct Counting;

impl Standard for Counting {
    type Type = Ty;
    type Op = Op;
    fn binds(&self, op: &Op, i: usize, before: &[Ty]) -> Vec<Ty> {
        match (op, i, before) {
            (Op::Let, 1, [a]) => vec![a.clone()],
            (Op::Pair, 2, [a, b]) => vec![a.clone(), b.clone()],
            _ => vec![],
        }
    }
    fn gives(&self, op: &Op, takes: &[Ty]) -> Option<Ty> {
        match (op, takes) {
            (Op::Zero, [])
            | (Op::Succ | Op::Pred, [Ty::Nat])
            | (Op::IfZero, [Ty::Nat, Ty::Nat, Ty::Nat]) => Some(Ty::Nat),
            (Op::Let, [_, b]) | (Op::Pair, [_, _, b]) => Some(b.clone()),
            _ => None,
        }
    }
}

type Counted = Result<u64, Fault<&'static str>>;

impl Reading for Counting {
    type Standard = Counting;
    type Value = u64;
    type Fault = &'static str;
    fn has_type(&self, _: &u64, _: &Ty) -> bool {
        true
    }
    fn apply(&self, op: &Op, args: &[Counted]) -> Counted {
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
    fn bind(
        &self,
        op: &Op,
        args: &[Counted],
        run: &mut dyn FnMut(usize, &[Counted]) -> Counted,
    ) -> Counted {
        match (op, args) {
            (Op::Let, [a, _]) => run(1, std::slice::from_ref(a)),
            (Op::Pair, [a, b, _]) => run(2, &[a.clone(), b.clone()]),
            _ => Err(Fault::Stuck),
        }
    }
}

type E = Expression<Op>;

fn op(op: Op, args: Vec<E>) -> E {
    apply(op, args)
}
fn name(x: &str) -> E {
    Expression::Name(x.to_string())
}
fn n() -> E {
    name("n")
}
fn zero() -> E {
    op(Op::Zero, vec![])
}
fn succ(e: E) -> E {
    op(Op::Succ, vec![e])
}
fn pred(e: E) -> E {
    op(Op::Pred, vec![e])
}
fn let_(x: &str, value: E, body: E) -> E {
    let_in(Op::Let, x, value, body)
}
/// `let (x, y) = (a, b); body`.
fn pair(x: &str, y: &str, a: E, b: E, body: E) -> E {
    let plain = |body| Arg {
        binds: vec![],
        body,
    };
    let names = vec![x.to_string(), y.to_string()];
    Expression::Apply(
        Op::Pair,
        vec![plain(a), plain(b), Arg { binds: names, body }],
    )
}
fn nat() -> BTreeMap<String, Ty> {
    BTreeMap::from([("n".to_string(), Ty::Nat)])
}
fn at(v: u64) -> BTreeMap<String, u64> {
    BTreeMap::from([("n".to_string(), v)])
}
fn is_let(op: &Op) -> bool {
    matches!(op, Op::Let)
}

#[test]
fn checking() {
    assert_eq!(check(&Counting, &nat(), &succ(n())), Some(Ty::Nat));
    assert_eq!(check(&Counting, &BTreeMap::new(), &n()), None);
    assert_eq!(check(&Counting, &nat(), &op(Op::Succ, vec![])), None);
    assert_eq!(
        check(&Counting, &nat(), &op(Op::Succ, vec![n(), n()])),
        None
    );
}

#[test]
fn evaluating() {
    assert_eq!(eval(&Counting, &at(4), &pred(n())), Ok(3));
    assert_eq!(
        eval(&Counting, &at(0), &pred(n())),
        Err(Fault::Op("pred of zero"))
    );
    assert_eq!(eval(&Counting, &BTreeMap::new(), &n()), Err(Fault::Stuck));
    // The branch not taken may fault.
    let guarded = op(Op::IfZero, vec![n(), zero(), pred(n())]);
    assert_eq!(eval(&Counting, &at(0), &guarded), Ok(0));
    // A strict operation gives the first fault, left to right.
    let first: Vec<Counted> = vec![
        Ok(1),
        Err(Fault::Op("a")),
        Err(Fault::Stuck),
        Err(Fault::Op("b")),
    ];
    assert_eq!(strict(&first), Err(Fault::Op("a")));
    assert_eq!(
        eval(&Counting, &at(u64::MAX), &succ(n())),
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

#[test]
fn letting() {
    // let x = n + 1; x + 1
    let e = let_("x", succ(n()), succ(name("x")));
    assert_eq!(check(&Counting, &nat(), &e), Some(Ty::Nat));
    assert_eq!(eval(&Counting, &at(1), &e), Ok(3));
    // Shadowing: let x = n + 1; let x = x + 1; x
    let e = let_("x", succ(n()), let_("x", succ(name("x")), name("x")));
    assert_eq!(eval(&Counting, &at(1), &e), Ok(3));
    // An input shadowed: let n = 0; n + 1
    assert_eq!(
        eval(&Counting, &at(5), &let_("n", zero(), succ(n()))),
        Ok(1)
    );
    // Lexical scope: let x = n; let n = 0; x is the input n.
    let e = let_("x", n(), let_("n", zero(), name("x")));
    assert_eq!(eval(&Counting, &at(5), &e), Ok(5));
    // A name is bound in the body only.
    let e = op(
        Op::IfZero,
        vec![let_("x", zero(), name("x")), name("x"), name("x")],
    );
    assert_eq!(check(&Counting, &nat(), &e), None);
    assert_eq!(eval(&Counting, &at(5), &e), Err(Fault::Stuck));
    assert_eq!(
        check(&Counting, &nat(), &let_("x", name("x"), zero())),
        None
    );
    // What a let binds checks even when it is not read, as in Rust.
    let e = let_("x", op(Op::Succ, vec![]), zero());
    assert_eq!(check(&Counting, &nat(), &e), None);
    // A fault in what a let binds counts where it is read only: unlike
    // Rust, `let x = 0 - 1; 0` has a value.
    assert_eq!(
        eval(&Counting, &at(5), &let_("x", pred(zero()), zero())),
        Ok(0)
    );
    assert_eq!(
        eval(&Counting, &at(5), &let_("x", pred(zero()), succ(name("x")))),
        Err(Fault::Op("pred of zero"))
    );
    // let x = n - 1; if n == 0 { 0 } else { x }
    let guarded = let_("x", pred(n()), op(Op::IfZero, vec![n(), zero(), name("x")]));
    assert_eq!(eval(&Counting, &at(0), &guarded), Ok(0));
    assert_eq!(eval(&Counting, &at(3), &guarded), Ok(2));
}

/// An argument binds the names its operation binds there, no more and
/// no fewer, for its own body only. Where it does not, the expression
/// does not check, and evaluating it is stuck.
#[test]
fn binding() {
    // let (x, y) = (n + 1, 0); if y == 0 { x } else { y }
    let e = pair(
        "x",
        "y",
        succ(n()),
        zero(),
        op(Op::IfZero, vec![name("y"), name("x"), name("y")]),
    );
    assert_eq!(check(&Counting, &nat(), &e), Some(Ty::Nat));
    assert_eq!(eval(&Counting, &at(4), &e), Ok(5));
    // Two names alike: the last is the one read, in type and in value.
    let e = pair("x", "x", zero(), succ(n()), name("x"));
    assert_eq!(eval(&Counting, &at(4), &e), Ok(5));
    let wrong: Vec<E> = vec![
        // A name on an argument that binds none.
        Expression::Apply(
            Op::Let,
            vec![
                Arg {
                    binds: vec!["x".to_string()],
                    body: zero(),
                },
                Arg {
                    binds: vec!["y".to_string()],
                    body: name("y"),
                },
            ],
        ),
        // No name where one is bound.
        op(Op::Let, vec![zero(), zero()]),
        // An operation that binds nothing, given a name.
        Expression::Apply(
            Op::Succ,
            vec![Arg {
                binds: vec!["x".to_string()],
                body: zero(),
            }],
        ),
        // Two names where one is bound.
        Expression::Apply(
            Op::Let,
            vec![
                Arg {
                    binds: vec![],
                    body: zero(),
                },
                Arg {
                    binds: vec!["x".to_string(), "y".to_string()],
                    body: name("x"),
                },
            ],
        ),
    ];
    for e in wrong {
        assert_eq!(check(&Counting, &nat(), &e), None, "{e:?}");
        assert_eq!(eval(&Counting, &at(4), &e), Err(Fault::Stuck), "{e:?}");
    }
}

/// `substitute` writes an expression in place of a name, never inside
/// an argument that binds that name, and renames a name an argument
/// binds before it catches one of what is written.
#[test]
fn substituting() {
    // y := n + 1 in `if y == 0 { y } else { 0 }`.
    let e = op(Op::IfZero, vec![name("y"), name("y"), zero()]);
    assert_eq!(
        substitute(&e, "y", &succ(n())),
        op(Op::IfZero, vec![succ(n()), succ(n()), zero()])
    );
    // Shadowed: in `let y = 0; y`, y is not the one written.
    let e = let_("y", zero(), name("y"));
    assert_eq!(substitute(&e, "y", &n()), e);
    // In `let x = 0; y`, writing `x` for `y` renames the let's x first.
    let e = let_("x", zero(), succ(name("y")));
    assert_eq!(
        substitute(&e, "y", &name("x")),
        let_("x'", zero(), succ(name("x")))
    );
    // The new name is not one the body uses.
    let e = let_(
        "x",
        zero(),
        op(Op::IfZero, vec![name("y"), name("x'"), name("x")]),
    );
    assert_eq!(
        substitute(&e, "y", &name("x")),
        let_(
            "x''",
            zero(),
            op(Op::IfZero, vec![name("x"), name("x'"), name("x''")])
        )
    );
    // Nothing is renamed where the name is not read.
    let e = let_("x", zero(), zero());
    assert_eq!(substitute(&e, "y", &name("x")), e);
    // Two names alike, the last read: each renamed, the read one kept so.
    // `let (x, x) = (0, n + 1); if y == 0 { x } else { 0 }`, y := the input x.
    let e = pair(
        "x",
        "x",
        zero(),
        succ(n()),
        op(Op::IfZero, vec![name("y"), name("x"), zero()]),
    );
    let written = substitute(&e, "y", &name("x"));
    let values = BTreeMap::from([("x".to_string(), 0), ("n".to_string(), 6)]);
    assert_eq!(eval(&Counting, &values, &written), Ok(7));
    let values = BTreeMap::from([("x".to_string(), 1), ("n".to_string(), 6)]);
    assert_eq!(eval(&Counting, &values, &written), Ok(0));
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
    fn apply(&self, op: &Op, args: &[Counted]) -> Counted {
        self.0.set(self.0.get() + 1);
        Counting.apply(op, args)
    }
    fn bind(
        &self,
        op: &Op,
        args: &[Counted],
        run: &mut dyn FnMut(usize, &[Counted]) -> Counted,
    ) -> Counted {
        Counting.bind(op, args, run)
    }
}

/// What a let binds is evaluated once, however often it is read.
#[test]
fn a_let_is_evaluated_once() {
    let twice = succ(succ(n()));
    let e = let_(
        "x",
        twice,
        op(Op::IfZero, vec![name("x"), name("x"), name("x")]),
    );
    let (shared, written) = (Tally(Cell::new(0)), Tally(Cell::new(0)));
    assert_eq!(eval(&shared, &at(1), &e), Ok(3));
    assert_eq!(eval(&written, &at(1), &inline(&e, &is_let)), Ok(3));
    assert_eq!((shared.0.get(), written.0.get()), (3, 7));
}

/// The names the random expressions read and bind: `n` is also an
/// input, and `x'` is what `substitute` renames `x` to first.
const NAMES: [&str; 4] = ["x", "y", "n", "x'"];

/// A random expression, its names among `free` and those bound in scope.
fn random(rng: &mut Rng, depth: u32, scope: &mut Vec<String>, free: &[&str]) -> E {
    if depth == 0 || rng.below(4) == 0 {
        return match rng.below(3) {
            0 if !scope.is_empty() => name(rng.pick(scope)),
            0 | 1 => name(rng.pick(free)),
            _ => zero(),
        };
    }
    let d = depth - 1;
    let sub = |rng: &mut Rng, binds: &[&str], scope: &mut Vec<String>| {
        scope.extend(binds.iter().map(|b| b.to_string()));
        let e = random(rng, d, scope, free);
        scope.truncate(scope.len() - binds.len());
        e
    };
    match rng.below(5) {
        0 => succ(sub(rng, &[], scope)),
        1 => pred(sub(rng, &[], scope)),
        2 => op(Op::IfZero, (0..3).map(|_| sub(rng, &[], scope)).collect()),
        3 => {
            let x = *rng.pick(&NAMES);
            let value = sub(rng, &[], scope);
            let_(x, value, sub(rng, &[x], scope))
        }
        _ => {
            let (x, y) = (*rng.pick(&NAMES), *rng.pick(&NAMES));
            let (a, b) = (sub(rng, &[], scope), sub(rng, &[], scope));
            pair(x, y, a, b, sub(rng, &[x, y], scope))
        }
    }
}

fn all_nat() -> BTreeMap<String, Ty> {
    NAMES.iter().map(|x| (x.to_string(), Ty::Nat)).collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3000))]

    /// S2 and S4 on Counting.
    #[test]
    fn preservation_progress_determinism(seed in any::<u64>(), v in any::<u64>(), depth in 0u32..8) {
        let e = random(&mut Rng::new(seed), depth, &mut vec![], &["n"]);
        prop_assert_eq!(check(&Counting, &nat(), &e), Some(Ty::Nat));
        let out = eval(&Counting, &at(v), &e);
        prop_assert!(!matches!(out, Err(Fault::Stuck)));
        prop_assert_eq!(out, eval(&Counting, &at(v), &e));
    }

    /// Substitution, in the core: writing `e₁` in place of `x` in `e₂`
    /// gives what `e₂` gives with `x` standing for what `e₁` gives, and
    /// the same type. The names of `e₁` are those the binders of `e₂`
    /// bind, so a substitution that caught one would show.
    #[test]
    fn substitution(seed in any::<u64>(), values in proptest::array::uniform4(0u64..3), depth in 0u32..7) {
        let mut rng = Rng::new(seed);
        let e1 = random(&mut rng, depth.min(3), &mut vec![], &NAMES);
        let e2 = random(&mut rng, depth, &mut vec![], &NAMES);
        let written = substitute(&e2, "x", &e1);
        prop_assert_eq!(check(&Counting, &all_nat(), &written), Some(Ty::Nat));
        let values: BTreeMap<String, u64> = NAMES.iter().map(|x| x.to_string()).zip(values).collect();
        if let Ok(v) = eval(&Counting, &values, &e1) {
            let mut with_x = values.clone();
            with_x.insert("x".to_string(), v);
            prop_assert_eq!(eval(&Counting, &with_x, &e2), eval(&Counting, &values, &written), "{:?}\n{:?}", e2, written);
        }
    }

    /// S6 on Counting: a let gives what the expression with every let
    /// written out gives, faults included, and that checks too.
    #[test]
    fn a_let_shares(seed in any::<u64>(), v in 0u64..4, depth in 0u32..8) {
        let e = random(&mut Rng::new(seed), depth, &mut vec![], &["n"]);
        let written = inline(&e, &is_let);
        prop_assert_eq!(check(&Counting, &nat(), &written), Some(Ty::Nat));
        prop_assert_eq!(eval(&Counting, &at(v), &e), eval(&Counting, &at(v), &written));
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Num;

#[derive(Clone, Debug, PartialEq)]
enum Pick {
    Int(i64),
    Sub,
    /// `choose(a, b)`: `a` or `b`.
    Choose,
    Let,
}

struct Picking;

impl Standard for Picking {
    type Type = Num;
    type Op = Pick;
    fn binds(&self, op: &Pick, i: usize, before: &[Num]) -> Vec<Num> {
        match (op, i, before) {
            (Pick::Let, 1, [a]) => vec![a.clone()],
            _ => vec![],
        }
    }
    fn gives(&self, op: &Pick, takes: &[Num]) -> Option<Num> {
        match (op, takes) {
            (Pick::Int(_), [])
            | (Pick::Sub | Pick::Choose, [Num, Num])
            | (Pick::Let, [Num, Num]) => Some(Num),
            _ => None,
        }
    }
}

/// The reading of what may come out: a set of numbers. `let` chooses
/// once: `x` is one of what its value may be, the same wherever it is
/// read, and the body is read once for each.
struct Sets;

type Picked = Outcome<BTreeSet<i64>, Infallible>;

impl Reading for Sets {
    type Standard = Picking;
    type Value = BTreeSet<i64>;
    type Fault = Infallible;
    fn has_type(&self, value: &BTreeSet<i64>, _: &Num) -> bool {
        !value.is_empty()
    }
    fn apply(&self, op: &Pick, args: &[Picked]) -> Picked {
        Ok(match (op, strict(args)?.as_slice()) {
            (Pick::Int(n), []) => BTreeSet::from([*n]),
            (Pick::Sub, [a, b]) => a
                .iter()
                .flat_map(|x| b.iter().map(move |y| x.wrapping_sub(*y)))
                .collect(),
            (Pick::Choose, [a, b]) => a.union(b).copied().collect(),
            _ => return Err(Fault::Stuck),
        })
    }
    fn bind(
        &self,
        op: &Pick,
        args: &[Picked],
        run: &mut dyn FnMut(usize, &[Picked]) -> Picked,
    ) -> Picked {
        match (op, args) {
            (Pick::Let, [value, _]) => {
                let mut all = BTreeSet::new();
                for x in value.clone()? {
                    all.extend(run(1, &[Ok(BTreeSet::from([x]))])?);
                }
                Ok(all)
            }
            _ => Err(Fault::Stuck),
        }
    }
}

fn pick(op: Pick, args: Vec<Expression<Pick>>) -> Expression<Pick> {
    apply(op, args)
}
fn int(n: i64) -> Expression<Pick> {
    pick(Pick::Int(n), vec![])
}

/// `let x = choose(0, 1); x - x` is 0: both `x` are the one choice.
/// Written out, `choose(0, 1) - choose(0, 1)` may be -1, 0 or 1. So on a
/// reading where a value may be one of several, a let is not what
/// writing out its value gives (S6 fails, by design).
#[test]
fn a_let_chooses_once() {
    let x = || Expression::Name("x".to_string());
    let coin = pick(Pick::Choose, vec![int(0), int(1)]);
    let e = let_in(
        Pick::Let,
        "x",
        coin.clone(),
        pick(Pick::Sub, vec![x(), x()]),
    );
    assert_eq!(check(&Picking, &BTreeMap::new(), &e), Some(Num));
    assert_eq!(eval(&Sets, &BTreeMap::new(), &e), Ok(BTreeSet::from([0])));
    let written = inline(&e, &|op| matches!(op, Pick::Let));
    assert_eq!(written, pick(Pick::Sub, vec![coin.clone(), coin]));
    assert_eq!(
        eval(&Sets, &BTreeMap::new(), &written),
        Ok(BTreeSet::from([-1, 0, 1]))
    );
}

/// A random expression of Picking, its names `c`, an input, and those
/// bound in scope.
fn random_pick(rng: &mut Rng, depth: u32, scope: &mut Vec<String>) -> Expression<Pick> {
    if depth == 0 || rng.below(4) == 0 {
        return match rng.below(3) {
            0 if !scope.is_empty() => Expression::Name(rng.pick(scope).clone()),
            0 => Expression::Name("c".to_string()),
            _ => int(rng.below(3) as i64 - 1),
        };
    }
    let d = depth - 1;
    match rng.below(3) {
        0 => {
            let x = rng.pick(&["x", "y", "c"]).to_string();
            let value = random_pick(rng, d, scope);
            scope.push(x.clone());
            let body = random_pick(rng, d, scope);
            scope.pop();
            let_in(Pick::Let, &x, value, body)
        }
        k => {
            let op = [Pick::Sub, Pick::Choose][k - 1].clone();
            let (a, b) = (random_pick(rng, d, scope), random_pick(rng, d, scope));
            pick(op, vec![a, b])
        }
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    /// Choosing once gives fewer: what a let gives is among what writing
    /// it out gives, and the same where nothing is chosen. Names keep
    /// substitution: `c` standing for a set gives what writing out an
    /// expression of that set gives.
    #[test]
    fn choosing_once_gives_fewer(seed in any::<u64>(), depth in 0u32..6) {
        let mut rng = Rng::new(seed);
        let e = random_pick(&mut rng, depth, &mut vec![]);
        let c = BTreeMap::from([("c".to_string(), Num)]);
        prop_assert_eq!(check(&Picking, &c, &e), Some(Num));
        let coin = pick(Pick::Choose, vec![int(2), int(5)]);
        let values = BTreeMap::from([("c".to_string(), BTreeSet::from([2, 5]))]);
        let shared = eval(&Sets, &values, &e).unwrap();
        let written = eval(&Sets, &values, &inline(&e, &|op| matches!(op, Pick::Let))).unwrap();
        prop_assert!(shared.is_subset(&written), "{:?}: {:?} and {:?}", e, shared, written);
        let chooses = format!("{e:?}").contains("Choose") || format!("{e:?}").contains("Name(\"c\")");
        prop_assert!(chooses || shared == written);
        prop_assert_eq!(eval(&Sets, &values, &e), eval(&Sets, &BTreeMap::new(), &substitute(&e, "c", &coin)));
    }
}
