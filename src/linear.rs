//! Linear facts: a small decision, by substitution, of claims that relate
//! two values, which bounds read one at a time cannot (`c == n`, kept
//! by `c + 1 == n + 1`). Used by `agree` for `by invariant`.
//!
//! An expression of numbers is read as a sum of names, each times a
//! number, and a number: `2 * a - b + 3`. The length of a sequence is
//! read too: `(rs ++ [o]).len()` is `rs.len() + 1`. A claim is read as
//! comparisons of those, `==`, `<=` and `<`, under `&&`, `||` and `!`.
//! Anything else is not read, and nothing is decided from it.
//!
//! `holds(hypotheses, goal)`: each equality of the hypotheses gives one
//! name in terms of the others, put in place everywhere (as by hand);
//! then each comparison of the goal must be true of any values, or
//! follow from one comparison of the hypotheses by a number. Sound, and
//! far from complete: what it does not decide stays unknown.

use crate::expression::Expression;
use crate::simulation::Op;
use num_bigint::BigInt;
use num_rational::BigRational;
use num_traits::{Signed, Zero};
use std::collections::BTreeMap;

type E = Expression<Op>;

/// `Σ coefficient · name + constant`.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Sum {
    names: BTreeMap<String, BigRational>,
    constant: BigRational,
}

impl Sum {
    fn number(n: BigRational) -> Sum {
        Sum {
            names: BTreeMap::new(),
            constant: n,
        }
    }
    fn name(n: &str) -> Sum {
        let mut names = BTreeMap::new();
        names.insert(n.to_string(), BigRational::from_integer(1.into()));
        Sum {
            names,
            constant: BigRational::zero(),
        }
    }
    fn plus(mut self, other: &Sum, times: &BigRational) -> Sum {
        for (n, c) in &other.names {
            let entry = self
                .names
                .entry(n.clone())
                .or_insert_with(BigRational::zero);
            *entry += c * times;
        }
        self.names.retain(|_, c| !c.is_zero());
        self.constant += &other.constant * times;
        self
    }
    fn times(&self, k: &BigRational) -> Sum {
        Sum::number(BigRational::zero()).plus(self, k)
    }
    fn constant(&self) -> Option<&BigRational> {
        self.names.is_empty().then_some(&self.constant)
    }
    /// `self` with `name` replaced by `by`.
    fn put(&self, name: &str, by: &Sum) -> Sum {
        match self.names.get(name) {
            Some(c) => {
                let c = c.clone();
                let mut rest = self.clone();
                rest.names.remove(name);
                rest.plus(by, &c)
            }
            None => self.clone(),
        }
    }
}

/// A value as read: a number, a sequence by its length, a record by its
/// fields, or a name not known, read by what is asked of it.
#[derive(Clone, Debug)]
enum Read {
    Sum(Sum),
    Seq(Sum),
    Record(BTreeMap<String, Read>),
    Name(String),
}

impl Read {
    fn sum(&self) -> Option<Sum> {
        match self {
            Read::Sum(s) => Some(s.clone()),
            Read::Name(n) => Some(Sum::name(n)),
            _ => None,
        }
    }
    fn length(&self) -> Option<Sum> {
        match self {
            Read::Seq(l) => Some(l.clone()),
            Read::Name(n) => Some(Sum::name(&format!("{n}.len()"))),
            _ => None,
        }
    }
    fn field(&self, f: &str) -> Option<Read> {
        match self {
            Read::Record(fields) => fields.get(f).cloned(),
            Read::Name(n) => Some(Read::Name(format!("{n}.{f}"))),
            _ => None,
        }
    }
}

/// A comparison: `sum == 0`, `sum >= 0`, or `sum > 0`.
#[derive(Clone, Debug, PartialEq, Eq)]
enum Is {
    Zero,
    AtLeastZero,
    AboveZero,
}

#[derive(Clone, Debug)]
enum Fact {
    Compare(Is, Sum),
    All(Vec<Fact>),
    Any(Vec<Fact>),
    Known(bool),
}

type Env = BTreeMap<String, Read>;

/// How many names a `let` made: each read under a name of its own, so
/// that no name is caught by another of the same spelling.
struct Reader {
    fresh: usize,
}

impl Reader {
    fn read(&mut self, e: &E, env: &Env) -> Option<Read> {
        let Expression::Apply(op, args) = e else {
            let Expression::Name(n) = e else {
                return None;
            };
            return Some(env.get(n).cloned().unwrap_or_else(|| Read::Name(n.clone())));
        };
        let arg = |r: &mut Reader, i: usize| -> Option<Read> {
            let a = args.get(i)?;
            if !a.binds.is_empty() {
                return None;
            }
            r.read(&a.body, env)
        };
        Some(match (op, args.len()) {
            (Op::Int(n), 0) => Read::Sum(Sum::number(BigRational::from_integer(n.clone()))),
            (Op::Real(r), 0) => Read::Sum(Sum::number(BigRational::new(
                r.numer().clone(),
                r.denom().clone(),
            ))),
            (Op::Add | Op::Sub, 2) => {
                let (a, b) = (arg(self, 0)?.sum()?, arg(self, 1)?.sum()?);
                let sign = match op {
                    Op::Add => 1,
                    _ => -1,
                };
                Read::Sum(a.plus(&b, &BigRational::from_integer(sign.into())))
            }
            (Op::Mul, 2) => {
                let (a, b) = (arg(self, 0)?.sum()?, arg(self, 1)?.sum()?);
                match (a.constant(), b.constant()) {
                    (Some(k), _) => Read::Sum(b.times(k)),
                    (_, Some(k)) => Read::Sum(a.times(k)),
                    _ => return None,
                }
            }
            (Op::Seq(_), n) => {
                for i in 0..n {
                    arg(self, i)?;
                }
                Read::Seq(Sum::number(BigRational::from_integer(BigInt::from(n))))
            }
            (Op::Text(s), 0) => Read::Seq(Sum::number(BigRational::from_integer(BigInt::from(
                s.len(),
            )))),
            (Op::Join, 2) => {
                let (a, b) = (arg(self, 0)?.length()?, arg(self, 1)?.length()?);
                Read::Seq(a.plus(&b, &BigRational::from_integer(1.into())))
            }
            (Op::Len, 1) => Read::Sum(arg(self, 0)?.length()?),
            (Op::Make(r), n) if n == r.fields.len() => {
                let mut fields = BTreeMap::new();
                for (i, (f, _)) in r.fields.iter().enumerate() {
                    // A field not read is no fact: what it holds is not asked.
                    let value = arg(self, i).unwrap_or_else(|| {
                        self.fresh += 1;
                        Read::Name(format!("#{}", self.fresh))
                    });
                    fields.entry(f.clone()).or_insert(value);
                }
                Read::Record(fields)
            }
            (Op::Field(f), 1) => arg(self, 0)?.field(f)?,
            (Op::Let, 2) => {
                let [value, body] = args.as_slice() else {
                    return None;
                };
                let [x] = body.binds.as_slice() else {
                    return None;
                };
                let bound = match value.binds.is_empty() {
                    true => self.read(&value.body, env),
                    false => None,
                };
                let bound = bound.unwrap_or_else(|| {
                    self.fresh += 1;
                    Read::Name(format!("{x}#{}", self.fresh))
                });
                let mut inner = env.clone();
                inner.insert(x.clone(), bound);
                self.read(&body.body, &inner)?
            }
            _ => return None,
        })
    }

    fn fact(&mut self, e: &E, env: &Env) -> Option<Fact> {
        let Expression::Apply(op, args) = e else {
            return None;
        };
        let plain = args.iter().all(|a| a.binds.is_empty());
        let sides = |r: &mut Reader| -> Option<(Sum, Sum)> {
            Some((
                r.read(&args.first()?.body, env)?.sum()?,
                r.read(&args.get(1)?.body, env)?.sum()?,
            ))
        };
        let minus = -BigRational::from_integer(1.into());
        Some(match (op, args.len()) {
            (Op::Truth(b), 0) => Fact::Known(*b),
            (Op::And | Op::Or, 2) if plain => {
                let a = self.fact(&args[0].body, env)?;
                let b = self.fact(&args[1].body, env)?;
                match op {
                    Op::And => Fact::All(vec![a, b]),
                    _ => Fact::Any(vec![a, b]),
                }
            }
            // `a == b`: `a - b == 0`; `a <= b`: `b - a >= 0`; `a < b`: `b - a > 0`.
            (Op::Equal, 2) if plain => {
                let (a, b) = sides(self)?;
                Fact::Compare(Is::Zero, a.plus(&b, &minus))
            }
            (Op::AtMost, 2) if plain => {
                let (a, b) = sides(self)?;
                Fact::Compare(Is::AtLeastZero, b.plus(&a, &minus))
            }
            (Op::Less, 2) if plain => {
                let (a, b) = sides(self)?;
                Fact::Compare(Is::AboveZero, b.plus(&a, &minus))
            }
            (Op::Not, 1) if plain => not(self.fact(&args[0].body, env)?)?,
            (Op::Let, 2) => {
                let [value, body] = args.as_slice() else {
                    return None;
                };
                let [x] = body.binds.as_slice() else {
                    return None;
                };
                let bound = match value.binds.is_empty() {
                    true => self.read(&value.body, env),
                    false => None,
                };
                let bound = bound.unwrap_or_else(|| {
                    self.fresh += 1;
                    Read::Name(format!("{x}#{}", self.fresh))
                });
                let mut inner = env.clone();
                inner.insert(x.clone(), bound);
                self.fact(&body.body, &inner)?
            }
            _ => return None,
        })
    }
}

/// The negation of a fact, when it is one this reads.
fn not(f: Fact) -> Option<Fact> {
    let minus = -BigRational::from_integer(1.into());
    Some(match f {
        Fact::Known(b) => Fact::Known(!b),
        Fact::Compare(Is::AtLeastZero, s) => Fact::Compare(Is::AboveZero, s.times(&minus)),
        Fact::Compare(Is::AboveZero, s) => Fact::Compare(Is::AtLeastZero, s.times(&minus)),
        Fact::Compare(Is::Zero, _) => return None,
        Fact::All(fs) => Fact::Any(fs.into_iter().map(not).collect::<Option<_>>()?),
        Fact::Any(fs) => Fact::All(fs.into_iter().map(not).collect::<Option<_>>()?),
    })
}

/// Each comparison a fact says for sure: those under `&&`.
fn sure(f: Fact, out: &mut Vec<(Is, Sum)>) {
    match f {
        Fact::Compare(is, s) => out.push((is, s)),
        Fact::All(fs) => fs.into_iter().for_each(|f| sure(f, out)),
        Fact::Any(_) | Fact::Known(_) => {}
    }
}

/// `f` with `name` replaced by `by` in every comparison.
fn put(f: &Fact, name: &str, by: &Sum) -> Fact {
    match f {
        Fact::Compare(is, s) => Fact::Compare(is.clone(), s.put(name, by)),
        Fact::All(fs) => Fact::All(fs.iter().map(|f| put(f, name, by)).collect()),
        Fact::Any(fs) => Fact::Any(fs.iter().map(|f| put(f, name, by)).collect()),
        Fact::Known(b) => Fact::Known(*b),
    }
}

/// Whether `goal` follows from `hypotheses`, each a truth expression;
/// the names free in them stand for any values. False when not shown.
pub fn holds(hypotheses: &[E], goal: &E) -> bool {
    let mut reader = Reader { fresh: 0 };
    let env = Env::new();
    let Some(mut goal) = reader.fact(goal, &env) else {
        return false;
    };
    // A hypothesis this cannot read is left out: it only weakens.
    let mut known = Vec::new();
    for h in hypotheses {
        if let Some(f) = reader.fact(h, &env) {
            sure(f, &mut known);
        }
    }
    // Each equality gives one of its names in terms of the others.
    let mut i = 0;
    while i < known.len() {
        let (is, s) = known[i].clone();
        if is == Is::Zero {
            if let Some((name, c)) = s
                .names
                .iter()
                .next_back()
                .map(|(n, c)| (n.clone(), c.clone()))
            {
                // name = -(s - c·name) / c
                let mut rest = s.clone();
                rest.names.remove(&name);
                let by = rest.times(&(-BigRational::from_integer(1.into()) / c));
                // Itself too: it becomes `0 == 0`.
                for (_, other) in known.iter_mut() {
                    *other = other.put(&name, &by);
                }
                goal = put(&goal, &name, &by);
            }
        }
        i += 1;
    }
    shown(&goal, &known)
}

fn shown(goal: &Fact, known: &[(Is, Sum)]) -> bool {
    match goal {
        Fact::Known(b) => *b,
        Fact::All(fs) => fs.iter().all(|f| shown(f, known)),
        Fact::Any(fs) => fs.iter().any(|f| shown(f, known)),
        Fact::Compare(is, s) => {
            // True of any values, or a known comparison plus a number.
            let by = |k: &BigRational, strict: bool| match is {
                Is::Zero => k.is_zero() && !strict,
                Is::AtLeastZero => !k.is_negative(),
                Is::AboveZero => k.is_positive() || (strict && k.is_zero()),
            };
            if let Some(k) = s.constant() {
                return by(k, false);
            }
            known.iter().any(|(was, h)| {
                // `s = λ·h + k`, `λ > 0`: as `h` is, so is `s`, past `k`.
                let Some((first, c)) = h.names.iter().next() else {
                    return false;
                };
                let Some(lambda) = s.names.get(first).map(|d| d / c) else {
                    return false;
                };
                if !lambda.is_positive() && *was != Is::Zero {
                    return false;
                }
                let Some(k) = s.clone().plus(h, &-lambda).constant().cloned() else {
                    return false;
                };
                match was {
                    // `h == 0`: `s` is `k`.
                    Is::Zero => by(&k, false),
                    Is::AtLeastZero => *is != Is::Zero && by(&k, false),
                    Is::AboveZero => *is != Is::Zero && by(&k, true),
                }
            })
        }
    }
}
