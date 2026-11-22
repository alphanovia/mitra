//! The model (`docs/model.md`): what you write about a system, as data.
//!
//! A model is parts and ties. A part is in the system (a subject) or
//! outside it (an environment). It has records and functions, what it
//! requires and what it ensures. Records and functions carry claims,
//! expressions of simulation that must be true of them, and rules,
//! propositions that give a proposition. A tie says where an item is in
//! the real system.
//!
//! `check` (`src/check.rs`) says where a model is not well formed.
//! `props` gives the propositions judgement reasons about, each with
//! the path of what it is about, and `judgement` what the rules among
//! them say, each known by its path. `levels` and `leaves` are read from
//! the model, never declared: how far each record and function is
//! specified, and which propositions nothing concludes, so that only someone
//! taking them can hold them.

use crate::expression::{check as typed, eval, Expression};
use crate::judgement::{self, Definition, Judgement, Opinion, Prop, State, Taking};
use crate::location::Location;
use crate::simulation::{self, Type};
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
    pub(crate) fn then(&self, name: &str) -> Path {
        let mut names = self.names.clone();
        names.push(name.to_string());
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
/// `examples` say what it gives at some points.
#[derive(Clone, Debug, PartialEq)]
pub struct Function {
    pub name: String,
    pub takes: Vec<(String, Type)>,
    pub gives: (String, Type),
    pub body: Option<Expression<simulation::Op>>,
    pub examples: Vec<Example>,
    pub claims: Vec<Claim>,
    pub rules: Vec<Rule>,
}

/// A row of a function's examples, `(6, 2) -> 3`: at what it takes, in
/// order, what it gives. The model's answer at that point, beside the
/// body if there is one, or in its place.
#[derive(Clone, Debug, PartialEq)]
pub struct Example {
    pub takes: Vec<simulation::Value>,
    pub gives: simulation::Value,
}

/// A proposition written as an expression: true of its record, read as
/// `self`, or of its function, read with what it takes and gives.
#[derive(Clone, Debug, PartialEq)]
pub struct Claim {
    pub name: String,
    pub holds: Expression<simulation::Op>,
}

/// Its premises give its conclusion. `proof` is `None` when nothing
/// shows it: the rule is assumed. A proof reads the premises as
/// opinions, and gives one of the conclusion.
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub name: String,
    pub premises: Vec<Term>,
    pub conclusion: Term,
    pub proof: Option<Expression<judgement::Op>>,
}

/// A term of a rule: a proposition as the rule writes it. The claim,
/// rule, require or ensure at a path in the rule's part, or an
/// expression written in place, read as a claim of the rule's record or
/// function.
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
    /// Claims, rules or examples, but no fields or body.
    Partial,
    /// Fields, or a body.
    Total,
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
            let said = !f.claims.is_empty() || !f.rules.is_empty() || !f.examples.is_empty();
            levels.push((at.then(&f.name), level(f.body.is_some(), said)));
        }
    }
    levels
}

/// The propositions of a model, as judgement reads them, each with the
/// path of what it is about:
///
/// - a claim, a rule, a require or an ensure is the proposition named
///   by its path; what a rule says, `judgement` knows;
/// - an expression written in a rule is the proposition named by the
///   rule's path and its place, `0`, `1`, … for a premise and `then` for
///   the conclusion;
/// - a require from an ensure adds `{ensure} require {require}`, and an
///   ensure adds `{list} ensure {ensure}` for each list of its `from`;
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
                props.push((at.clone(), named));
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
                    let pre: Option<Vec<Prop>> = list.iter().map(|p| index.prop_at(p)).collect();
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

/// Judgement that knows the rules of a model, each by its path: its
/// premises give its conclusion, as `props` gives them. A rule a term of
/// which names nothing is not known, nor one named through itself:
/// `check` finds both.
pub fn judgement(model: &Model) -> Judgement {
    Index::of(model).judgement
}

/// The propositions nothing in the model concludes: only someone taking them
/// holds them. A rule concludes its conclusion, a require from an
/// ensure is concluded by it, and an ensure by its lists. A rule with a
/// proof is a leaf all the same: when the proof shows it, the checker
/// takes it (`machine:mitra-check`), as Lean or Verus could, a firm
/// taking but a taking. A tie is always a leaf. Read from any model,
/// they are all that holds the rest only in one `check` accepts.
pub fn leaves(model: &Model) -> Vec<(Path, Prop)> {
    let index = Index::of(model);
    let props = props(model);
    let mut concluded = BTreeSet::new();
    for (_, prop) in &props {
        if let Some(posts) = index.judgement.posts(prop) {
            concluded.extend(posts.iter().cloned());
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

/// What a path names.
#[derive(Clone, Copy)]
pub(crate) enum Item<'m> {
    Part,
    Record,
    Function,
    Claim,
    Rule(&'m Rule),
    Require(&'m Require),
    Ensure(&'m Ensure),
}

/// Every path of a model, and each record by name: the first, where a
/// name is declared twice (`check` says so). And judgement, knowing
/// every rule it can, by path: not those named through themselves
/// (`cyclic`), nor those a term of which names nothing.
pub(crate) struct Index<'m> {
    pub(crate) items: BTreeMap<Path, Item<'m>>,
    pub(crate) records: BTreeMap<&'m str, &'m Record>,
    pub(crate) judgement: Judgement,
    pub(crate) cyclic: BTreeSet<String>,
    /// The rules judgement does not know.
    unknown: BTreeSet<String>,
}

impl<'m> Index<'m> {
    pub(crate) fn of(model: &'m Model) -> Index<'m> {
        let mut index = Index {
            items: BTreeMap::new(),
            records: BTreeMap::new(),
            judgement: Judgement::default(),
            cyclic: BTreeSet::new(),
            unknown: BTreeSet::new(),
        };
        for part in &model.parts {
            let at = Path::to(&[&part.name]);
            index.add(&at, Item::Part);
            for r in &part.records {
                index.records.entry(&r.name).or_insert(r);
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
        // Every rule as far as its terms name propositions, and the rules
        // all of whose terms do.
        let (mut all, mut whole) = (BTreeMap::new(), BTreeMap::new());
        for (at, item) in &index.items {
            if let Item::Rule(rule) = item {
                let (definition, named) = index.definition(at, rule);
                if named {
                    whole.insert(at.to_string(), definition.clone());
                } else {
                    index.unknown.insert(at.to_string());
                }
                all.insert(at.to_string(), definition);
            }
        }
        // A rule is named through itself whatever else is wrong with it.
        index.cyclic = Judgement::with(all).err().unwrap_or_default();
        whole.retain(|name, _| !index.cyclic.contains(name));
        index.unknown.extend(index.cyclic.iter().cloned());
        // Without those, none left is named through itself.
        index.judgement = Judgement::with(whole).unwrap_or_default();
        index
    }
    /// `at` names `item`, unless it names something already.
    fn add(&mut self, at: &Path, item: Item<'m>) {
        self.items.entry(at.clone()).or_insert(item);
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
    /// The proposition at `at`, named by its path: a claim, a rule, a
    /// require or an ensure. `None` for anything else.
    pub(crate) fn prop_at(&self, at: &Path) -> Option<Prop> {
        match self.items.get(at)? {
            Item::Claim | Item::Rule(_) | Item::Require(_) | Item::Ensure(_) => {
                Some(Prop::Named(at.to_string()))
            }
            Item::Part | Item::Record | Item::Function => None,
        }
    }
    /// `term`, at `place` in the rule at `rule`, as judgement reads it: `None`
    /// when it names no proposition.
    fn prop(&self, rule: &Path, place: &str, term: &Term) -> Option<Prop> {
        match term {
            Term::At(at) => self.prop_at(at),
            Term::Holds(_) => Some(Prop::Named(format!("{rule}::{place}"))),
        }
    }
    /// What the rule at `at` says, as judgement reads it, but for the
    /// terms that name no proposition; and whether every term names one.
    fn definition(&self, at: &Path, rule: &Rule) -> (Definition, bool) {
        let pre = rule.premises.iter().enumerate();
        let pre: Vec<_> = pre.map(|(i, p)| self.prop(at, &i.to_string(), p)).collect();
        let post = self.prop(at, "then", &rule.conclusion);
        let named = post.is_some() && pre.iter().all(Option::is_some);
        let pre = pre.into_iter().flatten().collect();
        (
            Definition {
                pre,
                post: post.into_iter().collect(),
            },
            named,
        )
    }
    /// Whether the proof of the rule at `at` shows it. It reads the
    /// rule's premises, each by its path or, written in place, by its
    /// place: nothing else has a type. It must check to an opinion of
    /// the conclusion. And since an opinion can be against its
    /// proposition (modus tollens) or come from a conflict (reductio), it
    /// is run once, on all it reads taken true: it must give the
    /// conclusion true. `None` when there is no proof, or when a term
    /// names no proposition, or the proof applies a rule of the model
    /// judgement does not know: a problem found where it is.
    pub(crate) fn shows(&self, at: &Path, rule: &Rule) -> Option<bool> {
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
        let unknown = |t: &Prop| matches!(t, Prop::Named(n) if self.unknown.contains(n));
        if applied(proof).into_iter().any(unknown) {
            return None;
        }
        let types = reads.iter().map(|(name, prop)| {
            let opinion = judgement::Type::Opinion(prop.clone());
            (name.clone(), opinion)
        });
        let gives = typed(&self.judgement, &types.collect(), proof);
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
        let given = eval(&self.judgement, &values, proof);
        Some(matches!(given, Ok(judgement::Value::Opinion(o)) if o.state() == State::True))
    }
}

/// The rules a proof applies, by modus ponens or tollens.
fn applied(proof: &Expression<judgement::Op>) -> Vec<&Prop> {
    let (mut applied, mut todo) = (Vec::new(), vec![proof]);
    while let Some(e) = todo.pop() {
        if let Expression::Apply(op, args) = e {
            if let judgement::Op::Ponens { t, .. } | judgement::Op::Tollens { t, .. } = op {
                applied.push(t);
            }
            todo.extend(args.iter().map(|arg| &arg.body));
        }
    }
    applied
}
