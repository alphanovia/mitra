//! From evidence to verdicts: one per instance (claim on concrete inputs),
//! one per claim.
//!
//! The rules are placeholders for the confidence calculus (see
//! docs/seed.md). They are small on purpose, so they can be proven later.

use crate::ast::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    /// A support at or above `tau` decides.
    pub tau: f64,
    /// Unbounded domains only: how much one checked instance supports the
    /// universal claim. Finite domains are enumerated instead.
    pub sample_weight: f64,
}

impl Default for Policy {
    fn default() -> Self {
        Policy {
            tau: 0.5,
            sample_weight: 0.1,
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

    fn decide(p: f64, f: f64, tau: f64) -> Verdict {
        match (p >= tau, f >= tau) {
            (true, false) => Verdict::Pass,
            (false, true) => Verdict::Fail,
            (true, true) => Verdict::Conflict,
            (false, false) => Verdict::Uncertain,
        }
    }
}

/// The claim on one tuple of inputs.
#[derive(Debug, Clone, PartialEq)]
pub struct InstanceResult {
    pub inputs: Vec<Value>,
    pub verdict: Verdict,
    /// Most confident pass on these inputs.
    pub p: f64,
    /// Most confident fail on these inputs.
    pub f: f64,
    /// The evidence on these inputs, in file order.
    pub evidence: Vec<Evidence>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ClaimResult {
    pub claim: Claim,
    pub verdict: Verdict,
    /// Support for "holds on every input", 0..1.
    pub p: f64,
    /// Support for "fails on some input", 0..1.
    pub f: f64,
    /// Instances with a pass or fail verdict.
    pub decided: usize,
    /// Number of instances in the domain; `None` when unbounded.
    pub domain_size: Option<usize>,
    /// Every instance of a finite domain, or the evidenced ones otherwise.
    pub instances: Vec<InstanceResult>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Report {
    pub policy: Policy,
    pub claims: Vec<ClaimResult>,
}

pub fn evaluate(model: &Model, policy: &Policy) -> Report {
    let claims = model
        .claims
        .iter()
        .map(|c| evaluate_claim(c, &model.evidence, policy))
        .collect();
    Report {
        policy: policy.clone(),
        claims,
    }
}

fn evaluate_claim(claim: &Claim, evidence: &[Evidence], policy: &Policy) -> ClaimResult {
    let mut by_input: BTreeMap<Vec<Value>, Vec<Evidence>> = BTreeMap::new();
    if let Some(all) = domain(claim) {
        for inputs in all {
            by_input.insert(inputs, Vec::new());
        }
    }
    for e in evidence.iter().filter(|e| e.claim == claim.name) {
        by_input
            .entry(e.inputs.clone())
            .or_default()
            .push(e.clone());
    }
    let instances: Vec<InstanceResult> = by_input
        .into_iter()
        .map(|(inputs, evidence)| {
            let best = |o| {
                evidence
                    .iter()
                    .filter(|e| e.outcome == o)
                    .map(|e| e.confidence)
                    .fold(0.0, f64::max)
            };
            let (p, f) = (best(Outcome::Pass), best(Outcome::Fail));
            InstanceResult {
                inputs,
                verdict: Verdict::decide(p, f, policy.tau),
                p,
                f,
                evidence,
            }
        })
        .collect();

    // One counterexample refutes a universal claim.
    let f = instances.iter().map(|i| i.f).fold(0.0, f64::max);
    let domain_size = domain(claim).map(|d| d.len());
    let p = if domain_size.is_some() {
        // Finite: the claim is the conjunction of its instances, so it is
        // only as supported as its weakest (an unchecked instance has p = 0).
        instances.iter().map(|i| i.p).fold(1.0, f64::min)
    } else {
        // Unbounded: checked instances are samples, combined by policy.
        1.0 - instances
            .iter()
            .map(|i| 1.0 - policy.sample_weight * i.p)
            .product::<f64>()
    };
    let decided = instances
        .iter()
        .filter(|i| matches!(i.verdict, Verdict::Pass | Verdict::Fail))
        .count();
    ClaimResult {
        claim: claim.clone(),
        verdict: Verdict::decide(p, f, policy.tau),
        p,
        f,
        decided,
        domain_size,
        instances,
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
    /// The result file: stable, readable JSON, numbers rounded to 4 decimals.
    pub fn to_json(&self) -> String {
        let mut s = String::from("{\n");
        s += &format!(
            "  \"policy\": {{ \"tau\": {}, \"sample_weight\": {} }},\n",
            num(self.policy.tau),
            num(self.policy.sample_weight)
        );
        s += "  \"claims\": [";
        for (i, c) in self.claims.iter().enumerate() {
            s += if i == 0 { "\n" } else { ",\n" };
            let coverage = match c.domain_size {
                Some(n) => format!("{}/{}", c.decided, n),
                None => format!("{}/unbounded", c.decided),
            };
            s += "    {\n";
            s += &format!("      \"claim\": \"{}\",\n", signature(&c.claim));
            s += &format!(
                "      \"verdict\": \"{}\", \"p\": {}, \"f\": {}, \"coverage\": \"{}\",\n",
                c.verdict.as_str(),
                num(c.p),
                num(c.f),
                coverage
            );
            s += "      \"instances\": [";
            for (j, inst) in c.instances.iter().enumerate() {
                s += if j == 0 { "\n" } else { ",\n" };
                let args: Vec<String> = inst.inputs.iter().map(|v| v.to_string()).collect();
                s += &format!(
                    "        {{ \"instance\": \"{}({})\", \"verdict\": \"{}\", \"p\": {}, \"f\": {},\n",
                    c.claim.name,
                    args.join(", "),
                    inst.verdict.as_str(),
                    num(inst.p),
                    num(inst.f)
                );
                let ev: Vec<String> = inst
                    .evidence
                    .iter()
                    .map(|e| {
                        let o = if e.outcome == Outcome::Pass {
                            "pass"
                        } else {
                            "fail"
                        };
                        format!(
                            "\"{}({}) {} @ {}\"",
                            e.kind.name(),
                            e.source,
                            o,
                            num(e.confidence)
                        )
                    })
                    .collect();
                s += &format!("          \"evidence\": [{}] }}", ev.join(", "));
            }
            s += if c.instances.is_empty() {
                "]\n"
            } else {
                "\n      ]\n"
            };
            s += "    }";
        }
        s += if self.claims.is_empty() {
            "]\n}\n"
        } else {
            "\n  ]\n}\n"
        };
        s
    }
}

fn signature(c: &Claim) -> String {
    let params: Vec<String> = c
        .params
        .iter()
        .map(|p| match p.domain {
            Domain::Bool => format!("{}: bool", p.name),
            Domain::Any => p.name.clone(),
        })
        .collect();
    format!("{}({})", c.name, params.join(", "))
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
