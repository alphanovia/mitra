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
then its name: `src/lib.rs/div`. The file is read from the project's
root (the folder of `mitra.toml`), wherever the model sits.

## Where the model lives

A model file can sit anywhere in the project. The habit Mitra suggests
is beside the code it models: `src/check.mitra` next to `src/check.rs`.
Mitra's own code is modelled that way: each `src/<module>.mitra` holds
one module's part, and `src/bin/mitra.mitra` is the whole, reading the
others with `use` ([chapter 7](7-contracts-and-use.md#use)).

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

`mitra explain` runs the tests itself, at the project's root when it
has a `Cargo.toml`, and reads which passed from what they printed:

```text
mitra explain model.mitra --code src
ran `cargo test --no-fail-fast -- --show-output` in …: 3 tests, 3 passed, 0 failed, 0 ignored
```

The command is `[test] command` in `mitra.toml`. In CI, run the tests
yourself and give what they printed, `--report report.txt`; `--no-test`
runs none. A test binary that crashed before its result is listed
(R009): its tests after the crash measured nothing.

A test that passed saw its row. One that failed saw something else at
those inputs. A test that ran and measures nothing Mitra knows is
listed (R004), never dropped silently.

A measurement is always through a tie: a test of a function with no tie
is reported (R007), since it would say nothing of the real system.

## Tests that write what they saw

The `mitra` test crate (`crates/mitra`, a dev-dependency) writes the
row for you. `check_eq!` calls the function, prints the row it saw,
then asserts. From [examples/heater/src/lib.rs](../../examples/heater/src/lib.rs):

```rust
#[test]
fn heats_below_the_target() {
    let cold = Heater { target: 20, on: false };
    mitra::check_eq!(Heater::decide(cold, 18), Heater { target: 20, on: true });
}
```

It prints `mitra: row Heater::decide (Heater { target: 20, on: false }, 18) -> Heater { target: 20, on: true }`,
and `mitra explain` reads it as a measurement by
`test:tests::heats_below_the_target`, of the model's function tied to
`Heater::decide` (`// mitra: implements Heater::decide`). The row is
what the system gave, whether the test passed or not. `check!(f(a))`
does the same for a function that gives `bool`, and asserts `true`.

The call is by path, a method with its receiver first. Values are
written as Mitra writes them by the `Observe` trait: derive it,
`#[cfg_attr(test, derive(mitra::Observe))]`, for a struct with named
fields or an enum. A row the reader cannot place is R001; through no
tie, R007.

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
- `review path from holds by "voice";` reviews a **link** instead: that
  what an ensure is `from` gives it. An ensure with several lists names
  the one, `from (a::x, b::y)`.
- `… seeing "test:name";` says the voice saw that test pass. The review
  counts only while it passes: when it fails, or does not run, it is
  dropped (R008). Several tests: `seeing "test:a", "test:b"`. A name two
  test files share takes its file: `"test:tests/model.rs::each_error_is_found"`.

A **voice** says who: `human:name`, `agent:vendor/model/session`,
`machine:tool`. Tests are `test:name`, and only measure: a review by a
test is refused (V007). One voice reviews one statement, or one link,
once.

## What explain asks you to review

A link, or an ensure said only in words, is only the model's word
until someone reviews it. `mitra explain` ends with each one no review
answers (X012), and the line to write once you have:

```text
ask[X012]: only words say `Heater::steady`, and nothing below gives it: review it, seeing the test that shows it if one does
 --> Heater::steady
  = help: once reviewed, write in the store `review Heater::steady holds by "human:<name>" seeing "test:<name>";`
```

```text
mitra explain model.mitra --code src --store evidence.mitra
```
