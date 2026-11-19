# Design: the product, in four levels

Status: **planned**. Nothing here renames or adds a command on main.
What exists is in the [commands reference](../reference/commands.md);
the table below says which of today's commands each level is made of.
Settled with the user in chat on 2026-11-19, unless marked open.

## What Mitra is for

- Mitra is a language that compiles to a diagnostic.
- It reduces the engineer's work to modelling and what matters.
- It says what a human must review and what can be delegated to an
  agent, so that nobody has to review all the code.
- "Mitra is a boundary between two actors, agent or human" (the user).
  The model and the reports are what crosses it.

The command line is a product: named vertical slices and work loops.
Many small commands, each useful alone and fast (less compute, a
quicker loop on one problem), combined into a few macro commands.

## The four levels

Each level is one question, and one macro command. A macro command runs
the levels before it and stops at the first that fails.

| Level | The question (the user's words) | It says | Small commands |
|---|---|---|---|
| `mitra build` | "Given this model alone, what can you say about it?" | It parses and type checks; no claim contradicts another; each `ensure` has claims and each `require` a provider. Then, asked apart: what evidence is owed, what exploring finds. | check, explore, plan |
| `mitra conform` | "Given this model and the code, what can you say about it?" | Ties agree; calls match `uses`; which tests are tied and what they pin; coverage and relative size ("N lines of model over M lines of code, in S subjects with E ensures and R requires"). | tie, coverage |
| `mitra assess` | "Given the model, the code and the evidence ledger containing the manual reviews and validations, what can you say about it?" | A verdict per claim and per ensure; suspects where evidence contradicts; what only a human can settle and what can be delegated. | ingest, report, review, vouch |
| `mitra diagnose` | Given this bug or observation of something wrong, the model, the code and the ledger. | See below. | file, suspects, next |

**build.** The first thing is a model that is syntactically correct,
read by Mitra, with no error that can be found without evidence. "At
this point you don't want a report of what is owed, only that the model
can be evaluated" (the user). What is owed is asked next, and handed to
an agent with the model.

**conform.** The model-to-code drift check: what comes back after the
model was handed to an agent. The work owed is given, the model is
fully tied, with the metrics.

**assess.** After the agent's work some evidence is still missing: "you
get from Mitra what code to review and vouch for" (the user).

**diagnose.** Whether the observation contradicts anything (if not, it
is outside what was promised); the ranked pieces that cannot all be
right; whose each is; what was trusted and should not have been; what
else is now in doubt; the evidence that would separate the suspects and
who can produce it; what closes it (the observation becomes a pinned
point). It cannot speak about code the model does not cover.

### Today's commands

| On main today | Level | Becomes part of |
|---|---|---|
| `mitra check` | build | check |
| `mitra plan` | build | plan |
| `mitra tie` | conform | tie |
| `scripts/coverage.py` (not a command) | conform | coverage |
| `mitra ingest` | assess | ingest |
| `mitra add` | assess, diagnose | review, vouch, file |
| `mitra eval` | assess | report; its errors and candidates are also diagnose's suspects |
| `mitra next` | assess, diagnose | next |
| `mitra explain` | every level | unchanged |
| `mitra init` | before build | unchanged |

Not on main: `build` as a command and its artifact, `conform`, `assess`
and `diagnose` as macro commands, `explore`, `coverage` as a command,
`file`, `suspects`, `review`, `vouch`. The final verbs of the small
commands are open.

## The build artifact (agreed: "yes to build")

`mitra build` writes a compiled model under `.mitra/build/`:

- every piece with its full name and kind, the implied ones included
  (`f::correct`, `f::deterministic`, `f::uses`, captures);
- resolved types and library claims expanded;
- what follows from what;
- the interface the code must meet: the file handed to an agent;
- the evidence owed;
- a fingerprint per piece.

What it is for:

- later steps do not read the model again;
- evidence records the fingerprint of its piece, so stale evidence is
  detected without git;
- comparing two builds shows a change of intent.

Open, proposed by the coordinator and not decided: commit a small
lockfile-like part of the build; never edit it by hand; later commands
refuse a build older than the model.

## The loops

Named in chat; the user agreed "mostly". One line each is this
document's reading, to be corrected.

| Loop | Lives in | In one line |
|---|---|---|
| scaffold | build | Write the model of a project before its code, until it builds. |
| delegate | build, conform | Hand the model and what is owed to an agent; get the conformance report back. |
| review | assess | Read what Mitra says only a human can settle, and vouch or refuse. |
| bug | diagnose | File an observation; follow the suspects to the piece that is wrong. |
| model correction | diagnose, build | The wrong piece is the model: change it, rebuild, see what the change puts in doubt. |
| buy | build, assess | Model something bought as a boundary: what is required of it, and whether it behaves so. |
| grow | build, conform | Add a subject or a promise to a model in use. |
| change (not built) | build | Compare two builds: what the change of intent touches. |
| explore (not built) | build | Run what can be run of the model alone. |

The user's order for a new project: write the model and get it to
build; ask what is owed and hand it to an agent; get the conformance
report; then review what is left.

## Explore

The user wants it early. What it is expected to find:

- contradictions inside the model;
- paths not accounted for, or evidence that all are;
- with stochastic boundaries injected: where the model breaks, peaks of
  behaviour;
- redundancy: evidence that is not needed once another piece is there;
- weak claims, by mutation;
- proposals of invariants.

The user also floats "a mutating and inferring engine to try to rework
claims and things to fix problem or reduce the amount of evidence
needed".

Caveat (the coordinator's): a model written by intent has few bodies,
so exploring it is deduction more than simulation.

Layering (the coordinator's proposal, not decided): the explorer is a
layer apart from the judge. It runs only the model, and its output is
evidence like any other.

## Crates

On main:

| Crate | Holds | Touches |
|---|---|---|
| `mitra-lang` | the language and its evaluation, the ranking of culprits, the rules that turn a test result into evidence, the JUnit reader | nothing: it is handed named texts and reads no file |
| `mitra-store` | everything Mitra keeps on disk: the model files, the files they import, the ledger | the file system |
| `mitra-rust` | the reader of Rust code and of `cargo test` reports | the Rust files |
| `mitra-cli` | the commands | arguments, the terminal |

`mitra-lang` is pure so that it can be proven: "better for proving it if
there is no IO" (the user). A judge crate is separated only "when we
prove it" (the user). What the splits showed is in the review list
(increments 66 and 68).

The trust calculus on one question (`Mass`, Dempster's and Yager's
rules) was never called by the evaluation. The user, 2026-11-19: "let's
remove the dead code, and I'll work with you on the trust model, and
calculus will come back when we need it from the model." It is removed;
`culprits::rank` is what runs.

## Where the build file lives (proposed, not confirmed)

The coordinator's answer to the user's question, not built:

- `mitra-lang`: the content of a compiled model (pieces, types, links,
  fingerprints) and its text form. Pure.
- `mitra-store`: writing and reading `.mitra/build/`, and noticing that
  it is older than the model.
- `mitra-cli`: the `mitra build` command and what it prints.

The fingerprint is computed in `mitra-lang` from what a piece means, not
from the bytes of its file: reformatting or a comment does not make
evidence look stale.

## Keeping the ledger (agreed in outline, details to settle)

The user does not want an append-only ledger: "if the model changes a
lot, some evidence doesn't mean anything anymore. You don't want to keep
a big ledger of noise". The outline, approved 2026-11-19 ("lgtm, we will
work out the details later"):

- Nothing leaves the ledger without being shown.
- Evidence observed by a machine (ties, test results, samples) is a
  snapshot of one revision: each run writes it again, and the report
  says what was replaced.
- Evidence said by a person (a vouch, a review, a bug report) is never
  dropped silently. When its piece is gone or was reworded it is listed
  as orphaned or stale, counts for nothing, and waits for a decision
  instead of expiring.
- Removing evidence is an act of its own, which lists what goes.
- Possibly two files in the store: one written again, one kept.

Not built: it needs the fingerprints of the build. On main the ledger
only grows, and evidence naming a piece the model no longer has is an
error that stops every command. The four promises are written in
mitra-store's own model (`crates/mitra-store/.mitra/ledger.mitra`,
subject `keeping`) with nothing to support them, which is their honest
state.

## Open decisions

1. How a passing sample counts. Today it changes no verdict
   ([sampled](../reference/kw-evidence.md#sampled), decision 339). The
   coordinator proposes `holds assuming` a named sample assumption.
2. Implied `deterministic`: one summary line instead of one claim and
   one warning per function.
3. The final verbs of the small commands.
4. The committed part of the build, and refusing an old build.
