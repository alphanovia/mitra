//! Mitra, what is frozen. Each module is one item of `docs/frozen.md`.
//!
//! On the experiment branch `core2`: `expression` (to freeze), and the
//! two standards on it, `simulation` and `judgement` (`docs/core2.md`),
//! `model`, what is written about a system (`docs/model.md`), `check`,
//! whether it is well formed, `catalogue`, the codes of what Mitra
//! reports (`docs/reference`), and `tactic`, what writes proofs for
//! judgement to check. `parse` and `print` read and write the text of a
//! model (`docs/syntax.md`), and `reader` reads ties and tests from Rust
//! code and a store of what people saw and said: the ledger, which
//! `evidence` holds and checks against a model. `agree` compares what
//! the model says twice, with no evidence.

pub mod agree;
pub mod catalogue;
pub mod check;
pub mod evidence;
pub mod expression;
mod graph;
pub mod judgement;
pub mod location;
pub mod model;
pub mod parse;
pub mod print;
pub mod reader;
pub mod simulation;
pub mod tactic;
