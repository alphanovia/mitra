//! The model (`docs/model.md`): what you write about a system, as data.
//!
//! A model is parts and ties. A part is in the system (a subject) or
//! outside it (an environment). It has records and functions, what it
//! requires and what it ensures. Records and functions carry claims,
//! expressions of simulation that must be true of them, and rules,
//! propositions that give a proposition. A tie says where an item is in
//! the real system.
//!
//! `check` says where a model is not well formed. It runs nothing but
//! each rule's proof, once, on what the proof reads taken true.
//! `props` gives the propositions judgement reasons about, each with
//! the path of what it is about. `levels` and `leaves` are read from
//! the model, never declared: how far each record and function is
//! specified, and which propositions nothing concludes, so that only someone
//! taking them can hold them.

use crate::expression::{check as typed, eval, Expression};
use crate::judgement::{self, Judgement, Opinion, Prop, State, Taking};
use crate::location::Location;
use crate::simulation::{self, Simulation, Type};
use std::collections::{BTreeMap, BTreeSet};

/// Where something is in a model, always from its part: a part, then
/// one of its records, functions, requires or ensures, then a claim or
/// a rule of a record or function.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Path {
    pub names: Vec<String>,
}

impl Path {
    pub fn to(names: &[&str]) -> Path {
        Path {
            names: names.iter().map(|n| n.to_string()).collect(),
        }
    }
    fn then(&self, name: &str) -> Path {
        let mut names = self.names.clone();
        names.push(name.to_string());
        Path { names }
    }
    /// The path written `a::b::c`.
    fn read(written: &str) -> Path {
        let names = written.split("::").map(str::to_string).collect();
        Path { names }
    }
}

/// `stone::drop::falls`.
impl std::fmt::Display for Path {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.names.join("::"))
    }
}

/// What you write about a system.
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub parts: Vec<Part>,
    pub ties: Vec<Tie>,
}

/// In the system, or outside it. A label for now: nothing reads it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// In the system: what is built, or bought and used.
    Subject,
    /// Outside it: what gives the system flows, or takes them.
    Environment,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Part {
    pub name: String,
    pub kind: Kind,
    pub records: Vec<Record>,
    pub functions: Vec<Function>,
    pub requires: Vec<Require>,
    pub ensures: Vec<Ensure>,
}

/// Data. `fields` is `None` when they are not known: the record is
/// opaque. Its type is named by the record, so record names are unique
/// in a model.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub name: String,
    pub fields: Option<Vec<(String, Type)>>,
    pub claims: Vec<Claim>,
    pub rules: Vec<Rule>,
}

/// From what it takes, what it gives, named so that claims can read
/// it. `body` is `None` when how is not known: the function is opaque.
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub takes: Vec<(String, Type)>,
    pub gives: (String, Type),
    pub body: Option<Expression<simulation::Op>>,
    pub claims: Vec<Claim>,
    pub rules: Vec<Rule>,
}

/// A proposition written as an expression: true of its record, read as
/// `self`, or of its function, read with what it takes and gives.
#[derive(Clone, Debug, PartialEq)]
pub struct Claim {
    pub name: String,
    pub holds: Expression<simulation::Op>,
}

/// Its premises give its conclusion. `proof` is `None` when nothing
/// shows it: the rule is assumed. A proof reads the premises, and the
/// rules of its part, as opinions, and gives one of the conclusion.
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub name: String,
    pub premises: Vec<Term>,
    pub conclusion: Term,
    pub proof: Option<Expression<judgement::Op>>,
}

/// A term of a rule: a proposition as the rule writes it. The claim,
/// require or ensure at a path in the rule's part, or an expression
/// written in place, read as a claim of the rule's record or function.
/// A rule is not one: a proof uses a rule by its path.
#[derive(Clone, Debug, PartialEq)]
pub enum Term {
    At(Path),
    Holds(Expression<simulation::Op>),
}

/// What a part needs.
#[derive(Clone, Debug, PartialEq)]
pub struct Require {
    pub name: String,
    pub from: Source,
}

/// Where a require comes from.
#[derive(Clone, Debug, PartialEq)]
pub enum Source {
    /// Said in words. Nothing gives it: it is a hypothesis.
    Prose(String),
    /// What another part ensures, which gives it.
    Ensure(Path),
}

/// What a part promises, in words. `from` is how it follows: from any
/// one of its lists, each of claims, rules, requires and ensures of its
/// part that hold together. When it is empty, only someone taking the
/// ensure holds it.
#[derive(Clone, Debug, PartialEq)]
pub struct Ensure {
    pub name: String,
    pub prose: String,
    pub from: Vec<Vec<Path>>,
}

/// Where a part, record or function is in the real system: that it is
/// what is there, and that the model says all that matters of it there.
/// One belief, which can be wrong.
#[derive(Clone, Debug, PartialEq)]
pub struct Tie {
    pub item: Path,
    pub at: Location,
}

/// How far a record or function is specified. Read, never declared.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Level {
    /// Nothing but its name and types.
    Opaque,
    /// Claims or rules, but no fields or body.
    Partial,
    /// Fields, or a body.
    Total,
}

/// Where a model is not well formed. Each names the declaration at
/// fault.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Problem {
    /// A name declared twice where it must be once.
    Twice(Path),
    /// A path that names nothing its place can reach, or nothing of the
    /// kind needed there.
    Unknown { at: Path, names: Path },
    /// An expression that does not check to what it must, a proof that
    /// does not show its rule, or a type or a record made that no
    /// record declares.
    Mistyped(Path),
    /// A list of an ensure's `from` with nothing in it.
    Empty(Path),
    /// A declaration on a cycle: what it gives is among what it rests
    /// on.
    Circular(Path),
    /// A name that is not a word: letters, digits and `_`, and not
    /// first a digit.
    Misnamed(Path),
}

/// The problems of a model, none when it is well formed: each name a
/// word, declared once; each path naming what it must, in its own part,
/// or, for a require, in another; each claim a truth, each body of the
/// type its function gives, each record made as declared; each proof
/// showing its rule; and nothing resting on itself.
pub fn check(model: &Model) -> Vec<Problem> {
    let index = Index::of(model);
    let mut problems = index.problems.clone();
    for part in &model.parts {
        let at = Path::to(&[&part.name]);
        for record in &part.records {
            let here = at.then(&record.name);
            let fields = record.fields.iter().flatten();
            if !fields.clone().all(|(_, t)| index.known(t)) {
                problems.push(Problem::Mistyped(here.clone()));
            }
            problems.extend(named(&here, fields.map(|(name, _)| name)));
            index.check_said(
                &here,
                Of::Record(record),
                &record.claims,
                &record.rules,
                &mut problems,
            );
        }
        for function in &part.functions {
            let here = at.then(&function.name);
            let names = function.takes.iter().chain([&function.gives]);
            problems.extend(named(&here, names.clone().map(|(name, _)| name)));
            let takes = function.takes.iter().cloned().collect();
            let body = function.body.as_ref();
            let gives = body.map(|b| typed(&Simulation, &takes, b));
            if !names.clone().all(|(_, t)| index.known(t))
                || gives.is_some_and(|t| t.as_ref() != Some(&function.gives.1))
                || body.is_some_and(|b| !index.made(b))
            {
                problems.push(Problem::Mistyped(here.clone()));
            }
            index.check_said(
                &here,
                Of::Function(function),
                &function.claims,
                &function.rules,
                &mut problems,
            );
        }
        for require in &part.requires {
            if let Source::Ensure(names) = &require.from {
                let ensure = matches!(index.items.get(names), Some(Item::Ensure(_)));
                if !ensure || names.names.first() == Some(&part.name) {
                    let at = at.then(&require.name);
                    problems.push(Problem::Unknown {
                        at,
                        names: names.clone(),
                    });
                }
            }
        }
        for ensure in &part.ensures {
            let here = at.then(&ensure.name);
            for list in &ensure.from {
                if list.is_empty() {
                    problems.push(Problem::Empty(here.clone()));
                }
                for names in list.iter().filter(|p| !index.reaches(&part.name, p, true)) {
                    let at = here.clone();
                    problems.push(Problem::Unknown {
                        at,
                        names: names.clone(),
                    });
                }
            }
        }
    }
    for tie in &model.ties {
        let item = index.items.get(&tie.item);
        if !matches!(item, Some(Item::Part | Item::Record | Item::Function)) {
            let (at, names) = (tie.item.clone(), tie.item.clone());
            problems.push(Problem::Unknown { at, names });
        }
    }
    problems.extend(index.circular());
    problems.sort();
    problems.dedup();
    problems
}

/// How far each record and function is specified.
pub fn levels(model: &Model) -> Vec<(Path, Level)> {
    let level = |known: bool, said: bool| match (known, said) {
        (true, _) => Level::Total,
        (false, true) => Level::Partial,
        (false, false) => Level::Opaque,
    };
    let mut levels = Vec::new();
    for part in &model.parts {
        let at = Path::to(&[&part.name]);
        for r in &part.records {
            let said = !r.claims.is_empty() || !r.rules.is_empty();
            levels.push((at.then(&r.name), level(r.fields.is_some(), said)));
        }
        for f in &part.functions {
            let said = !f.claims.is_empty() || !f.rules.is_empty();
            levels.push((at.then(&f.name), level(f.body.is_some(), said)));
        }
    }
    levels
}

/// The propositions of a model, as judgement reads them, each with the
/// path of what it is about:
///
/// - a claim, a require or an ensure is the proposition named by its path;
/// - a rule is `{premises} rule {conclusion}`, an expression written in
///   it the proposition named by the rule's path and its place, `0`, `1`, …
///   for a premise and `then` for the conclusion;
/// - a require from an ensure adds `{ensure} require {require}`, and an
///   ensure adds `{list} ensure {ensure}` for each list of its `from`,
///   where a rule is its proposition;
/// - a tie is the proposition named `item at place`.
///
/// What names nothing gives no proposition, and a proposition is given once.
pub fn props(model: &Model) -> Vec<(Path, Prop)> {
    let index = Index::of(model);
    let mut props = Vec::new();
    for (at, item) in &index.items {
        let named = Prop::Named(at.to_string());
        match item {
            Item::Claim => props.push((at.clone(), named)),
            Item::Rule(rule) => {
                let written = rule.premises.iter().enumerate();
                let written = written.map(|(i, p)| (i.to_string(), p));
                for (place, term) in written.chain([("then".to_string(), &rule.conclusion)]) {
                    if let Term::Holds(_) = term {
                        props.push((at.clone(), Prop::Named(format!("{at}::{place}"))));
                    }
                }
                props.extend(index.rule_prop(at, rule).map(|p| (at.clone(), p)));
            }
            Item::Require(require) => {
                props.push((at.clone(), named.clone()));
                if let Source::Ensure(ensure) = &require.from {
                    if let Some(Item::Ensure(_)) = index.items.get(ensure) {
                        let pre = vec![Prop::Named(ensure.to_string())];
                        props.push((at.clone(), implies(pre, at, named)));
                    }
                }
            }
            Item::Ensure(ensure) => {
                props.push((at.clone(), named.clone()));
                for list in &ensure.from {
                    let pre: Option<Vec<Prop>> = list.iter().map(|p| index.listed(p)).collect();
                    if let Some(pre) = pre.filter(|pre| !pre.is_empty()) {
                        props.push((at.clone(), implies(pre, at, named.clone())));
                    }
                }
            }
            Item::Part | Item::Record | Item::Function => {}
        }
    }
    for tie in &model.ties {
        if let Some(Item::Part | Item::Record | Item::Function) = index.items.get(&tie.item) {
            let named = format!("{} at {}", tie.item, tie.at.path.join("/"));
            props.push((tie.item.clone(), Prop::Named(named)));
        }
    }
    let mut given = BTreeSet::new();
    props.retain(|prop| given.insert(prop.clone()));
    props
}

/// The propositions nothing in the model concludes: only someone taking them
/// holds them. A rule concludes its conclusion, a require from an
/// ensure is concluded by it, an ensure by its lists, and a rule by its
/// proof when the proof shows it. A tie is always a leaf. Read from any
/// model, they are all that holds the rest only in one `check` accepts.
pub fn leaves(model: &Model) -> Vec<(Path, Prop)> {
    let index = Index::of(model);
    let props = props(model);
    let mut concluded = BTreeSet::new();
    for (at, prop) in &props {
        if let Prop::Implies { post, .. } = prop {
            concluded.extend(post.iter().cloned());
            if let Some(Item::Rule(rule)) = index.items.get(at) {
                if index.shows(at, rule) == Some(true) {
                    concluded.insert(prop.clone());
                }
            }
        }
    }
    props
        .into_iter()
        .filter(|(_, p)| !concluded.contains(p))
        .collect()
}

/// `{pre} at {post}`.
fn implies(pre: Vec<Prop>, at: &Path, post: Prop) -> Prop {
    Prop::Implies {
        pre,
        subject: at.to_string(),
        post: vec![post],
    }
}

/// Letters, digits and `_`, and not first a digit.
fn word(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// A problem for each name of `names` that is not a word, or that is
/// met before.
fn named<'a>(at: &Path, names: impl Iterator<Item = &'a String>) -> Vec<Problem> {
    let (mut seen, mut problems) = (BTreeSet::new(), Vec::new());
    for name in names {
        if !word(name) {
            problems.push(Problem::Misnamed(at.then(name)));
        }
        if !seen.insert(name) {
            problems.push(Problem::Twice(at.then(name)));
        }
    }
    problems
}

/// What a path names.
#[derive(Clone, Copy)]
enum Item<'m> {
    Part,
    Record,
    Function,
    Claim,
    Rule(&'m Rule),
    Require(&'m Require),
    Ensure(&'m Ensure),
}

/// What a claim or a rule is of.
#[derive(Clone, Copy)]
enum Of<'m> {
    Record(&'m Record),
    Function(&'m Function),
}

/// The names a claim of `of` reads, and their types.
fn scope(of: Of) -> BTreeMap<String, Type> {
    match of {
        Of::Record(r) => BTreeMap::from([("self".to_string(), record_type(r))]),
        Of::Function(f) => f.takes.iter().chain([&f.gives]).cloned().collect(),
    }
}

/// The type of a record's values. An opaque one has no field to read.
fn record_type(r: &Record) -> Type {
    let fields = r.fields.clone().unwrap_or_default();
    Type::Record(simulation::Record {
        name: r.name.clone(),
        fields,
    })
}

/// The names an expression reads from outside it.
fn free<Op>(e: &Expression<Op>) -> BTreeSet<String> {
    fn walk<Op>(e: &Expression<Op>, bound: &mut Vec<String>, found: &mut BTreeSet<String>) {
        match e {
            Expression::Name(n) => {
                if !bound.contains(n) {
                    found.insert(n.clone());
                }
            }
            Expression::Apply(_, args) => {
                for arg in args {
                    let depth = bound.len();
                    bound.extend(arg.binds.iter().cloned());
                    walk(&arg.body, bound, found);
                    bound.truncate(depth);
                }
            }
        }
    }
    let mut found = BTreeSet::new();
    walk(e, &mut Vec::new(), &mut found);
    found
}

/// The component of each node, where `out` holds the nodes each goes
/// to: two nodes are in one component when each reaches the other.
/// Kosaraju's walk, on stacks of its own, so that a long chain does not
/// run out of the call stack.
fn components(out: &[Vec<usize>]) -> Vec<usize> {
    let n = out.len();
    let (mut finished, mut seen) = (Vec::with_capacity(n), vec![false; n]);
    for start in 0..n {
        if seen[start] {
            continue;
        }
        seen[start] = true;
        let mut walk = vec![(start, 0)];
        while let Some(&(node, next)) = walk.last() {
            let top = walk.len() - 1;
            match out[node].get(next) {
                Some(&to) => {
                    walk[top].1 += 1;
                    if !seen[to] {
                        seen[to] = true;
                        walk.push((to, 0));
                    }
                }
                None => {
                    finished.push(node);
                    walk.pop();
                }
            }
        }
    }
    let mut back = vec![Vec::new(); n];
    for (from, to) in out.iter().enumerate() {
        for &to in to {
            back[to].push(from);
        }
    }
    let mut component = vec![usize::MAX; n];
    for (c, &start) in finished.iter().rev().enumerate() {
        if component[start] != usize::MAX {
            continue;
        }
        component[start] = c;
        let mut walk = vec![start];
        while let Some(node) = walk.pop() {
            for &from in &back[node] {
                if component[from] == usize::MAX {
                    component[from] = c;
                    walk.push(from);
                }
            }
        }
    }
    component
}

/// Every path of a model, each record by name, and the problems found
/// on the way: names that are not words, or declared twice.
struct Index<'m> {
    items: BTreeMap<Path, Item<'m>>,
    records: BTreeMap<&'m str, &'m Record>,
    problems: Vec<Problem>,
}

impl<'m> Index<'m> {
    fn of(model: &'m Model) -> Index<'m> {
        let mut index = Index {
            items: BTreeMap::new(),
            records: BTreeMap::new(),
            problems: Vec::new(),
        };
        for part in &model.parts {
            let at = Path::to(&[&part.name]);
            index.add(&at, Item::Part);
            for r in &part.records {
                if index.records.insert(&r.name, r).is_some() {
                    index.problems.push(Problem::Twice(at.then(&r.name)));
                }
                index.add_said(&at.then(&r.name), Item::Record, &r.claims, &r.rules);
            }
            for f in &part.functions {
                index.add_said(&at.then(&f.name), Item::Function, &f.claims, &f.rules);
            }
            for q in &part.requires {
                index.add(&at.then(&q.name), Item::Require(q));
            }
            for e in &part.ensures {
                index.add(&at.then(&e.name), Item::Ensure(e));
            }
        }
        index
    }
    /// `at` names `item`, unless it names something already.
    fn add(&mut self, at: &Path, item: Item<'m>) {
        if !at.names.last().is_some_and(|n| word(n)) {
            self.problems.push(Problem::Misnamed(at.clone()));
        }
        match self.items.contains_key(at) {
            true => self.problems.push(Problem::Twice(at.clone())),
            false => _ = self.items.insert(at.clone(), item),
        }
    }
    /// A record or function, and what it says.
    fn add_said(&mut self, at: &Path, item: Item<'m>, claims: &[Claim], rules: &'m [Rule]) {
        self.add(at, item);
        for c in claims {
            self.add(&at.then(&c.name), Item::Claim);
        }
        for r in rules {
            self.add(&at.then(&r.name), Item::Rule(r));
        }
    }
    /// `p` names a claim, a require or an ensure of the part `part`, or
    /// a rule of it too when `rules`.
    fn reaches(&self, part: &str, p: &Path, rules: bool) -> bool {
        let here = p.names.first().is_some_and(|name| name == part);
        here && match self.items.get(p) {
            Some(Item::Claim | Item::Require(_) | Item::Ensure(_)) => true,
            Some(Item::Rule(_)) => rules,
            _ => false,
        }
    }
    /// `t` is a type: a truth, a number, a tuple of types, or the type
    /// of a record of the model.
    fn known(&self, t: &Type) -> bool {
        match t {
            Type::Truth | Type::Int | Type::Real => true,
            // A tuple: a record with no name, its fields `0`, `1`, ….
            Type::Record(r) if r.name.is_empty() => {
                let mut fields = r.fields.iter().enumerate();
                fields.all(|(i, (name, t))| *name == i.to_string() && self.known(t))
            }
            Type::Record(r) => {
                let declared = self.records.get(r.name.as_str());
                declared.is_some_and(|d| record_type(d) == *t)
                    && r.fields.iter().all(|(_, t)| self.known(t))
            }
        }
    }
    /// Each record `e` makes is a tuple, or a record of the model with
    /// its fields known, made with them.
    fn made(&self, e: &Expression<simulation::Op>) -> bool {
        let Expression::Apply(op, args) = e else {
            return true;
        };
        let fits = match op {
            simulation::Op::Make(r) => {
                let declared = self.records.get(r.name.as_str());
                (r.name.is_empty() || declared.is_some_and(|d| d.fields.is_some()))
                    && self.known(&Type::Record(r.clone()))
            }
            _ => true,
        };
        fits && args.iter().all(|arg| self.made(&arg.body))
    }
    /// `term`, at `place` in the rule at `rule`, as judgement reads it: `None`
    /// when it names no claim, require or ensure.
    fn prop(&self, rule: &Path, place: &str, term: &Term) -> Option<Prop> {
        match term {
            Term::At(at) => match self.items.get(at)? {
                Item::Claim | Item::Require(_) | Item::Ensure(_) => {
                    Some(Prop::Named(at.to_string()))
                }
                _ => None,
            },
            Term::Holds(_) => Some(Prop::Named(format!("{rule}::{place}"))),
        }
    }
    /// `{premises} rule {conclusion}`, for the rule at `at`.
    fn rule_prop(&self, at: &Path, rule: &Rule) -> Option<Prop> {
        let pre = rule.premises.iter().enumerate();
        let pre: Option<Vec<Prop>> = pre.map(|(i, p)| self.prop(at, &i.to_string(), p)).collect();
        Some(implies(pre?, at, self.prop(at, "then", &rule.conclusion)?))
    }
    /// What a list names, as judgement reads it: a claim, a require or an
    /// ensure by its path, a rule by its own proposition.
    fn listed(&self, at: &Path) -> Option<Prop> {
        match self.items.get(at)? {
            Item::Claim | Item::Require(_) | Item::Ensure(_) => Some(Prop::Named(at.to_string())),
            Item::Rule(rule) => self.rule_prop(at, rule),
            Item::Part | Item::Record | Item::Function => None,
        }
    }
    /// Whether the proof of the rule at `at` shows it. It may read the
    /// rule's premises, each by its path or, written in place, by its
    /// place, and the rules of its part, by their paths: nothing else
    /// has a type. It must check to an opinion of the conclusion. And
    /// since an opinion can be against its proposition (modus tollens) or come
    /// from a conflict (reductio), it is run once, on all it may read
    /// taken true: it must give the conclusion true. `None` when there
    /// is no proof, or when what it reads has no proposition, a problem found
    /// where it is.
    fn shows(&self, at: &Path, rule: &Rule) -> Option<bool> {
        let proof = rule.proof.as_ref()?;
        let conclusion = self.prop(at, "then", &rule.conclusion)?;
        let mut reads = BTreeMap::new();
        for (i, p) in rule.premises.iter().enumerate() {
            let name = match p {
                Term::At(path) => path.to_string(),
                Term::Holds(_) => i.to_string(),
            };
            reads.insert(name, self.prop(at, &i.to_string(), p)?);
        }
        for name in free(proof) {
            let cited = Path::read(&name);
            if let Some(Item::Rule(r)) = self.items.get(&cited) {
                if cited.names[0] == at.names[0] {
                    reads.insert(name, self.rule_prop(&cited, r)?);
                }
            }
        }
        let types = reads.iter().map(|(name, prop)| {
            let opinion = judgement::Type::Opinion(prop.clone());
            (name.clone(), opinion)
        });
        let gives = typed(&Judgement, &types.collect(), proof);
        if gives != Some(judgement::Type::Opinion(conclusion)) {
            return Some(false);
        }
        let mut values = BTreeMap::new();
        for (name, prop) in reads {
            let taking = Taking {
                prop: prop.clone(),
                value: true,
                count: 1,
                by: name.clone(),
            };
            let opinion = Opinion::taken(prop, vec![taking])?;
            values.insert(name, judgement::Value::Opinion(opinion));
        }
        let given = eval(&Judgement, &values, proof);
        Some(matches!(given, Ok(judgement::Value::Opinion(o)) if o.state() == State::True))
    }
    /// A problem for each declaration on a cycle of what rests on what:
    /// a rule's conclusion on its premises and on the rule, a rule on
    /// the rules its proof names, a require on its ensure, and an ensure
    /// on what its lists name.
    fn circular(&self) -> Vec<Problem> {
        // From, to, and the declaration that says so.
        let mut rests: Vec<(Path, Path, &Path)> = Vec::new();
        for (at, item) in &self.items {
            match item {
                Item::Rule(rule) => {
                    let node = |place: String, term: &Term| match term {
                        Term::At(p) => p.clone(),
                        Term::Holds(_) => at.then(&place),
                    };
                    let then = node("then".to_string(), &rule.conclusion);
                    for (i, p) in rule.premises.iter().enumerate() {
                        rests.push((then.clone(), node(i.to_string(), p), at));
                    }
                    rests.push((then, at.clone(), at));
                    for name in rule.proof.iter().flat_map(free) {
                        let cited = Path::read(&name);
                        if let Some(Item::Rule(_)) = self.items.get(&cited) {
                            rests.push((at.clone(), cited, at));
                        }
                    }
                }
                Item::Require(require) => {
                    if let Source::Ensure(ensure) = &require.from {
                        rests.push((at.clone(), ensure.clone(), at));
                    }
                }
                Item::Ensure(ensure) => {
                    for p in ensure.from.iter().flatten() {
                        rests.push((at.clone(), p.clone(), at));
                    }
                }
                Item::Part | Item::Record | Item::Function | Item::Claim => {}
            }
        }
        let mut number: BTreeMap<&Path, usize> = BTreeMap::new();
        for (from, to, _) in &rests {
            for p in [from, to] {
                let next = number.len();
                number.entry(p).or_insert(next);
            }
        }
        let mut out = vec![Vec::new(); number.len()];
        for (from, to, _) in &rests {
            out[number[from]].push(number[to]);
        }
        let component = components(&out);
        let on_cycle = |from: &Path, to: &Path| component[number[from]] == component[number[to]];
        rests
            .iter()
            .filter(|(from, to, _)| on_cycle(from, to))
            .map(|(_, _, at)| Problem::Circular((*at).clone()))
            .collect()
    }
    /// The claims and rules of the record or function at `at`.
    fn check_said(
        &self,
        at: &Path,
        of: Of,
        claims: &[Claim],
        rules: &[Rule],
        problems: &mut Vec<Problem>,
    ) {
        let scope = scope(of);
        let truth = |e| typed(&Simulation, &scope, e) == Some(Type::Truth) && self.made(e);
        for c in claims.iter().filter(|c| !truth(&c.holds)) {
            problems.push(Problem::Mistyped(at.then(&c.name)));
        }
        for rule in rules {
            let here = at.then(&rule.name);
            for term in rule.premises.iter().chain([&rule.conclusion]) {
                match term {
                    Term::At(names) if !self.reaches(&at.names[0], names, false) => {
                        let (at, names) = (here.clone(), names.clone());
                        problems.push(Problem::Unknown { at, names });
                    }
                    Term::Holds(e) if !truth(e) => problems.push(Problem::Mistyped(here.clone())),
                    _ => {}
                }
            }
            if self.shows(&here, rule) == Some(false) {
                problems.push(Problem::Mistyped(here.clone()));
            }
        }
    }
}
