# Keywords: types

Part of the [language reference](language.md); index in
[keywords.md](keywords.md). Every block fenced `mitra` is a whole model,
`evidence` is evidence for it, `text` is what `mitra eval` prints for
the two. A test runs them all.

A type is a set of values. Anywhere a type is written, a narrower set
may be: the narrower it is, the less evidence a claim owes.

## Bool, Int, Float64, String, Value

`Bool` (`bool`), `Int` (every integer), `Float64` (`f64`), `String`,
`Value` (everything). Rust's integer types are ranges: `i8`…`i64`,
`u8`…`u32`, `u64` and `usize` (`0..`). `()` is the type with one value.

```mitra
fn describe(n: u8, ratio: f64, loud: bool) -> String;
claim described(n: {0, 255}) { describe(n, 0.5, true) != "" }
```
```evidence
evidence a { for describe given n = 0, ratio = 0.5, loud = true gives "0" }
```
```text
described: unverified
  passes on       `n = 0`
  no evidence on  `n = 255`
```

## Ranges

`0..=9` from 0 to 9, `1..` from 1 up, `..=5` up to 5.

```mitra
fn pick(n: 0..=2) -> 1..;
claim picks(n) { pick(n) >= 1 }
```
```evidence
evidence a { for pick given n = 0 gives 1 }
evidence b { for pick given n = 1 gives 7 }
```
```text
picks: unverified
  passes on       `n = 0`, `n = 1`
  no evidence on  `n = 2`
```

## Sets of values

`{"a", "b"}`, `{1, 2, 3}`: exactly these values.

```mitra
fn light(state: {"red", "green"}) -> {"stop", "go"};
claim red_stops() { light("red") == "stop" }
```
```evidence
evidence a { for light given state = "red" gives "stop" }
```
```text
red_stops: holds

```

## Union, both, minus

`A | B`, `A & B`, `A \ B`.

```mitra
fn safe_div(a: 0..=4, b: 0..=4 \ {0}) -> 0..=4 | {-1};
claim bounded(a, b) { safe_div(a, b) <= a }
```
```evidence
evidence a { for safe_div given a = 4, b = 2 gives 2 }
```
```text
bounded: unverified
  passes on       `a = 4, b = 2`
  no evidence on  (a, b) in {(0, 1), (0, 2), (0, 3), (0, 4), (1, 1), (1, 2), (1, 3), (1, 4), (2, 1), (2, 2), (2, 3), (2, 4), (3, 1), (3, 2), (3, 3), (3, 4), (4, 1), (4, 3), (4, 4)}
```

## Tuples

`(A, B)`: a pair, written `(1, "a")`, read `t.0`, `t.1`.

```mitra
fn swap(p: (0..=1, Bool)) -> (Bool, 0..=1) { (p.1, p.0) }
claim swaps() { swap((1, true)) == (true, 1) }
```
```text
swaps: holds

```

## List, Set, Map

`List<T>` (`Vec<T>` is the same), `Set<T>`, `Map<K, V>`.

```mitra
fn count(words: List<String>) -> Map<String, 1..>;
claim counted(words: {["a", "b", "a"]}) { keys(count(words)) == {"a", "b"} }
```
```evidence
evidence a { for count given words = ["a", "b", "a"] gives {"a": 2, "b": 1} }
```
```text
counted: holds

```

## Option, Result

Enums of `std::rust`, to import: `Option<T> { None, Some(value: T) }`,
`Result<T, E> { Ok(value: T), Err(error: E) }`.

```mitra
import std::rust::{Option};
fn find(xs: List<Int>, x: Int) -> Option<0..>;
claim finds() { find([7, 8], 8) == Some(1) }
```
```evidence
evidence a { for find given xs = [7, 8], x = 8 gives Some(1) }
```
```text
finds: holds

```

## Type parameters

`fn first<T>(xs: List<T>) -> T`, `record Pair<A, B> { … }`, `enum
Tree<T> { … }`: each parameter is any value.

```mitra
record Pair<A, B> { first: A, second: B }
fn swap<A, B>(p: Pair<A, B>) -> Pair<B, A> { Pair(p.second, p.first) }
claim swaps() { swap(Pair(1, "a")) == Pair("a", 1) }
```
```text
swaps: holds

```

## Function types

`fn(A) -> B`: the functions declared above that fit. The name of a
function is then a value.

```mitra
fn double(x: Int) -> Int { x * 2 }
fn twice(f: fn(Int) -> Int, x: Int) -> Int { f(f(x)) }
claim twices() { twice(double, 3) == 12 }
```
```text
twices: holds

```
