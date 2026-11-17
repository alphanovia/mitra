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
(a `split` of `add_ok`). The spec says what correct means in each.

| Region | Inputs | Correct behaviour |
|---|---|---|
| `small` | two integers whose sum fits in i64 | prints the sum, exit 0 |
| `overflow` | two integers whose sum does not fit | error message, exit ≠ 0 |
| `bad_input` | a non-integer argument | error message, exit ≠ 0 |
| `arity` | not exactly two arguments | usage message, exit ≠ 0 |

Each rung has a ground-truth table: for each region, is the code correct or
not. Mitra never sees it; it is what Mitra's output is scored against.

## The ladder

| Rung | Code | Evidence | Truth | What Mitra should say |
|---|---|---|---|---|
| R0 | naive: parse, `a + b` | usage only | `small` ok, the other three wrong | No usage: vacuous everywhere. After enough usage: confident `small` is correct, confident the others are not, still uncertain about regions usage rarely reaches. |
| R1 | naive | usage + example tests on small numbers | same as R0 | `small` trusted early; `next` points at the untested regions, where the bugs are. |
| R2 | checks overflow, input and arity | example tests per region | all ok | Strong trust in every region with little usage. |
| R3 | as R2 | + property-based tests | all ok | Same as R2, reached with fewer observations. |
| R4 | as R2, core `add` proven with Verus | proof + tests | all ok | Shows what is proven (the arithmetic) and what is only tested (argument parsing), and where to explore next. |

## Evidence

- **Annotations.** Each rung's source carries ` ```mitra ` blocks in its
  docstrings: claims, splits, tests and proofs. Mitra parses them into the model.
- **Usage.** A simulator plays a user: it samples inputs (mostly small numbers,
  sometimes huge ones, occasionally junk), runs the CLI, compares with an
  exact oracle, and appends `evidence add_ok.<region> : usage +1` or `-1`.
  This exercises incremental use: evidence arrives one run at a time.

## Metrics

1. **Evidence to trust (headline curve).** For each rung, the number of usage
   runs until every region is *decided*: P ≥ 0.9 and u ≤ 0.1 if correct,
   P ≤ 0.1 and u ≤ 0.1 if not. Plotted per rung, this should drop sharply
   from R0 to R4.
2. **Accuracy.** Brier score of each region's P against ground truth (1 or 0),
   after N runs.
3. **Honesty.** Regions with no evidence have u = 1. Mitra never reports a
   decided region the ground truth contradicts.
4. **Diagnosis.** On R1, `next add_ok` ranks a region with a real bug first.
   On R0 before splitting, `conflicts` fires on `add_ok` (passes and failures
   on the same claim).

Note for calibration: with the default evidence mapping, u ≤ 0.1 needs at
least 18 observations in a region, so regions that usage rarely reaches stay
uncertain, as they should.

## Open questions

- **Who writes the split?** Here it is written from the spec. A stronger result
  would be Mitra suggesting it: a claim with both passing and failing usage is
  a conflict, which is a hint that it should be split.
- **What counts as "a human would trust it"?** The decided thresholds above are
  a first guess, to be compared with a person's own judgment on R0 and R1.
