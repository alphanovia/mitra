//! The core of Mitra, from the inside out.
//!
//! Two sides that never read each other, and one crossing:
//!
//! - [`model`]: places, pieces, opinions, and the calculus that judges
//!   every piece from what was taken. It knows nothing of values.
//! - [`value`]: a quantity with a type, the operations declared on
//!   types, and expressions of them. It knows nothing of pieces.
//! - [`standard`]: the standard types and operations. It reads `value`
//!   only.
//! - [`bounds`]: a second reading of the standard operations, on the
//!   bounds a value lies in. It reads `value` and `standard`.
//! - [`claim`]: the crossing. A claim is evaluated, a voice takes what
//!   it gave, and the model judges. It reads both sides.
//!
//! Each module is the code of the model file of the same name in
//! `model/`.

pub mod bounds;
pub mod claim;
pub mod model;
pub mod standard;
pub mod value;
