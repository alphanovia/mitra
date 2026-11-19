//! Everything Mitra keeps on a disk, and every read and write of it on
//! the model's side.
//!
//! `mitra-lang` reads no file. This crate finds the files of a model
//! (one file, or the `.mitra/*.mitra` of a store), reads them and the
//! modules they import, and hands them over as named texts
//! ([`mitra_lang::Modules`]). It also keeps the ledger file of a store:
//! reading it, and appending to it once the new evidence was checked.

pub mod source;
pub mod store;

pub use store::Store;
