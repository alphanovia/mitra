# Language spec (draft 3)

Mitra makes the graph of trust explicit and gives every claim a **verdict**
and a **confidence**. This page is the meaning; the syntax is one projection
of it and may change. The confidence calculus is deliberately minimal: just
enough for the toy models, with the real choice left open (see the end and
[research/calculus.md](research/calculus.md)).

## Claims

A claim reads like a function over mathematical values. It is untyped in the
MVP: arguments are integers, rationals or strings, with exact arithmetic
(`a + b` never overflows in Mitra).

```mitra
subject add "the CLI `add A B`, observed from outside"

claim add_correct(a, b): add(a, b) = a + b
```

- A **subject** is something outside Mitra that evidence observes: a CLI, a
  function, an agent. Mitra never runs it. `add(a, b)` means "what the subject
  outputs on a, b".
- Everything else in a claim is mathematics, which Mitra can evaluate.
  `error` is a distinguished value: the subject reports an error (non-zero
  exit, exception).
- A claim is **universal**: it holds for every value of its arguments.

## Narrowing: guarded branches

A claim can be split by guards, in the spirit of ACSL behaviours. Each branch
is a sub-claim with its own name and its own verdict.

```mitra
claim add_correct(a, b):
  fits     when -2^63 <= a + b < 2^63 : add(a, b) = a + b
  overflow otherwise                  : add(a, b) = error
```

- Branches are checked in order; `otherwise` makes them exhaustive by
  construction, so no separate "the split is complete" claim is needed.
- Sub-claims are named `add_correct.fits` and `add_correct.overflow`.
- The parent holds when every branch holds.

**Honest alternatives.** When a claim turns out false, write what the subject
actually does as another claim. Both are scored on the same evidence:

```mitra
claim add_wraps(a, b): add(a, b) = (a + b + 2^63) mod 2^64 - 2^63
```

## Evidence

```text
evidence <kind>(<source>) : <claim>(<inputs>) = pass | fail @ <confidence>
```

```mitra
evidence usage(session_42) : add_correct(2, 3)         = pass @ 0.9
evidence usage(session_43) : add_correct(2^62, 2^62)   = fail @ 0.95
evidence test(t_small)     : add_correct(1, 1)         = pass @ 0.99
evidence proof(verus_add)  : add_correct.fits(*)       = pass @ 0.95
```

- **What was checked:** a claim (or branch) on concrete inputs, or `(*)` for
  every input of it (a proof, an exhaustive or property-based test). Mitra
  evaluates guards to route concrete inputs to the right branch.
- **The verdict observed:** `pass` or `fail`.
- **Confidence (0 to 1):** how sure the source is of that verdict. A person
  who checked with two calculators says `@ 0.95`; one who thinks the output
  looks wrong but could not verify it says `fail @ 0.3`.
- **Observations.** When the output itself is known, state it instead of a
  verdict: `observe usage(session_43) : add(2^62, 2^62) = -2^63 @ 0.95`.
  Mitra derives pass or fail for every claim about `add`, so a claim and its
  honest alternative are scored on the same evidence.
- **Re-checks.** For the same claim and inputs, the most confident
  observation wins: re-checking a doubtful counterexample settles it.
- **Kind:** `test`, `proof`, `usage`, `review`, `agent`. Used by policy and
  reported in diagnostics. **Source** names who or what produced it, so that
  evidence from one source is never counted as independent confirmation.

## Output: verdict and confidence

Every claim gets two values, never a single score:

```text
add_correct          fail      @ 0.95
  add_correct.fits      uncertain (leaning pass @ 0.61)
  add_correct.overflow  fail      @ 0.95
```

Rules for a universal claim (minimal, for the toy models):

1. **Counterexamples decide.** The fail support *F* is the highest confidence
   of any `fail` on the claim. If F ≥ θ_fail (default 0.5), the verdict is
   **fail @ F**: one trusted counterexample refutes a universal claim.
2. **Universal passes decide.** `(*)` passes at confidences cᵢ give
   U = 1 − ∏(1 − cᵢ).
3. **Instances only nudge.** Each concrete pass at confidence c adds a small
   weight κ·c (κ = 0.1 by default, a policy):
   S = 1 − (1 − U) · ∏(1 − κ·cᵢ).
   If S ≥ θ_pass (default 0.9) and there is no fail evidence at all, the
   verdict is **pass @ S**. A standing counterexample, however doubtful,
   blocks a pass: it must be re-checked.
4. Otherwise the verdict is **uncertain**, reported with its leaning (the
   larger of S and F).
5. A **parent** with branches fails if any branch fails (at that branch's
   confidence), passes if all pass (at the lowest confidence), otherwise is
   uncertain.

## Assumptions and policy

```mitra
assume x_covers_add "test X exercises add's specified behaviour" @ 0.8
policy usage kappa = 0.1
policy decide pass >= 0.9, fail >= 0.5
```

An assumption is a claim accepted at a stated confidence, not computed. Every
verdict lists the assumptions and policies it rests on.

## Diagnostics

| Query | Answers |
|---|---|
| `query c` | c's verdict and confidence, its branches, and what it rests on. |
| `uncertain` | Every claim whose verdict is uncertain, closest to deciding first. |
| `next c` | Which evidence would most change c's verdict. A low-confidence counterexample is always first: re-check it. |
| `conflicts` | Claims with both pass and fail support. On an unbranched claim this suggests a split: it holds on some inputs and not others. |

## Open questions

- **The confidence calculus.** Rules 1 to 5 are a placeholder. Candidates are
  in [research/calculus.md](research/calculus.md).
- **Universal vs statistical claims.** Usage samples support "fails rarely"
  far better than "never fails". Should `κ` be replaced by an explicit
  uniformity assumption ("all small pairs take the same path")?
- **Relationships as claims.** Can a confidence refer to another claim
  (`@ x_ok`), so that "test X is correct" is scored rather than assumed?
- **Mathematical types.** Are integers, rationals and strings enough for the
  MVP?
