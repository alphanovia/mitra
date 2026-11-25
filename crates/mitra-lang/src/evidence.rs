//! Evidence (`docs/design/evidence.md`): what was measured and reviewed of the
//! real system, one set at one time, and whether it reads against a
//! model. A measurement is always through a tie: of the real thing, not
//! of the model (examples are the model's spec, simulation the model
//! run on itself). A review is one voice saying a proposition holds or
//! not: no count, since independent voices are what weigh. A tie is a
//! proposition too, and is reviewed by its item's path.
//!
//! `check(model, ledger)` gives every error of suit `V`, each at the
//! measurement or review to fix: `measurements::3`, `reviews::0`.

use crate::catalogue::{Error, V001, V002, V003, V004, V005, V006, V007};
use crate::expression::Reading;
use crate::model::{Function, Index, Item, Model, Path};
use crate::simulation::{Exact, Type, Value};
use std::collections::BTreeSet;

/// What a tied function was seen to do, `count` times: at its inputs,
/// each by its path (`calc::div::a`), it gave its outputs (`calc::div::q`)
/// when `seen`, and something else when not. The test states the
/// outputs it expects; `seen` says whether the system gave them. An
/// input is what a function takes, or what one with no body gave (the
/// world's value then); an output is what a function gives. Inputs and
/// outputs may be of different functions, as in an end-to-end test.
#[derive(Clone, Debug, PartialEq)]
pub struct Measurement {
    pub inputs: Vec<(Path, Value)>,
    pub outputs: Vec<(Path, Value)>,
    pub seen: bool,
    pub count: u64,
    pub by: String,
    /// The calls to functions with no body the system made, as `then`
    /// lists them after the value (BDD step 2): each one the model's run
    /// does not make counts against it.
    pub effects: crate::model::Effects,
}

/// Someone says the claim, rule, require, ensure or tied item at `prop`
/// holds, or not. Of a tied item, it is said of each of its ties.
///
/// With `from`, it is said of a link instead: that what `prop` is from
/// gives it. Of an ensure, one of its `from` lists; of a require, the
/// ensure it is from. With `seeing`, the voice says it seeing those
/// tests pass: the review counts only while every one passes
/// (`reader::ledger` drops it otherwise, and says so). A test is never
/// the voice of a review: it measures; someone reviews, seeing it.
#[derive(Clone, Debug, PartialEq)]
pub struct Review {
    pub prop: Path,
    pub from: Option<Vec<Path>>,
    pub holds: bool,
    pub by: String,
    pub seeing: Vec<String>,
}

/// The links of the piece at `at` that a review can name, each the list
/// it is from: an ensure's `from` lists, a require's ensure.
pub(crate) fn links(index: &Index, at: &Path) -> Vec<Vec<Path>> {
    match index.items.get(at) {
        Some(Item::Ensure(e)) => e.from.clone(),
        Some(Item::Require(r)) => match &r.from {
            crate::model::Source::Ensure(e) => vec![vec![e.clone()]],
            crate::model::Source::Prose(_) => vec![],
        },
        _ => vec![],
    }
}

impl Review {
    /// What it says holds or fails: `a::e`, or `a::e from (b::c, b::d)`.
    pub fn said(&self) -> String {
        match &self.from {
            None => self.prop.to_string(),
            Some(from) => {
                let list: Vec<String> = from.iter().map(|p| p.to_string()).collect();
                format!("{} from ({})", self.prop, list.join(", "))
            }
        }
    }
}

/// The evidence, one set at one time.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ledger {
    pub measurements: Vec<Measurement>,
    pub reviews: Vec<Review>,
}

/// What a port names: the function, its path, the port's type, and
/// whether it is what the function gives.
pub(crate) struct Port<'m> {
    pub(crate) function: &'m Function,
    pub(crate) at: Path,
    pub(crate) ty: Type,
    pub(crate) gives: bool,
}

/// The port `p`, `part::function::name`: what a function takes or gives.
pub(crate) fn port<'m>(index: &Index<'m>, p: &Path) -> Option<Port<'m>> {
    let [part, f, name] = p.names.as_slice() else {
        return None;
    };
    let function = *index.functions.get(&format!("{part}::{f}"))?;
    let at = Path::to(&[part, f]);
    match function.takes.iter().find(|(n, _)| n == name) {
        Some((_, t)) => Some(Port {
            function,
            at,
            ty: t.clone(),
            gives: false,
        }),
        None if &function.gives.0 == name => {
            let ty = function.gives.1.clone();
            Some(Port {
                function,
                at,
                ty,
                gives: true,
            })
        }
        None => None,
    }
}

/// The kinds of voice: `human:name`, `agent:…`, `machine:tool`,
/// `test:name`.
pub const KINDS: [&str; 4] = ["human", "agent", "machine", "test"];

/// What is wrong with a voice the ledger gives, if anything: none, one
/// not of a kind, or one of Mitra's own (`model`, `machine:mitra-check`,
/// `assumed:link`),
/// which only explain takes with.
fn voice_error(by: &str) -> Option<String> {
    if by.is_empty() {
        return Some("it says no voice".to_string());
    }
    if [
        crate::explain::MODEL,
        crate::explain::CHECK,
        crate::explain::LINK,
        crate::explain::PATH,
    ]
    .contains(&by)
    {
        let message =
            format!("`{by}` is Mitra's own voice: the ledger says who else saw or said it");
        return Some(message);
    }
    let kind = by
        .split_once(':')
        .filter(|(k, name)| KINDS.contains(k) && !name.is_empty());
    kind.is_none().then(|| {
        format!("`{by}` is no voice: `human:name`, `agent:name`, `machine:tool` or `test:name`")
    })
}

/// The errors of `ledger` read against `model`, none when it reads.
pub fn check(model: &Model, ledger: &Ledger) -> Vec<Error> {
    let index = Index::of(model);
    let tied: BTreeSet<&Path> = model.ties.iter().map(|t| &t.item).collect();
    let mut errors = Vec::new();
    for (i, m) in ledger.measurements.iter().enumerate() {
        let at = Path::to(&["measurements", &i.to_string()]);
        let error = |code, message: String| Error {
            code,
            at: at.clone(),
            message,
        };
        if m.count == 0 {
            errors.push(error(V004, "it counts no run".to_string()));
        }
        if let Some(message) = voice_error(&m.by) {
            errors.push(error(V007, message));
        }
        if m.outputs.is_empty() {
            errors.push(error(V006, "it measures no output".to_string()));
        }
        let mut given = BTreeSet::new();
        let mut needs = Vec::new();
        let mut through = BTreeSet::new();
        for (side, ports) in [("input", &m.inputs), ("output", &m.outputs)] {
            let mut seen = BTreeSet::new();
            for (p, v) in ports.iter() {
                let Some(port) = port(&index, p) else {
                    errors.push(error(
                        V001,
                        format!("`{p}` is nothing a function takes or gives"),
                    ));
                    continue;
                };
                if !seen.insert(p) {
                    errors.push(error(V001, format!("`{p}` is given twice")));
                }
                match (side, port.gives) {
                    ("input", true) if port.function.body.is_some() => {
                        let message = format!("`{p}` is what `{}` gives, and the model computes it: an output, not an input", port.at);
                        errors.push(error(V001, message));
                    }
                    ("output", false) => {
                        let message = format!(
                            "`{p}` is what `{}` takes: an output is what a function gives",
                            port.at
                        );
                        errors.push(error(V001, message));
                    }
                    _ => {}
                }
                if !Exact::default().has_type(v, &port.ty) {
                    errors.push(error(
                        V003,
                        format!("the value of `{p}` is not of its type"),
                    ));
                } else if !crate::check::value_made(&index, v) {
                    let message = format!("the value of `{p}` makes a record other than as declared: no value of an opaque record is written");
                    errors.push(error(V003, message));
                }
                if side == "input" {
                    given.insert(p.clone());
                }
                // What the model must compute: an output, or a value the
                // world gave, at the point its function was given.
                if port.gives && (side == "output" || port.function.body.is_none()) {
                    needs.push((port.at.clone(), port.function));
                }
                through.insert(port.at);
            }
        }
        // What the system was given is not what it gave: a value on both
        // sides would be its own expected value.
        let outputs: BTreeSet<&Path> = m.outputs.iter().map(|(p, _)| p).collect();
        for (p, _) in &m.inputs {
            if outputs.contains(p) {
                let message = format!(
                    "`{p}` is both an input and an output: it would be its own expected value"
                );
                errors.push(error(V001, message));
            }
        }
        for f in &through {
            let part = Path::to(&[&f.names[0]]);
            if !tied.contains(f) && !tied.contains(&part) {
                let message =
                    format!("`{f}` is tied to nothing: a measurement is always through a tie");
                errors.push(error(V002, message));
            }
        }
        for (f, function) in needs {
            for (name, _) in &function.takes {
                if !given.contains(&f.then(name)) {
                    errors.push(error(V006, format!("`{f}::{name}` is not given")));
                }
            }
        }
    }
    let mut said = BTreeSet::new();
    for (i, r) in ledger.reviews.iter().enumerate() {
        let at = Path::to(&["reviews", &i.to_string()]);
        let error = |code, message: String| Error {
            code,
            at: at.clone(),
            message,
        };
        let prop = matches!(
            index.items.get(&r.prop),
            Some(Item::Claim | Item::Rule(_) | Item::Require(_) | Item::Ensure(_))
        );
        if let Some(from) = &r.from {
            if !links(&index, &r.prop).contains(from) {
                let list: Vec<String> = from.iter().map(|p| p.to_string()).collect();
                let message = format!(
                    "`{}` is not from ({}): a link reviewed is an ensure's `from` list, or a require's ensure",
                    r.prop,
                    list.join(", ")
                );
                errors.push(error(V005, message));
            }
        } else if !prop && !tied.contains(&r.prop) {
            let message = format!(
                "`{}` is no claim, rule, require, ensure or tied item",
                r.prop
            );
            errors.push(error(V005, message));
        }
        if let Some(message) = voice_error(&r.by) {
            errors.push(error(V007, message));
        } else if r.by.starts_with("test:") {
            let message = format!(
                "`{}` is a test, and a test is no reviewer: it measures; write who reviewed, `seeing \"{}\"`",
                r.by, r.by
            );
            errors.push(error(V007, message));
        } else if !said.insert((&r.prop, &r.from, &r.by)) {
            let message = format!("`{}` reviews `{}` twice", r.by, r.said());
            errors.push(error(V007, message));
        }
    }
    errors.sort();
    errors.dedup();
    errors
}
