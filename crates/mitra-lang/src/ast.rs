//! The semantic model, which `parse` builds from the text.

use crate::set::{Set, Value};
use crate::tuples::Tuples;
use crate::{Error, Input};
use std::collections::BTreeMap;

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
    /// `all(x in xs, cond)`, `any`, `filter`, `map`: over each element of
    /// a list or a set. `fold(x in xs, let acc := start; step)`: `step`
    /// gives the next `acc`.
    Each(&'static str, String, Box<Expr>, Box<Expr>),
    /// `correct(f(a, b))` as a value: true when evidence
    /// judges that call right, false when wrong, no value when nothing
    /// says.
    Correct(String, Vec<Expr>),
}

/// The functions over each element of a list or a set.
pub const EACH: [&str; 5] = ["all", "any", "filter", "map", "fold"];

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
    /// enum is then `Any`.
    pub of: Option<String>,
    /// `record R;` or `record R { a: T, .. }`: it has parts the model
    /// does not state, held in a last field named `..`. No value of it
    /// can be written.
    pub open: bool,
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

/// A function: the code under review (observed) when `body` is `None`, a
/// `spec` (a definition Mitra evaluates) otherwise.
#[derive(Debug, Clone, PartialEq)]
pub struct Function {
    pub name: String,
    /// Each parameter with its type: the set of values it accepts.
    pub params: Vec<(String, Set)>,
    /// The result's type.
    pub range: Set,
    pub body: Option<Expr>,
    /// With type parameters, `f(T)(x: T) -> T`: its parameters and result
    /// as written. In the sets, each type parameter is `Any`.
    pub written: Option<String>,
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

/// One argument of a call: a name bound to an input (`a`, or `a: S`), or
/// a value.
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
    /// The code of the subject is tied to the model: it exists where a
    /// tie comment says, with the inputs and the result the model gives.
    /// Mitra can't check it: a reader of the code or a reviewer says so.
    Tied,
    /// One tested input of a set stands for the whole set, and nothing is
    /// said outside it (`represented(f, S)`). The set is
    /// the inputs where the named spec is true. Mitra can't check it.
    Represented(String),
}

impl Target {
    /// A claim about the subject's structure, not about its results.
    pub fn is_structural(&self) -> bool {
        matches!(
            self,
            Target::Uniform(_) | Target::Uses { .. } | Target::Tied | Target::Represented(_)
        )
    }
}

/// The most classes a `uniform` claim's spec may have.
pub const MAX_CLASSES: f64 = 64.0;

/// `claim name(a: S, …) { correct(f(args)) }`, `{ f(args) == e }`, any
/// expression with a Bool value, or `claim name = uniform(f, g);`
// mitra: implements reader::Claim
#[derive(Debug, Clone, PartialEq)]
pub struct Claim {
    pub name: String,
    pub func: String,
    pub pattern: Pattern,
    pub target: Target,
    /// Premises: when they all hold, this claim holds on its whole domain.
    pub from: Vec<String>,
    /// The premises were not written: they are the specs of the functions
    /// the claim calls. It then follows only where the implication is
    /// checked on every input.
    pub auto: bool,
    /// It follows from its premises on every input, by reasoning on the
    /// claims of the functions it reaches.
    pub proved: bool,
    /// The trust given to the statement itself, when one is.
    pub trust: Option<f64>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Judgment {
    /// `holds` / `fails`, on a function of the code.
    Correct(bool),
    /// `f(args) = value`: the observed output of a subject.
    Output(Value),
    /// `holds` / `fails`, on a claim.
    Holds(bool),
    /// `passes`: a test passed on inputs it does not state. It supports
    /// the claim and settles nothing.
    Passed,
    /// `sampled 1000`: a test passed (or `sampled 1000 fails`: failed) on
    /// that many inputs drawn from a stated domain. Passing, it covers no
    /// input; failing, it refutes the claim on an input it does not state.
    Sampled {
        count: u64,
        ok: bool,
        domain: String,
    },
    /// `rate(x in S, cond) = 0.4`: a measure, as measured. The evidence is
    /// then about the measure as written.
    Measured(Expr, f64),
}

/// `evidence name { for target given … holds|fails|gives v trust t }`
#[derive(Debug, Clone, PartialEq)]
pub struct Evidence {
    /// A subject, or a claim for `holds`/`fails`.
    pub about: String,
    pub pattern: Pattern,
    pub judgment: Judgment,
    pub confidence: f64,
    /// `source "…"`: who or what observed it (a CI job, a reviewer).
    pub source: Option<String>,
    /// `rev "…"`: the revision of the code it was observed on.
    pub rev: Option<String>,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Record(usize),
    Enum(usize),
}

#[derive(Debug, Clone, PartialEq, Default)]
// mitra: implements reader::Model
pub struct Model {
    /// The claims on determinism: the claim, its function, and whether
    /// it says the same input always gives the same output (`false`:
    /// that it may not, `!deterministic(f)`).
    pub determinism: Vec<(String, String, bool)>,
    /// The trust written on a helper of the model, `fn h(…) -> T trust
    /// 0.9 { … }`: 1 where none is.
    pub helper_trust: Vec<(String, f64)>,
    pub records: Vec<Record>,
    pub enums: Vec<Enum>,
    /// Sets written with an enum, such as `Option(Int)`, by their printed
    /// form: reports print them as written.
    pub aliases: Vec<(String, String)>,
    /// The types that contain themselves, as written (`Tree(Int)`). In a
    /// set, such a type inside itself is `Any`; here it is the mark
    /// `$Tree(Int)`, so that a value can be checked all the way down.
    pub recursive: BTreeMap<String, Set>,
    /// For each function, the types of its parameters then of its result,
    /// with the marks of `recursive`; under `record Name`, the fields of a
    /// record that holds itself.
    pub deep: BTreeMap<String, Vec<Set>>,
    /// The records and enums in the order declared, by their place in
    /// `records` and `enums`.
    pub declared: Vec<Decl>,
    pub functions: Vec<Function>,
    /// One hidden function per `Property` claim, named after it, taking
    /// the names the claim binds. Filled by `check`.
    pub properties: Vec<Function>,
    pub claims: Vec<Claim>,
    /// The lines of the model and of the evidence: reports quote them.
    pub written: Option<(Vec<String>, Vec<String>)>,
    /// Evidence that does not fit the signature of its function, and why:
    /// it judges nothing, and is reported against the signature.
    pub misfits: Vec<(Evidence, String)>,
    /// The functions of the environment: one may answer
    /// differently each time, so two outputs on one input both stand.
    pub boundaries: Vec<String>,
    /// The requirements in prose (`ensure name "…";`).
    pub ensures: Vec<Ensure>,
    /// Where what a block requires and what its functions use disagree.
    pub lints: Vec<Lint>,
    pub evidence: Vec<Evidence>,
}

/// `ensure name "prose";`: what a subject promises, or what a boundary
/// is promised to do. Mitra never reads the prose. It is a claim of the
/// same name, which rests on `claims`, on the requirements it needs
/// (`require other::name;`) and, in a subject, on `name::captured`:
/// someone's word that the claims say what the prose says.
/// A warning from the model alone: its code, its line, what it says and
/// what to do.
#[derive(Debug, Clone, PartialEq)]
pub struct Lint {
    pub code: &'static str,
    pub line: usize,
    pub message: String,
    pub help: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ensure {
    pub name: String,
    pub prose: String,
    /// Of a boundary: assumed, with nothing of yours behind it.
    pub boundary: bool,
    /// Written in a boundary: a promise of the world.
    pub world: bool,
    /// A `require name = other::promise;`: the promise it names.
    pub needs: Option<String>,
    /// The claims of its subject.
    pub claims: Vec<String>,
    /// The requirements it needs.
    pub requires: Vec<String>,
    pub line: usize,
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

    /// Whether `v` fits parameter `i` of `f` all the way down (its result,
    /// when `i` is the number of parameters): a set only checks a type
    /// that contains itself one level deep.
    pub fn conforms(&self, f: &str, i: usize, v: &Value) -> bool {
        let deep = self.deep.get(f).and_then(|d| d.get(i));
        self.recursive.is_empty() || deep.is_none_or(|s| s.conforms(v, &self.recursive))
    }

    /// `text` with each record's whole set, `Number(Int, String)`, written
    /// as its name. Both read back as the same set.
    pub fn named(&self, text: &str) -> String {
        let mut out = text.to_string();
        let mut aliases = self.aliases.clone();
        aliases.sort_by_key(|a| std::cmp::Reverse(a.0.len()));
        for (set, written) in &aliases {
            out = out.replace(set, written);
            out = out.replace(&format!("({written}) \\"), &format!("{written} \\"));
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
        // The names Mitra gives to a body and to classes written inline
        // are not shown: the function, the claim.
        for f in &self.functions {
            for hidden in ["::body", "::case"] {
                if let Some(shown) = f.name.strip_suffix(hidden) {
                    out = out.replace(&f.name, shown);
                }
            }
        }
        out
    }
    /// `text` with each type that contains itself written by its name:
    /// read back, a name is the type all the way down, its pieces are not.
    pub fn named_recursive(&self, text: &str) -> String {
        let mut aliases: Vec<_> = self.aliases.iter().collect();
        aliases.sort_by_key(|a| std::cmp::Reverse(a.0.len()));
        let mut out = text.to_string();
        for (set, written) in aliases {
            if self.recursive.contains_key(written) {
                out = out.replace(set, written);
            }
        }
        for name in self.recursive.keys() {
            if let Some(r) = self.record(name).filter(|r| r.of.is_none()) {
                out = out.replace(&r.set().to_string(), name);
            }
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
    /// The requirement in prose of this name.
    pub fn ensure(&self, name: &str) -> Option<&Ensure> {
        self.ensures.iter().find(|e| e.name == name)
    }
    /// The requirement that the claim `name::captured` is about: someone's
    /// word that its claims say what its prose says.
    pub fn captured(&self, name: &str) -> Option<&Ensure> {
        let of = name.strip_suffix("::captured")?;
        self.ensure(of).filter(|e| !e.boundary)
    }
    /// Whether `c` is a claim on a record alone, `claim c(r: R) { … }`
    /// calling no code: true of every `R` the code builds, so judged on
    /// the values observed, never on every value of the type.
    pub fn invariant(&self, c: &Claim) -> bool {
        let (
            Target::Property(statement),
            [Position {
                var: Some(_),
                set: Some(of),
            }],
        ) = (&c.target, c.pattern.as_slice())
        else {
            return false;
        };
        let code = |g: &String| self.declared_function(g).is_some_and(|g| !g.is_spec());
        self.records
            .iter()
            .any(|r| r.of.is_none() && &r.set() == of)
            && !calls(statement).iter().any(code)
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
                "`{kind}({f})` needs a function of the code that answers true or false"
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
    // mitra: implements types::Model::check
    pub fn check(mut self) -> Result<Model, (Input, Error)> {
        let m = |line, msg: String| (Input::Model, Error::new(line, msg));
        let mut names: Vec<(&String, usize)> =
            self.records.iter().map(|r| (&r.name, r.line)).collect();
        // A record with type parameters is an enum of one variant, of
        // the same name.
        let generic_record = |e: &&Enum| e.variants.len() == 1 && e.variants[0] == e.name;
        let enums = self.enums.iter().filter(|e| !generic_record(e));
        names.extend(enums.map(|e| (&e.name, e.line)));
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
        for (i, (n, line)) in names.iter().enumerate() {
            if names[..i].iter().any(|(o, _)| o == n) {
                return Err(m(*line, format!("`{n}` is defined twice")));
            }
        }
        for (i, f) in self.functions.iter().enumerate() {
            if let Some(body) = &f.body {
                // A spec may call the specs defined before it, and itself
                // on a piece of its input: it always ends.
                let earlier: Vec<&str> = self.functions[..=i]
                    .iter()
                    .filter(|g| g.is_spec())
                    .map(|g| g.name.as_str())
                    .collect();
                let mut own = Vec::new();
                let whole = |(i, p): (usize, &(String, Set))| (p.0.clone(), Some((i, false)));
                let mut scope = f.params.iter().enumerate().map(whole).collect();
                descents(body, &f.name, &mut scope, &mut own);
                let ends = (0..f.params.len()).any(|i| own.iter().all(|c| c.get(i) == Some(&true)));
                if !own.is_empty() && !ends {
                    return Err(m(
                        f.line,
                        format!(
                            "`{0}` calls itself, and must do so on a piece of one of its \
                             inputs, always the same one: a name bound by `match`, a field, \
                             or an element (`all(x in xs, …)`)",
                            f.name
                        ),
                    ));
                }
                // A name the spec binds may hold a function.
                let mut bound: Vec<String> = f.params.iter().map(|p| p.0.clone()).collect();
                walk(body, &mut |e| match e {
                    Expr::Let(x, ..) | Expr::Each(_, x, ..) | Expr::Rate(x, ..) => {
                        bound.push(x.clone())
                    }
                    Expr::Match(_, arms) => {
                        let names = arms.iter().filter_map(|a| a.variant.as_ref());
                        bound.extend(names.flat_map(|v| v.1.clone()));
                    }
                    _ => {}
                });
                // A body may call the code of a subject: it is then
                // computed where that code was observed, and reasoned
                // about from the claims on it.
                let code = |c: &String| {
                    let opaque = self.functions.iter().any(|g| &g.name == c && !g.is_spec());
                    opaque && !self.boundaries.contains(c)
                };
                if let Some(bad) = calls(body)
                    .into_iter()
                    .find(|c| !earlier.contains(&c.as_str()) && !bound.contains(c) && !code(c))
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
        let (deep, recursive) = (self.deep.clone(), self.recursive.clone());
        let conforms = |f: &str, i: usize, v: &Value| {
            let deep = deep.get(f).and_then(|d| d.get(i));
            deep.is_none_or(|s| s.conforms(v, &recursive))
        };
        let outside = |func: &Function, p: &Pattern| {
            p.iter()
                .zip(&func.params)
                .enumerate()
                .any(|(i, (pos, (_, ty)))| {
                    pos.set.as_ref().is_some_and(|s| {
                        !s.is_subset(ty) || s.single().is_some_and(|v| !conforms(&func.name, i, &v))
                    })
                })
        };
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
            let outside = outside(func, p);
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
                // A bound name may hold a function.
                if c.pattern.iter().any(|p| p.var.as_ref() == Some(&g)) {
                    continue;
                }
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
                            "`{x}` is not passed to a function as it is: give its set, `{x}: S`"
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
                written: None,
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
                    "`{}` is a spec: `uniform` and `uses` are for the functions of the code",
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
                            return Err(m(
                                c.line,
                                format!("`{g}` is not a function of the code: `uses` lists those"),
                            ));
                        }
                        if g == &c.func {
                            return Err(m(c.line, format!("`{g}` can't use itself")));
                        }
                    }
                    self.claims[i].pattern = vec![whole; arity];
                    continue;
                }
                _ => {
                    self.claims[i].pattern = vec![whole; arity];
                    continue;
                }
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
                    // A bound name may hold a function.
                    if self.declared_function(&name).is_none() && !bound(&name) {
                        return Err(m(c.line, format!("unknown function `{name}`")));
                    }
                }
            } else if c.target == Target::Correct && f.is_spec() {
                return Err(m(
                    c.line,
                    format!(
                        "`{}` is a spec: `correct` is for the functions of the code; write `{}(…) == …`",
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
        // Evidence on a claim without `given`: on its whole domain.
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
        for e in self.evidence.iter_mut() {
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
        // Evidence that does not fit a signature: set aside, with why.
        let mut misfits: Vec<(usize, String)> = Vec::new();
        let evidence = self.evidence.iter().map(|e| (e, Input::Evidence));
        for (e, input) in evidence {
            let ev = |line, msg: String| (input, Error::new(line, msg));
            match (
                &e.judgment,
                self.declared_function(&e.about),
                self.claim(&e.about),
            ) {
                (Judgment::Holds(_), _, Some(c)) => {
                    fits(self.function(&c.func).unwrap(), &e.pattern, e.line, input)?
                }
                // A test that states no input: nothing to compare.
                (Judgment::Passed | Judgment::Sampled { .. }, _, _) => {}
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
                    let kept = input == Input::Evidence && e.pattern.len() == f.params.len();
                    if kept && outside(f, &e.pattern) {
                        let msg = format!("its inputs are outside the inputs of `{}`", f.name);
                        misfits.push((e.line, msg));
                        continue;
                    }
                    fits(f, &e.pattern, e.line, input)?;
                    if matches!(e.judgment, Judgment::Output(_)) && single(&e.pattern).is_none() {
                        return Err(ev(
                            e.line,
                            "an observed output needs a single input value".into(),
                        ));
                    }
                    if let Judgment::Output(v) = &e.judgment {
                        let fit = f.range.contains(v) && self.conforms(&f.name, f.params.len(), v);
                        if !fit && kept && single(&e.pattern).is_some() {
                            let i = self.functions.iter().position(|g| g.name == f.name);
                            let msg = format!(
                                "its output {v} is outside the result type of `{}`: {}",
                                f.name,
                                ranges[i.unwrap()]
                            );
                            misfits.push((e.line, msg));
                            continue;
                        }
                        if !fit {
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
                        false => format!(", or `for {c} given … holds` for some inputs"),
                    };
                    return Err(ev(
                        e.line,
                        format!(
                            "`{c}` is a claim, not a function: an output is given for a function\n  \
                             help: to vouch for the claim, write `evidence r {{ for {c} holds }}`{inputs}"
                        ),
                    ));
                }
                (_, None, _) => {
                    return Err(ev(
                        e.line,
                        format!("`{}` is not a function of the model", e.about),
                    ))
                }
            }
        }
        for (line, why) in misfits {
            let at = self.evidence.iter().position(|e| e.line == line);
            let e = self.evidence.remove(at.expect("its line"));
            self.misfits.push((e, why));
        }
        crate::types::check(&self).map_err(|e| (Input::Model, e))?;
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

/// A name in scope: the parameter it is a part of, and whether it is a
/// strict piece of it.
type Part = Option<(usize, bool)>;

/// For each call of `name` inside `e`: whether each argument is a strict
/// piece of the parameter at the same place.
fn descents(e: &Expr, name: &str, scope: &mut Vec<(String, Part)>, out: &mut Vec<Vec<bool>>) {
    fn part(e: &Expr, scope: &[(String, Part)]) -> Part {
        match e {
            Expr::Var(x) => scope.iter().rev().find(|s| &s.0 == x)?.1,
            Expr::Field(a, _) | Expr::Index(a, _) => part(a, scope).map(|(i, _)| (i, true)),
            _ => None,
        }
    }
    let piece = |e: &Expr, scope: &[(String, Part)]| part(e, scope).map(|(i, _)| (i, true));
    match e {
        Expr::Lit(_) | Expr::Var(_) | Expr::Score(..) => {}
        Expr::Call(f, args) => {
            if f == name {
                let strict = |(i, a)| part(a, scope) == Some((i, true));
                out.push(args.iter().enumerate().map(strict).collect());
            }
            args.iter().for_each(|a| descents(a, name, scope, out));
        }
        Expr::Let(x, a, b) => {
            descents(a, name, scope, out);
            scope.push((x.clone(), part(a, scope)));
            descents(b, name, scope, out);
            scope.pop();
        }
        Expr::Match(a, arms) => {
            descents(a, name, scope, out);
            let inside = piece(a, scope);
            for arm in arms {
                let n = scope.len();
                let names = arm.variant.iter().flat_map(|v| v.1.clone());
                scope.extend(names.map(|x| (x, inside)));
                descents(&arm.body, name, scope, out);
                scope.truncate(n);
            }
        }
        Expr::Each(_, x, list, body) => {
            descents(list, name, scope, out);
            scope.push((x.clone(), piece(list, scope)));
            descents(body, name, scope, out);
            scope.pop();
        }
        Expr::Rate(x, _, a) => {
            scope.push((x.clone(), None));
            descents(a, name, scope, out);
            scope.pop();
        }
        Expr::Unary(_, a) | Expr::Field(a, _) | Expr::Len(a) => descents(a, name, scope, out),
        Expr::Tuple(args) | Expr::Correct(_, args) | Expr::Record(_, args) | Expr::List(args) => {
            args.iter().for_each(|a| descents(a, name, scope, out))
        }
        Expr::Binary(_, a, b) | Expr::Index(a, b) => {
            descents(a, name, scope, out);
            descents(b, name, scope, out);
        }
        Expr::If(a, b, c) => {
            descents(a, name, scope, out);
            descents(b, name, scope, out);
            descents(c, name, scope, out);
        }
    }
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
        Expr::Call(_, args)
        | Expr::Tuple(args)
        | Expr::Correct(_, args)
        | Expr::Record(_, args)
        | Expr::List(args) => args.iter().for_each(|a| walk(a, f)),
        Expr::Binary(_, a, b) | Expr::Let(_, a, b) | Expr::Index(a, b) | Expr::Each(_, _, a, b) => {
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
        Expr::Call(_, args)
        | Expr::Tuple(args)
        | Expr::Correct(_, args)
        | Expr::Record(_, args)
        | Expr::List(args) => args.iter().flat_map(arms).collect(),
        Expr::Binary(_, a, b) | Expr::Let(_, a, b) | Expr::Index(a, b) | Expr::Each(_, _, a, b) => {
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
        Expr::Tuple(args) | Expr::Correct(_, args) | Expr::Record(_, args) | Expr::List(args) => {
            args.iter().flat_map(calls).collect()
        }
        Expr::Binary(_, a, b) | Expr::Let(_, a, b) | Expr::Index(a, b) | Expr::Each(_, _, a, b) => {
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
