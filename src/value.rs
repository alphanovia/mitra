//! Values, types, operations, expressions (`model/value.mitra`).
//!
//! A value is a quantity with a type. Operations are declared on
//! types, to transform or to convert. An expression is nested
//! operations that can be evaluated. Nothing is built in: this file
//! declares no type, no operation and no kind of quantity, and never
//! reads one.
//!
//! Nothing here is right or wrong, believed or signed: a value comes
//! from its expression and its inputs alone.

/// What a value can be. Known by its name only.
// mitra: implements Type
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Type {
    pub name: String,
}

/// A quantity with a type. What a quantity is, `Q`, is not said here:
/// one exact amount, the bounds it lies in, or another reading. Only
/// what applies an operation knows what is inside.
// mitra: implements Value
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Value<Q> {
    pub quantity: Q,
    pub of: Type,
}

/// What is declared on types: it takes values of these types and gives
/// one of that type. Same types in and out, it transforms; another
/// type out, it converts.
// mitra: implements value::Operation
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Operation {
    pub name: String,
    pub takes: Vec<Type>,
    pub gives: Type,
}

/// Nested operations that can be evaluated.
// mitra: implements value::Expression
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Expression<Q> {
    /// A value as it is.
    Given(Value<Q>),
    /// A named input of a type.
    Input(String, Type),
    /// An operation on what the others give.
    Apply(Operation, Vec<Expression<Q>>),
}

/// What applies operations on one kind of quantity: the contract
/// `Applies` of the model. Each is one reading of the same
/// expressions. The core has none of its own.
pub trait Applies<Q> {
    /// The quantity an operation gives on these values, or none: an
    /// operation it does not know, or a result that does not exist.
    /// An argument that has no value is given as `None`: what the
    /// operation gives then is its own to say.
    fn apply(&self, op: &Operation, args: &[Option<Value<Q>>]) -> Option<Q>;
}

// mitra: implements value::type_of
pub fn type_of<Q>(v: &Value<Q>) -> Type {
    v.of.clone()
}

/// The type an expression gives. `None` when it does not check: an
/// operation applied to what it does not take.
// mitra: implements value::check
pub fn check<Q>(e: &Expression<Q>) -> Option<Type> {
    Some(match e {
        Expression::Given(v) => v.of.clone(),
        Expression::Input(_, of) => of.clone(),
        Expression::Apply(op, args) => {
            if op.takes.len() != args.len() {
                return None;
            }
            for (arg, takes) in args.iter().zip(&op.takes) {
                if &check(arg)? != takes {
                    return None;
                }
            }
            op.gives.clone()
        }
    })
}

/// The value of an expression at these inputs. `None` when it has
/// none: an input not given or of another type, an operation on what
/// it does not take, or one that `by` gives nothing for. An argument
/// with no value is passed on as none: `by` says what the operation
/// gives then.
// mitra: implements value::eval
pub fn eval<Q: Clone>(
    e: &Expression<Q>,
    inputs: &[(String, Value<Q>)],
    by: &impl Applies<Q>,
) -> Option<Value<Q>> {
    Some(match e {
        Expression::Given(v) => v.clone(),
        Expression::Input(name, of) => {
            let v = &inputs.iter().find(|(n, _)| n == name)?.1;
            match &v.of == of {
                true => v.clone(),
                false => return None,
            }
        }
        Expression::Apply(op, args) => {
            if op.takes.len() != args.len() {
                return None;
            }
            let mut values = Vec::with_capacity(args.len());
            for (arg, takes) in args.iter().zip(&op.takes) {
                let v = eval(arg, inputs, by);
                if v.as_ref().is_some_and(|v| &v.of != takes) {
                    return None;
                }
                values.push(v);
            }
            Value {
                quantity: by.apply(op, &values)?,
                of: op.gives.clone(),
            }
        }
    })
}
