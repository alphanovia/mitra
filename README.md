# Mitra, the core, started again

A branch that starts from nothing, to converge on the core of Mitra
before `main` is reworked around it. `main` is frozen meanwhile.

**The rule.** One layer at a time, from the inside out. Each layer is
a model file and the smallest code that runs it, and it is read before
the next one starts. Code and tests come over from `main` only when a
layer needs them. `main`'s trust goldens, ladder rungs and breaks are
the yardstick.

## The foundation2 branch

This branch, `foundation2`, is cut from `foundation`. The calculus
underneath is kept; the layer above it is restated. The atoms of a
model are records and functions inside a part, each specified from
opaque to total. A piece is generic in what it is about, and that type
gives its keyword: evidence, tie, require, ensure, claim. An opinion
holds no state inside: what counts for and against a piece, what each
rests on, and the bias; the state is read from them. Three verbs still
cross from a value to a verdict: a claim is **evaluated**, a voice
**takes** what that gave, and the calculus **judges** every piece.

| Model file | Subject | What it is | Code |
|---|---|---|---|
| `model/model.mitra` | `location` | an address in the real system, frozen in `main2` | `src/location.rs` |
| | `piece` | {pre} about {post}, generic in what it is about; require, ensure, tie; the opinion of a piece | |
| | `model` | pieces, what was evaluated and taken, voices, the form of a taking | |
| | `calculus` | how every piece is judged, how sure, what it rests on, the suspects | |
| `model/atom.mitra` | `part` | records and functions in a subject or a boundary, how far each is specified, a measurement | `src/atom.rs` |
| `model/value.mitra` | `value` | a quantity with a type, operations declared on types, expressions of them | `src/value.rs` |
| `model/standard.mitra` | `standard` | the standard types and operations read exactly | `src/standard.rs` |
| `model/bounds.mitra` | `bounds` | the same operations read on the bounds a value lies in | `src/bounds.rs` |
| `model/claim.mitra` | `claims` | the crossing: a claim, evidence of it on a measurement, evaluate and take | `src/claim.rs` |

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
cargo test                # 74 tests
cargo run --example falling
mitra conform             # with main's binary, from this folder
```

- `tests/model.rs`: the piece system, one piece and a few shapes.
- `tests/falling.rs`: the fall, from a value to a verdict, and six
  checks that the logic is sound on it.
- `tests/value.rs`, `standard.rs`, `bounds.rs`: the values, unchanged.
- `tests/wall.rs`: the walls, and no disk.
