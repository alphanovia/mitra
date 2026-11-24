//! Explain (`docs/design/explain.md`): does the model explain the system,
//! assuming the evidence, and if not, where? The second question.
//!
//! Explain reads a model `check` accepts and a ledger. It writes what
//! each holds as takings: each leaf of the model is taken true once by
//! the voice `model`, with no weight (count 0): what the model alone says,
//! and every one of them can be blamed. A rule a proof shows is taken by
//! the checker instead. A review takes its proposition, or each tie of
//! the item it names. A measurement takes, at the point it measured,
//! that the function is correct there, and that each of its claims
//! holds there, when the model says what to expect: its body or an
//! example gives a value, or its claims decide. Each of those it links
//! to the model by a rule it writes, which the checker takes: the ties
//! of what the measurement went through, with a claim for a claim's
//! instance, give it.
//!
//! Then judgement applies every rule it can, forward (ponens) and back
//! (tollens), and alls and anys by their points, until nothing changes.
//! Each proposition is explained when some set of takings it follows
//! from holds none of the model's own, contradicted when it follows
//! false from some, and unsupported otherwise. Nothing given is dropped
//! quietly: what explain cannot read is a finding (suit `X`).

use crate::catalogue::{
    Ask, Finding, Hint, X001, X002, X003, X004, X005, X006, X007, X008, X009, X010, X011, X012,
    X013,
};
use crate::evidence::{self, port, Ledger, Measurement};
use crate::expression::{eval, Arg, Expression, Fault};
use crate::judgement::{self, Definition, Judgement, Opinion, Point, Prop, Rests, Taking, Term};
use crate::model::{self, Function, Index, Item, Kind, Model, Path};
use crate::print;
use crate::simulation::{call_at, matches, Calls, Exact, Op, Undefined, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// The voice of what only the model says: each of its leaves, taken
/// true once, with no weight.
pub const MODEL: &str = "model";
/// The voice of what Mitra checked: a rule a proof shows, and each rule
/// explain writes to link a measurement to the model.
pub const CHECK: &str = "machine:mitra-check";

/// How far explain goes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    /// Rounds of judging, at most: each applies every rule once.
    /// Default 64. A finding says when judging stopped there.
    pub rounds: usize,
    /// An all or an any is decided by every one of its points only when
    /// its ranges hold at most this many. Default 1,024.
    pub cases: usize,
    /// A rule is applied only when the sets of takings its premises rest
    /// on, taken together, number at most this many: each way to have
    /// every premise is one more set its conclusion rests on, and
    /// judgement keeps the smallest by comparing each with each. Default
    /// 1,024. A finding names each rule left out.
    pub rests: usize,
    /// How far one computation through calls may go: past it, X009, and
    /// nothing is decided there.
    pub budget: crate::simulation::Budget,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            rounds: 64,
            cases: 1024,
            rests: 1024,
            budget: crate::simulation::Budget::default(),
        }
    }
}

/// What the evidence says of a proposition.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Verdict {
    /// It follows true from some set of takings with none by `model`:
    /// evidence and checks alone.
    Explained,
    /// It follows false from these sets of takings.
    Contradicted { against: Rests },
    /// Only what the model alone says holds it, or nothing does.
    Unsupported,
}

/// The verdict on a proposition, with the path of what it is about.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Verdicted {
    pub at: Path,
    pub prop: Prop,
    pub verdict: Verdict,
}

/// Where a taking comes from.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Source {
    /// A leaf of the model, which only the model says.
    Model,
    /// The measurement of the ledger at this place.
    Measurement(usize),
    /// The review of the ledger at this place.
    Review(usize),
    /// A rule of the model its proof shows.
    Proof,
    /// A rule explain wrote to link a measurement to the model.
    Written,
}

/// What explain gives.
#[derive(Clone, Debug, Default)]
pub struct Explanation {
    /// Each proposition of the model, then each a measurement wrote:
    /// where a function is correct, where a claim holds.
    pub verdicts: Vec<Verdicted>,
    /// All that judging gave, by proposition: what blame reads.
    pub opinions: BTreeMap<Prop, Opinion>,
    /// Where each taking comes from.
    pub sources: BTreeMap<Taking, Source>,
    /// Measurements that say the same, by the same voice, read as one,
    /// their counts added: by the first one's place, all of theirs.
    pub same: BTreeMap<usize, Vec<usize>>,
    /// Each taking a proof by cases read at a point, with the others that
    /// say the same there: dropping it alone leaves the all to them.
    pub alike: BTreeMap<Taking, Vec<Taking>>,
    pub findings: Vec<Finding>,
    pub hints: Vec<Hint>,
    /// The reviews the model needs and no voice gave: of each link, and
    /// of each ensure said only in words.
    pub asks: Vec<Ask>,
}

/// Explain the system: the model, assuming the evidence.
pub fn explain(model: &Model, ledger: &Ledger, settings: &Settings) -> Explanation {
    let index = Index::of(model);
    let library = model::calls(model)
        .unwrap_or_default()
        .budget(settings.budget);
    let refused: BTreeSet<Path> = evidence::check(model, ledger)
        .into_iter()
        .map(|e| e.at)
        .collect();
    let mut given = Given {
        index: &index,
        model,
        library: &library,
        takings: Vec::new(),
        sources: BTreeMap::new(),
        rules: index.definitions.clone(),
        written: Vec::new(),
        findings: Vec::new(),
        observed: BTreeMap::new(),
    };
    // What the model alone says: each leaf, taken true once.
    for (at, prop) in model::leaves(model) {
        let shown = match index.items.get(&at) {
            Some(Item::Rule(rule)) if prop == Prop::Named(at.to_string()) => {
                index.shows(&at, rule) == Some(true)
            }
            _ => false,
        };
        let (by, count) = match shown {
            true => (CHECK, 1),
            false => (MODEL, 0),
        };
        let source = match shown {
            true => Source::Proof,
            false => Source::Model,
        };
        given.take(prop, true, count, by, source);
    }
    for (i, r) in ledger.reviews.iter().enumerate() {
        if refused.contains(&Path::to(&["reviews", &i.to_string()])) {
            continue;
        }
        let props = match (&r.from, index.prop_at(&r.prop)) {
            (Some(from), _) => model::link(&index, &r.prop, from).into_iter().collect(),
            (None, Some(p)) => vec![p],
            (None, None) => model
                .ties
                .iter()
                .filter(|t| t.item == r.prop)
                .map(|t| t.prop())
                .collect(),
        };
        for p in props {
            given.take(p, r.holds, 1, &r.by, Source::Review(i));
        }
    }
    // Measurements that say the same, by the same voice, are one, their
    // counts added: the same run written twice says nothing new apart.
    let mut read: Vec<(usize, Measurement)> = Vec::new();
    let mut same: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    for (i, m) in ledger.measurements.iter().enumerate() {
        if refused.contains(&Path::to(&["measurements", &i.to_string()])) {
            continue;
        }
        match read.iter_mut().find(|(_, r)| alike(r, m)) {
            Some((first, r)) => {
                r.count = r.count.saturating_add(m.count);
                same.entry(*first).or_insert_with(|| vec![*first]).push(i);
            }
            None => read.push((i, m.clone())),
        }
    }
    for (i, m) in &read {
        given.measurement(*i, m);
    }
    let Given {
        takings,
        sources,
        rules,
        written,
        mut findings,
        observed,
        ..
    } = given;
    let hints = observations(model, observed, &mut findings);
    // Judge.
    let names: Vec<Prop> = rules.keys().map(|n| Prop::Named(n.clone())).collect();
    let judgement = Judgement::with(rules).unwrap_or_else(|_| index.judgement.clone());
    let props = model::props(model);
    let mut grouped: BTreeMap<Prop, Vec<Taking>> = BTreeMap::new();
    let mut evidence: BTreeMap<Prop, Vec<Taking>> = BTreeMap::new();
    for t in takings {
        if t.by != MODEL && t.by != CHECK {
            evidence.entry(t.prop.clone()).or_default().push(t.clone());
        }
        grouped.entry(t.prop.clone()).or_default().push(t);
    }
    let mut judge = Judge {
        judgement,
        opinions: BTreeMap::new(),
        changed: Changed::default(),
        stopped: BTreeSet::new(),
        alike: BTreeMap::new(),
    };
    for (p, ts) in grouped {
        if let Some(o) = Opinion::taken(p, ts) {
            judge.add(o);
        }
    }
    let mut rule_props: BTreeSet<Prop> = props.iter().map(|(_, p)| p.clone()).collect();
    rule_props.extend(names);
    rule_props.retain(|p| judge.judgement.premises(p).is_some());
    let rule_props: Vec<Prop> = rule_props.into_iter().collect();
    let quantified: Vec<Prop> = props
        .iter()
        .map(|(_, p)| p.clone())
        .filter(|p| matches!(p, Prop::All { .. } | Prop::Any { .. }))
        .collect::<BTreeSet<Prop>>()
        .into_iter()
        .collect();
    for q in &quantified {
        if let Some(point) = undecided(q, settings) {
            let at = props.iter().find(|(_, p)| p == q).map(|(at, _)| at.clone());
            let (Prop::All { over, .. } | Prop::Any { over, .. }) = q else {
                continue;
            };
            findings.push(Finding {
                code: X009,
                at: at.unwrap_or_else(|| Path::to(&["explain"])),
                message: format!(
                    "`{}`: the condition of the set {} was not read to its end at {} (a budget reached, or a call with no value there): whether that point is in it is not known, so it is given nothing from its points",
                    print::print_prop(q),
                    print::print_set(over),
                    print::values(&point)
                ),
            });
            continue;
        }
        if none_in(q, settings) {
            let at = props.iter().find(|(_, p)| p == q).map(|(at, _)| at.clone());
            let (Prop::All { over, .. } | Prop::Any { over, .. }) = q else {
                continue;
            };
            let (said, value) = match q {
                Prop::All { .. } => ("all", "true"),
                _ => ("any", "false"),
            };
            findings.push(Finding {
                code: X013,
                at: at.unwrap_or_else(|| Path::to(&["explain"])),
                message: format!(
                    "`{}`: no point is in the set {}: the {said} would be {value} with no point to read, so it is given nothing from its points",
                    print::print_prop(q),
                    print::print_set(over)
                ),
            });
            continue;
        }
        if let Some(o) = judge.cases(q, &evidence, settings) {
            judge.add(o);
        }
    }
    let done = judge.run(&rule_props, &quantified, settings);
    for t in &judge.stopped {
        findings.push(Finding {
            code: X006,
            at: Path::to(&["explain"]),
            message: format!(
                "rule `{}`: its premises came to rest on more than {} sets of takings together (`Settings::rests`), and it was left out from then on: what follows from it may not be said",
                print::print_prop(t),
                settings.rests
            ),
        });
    }
    if !done {
        findings.push(Finding {
            code: X006,
            at: Path::to(&["explain"]),
            message: format!(
                "judging stopped after {} rounds (`Settings::rounds`): what follows further is not said",
                settings.rounds
            ),
        });
    }
    // Verdicts.
    let mut verdicts = Vec::new();
    let mut seen = BTreeSet::new();
    for (at, prop) in props.into_iter().chain(written) {
        if !seen.insert(prop.clone()) {
            continue;
        }
        let verdict = verdict(judge.opinion(&prop).as_ref());
        verdicts.push(Verdicted { at, prop, verdict });
    }
    findings.sort();
    findings.dedup();
    let asks = asks(model, ledger, &refused);
    Explanation {
        verdicts,
        opinions: judge.opinions,
        sources,
        same,
        alike: judge.alike,
        findings,
        hints,
        asks,
    }
}

/// The reviews the model needs and the ledger does not give (X012).
/// A link, an ensure from a list, is only the model's word until a
/// voice reviews it; an ensure said only in words, with nothing below
/// it, likewise. A require from an ensure is that ensure, named: it is
/// not asked for. A review that holds or fails
/// answers it: someone looked. A refused one does not.
pub fn asks(model: &Model, ledger: &Ledger, refused: &BTreeSet<Path>) -> Vec<Ask> {
    let index = Index::of(model);
    let reviewed: BTreeSet<(&Path, &Option<Vec<Path>>)> = ledger
        .reviews
        .iter()
        .enumerate()
        .filter(|(i, _)| !refused.contains(&Path::to(&["reviews", &i.to_string()])))
        .map(|(_, r)| (&r.prop, &r.from))
        .collect();
    let mut asks = Vec::new();
    for (at, item) in &index.items {
        // A require from an ensure is that ensure, named: no words of
        // its own, so its link is not asked for (a review may still
        // name it).
        let links = match item {
            Item::Ensure(_) => evidence::links(&index, at),
            _ => vec![],
        };
        for list in &links {
            let from = Some(list.clone());
            if reviewed.contains(&(at, &from)) || model::link(&index, at, list).is_none() {
                continue;
            }
            let names: Vec<String> = list.iter().map(|p| p.to_string()).collect();
            let names = names.join(", ");
            let written = match links.len() {
                1 => "from".to_string(),
                _ => format!("from ({names})"),
            };
            let what = match list.len() {
                1 => format!("that `{names}` gives `{at}`"),
                _ => format!("that ({names}) together give `{at}`"),
            };
            asks.push(Ask {
                code: X012,
                at: at.clone(),
                from,
                message: format!("only the model says {what}: review the link"),
                line: format!("review {at} {written} holds by \"human:<name>\";"),
            });
        }
        if let Item::Ensure(e) = item {
            if e.from.is_empty() && !reviewed.contains(&(at, &None)) {
                asks.push(Ask {
                    code: X012,
                    at: at.clone(),
                    from: None,
                    message: format!(
                        "only words say `{at}`, and nothing below gives it: review it, seeing the test that shows it if one does"
                    ),
                    line: format!("review {at} holds by \"human:<name>\" seeing \"test:<name>\";"),
                });
            }
        }
    }
    asks
}

/// Two measurements that say the same, by the same voice, whatever
/// their counts: the same inputs and outputs, in any order.
fn alike(a: &Measurement, b: &Measurement) -> bool {
    let sorted = |ports: &[(Path, Value)]| {
        let mut ports = ports.to_vec();
        ports.sort();
        ports
    };
    a.seen == b.seen
        && a.by == b.by
        && sorted(&a.inputs) == sorted(&b.inputs)
        && sorted(&a.outputs) == sorted(&b.outputs)
}

/// The verdict an opinion gives.
fn verdict(opinion: Option<&Opinion>) -> Verdict {
    let Some(o) = opinion else {
        return Verdict::Unsupported;
    };
    if !o.rests_against().is_empty() {
        return Verdict::Contradicted {
            against: o.rests_against().clone(),
        };
    }
    let evidenced = |rest: &BTreeSet<Taking>| rest.iter().all(|t| t.by != MODEL);
    match o.rests_for().iter().any(evidenced) {
        true => Verdict::Explained,
        false => Verdict::Unsupported,
    }
}

/// What opaque functions gave where nothing said what they should: by
/// function and point, each measurement and the value it saw.
type Observed = BTreeMap<(Path, Vec<Value>), Vec<(usize, Value)>>;
/// A point, and each value seen there.
type Seen = (Vec<Value>, Vec<Value>);

/// What explain reads from the model and the ledger, as it goes.
struct Given<'a, 'm> {
    index: &'a Index<'m>,
    model: &'m Model,
    library: &'a Calls,
    takings: Vec<Taking>,
    sources: BTreeMap<Taking, Source>,
    /// Every rule judgement will know: the model's, and those written to
    /// link a measurement to it.
    rules: BTreeMap<String, Definition>,
    /// The propositions measurements wrote, each with its path.
    written: Vec<(Path, Prop)>,
    findings: Vec<Finding>,
    /// What opaque functions gave where nothing said what they should:
    /// by function and point, each measurement and value.
    observed: Observed,
}

impl Given<'_, '_> {
    fn take(&mut self, prop: Prop, value: bool, count: u64, by: &str, source: Source) {
        let taking = Taking {
            prop,
            value,
            count,
            by: by.to_string(),
        };
        // Two measurements that say one thing, by one voice and count, are
        // one taking: judgement cannot tell them apart.
        if let (Some(Source::Measurement(j)), Source::Measurement(i)) =
            (self.sources.get(&taking), source)
        {
            if *j != i {
                let message = format!(
                    "it says `{}` is {}, as measurements::{j} does, by the same voice and count: read once, and blamed with measurements::{j}",
                    print::print_prop(&taking.prop),
                    taking.value
                );
                let at = Path::to(&["measurements", &i.to_string()]);
                self.finding(X008, &at, message);
            }
        }
        self.sources.entry(taking.clone()).or_insert(source);
        self.takings.push(taking);
    }

    /// A rule `pre |- post`, by `name`, written once and taken by the
    /// checker.
    fn write(&mut self, name: String, pre: Vec<Prop>, post: Prop) {
        if self.rules.contains_key(&name) {
            return;
        }
        self.rules.insert(
            name.clone(),
            Definition {
                pre,
                post: vec![post],
            },
        );
        self.take(Prop::Named(name), true, 1, CHECK, Source::Written);
    }

    fn finding(&mut self, code: crate::catalogue::Code, at: &Path, message: String) {
        self.findings.push(Finding {
            code,
            at: at.clone(),
            message,
        });
    }

    /// The ties of these functions, each by its own tie or its part's.
    fn ties(&self, functions: &BTreeSet<String>) -> Vec<Prop> {
        let parts: BTreeSet<&str> = functions
            .iter()
            .filter_map(|f| f.split("::").next())
            .collect();
        self.model
            .ties
            .iter()
            .filter(|t| {
                let item = t.item.to_string();
                functions.contains(&item)
                    || (t.item.names.len() == 1 && parts.contains(item.as_str()))
            })
            .map(|t| t.prop())
            .collect::<BTreeSet<Prop>>()
            .into_iter()
            .collect()
    }

    /// The functions these expressions call, and those they call in
    /// turn, by path.
    fn reach<'e>(&self, from: impl IntoIterator<Item = &'e Expression<Op>>) -> BTreeSet<String> {
        let mut found = BTreeSet::new();
        let mut todo: Vec<&Expression<Op>> = from.into_iter().collect();
        while let Some(e) = todo.pop() {
            if let Expression::Apply(op, args) = e {
                if let Op::Call(f) = op {
                    if found.insert(f.clone()) {
                        if let Some(body) =
                            self.index.functions.get(f).and_then(|g| g.body.as_ref())
                        {
                            todo.push(body);
                        }
                    }
                }
                todo.extend(args.iter().map(|a| &a.body));
            }
        }
        found
    }

    /// One measurement: at each output, what the model expects there,
    /// and what the measurement then says.
    /// A measurement outside a range of `f` at `x`: a finding, and, when
    /// the system was seen to give it, a taking that `f` is not correct
    /// there, by its ties: a drift between the model and the code, unless
    /// the measurement was taken wrong.
    #[allow(clippy::too_many_arguments)]
    fn drift(
        &mut self,
        i: usize,
        m: &Measurement,
        at: &Path,
        f: &Path,
        x: &[Value],
        point: &str,
        drift: &str,
    ) {
        let message = match m.seen {
            true => format!("at {point}, {drift}: the system and the model disagree there, a drift between them or a measurement taken wrong"),
            false => format!("the test expected, at {point}, {drift}: the model says nothing there"),
        };
        self.finding(X011, at, message);
        if !m.seen {
            return;
        }
        let path = f.to_string();
        let prop = Prop::Correct {
            function: path.clone(),
            at: Term::Point(x.to_vec()),
        };
        self.take(prop.clone(), false, m.count, &m.by, Source::Measurement(i));
        let ties = self.ties(&BTreeSet::from([path.clone()]));
        self.write(
            format!("{path} at {point}, by its ties"),
            ties,
            prop.clone(),
        );
        self.written.push((f.clone(), prop));
    }

    fn measurement(&mut self, i: usize, m: &Measurement) {
        let at = Path::to(&["measurements", &i.to_string()]);
        let index = self.index;
        let given: BTreeMap<&Path, &Value> = m.inputs.iter().map(|(p, v)| (p, v)).collect();
        let args_of = |f: &Path, function: &Function| -> Option<Vec<Value>> {
            let args = function
                .takes
                .iter()
                .map(|(n, _)| given.get(&f.then(n)).map(|v| (*v).clone()));
            args.collect()
        };
        // What a function with no body gave, at its point: read before
        // its examples.
        let mut library = self.library.clone();
        let mut rows: BTreeMap<String, &Path> = BTreeMap::new();
        for (p, v) in &m.inputs {
            let Some(port) = port(index, p) else { continue };
            if !port.gives || port.function.body.is_some() {
                continue;
            }
            let path = port.at.to_string();
            if let Some(args) = args_of(&port.at, port.function) {
                // A value outside its range: the system gave what the
                // model says it never gives.
                if let Some(drift) = outside(port.function, &args, v) {
                    let point = print::values(&args);
                    self.drift(i, m, &at, &port.at, &args, &point, &drift);
                    continue;
                }
                // Refused only for a value of another type, which V003
                // refuses before.
                library = library
                    .given(&path, args, v.clone())
                    .unwrap_or_else(|same| same);
                rows.insert(path, p);
            }
        }
        let exact = Exact {
            calls: Arc::new(library),
        };
        let mut read: BTreeSet<&Path> = BTreeSet::new();
        for (o, y) in &m.outputs {
            let Some(port) = port(index, o) else { continue };
            let f = port.at.clone();
            let path = f.to_string();
            let function = port.function;
            let Some(x) = args_of(&f, function) else {
                continue;
            };
            let point = print::values(&x);
            // Outside a range: the system and the model's range disagree.
            if let Some(drift) = outside(function, &x, y) {
                for (p, _) in &m.inputs {
                    if p.names.len() == 3 && format!("{}::{}", p.names[0], p.names[1]) == path {
                        read.insert(p);
                    }
                }
                self.drift(i, m, &at, &f, &x, &point, &drift);
                continue;
            }
            // What it reads: its arguments, and what the functions its
            // body and claims call gave.
            let claims = function.claims.iter().map(|c| &c.holds);
            let reached = self.reach(function.body.iter().chain(claims));
            for (p, _) in &m.inputs {
                let [part, g, _] = p.names.as_slice() else {
                    continue;
                };
                let g = format!("{part}::{g}");
                // Its own arguments, and what a function it reaches gave,
                // with that function's arguments, read as its row.
                if g == path || (reached.contains(&g) && rows.contains_key(&g)) {
                    read.insert(p);
                }
            }
            // What the model expects: the value it computes.
            let (call, args) = call_at(&path, &x);
            let expected = eval(&exact, &args, &call);
            let shown = |v: &Value| format!("`{o} = {}`", print::value(v));
            let mut correct: Option<bool> = None;
            let mut opaque = false;
            match &expected {
                Ok(v) if matches(v, y).is_err() => {
                    let g = matches(v, y).unwrap_err();
                    let message = format!(
                        "at {point}, {} is known in part: only `{g}`, not known, could tell it from what the model gives, {}",
                        shown(y),
                        print::value(v)
                    );
                    self.finding(X010, &at, message);
                }
                Ok(v) if m.seen => correct = Some(matches(v, y) == Ok(true)),
                Ok(v) if matches(v, y) == Ok(true) => correct = Some(false),
                Ok(v) => {
                    let message = format!(
                        "the test expected {} at {point}, the model gives {}: the system did not give the first, and whether it gave the second is not known",
                        shown(y),
                        print::value(v)
                    );
                    self.finding(X002, &at, message);
                }
                Err(Fault::Op(Undefined::Unknown(g))) if g == &path => opaque = true,
                Err(Fault::Op(Undefined::Unknown(g)))
                    if !self.index.functions.contains_key(g.as_str()) =>
                {
                    let message = format!(
                        "at {point}, `{path}` reads `{g}`, which this measurement does not know"
                    );
                    self.finding(X010, &at, message);
                }
                Err(Fault::Op(Undefined::Budget(why))) => {
                    self.finding(X009, &at, format!("at {point}, `{path}`: {why}"));
                }
                Err(Fault::Op(Undefined::Unknown(g))) => {
                    let message = format!(
                        "at {point}, `{path}` needs what `{g}` gave, which nothing gives: measure it as an input"
                    );
                    self.finding(X004, &at, message);
                }
                // The model gives no value there.
                Err(_) if m.seen => correct = Some(false),
                Err(_) => {
                    let message = format!(
                        "the test expected {} at {point}, and the model gives no value there",
                        shown(y)
                    );
                    self.finding(X002, &at, message);
                }
            }
            // Each claim, read with what the system gave.
            let mut held = Vec::new();
            let mut undecided = false;
            if m.seen {
                let mut values: BTreeMap<String, Value> = function
                    .takes
                    .iter()
                    .map(|(n, _)| n.clone())
                    .zip(x.iter().cloned())
                    .collect();
                values.insert(function.gives.0.clone(), y.clone());
                for c in &function.claims {
                    match eval(&exact, &values, &c.holds) {
                        Ok(Value::Truth(b)) => held.push((c, b)),
                        Err(Fault::Op(Undefined::Unknown(g)))
                            if !self.index.functions.contains_key(g.as_str()) =>
                        {
                            undecided = true;
                            let message = format!(
                                "claim `{}` at {point}: it reads `{g}`, which this measurement does not know",
                                c.name
                            );
                            self.finding(X010, &at, message);
                        }
                        Err(Fault::Op(Undefined::Unknown(g))) => {
                            undecided = true;
                            let message = format!(
                                "claim `{}` at {point}: it needs what `{g}` gave, which nothing gives",
                                c.name
                            );
                            self.finding(X003, &at, message);
                        }
                        Err(Fault::Op(Undefined::Budget(why))) => {
                            undecided = true;
                            let message = format!("claim `{}` at {point}: {why}", c.name);
                            self.finding(X009, &at, message);
                        }
                        _ => {
                            undecided = true;
                            let message =
                                format!("claim `{}` at {point}: it gives no truth there", c.name);
                            self.finding(X003, &at, message);
                        }
                    }
                }
            }
            // With no value to compare, the claims decide: false when one
            // is, true when all are.
            if opaque && !held.is_empty() {
                if held.iter().any(|(_, b)| !b) {
                    correct = Some(false);
                } else if !undecided {
                    correct = Some(true);
                }
            }
            if opaque && !function.claims.is_empty() && !m.seen {
                let message = format!(
                    "the test expected {} at {point}, and the system gave something else: nothing in the model says what `{path}` gives there, and its claims need what it gave",
                    shown(y)
                );
                self.finding(X002, &at, message);
            }
            if opaque && function.claims.is_empty() {
                match m.seen {
                    true => self
                        .observed
                        .entry((f.clone(), x.clone()))
                        .or_default()
                        .push((i, y.clone())),
                    false => {
                        let message = format!(
                            "the test expected {} at {point}, and nothing in the model says what `{path}` gives there",
                            shown(y)
                        );
                        self.finding(X002, &at, message);
                    }
                }
            }
            // Through what the model's value came: the body and what it
            // calls, or, with no value, the claims that decided.
            let mut through = match opaque {
                true => self.reach(function.claims.iter().map(|c| &c.holds)),
                false => self.reach(function.body.iter()),
            };
            through.insert(path.clone());
            let ties = self.ties(&through);
            if let Some(value) = correct {
                let prop = Prop::Correct {
                    function: path.clone(),
                    at: Term::Point(x.clone()),
                };
                self.take(prop.clone(), value, m.count, &m.by, Source::Measurement(i));
                self.write(
                    format!("{path} at {point}, by its ties"),
                    ties.clone(),
                    prop.clone(),
                );
                self.written.push((f.clone(), prop));
            }
            for (c, value) in held {
                let claim = f.then(&c.name);
                let instance = Prop::Named(format!("{claim} at {point}"));
                self.take(
                    instance.clone(),
                    value,
                    m.count,
                    &m.by,
                    Source::Measurement(i),
                );
                let mut through = self.reach([&c.holds]);
                through.insert(path.clone());
                let pre = [vec![Prop::Named(claim.to_string())], self.ties(&through)].concat();
                self.write(
                    format!("{claim} at {point}, by its ties"),
                    pre,
                    instance.clone(),
                );
                self.written.push((claim, instance));
            }
        }
        for (p, _) in &m.inputs {
            if !read.contains(p) {
                let message =
                    format!("`{p}` is given, and nothing this measurement measures reads it");
                self.finding(X005, &at, message);
            }
        }
    }
}

/// What opaque functions gave where nothing said what they should: each
/// a finding, and a hint to copy them as examples when a subject's
/// function gave one value at each point.
fn observations(model: &Model, observed: Observed, findings: &mut Vec<Finding>) -> Vec<Hint> {
    let mut by_function: BTreeMap<Path, Vec<Seen>> = BTreeMap::new();
    for ((f, x), seen) in observed {
        for (i, y) in &seen {
            findings.push(Finding {
                code: X001,
                at: Path::to(&["measurements", &i.to_string()]),
                message: format!(
                    "`{f}` gave {} at {}: nothing in the model says what it should give there, and nothing measured reads it",
                    print::value(y),
                    print::values(&x)
                ),
            });
        }
        let values = seen.into_iter().map(|(_, y)| y).collect();
        by_function.entry(f).or_default().push((x, values));
    }
    let mut hints = Vec::new();
    for (f, points) in by_function {
        let subject = model
            .parts
            .iter()
            .any(|p| p.name == f.names[0] && p.kind == Kind::Subject);
        let steady = points
            .iter()
            .all(|(_, ys)| ys.windows(2).all(|w| w[0] == w[1]));
        if subject && steady {
            let rows: Vec<String> = points
                .iter()
                .map(|(x, ys)| print::print_row(x, &ys[0]))
                .collect();
            hints.push(Hint {
                code: X007,
                at: f.clone(),
                message: format!(
                    "`{f}` gave one value at each point measured: copy them as examples? `examples {{ {} }}`",
                    rows.join(", ")
                ),
            });
        }
    }
    hints
}

/// Judgement at work: an opinion of each proposition, joined as rules
/// give more. A negation is kept with what it negates: an opinion of
/// `!p` is one of `p`, its sides swapped, so that `p` and `!p` meet.
/// The first point of the set of an all or an any, listed as `cases`
/// lists it, whose condition was not read to its end.
fn undecided(q: &Prop, settings: &Settings) -> Option<Vec<Value>> {
    let (Prop::All { over, .. } | Prop::Any { over, .. }) = q else {
        return None;
    };
    let mut size = num_bigint::BigInt::from(1);
    for (_, lo, hi) in &over.ranges {
        let (Value::Int(lo), Value::Int(hi)) = (lo, hi) else {
            return None;
        };
        size *= (hi - lo).max(num_bigint::BigInt::from(0));
    }
    if size > num_bigint::BigInt::from(settings.cases) {
        return None;
    }
    over.undecided()
}

/// An all or an any whose set is listed, few enough points in its
/// ranges, and no point is in it: its condition false at each (X013).
fn none_in(q: &Prop, settings: &Settings) -> bool {
    let (Prop::All { over, .. } | Prop::Any { over, .. }) = q else {
        return false;
    };
    let mut size = num_bigint::BigInt::from(1);
    for (_, lo, hi) in &over.ranges {
        let (Value::Int(lo), Value::Int(hi)) = (lo, hi) else {
            return false;
        };
        size *= (hi - lo).max(num_bigint::BigInt::from(0));
    }
    size <= num_bigint::BigInt::from(settings.cases)
        && over
            .points()
            .is_some_and(|mut points| points.next().is_none())
}

struct Judge {
    judgement: Judgement,
    opinions: BTreeMap<Prop, Opinion>,
    /// What follows true from more, or false from more, since the rules
    /// last read it.
    changed: Changed,
    /// The rules left out: their premises rest on too many sets.
    stopped: BTreeSet<Prop>,
    /// Each taking a proof by cases read at a point, with the others
    /// that say the same there.
    alike: BTreeMap<Taking, Vec<Taking>>,
}

/// What lies outside a range of `f`, called with `x` and giving `y`:
/// each such value with its range, or `None`.
fn outside(f: &Function, x: &[Value], y: &Value) -> Option<String> {
    let mut said = Vec::new();
    for ((n, _), v) in f.takes.iter().zip(x) {
        if let Some(range) = f.ranges.takes.get(n).filter(|r| !r.holds(v)) {
            said.push(format!(
                "`{n}` = {} lies outside its range {range}",
                print::value(v)
            ));
        }
    }
    if let Some((q, range)) = f.ranges.gives.as_ref().filter(|(_, r)| !r.holds(y)) {
        said.push(format!(
            "`{q}` = {} lies outside its range {range}",
            print::value(y)
        ));
    }
    match said.is_empty() {
        true => None,
        false => Some(said.join(", and ")),
    }
}

/// What `p` is kept under: what it negates, however many times.
fn kept(p: &Prop) -> &Prop {
    match p {
        Prop::Not(x) => kept(x),
        _ => p,
    }
}

/// Whether `p` negates what it is kept under: an odd number of `!`.
fn negated(p: &Prop) -> bool {
    match p {
        Prop::Not(x) => !negated(x),
        _ => false,
    }
}

/// What changed, by side: the propositions that follow true from more
/// sets, and those that follow false from more.
#[derive(Default)]
struct Changed {
    for_: BTreeSet<Prop>,
    against: BTreeSet<Prop>,
}

impl Changed {
    fn is_empty(&self) -> bool {
        self.for_.is_empty() && self.against.is_empty()
    }
    /// `p` follows true from more: of `!p`, `p` false from more.
    fn true_of(&self, p: &Prop) -> bool {
        match negated(p) {
            false => self.for_.contains(kept(p)),
            true => self.against.contains(kept(p)),
        }
    }
    fn false_of(&self, p: &Prop) -> bool {
        match negated(p) {
            false => self.against.contains(kept(p)),
            true => self.for_.contains(kept(p)),
        }
    }
    fn either(&self, p: &Prop) -> bool {
        self.for_.contains(kept(p)) || self.against.contains(kept(p))
    }
}

impl Judge {
    /// What `op` gives on these values, `None` when it does not apply.
    fn apply(&self, op: judgement::Op, args: Vec<judgement::Value>) -> Option<Opinion> {
        let names: Vec<String> = (0..args.len()).map(|i| format!("a{i}")).collect();
        let e = Expression::Apply(
            op,
            names
                .iter()
                .map(|n| Arg {
                    binds: vec![],
                    body: Expression::Name(n.clone()),
                })
                .collect(),
        );
        let values: BTreeMap<String, judgement::Value> = names.into_iter().zip(args).collect();
        match eval(&self.judgement, &values, &e) {
            Ok(judgement::Value::Opinion(o)) => Some(o),
            _ => None,
        }
    }

    /// `!o`, by judgement's `not`.
    fn not(&self, o: Opinion) -> Option<Opinion> {
        self.apply(judgement::Op::Not, vec![judgement::Value::Opinion(o)])
    }

    /// What is known of `p`: of `!p`, what is known of `p`, its sides
    /// swapped.
    fn opinion(&self, p: &Prop) -> Option<Opinion> {
        match p {
            Prop::Not(x) => self.not(self.opinion(x)?),
            _ => self.opinions.get(p).cloned(),
        }
    }

    /// Join `o` into what is known of its proposition: whether it adds.
    fn add(&mut self, mut o: Opinion) -> bool {
        while let Prop::Not(_) = o.about() {
            match self.not(o.clone()) {
                Some(n) => o = n,
                None => return false,
            }
        }
        let about = o.about().clone();
        let (joined, more_for, more_against) = match self.opinions.get(&about) {
            None => {
                let sides = (!o.rests_for().is_empty(), !o.rests_against().is_empty());
                (o, sides.0, sides.1)
            }
            Some(old) => {
                let both = vec![
                    judgement::Value::Opinion(old.clone()),
                    judgement::Value::Opinion(o),
                ];
                match self.apply(judgement::Op::Join, both) {
                    Some(j) if &j != old => {
                        let more_for = j.rests_for() != old.rests_for();
                        let more_against = j.rests_against() != old.rests_against();
                        (j, more_for, more_against)
                    }
                    _ => return false,
                }
            }
        };
        self.opinions.insert(about.clone(), joined);
        if more_for {
            self.changed.for_.insert(about.clone());
        }
        if more_against {
            self.changed.against.insert(about);
        }
        true
    }

    /// The opinion of `p`, when it follows true from something.
    fn told_true(&self, p: &Prop) -> Option<judgement::Value> {
        let o = self.opinion(p).filter(|o| !o.rests_for().is_empty())?;
        Some(judgement::Value::Opinion(o))
    }

    /// How many sets these opinions rest on together, each on one side.
    fn ways(sides: impl IntoIterator<Item = usize>) -> usize {
        sides.into_iter().fold(1usize, usize::saturating_mul)
    }

    /// Every rule, forward and back, and every all and any by the
    /// points opinions are about, until nothing changes: whether it got
    /// there within the rounds. A rule is read again only when the side
    /// it reads of something changed: forward, its premises true; back,
    /// a post false, or the other premises true. What a round gives is
    /// joined at its end, so every rule of a round reads what the round
    /// began with: one link of a chain a round.
    fn run(&mut self, rules: &[Prop], quantified: &[Prop], settings: &Settings) -> bool {
        for round in 0..settings.rounds {
            let changed = std::mem::take(&mut self.changed);
            if changed.is_empty() && round > 0 {
                return true;
            }
            let first = round == 0;
            let mut gave: Vec<Opinion> = Vec::new();
            for t in rules {
                let (Some(premises), Some(posts)) = (
                    self.judgement.premises(t),
                    self.judgement.posts(t).map(<[Prop]>::to_vec),
                ) else {
                    continue;
                };
                let premises_more = first || premises.iter().any(|p| changed.true_of(p));
                if !premises_more && !posts.iter().any(|q| changed.false_of(q)) {
                    continue;
                }
                // Forward: from every premise, each post.
                let all: Option<Vec<_>> = match premises_more {
                    true => premises.iter().map(|p| self.told_true(p)).collect(),
                    false => None,
                };
                if let Some(all) = all {
                    let ways = Self::ways(all.iter().map(|v| match v {
                        judgement::Value::Opinion(o) => o.rests_for().len(),
                        _ => 1,
                    }));
                    if ways > settings.rests {
                        self.stopped.insert(t.clone());
                    } else {
                        for post in 0..posts.len() {
                            let op = judgement::Op::Ponens { t: t.clone(), post };
                            gave.extend(self.apply(op, all.clone()));
                        }
                    }
                }
                // Back: from a post false and every premise but one, that
                // one is false.
                for (post, q) in posts.iter().enumerate() {
                    if !premises_more && !changed.false_of(q) {
                        continue;
                    }
                    let Some(false_q) = self.opinion(q).filter(|o| !o.rests_against().is_empty())
                    else {
                        continue;
                    };
                    for blame in 0..premises.len() {
                        let others: Option<Vec<_>> = premises
                            .iter()
                            .enumerate()
                            .filter(|(k, _)| *k != blame)
                            .map(|(_, p)| self.told_true(p))
                            .collect();
                        let Some(others) = others else { continue };
                        let ways =
                            Self::ways(std::iter::once(false_q.rests_against().len()).chain(
                                others.iter().map(|v| match v {
                                    judgement::Value::Opinion(o) => o.rests_for().len(),
                                    _ => 1,
                                }),
                            ));
                        if ways > settings.rests {
                            self.stopped.insert(t.clone());
                            continue;
                        }
                        let args =
                            [vec![judgement::Value::Opinion(false_q.clone())], others].concat();
                        let op = judgement::Op::Tollens {
                            t: t.clone(),
                            post,
                            blame,
                        };
                        gave.extend(self.apply(op, args));
                    }
                }
            }
            for o in gave {
                self.add(o);
            }
            for q in quantified {
                self.points(q, round == 0, &changed);
            }
        }
        self.changed.is_empty()
    }

    /// An all or an any at the points opinions are about: down to each
    /// (an instance), and up from each (a witness: an all false, an any
    /// true). Only where something changed, after the first round.
    fn points(&mut self, q: &Prop, first: bool, changed: &Changed) {
        let mut points: BTreeSet<Vec<Value>> = BTreeSet::new();
        for p in self.opinions.keys() {
            if let Prop::Correct {
                at: Term::Point(x), ..
            } = p
            {
                points.insert(x.clone());
            }
        }
        let q_changed = first || changed.either(q);
        let fresh_points: BTreeSet<&Vec<Value>> = changed
            .for_
            .iter()
            .chain(&changed.against)
            .filter_map(|p| match p {
                Prop::Correct {
                    at: Term::Point(y), ..
                } => Some(y),
                _ => None,
            })
            .collect();
        for x in points {
            let Some(body) = q.at(&x) else { continue };
            let fresh = fresh_points.contains(&x);
            let point = || judgement::Value::Point(Point::of(x.clone()));
            if q_changed || fresh {
                if let Some(of_q) = self.opinions.get(q).cloned() {
                    let op = judgement::Op::Instance { q: q.clone() };
                    if let Some(o) = self.apply(op, vec![judgement::Value::Opinion(of_q), point()])
                    {
                        self.add(o);
                    }
                }
            }
            if first || fresh || changed.either(&body) {
                if let Some(of_body) = self.opinion(&body) {
                    let op = judgement::Op::Witness { q: q.clone() };
                    if let Some(o) =
                        self.apply(op, vec![judgement::Value::Opinion(of_body), point()])
                    {
                        self.add(o);
                    }
                }
            }
        }
    }

    /// An all true, or an any false, from every one of its points, when
    /// its ranges hold few enough: each point by the first taking of the
    /// ledger that says so there. One taking a point: the ways to have
    /// every point grow as the product of the ways at each, so the
    /// others, and what the model alone says, are left to each point;
    /// the others are kept in `alike`, for blame.
    fn cases(
        &mut self,
        q: &Prop,
        evidence: &BTreeMap<Prop, Vec<Taking>>,
        settings: &Settings,
    ) -> Option<Opinion> {
        let (Prop::All { over, .. } | Prop::Any { over, .. }) = q else {
            return None;
        };
        let all = matches!(q, Prop::All { .. });
        // The ranges are read whole, the condition after: a set with few
        // points in large ranges is not listed.
        let mut size = num_bigint::BigInt::from(1);
        for (_, lo, hi) in &over.ranges {
            let (Value::Int(lo), Value::Int(hi)) = (lo, hi) else {
                return None;
            };
            size *= (hi - lo).max(num_bigint::BigInt::from(0));
        }
        if size > num_bigint::BigInt::from(settings.cases) {
            return None;
        }
        let listed: Vec<Vec<Value>> = over.points()?.collect::<Option<_>>()?;
        let mut alike = Vec::new();
        let mut cases = Vec::with_capacity(listed.len());
        for x in &listed {
            let body = q.at(x)?;
            // `!p` at the point: `p` taken the other way.
            let (said, value) = match &body {
                Prop::Not(p) => ((**p).clone(), !all),
                _ => (body.clone(), all),
            };
            let mut these = evidence.get(&said)?.iter().filter(|t| t.value == value);
            let first = these.next()?;
            let others: Vec<Taking> = these.cloned().collect();
            if !others.is_empty() {
                alike.push((first.clone(), others));
            }
            let mut o = Opinion::taken(said, vec![first.clone()])?;
            if &body != o.about() {
                o = self.not(o)?;
            }
            if &body != o.about() {
                return None;
            }
            cases.push(judgement::Value::Opinion(o));
        }
        let o = self.apply(judgement::Op::Cases { q: q.clone() }, cases)?;
        self.alike.extend(alike);
        Some(o)
    }
}
