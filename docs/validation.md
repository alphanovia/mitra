# Validation: the `add` ladder

How we find out whether Mitra's diagnosis is any good: toy CLI apps whose
correctness we know exactly, annotated in Mitra, with Mitra's output scored
against that ground truth. The [field trials](trials.md) test Mitra on
real work, where the truth is not known.

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
| R0 | naive: parse, `a + b` | usage only | `fits` ok, the other three wrong | No usage: unverified everywhere. After enough usage: `fits` holds, the others fail on the first trusted counterexample, unverified where usage never went. Honest alternatives (`add_wraps`) hold. |
| R1 | naive | usage + example tests on small numbers | same as R0 | `fits` trusted early; `next` points at the untested regions, where the bugs are. |
| R2 | checks overflow, input and arity | example tests per region | all ok | Strong trust in every region with little usage. |
| R3 | as R2 | + property-based tests | all ok | Same as R2, reached with fewer observations. |
| R4 | as R2, core `add` proven with Verus | proof + tests | all ok | Shows what is proven (the arithmetic) and what is only tested (argument parsing), and where to explore next. |

## Evidence

- **Model.** Each rung has a store: a model aside from the code
  (subjects, claims, point claims), the code tied to it, and test
  reports ingested as evidence for the claims the tests name.
- **Usage.** A simulator plays a user: it samples inputs (mostly small numbers,
  sometimes huge ones, occasionally junk), runs the CLI, and appends
  an evidence block that `holds` or `fails` on `add` for these inputs, with trust `c`.
  Evidence arrives one run at a time, which exercises incremental use.
- **Human trust.** The simulated user sets the bias of each run: high
  when it can check the result, low when it cannot, and it is occasionally
  wrong. A wrong run then shows as an error, and Mitra's ranking must
  point at it.

## Metrics

1. **Evidence to verdict (headline curve).** For each rung, the number of
   usage runs until every region has the correct verdict (holds or fails, not
   unverified). Plotted per rung, this should drop sharply from R0 to R4.
2. **Accuracy.** No region ever gets the wrong verdict.
3. **Honesty.** Regions with no evidence are unverified. A doubtful
   counterexample leaves a region unverified, never holds.
4. **Diagnosis.** On R1, `mitra next` points first at a region with a real
   bug. On R0 without regions, `add_correct` shows contested inputs (passes
   and failures on the same claim), suggesting the split.

## Success criteria

The metrics above check Mitra's verdicts. These check that Mitra is useful
([vision.md](vision.md#what-success-looks-like)):

1. **Root-cause rank.** Inject mutations (one wrong function at a time) into
   the add ladder and into `mitra-ds`, add a bug report as
   evidence, and record the rank of the mutated function among the
   candidates of the error. Target: top 1 to 3. `scripts/breaks.py`
   does the injection on Mitra's own crates
   ([dogfooding](design/dogfooding.md)).
2. **Tests to nothing missed.** Count the tests needed until every claim
   holds when following `mitra plan` and `mitra next`, against a naive baseline (random
   inputs) and a coverage-guided one. Target: clearly fewer.
3. **Escaped defects.** On the dogfood repository, count bugs found after a
   claim covering them held. Target: rare, and each one explained by a
   named assumption.
4. **Culprit rank.** On an error with a known cause (a wrong test, a wrong
   assumption, a wrong function), the cause is the most plausible piece
   in the report.

## Does Mitra help? (proposed)

Proposed 2026-11-18, not run. Beside the criteria above, not instead of
them. Mitra is for work where agents write the model and the code, and a
person gives the intent and reads the model and the report. So two
things are measured, each with and without Mitra.

**A. Does it help an agent produce the code?** The same tasks (a spec in
prose, hidden acceptance tests), two arms: the agent alone, and the agent
with a Mitra model and `mitra eval` in its loop.

| Measured | How |
|---|---|
| Hidden tests passed | The acceptance tests the agent never sees. |
| Rounds to done | Edits until the agent stops. |
| Requirements dropped silently | Requirements of the prose spec with no code and no mention in the result. With Mitra: a claim with no evidence, or no claim at all. |

**B. Does it catch a sloppy or drifting agent?** Planted faults, one at a
time, on a project with a model, reusing `scripts/breaks.py`: code faults
(today's lists), and intent faults: a requirement ignored, a test
weakened until it passes, a claim left without evidence, a signature
changed without the model (`mitra tie`). Measured: faults flagged, false
alarms on unchanged code, and whether the report names the faulty piece.
Baseline: `cargo test` alone.

**C. Does it help a person review the result?** The same change reviewed
from the diff alone, and from the model, `mitra tie` and the report.
Measured: planted faults found, time spent, and what the reviewer says
they relied on. Small numbers: this is a study of a few people, read as
cases, not as statistics.

Open: who writes the tasks and the planted faults so that neither arm is
favoured; whether the agent of arm A writes the model itself or receives
it; how many tasks make a difference visible.

## Open questions

- **Who writes the split?** Here it is written from the spec. A stronger result
  would be Mitra suggesting it: a claim with both passing and failing usage is
  a conflict, which is a hint that it should be split.
- **What counts as "a human would trust it"?** Whether a person accepts
  a `holds` that rests on a reviewed assumption A, after reading A, to be compared with their own
  judgment on R0 and R1.
