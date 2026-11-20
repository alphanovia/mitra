# Layout: one axis for the model and the code

A proposal and its log. The reference is not touched: no behaviour
changes.

## The problem

In mitra-lang the model is cut by concept (types, functions, claims,
intent, contracts, ledger, verdicts…). The old core of the code is cut
by phase: `parse.rs` (6,100 lines), `eval.rs` (4,100), `ast.rs`,
`report.rs`, `set.rs` over 1,200 each. A concept has a model file and
no home in the code. No agent reads those files whole.

## The axis: concepts

Recent increments add concepts (contracts, generics, voices, settings),
not phases, and each newer module already matches its model file one
to one. So a concept is a folder of the code, named as its model file,
holding its tree types, its reading and its checks.

Two things do not cut by concept, and are concepts of their own:

- **Judging.** A verdict combines claims, evidence, uniformity and
  promises in one algorithm. It is the concept `verdicts`, cut into
  its steps.
- **The pipeline.** Read, check, judge, report is a thin driver
  (`lib.rs`), and a subject of the model whose promises require each
  other in that order.

## The map (mitra-lang)

| Folder | Model file | Holds |
|---|---|---|
| `reader/` | reader | tokens, the cursor, the items of a file, the driver of a read |
| `types/` | types, operators | sets of values, tuples, the syntax of a type |
| `functions/` | functions | expressions, functions, their reading and computing |
| `claims/` | claims | claims and their shapes |
| `intent/` | intent | ensures, requires, what a promise rests on |
| `contracts/` | contracts | contracts, impls, type parameters |
| `modules/` | modules | imports, other stores |
| `ledger/` | ledger, results, voices | evidence, acts, voices, test results |
| `model/` | model | the model as a whole and its checks |
| `verdicts/` | verdicts, culprits | judging, step by step; suspects |
| `report/` | report, print, advice | what is printed |
| `build/` | build, fingerprint | the build file, fingerprints, levels |
| `settings.rs`, `frontend.rs` | settings, frontend | as now |

The other crates keep their files; those over the ceiling are cut the
same way.

## The ceiling

1,000 lines for every file of code, tests and model. At 40 characters
a line that is about 11,000 tokens: one read, whole. A test enforces
it (`crates/mitra-cli/tests/layout.rs`), with a list of the files not
yet cut that only shrinks.

## Steps, each leaving main green

1. The ceiling test, with today's exceptions.
2. `parse.rs` into the concept folders.
3. `eval.rs`: the long `evaluate` into the steps of `verdicts/`.
4. `ast.rs`, `report.rs`, `set.rs`, and the long files of the other
   crates and of the tests.
5. The model follows: the pipeline as a subject, files renamed where
   the code showed a better cut.
6. The breaks benchmark again, and the judgement.

## Log: where one corrected the other

1. Code corrected the model (step 2). Cutting the reader showed that
   reading an evidence block shares nothing with reading a claim but
   the cursor: it went to `ledger/read.rs`. The model already had
   `ledger::entries` apart from `reader::model`; its comment pointed
   at the wrong file and now points at the right one.
2. Model corrected the code (step 2). The model has one `reader`
   subject whose promise is "a text gives a model or one placed
   error". The code keeps one reader type (`reader.rs`) and each
   concept adds its methods to it, so the promise still has one
   owner; I did not make a reader per concept.
3. A tool corrected the layout (step 2). `scripts/coverage.py` stops
   at the first `#[cfg(test)]` of a file, so a `mod tests;` at the top
   hid the whole reader. Test modules are declared last.
