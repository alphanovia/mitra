# tickets benchmark: first run (2026-11-24)

One run of each, by cold agents (Claude Code subagents, same model),
each given only its prompt from [../README.md](../README.md) and the
files that prompt names. One run is an anecdote, not a statistic.

## Numbers

| | Control: the task in words | Mitra: the spec |
|---|---|---|
| Rounds of tests | 2 | 3 |
| Code lines (`src/`, no tests) | 294 | 424 |
| Tests | 14 (262 lines) | 51 (490 lines) |
| Model lines (non-blank, non-comment) | | 100 (0.24 of the code) |
| Acceptance, 30 rows | 29: **loses acknowledged adds when two run at once**, 10 times in 10 | **30** |
| Planted bug: first diagnosis | | the broken function's tie |

Explain, round by round, on the Mitra run:

```text
round 1: explain: 102 propositions,  64 explained, 0 contradicted, 38 unsupported; 8 findings, 1 hints, 11 reviews asked
round 2: explain: 102 propositions,  64 explained, 0 contradicted, 38 unsupported; 6 findings, 0 hints, 11 reviews asked
round 3: explain: 102 propositions, 102 explained, 0 contradicted,  0 unsupported; 6 findings, 0 hints, 0 reviews asked
```

The 64 of round 1 are the rows: every example of the model has a test
that measures it. The 38 left are the claims over every input, the
ties, the links behind the ensures and the disk's ensures in words: no
test can hold those, only a review. Round 3 is the agent's own reviews
([../evidence.mitra](../evidence.mitra)), each saying in
[mitra/log.md](mitra/log.md) what it rests on. So "102 explained" is
honest about its strength: most of those 38 stand at 0.67, one voice.

## What the spec bought

**A spec bug, found because Mitra asked.** In round 1, explain asked
for a review of the link behind `keeps` (X012):

```text
ask[X012]: only the model says that (Disk::durable, Cli::acks, Board::skips) together give `Tickets::keeps`: review the link
```

Reviewing it, the agent found that the three do not give it: two runs
at once both read the board, both plan `Added(next, …)` with the same
id, both are acknowledged, and replay drops the second. An acknowledged
change lost, with no crash. The model needs "one command at a time".
The agent locked the log from read to append, tested it
(`concurrent_adds_get_unique_ids`), and wrote the gap in its log.

**The control has exactly that bug.** Its prompt says the same words
("a change the tool acknowledged must survive a crash", "ids are
unique"), but it never asked itself why they hold. Twenty adds at once:

```text
FAIL  [parallel adds] 8 different ids printed, 8 tickets listed, of 20
```

All 20 printed an id; 8 tickets were kept. Twelve were told `#n` for a
ticket that is not there (7 to 12 lost in three more tries). It failed
10 runs in 10; the Mitra run passed 10 in 10.

**Tests tied to the spec.** 51 tests against 14, 39 of them measuring a
row of the model. The control's tests were good where it thought to
look: cuts at every byte, a write failing after every byte.

## Blame finds a planted bug

In the Mitra run's code, `answer` acknowledges a change whatever the
disk said ([planted/bug.diff](planted/bug.diff)):

```diff
-            Step::Write(_, r) => {
-                if written {
-                    Outcome::Printed(r.clone())
-                } else {
-                    Outcome::Failed(Failure::Storage)
-                }
-            }
+            Step::Write(_, r) => Outcome::Printed(r.clone()),
```

The black-box acceptance does **not** see it: 30 rows in 30 still pass,
since a disk that refuses the write also refuses the read before it.
Two tied unit tests do (`answer_an_unwritten_change`,
`answer_an_unwritten_done`), because the model split `answer` out with
`written` as an input. With the tests alone, blame
([planted/blame-tests.txt](planted/blame-tests.txt)):

```text
diagnosis 1 (doubt 0.50): drop Cli::answer at src/lib.rs/Cli/answer (only the model says it)
diagnosis 2 (doubt 0.11): drop measurements::35 (test:tests::answer_an_unwritten_change, …); measurements::36 (…)
```

With the agent's reviews, written before the bug, the tie of `answer`
and the two tests are level at 0.11
([planted/blame-store.txt](planted/blame-store.txt)): the stale review
of `Cli::answer` shields it, as in semver (finding F6 there).

## What changed in the grading after the runs

- The row **parallel adds** was added after reading the Mitra run's
  log. It tests what both prompts state (unique ids, acknowledged
  changes kept). Both runs were graded with it.
- The **cut last line** row first refused a line cut only before its
  final line break. That line is whole: the model reads it as its event
  (`reads` makes only a strict cut `Cut`). The row now accepts either.
  The Mitra run kept such a line; the control dropped it.

## The spec, after

The runs got [spec.mitra](spec.mitra), kept as it was so a rerun is the
same benchmark. [../tickets.mitra](../tickets.mitra) now says what the
run found missing:

```mitra
    require alone: "one command runs at a time, from reading the log to its answer";
    ensure keeps: "a change the tool acknowledged survives a crash" from (Disk::durable, Cli::acks, Board::skips, alone);
```

With it, explain shows `Tickets::alone` unsupported: a hypothesis in
words, until someone reviews the code's lock.

## Findings: what Mitra could not do

| # | Finding |
|---|---|
| T1 | A method on a value carried by `fold` is refused (P013, "type cannot be told"): `b.apply(e)` had to be written `apply(b, e)`. |
| T2 | X004 on every row of `Codec::reads`: it calls the bodiless `encode`, and X004 says to measure it as an input, but no form of `measures` or of the store writes such an input. |
| T3 | Measuring a bodiless function at a point outside its examples gives X001: the agent dropped two `measures` lines on `Cli::read`. |
| T4 | Tests with no row to measure (end to end, every cut) count only through a review's `seeing`; 12 tests listed as R004. |
| T5 | More counts for can score lower: `Tickets::keeps` 0.54 with "6 for" against 0.67 with "1 for". |
| T6 | Nothing carries tested points to a claim over every input: 38 propositions can only be held by reviews. |
| T7 | The model has no word for "one command at a time": `keeps` is missing a premise, and nothing could say so but a review. |
| T8 | As semver's F6: a review written before a change still counts after it. |
