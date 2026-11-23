//! Agree (`docs/agree.md`): is the model sound on itself? With no
//! evidence, it compares what the model says twice: each function's
//! body with its claims and its examples, its examples with its claims
//! and with each other, its claims together, each contract's claim read
//! of a function that fits it, and each example of the model with what
//! the model computes, through calls.
//!
//! A comparison `Holds`, `Fails` at a point, or is `Unknown`. It holds
//! when `Ranges`, over all of what it reads, says surely true, or when
//! its domain is finite and every point was tried; it fails at a point
//! `Exact` confirms, or, for claims no value keeps together, by bounds or
//! at every point; else it is unknown, and says how far it looked.
//! Nothing is solved: no SMT, by your choice; points are tried up to
//! `Settings::points`. Every failure and every unknown is a finding of
//! suit `A`; nothing is dropped.
//!
//! It reads a model `check` accepts: on another, it compares nothing.

use crate::catalogue::{Code, Finding, A001, A002, A003, A004, A005, A006, A007, A009};
use crate::evidence::port;
use crate::expression::{eval, Arg, Expression, Fault};
use crate::model::{calls, contract_claims, Function, Index, Model, ModelExample, Path};
use crate::print;
use crate::simulation::{
    Bounds, Budget, Calls, Exact, Op, Ranges, Rational, Span, Type, Undefined, Value,
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
    /// The most numbers bounds cut at zero, each read below zero and
    /// above it: `x * x` is never negative on either side. `n` of them
    /// is `2^n` readings by bounds per comparison.
    pub splits: usize,
    /// How far one computation through calls may go: past it, the
    /// comparison is unknown, and says which budget it reached.
    pub budget: Budget,
}

/// 64 points per comparison: every example row, and for one or two
/// arguments the small numbers and constants of a typical claim. 4
/// numbers cut at zero: 16 readings by bounds, enough for the squares and
/// signs of a function of a few numbers.
impl Default for Settings {
    fn default() -> Settings {
        Settings {
            points: 64,
            splits: 4,
            budget: Budget::default(),
        }
    }
}

/// What a comparison found.
#[derive(Clone, Debug, PartialEq)]
pub enum Agreement {
    Holds,
    /// At these values of what it reads, the model gives `gives` (none:
    /// a fault), and the comparison is false there. Confirmed by `Exact`.
    /// For claims no value keeps together, no point: false at all.
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
/// hold, and for each input of an example of the model nothing reads.
pub fn agree(model: &Model, settings: &Settings) -> (Vec<Agreed>, Vec<Finding>) {
    let Ok(library) = calls(model) else {
        return (vec![], vec![]);
    };
    let index = Index::of(model);
    let library = library.budget(settings.budget);
    let reading = Reading::new(&index, Arc::new(library), *settings);
    let mut agreed = Vec::new();
    for part in &model.parts {
        for f in &part.functions {
            let at = Path::to(&[&part.name, &f.name]);
            reading.function(&at, f, &mut agreed);
        }
    }
    for c in contract_claims(model) {
        let agreement = reading.claim(&c.over, &c.holds, &[]);
        agreed.push(Agreed {
            at: c.at,
            against: "every value of what it reads".to_string(),
            agreement,
        });
    }
    let mut findings = Vec::new();
    for (i, example) in model.examples.iter().enumerate() {
        let (compared, unread) = reading.model_example(&index, i, example);
        agreed.extend(compared);
        findings.extend(unread);
    }
    findings.extend(agreed.iter().filter_map(finding));
    findings.sort_by(|a, b| (&a.at, a.code, &a.message).cmp(&(&b.at, b.code, &b.message)));
    (agreed, findings)
}

/// How agree reads the model: exactly and by bounds, through its calls,
/// knowing its opaque records and what each function says.
struct Reading<'m> {
    exact: Exact,
    ranges: Ranges,
    library: Arc<Calls>,
    /// The records no value of which is written. Simulation reads one as
    /// a record of no field, a single value, so bounds would find any two
    /// equal: agree reads no bounds over one, and makes none of its values.
    opaque: BTreeSet<&'m str>,
    functions: BTreeMap<String, &'m Function>,
    settings: Settings,
}

impl<'m> Reading<'m> {
    fn new(index: &Index<'m>, library: Arc<Calls>, settings: Settings) -> Reading<'m> {
        let opaque = index
            .records
            .iter()
            .filter(|(_, r)| r.fields.is_none())
            .map(|(name, _)| *name)
            .collect();
        Reading {
            exact: Exact {
                calls: library.clone(),
            },
            ranges: Ranges {
                calls: library.clone(),
            },
            library,
            opaque,
            functions: index.functions.clone(),
            settings,
        }
    }

    /// The comparisons of one function.
    fn function(&self, at: &Path, f: &Function, agreed: &mut Vec<Agreed>) {
        let rows: Vec<Vec<(String, Value)>> = f
            .examples
            .iter()
            .map(|e| named(&f.takes, &e.takes))
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
                let agreement = self.claim(&f.takes, &with_body, &rows);
                push(format!("claim `{}`", c.name), agreement);
            }
            // Each example: the body at its point.
            for (i, (e, row)) in f.examples.iter().zip(&rows).enumerate() {
                let agreement = match eval(&self.exact, &row.iter().cloned().collect(), body) {
                    Ok(v) if v == e.gives => Agreement::Holds,
                    Ok(v) => Agreement::Fails {
                        at: row.clone(),
                        gives: Some(v),
                    },
                    Err(Fault::Op(Undefined::Unknown(g))) => Agreement::Unknown {
                        why: format!("no value for `{g}` here"),
                    },
                    Err(Fault::Op(Undefined::Budget(why))) => Agreement::Unknown { why },
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
                let agreement = match eval(&self.exact, &values, &c.holds) {
                    Ok(Value::Truth(true)) => Agreement::Holds,
                    Err(Fault::Op(Undefined::Unknown(g))) => Agreement::Unknown {
                        why: format!("no value for `{g}` here"),
                    },
                    Err(Fault::Op(Undefined::Budget(why))) => Agreement::Unknown { why },
                    // False, or no value: a claim with none does not hold.
                    _ => Agreement::Fails {
                        at: row.clone(),
                        gives: Some(e.gives.clone()),
                    },
                };
                push(format!("example {i} against claim `{}`", c.name), agreement);
            }
        }
        // Two examples at one point that differ: check refuses them (M024).
        // Some value keeps all its claims.
        if !f.claims.is_empty() {
            push("its claims together".to_string(), self.together(f, &rows));
        }
    }

    /// Whether `holds`, reading `over`, is true at every value of it: by
    /// bounds over all of them, then at points, `first` first.
    fn claim(
        &self,
        over: &[(String, Type)],
        holds: &Expression<Op>,
        first: &[Vec<(String, Value)>],
    ) -> Agreement {
        let bounded = self.bounded(over, holds);
        if bounded && self.surely(over, holds) == Some(true) {
            return Agreement::Holds;
        }
        let (mut tried, mut unknown) = (0, BTreeSet::new());
        for point in self.candidates(over, holds, first) {
            tried += 1;
            match eval(&self.exact, &point.iter().cloned().collect(), holds) {
                Ok(Value::Truth(true)) => {}
                Err(Fault::Op(Undefined::Unknown(g))) => _ = unknown.insert(g),
                Err(Fault::Op(Undefined::Budget(why))) => return Agreement::Unknown { why },
                Ok(v) => {
                    return Agreement::Fails {
                        at: point,
                        gives: Some(v),
                    }
                }
                // No value: a claim with none does not hold.
                Err(_) => {
                    return Agreement::Fails {
                        at: point,
                        gives: None,
                    }
                }
            }
        }
        // Every value of a finite domain, each true: shown, one by one.
        if self.size(over) == Some(tried) && unknown.is_empty() {
            return Agreement::Holds;
        }
        let why = match unknown.first() {
            Some(g) => format!("no value for `{g}` at some of the {tried} points tried"),
            None if bounded => format!(
                "not decided by bounds over all of what it reads, nor at {tried} points"
            ),
            None => format!("it reads an opaque record, which bounds cannot tell apart, and no value of which is written: not decided at {tried} points"),
        };
        Agreement::Unknown { why }
    }

    /// Whether some value keeps all the claims of `f` together. False by
    /// bounds whatever the values, or at every point of a finite domain;
    /// true at a point where all hold: its examples first, then what its
    /// body gives, then candidates; else unknown.
    fn together(&self, f: &Function, rows: &[Vec<(String, Value)>]) -> Agreement {
        let over: Vec<(String, Type)> = f.takes.iter().chain([&f.gives]).cloned().collect();
        let Some(all) = f
            .claims
            .iter()
            .map(|c| c.holds.clone())
            .reduce(|a, b| Expression::Apply(Op::And, vec![plain(a), plain(b)]))
        else {
            return Agreement::Holds;
        };
        let bounded = self.bounded(&over, &all);
        if bounded && self.surely(&over, &all) == Some(false) {
            return Agreement::Fails {
                at: vec![],
                gives: None,
            };
        }
        let gives = |point: &[(String, Value)], v: &Value| {
            let mut point = point.to_vec();
            point.push((f.gives.0.clone(), v.clone()));
            point
        };
        let mut first: Vec<Vec<(String, Value)>> = f
            .examples
            .iter()
            .zip(rows)
            .map(|(e, row)| gives(row, &e.gives))
            .collect();
        let keeps = |point: &[(String, Value)]| {
            eval(&self.exact, &point.iter().cloned().collect(), &all) == Ok(Value::Truth(true))
        };
        if first.iter().any(|point| keeps(point)) {
            return Agreement::Holds;
        }
        // What the body gives, point by point: the first that keeps them
        // all ends the search, before the body is read anywhere else.
        if let Some(body) = &f.body {
            for point in self.candidates(&f.takes, body, rows) {
                if let Ok(v) = eval(&self.exact, &point.iter().cloned().collect(), body) {
                    let point = gives(&point, &v);
                    if keeps(&point) {
                        return Agreement::Holds;
                    }
                    first.push(point);
                }
            }
        }
        let (mut tried, mut unknown) = (0, BTreeSet::new());
        for point in self.candidates(&over, &all, &first) {
            tried += 1;
            match eval(&self.exact, &point.iter().cloned().collect(), &all) {
                Ok(Value::Truth(true)) => return Agreement::Holds,
                Err(Fault::Op(Undefined::Unknown(g))) => _ = unknown.insert(g),
                Err(Fault::Op(Undefined::Budget(why))) => return Agreement::Unknown { why },
                _ => {}
            }
        }
        // Every value of a finite domain, none keeping them all.
        if self.size(&over) == Some(tried) && unknown.is_empty() {
            return Agreement::Fails {
                at: vec![],
                gives: None,
            };
        }
        let why = match unknown.first() {
            Some(g) => format!("no value for `{g}` at some of the {tried} points tried"),
            None if bounded => format!(
                "not false by bounds, and none of {tried} points keeps them all"
            ),
            None => format!("it reads an opaque record, which bounds cannot tell apart, and no value of which is written: none of {tried} points keeps them all"),
        };
        Agreement::Unknown { why }
    }

    /// What bounds over every value of `over` say of `e`, when sure: the
    /// same in every reading, its first numbers (up to `splits`) each cut
    /// at zero. In a reading where the comparisons `e` joins by `&&` keep
    /// no value of a name, `e` is false there.
    fn surely(&self, over: &[(String, Type)], e: &Expression<Op>) -> Option<bool> {
        let all: BTreeMap<String, Bounds> = over
            .iter()
            .map(|(n, t)| (n.clone(), Bounds::any(t)))
            .collect();
        let numbers: Vec<&String> = over
            .iter()
            .filter(|(_, t)| matches!(t, Type::Int | Type::Real))
            .map(|(n, _)| n)
            .take(self.settings.splits)
            .collect();
        let zero = || Some(Rational::integer(0));
        let mut readings = vec![all];
        for n in numbers {
            readings = readings
                .into_iter()
                .flat_map(|r| {
                    [Span::Number(None, zero()), Span::Number(zero(), None)].map(|span| {
                        let mut r = r.clone();
                        r.insert(
                            n.clone(),
                            Bounds {
                                span,
                                may_fault: false,
                            },
                        );
                        r
                    })
                })
                .collect();
        }
        let mut sure = None;
        for bounds in readings {
            let here = match self.ranges.kept(over, &bounds, e) {
                None => Some(false),
                Some(_) => eval(&self.ranges, &bounds, e).ok().and_then(|b| b.surely()),
            };
            match (here, sure) {
                (None, _) => return None,
                (Some(h), Some(s)) if h != s => return None,
                (h, _) => sure = h,
            }
        }
        sure
    }

    /// Whether bounds may be read of `e` over `over`: no value of an
    /// opaque record is read, there or through the calls `e` makes.
    fn bounded(&self, over: &[(String, Type)], e: &Expression<Op>) -> bool {
        let signature = |f: &Function| -> bool {
            f.takes
                .iter()
                .chain([&f.gives])
                .any(|(_, t)| self.opaque_in(t))
        };
        !over.iter().any(|(_, t)| self.opaque_in(t))
            && !self
                .called(e, true)
                .iter()
                .any(|f| self.functions.get(f).is_some_and(|f| signature(f)))
    }

    /// Whether a value of `t` holds a value of an opaque record.
    fn opaque_in(&self, t: &Type) -> bool {
        match t {
            Type::Record(r) => {
                self.opaque.contains(r.name.as_str())
                    || r.fields.iter().any(|(_, t)| self.opaque_in(t))
            }
            Type::Truth | Type::Int | Type::Real | Type::Param { .. } => false,
        }
    }

    /// The functions `e` calls, however far: through bodies, and through
    /// claims too when `claims` (bounds read the claims of a function
    /// with no body).
    fn called(&self, e: &Expression<Op>, claims: bool) -> BTreeSet<String> {
        let mut found = BTreeSet::new();
        let mut todo = vec![e];
        while let Some(e) = todo.pop() {
            let Expression::Apply(op, args) = e else {
                continue;
            };
            if let Op::Call(f) = op {
                if found.insert(f.clone()) {
                    if let Some(function) = self.functions.get(f) {
                        todo.extend(function.body.iter());
                        if claims {
                            todo.extend(function.claims.iter().map(|c| &c.holds));
                        }
                    }
                }
            }
            todo.extend(args.iter().map(|a| &a.body));
        }
        found
    }

    /// The numbers `e` writes, and those of the functions it calls,
    /// however far.
    fn constants(&self, e: &Expression<Op>) -> Vec<Rational> {
        let mut todo = vec![e];
        for f in self.called(e, true) {
            if let Some(function) = self.functions.get(&f) {
                todo.extend(function.body.iter());
                todo.extend(function.claims.iter().map(|c| &c.holds));
            }
        }
        let mut found = BTreeSet::new();
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

    /// How many values `over` has together, when few enough to list.
    fn size(&self, over: &[(String, Type)]) -> Option<usize> {
        over.iter()
            .map(|(_, t)| self.size_of(t))
            .try_fold(1usize, |n, k| n.checked_mul(k?))
    }

    /// How many values `t` has, when few enough to list: an opaque
    /// record's are unknown.
    fn size_of(&self, t: &Type) -> Option<usize> {
        match t {
            Type::Truth => Some(2),
            Type::Record(r) if self.opaque.contains(r.name.as_str()) => None,
            Type::Record(r) => r
                .fields
                .iter()
                .map(|(_, t)| self.size_of(t))
                .try_fold(1usize, |n, k| n.checked_mul(k?)),
            Type::Int | Type::Real | Type::Param { .. } => None,
        }
    }

    /// Up to `Settings::points` points of `over`, each once: `first`,
    /// then each name's values combined, the smallest combinations first.
    /// With nothing to read, the one point `()`.
    fn candidates(
        &self,
        over: &[(String, Type)],
        e: &Expression<Op>,
        first: &[Vec<(String, Value)>],
    ) -> Vec<Vec<(String, Value)>> {
        let n = self.settings.points;
        let mut points: Vec<Vec<(String, Value)>> = Vec::new();
        let mut seen: BTreeSet<Vec<Value>> = BTreeSet::new();
        for point in first {
            let values = point.iter().map(|(_, v)| v.clone()).collect();
            if points.len() < n && seen.insert(values) {
                points.push(point.clone());
            }
        }
        let constants = self.constants(e);
        let lists: Vec<Vec<Value>> = over
            .iter()
            .map(|(_, t)| self.values_of(t, &constants))
            .collect();
        if lists.iter().any(Vec::is_empty) {
            return points;
        }
        // Index tuples by their sum, smallest first, until `n`.
        let longest = lists.iter().map(Vec::len).max().unwrap_or(0);
        let mut sum = 0;
        while points.len() < n && (sum == 0 || sum < longest * over.len()) {
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

    /// Values of `t` worth trying: small ones, the constants, and their
    /// neighbours, in that order, each once. Of a record, every value
    /// when they are at most `Settings::points`, else the i-th value of
    /// each field together; of an opaque record, none.
    fn values_of(&self, t: &Type, constants: &[Rational]) -> Vec<Value> {
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
            Type::Record(r) if self.opaque.contains(r.name.as_str()) => {}
            Type::Record(r) => {
                let fields: Vec<Vec<Value>> = r
                    .fields
                    .iter()
                    .map(|(_, t)| self.values_of(t, constants))
                    .collect();
                if fields.iter().any(Vec::is_empty) {
                    return out;
                }
                if self.size_of(t).is_some_and(|k| k <= self.settings.points) {
                    for values in product(&fields) {
                        add(Value::Record(r.clone(), values));
                    }
                } else {
                    let most = fields.iter().map(Vec::len).max().unwrap_or(1).max(1);
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
    /// through calls, a function with no body giving the value the
    /// example says at its point; each such value against the function's
    /// own examples there; and an A009 for each input no output reads.
    fn model_example(
        &self,
        index: &Index,
        i: usize,
        example: &ModelExample,
    ) -> (Vec<Agreed>, Vec<Finding>) {
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
        let mut agreed = Vec::new();
        // What each function with no body is given, read first.
        let mut library = (*self.library).clone();
        for (p, v) in &example.inputs {
            let Some(port) = port(index, p) else { continue };
            if !port.gives || port.function.body.is_some() {
                continue;
            }
            let Some(args) = args_of(&port.at, port.function) else {
                continue;
            };
            // What the function's own examples say there.
            for (j, e) in port.function.examples.iter().enumerate() {
                if e.takes != args {
                    continue;
                }
                let agreement = match e.gives == *v {
                    true => Agreement::Holds,
                    false => Agreement::Fails {
                        at: named(&port.function.takes, &args),
                        gives: Some(e.gives.clone()),
                    },
                };
                agreed.push(Agreed {
                    at: at.clone(),
                    against: format!("`{p}` against example {j} of `{}`", port.at),
                    agreement,
                });
            }
            // Refused only for a value of another type, which M022 refuses.
            library = library
                .given(&port.at.to_string(), args, v.clone())
                .unwrap_or_else(|same| same);
        }
        let exact = Exact {
            calls: Arc::new(library),
        };
        for (p, expected) in &example.outputs {
            let Some(port) = port(index, p) else { continue };
            let Some(args) = args_of(&port.at, port.function) else {
                continue;
            };
            let point = named(&port.function.takes, &args);
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
                Err(Fault::Op(Undefined::Budget(why))) => Agreement::Unknown { why },
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
        (agreed, self.unread(index, &at, example))
    }

    /// A009 for each input of `example` no output reads: what an output's
    /// function takes, a value of the world its computation calls for,
    /// and what that function takes, are read; nothing else is.
    fn unread(&self, index: &Index, at: &Path, example: &ModelExample) -> Vec<Finding> {
        let outputs: BTreeSet<String> = example
            .outputs
            .iter()
            .filter_map(|(p, _)| port(index, p))
            .map(|port| port.at.to_string())
            .collect();
        let mut reached = outputs.clone();
        for f in &outputs {
            if let Some(body) = self.functions.get(f).and_then(|f| f.body.as_ref()) {
                reached.extend(self.called(body, false));
            }
        }
        let given: BTreeSet<&Path> = example.inputs.iter().map(|(p, _)| p).collect();
        let mut findings = Vec::new();
        for (p, _) in &example.inputs {
            let Some(port) = port(index, p) else { continue };
            let f = port.at.to_string();
            let world = port.function.body.is_none()
                && given.contains(&port.at.then(&port.function.gives.0));
            let why = if !reached.contains(&f) {
                format!("no output calls `{f}`")
            } else if port.gives || outputs.contains(&f) || world {
                continue;
            } else if port.function.body.is_some() {
                format!("the model computes what `{f}` takes where it is called")
            } else {
                format!("no value `{f}` gives is given with it")
            };
            findings.push(Finding {
                code: A009,
                at: at.clone(),
                message: format!("`{p}` is given and no output reads it: {why}"),
            });
        }
        findings
    }
}

/// The names of `takes` with `values`, in order.
fn named(takes: &[(String, Type)], values: &[Value]) -> Vec<(String, Value)> {
    takes
        .iter()
        .map(|(n, _)| n.clone())
        .zip(values.iter().cloned())
        .collect()
}

/// Every way to take one value of each list, the first list slowest.
fn product(lists: &[Vec<Value>]) -> Vec<Vec<Value>> {
    lists.iter().fold(vec![vec![]], |ways, list| {
        ways.into_iter()
            .flat_map(|way: Vec<Value>| {
                list.iter()
                    .map(move |v| [way.clone(), vec![v.clone()]].concat())
            })
            .collect()
    })
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
        match shown.is_empty() {
            true => "()".to_string(),
            false => shown.join(", "),
        }
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
                match g {
                    Some(_) => (A005, format!("false at {}", point(at))),
                    None => (A005, format!("no value at {}", point(at))),
                }
            } else if against == "its claims together" {
                (A004, "no value keeps all its claims".to_string())
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
                let none = if g.is_none() {
                    ", where it has no value"
                } else {
                    ""
                };
                (
                    A001,
                    format!("the body breaks {against} at {}{none}", point(at)),
                )
            }
        }
    };
    Some(Finding {
        code,
        at: a.at.clone(),
        message,
    })
}
