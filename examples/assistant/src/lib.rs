//! The personal assistant: a local server and a CLI, with fake models.
//!
//! The model is in the `.mitra` files beside each Rust file. Every model
//! machine (the frontier LLM, our local LLM, speech, the tools, the
//! accounts) is played by `fake`, which tests script. A machine that is
//! not scripted may be answered by a real model through Ollama, when it
//! is set up (see `ollama`); else the fake's rules answer.

pub mod cli;
pub mod fake;
pub mod harness;
pub mod ollama;
pub mod server;
pub mod vocabulary;
pub mod wire;
