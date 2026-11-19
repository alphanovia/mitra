# Keywords: declarations

Part of the [language reference](language.md); index in
[keywords.md](keywords.md). Every block fenced `mitra` is a whole model,
`evidence` is evidence for it, `text` is what `mitra eval` prints for
the two. A test runs them all.

## subject

Real code under review. A namespace: its functions and claims are named
`calc::add`, `calc::add_small`. Inside it, a sibling is named without
the prefix.

```mitra
subject calc {
    fn add(a: 0..=2, b: 0..=2) -> 0..=4;
    claim add_small(a: 0..=2, b: 0..=2) { add(a, b) <= 4 }
}
```
```evidence
evidence t { for calc::add given a = 1, b = 2 gives 3 }
```
```text
calc::add::correct: unverified
  no evidence on  (a, b) in (0..=2, 0..=2)
calc::add_small: unverified
  passes on       `a = 1, b = 2`
  no evidence on  (a, b) in {(0, 0), (0, 1), (0, 2), (1, 0), (1, 1), (2, 0), (2, 1), (2, 2)}
```

Without a name, what it holds is named as written (`add`,
`add_small`), in the file and in a file that imports it. For one
function, one word is enough: `subject fn add(…) -> T;`. Claims on it
are then written beside it. A file may hold several unnamed blocks.

```mitra
subject fn add(a: 0..=2, b: 0..=2) -> 0..=4;
subject {
    fn half(x: 0..=4) -> 0..=2;
}
claim add_small(a: 0..=2, b: 0..=2) { add(a, b) <= 4 }
```
```text
add::correct: unverified
  no evidence on  (a, b) in (0..=2, 0..=2)
half::correct: unverified
  no evidence on  x in 0..=4
add_small: unverified
  no evidence on  (a, b) in (0..=2, 0..=2)
```

Only a named subject can be pointed at as a whole.

## boundary

The environment: what your code talks to and does not own. Its functions
may answer differently each time, and are never the piece to fix.
Claims on it are written with [assume](kw-claims.md#assume).

```mitra
boundary net {
    fn fetch(url: {"a", "b"}) -> {"ok", "down"};
}
subject loader {
    fn load(url: {"a", "b"}) -> {"page", "default"};
    claim falls_back(url: {"a", "b"}) when net::fetch(url) == "down" { load(url) == "default" }
}
```
```evidence
evidence up { for net::fetch given url = "b" gives "ok" }
evidence down { for net::fetch given url = "b" gives "down" }
evidence l { for loader::load given url = "b" gives "default" }
```
```text
loader::load::correct: unverified
  no evidence on  `loader::load("a")`, `loader::load("b")`
loader::falls_back: unverified
  passes on       `url = "b"`
  no evidence on  `url = "a"`
```

The two answers of `fetch("b")` both stand.

Like a subject, a boundary may have no name, and one function takes one
word:

```mitra
boundary fn now() -> 0..=9;
assume noon() { now() == 5 }
subject fn left() -> 0..=9;
claim left_fits() { left() + now() <= 9 }
```
```evidence
evidence l { for left gives 3 }
```
```text
noon: holds assuming [itself]
left::correct: unverified
  no evidence on  `left()`
left_fits: holds assuming [noon]
```

## fn

Three forms.

`fn f(a: T) -> U;` with no body, in a subject or a boundary (or after
the word: `subject fn f(…);`, `boundary fn f(…);`): code that is not
modelled. Only claims and evidence say something of it. Without `-> U`
the result is `()`. In a subject, Mitra adds the claim `f::correct`: the
code is right on every input ([correct](kw-claims.md#correct)).

`fn f(a: T) -> U { e }` in a subject: the code, and its specification.
Mitra adds the claim `f::spec`: the code returns what the body returns.

`fn f(a: T) -> U { e }` alone: a helper of the model. It is computed,
there is no code behind it, no claim, and it is never a suspect. It is
written above the bodies that call it. Alone, a `fn` with no body is
refused: Mitra asks whether it is your code or the world.

```mitra
fn double(x: Int) -> Int { x * 2 }

subject calc {
    fn half(x: 0..=4) -> 0..=2;
    fn twice(x: 0..=2) -> 0..=4 { double(x) }
    claim back(x: 0..=2) { half(twice(x)) == x }
}
```
```evidence
evidence a { for calc::twice given x = 1 gives 2 }
evidence b { for calc::half given x = 2 gives 1 }
```
```text
calc::half::correct: unverified
  no evidence on  x in 0..=4
calc::twice::spec: unverified
  passes on       `calc::twice(1)`
  no evidence on  `calc::twice(0)`, `calc::twice(2)`
calc::back: unverified
  passes on       `x = 1`
  no evidence on  `x = 0`, `x = 2`
```

## uses

`fn f(…) -> U uses g, h;` says what the code of `f` calls. `uses only g`
says it calls nothing else. It adds the claim `f::uses`, taken as
declared. `mitra tie` checks it against the Rust code.

`uses` says that `f` depends on `g`, not how. Claims on `g` reach a
claim on `f` only through a body, or through a claim that ties the two
([claims.md](claims.md), item 3).

```mitra
subject shop {
    fn price(item: {"pen", "ink"}) -> 1..=9;
    fn total(a: {"pen", "ink"}, b: {"pen", "ink"}) -> 2..=18 uses only price;
}
```
```text
shop::price::correct: unverified
  no evidence on  `shop::price("ink")`, `shop::price("pen")`
shop::total::uses: holds
shop::total::correct: unverified
  no evidence on  (a, b) in ({"ink", "pen"}, {"ink", "pen"})
```

## import

`import std::claims::{commutative};` brings the named declarations of a
module shipped with Mitra: `std::claims` (see
[kw-claims.md](kw-library.md#stdclaims)) and `std::rust` (`Option<T> {
None, Some(value: T) }`, `Result<T, E> { Ok(value: T), Err(error: E)
}`). `mitra explain std::rust` prints a module.

`import geometry::{Point};` brings them from `geometry.mitra`, beside
the model (`a::b::{x}` reads `a/b.mitra`). Only the names listed come.

```mitra
import std::rust::{Option};
fn first(xs: List<Int>) -> Option<Int> { if len(xs) == 0 { None } else { Some(xs[0]) } }
claim firsts() { first([3, 4]) == Some(3) && first([]) == None }
```
```text
firsts: holds

```

## Comments

`//` to the end of the line. A `// mitra: implements calc::add` comment
belongs to the Rust code, not to the model ([evidence.md](evidence.md)).
