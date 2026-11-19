# Keywords: claims with parameters, the library

Part of the [language reference](language.md); index in
[keywords.md](keywords.md). Every block fenced `mitra` is a whole model,
`evidence` is evidence for it, `text` is what `mitra eval` prints for
the two. A test runs them all.

## Claims with parameters

`claim c(f, T) { given a: T then … }`: names without a type, a contract
as body. Not a claim by itself. `claim d = c(g, 0..=9);` is that claim
with `f` and `T` replaced by a function, a set, a value or an inline
function `|a, b| b > 0`. Written above its use.

```mitra
claim never_above(f, T, top) { given a: T then f(a) <= top }
fn clamp(x: 0..=9) -> 0..=9;
claim clamp_low = never_above(clamp, 0..=9, 5) trust 0.9;
```
```evidence
evidence t { for clamp given x = 7 gives 5 }
```
```text
clamp_low: unverified
  passes on       `a = 7`
  no evidence on  a in 0..=6 | 8..=9
```

## std::claims

`import std::claims::{…};`. Each is a claim with parameters; `mitra
explain std::claims` prints them.

| Use | Says |
|---|---|
| `commutative(f, T)` | `f(a, b) == f(b, a)` |
| `associative(f, T)` | `f(f(a, b), c) == f(a, f(b, c))` |
| `idempotent(f, T)` | `f(f(a)) == f(a)` |
| `neutral(f, T, zero)` | `f(a, zero) == a && f(zero, a) == a` |
| `inverse(f, g, T)` | `g(f(a)) == a` |
| `monotone(f, T)` | `a <= b` gives `f(a) <= f(b)` |
| `within(f, T, low, high)` | `low <= f(a) <= high` |
| `uniform(f)`, `uniform(f, case)` | see [uniform](#uniform) |
| `represented(f, S)` | see [represented](#represented) |

```mitra
import std::claims::{commutative, neutral, within};
fn add(a: 0..=1, b: 0..=1) -> 0..=2;
claim add_commutes = commutative(add, 0..=1);
claim add_zero = neutral(add, 0..=1, 0);
fn half(x: 0..=4) -> 0..=2;
claim half_small = within(half, 0..=4, 0, 2);
```
```evidence
evidence a { for add given a = 0, b = 1 gives 1 }
evidence b { for add given a = 1, b = 0 gives 1 }
```
```text
add_commutes: unverified
  passes on       `add(0, 1)`, `add(1, 0)`
  no evidence on  `add(0, 0)`, `add(1, 1)`
add_zero: unverified
  passes on       `a = 1`
  no evidence on  `a = 0`
half_small: unverified
  no evidence on  a in 0..=4
```

## uniform

`uniform(f)`: every input of `f` behaves the same, so one tested input
stands for all. `uniform(f, |a, b| b == 0)`: the same within each
class. Nobody can compute it: a reviewer vouches for it. One per
function.

```mitra
import std::claims::{uniform};
fn div(a: 0..=9, b: 0..=9) -> 0..=9;
claim div_ok(a, b) { correct(div(a, b)) }
claim div_classes = uniform(div, |a, b| b == 0);
```
```evidence
evidence t { for div given a = 6, b = 2 holds source "test:div_by_two" }
evidence r { for div_classes holds source "review:me" }
```
```text
div_ok: unverified
  passes on       `div(6, 2)`
  assumed on      class false of `div_classes`, by `div_classes`
  no evidence on  (a, b) in {(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0)}
div_classes: holds
```

## represented

`represented(f, S)`: one tested input of the set `S` stands for all of
`S`. Nothing is said outside `S`. Several per function.

```mitra
import std::claims::{represented};
fn div(a: 0..=9, b: 0..=9) -> 0..=9;
claim div_ok(a, b) { correct(div(a, b)) }
claim div_rest = represented(div, { (a, b) | b != 0 });
```
```evidence
evidence t { for div given a = 6, b = 2 holds source "test:div_by_two" }
evidence r { for div_rest holds source "review:me" }
```
```text
div_ok: unverified
  passes on       `div(6, 2)`
  assumed on      89 inputs, by `div_rest`
  no evidence on  (a, b) in {(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0)}
div_rest: holds
```
