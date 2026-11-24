//! Mitra's reader of Rust and cargo: the only crate that knows a
//! language of code. `reader` reads ties and tests from Rust code and
//! what `cargo test` printed; `run` runs the tests and finds the files.
//! What it finds is plain evidence: `mitra-lang` never depends on it.

pub mod reader;
pub mod run;
