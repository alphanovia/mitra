# The four levels

Part of the [language reference](language.md). Four commands, each one
question, each running the ones before it and stopping at the first
that fails. They work on the store of the current folder.

```sh
mitra build
mitra conform  [--rev <commit>]
mitra assess   [<report>...] [--rev <commit>]
mitra diagnose '<evidence>' [<report>...] [--rev <commit>]
```

| Level | Reads | Fails (exit 1) when |
|---|---|---|
| `build` | the model alone | the model contradicts itself |
| `conform` | and the code | a tie differs, or a piece of a subject has no code |
| `assess` | and the ledger | the report has an error |
| `diagnose` | and an observation | the observation contradicts something |

A model that does not read stops every level with status 2. The other
commands ([commands.md](commands.md)) are the small steps under these:
`check` and `plan` under build, `tie` under conform, `ingest`, `eval`,
`add`, `remove` and `next` under assess.

## build

```
build: ok, model 2e0e8f214c1c: functions 1, claims 2, promises 1, requires 0, blocks 1, warnings 0
  written: .mitra/build/model.lock, interface.mitra, owed.txt, settings.toml
```

It refuses stores that import each other
([stores.md](stores.md#no-round)). It reads the model with no evidence. It prints the errors that need no
evidence (a claim that contradicts the specs of what it calls), the
warnings `M105` and `M106` on `require` and `uses`, and each promise of
a subject that no claim says. It prints nothing on the evidence owed.
With an error nothing is written.

The three files are written again by each build, never edited and never
read back by Mitra:

- `model.lock`: `mitra build 1`, `model <fingerprint of the whole>`,
  then one line per piece in the order of the names,
  `kind name fingerprint <- what it rests on`:

  ```
  function calc::add 938e6af6cc0e
  implied calc::add::correct e7c4c9880fa6
  ensure calc::adds d6139ec73bdf <- calc::add::correct, calc::small, calc::adds::captured
  ```

  Kinds: `function` (code of a subject), `spec` (the same, with a
  body), `helper`, `world` (of a boundary), `record`, `enum`, `claim`,
  `implied` (`f::correct`, `f::spec`, `f::deterministic`), `ensure`,
  `require`, `promise` (an `ensure` of a boundary), `capture`.
  Fingerprints: [ledger.md](ledger.md). Two builds with the same
  `model` line say the same thing.
- `interface.mitra`: what the code must meet, to hand to whoever writes
  it. Each function, record and enum of a subject of this store, under
  the comment that ties code to it.
- `owed.txt`: what `mitra plan` says of the model with no evidence.
- `settings.toml`: the [settings](settings.md) in force, defaults included.

## conform

```
no code is tied to: calc::sub
conform: drift: 1 ties agree, 0 differ, 1 pieces untied; 3 tests tied, 2 pin an input
  6 lines of model over 41 lines of code, in 1 blocks with 1 promises and 0 requires
```

It does what `mitra tie` does, and writes the ties to the ledger in
place of the last ones. Then it counts: the tests tied to a claim and
how many pin an input; the lines of the model files and of `src/` that
are neither blank nor comment (in `src/`, above `#[cfg(test)]`). A
function of a boundary needs no code.

## assess

`mitra assess report.txt --rev R` ingests the reports first, as `mitra
ingest` does. It prints what `mitra eval` prints, then:

```
assess: 0 errors, 5 warnings
  for a person to review or decide (2):
    calc::add::deterministic: a reviewer's word that `calc::add` gives one output per input
    calc::adds::captured: a reviewer's word that the claims say what the text says
  to hand over (tests to write, code to tie) (1):
    calc::commutes: (Int, Int) is infinite: a claim one point can stand for, or a proof
```

The split follows what `mitra plan` says each claim owes: a review or a
reviewer's word is a person's, and so is evidence a person said that
was set aside (`M107`, `M108`); anything else can be handed over.

## diagnose

The observation is evidence, as `mitra add` takes it. It is judged and
not written.

```
assess: 0 errors, 5 warnings
diagnose: the observation contradicts what the model and its ledger say
  calc::small: holds -> fails

error[M001]: evidence refutes a claim
  …
  to file it: mitra add 'evidence bug { for calc::add given a = 1, b = 2 gives 4 }' --source <who>
```

The report with the observation is compared with the report without
it. Each verdict that changes is listed, then each error that is new,
with its candidates as `mitra eval` ranks them. When nothing comes to
fail and no error is new:

```
diagnose: the observation contradicts nothing: it agrees with the model, or it is outside what the model promises
  no verdict changes
```

On the way, `assess` prints its count only, and its errors if it has
any: an error already in the ledger stops the diagnosis.

## Not done

- No later level refuses a build older than the model: each level
  builds again.
- `explore`, a comparison of two builds, coverage per function
  (`scripts/coverage.py` does it for Mitra itself).
- `diagnose` does not say whose each suspect is, what else is now in
  doubt, or which evidence would separate the suspects.
