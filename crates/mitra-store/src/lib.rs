//! Everything Mitra keeps on a disk, on the model's side: the project's
//! root and its `.mitra` files (`project`), `mitra.toml` (`settings`).
//! `mitra-lang` reads no file: this crate hands it texts.

pub mod project;
pub mod settings;

/// The store's file: its text, read with `mitra_lang::store`.
pub use project::read;
