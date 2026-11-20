//! What the last two levels say beyond the report: what is left to a
//! person and what can be handed over (`assess`), and what an
//! observation changes (`diagnose`).

use crate::ast::Model;
use crate::report::Places;
use crate::verdicts::{Display, Logic, Report};

/// What is still owed, split by who can settle it.
#[derive(Debug, Clone, PartialEq, Default)]
// mitra: implements levels::Left
pub struct Left {
    /// Only a person can: a review, a vouch, a decision on evidence set
    /// aside.
    pub person: Vec<String>,
    /// A test to write or code to tie: it can be handed to an agent.
    pub delegate: Vec<String>,
}

/// Split what `mitra plan` says is owed: an entry that asks for a review
/// or a reviewer's word is a person's, any other can be handed over.
/// Evidence a person said that was set aside waits for a person too.
// mitra: implements levels::left
pub fn left(model: &Model, report: &Report) -> Left {
    let mut left = Left::default();
    for line in crate::plan(model, report).lines() {
        let Some((name, owed)) = line.split_once(" owed:") else {
            continue;
        };
        if line.starts_with(' ') {
            continue;
        }
        match owed.contains("review") {
            true => left.person.push(format!("{name}:{owed}")),
            false if owed.trim().is_empty() => left.delegate.push(format!("{name}: tests")),
            false => left.delegate.push(format!("{name}:{owed}")),
        }
    }
    for a in &model.aside {
        if !crate::ast::machine(a.source.as_deref()) {
            let who = a.source.as_deref().unwrap_or("someone");
            let said = format!(
                "{}: what {who} said of it was set aside, say it again or remove it",
                a.about
            );
            left.person.push(said);
        }
    }
    left
}

/// What an observation changes.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements levels::Diagnosis
pub struct Diagnosis {
    /// It makes a claim fail, or cannot be true with the rest.
    pub contradicts: bool,
    /// The verdicts that change, `claim: before -> after`.
    pub changed: Vec<String>,
    pub text: String,
}

/// Compare the report without an observation and the report with it.
pub fn diagnosis(
    before: &Report,
    after: &Report,
    display: &Display,
    places: &dyn Places,
) -> Diagnosis {
    let was = |claim: &str| before.verdicts.iter().find(|v| v.claim == claim);
    let mut changed = Vec::new();
    let mut fails = false;
    for v in &after.verdicts {
        let old = was(&v.claim).map(|w| w.logic);
        if old != Some(v.logic) {
            let old = old.map_or("nothing", |l| l.as_str());
            changed.push(format!("{}: {old} -> {}", v.claim, v.logic.as_str()));
            fails |= v.logic == Logic::Fails;
        }
    }
    let old = before.problem_texts(display, places);
    let new: Vec<String> = after.problem_texts(display, places);
    let new: Vec<&String> = new.iter().filter(|p| !old.contains(p)).collect();
    let contradicts = fails || !new.is_empty();
    let mut text = match contradicts {
        true => "diagnose: the observation contradicts what the model and its ledger say\n",
        false => {
            "diagnose: the observation contradicts nothing: it agrees with the model, \
             or it is outside what the model promises\n"
        }
    }
    .to_string();
    for c in &changed {
        text += &format!("  {c}\n");
    }
    if changed.is_empty() {
        text += "  no verdict changes\n";
    }
    for p in new {
        text += p;
    }
    Diagnosis {
        contradicts,
        changed,
        text,
    }
}
