# Keywords: expressions

Part of the [language reference](language.md); index in
[keywords.md](keywords.md). Every block fenced `mitra` is a whole model,
`evidence` is evidence for it, `text` is what `mitra eval` prints for
the two. A test runs them all.

Most examples are point claims on helpers: computed, they hold alone.

## Operators

`+ - * /` on numbers (integer division on integers), `== != < <= > >=`,
`&& || !`, and `a <=> b`: both true or both false. A result outside
64-bit integers, or a division by zero, is an error, never a wrap.

```mitra
claim ops() { 7 / 2 == 3 && 1 + 2 * 3 == 7 && !(1 > 2) && ((1 < 2) <=> (2 < 3)) }
```
```text
ops: holds

```

## if, else

`if c { a } else { b }`. Both branches are needed. It is a value.

```mitra
fn sign(x: Int) -> {-1, 0, 1} { if x > 0 { 1 } else { if x < 0 { -1 } else { 0 } } }
claim signs() { sign(5) == 1 && sign(-3) == -1 && sign(0) == 0 }
```
```text
signs: holds

```

## let

`let x = e; rest`: a name for a value in what follows.

```mitra
fn mean(a: Int, b: Int) -> Int { let sum = a + b; sum / 2 }
claim means() { let m = mean(2, 6); m == 4 }
```
```text
means: holds

```

## match

`match e { A => …, B(x) => …, _ => … }`: by variant of an
[enum](kw-records.md#enum), with a name for each field. `_` is
every other variant.

```mitra
enum Token { Num(n: Int), Plus, Minus }
fn weight(t: Token) -> Int { match t { Num(n) => n, _ => 0 } }
claim weights() { weight(Num(4)) == 4 && weight(Plus) == 0 }
```
```text
weights: holds

```

## is

`x is Variant`: true when `x` is that variant.

```mitra
import std::rust::{Result};
subject fn parse(s: {"1", "x"}) -> Result<Int, String>;
claim rejects_letters() { parse("x") is Err }
```
```evidence
evidence t { for parse given s = "x" gives Err("not a number") }
```
```text
parse::correct: unverified
  no evidence on  `parse("1")`, `parse("x")`
rejects_letters: holds
```

## in

`x in xs`: `x` is an element of a list or a set, or a key of a map. It
also names each element in `all`, `any`, `filter`, `map`, `fold`,
`rate`, and each value of an input in evidence (`given a in 0..=3`).

```mitra
claim members() { 2 in [1, 2] && !(4 in {1, 2}) && "door" in {"door": 1} }
```
```text
members: holds

```

## Lists, sets, maps

`[1, 2]`, `{1, 2}`, `{"k": 1}`, the empty map `{:}`. `xs[i]` from 0,
`m[k]`, `len(xs)`, `keys(m)`. On sets: `|` union, `&` both, `\` minus.
Tuples `(a, b)` are read `t.0`, `t.1`.

```mitra
claim collections() {
    [5, 6][1] == 6 && len({1, 1, 2}) == 2 && {"k": 7}["k"] == 7
        && keys({"a": 1, "b": 2}) == {"a", "b"} && ({1, 2} | {3}) \ {1} == {2, 3}
        && (4, 5).0 == 4 && len({:}) == 0
}
```
```text
collections: holds

```

## all, any, filter, map, fold

Over each element of a list or a set. `all` and `any` give a Bool,
`filter` and `map` a list. `fold(x in xs, let acc = start; step)`:
`step` is the next value of `acc`. `{ x in xs | cond }` is `filter`
that keeps a set a set.

```mitra
fn total(xs: List<Int>) -> Int { fold(x in xs, let sum = 0; sum + x) }
claim over_lists() {
    all(x in [1, 2], x > 0) && any(x in [1, 2], x == 2)
        && filter(x in [1, 2, 3], x != 2) == [1, 3] && map(x in [1, 2], x * 10) == [10, 20]
        && total([1, 2, 3]) == 6 && { x in {1, 2, 3} | x > 1 } == {2, 3}
}
```
```text
over_lists: holds

```

## rate

`rate(x in S, cond)`: the share of the values of `S` where `cond` is
true, from 0 to 1. Computed from the outputs observed when they cover
`S`; else measured, with `gives rate = …`.

```mitra
subject fn up(day: 1..=4) -> Bool;
claim mostly_up() { rate(d in 1..=4, up(d)) >= 0.75 }
```
```evidence
evidence m { for mostly_up gives rate = 0.8 source "monitor:week_12" }
```
```text
up::correct: unverified
  no evidence on  day in 1..=4
mostly_up: holds
```

## precision, recall, false_positive_rate

Measures of a function that answers true or false, from its observed
outputs (`gives`) and from what was judged right (`holds` / `fails`).

```mitra
subject fn is_spam(mail: 1..=4) -> Bool;
claim precise() { precision(is_spam) >= 0.5 }
claim finds_most() { recall(is_spam) >= 0.9 }
claim few_alarms() { false_positive_rate(is_spam) <= 0.5 }
```
```evidence
evidence a1 { for is_spam given mail = 1 gives true }
evidence a2 { for is_spam given mail = 2 gives true }
evidence b3 { for is_spam given mail = 3 gives false }
evidence b4 { for is_spam given mail = 4 gives false }
evidence c { for is_spam given mail = 1 holds }
evidence d { for is_spam given mail = 2 fails }
evidence e { for is_spam given mail in 3..=4 holds }
```
```text
is_spam::correct: fails
  fails on        `is_spam(2)`
  passes on       `is_spam(1)`, `is_spam(3)`, `is_spam(4)`
precise: holds
finds_most: holds
few_alarms: holds
```

## Inline functions

`|a, b| a - b`: a function written where it is used, as the class of
`uniform` or as a parameter of a claim. Elsewhere it exists only
applied: `(|a, b| a - b)(3, 1)`. It is not a value.

```mitra
claim applied() { (|a, b| a - b)(3, 1) == 2 }
```
```text
applied: holds

```

## true, false

The two values of `Bool` (`bool` is the same type).
