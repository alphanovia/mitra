//! The semantic model, which `parse` builds from the text.

use crate::types::set::{Set, Value};
use crate::types::tuples::Tuples;
use std::collections::BTreeMap;

/// Expressions of the spec language: a small, pure, first-order core.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements functions::Expr
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
// mitra: implements functions::Function
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
// mitra: implements claims::Target
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
// mitra: implements claims::Claim
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
// mitra: implements ledger::Judgment
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
        /// The domain as inputs, an empty pattern being every input.
        /// `None` when a helper names it: it is then not worked out.
        over: Option<Pattern>,
    },
    /// `rate(x in S, cond) = 0.4`: a measure, as measured. The evidence is
    /// then about the measure as written.
    Measured(Expr, f64),
}

/// `evidence name { for target given … holds|fails|gives v trust t }`
#[derive(Debug, Clone, PartialEq)]
// mitra: implements ledger::Evidence
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
    /// `seen "…"`: the fingerprint of its piece when a person said it.
    pub seen: Option<String>,
    pub line: usize,
    /// `act test`: what was done, when the block says it.
    pub act: Option<String>,
    /// `by "kind:…"`: who did it.
    pub by: Option<String>,
    /// `written "kind:…"`: who wrote what a machine ran.
    pub written: Option<String>,
    /// `cold`: done with the model and the code only.
    pub cold: bool,
    /// `of "test:name"`: a review of the evidence of that source, not of
    /// the claim.
    pub of: Option<String>,
}

/// What an act is: a point observed, a domain run through, a judgement.
pub const ACTS: [&str; 8] = [
    "test",
    "sample",
    "exhaustive",
    "proof",
    "review",
    "measure",
    "tie",
    "report",
];

/// How far apart two voices are, from the nearest.
pub const APART: [&str; 6] = ["same", "session", "model", "vendor", "kind", "unknown"];

/// A voice as its parts: `agent:anthropic/opus-5.5/s-8f2a` is the kind
/// `agent` and the path `anthropic`, `opus-5.5`, `s-8f2a`.
// mitra: implements voices::parts
pub fn voice_parts(voice: &str) -> Option<(&str, Vec<&str>)> {
    let (kind, rest) = voice.split_once(':')?;
    let known = ["agent", "human", "machine"].contains(&kind);
    let path: Vec<&str> = rest.split('/').collect();
    (known && path.iter().all(|p| !p.is_empty())).then_some((kind, path))
}

/// How far apart two voices are, as an index in [`APART`]: the same
/// voice, two sessions of a model, two models of a vendor, two vendors,
/// two kinds. A voice nobody named is apart from nothing: `unknown`.
// mitra: implements voices::apart
pub fn apart(a: &str, b: &str) -> usize {
    let (Some((ka, pa)), Some((kb, pb))) = (voice_parts(a), voice_parts(b)) else {
        return 5;
    };
    if ka != kb {
        return 4;
    }
    if pa == pb {
        return 0;
    }
    // A human and a machine are a name: two of them are two voices.
    if ka != "agent" {
        return 4;
    }
    match pa.iter().zip(&pb).take_while(|(x, y)| x == y).count() {
        0 => 3,
        1 => 2,
        _ => 1,
    }
}

impl Evidence {
    /// What was done: the act the block states, or the one its shape
    /// says. A sample is a sample, a measure a measure; a machine's
    /// source says a test or a tie; any other is someone's word, a review.
    // mitra: implements ledger::Evidence::act
    pub fn act(&self) -> &str {
        if let Some(a) = &self.act {
            return a;
        }
        let source = self.source.as_deref().unwrap_or("");
        match &self.judgment {
            Judgment::Sampled { .. } => "sample",
            Judgment::Measured(..) => "measure",
            _ if source.starts_with("reader:") => "tie",
            _ if source.starts_with("test:") => "test",
            Judgment::Output(_) | Judgment::Passed => "test",
            _ => "review",
        }
    }

    /// Whose word it is: who wrote what a machine ran, or who did it.
    // mitra: implements ledger::Evidence::voice
    pub fn voice(&self) -> Option<&str> {
        self.written.as_deref().or(self.by.as_deref())
    }

    /// Whether a machine observed it: a test (`test:`) or the reader of
    /// the code (`reader:`). Such evidence is a snapshot, written again
    /// by each run; any other is someone's word.
    // mitra: implements ledger::Evidence::by_machine
    pub fn by_machine(&self) -> bool {
        machine(self.source.as_deref())
    }
}

/// Whether a source names a machine: `test:…` or `reader:…`.
pub fn machine(source: Option<&str>) -> bool {
    source.is_some_and(|s| s.starts_with("test:") || s.starts_with("reader:"))
}

/// Why evidence of the ledger counts for nothing.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements ledger::Why
pub enum Why {
    /// The model has no piece of that name any more.
    Orphaned,
    /// Its piece says something else than when it was said.
    Stale { seen: String, now: String },
}

/// Evidence set aside: kept in the ledger, shown in every report,
/// judging nothing, until someone removes it or says it again.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements ledger::Aside
pub struct Aside {
    /// The piece it names, as written.
    pub about: String,
    pub source: Option<String>,
    pub rev: Option<String>,
    pub why: Why,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Decl {
    Record(usize),
    Enum(usize),
}

#[derive(Debug, Clone, PartialEq, Default)]
// mitra: implements model::Model
pub struct Model {
    /// How each record prints, kept for [`Model::named`]: the number of
    /// records it was worked out for, and the texts.
    wholes: std::cell::RefCell<(usize, std::rc::Rc<Vec<(String, String)>>)>,
    /// The claims on determinism: the claim, its function, and whether
    /// it says the same input always gives the same output (`false`:
    /// that it may not, `!deterministic(f)`).
    pub determinism: Vec<(String, String, bool)>,
    /// The trust written on a helper of the model, `fn h(…) -> T trust
    /// 0.9 { … }`: 1 where none is.
    pub helper_trust: Vec<(String, f64)>,
    pub records: Vec<Record>,
    pub enums: Vec<Enum>,
    /// The named blocks, in the order written.
    pub blocks: Vec<Block>,
    /// The evidence of the ledger that counts for nothing, and why.
    pub aside: Vec<Aside>,
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
    /// The inputs bound by a contract, `l: impl Link` or `l: L` with
    /// `L: Link`.
    pub bounds: Vec<Bounded>,
    /// The functions with an opaque input, as their bodies see them.
    pub generics: Vec<Generic>,
    /// The settings in force: what a sample is worth.
    pub settings: crate::settings::Settings,
    /// The reviews of evidence, `of "test:name"`: they judge nothing.
    pub vouches: Vec<Evidence>,
}

/// A function as its own body sees it: a type parameter, or an input
/// written `impl Contract`, is an opaque type there, known by the
/// contracts that bound it. A caller gives any value.
#[derive(Debug, Clone, PartialEq)]
pub struct Generic {
    pub function: String,
    pub params: Vec<(String, Set)>,
    pub known: Vec<(Set, Vec<String>)>,
}

/// An input of a function known only by the contracts it fills.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements model::Bounded
pub struct Bounded {
    pub function: String,
    pub input: String,
    pub contracts: Vec<String>,
}

/// `ensure name "prose";`: what a subject promises, or what a boundary
/// is promised to do. Mitra never reads the prose. It is a claim of the
/// same name, which rests on `claims`, on the requirements it needs
/// (`require other::name;`) and, in a subject, on `name::captured`:
/// someone's word that the claims say what the prose says.
/// Which side of the line a block is on: `subject`, code we built, or
/// `boundary`, an environment we bought.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// mitra: implements model::Side
pub enum Side {
    Subject,
    Boundary,
    /// `contract name { … }`: a role, filled by subjects and boundaries.
    Contract,
}

/// `subject name { … }` or `boundary name { … }`: what is declared in it
/// is named `name::…`. A block written twice is one block.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements model::Block
pub struct Block {
    pub name: String,
    pub side: Side,
    /// The store it was brought from, `import store::name::{…};`: its
    /// code is tied there, not here.
    pub from: Option<String>,
    pub line: usize,
}

impl Model {
    /// The promises of a subject that no claim of it says.
    pub fn bare_ensures(&self) -> Vec<&Ensure> {
        let bare = |e: &&Ensure| !e.boundary && e.needs.is_none() && e.claims.is_empty();
        self.ensures.iter().filter(bare).collect()
    }

    /// The block a piece is declared in, by the name of the piece.
    // mitra: implements model::Model::block_of
    pub fn block_of(&self, piece: &str) -> Option<&Block> {
        let first = piece.split("::").next()?;
        self.blocks
            .iter()
            .find(|b| b.name == first && piece != first)
    }
}

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
// mitra: implements intent::Ensure
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
    pub(crate) fn spec_of_classes(&self, f: &Function, g: Option<&str>) -> Result<(), String> {
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

    /// Each record's whole set as it prints, the records declared above
    /// it written by name, with its name. Worked out once for the records
    /// there are: a record of records prints long.
    fn wholes(&self) -> std::rc::Rc<Vec<(String, String)>> {
        let mut kept = self.wholes.borrow_mut();
        if kept.0 != self.records.len() {
            let mut all: Vec<(String, String)> = Vec::new();
            for r in self.records.iter().filter(|r| r.of.is_none()) {
                // In the order declared, as each was written when read.
                let whole = all
                    .iter()
                    .fold(r.set().to_string(), |t, o| t.replace(&o.0, &o.1));
                all.push((whole, r.name.clone()));
            }
            *kept = (self.records.len(), std::rc::Rc::new(all));
        }
        kept.1.clone()
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
        for (whole, name) in self.wholes().iter() {
            if out.contains(whole.as_str()) {
                out = out.replace(whole.as_str(), name);
            }
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
    pub(crate) fn scored(&self, kind: &str, f: &str) -> Result<(), String> {
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
pub(crate) fn arms(e: &Expr) -> Vec<(String, Vec<String>)> {
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
