//! Agree (`docs/design/agree.md`): is the model sound on itself? With no
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

use crate::catalogue::{
    Code, Finding, Hint, A001, A002, A003, A004, A005, A006, A007, A009, A010, A011, A012, A013, A014, A015, A016, A017,
};
use crate::evidence::port;
use crate::expression::{eval, Arg, Expression, Fault};
use crate::model::{calls, contract_claims, Function, Index, Model, ModelExample, Path};
use crate::print;
use crate::simulation::{
    call_at, matches, Bounds, Budget, Calls, Exact, Numbers, Op, Range, Ranges, Rational, Span,
    Type, Undefined, Value,
};
use num_bigint::BigInt;
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
    /// The longest sequence tried as a point: from `[]` up to this
    /// length (finding 3, step 2).
    pub length: usize,
    /// The tallest tree tried as a point: a tree in itself at most this
    /// many levels deep (trees, the user's go 12:33).
    pub height: usize,
}

/// What the expressions write, worth trying as points.
struct Constants {
    numbers: Vec<Rational>,
    texts: Vec<String>,
}

/// 64 points per comparison: every example row, and for one or two
/// arguments the small numbers and constants of a typical claim. 4
/// numbers cut at zero: 16 readings by bounds, enough for the squares and
/// signs of a function of a few numbers. Sequences up to 3 long: empty,
/// one value, and two or three to compare.
impl Default for Settings {
    fn default() -> Settings {
        Settings {
            points: 64,
            splits: 4,
            budget: Budget::default(),
            length: 3,
            height: 3,
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
    /// Not decided, because at these values an `until` came back to
    /// values it had carried: it never ends (finding 39 G). Where, and
    /// the state that came back.
    Endless {
        at: Vec<(String, Value)>,
        fault: Undefined,
    },
    /// An example that fails for a reason of its own (finding 39 B): its
    /// code, and why, as said after the example's name.
    Wrong {
        code: Code,
        why: String,
    },
    /// A call a row's `then` lists, which the run did not make (BDD step
    /// 2): the call as written, and the calls to functions with no body
    /// it made, in order.
    Uncalled {
        call: String,
        made: Vec<String>,
    },
}

/// A point, as findings say it: `n = 3, m = 4`, or `()`.
fn point_of(at: &[(String, Value)]) -> String {
    let shown: Vec<String> = at
        .iter()
        .map(|(n, v)| format!("{n} = {}", print::value(v)))
        .collect();
    match shown.is_empty() {
        true => "()".to_string(),
        false => shown.join(", "),
    }
}

/// What a comparison is when an `until` never ends at `at`.
fn endless(at: &[(String, Value)], e: &Undefined) -> Agreement {
    Agreement::Endless {
        at: at.to_vec(),
        fault: e.clone(),
    }
}

/// One comparison: of the item at `at`, against what, and its outcome.
#[derive(Clone, Debug, PartialEq)]
pub struct Agreed {
    pub at: Path,
    pub against: String,
    pub agreement: Agreement,
    /// What the outcome is given: the claims of functions with no body
    /// that bounds read as bounds on what they give, by path. Empty for
    /// an outcome that needs none.
    pub given: Vec<String>,
}

/// Every comparison of the model, and a finding for each that does not
/// hold, and for each input of an example of the model nothing reads.
pub fn agree(model: &Model, settings: &Settings) -> (Vec<Agreed>, Vec<Finding>) {
    let Ok(library) = calls(model) else {
        return (vec![], vec![]);
    };
    let index = Index::of(model);
    let library = library.budget(settings.budget);
    let mut reading = Reading::new(&index, Arc::new(library), *settings);
    for (claim, calls) in crate::model::hypotheses(model) {
        reading
            .induction
            .insert(claim, Arc::new(calls.budget(settings.budget)));
    }
    for (claim, calls) in crate::model::invariants(model) {
        reading
            .invariant
            .insert(claim, Arc::new(calls.budget(settings.budget)));
    }
    let mut agreed = Vec::new();
    for part in &model.parts {
        for f in &part.functions {
            let at = Path::to(&[&part.name, &f.name]);
            reading.function(&at, f, &mut agreed);
        }
    }
    // Each value written in a row, against the claims of each record
    // in it (finding 10b): a row is where the world's values enter.
    for part in &model.parts {
        for f in &part.functions {
            let at = Path::to(&[&part.name, &f.name]);
            for (i, e) in f.examples.iter().enumerate() {
                let named = f.takes.iter().map(|(n, _)| n.clone()).zip(e.takes.iter());
                let gives = e.gives.iter().map(|v| (f.gives.0.clone(), v));
                let mut values: Vec<(String, &Value)> = named.chain(gives).collect();
                values.extend(world_values(&e.given, &e.effects));
                for (what, v) in values {
                    reading.row_value(&at, &format!("example {i}: `{what}`"), v, &mut agreed);
                }
            }
        }
    }
    for (i, e) in model.examples.iter().enumerate() {
        let at = Path::to(&["examples", &i.to_string()]);
        let ports = e
            .inputs
            .iter()
            .chain(&e.outputs)
            .map(|(p, v)| (p.to_string(), v));
        let mut values: Vec<(String, &Value)> = ports.collect();
        values.extend(world_values(&e.given, &e.effects));
        if let Some((f, args, gives)) = &e.when {
            values.extend(args.iter().map(|v| (format!("{f}(…)"), v)));
            values.extend(gives.iter().map(|v| (format!("{f}"), v)));
        }
        for (what, v) in values {
            reading.row_value(&at, &format!("`{what}`"), v, &mut agreed);
        }
    }
    for c in contract_claims(model) {
        let agreement = reading.claim(&c.over, &c.holds, &[]);
        agreed.push(Agreed {
            at: c.at,
            against: CONTRACT.to_string(),
            agreement,
            given: reading.given(),
        });
    }
    let mut findings = Vec::new();
    for (i, example) in model.examples.iter().enumerate() {
        let (compared, unread) = reading.model_example(&index, i, example);
        agreed.extend(compared);
        findings.extend(unread);
    }
    findings.extend(laws_broken_everywhere(&agreed));
    findings.extend(
        agreed
            .iter()
            .filter(|a| !everywhere_broken(&agreed, a))
            .filter_map(finding),
    );
    findings.sort_by(|a, b| (&a.at, a.code, &a.message).cmp(&(&b.at, b.code, &b.message)));
    (agreed, findings)
}

/// How agree reads the model: exactly and by bounds, through its calls,
/// knowing its opaque records and what each function says.
struct Reading<'m> {
    exact: Exact,
    /// Bounds: through the library, or, for a claim shown by induction,
    /// through the library where the function says only that claim.
    ranges: std::cell::RefCell<Ranges>,
    /// For each claim shown `by induction` (`Function::Claim`'s path),
    /// the library where its function has no body and promises it: the
    /// hypothesis on its parts (trees, the user's go 12:33).
    induction: BTreeMap<String, Arc<Calls>>,
    /// For each claim `by invariant`, the library where one more
    /// function gives any value its loop carries that keeps the claim.
    invariant: BTreeMap<String, Arc<Calls>>,
    library: Arc<Calls>,
    /// The records no value of which is written. Simulation reads one as
    /// a record of no field, a single value, so bounds would find any two
    /// equal: agree reads no bounds over one, and makes none of its values.
    opaque: BTreeSet<&'m str>,
    functions: BTreeMap<String, &'m Function>,
    /// The records of the model, by name: their claims hold of each
    /// value a function of the model gives (finding 10b).
    records: BTreeMap<&'m str, &'m crate::model::Record>,
    settings: Settings,
    /// What the last outcome decided by bounds was given.
    last_given: std::cell::RefCell<Vec<String>>,
    /// The ranges of the names of the function read now: points are
    /// tried in them only, every one when they are few enough.
    domains: std::cell::RefCell<BTreeMap<String, Range>>,
    /// The budget a reading by bounds reached since the last claim began,
    /// said in what agree says of it (finding 24): never a silent fall
    /// back to points.
    bounds_budget: std::cell::RefCell<Option<String>>,
}

impl<'m> Reading<'m> {
    /// What the last outcome was given, taken: empty when it needed none.
    fn given(&self) -> Vec<String> {
        std::mem::take(&mut *self.last_given.borrow_mut())
    }

    /// `surely`, keeping what the bounds read was given when sure.
    fn surely_given(&self, over: &[(String, Type)], e: &Expression<Op>) -> Option<bool> {
        self.library.read();
        let sure = self.surely(over, e);
        let read = self.library.read();
        if sure.is_some() {
            *self.last_given.borrow_mut() = read.into_iter().collect();
        }
        sure
    }

    fn new(index: &Index<'m>, library: Arc<Calls>, settings: Settings) -> Reading<'m> {
        let opaque = index
            .records
            .iter()
            .filter(|(_, r)| r.fields.is_none())
            .map(|(name, _)| *name)
            // The fields a record with `..` does not state: no value.
            .chain([crate::model::REST])
            .collect();
        Reading {
            exact: Exact {
                calls: library.clone(),
                log: Default::default(),
            },
            ranges: std::cell::RefCell::new(Ranges {
                calls: library.clone(),
            }),
            induction: BTreeMap::new(),
            invariant: BTreeMap::new(),
            library,
            opaque,
            functions: index.functions.clone(),
            records: index.records.clone(),
            settings,
            last_given: Default::default(),
            domains: Default::default(),
            bounds_budget: Default::default(),
        }
    }

    /// The model's functions, with what the environment gave in `world`
    /// read first (finding 16); as they are, in none.
    fn in_world(&self, world: &crate::model::Given) -> Exact {
        if world.0.is_empty() {
            return self.exact.clone();
        }
        let mut calls = (*self.exact.calls).clone();
        for (g, args, v) in &world.0 {
            calls = calls
                .given_any(g, args.clone(), v.clone())
                .unwrap_or_else(|same| same);
        }
        Exact {
            calls: std::sync::Arc::new(calls),
            log: Default::default(),
        }
    }

    /// The comparisons of one function.
    fn function(&self, at: &Path, f: &Function, agreed: &mut Vec<Agreed>) {
        let ranges = f
            .ranges
            .takes
            .iter()
            .chain(f.ranges.gives.iter().map(|(q, r)| (q, r)));
        *self.domains.borrow_mut() = ranges.map(|(n, r)| (n.clone(), r.clone())).collect();
        self.function_in(at, f, agreed);
        self.domains.borrow_mut().clear();
    }

    fn function_in(&self, at: &Path, f: &Function, agreed: &mut Vec<Agreed>) {
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
                given: self.given(),
            })
        };
        // Its ranges of what it takes: a claim is read where they hold.
        // And the claims of each record it takes, of what it is given
        // (finding 10b): every value of a record enters the model where
        // they are read, so what takes one may count on them.
        let mut promised: Vec<Expression<Op>> = Vec::new();
        for (n, t) in &f.takes {
            self.promises(&Expression::Name(n.clone()), t, 0, &mut promised);
        }
        // Bounds read one value at a time: what relates two fields is
        // decided, where it can be, as linear facts.
        let linear = |agreement: Agreement, holds: &Expression<Op>| match agreement {
            Agreement::Unknown { .. }
                if !promised.is_empty() && crate::linear::holds(&promised, holds) =>
            {
                Agreement::Holds
            }
            agreement => agreement,
        };
        let takes_in = conjunction(
            f.ranges
                .takes
                .iter()
                .map(|(n, r)| in_range(n, r))
                .chain(promised.iter().cloned()),
        );
        let read_in = |holds: Expression<Op>| match &takes_in {
            Some(r) => Expression::Apply(
                Op::Or,
                vec![
                    plain(Expression::Apply(Op::Not, vec![plain(r.clone())])),
                    plain(holds),
                ],
            ),
            None => holds,
        };
        if let Some(body) = &f.body {
            // Each claim, with the body read in place of what it gives.
            for c in &f.claims {
                let with_body = let_in(&f.gives.0, body, &read_in(c.holds.clone()));
                // `by invariant`: its start and a turn, each as a claim.
                let kept =
                    c.invariant
                        .as_ref()
                        .map(|_| match loop_claims(at, f, c, &self.library) {
                            Some(Loop {
                                start,
                                turn,
                                linear,
                            }) => {
                                // By bounds and points, else as linear facts.
                                let mut start_kept =
                                    self.claim(&start.0, &read_in(start.1.clone()), &rows);
                                if crate::linear::holds(&[], &start.1) {
                                    start_kept = Agreement::Holds;
                                }
                                let mut turn_kept =
                                    self.by_carrier(&at.then(&c.name), &turn.0, &read_in(turn.1));
                                if crate::linear::holds(&linear.0, &linear.1) {
                                    turn_kept = Agreement::Holds;
                                }
                                kept(start_kept, turn_kept)
                            }
                            None => {
                                Err("its loop's values are named as the function's inputs"
                                    .to_string())
                            }
                        });
                let agreement = match self.by_induction(&at.then(&c.name), &f.takes, &with_body) {
                    true => Agreement::Holds,
                    false if matches!(kept, Some(Ok(()))) => Agreement::Holds,
                    false => match (self.claim(&f.takes, &with_body, &rows), kept) {
                        (Agreement::Unknown { why }, Some(Err(not))) => Agreement::Unknown {
                            why: format!("{why}; `by invariant`: {not}"),
                        },
                        (agreement, _) => linear(agreement, &with_body),
                    },
                };
                push(format!("claim `{}`", c.name), agreement);
            }
            // The claims of the record it gives, of the value it gives
            // (finding 10b): what the model makes keeps them.
            if let Type::Record(r) = &f.gives.1 {
                // No claim of a record is by induction or invariant
                // (M031, M033).
                let claims = self.records.get(r.name.as_str()).map(|r| &r.claims);
                for c in claims.into_iter().flatten() {
                    // Its ranges and promises read outside: `self` in
                    // them is what it takes, here what it gives.
                    let with_body = read_in(let_in("self", body, &c.holds));
                    let agreement = self.claim(&f.takes, &with_body, &rows);
                    let agreement = linear(agreement, &with_body);
                    push(
                        format!("claim `{}::{}` of what it gives", r.name, c.name),
                        agreement,
                    );
                }
            }
            // Each record the body makes on its way, as an argument or
            // in a `let` (finding 10b): what takes it counts on its
            // claims, so they are read where it is made.
            let gives = match &f.gives.1 {
                Type::Record(r) => Some(r.name.as_str()),
                _ => None,
            };
            let mut made = Vec::new();
            constructions(body, gives, true, &[], &[], &BTreeSet::new(), &mut made);
            for m in made {
                let Some(record) = self.records.get(m.record.name.as_str()) else {
                    continue;
                };
                for c in &record.claims {
                    let against = format!(
                        "claim `{}::{}` of the `{}` it makes",
                        m.record.name, c.name, m.record.name
                    );
                    if let Some(name) = &m.bound {
                        let why = format!("it is made of `{name}`, bound by a `match` or a loop, which agree does not read here");
                        push(against, Agreement::Unknown { why });
                        continue;
                    }
                    let holds = let_in("self", &m.make, &c.holds);
                    let holds_alone = holds.clone();
                    let guarded = match conjunction(m.guards.iter().cloned()) {
                        Some(g) => Expression::Apply(
                            Op::Or,
                            vec![
                                plain(Expression::Apply(Op::Not, vec![plain(g)])),
                                plain(holds),
                            ],
                        ),
                        None => holds,
                    };
                    let within = m
                        .lets
                        .iter()
                        .rev()
                        .fold(guarded, |e, (n, v)| let_in(n, v, &e));
                    let with_body = read_in(within);
                    // As linear facts: the promises and its `if`s known,
                    // the claim to show, each under its `let`s.
                    let wrap = |e: Expression<Op>| {
                        m.lets.iter().rev().fold(e, |e, (n, v)| let_in(n, v, &e))
                    };
                    let mut facts = promised.clone();
                    facts.extend(m.guards.iter().cloned().map(wrap));
                    let goal = wrap(holds_alone.clone());
                    // Its `if`s relate names: as linear facts too.
                    let agreement = match self.claim(&f.takes, &with_body, &rows) {
                        Agreement::Unknown { .. } if crate::linear::holds(&facts, &goal) => {
                            Agreement::Holds
                        }
                        agreement => agreement,
                    };
                    push(against, agreement);
                }
            }
            // The range of what it gives, as a claim the body keeps.
            if let Some((q, range)) = &f.ranges.gives {
                let with_body = let_in(&f.gives.0, body, &read_in(in_range(q, range)));
                let agreement = self.claim(&f.takes, &with_body, &rows);
                push(format!("its range `{q}: {range}`"), agreement);
            }
            // Each example: the body at its point, one run (finding 39 B).
            for (i, (e, row)) in f.examples.iter().zip(&rows).enumerate() {
                let exact = Exact {
                    log: crate::simulation::Log::kept(),
                    ..self.in_world(&e.given)
                };
                let ran = eval(&exact, &row.iter().cloned().collect(), body);
                for (call, agreement) in calls_made(&exact.log, &e.effects) {
                    push(format!("example {i}: the call `{call}`"), agreement);
                }
                for (_, agreement) in self.givens_used(&exact.log, &e.given) {
                    push(format!("example {i}"), agreement);
                }
                let Some(expected) = &e.gives else {
                    continue;
                };
                let one_run = e.takes.iter().all(crate::simulation::known);
                let here = at.to_string();
                let agreement = self.run_against(ran, expected, row, &exact.log, one_run, Some(&here), || {
                    let bounds = row
                        .iter()
                        .map(|(n, v)| (n.clone(), Bounds::exactly(v)))
                        .collect();
                    self.library.read();
                    let b = self.by_bounds(&bounds, body);
                    (b, self.library.read())
                });
                push(format!("example {i}"), agreement);
            }
        }
        // Each example against each claim, the example's value read as what
        // the function gives.
        for (i, (e, row)) in f.examples.iter().zip(&rows).enumerate() {
            let Some(expected) = &e.gives else {
                continue;
            };
            for c in &f.claims {
                let mut values: BTreeMap<String, Value> = row.iter().cloned().collect();
                values.insert(f.gives.0.clone(), expected.clone());
                let agreement = match eval(&self.in_world(&e.given), &values, &c.holds) {
                    Ok(Value::Truth(true)) => Agreement::Holds,
                    Err(Fault::Op(Undefined::Unknown(g))) => Agreement::Unknown {
                        why: format!("no value for {} here", unstated(&g)),
                    },
                    Err(Fault::Op(e @ Undefined::EndlessRepeat { .. })) => endless(row, &e),
                    Err(Fault::Op(Undefined::Budget(why))) => Agreement::Unknown { why },
                    // False, or no value: a claim with none does not hold.
                    _ => Agreement::Fails {
                        at: row.clone(),
                        gives: Some(expected.clone()),
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

    /// The claims of the record of type `t` that `e` reads, and of each
    /// record in its fields, a few levels down: what a function that
    /// takes it counts on (finding 10b).
    fn promises(&self, e: &Expression<Op>, t: &Type, depth: usize, out: &mut Vec<Expression<Op>>) {
        let Type::Record(r) = t else { return };
        if depth > 3 {
            return;
        }
        if let Some(record) = self.records.get(r.name.as_str()) {
            out.extend(record.claims.iter().map(|c| let_in("self", e, &c.holds)));
        }
        for (name, ft) in &r.fields {
            let field = Expression::Apply(Op::Field(name.clone()), vec![plain(e.clone())]);
            self.promises(&field, ft, depth + 1, out);
        }
    }

    /// `v`, written in a row as `what`, against the claims of each record
    /// in it, at any depth: one comparison each.
    fn row_value(&self, at: &Path, what: &str, v: &Value, agreed: &mut Vec<Agreed>) {
        for (claim, w, agreement) in record_claims(&self.records, &self.exact, v) {
            agreed.push(Agreed {
                at: at.clone(),
                against: format!(
                    "{what} = {} against record claim `{claim}`",
                    print::value(&w)
                ),
                agreement,
                given: vec![],
            });
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
        self.given();
        self.bounds_budget.borrow_mut().take();
        if bounded && self.surely_given(over, holds) == Some(true) {
            return Agreement::Holds;
        }
        self.given();
        let (mut tried, mut unknown) = (0, BTreeSet::new());
        for point in self.candidates(over, holds, first) {
            tried += 1;
            match eval(&self.exact, &point.iter().cloned().collect(), holds) {
                Ok(Value::Truth(true)) => {}
                Err(Fault::Op(Undefined::Unknown(g))) => _ = unknown.insert(g),
                Err(Fault::Op(e @ Undefined::EndlessRepeat { .. })) => return endless(&point, &e),
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
        let budget = self.bounds_budget.borrow_mut().take();
        let why = match (unknown.first(), budget) {
            (Some(g), _) => format!("no value for {} at some of the {tried} points tried", unstated(&g)),
            (None, Some(why)) if bounded => format!(
                "not decided by bounds, which stopped at {why}, nor at {tried} points"
            ),
            (None, None) if bounded => format!(
                "not decided by bounds over all of what it reads, nor at {tried} points"
            ),
            (None, _) => format!("it reads an opaque record, which bounds cannot tell apart, and whose values are tried known in part, its opaque fields not known: not decided at {tried} points"),
        };
        Agreement::Unknown { why }
    }

    /// A claim `by induction` true by bounds where each call of its
    /// function on a part of its input gives what the claim says there:
    /// the step, every variant at once, the leaves among them.
    fn by_induction(&self, claim: &Path, over: &[(String, Type)], e: &Expression<Op>) -> bool {
        let Some(calls) = self.induction.get(&claim.to_string()) else {
            return false;
        };
        let library = Ranges {
            calls: calls.clone(),
        };
        let before = self.ranges.replace(library);
        self.given();
        let sure = self.bounded(over, e) && self.surely_given(over, e) == Some(true);
        self.ranges.replace(before);
        sure
    }

    /// A turn of a loop `by invariant` true by bounds, through the
    /// library where what it carries is any value that keeps the claim
    /// (`model::invariants`). Points cannot tell: that value has none.
    fn by_carrier(&self, claim: &Path, over: &[(String, Type)], e: &Expression<Op>) -> Agreement {
        let Some(calls) = self.invariant.get(&claim.to_string()) else {
            return Agreement::Unknown {
                why: "its loop is not read".to_string(),
            };
        };
        let library = Ranges {
            calls: calls.clone(),
        };
        let before = self.ranges.replace(library);
        self.given();
        self.bounds_budget.borrow_mut().take();
        let sure = self.bounded(over, e) && self.surely_given(over, e) == Some(true);
        self.ranges.replace(before);
        self.given();
        match (sure, self.bounds_budget.borrow_mut().take()) {
            (true, _) => Agreement::Holds,
            (false, Some(why)) => Agreement::Unknown {
                why: format!("not decided by bounds, which stopped at {why}"),
            },
            (false, None) => Agreement::Unknown {
                why: "not decided by bounds over any value it carries that keeps the claim"
                    .to_string(),
            },
        }
    }

    /// Whether some value keeps all the claims of `f` together. False by
    /// bounds whatever the values, or at every point of a finite domain;
    /// true at a point where all hold: its examples first, then what its
    /// body gives, then candidates; else unknown.
    fn together(&self, f: &Function, rows: &[Vec<(String, Value)>]) -> Agreement {
        let over: Vec<(String, Type)> = f.takes.iter().chain([&f.gives]).cloned().collect();
        if f.claims.is_empty() {
            return Agreement::Holds;
        }
        self.bounds_budget.borrow_mut().take();
        // Its ranges first: a value that keeps the claims is in them.
        let ranges = f
            .ranges
            .takes
            .iter()
            .chain(f.ranges.gives.iter().map(|(q, r)| (q, r)));
        let Some(all) = conjunction(
            ranges
                .map(|(n, r)| in_range(n, r))
                .chain(f.claims.iter().map(|c| c.holds.clone())),
        ) else {
            return Agreement::Holds;
        };
        let bounded = self.bounded(&over, &all);
        self.given();
        if bounded && self.surely_given(&over, &all) == Some(false) {
            return Agreement::Fails {
                at: vec![],
                gives: None,
            };
        }
        self.given();
        let gives = |point: &[(String, Value)], v: &Value| {
            let mut point = point.to_vec();
            point.push((f.gives.0.clone(), v.clone()));
            point
        };
        let mut first: Vec<Vec<(String, Value)>> = f
            .examples
            .iter()
            .zip(rows)
            .filter_map(|(e, row)| e.gives.as_ref().map(|v| gives(row, v)))
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
                Err(Fault::Op(e @ Undefined::EndlessRepeat { .. })) => return endless(&point, &e),
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
        let budget = self.bounds_budget.borrow_mut().take();
        let why = match (unknown.first(), budget) {
            (Some(g), _) => format!("no value for {} at some of the {tried} points tried", unstated(&g)),
            (None, Some(why)) if bounded => format!(
                "not false by bounds, which stopped at {why}, and none of {tried} points keeps them all"
            ),
            (None, None) if bounded => format!(
                "not false by bounds, and none of {tried} points keeps them all"
            ),
            (None, _) => format!("it reads an opaque record, which bounds cannot tell apart, and no value of which is written: none of {tried} points keeps them all"),
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
        let numbers: Vec<(&String, Numbers)> = over
            .iter()
            .filter_map(|(n, t)| match t {
                Type::Int => Some((n, Numbers::Int)),
                Type::Real => Some((n, Numbers::Real)),
                _ => None,
            })
            .take(self.settings.splits)
            .collect();
        let zero = || Some(Rational::integer(0));
        let mut readings = vec![all];
        for (n, of) in numbers {
            readings = readings
                .into_iter()
                .flat_map(|r| {
                    [
                        Span::Number(of, None, zero()),
                        Span::Number(of, zero(), None),
                    ]
                    .map(|span| {
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
            let here = match self.ranges.borrow().kept(over, &bounds, e) {
                None => Some(false),
                Some(_) => self.by_bounds(&bounds, e).and_then(|b| b.surely()),
            };
            let here = match here {
                Some(true) => here,
                _ if self.implied(over, &bounds, e) => Some(true),
                _ => here,
            };
            match (here, sure) {
                (None, _) => return None,
                (Some(h), Some(s)) if h != s => return None,
                (h, _) => sure = h,
            }
        }
        sure
    }

    /// Whether `e`, read as `let`s around `!p || … || c || …`, is true
    /// everywhere in `bounds`: each `c` read where the premises `p` keep
    /// their names (`Ranges::kept`), and true wherever a premise is false.
    /// Only premises that read no name a `let` binds, and only when no
    /// `let` value nor premise may fault in `bounds`, for a fault there
    /// would be what `e` gives: nor any conclusion, which may be read
    /// before a premise.
    fn implied(
        &self,
        over: &[(String, Type)],
        bounds: &BTreeMap<String, Bounds>,
        e: &Expression<Op>,
    ) -> bool {
        let mut lets: Vec<(String, Expression<Op>)> = Vec::new();
        let mut body = e;
        while let Expression::Apply(Op::Let, args) = body {
            match args.as_slice() {
                [value, then] if value.binds.is_empty() && then.binds.len() == 1 => {
                    lets.push((then.binds[0].clone(), value.body.clone()));
                    body = &then.body;
                }
                _ => return false,
            }
        }
        let mut disjuncts = Vec::new();
        let mut todo = vec![body];
        while let Some(d) = todo.pop() {
            match d {
                Expression::Apply(Op::Or, args)
                    if args.iter().all(|a| a.binds.is_empty()) && args.len() == 2 =>
                {
                    todo.push(&args[1].body);
                    todo.push(&args[0].body);
                }
                _ => disjuncts.push(d),
            }
        }
        let bound: BTreeSet<&str> = lets.iter().map(|(n, _)| n.as_str()).collect();
        let (mut premises, mut conclusions) = (Vec::new(), Vec::new());
        // Whether every premise is read before any conclusion.
        let mut premises_first = true;
        for d in disjuncts {
            match d {
                Expression::Apply(Op::Not, args)
                    if args.len() == 1
                        && args[0].binds.is_empty()
                        && !reads_any(&args[0].body, &bound) =>
                {
                    premises_first &= conclusions.is_empty();
                    premises.push(args[0].body.clone())
                }
                _ => conclusions.push(d.clone()),
            }
        }
        let (Some(premise), Some(conclusion)) = (
            premises
                .iter()
                .cloned()
                .reduce(|a, b| Expression::Apply(Op::And, vec![plain(a), plain(b)])),
            conclusions
                .into_iter()
                .reduce(|a, b| Expression::Apply(Op::Or, vec![plain(a), plain(b)])),
        ) else {
            return false;
        };
        let around = |inner: Expression<Op>, upto: usize| {
            lets[..upto]
                .iter()
                .rev()
                .fold(inner, |body, (n, v)| let_in(n, v, &body))
        };
        // No fault in `bounds`: each premise, read wherever `e` is.
        let safe_in = |b: &BTreeMap<String, Bounds>, x: Expression<Op>| matches!(self.by_bounds(b, &x), Some(b) if !b.may_fault);
        if !premises.iter().all(|p| safe_in(bounds, p.clone())) {
            return false;
        }
        let Some(kept) = self.ranges.borrow().kept(over, bounds, &premise) else {
            // No value keeps the premises: a premise is false, and `e`
            // true, wherever the premises are all read first.
            return premises_first || {
                (0..lets.len()).all(|i| safe_in(bounds, around(lets[i].1.clone(), i)))
                    && safe_in(bounds, around(conclusion, lets.len()))
            };
        };
        // A let's value is read only through its name, in a conclusion.
        // Every premise read first: a conclusion only where they all
        // hold, so only there may it not fault. Else a conclusion may be
        // read before a false premise: nowhere in `bounds`.
        let reach = if premises_first { &kept } else { bounds };
        if !(0..lets.len()).all(|i| safe_in(reach, around(lets[i].1.clone(), i)))
            || !safe_in(reach, around(conclusion.clone(), lets.len()))
        {
            return false;
        }
        let read = around(conclusion, lets.len());
        self.by_bounds(&kept, &read).and_then(|b| b.surely()) == Some(true)
    }

    /// `e` read by bounds, none on a fault; a budget reached is kept to
    /// be said (`bounds_budget`).
    fn by_bounds(&self, bounds: &BTreeMap<String, Bounds>, e: &Expression<Op>) -> Option<Bounds> {
        match eval(&*self.ranges.borrow(), bounds, e) {
            Ok(b) => Some(b),
            Err(Fault::Op(Undefined::Budget(why))) => {
                self.bounds_budget.borrow_mut().get_or_insert(why);
                None
            }
            Err(_) => None,
        }
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
            // Bounds hold the fields a record with `..` does not state.
            Type::Record(r) if r.name == crate::model::REST => false,
            Type::Record(r) => {
                self.opaque.contains(r.name.as_str())
                    || r.fields.iter().any(|(_, t)| self.opaque_in(t))
            }
            Type::Enum(e) => e
                .variants
                .iter()
                .any(|(_, ts)| ts.iter().any(|t| self.opaque_in(t))),
            Type::Seq(t) | Type::Encoded(t) => self.opaque_in(t),
            // Bounds hold any letter or byte.
            Type::Truth | Type::Int | Type::Real | Type::Param { .. } => false,
            Type::Char | Type::Byte => false,
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

    /// The numbers and texts `e` writes, and those of the functions it
    /// calls, however far.
    fn constants(&self, e: &Expression<Op>) -> Constants {
        let mut todo = vec![e];
        for f in self.called(e, true) {
            if let Some(function) = self.functions.get(&f) {
                todo.extend(function.body.iter());
                todo.extend(function.claims.iter().map(|c| &c.holds));
            }
        }
        let (mut found, mut texts) = (BTreeSet::new(), BTreeSet::new());
        while let Some(e) = todo.pop() {
            if let Expression::Apply(op, args) = e {
                match op {
                    Op::Int(n) => _ = found.insert(Rational::integer(n.clone())),
                    Op::Real(q) => _ = found.insert(q.clone()),
                    Op::Text(s) => _ = texts.insert(s.clone()),
                    _ => {}
                }
                todo.extend(args.iter().map(|a| &a.body));
            }
        }
        Constants {
            numbers: found.into_iter().collect(),
            texts: texts.into_iter().collect(),
        }
    }

    /// How many values `over` has together, when few enough to list.
    fn size(&self, over: &[(String, Type)]) -> Option<usize> {
        let domains = self.domains.borrow();
        over.iter()
            .map(|(n, t)| match domains.get(n) {
                Some(range) => size_of_range(range),
                None => self.size_of(t),
            })
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
            // A tree: as many values as it is tall, too many to list.
            Type::Enum(e) if e.cut() => None,
            Type::Enum(e) => e
                .variants
                .iter()
                .map(|(_, ts)| {
                    ts.iter()
                        .map(|t| self.size_of(t))
                        .try_fold(1usize, |n, k| n.checked_mul(k?))
                })
                .try_fold(0usize, |n, k| n.checked_add(k?)),
            // Sequences of every length: too many to list.
            Type::Int | Type::Real | Type::Seq(_) | Type::Param { .. } => None,
            Type::Encoded(_) | Type::Char | Type::Byte => None,
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
        let domains = self.domains.borrow();
        // In a range, every number when they all fit in `n` together,
        // else the values worth trying that lie in it, and its ends.
        let every = self.size(over).is_some_and(|k| k <= n);
        let lists: Vec<Vec<Value>> = over
            .iter()
            .map(|(name, t)| match domains.get(name) {
                Some(range) if every => num_iter(range).map(Value::Int).collect(),
                Some(range) => {
                    let ends = [&range.low, &range.high].map(|e| Value::Int(e.clone()));
                    let mut values: Vec<Value> = self
                        .values_of(t, &constants)
                        .into_iter()
                        .filter(|v| range.holds(v))
                        .collect();
                    for v in ends {
                        if !values.contains(&v) {
                            values.push(v);
                        }
                    }
                    values
                }
                None => self.values_of(t, &constants),
            })
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
    fn values_of(&self, t: &Type, constants: &Constants) -> Vec<Value> {
        self.values_in(t, constants, &[], self.settings.height)
    }

    /// `values_of`, inside the enums `around`; a tree no taller than
    /// `height` levels more (`Settings::height`).
    fn values_in(
        &self,
        t: &Type,
        constants: &Constants,
        around: &[&crate::simulation::Enum],
        height: usize,
    ) -> Vec<Value> {
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
                for c in constants.numbers.iter().filter(|c| c.is_integer()) {
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
                for c in &constants.numbers {
                    for d in [0i64, -1, 1] {
                        add(Value::Real(c.plus(&Rational::integer(d))));
                    }
                }
            }
            Type::Record(r) if self.opaque.contains(r.name.as_str()) => {}
            Type::Record(r) => {
                // A field of which no value is written (an opaque record)
                // is not known: the record is known in part (finding 2).
                let fields: Vec<Vec<Option<Value>>> = r
                    .fields
                    .iter()
                    .map(|(_, t)| {
                        let values = self.values_in(t, constants, around, height);
                        match values.is_empty() {
                            true => vec![None],
                            false => values.into_iter().map(Some).collect(),
                        }
                    })
                    .collect();
                if self.size_of(t).is_some_and(|k| k <= self.settings.points) {
                    for values in product(&fields) {
                        add(Value::partial(r.clone(), values));
                    }
                } else {
                    let most = fields.iter().map(Vec::len).max().unwrap_or(1).max(1);
                    for i in 0..most {
                        let values = fields
                            .iter()
                            .map(|f| f[i.min(f.len() - 1)].clone())
                            .collect();
                        add(Value::partial(r.clone(), values));
                    }
                }
            }
            // Each variant: what it holds, every value together when few
            // enough, else the i-th of each; none when what it holds has
            // no value written (an opaque record).
            // Where a tree holds itself: the tree around it, one level
            // lower; none past the tallest.
            Type::Enum(e) if e.cut() => {
                let full = around.iter().rev().find(|a| a.name == e.name);
                if let (Some(full), Some(lower)) = (full, height.checked_sub(1)) {
                    let full = Type::Enum((*full).clone());
                    for v in self.values_in(&full, constants, around, lower) {
                        add(v);
                    }
                }
            }
            Type::Enum(e) => {
                let around: Vec<&crate::simulation::Enum> =
                    around.iter().copied().chain([e]).collect();
                let around = around.as_slice();
                let few = self.size_of(t).is_some_and(|k| k <= self.settings.points);
                for (i, (_, ts)) in e.variants.iter().enumerate() {
                    let holds: Vec<Vec<Option<Value>>> = ts
                        .iter()
                        .map(|t| {
                            self.values_in(t, constants, around, height)
                                .into_iter()
                                .map(Some)
                                .collect()
                        })
                        .collect();
                    if holds.iter().any(Vec::is_empty) {
                        continue;
                    }
                    let rows: Vec<Vec<Option<Value>>> = match few {
                        true => product(&holds),
                        false => {
                            let most = holds.iter().map(Vec::len).max().unwrap_or(1).max(1);
                            (0..most)
                                .map(|k| {
                                    holds
                                        .iter()
                                        .map(|h| h[k.min(h.len() - 1)].clone())
                                        .collect()
                                })
                                .collect()
                        }
                    };
                    for row in rows {
                        add(Value::Variant(
                            e.clone(),
                            i,
                            row.into_iter().flatten().collect(),
                        ));
                    }
                }
            }
            // Up to `Settings::length` values: the empty one first, then
            // for each length, from each value of its type on, the next
            // ones in turn, and that value repeated.
            Type::Seq(t) => {
                let each = self.values_in(t, constants, around, height);
                add(Value::Seq((**t).clone(), vec![]));
                for k in 1..=self.settings.length {
                    for i in 0..each.len() {
                        let turn = (0..k).map(|j| each[(i + j) % each.len()].clone());
                        add(Value::Seq((**t).clone(), turn.collect()));
                        add(Value::Seq((**t).clone(), vec![each[i].clone(); k]));
                    }
                }
            }
            // Text: the empty one, the texts the model writes, each two
            // of them joined, and their prefixes (finding 3, step 3).
            Type::Encoded(_) => {
                add(Value::Text(String::new()));
                for s in &constants.texts {
                    add(Value::Text(s.clone()));
                }
                for a in &constants.texts {
                    for b in &constants.texts {
                        add(Value::Text(format!("{a}{b}")));
                    }
                }
                for s in &constants.texts {
                    for (k, _) in s.char_indices().skip(1) {
                        add(Value::Text(s[..k].to_string()));
                    }
                }
            }
            // No value of a letter or a byte is written yet.
            Type::Char | Type::Byte | Type::Param { .. } => {}
        }
        out
    }

    /// What one run of an example says, against the value it expects
    /// (finding 39 B): holds or fails, never unknown, unless it is not
    /// one run (an input known only in part). `bounds`, what the body
    /// gives by bounds at its point and the promises they read.
    fn run_against(
        &self,
        ran: Result<Value, Fault<Undefined>>,
        expected: &Value,
        row: &[(String, Value)],
        log: &crate::simulation::Log,
        one_run: bool,
        here: Option<&str>,
        bounds: impl FnOnce() -> (Option<Bounds>, BTreeSet<String>),
    ) -> Agreement {
        match ran {
            Ok(v) => match matches(&v, expected) {
                Ok(true) => Agreement::Holds,
                Ok(false) => Agreement::Fails {
                    at: row.to_vec(),
                    gives: Some(v),
                },
                Err(g) => Agreement::Unknown {
                    why: format!("`{g}` is not known, and only it could tell"),
                },
            },
            Err(Fault::Op(Undefined::Unknown(g))) if one_run && self.functions.contains_key(g.as_str()) => {
                let call = log
                    .calls()
                    .into_iter()
                    .rev()
                    .find(|(h, _)| *h == g)
                    .map_or(format!("{g}(…)"), |(h, at)| format!("{h}{}", print::values(&at)));
                let (b, read) = bounds();
                if b.is_some_and(|b| !crate::simulation::lies_in(expected, &b.span)) {
                    let read: Vec<String> = read.iter().map(|p| format!("`{p}`")).collect();
                    let (claims, keep) = match read.len() {
                        1 => ("claim", "keeps"),
                        _ => ("claims", "keep"),
                    };
                    return Agreement::Wrong {
                        code: A017,
                        why: format!(
                            "fails: `{call}` has no value here, and the {claims} {} {keep} the run from giving {}",
                            read.join(", "),
                            print::value(expected)
                        ),
                    };
                }
                Agreement::Wrong {
                    code: A014,
                    why: format!(
                        "fails: no value for `{call}`: the run calls it, and nothing gives it there; give it in the example (`given {call} -> …`), or give `{g}` a body"
                    ),
                }
            }
            Err(Fault::Op(Undefined::Unknown(g))) => Agreement::Unknown {
                why: format!("no value for {} here", unstated(&g)),
            },
            Err(Fault::Op(e @ Undefined::EndlessRepeat { .. })) if one_run => Agreement::Wrong {
                code: A013,
                why: format!("fails, at {}, {}", point_of(row), e.endless(here).unwrap_or_default()),
            },
            Err(Fault::Op(e @ Undefined::EndlessRepeat { .. })) => endless(row, &e),
            Err(Fault::Op(Undefined::Budget(why))) if one_run => Agreement::Wrong {
                code: A016,
                why: format!("fails, at {}, {why} before the run gave a value", point_of(row)),
            },
            Err(Fault::Op(Undefined::Budget(why))) => Agreement::Unknown { why },
            Err(_) => Agreement::Fails {
                at: row.to_vec(),
                gives: None,
            },
        }
    }

    /// Each given of an example the run did not call at its arguments
    /// (finding 39 B).
    fn givens_used(
        &self,
        log: &crate::simulation::Log,
        given: &crate::model::Given,
    ) -> Vec<(String, Agreement)> {
        let made = log.calls();
        let mut out = Vec::new();
        for (g, args, v) in &given.0 {
            let shown: Vec<String> = args
                .iter()
                .map(|a| a.as_ref().map_or("_".to_string(), print::value))
                .collect();
            let _ = v;
            let written = format!("{g}({})", shown.join(", "));
            let fits = |(h, at): &(String, Vec<Value>)| {
                h == g
                    && at.len() == args.len()
                    && at.iter().zip(args).all(|(v, a)| a.as_ref().is_none_or(|a| a == v))
            };
            // A given of a function with a body is M022: never here.
            let why = if made.iter().any(fits) {
                continue;
            } else {
                let made: Vec<String> = made
                    .iter()
                    .map(|(h, at)| format!("`{h}{}`", print::values(at)))
                    .collect();
                match made.is_empty() {
                    true => "the run calls no function with no body".to_string(),
                    false => format!("the run calls {}", made.join(", ")),
                }
            };
            out.push((
                written.clone(),
                Agreement::Wrong {
                    code: A015,
                    why: format!("`{written}` is given but never called: {why}"),
                },
            ));
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
                let Some(expected) = &e.gives else {
                    continue;
                };
                if e.takes != args || !e.given.0.is_empty() {
                    continue;
                }
                let agreement = match matches(v, expected) {
                    Ok(true) => Agreement::Holds,
                    Ok(false) => Agreement::Fails {
                        at: named(&port.function.takes, &args),
                        gives: Some(expected.clone()),
                    },
                    Err(g) => Agreement::Unknown {
                        why: format!("`{g}` is not known, and only it could tell"),
                    },
                };
                agreed.push(Agreed {
                    at: at.clone(),
                    against: format!("`{p}` against example {j} of `{}`", port.at),
                    agreement,
                    given: vec![],
                });
            }
            // Refused only for a value of another type, which M022 refuses.
            library = library
                .given(&port.at.to_string(), args, v.clone())
                .unwrap_or_else(|same| same);
        }
        // What the environment gave, as BDD says it: `_` any argument.
        for (g, args, v) in &example.given.0 {
            library = library
                .given_any(g, args.clone(), v.clone())
                .unwrap_or_else(|same| same);
        }
        // One run, its calls kept (finding 39 B): what `then` lists, and
        // what the example gives, are of the calls it made.
        let exact = Exact {
            calls: Arc::new(library),
            log: crate::simulation::Log::kept(),
        };
        // The calls the run of `when` makes, as `then` lists them.
        if let Some((f, args, _)) = example
            .when
            .as_ref()
            .filter(|_| !example.effects.0.is_empty())
        {
            let (call, bound) = call_at(&f.to_string(), args);
            let _ = eval(&exact, &bound, &call);
            for (written, agreement) in calls_made(&exact.log, &example.effects) {
                agreed.push(Agreed {
                    at: at.clone(),
                    against: format!("the call `{written}`"),
                    agreement,
                    given: vec![],
                });
            }
        }
        let one_run = example.inputs.iter().all(|(_, v)| crate::simulation::known(v));
        for (p, expected) in &example.outputs {
            let Some(port) = port(index, p) else { continue };
            let Some(args) = args_of(&port.at, port.function) else {
                continue;
            };
            let point = named(&port.function.takes, &args);
            let (call, bound) = call_at(&port.at.to_string(), &args);
            let ran = eval(&exact, &bound, &call);
            let here = port.at.to_string();
            let agreement = self.run_against(ran, expected, &point, &exact.log, one_run, Some(&here), || {
                let bounds = bound
                    .iter()
                    .map(|(n, v)| (n.clone(), Bounds::exactly(v)))
                    .collect();
                self.library.read();
                let b = self.by_bounds(&bounds, &call);
                (b, self.library.read())
            });
            agreed.push(Agreed {
                at: at.clone(),
                against: format!("`{p}`"),
                agreement,
                given: vec![],
            });
        }
        for (_, agreement) in self.givens_used(&exact.log, &example.given) {
            agreed.push(Agreed {
                at: at.clone(),
                against: "the example".to_string(),
                agreement,
                given: vec![],
            });
        }
        (agreed, self.unread(index, &at, example))
    }

    /// A009 for each input of `example` no output reads: what an output's
    /// function takes, a value of the world its computation calls for,
    /// and what that function takes, are read; nothing else is.
    fn unread(&self, index: &Index, at: &Path, example: &ModelExample) -> Vec<Finding> {
        let mut outputs: BTreeSet<String> = example
            .outputs
            .iter()
            .filter_map(|(p, _)| port(index, p))
            .map(|port| port.at.to_string())
            .collect();
        outputs.extend(example.when.iter().map(|(f, _, _)| f.to_string()));
        let mut reached = outputs.clone();
        for f in &outputs {
            if let Some(body) = self.functions.get(f).and_then(|f| f.body.as_ref()) {
                reached.extend(self.called(body, false));
            }
        }
        let given: BTreeSet<&Path> = example.inputs.iter().map(|(p, _)| p).collect();
        let mut findings = Vec::new();
        // A given the run never calls is A015, of the run itself.
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
fn product<V: Clone>(lists: &[Vec<V>]) -> Vec<Vec<V>> {
    lists.iter().fold(vec![vec![]], |ways, list| {
        ways.into_iter()
            .flat_map(|way: Vec<V>| {
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

/// Whether `e` reads any of `names`, bound there or not.
fn reads_any(e: &Expression<Op>, names: &BTreeSet<&str>) -> bool {
    match e {
        Expression::Name(n) => names.contains(n.as_str()),
        Expression::Apply(_, args) => args.iter().any(|a| {
            a.binds.iter().any(|b| names.contains(b.as_str())) || reads_any(&a.body, names)
        }),
    }
}

/// How many numbers `range` holds, when that fits.
fn size_of_range(range: &Range) -> Option<usize> {
    let k: BigInt = &range.high - &range.low + 1;
    usize::try_from(k).ok().filter(|k| *k > 0)
}

/// The numbers of `range`, from its low end up.
fn num_iter(range: &Range) -> impl Iterator<Item = BigInt> + '_ {
    let mut next = range.low.clone();
    std::iter::from_fn(move || {
        (next <= range.high).then(|| {
            let n = next.clone();
            next += 1;
            n
        })
    })
}

/// `a && b && …`; `None` for none.
fn conjunction(all: impl Iterator<Item = Expression<Op>>) -> Option<Expression<Op>> {
    all.reduce(|a, b| Expression::Apply(Op::And, vec![plain(a), plain(b)]))
}

/// `low <= n && n <= high`.
fn in_range(n: &str, range: &Range) -> Expression<Op> {
    let int = |v: &BigInt| Expression::Apply(Op::Int(v.clone()), vec![]);
    let name = || Expression::Name(n.to_string());
    Expression::Apply(
        Op::And,
        vec![
            plain(Expression::Apply(
                Op::AtMost,
                vec![plain(int(&range.low)), plain(name())],
            )),
            plain(Expression::Apply(
                Op::AtMost,
                vec![plain(name()), plain(int(&range.high))],
            )),
        ],
    )
}

/// What `by invariant` reads of a loop: two claims, each with what it
/// reads, and the turn as `linear` reads it, what is carried a name.
struct Loop {
    start: (Vec<(String, Type)>, Expression<Op>),
    turn: (Vec<(String, Type)>, Expression<Op>),
    linear: (Vec<Expression<Op>>, Expression<Op>),
}

/// The two claims `by invariant` reads of the loop that is `f`'s body,
/// at `at`, each with what it reads. At the start: the claim of what the
/// loop starts from, `n` 0. On a turn: from what the loop carries, any
/// value that keeps the claim, `carrier` called (`model::invariants`),
/// with `0 <= n < xs.len()` under `by invariant n`, an `until` not yet
/// done: the claim of what it carries next, `n + 1`. Both true, every
/// turn keeps it to the end, where the loop gives what `f` gives (`n`
/// then `xs.len()`). `None` when a value the loop names is named as one
/// of `f`'s inputs.
fn loop_claims(
    at: &Path,
    f: &Function,
    c: &crate::model::Claim,
    library: &Arc<Calls>,
) -> Option<Loop> {
    let (q, gives) = (&f.gives.0, &f.gives.1);
    let claim = c.kept();
    let turns = c.invariant.as_ref()?.turns.as_ref();
    let name = |n: &str| Expression::Name(n.to_string());
    let int = |i: i64| Expression::Apply(Op::Int(BigInt::from(i)), vec![]);
    let apply = |op: Op, args: Vec<Expression<Op>>| {
        Expression::Apply(op, args.into_iter().map(plain).collect())
    };
    let types: BTreeMap<String, Type> = f.takes.iter().cloned().collect();
    let simulation = crate::simulation::Simulation {
        calls: library.clone(),
    };
    let (start, next, mut names, mut before) = match f.body.as_ref()? {
        Expression::Apply(Op::Fold, args) => {
            let (xs, start, next) = (&args[0].body, &args[1].body, &args[2]);
            let each = match crate::expression::check(&simulation, &types, xs)? {
                Type::Seq(t) | Type::Encoded(t) => *t,
                _ => return None,
            };
            let names = vec![(next.binds.first()?.clone(), each)];
            let mut before = vec![];
            if let Some(n) = turns {
                let length = apply(Op::Len, vec![xs.clone()]);
                before.push(apply(Op::AtMost, vec![int(0), name(n)]));
                before.push(apply(Op::Less, vec![name(n), length]));
            }
            (start, next, names, before)
        }
        Expression::Apply(Op::Until, args) => {
            let (start, done, next) = (&args[0].body, &args[1], &args[2]);
            if done.binds != next.binds {
                return None;
            }
            let not_done = apply(Op::Not, vec![done.body.clone()]);
            (start, next, vec![], vec![not_done])
        }
        _ => return None,
    };
    // What the loop carries, by the names its block binds.
    let labels = next.binds[names.len()..].to_vec();
    let carried_types = crate::simulation::carried(gives);
    if carried_types.len() != labels.len() {
        return None;
    }
    names.extend(labels.iter().cloned().zip(carried_types));
    let mut over = f.takes.clone();
    // The loop's value, the turns, and what it carries, under the call.
    over.push(names[0].clone());
    if let Some(n) = turns {
        over.push((n.clone(), Type::Int));
    }
    let clash = |n: &String| types.contains_key(n) || turns == Some(n) || n == "__next";
    if names.iter().any(|(n, _)| clash(n)) {
        return None;
    }
    // Any value it carries that keeps the claim.
    let mut args: Vec<Expression<Op>> = f.takes.iter().map(|(n, _)| name(n)).collect();
    args.extend(turns.map(|n| name(n)));
    let path = format!(
        "{}::{}",
        at.names[0],
        crate::model::carrier(&f.name, &c.name)
    );
    let carried = apply(Op::Call(path), args);
    // The block's values, under the labels of what it carries.
    let next_given = match (gives, labels.as_slice()) {
        (_, [_]) => next.body.clone(),
        (Type::Record(r), _) => {
            let mut inner: BTreeMap<String, Type> = types.clone();
            inner.extend(names.iter().cloned());
            let Type::Record(tuple) = crate::expression::check(&simulation, &inner, &next.body)?
            else {
                return None;
            };
            let fields = tuple
                .fields
                .iter()
                .map(|(field, _)| apply(Op::Field(field.clone()), vec![name("__next")]))
                .collect();
            let_in("__next", &next.body, &apply(Op::Make(r.clone()), fields))
        }
        _ => return None,
    };
    let at_turns = |count: Expression<Op>, holds: Expression<Op>| match turns {
        Some(n) => let_in(n, &count, &holds),
        None => holds,
    };
    let at_start = at_turns(int(0), let_in(q, start, claim));
    let one_more = turns.map(|n| apply(Op::Add, vec![name(n), int(1)]));
    let after = let_in(
        q,
        &next_given,
        &match one_more {
            Some(count) => at_turns(count, claim.clone()),
            None => claim.clone(),
        },
    );
    // For `linear`: the claim of what is carried, a value of each label.
    let labelled = match (gives, labels.as_slice()) {
        (_, [one]) => name(one),
        (Type::Record(r), _) => apply(
            Op::Make(r.clone()),
            labels.iter().map(|l| name(l)).collect(),
        ),
        _ => return None,
    };
    let mut hypotheses = before.clone();
    hypotheses.push(at_turns(
        turns.map_or_else(|| int(0), |n| name(n)),
        let_in(q, &labelled, claim),
    ));
    let linear = (hypotheses, after.clone());
    let mut turn = match before.is_empty() {
        true => after,
        false => {
            let before = std::mem::take(&mut before)
                .into_iter()
                .reduce(|a, b| apply(Op::And, vec![a, b]))?;
            apply(Op::Or, vec![apply(Op::Not, vec![before]), after])
        }
    };
    // Each label, from the value carried.
    let one = labels.len() == 1;
    for label in labels.iter().rev() {
        let value = match one {
            true => name("__carried"),
            false => apply(Op::Field(label.clone()), vec![name("__carried")]),
        };
        turn = let_in(label, &value, &turn);
    }
    let turn = let_in("__carried", &carried, &turn);
    Some(Loop {
        start: (f.takes.clone(), at_start),
        turn: (over, turn),
        linear,
    })
}

/// `by invariant` shown, or why not: from its start and a turn.
fn kept(start: Agreement, turn: Agreement) -> Result<(), String> {
    let at = |at: &[(String, Value)]| {
        let shown: Vec<String> = at
            .iter()
            .map(|(n, v)| format!("{n} = {}", print::value(v)))
            .collect();
        shown.join(", ")
    };
    match (start, turn) {
        (Agreement::Holds, Agreement::Holds) => Ok(()),
        (Agreement::Fails { at: p, .. }, _) => {
            Err(format!("false at the loop's start, at {}", at(&p)))
        }
        (_, Agreement::Fails { at: p, .. }) => {
            Err(format!("a turn does not keep it, from {}", at(&p)))
        }
        (Agreement::Unknown { why }, _) => Err(format!("its start is not decided: {why}")),
        (_, Agreement::Unknown { why }) => Err(format!("a turn is not decided: {why}")),
        (Agreement::Endless { fault, .. }, _) => Err(format!(
            "its start is not decided: {}",
            fault.endless(None).unwrap_or_default()
        )),
        (_, Agreement::Endless { fault, .. }) => Err(format!(
            "a turn is not decided: {}",
            fault.endless(None).unwrap_or_default()
        )),
        // Never of a claim: only an example fails so, or is uncalled.
        (Agreement::Wrong { why, .. }, _) | (_, Agreement::Wrong { why, .. }) => Err(why),
        (Agreement::Uncalled { call, .. }, _) | (_, Agreement::Uncalled { call, .. }) => {
            Err(format!("`{call}` is not made"))
        }
    }
}

fn plain(body: Expression<Op>) -> Arg<Op> {
    Arg {
        binds: vec![],
        body,
    }
}

/// The finding of a comparison that does not hold.
/// `, given `sun::light::bright``: what a result by bounds assumed.
fn given(paths: &[String]) -> String {
    match paths.is_empty() {
        true => String::new(),
        false => {
            let each: Vec<String> = paths.iter().map(|p| format!("`{p}`")).collect();
            format!(", given {}", each.join(", "))
        }
    }
}

/// A hint for each comparison that holds given what a function with no
/// body promises (A010): nothing checked those claims, so the result
/// rests on them, and says so.
pub fn hints(agreed: &[Agreed]) -> Vec<Hint> {
    agreed
        .iter()
        .filter(|a| a.agreement == Agreement::Holds && !a.given.is_empty())
        .map(|a| Hint {
            code: A010,
            at: a.at.clone(),
            message: format!("{} holds{}", a.against, given(&a.given)),
        })
        .collect()
}

/// Each call `effects` lists, and whether the run whose calls `log`
/// kept made it, at least once, `_` any argument.
fn calls_made(
    log: &crate::simulation::Log,
    effects: &crate::model::Effects,
) -> Vec<(String, Agreement)> {
    let made = log.calls();
    effects
        .0
        .iter()
        .map(|(f, args)| {
            let written = print::effects(&crate::model::Effects(vec![(f.clone(), args.clone())]));
            let fits = |(g, at): &(String, Vec<Value>)| {
                g == f
                    && at.len() == args.len()
                    && at
                        .iter()
                        .zip(args)
                        .all(|(v, a)| a.as_ref().is_none_or(|a| a == v))
            };
            let agreement = match made.iter().any(fits) {
                true => Agreement::Holds,
                false => Agreement::Uncalled {
                    call: written.clone(),
                    made: made
                        .iter()
                        .map(|(g, at)| format!("`{g}{}`", print::values(at)))
                        .collect(),
                },
            };
            (written, agreement)
        })
        .collect()
}

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
        Agreement::Endless { at, fault } => {
            // The function compared, unless an example of a part's.
            let here = (a.at.names.first().map(String::as_str) != Some("examples"))
                .then(|| a.at.to_string());
            let why = fault.endless(here.as_deref()).unwrap_or_default();
            (A013, format!("{}: unknown, at {}, {why}", a.against, point(at)))
        }
        Agreement::Wrong { code, why } => (*code, format!("{}: {why}", a.against)),
        Agreement::Uncalled { call: _, made } => {
            let made = match made.is_empty() {
                true => "it calls no function with no body".to_string(),
                false => format!("it calls {}", made.join(", ")),
            };
            (
                A011,
                format!("{}: the run does not make it; {made}", a.against),
            )
        }
        Agreement::Fails { .. } if a.against.contains(" against record claim `") => (
            A012,
            a.against
                .replace(" against record claim ", " breaks record claim "),
        ),
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
        message: format!("{message}{}", given(&a.given)),
    })
}

/// The contract and law of a comparison of a contract's claim,
/// `part::f::Contract::law`.
fn law_of(a: &Agreed) -> Option<(&str, &str)> {
    match a.at.names.as_slice() {
        [_, _, contract, law] if a.against == CONTRACT => Some((contract, law)),
        _ => None,
    }
}

/// What agree says a contract's claim is compared against.
const CONTRACT: &str = "every value of what it reads";

/// The comparisons of the law of `a`'s contract, each impl's, when there
/// are two or more and every one fails: the law, not each impl, is then
/// what to look at (finding 10b).
fn broken_everywhere<'a>(agreed: &'a [Agreed], law: (&str, &str)) -> Option<Vec<&'a Agreed>> {
    let all: Vec<&Agreed> = agreed.iter().filter(|a| law_of(a) == Some(law)).collect();
    let fails = all
        .iter()
        .all(|a| matches!(a.agreement, Agreement::Fails { .. }));
    (all.len() >= 2 && fails).then_some(all)
}

fn everywhere_broken(agreed: &[Agreed], a: &Agreed) -> bool {
    law_of(a).is_some_and(|law| broken_everywhere(agreed, law).is_some())
}

/// One A005 for each contract's claim every impl breaks, at the
/// contract, listing where each breaks it.
fn laws_broken_everywhere(agreed: &[Agreed]) -> Vec<Finding> {
    let laws: BTreeSet<(&str, &str)> = agreed.iter().filter_map(law_of).collect();
    let mut findings = Vec::new();
    for law in laws {
        let Some(all) = broken_everywhere(agreed, law) else {
            continue;
        };
        let each: Vec<String> = all
            .iter()
            .filter_map(|a| {
                finding(a).map(|f| format!("`{}::{}` {}", a.at.names[0], a.at.names[1], f.message))
            })
            .collect();
        findings.push(Finding {
            code: A005,
            at: Path::to(&[law.0, law.1]),
            message: format!(
                "every impl of `{}` breaks it: {}; the claim, or every impl, is to look at",
                law.0,
                each.join("; ")
            ),
        });
    }
    findings
}

/// Each record in `v`, at any depth, against each of its claims, read
/// of it as `self`: the claim's path (`Span::ordered`), the record's
/// value, and whether it holds there (finding 10b).
pub(crate) fn record_claims(
    records: &BTreeMap<&str, &crate::model::Record>,
    exact: &Exact,
    v: &Value,
) -> Vec<(String, Value, Agreement)> {
    let (record, inner): (Option<&str>, Vec<&Value>) = match v {
        Value::Record(r, fields) => (Some(&r.name), fields.iter().collect()),
        Value::Partial(r, fields) => (Some(&r.name), fields.iter().flatten().collect()),
        Value::Variant(_, _, held) => (None, held.iter().collect()),
        Value::Seq(_, held) => (None, held.iter().collect()),
        _ => (None, vec![]),
    };
    let mut out = Vec::new();
    let claims = record.and_then(|r| records.get(r).map(|rec| (r, &rec.claims)));
    if let Some((r, claims)) = claims {
        for c in claims {
            let values = BTreeMap::from([("self".to_string(), v.clone())]);
            let agreement = match eval(exact, &values, &c.holds) {
                Ok(Value::Truth(true)) => Agreement::Holds,
                Err(Fault::Op(Undefined::Unknown(g))) => Agreement::Unknown {
                    why: format!("no value for {} here", unstated(&g)),
                },
                Err(Fault::Op(e @ Undefined::EndlessRepeat { .. })) => {
                    endless(&[("self".to_string(), v.clone())], &e)
                }
                Err(Fault::Op(Undefined::Budget(why))) => Agreement::Unknown { why },
                // False, or no value: a claim with none does not hold.
                _ => Agreement::Fails {
                    at: vec![("self".to_string(), v.clone())],
                    gives: None,
                },
            };
            out.push((format!("{r}::{}", c.name), v.clone(), agreement));
        }
    }
    for w in inner {
        out.extend(record_claims(records, exact, w));
    }
    out
}

/// A record a body makes: its `Make`, the `let`s around it (outermost
/// first), the conditions of the `if`s it is under, and, when it reads
/// a name bound otherwise (a `match` arm, a loop), that name.
struct Made {
    record: crate::simulation::Record,
    make: Expression<Op>,
    lets: Vec<(String, Expression<Op>)>,
    guards: Vec<Expression<Op>>,
    bound: Option<String>,
}

/// Each record `e` makes but does not give: one it gives is read as
/// what the function gives (`gives`, the record of what it gives, and
/// `tail`, whether `e` is what it gives).
fn constructions(
    e: &Expression<Op>,
    gives: Option<&str>,
    tail: bool,
    lets: &[(String, Expression<Op>)],
    guards: &[Expression<Op>],
    bound: &BTreeSet<String>,
    out: &mut Vec<Made>,
) {
    let Expression::Apply(op, args) = e else {
        return;
    };
    let names = |e: &Expression<Op>| -> Option<String> {
        bound
            .iter()
            .find(|n| reads_any(e, &BTreeSet::from([n.as_str()])))
            .cloned()
    };
    match (op, args.as_slice()) {
        (Op::Make(r), _) => {
            if !(tail && gives == Some(r.name.as_str())) {
                out.push(Made {
                    record: r.clone(),
                    make: e.clone(),
                    lets: lets.to_vec(),
                    guards: guards.to_vec(),
                    bound: names(e),
                });
            }
            for a in args {
                constructions(&a.body, gives, false, lets, guards, bound, out);
            }
        }
        (Op::Let, [value, body]) if value.binds.is_empty() && body.binds.len() == 1 => {
            constructions(&value.body, gives, false, lets, guards, bound, out);
            let mut inner = lets.to_vec();
            inner.push((body.binds[0].clone(), value.body.clone()));
            // A name bound again is the `let`'s, no longer a match's.
            let mut bound = bound.clone();
            bound.remove(&body.binds[0]);
            constructions(&body.body, gives, tail, &inner, guards, &bound, out);
        }
        (Op::If, [c, yes, no]) => {
            constructions(&c.body, gives, false, lets, guards, bound, out);
            let mut on = guards.to_vec();
            on.push(c.body.clone());
            constructions(&yes.body, gives, tail, lets, &on, bound, out);
            let mut off = guards.to_vec();
            off.push(Expression::Apply(Op::Not, vec![plain(c.body.clone())]));
            constructions(&no.body, gives, tail, lets, &off, bound, out);
        }
        _ => {
            for (i, a) in args.iter().enumerate() {
                let mut bound = bound.clone();
                bound.extend(a.binds.iter().cloned());
                // A `match` gives what its arms give.
                let tail = tail && matches!(op, Op::Match(_)) && i > 0;
                constructions(&a.body, gives, tail, lets, guards, &bound, out);
            }
        }
    }
}

/// The values a row's world holds: what each function with no body
/// was given and gave, and the calls its `then` lists.
fn world_values<'e>(
    given: &'e crate::model::Given,
    effects: &'e crate::model::Effects,
) -> Vec<(String, &'e Value)> {
    let mut values = Vec::new();
    for (g, args, v) in &given.0 {
        values.extend(args.iter().flatten().map(|a| (format!("{g}(…)"), a)));
        values.push((format!("{g}()"), v));
    }
    for (g, args) in &effects.0 {
        values.extend(args.iter().flatten().map(|a| (format!("{g}(…)"), a)));
    }
    values
}

/// What has no value, for messages: `` `a::b` ``, or for a record's
/// `..`, the fields its model does not state.
fn unstated(g: &impl std::fmt::Display) -> String {
    let g = g.to_string();
    match g.strip_suffix(&format!("::{}", crate::model::REST)) {
        Some(r) => format!("the fields of `{r}` the model does not state (`..`)"),
        None => format!("`{g}`"),
    }
}
