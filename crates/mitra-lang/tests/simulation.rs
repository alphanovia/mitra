//! The simulation standard (`docs/design/core2.md`): S1 to S5 on random
//! expressions, and the faults found in foundation2's reading, each as a
//! case.

mod common;

use common::{apply, inline, let_in, Rng};
use mitra_lang::expression::{
    check, eval, fits, substitute, Arg, Expression, Fault, Outcome, Reading, Standard,
};
use mitra_lang::simulation::{
    covers, lies_in, Bound, Bounds, Callee, Calls, Enum, Exact, Numbers, Op, Ranges, Rational,
    Record, Simulation, Span, Type, Undefined, Value,
};
use num_bigint::BigInt;
use num_traits::{Signed, Zero};
use proptest::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;

type E = Expression<Op>;

fn is_let(op: &Op) -> bool {
    matches!(op, Op::Let)
}
/// `(lo..hi).all(|x| p)`.
fn all(lo: E, hi: E, x: &str, p: E) -> E {
    let plain = |body| Arg {
        binds: vec![],
        body,
    };
    let p = Arg {
        binds: vec![x.to_string()],
        body: p,
    };
    Expression::Apply(Op::All, vec![plain(lo), plain(hi), p])
}
fn input(name: &str) -> E {
    Expression::Name(name.to_string())
}
fn int(n: i64) -> E {
    apply(Op::Int(BigInt::from(n)), vec![])
}
fn real(n: i64, d: i64) -> E {
    apply(Op::Real(Rational::new(n, d).unwrap()), vec![])
}
fn record(name: &str, fields: &[(&str, Type)]) -> Record {
    Record {
        args: vec![],
        name: name.to_string(),
        fields: fields
            .iter()
            .map(|(n, t)| (n.to_string(), t.clone()))
            .collect(),
    }
}

/// The records the generators use: a record named "rational", one nested,
/// one empty, and one with two fields of one name.
fn records() -> Vec<Record> {
    let point = record("Point", &[("x", Type::Real), ("y", Type::Real)]);
    vec![
        point.clone(),
        record("rational", &[("n", Type::Int), ("d", Type::Int)]),
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

/// The enums the generators use: one with a unit and a number, one of
/// units only, one holding a record and two values.
fn enums() -> Vec<Enum> {
    let point = records()[0].clone();
    let e = |name: &str, variants: &[(&str, &[Type])]| Enum {
        name: name.to_string(),
        args: vec![],
        variants: variants
            .iter()
            .map(|(n, ts)| (n.to_string(), ts.to_vec()))
            .collect(),
    };
    vec![
        e("Maybe", &[("None", &[]), ("Some", &[Type::Int])]),
        e("Light", &[("Red", &[]), ("Amber", &[]), ("Green", &[])]),
        e(
            "Either",
            &[
                ("L", &[Type::Real]),
                ("R", &[Type::Record(point), Type::Truth]),
            ],
        ),
    ]
}

struct Gen {
    rng: Rng,
    records: Vec<Record>,
    enums: Vec<Enum>,
    inputs: BTreeMap<String, Type>,
    /// The names `let` binds in scope, the nearest last.
    scope: Vec<(String, Type)>,
    /// The functions an expression may call.
    calls: Arc<Calls>,
}

impl Gen {
    fn new(seed: u64) -> Gen {
        Gen::library(seed, true)
    }
    /// A generator whose library's functions all have bodies, unless
    /// `opaque`: then some have only rows, or nothing.
    fn library(seed: u64, opaque: bool) -> Gen {
        let mut g = Gen {
            rng: Rng::new(seed),
            records: records(),
            enums: enums(),
            inputs: BTreeMap::new(),
            scope: Vec::new(),
            calls: Arc::default(),
        };
        // Three functions, each calling only those before it: with a
        // body, with rows (one of no argument always found), or opaque.
        let mut callees = BTreeMap::new();
        for i in 0..3 {
            let takes: Vec<(String, Type)> = (0..g.rng.below(3))
                .map(|k| (format!("a{k}"), g.ty()))
                .collect();
            let gives = g.ty();
            let (body, rows) = match g.rng.below(4) {
                n if n < 2 || !opaque => {
                    let mut sub = Gen {
                        rng: Rng::new(g.rng.next()),
                        records: g.records.clone(),
                        enums: g.enums.clone(),
                        inputs: takes.iter().cloned().collect(),
                        scope: Vec::new(),
                        calls: Arc::new(Calls::with(callees.clone()).unwrap()),
                    };
                    (Some(sub.expr(&gives, 3)), vec![])
                }
                2 => {
                    // One row a point: a function gives one value there.
                    let mut rows: Vec<(Vec<Value>, Value)> = Vec::new();
                    for _ in 0..1 + g.rng.below(3) {
                        let at: Vec<Value> = takes.iter().map(|(_, t)| g.value(t)).collect();
                        let gives = g.value(&gives);
                        if !rows.iter().any(|(a, _)| *a == at) {
                            rows.push((at, gives));
                        }
                    }
                    (None, rows)
                }
                _ => (None, vec![]),
            };
            let callee = Callee {
                promises: None,
                takes,
                gives,
                body,
                rows,
                ranges: Default::default(),
            };
            callees.insert(format!("lib::f{i}"), callee);
        }
        g.calls = Arc::new(Calls::with(callees).unwrap());
        for i in 0..g.rng.below(6) {
            let t = g.ty();
            g.inputs.insert(format!("x{i}"), t);
        }
        // A truth the ends of ranges may turn on.
        g.inputs.insert("c".to_string(), Type::Truth);
        g
    }
    fn standard(&self) -> Simulation {
        Simulation {
            calls: self.calls.clone(),
        }
    }
    fn exact(&self) -> Exact {
        Exact {
            calls: self.calls.clone(),
            log: Default::default(),
        }
    }
    fn ranges(&self) -> Ranges {
        Ranges {
            calls: self.calls.clone(),
        }
    }
    fn ty(&mut self) -> Type {
        let mut all = vec![Type::Truth, Type::Int, Type::Real];
        all.extend(self.records.iter().cloned().map(Type::Record));
        all.extend(self.enums.iter().cloned().map(Type::Enum));
        // Sequences: of numbers, and of an enum (finding 3, step 2).
        all.push(Type::Seq(Box::new(Type::Int)));
        all.push(Type::Seq(Box::new(Type::Enum(self.enums[0].clone()))));
        // Text, and its bytes (finding 3, step 3).
        all.push(Type::Encoded(Box::new(Type::Char)));
        all.push(Type::Byte);
        self.rng.pick(&all).clone()
    }
    /// A short text: ASCII, a letter of two bytes, one of four, and what
    /// a string escapes.
    fn text(&mut self) -> String {
        let pieces = ["a", "b", "é", "𝄞", "\"", "\\", "\n", "ab"];
        (0..self.length())
            .map(|_| *self.rng.pick(&pieces))
            .collect()
    }
    /// A text of `n` bytes.
    fn text_of(&mut self, n: usize) -> String {
        let mut s = String::new();
        while s.len() < n {
            s.push_str(match n - s.len() >= 2 && self.rng.coin() {
                true => "é",
                false => "a",
            });
        }
        s
    }
    /// A short length, `[]` often.
    fn length(&mut self) -> usize {
        self.rng.below(4)
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
            Type::Param { .. } => unreachable!("no value is of a parameter"),
            Type::Enum(e) => {
                let i = self.rng.below(e.variants.len());
                let holds = e.variants[i].1.iter().map(|t| self.value(t)).collect();
                Value::Variant(e.clone(), i, holds)
            }
            Type::Truth => Value::Truth(self.rng.coin()),
            Type::Int => Value::Int(self.int()),
            Type::Real => Value::Real(self.rational()),
            Type::Record(r) => Value::Record(
                r.clone(),
                r.fields.iter().map(|(_, f)| self.value(f)).collect(),
            ),
            Type::Seq(t) => {
                let k = self.length();
                Value::Seq((**t).clone(), (0..k).map(|_| self.value(t)).collect())
            }
            Type::Encoded(_) => Value::Text(self.text()),
            Type::Byte => Value::Byte(self.rng.below(256) as u8),
            Type::Char => unreachable!("no value of a letter is written yet"),
        }
    }
    fn constant(&mut self, t: &Type) -> E {
        match t {
            Type::Param { .. } => unreachable!("no value is of a parameter"),
            Type::Enum(e) => {
                let i = self.rng.below(e.variants.len());
                let holds = e.variants[i].1.iter().map(|t| self.constant(t)).collect();
                apply(Op::Variant(e.clone(), i), holds)
            }
            Type::Truth => apply(Op::Truth(self.rng.coin()), vec![]),
            Type::Int => apply(Op::Int(self.int()), vec![]),
            Type::Real => apply(Op::Real(self.rational()), vec![]),
            Type::Record(r) => {
                let fields = r.fields.iter().map(|(_, f)| self.constant(f)).collect();
                apply(Op::Make(r.clone()), fields)
            }
            Type::Seq(t) => {
                let k = self.length();
                let holds = (0..k).map(|_| self.constant(t)).collect();
                apply(Op::Seq((**t).clone()), holds)
            }
            Type::Encoded(_) => apply(Op::Text(self.text()), vec![]),
            // No byte is written: a place of a text, which may be none.
            Type::Byte => {
                let s = apply(Op::Text(self.text()), vec![]);
                apply(Op::Index, vec![s, int(self.rng.below(3) as i64)])
            }
            Type::Char => unreachable!("no value of a letter is written yet"),
        }
    }
    fn leaf(&mut self, t: &Type) -> E {
        let mut visible = self.inputs.clone();
        visible.extend(self.scope.iter().cloned());
        // A name a let binds, read often.
        let bound: Vec<String> = self
            .scope
            .iter()
            .filter(|(n, _)| &visible[n] == t)
            .map(|(n, _)| n.clone())
            .collect();
        if !bound.is_empty() && self.rng.coin() {
            return Expression::Name(self.rng.pick(&bound).clone());
        }
        let names: Vec<String> = visible
            .iter()
            .filter(|(_, it)| *it == t)
            .map(|(n, _)| n.clone())
            .collect();
        match names.is_empty() || self.rng.coin() {
            true => self.constant(t),
            false => Expression::Name(self.rng.pick(&names).clone()),
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
            // A let, which may shadow an input or another let, of any type.
            2 => {
                let name = self.rng.pick(&["l0", "l1", "x0", "x1"]).to_string();
                let u = match self.rng.coin() {
                    true => t.clone(),
                    false => self.ty(),
                };
                let value = self.expr(&u, d);
                self.scope.push((name.clone(), u));
                let body = self.expr(t, d);
                self.scope.pop();
                return let_in(Op::Let, &name, value, body);
            }
            // `(lo..hi).all(|x| p)` over a short range, whose ends may be
            // uncertain, `x` shadowing what it may.
            3 if t == &Type::Truth => {
                let name = self.rng.pick(&["l0", "l1", "x0", "x1"]).to_string();
                let (lo, hi) = (self.end(d), self.end(d));
                self.scope.push((name.clone(), Type::Int));
                let p = self.expr(t, d);
                self.scope.pop();
                return all(lo, hi, &name, p);
            }
            // A match on an enum, each arm binding what its variant
            // holds, names that may shadow.
            6 => {
                let e = self.rng.pick(&self.enums.clone()).clone();
                let of = self.expr(&Type::Enum(e.clone()), d);
                let mut args = vec![Arg {
                    binds: vec![],
                    body: of,
                }];
                for (_, holds) in &e.variants {
                    let binds: Vec<String> = holds
                        .iter()
                        .map(|_| self.rng.pick(&["l0", "l1", "x0", "x1"]).to_string())
                        .collect();
                    for (n, t) in binds.iter().zip(holds) {
                        self.scope.push((n.clone(), t.clone()));
                    }
                    let body = self.expr(t, d);
                    for _ in holds {
                        self.scope.pop();
                    }
                    args.push(Arg { binds, body });
                }
                return Expression::Apply(Op::Match(e), args);
            }
            1 if !reads.is_empty() => {
                let (r, name) = self.rng.pick(&reads).clone();
                let of = self.expr(&Type::Record(r), d);
                return apply(Op::Field(name), vec![of]);
            }
            // `xs[i]` of any type, at an index that may be out of range;
            // `xs.len()`.
            7 => {
                let xs = self.expr(&Type::Seq(Box::new(t.clone())), d);
                let i = match self.rng.coin() {
                    true => int(self.rng.below(5) as i64 - 1),
                    false => self.expr(&Type::Int, d),
                };
                return apply(Op::Index, vec![xs, i]);
            }
            8 if t == &Type::Int => {
                let u = match self.rng.coin() {
                    true => Type::Seq(Box::new(self.ty())),
                    false => Type::Encoded(Box::new(Type::Char)),
                };
                let xs = self.expr(&u, d);
                return apply(Op::Len, vec![xs]);
            }
            // `s[i]` of a text.
            8 if t == &Type::Byte => {
                let s = self.expr(&Type::Encoded(Box::new(Type::Char)), d);
                let i = match self.rng.coin() {
                    true => int(self.rng.below(5) as i64 - 1),
                    false => self.expr(&Type::Int, d),
                };
                return apply(Op::Index, vec![s, i]);
            }
            // `xs.decoded()`: itself.
            9 if matches!(t, Type::Seq(_)) => {
                let xs = self.expr(t, d);
                return apply(Op::Decoded, vec![xs]);
            }
            // A call of a function that gives `t`.
            4 | 5 => {
                let giving: Vec<(String, Callee)> = self
                    .calls
                    .iter()
                    .filter(|(_, c)| &c.gives == t)
                    .map(|(f, c)| (f.clone(), c.clone()))
                    .collect();
                if !giving.is_empty() {
                    let (f, callee) = self.rng.pick(&giving).clone();
                    let args = callee.takes.iter().map(|(_, u)| self.expr(u, d)).collect();
                    return apply(Op::Call(f), args);
                }
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
                4 => {
                    let u = self.ty();
                    let (a, b) = (self.expr(&u, d), self.expr(&u, d));
                    apply(Op::Equal, vec![a, b])
                }
                // On text or a sequence: `starts_with`; on text, `<`.
                5 => {
                    let u = match self.rng.coin() {
                        true => Type::Encoded(Box::new(Type::Char)),
                        false => Type::Seq(Box::new(self.ty())),
                    };
                    let (a, b) = (self.expr(&u, d), self.expr(&u, d));
                    let op = match (self.rng.below(3), &u) {
                        (0, Type::Encoded(_)) => Op::Less,
                        (1, Type::Encoded(_)) => Op::AtMost,
                        _ => Op::StartsWith,
                    };
                    apply(op, vec![a, b])
                }
                _ => {
                    let n = self.number_type();
                    let args = (0..3).map(|_| self.expr(&n, d)).collect();
                    apply(Op::Within, args)
                }
            },
            Type::Int | Type::Real => {
                // `/` on both, `%` on `Int` only (finding 11).
                let ops = [Op::Add, Op::Sub, Op::Mul, Op::Div, Op::Rem];
                let op = self
                    .rng
                    .pick(&ops[..if t == &Type::Real { 4 } else { 5 }])
                    .clone();
                let (a, b) = (self.expr(t, d), self.expr(t, d));
                apply(op, vec![a, b])
            }
            Type::Record(r) => {
                let fields = r.fields.iter().map(|(_, f)| self.expr(f, d)).collect();
                apply(Op::Make(r.clone()), fields)
            }
            Type::Param { .. } => unreachable!("no value is of a parameter"),
            Type::Enum(e) => {
                let i = self.rng.below(e.variants.len());
                let holds = e.variants[i].1.iter().map(|t| self.expr(t, d)).collect();
                apply(Op::Variant(e.clone(), i), holds)
            }
            Type::Seq(_) | Type::Encoded(_) if self.rng.below(3) == 0 => {
                let (a, b) = (self.expr(t, d), self.expr(t, d));
                apply(Op::Join, vec![a, b])
            }
            Type::Seq(u) => {
                let k = self.length();
                let holds = (0..k).map(|_| self.expr(u, d)).collect();
                apply(Op::Seq((**u).clone()), holds)
            }
            Type::Encoded(_) | Type::Byte => self.leaf(t),
            Type::Char => unreachable!("no value of a letter is written yet"),
        }
    }
    /// An end of a range: a small number, or one of two others by a test
    /// that may be uncertain, or fault.
    fn end(&mut self, depth: u32) -> E {
        let a = self.rng.below(4);
        let b = (a + 1 + self.rng.below(3)) % 4;
        let test = match self.rng.below(4) {
            0 => return int(a as i64 - 1),
            1 if self.inputs.contains_key("c") => input("c"),
            // `1 / r < 0`, which has no value where `r` is 0.
            2 => {
                let r = self.leaf(&Type::Real);
                apply(
                    Op::Less,
                    vec![apply(Op::Div, vec![real(1, 1), r]), real(0, 1)],
                )
            }
            _ => self.expr(&Type::Truth, depth.clamp(1, 2)),
        };
        apply(Op::If, vec![test, int(a as i64 - 1), int(b as i64 - 1)])
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
            Type::Int | Type::Real if self.rng.below(5) == 0 => {
                Span::Number(numbers(t), None, None)
            }
            // A half-line: one end, the other side open.
            Type::Int | Type::Real if self.rng.below(4) == 0 => {
                let end = match t {
                    Type::Int => Rational::integer(self.int()),
                    _ => self.rational(),
                };
                match self.rng.coin() {
                    true => Span::Number(numbers(t), Some(end), None),
                    false => Span::Number(numbers(t), None, Some(end)),
                }
            }
            Type::Int => {
                let low = self.int();
                let wide = match self.rng.coin() {
                    true => BigInt::from(self.rng.below(4)),
                    false => self.int().abs(),
                };
                let high = &low + wide;
                Span::Number(
                    Numbers::Int,
                    Some(Rational::integer(low)),
                    Some(Rational::integer(high)),
                )
            }
            Type::Real => {
                let low = self.rational();
                let wide = match self.rng.below(3) {
                    0 => Rational::integer(0),
                    _ => self.rational().abs(),
                };
                let high = low.plus(&wide);
                Span::Number(Numbers::Real, Some(low), Some(high))
            }
            Type::Record(r) => Span::Record(
                r.clone(),
                r.fields.iter().map(|(_, f)| self.span(f)).collect(),
            ),
            Type::Param { .. } => unreachable!("no value is of a parameter"),
            Type::Enum(e) => {
                // Each variant may be or not, at least one.
                let sure = self.rng.below(e.variants.len());
                let may = e
                    .variants
                    .iter()
                    .enumerate()
                    .map(|(i, (_, holds))| {
                        (i == sure || self.rng.coin())
                            .then(|| holds.iter().map(|t| self.span(t)).collect())
                    })
                    .collect();
                Span::Variant(e.clone(), may)
            }
            // A length from 0 or more, perhaps with no end; a span of
            // what it holds.
            Type::Seq(u) => {
                let low = self.rng.below(3) as i64;
                let high = match self.rng.below(3) {
                    0 => None,
                    _ => Some(Rational::integer(low + self.rng.below(3) as i64)),
                };
                // Places surely held, each in the span of every value.
                let each = self.span(u);
                let first = (0..self.rng.below(low as usize + 1))
                    .map(|_| each.clone())
                    .collect();
                Span::Seq(
                    (**u).clone(),
                    Box::new(Span::Number(
                        Numbers::Int,
                        Some(Rational::integer(low)),
                        high,
                    )),
                    Box::new(each),
                    first,
                )
            }
            // Text known, or of a length only.
            Type::Encoded(_) if self.rng.coin() => {
                let s = self.text();
                Span::Text(
                    Box::new(Span::Number(
                        Numbers::Int,
                        Some(Rational::integer(s.len() as i64)),
                        Some(Rational::integer(s.len() as i64)),
                    )),
                    Some(s),
                )
            }
            Type::Encoded(_) => {
                let low = self.rng.below(3) as i64;
                let high = match self.rng.below(3) {
                    0 => None,
                    _ => Some(Rational::integer(low + self.rng.below(3) as i64)),
                };
                Span::Text(
                    Box::new(Span::Number(
                        Numbers::Int,
                        Some(Rational::integer(low)),
                        high,
                    )),
                    None,
                )
            }
            Type::Byte => Span::Byte(match self.rng.coin() {
                true => Some(self.rng.below(256) as u8),
                false => None,
            }),
            Type::Char => Span::Char,
        }
    }
    /// A value in `span`: γ, sampled.
    fn member(&mut self, span: &Span, t: &Type) -> Value {
        match (span, t) {
            (Span::Truth { yes, no }, _) => Value::Truth(match (yes, no) {
                (true, true) => self.rng.coin(),
                (yes, _) => *yes,
            }),
            (Span::Number(_, None, None), t) if self.rng.below(3) > 0 => self.value(t),
            (Span::Number(_, None, None), Type::Int) => Value::Int(BigInt::zero()),
            (Span::Number(_, None, None), _) => Value::Real(Rational::integer(0)),
            // A half-line: its end, or a step past it on its open side.
            (Span::Number(_, low, high), t) if low.is_none() || high.is_none() => {
                let end = low.clone().or(high.clone()).unwrap();
                let up = low.is_some();
                let zero = Rational::integer(0);
                if self.rng.coin() && (up == (end <= zero)) {
                    // Zero lies on it: faults are there.
                    return match t {
                        Type::Int => Value::Int(BigInt::zero()),
                        _ => Value::Real(zero),
                    };
                }
                let step = match (self.rng.below(3), t) {
                    (0, _) => zero,
                    (_, Type::Int) => Rational::integer(self.int().abs()),
                    _ => self.rational().abs(),
                };
                let q = if up {
                    end.plus(&step)
                } else {
                    end.minus(&step)
                };
                match t {
                    Type::Int => Value::Int(q.numer().clone()),
                    _ => Value::Real(q),
                }
            }
            // Zero, where it lies in the span, is where faults are: take it often.
            (Span::Number(_, Some(low), Some(high)), t)
                if low <= &Rational::integer(0)
                    && &Rational::integer(0) <= high
                    && self.rng.coin() =>
            {
                match t {
                    Type::Int => Value::Int(BigInt::zero()),
                    _ => Value::Real(Rational::integer(0)),
                }
            }
            (Span::Number(_, Some(low), Some(high)), Type::Int) => {
                let (low, high) = (low.numer().clone(), high.numer().clone());
                let wide: BigInt = &high - &low + 1;
                let n = match self.rng.below(3) {
                    0 => low,
                    1 => high,
                    _ => &low + BigInt::from(self.rng.next()) % wide,
                };
                Value::Int(n)
            }
            (Span::Number(_, Some(low), Some(high)), _) => {
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
            (Span::Variant(e, may), Type::Enum(_)) => {
                let there: Vec<usize> = (0..may.len()).filter(|i| may[*i].is_some()).collect();
                let i = *self.rng.pick(&there);
                let spans = may[i].clone().unwrap();
                let holds = spans
                    .iter()
                    .zip(&e.variants[i].1)
                    .map(|(s, t)| self.member(s, t))
                    .collect();
                Value::Variant(e.clone(), i, holds)
            }
            (Span::Seq(u, length, each, _), Type::Seq(_)) => {
                let k = match self.member(length, &Type::Int) {
                    // Long ones cut to 4, past every low end here.
                    Value::Int(k) => usize::try_from(k.min(BigInt::from(4))).unwrap_or(0),
                    _ => 0,
                };
                Value::Seq(u.clone(), (0..k).map(|_| self.member(each, u)).collect())
            }
            (Span::Text(_, Some(s)), _) => Value::Text(s.clone()),
            (Span::Text(length, None), _) => {
                let k = match self.member(length, &Type::Int) {
                    Value::Int(k) => usize::try_from(k.min(BigInt::from(4))).unwrap_or(0),
                    _ => 0,
                };
                Value::Text(self.text_of(k))
            }
            (Span::Byte(Some(b)), _) => Value::Byte(*b),
            (Span::Byte(None), _) => Value::Byte(self.rng.below(256) as u8),
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
        let e = self.rng.pick(&self.enums.clone()).clone();
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
            Op::Rem,
            Op::Less,
            Op::AtMost,
            Op::Equal,
            Op::Within,
            Op::If,
            Op::Make(r),
            Op::Variant(e.clone(), self.rng.below(3)),
            Op::Match(e),
            Op::Field(name),
            Op::Let,
            Op::All,
            Op::Call(format!("lib::f{}", self.rng.below(4))),
            Op::Seq(self.ty()),
            Op::Len,
            Op::Index,
            Op::Text("é\"".to_string()),
            Op::Join,
            Op::StartsWith,
            Op::Decoded,
        ];
        self.rng.pick(&all).clone()
    }
    /// Any expression, typed or not.
    fn wild(&mut self, depth: u32) -> E {
        if depth == 0 || self.rng.below(4) == 0 {
            return match self.rng.coin() {
                true => Expression::Name(format!("x{}", self.rng.below(8))),
                false => {
                    let t = self.ty();
                    self.constant(&t)
                }
            };
        }
        if self.rng.below(6) == 0 {
            let name = format!("x{}", self.rng.below(8));
            let (value, body) = (self.wild(depth - 1), self.wild(depth - 1));
            return let_in(Op::Let, &name, value, body);
        }
        let op = self.wild_op();
        let args = (0..self.rng.below(4))
            .map(|i| {
                // The ends of a range are short, else `all` runs long.
                if op == Op::All && i < 2 {
                    return Arg {
                        binds: vec![],
                        body: int(self.rng.below(4) as i64 - 1),
                    };
                }
                let binds = (0..[0, 0, 0, 1, 2][self.rng.below(5)])
                    .map(|_| format!("x{}", self.rng.below(8)))
                    .collect();
                Arg {
                    binds,
                    body: self.wild(depth - 1),
                }
            })
            .collect();
        Expression::Apply(op, args)
    }
    /// Any value, of any shape: records with fields missing or wrong.
    fn wild_value(&mut self) -> Value {
        let t = self.ty();
        match (self.rng.below(4), t) {
            (0, Type::Record(r)) => {
                let fields = (0..self.rng.below(3)).map(|_| self.wild_value()).collect();
                Value::Record(r, fields)
            }
            (0, Type::Enum(e)) => {
                let holds = (0..self.rng.below(3)).map(|_| self.wild_value()).collect();
                Value::Variant(e, self.rng.below(4), holds)
            }
            (0, Type::Seq(u)) => {
                let holds = (0..self.rng.below(3)).map(|_| self.wild_value()).collect();
                Value::Seq(*u, holds)
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
            1 => Span::Number(
                Numbers::Real,
                Some(Rational::integer(2)),
                Some(Rational::new(1, 2).unwrap()),
            ),
            2 => match t {
                Type::Record(r) => Span::Record(r, vec![]),
                // No variant may be, or one holds too few.
                Type::Enum(e) => match self.rng.coin() {
                    true => Span::Variant(e.clone(), vec![None; e.variants.len()]),
                    false => Span::Variant(e, vec![Some(vec![])]),
                },
                // A length below zero, or of reals.
                Type::Seq(u) => Span::Seq(
                    *u.clone(),
                    Box::new(match self.rng.coin() {
                        true => Span::Number(Numbers::Int, Some(Rational::integer(-2)), None),
                        false => Span::Number(Numbers::Real, None, None),
                    }),
                    Box::new(self.span(&u)),
                    vec![],
                ),
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
        prop_assert_eq!(check(&g.standard(), &g.inputs, &e), Some(t.clone()));
        let values = g.values();
        prop_assert!(fits(&g.exact(), &g.inputs, &values));
        let out = eval(&g.exact(), &values, &e);
        prop_assert!(progress(&g.exact(), &out, &t), "{:?} gave {:?}", e, out);
        prop_assert_eq!(out, eval(&g.exact(), &values, &e));
    }

    /// S1, S2, S4 on the reading by bounds.
    #[test]
    fn ranges_preserve_progress_and_are_determined(seed in any::<u64>(), depth in 1u32..6) {
        let mut g = Gen::new(seed);
        let t = g.ty();
        let e = g.expr(&t, depth);
        let (bounds, _) = g.bounded();
        prop_assert!(fits(&g.ranges(), &g.inputs, &bounds));
        let out = eval(&g.ranges(), &bounds, &e);
        prop_assert!(progress(&g.ranges(), &out, &t), "{:?} gave {:?}", e, out);
        prop_assert_eq!(out, eval(&g.ranges(), &bounds, &e));
    }

    /// S6: a let shares. Each reading gives what the expression with
    /// every let written out gives, and that checks to the same type.
    #[test]
    fn a_let_shares(seed in any::<u64>(), depth in 1u32..6) {
        let mut g = Gen::new(seed);
        let t = g.ty();
        let e = g.expr(&t, depth);
        let written = inline(&e, &is_let);
        prop_assert_eq!(check(&g.standard(), &g.inputs, &written), Some(t.clone()));
        let (bounds, exact) = g.bounded();
        prop_assert_eq!(eval(&g.exact(), &exact, &e), eval(&g.exact(), &exact, &written));
        prop_assert_eq!(eval(&g.ranges(), &bounds, &e), eval(&g.ranges(), &bounds, &written));
    }

    /// S6 on any expression, checked or not.
    #[test]
    fn a_let_shares_anything(seed in any::<u64>(), depth in 0u32..6) {
        let mut g = Gen::new(seed);
        let e = g.wild(depth);
        let written = inline(&e, &is_let);
        if let Some(t) = check(&g.standard(), &g.inputs, &e) {
            prop_assert_eq!(check(&g.standard(), &g.inputs, &written), Some(t));
        }
        let values: BTreeMap<String, Value> = (0..8).map(|i| (format!("x{i}"), g.wild_value())).collect();
        let bounds: BTreeMap<String, Bounds> = (0..8).map(|i| (format!("x{i}"), g.wild_bounds())).collect();
        prop_assert_eq!(eval(&g.exact(), &values, &e), eval(&g.exact(), &values, &written));
        prop_assert_eq!(eval(&g.ranges(), &bounds, &e), eval(&g.ranges(), &bounds, &written));
    }

    /// S5: every exact outcome lies in the bounded one, when each exact
    /// input lies in its bounds. Depth 1 is each operation alone.
    #[test]
    fn exact_lies_in_ranges(seed in any::<u64>(), depth in 1u32..6) {
        let mut g = Gen::new(seed);
        let t = g.ty();
        let e = g.expr(&t, depth);
        let (bounds, exact) = g.bounded();
        prop_assert!(fits(&g.ranges(), &g.inputs, &bounds));
        for (name, value) in &exact {
            let (b, x) = (Ok(bounds[name].clone()), Ok(value.clone()));
            prop_assert!(covers(&g.inputs[name], &b, &x));
        }
        let (b, x) = (eval(&g.ranges(), &bounds, &e), eval(&g.exact(), &exact, &e));
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
        if args.iter().any(|a| !a.binds.is_empty()) {
            return Ok(());
        }
        let takes: Vec<Type> = args.iter().map(|a| check(&g.standard(), &g.inputs, &a.body).unwrap()).collect();
        prop_assert_eq!(g.standard().gives(&op, &takes), Some(t.clone()));
        let (bounds, exact): (Vec<_>, Vec<_>) = takes.iter().map(|ty| g.outcome(ty)).unzip();
        for ((ty, b), x) in takes.iter().zip(&bounds).zip(&exact) {
            prop_assert!(b.as_ref().map_or(true, |b| g.ranges().has_type(b, ty)));
            prop_assert!(covers(ty, b, x));
        }
        let (b, x) = (g.ranges().apply(&op, &bounds), g.exact().apply(&op, &exact));
        prop_assert!(covers(&t, &b, &x), "{:?} on {:?}\nbounds {:?}\nexact {:?}", op, takes, b, x);
    }

    /// S5 on points: when every input's bounds are its exact value, the
    /// bounds of the outcome are the exact outcome, or its fault. Bounds
    /// that said "anything, may fault" everywhere would cover, and fail
    /// here. Every function called has a body: of one without, `Exact`
    /// has no value and `Ranges` knows only the type.
    #[test]
    fn points_are_exact(seed in any::<u64>(), depth in 1u32..6) {
        let mut g = Gen::library(seed, false);
        let t = g.ty();
        let e = g.expr(&t, depth);
        let exact = g.values();
        let points: BTreeMap<_, _> = exact.iter().map(|(n, v)| (n.clone(), Bounds::exactly(v))).collect();
        let (b, x) = (eval(&g.ranges(), &points, &e), eval(&g.exact(), &exact, &e));
        prop_assert!(covers(&t, &b, &x), "{:?}\nbounds {:?}\nexact {:?}", e, b, x);
        prop_assert_eq!(&b, &x.clone().map(|v| Bounds::exactly(&v)), "{:?}", e);
    }

    /// S5 for `all`, over ends that may be uncertain, or fault, and a
    /// body that reads `x`, which may shadow an input.
    #[test]
    fn all_covers(seed in any::<u64>(), depth in 0u32..4) {
        let mut g = Gen::new(seed);
        for end in ["lo", "hi"] {
            g.inputs.insert(end.to_string(), Type::Int);
        }
        let x = g.rng.pick(&["i", "lo", "x0"]).to_string();
        g.scope.push((x.clone(), Type::Int));
        let p = g.expr(&Type::Truth, depth);
        g.scope.pop();
        let e = all(input("lo"), input("hi"), &x, p);
        prop_assert_eq!(check(&g.standard(), &g.inputs, &e), Some(Type::Truth));
        let (mut bounds, mut exact) = g.bounded();
        for end in ["lo", "hi"] {
            let low = g.rng.below(5) as i64 - 2;
            let high = low + g.rng.below(3) as i64;
            let mut b = Bounds::between(Numbers::Int, Rational::integer(low), Rational::integer(high)).unwrap();
            b.may_fault = g.rng.below(4) == 0;
            exact.insert(end.to_string(), Value::Int(BigInt::from(low + g.rng.below((high - low + 1) as usize) as i64)));
            bounds.insert(end.to_string(), b);
        }
        let (b, v) = (eval(&g.ranges(), &bounds, &e), eval(&g.exact(), &exact, &e));
        prop_assert!(covers(&Type::Truth, &b, &v), "{:?}\nbounds {:?}\nexact {:?}\nat {:?} in {:?}", e, b, v, exact, bounds);
    }

    /// S3: no expression and no values make `check` or `eval` panic.
    #[test]
    fn nothing_panics(seed in any::<u64>(), depth in 0u32..6) {
        let mut g = Gen::new(seed);
        let e = g.wild(depth);
        let values: BTreeMap<_, _> = (0..8).map(|i| (format!("x{i}"), g.wild_value())).collect();
        let bounds: BTreeMap<_, _> = (0..8).map(|i| (format!("x{i}"), g.wild_bounds())).collect();
        let _ = check(&g.standard(), &g.inputs, &e);
        let _ = eval(&g.exact(), &values, &e);
        let _ = eval(&g.ranges(), &bounds, &e);
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
    assert_eq!(
        eval(&Exact::default(), &BTreeMap::new(), &e),
        Ok(Value::Truth(false))
    );
    assert_eq!(
        eval(&Ranges::default(), &BTreeMap::new(), &e)
            .unwrap()
            .surely(),
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
        eval(&Exact::default(), &BTreeMap::new(), &e),
        Ok(Value::Real(sum.clone()))
    );
    let b = eval(&Ranges::default(), &BTreeMap::new(), &e).unwrap();
    assert_eq!(
        b.span,
        Span::Number(Numbers::Real, Some(sum.clone()), Some(sum))
    );
    assert!(!b.may_fault);
}

/// foundation2: a truth was accepted as an Int, and a record named
/// "rational" could pass for a real. Value typing reads the value.
#[test]
fn a_value_is_of_its_type_only() {
    let fake = record("rational", &[("n", Type::Int), ("d", Type::Int)]);
    let half = Value::Record(
        fake.clone(),
        vec![Value::Int(1.into()), Value::Int(2.into())],
    );
    assert!(!Exact::default().has_type(&Value::Truth(true), &Type::Int));
    assert!(!Exact::default().has_type(&half, &Type::Real));
    assert!(Exact::default().has_type(&half, &Type::Record(fake.clone())));
    // A record of the right name with fields of the wrong type is not one.
    let point = record("Point", &[("x", Type::Real), ("y", Type::Real)]);
    let wrong = Value::Record(
        point.clone(),
        vec![Value::Int(1.into()), Value::Int(2.into())],
    );
    assert!(!Exact::default().has_type(&wrong, &Type::Record(point)));
    // `+` does not take two of the fake.
    let inputs = typed(&[("a", Type::Record(fake.clone())), ("b", Type::Record(fake))]);
    assert_eq!(
        check(
            &Simulation::default(),
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
    assert_eq!(check(&Simulation::default(), &inputs, &e), Some(Type::Int));
    let e = apply(Op::Add, vec![input("a"), real(1, 2)]);
    assert_eq!(check(&Simulation::default(), &inputs, &e), None);
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
        eval(&Exact::default(), &at_zero, &inverse),
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
    assert_eq!(
        eval(&Exact::default(), &at_zero, &guarded),
        Ok(Value::Truth(false))
    );
    let unguarded = apply(Op::Div, vec![real(1, 1), x()]);
    assert_eq!(
        eval(&Exact::default(), &at_zero, &unguarded),
        Err(Fault::Op(mitra_lang::simulation::Undefined::DivisionByZero))
    );
    // By bounds, x in [-1, 1]: the inverse may fault, the guarded `if` may too
    // (it may divide near 0), and both are sound.
    let around = BTreeMap::from([(
        "x".to_string(),
        Bounds::between(Numbers::Real, Rational::integer(-1), Rational::integer(1)).unwrap(),
    )]);
    assert!(
        eval(&Ranges::default(), &around, &unguarded)
            .unwrap()
            .may_fault
    );
    assert!(covers(
        &Type::Real,
        &eval(&Ranges::default(), &around, &inverse),
        &eval(&Exact::default(), &at_zero, &inverse)
    ));
}

/// A claim proved over a whole domain: for every h in [0, 20],
/// 2 · 9.81 · h < 400.
#[test]
fn a_claim_holds_over_a_domain() {
    let h = BTreeMap::from([(
        "h".to_string(),
        Bounds::between(Numbers::Real, Rational::integer(0), Rational::integer(20)).unwrap(),
    )]);
    let product = apply(
        Op::Mul,
        vec![apply(Op::Mul, vec![real(2, 1), real(981, 100)]), input("h")],
    );
    let low = apply(Op::Less, vec![product.clone(), real(400, 1)]);
    let inputs = typed(&[("h", Type::Real)]);
    assert_eq!(
        check(&Simulation::default(), &inputs, &low),
        Some(Type::Truth)
    );
    assert_eq!(
        eval(&Ranges::default(), &h, &low).unwrap().surely(),
        Some(true)
    );
    let too_low = apply(Op::Less, vec![product, real(300, 1)]);
    assert_eq!(
        eval(&Ranges::default(), &h, &too_low).unwrap().surely(),
        None
    );
}

/// Two fields of one name: `r.x` is the first, in type and in value.
#[test]
fn the_first_field_of_a_name() {
    let dup = record("Dup", &[("x", Type::Int), ("x", Type::Truth)]);
    let made = apply(Op::Make(dup), vec![int(7), apply(Op::Truth(true), vec![])]);
    let read = apply(Op::Field("x".to_string()), vec![made]);
    assert_eq!(
        check(&Simulation::default(), &BTreeMap::new(), &read),
        Some(Type::Int)
    );
    assert_eq!(
        eval(&Exact::default(), &BTreeMap::new(), &read),
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
    let unit = |of| Ok(Bounds::between(of, Rational::integer(0), Rational::integer(1)).unwrap());
    let (ints, reals) = (unit(Numbers::Int), unit(Numbers::Real));
    assert!(covers(&Type::Int, &ints, &Ok(Value::Int(BigInt::from(1)))));
    assert!(!covers(&Type::Int, &ints, &Ok(Value::Real(half.clone()))));
    assert!(covers(&Type::Real, &reals, &Ok(Value::Real(half.clone()))));
    // Since finding 11 a span says its numbers: a whole number is in no
    // span of reals, nor a real in a span of whole numbers.
    assert!(!covers(&Type::Real, &ints, &Ok(Value::Real(half.clone()))));
    assert!(!lies_in(
        &Value::Int(BigInt::from(1)),
        &reals.as_ref().unwrap().span
    ));
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
    let fractional = Bounds::between(Numbers::Real, half, Rational::new(3, 2).unwrap()).unwrap();
    assert!(!Ranges::default().has_type(&fractional, &Type::Int));
    assert!(Ranges::default().has_type(&fractional, &Type::Real));
    let x = BTreeMap::from([("x".to_string(), Type::Int)]);
    let bounds = BTreeMap::from([("x".to_string(), fractional)]);
    assert!(!fits(&Ranges::default(), &x, &bounds));
}

/// `let q = 1 / x; if x != 0 { q } else { 0 }` has a value at 0: the
/// fault of what a let binds counts only where it is read. Rust would
/// fault at the let. By bounds, over [-1, 1], it may fault.
#[test]
fn a_shared_division() {
    let x = || input("x");
    let zero = || real(0, 1);
    let nonzero = apply(Op::Not, vec![apply(Op::Equal, vec![x(), zero()])]);
    let e = let_in(
        Op::Let,
        "q",
        apply(Op::Div, vec![real(1, 1), x()]),
        apply(Op::If, vec![nonzero, input("q"), zero()]),
    );
    let reals = typed(&[("x", Type::Real)]);
    assert_eq!(check(&Simulation::default(), &reals, &e), Some(Type::Real));
    let value = |x: i64| {
        eval(
            &Exact::default(),
            &at(&[("x", Value::Real(Rational::integer(x)))]),
            &e,
        )
    };
    assert_eq!(value(0), Ok(Value::Real(Rational::integer(0))));
    assert_eq!(value(2), Ok(Value::Real(Rational::new(1, 2).unwrap())));
    let around = BTreeMap::from([(
        "x".to_string(),
        Bounds::between(Numbers::Real, Rational::integer(-1), Rational::integer(1)).unwrap(),
    )]);
    assert!(eval(&Ranges::default(), &around, &e).unwrap().may_fault);
}

/// S3's bound on depth, on a thread of 2 MiB: 1,000 levels of `if`, of
/// `let`s in value position, in body position, and of `if` and `let` in
/// turn; and 333 of `all`, each in the body of the one before, as each
/// counts as three. Checked, read by both readings, cloned, compared and
/// dropped.
/// `(lo..hi).all(|i| i < k)`, and with `<=` and `==`, on ends that are
/// numbers: by bounds `i` is each number in turn, so nothing is lost.
#[test]
fn all_on_numbers_reads_each() {
    let none: BTreeMap<String, Bounds> = BTreeMap::new();
    for lo in -1..3 {
        for hi in -1..4 {
            for k in -1..4 {
                for op in [Op::Less, Op::AtMost, Op::Equal] {
                    let p = apply(op, vec![input("i"), int(k)]);
                    let e = all(int(lo), int(hi), "i", p);
                    let exact = eval(&Exact::default(), &BTreeMap::new(), &e);
                    let exactly = exact.map(|v| Bounds::exactly(&v));
                    assert_eq!(eval(&Ranges::default(), &none, &e), exactly, "{e:?}");
                }
            }
        }
    }
}

/// `(0..10^15).all(|i| c)` by bounds, `c` false or a fault: the outcome
/// is fixed at the first number, so no other is read.
#[test]
fn all_stops_once_its_outcome_is_fixed() {
    let big = apply(Op::Int(BigInt::from(10u64.pow(15))), vec![]);
    let e = all(int(0), big, "i", input("c"));
    let c = Bounds {
        span: Span::Truth {
            yes: false,
            no: true,
        },
        may_fault: true,
    };
    let bounds = BTreeMap::from([("c".to_string(), c.clone())]);
    assert_eq!(eval(&Ranges::default(), &bounds, &e), Ok(c));
}

#[test]
fn a_thousand_levels() {
    // S3: 1,000 levels in a debug build, 1,800 in a release build: the
    // promise is the depth budget, 1,000 (finding 31: 2,000 overflowed
    // when unrelated code moved the layout; the user's choice, 11:16).
    let levels = if cfg!(debug_assertions) { 1000 } else { 1800 };
    let deep = |start: E, shape: fn(usize, E) -> E, levels: usize, of: Type, gives: Value| {
        std::thread::Builder::new()
            .stack_size(2 << 20)
            .spawn(move || {
                let mut e = start;
                for level in 0..levels {
                    e = shape(level, e);
                }
                let x = typed(&[("x", Type::Int)]);
                let value = at(&[("x", Value::Int(BigInt::from(7)))]);
                let bounds = BTreeMap::from([("x".to_string(), Bounds::exactly(&value["x"]))]);
                let same = e.clone() == e;
                // `x := y`, which renames every `y` an argument binds
                // around an `x`.
                let written = substitute(&e, "x", &input("y"));
                let y = typed(&[("y", Type::Int)]);
                let at_y = at(&[("y", Value::Int(BigInt::from(7)))]);
                (
                    same && check(&Simulation::default(), &x, &e) == Some(of.clone()),
                    eval(&Exact::default(), &value, &e) == Ok(gives.clone()),
                    eval(&Ranges::default(), &bounds, &e) == Ok(Bounds::exactly(&gives)),
                    check(&Simulation::default(), &y, &written) == Some(of)
                        && eval(&Exact::default(), &at_y, &written) == Ok(gives),
                )
            })
            .unwrap()
            .join()
            .unwrap()
    };
    let seven = Value::Int(BigInt::from(7));
    let shapes: [fn(usize, E) -> E; 5] = [
        |_, e| apply(Op::If, vec![apply(Op::Truth(true), vec![]), e, int(0)]),
        |_, e| let_in(Op::Let, "y", e, input("y")),
        |level, e| match level % 2 {
            0 => apply(Op::If, vec![apply(Op::Truth(true), vec![]), e, int(0)]),
            _ => let_in(Op::Let, "y", e, input("y")),
        },
        // A chain: `let x = x + 0; let x = x + 0; …; x`.
        |_, e| let_in(Op::Let, "x", apply(Op::Add, vec![input("x"), int(0)]), e),
        // `let y = 0; let y = 0; …; x`: each `y` renamed for `x := y`.
        |_, e| let_in(Op::Let, "y", int(0), e),
    ];
    for shape in shapes {
        let got = deep(input("x"), shape, levels, Type::Int, seven.clone());
        assert_eq!(got, (true, true, true, true));
    }
    // `(0..1).all(|i| (0..1).all(|i| … x == 7))`, each `all` three levels.
    let start = apply(Op::Equal, vec![input("x"), int(7)]);
    let nested = |_: usize, e: E| all(int(0), int(1), "i", e);
    let got = deep(start, nested, levels / 3, Type::Truth, Value::Truth(true));
    assert_eq!(got, (true, true, true, true));
}

/// S3 through calls: a chain of functions, each calling the next, each
/// call two levels. The large frame of a reading is not held while a
/// callee's body runs (red team round 7: debug overflowed at 435).
#[test]
fn a_thousand_levels_of_calls() {
    let levels = if cfg!(debug_assertions) { 1000 } else { 2000 };
    let name = |i: usize| format!("c::f{i:06}");
    // `f(a) = g(a)`, two levels; `f(a) = { let b = g(a); b }`, three.
    type Shape = (usize, fn(E) -> E);
    let shapes: [Shape; 2] = [
        (2, |next| next),
        (3, |next| let_in(Op::Let, "b", next, input("b"))),
    ];
    for (per, shape) in shapes {
        let n = levels / per;
        let chain = (0..n).map(|i| {
            let body = match i + 1 == n {
                true => input("a"),
                false => shape(call(&name(i + 1), vec![input("a")])),
            };
            (name(i), callee(&[("a", Type::Int)], Type::Int, Some(body)))
        });
        // Past the default depth budget in release: this test is of the
        // stack, not of the budget.
        let budget = mitra_lang::simulation::Budget {
            depth: levels + 1,
            ..mitra_lang::simulation::Budget::default()
        };
        let calls = Arc::new(Calls::with(chain.collect()).unwrap().budget(budget));
        let e = call(&name(0), vec![int(7)]);
        let seven = Value::Int(BigInt::from(7));
        let got = std::thread::Builder::new()
            .stack_size(2 << 20)
            .spawn(move || {
                let standard = Simulation {
                    calls: calls.clone(),
                };
                (
                    check(&standard, &BTreeMap::new(), &e) == Some(Type::Int),
                    eval(
                        &Exact {
                            calls: calls.clone(),
                            log: Default::default(),
                        },
                        &BTreeMap::new(),
                        &e,
                    ) == Ok(seven.clone()),
                    eval(&Ranges { calls }, &BTreeMap::new(), &e) == Ok(Bounds::exactly(&seven)),
                )
            })
            .unwrap()
            .join()
            .unwrap();
        assert_eq!(got, (true, true, true), "{per} levels a call");
    }
}

/// Refusing what calls one refused is one walk back (red team round 7:
/// it was quadratic, 8,000 functions in 4 s).
#[test]
fn a_long_chain_is_refused_quickly() {
    let n = 20_000;
    let name = |i: usize| format!("c::f{i:06}");
    let chain = (0..n).map(|i| {
        let body = match i + 1 == n {
            true => apply(Op::Truth(true), vec![]),
            false => call(&name(i + 1), vec![]),
        };
        (name(i), callee(&[], Type::Int, Some(body)))
    });
    let start = std::time::Instant::now();
    let refused = Calls::with(chain.collect()).unwrap_err();
    assert_eq!(refused.len(), n);
    assert!(
        start.elapsed() < std::time::Duration::from_secs(2),
        "{:?}",
        start.elapsed()
    );
}

/// Rows are not compared, here: beside a body they are never read, and
/// of two at one point the first is (`agree` compares them).
#[test]
fn rows_are_read_only_without_a_body() {
    let n = |k: i64| Value::Int(k.into());
    let plus_one = Some(apply(Op::Add, vec![input("a"), int(1)]));
    let both = Callee {
        promises: None,
        rows: vec![(vec![n(1)], n(5))],
        ..callee(&[("a", Type::Int)], Type::Int, plus_one)
    };
    let twice = Callee {
        promises: None,
        rows: vec![(vec![n(1)], n(2)), (vec![n(1)], n(3))],
        ..callee(&[("a", Type::Int)], Type::Int, None)
    };
    // Two rows at one point that differ: refused, as a body that does
    // not check is. The same row twice is one value.
    assert_eq!(library(vec![("g", twice)]), Err(names(&["g"])));
    let same = Callee {
        promises: None,
        rows: vec![(vec![n(1)], n(2)), (vec![n(1)], n(2))],
        ..callee(&[("a", Type::Int)], Type::Int, None)
    };
    let calls = Arc::new(library(vec![("f", both), ("g", same)]).unwrap());
    let exact = Exact {
        calls,
        log: Default::default(),
    };
    let at_one = |f: &str| eval(&exact, &BTreeMap::new(), &call(f, vec![int(1)]));
    assert_eq!(at_one("f"), Ok(n(2)));
    assert_eq!(at_one("g"), Ok(n(2)));
    // What a reading is given comes before the rows of a function with
    // no body; a body, a value of another type or an unknown function
    // takes none.
    let opaque = Callee {
        promises: None,
        rows: vec![(vec![n(1)], n(2))],
        ..callee(&[("a", Type::Int)], Type::Int, None)
    };
    let with_body = callee(
        &[("a", Type::Int)],
        Type::Int,
        Some(apply(Op::Add, vec![input("a"), int(1)])),
    );
    let lib = library(vec![("g", opaque), ("f", with_body)]).unwrap();
    assert!(lib.clone().given("f", vec![n(1)], n(7)).is_err());
    assert!(lib
        .clone()
        .given("g", vec![n(1)], Value::Truth(true))
        .is_err());
    assert!(lib.clone().given("g", vec![], n(7)).is_err());
    assert!(lib.clone().given("h", vec![n(1)], n(7)).is_err());
    let exact = Exact {
        calls: Arc::new(lib.given("g", vec![n(1)], n(7)).unwrap()),
        log: Default::default(),
    };
    assert_eq!(
        eval(&exact, &BTreeMap::new(), &call("g", vec![int(1)])),
        Ok(n(7))
    );
    assert_eq!(
        eval(&exact, &BTreeMap::new(), &call("f", vec![int(1)])),
        Ok(n(2))
    );
}

fn call(f: &str, args: Vec<E>) -> E {
    apply(Op::Call(f.to_string()), args)
}
fn callee(takes: &[(&str, Type)], gives: Type, body: Option<E>) -> Callee {
    Callee {
        promises: None,
        takes: takes
            .iter()
            .map(|(n, t)| (n.to_string(), t.clone()))
            .collect(),
        gives,
        body,
        rows: vec![],
        ranges: Default::default(),
    }
}
fn library(callees: Vec<(&str, Callee)>) -> Result<Calls, std::collections::BTreeSet<String>> {
    Calls::with(
        callees
            .into_iter()
            .map(|(n, c)| (n.to_string(), c))
            .collect(),
    )
}
fn names(these: &[&str]) -> std::collections::BTreeSet<String> {
    these.iter().map(|n| n.to_string()).collect()
}
fn param(name: &str, bound: Bound) -> Type {
    Type::Param {
        name: name.to_string(),
        bound,
    }
}

/// `use`: a function of another part, called by its path. The body
/// runs; or a row gives the value; with neither there is none, by name.
/// Ranges runs the body, or allows any value of the type.
#[test]
fn calls_walked_through() {
    let double = callee(
        &[("a", Type::Int)],
        Type::Int,
        Some(apply(Op::Add, vec![input("a"), input("a")])),
    );
    let quad = callee(
        &[("a", Type::Int)],
        Type::Int,
        Some(call(
            "lib::double",
            vec![call("lib::double", vec![input("a")])],
        )),
    );
    let light = Callee {
        promises: None,
        rows: vec![(vec![], Value::Int(850.into()))],
        ..callee(&[], Type::Int, None)
    };
    let square = Callee {
        promises: None,
        rows: vec![(vec![Value::Int(2.into())], Value::Int(4.into()))],
        ..callee(&[("a", Type::Int)], Type::Int, None)
    };
    let half = callee(&[("r", Type::Real)], Type::Real, None);
    let one = callee(&[("r", Type::Real)], Type::Int, Some(int(1)));
    let calls = Arc::new(
        library(vec![
            ("lib::double", double),
            ("lib::quad", quad),
            ("sun::light", light),
            ("lib::square", square),
            ("lib::half", half),
            ("lib::one", one),
        ])
        .unwrap(),
    );
    let (standard, exact, ranges) = (
        Simulation {
            calls: calls.clone(),
        },
        Exact {
            calls: calls.clone(),
            log: Default::default(),
        },
        Ranges {
            calls: calls.clone(),
        },
    );
    let ints = typed(&[("a", Type::Int)]);
    let reals = typed(&[("r", Type::Real)]);
    let at_a = |n: i64| at(&[("a", Value::Int(n.into()))]);
    let int_value = |n: i64| Ok(Value::Int(n.into()));
    let between = |lo: i64, hi: i64| {
        Bounds::between(Numbers::Int, Rational::integer(lo), Rational::integer(hi)).unwrap()
    };

    // A body, through another call.
    let e = call("lib::quad", vec![input("a")]);
    assert_eq!(check(&standard, &ints, &e), Some(Type::Int));
    assert_eq!(eval(&exact, &at_a(3), &e), int_value(12));
    let b = BTreeMap::from([("a".to_string(), between(1, 2))]);
    assert_eq!(eval(&ranges, &b, &e), Ok(between(4, 8)));

    // Rows: a value where a row is, none elsewhere, named.
    let e = call("sun::light", vec![]);
    assert_eq!(check(&standard, &ints, &e), Some(Type::Int));
    assert_eq!(eval(&exact, &at_a(0), &e), int_value(850));
    let unknown = |f: &str| Err(Fault::Op(Undefined::Unknown(f.to_string())));
    let e = call("lib::square", vec![input("a")]);
    assert_eq!(eval(&exact, &at_a(2), &e), int_value(4));
    assert_eq!(eval(&exact, &at_a(3), &e), unknown("lib::square"));
    // By bounds, a function with no body gives a value of the world:
    // any, and not knowing it is no fault (finding 8).
    let any_int = Ok(Bounds::any(&Type::Int));
    let b = BTreeMap::from([("a".to_string(), between(2, 2))]);
    assert_eq!(eval(&ranges, &b, &e), any_int);

    // Opaque: none at all, and any real by bounds.
    let e = call("lib::half", vec![input("r")]);
    assert_eq!(check(&standard, &reals, &e), Some(Type::Real));
    let r = at(&[("r", Value::Real(Rational::integer(1)))]);
    assert_eq!(eval(&exact, &r, &e), unknown("lib::half"));
    let b = BTreeMap::from([("r".to_string(), Bounds::exactly(&r["r"]))]);
    assert_eq!(eval(&ranges, &b, &e), Ok(Bounds::any(&Type::Real)));

    // Strict, as Rust: a fault in an argument is the call's, read or not.
    let e = call(
        "lib::one",
        vec![apply(Op::Div, vec![real(1, 1), input("r")])],
    );
    let zero = at(&[("r", Value::Real(Rational::integer(0)))]);
    assert_eq!(
        eval(&exact, &zero, &e),
        Err(Fault::Op(Undefined::DivisionByZero))
    );
    assert_eq!(eval(&exact, &r, &e), int_value(1));
    let b = BTreeMap::from([(
        "r".to_string(),
        Bounds::between(Numbers::Real, Rational::integer(-1), Rational::integer(1)).unwrap(),
    )]);
    let got = eval(&ranges, &b, &e).unwrap();
    assert_eq!((got.span, got.may_fault), (between(1, 1).span, true));

    // Typing: the callee's signature, nothing else.
    for wrong in [
        call("lib::double", vec![]),
        call("lib::double", vec![real(1, 2)]),
        call("lib::double", vec![int(1), int(2)]),
        call("lib::nowhere", vec![int(1)]),
        call("lib::double", vec![input("r")]),
    ] {
        assert_eq!(check(&standard, &ints, &wrong), None, "{wrong:?}");
    }
    // A standard with no functions knows none of them.
    let e = call("lib::double", vec![int(1)]);
    assert_eq!(check(&Simulation::default(), &ints, &e), None);
    // Run unchecked, an unknown function or a wrong count is stuck.
    assert_eq!(eval(&Exact::default(), &at_a(0), &e), Err(Fault::Stuck));
    assert_eq!(
        eval(&Ranges::default(), &BTreeMap::new(), &e),
        Err(Fault::Stuck)
    );
    for e in [
        call("lib::double", vec![]),
        call("lib::double", vec![int(1), int(2)]),
        call("lib::square", vec![]),
        call("lib::square", vec![int(1), int(2)]),
    ] {
        assert_eq!(eval(&exact, &at_a(0), &e), Err(Fault::Stuck), "{e:?}");
        assert_eq!(
            eval(&ranges, &BTreeMap::new(), &e),
            Err(Fault::Stuck),
            "{e:?}"
        );
    }
}

/// Calls never go round, and every function is checked once, when the
/// library is made: `Calls::with` names each it refuses.
#[test]
fn calls_go_nowhere_round() {
    let calling = |f: &str| {
        callee(
            &[("a", Type::Int)],
            Type::Int,
            Some(call(f, vec![input("a")])),
        )
    };
    let fine = || callee(&[("a", Type::Int)], Type::Int, Some(input("a")));
    // Itself.
    assert_eq!(library(vec![("f", calling("f"))]), Err(names(&["f"])));
    // Round two, and whatever calls into them, however far.
    let got = library(vec![
        ("f", calling("g")),
        ("g", calling("f")),
        ("h", calling("f")),
        ("k", calling("h")),
        ("ok", fine()),
    ]);
    assert_eq!(got, Err(names(&["f", "g", "h", "k"])));
    // A function no library has.
    assert_eq!(
        library(vec![("f", calling("missing")), ("ok", fine())]),
        Err(names(&["f"]))
    );
    // A body not of the type given, or reading what is not taken.
    let wrong = callee(
        &[("a", Type::Int)],
        Type::Int,
        Some(apply(Op::Truth(true), vec![])),
    );
    let reads = callee(&[("a", Type::Int)], Type::Int, Some(input("b")));
    assert_eq!(
        library(vec![("f", wrong), ("g", reads), ("h", calling("g"))]),
        Err(names(&["f", "g", "h"]))
    );
    // Two arguments of one name.
    let twice = callee(
        &[("a", Type::Int), ("a", Type::Int)],
        Type::Int,
        Some(input("a")),
    );
    assert_eq!(library(vec![("f", twice)]), Err(names(&["f"])));
    // With rows, or bare, too (red team round 7).
    let twice = || callee(&[("a", Type::Int), ("a", Type::Int)], Type::Int, None);
    let one = || Value::Int(1.into());
    let rows = Callee {
        promises: None,
        rows: vec![(vec![one(), one()], one())],
        ..twice()
    };
    assert_eq!(library(vec![("f", rows)]), Err(names(&["f"])));
    assert_eq!(library(vec![("f", twice())]), Err(names(&["f"])));
    // Rows of the wrong count or type.
    let row = |at: Vec<Value>, gives: Value| Callee {
        promises: None,
        rows: vec![(at, gives)],
        ..callee(&[("a", Type::Int)], Type::Int, None)
    };
    for bad in [
        row(vec![], one()),
        row(vec![one(), one()], one()),
        row(vec![Value::Truth(true)], one()),
        row(vec![one()], Value::Truth(true)),
    ] {
        assert_eq!(library(vec![("f", bad)]), Err(names(&["f"])));
    }
    assert!(library(vec![("f", row(vec![one()], one()))]).is_ok());
    // A parameter: only bare, its contract says the rest.
    let t = param("T", Bound::Any);
    let open = |body: Option<E>, rows: Vec<(Vec<Value>, Value)>| Callee {
        promises: None,
        rows,
        ..callee(&[("x", t.clone())], t.clone(), body)
    };
    assert_eq!(
        library(vec![("f", open(Some(input("x")), vec![]))]),
        Err(names(&["f"]))
    );
    let takes_open = callee(&[("x", t.clone())], Type::Int, Some(int(1)));
    assert_eq!(library(vec![("f", takes_open)]), Err(names(&["f"])));
    let point = record("P", &[("x", t.clone())]);
    let half_open = Callee {
        promises: None,
        rows: vec![(vec![one()], one())],
        ..callee(&[("a", Type::Int)], Type::Record(point), None)
    };
    assert_eq!(library(vec![("f", half_open)]), Err(names(&["f"])));
    // A library that holds: each function found by its path.
    let calls = library(vec![
        ("f", calling("g")),
        ("g", fine()),
        ("t", open(None, vec![])),
    ])
    .unwrap();
    assert_eq!(calls.get("g"), Some(&fine()));
    assert_eq!(calls.get("nowhere"), None);
    assert_eq!(calls.iter().count(), 3);
    assert!(library(vec![]).is_ok());
}

/// Type parameters, Rust's `T` and `T: Number`: what a contract can
/// write about a type it does not fix. No value is of one.
#[test]
fn type_parameters() {
    let (t, n, m) = (
        param("T", Bound::Any),
        param("N", Bound::Number),
        param("M", Bound::Number),
    );
    let inputs = typed(&[
        ("x", n.clone()),
        ("y", n.clone()),
        ("u", m.clone()),
        ("a", t.clone()),
        ("b", t.clone()),
    ]);
    let two = |op: Op, a: &str, b: &str| apply(op, vec![input(a), input(b)]);
    let standard = Simulation::default();
    let typed_as = |e: &E| check(&standard, &inputs, e);
    // A number: + - *, < <=, within and ==; no /, no literal.
    for op in [Op::Add, Op::Sub, Op::Mul] {
        assert_eq!(typed_as(&two(op, "x", "y")), Some(n.clone()));
    }
    for op in [Op::Less, Op::AtMost, Op::Equal] {
        assert_eq!(typed_as(&two(op, "x", "y")), Some(Type::Truth));
    }
    let within = apply(Op::Within, vec![input("x"), input("y"), input("x")]);
    assert_eq!(typed_as(&within), Some(Type::Truth));
    assert_eq!(typed_as(&two(Op::Div, "x", "y")), None);
    assert_eq!(typed_as(&two(Op::Rem, "x", "y")), None);
    assert_eq!(typed_as(&apply(Op::Add, vec![input("x"), int(1)])), None);
    assert_eq!(
        typed_as(&apply(Op::Add, vec![input("x"), real(1, 1)])),
        None
    );
    // Two parameters are two types, though both are numbers.
    assert_eq!(typed_as(&two(Op::Add, "x", "u")), None);
    // Any type: == only.
    assert_eq!(typed_as(&two(Op::Equal, "a", "b")), Some(Type::Truth));
    for op in [Op::Add, Op::Less, Op::AtMost] {
        assert_eq!(typed_as(&two(op, "a", "b")), None);
    }
    let pick = apply(
        Op::If,
        vec![two(Op::Equal, "a", "b"), input("a"), input("b")],
    );
    assert_eq!(typed_as(&pick), Some(t.clone()));
    // A call of a function the contract names, by its signature.
    let calls =
        Arc::new(library(vec![("C::f", callee(&[("x", n.clone())], n.clone(), None))]).unwrap());
    let standard = Simulation {
        calls: calls.clone(),
    };
    let e = apply(
        Op::Less,
        vec![
            call("C::f", vec![input("x")]),
            call("C::f", vec![input("y")]),
        ],
    );
    assert_eq!(check(&standard, &inputs, &e), Some(Type::Truth));
    assert_eq!(
        check(&standard, &inputs, &call("C::f", vec![input("u")])),
        None
    );
    // Run by bounds anyway, it has no value: none is of `N`.
    let e = call("C::f", vec![int(1)]);
    let unknown = Err(Fault::Op(Undefined::Unknown("C::f".to_string())));
    assert_eq!(
        eval(
            &Ranges {
                calls: calls.clone()
            },
            &BTreeMap::new(),
            &e
        ),
        unknown
    );
    let unknown = Err(Fault::Op(Undefined::Unknown("C::f".to_string())));
    assert_eq!(
        eval(
            &Exact {
                calls,
                log: Default::default()
            },
            &BTreeMap::new(),
            &e
        ),
        unknown
    );
    // What a bound admits.
    for (bound, ty, admits) in [
        (Bound::Number, Type::Int, true),
        (Bound::Number, Type::Real, true),
        (Bound::Number, n.clone(), true),
        (Bound::Number, Type::Truth, false),
        (Bound::Number, t.clone(), false),
        (
            Bound::Number,
            Type::Record(record("P", &[("x", Type::Int)])),
            false,
        ),
        (Bound::Any, Type::Truth, true),
        (Bound::Any, t.clone(), true),
    ] {
        assert_eq!(bound.admits(&ty), admits, "{bound:?} {ty:?}");
    }
    // Given: in records too; a parameter not given stays.
    let point = Type::Record(record("P", &[("x", n.clone()), ("y", m.clone())]));
    assert!(point.open() && n.open() && !Type::Int.open());
    let given = BTreeMap::from([("N".to_string(), Type::Real)]);
    let half = Type::Record(record("P", &[("x", Type::Real), ("y", m.clone())]));
    assert_eq!(point.given(&given), half);
    assert!(half.open());
    let full = BTreeMap::from([("N".to_string(), Type::Real), ("M".to_string(), Type::Int)]);
    assert!(!point.given(&full).open());
    assert_eq!(Type::Truth.given(&full), Type::Truth);
    // No value, and no bounds, are of a parameter.
    for v in [
        Value::Int(1.into()),
        Value::Real(Rational::integer(1)),
        Value::Truth(true),
    ] {
        assert!(!Exact::default().has_type(&v, &n));
        assert!(!Exact::default().has_type(&v, &t));
        assert!(!Ranges::default().has_type(&Bounds::exactly(&v), &n));
    }
    assert!(!Ranges::default().has_type(&Bounds::any(&n), &n));
}

// ----------------------------------------------------- half-lines (A1, A2)

proptest! {
    #![proptest_config(ProptestConfig::with_cases(20000))]

    /// Each operation on numbers, on spans with one end or none as often
    /// as with two, covers what `Exact` gives at members of them.
    #[test]
    fn numbers_on_half_lines_cover(seed in any::<u64>(), which in 0usize..8, real in any::<bool>()) {
        let mut g = Gen::new(seed);
        let t = if real || which == 3 { Type::Real } else { Type::Int };
        let (op, arity, gives) = [
            (Op::Add, 2, t.clone()),
            (Op::Sub, 2, t.clone()),
            (Op::Mul, 2, t.clone()),
            (Op::Div, 2, Type::Real),
            (Op::Less, 2, Type::Truth),
            (Op::AtMost, 2, Type::Truth),
            (Op::Equal, 2, Type::Truth),
            (Op::Within, 3, Type::Truth),
        ][which].clone();
        let spans: Vec<Span> = (0..arity).map(|_| g.span(&t)).collect();
        let values: Vec<Value> = spans.iter().map(|s| g.member(s, &t)).collect();
        for (s, v) in spans.iter().zip(&values) {
            prop_assert!(mitra_lang::simulation::lies_in(v, s), "{:?} not in {:?}", v, s);
        }
        let bounds: Vec<_> = spans.iter().map(|s| Ok(Bounds { span: s.clone(), may_fault: false })).collect();
        let exact: Vec<_> = values.iter().map(|v| Ok(v.clone())).collect();
        let (b, x) = (g.ranges().apply(&op, &bounds), g.exact().apply(&op, &exact));
        prop_assert!(covers(&gives, &b, &x), "{:?} on {:?}\nbounds {:?}\nexact {:?}", op, spans, b, x);
    }
}

/// The numbers of a span of type `t`.
fn numbers(t: &Type) -> Numbers {
    match t {
        Type::Int => Numbers::Int,
        _ => Numbers::Real,
    }
}

fn q(n: i64) -> Option<Rational> {
    Some(Rational::integer(n))
}

fn on(op: Op, spans: &[Span]) -> Span {
    let bounds: Vec<_> = spans
        .iter()
        .map(|s| {
            Ok(Bounds {
                span: s.clone(),
                may_fault: false,
            })
        })
        .collect();
    Ranges::default().apply(&op, &bounds).unwrap().span
}

#[test]
fn half_lines_by_hand() {
    let below_zero = Span::Number(Numbers::Real, None, q(0));
    let above_zero = Span::Number(Numbers::Real, q(0), None);
    let above_one = Span::Number(Numbers::Real, q(1), None);
    let any = Span::Number(Numbers::Real, None, None);
    // A product of two of one sign is never negative.
    assert_eq!(
        on(Op::Mul, &[below_zero.clone(), below_zero.clone()]),
        above_zero
    );
    assert_eq!(
        on(Op::Mul, &[above_zero.clone(), above_zero.clone()]),
        above_zero
    );
    assert_eq!(
        on(Op::Mul, &[below_zero.clone(), above_zero.clone()]),
        below_zero
    );
    assert_eq!(on(Op::Mul, &[any.clone(), any.clone()]), any);
    assert_eq!(
        on(
            Op::Mul,
            &[Span::Number(Numbers::Real, q(0), q(0)), any.clone()]
        ),
        Span::Number(Numbers::Real, q(0), q(0))
    );
    // Sums keep the ends both have.
    assert_eq!(
        on(Op::Add, &[above_one.clone(), above_zero.clone()]),
        above_one
    );
    assert_eq!(on(Op::Sub, &[above_one.clone(), above_zero.clone()]), any);
    // `q >= 1` is never below 0, and may be above it.
    let zero = Span::Number(Numbers::Real, q(0), q(0));
    let truth = |yes, no| Span::Truth { yes, no };
    assert_eq!(
        on(Op::Less, &[above_one.clone(), zero.clone()]),
        truth(false, true)
    );
    assert_eq!(
        on(Op::AtMost, &[zero.clone(), above_zero.clone()]),
        truth(true, false)
    );
    // A finite number over one with no end: from 0 to the number.
    let two = Span::Number(Numbers::Real, q(2), q(2));
    assert_eq!(
        on(Op::Div, &[two, above_one]),
        Span::Number(Numbers::Real, q(0), q(2))
    );
}

// ------------------------------------------------- remembered calls (A8)

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    /// A reading that remembers calls gives what one that remembers
    /// none gives: one `Exact` and one `Ranges` read many points, each
    /// compared with a fresh one (a copy of the calls starts empty).
    #[test]
    fn remembered_calls_change_no_answer(seed in any::<u64>(), depth in 1u32..5) {
        let mut g = Gen::library(seed, true);
        let t = g.ty();
        let e = g.expr(&t, depth);
        let (exact, ranges) = (g.exact(), g.ranges());
        for _ in 0..6 {
            let values = g.values();
            let fresh = Exact { calls: Arc::new((*g.calls).clone()), log: Default::default() };
            prop_assert_eq!(eval(&exact, &values, &e), eval(&fresh, &values, &e));
            let (bounds, _) = g.bounded();
            let fresh = Ranges { calls: Arc::new((*g.calls).clone()) };
            prop_assert_eq!(eval(&ranges, &bounds, &e), eval(&fresh, &bounds, &e));
        }
    }
}
