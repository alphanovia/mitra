//! Mitra's reader of Rust: the only part of Mitra that knows a language
//! of code. It reads the comments that tie code to a model
//! (`// mitra: implements …`, `// mitra: evidence for …`), compares
//! signatures, structs and calls with the model, and reads test reports.
//! What it finds is plain evidence: the language crate never depends on
//! it, and a reader of another language would sit beside it.

pub mod report;
pub mod source;
pub mod tie;
