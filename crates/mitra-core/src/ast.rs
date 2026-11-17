//! The semantic model. Text is one projection of it (see `parse`).
//!
//! Values are booleans only for now.

use crate::{Error, Input};

/// A function, task or process under review. Mitra never runs it.
#[derive(Debug, Clone, PartialEq)]
pub struct Subject {
    pub name: String,
    pub params: Vec<String>,
    pub line: usize,
}

/// An argument of the subject in a claim: a variable bound by `forall`, or
/// a fixed value.
#[derive(Debug, Clone, PartialEq)]
pub enum Arg {
    Var(String),
    Value(bool),
}

/// The specification side of a claim. Parsed and kept, not evaluated:
/// evidence says directly whether a result was correct.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Var(String),
    Value(bool),
    Call(String, Vec<Expr>),
    Not(Box<Expr>),
}

/// `claim name: forall a: bool. subject(args) = spec`
///
/// The subject is correct on every input the claim covers.
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    pub name: String,
    /// Variables bound by `forall`, each ranging over `bool`.
    pub forall: Vec<String>,
    pub subject: String,
    pub args: Vec<Arg>,
    pub spec: Expr,
    pub line: usize,
}

impl Claim {
    /// Every input of the subject this claim covers.
    pub fn inputs(&self) -> Vec<Vec<bool>> {
        let n = self.forall.len();
        (0..1usize << n)
            .map(|bits| {
                let value = |v: &str| {
                    let i = self
                        .forall
                        .iter()
                        .position(|f| f == v)
                        .expect("checked by Model::new");
                    bits >> (n - 1 - i) & 1 == 1
                };
                self.args
                    .iter()
                    .map(|a| match a {
                        Arg::Var(v) => value(v),
                        Arg::Value(b) => *b,
                    })
                    .collect()
            })
            .collect()
    }
}

/// `evidence subject(inputs) correct|wrong @ confidence`
///
/// The subject was run on `inputs`; its result was judged correct (or
/// wrong) with the given confidence.
#[derive(Debug, Clone, PartialEq)]
pub struct Evidence {
    pub subject: String,
    pub inputs: Vec<bool>,
    pub correct: bool,
    pub confidence: f64,
    pub line: usize,
}

/// Subjects, claims and evidence, checked against each other.
#[derive(Debug, Clone, PartialEq)]
pub struct Model {
    pub subjects: Vec<Subject>,
    pub claims: Vec<Claim>,
    pub evidence: Vec<Evidence>,
}

impl Model {
    pub fn new(
        subjects: Vec<Subject>,
        claims: Vec<Claim>,
        evidence: Vec<Evidence>,
    ) -> Result<Model, (Input, Error)> {
        let model_err = |line, m: String| (Input::Model, Error::new(line, m));
        let arity = |name: &str| {
            subjects
                .iter()
                .find(|s| s.name == name)
                .map(|s| s.params.len())
        };
        for (i, s) in subjects.iter().enumerate() {
            if subjects[..i].iter().any(|t| t.name == s.name) {
                return Err(model_err(
                    s.line,
                    format!("subject `{}` is declared twice", s.name),
                ));
            }
        }
        for (i, c) in claims.iter().enumerate() {
            if claims[..i].iter().any(|d| d.name == c.name) {
                return Err(model_err(
                    c.line,
                    format!("claim `{}` is defined twice", c.name),
                ));
            }
            match arity(&c.subject) {
                None => {
                    return Err(model_err(
                        c.line,
                        format!("unknown subject `{}`", c.subject),
                    ))
                }
                Some(n) if n != c.args.len() => {
                    return Err(model_err(
                        c.line,
                        format!("`{}` takes {} input(s), got {}", c.subject, n, c.args.len()),
                    ))
                }
                _ => {}
            }
            for a in &c.args {
                if let Arg::Var(v) = a {
                    if !c.forall.contains(v) {
                        return Err(model_err(c.line, format!("`{v}` is not bound by `forall`")));
                    }
                }
            }
        }
        for e in &evidence {
            let err = |m: String| (Input::Evidence, Error::new(e.line, m));
            match arity(&e.subject) {
                None => return Err(err(format!("unknown subject `{}`", e.subject))),
                Some(n) if n != e.inputs.len() => {
                    return Err(err(format!(
                        "`{}` takes {} input(s), got {}",
                        e.subject,
                        n,
                        e.inputs.len()
                    )))
                }
                _ => {}
            }
        }
        Ok(Model {
            subjects,
            claims,
            evidence,
        })
    }
}
