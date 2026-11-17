//! From evidence to verdicts. Only evidence at confidence 0 or 1 exists so
//! far: evidence at 1 is fully believed, evidence at 0 is ignored.

use crate::ast::*;
use crate::set::Set;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Logic {
    Holds,
    Fails,
    Uncertain,
}

impl Logic {
    pub fn as_str(self) -> &'static str {
        match self {
            Logic::Holds => "holds",
            Logic::Fails => "fails",
            Logic::Uncertain => "uncertain",
        }
    }
}

/// The verdict on one claim under one policy.
#[derive(Debug, Clone, PartialEq)]
pub struct Verdict {
    pub claim: String,
    pub policy: String,
    /// What the fully believed evidence proves, by strict logic.
    pub logic: Logic,
    /// The claim's inputs judged correct and not contradicted.
    pub covered: Set,
    /// The claim's inputs judged wrong and not contradicted.
    pub refuted: Set,
    /// The inputs the claim is about.
    pub domain: Set,
    /// How much the policy believes the claim, from 0 to 1.
    pub belief: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub verdicts: Vec<Verdict>,
}

pub fn evaluate(model: &Model) -> Report {
    let mut verdicts = Vec::new();
    for c in &model.claims {
        let domain = model.covers(&c.subject, &c.pattern);
        let believed = |correct: bool| {
            model
                .evidence
                .iter()
                .filter(|e| e.subject == c.subject && e.correct == correct && e.confidence == 1.0)
                .fold(Set::empty(), |s, e| {
                    s.union(&model.covers(&e.subject, &e.pattern))
                })
                .intersect(&domain)
        };
        let (correct, wrong) = (believed(true), believed(false));
        // An input judged both correct and wrong is a contradiction: it
        // counts on neither side.
        let covered = correct.minus(&wrong);
        let refuted = wrong.minus(&correct);
        let logic = if !refuted.is_empty() {
            Logic::Fails
        } else if covered == domain {
            Logic::Holds
        } else {
            Logic::Uncertain
        };
        for p in &model.policies {
            let belief = match (&p.kind, logic) {
                (_, Logic::Fails) => 0.0,
                (PolicyKind::Strict, l) => (l == Logic::Holds) as u8 as f64,
                (PolicyKind::Coverage(profile), l) => {
                    let measure = |s: &Set| measure(model, &c.subject, profile, s);
                    let whole = measure(&domain);
                    if whole > 0.0 {
                        measure(&covered) / whole
                    } else {
                        (l == Logic::Holds) as u8 as f64
                    }
                }
            };
            verdicts.push(Verdict {
                claim: c.name.clone(),
                policy: p.name.clone(),
                logic,
                covered: covered.clone(),
                refuted: refuted.clone(),
                domain: domain.clone(),
                belief,
            });
        }
    }
    Report { verdicts }
}

/// The share of a subject's inputs that `s` represents under a profile.
///
/// Each region gets its share of the total weight. Inside a finite region
/// every value weighs the same. Inside an infinite region a set counts only
/// if it covers the whole region: any finite sample of it measures 0.
fn measure(model: &Model, subject: &str, profile: &str, s: &Set) -> f64 {
    let domain = &model.subject(subject).domain;
    let regions: Vec<(Option<Set>, f64)> = model
        .profiles
        .iter()
        .find(|p| p.name == profile && p.subject == subject)
        .map(|p| p.regions.clone())
        .unwrap_or_else(|| vec![(None, 1.0)]); // `uniform`, or a profile for another subject
    let total: f64 = regions.iter().map(|r| r.1).sum();
    let mut seen = Set::empty();
    let mut m = 0.0;
    for (region, w) in regions {
        let region = region.unwrap_or_else(|| domain.clone()).minus(&seen);
        seen = seen.union(&region);
        let inside = s.intersect(&region);
        let share = match region.count() {
            _ if region.is_empty() => 0.0,
            Some(n) => inside.count().unwrap() / n,
            None => (inside == region) as u8 as f64,
        };
        m += w / total * share;
    }
    m
}

impl Report {
    /// The result file: one line per claim and policy.
    pub fn to_text(&self) -> String {
        self.verdicts
            .iter()
            .map(|v| {
                let set = match v.logic {
                    Logic::Fails => format!("refuted: {}", v.refuted),
                    _ => format!("covered: {}", v.covered),
                };
                format!(
                    "verdict {} [{}] = {{ logic: {}, {}, domain: {}, belief: {} }}\n",
                    v.claim,
                    v.policy,
                    v.logic.as_str(),
                    set,
                    v.domain,
                    num(v.belief)
                )
            })
            .collect()
    }
}

fn num(x: f64) -> String {
    let s = format!("{:.4}", x);
    s.trim_end_matches('0').trim_end_matches('.').to_string()
}
