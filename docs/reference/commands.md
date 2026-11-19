# Commands

Part of the [language reference](language.md); says what the code does
today, and changes in the same commit as the code.

```sh
mitra init
mitra check [model.mitra [evidence.mitra]] [--rev <commit>]
mitra eval  [model.mitra [evidence.mitra]] [--rev <commit>] [--candidates <n>]
            [--threshold <p>] [--all] [-o result.txt]
mitra plan  [model.mitra [evidence.mitra]] [--rev <commit>]
mitra next  [model.mitra [evidence.mitra]] [--rev <commit>]
mitra add   '<evidence blocks>' [--source <who>] [--rev <commit>]
mitra ingest <report> [--format libtest|junit] [--rev <commit>]
mitra tie   [--rev <commit>]
mitra explain <code | claim | std::module>
```

When Mitra can't read or check its input it prints `file: line N:
message` and exits with status 2.

- **The store** is the nearest `.mitra/` at or above the current folder:
  `model.mitra`, every other `.mitra` file beside it (read after it, in
  name order) and `evidence.mitra`. `init` creates it, with a
  `.gitattributes` that merges `evidence.mitra` with git's `union`
  driver, and never overwrites one. The evidence file is only appended
  to, so two branches that add evidence merge as the union of their lines.
- **Without a store**, name the model and the evidence file, or the
  model alone: it is then read with no evidence, which is the way to
  check a model while writing it. `explain <claim>` takes them too.
- **`check`** reads and evaluates, and prints `ok: F functions, C claims,
  E evidence`.
- **`eval`** prints each claim's verdict, then errors and warnings (see
  Output), to standard output or to the `-o` file (not a `.mitra` file).
  It prints `mitra: E errors, W warnings` on standard error and exits
  with 1 when there is an error. An error lists at most 5 candidates and
  none under a plausibility of 0.05; `--candidates`, `--threshold` and
  `--all` change that.
- **`plan`** lists the evidence each claim owes (see `mitra plan`).
- **`next`** prints the evidence that can't all be true, each failing
  claim (root causes first), each claim that `holds assuming` with what
  it assumes, then each unverified claim with what it misses and the
  evidence block to write, when at most 5 inputs are missing. With a
  `uniform` claim it lists the classes with no passing input.
- **`add`** takes evidence blocks as one argument, writes `--source` and
  `--rev` into each block that does not say them (twice is an error),
  checks the store with the new evidence, and appends the blocks as
  written. The store must evaluate without a read error before and
  after; otherwise nothing is written.
- **`ingest`** and **`tie`**: see Tests and The tie to the code. Report
  formats: `libtest` (lines `test <path> ... ok|FAILED` as `cargo test`
  prints them) and `junit` (`<testcase name="…">`, failed with
  `<failure` or `<error`, left out with `<skipped`); a report starting
  with `<` is JUnit. A test's name is the last `::` part of its path. A
  test that appears twice in a report is left out, with a note.
- **`explain`** describes an error code (`mitra explain M004`), prints a
  library module (`mitra explain std::claims`), or prints a claim as
  written, as read, the count of evidence on it and its verdict
  (`mitra explain div_rest`, the last part of the name is enough).
- **`--rev R`** on `check`, `eval`, `next`, `plan` counts only evidence
  recorded at `R`: any other, or none, is stale and left out, and the
  number left out is printed. Revisions are compared as text. A revision
  must not be empty, start with `-`, or hold quotes or line breaks.

## `mitra plan`: the evidence owed

`mitra plan` lists, for each claim that does not hold yet, the evidence
it still owes. With no evidence it is the list of tests to write, from
the model alone; with evidence, what is left. `nothing owed` when every
claim holds.

```
calc::div::spec owed:
  any one point of { (a, b) | !(b == 0) }
  any one point of { (a, b) | b == 0 }
  and `calc::div_classes`, which says one point is enough
calc::div_classes owed: a review that `calc::div` behaves the same within each class
calc::div_small owed:
  calc::div(6, 2), expected 3
  or nothing, once calc::div::spec holds
calc::add_commutes owed:
  calc::add(0, 0)
  calc::add(0, 1)
```

- A claim with a `uniform` claim on its function owes one point per class
  not yet tested; with `represented` claims, one point per set not yet
  tested, then the points no set covers.
- Otherwise it owes each call it reads and that was not observed, when
  there are at most 12 points; `expected` is what the spec or the claim
  itself says the output is (`f(…) == e`, with `e` computed).
- A larger or infinite set owes "every point, or a claim one point can
  stand for".
- A claim that holds only through other claims (`holds assuming`) still
  lists its own points, with "or nothing, once … holds".
- `uniform`, `uses` and ties owe a review or `mitra tie`. An assumption
  owes nothing. A failing claim owes nothing: `mitra eval` says why.

A pinned point is a claim: `claim div_small() { div(6, 2) == 3 }`.
`examples/reworked/plan`.
