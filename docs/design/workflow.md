# Design: the developer loop

Status: **partly implemented**. The [reference](../reference/language.md#running-mitra)
describes what exists; the rest is a proposal.

## The loop

1. Write the code, the claims and the `test` declarations.
2. Run the tests, locally or in CI, and ingest the reports as evidence.
3. Ask Mitra what is uncertain and what to add next.
4. Add manual evidence, such as a review, for the current revision.
5. Add usage statistics: evidence that the code works in real use.
6. Report a bug as evidence, and get a ranked list of likely root causes.

## Commands

| Command | Does | Status |
|---|---|---|
| `mitra init` | Create the store `.mitra/` | done |
| `mitra check`, `mitra eval` | Check the model; print verdicts | done |
| `mitra add '<evidence>' --source <who> --rev <r>` | Record manual evidence | done |
| `mitra ingest <report> --rev <r>` | Turn a test report (libtest, JUnit) into evidence | done |
| `mitra next` | Per uncertain claim, the inputs missing and the evidence to add | done, unranked |
| `mitra ingest usage <file>` | Turn usage logs into evidence and a usage profile | proposed |
| `mitra bug '<evidence>'` | Record a bug as a counterexample | proposed |
| `mitra diag` | Verdicts, suspects, conflicts, ranked root causes | proposed |

## Tying test runs to evidence (done)

The model declares each test with the evidence its result stands for:

```lean
test t_small : ∀ x ∈ Icc 0 10, correct (add x 1)
```

`mitra ingest` matches tests **by name** in the report. CI needs no plugin,
only a step that runs `mitra ingest` and keeps the store. Proposed next:
the declaration next to the test's code, in a comment, and the test's own
claims ([composition](composition.md#3-tests-are-subjects-too)).

## Evidence store

Done: `.mitra/evidence.mitra` is an append-only file of evidence lines,
each with its `source` and `rev`. Git merges it with the `union` driver, so
merges never conflict. `--rev R` counts only evidence recorded at R.

Proposed: record a **content hash** of the subject each piece observed, so
evidence goes stale only when its subject changes, not on every commit.
The policy then decides how much stale evidence still counts.

## Bugs and root causes (proposed)

`mitra bug` adds the bug report as a counterexample. `mitra diag` then ranks
where the bug most likely is:

1. the weakest premise of the refuted claim (lowest belief first);
2. evidence that went stale because its subject changed;
3. subjects linked to the refuted claim by `uses` that changed since the
   claim last held.
