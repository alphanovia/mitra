# The reader on core2

`crates/mitra-rust/src/reader.rs` reads what comes from outside the model, and gives the
ledger: what was measured and what was reviewed. Nothing given is
dropped quietly: what it cannot use is a finding of suit `R` (flagged),
written as rustc writes, `finding[R004]: … --> file:line`.

## Ties, in Rust code

As on main: a comment above a `fn`, `struct` or `enum` (attributes and
doc comments may come between).

```rust
// mitra: implements calc::div
pub fn div(a: u32, b: u32) -> u32 { if b == 0 { 0 } else { a / b } }
```

The tie's location is the file, then the `mod` and `impl` around the
item, then its name: `src/lib.rs/div`, `src/lib.rs/Calc/div`, as the
model writes `at "src/lib.rs/div"` (flagged). The item must be a part,
record or function of the model (R002). The reader counts braces line
by line to know what is around an item: a brace in a string counts too.

## Tests, in Rust code (flagged)

```rust
// mitra: measures calc::div (6, 2) -> 3
#[test]
fn div_by_two() { assert_eq!(div(6, 2), 3); }
```

A row as the model's examples write it. One comment per row; a test may
have several. The function is named whole, or by its name when only one
function has it. Main's `// mitra: evidence for …` is not read (R001).

## `cargo test`

`cargo test > report.txt 2>&1`, then `read_cargo_test(report)` reads
each `test name ... ok | FAILED | ignored` line. A measuring test that
passed saw its row (`seen`); one that failed saw something else at its
inputs (not `seen`); one ignored or absent measured nothing (R005). A
test that ran and that no comment names is listed (R004). Each is a
`Measurement`, by `test:<name>` (flagged), count 1, always through a tie
(R007): a tie in the model or in the code.

## The store, written by hand (flagged)

```text
// evidence.mitra
measure calc::div (6, 2) -> 3 by "human:ana";
measure div (9, 3) -> 3 count 4 by "human:ana";
measure calc::div (7, 0) -> 0 failed by "agent:anthropic/opus-5.5/s-1";
review calc::div::spec holds by "human:ana";
review calc::inputs fails by "human:bo";
```

`measure` is a measurement: a row, how many times it was seen
(`count`, 1 when not said), `failed` when the function gave something
other than the row's result. `review` says that a claim, rule, require,
ensure or tied item holds or fails, as one voice. A voice is
`human:…`, `agent:vendor/model/session` or `machine:…`, as on main.
`print_store` writes a ledger back in this form.

## For the CLI

`reader::ledger(model, files, report, store)` reads it all: each Rust
file's path and text, the `cargo test` report, the store's path and
text. It gives every tie (the model's and the code's), the `Ledger`,
and the findings. `Measurement`, `Review` and `Ledger` are the shapes
the evidence layer fixes; they move to `crates/mitra-lang/src/evidence.rs` when it lands.
