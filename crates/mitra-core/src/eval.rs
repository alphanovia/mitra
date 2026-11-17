//! From evidence to a verdict and a certainty per claim.
//!
//! Only the extremes are defined so far: evidence at confidence 1 is fully
//! believed and evidence at 0 brings nothing. The calculus for confidences
//! in between is still open (see docs/seed.md).

use crate::ast::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Holds,
    Fails,
    Uncertain,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Holds => "holds",
            Verdict::Fails => "fails",
            Verdict::Uncertain => "uncertain",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClaimResult {
    pub claim: String,
    pub verdict: Verdict,
    /// How certain the verdict is: 1 is certain, 0 is no idea. Its exact
    /// meaning (probability, opinion, ...) is not decided yet.
    pub certainty: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub claims: Vec<ClaimResult>,
}

/// What the fully believed evidence says about one input.
#[derive(Debug, Clone, Copy, PartialEq)]
enum Known {
    Correct,
    Wrong,
    /// No fully believed evidence.
    Nothing,
    /// Fully believed evidence both ways.
    Contradiction,
}

fn known(model: &Model, subject: &str, inputs: &[bool]) -> Known {
    let believed = |correct| {
        model.evidence.iter().any(|e| {
            e.subject == subject
                && e.inputs == inputs
                && e.correct == correct
                && e.confidence == 1.0
        })
    };
    match (believed(true), believed(false)) {
        (true, false) => Known::Correct,
        (false, true) => Known::Wrong,
        (false, false) => Known::Nothing,
        (true, true) => Known::Contradiction,
    }
}

pub fn evaluate(model: &Model) -> Report {
    let claims = model
        .claims
        .iter()
        .map(|c| {
            let known: Vec<Known> = c
                .inputs()
                .iter()
                .map(|i| known(model, &c.subject, i))
                .collect();
            // One input known wrong refutes the claim; it holds only when
            // every input it covers is known correct; otherwise no idea.
            let (verdict, certainty) = if known.contains(&Known::Wrong) {
                (Verdict::Fails, 1.0)
            } else if known.iter().all(|k| *k == Known::Correct) {
                (Verdict::Holds, 1.0)
            } else {
                (Verdict::Uncertain, 0.0)
            };
            ClaimResult {
                claim: c.name.clone(),
                verdict,
                certainty,
            }
        })
        .collect();
    Report { claims }
}

impl Report {
    /// The result file: one line per claim, `name  verdict @ certainty`.
    pub fn to_text(&self) -> String {
        let width = self.claims.iter().map(|c| c.claim.len()).max().unwrap_or(0);
        self.claims
            .iter()
            .map(|c| {
                format!(
                    "{:width$}  {} @ {}\n",
                    c.claim,
                    c.verdict.as_str(),
                    c.certainty
                )
            })
            .collect()
    }
}
