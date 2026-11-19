# Design: the developer loop

Status: **partly implemented**. The references
([language](../reference/language.md), [commands](../reference/commands.md))
describe what exists; the rest is a proposal.

## The loop

1. Write the model, aside from the code: subjects, what each promises
   in plain words (`ensure`), signatures, claims, assumptions. A person or an agent writes it
   ([who it is for](../vision.md#who-it-is-for)).
2. `mitra eval` on the model alone: claims that the specs contradict are
   errors before any code exists. `mitra plan` lists the evidence owed,
   which is the list of tests to write.
3. Write the code and the tests. Above each function,
   `// mitra: implements subject::f`; above each test,
   `// mitra: evidence for claim`.
4. `mitra tie` compares the code with the model (inputs, result, calls).
   Run the tests and `mitra ingest` the report.
5. `mitra eval`, `mitra next`: what holds, what is only assumed, what is
   missing, what conflicts.
6. Add evidence by hand for the current revision: a review that vouches
   for an assumption or that the claims say what a requirement says, a
   bug report on a function or on a requirement.
7. When evidence shows the model is wrong, change the model, then the
   code.

## Commands

| Command | Does | Status |
|---|---|---|
| `mitra init` | Create the store `.mitra/` | done |
| `mitra check`, `mitra eval` | Check the model; print verdicts, errors, warnings | done |
| `mitra plan` | Per claim, the evidence still owed | done |
| `mitra tie --rev <r>` | Compare the tied Rust code with the model, as evidence | done, reads text |
| `mitra ingest <report> --rev <r>` | Turn a test report (libtest, JUnit) into evidence for the claims the tests name | done |
| `mitra add '<evidence>' --source <who> --rev <r>` | Record evidence by hand | done |
| `mitra next` | Per unverified claim, the inputs missed and the evidence to add | done, unranked |
| `mitra explain <code or claim>` | An error code, a claim as written and as read, a library module | done |
| `mitra ingest usage <file>` | Turn usage logs into evidence | proposed |
| `mitra bug '<evidence>'` | Record a bug as a counterexample | proposed; `mitra add` does it today |
| `mitra diag` | Verdicts, suspects, conflicts, ranked root causes | proposed; errors already rank their candidates |

## Tying code and tests to the model (done)

Two comments, read by `mitra tie` and `mitra ingest`
([reference](../reference/evidence.md#the-tie-to-the-code)). CI needs no
plugin, only steps that run them and keep the store. A test names a claim,
usually a point claim; its result is evidence for that claim and, on a
point claim, the observed output of the code.

A test is code too, and may be the piece that is wrong: its evidence is
a candidate in every error it takes part in.

## Evidence store

Done: `.mitra/evidence.mitra` is an append-only file of evidence lines,
each with its `source` and `rev`. Git merges it with the `union` driver, so
merges never conflict. `--rev R` counts only evidence recorded at R.

Proposed: record a **content hash** of the subject each piece observed, so
evidence goes stale only when its subject changes, not on every commit.
Staleness then lowers the trust in that evidence when culprits are ranked.

## Bugs and root causes (proposed)

`mitra bug` adds the bug report as a counterexample. `mitra diag` then ranks
where the bug most likely is: by the plausibility of each piece being
wrong, computed from the report, the `uses` clauses, stale evidence and
the last diff ([semantics](semantics.md#4-opinion-which-piece-is-wrong)).
