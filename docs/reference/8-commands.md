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

## Exit codes

| Code | Meaning |
|---|---|
| 0 | done; findings and verdicts are answers, not faults |
| 1 | something is not well formed: the model or the evidence |
| 2 | a wrong command line, or a file that cannot be read |

Messages are written as `rustc` writes them: `error[M005]` or
`finding[A007]`, a line, then where. Each code has a page in
[codes/](codes/README.md).
