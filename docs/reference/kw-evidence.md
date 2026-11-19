# Keywords: evidence

Part of the [language reference](language.md); index in
[keywords.md](keywords.md). Every block fenced `mitra` is a whole model,
`evidence` is evidence for it, `text` is what `mitra eval` prints for
the two. A test runs them all.

Evidence lives in `evidence.mitra`, never in the model. `mitra add`,
`mitra ingest` and `mitra tie` write to it; it can be written by hand.

## evidence

`evidence name { for … }`: one thing someone saw or judged. The name is
free and may repeat.

```mitra
subject fn abs(x: -2..=2) -> 0..=2;
claim abs_is(x) { abs(x) == if x < 0 { 0 - x } else { x } }
```
```evidence
evidence run_1 {
    for abs
    given x = -2
    gives 2
    trust 0.9
    source "trace:run_1"
    rev "3f2a1c"
}
```
```text
abs::correct: unverified
  no evidence on  x in -2..=2
abs_is: unverified
  passes on       `abs(-2)`
  no evidence on  x in -1..=2
```

## for

What the evidence is on: a function or a claim. The last part of the
name is enough when only one has it (`for add`, or `for calc::add`).

## given

`given a = 1, b in 0..=3`: the inputs, by name. An input left out is
every value. An observed output (`gives`) needs one value per input. With no `given` on a claim, the evidence is on the whole
claim.

```mitra
subject fn add(a: 0..=3, b: 0..=3) -> 0..=6;
claim add_ok(a, b) { correct(add(a, b)) }
```
```evidence
evidence zero { for add given b = 0 holds source "review:me" }
evidence one { for add given a = 1, b in 1..=2 holds source "review:me" }
```
```text
add_ok: unverified
  passes on       (a, b) in {(0, 0), (1, 0), (1, 1), (1, 2), (2, 0), (3, 0)}
  no evidence on  (a, b) in {(0, 1), (0, 2), (0, 3), (1, 3), (2, 1), (2, 2), (2, 3), (3, 1), (3, 2), (3, 3)}
```

## holds, fails

On a function: the code is right / wrong on these inputs. On a claim:
the claim is true / false there; with no `given`, someone vouches for
the whole claim.

```mitra
subject fn add(a: 0..=1, b: 0..=1) -> 0..=2;
claim add_ok(a, b) { correct(add(a, b)) }
claim add_commutes(a: 0..=1, b: 0..=1) { add(a, b) == add(b, a) }
```
```evidence
evidence bug { for add given a = 1, b = 1 fails source "test:add_ones" }
evidence r { for add_commutes holds source "review:me" }
```
```text
add_ok: fails
  fails on        `add(1, 1)`
  no evidence on  `add(0, 0)`, `add(0, 1)`, `add(1, 0)`
add_commutes: holds
```

## passes

A test passed on inputs it does not state: what `mitra ingest` writes
for a test tied to a claim with inputs by a comment that pins nothing.
It supports the claim and settles nothing. On a claim or a function
with no input it is `holds`.

```mitra
subject fn add(a: 0..=1, b: 0..=1) -> 0..=2;
claim add_commutes(a: 0..=1, b: 0..=1) { add(a, b) == add(b, a) }
```
```evidence
evidence test { for add_commutes passes source "test:sweep" }
```
```text
add_commutes: unverified
  passes on       inputs not stated (test:sweep)
```

## sampled

`sampled 1000`: a test ran on that many inputs drawn from a domain,
and passed. The domain is the `given` before it: a set, or a helper of
the model that says which values are in (`m in valid`); with none, the
inputs of the claim. The count and the domain are the author's word. A
sample covers no input: the claim stays `unverified`, with the sample
under it. `sampled 1000 fails`: one of them failed, the claim fails,
and no input is named. From a test: `// mitra: evidence for halves
given a in small sampled 1000`.

```mitra
fn small(a: Int) -> Bool { a >= 0 && a < 100 }
subject fn half(a: Int) -> Int;
claim halves(a: Int) { half(a) * 2 <= a }
```
```evidence
evidence sweep { for halves given a in small sampled 1000 source "test:sweep" }
evidence other { for halves given a in 100..=199 sampled 20 fails source "test:other" }
```
```text
halves: fails
  fails on        one of 20 drawn from (a in 100 ..= 199), not stated (test:other)
  sampled on      1000 drawn from (a in small) (test:sweep)
```

## gives

`gives value`: the output observed on these inputs. `gives rate = 0.9`
(or `precision`, `recall`, `false_positive_rate`): a measurement, on a
claim that holds one measure.

```mitra
subject fn up(day: 1..=4) -> Bool;
claim day_one() { up(1) }
claim mostly_up() { rate(d in 1..=4, up(d)) >= 0.75 }
```
```evidence
evidence o { for up given day = 1 gives true }
evidence m { for mostly_up gives rate = 0.5 }
```
```text
up::correct: unverified
  no evidence on  day in 1..=4
day_one: holds
mostly_up: fails
  fails on        the statement itself
```

## trust

`trust n`, from 0 to 1, 1 when not said: how much this piece is
believed. It never changes a verdict. When two pieces contradict each
other, it ranks which is more plausibly wrong.

```mitra
subject fn inc(x: 0..=2) -> 1..=3;
claim inc_ok(x) { correct(inc(x)) }
```
```evidence
evidence a { for inc given x = 1 holds source "review:me" trust 0.6 }
evidence b { for inc given x = 1 fails source "test:inc_one" }
```
```text
inc_ok: unverified
  no evidence on  `inc(0)`, `inc(1)`, `inc(2)`
```

## source

`source "test:name"`, `"review:who"`, `"trace:run"`: where it comes
from. The name of the block when not said. Errors quote it.

## rev

`rev "3f2a1c"`: the revision of the code it was seen on. `mitra eval
--rev R` counts only the evidence recorded at `R`.

## seen

`seen "c998bf7e792a"`: the fingerprint of the piece when a person said
this. `mitra add` writes it. Once the piece says something else the
entry is stale: kept, shown, judging nothing ([ledger.md](ledger.md)).

```mitra
subject fn half(n: Int) -> Int;
claim small() { half(4) == 2 }
```
```evidence
evidence r { for small holds source "review:ana" seen "000000000000" }
```
```text
small: unverified
warning[M108]: evidence said before its piece changed
```
