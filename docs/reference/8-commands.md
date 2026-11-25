# 8. The commands

```text
mitra check MODEL [--points N]
mitra explain MODEL [--code DIR]... [--report FILE | --no-test] [--store FILE]
mitra blame MODEL [--code DIR]... [--report FILE | --no-test] [--store FILE]
```

- `MODEL`: a `.mitra` file of the project. Paths in ties
  (`src/lib.rs/div`) are read from the project's root.
- `--code DIR`: Rust code to read for ties and measuring tests. Give it
  more than once for several folders.
- `--report FILE`: what the tests printed, read instead of running them
  (`cargo test --no-fail-fast -- --show-output > report.txt 2>&1`), for CI.
  Without it, `explain` and `blame` run `[test] command` at the root
  when it has a `Cargo.toml`, and say how many tests ran and passed.
- `--no-test`: run no tests and read no report.
- `--store FILE`: measurements and reviews written by hand.
- `--points N`: the most points `check` tries per comparison (default 64).

## check

Reads the model alone. First whether it is **well formed** (it parses,
names resolve, types fit): the first problem is an error, and nothing
else runs. Then **agree** compares what the model says twice (bodies,
claims, examples, contracts) and gives findings.

```text
check: well formed; agree: 9 comparisons, 9 hold, 0 findings
```

## explain

Reads the evidence and gives each statement of the model a verdict,
with its numbers ([chapter 1](1-ideas.md#verdicts)), then a summary:

```text
explain: 20 propositions, 10 explained, 3 assumed (2 waiting on a review), 6 contradicted, 1 unsupported; 0 findings, 0 hints, 1 reviews asked
```

"Waiting on a review" counts the assumed statements that rest on a
link no one reviewed: reviewing the links asked for turns them into
explained. The others assume only requires said in words.

Before the summary, the reviews the model asks for, the links that the
most assumed statements rest on first ([chapter 5](5-ties-and-evidence.md#what-explain-asks-you-to-review)).

## blame

When evidence and model conflict, lists **diagnoses**: sets of
statements that, if dropped, leave no conflict. The most doubtful
first.

```text
diagnosis 1 (doubt 0.33): drop measurements::3 (human:ana, saw calc::div::q = 4)
```

## mitra.toml

The project's root is the folder of `mitra.toml`, up from the model's
folder (else the git work tree, else the model's folder). The file holds each layer's settings, a section each; an empty
file only marks the root. Every key is optional, here at its default:

```toml
[parse]
depth = 2000           # how deep a text may nest; twice budget's depth when not said
[agree]
points = 64            # the most points tried per comparison (`--points` overrides it)
splits = 4             # the most numbers bounds cut at zero
length = 3             # the longest sequence tried as a point
height = 3             # the tallest tree tried as a point
[budget]               # how far one computation goes, in agree and explain
steps = 1000000
depth = 1000
[explain]
rounds = 64            # rounds of judging
cases = 1024           # the most points an `all` or `any` is decided on
rests = 1024           # the most sets of evidence a rule combines, duplicates removed
[numbers]
base_rate = 0.5        # what a statement is expected to be with no evidence
prior_weight = 2.0     # how many runs uncertainty weighs as
checker = 1000.0       # how many runs one check by Mitra is
link = 0.5             # how many runs a link no one reviewed is (`assumed:link`)
[numbers.voices]       # how many runs one count of a voice is (1.0 when not said)
"human:ana" = 2.0
[blame]
diagnoses = 10         # the most diagnoses given
tries = 100000         # the most sets of suspects tried
[test]                 # what explain and blame run when no --report is given
command = ["cargo", "test", "--no-fail-fast", "--", "--show-output"]
```

A count is a whole number, a weight is written with its point (`2.0`).
A section, a key or a value Mitra does not take is
[P015](codes/P015.md), and nothing runs.

## Exit codes

| Code | Meaning |
|---|---|
| 0 | done; findings and verdicts are answers, not faults |
| 1 | something is not well formed: the model, the evidence or `mitra.toml` |
| 2 | a wrong command line, or a file that cannot be read |

Messages are written as `rustc` writes them: `error[M005]` or
`finding[A007]`, a line, then where. Each code has a page in
[codes/](codes/README.md).
