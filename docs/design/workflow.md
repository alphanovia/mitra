# Design: the developer loop

Status: **proposal**, not implemented. Today's CLI has only `mitra eval`
([reference](../reference/language.md#running-mitra)).

## The loop

1. Write the code and the claims (in `mitra` blocks in comments, or in
   `.mitra` files).
2. Run the tests, locally or in CI. Ingest the reports as evidence.
3. Ask Mitra for a diagnostic.
4. Add manual evidence, such as a review, for the current revision.
5. Add usage statistics: evidence that the code works in real use.
6. Report a bug as evidence, and get a ranked list of likely root causes.

## Commands

| Command | Does |
|---|---|
| `mitra check` | Parse every model, report errors and contested evidence |
| `mitra ingest junit\|cargo-json\|pytest <report> --rev <sha>` | Turn a test report into evidence for that commit |
| `mitra add "<evidence>" --source review:<who>` | Record manual evidence with its source |
| `mitra ingest usage <file>` | Turn usage logs into evidence, and update the usage profile |
| `mitra bug "<evidence>"` | Record a bug, for example `not_not(true) -> false @ 0.9` |
| `mitra diag` | Verdicts, uncertain claims, suspects, conflicts |
| `mitra next` | The evidence that would change verdicts the most |

## Tying CI test runs to evidence

Each test states what it covers, next to its code:

```rust
// mitra: t_small exercises add(x in [0, 10]) judges add by add_spec
#[test]
fn t_small() { … }
```

`mitra ingest` reads the standard report formats (JUnit XML, cargo's JSON
output, pytest) and matches tests **by name**. A passing test derives
evidence on `add` over `[0, 10]`; a failing one derives "wrong somewhere in
[0, 10]" ([composition.md](composition.md#3-tests-are-subjects-too)). CI
needs no special plugin, only a step that runs `mitra ingest` and stores the
result.

## Evidence store

- Every piece of evidence records the **commit** it was observed at and a
  **content hash** of the subject it observed (its source, or its AST).
- When a subject changes, its old evidence becomes **stale**. It is not
  deleted; the policy decides how much stale evidence still counts.
- The store is an **append-only log** of Mitra evidence lines, each
  identified by its content hash. It lives in git, either under
  `.mitra/evidence/` or as git notes. Merging two branches is just the union
  of their lines, so merges never conflict.

## Bugs and root causes

`mitra bug` adds the bug report as a counterexample. `mitra diag` then ranks
where the bug most likely is:

1. the weakest premise of the refuted claim (lowest belief first);
2. evidence that went stale because its subject changed;
3. subjects linked to the refuted claim by `uses` that changed since the
   last time the claim held.
