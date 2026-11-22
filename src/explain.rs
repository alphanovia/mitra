//! Explain (`docs/explain.md`): does the model explain the system,
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

use crate::catalogue::{Finding, Hint, X001, X002, X003, X004, X005, X006, X007};
use crate::evidence::{self, port, Ledger, Measurement};
use crate::expression::{eval, Arg, Expression, Fault};
use crate::judgement::{self, Definition, Judgement, Opinion, Point, Prop, Rests, Taking, Term};
use crate::model::{self, Function, Index, Item, Kind, Model, Path};
use crate::print;
use crate::simulation::{Callee, Calls, Exact, Op, Undefined, Value};
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
    /// its set has at most this many. Default 1,024.
    pub cases: usize,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            rounds: 64,
            cases: 1024,
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

/// What explain gives.
#[derive(Clone, Debug, Default)]
pub struct Explanation {
    /// Each proposition of the model, then each a measurement wrote:
    /// where a function is correct, where a claim holds.
    pub verdicts: Vec<Verdicted>,
    /// All that judging gave, by proposition: what blame reads.
    pub opinions: BTreeMap<Prop, Opinion>,
    pub findings: Vec<Finding>,
    pub hints: Vec<Hint>,
}

/// Explain the system: the model, assuming the evidence.
pub fn explain(model: &Model, ledger: &Ledger, settings: &Settings) -> Explanation {
    let index = Index::of(model);
    let library = model::calls(model).unwrap_or_default();
    let refused: BTreeSet<Path> = evidence::check(model, ledger)
        .into_iter()
        .map(|e| e.at)
        .collect();
    let mut given = Given {
        index: &index,
        model,
        library: &library,
        takings: Vec::new(),
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
        given.take(prop, true, count, by);
    }
    for (i, r) in ledger.reviews.iter().enumerate() {
        if refused.contains(&Path::to(&["reviews", &i.to_string()])) {
            continue;
        }
        let props = match index.prop_at(&r.prop) {
            Some(p) => vec![p],
            None => model
                .ties
                .iter()
                .filter(|t| t.item == r.prop)
                .map(|t| t.prop())
                .collect(),
        };
        for p in props {
            given.take(p, r.holds, 1, &r.by);
        }
    }
    for (i, m) in ledger.measurements.iter().enumerate() {
        if !refused.contains(&Path::to(&["measurements", &i.to_string()])) {
            given.measurement(i, m);
        }
    }
    let Given {
        takings,
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
    let mut opinions = BTreeMap::new();
    for (p, ts) in grouped {
        if let Some(o) = Opinion::taken(p.clone(), ts) {
            opinions.insert(p, o);
        }
    }
    let mut rule_props: Vec<Prop> = props.iter().map(|(_, p)| p.clone()).collect();
    rule_props.extend(names);
    rule_props.retain(|p| judgement.premises(p).is_some());
    rule_props.dedup();
    let quantified: Vec<Prop> = props
        .iter()
        .map(|(_, p)| p.clone())
        .filter(|p| matches!(p, Prop::All { .. } | Prop::Any { .. }))
        .collect();
    let mut judge = Judge {
        judgement,
        opinions,
    };
    for q in &quantified {
        if let Some(o) = judge.cases(q, &evidence, settings) {
            judge.add(o);
        }
    }
    let done = judge.run(&rule_props, &quantified, settings);
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
        let verdict = verdict(judge.opinions.get(&prop));
        verdicts.push(Verdicted { at, prop, verdict });
    }
    findings.sort();
    Explanation {
        verdicts,
        opinions: judge.opinions,
        findings,
        hints,
    }
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
    fn take(&mut self, prop: Prop, value: bool, count: u64, by: &str) {
        self.takings.push(Taking {
            prop,
            value,
            count,
            by: by.to_string(),
        });
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
        self.take(Prop::Named(name), true, 1, CHECK);
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
        // What a function with no body gave, at its point: a row read
        // before its examples.
        let mut callees: BTreeMap<String, Callee> = self
            .library
            .iter()
            .map(|(n, c)| (n.clone(), c.clone()))
            .collect();
        let mut rows: BTreeMap<String, &Path> = BTreeMap::new();
        for (p, v) in &m.inputs {
            let Some(port) = port(index, p) else { continue };
            if !port.gives || port.function.body.is_some() {
                continue;
            }
            let path = port.at.to_string();
            if let (Some(args), Some(callee)) =
                (args_of(&port.at, port.function), callees.get_mut(&path))
            {
                callee.rows.insert(0, (args, v.clone()));
                rows.insert(path, p);
            }
        }
        let library = Calls::with(callees).unwrap_or_else(|_| self.library.clone());
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
            // What it reads: its arguments, and what the functions its
            // body and claims call gave.
            let claims = function.claims.iter().map(|c| &c.holds);
            let reached = self.reach(function.body.iter().chain(claims));
            for (p, _) in &m.inputs {
                let [part, g, _] = p.names.as_slice() else {
                    continue;
                };
                let g = format!("{part}::{g}");
                if g == path || (reached.contains(&g) && rows.get(&g) == Some(&p)) {
                    read.insert(p);
                }
            }
            // What the model expects: the value it computes.
            let call = Expression::Apply(
                Op::Call(path.clone()),
                x.iter().map(|v| plain(literal(v))).collect(),
            );
            let expected = eval(&exact, &BTreeMap::new(), &call);
            let shown = |v: &Value| format!("`{o} = {}`", print::value(v));
            let mut correct: Option<bool> = None;
            let mut opaque = false;
            match &expected {
                Ok(v) if m.seen => correct = Some(v == y),
                Ok(v) if v == y => correct = Some(false),
                Ok(v) => {
                    let message = format!(
                        "the test expected {} at {point}, the model gives {}: the system did not give the first, and whether it gave the second is not known",
                        shown(y),
                        print::value(v)
                    );
                    self.finding(X002, &at, message);
                }
                Err(Fault::Op(Undefined::Unknown(g))) if g == &path => opaque = true,
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
                        Err(Fault::Op(Undefined::Unknown(g))) => {
                            undecided = true;
                            let message = format!(
                                "claim `{}` at {point}: it needs what `{g}` gave, which nothing gives",
                                c.name
                            );
                            self.finding(X003, &at, message);
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
            let mut through = reached.clone();
            through.insert(path.clone());
            let ties = self.ties(&through);
            if let Some(value) = correct {
                let prop = Prop::Correct {
                    function: path.clone(),
                    at: Term::Point(x.clone()),
                };
                self.take(prop.clone(), value, m.count, &m.by);
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
                self.take(instance.clone(), value, m.count, &m.by);
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
fn observations(
    model: &Model,
    observed: Observed,
    findings: &mut Vec<Finding>,
) -> Vec<Hint> {
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
/// give more.
struct Judge {
    judgement: Judgement,
    opinions: BTreeMap<Prop, Opinion>,
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

    /// Join `o` into what is known of its proposition: whether it adds.
    fn add(&mut self, o: Opinion) -> bool {
        let about = o.about().clone();
        let joined = match self.opinions.get(&about) {
            None => o,
            Some(old) => {
                let both = vec![
                    judgement::Value::Opinion(old.clone()),
                    judgement::Value::Opinion(o),
                ];
                match self.apply(judgement::Op::Join, both) {
                    Some(j) if &j != old => j,
                    _ => return false,
                }
            }
        };
        self.opinions.insert(about, joined);
        true
    }

    /// The opinion of `p`, when it follows true from something.
    fn told_true(&self, p: &Prop) -> Option<judgement::Value> {
        let o = self.opinions.get(p).filter(|o| !o.rests_for().is_empty())?;
        Some(judgement::Value::Opinion(o.clone()))
    }

    /// Every rule, forward and back, and every all and any by the
    /// points opinions are about, until nothing changes: whether it got
    /// there within the rounds.
    fn run(&mut self, rules: &[Prop], quantified: &[Prop], settings: &Settings) -> bool {
        for _ in 0..settings.rounds {
            let mut changed = false;
            for t in rules {
                let (Some(premises), Some(posts)) = (
                    self.judgement.premises(t),
                    self.judgement.posts(t).map(<[Prop]>::to_vec),
                ) else {
                    continue;
                };
                // Forward: from every premise, each post.
                let all: Option<Vec<_>> = premises.iter().map(|p| self.told_true(p)).collect();
                if let Some(all) = all {
                    for post in 0..posts.len() {
                        let op = judgement::Op::Ponens { t: t.clone(), post };
                        if let Some(o) = self.apply(op, all.clone()) {
                            changed |= self.add(o);
                        }
                    }
                }
                // Back: from a post false and every premise but one, that
                // one is false.
                for (post, q) in posts.iter().enumerate() {
                    let Some(false_q) = self
                        .opinions
                        .get(q)
                        .filter(|o| !o.rests_against().is_empty())
                        .cloned()
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
                        let args =
                            [vec![judgement::Value::Opinion(false_q.clone())], others].concat();
                        let op = judgement::Op::Tollens {
                            t: t.clone(),
                            post,
                            blame,
                        };
                        if let Some(o) = self.apply(op, args) {
                            changed |= self.add(o);
                        }
                    }
                }
            }
            for q in quantified {
                changed |= self.points(q);
            }
            if !changed {
                return true;
            }
        }
        false
    }

    /// An all or an any at the points opinions are about: down to each
    /// (an instance), and up from each (a witness: an all false, an any
    /// true).
    fn points(&mut self, q: &Prop) -> bool {
        let mut changed = false;
        let mut points: BTreeSet<Vec<Value>> = BTreeSet::new();
        for p in self.opinions.keys() {
            if let Prop::Correct {
                at: Term::Point(x), ..
            } = p
            {
                points.insert(x.clone());
            }
        }
        for x in points {
            let Some(body) = q.at(&x) else { continue };
            let point = || judgement::Value::Point(Point::of(x.clone()));
            if let Some(of_q) = self.opinions.get(q).cloned() {
                let op = judgement::Op::Instance { q: q.clone() };
                if let Some(o) = self.apply(op, vec![judgement::Value::Opinion(of_q), point()]) {
                    changed |= self.add(o);
                }
            }
            if let Some(of_body) = self.opinions.get(&body).cloned() {
                let op = judgement::Op::Witness { q: q.clone() };
                if let Some(o) = self.apply(op, vec![judgement::Value::Opinion(of_body), point()]) {
                    changed |= self.add(o);
                }
            }
        }
        changed
    }

    /// An all true, or an any false, from every one of its points, when
    /// its set is small enough: each point by the first taking of the
    /// ledger that says so there. One taking a point: the ways to have
    /// every point grow as the product of the ways at each, so the
    /// others, and what the model alone says, are left to each point.
    fn cases(
        &self,
        q: &Prop,
        evidence: &BTreeMap<Prop, Vec<Taking>>,
        settings: &Settings,
    ) -> Option<Opinion> {
        let (Prop::All { over, .. } | Prop::Any { over, .. }) = q else {
            return None;
        };
        let all = matches!(q, Prop::All { .. });
        let listed: Vec<Vec<Value>> = over
            .points()?
            .take(settings.cases.saturating_add(1))
            .collect();
        if listed.len() > settings.cases {
            return None;
        }
        let cases: Option<Vec<_>> = listed
            .iter()
            .map(|x| {
                let body = q.at(x)?;
                let first = evidence.get(&body)?.iter().find(|t| t.value == all)?;
                let o = Opinion::taken(body, vec![first.clone()])?;
                Some(judgement::Value::Opinion(o))
            })
            .collect();
        self.apply(judgement::Op::Cases { q: q.clone() }, cases?)
    }
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
