# How much of Mitra its model covers

Written by `scripts/coverage.py --write`. A function is covered when a
`// mitra: implements …` comment ties it to the model. Tests are left out.

| Crate | Functions | Tied | Share | Lines in functions | In tied ones | Share | Code lines | Model lines | Model / code |
|---|---|---|---|---|---|---|---|---|---|
| mitra-lang | 464 | 58 | 12% | 14718 | 1767 | 12% | 15734 | 1061 | 7% |
| mitra-store | 27 | 13 | 48% | 415 | 206 | 50% | 424 | 105 | 25% |
| mitra-rust | 26 | 8 | 31% | 1024 | 576 | 56% | 1082 | 44 | 4% |
| mitra-cli | 23 | 2 | 9% | 701 | 60 | 9% | 740 | 57 | 8% |
| all | 540 | 81 | 15% | 16858 | 2609 | 15% | 17980 | 1267 | 7% |

- mitra-lang: 1061 lines of model over 15734 lines of code, in 22 subjects with 85 ensures and 9 requires; apart from it, 97 lines of point claims and 24 tests pinned in the code.
- mitra-store: 105 lines of model over 424 lines of code, in 4 subjects with 22 ensures and 8 requires; apart from it, 0 lines of point claims and 0 tests pinned in the code.
- mitra-rust: 44 lines of model over 1082 lines of code, in 2 subjects with 3 ensures and 3 requires; apart from it, 28 lines of point claims and 0 tests pinned in the code.
- mitra-cli: 57 lines of model over 740 lines of code, in 1 subject with 11 ensures and 18 requires; apart from it, 0 lines of point claims and 4 tests pinned in the code.

## mitra-lang

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| build.rs | `build` | 39 | 61 | build::build |
| build.rs | `lock` | 125 | 3 | build::Built::lock |
| build.rs | `count` | 131 | 3 | build::Built::count |
| build.rs | `interface` | 140 | 76 | build::interface |
| build/fingerprint.rs | `hash` | 12 | 8 | fingerprint::hash |
| build/fingerprint.rs | `all` | 91 | 3 | fingerprint::all |
| build/fingerprint.rs | `of` | 98 | 9 | fingerprint::of |
| build/levels.rs | `left` | 24 | 27 | levels::left |
| eval.rs | `unary` | 745 | 12 | eval::unary |
| eval.rs | `binary` | 760 | 63 | eval::binary |
| frontend.rs | `results` | 30 | 1 | Reader::results |
| frontend.rs | `tests` | 34 | 1 | Reader::tests |
| frontend.rs | `tie` | 38 | 1 | Reader::tie |
| frontend.rs | `erasures` | 43 | 1 | Reader::erasures |
| ledger/read.rs | `entries` | 343 | 49 | ledger::entries |
| ledger/results.rs | `from_tests` | 62 | 95 | results::from_tests |
| ledger/results.rs | `signed` | 169 | 16 | results::signed |
| ledger/results.rs | `short` | 188 | 4 | results::short |
| ledger/results.rs | `attr` | 196 | 30 | results::attr |
| ledger/results.rs | `strip` | 229 | 11 | results::strip |
| ledger/results.rs | `junit` | 242 | 28 | results::junit |
| ledger/voices.rs | `word` | 37 | 15 | voices::word |
| ledger/voices.rs | `error_rate` | 109 | 11 | voices::error_rate |
| ledger/voices.rs | `support` | 123 | 65 | voices::support |
| ledger/voices.rs | `missing` | 202 | 26 | voices::missing |
| ledger/voices.rs | `policy_reads` | 231 | 19 | voices::policy_reads |
| ledger/voices.rs | `summary` | 292 | 67 | voices::summary |
| model.rs | `voice_parts` | 282 | 6 | voices::parts |
| model.rs | `apart` | 293 | 19 | voices::apart |
| model.rs | `act` | 319 | 14 | ledger::Evidence::act |
| model.rs | `voice` | 336 | 3 | ledger::Evidence::voice |
| model.rs | `by_machine` | 344 | 3 | ledger::Evidence::by_machine |
| model.rs | `block_of` | 502 | 6 | model::Model::block_of |
| modules.rs | `alone` | 34 | 4 | modules::Modules::alone |
| modules.rs | `new` | 42 | 20 | modules::Modules::new |
| modules.rs | `text` | 71 | 3 | modules::Modules::text |
| modules.rs | `place` | 77 | 6 | modules::Modules::place |
| modules.rs | `locate` | 87 | 6 | modules::Modules::locate |
| modules/read.rs | `imports` | 183 | 26 | modules::imports |
| pipeline.rs | `load` | 12 | 3 | types::load |
| pipeline.rs | `load_modules` | 19 | 7 | types::load_modules |
| pipeline.rs | `run` | 375 | 3 | verdicts::run |
| reader.rs | `model` | 692 | 3 | reader::model |
| report.rs | `word` | 50 | 11 | report::word |
| report/plan.rs | `plan` | 17 | 276 | advice::plan |
| report/print.rs | `negative` | 117 | 7 | print::negative |
| report/print.rs | `level` | 131 | 9 | print::level |
| settings.rs | `read` | 56 | 68 | settings::Settings::read |
| settings.rs | `sample` | 129 | 3 | settings::Settings::sample |
| settings.rs | `doubt` | 135 | 4 | settings::Settings::doubt |
| settings.rs | `strength` | 143 | 3 | settings::Settings::strength |
| settings.rs | `written` | 150 | 20 | settings::Settings::written |
| types/check.rs | `check` | 110 | 417 | types::Model::check |
| types/set/show.rs | `quote` | 163 | 13 | print::quote |
| verdicts.rs | `as_str` | 45 | 8 | report::Logic::as_str |
| verdicts.rs | `evaluate` | 417 | 24 | verdicts::evaluate |
| verdicts/culprits.rs | `rank` | 59 | 22 | culprits::rank |
| verdicts/prove.rs | `derive` | 478 | 45 | verdicts::derive |
| build.rs | `hidden` | 103 | 5 |  |
| build.rs | `lines` | 111 | 11 |  |
| build/fingerprint.rs | `said` | 22 | 64 |  |
| build/levels.rs | `diagnosis` | 64 | 44 |  |
| claims/read.rs | `claim` | 7 | 237 |  |
| claims/read.rs | `equivalence` | 267 | 124 |  |
| claims/read.rs | `represented` | 400 | 87 |  |
| claims/read.rs | `instance` | 492 | 92 |  |
| contracts/impls.rs | `impls` | 11 | 190 |  |
| contracts/impls.rs | `closing` | 210 | 16 |  |
| contracts/impls.rs | `impl_types` | 230 | 90 |  |
| contracts/impls.rs | `given_types` | 326 | 18 |  |
| contracts/instances.rs | `pieces` | 12 | 19 |  |
| contracts/instances.rs | `within` | 34 | 15 |  |
| contracts/instances.rs | `type_of` | 51 | 23 |  |
| contracts/instances.rs | `given` | 77 | 19 |  |
| contracts/instances.rs | `unify` | 99 | 62 |  |
| contracts/instances.rs | `seen` | 167 | 42 |  |
| contracts/read.rs | `type_params` | 8 | 22 |  |
| contracts/read.rs | `bounded_by` | 32 | 23 |  |
| contracts/read.rs | `unbounded` | 58 | 8 |  |
| contracts/read.rs | `type_tokens` | 68 | 15 |  |
| contracts/read.rs | `contracts` | 109 | 348 |  |
| contracts/read.rs | `bring_contracts` | 471 | 118 |  |
| eval.rs | `rules` | 77 | 82 |  |
| eval.rs | `relations` | 169 | 22 |  |
| eval.rs | `bare` | 194 | 16 |  |
| eval.rs | `eval` | 215 | 295 |  |
| eval.rs | `call` | 521 | 38 |  |
| eval.rs | `call_inner` | 560 | 104 |  |
| eval.rs | `target` | 669 | 23 |  |
| eval.rs | `compare` | 695 | 34 |  |
| eval.rs | `product` | 733 | 10 |  |
| eval.rs | `computed` | 833 | 9 |  |
| eval/counting.rs | `close` | 18 | 64 |  |
| eval/counting.rs | `computable` | 84 | 23 |  |
| eval/counting.rs | `subject_calls` | 110 | 38 |  |
| eval/counting.rs | `measure` | 154 | 36 |  |
| eval/counting.rs | `count` | 194 | 91 |  |
| eval/counting.rs | `candidates` | 291 | 47 |  |
| eval/counting.rs | `judged` | 342 | 43 |  |
| eval/counting.rs | `forced` | 391 | 58 |  |
| eval/counting.rs | `counter` | 456 | 16 |  |
| eval/counting.rs | `implication` | 475 | 25 |  |
| functions/expr.rs | `args` | 8 | 60 |  |
| functions/expr.rs | `expr` | 73 | 7 |  |
| functions/expr.rs | `or` | 81 | 7 |  |
| functions/expr.rs | `and` | 89 | 7 |  |
| functions/expr.rs | `compare` | 97 | 25 |  |
| functions/expr.rs | `sum` | 124 | 11 |  |
| functions/expr.rs | `product` | 137 | 11 |  |
| functions/expr.rs | `unary` | 149 | 42 |  |
| functions/expr.rs | `method` | 198 | 86 |  |
| functions/expr.rs | `exprs` | 290 | 11 |  |
| functions/expr.rs | `block` | 303 | 6 |  |
| functions/expr.rs | `sequence` | 311 | 16 |  |
| functions/expr.rs | `bars` | 330 | 11 |  |
| functions/expr.rs | `primary` | 342 | 275 |  |
| functions/read.rs | `trust` | 8 | 10 |  |
| functions/read.rs | `state` | 20 | 27 |  |
| functions/read.rs | `define` | 49 | 20 |  |
| functions/read.rs | `declare` | 71 | 17 |  |
| functions/read.rs | `function` | 89 | 213 |  |
| functions/read.rs | `sig` | 320 | 11 |  |
| functions/read.rs | `inputs` | 335 | 27 |  |
| intent/read.rs | `requirements` | 9 | 144 |  |
| intent/read.rs | `reliance` | 167 | 166 |  |
| ledger/read.rs | `evidence` | 25 | 281 |  |
| ledger/read.rs | `evidence` | 321 | 12 |  |
| ledger/read.rs | `evidence_alone` | 335 | 3 |  |
| ledger/read.rs | `blocks` | 396 | 84 |  |
| ledger/read.rs | `free` | 489 | 60 |  |
| ledger/read.rs | `unwritten` | 551 | 6 |  |
| ledger/read.rs | `records_in` | 558 | 32 |  |
| ledger/read.rs | `written` | 593 | 20 |  |
| ledger/read.rs | `stamp` | 616 | 59 |  |
| ledger/read.rs | `tied` | 679 | 4 |  |
| ledger/voices.rs | `behind` | 56 | 36 |  |
| ledger/voices.rs | `parts` | 99 | 7 |  |
| ledger/voices.rs | `rounded` | 251 | 4 |  |
| ledger/voices.rs | `by_line` | 258 | 30 |  |
| lib.rs | `new` | 63 | 6 |  |
| lib.rs | `fmt` | 72 | 3 |  |
| model.rs | `set` | 92 | 6 |  |
| model.rs | `is_spec` | 118 | 3 |  |
| model.rs | `domain` | 122 | 3 |  |
| model.rs | `is_structural` | 173 | 6 |  |
| model.rs | `machine` | 350 | 3 |  |
| model.rs | `bare_ensures` | 495 | 4 |  |
| model.rs | `spec_of_classes` | 540 | 22 |  |
| model.rs | `conforms` | 566 | 4 |  |
| model.rs | `wholes` | 574 | 14 |  |
| model.rs | `named` | 592 | 22 |  |
| model.rs | `named_recursive` | 618 | 16 |  |
| model.rs | `record` | 634 | 3 |  |
| model.rs | `function` | 638 | 4 |  |
| model.rs | `ensure` | 643 | 3 |  |
| model.rs | `captured` | 648 | 4 |  |
| model.rs | `invariant` | 655 | 17 |  |
| model.rs | `declared_function` | 674 | 3 |  |
| model.rs | `scored` | 679 | 8 |  |
| model.rs | `subjects_of` | 689 | 14 |  |
| model.rs | `claim` | 703 | 3 |  |
| model.rs | `covers` | 708 | 9 |  |
| model.rs | `walk` | 720 | 26 |  |
| model.rs | `arms` | 748 | 21 |  |
| model.rs | `calls` | 771 | 21 |  |
| model.rs | `single` | 794 | 5 |  |
| modules.rs | `files` | 65 | 3 |  |
| modules.rs | `first` | 94 | 3 |  |
| modules.rs | `in_order` | 102 | 50 |  |
| modules/read.rs | `library` | 32 | 3 |  |
| modules/read.rs | `module` | 37 | 15 |  |
| modules/read.rs | `from_store` | 57 | 74 |  |
| modules/read.rs | `imports_of` | 138 | 31 |  |
| modules/read.rs | `of_store` | 173 | 6 |  |
| modules/read.rs | `import` | 213 | 109 |  |
| pipeline.rs | `read` | 27 | 77 |  |
| pipeline.rs | `points` | 114 | 84 |  |
| pipeline.rs | `determinism` | 204 | 24 |  |
| pipeline.rs | `invariants` | 232 | 40 |  |
| pipeline.rs | `tie` | 274 | 16 |  |
| pipeline.rs | `tie_uses` | 293 | 23 |  |
| pipeline.rs | `tie_records` | 319 | 45 |  |
| pipeline.rs | `read_evidence` | 368 | 4 |  |
| reader.rs | `bin` | 188 | 3 |  |
| reader.rs | `line` | 193 | 4 |  |
| reader.rs | `err` | 198 | 3 |  |
| reader.rs | `peek` | 202 | 3 |  |
| reader.rs | `peek_at` | 206 | 3 |  |
| reader.rs | `kind` | 210 | 3 |  |
| reader.rs | `kind_at` | 214 | 3 |  |
| reader.rs | `is` | 218 | 3 |  |
| reader.rs | `eat` | 222 | 5 |  |
| reader.rs | `expect` | 228 | 12 |  |
| reader.rs | `name` | 241 | 9 |  |
| reader.rs | `path` | 252 | 7 |  |
| reader.rs | `qualified` | 260 | 6 |  |
| reader.rs | `resolve` | 268 | 19 |  |
| reader.rs | `find` | 291 | 18 |  |
| reader.rs | `old` | 310 | 9 |  |
| reader.rs | `next_old` | 320 | 5 |  |
| reader.rs | `at_end` | 326 | 3 |  |
| reader.rs | `is_name` | 330 | 3 |  |
| reader.rs | `ident` | 335 | 9 |  |
| reader.rs | `word` | 347 | 6 |  |
| reader.rs | `local` | 356 | 6 |  |
| reader.rs | `is_keyword` | 363 | 3 |  |
| reader.rs | `eat_keyword` | 367 | 5 |  |
| reader.rs | `names_between` | 374 | 13 |  |
| reader.rs | `items` | 388 | 284 |  |
| reader.rs | `model_with` | 697 | 5 |  |
| reader.rs | `declared` | 705 | 30 |  |
| reader.rs | `read` | 736 | 60 |  |
| reader/finish.rs | `finish` | 7 | 176 |  |
| reader/tokens.rs | `lex` | 9 | 43 |  |
| reader/tokens.rs | `types_named` | 55 | 20 |  |
| reader/tokens.rs | `lex_raw` | 104 | 67 |  |
| reader/tokens.rs | `earlier` | 176 | 31 |  |
| reader/tokens.rs | `unquote` | 253 | 15 |  |
| reader/tokens.rs | `join` | 270 | 4 |  |
| reader/tokens.rs | `tidy` | 277 | 18 |  |
| report.rs | `model` | 25 | 1 |  |
| report.rs | `evidence` | 26 | 1 |  |
| report.rs | `model` | 36 | 3 |  |
| report.rs | `evidence` | 39 | 3 |  |
| report.rs | `headline` | 44 | 3 |  |
| report.rs | `claim` | 70 | 3 |  |
| report.rs | `verdict` | 74 | 4 |  |
| report.rs | `at_claim` | 80 | 8 |  |
| report.rs | `evidence_text` | 89 | 16 |  |
| report.rs | `entry` | 108 | 8 |  |
| report.rs | `declared` | 119 | 5 |  |
| report.rs | `logic` | 126 | 4 |  |
| report.rs | `stated` | 132 | 6 |  |
| report.rs | `evidence_at` | 140 | 6 |  |
| report.rs | `role` | 147 | 16 |  |
| report.rs | `inputs` | 165 | 41 |  |
| report.rs | `missed` | 210 | 21 |  |
| report.rs | `verdicts` | 234 | 176 |  |
| report.rs | `lints` | 416 | 13 |  |
| report.rs | `warnings` | 430 | 216 |  |
| report.rs | `to_text` | 552 | 3 |  |
| report.rs | `to_text_with` | 557 | 7 |  |
| report.rs | `model_text_at` | 568 | 20 |  |
| report.rs | `problem_texts` | 590 | 16 |  |
| report.rs | `model_warnings` | 608 | 3 |  |
| report.rs | `to_text_at` | 615 | 47 |  |
| report/codes.rs | `explain` | 175 | 4 |  |
| report/errors.rs | `quoted` | 7 | 13 |  |
| report/errors.rs | `piece` | 22 | 109 |  |
| report/errors.rs | `ranking` | 135 | 87 |  |
| report/errors.rs | `error` | 228 | 229 |  |
| report/next.rs | `next` | 16 | 386 |  |
| report/next.rs | `ask` | 392 | 17 |  |
| report/next.rs | `vouch` | 411 | 4 |  |
| report/plan.rs | `owed` | 79 | 158 |  |
| report/plan.rs | `point` | 243 | 6 |  |
| report/plan.rs | `subject_call` | 250 | 3 |  |
| report/plan.rs | `calls_no_subject` | 254 | 5 |  |
| report/plan.rs | `class_set` | 261 | 28 |  |
| report/plan.rs | `observed` | 291 | 8 |  |
| report/plan.rs | `set_text` | 301 | 11 |  |
| report/print.rs | `print_claim` | 8 | 3 |  |
| report/print.rs | `print_evidence` | 14 | 73 |  |
| report/print.rs | `pattern` | 89 | 19 |  |
| report/print.rs | `call` | 111 | 4 |  |
| report/print.rs | `expr` | 141 | 3 |  |
| report/print.rs | `contract` | 146 | 112 |  |
| report/print.rs | `at` | 262 | 101 |  |
| settings.rs | `default` | 39 | 9 |  |
| types.rs | `check` | 20 | 29 |  |
| types.rs | `inside` | 52 | 11 |  |
| types.rs | `contract` | 65 | 5 |  |
| types.rs | `plain` | 72 | 4 |  |
| types.rs | `show` | 77 | 7 |  |
| types.rs | `unknown` | 85 | 4 |  |
| types.rs | `numbers` | 93 | 3 |  |
| types.rs | `needs` | 98 | 11 |  |
| types.rs | `call` | 111 | 8 |  |
| types.rs | `of` | 121 | 233 |  |
| types/check.rs | `arities` | 9 | 9 |  |
| types/check.rs | `passed` | 20 | 13 |  |
| types/check.rs | `descents` | 40 | 66 |  |
| types/check.rs | `part` | 46 | 7 |  |
| types/read.rs | `number` | 8 | 12 |  |
| types/read.rs | `signed` | 22 | 16 |  |
| types/read.rs | `numeral` | 40 | 12 |  |
| types/read.rs | `values` | 54 | 8 |  |
| types/read.rs | `value` | 65 | 10 |  |
| types/read.rs | `value_inner` | 77 | 86 |  |
| types/read.rs | `bound` | 169 | 20 |  |
| types/read.rs | `set` | 191 | 7 |  |
| types/read.rs | `set_inter` | 200 | 12 |  |
| types/read.rs | `sets_in_parentheses` | 214 | 13 |  |
| types/read.rs | `set_atom` | 228 | 226 |  |
| types/read.rs | `variants` | 477 | 70 |  |
| types/read.rs | `both` | 550 | 12 |  |
| types/read.rs | `set_both` | 564 | 4 |  |
| types/read.rs | `binders` | 570 | 33 |  |
| types/read.rs | `enumeration` | 609 | 35 |  |
| types/scope.rs | `types_of` | 10 | 55 |  |
| types/scope.rs | `merge_types` | 68 | 27 |  |
| types/scope.rs | `scope_types` | 99 | 56 |  |
| types/scope.rs | `angle` | 159 | 17 |  |
| types/scope.rs | `param_names` | 179 | 18 |  |
| types/scope.rs | `type_args` | 199 | 18 |  |
| types/scope.rs | `with_params` | 220 | 24 |  |
| types/set.rs | `set_of` | 53 | 5 |  |
| types/set.rs | `map_of` | 59 | 11 |  |
| types/set.rs | `as_set` | 71 | 9 |  |
| types/set.rs | `as_map` | 81 | 18 |  |
| types/set.rs | `arity` | 102 | 5 |  |
| types/set.rs | `value` | 107 | 6 |  |
| types/set.rs | `show` | 114 | 7 |  |
| types/set.rs | `rank` | 125 | 11 |  |
| types/set.rs | `parts` | 137 | 7 |  |
| types/set.rs | `cmp` | 147 | 12 |  |
| types/set.rs | `partial_cmp` | 161 | 3 |  |
| types/set.rs | `eq` | 166 | 3 |  |
| types/set.rs | `fmt` | 173 | 37 |  |
| types/set.rs | `cmp` | 221 | 3 |  |
| types/set.rs | `partial_cmp` | 226 | 3 |  |
| types/set.rs | `eq` | 231 | 3 |  |
| types/set.rs | `none` | 245 | 3 |  |
| types/set.rs | `all` | 248 | 3 |  |
| types/set.rs | `union` | 251 | 8 |  |
| types/set.rs | `complement` | 259 | 6 |  |
| types/set.rs | `none` | 289 | 6 |  |
| types/set.rs | `full` | 295 | 3 |  |
| types/set.rs | `of` | 298 | 7 |  |
| types/set.rs | `build` | 305 | 10 |  |
| types/set.rs | `union` | 315 | 9 |  |
| types/set.rs | `intersect` | 324 | 15 |  |
| types/set.rs | `complement` | 340 | 8 |  |
| types/set.rs | `eq` | 351 | 8 |  |
| types/set.rs | `all` | 370 | 3 |  |
| types/set.rs | `contains` | 373 | 5 |  |
| types/set.rs | `intersect` | 378 | 10 |  |
| types/set.rs | `complement` | 389 | 15 |  |
| types/set.rs | `empty` | 422 | 10 |  |
| types/set.rs | `bool` | 432 | 6 |  |
| types/set.rs | `int` | 438 | 6 |  |
| types/set.rs | `string` | 444 | 6 |  |
| types/set.rs | `float` | 450 | 6 |  |
| types/set.rs | `any` | 456 | 20 |  |
| types/set.rs | `int_type` | 477 | 7 |  |
| types/set.rs | `range` | 486 | 8 |  |
| types/set.rs | `point` | 494 | 26 |  |
| types/set.rs | `list` | 521 | 10 |  |
| types/set.rs | `sets` | 532 | 3 |  |
| types/set.rs | `maps` | 536 | 3 |  |
| types/set.rs | `composite` | 540 | 6 |  |
| types/set.rs | `tuple` | 547 | 3 |  |
| types/set.rs | `record` | 551 | 4 |  |
| types/set.rs | `component` | 557 | 5 |  |
| types/set.rs | `interval` | 563 | 11 |  |
| types/set.rs | `union` | 575 | 19 |  |
| types/set.rs | `complement` | 596 | 23 |  |
| types/set.rs | `intersect` | 620 | 13 |  |
| types/set.rs | `complement_scalars` | 637 | 12 |  |
| types/set.rs | `minus` | 650 | 14 |  |
| types/set.rs | `is_empty` | 667 | 9 |  |
| types/set.rs | `is_subset` | 677 | 3 |  |
| types/set.rs | `contains` | 681 | 13 |  |
| types/set.rs | `shapes` | 696 | 3 |  |
| types/set.rs | `conforms` | 701 | 33 |  |
| types/set.rs | `values` | 737 | 29 |  |
| types/set.rs | `single` | 768 | 6 |  |
| types/set.rs | `count` | 776 | 30 |  |
| types/set/lists.rs | `none` | 16 | 3 |  |
| types/set/lists.rs | `build` | 21 | 42 |  |
| types/set/lists.rs | `points` | 66 | 3 |  |
| types/set/lists.rs | `contains` | 69 | 4 |  |
| types/set/lists.rs | `union` | 73 | 6 |  |
| types/set/lists.rs | `intersect` | 79 | 9 |  |
| types/set/lists.rs | `complement` | 88 | 15 |  |
| types/set/lists.rs | `is_empty` | 104 | 3 |  |
| types/set/lists.rs | `show` | 111 | 48 |  |
| types/set/lists.rs | `entries` | 166 | 16 |  |
| types/set/lists.rs | `eq` | 184 | 4 |  |
| types/set/show.rs | `fmt` | 8 | 138 |  |
| types/set/show.rs | `n_is_set` | 157 | 3 |  |
| types/set/show.rs | `paren` | 178 | 8 |  |
| types/tuples.rs | `empty` | 21 | 6 |  |
| types/tuples.rs | `product` | 29 | 9 |  |
| types/tuples.rs | `point` | 39 | 3 |  |
| types/tuples.rs | `boxes` | 44 | 3 |  |
| types/tuples.rs | `arity` | 48 | 3 |  |
| types/tuples.rs | `union` | 52 | 6 |  |
| types/tuples.rs | `intersect` | 59 | 15 |  |
| types/tuples.rs | `minus` | 75 | 10 |  |
| types/tuples.rs | `is_empty` | 86 | 3 |  |
| types/tuples.rs | `is_subset` | 90 | 3 |  |
| types/tuples.rs | `same` | 95 | 3 |  |
| types/tuples.rs | `contains` | 99 | 4 |  |
| types/tuples.rs | `as_set` | 105 | 4 |  |
| types/tuples.rs | `values` | 111 | 22 |  |
| types/tuples.rs | `size_bound` | 137 | 6 |  |
| types/tuples.rs | `count` | 144 | 7 |  |
| types/tuples.rs | `show` | 156 | 41 |  |
| types/tuples.rs | `show_box` | 202 | 4 |  |
| types/tuples.rs | `box_minus` | 209 | 17 |  |
| verdicts.rs | `as_str` | 70 | 7 |  |
| verdicts.rs | `text` | 208 | 14 |  |
| verdicts.rs | `declared_claim` | 266 | 7 |  |
| verdicts.rs | `code` | 340 | 11 |  |
| verdicts.rs | `default` | 363 | 6 |  |
| verdicts.rs | `model_err` | 388 | 3 |  |
| verdicts.rs | `warnings` | 450 | 13 |  |
| verdicts.rs | `own_word` | 469 | 3 |  |
| verdicts/causes.rs | `failing` | 7 | 14 |  |
| verdicts/causes.rs | `suspects` | 23 | 17 |  |
| verdicts/causes.rs | `roots` | 43 | 46 |  |
| verdicts/classes.rs | `class_text` | 19 | 19 |  |
| verdicts/classes.rs | `sorted` | 68 | 90 |  |
| verdicts/classes.rs | `class_of` | 164 | 10 |  |
| verdicts/classes.rs | `same` | 177 | 4 |  |
| verdicts/classes.rs | `points_of` | 184 | 15 |  |
| verdicts/classes.rs | `blind` | 203 | 3 |  |
| verdicts/classes.rs | `sig` | 208 | 50 |  |
| verdicts/classes.rs | `every` | 262 | 32 |  |
| verdicts/classes.rs | `conflicts` | 296 | 68 |  |
| verdicts/classes.rs | `broken` | 369 | 3 |  |
| verdicts/classes.rs | `usable` | 375 | 3 |  |
| verdicts/classes.rs | `vouched` | 379 | 3 |  |
| verdicts/culprits.rs | `weigh` | 88 | 51 |  |
| verdicts/evidence.rs | `listed` | 36 | 6 |  |
| verdicts/evidence.rs | `shown` | 44 | 13 |  |
| verdicts/evidence.rs | `clash` | 59 | 23 |  |
| verdicts/evidence.rs | `new` | 86 | 43 |  |
| verdicts/evidence.rs | `split` | 132 | 43 |  |
| verdicts/evidence.rs | `direct` | 181 | 180 |  |
| verdicts/evidence.rs | `judged` | 386 | 27 |  |
| verdicts/problems.rs | `uses_of` | 9 | 23 |  |
| verdicts/problems.rs | `is_subject` | 33 | 3 |  |
| verdicts/problems.rs | `quoted` | 37 | 3 |  |
| verdicts/problems.rs | `shown` | 41 | 3 |  |
| verdicts/problems.rs | `problems` | 45 | 18 |  |
| verdicts/problems.rs | `failures` | 67 | 254 |  |
| verdicts/problems.rs | `misfits` | 353 | 37 |  |
| verdicts/problems.rs | `measures` | 393 | 62 |  |
| verdicts/problems.rs | `conflicting` | 459 | 66 |  |
| verdicts/prove.rs | `val` | 36 | 7 |  |
| verdicts/prove.rs | `app` | 43 | 4 |  |
| verdicts/prove.rs | `add` | 47 | 9 |  |
| verdicts/prove.rs | `grow` | 72 | 10 |  |
| verdicts/prove.rs | `find` | 82 | 6 |  |
| verdicts/prove.rs | `union` | 88 | 13 |  |
| verdicts/prove.rs | `truth` | 101 | 6 |  |
| verdicts/prove.rs | `suppose` | 108 | 5 |  |
| verdicts/prove.rs | `open` | 114 | 32 |  |
| verdicts/prove.rs | `settle` | 147 | 69 |  |
| verdicts/prove.rs | `impossible` | 222 | 24 |  |
| verdicts/prove.rs | `operator` | 248 | 6 |  |
| verdicts/prove.rs | `term` | 264 | 71 |  |
| verdicts/prove.rs | `statement` | 342 | 29 |  |
| verdicts/prove.rs | `follows` | 373 | 65 |  |
| verdicts/prove.rs | `reach` | 443 | 31 |  |
| verdicts/rank.rs | `rank` | 7 | 146 |  |
| verdicts/rank.rs | `brief` | 163 | 14 |  |
| verdicts/verdict.rs | `represented` | 15 | 23 |  |
| verdicts/verdict.rs | `stands` | 40 | 24 |  |
| verdicts/verdict.rs | `drawn` | 67 | 9 |  |
| verdicts/verdict.rs | `verdicts` | 78 | 11 |  |
| verdicts/verdict.rs | `verdict` | 90 | 328 |  |

## mitra-store

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| source.rs | `read` | 10 | 3 | files::read |
| source.rs | `file` | 16 | 3 | files::file |
| source.rs | `round` | 129 | 33 | files::round |
| store.rs | `of` | 87 | 6 | store::Store::of |
| store.rs | `model_path` | 116 | 3 | store::Store::model_path |
| store.rs | `source` | 124 | 14 | store::Store::source |
| store.rs | `settings` | 142 | 9 | store::Store::settings |
| store.rs | `evidence_path` | 153 | 3 | store::Store::evidence_path |
| store.rs | `add` | 160 | 16 | ledger::Store::add |
| store.rs | `write_build` | 245 | 13 | store::Store::write_build |
| store.rs | `replace` | 264 | 41 | keeping::Store::replace |
| store.rs | `remove` | 311 | 27 | keeping::Store::remove |
| store.rs | `append` | 354 | 35 | ledger::Store::append |
| source.rs | `files` | 22 | 42 |  |
| source.rs | `store_beside` | 68 | 11 |  |
| source.rs | `model_files` | 81 | 10 |  |
| source.rs | `other_store` | 94 | 8 |  |
| source.rs | `leans_on` | 105 | 19 |  |
| source.rs | `walk` | 130 | 24 |  |
| store.rs | `said` | 51 | 8 |  |
| store.rs | `find` | 74 | 9 |  |
| store.rs | `init` | 95 | 14 |  |
| store.rs | `root` | 111 | 3 |  |
| store.rs | `stamped` | 181 | 25 |  |
| store.rs | `without` | 208 | 15 |  |
| store.rs | `rewrite` | 227 | 13 |  |
| store.rs | `model` | 340 | 8 |  |

## mitra-rust

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| report.rs | `libtest` | 11 | 17 | reader::libtest |
| source.rs | `mitra_line` | 24 | 8 | source::mitra_line |
| source.rs | `fn_below` | 105 | 35 | source::fn_below |
| source.rs | `is_tie` | 144 | 3 | source::is_tie |
| tie.rs | `results` | 26 | 3 | reader::Rust::results |
| tie.rs | `tests` | 32 | 38 | reader::Rust::tests |
| tie.rs | `erasures` | 72 | 3 | reader::Rust::erasures |
| tie.rs | `tie` | 78 | 469 | reader::Rust::tie |
| source.rs | `rust_files` | 6 | 14 |  |
| source.rs | `scan` | 45 | 55 |  |
| tie/code.rs | `traits` | 24 | 14 |  |
| tie/code.rs | `bounds_of` | 41 | 11 |  |
| tie/code.rs | `head_bounds` | 56 | 13 |  |
| tie/code.rs | `variants` | 86 | 25 |  |
| tie/code.rs | `shape_below` | 114 | 22 |  |
| tie/code.rs | `fields` | 138 | 18 |  |
| tie/code.rs | `balanced` | 158 | 12 |  |
| tie/code.rs | `pieces` | 173 | 20 |  |
| tie/code.rs | `as_model` | 195 | 52 |  |
| tie/code.rs | `fits` | 254 | 25 |  |
| tie/code.rs | `implemented` | 284 | 12 |  |
| tie/code.rs | `words` | 298 | 13 |  |
| tie/code.rs | `read` | 313 | 90 |  |
| tie/code.rs | `calls` | 409 | 8 |  |
| tie/erasure.rs | `forgot` | 37 | 31 |  |
| tie/erasure.rs | `result_of` | 71 | 13 |  |

## mitra-cli

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| main.rs | `conform` | 118 | 52 | commands::conform |
| main.rs | `check_rev` | 386 | 8 | commands::check_rev |
| main.rs | `build` | 37 | 75 |  |
| main.rs | `assess` | 173 | 37 |  |
| main.rs | `diagnose` | 214 | 28 |  |
| main.rs | `in_place` | 244 | 7 |  |
| main.rs | `shown` | 253 | 8 |  |
| main.rs | `main` | 280 | 17 |  |
| main.rs | `inputs` | 302 | 14 |  |
| main.rs | `reader` | 322 | 3 |  |
| main.rs | `read` | 326 | 3 |  |
| main.rs | `located` | 331 | 9 |  |
| main.rs | `relative` | 342 | 7 |  |
| main.rs | `model` | 357 | 3 |  |
| main.rs | `evidence` | 360 | 3 |  |
| main.rs | `model` | 372 | 3 |  |
| main.rs | `evidence` | 375 | 6 |  |
| main.rs | `evaluate` | 399 | 25 |  |
| main.rs | `take_rev` | 427 | 18 |  |
| main.rs | `take_value` | 447 | 11 |  |
| main.rs | `take_display` | 461 | 20 |  |
| main.rs | `run` | 483 | 3 |  |
| main.rs | `run_command` | 488 | 341 |  |
