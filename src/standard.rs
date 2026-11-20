//! The standard operations (`model/standard.mitra`): a truth, a whole
//! number, a real, and records, with what applies them. Beside the core, not
//! in it: `value` knows none of this.

use crate::value::{Applies, Operation, Type, Value};
use Exact::{Many, Truth, Whole};

/// An exact quantity: one truth, one whole number, or several of
/// them, as a real and a record are.
// mitra: implements standard::Exact
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Exact {
    Truth(bool),
    Whole(i64),
    Many(Vec<Exact>),
}

fn named(name: &str) -> Type {
    Type {
        name: name.to_string(),
    }
}
fn operation(name: &str, takes: &[Type], gives: Type) -> Operation {
    Operation {
        name: name.to_string(),
        takes: takes.to_vec(),
        gives,
    }
}

// mitra: implements standard::truth
pub fn truth() -> Type {
    named("Truth")
}
/// A whole number, 64 bits signed.
// mitra: implements standard::whole
pub fn whole() -> Type {
    named("Int")
}

// mitra: implements standard::yes
pub fn yes() -> Value<Exact> {
    Value {
        quantity: Truth(true),
        of: truth(),
    }
}
// mitra: implements standard::no
pub fn no() -> Value<Exact> {
    Value {
        quantity: Truth(false),
        of: truth(),
    }
}
// mitra: implements standard::number
pub fn number(n: i64) -> Value<Exact> {
    Value {
        quantity: Whole(n),
        of: whole(),
    }
}
/// A number of the world: a ratio of two whole numbers, exact.
// mitra: implements standard::real
pub fn real() -> Type {
    named("Real")
}
/// The real `n / d`, in lowest terms. None when `d` is 0.
// mitra: implements standard::ratio
pub fn ratio(n: i64, d: i64) -> Option<Value<Exact>> {
    Some(Value {
        quantity: lowest(n as i128, d as i128)?,
        of: real(),
    })
}

/// A ratio in lowest terms, its lower part above 0. None when the
/// lower part is 0 or a part does not fit 64 bits.
fn lowest(n: i128, d: i128) -> Option<Exact> {
    if d == 0 {
        return None;
    }
    let (mut a, mut b) = (n.unsigned_abs(), d.unsigned_abs());
    while b != 0 {
        (a, b) = (b, a % b);
    }
    let by = a.max(1) as i128 * d.signum();
    let n = i64::try_from(n / by).ok()?;
    let d = i64::try_from(d / by).ok()?;
    Some(Many(vec![Whole(n), Whole(d)]))
}
/// The two parts of a real.
fn parts(q: &Exact) -> Option<(i128, i128)> {
    match q {
        Many(both) => match both.as_slice() {
            [Whole(n), Whole(d)] if *d > 0 => Some((*n as i128, *d as i128)),
            _ => None,
        },
        _ => None,
    }
}
/// A number as a ratio: a whole number over 1, or the parts of a real.
fn as_ratio(q: &Exact) -> Option<(i128, i128)> {
    match q {
        Whole(n) => Some((*n as i128, 1)),
        _ => parts(q),
    }
}

// mitra: implements standard::not
pub fn not() -> Operation {
    operation("!", &[truth()], truth())
}
// mitra: implements standard::and
pub fn and() -> Operation {
    operation("&&", &[truth(), truth()], truth())
}
// mitra: implements standard::or
pub fn or() -> Operation {
    operation("||", &[truth(), truth()], truth())
}
/// `<` on two numbers of one type: whole or real.
// mitra: implements standard::less
pub fn less(of: &Type) -> Operation {
    operation("<", &[of.clone(), of.clone()], truth())
}
// mitra: implements standard::at_most
pub fn at_most(of: &Type) -> Operation {
    operation("<=", &[of.clone(), of.clone()], truth())
}
// mitra: implements standard::more
pub fn more(of: &Type) -> Operation {
    operation(">", &[of.clone(), of.clone()], truth())
}
// mitra: implements standard::at_least
pub fn at_least(of: &Type) -> Operation {
    operation(">=", &[of.clone(), of.clone()], truth())
}
/// `+` on two numbers of one type: whole or real.
// mitra: implements standard::add
pub fn add(of: &Type) -> Operation {
    operation("+", &[of.clone(), of.clone()], of.clone())
}
// mitra: implements standard::sub
pub fn sub(of: &Type) -> Operation {
    operation("-", &[of.clone(), of.clone()], of.clone())
}
// mitra: implements standard::mul
pub fn mul(of: &Type) -> Operation {
    operation("*", &[of.clone(), of.clone()], of.clone())
}
/// Two numbers of one type are no further apart than a third: the
/// distance between the first two is at most the tolerance.
// mitra: implements standard::within
pub fn within(of: &Type) -> Operation {
    operation("within", &[of.clone(), of.clone(), of.clone()], truth())
}
/// `==` on two values of one type, whatever the type.
// mitra: implements standard::equal
pub fn equal(of: &Type) -> Operation {
    operation("==", &[of.clone(), of.clone()], truth())
}
/// `if` on a truth and two values of one type.
// mitra: implements standard::choose
pub fn choose(of: &Type) -> Operation {
    operation("if", &[truth(), of.clone(), of.clone()], of.clone())
}

/// A whole number of 8 bits, signed, and one of 16.
// mitra: implements standard::int8
pub fn int8() -> Type {
    named("Int8")
}
// mitra: implements standard::int16
pub fn int16() -> Type {
    named("Int16")
}
/// The Int8 `n`. None outside -128 to 127.
// mitra: implements standard::small
pub fn small(n: i64) -> Option<Value<Exact>> {
    Some(Value {
        quantity: Whole(i8::try_from(n).ok()? as i64),
        of: int8(),
    })
}
/// Three additions of two Int8, named as Rust names them. The first
/// gives an Int16 and always has a value. The second gives an Int8 and
/// wraps around. The third gives an Int8, and no value when the sum
/// does not fit.
// mitra: implements standard::widening_add
pub fn widening_add() -> Operation {
    operation("widening_add", &[int8(), int8()], int16())
}
// mitra: implements standard::wrapping_add
pub fn wrapping_add() -> Operation {
    operation("wrapping_add", &[int8(), int8()], int8())
}
// mitra: implements standard::checked_add
pub fn checked_add() -> Operation {
    operation("checked_add", &[int8(), int8()], int8())
}

/// A record: a type of its name, made of fields of these types.
// mitra: implements standard::Shape
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Shape {
    pub name: String,
    pub fields: Vec<Type>,
}

/// What makes a record of its fields.
// mitra: implements standard::make
pub fn make(shape: &Shape) -> Operation {
    operation(&shape.name, &shape.fields, named(&shape.name))
}
/// What reads a field of a record, by its place. `None` past the last.
// mitra: implements standard::field
pub fn field(shape: &Shape, place: usize) -> Option<Operation> {
    let gives = shape.fields.get(place)?.clone();
    Some(operation(
        &format!("{}.{place}", shape.name),
        &[named(&shape.name)],
        gives,
    ))
}

/// What applies the standard operations, and those of the records it
/// is given.
// mitra: implements standard::Table
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Table {
    pub shapes: Vec<Shape>,
}

/// The operation is this one: its name is already matched.
pub(crate) fn is(op: &Operation, takes: &[&str], gives: &str) -> bool {
    op.gives.name == gives
        && op.takes.len() == takes.len()
        && op.takes.iter().zip(takes).all(|(t, name)| t.name == *name)
}

impl Applies<Exact> for Table {
    // mitra: implements standard::Table::apply
    fn apply(&self, op: &Operation, args: &[Option<Value<Exact>>]) -> Option<Exact> {
        const T: &str = "Truth";
        // `if` gives the branch its test takes, even when the other
        // has no value.
        if let ("if", [Some(test), yes, no]) = (op.name.as_str(), args) {
            let taken = match test.quantity {
                Truth(true) => yes.as_ref()?,
                Truth(false) => no.as_ref()?,
                _ => return None,
            };
            return match is(op, &[T, &taken.of.name, &taken.of.name], &taken.of.name) {
                true => Some(taken.quantity.clone()),
                false => None,
            };
        }
        // Every other operation needs all its values.
        let present: Option<Vec<Value<Exact>>> = args.iter().cloned().collect();
        let present = present?;
        let args = present.as_slice();
        const I: &str = "Int";
        const R: &str = "Real";
        match (op.name.as_str(), args) {
            ("!", [x]) if is(op, &[T], T) => match &x.quantity {
                Truth(x) => Some(Truth(!x)),
                _ => None,
            },
            ("&&" | "||", [x, y]) if is(op, &[T, T], T) => match (&x.quantity, &y.quantity) {
                (Truth(x), Truth(y)) if op.name == "&&" => Some(Truth(*x && *y)),
                (Truth(x), Truth(y)) => Some(Truth(*x || *y)),
                _ => None,
            },
            ("<" | "<=" | ">" | ">=", [x, y]) if is(op, &[I, I], T) || is(op, &[R, R], T) => {
                let ((xn, xd), (yn, yd)) = (as_ratio(&x.quantity)?, as_ratio(&y.quantity)?);
                let (left, right) = (xn * yd, yn * xd);
                Some(Truth(match op.name.as_str() {
                    "<" => left < right,
                    "<=" => left <= right,
                    ">" => left > right,
                    _ => left >= right,
                }))
            }
            ("+" | "-" | "*", [x, y]) if is(op, &[I, I], I) => match (&x.quantity, &y.quantity) {
                (Whole(x), Whole(y)) => Some(Whole(match op.name.as_str() {
                    "+" => x.checked_add(*y)?,
                    "-" => x.checked_sub(*y)?,
                    _ => x.checked_mul(*y)?,
                })),
                _ => None,
            },
            ("+" | "-" | "*", [x, y]) if is(op, &[R, R], R) => {
                let ((xn, xd), (yn, yd)) = (parts(&x.quantity)?, parts(&y.quantity)?);
                match op.name.as_str() {
                    "+" => lowest(xn * yd + yn * xd, xd * yd),
                    "-" => lowest(xn * yd - yn * xd, xd * yd),
                    _ => lowest(xn * yn, xd * yd),
                }
            }
            ("widening_add" | "wrapping_add" | "checked_add", [x, y])
                if is(op, &["Int8", "Int8"], "Int16") || is(op, &["Int8", "Int8"], "Int8") =>
            {
                let (x, y) = match (&x.quantity, &y.quantity) {
                    (Whole(x), Whole(y)) => (i8::try_from(*x).ok()?, i8::try_from(*y).ok()?),
                    _ => return None,
                };
                match (op.name.as_str(), op.gives.name.as_str()) {
                    ("widening_add", "Int16") => Some(Whole(x as i64 + y as i64)),
                    ("wrapping_add", "Int8") => Some(Whole(x.wrapping_add(y) as i64)),
                    ("checked_add", "Int8") => Some(Whole(x.checked_add(y)? as i64)),
                    _ => None,
                }
            }
            ("within", [x, y, by]) if is(op, &[I, I, I], T) || is(op, &[R, R, R], T) => {
                let ((xn, xd), (yn, yd)) = (as_ratio(&x.quantity)?, as_ratio(&y.quantity)?);
                let (tn, td) = as_ratio(&by.quantity)?;
                // The distance, as a number of the type: none when it
                // does not fit.
                let (n, d) = match lowest(xn * yd - yn * xd, xd * yd)? {
                    Many(both) => parts(&Many(both))?,
                    _ => return None,
                };
                Some(Truth(n.abs() * td <= tn * d))
            }
            ("==", [x, y]) if is(op, &[&x.of.name, &x.of.name], T) => {
                Some(Truth(x.quantity == y.quantity))
            }
            _ => self.record(op, args),
        }
    }
}

impl Table {
    /// The operations of the records the table was given.
    fn record(&self, op: &Operation, args: &[Value<Exact>]) -> Option<Exact> {
        for shape in &self.shapes {
            if op == &make(shape) {
                return Some(Many(args.iter().map(|v| v.quantity.clone()).collect()));
            }
            if let ([made], Some(dot)) = (args, op.name.rfind('.')) {
                let place: usize = op.name[dot + 1..].parse().ok()?;
                if field(shape, place).as_ref() == Some(op) {
                    return match &made.quantity {
                        Many(fields) => fields.get(place).cloned(),
                        _ => None,
                    };
                }
            }
        }
        None
    }
}
