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
| `model/value.mitra` | `value` | what is computed, its type, its expression | `src/value.rs` |
| `model/claim.mitra` | `claims` | the seam: a statement whose content is an expression; computing it makes evidence | `src/claim.rs` |

`model.mitra` and `value.mitra` read nothing and never read each
other; only the seam reads both. Inside the model a place and a piece
read nothing, and each kind of piece reads only what it requires.
`tests/wall.rs` reads the sources and says so.

Not here yet, on purpose: a reader of `.mitra` text, a ledger on disk,
settings, voices read as agent, person or machine, verdict words and
reports, a build that judges every piece at once, functions and the
code they stand for, tests of that code, a statement over many inputs.
The models are read by `main`'s `mitra`, the code is tied to them by
hand.

## To test

```sh
cargo test                # 74 tests
mitra conform             # with main's binary, from this folder
```

- `tests/model.rs`, `value.rs`, `claim.rs`: each claim of the models as
  one test, tied by name, then the promises on small worlds.
- `tests/goldens.rs`: 20 of `main`'s 22 trust goldens as a table over
  the calculus (case 7 needs settings, case 6 a report).
- `tests/wall.rs`: the walls, and no disk.

`mitra conform` says 43 ties agree and 5 differ, with one warning.
The five look like faults of `main`'s reader of Rust and are left as
they are: `Vec<Related>` and `Statement<Formal>` compared wrongly with
the model's (three), a `uses` asked on `moved` whose body already
calls `piece::projected`, and `all` not seen under `opinion`. The
warning is true: `tie` requires `statement` and names a statement only
by its piece.

See [docs/review.md](docs/review.md) for what was decided alone and
what is open.
