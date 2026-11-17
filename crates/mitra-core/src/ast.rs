//! The semantic model. Text is one projection of it (see `parse`).

use crate::{Error, Input};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Value {
    Bool(bool),
    Int(i64),
}

/// The values a parameter ranges over. `Bool` is finite, so evidence can
/// cover it exhaustively; `Any` (untyped) never can.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Domain {
    Bool,
    Any,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Param {
    pub name: String,
    pub domain: Domain,
}

/// The body of a claim. Calls name the subject under trust (`not(a)`); the
/// rest is mathematics. Parsed and kept, not yet evaluated.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Var(String),
    Lit(Value),
    Call(String, Vec<Expr>),
    Not(Box<Expr>),
    Eq(Box<Expr>, Box<Expr>),
}

/// A universal claim: `body` holds for every value of `params`.
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Expr,
    pub line: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Test,
    Proof,
    Usage,
    Review,
    Agent,
}

impl Kind {
    pub fn from_name(s: &str) -> Option<Kind> {
        Some(match s {
            "test" => Kind::Test,
            "proof" => Kind::Proof,
            "usage" => Kind::Usage,
            "review" => Kind::Review,
            "agent" => Kind::Agent,
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Fail,
}

/// `evidence kind(source) : claim(inputs) = pass|fail @ confidence`
#[derive(Debug, Clone, PartialEq)]
pub struct Evidence {
    pub kind: Kind,
    pub source: String,
    pub claim: String,
    pub inputs: Vec<Value>,
    pub outcome: Outcome,
    pub confidence: f64,
    pub line: usize,
}

/// Claims plus the evidence about them, checked against each other.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub claims: Vec<Claim>,
    pub evidence: Vec<Evidence>,
}

impl Model {
    pub fn new(claims: Vec<Claim>, evidence: Vec<Evidence>) -> Result<Model, (Input, Error)> {
        for (i, c) in claims.iter().enumerate() {
            if claims[..i].iter().any(|d| d.name == c.name) {
                return Err((
                    Input::Claims,
                    Error::new(c.line, format!("claim `{}` is defined twice", c.name)),
                ));
            }
        }
        for e in &evidence {
            let err = |m: String| (Input::Evidence, Error::new(e.line, m));
            let Some(c) = claims.iter().find(|c| c.name == e.claim) else {
                return Err(err(format!("unknown claim `{}`", e.claim)));
            };
            if c.params.len() != e.inputs.len() {
                return Err(err(format!(
                    "`{}` takes {} input(s), got {}",
                    c.name,
                    c.params.len(),
                    e.inputs.len()
                )));
            }
            for (p, v) in c.params.iter().zip(&e.inputs) {
                if p.domain == Domain::Bool && !matches!(v, Value::Bool(_)) {
                    return Err(err(format!("`{}` is a bool, got {:?}", p.name, v)));
                }
            }
        }
        Ok(Model { claims, evidence })
    }
}
