# 3. Functions and claims

## Opaque, or with a body

A function says what a part does. It takes named inputs and gives a
**named result**:

```mitra
    fn light() -> w: rational;
```

With no body, it is **opaque**: the model does not say how it works,
only what is claimed of it. The world often works this way: how the
sun shines, how an LLM answers.

With a body, Mitra can run it:

```mitra
    fn speed(h: rational) -> v2: rational { 2.0 * 9.81 * h } where {
```

A body is an expression, as in Rust: no statements, no `return`.

## Claims

After the signature or the body, `where { … }` holds what is said of
the function. A **claim** is a named `bool` expression over the inputs
and the result. From [examples/div/model.mitra](../../examples/div/model.mitra):

```mitra
    fn div(a: integer, b: integer) -> q: integer where {
        claim spec: if b == 0 { q == 0 } else { q * b <= a && a < (q + 1) * b };
        claim bounded: b == 0 || q <= a;
```

A claim says what must be true for every input. `div` has no body:
its claims are its spec.

## Examples

`examples { … }` lists rows: the inputs, always in parentheses, then
the result.

```mitra
        examples { (6, 2) -> 3, (7, 2) -> 3, (9, 3) -> 3, (5, 0) -> 0 }
```

One input is still written in parentheses: `(5.0) -> 98.1`.

### Examples in a world

A function that calls the environment gives a result that depends on
what the world gave it: the same input can give two results on two
days. Its rows say the world too, in the form of behaviour-driven
testing (BDD): `given` what the world gave, `when` the inputs, `then`
the result. From
[examples/language/worlds.mitra](../../examples/language/worlds.mitra):

```mitra
        examples {
            given Sun::light() -> 2.0 when (5.0) then 10.0,
            given Sun::light() -> 3.0 when (5.0) then 15.0
        }
```

`(6, 2) -> 3` is short for `when (6, 2) then 3`. After `given` comes a
function with no body (by its path, or its name when only one function
has it), what it was called with, and what it gave. A call is given at
most once a row, for each list of arguments (M022).

`_` is any argument, as in a Rust pattern. A row with written arguments
wins where it fits, whatever the order:

```mitra
        examples {
            given Sun::hour(_) -> 6 when (1, 3) then 0,
            given Sun::hour(_) -> 6, Sun::hour(2) -> 7 when (1, 2) then 1
        }
```

A row of the whole model, end to end, names the function `when` calls:

```mitra
examples {
    given Sun::light() -> 2.0 when Stone::heat(5.0) then 10.0
}
```

It is the same row as `(Stone::heat::h = 5.0) -> Stone::heat::e = 10.0`
in the world given; `given` may stand in front of that form too. A
`given` that is not what a function with no body takes and gives is an
error (M022), a `when` naming no function of the model too (M021), and a
`given` no run reads is a finding (A009).

## What `mitra check` does with them

`check` compares everything the model says twice:

| It compares | With | Finding when they disagree |
|---|---|---|
| a body | each claim | A001 |
| a body | each example | A002 |
| an example | each claim | A003 |
| the claims | each other: can any result keep them all? | A004 |

A comparison **holds** when Mitra shows it for every input: by bounds,
or by trying every point when there are few enough (64 by default,
`--points N`). Otherwise it is **unknown** (A007): Mitra tried, found
nothing wrong, and could not cover every input. That is not an error.
It says where evidence is needed.

From [examples/language/parts.mitra](../../examples/language/parts.mitra),
a claim over every integer target:

```text
finding[A007]: claim `keeps_target`: unknown, not decided by bounds over all of what it reads, nor at 26 points
 --> Heater::decide
```

## Ranges

An integer input or result can be given a range: `0..=9` is an
`integer` from 0 to 9, checked. From
[examples/language/values.mitra](../../examples/language/values.mitra):

```mitra
    fn digit(n: 0..=49) -> d: 0..=9 { n % 10 } where {
        claim at_most: d <= n;
        examples { (42) -> 2 }
    }
```

With 50 inputs, `check` tries them all, and the claim holds. A result
outside its range is a finding (A001 "its range"); an example outside it
is M026.
