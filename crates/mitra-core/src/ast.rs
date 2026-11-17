//! The semantic model. Text is one projection of it (see `parse`).

use crate::set::{Set, Value};
use crate::{Error, Input};

/// Expressions of the spec language: a small, pure, first-order core.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit(Value),
    Var(String),
    /// A call of a spec or a subject, with one argument.
    Call(String, Box<Expr>),
    Unary(&'static str, Box<Expr>),
    Binary(&'static str, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
}

/// A function with one parameter: a `subject` (opaque, observed) when
/// `body` is `None`, a `spec` (a definition Mitra evaluates) otherwise.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    pub param: String,
    /// The parameter's type: the set of inputs the function accepts.
    pub domain: Set,
    /// The result's type.
    pub range: Set,
    pub body: Option<Expr>,
    pub line: usize,
}

impl Function {
    pub fn is_spec(&self) -> bool {
        self.body.is_some()
    }
}

/// Which inputs a claim or a piece of evidence is about: `_`, a value,
/// `x` or `x in <set>`.
#[derive(Debug, Clone, PartialEq)]
pub struct Pattern {
    /// The name bound to the input, if any.
    pub var: Option<String>,
    /// `None` is the whole domain.
    pub set: Option<Set>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    /// Judged by people: evidence says `correct` or `wrong`.
    Correct,
    /// Equal to this expression on every input. A bare function name means
    /// that function applied to the same input.
    Matches(Expr),
}

/// `claim name: f(pattern) correct|matches <expr> [from c1, c2]`
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    pub name: String,
    pub func: String,
    pub pattern: Pattern,
    pub target: Target,
    /// Premises: when they all hold, this claim holds on its whole domain.
    pub from: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Judgment {
    /// `correct` / `wrong`, on a subject.
    Correct(bool),
    /// `-> value`: the observed output of a subject.
    Output(Value),
    /// `holds` / `fails`, on a claim.
    Holds(bool),
}

/// `evidence name(pattern) <judgment> @ confidence`
#[derive(Debug, Clone, PartialEq)]
pub struct Evidence {
    /// A subject, or a claim for `holds`/`fails`.
    pub about: String,
    pub pattern: Pattern,
    pub judgment: Judgment,
    pub confidence: f64,
    pub line: usize,
}

/// `profile name for subject:` then indented weighted regions.
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

#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    pub name: String,
    pub kind: PolicyKind,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Model {
    pub functions: Vec<Function>,
    pub claims: Vec<Claim>,
    pub profiles: Vec<Profile>,
    pub policies: Vec<Policy>,
    pub evidence: Vec<Evidence>,
}

impl Model {
    pub fn function(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|f| f.name == name)
    }
    pub fn claim(&self, name: &str) -> Option<&Claim> {
        self.claims.iter().find(|c| c.name == name)
    }

    /// The set of inputs a pattern covers for a function.
    pub fn covers(&self, func: &str, p: &Pattern) -> Set {
        p.set
            .clone()
            .unwrap_or_else(|| self.function(func).expect("checked").domain.clone())
    }

    /// Check names, references and that patterns stay inside domains.
    /// With no policy, `strict` is used.
    pub fn check(mut self) -> Result<Model, (Input, Error)> {
        let m = |line, msg: String| (Input::Model, Error::new(line, msg));
        let ev = |line, msg: String| (Input::Evidence, Error::new(line, msg));
        let mut names: Vec<(&String, usize)> =
            self.functions.iter().map(|f| (&f.name, f.line)).collect();
        names.extend(self.claims.iter().map(|c| (&c.name, c.line)));
        for (i, (n, line)) in names.iter().enumerate() {
            if names[..i].iter().any(|(o, _)| o == n) {
                return Err(m(*line, format!("`{n}` is defined twice")));
            }
        }
        for (i, f) in self.functions.iter().enumerate() {
            if let Some(body) = &f.body {
                // Specs may only call specs defined before them: no recursion.
                let earlier: Vec<&str> = self.functions[..i]
                    .iter()
                    .filter(|g| g.is_spec())
                    .map(|g| g.name.as_str())
                    .collect();
                if let Some(bad) = calls(body)
                    .into_iter()
                    .find(|c| !earlier.contains(&c.as_str()))
                {
                    return Err(m(
                        f.line,
                        format!("`{bad}` is not a spec defined above `{}`", f.name),
                    ));
                }
            }
        }
        let inside =
            |func: &Function, p: &Pattern| p.set.as_ref().is_none_or(|s| s.is_subset(&func.domain));
        for (i, c) in self.claims.iter().enumerate() {
            let Some(f) = self.function(&c.func) else {
                return Err(m(c.line, format!("unknown function `{}`", c.func)));
            };
            if !inside(f, &c.pattern) {
                return Err(m(
                    c.line,
                    format!("inputs outside the domain of `{}`", c.func),
                ));
            }
            if let Target::Matches(e) = &c.target {
                let fun_ref = matches!(e, Expr::Var(v) if self.function(v).is_some() && Some(v) != c.pattern.var.as_ref());
                for name in calls(e).into_iter().chain(fun_ref.then(|| match e {
                    Expr::Var(v) => v.clone(),
                    _ => unreachable!(),
                })) {
                    if self.function(&name).is_none() {
                        return Err(m(c.line, format!("unknown function `{name}`")));
                    }
                }
            } else if f.is_spec() {
                return Err(m(c.line, format!("`{}` is a spec: use `matches`", c.func)));
            }
            for p in &c.from {
                if !self.claims[..i].iter().any(|d| &d.name == p) {
                    return Err(m(
                        c.line,
                        format!("`{p}` is not a claim defined above `{}`", c.name),
                    ));
                }
            }
        }
        for p in &self.profiles {
            let Some(f) = self.function(&p.subject) else {
                return Err(m(p.line, format!("unknown function `{}`", p.subject)));
            };
            for (r, w) in &p.regions {
                if r.as_ref().is_some_and(|r| !r.is_subset(&f.domain)) {
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
            match (&e.judgment, self.function(&e.about), self.claim(&e.about)) {
                (Judgment::Holds(_), _, Some(c)) => {
                    if !inside(self.function(&c.func).unwrap(), &e.pattern) {
                        return Err(ev(
                            e.line,
                            format!("inputs outside the domain of `{}`", c.name),
                        ));
                    }
                }
                (Judgment::Holds(_), _, None) => {
                    return Err(ev(
                        e.line,
                        format!("`holds`/`fails` needs a claim, `{}` is not one", e.about),
                    ))
                }
                (_, Some(f), _) => {
                    if !inside(f, &e.pattern) {
                        return Err(ev(
                            e.line,
                            format!("inputs outside the domain of `{}`", f.name),
                        ));
                    }
                    if matches!(e.judgment, Judgment::Output(_))
                        && e.pattern.set.as_ref().and_then(|s| s.single()).is_none()
                    {
                        return Err(ev(
                            e.line,
                            "an observed output needs a single input value".into(),
                        ));
                    }
                }
                (_, None, _) => return Err(ev(e.line, format!("`{}` is not a subject", e.about))),
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

/// Every function called in an expression.
pub fn calls(e: &Expr) -> Vec<String> {
    match e {
        Expr::Lit(_) | Expr::Var(_) => vec![],
        Expr::Call(f, a) => [vec![f.clone()], calls(a)].concat(),
        Expr::Unary(_, a) => calls(a),
        Expr::Binary(_, a, b) | Expr::Let(_, a, b) => [calls(a), calls(b)].concat(),
        Expr::If(a, b, c) => [calls(a), calls(b), calls(c)].concat(),
    }
}
