//! Checks on Mitra's grammar. Not the production parser (that is
//! `mitra_core::parse`).
//!
//! - [`lr`]: the reference grammar as LALRPOP. It builds only if the grammar
//!   has no conflicts, i.e. is LR(1).
//! - [`ll1`]: the same grammar, left-factored, checked LL(1).
//! - [`gen`]: random valid programs, written from the grammar.

pub mod gen;
pub mod ll1;

use lalrpop_util::lalrpop_mod;

lalrpop_mod!(
    #[allow(clippy::all, unused)]
    pub lr,
    "/mitra.rs"
);

/// Does the reference grammar accept this model file?
pub fn accepts_model(src: &str) -> Result<(), String> {
    lr::ModelParser::new().parse(src).map_err(|e| e.to_string())
}

/// Does the reference grammar accept this evidence file?
pub fn accepts_evidence(src: &str) -> Result<(), String> {
    lr::EvidencesParser::new()
        .parse(src)
        .map_err(|e| e.to_string())
}
