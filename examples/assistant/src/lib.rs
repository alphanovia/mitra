//! The personal assistant: a local server and a CLI, with fake models.
//!
//! The model is in the `.mitra` files beside each Rust file. Every model
//! machine (the frontier LLM, our local LLM, speech, the tools, the
//! accounts) is played by `fake`, which tests script. No inference runs.

pub mod cli;
pub mod fake;
pub mod harness;
pub mod server;
pub mod vocabulary;
