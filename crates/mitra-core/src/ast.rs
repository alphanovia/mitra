//! The semantic model. Text is one projection of it (see `parse`).

use crate::set::Set;
use crate::{Error, Input};

/// A function, task or process under review, with one parameter for now.
/// Mitra never runs it.
#[derive(Debug, Clone, PartialEq)]
pub struct Subject {
    pub name: String,
    pub param: String,
    /// The parameter's type: the set of inputs the subject accepts.
    pub domain: Set,
    pub line: usize,
}

/// Which inputs of a subject a claim or a piece of evidence is about.
#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    /// `_`: the whole domain.
    Any,
    /// A value, or `x in <set>`.
    In(Set),
}

/// `claim name: subject(pattern) correct`: the subject is correct on every
/// input the pattern covers.
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    pub name: String,
    pub subject: String,
    pub pattern: Pattern,
    pub line: usize,
}

/// `evidence subject(pattern) correct|wrong @ confidence`: the subject's
/// results on these inputs were judged correct (or wrong).
#[derive(Debug, Clone, PartialEq)]
pub struct Evidence {
    pub subject: String,
    pub pattern: Pattern,
    pub correct: bool,
    pub confidence: f64,
    pub line: usize,
}

/// `profile name for subject:` then indented `x in <set> weight w` lines and
/// an optional `otherwise weight w`. Regions are matched in order.
#[derive(Debug, Clone, PartialEq)]
pub struct Profile {
    pub name: String,
    pub subject: String,
    /// `None` is `otherwise`.
    pub regions: Vec<(Option<Set>, f64)>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PolicyKind {
    Strict,
    /// Belief is the share of the domain covered, measured by a profile.
    /// `uniform` is built in.
    Coverage(String),
}

/// `policy strict` or `policy name = coverage(profile)`.
#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    pub name: String,
    pub kind: PolicyKind,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Model {
    pub subjects: Vec<Subject>,
    pub claims: Vec<Claim>,
    pub profiles: Vec<Profile>,
    pub policies: Vec<Policy>,
    pub evidence: Vec<Evidence>,
}

impl Model {
    pub fn subject(&self, name: &str) -> &Subject {
        self.subjects
            .iter()
            .find(|s| s.name == name)
            .expect("checked by Model::check")
    }

    /// The set of inputs a pattern covers for a subject.
    pub fn covers(&self, subject: &str, p: &Pattern) -> Set {
        match p {
            Pattern::Any => self.subject(subject).domain.clone(),
            Pattern::In(s) => s.clone(),
        }
    }

    /// Check names, references and that every pattern stays in its
    /// subject's domain. With no policy, `strict` is used.
    pub fn check(mut self) -> Result<Model, (Input, Error)> {
        let m = |line, msg: String| (Input::Model, Error::new(line, msg));
        let ev = |line, msg: String| (Input::Evidence, Error::new(line, msg));
        let dup = |names: Vec<(&String, usize)>, what: &str| {
            for (i, (n, line)) in names.iter().enumerate() {
                if names[..i].iter().any(|(o, _)| o == n) {
                    return Err(m(*line, format!("{what} `{n}` is defined twice")));
                }
            }
            Ok(())
        };
        dup(
            self.subjects.iter().map(|s| (&s.name, s.line)).collect(),
            "subject",
        )?;
        dup(
            self.claims.iter().map(|c| (&c.name, c.line)).collect(),
            "claim",
        )?;
        dup(
            self.profiles.iter().map(|p| (&p.name, p.line)).collect(),
            "profile",
        )?;
        dup(
            self.policies.iter().map(|p| (&p.name, p.line)).collect(),
            "policy",
        )?;
        let known = |s: &str| self.subjects.iter().any(|t| t.name == s);
        let inside = |subject: &str, p: &Pattern| match p {
            Pattern::Any => true,
            Pattern::In(s) => s.is_subset(&self.subject(subject).domain),
        };
        for c in &self.claims {
            if !known(&c.subject) {
                return Err(m(c.line, format!("unknown subject `{}`", c.subject)));
            }
            if !inside(&c.subject, &c.pattern) {
                return Err(m(
                    c.line,
                    format!("inputs outside the domain of `{}`", c.subject),
                ));
            }
        }
        for p in &self.profiles {
            if !known(&p.subject) {
                return Err(m(p.line, format!("unknown subject `{}`", p.subject)));
            }
            for (r, w) in &p.regions {
                if r.as_ref()
                    .is_some_and(|r| !r.is_subset(&self.subject(&p.subject).domain))
                {
                    return Err(m(
                        p.line,
                        format!("a region is outside the domain of `{}`", p.subject),
                    ));
                }
                if *w < 0.0 {
                    return Err(m(p.line, "weights must not be negative".into()));
                }
            }
            if p.regions.iter().map(|r| r.1).sum::<f64>() <= 0.0 {
                return Err(m(p.line, "weights must not all be 0".into()));
            }
        }
        for p in &self.policies {
            if let PolicyKind::Coverage(name) = &p.kind {
                if name != "uniform" && !self.profiles.iter().any(|q| &q.name == name) {
                    return Err(m(p.line, format!("unknown profile `{name}`")));
                }
            }
        }
        for e in &self.evidence {
            if !known(&e.subject) {
                return Err(ev(e.line, format!("unknown subject `{}`", e.subject)));
            }
            if !inside(&e.subject, &e.pattern) {
                return Err(ev(
                    e.line,
                    format!("inputs outside the domain of `{}`", e.subject),
                ));
            }
        }
        if self.policies.is_empty() {
            self.policies.push(Policy {
                name: "strict".into(),
                kind: PolicyKind::Strict,
                line: 0,
            });
        }
        Ok(self)
    }
}
