# semver benchmark: first run (2026-11-24)

One run of each, by cold agents (Claude Code subagents, same model),
each given only its prompt from [../README.md](../README.md) and the
files that prompt names. One run is an anecdote, not a statistic.

## Numbers

| | Control: a sentence | Mitra: the spec |
|---|---|---|
| Rounds of `cargo test` | 2 | 3 |
| Code lines (`src/`, no tests) | 164 | 156 |
| Tests | 9 (111 lines) | 37 (292 lines) |
| Model lines (non-blank, non-comment) | | 103 (0.66 of the code) |
| Acceptance, 29 rows | 28 | **29** |
| Planted bug: first diagnosis | | the broken function's tie |

Explain, round by round, on the Mitra run:

```text
round 1: explain: 78 propositions, 58 explained, 0 contradicted, 20 unsupported
round 2: explain: 78 propositions, 70 explained, 0 contradicted,  8 unsupported
round 3: explain: 78 propositions, 74 explained, 0 contradicted,  4 unsupported
```

Round 1 tied every example row to a test. Round 2 added tests over many
points for each claim, and the agent's reviews of what it checked
([../evidence.mitra](../evidence.mitra)). Round 3 reviewed the ensures.
The 4 left are the `from` rules of the four ensures: no review can name
them (finding F3 below).

## What the spec bought

**The one acceptance miss of the control** is `semver bump minor
1.4.2+build`: the control prints `1.5.0`, keeping nothing of the build
tag. The sentence never said; the spec has the row
`("1.4.2+build") -> Err(Failure::BadVersion)`. The control otherwise did
well: semver is a task every model knows.

**A bug in the spec, found by the agent working from it.** Thinking
about the claim `render::round_trip` (`parse(s) == Ok(self)`), the agent
saw that the model's own `Version` allows `pre: Some("")` and negative
numbers, which render to texts `parse` must refuse. So the claim is
false of the model. `mitra check` did not see it: `parse` has no body,
so check can only try the claim at the rows of the examples. The agent
closed the hole in the code (`Version::new` refuses such values) and
wrote it in its log ([mitra/log.md](mitra/log.md), round 1). The model
cannot yet say "a pre-release is never empty" of a record (finding F9).

**Tests tied to the spec.** 37 tests against 9; every example row has its
test, and each test says which row it measures.

## Blame finds a planted bug

In the Mitra run's code, a minor bump forgets to reset the patch
([planted/bug.diff](planted/bug.diff)):

```diff
-            Level::Minor => Version { minor: self.minor.checked_add(1)?, patch: 0, pre: None, ..self.clone() },
+            Level::Minor => Version { minor: self.minor.checked_add(1)?, pre: None, ..self.clone() },
```

Three tests fail (`bump_minor`, `run_minor`, and the end-to-end
`prints_bumped_version`). With the tests alone, blame puts the tie of
`bump` first, far ahead
([planted/blame.txt](planted/blame.txt)):

```text
diagnosis 1 (doubt 0.50): drop Version::bump at src/lib.rs/Version/bump (only the model says it)
diagnosis 2 (doubt 0.17): drop measurements::0 (test:tests::bump_minor, …); Cli::level at src/lib.rs/level (only the model says it)
```

Read: "the code at `src/lib.rs/Version/bump` is probably not the model's
`bump`". Right.

With the agent's reviews added, the tie of `bump` is still first, but
with the agent's review of it, and at a doubt of 0.11:

```text
diagnosis 1 (doubt 0.11): drop reviews::7 (agent:…: Version::bump holds); Version::bump at src/lib.rs/Version/bump (only the model says it)
```

Two things weakened it, both findings:

- The review of `bump` was written before the bug, and still counts
  (F6). A stale review shields the code it once read.
- With plain `cargo test`, the first failing test binary stops the run,
  the end-to-end tests are missing from the report (R005), and the two
  failing tests come first, tied with the culprit at 0.11
  ([planted/blame-fail-fast.txt](planted/blame-fail-fast.txt)). The
  README now says `cargo test --no-fail-fast`.

## Findings: what Mitra could not do

| # | Finding |
|---|---|
| F1 | `Version::parse` written whole from another part is refused (M018, "use Version::parse"), though the reference says a whole name works without `use`. |
| F2 | `#[test] fn a() { … }` on one line is not read: R003 "above no fn". The reader wants the attribute on its own line. |
| F3 | An ensure's `from` rule (`{premises} E {E}`) can be held by nothing but a proof: no review form names it. It stays unsupported forever. |
| F4 | A test can only measure a row, not a claim: tests over many points for a claim give R004 "measures nothing", and only a review carries them. |
| F5 | `explain` prints R004/R005 findings, then a summary of "0 findings". |
| F6 | A review is not tied to the code it read: after the code changes, the old review still counts. |
| F7 | When `run` fails, the ties of `parse`, `render` and `level` are also contradicted ("0 for, 2 against"): every tie `run` calls takes a share. Noisy for the innocent ones. |
| F8 | Counts and numbers disagree on display: "1 for" with b 0.50 for a row measured by two tests (one in `src`, one in `tests`); "1 for, 6 against" with b 0.33, d 0.12. |
| F9 | No way to say what a record's values are (a pre-release is never empty, numbers are 0 or more), so `render::round_trip` is false of the model and `check` cannot see it. |
| F10 | No machine bounds: the model's integers never overflow, Rust's `u64` does; the agent's overflow tests measure nothing. |
| F11 | The getting-started guide uses plain `cargo test`, which stops at the first failing test binary; blame then sees fewer tests. |
