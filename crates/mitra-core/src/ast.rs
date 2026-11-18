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
    /// `(a, b)`: a tuple of two values or more.
    Tuple(Vec<Expr>),
    /// `Number(a, b)`: a record, one expression per field.
    Record(String, Vec<Expr>),
    /// `n.value` on a record, `t.0` on a tuple.
    Field(Box<Expr>, String),
    /// `[a, b]`: a list.
    List(Vec<Expr>),
    /// `xs[i]`: the element at a position, from 0.
    Index(Box<Expr>, Box<Expr>),
    /// `len(xs)`: the number of elements.
    Len(Box<Expr>),
    /// `match e { None => a, Some(v) => b, _ => c }`: by variant.
    Match(Box<Expr>, Vec<Arm>),
    /// `rate(x in S, cond)`: the share of the values of S where `cond` is
    /// true, from 0 to 1.
    Rate(String, Set, Box<Expr>),
    /// `precision(f)`, `recall(f)`, `false_positive_rate(f)`: a measure
    /// of a subject that answers true or false.
    Score(&'static str, String),
}

/// The measures of a subject that answers true or false.
pub const SCORES: [&str; 3] = ["precision", "recall", "false_positive_rate"];

/// One arm of a `match`: a variant with a name for each of its fields, or
/// `_` for any value.
#[derive(Debug, Clone, PartialEq)]
pub struct Arm {
    pub variant: Option<(String, Vec<String>)>,
    pub body: Expr,
}

/// `enum Option(T) { None, Some(value: T) }`: a type made of variants,
/// each a record, with sets as parameters.
#[derive(Debug, Clone, PartialEq)]
pub struct Enum {
    pub name: String,
    pub params: Vec<String>,
    /// The variants as written, between the braces.
    pub body: String,
    pub variants: Vec<String>,
    pub line: usize,
}

/// `record Number(value: Int, unit: String)`: a named group of fields.
#[derive(Debug, Clone, PartialEq)]
pub struct Record {
    pub name: String,
    pub fields: Vec<(String, Set)>,
    /// The enum it is a variant of. A field typed by a parameter of the
    /// enum is then `Value`.
    pub of: Option<String>,
    pub line: usize,
}

impl Record {
    /// Every value of the record.
    pub fn set(&self) -> Set {
        Set::record(
            &self.name,
            self.fields.iter().map(|f| f.1.clone()).collect(),
        )
    }
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
    /// True on every input: any expression with a Bool value, over the
    /// names the claim binds. It may call several subjects.
    Property(Expr),
    /// `uniform f by g`: the subject behaves the same on every input of a
    /// class, the classes being the values of the spec `g`. So one judged
    /// input stands for its whole class. `uniform f`, without a spec: every
    /// input is in one class.
    Uniform(Option<String>),
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
    /// `rate(x in S, cond) = 0.4`: a measure, as measured. The evidence is
    /// then about the measure as written.
    Measured(Expr, f64),
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

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Record(usize),
    Enum(usize),
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Model {
    pub records: Vec<Record>,
    pub enums: Vec<Enum>,
    /// Sets written with an enum, such as `Option(Int)`, by their printed
    /// form: reports print them as written.
    pub aliases: Vec<(String, String)>,
    /// The records and enums in the order declared, by their place in
    /// `records` and `enums`.
    pub declared: Vec<Decl>,
    pub functions: Vec<Function>,
    /// One hidden function per `Property` claim, named after it, taking
    /// the names the claim binds. Filled by `check`.
    pub properties: Vec<Function>,
    pub claims: Vec<Claim>,
    pub tests: Vec<Test>,
    pub evidence: Vec<Evidence>,
}

impl Model {
    /// Check the spec `g` of `uniform f by g`. Without one, nothing to check.
    fn spec_of_classes(&self, f: &Function, g: Option<&str>) -> Result<(), String> {
        let Some(name) = g else {
            return Ok(());
        };
        let Some(g) = self.function(name).filter(|g| g.is_spec()) else {
            return Err(format!("`{name}` is not a spec"));
        };
        let types = |f: &Function| f.params.iter().map(|p| p.1.clone()).collect::<Vec<_>>();
        if types(f) != types(g) {
            return Err(format!(
                "`{}` and `{}` must take the same inputs",
                f.name, g.name
            ));
        }
        if g.range.count().is_none_or(|n| n > MAX_CLASSES) {
            return Err(format!(
                "`{}` must have at most {MAX_CLASSES} results, one per class",
                g.name
            ));
        }
        Ok(())
    }

    /// `text` with each record's whole set, `Number(Int, String)`, written
    /// as its name. Both read back as the same set.
    pub fn named(&self, text: &str) -> String {
        let mut out = text.to_string();
        let mut aliases = self.aliases.clone();
        aliases.sort_by_key(|a| std::cmp::Reverse(a.0.len()));
        for (set, written) in &aliases {
            out = out.replace(set, written);
        }
        for r in self.records.iter().filter(|r| r.of.is_none()) {
            let whole = self
                .records
                .iter()
                .take_while(|o| o.name != r.name)
                .filter(|o| o.of.is_none())
                .fold(r.set().to_string(), |t, o| {
                    t.replace(&o.set().to_string(), &o.name)
                });
            out = out.replace(&whole, &r.name);
        }
        out
    }
    pub fn record(&self, name: &str) -> Option<&Record> {
        self.records.iter().find(|r| r.name == name)
    }
    /// A function, or the hidden function of a property claim.
    pub fn function(&self, name: &str) -> Option<&Function> {
        let hidden = || self.properties.iter().find(|f| f.name == name);
        self.declared_function(name).or_else(hidden)
    }
    /// A function the model declares.
    pub fn declared_function(&self, name: &str) -> Option<&Function> {
        self.functions.iter().find(|f| f.name == name)
    }
    /// `precision(f)` and the like need a subject that answers true or
    /// false.
    fn scored(&self, kind: &str, f: &str) -> Result<(), String> {
        match self.declared_function(f) {
            Some(g) if !g.is_spec() && g.range.is_subset(&Set::bool()) => Ok(()),
            _ => Err(format!(
                "`{kind}({f})` needs a subject that answers true or false"
            )),
        }
    }
    /// The subjects a claim is about: its own, or for a property those it
    /// calls.
    pub fn subjects_of(&self, c: &Claim) -> Vec<String> {
        let subject = |g: &String| self.declared_function(g).is_some_and(|g| !g.is_spec());
        let all = match &c.target {
            Target::Property(e) => calls(e).into_iter().filter(subject).collect(),
            _ => vec![c.func.clone()],
        };
        let mut out: Vec<String> = Vec::new();
        for g in all {
            if !out.contains(&g) {
                out.push(g);
            }
        }
        out
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
            self.records.iter().map(|r| (&r.name, r.line)).collect();
        names.extend(self.enums.iter().map(|e| (&e.name, e.line)));
        for r in &self.records {
            for (i, (field, _)) in r.fields.iter().enumerate() {
                if r.fields[..i].iter().any(|f| &f.0 == field) {
                    let msg = format!("`{}` has two fields `{field}`", r.name);
                    return Err(m(r.line, msg));
                }
            }
        }
        names.extend(self.functions.iter().map(|f| (&f.name, f.line)));
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
        // An arm of a `match` names a variant, with a name for each field.
        let bodies = self
            .functions
            .iter()
            .filter_map(|f| Some((f.body.as_ref()?, f.line)));
        let claimed = self.claims.iter().filter_map(|c| match &c.target {
            Target::Matches(e) | Target::Property(e) => Some((e, c.line)),
            _ => None,
        });
        for (e, line) in bodies.chain(claimed) {
            for (name, fields) in arms(e) {
                let Some(r) = self
                    .records
                    .iter()
                    .find(|r| r.name == name && r.of.is_some())
                else {
                    return Err(m(line, format!("`{name}` is not a variant of an enum")));
                };
                if r.fields.len() != fields.len() {
                    let msg = format!(
                        "`{name}` has {} field(s), the arm names {}",
                        r.fields.len(),
                        fields.len()
                    );
                    return Err(m(line, msg));
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
        // A property is about the names it binds: its hidden function takes
        // them as inputs, each with the set given or the type of the
        // parameters it is passed to.
        let mut properties = Vec::new();
        for c in &self.claims {
            let Target::Property(e) = &c.target else {
                continue;
            };
            for (g, n) in arities(e) {
                match self.declared_function(&g) {
                    None => return Err(m(c.line, format!("unknown function `{g}`"))),
                    Some(f) if f.params.len() != n => {
                        let k = f.params.len();
                        return Err(m(c.line, format!("`{g}` takes {k} input(s), got {n}")));
                    }
                    _ => {}
                }
            }
            let mut scored = Vec::new();
            walk(e, &mut |e| {
                if let Expr::Score(kind, f) = e {
                    scored.push((*kind, f.clone()));
                }
            });
            for (kind, f) in scored {
                self.scored(kind, &f).map_err(|msg| m(c.line, msg))?;
            }
            let mut params = Vec::new();
            for p in &c.pattern {
                let x = p.var.as_ref().expect("a property binds names");
                let mut types = Vec::new();
                passed(e, x, &self, &mut types);
                let set = match (&p.set, types.first()) {
                    (Some(s), _) => {
                        if let Some((g, _)) = types.iter().find(|t| !s.is_subset(&t.1)) {
                            let msg = format!("`{x}` goes outside the inputs of `{g}`");
                            return Err(m(c.line, msg));
                        }
                        s.clone()
                    }
                    (None, Some(_)) => types.iter().fold(Set::any(), |a, t| a.intersect(&t.1)),
                    (None, None) => {
                        let msg = format!(
                            "`{x}` is not passed to a function as it is: give its set, `forall {x} in S,`"
                        );
                        return Err(m(c.line, msg));
                    }
                };
                params.push((x.clone(), set));
            }
            properties.push(Function {
                name: c.name.clone(),
                params,
                range: Set::bool(),
                body: None,
                line: c.line,
            });
        }
        self.properties = properties;
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
            self.spec_of_classes(f, g.as_deref())
                .map_err(|msg| m(c.line, msg))?;
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
                    if self.declared_function(&name).is_none() {
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
        let ranges: Vec<String> = self
            .functions
            .iter()
            .map(|f| self.named(&f.range.to_string()))
            .collect();
        let arities: Vec<(String, usize)> = self
            .claims
            .iter()
            .filter_map(|c| Some((c.name.clone(), self.function(&c.func)?.params.len())))
            .collect();
        let tests = self.tests.iter_mut().map(|t| &mut t.judgment);
        for e in self.evidence.iter_mut().chain(tests) {
            let arity = arities.iter().find(|a| a.0 == e.about).map(|a| a.1);
            if let (Judgment::Holds(_), true, Some(n)) = (&e.judgment, e.pattern.is_empty(), arity)
            {
                let whole = Position {
                    var: None,
                    set: None,
                };
                e.pattern = vec![whole; n];
            }
        }
        let tests = self.tests.iter().map(|t| (&t.judgment, Input::Model));
        let evidence = self.evidence.iter().map(|e| (e, Input::Evidence));
        for (e, input) in evidence.chain(tests) {
            let ev = |line, msg: String| (input, Error::new(line, msg));
            match (
                &e.judgment,
                self.declared_function(&e.about),
                self.claim(&e.about),
            ) {
                (Judgment::Holds(_), _, Some(c)) => {
                    fits(self.function(&c.func).unwrap(), &e.pattern, e.line, input)?
                }
                (Judgment::Measured(measure, value), _, _) => {
                    let mut bad = None;
                    if !(0.0..=1.0).contains(value) {
                        bad = Some(format!("a measure goes from 0 to 1, got {value}"));
                    }
                    walk(measure, &mut |e| match e {
                        Expr::Call(f, _) if self.declared_function(f).is_none() => {
                            bad = Some(format!("unknown function `{f}`"));
                        }
                        Expr::Score(kind, f) => bad = bad.take().or(self.scored(kind, f).err()),
                        _ => {}
                    });
                    if let Some(msg) = bad {
                        return Err(ev(e.line, msg));
                    }
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
                                    f.name,
                                    ranges[self
                                        .functions
                                        .iter()
                                        .position(|g| g.name == f.name)
                                        .unwrap()]
                                ),
                            ));
                        }
                    }
                }
                (_, None, _) if self.claim(&e.about).is_some() => {
                    let c = &e.about;
                    let inputs = match self.claim(c).is_some_and(|c| c.target.is_structural()) {
                        true => String::new(),
                        false => format!(", or `evidence {c}(inputs) at 1` for some inputs"),
                    };
                    return Err(ev(
                        e.line,
                        format!(
                            "`{c}` is a claim, not a subject: `correct(…)` and `= value` are for subjects\n  \
                             help: to vouch for the claim, write `evidence {c} at 1`{inputs}"
                        ),
                    ));
                }
                (_, None, _) => return Err(ev(e.line, format!("`{}` is not a subject", e.about))),
            }
        }
        Ok(self)
    }
}

/// Every call in an expression, with its number of arguments.
fn arities(e: &Expr) -> Vec<(String, usize)> {
    let mut out = Vec::new();
    walk(e, &mut |e| {
        if let Expr::Call(f, args) = e {
            out.push((f.clone(), args.len()));
        }
    });
    out
}

/// The parameter types of the functions that `x` is passed to as it is.
fn passed(e: &Expr, x: &str, model: &Model, out: &mut Vec<(String, Set)>) {
    walk(e, &mut |e| {
        let Expr::Call(f, args) = e else { return };
        let Some(f) = model.declared_function(f) else {
            return;
        };
        for (a, (_, ty)) in args.iter().zip(&f.params) {
            if matches!(a, Expr::Var(v) if v == x) {
                out.push((f.name.clone(), ty.clone()));
            }
        }
    });
}

/// Calls `f` on an expression and on every expression inside it.
pub fn walk(e: &Expr, f: &mut impl FnMut(&Expr)) {
    f(e);
    match e {
        Expr::Lit(_) | Expr::Var(_) => {}
        Expr::Match(a, arms) => {
            walk(a, f);
            arms.iter().for_each(|arm| walk(&arm.body, f));
        }
        Expr::Score(..) => {}
        Expr::Unary(_, a) | Expr::Field(a, _) | Expr::Len(a) | Expr::Rate(_, _, a) => walk(a, f),
        Expr::Call(_, args) | Expr::Tuple(args) | Expr::Record(_, args) | Expr::List(args) => {
            args.iter().for_each(|a| walk(a, f))
        }
        Expr::Binary(_, a, b) | Expr::Let(_, a, b) | Expr::Index(a, b) => {
            walk(a, f);
            walk(b, f);
        }
        Expr::If(a, b, c) => {
            walk(a, f);
            walk(b, f);
            walk(c, f);
        }
    }
}

/// The variant of every arm of every `match` in an expression.
fn arms(e: &Expr) -> Vec<(String, Vec<String>)> {
    match e {
        Expr::Lit(_) | Expr::Var(_) => vec![],
        Expr::Match(a, list) => {
            let own = list.iter().filter_map(|arm| arm.variant.clone());
            let inner = list.iter().flat_map(|arm| arms(&arm.body));
            arms(a).into_iter().chain(own).chain(inner).collect()
        }
        Expr::Score(..) => vec![],
        Expr::Unary(_, a) | Expr::Field(a, _) | Expr::Len(a) | Expr::Rate(_, _, a) => arms(a),
        Expr::Call(_, args) | Expr::Tuple(args) | Expr::Record(_, args) | Expr::List(args) => {
            args.iter().flat_map(arms).collect()
        }
        Expr::Binary(_, a, b) | Expr::Let(_, a, b) | Expr::Index(a, b) => {
            [arms(a), arms(b)].concat()
        }
        Expr::If(a, b, c) => [arms(a), arms(b), arms(c)].concat(),
    }
}

/// Every function called in an expression.
pub fn calls(e: &Expr) -> Vec<String> {
    match e {
        Expr::Lit(_) | Expr::Var(_) => vec![],
        Expr::Call(f, args) => std::iter::once(f.clone())
            .chain(args.iter().flat_map(calls))
            .collect(),
        Expr::Match(a, arms) => calls(a)
            .into_iter()
            .chain(arms.iter().flat_map(|arm| calls(&arm.body)))
            .collect(),
        Expr::Score(_, f) => vec![f.clone()],
        Expr::Unary(_, a) | Expr::Field(a, _) | Expr::Len(a) | Expr::Rate(_, _, a) => calls(a),
        Expr::Tuple(args) | Expr::Record(_, args) | Expr::List(args) => {
            args.iter().flat_map(calls).collect()
        }
        Expr::Binary(_, a, b) | Expr::Let(_, a, b) | Expr::Index(a, b) => {
            [calls(a), calls(b)].concat()
        }
        Expr::If(a, b, c) => [calls(a), calls(b), calls(c)].concat(),
    }
}

/// The input tuple of a pattern made only of values.
pub fn single(p: &Pattern) -> Option<Vec<Value>> {
    p.iter()
        .map(|pos| pos.set.as_ref().and_then(|s| s.single()))
        .collect()
}
