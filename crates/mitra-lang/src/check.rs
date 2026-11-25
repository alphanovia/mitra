//! Is the model well formed? `check` gives every error, each with its
//! code (`src/catalogue.rs`, `docs/reference/codes`) and the declaration to
//! fix. It runs nothing but each rule's proof, once, on what the proof
//! reads taken true.

use crate::catalogue::{
    Code, Error, M001, M002, M003, M004, M005, M006, M007, M008, M009, M010, M011, M012, M013,
    M014, M015, M016, M017, M018, M019, M020, M021, M022, M023, M024, M025, M026, M027, M028, M029,
    M030, M031, M032, M033, M034,
};
use crate::expression::Reading;
use crate::expression::{check as typed, Expression};
use crate::graph::components;
use crate::judgement::{self, Prop};
use crate::model::own;
use crate::model::{
    contract_calls, signatures, Claim, Contract, Function, Implements, Index, Item, Model, Part,
    Path, Record, Rule, Source, Step, Term, Use,
};
use crate::simulation::{self, Bound, Exact, Simulation, Type, Value};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// The errors of a model, none when it is well formed: each name a
/// word, declared once; each type declared, each record made as
/// declared, each body of the type its function gives, each claim a
/// truth; each path naming what it must, in its own part, or, for a
/// require, an ensure of another; another part's functions and records
/// read only through a use, and no function calling itself; each
/// contract well formed and each impl fitting it; each example of the
/// model naming inputs and outputs, of their types; no empty list; each
/// proof showing its rule; nothing resting on itself, and no rule named
/// through itself.
pub fn check(model: &Model) -> Vec<Error> {
    let index = Index::of(model);
    let standard = Simulation {
        calls: Arc::new(signatures(model)),
    };
    let mut errors = names(model);
    for contract in &model.contracts {
        errors.extend(contract_errors(&index, contract));
    }
    for part in &model.parts {
        let at = Path::to(&[&part.name]);
        errors.extend(uses(&index, part));
        for e in &part.enums {
            let here = at.then(own(&e.name));
            let holds: Vec<(String, Type)> = e
                .variants
                .iter()
                .flat_map(|(v, ts)| {
                    ts.iter()
                        .enumerate()
                        .map(move |(k, t)| (format!("{v}::{k}"), t.clone()))
                })
                .collect();
            typed_in(&index, &here, holds.iter(), &e.params, &mut errors);
            for (name, t) in &holds {
                errors.extend(unused_records(&index, part, &here.then(name), t));
            }
        }
        for record in &part.records {
            let here = at.then(own(&record.name));
            let known = typed_in(
                &index,
                &here,
                record.fields.iter().flatten(),
                &record.params,
                &mut errors,
            );
            for (name, t) in record.fields.iter().flatten() {
                errors.extend(unused_records(&index, part, &here.then(name), t));
            }
            let (of, claims, rules) = (Of::Record(record), &record.claims, &record.rules);
            said(
                &index,
                &standard,
                part,
                &here,
                of,
                known,
                claims,
                rules,
                &mut errors,
            );
        }
        for function in &part.functions {
            let here = at.then(&function.name);
            let names = function.takes.iter().chain([&function.gives]);
            let known = types(&index, &here, names, &mut errors);
            for (name, t) in function.takes.iter().chain([&function.gives]) {
                errors.extend(unused_records(&index, part, &here.then(name), t));
            }
            if let Some(body) = &function.body {
                let read = reads(&index, part, &here, body);
                // A call of no function is the error, not the type it makes.
                let calls = !read.iter().any(|e| e.code == M017);
                errors.extend(read);
                let takes = function.takes.iter().cloned().collect();
                let gives = &function.gives;
                if known && calls && typed(&standard, &takes, body).as_ref() != Some(&gives.1) {
                    let why = Mistyped {
                        index: &index,
                        part: Some(&part.name),
                        contract: None,
                        standard: &standard,
                    }
                    .explain(&takes, body, &gives.1);
                    let message = format!("the body is not of the type of `{}`{why}", gives.0);
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
            let of_types = |e: &crate::model::Example| {
                e.takes.len() == function.takes.len()
                    && e.takes
                        .iter()
                        .zip(&function.takes)
                        .all(|(v, (_, t))| Exact::default().has_type(v, t))
                    && e.gives
                        .as_ref()
                        .is_none_or(|v| Exact::default().has_type(v, &function.gives.1))
            };
            // Each range holds a number (M025); each example lies in them (M026).
            errors.extend(empty_ranges(function, &here));
            for (i, example) in function.examples.iter().enumerate() {
                let named = function.takes.iter().map(|(n, _)| n).zip(&example.takes);
                let gives = example.gives.iter().map(|v| (&function.gives.0, v));
                for (n, v) in named.chain(gives) {
                    if let Some(range) = range_of(function, n) {
                        if !range.holds(v) && matches!(v, Value::Int(_)) {
                            let message = format!(
                                "example {i} has `{n}` = {}, outside its range {range}",
                                crate::print::value(v)
                            );
                            errors.push(error(M026, &here, message));
                        }
                    }
                }
                let fits = of_types(example);
                if known && !fits {
                    let message = format!("example {i} is not of the types of `{}`", function.name);
                    errors.push(error(M015, &here, message));
                }
                let row = example.takes.iter().chain(&example.gives);
                if !row.into_iter().all(|v| value_made(&index, v)) {
                    let message = format!("example {i} makes a record other than as declared");
                    errors.push(error(M004, &here, message));
                }
                // A function gives one value at a point (of examples of
                // its types: M015 says the rest).
                // Each later one against the first at its point.
                // Each row of its world, what a function with no body gave,
                // as it takes and gives (finding 16).
                for message in given_wrong(&index, &example.given) {
                    errors.push(error(M022, &here, format!("example {i}: {message}")));
                }
                for message in effects_wrong(&index, &example.effects) {
                    errors.push(error(M022, &here, format!("example {i}: {message}")));
                }
                // In one world: two worlds may give two values.
                let first = function.examples[..i]
                    .iter()
                    .all(|e| e.takes != example.takes || e.given != example.given);
                let others = function.examples.iter().enumerate().skip(i + 1);
                for (j, other) in others.filter(|(_, o)| first && fits && of_types(o)) {
                    let (Some(a), Some(b)) = (&example.gives, &other.gives) else {
                        continue;
                    };
                    if other.takes == example.takes && other.given == example.given && a != b {
                        let message = format!(
                            "examples {i} and {j} are both at {}, giving {} and {}",
                            crate::print::values(&example.takes),
                            crate::print::value(a),
                            crate::print::value(b),
                        );
                        errors.push(error(M024, &here, message));
                    }
                }
            }
            let (of, claims, rules) = (Of::Function(function), &function.claims, &function.rules);
            said(
                &index,
                &standard,
                part,
                &here,
                of,
                known,
                claims,
                rules,
                &mut errors,
            );
        }
        for i in &part.implements {
            if let Some(message) = misfit(&index, part, i) {
                errors.push(error(M020, &impl_at(part, i), message));
            }
            for t in &i.types {
                errors.extend(unused_records(&index, part, &impl_at(part, i), t));
            }
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
        errors.extend(nesting(model, part, &at));
        for ensure in &part.ensures {
            let here = at.then(&ensure.name);
            for list in &ensure.from {
                if list.is_empty() {
                    errors.push(error(M011, &here, "a list is empty"));
                }
                for names in list {
                    // A part holding parts rests on its parts' ensures
                    // and its own requires only (M029).
                    if working(model, part) && index.prop_at(names).is_some() {
                        let first = names.names.first().map(String::as_str).unwrap_or("");
                        let item = index.items.get(names);
                        let fits = match first == part.name {
                            true => matches!(item, Some(Item::Require(_))),
                            false => {
                                part.inner.iter().any(|i| i == first)
                                    && matches!(item, Some(Item::Ensure(_)))
                            }
                        };
                        if !fits {
                            let message = format!(
                                "`{names}`: `{}` holds parts, and its ensures rest only on its parts' ensures and its own requires",
                                part.name
                            );
                            errors.push(error(M029, &here, message));
                        }
                        continue;
                    }
                    errors.extend(names_wrong(&index, &here, names));
                }
            }
        }
    }
    for tie in &model.ties {
        let item = index.items.get(&tie.item);
        if !matches!(
            item,
            Some(Item::Part | Item::Record | Item::Enum | Item::Function)
        ) {
            let message = format!("`{}` is no part, record, enum or function", tie.item);
            errors.push(error(M010, &tie.item, message));
        }
    }
    for (i, example) in model.examples.iter().enumerate() {
        errors.extend(model_example(&index, i, example));
    }
    errors.extend(calls_itself(model));
    errors.extend(inductions(model));
    errors.extend(invariants(model));
    errors.extend(declared_wrong(model));
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

/// M027 for each enum with no variant, each type parameter a record or
/// an enum does not use, and each record or enum that holds itself.
fn declared_wrong(model: &Model) -> Vec<Error> {
    let mut errors = Vec::new();
    // What each record and enum holds, by name.
    let mut holds: BTreeMap<&str, Vec<&Type>> = BTreeMap::new();
    let mut by_variant: BTreeMap<&str, Vec<Vec<&Type>>> = BTreeMap::new();
    let mut at: BTreeMap<&str, Path> = BTreeMap::new();
    // Each record and enum, its parameters, and the types it holds.
    type Declared<'a> = (Path, &'a [(String, Bound)], Vec<&'a Type>);
    let mut params: Vec<Declared> = Vec::new();
    for part in &model.parts {
        let here = Path::to(&[&part.name]);
        for e in &part.enums {
            let types: Vec<&Type> = e.variants.iter().flat_map(|(_, ts)| ts).collect();
            if e.variants.is_empty() {
                let message = format!("`{}` has no variant: no value is of it", e.name);
                errors.push(error(M027, &here.then(own(&e.name)), message));
            }
            params.push((here.then(own(&e.name)), &e.params, types.clone()));
            holds.entry(&e.name).or_insert(types);
            by_variant.entry(&e.name).or_insert(
                e.variants
                    .iter()
                    .map(|(_, ts)| ts.iter().collect())
                    .collect(),
            );
            at.entry(&e.name).or_insert(here.then(own(&e.name)));
        }
        for r in &part.records {
            let types: Vec<&Type> = r.fields.iter().flatten().map(|(_, t)| t).collect();
            params.push((here.then(own(&r.name)), &r.params, types.clone()));
            holds.entry(&r.name).or_insert(types);
            at.entry(&r.name).or_insert(here.then(own(&r.name)));
        }
    }
    for (here, ps, types) in params {
        for (p, _) in ps {
            if !types.iter().any(|t| names_param(t, p)) {
                let message =
                    format!("the type parameter `{p}` is used by nothing it holds: Rust's E0392");
                errors.push(error(M027, &here, message));
            }
        }
    }
    let ending = ends(&holds, &by_variant);
    let enums: BTreeSet<&str> = by_variant.keys().copied().collect();
    // A name it holds, however deep, by what each holds.
    for (name, here) in &at {
        let mut seen: BTreeSet<&str> = BTreeSet::new();
        let mut todo: Vec<&Type> = holds.get(name).cloned().unwrap_or_default();
        let mut itself = false;
        while let Some(t) = todo.pop() {
            for n in named_in(t) {
                if n == *name {
                    itself = true;
                } else if seen.insert(n) {
                    todo.extend(holds.get(n).cloned().unwrap_or_default());
                }
            }
        }
        if itself && !ending.contains(name) {
            let message = match enums.contains(name) {
                true => format!(
                    "`{name}` holds itself in every variant: no value of it ends; give it a variant that does not hold it, as a tree's `Leaf`"
                ),
                false => format!(
                    "`{name}` holds itself: no value of it ends; a tree is an enum with a variant that does not hold it"
                ),
            };
            errors.push(error(M027, here, message));
        }
    }
    errors
}

/// The records and enums of which a value ends: an enum with a variant
/// all it holds ends, a record all its fields end (trees, 12:33).
fn ends<'m>(
    holds: &BTreeMap<&'m str, Vec<&'m Type>>,
    variants: &BTreeMap<&'m str, Vec<Vec<&'m Type>>>,
) -> BTreeSet<&'m str> {
    fn ok(t: &Type, ending: &BTreeSet<&str>) -> bool {
        match t {
            Type::Record(r) if r.name.is_empty() => r.fields.iter().all(|(_, t)| ok(t, ending)),
            Type::Record(r) => ending.contains(r.name.as_str()),
            Type::Enum(e) => ending.contains(e.name.as_str()),
            _ => true,
        }
    }
    let mut ending: BTreeSet<&str> = BTreeSet::new();
    loop {
        let before = ending.len();
        for (name, types) in holds {
            let ends = match variants.get(name) {
                Some(vs) => vs.iter().any(|ts| ts.iter().all(|t| ok(t, &ending))),
                None => types.iter().all(|t| ok(t, &ending)),
            };
            if ends {
                ending.insert(name);
            }
        }
        if ending.len() == before {
            return ending;
        }
    }
}

/// `t` names the parameter `p`.
fn names_param(t: &Type, p: &str) -> bool {
    match t {
        Type::Param { name, .. } => name == p,
        Type::Record(r) => r
            .args
            .iter()
            .chain(r.fields.iter().map(|(_, t)| t))
            .any(|t| names_param(t, p)),
        Type::Enum(e) => e.args.iter().any(|t| names_param(t, p)),
        Type::Seq(t) | Type::Encoded(t) => names_param(t, p),
        _ => false,
    }
}

/// The records and enums `t` names, by name, as written.
fn named_in(t: &Type) -> Vec<&str> {
    match t {
        Type::Record(r) => {
            let mut found: Vec<&str> = match r.name.is_empty() {
                true => vec![],
                false => vec![r.name.as_str()],
            };
            for t in r.args.iter().chain(r.fields.iter().map(|(_, t)| t)) {
                found.extend(named_in(t));
            }
            found
        }
        Type::Enum(e) => {
            let mut found = vec![e.name.as_str()];
            for t in &e.args {
                found.extend(named_in(t));
            }
            found
        }
        Type::Seq(t) | Type::Encoded(t) => named_in(t),
        _ => vec![],
    }
}

/// M001 and M002: each name a word, declared once in its place, and the
/// name of each record and enum once in the model.
fn names(model: &Model) -> Vec<Error> {
    let (mut errors, mut declared, mut records) = (Vec::new(), Vec::new(), BTreeSet::new());
    for part in &model.parts {
        let at = Path::to(&[&part.name]);
        declared.push(at.clone());
        for e in &part.enums {
            let here = at.then(own(&e.name));
            if !records.insert(&e.name) {
                errors.push(twice(&here));
            }
            declared.push(here.clone());
            errors.extend(listed(
                &here,
                e.variants.iter().map(|(v, _)| (v, &Type::Truth)),
            ));
            errors.extend(listed(
                &here,
                e.params.iter().map(|(n, _)| (n, &Type::Truth)),
            ));
        }
        for r in &part.records {
            let here = at.then(own(&r.name));
            if !records.insert(&r.name) {
                errors.push(twice(&here));
            }
            declared.push(here.clone());
            declared.extend(said_at(&here, &r.claims, &r.rules));
            errors.extend(listed(
                &here,
                r.fields.iter().flatten().map(|(n, t)| (n, t)),
            ));
        }
        for f in &part.functions {
            let here = at.then(&f.name);
            declared.push(here.clone());
            declared.extend(said_at(&here, &f.claims, &f.rules));
            errors.extend(listed(
                &here,
                f.takes.iter().chain([&f.gives]).map(|(n, t)| (n, t)),
            ));
        }
        declared.extend(part.requires.iter().map(|q| at.then(&q.name)));
        declared.extend(part.ensures.iter().map(|e| at.then(&e.name)));
        // An impl is declared in the function it is of, beside its claims.
        declared.extend(
            part.implements
                .iter()
                .filter(|i| !i.functions.is_empty())
                .map(|i| impl_at(part, i)),
        );
    }
    // A contract is named as a part is: `Monotone::f` is a path.
    for c in &model.contracts {
        let here = Path::to(&[&c.name]);
        declared.push(here.clone());
        errors.extend(listed(
            &here,
            c.params.iter().map(|(n, _)| (n, &Type::Truth)),
        ));
        for f in &c.functions {
            declared.push(here.then(&f.name));
            errors.extend(listed(
                &here.then(&f.name),
                f.takes.iter().chain([&f.gives]).map(|(n, t)| (n, t)),
            ));
        }
        for law in &c.claims {
            declared.push(here.then(&law.name));
            errors.extend(listed(
                &here.then(&law.name),
                law.over.iter().map(|(n, t)| (n, t)),
            ));
        }
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
    // A nested part is named by its whole path: its own name is the word.
    let name = match at.names.len() {
        1 => own(name),
        _ => name,
    };
    let message = format!("`{name}` is not a word");
    // `..`, the fields a record does not state, is no name.
    (!word(name) && name != crate::model::REST).then(|| error(M001, at, message))
}

fn twice(at: &Path) -> Error {
    let name = at.names.last().map(String::as_str).unwrap_or_default();
    error(M002, at, format!("`{name}` is declared twice"))
}

/// M001 and M002 for each name of a list of names and types, read in
/// the item at `at`.
fn listed<'a>(at: &Path, names: impl Iterator<Item = (&'a String, &'a Type)>) -> Vec<Error> {
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
    typed_in(index, at, names, &[], errors)
}

/// `types`, in a contract with these parameters.
fn typed_in<'a>(
    index: &Index,
    at: &Path,
    names: impl Iterator<Item = &'a (String, Type)>,
    params: &[(String, Bound)],
    errors: &mut Vec<Error>,
) -> bool {
    let mut all = true;
    for (name, t) in names.filter(|(_, t)| !known_in(index, t, params)) {
        // `self` of a part with no record: M030 says it.
        if name == "self" && params.is_empty() {
            all = false;
            continue;
        }
        let message = match t.open() && params.is_empty() {
            true => format!("the type of `{name}` is a type parameter: only a contract, a record or an enum names one"),
            false => match std_named(t) {
                Some(n) => format!(
                    "the type of `{name}` names `{n}`, which no part of the model declares: it is in Mitra's standard model, `use {n} from \"std\";`"
                ),
                None => format!(
                    "the type of `{name}` names no record or enum of the model as declared"
                ),
            },
        };
        errors.push(error(M003, &at.then(name), message));
        all = false;
    }
    all
}

/// The first part of Mitra's standard model `t` names: for a message.
fn std_named(t: &Type) -> Option<String> {
    let name = match t {
        Type::Record(r) => r.name.clone(),
        Type::Enum(e) => e.name.clone(),
        Type::Seq(t) | Type::Encoded(t) => return std_named(t),
        _ => return None,
    };
    let declares = |text: &str| {
        crate::parse::parse(text).is_ok_and(|m| m.parts.iter().any(|p| p.name == name))
    };
    crate::parse::STD
        .iter()
        .any(|(_, text)| declares(text))
        .then_some(name)
}

/// `t` is a type: a truth, a number, a tuple of types, or the type of a
/// record of the model.
fn known(index: &Index, t: &Type) -> bool {
    known_in(index, t, &[])
}

/// `known`, or a parameter declared so in a contract with `params`.
fn known_in(index: &Index, t: &Type, params: &[(String, Bound)]) -> bool {
    match t {
        Type::Truth | Type::Int | Type::Real | Type::Char | Type::Byte => true,
        Type::Seq(t) | Type::Encoded(t) => known_in(index, t, params),
        Type::Param { name, bound } => params.iter().any(|(n, b)| n == name && b == bound),
        // A tuple: a record with no name, its fields `0`, `1`, …, or
        // each a label, no two alike: `(sum: integer, n: integer)`.
        Type::Record(r) if r.name.is_empty() => {
            let mut fields = r.fields.iter().enumerate();
            let labels: BTreeSet<&String> = r.fields.iter().map(|(n, _)| n).collect();
            match r.labelled() {
                true => {
                    labels.len() == r.fields.len()
                        && r.fields.len() > 1
                        && r.fields.iter().all(|(_, t)| known_in(index, t, params))
                }
                false => fields
                    .all(|(i, (name, t))| *name == i.to_string() && known_in(index, t, params)),
            }
        }
        // One that holds itself is M027's, not M003's.
        Type::Record(r) if holds_itself(t, &r.name, false) => true,
        Type::Enum(e) if holds_itself(t, &e.name, false) => true,
        // The fields a record with `..` does not state.
        Type::Record(r) if r.name == crate::model::REST => true,
        Type::Record(r) => {
            let declared = index.records.get(r.name.as_str());
            declared.is_some_and(|d| instance_of(&d.type_(), &d.params, &r.args, t, params))
                && r.fields.iter().all(|(_, t)| known_in(index, t, params))
        }
        Type::Enum(e) => {
            let declared = index.enums.get(e.name.as_str());
            declared.is_some_and(|d| instance_of(&d.type_(), &d.params, &e.args, t, params))
                && e.variants
                    .iter()
                    .flat_map(|(_, ts)| ts)
                    .all(|t| known_in(index, t, params))
        }
    }
}

/// `t`, below its top unless `here`, holds a record or an enum named
/// `name`: a type that holds itself, cut where the model reads it.
fn holds_itself(t: &Type, name: &str, here: bool) -> bool {
    match t {
        Type::Record(r) if here && r.name == name => true,
        Type::Enum(e) if here && e.name == name => true,
        Type::Record(r) => r.fields.iter().any(|(_, t)| holds_itself(t, name, true)),
        Type::Enum(e) => e
            .variants
            .iter()
            .flat_map(|(_, ts)| ts)
            .any(|t| holds_itself(t, name, true)),
        // A sequence of itself is a tree: not in this round (finding 3).
        Type::Seq(t) | Type::Encoded(t) => holds_itself(t, name, here),
        _ => false,
    }
}

/// `t` is the type `declared` declares, each of its parameters given the
/// type `args` gives it, in order, one its bound admits and known.
fn instance_of(
    declared: &Type,
    declared_params: &[(String, Bound)],
    args: &[Type],
    t: &Type,
    params: &[(String, Bound)],
) -> bool {
    if args.len() != declared_params.len() {
        return false;
    }
    let given: BTreeMap<String, Type> = declared_params
        .iter()
        .map(|(n, _)| n.clone())
        .zip(args.iter().cloned())
        .collect();
    declared_params
        .iter()
        .zip(args)
        .all(|((_, b), a)| b.admits(a) && (!a.open() || matches!(a, Type::Param { name, bound } if params.iter().any(|(n, c)| n == name && c == bound))))
        && declared.given(&given) == *t
}

/// Each record `e` makes is a tuple, or a record of the model with its
/// fields known, made with them.
fn made(index: &Index, e: &Expression<simulation::Op>) -> bool {
    made_in(index, e, &[])
}

/// `made`, in a contract with these parameters.
fn made_in(index: &Index, e: &Expression<simulation::Op>, params: &[(String, Bound)]) -> bool {
    let Expression::Apply(op, args) = e else {
        return true;
    };
    let fits = match op {
        simulation::Op::Make(r) => {
            let declared = index.records.get(r.name.as_str());
            (r.name.is_empty() || declared.is_some_and(|d| d.fields.is_some()))
                && known_in(index, &Type::Record(r.clone()), params)
        }
        simulation::Op::Variant(e, _) | simulation::Op::Match(e) => {
            known_in(index, &Type::Enum(e.clone()), params)
        }
        _ => true,
    };
    fits && args.iter().all(|arg| made_in(index, &arg.body, params))
}

/// Each record in `v` is one of the model, with its fields known: an
/// opaque record has no value.
pub(crate) fn value_made(index: &Index, v: &Value) -> bool {
    match v {
        Value::Record(r, values) => {
            let declared = index.records.get(r.name.as_str());
            let known = r.name.is_empty() || declared.is_some_and(|d| d.fields.is_some());
            known && values.iter().all(|v| value_made(index, v))
        }
        // Known in part: a declared record, of its fields; one not known
        // may be of an opaque record (finding 2).
        Value::Partial(r, values) => {
            let declared = index.records.get(r.name.as_str());
            declared.is_some_and(|d| d.fields.is_some())
                && values.iter().flatten().all(|v| value_made(index, v))
        }
        // A type the parser could not tell (`None`, `[]` given where no
        // such value goes) is the types' error, M015, not this one.
        Value::Variant(e, _, holds) => {
            index.enums.contains_key(e.name.as_str())
                && (untold(&Type::Enum(e.clone())) || known(index, &Type::Enum(e.clone())))
                && holds.iter().all(|v| value_made(index, v))
        }
        Value::Seq(t, holds) => {
            (untold(t) || known(index, t)) && holds.iter().all(|v| value_made(index, v))
        }
        Value::Truth(_) | Value::Int(_) | Value::Real(_) | Value::Text(_) | Value::Byte(_) => true,
    }
}

/// `t` names a parameter the parser could not tell, `?T`.
fn untold(t: &Type) -> bool {
    match t {
        Type::Param { name, .. } => name.starts_with('?'),
        Type::Record(r) => r.args.iter().any(untold),
        Type::Enum(e) => e.args.iter().any(untold),
        Type::Seq(t) | Type::Encoded(t) => untold(t),
        _ => false,
    }
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

/// The type of a record's values, its parameters open. An opaque one
/// has no field to read.
fn record_type(r: &Record) -> Type {
    r.type_()
}

/// A part that holds only its data: a record or an enum, perhaps with
/// claims, and nothing else. Any part may hold one, as a Rust module
/// holds its structs (finding 13, D3, to see in use).
pub(crate) fn data_only(part: &Part) -> bool {
    part.functions.is_empty()
        && part.uses.is_empty()
        && part.implements.is_empty()
        && part.requires.is_empty()
        && part.ensures.is_empty()
        && part.inner.is_empty()
}

/// Whether `part` holds a part that is more than its data: then it holds
/// only parts, requires, ensures and examples (M028, M029).
pub(crate) fn working(model: &Model, part: &Part) -> bool {
    part.inner.iter().any(|i| {
        model
            .parts
            .iter()
            .find(|p| &p.name == i)
            .is_some_and(|p| !data_only(p))
    })
}

/// M028 for a part holding parts that holds more than parts, requires,
/// ensures and examples; M030 for each `self` that is not the first of a
/// method of a part with its own record.
fn nesting(model: &Model, part: &Part, at: &Path) -> Vec<Error> {
    let mut errors = vec![];
    if working(model, part) {
        let mut held = vec![];
        if !part.records.is_empty() {
            held.push("a record");
        }
        if !part.enums.is_empty() {
            held.push("an enum");
        }
        if !part.functions.is_empty() {
            held.push("a function");
        }
        if !part.uses.is_empty() {
            held.push("a `use`");
        }
        if !part.implements.is_empty() {
            held.push("an `impl`");
        }
        if !held.is_empty() {
            let message = format!(
                "`{}` holds parts, and also {}: a part holding parts holds only them, requires, ensures and examples; move the rest into one of its parts",
                part.name,
                held.join(", ")
            );
            errors.push(error(M028, at, message));
        }
    }
    let own = part.records.iter().any(|r| r.name == part.name);
    for f in &part.functions {
        for (k, (n, _)) in f.takes.iter().enumerate() {
            if n != "self" {
                continue;
            }
            let why = match (k, own) {
                (0, true) => continue,
                (0, false) => format!(
                    "`{}` takes `self`, and `{}` has no record of its own: write `record {{ … }}` in it",
                    f.name, part.name
                ),
                _ => format!(
                    "`self` is taken by `{}` in place {k}: a method takes `self` first, and nothing else is so named",
                    f.name
                ),
            };
            errors.push(error(M030, &at.then(&f.name), why));
        }
    }
    errors
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
#[allow(clippy::too_many_arguments)]
fn said(
    index: &Index,
    standard: &Simulation,
    part: &Part,
    at: &Path,
    of: Of,
    known: bool,
    claims: &[Claim],
    rules: &[Rule],
    errors: &mut Vec<Error>,
) {
    let scope = scope(of);
    // M006, M004, M017 and M018 for an expression read as a claim, at `here`.
    let claim = |here: &Path, e, what: &str, errors: &mut Vec<Error>| {
        let read = reads(index, part, here, e);
        let calls = !read.iter().any(|e| e.code == M017);
        if known && calls && typed(standard, &scope, e) != Some(Type::Truth) {
            let why = Mistyped {
                index,
                part: Some(&part.name),
                contract: None,
                standard,
            }
            .explain(&scope, e, &Type::Truth);
            errors.push(error(M006, here, format!("{what} is not a truth{why}")));
        }
        if !made(index, e) {
            let message = format!("{what} makes a record other than as declared");
            errors.push(error(M004, here, message));
        }
        errors.extend(read);
    };
    for c in claims {
        // `by invariant n` of no `fold`: `n` has no last value (M033).
        let turns = matches!(
            &c.invariant,
            Some(crate::model::Invariant { turns: Some(_) })
        );
        if turns && std::ptr::eq(c.kept(), &c.holds) {
            continue;
        }
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
                Term::Prop(p) => errors.extend(correct_wrong(index, &here, &place, p)),
            }
        }
        if let (None, Some(step)) = (&rule.proof, rule.short) {
            errors.extend(short_wrong(index, &here, rule, step));
        } else if index.shows(&here, rule) == Some(false) {
            let message = format!("the proof does not show `{}`", rule.name);
            errors.push(error(M012, &here, message));
        }
    }
}

/// M032 when the step a rule's proof writes alone, `by tollens`, is
/// filled in by no proof of its premises, or by two.
fn short_wrong(index: &Index, here: &Path, rule: &Rule, step: Step) -> Option<Error> {
    let fills = index.fills(here, rule)?;
    let w = step.word();
    let full = match step {
        Step::Ponens => "`by ponens[rule, post](…)`",
        Step::Tollens => "`by !tollens[rule, post, blame](…)`",
        Step::Cases => "`by cases[all x in …: p](…)`",
        Step::Intro => "`by intro[all x in …: p](|x| …)`",
        Step::Reductio => "`by reductio[p](…)`",
        Step::Instance => "`by instance[all x in …: p](…, point(…))`",
        Step::Witness => "`by witness[any x in …: p](…, point(…))`",
    };
    let message = match fills.as_slice() {
        [_] => return None,
        [] => format!(
            "no `{w}` of the premises shows `{}`: write the proof in full, {full}",
            rule.name
        ),
        [..] => {
            let ways: Vec<String> = fills
                .iter()
                .map(|p| format!("`by {}`", crate::print::print_proof(p)))
                .collect();
            format!(
                "`by {w}` shows `{}` {} ways: {}; write the one you mean",
                rule.name,
                ways.len(),
                ways.join(", ")
            )
        }
    };
    Some(error(M032, here, message))
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
                    // One proposition, whichever rule writes it.
                    Term::Prop(p) => Path {
                        names: vec![format!("{p:?}")],
                    },
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
            Item::Part | Item::Record | Item::Enum | Item::Function | Item::Claim => {}
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

/// Where an impl is declared: in the first function it names, beside
/// its claims, `stone::speed::Monotone`; in its part when it names none.
fn impl_at(part: &Part, i: &Implements) -> Path {
    let at = Path::to(&[&part.name]);
    match i.functions.first() {
        Some(f) => at.then(f).then(&i.contract),
        None => at.then(&i.contract),
    }
}

/// M016: each use names another part, and functions or records of it,
/// each once in the part (Rust: E0252).
fn uses(index: &Index, part: &Part) -> Vec<Error> {
    let at = Path::to(&[&part.name]);
    let mut errors = Vec::new();
    let mut used = BTreeSet::new();
    for u in &part.uses {
        let of = u.part();
        let there = Path::to(&[of]);
        if of == part.name || !matches!(index.items.get(&there), Some(Item::Part)) {
            let message = format!("`use {of}` names no other part");
            errors.push(error(M016, &at, message));
            continue;
        }
        if let Use::Names { names, .. } = u {
            // `use Reply;`: the whole part.
            if names.len() == 1 && names[0] == of {
                if !used.insert((of, of)) {
                    let message = format!("`{of}` is used twice");
                    errors.push(error(M016, &at, message));
                }
                continue;
            }
            if names.is_empty() {
                let message = format!("`use {of}::{{}}` names nothing of `{of}`");
                errors.push(error(M016, &at, message));
            }
            for name in names {
                let item = index.items.get(&there.then(name));
                if !matches!(item, Some(Item::Function | Item::Record | Item::Enum)) {
                    let message =
                        format!("`{of}::{name}` is no function, record or enum of `{of}`");
                    errors.push(error(M016, &at, message));
                } else if !used.insert((of, name)) {
                    let message = format!("`{of}::{name}` is used twice");
                    errors.push(error(M016, &at, message));
                }
            }
        }
    }
    errors
}

/// `part` reads `name` of the part `of`: its own, or through a use.
fn visible(part: &Part, of: &str, name: &str) -> bool {
    of == part.name || part.uses.iter().any(|u| u.reads(of, name))
}

/// M018 for each record a type names that is of another part, not used,
/// read at `at`.
fn unused_records(index: &Index, part: &Part, at: &Path, t: &Type) -> Vec<Error> {
    let mut errors = Vec::new();
    let mut todo = vec![t];
    while let Some(t) = todo.pop() {
        if let Type::Record(r) = t {
            // A part's own type is named by the part: seen everywhere.
            if let Some(of) = index
                .record_parts
                .get(r.name.as_str())
                .filter(|of| **of != r.name)
            {
                if !visible(part, of, &r.name) {
                    let message = format!(
                        "`{of}::{}` is of another part: `use {of}::{};`",
                        r.name, r.name
                    );
                    errors.push(error(M018, at, message));
                }
            }
            todo.extend(r.fields.iter().map(|(_, t)| t));
        }
        if let Type::Enum(e) = t {
            if let Some(of) = index
                .enum_parts
                .get(e.name.as_str())
                .filter(|of| **of != e.name)
            {
                if !visible(part, of, &e.name) {
                    let message = format!(
                        "`{of}::{}` is of another part: `use {of}::{};`",
                        e.name, e.name
                    );
                    errors.push(error(M018, at, message));
                }
            }
            todo.extend(e.variants.iter().flat_map(|(_, ts)| ts));
        }
        if let Type::Seq(t) | Type::Encoded(t) = t {
            todo.push(t);
        }
    }
    errors
}

/// M017 and M018 for the calls and records of `e`, at `at` in `part`:
/// each call names a function of the model, and each function and record
/// of another part is used.
fn reads(index: &Index, part: &Part, at: &Path, e: &Expression<simulation::Op>) -> Vec<Error> {
    let mut errors = Vec::new();
    let mut todo = vec![e];
    while let Some(e) = todo.pop() {
        let Expression::Apply(op, args) = e else {
            continue;
        };
        match op {
            simulation::Op::Call(f) => match index.functions.get(f) {
                None => errors.push(error(M017, at, no_function(index, part, f))),
                Some(function) => {
                    let (of, name) = f.rsplit_once("::").unwrap_or_default();
                    // A method is seen wherever its part's type is.
                    let method = function.takes.first().is_some_and(|(n, _)| n == "self");
                    if !method && !visible(part, of, name) {
                        let message = format!("`{f}` is of another part: `use {f};`");
                        errors.push(error(M018, at, message));
                    }
                }
            },
            simulation::Op::Make(r) => {
                errors.extend(unused_records(index, part, at, &Type::Record(r.clone())))
            }
            simulation::Op::Variant(e, _) | simulation::Op::Match(e) => {
                errors.extend(unused_records(index, part, at, &Type::Enum(e.clone())))
            }
            _ => {}
        }
        todo.extend(args.iter().map(|a| &a.body));
    }
    errors
}

/// Why the call `f` in `part` names no function: none of that name, or,
/// for a name alone, several its uses read.
fn no_function(index: &Index, part: &Part, f: &str) -> String {
    let read: Vec<String> = part
        .uses
        .iter()
        .map(|u| format!("{}::{f}", u.part()))
        .filter(|g| {
            let (of, name) = g.rsplit_once("::").unwrap_or_default();
            index.functions.contains_key(g) && part.uses.iter().any(|u| u.reads(of, name))
        })
        .collect::<BTreeSet<String>>()
        .into_iter()
        .collect();
    match read.as_slice() {
        [_, _, ..] if !f.contains("::") => format!(
            "`{f}` names several functions its uses read, `{}`: write which",
            read.join("`, `")
        ),
        _ => format!("`{f}` names no function of the model"),
    }
}

/// M031 for each claim `by induction` on a name that is not an input of
/// its function that is a tree: an enum that holds itself.
fn inductions(model: &Model) -> Vec<Error> {
    let mut errors = Vec::new();
    for part in &model.parts {
        for r in &part.records {
            for c in r.claims.iter().filter(|c| c.induction.is_some()) {
                let at = Path::to(&[&part.name, &r.name, &c.name]);
                let message = "a claim of a record is not shown `by induction`: only a function's, on an input that is a tree".to_string();
                errors.push(error(M031, &at, message));
            }
        }
        for f in &part.functions {
            for c in &f.claims {
                let Some(t) = &c.induction else { continue };
                let at = Path::to(&[&part.name, &f.name, &c.name]);
                let tree = f.takes.iter().find(|(n, _)| n == t).map(|(_, ty)| ty);
                let place = f.takes.iter().position(|(n, _)| n == t);
                let path = Path::to(&[&part.name, &f.name]).to_string();
                let on = |k: usize| {
                    f.body
                        .as_ref()
                        .is_none_or(|b| simulation::decreasing(&f.takes, b, &path).contains(&k))
                };
                let message = match tree {
                    None => format!("`{t}` is no input of `{}`: `by induction` names the input the claim is shown on", f.name),
                    Some(ty @ Type::Enum(e)) if holds_itself(ty, &e.name, false) && place.is_some_and(on) => continue,
                    Some(Type::Enum(e)) if holds_itself(&Type::Enum(e.clone()), &e.name, false) => format!(
                        "`{}` calls itself on what is not a part of `{t}`: the claim is shown on the input every call takes apart",
                        f.name
                    ),
                    Some(ty) => format!(
                        "`{t}` is {}, not a tree: `by induction` is on an input whose enum holds itself",
                        crate::print::type_(ty)
                    ),
                };
                errors.push(error(M031, &at, message));
            }
        }
    }
    errors
}

/// M033 for each claim `by invariant` of a function whose body is no
/// loop, `by invariant n` of an `until`, which has no count to end at,
/// or with `n` a name the function or its loop already has.
fn invariants(model: &Model) -> Vec<Error> {
    let mut errors = Vec::new();
    for part in &model.parts {
        for r in &part.records {
            for c in r.claims.iter().filter(|c| c.invariant.is_some()) {
                let at = Path::to(&[&part.name, &r.name, &c.name]);
                let message = "a claim of a record is not shown `by invariant`: only a function's whose body is a loop".to_string();
                errors.push(error(M033, &at, message));
            }
        }
        for f in &part.functions {
            for c in &f.claims {
                let Some(invariant) = &c.invariant else {
                    continue;
                };
                let at = Path::to(&[&part.name, &f.name, &c.name]);
                let (fold, binds) = match &f.body {
                    Some(Expression::Apply(simulation::Op::Fold, args)) => (true, &args[2].binds),
                    Some(Expression::Apply(simulation::Op::Until, args)) => (false, &args[2].binds),
                    _ => {
                        let message = format!(
                            "the body of `{}` is no loop: `by invariant` is on a function whose body is `xs.fold(…) {{ … }}` or `until(…) {{ … }}`",
                            f.name
                        );
                        errors.push(error(M033, &at, message));
                        continue;
                    }
                };
                let Some(n) = &invariant.turns else { continue };
                let message = if !fold {
                    format!("`by invariant {n}` counts the turns of a `fold`; an `until` has no count to end at: write `by invariant`")
                } else if f.takes.iter().chain([&f.gives]).any(|(name, _)| name == n)
                    || binds.contains(n)
                {
                    format!("`{n}` is already a name of `{}` or of its loop: the turns take a name of their own", f.name)
                } else {
                    continue;
                };
                errors.push(error(M033, &at, message));
            }
        }
    }
    errors
}

/// M019 for each function on a cycle of calls, through bodies.
fn calls_itself(model: &Model) -> Vec<Error> {
    let mut paths: Vec<Path> = Vec::new();
    let mut bodies = Vec::new();
    let mut functions = Vec::new();
    for part in &model.parts {
        for f in &part.functions {
            paths.push(Path::to(&[&part.name, &f.name]));
            bodies.push(f.body.as_ref());
            functions.push(f);
        }
    }
    let number: BTreeMap<String, usize> = paths
        .iter()
        .enumerate()
        .rev()
        .map(|(i, p)| (p.to_string(), i))
        .collect();
    let mut out = vec![Vec::new(); paths.len()];
    for (i, body) in bodies.iter().enumerate() {
        let mut todo: Vec<&Expression<simulation::Op>> = body.iter().copied().collect();
        while let Some(e) = todo.pop() {
            if let Expression::Apply(op, args) = e {
                if let simulation::Op::Call(f) = op {
                    out[i].extend(number.get(f));
                }
                todo.extend(args.iter().map(|a| &a.body));
            }
        }
    }
    let component = components(&out);
    let mut errors = Vec::new();
    for (i, at) in paths.iter().enumerate() {
        let shared = component.iter().filter(|&&c| c == component[i]).count() > 1;
        let own = out[i].contains(&i)
            && !functions[i]
                .body
                .as_ref()
                .is_some_and(|b| simulation::structural(&functions[i].takes, b, &at.to_string()));
        if shared || own {
            let message = match shared {
                true => format!("`{at}` calls itself, through its body"),
                false => format!(
                    "`{at}` calls itself on what is not a part of its input taken apart by a `match`: a call on `l` of `match t {{ Node(l, …) => … }}` ends, another may not"
                ),
            };
            errors.push(error(M019, at, message));
        }
    }
    errors
}

/// A contract, checked once: its types known with its parameters, each
/// claim a truth over its arguments, calling only the contract's
/// functions (M003, M006, M004, M017).
fn contract_errors(index: &Index, c: &Contract) -> Vec<Error> {
    let here = Path::to(&[&c.name]);
    let mut errors = Vec::new();
    let mut known = true;
    for f in &c.functions {
        let names = f.takes.iter().chain([&f.gives]);
        known &= typed_in(index, &here.then(&f.name), names, &c.params, &mut errors);
    }
    let standard = Simulation {
        calls: Arc::new(contract_calls(c)),
    };
    for law in &c.claims {
        let at = here.then(&law.name);
        let ok = typed_in(index, &at, law.over.iter(), &c.params, &mut errors);
        let scope: BTreeMap<String, Type> = law.over.iter().cloned().collect();
        let mut calls = true;
        let mut todo = vec![&law.holds];
        while let Some(e) = todo.pop() {
            if let Expression::Apply(op, args) = e {
                if let simulation::Op::Call(f) = op {
                    if standard.calls.get(f).is_none() {
                        let message = format!("`{f}` is no function of `{}`", c.name);
                        errors.push(error(M017, &at, message));
                        calls = false;
                    }
                }
                todo.extend(args.iter().map(|a| &a.body));
            }
        }
        if known && ok && calls && typed(&standard, &scope, &law.holds) != Some(Type::Truth) {
            let why = Mistyped {
                index,
                part: None,
                contract: Some(&c.name),
                standard: &standard,
            }
            .explain(&scope, &law.holds, &Type::Truth);
            errors.push(error(
                M006,
                &at,
                format!("`{}` is not a truth{why}", law.name),
            ));
        }
        if !made_in(index, &law.holds, &c.params) {
            let message = format!("`{}` makes a record other than as declared", law.name);
            errors.push(error(M004, &at, message));
        }
    }
    errors
}

/// Why `i`, in `part`, does not fit its contract (M020), if it does not:
/// a type for each parameter, within its bound, and a function of the
/// part for each of the contract's, of its types.
pub(crate) fn misfit(index: &Index, part: &Part, i: &Implements) -> Option<String> {
    let Some(c) = index.contracts.get(i.contract.as_str()) else {
        return Some(format!("`{}` names no contract", i.contract));
    };
    if i.functions.is_empty() {
        return Some(format!(
            "`for ()` names no function: the claims of `{}` are read of the functions an impl names",
            c.name
        ));
    }
    if i.types.len() != c.params.len() {
        return Some(format!(
            "`{}` takes {} types, given {}",
            c.name,
            c.params.len(),
            i.types.len()
        ));
    }
    for ((name, bound), t) in c.params.iter().zip(&i.types) {
        if !known(index, t) || !bound.admits(t) {
            let wanted = match bound {
                Bound::Number => format!("`{name}: Number`"),
                Bound::Any => format!("`{name}`"),
            };
            return Some(format!(
                "`{}` cannot be {wanted} of `{}`",
                crate::print::type_(t),
                c.name
            ));
        }
    }
    if i.functions.len() != c.functions.len() {
        let message = format!(
            "`{}` has {} functions, given {}",
            c.name,
            c.functions.len(),
            i.functions.len()
        );
        return Some(message);
    }
    let given: BTreeMap<String, Type> = c
        .params
        .iter()
        .map(|(n, _)| n.clone())
        .zip(i.types.iter().cloned())
        .collect();
    for (wanted, name) in c.functions.iter().zip(&i.functions) {
        let Some(f) = part.functions.iter().find(|f| &f.name == name) else {
            return Some(format!("`{name}` is no function of `{}`", part.name));
        };
        let takes: Vec<Type> = wanted.takes.iter().map(|(_, t)| t.given(&given)).collect();
        let fits = f.takes.len() == takes.len()
            && f.takes.iter().zip(&takes).all(|((_, a), b)| a == b)
            && f.gives.1 == wanted.gives.1.given(&given);
        if !fits {
            return Some(format!(
                "`{name}` does not take and give what `{}::{}` does",
                c.name, wanted.name
            ));
        }
    }
    None
}

/// M021 and M022 for the example of the model at `i`: each input what a
/// function takes, or what one with no body gives; each output what a
/// function gives; each once, of its type; and each argument of a
/// function an output or an input reads given.
/// The range of what `f` takes or gives named `n`, if it has one.
fn range_of<'f>(f: &'f Function, n: &str) -> Option<&'f simulation::Range> {
    match &f.ranges.gives {
        Some((q, range)) if q == n => Some(range),
        _ => f.ranges.takes.get(n),
    }
}

/// M025: a range type that holds no number, `9..=0`.
fn empty_ranges(f: &Function, here: &Path) -> Vec<Error> {
    let all = f
        .ranges
        .takes
        .iter()
        .chain(f.ranges.gives.iter().map(|(q, r)| (q, r)));
    all.filter(|(_, r)| r.low > r.high)
        .map(|(n, r)| {
            let message = format!(
                "`{n}: {r}` holds no number: a range is written low end first, `{}..={}`",
                r.high, r.low
            );
            error(M025, here, message)
        })
        .collect()
}

fn model_example(index: &Index, i: usize, example: &crate::model::ModelExample) -> Vec<Error> {
    let at = Path::to(&["examples", &i.to_string()]);
    let mut errors = Vec::new();
    // `when f(x)` left as written: `f` is no function taking as many.
    let named = |f: &Path, n: usize| {
        index
            .functions
            .get(f.to_string().as_str())
            .is_some_and(|g| g.takes.len() == n)
    };
    if let Some((f, args, _)) = example.when.as_ref().filter(|(f, a, _)| !named(f, a.len())) {
        let message = format!(
            "`when {f}{}`: `{f}` is no function of the model taking {} arguments",
            crate::print::values(args),
            args.len()
        );
        errors.push(error(M021, &at, message));
        return errors;
    }
    for message in given_wrong(index, &example.given) {
        errors.push(error(M022, &at, message));
    }
    for message in effects_wrong(index, &example.effects) {
        errors.push(error(M022, &at, message));
    }
    // The function of a port, and the type of the port; whether it is
    // what the function gives.
    let port = |p: &Path| -> Option<(&Function, Path, Type, bool)> {
        let [part, f, name] = p.names.as_slice() else {
            return None;
        };
        let function = *index.functions.get(&format!("{part}::{f}"))?;
        let takes = function.takes.iter().find(|(n, _)| n == name);
        match takes {
            Some((_, t)) => Some((function, Path::to(&[part, f]), t.clone(), false)),
            None if &function.gives.0 == name => Some((
                function,
                Path::to(&[part, f]),
                function.gives.1.clone(),
                true,
            )),
            None => None,
        }
    };
    if example.outputs.is_empty() && example.effects.0.is_empty() {
        let message =
            "it states no output: an example of the model says what the model gives, or calls";
        errors.push(error(M021, &at, message.to_string()));
    }
    // What the model is given is not what it gives: a value on both
    // sides would be its own expected value.
    let outputs: BTreeSet<&Path> = example.outputs.iter().map(|(p, _)| p).collect();
    for (p, _) in &example.inputs {
        if outputs.contains(p) {
            let message =
                format!("`{p}` is both an input and an output: it would be its own expected value");
            errors.push(error(M021, &at, message));
        }
    }
    let mut given = BTreeSet::new();
    let mut needs: Vec<(Path, &Function)> = Vec::new();
    for (side, ports) in [("input", &example.inputs), ("output", &example.outputs)] {
        let mut seen = BTreeSet::new();
        for (p, v) in ports.iter() {
            let Some((function, f, t, gives)) = port(p) else {
                errors.push(error(
                    M021,
                    &at,
                    format!("`{p}` is nothing a function takes or gives"),
                ));
                continue;
            };
            if !seen.insert(p.clone()) {
                errors.push(error(M021, &at, format!("`{p}` is given twice")));
            }
            match (side, gives) {
                ("input", true) if function.body.is_some() => {
                    let message = format!("`{p}` is what `{f}` gives, and the model computes it: an output, not an input");
                    errors.push(error(M021, &at, message));
                }
                ("output", false) => {
                    let message =
                        format!("`{p}` is what `{f}` takes: an output is what a function gives");
                    errors.push(error(M021, &at, message));
                }
                _ => {}
            }
            if side == "input" {
                given.insert(p.clone());
            }
            // What the model must compute: an output, or a value the
            // world gave, at the point its function was given.
            if gives && (side == "output" || function.body.is_none()) {
                needs.push((f, function));
            }
            if known(index, &t) && !Exact::default().has_type(v, &t) {
                errors.push(error(
                    M022,
                    &at,
                    format!("the value of `{p}` is not of its type"),
                ));
            }
            let name = p.names.last().map(String::as_str).unwrap_or_default();
            if let Some(range) = range_of(function, name) {
                if matches!(v, Value::Int(_)) && !range.holds(v) {
                    let message = format!(
                        "`{p}` = {}, outside its range {range}",
                        crate::print::value(v)
                    );
                    errors.push(error(M026, &at, message));
                }
            }
            if !value_made(index, v) {
                errors.push(error(
                    M004,
                    &at,
                    format!("the value of `{p}` makes a record other than as declared"),
                ));
            }
        }
    }
    for (f, function) in needs {
        for (name, _) in &function.takes {
            if !given.contains(&f.then(name)) {
                errors.push(error(M022, &at, format!("`{f}::{name}` is not given")));
            }
        }
    }
    errors
}

/// M023 and M008 for a proposition written in the rule at `at`: where
/// functions of its part are correct, at points of their arguments, or
/// at the point an all or an any around binds, over a set of their
/// kinds.
fn correct_wrong(index: &Index, at: &Path, place: &str, p: &Prop) -> Vec<Error> {
    let mut errors = Vec::new();
    let wrong = |why: String| error(M023, at, format!("{place}: {why}"));
    // A set's condition calls only functions with a body, and they only
    // such (finding 12b): the model's function is the definition.
    for over in sets_of(p) {
        if let Some((called, missing)) = over.such.as_ref().and_then(|e| bodiless(index, e)) {
            let how = match called == missing {
                true => format!("`{called}` has no body"),
                false => format!("`{called}` calls `{missing}`, which has no body"),
            };
            errors.push(wrong(format!(
                "the condition of a set calls only functions with a body, read as their definition: {how}; write the bound by hand, or give it a body"
            )));
            return errors;
        }
    }
    if !p.well_formed() {
        errors.push(wrong(
            "a name no all or any binds, or a set that is not well formed".to_string(),
        ));
        return errors;
    }
    // Each proposition with the sets of the names bound around it.
    let mut todo: Vec<(&Prop, Vec<(&str, &judgement::Set)>)> = vec![(p, vec![])];
    while let Some((p, bound)) = todo.pop() {
        match p {
            Prop::Not(x) => todo.push((x, bound)),
            Prop::All { name, over, body } | Prop::Any { name, over, body } => {
                errors.extend(empty_set(at, place, over));
                let mut bound = bound.clone();
                bound.push((name, over));
                todo.push((body, bound));
            }
            Prop::Correct { function, at: term } => {
                let Some(f) = index.functions.get(function) else {
                    errors.push(wrong(format!(
                        "`{function}` names no function of the model"
                    )));
                    continue;
                };
                if function.rsplit_once("::").map(|(p, _)| p)
                    != at.names.first().map(String::as_str)
                {
                    let message = format!(
                        "`{function}` is of another part: parts meet only through requires"
                    );
                    errors.push(error(M008, at, message));
                }
                if let judgement::Term::Point(values)
                | judgement::Term::Given { point: values, .. } = term
                {
                    if !values.iter().all(|v| value_made(index, v)) {
                        let message =
                            format!("{place}: a point makes a record other than as declared");
                        errors.push(error(M004, at, message));
                    }
                }
                // A world: what functions with no body of the model gave,
                // each called as it takes.
                if let judgement::Term::Given { world, .. } = term {
                    for (g, args, v) in &world.0 {
                        let fits = index.functions.get(g.as_str()).is_some_and(|g| {
                            g.body.is_none()
                                && args.len() == g.takes.len()
                                && args
                                    .iter()
                                    .zip(&g.takes)
                                    .all(|(a, (_, t))| Exact::default().has_type(a, t))
                                && Exact::default().has_type(v, &g.gives.1)
                        });
                        if !fits {
                            errors.push(wrong(format!(
                                "`{g}` in the world of a point is no function with no body of the model, called and giving as it takes and gives"
                            )));
                        }
                    }
                }
                let fits = match term {
                    judgement::Term::Point(values)
                    | judgement::Term::Given { point: values, .. } => {
                        values.len() == f.takes.len()
                            && values
                                .iter()
                                .zip(&f.takes)
                                .all(|(v, (_, t))| Exact::default().has_type(v, t))
                    }
                    judgement::Term::Var(x) => {
                        let set = bound.iter().rev().find(|(n, _)| n == x).map(|(_, s)| s);
                        set.is_some_and(|s| {
                            s.ranges.len() == f.takes.len()
                                && s.ranges
                                    .iter()
                                    .zip(&f.takes)
                                    .all(|((_, lo, _), (_, t))| Exact::default().has_type(lo, t))
                        })
                    }
                    judgement::Term::Arbitrary { .. } => false,
                };
                if !fits {
                    errors.push(wrong(format!(
                        "`{function}` is not said correct at a point of what it takes"
                    )));
                }
            }
            Prop::Named(_) | Prop::Implies { .. } => {
                errors.push(wrong(
                    "only `correct`, `all`, `any` and `!` are written in place".to_string(),
                ));
            }
        }
    }
    errors
}

/// Every set a proposition names: of each all and any, and of each
/// point an intro binds.
fn sets_of(p: &Prop) -> Vec<&judgement::Set> {
    let mut sets = Vec::new();
    let mut todo = vec![p];
    while let Some(p) = todo.pop() {
        match p {
            Prop::Named(_) => {}
            Prop::Not(x) => todo.push(x),
            Prop::Implies { pre, post, .. } => todo.extend(pre.iter().chain(post)),
            Prop::Correct { at, .. } => {
                if let judgement::Term::Arbitrary { over, .. } = at {
                    sets.push(over);
                }
            }
            Prop::All { over, body, .. } | Prop::Any { over, body, .. } => {
                sets.push(over);
                todo.push(body);
            }
        }
    }
    sets
}

/// The first function `e` calls, and the one with no body it reaches
/// through calls, if any: the same when it has none itself.
fn bodiless(index: &Index, e: &Expression<simulation::Op>) -> Option<(String, String)> {
    fn calls(e: &Expression<simulation::Op>, out: &mut Vec<String>) {
        if let Expression::Apply(op, args) = e {
            if let simulation::Op::Call(f) = op {
                out.push(f.clone());
            }
            for a in args {
                calls(&a.body, out);
            }
        }
    }
    let mut first = Vec::new();
    calls(e, &mut first);
    for called in first {
        let mut seen = BTreeSet::new();
        let mut todo = vec![called.clone()];
        while let Some(f) = todo.pop() {
            if !seen.insert(f.clone()) {
                continue;
            }
            match index.functions.get(&f).and_then(|g| g.body.as_ref()) {
                Some(body) => calls(body, &mut todo),
                None => return Some((called, f)),
            }
        }
    }
    None
}

/// Each row of a `given` that is not what a function with no body of the
/// model gives, by its whole path, as it takes and gives.
fn given_wrong(index: &Index, given: &crate::model::Given) -> Vec<String> {
    let mut wrong = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for (g, args, v) in &given.0 {
        // One answer for each call: a second for the same arguments
        // would leave which one the run read to the order.
        if !seen.insert((g, args)) {
            let row = crate::print::givens(&crate::model::Given(vec![(
                g.clone(),
                args.clone(),
                v.clone(),
            )]));
            let row = row.trim_start().trim_start_matches("given ");
            let call = row.split(" -> ").next().unwrap_or(row);
            wrong.push(format!(
                "`{call}` is given twice in one row: a run reads one answer for each call"
            ));
            continue;
        }
        let fits = index.functions.get(g.as_str()).is_some_and(|g| {
            g.body.is_none()
                && args.len() == g.takes.len()
                && args
                    .iter()
                    .zip(&g.takes)
                    .all(|(a, (_, t))| a.as_ref().is_none_or(|a| Exact::default().has_type(a, t)))
                && Exact::default().has_type(v, &g.gives.1)
        });
        if !fits {
            let row = crate::print::givens(&crate::model::Given(vec![(
                g.clone(),
                args.clone(),
                v.clone(),
            )]));
            wrong.push(format!(
                "`{}` in `given` is not what a function with no body of the model gives, by its whole path, as it takes and gives",
                row.trim_start().trim_start_matches("given ")
            ));
        }
    }
    wrong
}

/// Each call a `then` lists that is not one to a function with no body
/// of the model, by its whole path, as it takes (BDD step 2).
fn effects_wrong(index: &Index, effects: &crate::model::Effects) -> Vec<String> {
    let mut wrong = Vec::new();
    for (f, args) in &effects.0 {
        let fits = index.functions.get(f.as_str()).is_some_and(|g| {
            g.body.is_none()
                && args.len() == g.takes.len()
                && args
                    .iter()
                    .zip(&g.takes)
                    .all(|(a, (_, t))| a.as_ref().is_none_or(|a| Exact::default().has_type(a, t)))
        });
        if !fits {
            wrong.push(format!(
                "`{}` in `then` is not a call to a function with no body of the model, by its whole path, as it takes",
                crate::print::effects(&crate::model::Effects(vec![(f.clone(), args.clone())]))
            ));
        }
    }
    wrong
}

/// M034: a range of a set whose ends leave no number between them,
/// `1..0` or `3..3`, as Clippy's `reversed_empty_ranges` says: the all
/// over it would hold of nothing, and the any be false of nothing.
fn empty_set(at: &Path, place: &str, over: &judgement::Set) -> Vec<Error> {
    use crate::print::value;
    use simulation::Value as V;
    let mut errors = Vec::new();
    for (n, lo, hi) in &over.ranges {
        let written = format!("{n}: {}..{}", value(lo), value(hi));
        let why = match (lo, hi) {
            _ if lo == hi => match lo {
                V::Int(l) => format!(
                    "a range leaves its high end out, as in Rust: did you mean `{n}: {l}..{}`, the one point {l}?",
                    l + 1
                ),
                _ => "a range leaves its high end out, as in Rust".to_string(),
            },
            (V::Int(l), V::Int(h)) if l > h => format!(
                "a range is written low end first: did you mean `{n}: {h}..{l}`?"
            ),
            (V::Real(l), V::Real(h)) if l > h => format!(
                "a range is written low end first: did you mean `{n}: {}..{}`?",
                value(hi),
                value(lo)
            ),
            _ => continue,
        };
        let message = format!("{place}: `{written}` holds no point: {why}");
        errors.push(error(M034, at, message));
    }
    errors
}

/// Why an expression is not of the type it should be, as rustc says it:
/// the expression, the innermost part that has no type underlined with
/// the type of each of its arguments, what the operation takes, and how
/// to write it when a number is the cause.
struct Mistyped<'a> {
    index: &'a Index<'a>,
    part: Option<&'a str>,
    contract: Option<&'a str>,
    standard: &'a Simulation,
}

type Sim = Expression<simulation::Op>;

impl Mistyped<'_> {
    fn show(&self, e: &Sim) -> String {
        crate::print::sim_from(&self.index.calling, self.part, self.contract, e)
    }

    /// The lines after the first of an error's message: empty when `e`
    /// has the type `want`.
    fn explain(&self, scope: &BTreeMap<String, Type>, e: &Sim, want: &Type) -> String {
        use crate::print::type_;
        let whole = self.show(e);
        let (part, label, notes) = match self.innermost(scope, e) {
            Some((part, takes)) => {
                let Expression::Apply(op, args) = part else {
                    return String::new();
                };
                let wanted = self.wanted(op);
                let each: Vec<String> = match &wanted {
                    // A call or a record: each argument that does not fit,
                    // against what it should be.
                    Some(want) => args
                        .iter()
                        .zip(&takes)
                        .zip(want)
                        .filter(|((_, t), (_, w))| *t != w)
                        .map(|((a, t), (n, w))| {
                            let (a, t, w) = (self.show(&a.body), type_(t), type_(w));
                            format!("`{a}` is {t} where `{n}` is {w}")
                        })
                        .collect(),
                    None => args
                        .iter()
                        .zip(&takes)
                        .map(|(a, t)| format!("`{}` is {}", self.show(&a.body), type_(t)))
                        .collect(),
                };
                let label = match each.split_last() {
                    Some((last, [])) => last.clone(),
                    Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
                    None if wanted.is_some_and(|w| w.len() != args.len()) => {
                        format!("{} arguments", args.len())
                    }
                    None => String::new(),
                };
                (part, label, self.notes(op, args, &takes))
            }
            None => match typed(self.standard, scope, e) {
                Some(t) if &t != want => {
                    let label = format!("this is {}", type_(&t));
                    let note = format!("note: it should be {}", type_(want));
                    (e, label, vec![note])
                }
                _ => return String::new(),
            },
        };
        let shown = self.show(part);
        let column = whole.find(&shown).map_or(0, |b| whole[..b].chars().count());
        let width = shown.chars().count().max(1);
        let mut out = format!(
            "\n  |\n  | {whole}\n  | {}{} {label}",
            " ".repeat(column),
            "^".repeat(width)
        );
        for note in notes {
            out.push_str(&format!("\n  = {note}"));
        }
        out
    }

    /// The innermost part of `e` whose arguments each have a type, but
    /// not ones its operation takes, with those types.
    fn innermost<'e>(
        &self,
        scope: &BTreeMap<String, Type>,
        e: &'e Sim,
    ) -> Option<(&'e Sim, Vec<Type>)> {
        use crate::expression::Standard;
        let Expression::Apply(op, args) = e else {
            return None;
        };
        let mut takes = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            let mut inner = scope.clone();
            let binds = self.standard.binds(op, i, &takes);
            inner.extend(arg.binds.iter().cloned().zip(binds));
            match typed(self.standard, &inner, &arg.body) {
                Some(t) => takes.push(t),
                None => return self.innermost(&inner, &arg.body),
            }
        }
        match self.standard.gives(op, &takes) {
            Some(_) => None,
            None => Some((e, takes)),
        }
    }

    /// What a call or a record takes, by name: `None` for any other
    /// operation.
    fn wanted(&self, op: &simulation::Op) -> Option<Vec<(String, Type)>> {
        match op {
            simulation::Op::Make(r) => Some(r.fields.clone()),
            simulation::Op::Variant(e, i) => e.variants.get(*i).map(|(_, ts)| {
                ts.iter()
                    .enumerate()
                    .map(|(k, t)| (k.to_string(), t.clone()))
                    .collect()
            }),
            simulation::Op::Call(f) => self.standard.calls.get(f).map(|c| c.takes.clone()),
            _ => None,
        }
    }

    /// What `op` takes, and a help when a whole number meets a `Real`
    /// or a type parameter.
    fn notes(
        &self,
        op: &simulation::Op,
        args: &[crate::expression::Arg<simulation::Op>],
        takes: &[Type],
    ) -> Vec<String> {
        use simulation::Op;
        const SAME: &str = "of one type: integer, rational, or one `T: Number`";
        let takes_what = match op {
            Op::Add => format!("`+` takes two numbers {SAME}"),
            Op::Sub => format!("`-` takes two numbers {SAME}"),
            Op::Mul => format!("`*` takes two numbers {SAME}"),
            Op::Div => "`/` takes two integer or two rational".to_string(),
            Op::Rem => "`%` takes two integer".to_string(),
            Op::Less | Op::AtMost
                if takes
                    .iter()
                    .any(|t| matches!(t, Type::Encoded(_) | Type::Char | Type::Byte)) =>
            {
                let symbol = if *op == Op::Less { "<" } else { "<=" };
                format!("`{symbol}` compares two texts byte by byte, or two `char`; a byte has `==` only")
            }
            Op::Less => format!("`<` compares two numbers {SAME}"),
            Op::AtMost => format!("`<=` compares two numbers {SAME}"),
            Op::Equal => "`==` compares two values of one type".to_string(),
            Op::Within => format!("`within` takes three numbers {SAME}"),
            Op::Not => "`!` takes a bool".to_string(),
            Op::And => "`&&` takes two bool".to_string(),
            Op::Or => "`||` takes two bool".to_string(),
            Op::If => "`if` takes a bool, then two branches of one type".to_string(),
            Op::All => "`all` takes two integer bounds and a bool".to_string(),
            Op::Field(name)
                if name == "len" && matches!(takes.first(), Some(Type::Seq(_) | Type::Encoded(_))) =>
            {
                "`.len` reads a field of a record that has one: the length of a sequence is `.len()`".to_string()
            }
            Op::Field(name) => format!("`.{name}` reads a field of a record that has one"),
            Op::Make(r) => {
                let fields: Vec<String> = r
                    .fields
                    .iter()
                    .map(|(n, t)| format!("{n}: {}", crate::print::type_(t)))
                    .collect();
                format!("`{}` takes {{ {} }}", r.name, fields.join(", "))
            }
            Op::Call(f) => match self.standard.calls.get(f) {
                Some(callee) => {
                    let takes: Vec<String> = callee
                        .takes
                        .iter()
                        .map(|(n, t)| format!("{n}: {}", crate::print::type_(t)))
                        .collect();
                    let name = self.show(&Expression::Apply(op.clone(), vec![]));
                    let name = name.trim_end_matches("()");
                    format!("`{name}` takes ({})", takes.join(", "))
                }
                None => return Vec::new(),
            },
            Op::Variant(e, i) => {
                let (name, ts) = &e.variants[(*i).min(e.variants.len().saturating_sub(1))];
                let ts: Vec<String> = ts.iter().map(crate::print::type_).collect();
                format!(
                    "`{name}` of `{}` holds ({})",
                    crate::print::type_(&Type::Enum(e.clone())),
                    ts.join(", ")
                )
            }
            Op::Match(e) => format!(
                "`match` takes a `{}`, then one arm for each of its variants, all of one type",
                crate::print::type_(&Type::Enum(e.clone()))
            ),
            Op::Seq(t) => format!(
                "`[…]` holds values of one type, here `{}`",
                crate::print::type_(t)
            ),
            Op::Len => "`.len()` reads a sequence, or a text in bytes".to_string(),
            Op::Index => {
                "`xs[i]` reads a sequence `xs` at an integer `i`; on a text, a byte".to_string()
            }
            Op::Join => "`++` joins two sequences of one type, or two texts".to_string(),
            Op::StartsWith => {
                "`a.starts_with(b)` takes two sequences of one type, or two texts".to_string()
            }
            Op::Decoded if matches!(takes.first(), Some(Type::Encoded(_))) => {
                "`.decoded()` on a text `[~char]` gives its letters, and waits for the grapheme decoder: not built yet".to_string()
            }
            Op::Decoded => "`.decoded()` reads a sequence".to_string(),
            Op::Fold => match takes.get(1) {
                Some(start) => format!(
                    "`xs.fold(start) {{ x -> next }}` reads a sequence `xs`, and its block gives the next values of what it carries, of the types `{}`, in place",
                    carried_types(start)
                ),
                None => "`xs.fold(start) { x -> next }` reads a sequence".to_string(),
            },
            Op::Until => match takes.first() {
                Some(start) => format!(
                    "`until(start, done) {{ next }}`: `done` is a bool, and the block gives the next values of what it carries, of the types `{}`, in place",
                    carried_types(start)
                ),
                None => "`until(start, done) { next }`: `done` is a bool".to_string(),
            },
            Op::Let | Op::Truth(_) | Op::Int(_) | Op::Real(_) | Op::Text(_) => {
                return Vec::new()
            }
        };
        let mut notes = vec![format!("note: {takes_what}")];
        // What each argument meets: the type the call or record wants
        // there, else the other arguments' types.
        let meets: Vec<Vec<Type>> = match self.wanted(op) {
            Some(want) => want.into_iter().map(|(_, t)| vec![t]).collect(),
            None => takes.iter().map(|_| takes.to_vec()).collect(),
        };
        let mut helps = BTreeSet::new();
        for ((a, t), meets) in args.iter().zip(takes).zip(&meets) {
            let Expression::Apply(Op::Int(n), none) = &a.body else {
                continue;
            };
            if t != &Type::Int || !none.is_empty() {
                continue;
            }
            if meets.contains(&Type::Real) {
                helps.insert(format!(
                    "help: Mitra converts no number; write `{n}.0` for a rational"
                ));
            }
            if let Some(Type::Param { name, .. }) = meets.iter().find(|t| t.open()) {
                helps.insert(format!(
                    "help: Mitra converts no number; a {name} comes from a function that gives one, declared beside the others"
                ));
            }
        }
        // `+` on two texts or two sequences: they are joined with `++`.
        if *op == Op::Add
            && takes.len() == 2
            && takes[0] == takes[1]
            && matches!(takes[0], Type::Seq(_) | Type::Encoded(_))
        {
            helps.insert("help: two texts or two sequences are joined with `++`".to_string());
        }
        notes.extend(helps);
        notes
    }
}

/// The types a loop carries, as written: `integer, bool`.
fn carried_types(start: &Type) -> String {
    simulation::carried(start)
        .iter()
        .map(crate::print::type_)
        .collect::<Vec<_>>()
        .join(", ")
}
