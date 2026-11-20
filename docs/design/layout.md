# Layout: one axis for the model and the code

A proposal, done on 2026-11-20, and its log. The reference is not
touched: no behaviour changes. The judgement is in
`docs/review/2026-11-18.md`, increment 81.

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
  (`pipeline.rs`). In the model it is not a subject: each phase is a
  subject that requires the one before (`types` requires
  `reader::reads`, `verdicts` requires `types::fits`, `report` requires
  `verdicts::one_each`), and `model.mitra` lists them in that order.

## The map (mitra-lang)

| Folder | Model file | Holds |
|---|---|---|
| `reader/` | reader | tokens, the cursor, the items of a file, the driver of a read |
| `types/` | types | sets of values, tuples, the syntax of a type, the checks of a model |
| `eval.rs`, `eval/` | eval | the value of an expression |
| `functions/` | functions | expressions, functions, their reading and computing |
| `claims/` | claims | claims and their shapes |
| `intent/` | intent | ensures, requires, what a promise rests on |
| `contracts/` | contracts | contracts, impls, type parameters |
| `modules/` | modules | imports, other stores |
| `ledger/` | ledger, results, voices | evidence, acts, voices, test results |
| `model.rs` | model | the model as a whole |
| `verdicts/` | verdicts, culprits | judging, step by step; ranking |
| `report/` | report, print, advice | what is printed |
| `build/` | build, fingerprint | the build file, fingerprints, levels |
| `settings.rs`, `frontend.rs` | settings, frontend | as now |

The other crates keep their files; those over the ceiling are cut the
same way.

## The ceiling

1,000 lines for every file of code, tests and model. At 40 characters
a line that is about 11,000 tokens: one read, whole. A test enforces
it (`crates/mitra-cli/tests/layout.rs`). The ledger (`evidence.mitra`)
is outside it.

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
4. Model corrected the code (step 3). The model had two subjects where
   the code had one file: `eval` (the value of an expression) and
   `verdicts`, with `require computed = eval::computes` between them.
   The code now has the same two: `eval.rs` knows nothing of verdicts
   but one word (`Implication`), and `verdicts/` calls it.
5. Code corrected the model (step 3). `evaluate` was one function of
   1,500 lines whose steps shared some twenty closures. Cut into
   steps, what they share had to be written down: `Judge` in
   `verdicts.rs`, thirteen fields, each filled by one step and read by
   the next. The model says nothing of these steps: see 8.
6. Model corrected the code (step 4). `Model::check` is
   `types::Model::check` in the model: a promise of `types`, "what a
   model must satisfy before anything is judged". The code had it in
   the file of the tree. It is now `types/check.rs`.
7. Code corrected the model (step 5). Moving `load`, `load_modules`
   and `run` out of `lib.rs` into `pipeline.rs` showed the order the
   model only implied. Two of its edges were written (`types` on
   `reader`, `verdicts` on `types`), one was missing: `report` did not
   require `verdicts`. Added. The model file `operators.mitra` held
   the subject `eval`: renamed `eval.mitra`.
8. Neither corrected the other (step 5). The five steps of `evaluate`
   are not in the model. They are how a verdict is computed, and the
   seven promises of `verdicts` cut across them (`conflict_kept` is
   kept by steps 1, 2 and 4). I left the model by intent, and each
   step file says in its first lines what it does.
