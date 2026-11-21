//! The side of trust (`model/model.mitra`). It reads nothing of the
//! other side.
//!
//! - [`place`]: a location in the system;
//! - [`piece`]: something said, right or wrong, and the opinion of it;
//! - the three kinds of piece: [`evidence`], [`statement`], [`tie`];
//! - here, `model`: all the pieces put together, and how they stand to
//!   each other. What is read of them is in `verdict`.

pub mod evidence;
pub mod piece;
pub mod place;
pub mod statement;
pub mod tie;

pub use evidence::{reliable, Evidence, Reliable, Voice};
pub use piece::{no, nothing, piece_id, projected, yes, Opinion, Piece};
pub use place::{within, Place};
pub use statement::Statement;
pub use tie::{complete, concerned, places, whole, Complete, Tie};

/// How pieces stand to each other, named after what is computed.
// mitra: implements model::Relation
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Relation {
    /// The first is all of the others together: believed as their
    /// product; a yes about it flows down to each, a no is shared.
    All,
    /// The first is evidence about the second: what it says counts as
    /// far as it is believed, and is fused with what else is said of
    /// that piece. Implied by each piece of evidence, see [`about`].
    About,
    /// The second is the first at given inputs: a claim, the claim at
    /// an input, and when there is one the piece that says the inputs
    /// behave alike. Read by `verdict` only.
    Instance,
}

// mitra: implements model::Related
#[derive(Clone, Debug, PartialEq)]
pub struct Related {
    pub how: Relation,
    pub pieces: Vec<Piece>,
}

/// All that is said of a system: the evidence, the relations between
/// pieces, the base rate of each piece that has one, the ties, and the
/// settings it is read under. A piece with no base rate, written or
/// set, has 0. The settings are given: the core holds no value for
/// them.
///
/// A relation is itself said: a statement whose content is the
/// relation. What is computed on it counts as far as that statement is
/// believed; one believed at 1 is plain structure.
///
/// Each tie implies its completeness, a part of every whole the tie is
/// a part of, and its whole: the tie and its statements together,
/// which is what an observation at the place is about.
// mitra: implements model::Model
#[derive(Clone, Debug, PartialEq)]
pub struct Model {
    pub evidence: Vec<Evidence>,
    pub related: Vec<Statement<Related>>,
    pub base: Vec<(Piece, f64)>,
    pub ties: Vec<Tie>,
    pub settings: Settings,
}

/// What a model is read under, apart from what it says: the weight of
/// a base rate, in counts, and the base rate of each kind of piece
/// when nobody wrote one. The core holds no value for any of them.
// mitra: implements model::Settings
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Settings {
    /// The weight of a base rate, in counts.
    pub weight: f64,
    /// The base rate of a statement: any piece that is none of the
    /// kinds below.
    pub statement: f64,
    /// The base rate of a piece of evidence.
    pub evidence: f64,
    /// The base rate of a tie.
    pub tie: f64,
    /// The base rate of the completeness of a tie.
    pub completeness: f64,
    /// The base rate of the reliability of a voice. A voice sure to be
    /// reliable, at 1, is not judged: its word counts whole.
    pub reliability: f64,
}

impl Model {
    /// Nothing said yet, under the two settings.
    pub fn empty(settings: Settings) -> Model {
        Model {
            evidence: Vec::new(),
            related: Vec::new(),
            base: Vec::new(),
            ties: Vec::new(),
            settings,
        }
    }
}

/// The relation a piece of evidence implies: it is about the piece it
/// speaks of. It is read from the evidence itself.
// mitra: implements model::about
pub fn about(e: &Evidence) -> Related {
    Related {
        how: Relation::About,
        pieces: vec![e.piece.clone(), e.about.clone()],
    }
}

/// The relation a tie implies: its whole is all of the tie and its
/// statements. It is structure, believed at 1.
fn implied(t: &Tie) -> Statement<Related> {
    let about = whole(t);
    let mut pieces = vec![about.clone(), t.piece.clone()];
    pieces.extend(t.statements.iter().cloned());
    Statement {
        piece: Piece::named(&format!("all: {}", about.name)),
        content: Related {
            how: Relation::All,
            pieces,
        },
    }
}

/// The model with the relation of each tie written out.
// mitra: implements model::written
pub fn written(model: &Model) -> Model {
    let mut out = model.clone();
    for t in &model.ties {
        let relation = implied(t);
        if !out.related.iter().any(|said| said.piece == relation.piece) {
            out.related.push(relation);
        }
    }
    out
}

/// The parts of a whole: the pieces the relation names after the
/// first, and the completeness of each tie among them.
// mitra: implements model::parts
pub fn parts(model: &Model, r: &Related) -> Vec<Piece> {
    let mut found: Vec<Piece> = r.pieces.iter().skip(1).cloned().collect();
    for t in &model.ties {
        let implied = complete(t).piece;
        if r.pieces[1..].contains(&t.piece) && !found.contains(&implied) {
            found.push(implied);
        }
    }
    found
}

/// What to expect of a piece with no evidence: its base rate. With
/// none written, it is the setting for its kind: the completeness of a
/// tie, a tie, the reliability of a voice, a piece of evidence, and a
/// statement for any other. The relation a tie implies is structure,
/// at 1.
// mitra: implements model::base
pub fn base(model: &Model, x: &Piece) -> f64 {
    if let Some((_, a)) = model.base.iter().find(|(p, _)| p == x) {
        return *a;
    }
    let set = &model.settings;
    if model.ties.iter().any(|t| &complete(t).piece == x) {
        return set.completeness;
    }
    if model.ties.iter().any(|t| &implied(t).piece == x) {
        return 1.0;
    }
    if model.ties.iter().any(|t| &t.piece == x) {
        return set.tie;
    }
    if model.evidence.iter().any(|e| &reliable(&e.by).piece == x) {
        return set.reliability;
    }
    match model.evidence.iter().any(|e| &e.piece == x) {
        true => set.evidence,
        false => set.statement,
    }
}
