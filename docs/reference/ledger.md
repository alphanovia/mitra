# Keeping the ledger

Part of the [language reference](language.md). The ledger is the
evidence file of a store, `model/evidence.mitra`. It is not
append-only: a model changes, and evidence that no longer means
anything must not pile up. Two rules hold: nothing leaves without being
shown, and what a person said never leaves by itself.

## Machine or person

| Source starts with | Said by | Kept how |
|---|---|---|
| `test:` | `mitra ingest` | a snapshot: the next `ingest` replaces it |
| `reader:` | `mitra tie` | a snapshot: the next `tie` replaces it |
| anything else, or none | a person, through `mitra add` | until `mitra remove` |

## A run replaces the last one

`mitra ingest` takes out every `test:` entry and writes the results of
the reports it is given; `mitra tie` does the same with `reader:`. Give
`ingest` every report of the revision in one run.

```
1 results: 1 passed, 0 failed; 1 evidence written, in place of 2 test results of rev r1
no longer observed, removed (1):
  evidence test { for flip given a = false gives true source "test:t_false" rev "r1" }
```

The first line counts what was replaced and names the revisions it was
from. The list is every entry that left with no entry of the same test
(or, for `tie`, of the reader) on the same piece written in its place: a
test that was deleted, renamed, or lost its tie comment. If the ledger with the new entries
does not read or evaluate, nothing is written.

## What a person said

`mitra add` writes into each block the fingerprint of the piece it
speaks of:

```
evidence r { for calc::commutes holds source "review:ana" seen "c998bf7e792a" }
```

A fingerprint is twelve hex digits worked out from what the piece says.
Comments, spacing and the order of the declarations do not change it.

| Piece | Changes when |
|---|---|
| function | a parameter, its type, the result type or the body changes |
| claim | what it states, the function it is on, or what it follows from changes |
| record, enum | a field, a variant or a type changes |
| `ensure` | its prose changes, or the promise a `require` names |
| capture (`name::captured`) | the prose changes, or a claim the promise rests on is added, removed or changes |

`f::correct`, `f::tied`, `f::uses` and the like have the fingerprint of
`f`. An entry with no `seen` is never stale.

Two things can happen to the piece:

- **It is gone or renamed.** The entry is *orphaned*: warning `M107`.
- **It says something else.** The entry is *stale*: warning `M108`, with
  both fingerprints.

Either way the entry stays in the file, is shown in every report, and
judges nothing: a review of a claim does not carry over to the claim
reworded, and a bug reported on it stays in sight until someone says
whether it still stands. Then:

```sh
mitra add 'evidence r { for calc::commutes holds }' --source review:ana   # it still stands
mitra remove --stale calc::commutes                                       # the old entry goes
mitra remove --orphaned
mitra remove --source ticket:7
mitra remove calc::commutes        # every entry on that piece
```

`remove` prints each entry it takes out, as it was written. A machine's
entry on a piece that is gone is orphaned too, and leaves at the next
run of that machine.

New evidence on a piece the model does not have is refused by `add`:
only what is already in a ledger is set aside.

## Not done

- The fingerprint of a claim does not follow the helpers it calls: a
  helper reworded leaves the claim's fingerprint as it was.
- The printed form behind a fingerprint may change with a version of
  Mitra, which would make every entry stale once.
- `mitra remove` asks nothing: git is the way back.
- Two evidence blocks on one line cannot be removed apart: `remove` and
  the runs refuse, naming the line.
