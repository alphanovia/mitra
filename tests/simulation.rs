//! The simulation standard (`docs/core2.md`): S1 to S5 on random
//! expressions, and the faults found in foundation2's reading, each as a
//! case.

mod common;

use common::Rng;
use mitra_core::expression::{check, eval, fits, Expression, Fault, Outcome, Reading, Standard};
use mitra_core::simulation::{
    covers, lies_in, Bounds, Exact, Op, Ranges, Rational, Record, Simulation, Span, Type,
    Undefined, Value,
};
use num_bigint::BigInt;
use num_traits::{Signed, Zero};
use proptest::prelude::*;
use std::collections::BTreeMap;

type E = Expression<Op>;

fn apply(op: Op, args: Vec<E>) -> E {
    Expression::Apply(op, args)
}
fn input(name: &str) -> E {
    Expression::Input(name.to_string())
}
fn int(n: i64) -> E {
    apply(Op::Int(BigInt::from(n)), vec![])
}
fn real(n: i64, d: i64) -> E {
    apply(Op::Real(Rational::new(n, d).unwrap()), vec![])
}
fn record(name: &str, fields: &[(&str, Type)]) -> Record {
    Record {
        name: name.to_string(),
        fields: fields
            .iter()
            .map(|(n, t)| (n.to_string(), t.clone()))
            .collect(),
    }
}

/// The records the generators use: a record named "Real", one nested,
/// one empty, and one with two fields of one name.
fn records() -> Vec<Record> {
    let point = record("Point", &[("x", Type::Real), ("y", Type::Real)]);
    vec![
        point.clone(),
        record("Real", &[("n", Type::Int), ("d", Type::Int)]),
        record(
            "Nest",
            &[
                ("p", Type::Record(point)),
                ("ok", Type::Truth),
                ("n", Type::Int),
            ],
        ),
        record("Empty", &[]),
        record("Dup", &[("x", Type::Int), ("x", Type::Truth)]),
    ]
}

struct Gen {
    rng: Rng,
    records: Vec<Record>,
    inputs: BTreeMap<String, Type>,
}

impl Gen {
    fn new(seed: u64) -> Gen {
        let mut g = Gen {
            rng: Rng::new(seed),
            records: records(),
            inputs: BTreeMap::new(),
        };
        for i in 0..g.rng.below(6) {
            let t = g.ty();
            g.inputs.insert(format!("x{i}"), t);
        }
        g
    }
    fn ty(&mut self) -> Type {
        let mut all = vec![Type::Truth, Type::Int, Type::Real];
        all.extend(self.records.iter().cloned().map(Type::Record));
        self.rng.pick(&all).clone()
    }
    fn number_type(&mut self) -> Type {
        match self.rng.coin() {
            true => Type::Int,
            false => Type::Real,
        }
    }
    fn int(&mut self) -> BigInt {
        match self.rng.below(8) {
            0 => BigInt::zero(),
            1 | 2 => BigInt::from(self.rng.below(11) as i64 - 5),
            3 => BigInt::from(i64::MIN),
            4 => BigInt::from(i64::MAX),
            5 => BigInt::from(2).pow(70) * if self.rng.coin() { 1 } else { -1 },
            _ => BigInt::from((self.next_i64()) >> self.rng.below(63)),
        }
    }
    fn next_i64(&mut self) -> i64 {
        self.rng.next() as i64
    }
    fn rational(&mut self) -> Rational {
        match self.rng.below(6) {
            0 => Rational::integer(0),
            1 => Rational::new(1, 4_294_967_291_i64).unwrap(),
            2 => Rational::new(1, 4_294_967_279_i64).unwrap(),
            3 => Rational::integer(self.int()),
            _ => loop {
                let (n, d) = (self.int(), self.int());
                if let Some(q) = Rational::new(n, d) {
                    break q;
                }
            },
        }
    }
    fn value(&mut self, t: &Type) -> Value {
        match t {
            Type::Truth => Value::Truth(self.rng.coin()),
            Type::Int => Value::Int(self.int()),
            Type::Real => Value::Real(self.rational()),
            Type::Record(r) => Value::Record(
                r.clone(),
                r.fields.iter().map(|(_, f)| self.value(f)).collect(),
            ),
        }
    }
    fn constant(&mut self, t: &Type) -> E {
        match t {
            Type::Truth => apply(Op::Truth(self.rng.coin()), vec![]),
            Type::Int => apply(Op::Int(self.int()), vec![]),
            Type::Real => apply(Op::Real(self.rational()), vec![]),
            Type::Record(r) => {
                let fields = r.fields.iter().map(|(_, f)| self.constant(f)).collect();
                apply(Op::Make(r.clone()), fields)
            }
        }
    }
    fn leaf(&mut self, t: &Type) -> E {
        let names: Vec<String> = self
            .inputs
            .iter()
            .filter(|(_, it)| *it == t)
            .map(|(n, _)| n.clone())
            .collect();
        match names.is_empty() || self.rng.coin() {
            true => self.constant(t),
            false => Expression::Input(self.rng.pick(&names).clone()),
        }
    }
    /// A random expression of type `t`.
    fn expr(&mut self, t: &Type, depth: u32) -> E {
        if depth == 0 || self.rng.below(5) == 0 {
            return self.leaf(t);
        }
        let d = depth - 1;
        let reads: Vec<(Record, String)> = self
            .records
            .iter()
            .flat_map(|r| {
                r.fields
                    .iter()
                    .filter(|(n, _)| r.field(n).is_some_and(|(_, f)| f == t))
                    .map(|(n, _)| (r.clone(), n.clone()))
            })
            .collect();
        match self.rng.below(10) {
            0 => {
                let test = self.expr(&Type::Truth, d);
                let (a, b) = (self.expr(t, d), self.expr(t, d));
                return apply(Op::If, vec![test, a, b]);
            }
            1 if !reads.is_empty() => {
                let (r, name) = self.rng.pick(&reads).clone();
                let of = self.expr(&Type::Record(r), d);
                return apply(Op::Field(name), vec![of]);
            }
            _ => {}
        }
        match t {
            Type::Truth => match self.rng.below(7) {
                0 => {
                    let a = self.expr(t, d);
                    apply(Op::Not, vec![a])
                }
                1 | 2 => {
                    let op = if self.rng.coin() { Op::And } else { Op::Or };
                    let (a, b) = (self.expr(t, d), self.expr(t, d));
                    apply(op, vec![a, b])
                }
                3 => {
                    let n = self.number_type();
                    let op = if self.rng.coin() {
                        Op::Less
                    } else {
                        Op::AtMost
                    };
                    let (a, b) = (self.expr(&n, d), self.expr(&n, d));
                    apply(op, vec![a, b])
                }
                4 | 5 => {
                    let u = self.ty();
                    let (a, b) = (self.expr(&u, d), self.expr(&u, d));
                    apply(Op::Equal, vec![a, b])
                }
                _ => {
                    let n = self.number_type();
                    let args = (0..3).map(|_| self.expr(&n, d)).collect();
                    apply(Op::Within, args)
                }
            },
            Type::Int | Type::Real => {
                let ops = [Op::Add, Op::Sub, Op::Mul, Op::Div];
                let op = self
                    .rng
                    .pick(&ops[..if t == &Type::Real { 4 } else { 3 }])
                    .clone();
                let (a, b) = (self.expr(t, d), self.expr(t, d));
                apply(op, vec![a, b])
            }
            Type::Record(r) => {
                let fields = r.fields.iter().map(|(_, f)| self.expr(f, d)).collect();
                apply(Op::Make(r.clone()), fields)
            }
        }
    }
    fn values(&mut self) -> BTreeMap<String, Value> {
        let inputs = self.inputs.clone();
        inputs
            .iter()
            .map(|(n, t)| (n.clone(), self.value(t)))
            .collect()
    }
    fn span(&mut self, t: &Type) -> Span {
        match t {
            Type::Truth => {
                let (yes, no) = *self.rng.pick(&[(true, false), (false, true), (true, true)]);
                Span::Truth { yes, no }
            }
            Type::Int | Type::Real if self.rng.below(5) == 0 => Span::Number(None),
            Type::Int => {
                let low = self.int();
                let wide = match self.rng.coin() {
                    true => BigInt::from(self.rng.below(4)),
                    false => self.int().abs(),
                };
                let high = &low + wide;
                Span::Number(Some((Rational::integer(low), Rational::integer(high))))
            }
            Type::Real => {
                let low = self.rational();
                let wide = match self.rng.below(3) {
                    0 => Rational::integer(0),
                    _ => self.rational().abs(),
                };
                let high = low.plus(&wide);
                Span::Number(Some((low, high)))
            }
            Type::Record(r) => Span::Record(
                r.clone(),
                r.fields.iter().map(|(_, f)| self.span(f)).collect(),
            ),
        }
    }
    /// A value in `span`: γ, sampled.
    fn member(&mut self, span: &Span, t: &Type) -> Value {
        match (span, t) {
            (Span::Truth { yes, no }, _) => Value::Truth(match (yes, no) {
                (true, true) => self.rng.coin(),
                (yes, _) => *yes,
            }),
            (Span::Number(None), t) if self.rng.below(3) > 0 => self.value(t),
            (Span::Number(None), Type::Int) => Value::Int(BigInt::zero()),
            (Span::Number(None), _) => Value::Real(Rational::integer(0)),
            // Zero, where it lies in the span, is where faults are: take it often.
            (Span::Number(Some((low, high))), t)
                if low <= &Rational::integer(0)
                    && &Rational::integer(0) <= high
                    && self.rng.coin() =>
            {
                match t {
                    Type::Int => Value::Int(BigInt::zero()),
                    _ => Value::Real(Rational::integer(0)),
                }
            }
            (Span::Number(Some((low, high))), Type::Int) => {
                let (low, high) = (low.numer().clone(), high.numer().clone());
                let wide: BigInt = &high - &low + 1;
                let n = match self.rng.below(3) {
                    0 => low,
                    1 => high,
                    _ => &low + BigInt::from(self.rng.next()) % wide,
                };
                Value::Int(n)
            }
            (Span::Number(Some((low, high))), _) => {
                let m = self.rng.below(7) as i64 + 1;
                let k = self.rng.below(m as usize + 1) as i64;
                let step = high.minus(low).times(&Rational::new(k, m).unwrap());
                Value::Real(low.plus(&step))
            }
            (Span::Record(r, spans), Type::Record(_)) => {
                let fields: Vec<Value> = spans
                    .iter()
                    .zip(&r.fields)
                    .map(|(s, (_, f))| self.member(s, f))
                    .collect();
                Value::Record(r.clone(), fields)
            }
            _ => unreachable!("a span of another type"),
        }
    }
    /// Bounds for every input, and an exact value in each.
    fn bounded(&mut self) -> (BTreeMap<String, Bounds>, BTreeMap<String, Value>) {
        let (mut bounds, mut exact) = (BTreeMap::new(), BTreeMap::new());
        for (name, t) in self.inputs.clone() {
            let b = Bounds {
                span: self.span(&t),
                may_fault: self.rng.below(5) == 0,
            };
            exact.insert(name.clone(), self.member(&b.span, &t));
            bounds.insert(name, b);
        }
        (bounds, exact)
    }
    /// Bounds of type `t` or a fault, and an exact outcome in γ of them:
    /// a fault where the bounds may fault or are one.
    fn outcome(&mut self, t: &Type) -> (Outcome<Bounds, Undefined>, Outcome<Value, Undefined>) {
        fn fault<V>() -> Outcome<V, Undefined> {
            Err(Fault::Op(Undefined::DivisionByZero))
        }
        if self.rng.below(6) == 0 {
            return (fault(), fault());
        }
        let b = Bounds {
            span: self.span(t),
            may_fault: self.rng.below(3) == 0,
        };
        let x = match b.may_fault && self.rng.coin() {
            true => fault(),
            false => Ok(self.member(&b.span, t)),
        };
        (Ok(b), x)
    }
    /// Any operation, with any parameters.
    fn wild_op(&mut self) -> Op {
        let r = self.rng.pick(&self.records.clone()).clone();
        let name = self
            .rng
            .pick(&["x", "y", "p", "ok", "n", "d", "zz"])
            .to_string();
        let all = [
            Op::Truth(true),
            Op::Int(BigInt::from(3)),
            Op::Real(Rational::new(1, 3).unwrap()),
            Op::Not,
            Op::And,
            Op::Or,
            Op::Add,
            Op::Sub,
            Op::Mul,
            Op::Div,
            Op::Less,
            Op::AtMost,
            Op::Equal,
            Op::Within,
            Op::If,
            Op::Make(r),
            Op::Field(name),
        ];
        self.rng.pick(&all).clone()
    }
    /// Any expression, typed or not.
    fn wild(&mut self, depth: u32) -> E {
        if depth == 0 || self.rng.below(4) == 0 {
            return match self.rng.coin() {
                true => Expression::Input(format!("x{}", self.rng.below(8))),
                false => {
                    let t = self.ty();
                    self.constant(&t)
                }
            };
        }
        let op = self.wild_op();
        let args = (0..self.rng.below(4))
            .map(|_| self.wild(depth - 1))
            .collect();
        apply(op, args)
    }
    /// Any value, of any shape: records with fields missing or wrong.
    fn wild_value(&mut self) -> Value {
        let t = self.ty();
        match (self.rng.below(4), t) {
            (0, Type::Record(r)) => {
                let fields = (0..self.rng.below(3)).map(|_| self.wild_value()).collect();
                Value::Record(r, fields)
            }
            (_, t) => self.value(&t),
        }
    }
    /// Any bounds, of any shape: spans out of order, truths that are
    /// neither, records with fields missing.
    fn wild_bounds(&mut self) -> Bounds {
        let t = self.ty();
        let span = match self.rng.below(5) {
            0 => Span::Truth {
                yes: false,
                no: false,
            },
            1 => Span::Number(Some((Rational::integer(2), Rational::new(1, 2).unwrap()))),
            2 => match t {
                Type::Record(r) => Span::Record(r, vec![]),
                t => self.span(&t),
            },
            _ => self.span(&t),
        };
        Bounds {
            span,
            may_fault: self.rng.coin(),
        }
    }
}

/// S2 for one reading: a value of the type, or an operation's fault.
fn progress<R: Reading<Standard = Simulation>>(
    r: &R,
    v: &Result<R::Value, Fault<R::Fault>>,
    t: &Type,
) -> bool {
    match v {
        Ok(v) => r.has_type(v, t),
        Err(Fault::Op(_)) => true,
        Err(Fault::Stuck) => false,
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3000))]

    /// S1, S2, S4 on the exact reading.
    #[test]
    fn exact_preserves_progresses_and_is_determined(seed in any::<u64>(), depth in 1u32..6) {
        let mut g = Gen::new(seed);
        let t = g.ty();
        let e = g.expr(&t, depth);
        prop_assert_eq!(check(&Simulation, &g.inputs, &e), Some(t.clone()));
        let values = g.values();
        prop_assert!(fits(&Exact, &g.inputs, &values));
        let out = eval(&Exact, &values, &e);
        prop_assert!(progress(&Exact, &out, &t), "{:?} gave {:?}", e, out);
        prop_assert_eq!(out, eval(&Exact, &values, &e));
    }

    /// S1, S2, S4 on the reading by bounds.
    #[test]
    fn ranges_preserve_progress_and_are_determined(seed in any::<u64>(), depth in 1u32..6) {
        let mut g = Gen::new(seed);
        let t = g.ty();
        let e = g.expr(&t, depth);
        let (bounds, _) = g.bounded();
        prop_assert!(fits(&Ranges, &g.inputs, &bounds));
        let out = eval(&Ranges, &bounds, &e);
        prop_assert!(progress(&Ranges, &out, &t), "{:?} gave {:?}", e, out);
        prop_assert_eq!(out, eval(&Ranges, &bounds, &e));
    }

    /// S5: every exact outcome lies in the bounded one, when each exact
    /// input lies in its bounds. Depth 1 is each operation alone.
    #[test]
    fn exact_lies_in_ranges(seed in any::<u64>(), depth in 1u32..6) {
        let mut g = Gen::new(seed);
        let t = g.ty();
        let e = g.expr(&t, depth);
        let (bounds, exact) = g.bounded();
        prop_assert!(fits(&Ranges, &g.inputs, &bounds));
        for (name, value) in &exact {
            let (b, x) = (Ok(bounds[name].clone()), Ok(value.clone()));
            prop_assert!(covers(&g.inputs[name], &b, &x));
        }
        let (b, x) = (eval(&Ranges, &bounds, &e), eval(&Exact, &exact, &e));
        prop_assert!(covers(&t, &b, &x), "{:?}\nbounds {:?}\nexact {:?}\nat {:?} in {:?}", e, b, x, exact, bounds);
    }

    /// S5 for each operation alone, its obligation in `expression`:
    /// arguments of their types or faults, each exact one in γ of its
    /// bounds at its type. The induction needs no more.
    #[test]
    fn each_operation_covers(seed in any::<u64>()) {
        let mut g = Gen::new(seed);
        let t = g.ty();
        let Expression::Apply(op, args) = g.expr(&t, 1) else {
            return Ok(());
        };
        let takes: Vec<Type> = args.iter().map(|a| check(&Simulation, &g.inputs, a).unwrap()).collect();
        prop_assert_eq!(Simulation.gives(&op, &takes), Some(t.clone()));
        let (bounds, exact): (Vec<_>, Vec<_>) = takes.iter().map(|ty| g.outcome(ty)).unzip();
        for ((ty, b), x) in takes.iter().zip(&bounds).zip(&exact) {
            prop_assert!(b.as_ref().map_or(true, |b| Ranges.has_type(b, ty)));
            prop_assert!(covers(ty, b, x));
        }
        let (b, x) = (Ranges.apply(&op, &bounds), Exact.apply(&op, &exact));
        prop_assert!(covers(&t, &b, &x), "{:?} on {:?}\nbounds {:?}\nexact {:?}", op, takes, b, x);
    }

    /// S5 on points: when every input's bounds are its exact value, the
    /// bounds of the outcome are the exact outcome, or its fault. Bounds
    /// that said "anything, may fault" everywhere would cover, and fail
    /// here.
    #[test]
    fn points_are_exact(seed in any::<u64>(), depth in 1u32..6) {
        let mut g = Gen::new(seed);
        let t = g.ty();
        let e = g.expr(&t, depth);
        let exact = g.values();
        let points: BTreeMap<_, _> = exact.iter().map(|(n, v)| (n.clone(), Bounds::exactly(v))).collect();
        let (b, x) = (eval(&Ranges, &points, &e), eval(&Exact, &exact, &e));
        prop_assert!(covers(&t, &b, &x), "{:?}\nbounds {:?}\nexact {:?}", e, b, x);
        prop_assert_eq!(&b, &x.clone().map(|v| Bounds::exactly(&v)), "{:?}", e);
    }

    /// S3: no expression and no values make `check` or `eval` panic.
    #[test]
    fn nothing_panics(seed in any::<u64>(), depth in 0u32..6) {
        let mut g = Gen::new(seed);
        let e = g.wild(depth);
        let values: BTreeMap<_, _> = (0..8).map(|i| (format!("x{i}"), g.wild_value())).collect();
        let bounds: BTreeMap<_, _> = (0..8).map(|i| (format!("x{i}"), g.wild_bounds())).collect();
        let _ = check(&Simulation, &g.inputs, &e);
        let _ = eval(&Exact, &values, &e);
        let _ = eval(&Ranges, &bounds, &e);
    }
}

fn at(pairs: &[(&str, Value)]) -> BTreeMap<String, Value> {
    pairs
        .iter()
        .map(|(n, v)| (n.to_string(), v.clone()))
        .collect()
}
fn typed(pairs: &[(&str, Type)]) -> BTreeMap<String, Type> {
    pairs
        .iter()
        .map(|(n, t)| (n.to_string(), t.clone()))
        .collect()
}

/// foundation2: `within(i64::MIN, i64::MAX, 0)` had no exact value and
/// a surely false bound. Here both say false.
#[test]
fn within_at_the_ends_of_i64() {
    let e = apply(Op::Within, vec![int(i64::MIN), int(i64::MAX), int(0)]);
    assert_eq!(eval(&Exact, &BTreeMap::new(), &e), Ok(Value::Truth(false)));
    assert_eq!(
        eval(&Ranges, &BTreeMap::new(), &e).unwrap().surely(),
        Some(false)
    );
}

/// foundation2: the exact sum overflowed, and the bounds gave [0, 2]
/// with no fault. Here the sum is exact and the bounds are its point.
#[test]
fn a_sum_of_small_reals_is_exact() {
    let e = apply(
        Op::Add,
        vec![real(1, 4_294_967_291), real(1, 4_294_967_279)],
    );
    let sum = Rational::new(
        4_294_967_291_i64 + 4_294_967_279_i64,
        BigInt::from(4_294_967_291_i64) * BigInt::from(4_294_967_279_i64),
    )
    .unwrap();
    assert_eq!(
        eval(&Exact, &BTreeMap::new(), &e),
        Ok(Value::Real(sum.clone()))
    );
    let b = eval(&Ranges, &BTreeMap::new(), &e).unwrap();
    assert_eq!(b.span, Span::Number(Some((sum.clone(), sum))));
    assert!(!b.may_fault);
}

/// foundation2: a truth was accepted as an Int, and a record named
/// "Real" could pass for a real. Value typing reads the value.
#[test]
fn a_value_is_of_its_type_only() {
    let fake = record("Real", &[("n", Type::Int), ("d", Type::Int)]);
    let half = Value::Record(
        fake.clone(),
        vec![Value::Int(1.into()), Value::Int(2.into())],
    );
    assert!(!Exact.has_type(&Value::Truth(true), &Type::Int));
    assert!(!Exact.has_type(&half, &Type::Real));
    assert!(Exact.has_type(&half, &Type::Record(fake.clone())));
    // A record of the right name with fields of the wrong type is not one.
    let point = record("Point", &[("x", Type::Real), ("y", Type::Real)]);
    let wrong = Value::Record(
        point.clone(),
        vec![Value::Int(1.into()), Value::Int(2.into())],
    );
    assert!(!Exact.has_type(&wrong, &Type::Record(point)));
    // `+` does not take two of the fake.
    let inputs = typed(&[("a", Type::Record(fake.clone())), ("b", Type::Record(fake))]);
    assert_eq!(
        check(
            &Simulation,
            &inputs,
            &apply(Op::Add, vec![input("a"), input("b")])
        ),
        None
    );
}

/// foundation2: any `+ : Int, Int -> Truth` could be declared and then
/// gave nothing. Here operations are the standard's: `+` on two Int
/// gives an Int, and S2 holds for all of them.
#[test]
fn operations_are_the_standards() {
    let inputs = typed(&[("a", Type::Int)]);
    let e = apply(Op::Add, vec![input("a"), int(1)]);
    assert_eq!(check(&Simulation, &inputs, &e), Some(Type::Int));
    let e = apply(Op::Add, vec![input("a"), real(1, 2)]);
    assert_eq!(check(&Simulation, &inputs, &e), None);
}

/// `if`, `&&` and `||` read only what they need, as in Rust:
/// `if x != 0 { 1 / x } else { 0 }` has a value at 0.
#[test]
fn branches_not_taken_are_not_read() {
    let x = || input("x");
    let zero = || real(0, 1);
    let nonzero = apply(Op::Not, vec![apply(Op::Equal, vec![x(), zero()])]);
    let inverse = apply(
        Op::If,
        vec![
            nonzero.clone(),
            apply(Op::Div, vec![real(1, 1), x()]),
            zero(),
        ],
    );
    let at_zero = at(&[("x", Value::Real(Rational::integer(0)))]);
    assert_eq!(
        eval(&Exact, &at_zero, &inverse),
        Ok(Value::Real(Rational::integer(0)))
    );
    let guarded = apply(
        Op::And,
        vec![
            nonzero.clone(),
            apply(
                Op::Less,
                vec![apply(Op::Div, vec![real(1, 1), x()]), real(2, 1)],
            ),
        ],
    );
    assert_eq!(eval(&Exact, &at_zero, &guarded), Ok(Value::Truth(false)));
    let unguarded = apply(Op::Div, vec![real(1, 1), x()]);
    assert_eq!(
        eval(&Exact, &at_zero, &unguarded),
        Err(Fault::Op(mitra_core::simulation::Undefined::DivisionByZero))
    );
    // By bounds, x in [-1, 1]: the inverse may fault, the guarded `if` may too
    // (it may divide near 0), and both are sound.
    let around = BTreeMap::from([(
        "x".to_string(),
        Bounds::between(Rational::integer(-1), Rational::integer(1)).unwrap(),
    )]);
    assert!(eval(&Ranges, &around, &unguarded).unwrap().may_fault);
    assert!(covers(
        &Type::Real,
        &eval(&Ranges, &around, &inverse),
        &eval(&Exact, &at_zero, &inverse)
    ));
}

/// A claim proved over a whole domain: for every h in [0, 20],
/// 2 · 9.81 · h < 400.
#[test]
fn a_claim_holds_over_a_domain() {
    let h = BTreeMap::from([(
        "h".to_string(),
        Bounds::between(Rational::integer(0), Rational::integer(20)).unwrap(),
    )]);
    let product = apply(
        Op::Mul,
        vec![apply(Op::Mul, vec![real(2, 1), real(981, 100)]), input("h")],
    );
    let low = apply(Op::Less, vec![product.clone(), real(400, 1)]);
    let inputs = typed(&[("h", Type::Real)]);
    assert_eq!(check(&Simulation, &inputs, &low), Some(Type::Truth));
    assert_eq!(eval(&Ranges, &h, &low).unwrap().surely(), Some(true));
    let too_low = apply(Op::Less, vec![product, real(300, 1)]);
    assert_eq!(eval(&Ranges, &h, &too_low).unwrap().surely(), None);
}

/// Two fields of one name: `r.x` is the first, in type and in value.
#[test]
fn the_first_field_of_a_name() {
    let dup = record("Dup", &[("x", Type::Int), ("x", Type::Truth)]);
    let made = apply(Op::Make(dup), vec![int(7), apply(Op::Truth(true), vec![])]);
    let read = apply(Op::Field("x".to_string()), vec![made]);
    assert_eq!(check(&Simulation, &BTreeMap::new(), &read), Some(Type::Int));
    assert_eq!(
        eval(&Exact, &BTreeMap::new(), &read),
        Ok(Value::Int(7.into()))
    );
}

/// Red team, round 1: γ is read at a type. A real is in no bounds of a
/// whole number, and a record with a field of another type, or none, is
/// in no bounds of its record. Bounds with fractional ends are bounds of
/// a real only.
#[test]
fn gamma_is_typed() {
    let half = Rational::new(1, 2).unwrap();
    let unit = Ok(Bounds::between(Rational::integer(0), Rational::integer(1)).unwrap());
    assert!(covers(&Type::Int, &unit, &Ok(Value::Int(BigInt::from(1)))));
    assert!(!covers(&Type::Int, &unit, &Ok(Value::Real(half.clone()))));
    assert!(covers(&Type::Real, &unit, &Ok(Value::Real(half.clone()))));
    let r = record("R", &[("f", Type::Int)]);
    let wrong = Value::Record(r.clone(), vec![Value::Truth(true)]);
    let span = Span::Record(
        r.clone(),
        vec![Span::Truth {
            yes: true,
            no: false,
        }],
    );
    assert!(lies_in(&wrong, &span));
    let wrong_bounds = Ok(Bounds {
        span,
        may_fault: false,
    });
    assert!(!covers(&Type::Record(r.clone()), &wrong_bounds, &Ok(wrong)));
    let none = Ok(Bounds {
        span: Span::Record(r.clone(), vec![]),
        may_fault: false,
    });
    let empty = Value::Record(r.clone(), vec![]);
    assert!(!covers(&Type::Record(r.clone()), &none, &Ok(empty)));
    let fractional = Bounds::between(half, Rational::new(3, 2).unwrap()).unwrap();
    assert!(!Ranges.has_type(&fractional, &Type::Int));
    assert!(Ranges.has_type(&fractional, &Type::Real));
    let x = BTreeMap::from([("x".to_string(), Type::Int)]);
    let bounds = BTreeMap::from([("x".to_string(), fractional)]);
    assert!(!fits(&Ranges, &x, &bounds));
}

/// S3's bound on depth: an `if` nested 700 deep is checked and read, by
/// both readings, on a thread of 2 MiB.
#[test]
fn seven_hundred_levels() {
    let deep = std::thread::Builder::new()
        .stack_size(2 << 20)
        .spawn(|| {
            let mut e = input("x");
            for _ in 0..700 {
                e = apply(Op::If, vec![apply(Op::Truth(true), vec![]), e, int(0)]);
            }
            let x = typed(&[("x", Type::Int)]);
            let value = at(&[("x", Value::Int(BigInt::from(7)))]);
            let bounds = BTreeMap::from([("x".to_string(), Bounds::exactly(&value["x"]))]);
            (
                check(&Simulation, &x, &e),
                eval(&Exact, &value, &e),
                eval(&Ranges, &bounds, &e),
            )
        })
        .unwrap()
        .join()
        .unwrap();
    assert_eq!(deep.0, Some(Type::Int));
    assert_eq!(deep.1, Ok(Value::Int(BigInt::from(7))));
    assert_eq!(deep.2, Ok(Bounds::exactly(&Value::Int(BigInt::from(7)))));
}
