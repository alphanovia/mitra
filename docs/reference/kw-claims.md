# Keywords: claims

Part of the [language reference](language.md); index in
[keywords.md](keywords.md). Every block fenced `mitra` is a whole model,
`evidence` is evidence for it, `text` is what `mitra eval` prints for
the two. A test runs them all.

## claim

`claim name(inputs) { e }`: for every value of the inputs, `e` is true.
The verdict is `holds`, `fails`, `unverified` or `holds assuming […]`.
An input with no type takes the type of the function it is passed to.

```mitra
subject fn abs(x: -2..=2) -> 0..=2;
claim abs_positive(x) { abs(x) >= 0 }
```
```evidence
evidence a { for abs given x = -2 gives 2 }
evidence b { for abs given x = 1 gives 1 }
```
```text
abs::correct: unverified
  no evidence on  x in -2..=2
abs_positive: unverified
  passes on       `x = -2`, `x = 1`
  no evidence on  `x = -1`, `x = 0`, `x = 2`
```

A claim with no input pins one point. It is what a test of the code
checks ([evidence.md](evidence.md), Tests).

```mitra
subject fn div(a: 0..=9, b: 0..=9) -> 0..=9;
claim div_small() { div(6, 2) == 3 }
```
```evidence
evidence t { for div_small holds source "test:div_by_two" }
```
```text
div::correct: unverified
  passes on       `div(6, 2)`
  no evidence on  99 inputs
div_small: holds
```

## when

`claim c(inputs) when cond { e }`: only where `cond` holds. On the
inputs alone, the refused inputs leave the claim. On what the code
returns, the claim is not judged where no output was observed.

```mitra
subject fn div(a: 0..=4, b: 0..=2) -> 0..=4;
claim div_bounded(a: 0..=4, b: 0..=2) when b != 0 { div(a, b) <= a }
```
```evidence
evidence t { for div given a = 4, b = 2 gives 2 }
```
```text
div::correct: unverified
  no evidence on  (a, b) in (0..=4, 0..=2)
div_bounded: unverified
  passes on       `a = 4, b = 2`
  no evidence on  (a, b) in {(0, 1), (0, 2), (1, 1), (1, 2), (2, 1), (2, 2), (3, 1), (3, 2), (4, 1)}
```

## given, then

The same claim written as a contract: `given` the inputs, `when` a
condition (optional), `then` what is true.

```mitra
subject fn div(a: 0..=4, b: 0..=2) -> 0..=4;
claim div_bounded { given a: 0..=4, b: 0..=2 when b != 0 then div(a, b) <= a }
```
```text
div::correct: unverified
  no evidence on  (a, b) in (0..=4, 0..=2)
div_bounded: unverified
  no evidence on  (a, b) in {(0, 1), (0, 2), (1, 1), (1, 2), (2, 1), (2, 2), (3, 1), (3, 2), (4, 1), (4, 2)}
```

## trust

`trust n`, from 0 to 1, on a claim, an `assume` or after a `uses` list:
how much the statement itself is trusted, 1 when not said. It never
changes a verdict. In an error, it ranks the statement among the
candidates to be wrong. Also a clause of [evidence](kw-evidence.md#trust).

```mitra
subject fn inc(x: 0..=2) -> 1..=3;
claim inc_is(x: 0..=2) trust 0.6 { inc(x) == x + 1 }
```
```evidence
evidence t { for inc given x = 1 gives 3 }
```
```text
inc::correct: unverified
  no evidence on  `inc(0)`, `inc(1)`, `inc(2)`
inc_is: fails
  fails on        `inc(1)`
  no evidence on  `inc(0)`, `inc(2)`
```

The error then lists the claim among the candidates:

```text
    likely    the code of `inc`, model.mitra:1          none   0.00 to 0.40
    unlikely  the claim `inc_is` itself, model.mitra:2  0.6    0.60 to 1.00
```

## correct

`correct(f(a))` alone in a claim: the code of `f` is right on these
inputs, as judged by a test or a person (`holds` / `fails` evidence on
`f`). Nothing is computed. Inside a larger expression it is a value
with three states ([claims.md](claims.md)).

A function of a subject with no body has this claim without a line for
it, on all its inputs: `f::correct`. Write your own only to claim less,
as below: `div` is not claimed correct on 0, and `div::correct` is not
added. A point (`claim t() { correct(div(6, 2)) }`) narrows nothing.

```mitra
subject calc {
    fn sort(xs: {[], [2, 1]}) -> List<Int>;
    fn div(a: 0..=2, b: 0..=2) -> 0..=2;
    claim div_ok(a, b: 1..=2) { correct(div(a, b)) }
}
```
```evidence
evidence r { for calc::sort given xs = [2, 1] holds source "review:me" }
```
```text
calc::sort::correct: unverified
  passes on       `calc::sort([2, 1])`
  no evidence on  `calc::sort([])`
calc::div_ok: unverified
  no evidence on  (a, b) in (0..=2, 1..=2)
```

## assume

A claim on a boundary, taken as true with no evidence. It prints `holds
assuming [itself]` until observations cover it; a claim that reads it
says so. Evidence against it is an error.

```mitra
boundary clock {
    fn now() -> 0..=9;
    assume noon() { now() == 5 }
}
subject timer {
    fn left() -> 0..=9;
    claim left_fits() { left() + clock::now() <= 9 }
}
```
```evidence
evidence l { for timer::left gives 3 }
```
```text
clock::noon: holds assuming [itself]
timer::left::correct: unverified
  no evidence on  `timer::left()`
timer::left_fits: holds assuming [clock::noon]
```
