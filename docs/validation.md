# Validation: the `add` ladder

How we find out whether Mitra's diagnosis is any good: toy CLI apps whose
correctness we know exactly, annotated in Mitra, with Mitra's output scored
against that ground truth.

## The toy

`add A B` prints A + B for two integers. Small enough that the truth is known
for every input, rich enough to go wrong in the usual ways: overflow, bad
input, wrong number of arguments.

## Ground truth: input regions

Every input falls in exactly one region, and each region gets its own claim
that `add` is correct on the inputs of that region. The spec says
what correct means in each.

| Region | Inputs | Correct behaviour |
|---|---|---|
| `fits` | two integers whose sum fits in i64 | prints the sum, exit 0 |
| `overflow` | two integers whose sum does not fit | error message, exit ≠ 0 |
| `bad_input` | a non-integer argument | error message, exit ≠ 0 |
| `arity` | not exactly two arguments | usage message, exit ≠ 0 |

Each rung has a ground-truth table: for each region, is the code correct or
not. Mitra never sees it; it is what Mitra's output is scored against.

## The ladder

| Rung | Code | Evidence | Truth | What Mitra should say |
|---|---|---|---|---|
| R0 | naive: parse, `a + b` | usage only | `fits` ok, the other three wrong | No usage: uncertain everywhere. After enough usage: `fits` holds, the others fail on the first trusted counterexample, uncertain where usage never went. Honest alternatives (`add_wraps`) hold. |
| R1 | naive | usage + example tests on small numbers | same as R0 | `fits` trusted early; `next` points at the untested regions, where the bugs are. |
| R2 | checks overflow, input and arity | example tests per region | all ok | Strong trust in every region with little usage. |
| R3 | as R2 | + property-based tests | all ok | Same as R2, reached with fewer observations. |
| R4 | as R2, core `add` proven with Verus | proof + tests | all ok | Shows what is proven (the arithmetic) and what is only tested (argument parsing), and where to explore next. |

## Evidence

- **Annotations.** Each rung's source carries ` ```mitra ` blocks in its
  docstrings: subjects, claims, tests and proofs. Mitra parses them into the model.
- **Usage.** A simulator plays a user: it samples inputs (mostly small numbers,
  sometimes huge ones, occasionally junk), runs the CLI, and appends
  `evidence add(a, b) correct @ c` or `wrong @ c`. Evidence arrives one run at
  a time, which exercises incremental use.
- **Human confidence.** The simulated user has a confidence profile: high
  when it can check the result, low when it cannot, and occasionally wrong.
  Mitra's verdicts must survive that noise.

## Metrics

1. **Evidence to verdict (headline curve).** For each rung, the number of
   usage runs until every region has the correct verdict (holds or fails, not
   uncertain). Plotted per rung, this should drop sharply from R0 to R4.
2. **Accuracy.** No region ever gets the wrong verdict; the confidence of
   correct verdicts is scored against ground truth (Brier score).
3. **Honesty.** Regions with no evidence are uncertain. A doubtful
   counterexample leaves a region uncertain, never holds.
4. **Diagnosis.** On R1, `next add_correct` ranks a region with a real bug
   first. On R0 without branches, `conflicts` fires on `add_correct` (passes
   and failures on the same claim), suggesting the split.

## Open questions

- **Who writes the split?** Here it is written from the spec. A stronger result
  would be Mitra suggesting it: a claim with both passing and failing usage is
  a conflict, which is a hint that it should be split.
- **What counts as "a human would trust it"?** The decided thresholds above are
  a first guess, to be compared with a person's own judgment on R0 and R1.
