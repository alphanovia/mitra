# 8. The commands

```text
mitra check MODEL [--points N]
mitra explain MODEL [--code DIR]... [--report FILE] [--store FILE]
mitra blame MODEL [--code DIR]... [--report FILE] [--store FILE]
```

- `MODEL`: the `.mitra` file at the root of the project. Paths in ties
  (`src/lib.rs/div`) are read from its folder.
- `--code DIR`: Rust code to read for ties and measuring tests. Give it
  more than once for several folders.
- `--report FILE`: what `cargo test` printed (`cargo test > report.txt 2>&1`).
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
explain: 20 propositions, 10 explained, 6 contradicted, 4 unsupported; 0 findings, 0 hints
```

## blame

When evidence and model conflict, lists **diagnoses**: sets of
statements that, if dropped, leave no conflict. The most doubtful
first.

```text
diagnosis 1 (doubt 0.33): drop measurements::3 (human:ana, saw calc::div::q = 4)
```

## mitra.toml

The project's root is the folder of `mitra.toml`, up from where `mitra`
runs. The file holds each layer's settings, a section each; an empty
file only marks the root. Every key is optional, here at its default:

```toml
[parse]
depth = 2000           # how deep a text may nest; twice budget's depth when not said
[agree]
points = 64            # the most points tried per comparison (`--points` overrides it)
splits = 4             # the most numbers bounds cut at zero
length = 3             # the longest sequence tried as a point
[budget]               # how far one computation goes, in agree and explain
steps = 1000000
depth = 1000
[explain]
rounds = 64            # rounds of judging
cases = 1024           # the most points an `all` or `any` is decided on
rests = 1024           # the most sets of evidence a rule combines
[numbers]
base_rate = 0.5        # what a statement is expected to be with no evidence
prior_weight = 2.0     # how many runs uncertainty weighs as
checker = 1000.0       # how many runs one check by Mitra is
[numbers.voices]       # how many runs one count of a voice is (1.0 when not said)
"human:ana" = 2.0
[blame]
diagnoses = 10         # the most diagnoses given
tries = 100000         # the most sets of suspects tried
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
