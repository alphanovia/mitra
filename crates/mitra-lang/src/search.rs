//! Two readings on symbolic runs (dry run, `exp`): the model checker,
//! `mitra mc`, and the invariant search, `mitra search`.
//!
//! The model checker runs each function with a body on its inputs left
//! free, the results of functions with no body unknown (`symbolic`), and
//! reads each of its claims on every path. A claim false on a path the
//! claims of the functions with no body allow is a finding (A024), with
//! the path written as an example row. With `steps`, it runs a function
//! `step(s, e) -> s` over a sequence of free events instead, a few steps
//! deep, and reads its claims at each step.
//!
//! The invariant search runs the examples of the model, each with the
//! given inputs, its `given`s dropped, so that every opaque result is
//! unknown. On each path it reads the claims of every function with a
//! body the run passes through. A path on which one breaks, and which no
//! claim of a function with no body rules out, says a condition the
//! claim needs of the opaque results that nothing promises (A025): a
//! missing promise. It does not guess invariants from the rows' values.

use crate::catalogue::{Finding, A024, A025};
use crate::expression::{eval, Arg, Expression, Fault};
use crate::model::{calls, Function, Index, Model, Path};
use crate::print;
use crate::simulation::{Calls, Exact, Op, Type, Undefined, Value};
use crate::symbolic::{explore, Explored, Free};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// How far the two readings look: each a setting.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    pub symbolic: crate::symbolic::Settings,
    /// The most conditions shown for one claim; the others are counted.
    pub shown: usize,
    /// How many steps `mc --steps` runs a step function.
    pub steps: usize,
    pub budget: crate::simulation::Budget,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            symbolic: Default::default(),
            shown: 3,
            steps: 3,
            budget: Default::default(),
        }
    }
}

/// What a claim came to over every path read.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// True on every path, none of them sampled, the exploration whole.
    Holds,
    /// True on every path read, but some took a sample of a number or a
    /// text, or the budget of runs was reached: not shown.
    HoldsOnRead { paths: usize, why: String },
    /// False on a path the promises allow.
    Fails,
    /// No path read gave it a value.
    Unknown { why: String },
}

/// One claim's outcome.
#[derive(Clone, Debug)]
pub struct Outcome {
    pub at: Path,
    pub claim: String,
    pub verdict: Verdict,
    /// The paths read, and those the promises ruled out.
    pub paths: usize,
    pub excluded: usize,
}

/// What a reading found.
#[derive(Clone, Debug, Default)]
pub struct Report {
    pub outcomes: Vec<Outcome>,
    pub findings: Vec<Finding>,
    /// Runs made, splits included; paths ended.
    pub runs: usize,
}

/// Evaluate `holds` of `f` at a run, the body traced when it reads calls.
fn claim_of(f: &Function, body: &Expression<Op>, holds: &Expression<Op>) -> Expression<Op> {
    let run = match crate::symbolic::reads_trace(holds) {
        true => Expression::Apply(
            Op::Traced,
            vec![Arg {
                binds: vec![],
                body: body.clone(),
            }],
        ),
        false => body.clone(),
    };
    Expression::Apply(
        Op::Let,
        vec![
            Arg {
                binds: vec![],
                body: run,
            },
            Arg {
                binds: vec![f.gives.0.clone()],
                body: holds.clone(),
            },
        ],
    )
}

/// What a claim gave on one path.
#[derive(Clone, Debug, PartialEq)]
pub enum Read {
    True,
    False(Option<Value>),
    NoValue(String),
}

fn read_of(outcome: Result<Value, Fault<Undefined>>) -> Result<Read, u32> {
    match outcome {
        Ok(Value::Truth(true)) => Ok(Read::True),
        Ok(Value::Truth(false)) => Ok(Read::False(None)),
        Err(Fault::Op(Undefined::Split(id))) => Err(id),
        Err(Fault::Op(Undefined::Unknown(g))) => Ok(Read::NoValue(format!("no value for `{g}`"))),
        Err(Fault::Op(Undefined::Budget(why))) => Ok(Read::NoValue(why)),
        Err(Fault::Op(e @ Undefined::EndlessRepeat { .. })) => Ok(Read::NoValue(e.endless(None).unwrap_or_default())),
        // Not read: a name it has no value for, or of another type.
        Err(Fault::Stuck) => Ok(Read::NoValue("it cannot be read here".to_string())),
        // Another fault: a claim with no value does not hold.
        Err(_) => Ok(Read::False(None)),
        Ok(v) => Ok(Read::False(Some(v))),
    }
}

/// The model checker: each function with a body and claims, its inputs
/// free.
pub fn mc(model: &Model, settings: &Settings, only: Option<&str>) -> Report {
    let Ok(library) = calls(model) else {
        return Report::default();
    };
    let library = Arc::new(library.budget(settings.budget));
    let mut report = Report::default();
    for part in &model.parts {
        if part.file.as_deref().is_some_and(|f| f.starts_with("std/")) {
            continue;
        }
        for f in &part.functions {
            let at = Path::to(&[&part.name, &f.name]);
            if only.is_some_and(|o| o != at.to_string()) {
                continue;
            }
            let Some(body) = &f.body else { continue };
            if f.claims.is_empty() {
                continue;
            }
            let free: Free = f.takes.clone();
            // `by invariant n`: `n` is the loop's, not read here.
            let claims: Vec<(String, Expression<Op>)> = f
                .claims
                .iter()
                .filter(|c| c.invariant.as_ref().is_none_or(|i| i.turns.is_none()))
                .map(|c| (c.name.clone(), claim_of(f, body, &c.holds)))
                .collect();
            if claims.is_empty() {
                continue;
            }
            let call = Expression::Apply(
                Op::Call(at.to_string()),
                f.takes.iter().map(|(n, _)| Arg { binds: vec![], body: Expression::Name(n.clone()) }).collect(),
            );
            check_claims(&library, &at, &free, &claims, Some(&call), settings, &mut report, |inputs, path, ex| {
                row_for(ex, path, &at, &free, inputs)
            });
        }
    }
    report
}

/// The model checker over steps: `step(s, e) -> s` from `init()`, or a
/// free state, over `settings.steps` free events; its claims read at
/// each step.
pub fn mc_steps(model: &Model, settings: &Settings, step: &str, init: Option<&str>) -> Result<Report, String> {
    let index = Index::of(model);
    let f = index.functions.get(step).ok_or(format!("`{step}` is no function of the model"))?;
    let body = f.body.as_ref().ok_or(format!("`{step}` has no body to run"))?;
    let [(s_name, s_type), (e_name, e_type)] = f.takes.as_slice() else {
        return Err(format!("`{step}` takes a state and an event: two inputs"));
    };
    if f.gives.1 != *s_type {
        return Err(format!("`{step}` gives another type than the state it takes"));
    }
    let library = Arc::new(calls(model).map_err(|_| "the model's functions are not well formed".to_string())?.budget(settings.budget));
    let mut free: Free = (1..=settings.steps).map(|k| (format!("{e_name}{k}"), e_type.clone())).collect();
    let start = match init {
        Some(g) => {
            let gf = index.functions.get(g).ok_or(format!("`{g}` is no function of the model"))?;
            if !gf.takes.is_empty() || gf.gives.1 != *s_type {
                return Err(format!("`{g}` takes nothing and gives the state"));
            }
            Expression::Apply(Op::Call(g.to_string()), vec![])
        }
        None => {
            free.insert(0, (format!("{s_name}0"), s_type.clone()));
            Expression::Name(format!("{s_name}0"))
        }
    };
    // Each claim, read after each step: step k's state from step k - 1's.
    let mut claims = Vec::new();
    for c in &f.claims {
        let mut state = start.clone();
        let mut conj: Option<Expression<Op>> = None;
        for k in 1..=settings.steps {
            let call = Expression::Apply(
                Op::Call(step.to_string()),
                vec![
                    Arg { binds: vec![], body: state.clone() },
                    Arg { binds: vec![], body: Expression::Name(format!("{e_name}{k}")) },
                ],
            );
            let lets = |inner: Expression<Op>| {
                Expression::Apply(
                    Op::Let,
                    vec![
                        Arg { binds: vec![], body: state.clone() },
                        Arg {
                            binds: vec![s_name.clone()],
                            body: Expression::Apply(
                                Op::Let,
                                vec![
                                    Arg { binds: vec![], body: Expression::Name(format!("{e_name}{k}")) },
                                    Arg { binds: vec![e_name.clone()], body: inner },
                                ],
                            ),
                        },
                    ],
                )
            };
            let here = lets(claim_of(f, body, &c.holds));
            conj = Some(match conj {
                None => here,
                Some(before) => Expression::Apply(
                    Op::And,
                    vec![Arg { binds: vec![], body: before }, Arg { binds: vec![], body: here }],
                ),
            });
            state = call;
        }
        if let Some(e) = conj {
            claims.push((c.name.clone(), e));
        }
    }
    let at = Path::to(&step.split("::").collect::<Vec<_>>());
    let mut report = Report::default();
    let shown_free = free.clone();
    check_claims(&library, &at, &free, &claims, None, settings, &mut report, |inputs, path, ex| {
        let mut said: Vec<String> = ex
            .givens(path)
            .iter()
            .map(|(g, args, v)| {
                let args: Vec<String> = args.iter().map(|a| witness_text(a)).collect();
                format!("{g}({}) -> {}", args.join(", "), witness_text(v))
            })
            .collect();
        let events: Vec<String> = ex
            .inputs(path, &shown_free)
            .iter()
            .map(|(n, v)| format!("{n} = {}", witness_text(v)))
            .collect();
        let _ = inputs;
        said.sort();
        said.dedup();
        format!(
            "the steps: {}{}",
            events.join(", "),
            match said.is_empty() {
                true => String::new(),
                false => format!("; given {}", said.join(", ")),
            }
        )
    });
    Ok(report)
}

/// Each claim over every path of `free`, in one exploration: a claim
/// that needs a split splits the run for all.
#[allow(clippy::too_many_arguments)]
fn check_claims(
    library: &Arc<Calls>,
    at: &Path,
    free: &Free,
    claims: &[(String, Expression<Op>)],
    call: Option<&Expression<Op>>,
    settings: &Settings,
    report: &mut Report,
    row: impl Fn(&BTreeMap<String, Value>, &crate::symbolic::Path<Reads>, &Explored<Reads>) -> String,
) {
    let explored = explore(library, free, &settings.symbolic, &|exact, inputs| {
        // What the run gives, for the row; then each claim.
        let gave = match call {
            Some(call) => match eval(exact, inputs, call) {
                Ok(v) => Some(v),
                Err(Fault::Op(Undefined::Split(id))) => return Err(Fault::Op(Undefined::Split(id))),
                Err(_) => None,
            },
            None => None,
        };
        let mut here = Vec::new();
        for (_, e) in claims {
            // Each claim a run of its own, its calls kept.
            let exact = Exact {
                calls: library.clone(),
                log: crate::simulation::Log::kept(),
            };
            match read_of(eval(&exact, inputs, e)) {
                Ok(r) => here.push(r),
                Err(id) => return Err(Fault::Op(Undefined::Split(id))),
            }
        }
        Ok((here, gave))
    });
    report.runs += explored.paths.len();
    let ended: Vec<(&crate::symbolic::Path<Reads>, &Vec<Read>)> = explored
        .paths
        .iter()
        .filter_map(|p| p.outcome.as_ref().ok().map(|r| (p, &r.0)))
        .collect();
    // Paths ruled out before they ended (pruned), counted with the rest.
    let pruned: Vec<&String> = explored
        .paths
        .iter()
        .filter(|p| p.outcome.is_err())
        .filter_map(|p| p.excluded.as_ref())
        .collect();
    for (k, (name, _)) in claims.iter().enumerate() {
        let mut fails: Vec<&crate::symbolic::Path<Reads>> = Vec::new();
        let (mut paths, mut excluded, mut sampled, mut none) = (0, 0, false, None);
        let mut by: BTreeMap<String, usize> = BTreeMap::new();
        for why in &pruned {
            excluded += 1;
            *by.entry((*why).clone()).or_default() += 1;
        }
        for (p, r) in &ended {
            if let Some(why) = &p.excluded {
                excluded += 1;
                *by.entry(why.clone()).or_default() += 1;
                continue;
            }
            paths += 1;
            sampled |= p.sampled;
            match &r[k] {
                Read::True => {}
                Read::False(_) => fails.push(p),
                Read::NoValue(why) => _ = none.get_or_insert(why.clone()),
            }
        }
        let verdict = if !fails.is_empty() {
            Verdict::Fails
        } else if let Some(why) = none {
            Verdict::Unknown { why }
        } else if paths == 0 {
            Verdict::Unknown {
                why: format!("no path the promises allow, of {excluded}: {}", ruled(&by)),
            }
        } else if sampled || explored.cut || !explored.stuck.is_empty() {
            let why = match (explored.cut, sampled) {
                (true, _) => format!("the budget of {} runs was reached", settings.symbolic.runs),
                (false, true) => "some paths took sample numbers, texts or lengths".to_string(),
                _ => format!("no case of {}", explored.stuck.join(", ")),
            };
            Verdict::HoldsOnRead { paths, why }
        } else {
            Verdict::Holds
        };
        if let Some(p) = fails.first() {
            let inputs: BTreeMap<String, Value> = BTreeMap::new();
            report.findings.push(Finding {
                code: A024,
                at: at.clone(),
                message: format!(
                    "claim `{name}` is false on {} of {paths} paths; the first: {}",
                    fails.len(),
                    row(&inputs, p, &explored)
                ),
            });
        }
        report.outcomes.push(Outcome {
            at: at.clone(),
            claim: name.clone(),
            verdict,
            paths,
            excluded,
        });
    }
}

/// Why paths were ruled out, the most first.
fn ruled(by: &BTreeMap<String, usize>) -> String {
    let mut by: Vec<(&String, &usize)> = by.iter().collect();
    by.sort_by(|a, b| b.1.cmp(a.1).then(a.0.cmp(b.0)));
    let shown: Vec<String> = by.iter().take(3).map(|(w, n)| format!("{w} on {n}")).collect();
    shown.join(", ")
}

/// A path as an example row of the model: `given … when f(…) then …`.
fn row_for(ex: &Explored<Reads>, path: &crate::symbolic::Path<Reads>, at: &Path, free: &Free, _inputs: &BTreeMap<String, Value>) -> String {
    let mut given: Vec<String> = ex
        .givens(path)
        .iter()
        .map(|(g, args, v)| {
            let args: Vec<String> = args.iter().map(witness_text).collect();
            format!("{g}({}) -> {}", args.join(", "), witness_text(v))
        })
        .collect();
    given.dedup();
    let args: Vec<String> = ex.inputs(path, free).iter().map(|(_, v)| witness_text(v)).collect();
    let given = match given.is_empty() {
        true => String::new(),
        false => format!("given {} ", given.join(", ")),
    };
    let then = match &path.outcome {
        Ok((_, Some(v))) => format!(" then {}", witness_text(&ex.filled(path, v))),
        _ => " (it gives no value)".to_string(),
    };
    format!("{given}when {at}({}){then}", args.join(", "))
}

/// What a path read of each claim, and what the run gave.
pub type Reads = (Vec<Read>, Option<Value>);

/// A value of a row: what the path did not read written as `_`, which a
/// row reads as any value.
fn witness_text(v: &Value) -> String {
    print::value(v)
}

/// The invariant search over every example of the model, its `given`s
/// dropped when `free` (else kept, only what they do not give unknown).
pub fn search(model: &Model, settings: &Settings, free: bool) -> Report {
    let Ok(library) = calls(model) else {
        return Report::default();
    };
    let index = Index::of(model);
    let library = Arc::new(library.budget(settings.budget));
    let mut report = Report::default();
    // Each example: where it is, the function `when` calls, its inputs,
    // and its world.
    let mut rows: Vec<(Path, String, Vec<Value>, crate::model::Given)> = Vec::new();
    for part in &model.parts {
        if part.file.as_deref().is_some_and(|f| f.starts_with("std/")) {
            continue;
        }
        for f in &part.functions {
            if f.body.is_none() {
                continue;
            }
            let at = Path::to(&[&part.name, &f.name]);
            for (i, e) in f.examples.iter().enumerate() {
                if e.takes.iter().all(crate::simulation::known) {
                    rows.push((at.then(&format!("example {i}")), at.to_string(), e.takes.clone(), e.given.clone()));
                }
            }
        }
    }
    for (i, e) in model.examples.iter().enumerate() {
        let Some((f, args, _)) = &e.when else {
            // Read once settled: its inputs as ports of one function.
            let mut by: BTreeMap<String, Vec<(String, Value)>> = BTreeMap::new();
            for (p, v) in &e.inputs {
                let names = p.to_string();
                if let Some((f, n)) = names.rsplit_once("::") {
                    by.entry(f.to_string()).or_default().push((n.to_string(), v.clone()));
                }
            }
            for (f, named) in by {
                let Some(function) = index.functions.get(f.as_str()) else { continue };
                let args: Option<Vec<Value>> = function
                    .takes
                    .iter()
                    .map(|(n, _)| named.iter().find(|(m, _)| m == n).map(|(_, v)| v.clone()))
                    .collect();
                if let Some(args) = args {
                    rows.push((Path::to(&["examples", &i.to_string()]), f, args, e.given.clone()));
                }
            }
            continue;
        };
        rows.push((Path::to(&["examples", &i.to_string()]), f.to_string(), args.clone(), e.given.clone()));
    }
    // Claims of each function with a body, by path: a route claim with
    // the body run again, traced; any other read of what the call gave.
    type Claims<'a> = BTreeMap<String, (&'a Function, Vec<(String, bool, Expression<Op>)>)>;
    let claims: Claims = index
        .functions
        .iter()
        .filter_map(|(p, f)| {
            let body = f.body.as_ref()?;
            let cs: Vec<(String, bool, Expression<Op>)> = f
                .claims
                .iter()
                .filter(|c| c.invariant.as_ref().is_none_or(|i| i.turns.is_none()))
                .map(|c| match crate::symbolic::reads_trace(&c.holds) {
                    true => (c.name.clone(), true, claim_of(f, body, &c.holds)),
                    false => (c.name.clone(), false, c.holds.clone()),
                })
                .collect();
            Some((p.clone(), (*f, cs)))
        })
        .collect();
    // Needs found, by claim: each condition once.
    let mut needs: BTreeMap<(String, String), (Vec<String>, BTreeSet<String>)> = BTreeMap::new();
    let mut tally: BTreeMap<(String, String), (usize, usize)> = BTreeMap::new();
    let trace_rows = std::env::var("MITRA_SEARCH_TRACE").is_ok();
    for (row_at, f, args, given) in rows {
        let started = std::time::Instant::now();
        let world = match free {
            true => library.clone(),
            false => {
                let mut c = (*library).clone();
                for (g, a, v) in &given.0 {
                    c = c.given_any(g, a.clone(), v.clone()).unwrap_or_else(|same| same);
                }
                Arc::new(c)
            }
        };
        // The run itself first, then the claims of each body it passed,
        // at what each was called with: all in the one run.
        let call = Expression::Apply(
            Op::Call(f.clone()),
            (0..args.len())
                .map(|k| Arg { binds: vec![], body: Expression::Name(format!("#{k}")) })
                .collect(),
        );
        let inputs: BTreeMap<String, Value> =
            args.iter().enumerate().map(|(k, v)| (format!("#{k}"), v.clone())).collect();
        let explored = explore(&world, &vec![], &settings.symbolic, &|exact, _| {
            let ran = eval(exact, &inputs, &call);
            if let Err(Fault::Op(Undefined::Split(id))) = ran {
                return Err(Fault::Op(Undefined::Split(id)));
            }
            let mut here = Vec::new();
            let mut read: BTreeSet<(String, Vec<Value>)> = BTreeSet::new();
            for c in exact.log.trace() {
                if !c.body || !read.insert(c.made.clone()) {
                    continue;
                }
                let Some((function, cs)) = claims.get(&c.made.0) else { continue };
                let mut values: BTreeMap<String, Value> =
                    function.takes.iter().map(|(n, _)| n.clone()).zip(c.made.1.iter().cloned()).collect();
                let Some(gave) = &c.gave else { continue };
                values.insert(function.gives.0.clone(), gave.clone());
                for (name, routed, e) in cs {
                    let exact = Exact {
                        calls: world.clone(),
                        log: match routed {
                            true => crate::simulation::Log::kept(),
                            false => Default::default(),
                        },
                    };
                    match read_of(eval(&exact, &values, e)) {
                        Ok(r) => here.push((c.made.0.clone(), name.clone(), r)),
                        Err(id) => return Err(Fault::Op(Undefined::Split(id))),
                    }
                }
            }
            Ok(here)
        });
        report.runs += explored.paths.len();
        if trace_rows {
            eprintln!("{row_at} {f}: {} paths, cut {}, {:?}", explored.paths.len(), explored.cut, started.elapsed());
        }
        let ended = explored.paths.iter().filter_map(|p| p.outcome.as_ref().ok().map(|r| (p, r)));
        for (p, reads) in ended {
            for (g, name, r) in reads {
                let key = (g.clone(), name.clone());
                let t = tally.entry(key.clone()).or_default();
                if p.excluded.is_some() {
                    continue;
                }
                t.0 += 1;
                if let Read::False(_) = r {
                    t.1 += 1;
                    let cond = explored.condition(p).join(", ");
                    let cond = match cond.is_empty() {
                        true => "on every path: no opaque result read".to_string(),
                        false => cond,
                    };
                    let entry = needs.entry(key).or_default();
                    if !entry.0.contains(&cond) {
                        entry.0.push(cond);
                    }
                    entry.1.insert(row_at.to_string());
                }
            }
        }
    }
    for ((g, name), (conds, rows)) in &needs {
        let shown: Vec<String> = conds.iter().take(settings.shown).map(|c| format!("not ({c})")).collect();
        let more = match conds.len() > settings.shown {
            true => format!(", and {} more", conds.len() - settings.shown),
            false => String::new(),
        };
        let rows: Vec<&str> = rows.iter().map(String::as_str).collect();
        report.findings.push(Finding {
            code: A025,
            at: Path::to(&g.split("::").collect::<Vec<_>>()).then(name),
            message: format!(
                "claim `{name}` holds on the runs of {} only if {}{more}: no claim of a function with no body says so",
                rows.join(", "),
                shown.join("; or ")
            ),
        });
    }
    // A claim of a function with a body that no row runs is said, not
    // left out (no silent diagnostics).
    for part in &model.parts {
        if part.file.as_deref().is_some_and(|f| f.starts_with("std/")) {
            continue;
        }
        for f in part.functions.iter().filter(|f| f.body.is_some()) {
            let g = Path::to(&[&part.name, &f.name]).to_string();
            for c in &f.claims {
                if c.invariant.as_ref().is_none_or(|i| i.turns.is_none()) && !tally.contains_key(&(g.clone(), c.name.clone())) {
                    report.outcomes.push(Outcome {
                        at: Path::to(&[&part.name, &f.name]),
                        claim: c.name.clone(),
                        verdict: Verdict::Unknown {
                            why: "no example runs it: `mitra mc` reads it on free inputs".to_string(),
                        },
                        paths: 0,
                        excluded: 0,
                    });
                }
            }
        }
    }
    for ((g, name), (paths, failed)) in tally {
        report.outcomes.push(Outcome {
            at: Path::to(&g.split("::").collect::<Vec<_>>()),
            claim: name,
            verdict: match failed {
                0 => Verdict::HoldsOnRead { paths, why: "the paths of the examples only".to_string() },
                _ => Verdict::Fails,
            },
            paths,
            excluded: 0,
        });
    }
    report
}

/// The type of what `f` gives, for a row of it.
pub fn gives(f: &Function) -> &Type {
    &f.gives.1
}
