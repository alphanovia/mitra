# 5. Ties and evidence

A model only becomes trustworthy when it is tied to the real system and
the system is observed.

## Ties

A **tie** says that a piece of the system is a piece of the model. In
Rust, it is a comment above the item. From
[examples/div/src/lib.rs](../../examples/div/src/lib.rs):

```rust
// mitra: implements calc::div
pub fn div(a: u32, b: u32) -> u32 {
```

The tie's place is the file, then the `mod` and `impl` around the item,
then its name: `src/lib.rs/div`.

A tie can also be written in the model, after a name, with `at`. From
[examples/fall/fall.mitra](../../examples/fall/fall.mitra), a tie to something that is
not code, the drop in a lab tower:

```mitra
    fn drop(h: rational) -> v: rational at "lab/tower/stone" where {
```

A tie is itself a statement: "this code is that function". It can be
wrong, and `blame` will say so when it probably is.

## Tests that measure

A test says what it measures with a comment, a row as the model's
examples write it:

```rust
// mitra: measures calc::div (6, 2) -> 3
#[test]
fn div_by_two() {
    assert_eq!(div(6, 2), 3);
}
```

Mitra reads which tests passed from what `cargo test` printed:

```text
cargo test > report.txt 2>&1
mitra explain model.mitra --code src --report report.txt
```

A test that passed saw its row. One that failed saw something else at
those inputs. A test that ran and measures nothing Mitra knows is
listed (R004), never dropped silently.

A measurement is always through a tie: a test of a function with no tie
is reported (R007), since it would say nothing of the real system.

## The store: evidence by hand

Not all evidence comes from tests. A store is a file of measurements
and reviews, from [examples/div/evidence.mitra](../../examples/div/evidence.mitra):

```text
measure calc::div (7, 2) -> 4 by "human:ana";
review calc::div::spec holds by "human:bo";
```

- `measure f (inputs) -> result by "voice";` is one run seen by hand.
  `count 4` says it was seen four times; `failed` that the function gave
  something else.
- `review path holds by "voice";` (or `fails`) is one voice's judgement
  of a claim, rule, require, ensure, or tied item.

A **voice** says who: `human:name`, `agent:vendor/model/session`,
`machine:tool`. Tests are `test:name`. One voice reviews one statement
once.

```text
mitra explain model.mitra --code src --report report.txt --store evidence.mitra
```
