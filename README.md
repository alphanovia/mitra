# Mitra, main2

`main2` holds only what the user has frozen. Nothing lands here unless
the user says to freeze it.

**The rule.** Every experiment branch contains `main2`:
`git merge-base --is-ancestor main2 <experiment>` holds. An experiment
that a freeze kills is retired and a new one is started from `main2`.
The old `main` stays frozen as it is.

## Frozen

| Item | Meaning | Model | Code |
|---|---|---|---|
| `location` | An address of something in the real system, the tether between the model and the system: a file, then what is in it. It is not right or wrong. `within` is a path prefix. | `model/location.mitra` | `src/location.rs` |

Test: `cargo test`.
