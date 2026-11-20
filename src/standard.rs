//! The standard operations (`model/standard.mitra`): a truth, a whole
//! number, and records, with what applies them. Beside the core, not
//! in it: `value` knows none of this.

use crate::value::Quantity::{Many, Truth, Whole};
use crate::value::{Applies, Operation, Quantity, Type, Value};

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
pub fn yes() -> Value {
    Value {
        quantity: Truth(true),
        of: truth(),
    }
}
// mitra: implements standard::no
pub fn no() -> Value {
    Value {
        quantity: Truth(false),
        of: truth(),
    }
}
// mitra: implements standard::number
pub fn number(n: i64) -> Value {
    Value {
        quantity: Whole(n),
        of: whole(),
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
// mitra: implements standard::less
pub fn less() -> Operation {
    operation("<", &[whole(), whole()], truth())
}
// mitra: implements standard::add
pub fn add() -> Operation {
    operation("+", &[whole(), whole()], whole())
}
// mitra: implements standard::sub
pub fn sub() -> Operation {
    operation("-", &[whole(), whole()], whole())
}
// mitra: implements standard::mul
pub fn mul() -> Operation {
    operation("*", &[whole(), whole()], whole())
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
fn is(op: &Operation, takes: &[&str], gives: &str) -> bool {
    op.gives.name == gives
        && op.takes.len() == takes.len()
        && op.takes.iter().zip(takes).all(|(t, name)| t.name == *name)
}

impl Applies for Table {
    // mitra: implements standard::Table::apply
    fn apply(&self, op: &Operation, args: &[Value]) -> Option<Quantity> {
        const T: &str = "Truth";
        const I: &str = "Int";
        match (op.name.as_str(), args) {
            ("!", [x]) if is(op, &[T], T) => match &x.quantity {
                Truth(x) => Some(Truth(!x)),
                _ => None,
            },
            ("&&" | "<" | "+" | "-" | "*", [x, y]) => match (&x.quantity, &y.quantity) {
                (Truth(x), Truth(y)) if op.name == "&&" && is(op, &[T, T], T) => {
                    Some(Truth(*x && *y))
                }
                (Whole(x), Whole(y)) if op.name == "<" && is(op, &[I, I], T) => Some(Truth(x < y)),
                (Whole(x), Whole(y)) if is(op, &[I, I], I) => Some(Whole(match op.name.as_str() {
                    "+" => x.checked_add(*y)?,
                    "-" => x.checked_sub(*y)?,
                    "*" => x.checked_mul(*y)?,
                    _ => return None,
                })),
                _ => None,
            },
            ("==", [x, y]) if is(op, &[&x.of.name, &x.of.name], T) => {
                Some(Truth(x.quantity == y.quantity))
            }
            ("if", [test, yes, no]) if is(op, &[T, &yes.of.name, &yes.of.name], &yes.of.name) => {
                match test.quantity {
                    Truth(true) => Some(yes.quantity.clone()),
                    Truth(false) => Some(no.quantity.clone()),
                    _ => None,
                }
            }
            _ => self.record(op, args),
        }
    }
}

impl Table {
    /// The operations of the records the table was given.
    fn record(&self, op: &Operation, args: &[Value]) -> Option<Quantity> {
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
