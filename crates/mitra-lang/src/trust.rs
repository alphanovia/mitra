//! The trust calculus Mitra runs: ranking the pieces of a conflict
//! ([`culprits`]). Its model is aside, in `.mitra/culprits.mitra`.
//!
//! Opinions on one question (`Mass`, Dempster's and Yager's rules) were
//! written here and never called by the evaluation: they were removed on
//! 2026-11-19 and are in the git history. They come back when the trust
//! model asks for them.

pub mod culprits;
