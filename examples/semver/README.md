# semver: a benchmark for Mitra

`semver` is a small Rust command line tool, and a benchmark: does a
coding agent that gets a Mitra model write a better tool than one that
gets the same task in a sentence?

```text
semver bump minor 1.4.2      prints 1.5.0
```

A bump adds one to its level and sets the lower levels to 0. A
pre-release tag (`-rc.1`) is dropped. Bad input is an error, never a
guess. The whole spec is [semver.mitra](semver.mitra).

## What is here

```text
examples/semver/
  semver.mitra          the spec, written before any code
  src/, tests/          the tool, written by a cold agent from the spec
  evidence.mitra        that agent's reviews, each saying what it checked
  Cargo.toml
  acceptance.sh         the black-box check every run is graded with
  README.md             this file: how to run the benchmark yourself
  benchmark/
    RESULTS.md          the first run of the benchmark, with examples
    control/            run 1: the task in a sentence, no spec, no Mitra
    mitra/              run 2: log.md, every round and what Mitra printed
    planted/            run 3: one bug planted in run 2's code, and blame
```

The tool at the top is run 2's code. Try it:

```text
cargo run -q -- bump minor 1.4.2
cargo test --no-fail-fast > report.txt 2>&1
mitra explain semver.mitra --code src --code tests --report report.txt --store evidence.mitra
```

## Run it yourself

You need Rust, [Claude Code](https://claude.com/claude-code), and a
clone of this repository with Mitra built:

```text
cargo build --release
export PATH="$PWD/target/release:$PATH"
```

Make two empty folders outside the clone, start Claude Code in each,
and paste one prompt in each. Start each in a fresh session, so neither
run sees the other.

### Run 1: cold, without a spec

```text
In this empty folder, write a Rust command line tool called `semver`,
with tests.

`semver bump <major|minor|patch> <version>` prints the bumped version:
`semver bump minor 1.4.2` prints `1.5.0`. A bump adds one to its level
and sets the lower levels to 0. A pre-release tag is dropped. Bad input
gives an error, as an enum, never a guess: the tool prints nothing on
standard output and exits with a code other than 0.

Work until `cargo test` passes. Do not read anything outside this
folder.

Keep a file log.md: for each round, the command you ran, its result in
short, and what you changed. At the end, add to log.md the number of
rounds, and the number of lines of code and of tests.
```

### Run 2: cold, with the spec and Mitra

Copy `semver.mitra` into the empty folder first. Replace `MITRA` below
with the path of your clone.

```text
In this folder, semver.mitra is the spec of a Rust command line tool
called `semver`, written in Mitra. Write the tool in Rust, with tests.

`semver bump <major|minor|patch> <version>` prints the bumped version.
On a failure, the tool prints nothing on standard output and exits with
a code other than 0.

Mitra is on your PATH as `mitra`. To learn it, read MITRA/README.md,
MITRA/docs/getting-started.md and MITRA/docs/reference/. Do not read
anything else outside this folder.

Tie the code to the model: `// mitra: implements Part::function` above
each function that implements one, and `// mitra: measures Part::function
(inputs) -> result` above each test, with the row written as the model
writes its examples.

Loop until `mitra explain` shows every proposition explained:
  mitra check semver.mitra
  cargo test --no-fail-fast > report.txt 2>&1
  mitra explain semver.mitra --code src --report report.txt
  mitra blame semver.mitra --code src --report report.txt
(add --code tests if you keep tests in tests/).
If something stays unsupported because only a review could hold it,
you may write a review in evidence.mitra, by "agent:<your model>", only
of what you checked yourself, and pass it with --store evidence.mitra.
Never change semver.mitra: if you think the spec is wrong, say so in
log.md and keep going.

Keep a file log.md: for each round, the commands you ran, everything
Mitra printed (verbatim), and what you changed. At the end, add the
number of rounds, the explain summary line of each round, and the
number of lines of code and of tests.
```

### Grade both

```text
./acceptance.sh path/to/run/folder
```

It builds the tool and runs it on every row of the spec's examples and
a few more (29 rows), as a user would: the arguments in, what is printed and the
exit code out.

### Plant a bug, and blame

In run 2's folder, change one thing in the code that the tests catch,
for example forget to reset the patch on a minor bump. Then:

```text
cargo test --no-fail-fast > report.txt 2>&1
mitra blame semver.mitra --code src --code tests --report report.txt --store evidence.mitra
```

The first diagnosis should name the tie of the function you broke. Use
`--no-fail-fast`: plain `cargo test` stops at the first test binary that
fails, the later tests are missing from the report, and blame sees less.

## Metrics

- **Model over code**: non-blank, non-comment lines of `semver.mitra`
  over those of `src/`.
- **Rounds**: how many times the agent ran the tests before it stopped.
- **Verdicts over time**: the explain summary line of each round.
- **Acceptance**: rows of `acceptance.sh` passed.
- **Blame**: is the planted bug's tie the first diagnosis?
