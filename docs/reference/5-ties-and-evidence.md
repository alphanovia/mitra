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
is beside the code it models: `check.mitra` next to `check.rs`. Mitra's
own code is modelled that way: each `crates/<crate>/src/<module>.mitra`
holds one module's part, and `crates/mitra-cli/src/main.mitra` is the
whole, reading the others with `use` ([chapter 7](7-contracts-and-use.md#use)).

A tie can also be written in the model, after a name, with `at`. From
[examples/fall/fall.mitra](../../examples/fall/fall.mitra), a tie to something that is
not code, the drop in a lab tower:

```mitra
    fn drop(h: rational) -> v: rational at "lab/tower/stone" where {
```

A tie is itself a statement: "this code is that function". It can be
wrong, and `blame` will say so when it probably is.

## Records tied to structs

A record tied to a Rust struct, or an enum to a Rust enum, is checked
against it each time the code is read. From
[examples/heater/src/lib.rs](../../examples/heater/src/lib.rs), tied to
the part's `record { target: integer, on: bool }`:

```rust
// mitra: implements Heater
pub struct Heater {
    pub target: i64,
    pub on: bool,
}
```

- Fields match by name, in any order; private fields count. A field on
  one side only is R011, unless the record ends with `..`: then the
  struct may hold fields the model does not state.
- Types are checked, never converted: every value of the Rust field
  must be one of the model's type (R012). `i8` to `u128` and `BigInt`
  fit `integer`; `u8` also `byte`; `String` and `&str` fit `[~char]`;
  `Vec<T>` and `[T; N]` fit `[T]`; `Option` and `Result` std's; a
  tuple a tuple; `Box`, `Rc`, `Arc` and `&` what they hold; a struct
  tied to a record, that record.
- Variants of an enum match by name, and what each holds by order and
  type.
- A form the model has none of yet is R013, and the tie is not checked:
  a generic struct, a tuple struct, a variant with named fields, `f32`
  or `f64`, a map or a set, a field named by a keyword of Mitra.

The reader reviews each tie it checks, by the voice
`machine:mitra/reader`: holding when the shapes agree, failing when
they do not, so `blame` sees a wrong tie.

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

Every test counts, wherever it is: a `#[cfg(test)]` module in `src/`,
`tests/`, examples, benches and doc tests, in every crate of a
workspace. A unit test is matched by its crate and module: `opinion`
in `crates/mitra-lang/src/numbers.rs` is the run
`numbers::tests::opinion` of `mitra_lang/src/lib.rs`, not the one of
the same name in another crate. A test line Mitra cannot read, or a
binary whose `test result:` counts more tests than Mitra read, is
listed (R010).

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

### What the environment gave a test

A function that reads the environment is tested with a fake of it. The
fake says what it gave with `given!`, which prints it and gives the
value back. From [examples/heater/src/lib.rs](../../examples/heater/src/lib.rs):

```rust
    #[cfg(test)]
    pub fn temperature() -> i64 {
        mitra::given!(Room::temperature(), NOW.get())
    }
```

The path is the model's function with no body (`environment Room`), as
written; it is never called. The arguments, if any, are what the
environment was called with: `given!(Clock::hour(day), 7)`. It prints
`mitra: given Room::temperature () -> 18`. The givens a test printed
since its last row are the world of its next row, so the test
`ticks_on_what_the_room_reads` is read as
`measure given Room::temperature() -> 18 when Heater::tick(Heater { target: 20, on: false }) then Heater { target: 20, on: true }`,
and explains `correct(Heater::tick, (Heater { target: 20, on: false }) given Room::temperature() -> 18)`.
Like any measurement, the environment's function needs a tie, here on
the real thermometer (`// mitra: implements Room::temperature`). A
given no row of its test follows is R014.

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
- `measure given sun::light() -> 2.0 when stone::heat(5.0) then 10.0 by "voice";`
  is one run in a world: what the environment gave it, then the call,
  then the result. The world is part of the point judged:
  `correct(stone::heat, (5.0) given sun::light() -> 2.0)`. Monday's
  light and Tuesday's are two points, each explained or contradicted on
  its own. An all over what a function takes holds in every world, so
  one world is an instance or a witness of it, never a case of a proof
  by cases. `when f(x) then y` alone is `f (x) -> y`. A given the store
  cannot read (a function with a body, none of the model, given twice,
  or not of its types) is R006.
- `measure when Notes::save("hi") then 2, Disk::write("notes.txt", "hi") by "voice";`
  says what the function gave, then the calls the system made to the
  environment. Each listed call the model's run does not make, `_` any
  argument, counts against `correct(Notes::save, ("hi"))`. The value
  comes first; a call no function with no body of the model takes is
  R006. A measurement that `failed` compares no call.
- `review path holds by "voice";` (or `fails`) is one voice's judgement
  of a claim, rule, require, ensure, or tied item.
- `review path from holds by "voice";` reviews a **link** instead: that
  what an ensure is `from` gives it. An ensure with several lists names
  the one, `from (a::x, b::y)`.
- `… seeing "test:name";` says the voice saw that test pass. The review
  counts only while it passes: when it fails, or does not run, it is
  dropped (R008). Several tests: `seeing "test:a", "test:b"`. A name two
  test files share takes its file: `"test:tests/model.rs::each_error_is_found"`,
  and a unit test its crate's: `"test:mitra-lang/src/lib.rs::numbers::tests::opinion"`.
  A doc test is named as the report names it, with or without its line:
  `"test:src/lib.rs - numbers::of"`.

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
