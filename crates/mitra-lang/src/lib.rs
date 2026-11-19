//! Mitra's embeddable library.
//!
//! Parse a model (subjects, boundaries, claims) and a list of evidence,
//! then compute a verdict for each claim, and for each error the pieces
//! most plausibly wrong.
//!
//! ```
//! let model = "subject s { fn flip(a: bool) -> bool;\n\
//!              claim flip_correct(a: bool) { correct(flip(a)) } }";
//! let evidence = "evidence t { for flip holds }";
//! let report = mitra_lang::run(model, evidence).unwrap();
//! assert!(report.to_text().contains("s::flip_correct: holds\n"));
//! ```

pub mod ast;
pub mod build;
pub mod eval;
pub mod fingerprint;
pub mod levels;
pub mod modules;
pub mod next;
pub mod parse;
pub mod plan;
pub mod print;
mod prove;
pub mod report;
pub mod results;
pub mod set;
pub mod trust;
pub mod tuples;
pub mod types;

pub use ast::{
    Arm, Aside, Block, Claim, Decl, Enum, Evidence, Expr, Function, Judgment, Model, Pattern,
    Position, Record, Side, Target, Why,
};
pub use eval::{
    evaluate, Assumed, Candidate, Conflict, Contradiction, Display, Flag, Implication, Kind, Logic,
    Piece, Problem, Report, Root, Suspect, Unranked, Verdict,
};
pub use modules::{Module, Modules};
pub use next::next;
pub use parse::Handed;
pub use plan::plan;
pub use report::{explain, Files, Places};
pub use set::{Set, Value};
pub use tuples::Tuples;

use std::fmt;

/// A problem in an input file, with the 1-based line it was found on.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements reader::Error
pub struct Error {
    pub line: usize,
    pub message: String,
}

impl Error {
    pub fn new(line: usize, message: impl Into<String>) -> Self {
        Error {
            line,
            message: message.into(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "line {}: {}", self.line, self.message)
    }
}

impl std::error::Error for Error {}

/// Which input an error belongs to.
#[derive(Debug, Clone, Copy, PartialEq)]
// mitra: implements types::Input
pub enum Input {
    Model,
    Evidence,
}

/// Parse both inputs and check them against each other: a model in one
/// text, importing nothing but `std`.
// mitra: implements types::load
pub fn load(model: &str, evidence: &str) -> Result<Model, (Input, Error)> {
    load_modules(&Modules::alone(model), evidence)
}

/// Parse a model handed as several files, with its evidence, and check
/// them against each other.
// mitra: implements types::load_modules
pub fn load_modules(model: &Modules, evidence: &str) -> Result<Model, (Input, Error)> {
    if let Some((line, hint)) = parse::earlier(model.text()) {
        let msg = format!("this syntax is no longer read: {hint}");
        return Err((Input::Model, Error::new(line, msg)));
    }
    read(model, evidence)
}

fn read(model: &Modules, evidence: &str) -> Result<Model, (Input, Error)> {
    let read = parse::model_with(model.text(), &model.handed);
    let (mut m, names) = read.map_err(|e| (Input::Model, e))?;
    // A claim follows from the specs of the functions it calls, when
    // their claims are above it.
    let above: Vec<String> = m.claims.iter().map(|c| c.name.clone()).collect();
    for (i, c) in m.claims.iter_mut().enumerate() {
        c.trust = names.trust.get(&c.name).copied();
        let specs = names.derived.get(&c.name).into_iter().flatten();
        let specs: Vec<String> = specs.filter(|s| above[..i].contains(s)).cloned().collect();
        if c.from.is_empty() && !specs.is_empty() {
            (c.from, c.auto) = (specs, true);
        }
    }
    let outside = |f: &&Function| !f.is_spec() && names.boundaries.contains(&f.name);
    m.boundaries = m
        .functions
        .iter()
        .filter(outside)
        .map(|f| f.name.clone())
        .collect();
    for (name, f, set, line) in &names.represented {
        m.claims.push(Claim {
            name: name.clone(),
            func: f.clone(),
            pattern: Vec::new(),
            target: Target::Represented(set.clone()),
            from: Vec::new(),
            auto: false,
            proved: false,
            trust: names.trust.get(name).copied(),
            line: *line,
        });
    }
    let lines = evidence.lines().count();
    let read = parse::evidence(evidence, &m, &names).map_err(|e| (Input::Evidence, e))?;
    let (mut read, aside) = read;
    m.aside = aside;
    // Said by a person of a piece that reads otherwise today: stale.
    let prints = fingerprint::all(&m);
    read.retain(|e| {
        let now = fingerprint::of(&prints, &e.about);
        let stale = match (&e.seen, now) {
            (Some(seen), Some(now)) if seen != now => Some((seen.clone(), now.clone())),
            _ => None,
        };
        if let Some((seen, now)) = &stale {
            let (about, source, rev) = (e.about.clone(), e.source.clone(), e.rev.clone());
            let why = Why::Stale {
                seen: seen.clone(),
                now: now.clone(),
            };
            m.aside.push(Aside {
                about,
                source,
                rev,
                why,
                line: e.line,
            });
        }
        stale.is_none()
    });
    tie(&mut m, &read, &names);
    tie_records(&mut m, &read);
    tie_uses(&mut m, &read, &names);
    m.evidence = read;
    points(&mut m);
    invariants(&mut m);
    determinism(&mut m);
    let own = |s: &str| s.lines().map(String::from).collect::<Vec<_>>();
    let mut written = own(evidence);
    written.truncate(lines);
    m.written = Some((own(model.text()), written));
    let mut m = m.check()?;
    prove::derive(&mut m);
    Ok(m)
}

/// What evidence on a point claim says of the code. A test tied to
/// `claim t() { f(6, 2) == 3 }` that passes is the output `f(6, 2) = 3`;
/// failing, it refutes the specs that give 3 there. On
/// `claim t() { correct(f(6, 2)) }` it judges that call.
fn points(m: &mut Model) {
    let mut more = Vec::new();
    for e in &m.evidence {
        let (Judgment::Holds(ok), true) = (&e.judgment, e.pattern.iter().all(|p| p.set.is_none()))
        else {
            continue;
        };
        // An assumption is nobody's observation.
        if e.source.as_deref() == Some("assumed") {
            continue;
        }
        let Some(c) = m
            .claim(&e.about)
            .filter(|c| c.pattern.iter().all(|p| p.var.is_none()))
        else {
            continue;
        };
        let on = |about: &str, pattern: &Pattern, judgment: Judgment| Evidence {
            about: about.to_string(),
            pattern: pattern.clone(),
            judgment,
            ..e.clone()
        };
        let judged = |f: &str| {
            let covers = |d: &Claim| d.func == f && d.target == Target::Correct;
            m.claims.iter().any(covers)
        };
        match &c.target {
            Target::Correct if !c.pattern.is_empty() || m.function(&c.func).is_some() => {
                more.push(on(&c.func, &c.pattern, Judgment::Correct(*ok)));
            }
            Target::Property(Expr::Binary("==", l, r)) => {
                let side = [(l, r), (r, l)].into_iter().find_map(|(call, other)| {
                    let Expr::Call(f, args) = &**call else {
                        return None;
                    };
                    let g = m.declared_function(f).filter(|g| !g.is_spec())?;
                    let x: Option<Vec<Value>> =
                        args.iter().map(|a| eval::computed(m, a, &[])).collect();
                    Some((g, x?, eval::computed(m, other, &[])?))
                });
                let Some((g, x, v)) = side else {
                    continue;
                };
                let at: Pattern = x
                    .iter()
                    .map(|v| Position {
                        var: None,
                        set: Some(Set::point(v)),
                    })
                    .collect();
                let other = match &v {
                    Value::Bool(b) if g.range == Set::bool() => Some(Value::Bool(!b)),
                    _ => None,
                };
                match (ok, other) {
                    (true, _) => more.push(on(&g.name, &at, Judgment::Output(v.clone()))),
                    // Not this output: with two possible, the other one.
                    (false, Some(other)) => more.push(on(&g.name, &at, Judgment::Output(other))),
                    (false, None) => {
                        // The specs that give this output there are refuted.
                        for d in m.claims.iter().filter(|d| d.func == g.name) {
                            let Target::Matches(target) = &d.target else {
                                continue;
                            };
                            let names: Vec<(String, Value)> = d
                                .pattern
                                .iter()
                                .zip(&x)
                                .filter_map(|(p, v)| Some((p.var.clone()?, v.clone())))
                                .collect();
                            let inside = m.covers(&d.func, &d.pattern).contains(&x);
                            if inside && eval::computed(m, target, &names).as_ref() == Some(&v) {
                                more.push(on(&d.name, &at, Judgment::Holds(false)));
                            }
                        }
                    }
                }
                if judged(&g.name) {
                    more.push(on(&g.name, &at, Judgment::Correct(*ok)));
                }
            }
            _ => {}
        }
    }
    m.evidence.extend(more);
}

/// A claim on determinism is judged on the outputs observed: two
/// different outputs on one input say the function is not deterministic.
fn determinism(m: &mut Model) {
    let mut more = Vec::new();
    for (claim, f, same) in &m.determinism {
        let seen = |e: &Evidence| match (&e.judgment, ast::single(&e.pattern)) {
            (Judgment::Output(v), Some(x)) if &e.about == f => Some((x, v.clone(), e.line)),
            _ => None,
        };
        let seen: Vec<(Vec<Value>, Value, usize)> = m.evidence.iter().filter_map(seen).collect();
        let differ = seen.iter().enumerate().find_map(|(n, a)| {
            let other = seen[..n].iter().find(|b| b.0 == a.0 && b.1 != a.1)?;
            Some((other.2, a.2))
        });
        if let Some((_, second)) = differ {
            let e = m.evidence.iter().find(|e| e.line == second).expect("seen");
            more.push(Evidence {
                about: claim.clone(),
                pattern: Vec::new(),
                judgment: Judgment::Holds(!same),
                ..e.clone()
            });
        }
    }
    m.evidence.extend(more);
}

/// A claim on a record, `claim c(r: R) { … }` calling no code, is true of
/// every `R` the code builds: an output observed on a function that
/// returns an `R` is evidence for or against it, on that value.
fn invariants(m: &mut Model) {
    let mut more = Vec::new();
    for c in &m.claims {
        let (Target::Property(statement), true) = (&c.target, m.invariant(c)) else {
            continue;
        };
        let [Position {
            var: Some(name),
            set: Some(of),
        }] = c.pattern.as_slice()
        else {
            continue;
        };
        for e in &m.evidence {
            let Judgment::Output(v) = &e.judgment else {
                continue;
            };
            let built = m
                .declared_function(&e.about)
                .is_some_and(|g| &g.range == of);
            if !built || !of.contains(v) {
                continue;
            }
            let Some(Value::Bool(ok)) = eval::computed(m, statement, &[(name.clone(), v.clone())])
            else {
                continue;
            };
            more.push(Evidence {
                about: c.name.clone(),
                pattern: vec![Position {
                    var: None,
                    set: Some(Set::point(v)),
                }],
                judgment: Judgment::Holds(ok),
                ..e.clone()
            });
        }
    }
    m.evidence.extend(more);
}

/// A tie is a claim once someone brings evidence on it.
fn tie(m: &mut Model, evidence: &[Evidence], names: &parse::Names) {
    for f in parse::tied(evidence, names) {
        let line = m.function(&f).map_or(0, |f| f.line);
        m.claims.push(Claim {
            name: format!("{f}::tied"),
            func: f,
            pattern: Vec::new(),
            target: Target::Tied,
            from: Vec::new(),
            auto: false,
            proved: false,
            trust: None,
            line,
        });
    }
}

/// A function that states no `uses` is taken to call nothing outside its
/// own subject. The claim exists once a reader of the code speaks of it.
fn tie_uses(m: &mut Model, evidence: &[Evidence], names: &parse::Names) {
    for f in &names.opaque {
        let name = format!("{f}::uses");
        let spoken = evidence.iter().any(|e| e.about == name);
        if !spoken || m.claim(&name).is_some() {
            continue;
        }
        m.claims.push(Claim {
            name,
            func: f.clone(),
            pattern: Vec::new(),
            target: Target::Uses {
                used: Vec::new(),
                only: true,
            },
            from: Vec::new(),
            auto: false,
            proved: false,
            trust: None,
            line: m.function(f).map_or(0, |f| f.line),
        });
    }
}

/// The same for a record and its struct: the claim is held by a hidden
/// function with no input, as the text of a requirement is.
fn tie_records(m: &mut Model, evidence: &[Evidence]) {
    let spoken = |r: &&Record| {
        evidence
            .iter()
            .any(|e| e.about == format!("{}::tied", r.name))
    };
    let all = m.records.iter().filter(|r| r.of.is_none());
    let mut all: Vec<(String, usize)> = all
        .filter(spoken)
        .map(|r| (r.name.clone(), r.line))
        .collect();
    // And for an enum and the enum of the code.
    let said = |e: &&Enum| {
        evidence
            .iter()
            .any(|x| x.about == format!("{}::tied", e.name))
    };
    all.extend(
        m.enums
            .iter()
            .filter(said)
            .map(|e| (e.name.clone(), e.line)),
    );
    for (name, line) in all {
        let func = format!("{name}::tied::prose");
        m.functions.push(Function {
            name: func.clone(),
            params: Vec::new(),
            range: Set::bool(),
            body: None,
            written: None,
            line,
        });
        m.claims.push(Claim {
            name: format!("{name}::tied"),
            func,
            pattern: Vec::new(),
            target: Target::Tied,
            from: Vec::new(),
            auto: false,
            proved: false,
            trust: None,
            line,
        });
    }
}

/// Parse new evidence for a model, in the syntax the model is written in.
/// Nothing is checked against the model beyond its names.
pub fn read_evidence(model: &Modules, evidence: &str) -> Result<Vec<Evidence>, Error> {
    let (m, names) = parse::model_with(model.text(), &model.handed)?;
    parse::evidence_alone(evidence, &m, &names)
}

/// Parse both inputs, check them against each other, and evaluate.
// mitra: implements verdicts::run
pub fn run(model: &str, evidence: &str) -> Result<Report, (Input, Error)> {
    evaluate(&load(model, evidence)?)
}
