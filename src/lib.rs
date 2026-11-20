//! The core of Mitra, from the inside out.
//!
//! Two sides that never read each other, and one seam:
//!
//! - [`model`]: places, pieces, and the three kinds of piece (evidence,
//!   statements, ties), put together. It knows nothing of values.
//! - [`value`]: what is computed, its type, and the expression that
//!   computes it. It knows nothing of pieces.
//! - [`claim`]: a statement whose content is an expression. Computing
//!   it makes evidence. It reads both.
//!
//! Each module is the code of the model file of the same name in
//! `model/`.

pub mod claim;
pub mod model;
pub mod value;
