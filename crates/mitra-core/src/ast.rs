//! The semantic model. Text is one projection of it (see `parse`).

use crate::set::{Set, Value};
use crate::tuples::Tuples;
use crate::{Error, Input};

/// Expressions of the spec language: a small, pure, first-order core.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Lit(Value),
    Var(String),
    /// A call of a spec or a subject.
    Call(String, Vec<Expr>),
    Unary(&'static str, Box<Expr>),
    Binary(&'static str, Box<Expr>, Box<Expr>),
    If(Box<Expr>, Box<Expr>, Box<Expr>),
    Let(String, Box<Expr>, Box<Expr>),
}

/// A function: a `subject` (opaque, observed) when `body` is `None`, a
/// `spec` (a definition Mitra evaluates) otherwise.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    /// Each parameter with its type: the set of values it accepts.
    pub params: Vec<(String, Set)>,
    /// The result's type.
    pub range: Set,
    pub body: Option<Expr>,
    pub line: usize,
}

impl Function {
    pub fn is_spec(&self) -> bool {
        self.body.is_some()
    }
    /// Every input tuple the function accepts.
    pub fn domain(&self) -> Tuples {
        Tuples::product(self.params.iter().map(|p| p.1.clone()).collect())
    }
}

/// Which inputs a claim or a piece of evidence is about: one position per
/// parameter.
pub type Pattern = Vec<Position>;

/// One argument of a call: a bound name (`∀ x,` or `∀ x ∈ S,`) or a value.
#[derive(Debug, Clone, PartialEq)]
pub struct Position {
    /// The name bound to the input, if any.
    pub var: Option<String>,
    /// `None` is the parameter's whole type.
    pub set: Option<Set>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Target {
    /// Judged by people: evidence says `correct` or `wrong`.
    Correct,
    /// Equal to this expression on every input. A bare function name means
    /// that function applied to the same input.
    Matches(Expr),
    /// `uniform f by g`: the subject behaves the same on every input of a
    /// class, the classes being the values of the spec `g`. So one judged
    /// input stands for its whole class.
    Uniform(String),
    /// `uses f [g, h]`: the subject calls these subjects, so a failure of
    /// one of them is the first place to look when `f` fails.
    /// `only uses f [g, h]` adds that it calls no other subject.
    Uses { used: Vec<String>, only: bool },
}

impl Target {
    /// A claim about the subject's structure, not about its results.
    pub fn is_structural(&self) -> bool {
        matches!(self, Target::Uniform(_) | Target::Uses { .. })
    }
}

/// The most classes a `uniform` claim's spec may have.
pub const MAX_CLASSES: f64 = 64.0;

/// `claim name : ∀ …, correct(f(args)) | f(args) = e [:= by derive [c1, c2]]`
/// or `claim name : uniform f by g`
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
    /// `correct(f(args))` / `¬ correct(f(args))`, on a subject.
    Correct(bool),
    /// `f(args) = value`: the observed output of a subject.
    Output(Value),
    /// `c` / `¬ c`, on a claim.
    Holds(bool),
}

/// `evidence ∀ …, <judgment> at confidence`
#[derive(Debug, Clone, PartialEq)]
pub struct Evidence {
    /// A subject, or a claim for `holds`/`fails`.
    pub about: String,
    pub pattern: Pattern,
    pub judgment: Judgment,
    pub confidence: f64,
    /// `@[source "…"]`: who or what observed it (a CI job, a reviewer).
    pub source: Option<String>,
    /// `@[rev "…"]`: the revision of the code it was observed on.
    pub rev: Option<String>,
    pub line: usize,
}

/// `test name : <judgment>`: a test of the code, and the evidence its pass
/// stands for. `mitra ingest` turns test results into evidence.
#[derive(Debug, Clone, PartialEq)]
pub struct Test {
    pub name: String,
    /// The judgment a pass supports, as evidence at confidence 1.
    pub judgment: Evidence,
    pub line: usize,
}

impl Test {
    /// The evidence a run gives: the judgment when it passed. When it
    /// failed, `¬ correct` or `¬ c` if the test claims `correct` or `c` on a
    /// single input. Otherwise a failure gives nothing: on a set, Mitra
    /// can't tell which input was wrong; for an output, it doesn't know the
    /// output, unless the subject's result type `result` is `Bool` (then it
    /// is the other one); and a failed negative test is no proof of
    /// correctness (it may have failed for another reason).
    pub fn outcome(&self, passed: bool, result: Option<&Set>) -> Result<Evidence, &'static str> {
        let mut e = self.judgment.clone();
        if passed {
            return Ok(e);
        }
        if e.pattern.is_empty() || single(&e.pattern).is_none() {
            return Err("it is about a set of inputs, and Mitra can't tell which one was wrong");
        }
        e.judgment =
            match e.judgment {
                Judgment::Correct(true) => Judgment::Correct(false),
                Judgment::Holds(true) => Judgment::Holds(false),
                Judgment::Output(Value::Bool(b)) if result == Some(&Set::bool()) => {
                    Judgment::Output(Value::Bool(!b))
                }
                Judgment::Output(_) => {
                    return Err("the output it checks was something else, Mitra doesn't know what")
                }
                _ => return Err(
                    "a failed negative test proves nothing: it may have failed for another reason",
                ),
            };
        Ok(e)
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Model {
    pub functions: Vec<Function>,
    pub claims: Vec<Claim>,
    pub tests: Vec<Test>,
    pub evidence: Vec<Evidence>,
}

impl Model {
    pub fn function(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|f| f.name == name)
    }
    pub fn claim(&self, name: &str) -> Option<&Claim> {
        self.claims.iter().find(|c| c.name == name)
    }

    /// The set of input tuples a pattern covers for a function.
    pub fn covers(&self, func: &str, p: &Pattern) -> Tuples {
        let f = self.function(func).expect("checked");
        Tuples::product(
            p.iter()
                .zip(&f.params)
                .map(|(pos, (_, ty))| pos.set.clone().unwrap_or(ty.clone()))
                .collect(),
        )
    }

    /// Check names, references and that patterns stay inside domains.
    pub fn check(mut self) -> Result<Model, (Input, Error)> {
        let m = |line, msg: String| (Input::Model, Error::new(line, msg));
        let mut names: Vec<(&String, usize)> =
            self.functions.iter().map(|f| (&f.name, f.line)).collect();
        names.extend(self.claims.iter().map(|c| (&c.name, c.line)));
        names.extend(self.tests.iter().map(|t| (&t.name, t.line)));
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
        let fits = |func: &Function, p: &Pattern, line: usize, input: Input| {
            let err = |msg: String| Err((input, Error::new(line, msg)));
            if p.len() != func.params.len() {
                let n = func.params.len();
                return err(format!(
                    "`{}` takes {n} input(s), got {}",
                    func.name,
                    p.len()
                ));
            }
            let outside = p
                .iter()
                .zip(&func.params)
                .any(|(pos, (_, ty))| pos.set.as_ref().is_some_and(|s| !s.is_subset(ty)));
            if outside {
                return err(format!("inputs outside the domain of `{}`", func.name));
            }
            Ok(())
        };
        // `uniform f by g` is about every input of `f`.
        for i in 0..self.claims.len() {
            let c = &self.claims[i];
            if !c.target.is_structural() {
                continue;
            }
            let Some(f) = self.function(&c.func) else {
                return Err(m(c.line, format!("unknown function `{}`", c.func)));
            };
            let whole = Position {
                var: None,
                set: None,
            };
            let arity = f.params.len();
            if f.is_spec() {
                let msg = format!(
                    "`{}` is a spec: `uniform` and `uses` are for subjects",
                    c.func
                );
                return Err(m(c.line, msg));
            }
            let g = match &c.target {
                Target::Uniform(g) => g,
                Target::Uses { used, .. } => {
                    for (n, g) in used.iter().enumerate() {
                        if used[..n].contains(g) {
                            return Err(m(c.line, format!("`{g}` is listed twice")));
                        }
                        if !self.function(g).is_some_and(|g| !g.is_spec()) {
                            return Err(m(c.line, format!("`{g}` is not a subject")));
                        }
                        if g == &c.func {
                            return Err(m(c.line, format!("`{g}` can't use itself")));
                        }
                    }
                    self.claims[i].pattern = vec![whole; arity];
                    continue;
                }
                _ => unreachable!(),
            };
            let Some(g) = self.function(g).filter(|g| g.is_spec()) else {
                return Err(m(c.line, format!("`{g}` is not a spec")));
            };
            let types = |f: &Function| f.params.iter().map(|p| p.1.clone()).collect::<Vec<_>>();
            if types(f) != types(g) {
                return Err(m(
                    c.line,
                    format!("`{}` and `{}` must take the same inputs", f.name, g.name),
                ));
            }
            if g.range.count().is_none_or(|n| n > MAX_CLASSES) {
                return Err(m(
                    c.line,
                    format!(
                        "`{}` must have at most {MAX_CLASSES} results, one per class",
                        g.name
                    ),
                ));
            }
            let twice = |d: &Claim| d.func == c.func && matches!(d.target, Target::Uniform(_));
            if self.claims[..i].iter().any(twice) {
                return Err(m(
                    c.line,
                    format!("`{}` already has a `uniform` claim", c.func),
                ));
            }
            self.claims[i].pattern = vec![whole; arity];
        }
        for (i, c) in self.claims.iter().enumerate() {
            let Some(f) = self.function(&c.func) else {
                return Err(m(c.line, format!("unknown function `{}`", c.func)));
            };
            fits(f, &c.pattern, c.line, Input::Model)?;
            if let Target::Matches(e) = &c.target {
                let bound = |v: &String| c.pattern.iter().any(|p| p.var.as_ref() == Some(v));
                let fun_ref = matches!(e, Expr::Var(v) if self.function(v).is_some() && !bound(v));
                for name in calls(e).into_iter().chain(fun_ref.then(|| match e {
                    Expr::Var(v) => v.clone(),
                    _ => unreachable!(),
                })) {
                    if self.function(&name).is_none() {
                        return Err(m(c.line, format!("unknown function `{name}`")));
                    }
                }
            } else if c.target == Target::Correct && f.is_spec() {
                return Err(m(
                    c.line,
                    format!(
                        "`{}` is a spec: `correct` is for subjects; write `{} … = …`",
                        c.func, c.func
                    ),
                ));
            }
            for p in &c.from {
                let structural = |d: &&Claim| &d.name == p && d.target.is_structural();
                if self.claims.iter().any(|d| structural(&d)) {
                    return Err(m(
                        c.line,
                        format!("`{p}` is a `uniform` or `uses` claim: it can't be a premise"),
                    ));
                }
                if !self.claims[..i].iter().any(|d| &d.name == p) {
                    return Err(m(
                        c.line,
                        format!("`{p}` is not a claim defined above `{}`", c.name),
                    ));
                }
            }
        }
        // `evidence c at 1`: a claim on its whole domain.
        let tests = self.tests.iter_mut().map(|t| &mut t.judgment);
        for e in self.evidence.iter_mut().chain(tests) {
            let claim_func = self
                .claims
                .iter()
                .find(|c| c.name == e.about)
                .map(|c| &c.func);
            let arity = claim_func.and_then(|f| self.functions.iter().find(|g| &g.name == f));
            if let (Judgment::Holds(_), true, Some(f)) = (&e.judgment, e.pattern.is_empty(), arity)
            {
                let whole = Position {
                    var: None,
                    set: None,
                };
                e.pattern = vec![whole; f.params.len()];
            }
        }
        let tests = self.tests.iter().map(|t| (&t.judgment, Input::Model));
        let evidence = self.evidence.iter().map(|e| (e, Input::Evidence));
        for (e, input) in evidence.chain(tests) {
            let ev = |line, msg: String| (input, Error::new(line, msg));
            match (&e.judgment, self.function(&e.about), self.claim(&e.about)) {
                (Judgment::Holds(_), _, Some(c)) => {
                    fits(self.function(&c.func).unwrap(), &e.pattern, e.line, input)?
                }
                (Judgment::Holds(_), _, None) => {
                    return Err(ev(
                        e.line,
                        format!(
                            "`{}` is not a claim (or an output is missing: `= value`)",
                            e.about
                        ),
                    ))
                }
                (_, Some(f), _) => {
                    fits(f, &e.pattern, e.line, input)?;
                    if matches!(e.judgment, Judgment::Output(_)) && single(&e.pattern).is_none() {
                        return Err(ev(
                            e.line,
                            "an observed output needs a single input value".into(),
                        ));
                    }
                    if let Judgment::Output(v) = &e.judgment {
                        if !f.range.contains(v) {
                            return Err(ev(
                                e.line,
                                format!(
                                    "output {v} is outside the result type of `{}`: {}",
                                    f.name, f.range
                                ),
                            ));
                        }
                    }
                }
                (_, None, _) => return Err(ev(e.line, format!("`{}` is not a subject", e.about))),
            }
        }
        Ok(self)
    }
}

/// Every function called in an expression.
pub fn calls(e: &Expr) -> Vec<String> {
    match e {
        Expr::Lit(_) | Expr::Var(_) => vec![],
        Expr::Call(f, args) => std::iter::once(f.clone())
            .chain(args.iter().flat_map(calls))
            .collect(),
        Expr::Unary(_, a) => calls(a),
        Expr::Binary(_, a, b) | Expr::Let(_, a, b) => [calls(a), calls(b)].concat(),
        Expr::If(a, b, c) => [calls(a), calls(b), calls(c)].concat(),
    }
}

/// The input tuple of a pattern made only of values.
pub fn single(p: &Pattern) -> Option<Vec<Value>> {
    p.iter()
        .map(|pos| pos.set.as_ref().and_then(|s| s.single()))
        .collect()
}
