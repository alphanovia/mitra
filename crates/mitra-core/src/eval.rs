//! From evidence to a verdict and two supports per claim.
//!
//! All rules here are placeholders for the confidence calculus (see
//! docs/seed.md). They are small on purpose, so they can be proven later.

use crate::ast::*;
use std::collections::BTreeMap;

/// Decision threshold and the weight of a concrete pass, per evidence kind.
#[derive(Debug, Clone)]
pub struct Policy {
    pub tau: f64,
    pub weight: fn(Kind) -> f64,
}

impl Default for Policy {
    fn default() -> Self {
        // One concrete pass is weak evidence for a universal claim,
        // whatever its kind: w = 0.1 for all kinds until we know better.
        Policy {
            tau: 0.5,
            weight: |_| 0.1,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Fail,
    Uncertain,
    Conflict,
}

impl Verdict {
    pub fn as_str(self) -> &'static str {
        match self {
            Verdict::Pass => "pass",
            Verdict::Fail => "fail",
            Verdict::Uncertain => "uncertain",
            Verdict::Conflict => "conflict",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClaimResult {
    pub claim: String,
    pub verdict: Verdict,
    /// Support for pass, 0..1.
    pub p: f64,
    /// Support for fail, 0..1.
    pub f: f64,
    /// Every input of a finite domain has a trusted pass.
    pub exhaustive: bool,
    /// Number of pieces of evidence about this claim.
    pub evidence: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub tau: f64,
    pub claims: Vec<ClaimResult>,
}

/// Best confidence seen for each outcome on one input.
#[derive(Default)]
struct Best {
    pass: f64,
    weighted_pass: f64,
    fail: f64,
}

pub fn evaluate(model: &Model, policy: &Policy) -> Report {
    let claims = model
        .claims
        .iter()
        .map(|c| evaluate_claim(c, &model.evidence, policy))
        .collect();
    Report {
        tau: policy.tau,
        claims,
    }
}

fn evaluate_claim(claim: &Claim, evidence: &[Evidence], policy: &Policy) -> ClaimResult {
    let tau = policy.tau;
    let mut by_input: BTreeMap<&[Value], Best> = BTreeMap::new();
    let mut count = 0;
    for e in evidence.iter().filter(|e| e.claim == claim.name) {
        count += 1;
        let b = by_input.entry(&e.inputs).or_default();
        match e.outcome {
            Outcome::Pass => {
                b.pass = b.pass.max(e.confidence);
                b.weighted_pass = b.weighted_pass.max((policy.weight)(e.kind) * e.confidence);
            }
            Outcome::Fail => b.fail = b.fail.max(e.confidence),
        }
    }

    // A counterexample refutes a universal claim: f is the most trusted one.
    let f = by_input.values().map(|b| b.fail).fold(0.0, f64::max);

    // Passes on distinct inputs combine: p = 1 - prod(1 - w * c).
    let mut p = 1.0
        - by_input
            .values()
            .map(|b| 1.0 - b.weighted_pass)
            .product::<f64>();

    // Finite domain covered by trusted passes: the claim is checked on every
    // input, so p is the weakest of those passes.
    let exhaustive = match domain(claim) {
        Some(inputs) => inputs
            .iter()
            .all(|i| by_input.get(i.as_slice()).is_some_and(|b| b.pass >= tau)),
        None => false,
    };
    if exhaustive {
        let weakest = domain(claim)
            .unwrap()
            .iter()
            .map(|i| by_input[i.as_slice()].pass)
            .fold(1.0, f64::min);
        p = p.max(weakest);
    }

    let verdict = match (p >= tau, f >= tau) {
        (true, false) => Verdict::Pass,
        (false, true) => Verdict::Fail,
        (true, true) => Verdict::Conflict,
        (false, false) => Verdict::Uncertain,
    };
    ClaimResult {
        claim: claim.name.clone(),
        verdict,
        p,
        f,
        exhaustive,
        evidence: count,
    }
}

/// Every input tuple of the claim, if all its parameters have finite domains.
fn domain(claim: &Claim) -> Option<Vec<Vec<Value>>> {
    let mut all = vec![vec![]];
    for p in &claim.params {
        let values = match p.domain {
            Domain::Bool => [Value::Bool(false), Value::Bool(true)],
            Domain::Any => return None,
        };
        all = all
            .into_iter()
            .flat_map(|t| values.iter().map(move |v| [t.clone(), vec![*v]].concat()))
            .collect();
    }
    Some(all)
}

impl Report {
    /// The result file: stable JSON, numbers rounded to 4 decimals.
    pub fn to_json(&self) -> String {
        let mut s = format!("{{\n  \"tau\": {},\n  \"claims\": [", num(self.tau));
        for (i, c) in self.claims.iter().enumerate() {
            s += if i == 0 { "\n" } else { ",\n" };
            s += &format!(
                "    {{ \"claim\": \"{}\", \"verdict\": \"{}\", \"p\": {}, \"f\": {}, \"exhaustive\": {}, \"evidence\": {} }}",
                c.claim,
                c.verdict.as_str(),
                num(c.p),
                num(c.f),
                c.exhaustive,
                c.evidence
            );
        }
        s += if self.claims.is_empty() {
            "]\n}\n"
        } else {
            "\n  ]\n}\n"
        };
        s
    }
}

fn num(x: f64) -> String {
    let s = format!("{:.4}", x);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".into()
    } else {
        s.into()
    }
}
