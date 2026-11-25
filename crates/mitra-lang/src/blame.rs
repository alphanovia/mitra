//! Blame (`docs/design/blame.md`): which evidence is wrong, given the model
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
//! first, then by the most trusted kind each drops: what only the model
//! assumed, then measurements, reviews, the rules explain wrote, and
//! proofs last.
//!
//! Conflicts that share no suspect are apart: each group's diagnoses are
//! found on their own, and a diagnosis of all is one of each group's.

use crate::catalogue::{Finding, B001, B002, B003};
use crate::explain::{Explanation, Source};
use crate::judgement::{Prop, Taking};
use crate::model::{Index, Item, Model, Path};
use std::cmp::Reverse;
use std::collections::{BTreeSet, BinaryHeap, VecDeque};

/// How far blame looks.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Settings {
    /// The diagnoses given, at most. Default 10.
    pub diagnoses: usize,
    /// The sets of suspects tried, at most, for each group of conflicts
    /// that share suspects. Default 100,000.
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
// A taking holds a value, a variant's among them: large beside an index.
#[allow(clippy::large_enum_variant)]
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
/// model says first, and the links no one reviewed, then measurements, reviews, the rules explain
/// wrote, and the proofs the checker took last. A diagnosis is ranked by
/// the most trusted kind it drops, then the next.
pub fn kind(explanation: &Explanation, s: &Suspect) -> u8 {
    match s {
        Suspect::Measurement(_) => 1,
        Suspect::Review(_) => 2,
        Suspect::Taking(t) => match explanation.sources.get(t) {
            Some(Source::Model | Source::Link | Source::Path(_) | Source::Shows) | None => 0,
            Some(Source::Measurement(_) | Source::Support(_)) => 1,
            Some(Source::Review(_)) => 2,
            Some(Source::Written) => 3,
            Some(Source::Proof | Source::Named) => 4,
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
    let rank = |d: &BTreeSet<Suspect>| {
        let mut kinds: Vec<u8> = d.iter().map(|s| kind(explanation, s)).collect();
        kinds.sort_unstable_by(|a, b| b.cmp(a));
        (d.len(), kinds, d.clone())
    };
    let (diagnoses, complete) = hitting_sets(&conflicts, settings, &rank);
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
    let diagnoses: Vec<Diagnosis> = diagnoses
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
    // A proof by cases read one taking a point: a diagnosis that drops it
    // and not the others that say the same there may leave the all.
    let mut outlived = BTreeSet::new();
    for d in &diagnoses {
        for t in &d.drop {
            let Some(others) = explanation.alike.get(t) else {
                continue;
            };
            let kept: Vec<String> = others
                .iter()
                .filter(|o| !d.drop.contains(*o))
                .map(|o| format!("`{}` by {}", crate::print::print_prop(&o.prop), o.by))
                .collect();
            if !kept.is_empty() && outlived.insert(t.clone()) {
                findings.push(Finding {
                    code: B003,
                    at: Path::to(&["blame"]),
                    message: format!(
                        "a proof by cases read `{}` by {} at its point, where {} says the same: a diagnosis that drops the first alone may leave the all, and its conflict, standing",
                        crate::print::print_prop(&t.prop),
                        t.by,
                        kept.join(", ")
                    ),
                });
            }
        }
    }
    findings.sort();
    findings.dedup();
    Blame {
        conflicts,
        diagnoses,
        complete,
        findings,
    }
}

/// `name` is the path of a claim.
fn is_claim(index: &Index, name: &str) -> bool {
    matches!(index.items.get(&index.path_of(name)), Some(Item::Claim))
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

/// The smallest sets that meet every conflict, ranked by `rank`, up to
/// `settings.diagnoses`, and whether none was left out. Conflicts that
/// share no suspect are apart: each group's are found on their own,
/// breadth first, and the first of all are taken one of each group's,
/// best first.
fn hitting_sets<K: Ord>(
    conflicts: &[BTreeSet<Suspect>],
    settings: &Settings,
    rank: &impl Fn(&BTreeSet<Suspect>) -> K,
) -> (Vec<BTreeSet<Suspect>>, bool) {
    if conflicts.is_empty() {
        return (vec![], true);
    }
    if settings.diagnoses == 0 {
        return (vec![], false);
    }
    // Groups of conflicts that share suspects.
    let mut groups: Vec<(BTreeSet<Suspect>, Vec<BTreeSet<Suspect>>)> = Vec::new();
    for c in conflicts {
        let (mut suspects, mut members) = (c.clone(), vec![c.clone()]);
        let mut apart = Vec::new();
        for (s, m) in groups {
            match s.is_disjoint(&suspects) {
                true => apart.push((s, m)),
                false => {
                    suspects.extend(s);
                    members.extend(m);
                }
            }
        }
        apart.push((suspects, members));
        groups = apart;
    }
    let mut complete = true;
    let mut each: Vec<Vec<BTreeSet<Suspect>>> = Vec::new();
    for (_, members) in &groups {
        let (mut found, whole) = group_hitting_sets(members, settings);
        complete &= whole;
        if found.is_empty() {
            return (vec![], false);
        }
        found.sort_by_cached_key(|d| rank(d));
        each.push(found);
    }
    // One of each group's, best first: the next best moves one group on.
    let union = |at: &[usize]| -> BTreeSet<Suspect> {
        at.iter()
            .zip(&each)
            .flat_map(|(i, found)| found[*i].iter().cloned())
            .collect()
    };
    let mut heap = BinaryHeap::new();
    let mut queued: BTreeSet<Vec<usize>> = BTreeSet::new();
    let start = vec![0; each.len()];
    heap.push(Reverse((rank(&union(&start)), start.clone())));
    queued.insert(start);
    let mut diagnoses = Vec::new();
    while let Some(Reverse((_, at))) = heap.pop() {
        if diagnoses.len() == settings.diagnoses {
            complete = false;
            break;
        }
        diagnoses.push(union(&at));
        for g in 0..at.len() {
            let mut next = at.clone();
            next[g] += 1;
            if next[g] < each[g].len() && queued.insert(next.clone()) {
                heap.push(Reverse((rank(&union(&next)), next)));
            }
        }
    }
    (diagnoses, complete)
}

/// The smallest sets that meet every conflict of one group, breadth
/// first (Reiter's tree): up to `settings.diagnoses`, trying at most
/// `settings.tries` sets, each set queued counting as tried. Whether
/// none was left out: blame looks for one more to know.
fn group_hitting_sets(
    conflicts: &[BTreeSet<Suspect>],
    settings: &Settings,
) -> (Vec<BTreeSet<Suspect>>, bool) {
    let mut found: Vec<BTreeSet<Suspect>> = Vec::new();
    let mut seen: BTreeSet<BTreeSet<Suspect>> = BTreeSet::new();
    let mut todo: VecDeque<BTreeSet<Suspect>> = VecDeque::from([BTreeSet::new()]);
    let mut tries = 1usize;
    while let Some(set) = todo.pop_front() {
        if found.iter().any(|d| d.is_subset(&set)) {
            continue;
        }
        match conflicts.iter().find(|c| c.is_disjoint(&set)) {
            None => {
                if found.len() == settings.diagnoses {
                    // One more than asked for: there are more.
                    return (found, false);
                }
                found.push(set);
            }
            Some(c) => {
                for t in c {
                    let mut next = set.clone();
                    next.insert(t.clone());
                    if seen.contains(&next) {
                        continue;
                    }
                    if tries >= settings.tries {
                        return (found, false);
                    }
                    tries += 1;
                    seen.insert(next.clone());
                    todo.push_back(next);
                }
            }
        }
    }
    (found, true)
}
