//! Symbolic runs (dry run, `exp`): a function with no body, called where
//! no row and no `given` says what it gives, gives an unknown value
//! instead of none. The run goes on; where an operation must read the
//! unknown (an `if` on it, a `match`, a field, `==`), it is refined: a
//! record into unknown fields, at once; a truth, an enum, a sequence or a
//! number by cases, each case a run of its own. Each run keeps the cases
//! it took: its condition, said of the opaque calls.
//!
//! Truths, enums, records, sequences up to a length are split exactly:
//! their cases are all the values there are. Numbers and text are not:
//! a few values are tried (`Samples`), and a run that took one is
//! `sampled`, so that what it shows is never read as shown for all.
//!
//! The state of the run under way is kept per thread, so that every
//! reading of `Exact` refines the same way without being told.

use crate::expression::{eval, Fault, Outcome};
use crate::simulation::{Calls, Enum, Exact, Log, Logged, Machine, Op, Record, Type, Undefined, Value};
use num_bigint::BigInt;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type Exactly = Outcome<Value, Undefined>;

/// Where an unknown value came from.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Origin {
    /// What a function with no body gave at these arguments.
    Call(String, Vec<Value>),
    /// An input left free, by its name.
    Input(String),
    /// A part of another unknown: of its case (a variant, a length, or
    /// 0 for a record), at this place.
    Part(u32, usize, usize),
}

/// The values tried for a number or a text: a few, never all.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Samples {
    pub ints: Vec<BigInt>,
    pub texts: Vec<String>,
    /// The longest sequence tried: from `[]` up to this length.
    pub length: usize,
}

impl Default for Samples {
    fn default() -> Samples {
        Samples {
            ints: [0, 1, -1, 2].into_iter().map(BigInt::from).collect(),
            texts: vec![String::new(), "a".to_string()],
            length: 2,
        }
    }
}

/// The unknowns of one exploration, each with its origin and type, kept
/// across its runs so that one call is one unknown in all of them.
#[derive(Debug, Default)]
struct Table {
    made: Vec<(Origin, Type)>,
    by_call: BTreeMap<(String, Vec<Value>), u32>,
    by_part: BTreeMap<(u32, usize, usize), u32>,
    /// The enums by name, whole, for the parts of a tree.
    enums: BTreeMap<String, Enum>,
    samples: Samples,
}

impl Table {
    fn intern(&mut self, origin: Origin, of: Type) -> u32 {
        if let Type::Enum(e) = &of {
            if !e.cut() {
                self.enums.entry(e.name.clone()).or_insert_with(|| e.clone());
            }
        }
        let key = match &origin {
            Origin::Call(f, args) => Some((f.clone(), args.clone())),
            _ => None,
        };
        if let Some(k) = &key {
            if let Some(id) = self.by_call.get(k) {
                return *id;
            }
        }
        if let Origin::Part(p, c, k) = origin {
            if let Some(id) = self.by_part.get(&(p, c, k)) {
                return *id;
            }
        }
        let id = self.made.len() as u32;
        self.made.push((origin.clone(), of));
        match origin {
            Origin::Call(..) => _ = self.by_call.insert(key.expect("a call"), id),
            Origin::Part(p, c, k) => _ = self.by_part.insert((p, c, k), id),
            Origin::Input(_) => {}
        }
        id
    }
    fn whole(&self, e: &Enum) -> Enum {
        match e.cut() {
            true => self.enums.get(&e.name).cloned().unwrap_or_else(|| e.clone()),
            false => e.clone(),
        }
    }
}

/// The run under way: the exploration's unknowns, and the cases this
/// run takes, in the order taken.
#[derive(Debug)]
struct Run {
    table: Arc<std::sync::Mutex<Table>>,
    fixed: BTreeMap<u32, Value>,
    order: Vec<u32>,
    /// The unknowns opaque calls gave in this run, in order.
    called: Vec<u32>,
    sampled: bool,
    /// While a claim is read: the unknowns it read (choice I4).
    reads: Option<BTreeSet<u32>>,
}

thread_local! {
    static RUN: RefCell<Option<Run>> = const { RefCell::new(None) };
}

/// Whether a symbolic run is under way on this thread.
pub fn active() -> bool {
    RUN.with(|r| r.borrow().is_some())
}

/// In a symbolic run, the unknown a call of `f` at `args` gives, `of`
/// its type; `None` outside one.
pub fn unknown_for(f: &str, args: &[Value], of: &Type) -> Option<u32> {
    RUN.with(|r| {
        let mut r = r.borrow_mut();
        let run = r.as_mut()?;
        let id = run
            .table
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .intern(Origin::Call(f.to_string(), args.to_vec()), of.clone());
        if !run.called.contains(&id) {
            run.called.push(id);
        }
        Some(id)
    })
}

/// The type of an unknown.
pub fn type_of(id: u32) -> Option<Type> {
    RUN.with(|r| {
        let r = r.borrow();
        let run = r.as_ref()?;
        let table = run.table.lock().unwrap_or_else(|e| e.into_inner());
        table.made.get(id as usize).map(|(_, t)| t.clone())
    })
}

/// From now, the unknowns read are kept, until `reads_taken`.
pub fn reads_kept() {
    RUN.with(|r| {
        if let Some(run) = r.borrow_mut().as_mut() {
            run.reads = Some(BTreeSet::new());
        }
    })
}

/// The unknowns read since `reads_kept`, and no longer kept.
pub fn reads_taken() -> BTreeSet<u32> {
    RUN.with(|r| r.borrow_mut().as_mut().and_then(|run| run.reads.take()).unwrap_or_default())
}

/// Whether `v` holds an unknown anywhere.
pub fn holds_unknown(v: &Value) -> bool {
    match v {
        Value::Unknown(_) => true,
        Value::Record(_, vs) | Value::Variant(_, _, vs) | Value::Seq(_, vs) => vs.iter().any(holds_unknown),
        Value::Partial(_, vs) => vs.iter().flatten().any(holds_unknown),
        _ => false,
    }
}

/// `v`, its top known: an unknown refined, or a split. Any other value
/// as it is.
pub fn resolve(v: &Value) -> Exactly {
    let Value::Unknown(id) = v else {
        return Ok(v.clone());
    };
    RUN.with(|r| {
        let mut r = r.borrow_mut();
        let Some(run) = r.as_mut() else {
            return Err(Fault::Stuck);
        };
        if let Some(reads) = &mut run.reads {
            reads.insert(*id);
        }
        if let Some(known) = run.fixed.get(id) {
            return Ok(known.clone());
        }
        let mut table = run.table.lock().unwrap_or_else(|e| e.into_inner());
        let Some((_, of)) = table.made.get(*id as usize).cloned() else {
            return Err(Fault::Stuck);
        };
        // A record has one case: its fields, each unknown.
        if let Type::Record(rec) = &of {
            let fields = rec
                .fields
                .iter()
                .enumerate()
                .map(|(k, (_, t))| Value::Unknown(table.intern(Origin::Part(*id, 0, k), t.clone())))
                .collect();
            let known = Value::Record(rec.clone(), fields);
            drop(table);
            run.fixed.insert(*id, known.clone());
            run.order.push(*id);
            return Ok(known);
        }
        Err(Fault::Op(Undefined::Split(*id)))
    })
}

/// An outcome, its value's top known.
pub fn resolved(v: &Exactly) -> Exactly {
    match v {
        Ok(v) => resolve(v),
        Err(f) => Err(f.clone()),
    }
}

/// `v` known all through, every unknown in it refined.
pub fn deep(v: &Value) -> Exactly {
    let v = resolve(v)?;
    Ok(match v {
        Value::Record(r, vs) => Value::Record(r, vs.iter().map(deep).collect::<Result<_, _>>()?),
        Value::Variant(e, i, vs) => Value::Variant(e, i, vs.iter().map(deep).collect::<Result<_, _>>()?),
        Value::Seq(t, vs) => Value::Seq(t, vs.iter().map(deep).collect::<Result<_, _>>()?),
        Value::Partial(r, vs) => Value::Partial(
            r,
            vs.iter().map(|v| v.as_ref().map(deep).transpose()).collect::<Result<_, _>>()?,
        ),
        v => v,
    })
}

/// The cases of an unknown of type `of`, its parts unknowns of their
/// own; and whether they are only samples. `None` when it has none
/// Mitra can write (a `char`, a type parameter).
fn cases(table: &mut Table, id: u32, of: &Type) -> Option<(Vec<Value>, bool)> {
    let samples = table.samples.clone();
    Some(match of {
        Type::Truth => (vec![Value::Truth(false), Value::Truth(true)], false),
        Type::Enum(e) => {
            let e = table.whole(e);
            let mut all = Vec::new();
            for (i, (_, holds)) in e.variants.iter().enumerate() {
                let parts = holds
                    .iter()
                    .enumerate()
                    .map(|(k, t)| {
                        let t = match t {
                            Type::Enum(c) if c.cut() => Type::Enum(table.whole(c)),
                            t => t.clone(),
                        };
                        Value::Unknown(table.intern(Origin::Part(id, i, k), t))
                    })
                    .collect();
                all.push(Value::Variant(e.clone(), i, parts));
            }
            (all, false)
        }
        Type::Seq(t) => {
            let mut all = Vec::new();
            for n in 0..=samples.length {
                let holds = (0..n)
                    .map(|k| Value::Unknown(table.intern(Origin::Part(id, n, k), (**t).clone())))
                    .collect();
                all.push(Value::Seq((**t).clone(), holds));
            }
            // Longer ones are not tried: a sequence is split by samples.
            (all, true)
        }
        Type::Int => (samples.ints.iter().cloned().map(Value::Int).collect(), true),
        Type::Machine(m) => (machine_samples(m, &samples.ints), true),
        Type::Real => (
            [0, 1, -1]
                .into_iter()
                .map(|n| Value::Real(crate::simulation::Rational::integer(n)))
                .collect(),
            true,
        ),
        Type::Encoded(_) => (samples.texts.iter().cloned().map(Value::Text).collect(), true),
        Type::Byte => (vec![Value::Byte(0), Value::Byte(b'a')], true),
        Type::Record(r) => (
            vec![Value::Record(
                r.clone(),
                r.fields
                    .iter()
                    .enumerate()
                    .map(|(k, (_, t))| Value::Unknown(table.intern(Origin::Part(id, 0, k), t.clone())))
                    .collect(),
            )],
            false,
        ),
        Type::Char | Type::Param { .. } => return None,
    })
}

/// The first value of a type, a few levels deep: what a row writes for
/// a value the path never read.
fn first_of(t: &Table, ty: &Type, depth: usize) -> Option<Value> {
    if depth > 6 {
        return None;
    }
    Some(match ty {
        Type::Truth => Value::Truth(false),
        Type::Int => Value::Int(BigInt::from(0)),
        Type::Real => Value::Real(crate::simulation::Rational::integer(0)),
        Type::Machine(m) => Value::Fixed(*m, if m.holds(&BigInt::from(0)) { BigInt::from(0) } else { m.lowest() }),
        Type::Encoded(_) => Value::Text(String::new()),
        Type::Byte => Value::Byte(0),
        Type::Seq(e) => Value::Seq((**e).clone(), vec![]),
        Type::Record(r) => Value::Record(
            r.clone(),
            r.fields.iter().map(|(_, f)| first_of(t, f, depth + 1)).collect::<Option<_>>()?,
        ),
        Type::Enum(e) => {
            let e = t.whole(e);
            // The first variant that ends, else the first.
            let order = (0..e.variants.len()).filter(|i| e.variants[*i].1.is_empty()).chain(0..e.variants.len());
            let mut found = None;
            for i in order {
                if let Some(holds) = e.variants[i].1.iter().map(|h| first_of(t, h, depth + 1)).collect::<Option<Vec<_>>>() {
                    found = Some(Value::Variant(e.clone(), i, holds));
                    break;
                }
            }
            found?
        }
        Type::Char | Type::Param { .. } => return None,
    })
}

/// The samples of a machine integer: those it holds, and its ends, where
/// overflows begin (choice S6).
fn machine_samples(m: &Machine, ints: &[BigInt]) -> Vec<Value> {
    let mut all: Vec<BigInt> = ints.iter().filter(|n| m.holds(n)).cloned().collect();
    for end in [m.lowest(), m.highest()] {
        if !all.contains(&end) {
            all.push(end);
        }
    }
    all.into_iter().map(|n| Value::Fixed(*m, n)).collect()
}

/// How far an exploration goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    /// The most runs, splits included.
    pub runs: usize,
    pub samples: Samples,
    /// Paths up to this many cases away from the first are run before
    /// the others: 0, depth first.
    pub spread: usize,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            runs: 512,
            samples: Samples::default(),
            spread: 0,
        }
    }
}

/// One path: the cases it took, by unknown, in order; what it gave; the
/// calls it made; whether it took a sample; and the claim of a function
/// with no body its cases break, when one does (the path cannot be).
#[derive(Clone, Debug)]
pub struct Path<T = Value> {
    pub cases: Vec<(u32, Value)>,
    pub outcome: Outcome<T, Undefined>,
    pub trace: Vec<Logged>,
    pub sampled: bool,
    pub excluded: Option<String>,
    /// The opaque calls of the run, as unknowns, in order.
    pub called: Vec<u32>,
    fixed: BTreeMap<u32, Value>,
}

/// What an exploration found: each path, whether it stopped at its
/// budget of runs, and the unknowns' origins and types.
#[derive(Clone, Debug)]
pub struct Explored<T = Value> {
    pub paths: Vec<Path<T>>,
    pub cut: bool,
    /// An unknown with no case Mitra can write, by its origin.
    pub stuck: Vec<String>,
    table: Arc<std::sync::Mutex<Table>>,
}

/// The inputs left free: each an unknown, by its name.
pub type Free = Vec<(String, Type)>;

/// A path still to run: how many of its cases are not the first, when
/// it was made, and what it fixes.
struct Pending {
    away: usize,
    made: u64,
    fixed: BTreeMap<u32, Value>,
    order: Vec<u32>,
    sampled: bool,
}

impl PartialEq for Pending {
    fn eq(&self, other: &Pending) -> bool {
        (self.away, self.made) == (other.away, other.made)
    }
}
impl Eq for Pending {}
impl PartialOrd for Pending {
    fn partial_cmp(&self, other: &Pending) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl Ord for Pending {
    /// The greatest is popped first: the fewest away, then the last made.
    fn cmp(&self, other: &Pending) -> std::cmp::Ordering {
        other.away.cmp(&self.away).then(self.made.cmp(&other.made))
    }
}

/// Every path of `run`, each opaque result an unknown: the inputs in
/// `free` unknowns too, given to `run` by name. `run` gets a log kept
/// for each run. The promises of each opaque call (its claims, read with
/// the unknown it gave) are read at the end of each run: a path they
/// refuse is `excluded`, not dropped.
pub fn explore<T: Clone>(
    calls: &Arc<Calls>,
    free: &Free,
    settings: &Settings,
    run: &dyn Fn(&Exact, &BTreeMap<String, Value>) -> Outcome<T, Undefined>,
) -> Explored<T> {
    let table = Arc::new(std::sync::Mutex::new(Table {
        samples: settings.samples.clone(),
        ..Table::default()
    }));
    let inputs: BTreeMap<String, Value> = {
        let mut t = table.lock().unwrap_or_else(|e| e.into_inner());
        free.iter()
            .map(|(n, ty)| (n.clone(), Value::Unknown(t.intern(Origin::Input(n.clone()), ty.clone()))))
            .collect()
    };
    // The paths to run, fewest cases away from the first first, up to
    // `spread` (limited discrepancy search, choice S10): a budget cut
    // leaves early choices tried, not one subtree read to its end. Among
    // as many, the last made first: with `spread` 0, a depth-first search.
    let mut stack: std::collections::BinaryHeap<Pending> = std::collections::BinaryHeap::new();
    let mut made = 0u64;
    stack.push(Pending { away: 0, made, fixed: BTreeMap::new(), order: vec![], sampled: false });
    let mut paths = Vec::new();
    let mut stuck = Vec::new();
    let mut runs = 0;
    let mut cut = false;
    while let Some(Pending { away, fixed, order, sampled, .. }) = stack.pop() {
        if runs >= settings.runs {
            cut = true;
            break;
        }
        runs += 1;
        RUN.with(|r| {
            *r.borrow_mut() = Some(Run {
                table: table.clone(),
                fixed: fixed.clone(),
                order: order.clone(),
                called: vec![],
                sampled,
                reads: None,
            })
        });
        let exact = Exact {
            calls: calls.clone(),
            log: Log::kept(),
        };
        let outcome = run(&exact, &inputs);
        let trace = exact.log.trace();
        // The promises of each opaque call, read in the same run.
        let mut excluded = None;
        let outcome = match outcome {
            // A promise already false rules the path out here: its cases
            // are not tried (choice S9, pruning).
            Err(Fault::Op(Undefined::Split(id))) => match broken_now(calls) {
                Some(broken) => {
                    excluded = Some(broken);
                    Err(Fault::Op(Undefined::Unknown("ruled out by a promise".to_string())))
                }
                None => Err(Fault::Op(Undefined::Split(id))),
            },
            outcome => match promises(calls) {
                Err(Err(split)) => Err(split),
                Err(Ok(_)) => Err(Fault::Stuck),
                Ok(broken) => {
                    excluded = broken;
                    outcome
                }
            },
        };
        let state = RUN.with(|r| r.borrow_mut().take()).expect("a run");
        match outcome {
            Err(Fault::Op(Undefined::Split(id))) => {
                let mut t = table.lock().unwrap_or_else(|e| e.into_inner());
                let of = t.made.get(id as usize).map(|(_, ty)| ty.clone());
                match of.and_then(|of| cases(&mut t, id, &of)) {
                    Some((all, by_samples)) => {
                        for (k, case) in all.into_iter().enumerate().rev() {
                            let mut fixed = state.fixed.clone();
                            fixed.insert(id, case);
                            let mut order = state.order.clone();
                            order.push(id);
                            made += 1;
                            stack.push(Pending {
                                away: (away + usize::from(k > 0)).min(settings.spread),
                                made,
                                fixed,
                                order,
                                sampled: state.sampled || by_samples,
                            });
                        }
                    }
                    None => {
                        stuck.push(origin_text(&t, id));
                        paths.push(Path {
                            cases: state.order.iter().map(|i| (*i, state.fixed[i].clone())).collect(),
                            outcome: Err(Fault::Op(Undefined::Unknown(origin_text(&t, id)))),
                            trace,
                            sampled: state.sampled,
                            excluded: None,
                            called: state.called.clone(),
                            fixed: state.fixed.clone(),
                        });
                    }
                }
            }
            outcome => paths.push(Path {
                cases: state.order.iter().map(|i| (*i, state.fixed[i].clone())).collect(),
                outcome,
                trace,
                sampled: state.sampled,
                excluded,
                called: state.called.clone(),
                fixed: state.fixed.clone(),
            }),
        }
    }
    Explored {
        paths,
        cut,
        stuck,
        table,
    }
}

/// A claim of a function with no body that the run's opaque calls
/// already break, whatever the unknowns not yet refined: those that need
/// a split are passed over.
fn broken_now(calls: &Arc<Calls>) -> Option<String> {
    let called: Vec<(u32, Origin)> = RUN.with(|r| {
        let r = r.borrow();
        let run = r.as_ref().expect("a run");
        let t = run.table.lock().unwrap_or_else(|e| e.into_inner());
        run.called.iter().map(|id| (*id, t.made[*id as usize].0.clone())).collect()
    });
    for (id, origin) in called {
        let Origin::Call(f, args) = origin else { continue };
        let Some(callee) = calls.get(&f) else { continue };
        let Some(promises) = &callee.promises else { continue };
        let mut values: BTreeMap<String, Value> =
            callee.takes.iter().map(|(n, _)| n.clone()).zip(args.iter().cloned()).collect();
        values.insert(promises.gives.clone(), Value::Unknown(id));
        let exact = Exact {
            calls: calls.clone(),
            log: Default::default(),
        };
        for (path, claim) in &promises.claims {
            let fixed = RUN.with(|r| r.borrow().as_ref().map(|run| (run.fixed.clone(), run.order.clone())));
            let read = eval(&exact, &values, claim);
            // Reading may refine records: put the run back as it was.
            RUN.with(|r| {
                if let (Some(run), Some((f, o))) = (r.borrow_mut().as_mut(), fixed) {
                    run.fixed = f;
                    run.order = o;
                }
            });
            if let Ok(Value::Truth(false)) = read {
                return Some(format!("`{path}` is false"));
            }
        }
    }
    None
}

/// The first claim of a function with no body that the run's opaque
/// calls break, by its path; `Err` with the outcome when reading them
/// needs a split.
fn promises(calls: &Arc<Calls>) -> Result<Option<String>, Exactly> {
    let called: Vec<(u32, Origin)> = RUN.with(|r| {
        let r = r.borrow();
        let run = r.as_ref().expect("a run");
        let t = run.table.lock().unwrap_or_else(|e| e.into_inner());
        run.called.iter().map(|id| (*id, t.made[*id as usize].0.clone())).collect()
    });
    for (id, origin) in called {
        let Origin::Call(f, args) = origin else { continue };
        let Some(callee) = calls.get(&f) else { continue };
        let Some(promises) = &callee.promises else { continue };
        let mut values: BTreeMap<String, Value> =
            callee.takes.iter().map(|(n, _)| n.clone()).zip(args.iter().cloned()).collect();
        values.insert(promises.gives.clone(), Value::Unknown(id));
        let exact = Exact {
            calls: calls.clone(),
            log: Default::default(),
        };
        for (path, claim) in &promises.claims {
            match eval(&exact, &values, claim) {
                Ok(Value::Truth(true)) => {}
                Err(Fault::Op(Undefined::Split(s))) => return Err(Err(Fault::Op(Undefined::Split(s)))),
                // False: this path is not one the claim allows.
                Ok(Value::Truth(false)) => {
                    if std::env::var_os("MITRA_SEARCH_TRACE").is_some() {
                        let shown: Vec<String> = values.iter().map(|(n, v)| format!("{n} = {}", deep(v).map(|v| crate::print::value(&v)).unwrap_or_else(|_| "?".into()))).collect();
                        eprintln!("promise {path} false: {}", shown.join(", "));
                    }
                    return Ok(Some(format!("`{path}` is false")));
                }
                // No value: not allowed either, and said apart, so that a
                // promise that can never be read is seen (no silent
                // diagnostics).
                Err(e) => return Ok(Some(format!("`{path}` has no value ({})", fault_text(&e)))),
                Ok(_) => return Ok(Some(format!("`{path}` is not a truth"))),
            }
        }
    }
    Ok(None)
}

fn fault_text(e: &Fault<Undefined>) -> String {
    match e {
        Fault::Op(Undefined::Unknown(g)) => format!("no value for {g}"),
        Fault::Op(Undefined::Budget(why)) => why.clone(),
        Fault::Stuck => "it cannot be read here".to_string(),
        Fault::Op(u) => format!("{u:?}"),
    }
}

fn origin_text(t: &Table, id: u32) -> String {
    match t.made.get(id as usize).map(|(o, _)| o) {
        Some(Origin::Call(f, args)) => {
            let shown: Vec<String> = args.iter().map(|a| shown_in(t, &BTreeMap::new(), a)).collect();
            format!("`{f}({})`", shown.join(", "))
        }
        Some(Origin::Input(n)) => format!("the input `{n}`"),
        Some(Origin::Part(p, _, k)) => format!("part {k} of {}", origin_text(t, *p)),
        None => format!("?{id}"),
    }
}

/// `v` as the path knows it: each unknown it refined written out, each
/// other `_`.
fn shown_in(t: &Table, fixed: &BTreeMap<u32, Value>, v: &Value) -> String {
    crate::print::value(&filled(t, fixed, v))
}

fn filled(t: &Table, fixed: &BTreeMap<u32, Value>, v: &Value) -> Value {
    match v {
        Value::Unknown(id) => match fixed.get(id) {
            Some(known) => filled(t, fixed, known),
            // Not read on this path: any value does; the first of its
            // type is written (choice S8).
            None => t
                .made
                .get(*id as usize)
                .and_then(|(_, ty)| first_of(t, ty, 0))
                .unwrap_or(Value::Unknown(*id)),
        },
        Value::Record(r, vs) => Value::Record(r.clone(), vs.iter().map(|v| filled(t, fixed, v)).collect()),
        Value::Variant(e, i, vs) => Value::Variant(e.clone(), *i, vs.iter().map(|v| filled(t, fixed, v)).collect()),
        Value::Seq(ty, vs) => Value::Seq(ty.clone(), vs.iter().map(|v| filled(t, fixed, v)).collect()),
        v => v.clone(),
    }
}

impl<T> Explored<T> {
    /// What a path took, said of the opaque calls and free inputs: each
    /// one it read, with what it was on this path, `_` where not read.
    pub fn condition(&self, p: &Path<T>) -> Vec<String> {
        self.condition_of(p, None)
    }
    /// The condition of path `p`, said of the opaque results whose
    /// unknowns are among `read` only (all of them with `None`).
    pub fn condition_of(&self, p: &Path<T>, read: Option<&BTreeSet<u32>>) -> Vec<String> {
        let t = self.table.lock().unwrap_or_else(|e| e.into_inner());
        let root_of = |id: u32| {
            let mut root = id;
            while let Some((Origin::Part(parent, _, _), _)) = t.made.get(root as usize) {
                root = *parent;
            }
            root
        };
        let wanted: Option<BTreeSet<u32>> = read.map(|r| r.iter().map(|i| root_of(*i)).collect());
        let mut said = Vec::new();
        let mut roots: Vec<u32> = Vec::new();
        for (id, _) in &p.cases {
            if wanted.as_ref().is_some_and(|w| !w.contains(&root_of(*id))) {
                continue;
            }
            let mut root = *id;
            while let Some((Origin::Part(parent, _, _), _)) = t.made.get(root as usize) {
                root = *parent;
            }
            if !roots.contains(&root) {
                roots.push(root);
            }
        }
        for root in roots {
            let value = shown_in(&t, &p.fixed, &Value::Unknown(root));
            let what = match &t.made[root as usize].0 {
                Origin::Call(f, args) => {
                    let shown: Vec<String> = args.iter().map(|a| shown_in(&t, &p.fixed, a)).collect();
                    format!("`{f}({})` gives {value}", shown.join(", "))
                }
                Origin::Input(n) => format!("`{n}` is {value}"),
                Origin::Part(..) => value,
            };
            said.push(what);
        }
        said
    }
    /// The opaque calls of a number whose value path `p` took, each by
    /// its function, as written (with its arguments), with its type and
    /// that value.
    pub fn numbers(&self, p: &Path<T>) -> Vec<(u32, String, String, Type, BigInt)> {
        let t = self.table.lock().unwrap_or_else(|e| e.into_inner());
        p.cases
            .iter()
            .filter_map(|(id, v)| {
                let (Origin::Call(f, args), ty) = &t.made[*id as usize] else { return None };
                let n = match v {
                    Value::Int(n) | Value::Fixed(_, n) => n.clone(),
                    _ => return None,
                };
                let shown: Vec<String> = args.iter().map(|a| shown_in(&t, &p.fixed, a)).collect();
                Some((*id, f.clone(), format!("`{f}({})`", shown.join(", ")), ty.clone(), n))
            })
            .collect()
    }
    /// `v` as path `p` knows it.
    pub fn shown(&self, p: &Path<T>, v: &Value) -> String {
        let t = self.table.lock().unwrap_or_else(|e| e.into_inner());
        shown_in(&t, &p.fixed, v)
    }
    /// `v` as path `p` knows it, as a value: unknowns it did not read
    /// left unknown.
    pub fn filled(&self, p: &Path<T>, v: &Value) -> Value {
        let t = self.table.lock().unwrap_or_else(|e| e.into_inner());
        filled(&t, &p.fixed, v)
    }
    /// The opaque calls of a path, each with what it gave there: as the
    /// `given` of an example row would say them.
    pub fn givens(&self, p: &Path<T>) -> Vec<(String, Vec<Value>, Value)> {
        let t = self.table.lock().unwrap_or_else(|e| e.into_inner());
        p.called
            .iter()
            .filter_map(|id| match &t.made[*id as usize].0 {
                Origin::Call(f, args) => Some((
                    f.clone(),
                    args.iter().map(|a| filled(&t, &p.fixed, a)).collect(),
                    filled(&t, &p.fixed, &Value::Unknown(*id)),
                )),
                _ => None,
            })
            .collect()
    }
    /// The value each free input had on path `p`.
    pub fn inputs(&self, p: &Path<T>, free: &Free) -> Vec<(String, Value)> {
        let t = self.table.lock().unwrap_or_else(|e| e.into_inner());
        free.iter()
            .filter_map(|(n, _)| {
                let id = t.made.iter().position(|(o, _)| o == &Origin::Input(n.clone()))?;
                Some((n.clone(), filled(&t, &p.fixed, &Value::Unknown(id as u32))))
            })
            .collect()
    }
}

/// A record of no field: what a call made for what it does gives.
pub fn unit() -> Value {
    Value::Record(
        Record {
            name: String::new(),
            args: vec![],
            fields: vec![],
        },
        vec![],
    )
}

/// Whether an expression reads the calls of the run: a route form or
/// `calls(f)`, anywhere in it.
pub fn reads_trace(e: &crate::expression::Expression<Op>) -> bool {
    match e {
        crate::expression::Expression::Name(_) => false,
        crate::expression::Expression::Apply(op, args) => {
            matches!(op, Op::Calls(_) | Op::Route(_)) || args.iter().any(|a| reads_trace(&a.body))
        }
    }
}
