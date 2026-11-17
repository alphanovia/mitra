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

/// One position of a claim's pattern: any value (`_`) or a fixed one.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Pattern {
    Any,
    Value(bool),
}

/// `claim name: subject(patterns) correct`
///
/// The subject is correct on every input the pattern matches.
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    pub name: String,
    pub subject: String,
    pub pattern: Vec<Pattern>,
    pub line: usize,
}

impl Claim {
    /// Every input of the subject the pattern matches.
    pub fn inputs(&self) -> Vec<Vec<bool>> {
        let mut all = vec![vec![]];
        for p in &self.pattern {
            let values: &[bool] = match p {
                Pattern::Any => &[false, true],
                Pattern::Value(true) => &[true],
                Pattern::Value(false) => &[false],
            };
            all = all
                .into_iter()
                .flat_map(|t: Vec<bool>| values.iter().map(move |v| [t.clone(), vec![*v]].concat()))
                .collect();
        }
        all
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
                Some(n) if n != c.pattern.len() => {
                    return Err(model_err(
                        c.line,
                        format!(
                            "`{}` takes {} input(s), got {}",
                            c.subject,
                            n,
                            c.pattern.len()
                        ),
                    ))
                }
                _ => {}
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
