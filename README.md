# Mitra, main2

`main2` holds only what the user has frozen: meaning, purpose and
code. Nothing lands here unless the user says to freeze it. The
language has not converged, so no model file is frozen yet.

**The rule.** Every experiment branch contains `main2`:
`git merge-base --is-ancestor main2 <experiment>` holds. An experiment
that a freeze kills is retired and a new one is started from `main2`.
The old `main` stays frozen as it is.

## Frozen

Each item is in [docs/design/frozen.md](docs/design/frozen.md).

| Item | Purpose | Code |
|---|---|---|
| Location | the tether between the model and the real system | `src/location.rs` |

Test: `cargo test`.
