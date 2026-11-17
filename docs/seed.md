# The seed

The first code: an embeddable library, `mitra-core`, and a thin CLI, `mitra`.

```sh
cargo run -p mitra-cli -- eval model.mitra evidence.mitra -o result.txt
```

## Inputs

The model declares the subject under review and the claims about it:

```mitra
subject not(a: bool) -> bool

claim not_correct: forall a: bool. not(a) = !a
claim not_on_true: not(true) = false
```

- A **subject** is a function, task or process under review. Mitra never
  runs it.
- A **claim** says the subject is correct on a set of inputs: all of them
  with `forall`, or a subset by fixing values. The right-hand side is the
  specification; it is parsed but not evaluated yet.

Evidence is a run of the subject with an opinion on its result:

```mitra
evidence not(true) correct @ 1
evidence not(false) wrong @ 1
```

`@` is the confidence in that opinion: 1 is fully believed, 0 is fully
distrusted. **Only 0 and 1 are supported for now.** Values are booleans only.

## Output

One line per claim: a verdict (`holds`, `fails` or `uncertain`) and a
certainty, whose exact meaning (probability, opinion, ...) is not decided yet.

```text
not_correct  fails @ 1
not_on_true  holds @ 1
```

## Rules at the extremes

1. Evidence at 0 brings nothing.
2. For each input a claim covers, fully believed evidence says it is
   correct, wrong, nothing, or both (a contradiction).
3. One input known wrong: the claim **fails @ 1**.
4. Every input known correct: the claim **holds @ 1**.
5. Otherwise (no evidence, missing inputs, a contradiction): **uncertain @ 0**.

Any calculus for confidences between 0 and 1 must give these results at the
extremes.

## Validation

`tests/golden/<subject>/<case>/` holds `model.mitra`, `evidence.mitra` and
the `expected.txt` Mitra must produce, written by hand. `cargo test` checks
every case, and checks that adding evidence at 0 to any case changes nothing.

| Case | not_correct | not_on_true |
|---|---|---|
| 01 no evidence | uncertain @ 0 | uncertain @ 0 |
| 02 correct on both | holds @ 1 | holds @ 1 |
| 03 correct on true only | uncertain @ 0 | holds @ 1 |
| 04 wrong on false only | fails @ 1 | uncertain @ 0 |
| 05 correct on true, wrong on false | fails @ 1 | holds @ 1 |
| 06 contradiction on false | uncertain @ 0 | holds @ 1 |
| 07 contradiction on true | uncertain @ 0 | uncertain @ 0 |
| 08 contradiction on false, wrong on true | fails @ 1 | fails @ 1 |
| 09 only evidence at 0 | uncertain @ 0 | uncertain @ 0 |
| 10 both correct @ 1, wrong on false @ 0 | holds @ 1 | holds @ 1 |

## Later

Confidences between 0 and 1, flagging contradicting evidence, evidence
shorthands (proof, test, usage), other value types. The core has no
dependencies so it can be proven with Verus and Lean 4, as planned.
