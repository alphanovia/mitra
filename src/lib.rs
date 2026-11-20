//! The core of Mitra, from the inside out.
//!
//! Two sides that never read each other, and one seam:
//!
//! - [`piece`]: something said, what is believed of it, and the evidence
//!   that moves that belief. It knows nothing of values.
//! - [`value`]: what is computed, its type, and the expression that
//!   computes it. It knows nothing of pieces.
//! - [`claim`]: a piece with a statement. Computing the statement makes
//!   evidence. It reads both.
//!
//! Each module is the code of the model file of the same name in
//! `model/`.

pub mod claim;
pub mod piece;
pub mod value;
