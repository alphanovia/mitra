# tickets: a benchmark for Mitra

`tickets` is a small Rust command line tool, a to-do list kept in a
log file, and the second benchmark after [semver](../semver/README.md).
It is larger on purpose: a disk that can fail or be cut off by a crash,
state rebuilt by replaying a log, and claims that hold across commands.

```text
tickets add "fix login"     prints #3
tickets done 3
tickets list                prints #3 [x] fix login, one line per ticket
tickets show 3
```

The spec is [tickets.mitra](tickets.mitra). It leaves the file format
to the code, and says what must hold whatever the format: ids are
unique and increasing, a done ticket stays done, a change the tool
acknowledged survives a crash, and a line cut off by a crash is
ignored, never misread.

## What is here

```text
examples/tickets/
  tickets.mitra         the spec, written before any code
  src/, tests/          the tool, written by a cold agent from the spec
  evidence.mitra        that agent's reviews, each saying what it checked
  Cargo.toml
  acceptance.sh         the black-box check every run is graded with
  README.md             this file
  benchmark/
    RESULTS.md          the first run of the benchmark
    control/            run 1: the task in words, no spec, no Mitra
    mitra/              run 2: log.md, every round and what Mitra printed
    planted/            run 3: one bug planted in run 2's code, and blame
```

## Run it yourself

You need Rust, [Claude Code](https://claude.com/claude-code), and a
clone of this repository with Mitra built:

```text
cargo build --release
export PATH="$PWD/target/release:$PATH"
```

Make two empty folders outside the clone, start Claude Code in each,
and paste one prompt in each, each in a fresh session.

### Run 1: cold, without a spec

```text
In this empty folder, write a Rust command line tool called `tickets`,
with tests: a to-do list kept in the file tickets.log in the current
folder.

  tickets add <title>   adds a ticket and prints its id: #3
  tickets done <id>     marks it done, prints nothing
  tickets list          prints every ticket, in id order, one per line:
                        #1 [ ] fix login   or   #1 [x] fix login
  tickets show <id>     prints that one ticket's line

Storage is an append-only log: each change appends one line, and the
state is rebuilt by replaying the log. Ids are unique and increasing. A
done ticket stays done. The disk may fail a write, and a crash may cut
the last line short: a change the tool acknowledged must survive a
crash, and a cut line is ignored, never misread. Bad input gives an
error, as an enum: the tool prints nothing on standard output and exits
with a code other than 0.

Work until `cargo test --no-fail-fast` passes. Do not read anything
outside this folder.

Keep a file log.md: for each round, the command you ran, its result in
short, and what you changed. At the end, add the number of rounds, and
the number of lines of code and of tests.
```

### Run 2: cold, with the spec and Mitra

Copy `tickets.mitra` into the empty folder first. Replace `MITRA` below
with the path of your clone.

```text
In this folder, tickets.mitra is the spec of a Rust command line tool
called `tickets`, written in Mitra. Write the tool in Rust, with tests.

The log is the file tickets.log in the current folder. What the tool
prints, which the spec leaves out:
  add          the new id: #3
  done         nothing
  list, show   one line per ticket, in id order: #1 [ ] fix login
               or #1 [x] fix login
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
  mitra check tickets.mitra
  mitra explain tickets.mitra --code src
  mitra blame tickets.mitra --code src
(explain and blame run the tests themselves; add --code tests if you
keep tests in tests/).
If something stays unsupported because only a review could hold it,
you may write a review in evidence.mitra, by "agent:<your model>", only
of what you checked yourself, and pass it with --store evidence.mitra.
Never change tickets.mitra: if you think the spec is wrong, say so in
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

It builds the tool and uses it as a person would, each case in a fresh
folder: the commands, the failures, odd titles, ids across restarts, a
crash that cuts the last line at every byte, and a disk that refuses a
write.

### Plant a bug, and blame

In run 2's folder, acknowledge a change before the disk has it: in the
code of `Cli::answer`, give the reply whatever the write gave. Then:

```text
mitra blame tickets.mitra --code src --code tests --store evidence.mitra
```

The first diagnosis should name the tie of `Cli::answer`.

## Metrics

- **Model over code**: non-blank, non-comment lines of `tickets.mitra`
  over those of `src/` without tests.
- **Rounds**: how many times the agent ran the tests before it stopped.
- **Verdicts over time**: the explain summary line of each round.
- **Acceptance**: rows of `acceptance.sh` passed.
- **Blame**: is the planted bug's tie the first diagnosis?
