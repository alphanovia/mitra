//! Blame (`docs/blame.md`): which evidence is wrong, given the model
//! explains the system? The third question.
//!
//! Where explain finds a proposition both true and false, the takings of
//! a set it follows true from and of one it follows false from cannot
//! all hold: a conflict. What can be doubted is a suspect: a measurement
//! or a review of the ledger, whole, or a taking of the model's own or
//! of a check. A diagnosis is a smallest set of suspects whose dropping
//! leaves no conflict: a minimal hitting set of the conflicts (Reiter,
//! 1987). Everything can be doubted but the model's claims: given the
//! model explains the system, they hold. Diagnoses come fewest suspects
//! first, then by kind: what only the model assumed, then measurements,
//! reviews, the rules explain wrote, and proofs.

use crate::catalogue::{Finding, B001, B002};
use crate::explain::{Explanation, Source};
use crate::judgement::{Prop, Taking};
use crate::model::{Index, Item, Model, Path};
use std::collections::{BTreeSet, VecDeque};

/// How far blame looks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    /// The diagnoses given, at most. Default 10.
    pub diagnoses: usize,
    /// The sets of takings tried, at most, in finding them. Default
    /// 100,000.
    pub tries: usize,
}

impl Default for Settings {
    fn default() -> Settings {
        Settings {
            diagnoses: 10,
            tries: 100_000,
        }
    }
}

/// What can be doubted: one measurement or one review of the ledger,
/// with all it says, or one taking of the model's own or of a check.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Suspect {
    Measurement(usize),
    Review(usize),
    Taking(Taking),
}

/// What dropped together leaves no conflict: the suspects, and every
/// taking they make.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Diagnosis {
    pub suspects: BTreeSet<Suspect>,
    pub drop: BTreeSet<Taking>,
}

/// What blame gives.
#[derive(Clone, Debug, Default)]
pub struct Blame {
    /// Each conflict: suspects that cannot all hold, none holding another.
    pub conflicts: Vec<BTreeSet<Suspect>>,
    /// The diagnoses, first the most likely.
    pub diagnoses: Vec<Diagnosis>,
    /// Whether every smallest diagnosis is here.
    pub complete: bool,
    pub findings: Vec<Finding>,
}

/// The suspect a taking is part of.
pub fn suspect(explanation: &Explanation, t: &Taking) -> Suspect {
    match explanation.sources.get(t) {
        Some(Source::Measurement(i)) => Suspect::Measurement(*i),
        Some(Source::Review(i)) => Suspect::Review(*i),
        _ => Suspect::Taking(t.clone()),
    }
}

/// The kind of a suspect, in the order blame doubts them: what only the
/// model says first, then measurements, reviews, the rules explain
/// wrote, and the proofs the checker took last.
pub fn kind(explanation: &Explanation, s: &Suspect) -> u8 {
    match s {
        Suspect::Measurement(_) => 1,
        Suspect::Review(_) => 2,
        Suspect::Taking(t) => match explanation.sources.get(t) {
            Some(Source::Model) | None => 0,
            Some(Source::Measurement(_)) => 1,
            Some(Source::Review(_)) => 2,
            Some(Source::Written) => 3,
            Some(Source::Proof) => 4,
        },
    }
}

/// Blame the evidence: what to drop so that nothing conflicts.
pub fn blame(model: &Model, explanation: &Explanation, settings: &Settings) -> Blame {
    let index = Index::of(model);
    // What holds, given the model explains the system: its claims.
    let fixed = |t: &Taking| {
        explanation.sources.get(t) == Some(&Source::Model)
            && matches!(&t.prop, Prop::Named(n) if is_claim(&index, n))
    };
    let mut findings = Vec::new();
    let mut found: BTreeSet<BTreeSet<Suspect>> = BTreeSet::new();
    for (prop, opinion) in &explanation.opinions {
        for r_for in opinion.rests_for() {
            for r_against in opinion.rests_against() {
                let conflict: BTreeSet<Suspect> = r_for
                    .iter()
                    .chain(r_against)
                    .filter(|t| !fixed(t))
                    .map(|t| suspect(explanation, t))
                    .collect();
                if conflict.is_empty() {
                    findings.push(Finding {
                        code: B001,
                        at: Path::to(&["explain"]),
                        message: format!(
                            "`{}` is both true and false from the model's claims alone: no evidence can be blamed (agree should say where)",
                            crate::print::print_prop(prop)
                        ),
                    });
                    continue;
                }
                found.insert(conflict);
            }
        }
    }
    let conflicts = smallest(found);
    let (mut diagnoses, complete) = hitting_sets(&conflicts, settings);
    diagnoses.sort_by_cached_key(|d| {
        let mut kinds: Vec<u8> = d.iter().map(|s| kind(explanation, s)).collect();
        kinds.sort();
        (d.len(), kinds, d.clone())
    });
    if !complete {
        findings.push(Finding {
            code: B002,
            at: Path::to(&["blame"]),
            message: format!(
                "the first {} diagnoses only: more may exist (`Settings::diagnoses`, `Settings::tries`)",
                diagnoses.len()
            ),
        });
    }
    findings.sort();
    findings.dedup();
    let diagnoses = diagnoses
        .into_iter()
        .map(|suspects| {
            let drop = explanation
                .sources
                .keys()
                .filter(|t| suspects.contains(&suspect(explanation, t)))
                .cloned()
                .collect();
            Diagnosis { suspects, drop }
        })
        .collect();
    Blame {
        conflicts,
        diagnoses,
        complete,
        findings,
    }
}

/// `name` is the path of a claim.
fn is_claim(index: &Index, name: &str) -> bool {
    let path = Path::to(&name.split("::").collect::<Vec<_>>());
    matches!(index.items.get(&path), Some(Item::Claim))
}

/// Only the smallest: every set that holds another goes.
fn smallest(sets: BTreeSet<BTreeSet<Suspect>>) -> Vec<BTreeSet<Suspect>> {
    let mut all: Vec<BTreeSet<Suspect>> = sets.into_iter().collect();
    all.sort_by_key(BTreeSet::len);
    let mut kept: Vec<BTreeSet<Suspect>> = Vec::new();
    for s in all {
        if !kept.iter().any(|k| k.is_subset(&s)) {
            kept.push(s);
        }
    }
    kept
}

/// The smallest sets that meet every conflict, fewest takings first,
/// breadth first (Reiter's tree): up to `settings.diagnoses` of the
/// smallest size reached, trying at most `settings.tries` sets. Whether
/// none was left out.
fn hitting_sets(
    conflicts: &[BTreeSet<Suspect>],
    settings: &Settings,
) -> (Vec<BTreeSet<Suspect>>, bool) {
    if conflicts.is_empty() {
        return (vec![], true);
    }
    let mut found: Vec<BTreeSet<Suspect>> = Vec::new();
    let mut seen: BTreeSet<BTreeSet<Suspect>> = BTreeSet::new();
    let mut todo: VecDeque<BTreeSet<Suspect>> = VecDeque::from([BTreeSet::new()]);
    let mut tries = 0usize;
    while let Some(set) = todo.pop_front() {
        if found.len() >= settings.diagnoses || tries >= settings.tries {
            return (found, false);
        }
        tries += 1;
        if found.iter().any(|d| d.is_subset(&set)) {
            continue;
        }
        match conflicts.iter().find(|c| c.is_disjoint(&set)) {
            None => found.push(set),
            Some(c) => {
                for t in c {
                    let mut next = set.clone();
                    next.insert(t.clone());
                    if seen.insert(next.clone()) {
                        todo.push_back(next);
                    }
                }
            }
        }
    }
    (found, true)
}
