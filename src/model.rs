//! The model (`docs/model.md`): what you write about a system, as data.
//!
//! A model is parts and ties. A part is in the system (a subject) or
//! outside it (an environment). It has records and functions, what it
//! requires and what it ensures. Records and functions carry claims,
//! expressions of simulation that must be true of them, and rules,
//! propositions that give a proposition. A tie says where an item is in
//! the real system.
//!
//! Nothing here runs or judges. `check` says where a model is not well
//! formed. `pieces` gives the propositions judgement reasons about, each
//! with the path of what it is about. `levels` and `leaves` are read
//! from the model, never declared: how far each record and function is
//! specified, and which pieces nothing concludes, so that only someone
//! taking them can hold them.

use crate::expression::{check as typed, Expression};
use crate::judgement::{self, Judgement, Piece};
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

/// In the system, or outside it.
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
/// shows it: the rule is assumed.
#[derive(Clone, Debug, PartialEq)]
pub struct Rule {
    pub name: String,
    pub premises: Vec<Prop>,
    pub conclusion: Prop,
    pub proof: Option<Expression<judgement::Op>>,
}

/// A proposition: the claim, rule, require or ensure at a path, or an
/// expression written in place, read as a claim of the rule's record or
/// function.
#[derive(Clone, Debug, PartialEq)]
pub enum Prop {
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
    /// What a part ensures, which gives it.
    Ensure(Path),
}

/// What a part promises, in words. `from` is how it follows: from any
/// one of its lists, each of propositions that hold together. When it
/// is empty, only someone taking the ensure holds it.
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
    /// A path that names nothing, or nothing of the kind needed there.
    Unknown { at: Path, names: Path },
    /// An expression that does not check to what it must, or a type no
    /// record declares.
    Mistyped(Path),
    /// A list of an ensure's `from` with nothing in it.
    Empty(Path),
    /// A rule that is among its own premises or conclusion, through
    /// rules.
    Circular(Path),
}

/// The problems of a model, none when it is well formed: each name
/// once, each path naming what it must, each claim a truth, each body
/// of the type its function gives, each proof an opinion of its rule's
/// conclusion from opinions of its premises, and each type declared.
pub fn check(model: &Model) -> Vec<Problem> {
    let index = Index::of(model);
    let mut problems = index.twice.clone();
    for part in &model.parts {
        let at = Path::to(&[&part.name]);
        for record in &part.records {
            let here = at.then(&record.name);
            let fields = record.fields.iter().flatten();
            if record.name.is_empty() || !fields.clone().all(|(_, t)| index.known(t)) {
                problems.push(Problem::Mistyped(here.clone()));
            }
            problems.extend(twice(&here, fields.map(|(name, _)| name)));
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
            problems.extend(twice(&here, names.clone().map(|(name, _)| name)));
            let takes = function.takes.iter().cloned().collect();
            let body = function
                .body
                .as_ref()
                .map(|b| typed(&Simulation, &takes, b));
            if !names.clone().all(|(_, t)| index.known(t))
                || body.is_some_and(|t| t.as_ref() != Some(&function.gives.1))
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
                if !matches!(index.items.get(names), Some(Item::Ensure(_))) {
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
                for names in list.iter().filter(|p| !index.is_prop(p)) {
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
/// - a claim, a require or an ensure is the piece named by its path;
/// - a rule is `{premises} rule {conclusion}`, an expression written in
///   it the piece named by the rule's path and its place, `0`, `1`, …
///   for a premise and `then` for the conclusion;
/// - a require from an ensure adds `{ensure} require {require}`, and an
///   ensure adds `{list} ensure {ensure}` for each list of its `from`;
/// - a tie is the piece named `item at place`.
///
/// What names nothing, or a rule on a cycle, gives no piece.
pub fn pieces(model: &Model) -> Vec<(Path, Piece)> {
    let index = Index::of(model);
    let mut pieces = Vec::new();
    for (at, item) in &index.items {
        let named = Piece::Named(at.to_string());
        match item {
            Item::Claim => pieces.push((at.clone(), named)),
            Item::Rule(rule) => {
                let written = rule.premises.iter().enumerate();
                let written = written.map(|(i, p)| (i.to_string(), p));
                for (place, prop) in written.chain([("then".to_string(), &rule.conclusion)]) {
                    if let Prop::Holds(_) = prop {
                        pieces.push((at.clone(), Piece::Named(format!("{at}::{place}"))));
                    }
                }
                pieces.extend(index.piece(at, &mut Vec::new()).map(|p| (at.clone(), p)));
            }
            Item::Require(require) => {
                pieces.push((at.clone(), named.clone()));
                if let Source::Ensure(ensure) = &require.from {
                    if let Some(Item::Ensure(_)) = index.items.get(ensure) {
                        let pre = vec![Piece::Named(ensure.to_string())];
                        pieces.push((at.clone(), implies(pre, at, named)));
                    }
                }
            }
            Item::Ensure(ensure) => {
                pieces.push((at.clone(), named.clone()));
                for list in &ensure.from {
                    let pre: Option<Vec<Piece>> = list
                        .iter()
                        .map(|p| index.piece(p, &mut Vec::new()))
                        .collect();
                    if let Some(pre) = pre.filter(|pre| !pre.is_empty()) {
                        pieces.push((at.clone(), implies(pre, at, named.clone())));
                    }
                }
            }
            Item::Part | Item::Record | Item::Function => {}
        }
    }
    for tie in &model.ties {
        let named = format!("{} at {}", tie.item, tie.at.path.join("/"));
        pieces.push((tie.item.clone(), Piece::Named(named)));
    }
    pieces
}

/// The pieces nothing in the model concludes: only someone taking them
/// holds them. A rule concludes its conclusion, a require from an
/// ensure is concluded by it, an ensure by its lists, and a rule with a
/// proof by its proof. A tie is always a leaf.
pub fn leaves(model: &Model) -> Vec<(Path, Piece)> {
    let index = Index::of(model);
    let pieces = pieces(model);
    let mut concluded = BTreeSet::new();
    for (at, piece) in &pieces {
        if let Piece::Implies { post, subject, .. } = piece {
            concluded.extend(post.iter().cloned());
            let proved = matches!(index.items.get(at), Some(Item::Rule(r)) if r.proof.is_some());
            if proved && *subject == at.to_string() {
                concluded.insert(piece.clone());
            }
        }
    }
    pieces
        .into_iter()
        .filter(|(_, p)| !concluded.contains(p))
        .collect()
}

/// `{pre} at {post}`.
fn implies(pre: Vec<Piece>, at: &Path, post: Piece) -> Piece {
    Piece::Implies {
        pre,
        subject: at.to_string(),
        post: vec![post],
    }
}

/// A problem for each name of `names` met before.
fn twice<'a>(at: &Path, names: impl Iterator<Item = &'a String>) -> Vec<Problem> {
    let mut seen = BTreeSet::new();
    names
        .filter(|n| !seen.insert(*n))
        .map(|n| Problem::Twice(at.then(n)))
        .collect()
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

/// Every path of a model, and each record by name.
struct Index<'m> {
    items: BTreeMap<Path, Item<'m>>,
    records: BTreeMap<&'m str, &'m Record>,
    twice: Vec<Problem>,
}

impl<'m> Index<'m> {
    fn of(model: &'m Model) -> Index<'m> {
        let mut index = Index {
            items: BTreeMap::new(),
            records: BTreeMap::new(),
            twice: Vec::new(),
        };
        for part in &model.parts {
            let at = Path::to(&[&part.name]);
            index.add(&at, Item::Part);
            for r in &part.records {
                if index.records.insert(&r.name, r).is_some() {
                    index.twice.push(Problem::Twice(at.then(&r.name)));
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
        match self.items.contains_key(at) {
            true => self.twice.push(Problem::Twice(at.clone())),
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
    /// `at` names a claim, a rule, a require or an ensure.
    fn is_prop(&self, at: &Path) -> bool {
        matches!(
            self.items.get(at),
            Some(Item::Claim | Item::Rule(_) | Item::Require(_) | Item::Ensure(_))
        )
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
    /// The piece of the proposition at `at`, `None` when there is none.
    /// `around` holds the rules whose piece is being made, so a rule on
    /// a cycle has none.
    fn piece(&self, at: &Path, around: &mut Vec<Path>) -> Option<Piece> {
        match self.items.get(at)? {
            Item::Claim | Item::Require(_) | Item::Ensure(_) => Some(Piece::Named(at.to_string())),
            Item::Rule(rule) if !around.contains(at) => {
                around.push(at.clone());
                let pre = rule.premises.iter().enumerate();
                let pre: Option<Vec<Piece>> = pre
                    .map(|(i, p)| self.prop(at, &i.to_string(), p, around))
                    .collect();
                let post = self.prop(at, "then", &rule.conclusion, around);
                around.pop();
                Some(implies(pre?, at, post?))
            }
            _ => None,
        }
    }
    /// The piece of `prop`, at `place` in the rule at `rule`.
    fn prop(&self, rule: &Path, place: &str, prop: &Prop, around: &mut Vec<Path>) -> Option<Piece> {
        match prop {
            Prop::At(at) => self.piece(at, around),
            Prop::Holds(_) => Some(Piece::Named(format!("{rule}::{place}"))),
        }
    }
    /// The rule at `at` is among its own premises or conclusion, through
    /// rules.
    fn circular(&self, at: &Path) -> bool {
        let (mut todo, mut seen) = (vec![at.clone()], BTreeSet::new());
        while let Some(next) = todo.pop() {
            if let Some(Item::Rule(rule)) = self.items.get(&next) {
                for prop in rule.premises.iter().chain([&rule.conclusion]) {
                    if let Prop::At(path) = prop {
                        if path == at {
                            return true;
                        }
                        if seen.insert(path.clone()) {
                            todo.push(path.clone());
                        }
                    }
                }
            }
        }
        false
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
        let truth = |e| typed(&Simulation, &scope, e) == Some(Type::Truth);
        for c in claims.iter().filter(|c| !truth(&c.holds)) {
            problems.push(Problem::Mistyped(at.then(&c.name)));
        }
        for rule in rules {
            let here = at.then(&rule.name);
            for prop in rule.premises.iter().chain([&rule.conclusion]) {
                match prop {
                    Prop::At(names) if !self.is_prop(names) => {
                        let (at, names) = (here.clone(), names.clone());
                        problems.push(Problem::Unknown { at, names });
                    }
                    Prop::Holds(e) if !truth(e) => problems.push(Problem::Mistyped(here.clone())),
                    _ => {}
                }
            }
            if self.circular(&here) {
                problems.push(Problem::Circular(here.clone()));
            } else if let Some(proof) = &rule.proof {
                // Each premise is read by its path, or, written in place,
                // by its place.
                let premises = rule.premises.iter().enumerate().map(|(i, p)| {
                    let name = match p {
                        Prop::At(path) => path.to_string(),
                        Prop::Holds(_) => i.to_string(),
                    };
                    let piece = self.prop(&here, &i.to_string(), p, &mut Vec::new())?;
                    Some((name, judgement::Type::Opinion(piece)))
                });
                let premises: Option<BTreeMap<_, _>> = premises.collect();
                let conclusion = self.prop(&here, "then", &rule.conclusion, &mut Vec::new());
                if let (Some(premises), Some(conclusion)) = (premises, conclusion) {
                    if typed(&Judgement, &premises, proof)
                        != Some(judgement::Type::Opinion(conclusion))
                    {
                        problems.push(Problem::Mistyped(here.clone()));
                    }
                }
            }
        }
    }
}
