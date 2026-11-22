# Evidence on core2

`src/evidence.rs` holds what was measured and reviewed of the real
system, one set at one time, and checks it against a model. The reader
(`src/reader.rs`, `docs/reader.md`) builds it from Rust tests, a
`cargo test` report and a store written by hand. Staleness and the
management of a ledger over time are not in this round.

## Data

```rust
struct Measurement { inputs: Vec<(Path, Value)>, outputs: Vec<(Path, Value)>, seen: bool, count: u64, by: String }
struct Review { prop: Path, holds: bool, by: String }
struct Ledger { measurements: Vec<Measurement>, reviews: Vec<Review> }
```

- **A measurement is always through a tie.** It is of the real thing:
  each function it names is tied, or its part is. A row with no tie is an
  example (the model's spec), and the model run on itself is simulation.
- **Inputs and outputs are named apart**, `calc::div::a`, so a unit test
  (one function) and an end-to-end test (inputs of one part, outputs of
  another) have one shape, the shape of an example of the model.
- **The test states its outputs** (your option (a) for tonight): `seen`
  says whether the system gave them. A failed test says only that it gave
  something else.
- **A review has no count**: independent voices are what weigh. One voice
  reviews one proposition once.
- **A tie is a proposition**, reviewed by its item's path: a review of
  `calc::div` is a review of each of its ties.
- **A voice** is `human:name`, `agent:…`, `machine:tool`, or `test:name`
  for a run of a test (the reader's).

## Checks

`check(model, ledger)` gives every error, each at `measurements::i` or
`reviews::i`.

| Code | The check |
|---|---|
| V001 | Each input is what a function takes, or what one with no body gave; each output what a function gives; each once on its side. |
| V002 | Each function a measurement names is tied, or its part is. |
| V003 | Each value is of the type of what it names. |
| V004 | A measurement counts one run at least. |
| V005 | A review names a claim, rule, require or ensure, or a tied item. |
| V006 | A measurement has an output, and every argument of a function whose output it measures (or whose value it gives, with no body) is given. |
| V007 | Every measurement and review has a voice, and one voice reviews one proposition once. |
