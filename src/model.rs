//! The model (`docs/model.md`): what you write about a system, as data.
//!
//! A model is parts, contracts, examples of the whole and ties. A part is
//! in the system (a subject) or outside it (an environment). It has
//! records and functions, what it requires and what it ensures, and the
//! contracts its functions fit. It reads another part's functions and
//! records through a `use`. Records and functions carry claims,
//! expressions of simulation that must be true of them, and rules,
//! propositions that give a proposition. A contract is a property
//! written once for every function that fits it. An example of the
//! whole says what the model gives at some inputs, across parts. A tie
//! says where an item is in the real system.
//!
//! `check` (`src/check.rs`) says where a model is not well formed.
//! `props` gives the propositions judgement reasons about, each with
//! the path of what it is about, and `judgement` what the rules among
//! them say, each known by its path. `levels` and `leaves` are read from
//! the model, never declared: how far each record and function is
//! specified, and which propositions nothing concludes, so that only someone
//! taking them can hold them.

use crate::expression::{check as typed, eval, Arg, Expression};
use crate::judgement::{self, Definition, Judgement, Opinion, Prop, State, Taking};
use crate::location::Location;
use crate::simulation::{self, Bound, Callee, Calls, Type, Value};
use std::collections::{BTreeMap, BTreeSet};

/// Where something is in a model, always from its part: a part, then
/// one of its records, functions, requires or ensures, then a claim or
/// a rule of a record or function, or the name of what it takes or
/// gives. A contract's claim, read of a function that fits it, is
/// under the function and the contract: `stone::speed::Monotone::rises`.
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
    pub contracts: Vec<Contract>,
    /// What the model gives across parts, end to end.
    pub examples: Vec<ModelExample>,
    pub ties: Vec<Tie>,
}

/// In the system, or outside it. Only explain reads it: what a
/// subject's opaque function was seen to give may be copied as its
/// examples, not what the environment gave.
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
    /// What it reads of other parts.
    pub uses: Vec<Use>,
    pub records: Vec<Record>,
    pub enums: Vec<Enum>,
    pub functions: Vec<Function>,
    /// The contracts its functions fit.
    pub implements: Vec<Implements>,
    pub requires: Vec<Require>,
    pub ensures: Vec<Ensure>,
    /// The file it was read from, when not the model's own:
    /// `std/core.mitra`, or `geometry.mitra` beside the model.
    pub file: Option<String>,
}

/// What a part reads of another, as Rust's `use`: the functions it
/// calls and the records it names. A call is written by its path,
/// `sun::light`, in `simulation::Op::Call`.
#[derive(Clone, Debug, PartialEq)]
pub enum Use {
    /// `use sun::{light, Ray};`
    Names { part: String, names: Vec<String> },
    /// `use sun::*;`: all its functions and records.
    Glob { part: String },
}

impl Use {
    /// The part it reads.
    pub fn part(&self) -> &str {
        match self {
            Use::Names { part, .. } | Use::Glob { part } => part,
        }
    }
    /// It lets its part read `name` of `part`.
    pub fn reads(&self, part: &str, name: &str) -> bool {
        match self {
            Use::Names { part: p, names } => p == part && names.iter().any(|n| n == name),
            Use::Glob { part: p } => p == part,
        }
    }
}

/// A property written once, over types it names but does not fix, for
/// every function that fits it: Rust's trait. `contract Monotone<T:
/// Number> { fn f(x: T) -> y: T; claim rises(a: T, b: T): !(a <= b) ||
/// f(a) <= f(b); }`. Its claims call its functions by `Monotone::f`.
/// It is checked once; an `impl` only checks that functions fit.
#[derive(Clone, Debug, PartialEq)]
pub struct Contract {
    pub name: String,
    /// `T`, or `T: Number`.
    pub params: Vec<(String, Bound)>,
    /// What each function that fits takes and gives.
    pub functions: Vec<Signature>,
    pub claims: Vec<Law>,
}

/// What a function takes and gives, by name.
#[derive(Clone, Debug, PartialEq)]
pub struct Signature {
    pub name: String,
    pub takes: Vec<(String, Type)>,
    pub gives: (String, Type),
}

/// A claim of a contract, with its own arguments: true at every value
/// of them. `rises(a: T, b: T) = !(a <= b) || f(a) <= f(b)`.
#[derive(Clone, Debug, PartialEq)]
pub struct Law {
    pub name: String,
    pub over: Vec<(String, Type)>,
    pub holds: Expression<simulation::Op>,
}

/// `impl Monotone<Real> for speed;`: functions of its part, one for each
/// function of the contract in order, fit it with these types, one for
/// each parameter in order. Each claim of the contract is then one about
/// the first of them.
#[derive(Clone, Debug, PartialEq)]
pub struct Implements {
    pub contract: String,
    pub types: Vec<Type>,
    pub functions: Vec<String>,
}

/// A row of the model as a whole: at these inputs, the model gives these
/// outputs. Each is named by its function and the name of what it takes
/// or gives, `stone::speed::h`, as in a measurement. An input is what a
/// function takes, or what one with no body gives (`sun::light::w`, the
/// world's value here); an output is what a function gives. The spec of
/// an end-to-end test: through calls, the model computes the outputs.
#[derive(Clone, Debug, PartialEq)]
pub struct ModelExample {
    pub inputs: Vec<(Path, Value)>,
    pub outputs: Vec<(Path, Value)>,
}

/// Data. `fields` is `None` when they are not known: the record is
/// opaque. Its type is named by the record, so record names are unique
/// in a model.
#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    pub name: String,
    /// Its type parameters, `record Pair<A, B> { a: A, b: B }`: each
    /// stands in its fields for a type given where the record is used.
    pub params: Vec<(String, Bound)>,
    pub fields: Option<Vec<(String, Type)>>,
    pub claims: Vec<Claim>,
    pub rules: Vec<Rule>,
}

/// A value that is one of its variants, Rust's `enum`: `enum Option<T>
/// { None, Some(T) }`. Each variant holds values of the types it lists,
/// none for `None`. Its name is a type's, unique among records and
/// enums of the model; its variants' names are read alone, `Some(1)`,
/// wherever it is used.
#[derive(Clone, Debug, PartialEq)]
pub struct Enum {
    pub name: String,
    pub params: Vec<(String, Bound)>,
    pub variants: Vec<(String, Vec<Type>)>,
}

impl Enum {
    /// The type it declares, its parameters left open.
    pub fn type_(&self) -> Type {
        Type::Enum(simulation::Enum {
            name: self.name.clone(),
            args: self
                .params
                .iter()
                .map(|(n, b)| Type::Param {
                    name: n.clone(),
                    bound: *b,
                })
                .collect(),
            variants: self.variants.clone(),
        })
    }
}

impl Record {
    /// The type it declares, its parameters left open; an opaque one has
    /// no field to read.
    pub fn type_(&self) -> Type {
        Type::Record(simulation::Record {
            name: self.name.clone(),
            args: self
                .params
                .iter()
                .map(|(n, b)| Type::Param {
                    name: n.clone(),
                    bound: *b,
                })
                .collect(),
            fields: self.fields.clone().unwrap_or_default(),
        })
    }
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
    /// The range types among what it takes and gives (`a: 0..=9`): each
    /// an `Int` that is checked (finding 11).
    pub ranges: simulation::Ranged,
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
/// rule, require or ensure at a path in the rule's part; an expression
/// written in place, read as a claim of the rule's record or function;
/// or where functions of its part are correct: `correct(div, (6, 2))`,
/// or an all or an any of those, `all x in (a: 0..10, b: 0..10):
/// correct(div, x)`, the function by its path.
#[derive(Clone, Debug, PartialEq)]
pub enum Term {
    At(Path),
    Holds(Expression<simulation::Op>),
    Prop(Prop),
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

impl Tie {
    /// The proposition the tie is: `item at place`.
    pub fn prop(&self) -> Prop {
        Prop::Named(format!("{} at {}", self.item, self.at.path.join("/")))
    }
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
            let fits = part
                .implements
                .iter()
                .any(|i| i.functions.first() == Some(&f.name));
            let said = !f.claims.is_empty() || !f.rules.is_empty() || !f.examples.is_empty();
            levels.push((at.then(&f.name), level(f.body.is_some(), said || fits)));
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
///   the conclusion; where functions are correct, written in a rule, is
///   that proposition itself, shared by every rule that writes it;
/// - a contract's claim, read of a function that fits it, is a claim;
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
                    match term {
                        Term::Holds(_) => {
                            props.push((at.clone(), Prop::Named(format!("{at}::{place}"))))
                        }
                        Term::Prop(p) if p.well_formed() => props.push((at.clone(), p.clone())),
                        Term::At(_) | Term::Prop(_) => {}
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
            Item::Part | Item::Record | Item::Enum | Item::Function => {}
        }
    }
    for tie in &model.ties {
        if let Some(Item::Part | Item::Record | Item::Enum | Item::Function) =
            index.items.get(&tie.item)
        {
            props.push((tie.item.clone(), tie.prop()));
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
    Enum,
    Function,
    Claim,
    Rule(&'m Rule),
    Require(&'m Require),
    Ensure(&'m Ensure),
}

/// Every path of a model, and each record by name with its part, each
/// function by its path, and each contract by name: the first, where a
/// name is declared twice (`check` says so). And judgement, knowing
/// every rule it can, by path: not those named through themselves
/// (`cyclic`), nor those a term of which names nothing.
pub(crate) struct Index<'m> {
    pub(crate) items: BTreeMap<Path, Item<'m>>,
    pub(crate) records: BTreeMap<&'m str, &'m Record>,
    pub(crate) record_parts: BTreeMap<&'m str, &'m str>,
    pub(crate) enums: BTreeMap<&'m str, &'m Enum>,
    pub(crate) enum_parts: BTreeMap<&'m str, &'m str>,
    pub(crate) functions: BTreeMap<String, &'m Function>,
    pub(crate) contracts: BTreeMap<&'m str, &'m Contract>,
    pub(crate) judgement: Judgement,
    /// What each rule judgement knows says, by path.
    pub(crate) definitions: BTreeMap<String, Definition>,
    pub(crate) cyclic: BTreeSet<String>,
    /// The rules judgement does not know.
    unknown: BTreeSet<String>,
    /// What a call by a name alone names, for messages.
    pub(crate) calling: crate::parse::Calling,
    /// The parts read from Mitra's standard model (`use std::…`).
    pub(crate) std: BTreeSet<&'m str>,
}

impl<'m> Index<'m> {
    pub(crate) fn of(model: &'m Model) -> Index<'m> {
        let mut index = Index {
            items: BTreeMap::new(),
            records: BTreeMap::new(),
            record_parts: BTreeMap::new(),
            enums: BTreeMap::new(),
            enum_parts: BTreeMap::new(),
            functions: BTreeMap::new(),
            contracts: BTreeMap::new(),
            judgement: Judgement::default(),
            definitions: BTreeMap::new(),
            cyclic: BTreeSet::new(),
            unknown: BTreeSet::new(),
            calling: crate::parse::Calling::of(model),
            std: model
                .parts
                .iter()
                .filter(|p| p.file.as_ref().is_some_and(|f| f.starts_with("std/")))
                .map(|p| p.name.as_str())
                .collect(),
        };
        for c in &model.contracts {
            index.contracts.entry(&c.name).or_insert(c);
        }
        for part in &model.parts {
            let at = Path::to(&[&part.name]);
            index.add(&at, Item::Part);
            for r in &part.records {
                index.records.entry(&r.name).or_insert(r);
                index.record_parts.entry(&r.name).or_insert(&part.name);
                index.add_said(&at.then(&r.name), Item::Record, &r.claims, &r.rules);
            }
            for e in &part.enums {
                index.enums.entry(&e.name).or_insert(e);
                index.enum_parts.entry(&e.name).or_insert(&part.name);
                index.add(&at.then(&e.name), Item::Enum);
            }
            for f in &part.functions {
                index
                    .functions
                    .entry(at.then(&f.name).to_string())
                    .or_insert(f);
                index.add_said(&at.then(&f.name), Item::Function, &f.claims, &f.rules);
            }
            for i in &part.implements {
                let (Some(c), Some(first)) = (
                    index.contracts.get(i.contract.as_str()),
                    i.functions.first(),
                ) else {
                    continue;
                };
                for law in &c.claims {
                    index.add(
                        &at.then(first).then(&i.contract).then(&law.name),
                        Item::Claim,
                    );
                }
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
        index.definitions = whole.clone();
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
            Item::Part | Item::Record | Item::Enum | Item::Function => None,
        }
    }
    /// `term`, at `place` in the rule at `rule`, as judgement reads it: `None`
    /// when it names no proposition.
    fn prop(&self, rule: &Path, place: &str, term: &Term) -> Option<Prop> {
        match term {
            Term::At(at) => self.prop_at(at),
            Term::Holds(_) => Some(Prop::Named(format!("{rule}::{place}"))),
            Term::Prop(p) => Some(p.clone()).filter(Prop::well_formed),
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
                Term::Holds(_) | Term::Prop(_) => i.to_string(),
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

/// The model's functions as simulation calls them, each by its path:
/// what it takes and gives, its body, and its examples as rows. The
/// names of the functions `Calls::with` refuses, when it does: `check`
/// says why.
pub fn calls(model: &Model) -> Result<Calls, BTreeSet<String>> {
    let mut callees = BTreeMap::new();
    for part in &model.parts {
        for f in &part.functions {
            let rows = f
                .examples
                .iter()
                .map(|e| (e.takes.clone(), e.gives.clone()));
            let path = Path::to(&[&part.name, &f.name]);
            // With no body, its claims promise what it gives.
            let promises = f.body.is_none().then(|| simulation::Promises {
                gives: f.gives.0.clone(),
                claims: f
                    .claims
                    .iter()
                    .map(|c| (path.then(&c.name).to_string(), c.holds.clone()))
                    .collect(),
            });
            let callee = Callee {
                promises,
                takes: f.takes.clone(),
                gives: f.gives.1.clone(),
                body: f.body.clone(),
                rows: rows.collect(),
                ranges: f.ranges.clone(),
            };
            let path = Path::to(&[&part.name, &f.name]).to_string();
            callees.entry(path).or_insert(callee);
        }
    }
    Calls::with(callees)
}

/// The model's functions by what they take and give only, each by its
/// path: enough to type a call, whatever their bodies.
pub(crate) fn signatures(model: &Model) -> Calls {
    let mut callees = BTreeMap::new();
    for part in &model.parts {
        for f in &part.functions {
            let path = Path::to(&[&part.name, &f.name]).to_string();
            callees
                .entry(path)
                .or_insert_with(|| bare(&f.takes, &f.gives.1));
        }
    }
    Calls::with(callees).unwrap_or_default()
}

/// A contract's functions, each by `Contract::f`, to type its claims.
pub(crate) fn contract_calls(contract: &Contract) -> Calls {
    let mut callees = BTreeMap::new();
    for f in &contract.functions {
        let path = format!("{}::{}", contract.name, f.name);
        callees
            .entry(path)
            .or_insert_with(|| bare(&f.takes, &f.gives.1));
    }
    Calls::with(callees).unwrap_or_default()
}

fn bare(takes: &[(String, Type)], gives: &Type) -> Callee {
    Callee {
        promises: None,
        takes: takes.to_vec(),
        gives: gives.clone(),
        body: None,
        rows: vec![],
        ranges: simulation::Ranged::default(),
    }
}

/// A contract's claim read of the functions that fit it: at its path,
/// over its arguments with the contract's types given, each call of a
/// function of the contract a call of the one that fits it.
#[derive(Clone, Debug, PartialEq)]
pub struct ContractClaim {
    pub at: Path,
    pub over: Vec<(String, Type)>,
    pub holds: Expression<simulation::Op>,
}

/// Each contract's claim, read of each `impl` of the contract. An
/// `impl` that does not fit (`check` says why) gives none.
pub fn contract_claims(model: &Model) -> Vec<ContractClaim> {
    let index = Index::of(model);
    let mut claims = Vec::new();
    for part in &model.parts {
        let at = Path::to(&[&part.name]);
        for i in &part.implements {
            let Some(contract) = index.contracts.get(i.contract.as_str()) else {
                continue;
            };
            let Some(first) = i.functions.first() else {
                continue;
            };
            if crate::check::misfit(&index, part, i).is_some() {
                continue;
            }
            let given: BTreeMap<String, Type> = contract
                .params
                .iter()
                .map(|(n, _)| n.clone())
                .zip(i.types.iter().cloned())
                .collect();
            let to: BTreeMap<String, String> = contract
                .functions
                .iter()
                .zip(&i.functions)
                .map(|(c, f)| {
                    (
                        format!("{}::{}", contract.name, c.name),
                        at.then(f).to_string(),
                    )
                })
                .collect();
            for law in &contract.claims {
                claims.push(ContractClaim {
                    at: at.then(first).then(&i.contract).then(&law.name),
                    over: law
                        .over
                        .iter()
                        .map(|(n, t)| (n.clone(), t.given(&given)))
                        .collect(),
                    holds: fitted(&law.holds, &to, &given),
                });
            }
        }
    }
    claims
}

/// `e` with each call `to` names made a call of the function it gives,
/// and each type parameter `given` names fixed, in what it makes.
fn fitted(
    e: &Expression<simulation::Op>,
    to: &BTreeMap<String, String>,
    given: &BTreeMap<String, Type>,
) -> Expression<simulation::Op> {
    use simulation::Op;
    match e {
        Expression::Name(n) => Expression::Name(n.clone()),
        Expression::Apply(op, args) => {
            let op = match op {
                Op::Call(f) => Op::Call(to.get(f).cloned().unwrap_or_else(|| f.clone())),
                Op::Make(r) => match Type::Record(r.clone()).given(given) {
                    Type::Record(r) => Op::Make(r),
                    _ => op.clone(),
                },
                Op::Variant(e, i) => match Type::Enum(e.clone()).given(given) {
                    Type::Enum(e) => Op::Variant(e, *i),
                    _ => op.clone(),
                },
                Op::Match(e) => match Type::Enum(e.clone()).given(given) {
                    Type::Enum(e) => Op::Match(e),
                    _ => op.clone(),
                },
                Op::Seq(t) => Op::Seq(t.given(given)),
                _ => op.clone(),
            };
            let args = args.iter().map(|a| Arg {
                binds: a.binds.clone(),
                body: fitted(&a.body, to, given),
            });
            Expression::Apply(op, args.collect())
        }
    }
}
