//! Agree (`docs/agree.md`): is the model sound on itself? With no
//! evidence, it compares what the model says twice: each function's
//! body with its claims and its examples, its examples with its claims,
//! each contract's claim read of a function that fits it, and each
//! example of the model with what the model computes, through calls.
//!
//! A comparison `Holds`, `Fails` at a point, or is `Unknown`. It holds
//! when `Ranges`, over all of what it reads, says surely true; it fails
//! only at a point `Exact` confirms; else it is unknown, and says how
//! far it looked. Nothing is solved: no SMT, by your choice; points are
//! tried up to `Settings::points`. Every failure and every unknown is a
//! finding of suit `A`; nothing is dropped.
//!
//! It reads a model `check` accepts: on another, it compares nothing.

use crate::catalogue::{Code, Finding, A001, A002, A003, A004, A005, A006, A007};
use crate::evidence::port;
use crate::expression::{eval, Arg, Expression, Fault};
use crate::model::{calls, contract_claims, Function, Index, Model, ModelExample, Path};
use crate::print;
use crate::simulation::{
    Bounds, Callee, Calls, Exact, Op, Ranges, Rational, Type, Undefined, Value,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// How far agree looks.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    /// The most points `Exact` tries for one comparison, after bounds
    /// leave it open: the examples' points first, then small numbers and
    /// the constants the expressions write, and their neighbours.
    pub points: usize,
}

/// 64 points per comparison: every example row, and for one or two
/// arguments the small numbers and constants of a typical claim.
impl Default for Settings {
    fn default() -> Settings {
        Settings { points: 64 }
    }
}

/// What a comparison found.
#[derive(Clone, Debug, PartialEq)]
pub enum Agreement {
    Holds,
    /// At these values of what it reads, the model gives `gives` (none:
    /// a fault), and the comparison is false there. Confirmed by `Exact`.
    Fails {
        at: Vec<(String, Value)>,
        gives: Option<Value>,
    },
    /// Not decided: why, and how far it looked.
    Unknown {
        why: String,
    },
}

/// One comparison: of the item at `at`, against what, and its outcome.
#[derive(Clone, Debug, PartialEq)]
pub struct Agreed {
    pub at: Path,
    pub against: String,
    pub agreement: Agreement,
}

/// Every comparison of the model, and a finding for each that does not
/// hold.
pub fn agree(model: &Model, settings: &Settings) -> (Vec<Agreed>, Vec<Finding>) {
    let Ok(library) = calls(model) else {
        return (vec![], vec![]);
    };
    let library = Arc::new(library);
    let mut agreed = Vec::new();
    for part in &model.parts {
        for f in &part.functions {
            let at = Path::to(&[&part.name, &f.name]);
            function(&library, &at, f, settings, &mut agreed);
        }
    }
    for c in contract_claims(model) {
        let (exact, ranges) = readings(&library);
        let agreement = claim(&exact, &ranges, &c.over, &c.holds, &[], settings);
        agreed.push(Agreed {
            at: c.at,
            against: "every value of what it reads".to_string(),
            agreement,
        });
    }
    let index = Index::of(model);
    for (i, example) in model.examples.iter().enumerate() {
        agreed.extend(model_example(&index, &library, i, example));
    }
    let mut findings: Vec<Finding> = agreed.iter().filter_map(finding).collect();
    findings.sort_by(|a, b| (&a.at, a.code, &a.message).cmp(&(&b.at, b.code, &b.message)));
    (agreed, findings)
}

fn readings(library: &Arc<Calls>) -> (Exact, Ranges) {
    (
        Exact {
            calls: library.clone(),
        },
        Ranges {
            calls: library.clone(),
        },
    )
}

/// The comparisons of one function.
fn function(
    library: &Arc<Calls>,
    at: &Path,
    f: &Function,
    settings: &Settings,
    agreed: &mut Vec<Agreed>,
) {
    let (exact, ranges) = readings(library);
    let rows: Vec<Vec<(String, Value)>> = f
        .examples
        .iter()
        .map(|e| {
            f.takes
                .iter()
                .map(|(n, _)| n.clone())
                .zip(e.takes.iter().cloned())
                .collect()
        })
        .collect();
    let mut push = |against: String, agreement| {
        agreed.push(Agreed {
            at: at.clone(),
            against,
            agreement,
        })
    };
    if let Some(body) = &f.body {
        // Each claim, with the body read in place of what it gives.
        for c in &f.claims {
            let with_body = let_in(&f.gives.0, body, &c.holds);
            let agreement = claim(&exact, &ranges, &f.takes, &with_body, &rows, settings);
            push(format!("claim `{}`", c.name), agreement);
        }
        // Each example: the body at its point.
        for (i, (e, row)) in f.examples.iter().zip(&rows).enumerate() {
            let gives = eval(&exact, &row.iter().cloned().collect(), body);
            let agreement = match gives {
                Ok(v) if v == e.gives => Agreement::Holds,
                Ok(v) => Agreement::Fails {
                    at: row.clone(),
                    gives: Some(v),
                },
                Err(Fault::Op(Undefined::Unknown(g))) => Agreement::Unknown {
                    why: format!("no value for `{g}` here"),
                },
                Err(_) => Agreement::Fails {
                    at: row.clone(),
                    gives: None,
                },
            };
            push(format!("example {i}"), agreement);
        }
    }
    // Each example against each claim, the example's value read as what
    // the function gives.
    for (i, (e, row)) in f.examples.iter().zip(&rows).enumerate() {
        for c in &f.claims {
            let mut values: BTreeMap<String, Value> = row.iter().cloned().collect();
            values.insert(f.gives.0.clone(), e.gives.clone());
            let agreement = match decided(&eval(&exact, &values, &c.holds)) {
                Some(true) => Agreement::Holds,
                Some(false) => Agreement::Fails {
                    at: row.clone(),
                    gives: Some(e.gives.clone()),
                },
                None => Agreement::Unknown {
                    why: "the claim calls a function with no value here".to_string(),
                },
            };
            push(format!("example {i} against claim `{}`", c.name), agreement);
        }
    }
    // Claims that no value can keep together, by bounds.
    if f.claims.len() > 1 {
        let all = f
            .claims
            .iter()
            .map(|c| c.holds.clone())
            .reduce(|a, b| Expression::Apply(Op::And, vec![plain(a), plain(b)]));
        let reads: Vec<(String, Type)> = f.takes.iter().chain([&f.gives]).cloned().collect();
        let bounds = reads
            .iter()
            .map(|(n, t)| (n.clone(), Bounds::any(t)))
            .collect();
        if let Some(all) = all {
            if eval(&ranges, &bounds, &all).ok().and_then(|b| b.surely()) == Some(false) {
                push(
                    "its claims together".to_string(),
                    Agreement::Fails {
                        at: vec![],
                        gives: None,
                    },
                );
            }
        }
    }
}

/// `let name = value; body`.
fn let_in(name: &str, value: &Expression<Op>, body: &Expression<Op>) -> Expression<Op> {
    Expression::Apply(
        Op::Let,
        vec![
            plain(value.clone()),
            Arg {
                binds: vec![name.to_string()],
                body: body.clone(),
            },
        ],
    )
}

fn plain(body: Expression<Op>) -> Arg<Op> {
    Arg {
        binds: vec![],
        body,
    }
}

/// A truth, or `None` for a function with no value here. A fault of its
/// own is false: a claim with no value at a point does not hold there.
fn decided(outcome: &Result<Value, Fault<Undefined>>) -> Option<bool> {
    match outcome {
        Ok(Value::Truth(b)) => Some(*b),
        Err(Fault::Op(Undefined::Unknown(_))) => None,
        _ => Some(false),
    }
}

/// Whether `holds`, reading `over`, is true at every value of it: by
/// bounds over all of them, then at points, `first` first.
fn claim(
    exact: &Exact,
    ranges: &Ranges,
    over: &[(String, Type)],
    holds: &Expression<Op>,
    first: &[Vec<(String, Value)>],
    settings: &Settings,
) -> Agreement {
    let bounds = over
        .iter()
        .map(|(n, t)| (n.clone(), Bounds::any(t)))
        .collect();
    if eval(ranges, &bounds, holds).ok().and_then(|b| b.surely()) == Some(true) {
        return Agreement::Holds;
    }
    let points = candidates(over, holds, first, settings.points);
    let (mut tried, mut unknown) = (0, BTreeSet::new());
    for point in &points {
        let values: BTreeMap<String, Value> = point.iter().cloned().collect();
        tried += 1;
        match eval(exact, &values, holds) {
            Err(Fault::Op(Undefined::Unknown(g))) => _ = unknown.insert(g),
            outcome if decided(&outcome) == Some(false) => {
                return Agreement::Fails {
                    at: point.clone(),
                    gives: None,
                }
            }
            _ => {}
        }
    }
    // Every value of a finite domain, each true: shown, one by one.
    let all = over
        .iter()
        .map(|(_, t)| size(t))
        .try_fold(1usize, |n, k| n.checked_mul(k?));
    if all.is_some_and(|all| all == tried) && unknown.is_empty() {
        return Agreement::Holds;
    }
    let why = match unknown.first() {
        Some(g) => format!("no value for `{g}` at some of the {tried} points tried"),
        None => format!("not decided by bounds over all of what it reads, nor at {tried} points"),
    };
    Agreement::Unknown { why }
}

/// How many values `t` has, when they are few enough to list.
fn size(t: &Type) -> Option<usize> {
    match t {
        Type::Truth => Some(2),
        Type::Record(r) => r
            .fields
            .iter()
            .map(|(_, t)| size(t))
            .try_fold(1usize, |n, k| n.checked_mul(k?)),
        Type::Int | Type::Real | Type::Param { .. } => None,
    }
}

/// Up to `n` points of `over`: `first`, then each name's candidates
/// combined, the smallest combinations first.
fn candidates(
    over: &[(String, Type)],
    holds: &Expression<Op>,
    first: &[Vec<(String, Value)>],
    n: usize,
) -> Vec<Vec<(String, Value)>> {
    let constants = constants(holds);
    let mut points: Vec<Vec<(String, Value)>> = first.iter().take(n).cloned().collect();
    let lists: Vec<Vec<Value>> = over.iter().map(|(_, t)| values_of(t, &constants)).collect();
    if lists.iter().any(Vec::is_empty) {
        return points;
    }
    // Index tuples by their sum, smallest first, until `n`.
    let mut seen: BTreeSet<Vec<Value>> = points
        .iter()
        .map(|p| p.iter().map(|(_, v)| v.clone()).collect())
        .collect();
    let longest = lists.iter().map(Vec::len).max().unwrap_or(0);
    let mut sum = 0;
    while points.len() < n && sum < longest * over.len().max(1) {
        let mut tuples = vec![vec![]];
        for list in &lists {
            tuples = tuples
                .into_iter()
                .flat_map(|t: Vec<usize>| {
                    (0..list.len()).map(move |i| [t.clone(), vec![i]].concat())
                })
                .filter(|t| t.iter().sum::<usize>() <= sum)
                .collect();
        }
        for t in tuples
            .into_iter()
            .filter(|t| t.iter().sum::<usize>() == sum)
        {
            let values: Vec<Value> = t.iter().zip(&lists).map(|(&i, l)| l[i].clone()).collect();
            if points.len() < n && seen.insert(values.clone()) {
                points.push(
                    over.iter()
                        .map(|(name, _)| name.clone())
                        .zip(values)
                        .collect(),
                );
            }
        }
        sum += 1;
    }
    points
}

/// The numbers an expression writes.
fn constants(e: &Expression<Op>) -> Vec<Rational> {
    let mut found = BTreeSet::new();
    let mut todo = vec![e];
    while let Some(e) = todo.pop() {
        if let Expression::Apply(op, args) = e {
            match op {
                Op::Int(n) => _ = found.insert(Rational::integer(n.clone())),
                Op::Real(q) => _ = found.insert(q.clone()),
                _ => {}
            }
            todo.extend(args.iter().map(|a| &a.body));
        }
    }
    found.into_iter().collect()
}

/// Values of `t` worth trying: small ones, the constants, and their
/// neighbours, in that order, each once.
fn values_of(t: &Type, constants: &[Rational]) -> Vec<Value> {
    let small = [0i64, 1, -1, 2, -2];
    let mut out: Vec<Value> = Vec::new();
    let mut add = |v: Value| {
        if !out.contains(&v) {
            out.push(v);
        }
    };
    match t {
        Type::Truth => {
            add(Value::Truth(false));
            add(Value::Truth(true));
        }
        Type::Int => {
            for n in small {
                add(Value::Int(n.into()));
            }
            for c in constants.iter().filter(|c| c.is_integer()) {
                let n = c.numer().clone();
                for d in [0i64, -1, 1] {
                    add(Value::Int(&n + d));
                }
            }
        }
        Type::Real => {
            for n in small {
                add(Value::Real(Rational::integer(n)));
            }
            add(Value::Real(Rational::new(1, 2).expect("2 is not 0")));
            for c in constants {
                for d in [0i64, -1, 1] {
                    add(Value::Real(c.plus(&Rational::integer(d))));
                }
            }
        }
        // A record: the i-th value of each field together.
        Type::Record(r) => {
            let fields: Vec<Vec<Value>> = r
                .fields
                .iter()
                .map(|(_, t)| values_of(t, constants))
                .collect();
            let most = fields.iter().map(Vec::len).max().unwrap_or(1).max(1);
            if fields.iter().all(|f| !f.is_empty()) {
                for i in 0..most {
                    let values = fields
                        .iter()
                        .map(|f| f[i.min(f.len() - 1)].clone())
                        .collect();
                    add(Value::Record(r.clone(), values));
                }
            }
        }
        Type::Param { .. } => {}
    }
    out
}

/// An example of the model: each output, computed from the inputs
/// through calls, a function with no body giving the value the example
/// says at its point.
fn model_example(
    index: &Index,
    library: &Arc<Calls>,
    i: usize,
    example: &ModelExample,
) -> Vec<Agreed> {
    let at = Path::to(&["examples", &i.to_string()]);
    let given: BTreeMap<&Path, &Value> = example.inputs.iter().map(|(p, v)| (p, v)).collect();
    // The arguments of the function at `f`, from the inputs.
    let args_of = |f: &Path, function: &Function| -> Option<Vec<Value>> {
        let args = function
            .takes
            .iter()
            .map(|(n, _)| given.get(&f.then(n)).map(|v| (*v).clone()));
        args.collect()
    };
    // A row for each function with no body whose value is given.
    let mut callees: BTreeMap<String, Callee> = library
        .iter()
        .map(|(n, c)| (n.clone(), c.clone()))
        .collect();
    for (p, v) in &example.inputs {
        let Some(port) = port(index, p) else { continue };
        if !port.gives || port.function.body.is_some() {
            continue;
        }
        let (Some(args), Some(callee)) = (
            args_of(&port.at, port.function),
            callees.get_mut(&port.at.to_string()),
        ) else {
            continue;
        };
        callee.rows.insert(0, (args, v.clone()));
    }
    let Ok(with_rows) = Calls::with(callees) else {
        return vec![];
    };
    let exact = Exact {
        calls: Arc::new(with_rows),
    };
    let mut agreed = Vec::new();
    for (p, expected) in &example.outputs {
        let Some(port) = port(index, p) else { continue };
        let Some(args) = args_of(&port.at, port.function) else {
            continue;
        };
        let names = port.function.takes.iter().map(|(n, _)| n.clone());
        let point: Vec<(String, Value)> = names.zip(args.clone()).collect();
        let call = Expression::Apply(
            Op::Call(port.at.to_string()),
            args.iter().map(|v| plain(literal(v))).collect(),
        );
        let agreement = match eval(&exact, &BTreeMap::new(), &call) {
            Ok(v) if &v == expected => Agreement::Holds,
            Ok(v) => Agreement::Fails {
                at: point,
                gives: Some(v),
            },
            Err(Fault::Op(Undefined::Unknown(g))) => Agreement::Unknown {
                why: format!("no value for `{g}` here"),
            },
            Err(_) => Agreement::Fails {
                at: point,
                gives: None,
            },
        };
        agreed.push(Agreed {
            at: at.clone(),
            against: format!("`{p}`"),
            agreement,
        });
    }
    agreed
}

/// An expression that gives `v`.
fn literal(v: &Value) -> Expression<Op> {
    let op = match v {
        Value::Truth(b) => Op::Truth(*b),
        Value::Int(n) => Op::Int(n.clone()),
        Value::Real(q) => Op::Real(q.clone()),
        Value::Record(r, values) => {
            return Expression::Apply(
                Op::Make(r.clone()),
                values.iter().map(|v| plain(literal(v))).collect(),
            )
        }
    };
    Expression::Apply(op, vec![])
}

/// The finding of a comparison that does not hold.
fn finding(a: &Agreed) -> Option<Finding> {
    let point = |at: &[(String, Value)]| {
        let shown: Vec<String> = at
            .iter()
            .map(|(n, v)| format!("{n} = {}", print::value(v)))
            .collect();
        shown.join(", ")
    };
    let gives = |g: &Option<Value>| match g {
        Some(v) => print::value(v),
        None => "no value".to_string(),
    };
    let (code, message): (Code, String) = match &a.agreement {
        Agreement::Holds => return None,
        Agreement::Unknown { why } => (A007, format!("{}: unknown, {why}", a.against)),
        Agreement::Fails { at, gives: g } => {
            let against = &a.against;
            if a.at.names.first().map(String::as_str) == Some("examples") {
                (
                    A006,
                    format!("{against}: the model gives {} at {}", gives(g), point(at)),
                )
            } else if a.at.names.len() == 4 {
                (A005, format!("false at {}", point(at)))
            } else if against == "its claims together" {
                (A004, "no value keeps all its claims, by bounds".to_string())
            } else if against.contains(" against claim") {
                (
                    A003,
                    format!("{against}: false at {}, giving {}", point(at), gives(g)),
                )
            } else if against.starts_with("example") {
                (
                    A002,
                    format!("{against}: the body gives {} at {}", gives(g), point(at)),
                )
            } else {
                (A001, format!("the body breaks {against} at {}", point(at)))
            }
        }
    };
    Some(Finding {
        code,
        at: a.at.clone(),
        message,
    })
}
