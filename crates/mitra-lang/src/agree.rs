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

use crate::catalogue::{Code, Finding, Hint, A001, A002, A003, A004, A005, A006, A007, A009, A010};
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
    for c in contract_claims(model) {
        let agreement = reading.claim(&c.over, &c.holds, &[]);
        agreed.push(Agreed {
            at: c.at,
            against: "every value of what it reads".to_string(),
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
    findings.extend(agreed.iter().filter_map(finding));
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
            },
            ranges: std::cell::RefCell::new(Ranges {
                calls: library.clone(),
            }),
            induction: BTreeMap::new(),
            invariant: BTreeMap::new(),
            library,
            opaque,
            functions: index.functions.clone(),
            settings,
            last_given: Default::default(),
            domains: Default::default(),
            bounds_budget: Default::default(),
        }
    }

    /// The model's functions, with what the environment gave in `world`
    /// read first (finding 16); as they are, in none.
    fn in_world(&self, world: &crate::judgement::World) -> Exact {
        if world.0.is_empty() {
            return self.exact.clone();
        }
        let mut calls = (*self.exact.calls).clone();
        for (g, args, v) in &world.0 {
            calls = calls
                .given(g, args.clone(), v.clone())
                .unwrap_or_else(|same| same);
        }
        Exact {
            calls: std::sync::Arc::new(calls),
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
        let takes_in = conjunction(f.ranges.takes.iter().map(|(n, r)| in_range(n, r)));
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
                        (agreement, _) => agreement,
                    },
                };
                push(format!("claim `{}`", c.name), agreement);
            }
            // The range of what it gives, as a claim the body keeps.
            if let Some((q, range)) = &f.ranges.gives {
                let with_body = let_in(&f.gives.0, body, &read_in(in_range(q, range)));
                let agreement = self.claim(&f.takes, &with_body, &rows);
                push(format!("its range `{q}: {range}`"), agreement);
            }
            // Each example: the body at its point.
            for (i, (e, row)) in f.examples.iter().zip(&rows).enumerate() {
                let exact = self.in_world(&e.given);
                let agreement = match eval(&exact, &row.iter().cloned().collect(), body) {
                    Ok(v) => match matches(&v, &e.gives) {
                        Ok(true) => Agreement::Holds,
                        Ok(false) => Agreement::Fails {
                            at: row.clone(),
                            gives: Some(v),
                        },
                        Err(g) => Agreement::Unknown {
                            why: format!("`{g}` is not known, and only it could tell"),
                        },
                    },
                    Err(Fault::Op(Undefined::Unknown(g))) => Agreement::Unknown {
                        why: format!("no value for {} here", unstated(&g)),
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
                let agreement = match eval(&self.in_world(&e.given), &values, &c.holds) {
                    Ok(Value::Truth(true)) => Agreement::Holds,
                    Err(Fault::Op(Undefined::Unknown(g))) => Agreement::Unknown {
                        why: format!("no value for {} here", unstated(&g)),
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
                if e.takes != args || !e.given.0.is_empty() {
                    continue;
                }
                let agreement = match matches(v, &e.gives) {
                    Ok(true) => Agreement::Holds,
                    Ok(false) => Agreement::Fails {
                        at: named(&port.function.takes, &args),
                        gives: Some(e.gives.clone()),
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
        let exact = Exact {
            calls: Arc::new(library),
        };
        for (p, expected) in &example.outputs {
            let Some(port) = port(index, p) else { continue };
            let Some(args) = args_of(&port.at, port.function) else {
                continue;
            };
            let point = named(&port.function.takes, &args);
            let (call, bound) = call_at(&port.at.to_string(), &args);
            let agreement = match eval(&exact, &bound, &call) {
                Ok(v) => match matches(&v, expected) {
                    Ok(true) => Agreement::Holds,
                    Ok(false) => Agreement::Fails {
                        at: point,
                        gives: Some(v),
                    },
                    Err(g) => Agreement::Unknown {
                        why: format!("`{g}` is not known, and only it could tell"),
                    },
                },
                Err(Fault::Op(Undefined::Unknown(g))) => Agreement::Unknown {
                    why: format!("no value for {} here", unstated(&g)),
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
                given: vec![],
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
        message: format!("{message}{}", given(&a.given)),
    })
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
