//! The core of Mitra, from the inside out.
//!
//! Two sides that never read each other, and one seam:
//!
//! - [`model`]: places, pieces, and the three kinds of piece (evidence,
//!   statements, ties), put together. It knows nothing of values.
//! - [`value`]: a quantity with a type, the operations declared on
//!   types, and expressions of them. It knows nothing of pieces, and
//!   has no type and no operation of its own.
//! - [`standard`]: the standard types and operations, and what applies
//!   them. It reads `value` only.
//! - [`bounds`]: a second reading of the standard operations, on the
//!   bounds a value lies in. It reads `value` and `standard`.
//! - [`claim`]: a statement whose content is an expression. Computing
//!   it makes evidence. It reads both.
//!
//! Each module is the code of the model file of the same name in
//! `model/`.

pub mod bounds;
pub mod claim;
pub mod model;
pub mod standard;
pub mod value;
