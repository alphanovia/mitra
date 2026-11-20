# Review notes of the `core` branch

Increments and decisions go on from `main`'s
`docs/review/2026-11-18.md` (increment 90, decision 556).

## Increment 91: layer 1, the two sides and the seam (2026-11-20)

**Asked.** "how about you start again from scratch on a new branch ?"
"when the branch has converged on a good code, we can rework main
around it." An orphan branch: no history and no file of `main`
(only `LICENSE` and `.gitignore` copied). One crate, `mitra-core`, no
dependency, 362 lines of code for 191 lines of model.

**Decisions taken alone.**

557. An orphan branch, not a branch off `main`: the tree starts empty
     and nothing is inherited by accident. The merge is to be decided.
558. One crate with three modules, not three crates. The wall is a
     test (`tests/wall.rs`). Crates when a layer needs them.
559. `Evidence` has a fourth field, `piece`: its own name. You agreed
     to `{ about, value, by }`; evidence is a piece, so it needs a name
     to be reviewed by, as `Claim` has one. `as_piece` is gone.
560. `Known` carries the base rate of each piece (`base`). Who sets it
     is an outer layer: settings by kind, or the certainty its author
     stated. A piece with none has 0.
561. `apart` knows two levels only, same voice or not. Reading a voice
     as agent, model, vendor is a later layer, and so is the
     interpolation between the two ends, which `fuse` already takes.
562. A new voice is as independent as it is from the nearest voice
     already heard on that piece.
563. A piece is believed on its own evidence or on what it rests on,
     whichever is more (not fused). What it stands on is a set of
     pieces, so one reached by two paths counts once.
564. A piece that leads back to itself (evidence about its own
     evidence, resting on itself) is taken with no evidence there.
565. `conflicts` is a function of its own and changes no opinion: the
     pieces of an `Exclude` that are each believed, and the evidence
     for and against one piece, least believed first.
566. An operator is strict: `&&` computes both sides. `eval` does not
     check: an expression that does not check can still have a value.
567. The evidence of `observe` is named after the claim, the inputs
     and the voice, so the same observation twice is one piece.
568. The verdict word (`holds`, `fails`, `unverified`) is not in the
     core. `tests/goldens.rs` writes it in six lines to compare with
     `main`.

**Still open, from the proposal.** A claim over many inputs (proposed:
a claim at one input is a piece, the whole rests on its points); how
far `machine:mitra` is believed (the seam test sets it to 1 by hand);
where a test of the code enters; the names of the variants; `Piece` as
a level apart; independence as a number; a review by the test's own
author.

**What to review.** The three model files, `ensure` lines first. Then
`src/piece/known.rs` (121 lines): `direct`, `support`, `together`.
Then `tests/goldens.rs` against the table of the calculus note.

**How to test.** `cargo test`. With `main`'s binary, `mitra conform`
in this folder.
