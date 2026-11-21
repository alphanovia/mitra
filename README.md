# Mitra, the core, started again

A branch that starts from nothing, to converge on the core of Mitra
before `main` is reworked around it. `main` is frozen meanwhile.

**The rule.** One layer at a time, from the inside out. Each layer is
a model file and the smallest code that runs it, and it is read before
the next one starts. Code and tests come over from `main` only when a
layer needs them. `main`'s trust goldens, ladder rungs and breaks are
the yardstick.

## Layer 1: the model, the values and the seam

| Model file | Subject | What it is | Code |
|---|---|---|---|
| `model/model.mitra` | `place` | a location in the system | `src/model/place.rs` |
| | `piece` | something said, right or wrong, and the opinion of it | `src/model/piece.rs` |
| | `evidence` | a piece that says an opinion about another piece | `src/model/evidence.rs` |
| | `statement` | an interpretation of the system; its content is not read | `src/model/statement.rs` |
| | `tie` | a believed link between a statement and a place | `src/model/tie.rs` |
| | `model` | all the pieces put together | `src/model.rs` |
| `model/value.mitra` | `value` | a quantity with a type, operations declared on types, expressions of them; no type, no operation and no kind of quantity built in | `src/value.rs` |
| `model/standard.mitra` | `standard` | the standard types and operations read exactly (a truth, whole numbers, a real, records) and the table that applies them | `src/standard.rs` |
| `model/bounds.mitra` | `bounds` | the same operations read on the bounds a value lies in, with a flag for what may have no value | `src/bounds.rs` |
| `model/function.mitra` | `functions` | a function: an operation the code implements, its tie and its statements; helpers; a test as a measurement | `src/function.rs` |
| `model/verdict.mitra` | `verdict` | the opinion of a piece: a state found by logic from the evidence taken as true, how sure it is, and what to give up on a conflict | `src/verdict.rs` |
| `model/claim.mitra` | `claims` | the seam: a statement whose content is an expression; computing it at an input makes evidence about that instance | `src/claim.rs` |

`model.mitra` and `value.mitra` read nothing and never read each
other; `standard.mitra` reads `value.mitra`, `bounds.mitra` reads both; only the seam reads both
sides. Inside the model a place and a piece
read nothing, and each kind of piece reads only what it requires.
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
cargo test                # 164 tests
mitra conform             # with main's binary, from this folder
```

- `tests/model.rs`, `value.rs`, `standard.rs`, `claim.rs`: each claim of the models as
  one test, tied by name, then the promises on small worlds.
- `tests/goldens.rs`: 20 of `main`'s 22 trust goldens as a table over
  the calculus (case 7 needs settings, case 6 a report).
- `tests/structure.rs`: how the shape of the graph changes what is read.
- `tests/wall.rs`: the walls, and no disk.

`mitra conform` says 122 ties agree and 19 differ, with one warning.
The five look like faults of `main`'s reader of Rust and are left as
they are: `Vec<Related>` and `Statement<Claim>` compared wrongly with
the model's (three), a `uses` asked on `moved` whose body already
calls `piece::projected`, and `all` not seen under `opinion`. The
warning is true: `tie` requires `statement` and names a statement only
by its piece.

See [docs/review.md](docs/review.md) for what was decided alone and
what is open.
