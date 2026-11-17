# Toy models

Small models with their results worked out by hand from the draft rules in
[language.md](language.md) (θ_pass = 0.9, θ_fail = 0.5, κ = 0.1). They are the
first tests. All use the `add` CLI from [validation.md](validation.md).

```mitra
subject add "the CLI `add A B`"
claim add_correct(a, b): add(a, b) = a + b
```

## 1. Nothing known

```text
add_correct   uncertain (no evidence)
```

## 2. Usage, then a counterexample

The naive CLI. A user tries 30 small sums and checks each one carefully.

```mitra
evidence usage(user) : add_correct(2, 3) = pass @ 0.9
# ... 30 such lines, all small inputs
```

S = 1 − (1 − 0.1 · 0.9)³⁰ = 1 − 0.91³⁰ = **0.941**. After 10 passes it was
1 − 0.91¹⁰ = 0.611, still uncertain.

```text
add_correct   pass @ 0.94
```

This is too generous: every input was small, and nothing was learned about
large ones. Example 4 fixes this with branches. Then the user tries a large
sum and checks it with two calculators:

```mitra
evidence usage(user) : add_correct(2^62, 2^62) = fail @ 0.95
```

```text
add_correct   fail @ 0.95      counterexample: (2^62, 2^62)
```

One trusted counterexample refutes a universal claim, whatever the passes said.

## 3. A doubtful counterexample

Same 30 passes, but the failure comes from someone who thinks the output looks
wrong and could not verify it:

```mitra
evidence usage(other_user) : add_correct(2^62, 2^62) = fail @ 0.3
```

F = 0.3 < θ_fail, so not refuted; but a standing counterexample blocks a pass.

```text
add_correct   uncertain (leaning pass @ 0.94; counterexample @ 0.3)
next: re-check add_correct(2^62, 2^62)
```

The re-check comes back `fail @ 0.95`; the most confident observation on that
input wins, and the verdict becomes **fail @ 0.95** as in example 2.

## 4. Narrowing, and what is actually true

Branches separate what usage covered from what it did not, and an honest
alternative says what the naive CLI really does (a release build wraps).

```mitra
claim add_correct(a, b):
  fits     when -2^63 <= a + b < 2^63 : add(a, b) = a + b
  overflow otherwise                  : add(a, b) = error

claim add_wraps(a, b): add(a, b) = (a + b + 2^63) mod 2^64 - 2^63

observe usage(user) : add(2, 3) = 5 @ 0.9
# ... 30 small sums, as before
observe usage(user) : add(2^62, 2^62) = -2^63 @ 0.95
```

The guards route the 30 small sums to `fits` and the large one to `overflow`.
The same observations are scored against both claims.

```text
add_correct            fail @ 0.95
  add_correct.fits       pass @ 0.94
  add_correct.overflow   fail @ 0.95     counterexample: (2^62, 2^62)
add_wraps              pass @ 0.95
```

`add_wraps`: 31 passing instances, S = 1 − 0.91³⁰ · (1 − 0.095) = 0.947.
This is the picture a person builds by exploring: correct on sums that fit,
wrong on overflow, and what it does instead.

## 5. Proof and tests: strong trust with little exploration

The fixed CLI. The core addition is proven with Verus; the overflow path has
one test.

```mitra
evidence proof(verus_add) : add_correct.fits(*)               = pass @ 0.95
evidence test(t_ovf)      : add_correct.overflow(2^62, 2^62)  = pass @ 0.99
```

- `fits`: universal pass, U = 0.95 → **pass @ 0.95**, with no usage at all.
- `overflow`: one instance, S = 0.1 · 0.99 = 0.099 → uncertain.

```text
add_correct            uncertain (overflow undecided)
  add_correct.fits       pass @ 0.95     proven (verus_add)
  add_correct.overflow   uncertain (leaning pass @ 0.10)
next: evidence for add_correct.overflow(*) (a property test or a proof)
```

Mitra shows what was proven and where exploration is still needed.
