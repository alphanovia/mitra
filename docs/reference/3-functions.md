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

A body is a block, as in Rust: `let`s, then the value it gives, with no
`return` ([chapter 4](4-values.md#blocks-let-and-if)).

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
error (M022), and a `when` naming no function of the model too (M021).

### An example is one run

An example is one run of the model, in the world it states: everything
it states is used, and on a function with a body it holds or fails,
never unknown. It fails when:

- the body gives another value (A002);
- the run calls a function with no body that has no value there: not
  given in the example, and none of that function's own examples is at
  those arguments (A014). A function with a body never needs a `given`;
- the claims of such a function keep the run from giving the value the
  example says (A017);
- a `given` the run never calls (A015);
- the run reaches the step or depth budget (A016), or an `until` comes
  back to its values (A013).

```text
finding[A014]: example 0: fails: no value for `Disk::big(3)`: the run calls it, and nothing gives it there; give it in the example (`given Disk::big(3) -> …`), or give `Disk::big` a body
finding[A015]: example 0: `Disk::small(3)` is given but never called: the run calls `Disk::big(3)`
```

An input known only in part, a record with `..`, is not one run: such an
example is read as a claim, and may be unknown (A007). An example of a
function with no body is not run by `check`: a test that measures the
code at its input decides it.

### What a run does to the world

A body calls the environment for what the call does in a block, `e;
r`. `then` lists such calls after the value, or alone. From
[examples/language/calls.mitra](../../examples/language/calls.mitra):

```mitra
        examples {
            when ("hi") then 2, Disk::write("notes.txt", "hi"),
            when ("hello") then Disk::write(_, "hello"),
            ("") -> 0
        }
```

Each listed call is to a function with no body (M022 otherwise), and the
run makes it at least once, with those arguments, `_` any: one it does
not make is a finding (A011), which lists the calls it made. Calls not
listed are not compared. A call is told from a value by its name,
lowercase as a function's, where a variant's is `CamelCase`. A row of
the whole model lists them too:

```mitra
examples {
    when Notes::save("ok") then 2, Disk::write("notes.txt", _)
}
```

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
`--points N`). Otherwise a claim is **unknown** (A007): Mitra tried, found
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
