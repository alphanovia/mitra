# Mitra, the core, started again

A branch that starts from nothing, to converge on the core of Mitra
before `main` is reworked around it. `main` is frozen meanwhile.

**The rule.** One layer at a time, from the inside out. Each layer is
a model file and the smallest code that runs it, and it is read before
the next one starts. Code and tests come over from `main` only when a
layer needs them. `main`'s trust goldens, ladder rungs and breaks are
the yardstick.

## The foundation branch

This branch, `foundation`, restates the core around one shape. A piece
is a proposed property of the system: given the pieces before it, its
subject gives the pieces after it. An opinion is read five ways:
evaluated, taken, judged, confidence, bias. Three verbs cross from a
value to a verdict: a claim is **evaluated**, a voice **takes** what
that gave, and the calculus **judges** every piece.

| Model file | Subject | What it is | Code |
|---|---|---|---|
| `model/model.mitra` | `place` | a location in the system | `src/model.rs` |
| | `piece` | {pre} subject {post}, and the opinion of it | |
| | `model` | pieces, what is given, what was evaluated and taken, voices | |
| | `calculus` | how every piece is judged, how sure, and the suspects | |
| `model/value.mitra` | `value` | a quantity with a type, operations declared on types, expressions of them | `src/value.rs` |
| `model/standard.mitra` | `standard` | the standard types and operations read exactly | `src/standard.rs` |
| `model/bounds.mitra` | `bounds` | the same operations read on the bounds a value lies in | `src/bounds.rs` |
| `model/claim.mitra` | `claims` | the crossing: evaluate a claim, take what it gave | `src/claim.rs` |

The foundation test model is the speed of a fall: `tests/falling.rs`,
printed by `cargo run --example falling` and kept in `docs/falling.md`.

`model.mitra` and `value.mitra` read nothing and never read each
other; `standard.mitra` reads `value.mitra`, `bounds.mitra` reads both; only the seam reads both
sides.
`tests/wall.rs` reads the sources and says so.

Not here yet, on purpose: a reader of `.mitra` text, a ledger on disk,
settings, voices read as agent, person or machine, verdict words and
reports, a build that judges every piece at once, a reader of the
code and of its test reports.
The models are read by `main`'s `mitra`, the code is tied to them by
hand.

## The ledger, when it comes

Decided, not built: `core` reads no ledger yet. One ledger file per
model file, beside it: `model.ledger.mitra` for `model.mitra`,
`value.ledger.mitra` for `value.mitra`, and so on. A piece of evidence
goes in the ledger of the file that holds the piece it is about. The
build groups them into one ledger, and the commands that write
evidence (`add`, `ingest`, `tie`) write to the right file.

Until then `model/evidence.mitra` is the single ledger that `main`'s
`mitra conform` writes when it is run here.

## To test

```sh
cargo test                # 63 tests
cargo run --example falling
mitra conform             # with main's binary, from this folder
```

- `tests/model.rs`: the piece system, one piece and a few shapes.
- `tests/falling.rs`: the fall, from a value to a verdict.
- `tests/value.rs`, `standard.rs`, `bounds.rs`: the values, unchanged.
- `tests/wall.rs`: the walls, and no disk.
