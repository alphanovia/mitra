//! Is the model well formed? `check` gives every error, each with its
//! code (`src/catalogue.rs`, `docs/reference`) and the declaration to
//! fix. It runs nothing but each rule's proof, once, on what the proof
//! reads taken true.

use crate::catalogue::{
    Code, Error, M001, M002, M003, M004, M005, M006, M007, M008, M009, M010, M011, M012, M013, M014,
};
use crate::expression::{check as typed, Expression};
use crate::graph::components;
use crate::model::{Claim, Function, Index, Item, Model, Path, Record, Rule, Source, Term};
use crate::simulation::{self, Simulation, Type};
use std::collections::{BTreeMap, BTreeSet};

/// The errors of a model, none when it is well formed: each name a
/// word, declared once; each type declared, each record made as
/// declared, each body of the type its function gives, each claim a
/// truth; each path naming what it must, in its own part, or, for a
/// require, an ensure of another; no empty list; each proof showing its
/// rule; nothing resting on itself, and no rule named through itself.
pub fn check(model: &Model) -> Vec<Error> {
    let index = Index::of(model);
    let mut errors = names(model);
    for part in &model.parts {
        let at = Path::to(&[&part.name]);
        for record in &part.records {
            let here = at.then(&record.name);
            let known = types(&index, &here, record.fields.iter().flatten(), &mut errors);
            let (of, claims, rules) = (Of::Record(record), &record.claims, &record.rules);
            said(&index, &here, of, known, claims, rules, &mut errors);
        }
        for function in &part.functions {
            let here = at.then(&function.name);
            let names = function.takes.iter().chain([&function.gives]);
            let known = types(&index, &here, names, &mut errors);
            if let Some(body) = &function.body {
                let takes = function.takes.iter().cloned().collect();
                let gives = &function.gives;
                if known && typed(&Simulation, &takes, body).as_ref() != Some(&gives.1) {
                    let message = format!("the body is not of the type of `{}`", gives.0);
                    errors.push(error(M005, &here, message));
                }
                if !made(&index, body) {
                    errors.push(error(
                        M004,
                        &here,
                        "the body makes a record other than as declared",
                    ));
                }
            }
            let (of, claims, rules) = (Of::Function(function), &function.claims, &function.rules);
            said(&index, &here, of, known, claims, rules, &mut errors);
        }
        for require in &part.requires {
            if let Source::Ensure(names) = &require.from {
                let ensure = matches!(index.items.get(names), Some(Item::Ensure(_)));
                if !ensure || names.names.first() == Some(&part.name) {
                    let message = format!("`{names}` is no ensure of another part");
                    errors.push(error(M009, &at.then(&require.name), message));
                }
            }
        }
        for ensure in &part.ensures {
            let here = at.then(&ensure.name);
            for list in &ensure.from {
                if list.is_empty() {
                    errors.push(error(M011, &here, "a list is empty"));
                }
                for names in list {
                    errors.extend(names_wrong(&index, &here, names));
                }
            }
        }
    }
    for tie in &model.ties {
        let item = index.items.get(&tie.item);
        if !matches!(item, Some(Item::Part | Item::Record | Item::Function)) {
            let message = format!("`{}` is no part, record or function", tie.item);
            errors.push(error(M010, &tie.item, message));
        }
    }
    errors.extend(rests_on_itself(&index));
    for at in index.items.keys() {
        if index.cyclic.contains(&at.to_string()) {
            let message = format!("`{at}` is named through itself");
            errors.push(error(M014, at, message));
        }
    }
    errors.sort();
    errors.dedup();
    errors
}

fn error(code: Code, at: &Path, message: impl Into<String>) -> Error {
    Error {
        code,
        at: at.clone(),
        message: message.into(),
    }
}

/// Letters, digits and `_`, and not first a digit.
fn word(name: &str) -> bool {
    let mut chars = name.chars();
    chars.next().is_some_and(|c| c.is_alphabetic() || c == '_')
        && chars.all(|c| c.is_alphanumeric() || c == '_')
}

/// M001 and M002: each name a word, declared once in its place, and the
/// name of each record once in the model.
fn names(model: &Model) -> Vec<Error> {
    let (mut errors, mut declared, mut records) = (Vec::new(), Vec::new(), BTreeSet::new());
    for part in &model.parts {
        let at = Path::to(&[&part.name]);
        declared.push(at.clone());
        for r in &part.records {
            let here = at.then(&r.name);
            if !records.insert(&r.name) {
                errors.push(twice(&here));
            }
            declared.push(here.clone());
            declared.extend(said_at(&here, &r.claims, &r.rules));
            errors.extend(listed(&here, r.fields.iter().flatten()));
        }
        for f in &part.functions {
            let here = at.then(&f.name);
            declared.push(here.clone());
            declared.extend(said_at(&here, &f.claims, &f.rules));
            errors.extend(listed(&here, f.takes.iter().chain([&f.gives])));
        }
        declared.extend(part.requires.iter().map(|q| at.then(&q.name)));
        declared.extend(part.ensures.iter().map(|e| at.then(&e.name)));
    }
    let mut seen = BTreeSet::new();
    for at in declared {
        errors.extend(misnamed(&at));
        if !seen.insert(at.clone()) {
            errors.push(twice(&at));
        }
    }
    errors
}

/// The paths of the claims and rules of the record or function at
/// `here`.
fn said_at(here: &Path, claims: &[Claim], rules: &[Rule]) -> Vec<Path> {
    let claims = claims.iter().map(|c| here.then(&c.name));
    claims
        .chain(rules.iter().map(|r| here.then(&r.name)))
        .collect()
}

fn misnamed(at: &Path) -> Option<Error> {
    let name = at.names.last().map(String::as_str).unwrap_or_default();
    let message = format!("`{name}` is not a word");
    (!word(name)).then(|| error(M001, at, message))
}

fn twice(at: &Path) -> Error {
    let name = at.names.last().map(String::as_str).unwrap_or_default();
    error(M002, at, format!("`{name}` is declared twice"))
}

/// M001 and M002 for each name of a list of names and types, read in
/// the item at `at`.
fn listed<'a>(at: &Path, names: impl Iterator<Item = &'a (String, Type)>) -> Vec<Error> {
    let (mut seen, mut errors) = (BTreeSet::new(), Vec::new());
    for (name, _) in names {
        let here = at.then(name);
        errors.extend(misnamed(&here));
        if !seen.insert(name) {
            errors.push(twice(&here));
        }
    }
    errors
}

/// M003 for each name of `names`, in the item at `at`, whose type is
/// not known. Whether all are.
fn types<'a>(
    index: &Index,
    at: &Path,
    names: impl Iterator<Item = &'a (String, Type)>,
    errors: &mut Vec<Error>,
) -> bool {
    let mut all = true;
    for (name, _) in names.filter(|(_, t)| !known(index, t)) {
        let message = format!("the type of `{name}` names no record of the model as declared");
        errors.push(error(M003, &at.then(name), message));
        all = false;
    }
    all
}

/// `t` is a type: a truth, a number, a tuple of types, or the type of a
/// record of the model.
fn known(index: &Index, t: &Type) -> bool {
    match t {
        Type::Truth | Type::Int | Type::Real => true,
        // A tuple: a record with no name, its fields `0`, `1`, ….
        Type::Record(r) if r.name.is_empty() => {
            let mut fields = r.fields.iter().enumerate();
            fields.all(|(i, (name, t))| *name == i.to_string() && known(index, t))
        }
        Type::Record(r) => {
            let declared = index.records.get(r.name.as_str());
            declared.is_some_and(|d| record_type(d) == *t)
                && r.fields.iter().all(|(_, t)| known(index, t))
        }
    }
}

/// Each record `e` makes is a tuple, or a record of the model with its
/// fields known, made with them.
fn made(index: &Index, e: &Expression<simulation::Op>) -> bool {
    let Expression::Apply(op, args) = e else {
        return true;
    };
    let fits = match op {
        simulation::Op::Make(r) => {
            let declared = index.records.get(r.name.as_str());
            (r.name.is_empty() || declared.is_some_and(|d| d.fields.is_some()))
                && known(index, &Type::Record(r.clone()))
        }
        _ => true,
    };
    fits && args.iter().all(|arg| made(index, &arg.body))
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

/// M007 or M008 when `names`, in a rule or list of the declaration at
/// `at`, names no proposition of its part.
fn names_wrong(index: &Index, at: &Path, names: &Path) -> Option<Error> {
    if index.prop_at(names).is_none() {
        let message = format!("`{names}` names no claim, rule, require or ensure");
        return Some(error(M007, at, message));
    }
    let other = names.names.first() != at.names.first();
    let message = format!("`{names}` is of another part: parts meet only through requires");
    other.then(|| error(M008, at, message))
}

/// The claims and rules of the record or function at `at`. Their types
/// are checked only when the types they read are `known`.
fn said(
    index: &Index,
    at: &Path,
    of: Of,
    known: bool,
    claims: &[Claim],
    rules: &[Rule],
    errors: &mut Vec<Error>,
) {
    let scope = scope(of);
    // M006 and M004 for an expression read as a claim, at `here`.
    let claim = |here: &Path, e, what: &str, errors: &mut Vec<Error>| {
        if known && typed(&Simulation, &scope, e) != Some(Type::Truth) {
            errors.push(error(M006, here, format!("{what} is not a truth")));
        }
        if !made(index, e) {
            let message = format!("{what} makes a record other than as declared");
            errors.push(error(M004, here, message));
        }
    };
    for c in claims {
        claim(
            &at.then(&c.name),
            &c.holds,
            &format!("`{}`", c.name),
            errors,
        );
    }
    for rule in rules {
        let here = at.then(&rule.name);
        let premises = rule.premises.iter().enumerate();
        let premises = premises.map(|(i, term)| (format!("premise {i}"), term));
        for (place, term) in premises.chain([("the conclusion".to_string(), &rule.conclusion)]) {
            match term {
                Term::At(names) => errors.extend(names_wrong(index, &here, names)),
                Term::Holds(e) => claim(&here, e, &place, errors),
            }
        }
        if index.shows(&here, rule) == Some(false) {
            let message = format!("the proof does not show `{}`", rule.name);
            errors.push(error(M012, &here, message));
        }
    }
}

/// M013 for each declaration on a cycle of what rests on what: a rule's
/// conclusion on its premises and on the rule, a require on its ensure,
/// and an ensure on what its lists name.
fn rests_on_itself(index: &Index) -> Vec<Error> {
    // From, to, and the declaration that says so.
    let mut rests: Vec<(Path, Path, &Path)> = Vec::new();
    for (at, item) in &index.items {
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
        .map(|(_, _, at)| {
            let message = format!("what `{at}` gives is among what it rests on");
            error(M013, at, message)
        })
        .collect()
}
