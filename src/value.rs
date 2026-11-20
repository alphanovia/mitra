//! Values, types, expressions (`model/value.mitra`).
//!
//! Nothing here is right or wrong, believed or signed: a value comes
//! from its expression and its inputs alone.

/// What a value can be.
// mitra: implements value::Type
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Type {
    OfTruth,
    OfNumber,
    OfShape(String, Vec<Type>),
}

/// What is computed.
// mitra: implements value::Value
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Value {
    Truth(bool),
    Number(i64),
    Made(String, Vec<Value>),
}

/// What computes a value from named inputs.
// mitra: implements value::Expression
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression {
    /// What is written as it is: `true`, `false`, `42`.
    Yes,
    No,
    Literal(i64),
    /// A record made of what the others give.
    Make(String, Vec<Expression>),
    /// A named input.
    Input(String),
    /// A field of a record, by its place.
    Field(Box<Expression>, usize),
    /// `!`, `&&`, `==`, `<`, `+`, `-`, `*` on what the others give.
    Op(String, Vec<Expression>),
    If(Box<Expression>, Box<Expression>, Box<Expression>),
}

// mitra: implements value::type_of
pub fn type_of(v: &Value) -> Type {
    match v {
        Value::Truth(_) => Type::OfTruth,
        Value::Number(_) => Type::OfNumber,
        Value::Made(name, fields) => {
            Type::OfShape(name.clone(), fields.iter().map(type_of).collect())
        }
    }
}

/// The type an expression gives when its inputs have these types.
/// `None` when it does not check.
// mitra: implements value::check
pub fn check(e: &Expression, inputs: &[(String, Type)]) -> Option<Type> {
    use Type::*;
    Some(match e {
        Expression::Yes | Expression::No => OfTruth,
        Expression::Literal(_) => OfNumber,
        Expression::Make(name, fields) => {
            let types: Option<Vec<Type>> = fields.iter().map(|f| check(f, inputs)).collect();
            OfShape(name.clone(), types?)
        }
        Expression::Input(name) => inputs.iter().find(|(n, _)| n == name)?.1.clone(),
        Expression::Field(record, place) => match check(record, inputs)? {
            OfShape(_, fields) => fields.get(*place)?.clone(),
            _ => return None,
        },
        Expression::Op(op, args) => {
            let types: Option<Vec<Type>> = args.iter().map(|a| check(a, inputs)).collect();
            match (op.as_str(), types?.as_slice()) {
                ("!", [OfTruth]) | ("&&", [OfTruth, OfTruth]) => OfTruth,
                ("==", [x, y]) if x == y => OfTruth,
                ("<", [OfNumber, OfNumber]) => OfTruth,
                ("+" | "-" | "*", [OfNumber, OfNumber]) => OfNumber,
                _ => return None,
            }
        }
        Expression::If(test, yes, no) => {
            let (t, y, n) = (
                check(test, inputs)?,
                check(yes, inputs)?,
                check(no, inputs)?,
            );
            match t == OfTruth && y == n {
                true => y,
                false => return None,
            }
        }
    })
}

/// The value of an expression at these inputs. `None` when it has none:
/// an input not given, a field that is not there, a number too large,
/// an operator on what it does not take.
// mitra: implements value::eval
pub fn eval(e: &Expression, inputs: &[(String, Value)]) -> Option<Value> {
    use Value::*;
    Some(match e {
        Expression::Yes => Truth(true),
        Expression::No => Truth(false),
        Expression::Literal(n) => Number(*n),
        Expression::Make(name, fields) => {
            let values: Option<Vec<Value>> = fields.iter().map(|f| eval(f, inputs)).collect();
            Made(name.clone(), values?)
        }
        Expression::Input(name) => inputs.iter().find(|(n, _)| n == name)?.1.clone(),
        Expression::Field(record, place) => match eval(record, inputs)? {
            Made(_, fields) => fields.get(*place)?.clone(),
            _ => return None,
        },
        Expression::Op(op, args) => {
            let values: Option<Vec<Value>> = args.iter().map(|a| eval(a, inputs)).collect();
            match (op.as_str(), values?.as_slice()) {
                ("!", [Truth(x)]) => Truth(!x),
                ("&&", [Truth(x), Truth(y)]) => Truth(*x && *y),
                ("==", [x, y]) if type_of(x) == type_of(y) => Truth(x == y),
                ("<", [Number(x), Number(y)]) => Truth(x < y),
                ("+", [Number(x), Number(y)]) => Number(x.checked_add(*y)?),
                ("-", [Number(x), Number(y)]) => Number(x.checked_sub(*y)?),
                ("*", [Number(x), Number(y)]) => Number(x.checked_mul(*y)?),
                _ => return None,
            }
        }
        Expression::If(test, yes, no) => match eval(test, inputs)? {
            Truth(true) => eval(yes, inputs)?,
            Truth(false) => eval(no, inputs)?,
            _ => return None,
        },
    })
}
