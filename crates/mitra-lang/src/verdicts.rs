//! From evidence to verdicts. Every piece of evidence counts, whatever its
//! bias: the bias only ranks what is most plausibly wrong when the model
//! and the evidence can't all be true.
//!
//! `evaluate` is the whole pipeline; each step is a file of `verdicts/`.

mod causes;
mod classes;
pub mod culprits;
mod evidence;
mod problems;
pub(crate) mod prove;
mod rank;
mod verdict;

pub use classes::class_text;
pub(crate) use classes::POINTS;
pub(crate) use rank::{brief, QUOTED, WIDE};
pub(crate) use verdict::represented;

use crate::eval::Eval;
use crate::model::*;
use crate::types::set::Value;
use crate::types::tuples::Tuples;
use crate::{Error, Input};
use std::collections::HashMap;

///
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
// mitra: implements verdicts::Logic
pub enum Logic {
    /// Evidence covers every input, directly or through claims that hold.
    Holds,
    /// The same, if the claims in `Verdict::assuming` hold: they are
    /// unverified.
    Assuming,
    /// Some inputs are missed.
    Unverified,
    /// Evidence refutes the claim.
    Fails,
}

impl Logic {
    // mitra: implements report::Logic::as_str
    pub fn as_str(self) -> &'static str {
        match self {
            Logic::Holds => "holds",
            Logic::Assuming => "holds assuming",
            Logic::Unverified => "unverified",
            Logic::Fails => "fails",
        }
    }
}

/// The status of the implication behind a `from`: premises ⇒ claim.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Implication {
    /// Substituting the premises into the claim gives its target on every
    /// input: the implication holds by computation.
    Checked,
    /// Mitra could not compute it (no shape, an infinite domain, or a
    /// `correct` claim). It is taken as stated by the author.
    Unchecked,
    /// Substituting the premises gives a different result on some input: the
    /// premises imply something else, so the derivation is not used.
    Refuted,
}

impl Implication {
    pub fn as_str(self) -> &'static str {
        match self {
            Implication::Checked => "checked",
            Implication::Unchecked => "unchecked",
            Implication::Refuted => "refuted",
        }
    }
}

/// The verdict on one claim.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements verdicts::Verdict
pub struct Verdict {
    /// The inputs a tested one stands for, with the `represented` claim
    /// that says so.
    pub stood: Vec<(String, Tuples)>,
    /// An input where the specs of the functions the claim calls say it
    /// is false: the model contradicts itself there.
    pub against: Option<Vec<Value>>,
    pub claim: String,
    pub logic: Logic,
    /// With `Logic::Assuming`: the unverified claims the verdict rests on.
    pub assuming: Vec<String>,
    /// The claim's inputs that evidence covers directly.
    pub tested: Tuples,
    /// The inputs that follow from the premises of the claim.
    pub derived: Tuples,
    /// With premises: the implication is not refuted and no premise
    /// fails.
    pub follows: bool,
    /// What a `uniform` claim on the subject adds, if there is one.
    pub assumed: Option<Assumed>,
    /// Why no test stands for other inputs, when the claim calls subjects
    /// and a `uniform` claim can't be used.
    pub unsorted: Option<String>,
    /// The inputs nothing covers.
    pub missed: Tuples,
    /// The claim's inputs where it is known to fail and not contradicted.
    pub refuted: Tuples,
    /// The inputs the claim is about.
    pub domain: Tuples,
    /// For a claim with `from`.
    pub implication: Option<Implication>,
}

/// What a `uniform` claim adds to another claim on the same subject: an
/// input that passed stands for its whole class.
#[derive(Debug, Clone, PartialEq)]
pub struct Assumed {
    /// The `uniform` claim, and its spec, whose values are the classes.
    /// Without a spec there is one class, of every input.
    pub by: String,
    pub spec: Option<String>,
    /// For a claim that calls subjects: the calls its inputs are sorted
    /// by, one class each, and the other `uniform` claims relied on.
    pub calls: Vec<String>,
    pub with: Vec<String>,
    /// Those of the `uniform` claims that no evidence vouches for.
    pub unvouched: Vec<String>,
    /// Evidence covers the `uniform` claim: then a failure stands for
    /// its class too.
    pub vouched: bool,
    /// The `uniform` claim does not fail: then `set` counts, under the
    /// assumption that it holds when it is not vouched for.
    pub usable: bool,
    /// A failure over a set too large to sort into classes: no class is
    /// relied on.
    pub blind: bool,
    /// The classes left out because one of their inputs has both a pass
    /// and a failure.
    pub tainted: Vec<Value>,
    /// The classes with an input that passed.
    pub classes: Vec<Value>,
    /// The classes the claim needs that have none.
    pub untested: Vec<Value>,
    /// The classes with an input that failed.
    pub failing: Vec<Value>,
    /// The inputs of the claim's domain that the classes stand for, when
    /// Mitra can write them as a set: all of them on a finite domain; on an
    /// infinite one, the rest of the domain once no class is untested.
    pub set: Tuples,
}

/// Knowledge that can't all be true: a `uniform` claim, and a pass and a
/// failure in one of its classes.
#[derive(Debug, Clone, PartialEq)]
pub struct Conflict {
    pub claim: String,
    pub spec: Option<String>,
    pub class: Value,
    /// For a claim that calls subjects: that claim, the calls its inputs
    /// are sorted by, and the other `uniform` claims relied on.
    pub on: Option<String>,
    pub calls: Vec<String>,
    pub with: Vec<String>,
    /// The input, its evidence line and the evidence as text.
    pub pass: (Vec<Value>, usize, String),
    pub fail: (Vec<Value>, usize, String),
    /// Every passing and every failing input of the class, with its line.
    pub passes: Vec<(Vec<Value>, usize)>,
    pub fails: Vec<(Vec<Value>, usize)>,
}

/// A subject that uses, directly or through others, a subject with a
/// failing claim: the place to look first.
#[derive(Debug, Clone, PartialEq)]
pub struct Suspect {
    pub subject: String,
    pub uses: String,
    /// The failing claims on `uses`.
    pub failing: Vec<String>,
}

/// A pass and a failure on the same input: they can't both be true.
#[derive(Debug, Clone, PartialEq)]
pub struct Contradiction {
    pub subject: String,
    /// The inputs both are about, as text.
    pub inputs: String,
    /// The evidence line and the evidence as text.
    pub pass: (usize, String),
    pub fail: (usize, String),
}

/// A failing subject whose `only uses` claim holds, while nothing fails
/// in what it uses: the fault is its own.
#[derive(Debug, Clone, PartialEq)]
pub struct Root {
    pub subject: String,
    /// The `only uses` claim.
    pub by: String,
    pub used: Vec<String>,
    /// Every claim on the subjects used holds. Otherwise the fault may
    /// also be where no claim covers them.
    pub covered: bool,
}

impl Root {
    pub fn text(&self) -> String {
        let used: Vec<String> = self.used.iter().map(|g| format!("`{g}`")).collect();
        format!(
            "`{}` fails and only uses {}, where nothing fails: the fault is in `{}` itself{}",
            self.subject,
            used.join(", "),
            self.subject,
            if self.covered {
                ""
            } else {
                ", or where no claim covers them"
            }
        )
    }
}

/// Evidence Mitra cannot use as given.
#[derive(Debug, Clone, PartialEq)]
pub struct Flag {
    pub line: usize,
    pub message: String,
}

/// Something that may be wrong.
///
#[derive(Debug, Clone, PartialEq)]
// mitra: implements report::Piece
pub enum Piece {
    /// A ledger entry, by its line.
    Evidence(usize),
    /// The code of a subject.
    Code(String),
    /// A claim as stated: the statement itself may be the wrong piece.
    Claim(String),
    /// The signature of a function: its inputs and its result type.
    Signature(String),
    /// An `assume`, by the name of its claim: taken as true, it may be
    /// what is wrong.
    Assumption(String),
    /// The `uses` clause of a function: what its code is declared to call.
    Uses(String),
    /// A requirement in prose (`ensure`), by its name: the prose itself
    /// may be what is wrong, or no longer what is wanted.
    Requirement(String),
    /// That the claims of a subject say what the prose of a requirement
    /// says: someone's word, by the name of the requirement.
    Capture(String),
    /// A helper of the model, as written: a definition a claim reads. It
    /// has no code to drift from, and may still be the wrong definition.
    Helper(String),
    /// That a function of the model and its code agree: the reader's
    /// word on names and types, and the model's arithmetic and types
    /// taken to be the code's.
    Tie(String),
}

impl Piece {
    /// The claim an assumption or a `uses` clause is stated as.
    pub fn declared_claim(&self) -> Option<String> {
        match self {
            Piece::Assumption(c) => Some(c.clone()),
            Piece::Uses(f) => Some(format!("{f}::uses")),
            _ => None,
        }
    }
}

/// How plausibly a piece is the wrong one.
///
#[derive(Debug, Clone, PartialEq)]
// mitra: implements report::Candidate
pub struct Candidate {
    pub piece: Piece,
    pub belief: f64,
    pub plausibility: f64,
}

/// Why the candidates of a problem are not ranked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Unranked {
    /// Too many pieces with a bias strictly between 0 and 1, counting
    /// those trusted at 1 when they can't all be right.
    TooMany,
}

/// An error: a claim that fails, or pieces that can't all be true.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements verdicts::Problem
pub struct Problem {
    pub kind: Kind,
    /// The evidence failing the model: line and text.
    pub evidence: Vec<(usize, String)>,
    /// Sets of pieces that can't all be right.
    pub clauses: Vec<Vec<Piece>>,
    /// The pieces, most plausibly wrong first.
    pub candidates: Result<Vec<Candidate>, Unranked>,
    /// How much the biases say that nothing is wrong.
    pub conflict: f64,
}

/// What is not possible.
#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    /// Claims on a subject or a spec that evidence refutes. Several claims
    /// failing on the same evidence share one error.
    Fails { claims: Vec<String> },
    /// A `uniform` or `uses` claim that evidence denies.
    Denied { claim: String },
    /// Two pieces of evidence that disagree on the same inputs.
    Contradiction { subject: String, inputs: String },
    /// A measurement that the observations, or another measurement of the
    /// same measure, can't agree with.
    Measure { measure: String, counted: String },
    /// A pass and a failure in one class of a `uniform` claim.
    Conflict {
        claim: String,
        spec: Option<String>,
        class: Value,
        calls: Vec<String>,
    },
    /// Evidence that does not fit the signature of its function.
    Misfit { function: String, why: String },
    /// A claim that the specs of the functions it calls contradict.
    Against {
        claim: String,
        specs: Vec<String>,
        input: String,
    },
}

impl Kind {
    /// The stable code of this kind of error.
    pub fn code(&self) -> &'static str {
        match self {
            Kind::Fails { .. } => "M001",
            Kind::Denied { .. } => "M002",
            Kind::Contradiction { .. } => "M003",
            Kind::Conflict { .. } => "M004",
            Kind::Measure { .. } => "M005",
            Kind::Misfit { .. } => "M006",
            Kind::Against { .. } => "M007",
        }
    }
}

/// How much of an error is shown.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Display {
    /// The most candidates listed per error.
    pub max: usize,
    /// Candidates under this plausibility are hidden.
    pub threshold: f64,
}

impl Default for Display {
    fn default() -> Self {
        Display {
            max: 5,
            threshold: 0.05,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
// mitra: implements verdicts::Report
pub struct Report {
    pub verdicts: Vec<Verdict>,
    pub problems: Vec<Problem>,
    pub flags: Vec<Flag>,
    pub contradictions: Vec<Contradiction>,
    pub conflicts: Vec<Conflict>,
    pub suspects: Vec<Suspect>,
    pub roots: Vec<Root>,
    /// The model and the ledger the report is about, to print it.
    pub model: Model,
}

/// Why judging stopped: the input at fault and the error.
pub(crate) type Fail = (Input, Error);

pub(crate) fn model_err(e: Error) -> Fail {
    (Input::Model, e)
}

/// What the steps of `evaluate` share: each step reads what the steps
/// before it left here.
pub(crate) struct Judge<'a> {
    pub(crate) model: &'a Model,
    pub(crate) ev: Eval<'a>,
    pub(crate) flags: Vec<Flag>,
    pub(crate) contradictions: Vec<Contradiction>,
    /// Inputs seen with two outputs: neither is relied on.
    pub(crate) split: Vec<(String, Vec<Value>)>,
    /// Per claim, what the evidence says of it.
    pub(crate) direct: Vec<evidence::Direct>,
    /// Per claim, how its inputs are sorted into classes: one class per
    /// call of a subject, each by that subject's `uniform` claim.
    pub(crate) sorting: Vec<classes::Sorted>,
    pub(crate) conflicts: Vec<Conflict>,
    pub(crate) stands: Vec<verdict::Stand>,
    pub(crate) logic_of: HashMap<&'a str, Logic>,
    pub(crate) assuming_of: HashMap<&'a str, Vec<String>>,
    pub(crate) verdicts: Vec<Verdict>,
    /// Per claim, the evidence lines of failures that refute it through a
    /// class, and the `uniform` claims relied on.
    pub(crate) through: Vec<(Vec<usize>, Vec<usize>)>,
}

// mitra: implements verdicts::evaluate
pub fn evaluate(model: &Model) -> Result<Report, (Input, Error)> {
    let mut j = Judge::new(model);
    // 1. What the evidence says about each claim.
    j.split();
    j.direct()?;
    // 2. Uniformity: a class with a pass and a failure is a conflict.
    j.sorting = (0..model.claims.len())
        .map(|i| classes::sorted(model, i))
        .collect();
    j.conflicts = j.conflicts()?;
    // 3. Verdicts, each on the verdicts of the claims before it.
    j.stands = j.stands();
    j.verdicts()?;
    // 4. Errors: each failing claim, and each set of pieces that can't
    // all be true, with the pieces that may be the wrong one.
    let problems = j.problems()?;
    // 5. Suspects and root causes.
    let suspects = j.suspects();
    let roots = j.roots(&suspects);
    Ok(Report {
        roots,
        model: model.clone(),
        problems,
        verdicts: j.verdicts,
        flags: j.flags,
        contradictions: j.contradictions,
        conflicts: j.conflicts,
        suspects,
    })
}

impl Report {
    /// The claims that are unverified, or hold only under an assumption.
    pub fn warnings(&self) -> usize {
        // A named need says what its promise says: no warning of its own.
        let need = |v: &Verdict| {
            let e = self.model.ensure(&v.claim);
            v.logic == Logic::Assuming && e.is_some_and(|e| e.needs.is_some())
        };
        let warns = |v: &&Verdict| {
            matches!(v.logic, Logic::Unverified | Logic::Assuming) && !v.own_word() && !need(v)
        };
        self.verdicts.iter().filter(warns).count()
            + self.flags.len()
            + self.model.lints.len()
            + self.model.aside.len()
    }
}

impl Verdict {
    /// An assumption on a boundary that rests on nothing but itself: the
    /// model says so on purpose.
    pub fn own_word(&self) -> bool {
        self.logic == Logic::Assuming && self.assuming == [self.claim.clone()]
    }
}
