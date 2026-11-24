# The command line on core2

One command per question (your choice, 21:47), as rustc writes its
reports. `cargo build --release`, then `target/release/mitra`.

```text
mitra check MODEL [--points N]
mitra explain MODEL [--code DIR]... [--report FILE] [--store FILE]
mitra blame MODEL [--code DIR]... [--report FILE] [--store FILE]
```

- `MODEL`: the `.mitra` file at the root of the project. Paths of Rust
  files are read from its folder, as ties write them: `src/lib.rs/div`,
  however the folders are spelled (`./`, absolute).
- `--code`: a folder (or a file) of Rust code: its `// mitra:
  implements` ties and `// mitra: measures` tests (`docs/design/reader.md`).
- `--report`: what `cargo test` printed (`cargo test > report.txt 2>&1`).
  With `--code` and no report, each measuring test is R005 ("not in the
  report"), after a line saying so.
- `--store`: measurements and reviews written by hand.

- `--points`: for `check` only, the most points agree tries for one
  comparison (its setting `points`, default 64). `main`'s div takes
  `a: 0..=9, b: 0..=9`, 100 pairs: with `--points 100` every pair is
  tried and it holds; at 64 it is unknown (A007).

`--report`, `--store` and `--points` are given once each.

A part the model uses and does not declare is read from `<part>.mitra`
beside the model file, and `use std::core::…` from Mitra's own standard
model, in the binary (P010 when neither has it).

| Command | Question | Says |
|---|---|---|
| `check` | 1: is the model sound on itself? | parse and `check` errors; then agree's findings and a count of comparisons |
| `explain` | 2: does it explain the system, assuming the evidence? | reader findings, evidence errors, a verdict on each proposition with its numbers, explain's findings and hints |
| `blame` | 3: which evidence is wrong, given it explains the system? | the diagnoses, most doubtful first, and blame's findings |

The exit code is 1 when something is not well formed (a parse, model
or evidence error), 2 for a wrong command line or a file that cannot be
read, else 0: findings are verdicts, not faults. When what reads the
output stops (`| head`), `mitra` ends quietly.

A verdict line: `explained correct(calc::div, (6, 2)) 0.67 (b 0.33, d
0.00, u 0.67: 1 for, 0 against, test:tests::div_by_two)`: the verdict,
the proposition, its expected value, belief, disbelief and uncertainty,
then its strongest rest: counts for and against, how many takings only
the model says, and the voices.

## Try it

```text
cargo build --release
M=target/release/mitra
E="--code examples/div/src --report examples/div/report.txt --store examples/div/evidence.mitra"
$M check examples/div/model.mitra
$M explain examples/div/model.mitra $E
$M blame examples/div/model.mitra $E
```

`examples/div`: main's div, tied in its code, three tests that pass, a
store with a measurement taken wrong (`(7, 2) -> 4`) and a review.
`blame` doubts that measurement first. A diagnosis names measurements
explain read as one together: `measurements::0 and 4`.

## Not in this round

- No `--json`; no colours; no other settings on the command line
  (each layer's defaults).
- `check` runs agree; `explain` and `blame` do not run agree again.
