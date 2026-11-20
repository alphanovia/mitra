# Mitra, the core, started again

A branch that starts from nothing, to converge on the core of Mitra
before `main` is reworked around it. `main` is frozen meanwhile.

**The rule.** One layer at a time, from the inside out. Each layer is
a model file and the smallest code that runs it, and it is read before
the next one starts. Code and tests come over from `main` only when a
layer needs them. `main`'s trust goldens, ladder rungs and breaks are
the yardstick.

## Layer 1: the two sides and the seam

| Model | Code | What it is | Reads |
|---|---|---|---|
| `model/piece.mitra` | `src/piece.rs`, `src/piece/` | something said, what is believed of it, the evidence that moves that belief | nothing |
| `model/value.mitra` | `src/value.rs` | what is computed, its type, the expression that computes it | nothing |
| `model/claim.mitra` | `src/claim.rs` | a piece with a statement; computing the statement makes evidence | both |

The two sides never read each other, and a value never depends on a
piece: `tests/wall.rs` reads the sources and says so.

Not here yet, on purpose: a reader of `.mitra` text, a ledger on disk,
settings, voices read as agent, person or machine, verdict words and
reports, functions and the code they stand for, tests of that code, a
claim over many inputs. The models are read by `main`'s `mitra`, the
code is tied to them by hand.

## To test

```sh
cargo test                # 38 tests
mitra conform             # with main's binary, from this folder
```

- `tests/piece.rs`, `value.rs`, `claim.rs`: each claim of the model as
  one test, tied by name, then the promises on small worlds.
- `tests/goldens.rs`: 20 of `main`'s 22 trust goldens as a table over
  the calculus (case 7 needs settings, case 6 a report).
- `tests/wall.rs`: the wall, and no disk.

`mitra conform` says 33 ties agree and 2 differ. Both differences
look like faults of `main`'s reader of Rust and are left as they are:
it does not see `apart` called from `opinion` through two helpers, and
it compares `Vec<Related>` with the model's `List<Related>` wrongly.

See [docs/review.md](docs/review.md) for what was decided alone and
what is open.
