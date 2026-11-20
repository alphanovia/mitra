# How much of Mitra its model covers

Written by `scripts/coverage.py --write`. A function is covered when a
`// mitra: implements …` comment ties it to the model. Tests are left out.

| Crate | Functions | Tied | Share | Lines in functions | In tied ones | Share | Code lines | Model lines | Model / code |
|---|---|---|---|---|---|---|---|---|---|
| mitra-lang | 430 | 58 | 13% | 14563 | 3100 | 21% | 15498 | 1060 | 7% |
| mitra-store | 27 | 13 | 48% | 415 | 206 | 50% | 424 | 105 | 25% |
| mitra-rust | 26 | 8 | 31% | 1024 | 576 | 56% | 1076 | 44 | 4% |
| mitra-cli | 23 | 2 | 9% | 700 | 60 | 9% | 739 | 57 | 8% |
| all | 506 | 81 | 16% | 16702 | 3942 | 24% | 17737 | 1266 | 7% |

- mitra-lang: 1060 lines of model over 15498 lines of code, in 22 subjects with 85 ensures and 8 requires; apart from it, 97 lines of point claims and 24 tests pinned in the code.
- mitra-store: 105 lines of model over 424 lines of code, in 4 subjects with 22 ensures and 8 requires; apart from it, 0 lines of point claims and 0 tests pinned in the code.
- mitra-rust: 44 lines of model over 1076 lines of code, in 2 subjects with 3 ensures and 3 requires; apart from it, 28 lines of point claims and 0 tests pinned in the code.
- mitra-cli: 57 lines of model over 739 lines of code, in 1 subject with 11 ensures and 18 requires; apart from it, 0 lines of point claims and 4 tests pinned in the code.

## mitra-lang

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| ast.rs | `voice_parts` | 283 | 6 | voices::parts |
| ast.rs | `apart` | 294 | 19 | voices::apart |
| ast.rs | `act` | 320 | 14 | ledger::Evidence::act |
| ast.rs | `voice` | 337 | 3 | ledger::Evidence::voice |
| ast.rs | `by_machine` | 345 | 3 | ledger::Evidence::by_machine |
| ast.rs | `block_of` | 503 | 6 | model::Model::block_of |
| ast.rs | `check` | 721 | 417 | types::Model::check |
| build.rs | `build` | 37 | 61 | build::build |
| build.rs | `lock` | 123 | 3 | build::Built::lock |
| build.rs | `count` | 129 | 3 | build::Built::count |
| build.rs | `interface` | 138 | 76 | build::interface |
| eval.rs | `as_str` | 29 | 8 | report::Logic::as_str |
| eval.rs | `unary` | 1495 | 12 | eval::unary |
| eval.rs | `binary` | 1510 | 63 | eval::binary |
| eval.rs | `evaluate` | 1828 | 1361 | verdicts::evaluate |
| fingerprint.rs | `hash` | 12 | 8 | fingerprint::hash |
| fingerprint.rs | `all` | 87 | 3 | fingerprint::all |
| fingerprint.rs | `of` | 94 | 9 | fingerprint::of |
| frontend.rs | `results` | 30 | 1 | Reader::results |
| frontend.rs | `tests` | 34 | 1 | Reader::tests |
| frontend.rs | `tie` | 38 | 1 | Reader::tie |
| frontend.rs | `erasures` | 43 | 1 | Reader::erasures |
| ledger/read.rs | `entries` | 343 | 49 | ledger::entries |
| levels.rs | `left` | 24 | 27 | levels::left |
| lib.rs | `load` | 99 | 3 | types::load |
| lib.rs | `load_modules` | 106 | 7 | types::load_modules |
| lib.rs | `run` | 462 | 3 | verdicts::run |
| modules.rs | `alone` | 34 | 4 | modules::Modules::alone |
| modules.rs | `new` | 42 | 20 | modules::Modules::new |
| modules.rs | `text` | 71 | 3 | modules::Modules::text |
| modules.rs | `place` | 77 | 6 | modules::Modules::place |
| modules.rs | `locate` | 87 | 6 | modules::Modules::locate |
| modules/read.rs | `imports` | 183 | 26 | modules::imports |
| plan.rs | `plan` | 17 | 272 | advice::plan |
| print.rs | `negative` | 117 | 7 | print::negative |
| print.rs | `level` | 131 | 9 | print::level |
| prove.rs | `derive` | 478 | 45 | verdicts::derive |
| reader.rs | `model` | 692 | 3 | reader::model |
| report.rs | `word` | 219 | 11 | report::word |
| results.rs | `from_tests` | 62 | 95 | results::from_tests |
| results.rs | `signed` | 169 | 16 | results::signed |
| results.rs | `short` | 188 | 4 | results::short |
| results.rs | `attr` | 196 | 30 | results::attr |
| results.rs | `strip` | 229 | 11 | results::strip |
| results.rs | `junit` | 242 | 28 | results::junit |
| set.rs | `quote` | 1146 | 13 | print::quote |
| settings.rs | `read` | 56 | 68 | settings::Settings::read |
| settings.rs | `sample` | 129 | 3 | settings::Settings::sample |
| settings.rs | `doubt` | 135 | 4 | settings::Settings::doubt |
| settings.rs | `strength` | 143 | 3 | settings::Settings::strength |
| settings.rs | `written` | 150 | 20 | settings::Settings::written |
| trust/culprits.rs | `rank` | 59 | 22 | culprits::rank |
| voices.rs | `word` | 37 | 15 | voices::word |
| voices.rs | `error_rate` | 109 | 11 | voices::error_rate |
| voices.rs | `support` | 123 | 65 | voices::support |
| voices.rs | `missing` | 202 | 26 | voices::missing |
| voices.rs | `policy_reads` | 231 | 19 | voices::policy_reads |
| voices.rs | `summary` | 292 | 67 | voices::summary |
| ast.rs | `set` | 93 | 6 |  |
| ast.rs | `is_spec` | 119 | 3 |  |
| ast.rs | `domain` | 123 | 3 |  |
| ast.rs | `is_structural` | 174 | 6 |  |
| ast.rs | `machine` | 351 | 3 |  |
| ast.rs | `bare_ensures` | 496 | 4 |  |
| ast.rs | `spec_of_classes` | 541 | 22 |  |
| ast.rs | `conforms` | 567 | 4 |  |
| ast.rs | `wholes` | 575 | 14 |  |
| ast.rs | `named` | 593 | 22 |  |
| ast.rs | `named_recursive` | 619 | 16 |  |
| ast.rs | `record` | 635 | 3 |  |
| ast.rs | `function` | 639 | 4 |  |
| ast.rs | `ensure` | 644 | 3 |  |
| ast.rs | `captured` | 649 | 4 |  |
| ast.rs | `invariant` | 656 | 17 |  |
| ast.rs | `declared_function` | 675 | 3 |  |
| ast.rs | `scored` | 680 | 8 |  |
| ast.rs | `subjects_of` | 690 | 14 |  |
| ast.rs | `claim` | 704 | 3 |  |
| ast.rs | `covers` | 709 | 9 |  |
| ast.rs | `arities` | 1159 | 9 |  |
| ast.rs | `passed` | 1170 | 13 |  |
| ast.rs | `descents` | 1190 | 61 |  |
| ast.rs | `part` | 1191 | 7 |  |
| ast.rs | `walk` | 1253 | 26 |  |
| ast.rs | `arms` | 1281 | 21 |  |
| ast.rs | `calls` | 1304 | 21 |  |
| ast.rs | `single` | 1327 | 5 |  |
| build.rs | `hidden` | 101 | 5 |  |
| build.rs | `lines` | 109 | 11 |  |
| claims/read.rs | `claim` | 7 | 237 |  |
| claims/read.rs | `equivalence` | 267 | 124 |  |
| claims/read.rs | `represented` | 400 | 87 |  |
| claims/read.rs | `instance` | 492 | 92 |  |
| contracts/impls.rs | `impls` | 11 | 190 |  |
| contracts/impls.rs | `closing` | 210 | 16 |  |
| contracts/impls.rs | `impl_types` | 230 | 90 |  |
| contracts/impls.rs | `given_types` | 326 | 18 |  |
| contracts/read.rs | `type_params` | 8 | 22 |  |
| contracts/read.rs | `bounded_by` | 32 | 23 |  |
| contracts/read.rs | `unbounded` | 58 | 8 |  |
| contracts/read.rs | `type_tokens` | 68 | 15 |  |
| contracts/read.rs | `contracts` | 109 | 348 |  |
| contracts/read.rs | `bring_contracts` | 471 | 118 |  |
| eval.rs | `as_str` | 54 | 7 |  |
| eval.rs | `text` | 192 | 14 |  |
| eval.rs | `declared_claim` | 250 | 7 |  |
| eval.rs | `code` | 324 | 11 |  |
| eval.rs | `default` | 347 | 6 |  |
| eval.rs | `rules` | 431 | 82 |  |
| eval.rs | `relations` | 523 | 22 |  |
| eval.rs | `close` | 557 | 64 |  |
| eval.rs | `eval` | 625 | 295 |  |
| eval.rs | `call` | 931 | 38 |  |
| eval.rs | `call_inner` | 970 | 104 |  |
| eval.rs | `target` | 1079 | 23 |  |
| eval.rs | `compare` | 1105 | 34 |  |
| eval.rs | `measure` | 1144 | 36 |  |
| eval.rs | `count` | 1184 | 91 |  |
| eval.rs | `candidates` | 1281 | 47 |  |
| eval.rs | `judged` | 1332 | 43 |  |
| eval.rs | `forced` | 1381 | 53 |  |
| eval.rs | `counter` | 1441 | 11 |  |
| eval.rs | `implication` | 1455 | 25 |  |
| eval.rs | `product` | 1483 | 10 |  |
| eval.rs | `class_text` | 1590 | 19 |  |
| eval.rs | `computable` | 1639 | 23 |  |
| eval.rs | `subject_calls` | 1665 | 34 |  |
| eval.rs | `sorted` | 1702 | 86 |  |
| eval.rs | `listed` | 1820 | 6 |  |
| eval.rs | `represented` | 3333 | 23 |  |
| eval.rs | `computed` | 3359 | 22 |  |
| eval.rs | `rank` | 3384 | 146 |  |
| eval.rs | `brief` | 3540 | 14 |  |
| eval.rs | `warnings` | 3565 | 13 |  |
| eval.rs | `own_word` | 3584 | 3 |  |
| fingerprint.rs | `said` | 22 | 60 |  |
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
| instances.rs | `pieces` | 12 | 19 |  |
| instances.rs | `within` | 34 | 15 |  |
| instances.rs | `type_of` | 51 | 23 |  |
| instances.rs | `given` | 77 | 19 |  |
| instances.rs | `unify` | 99 | 62 |  |
| instances.rs | `seen` | 167 | 42 |  |
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
| levels.rs | `diagnosis` | 64 | 44 |  |
| lib.rs | `new` | 72 | 6 |  |
| lib.rs | `fmt` | 81 | 3 |  |
| lib.rs | `read` | 114 | 77 |  |
| lib.rs | `points` | 201 | 84 |  |
| lib.rs | `determinism` | 291 | 24 |  |
| lib.rs | `invariants` | 319 | 40 |  |
| lib.rs | `tie` | 361 | 16 |  |
| lib.rs | `tie_uses` | 380 | 23 |  |
| lib.rs | `tie_records` | 406 | 45 |  |
| lib.rs | `read_evidence` | 455 | 4 |  |
| modules.rs | `files` | 65 | 3 |  |
| modules.rs | `first` | 94 | 3 |  |
| modules.rs | `in_order` | 102 | 50 |  |
| modules/read.rs | `library` | 32 | 3 |  |
| modules/read.rs | `module` | 37 | 15 |  |
| modules/read.rs | `from_store` | 57 | 74 |  |
| modules/read.rs | `imports_of` | 138 | 31 |  |
| modules/read.rs | `of_store` | 173 | 6 |  |
| modules/read.rs | `import` | 213 | 109 |  |
| next.rs | `next` | 16 | 385 |  |
| next.rs | `ask` | 391 | 17 |  |
| next.rs | `vouch` | 410 | 4 |  |
| plan.rs | `owed` | 79 | 158 |  |
| plan.rs | `point` | 243 | 6 |  |
| plan.rs | `subject_call` | 250 | 3 |  |
| plan.rs | `calls_no_subject` | 254 | 5 |  |
| plan.rs | `class_set` | 261 | 24 |  |
| plan.rs | `observed` | 287 | 8 |  |
| plan.rs | `set_text` | 297 | 11 |  |
| print.rs | `print_claim` | 8 | 3 |  |
| print.rs | `print_evidence` | 14 | 73 |  |
| print.rs | `pattern` | 89 | 19 |  |
| print.rs | `call` | 111 | 4 |  |
| print.rs | `expr` | 141 | 3 |  |
| print.rs | `contract` | 146 | 112 |  |
| print.rs | `at` | 262 | 99 |  |
| prove.rs | `val` | 36 | 7 |  |
| prove.rs | `app` | 43 | 4 |  |
| prove.rs | `add` | 47 | 9 |  |
| prove.rs | `grow` | 72 | 10 |  |
| prove.rs | `find` | 82 | 6 |  |
| prove.rs | `union` | 88 | 13 |  |
| prove.rs | `truth` | 101 | 6 |  |
| prove.rs | `suppose` | 108 | 5 |  |
| prove.rs | `open` | 114 | 32 |  |
| prove.rs | `settle` | 147 | 69 |  |
| prove.rs | `impossible` | 222 | 24 |  |
| prove.rs | `operator` | 248 | 6 |  |
| prove.rs | `term` | 264 | 71 |  |
| prove.rs | `statement` | 342 | 29 |  |
| prove.rs | `follows` | 373 | 65 |  |
| prove.rs | `reach` | 443 | 31 |  |
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
| report.rs | `model` | 17 | 1 |  |
| report.rs | `evidence` | 18 | 1 |  |
| report.rs | `model` | 28 | 3 |  |
| report.rs | `evidence` | 31 | 3 |  |
| report.rs | `explain` | 208 | 4 |  |
| report.rs | `headline` | 213 | 3 |  |
| report.rs | `claim` | 239 | 3 |  |
| report.rs | `verdict` | 243 | 4 |  |
| report.rs | `at_claim` | 249 | 8 |  |
| report.rs | `evidence_text` | 258 | 16 |  |
| report.rs | `entry` | 277 | 8 |  |
| report.rs | `declared` | 288 | 5 |  |
| report.rs | `logic` | 295 | 4 |  |
| report.rs | `stated` | 301 | 6 |  |
| report.rs | `evidence_at` | 309 | 6 |  |
| report.rs | `role` | 316 | 16 |  |
| report.rs | `inputs` | 334 | 41 |  |
| report.rs | `missed` | 379 | 21 |  |
| report.rs | `verdicts` | 403 | 174 |  |
| report.rs | `quoted` | 583 | 13 |  |
| report.rs | `piece` | 598 | 109 |  |
| report.rs | `ranking` | 711 | 87 |  |
| report.rs | `error` | 804 | 229 |  |
| report.rs | `lints` | 1037 | 13 |  |
| report.rs | `warnings` | 1051 | 216 |  |
| report.rs | `to_text` | 1173 | 3 |  |
| report.rs | `to_text_with` | 1178 | 7 |  |
| report.rs | `model_text_at` | 1189 | 20 |  |
| report.rs | `problem_texts` | 1211 | 16 |  |
| report.rs | `model_warnings` | 1229 | 3 |  |
| report.rs | `to_text_at` | 1236 | 47 |  |
| set.rs | `set_of` | 47 | 5 |  |
| set.rs | `map_of` | 53 | 11 |  |
| set.rs | `as_set` | 65 | 9 |  |
| set.rs | `as_map` | 75 | 18 |  |
| set.rs | `arity` | 96 | 5 |  |
| set.rs | `value` | 101 | 6 |  |
| set.rs | `show` | 108 | 7 |  |
| set.rs | `rank` | 119 | 11 |  |
| set.rs | `parts` | 131 | 7 |  |
| set.rs | `cmp` | 141 | 12 |  |
| set.rs | `partial_cmp` | 155 | 3 |  |
| set.rs | `eq` | 160 | 3 |  |
| set.rs | `fmt` | 167 | 37 |  |
| set.rs | `cmp` | 215 | 3 |  |
| set.rs | `partial_cmp` | 220 | 3 |  |
| set.rs | `eq` | 225 | 3 |  |
| set.rs | `none` | 239 | 3 |  |
| set.rs | `all` | 242 | 3 |  |
| set.rs | `union` | 245 | 8 |  |
| set.rs | `complement` | 253 | 6 |  |
| set.rs | `none` | 283 | 6 |  |
| set.rs | `full` | 289 | 3 |  |
| set.rs | `of` | 292 | 7 |  |
| set.rs | `build` | 299 | 10 |  |
| set.rs | `union` | 309 | 9 |  |
| set.rs | `intersect` | 318 | 15 |  |
| set.rs | `complement` | 334 | 8 |  |
| set.rs | `eq` | 345 | 8 |  |
| set.rs | `all` | 364 | 3 |  |
| set.rs | `contains` | 367 | 5 |  |
| set.rs | `intersect` | 372 | 10 |  |
| set.rs | `complement` | 383 | 15 |  |
| set.rs | `none` | 412 | 3 |  |
| set.rs | `build` | 417 | 42 |  |
| set.rs | `points` | 462 | 3 |  |
| set.rs | `contains` | 465 | 4 |  |
| set.rs | `union` | 469 | 6 |  |
| set.rs | `intersect` | 475 | 9 |  |
| set.rs | `complement` | 484 | 15 |  |
| set.rs | `is_empty` | 500 | 3 |  |
| set.rs | `show` | 507 | 48 |  |
| set.rs | `entries` | 562 | 16 |  |
| set.rs | `eq` | 580 | 4 |  |
| set.rs | `empty` | 601 | 10 |  |
| set.rs | `bool` | 611 | 6 |  |
| set.rs | `int` | 617 | 6 |  |
| set.rs | `string` | 623 | 6 |  |
| set.rs | `float` | 629 | 6 |  |
| set.rs | `any` | 635 | 20 |  |
| set.rs | `int_type` | 656 | 7 |  |
| set.rs | `range` | 665 | 8 |  |
| set.rs | `point` | 673 | 26 |  |
| set.rs | `list` | 700 | 10 |  |
| set.rs | `sets` | 711 | 3 |  |
| set.rs | `maps` | 715 | 3 |  |
| set.rs | `composite` | 719 | 6 |  |
| set.rs | `tuple` | 726 | 3 |  |
| set.rs | `record` | 730 | 4 |  |
| set.rs | `component` | 736 | 5 |  |
| set.rs | `interval` | 742 | 11 |  |
| set.rs | `union` | 754 | 19 |  |
| set.rs | `complement` | 775 | 23 |  |
| set.rs | `intersect` | 799 | 13 |  |
| set.rs | `complement_scalars` | 816 | 12 |  |
| set.rs | `minus` | 829 | 14 |  |
| set.rs | `is_empty` | 846 | 9 |  |
| set.rs | `is_subset` | 856 | 3 |  |
| set.rs | `contains` | 860 | 13 |  |
| set.rs | `shapes` | 875 | 3 |  |
| set.rs | `conforms` | 880 | 33 |  |
| set.rs | `values` | 916 | 29 |  |
| set.rs | `single` | 947 | 6 |  |
| set.rs | `count` | 955 | 30 |  |
| set.rs | `fmt` | 991 | 138 |  |
| set.rs | `n_is_set` | 1140 | 3 |  |
| set.rs | `paren` | 1161 | 8 |  |
| settings.rs | `default` | 39 | 9 |  |
| trust/culprits.rs | `weigh` | 88 | 51 |  |
| tuples.rs | `empty` | 21 | 6 |  |
| tuples.rs | `product` | 29 | 9 |  |
| tuples.rs | `point` | 39 | 3 |  |
| tuples.rs | `boxes` | 44 | 3 |  |
| tuples.rs | `arity` | 48 | 3 |  |
| tuples.rs | `union` | 52 | 6 |  |
| tuples.rs | `intersect` | 59 | 15 |  |
| tuples.rs | `minus` | 75 | 10 |  |
| tuples.rs | `is_empty` | 86 | 3 |  |
| tuples.rs | `is_subset` | 90 | 3 |  |
| tuples.rs | `same` | 95 | 3 |  |
| tuples.rs | `contains` | 99 | 4 |  |
| tuples.rs | `as_set` | 105 | 4 |  |
| tuples.rs | `values` | 111 | 22 |  |
| tuples.rs | `size_bound` | 137 | 6 |  |
| tuples.rs | `count` | 144 | 7 |  |
| tuples.rs | `show` | 156 | 37 |  |
| tuples.rs | `show_box` | 198 | 4 |  |
| tuples.rs | `box_minus` | 205 | 17 |  |
| types.rs | `check` | 17 | 29 |  |
| types.rs | `inside` | 49 | 11 |  |
| types.rs | `contract` | 62 | 5 |  |
| types.rs | `plain` | 69 | 4 |  |
| types.rs | `show` | 74 | 7 |  |
| types.rs | `unknown` | 82 | 4 |  |
| types.rs | `numbers` | 90 | 3 |  |
| types.rs | `needs` | 95 | 11 |  |
| types.rs | `call` | 108 | 8 |  |
| types.rs | `of` | 118 | 233 |  |
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
| types/read.rs | `set_atom` | 228 | 222 |  |
| types/read.rs | `variants` | 473 | 70 |  |
| types/read.rs | `both` | 546 | 12 |  |
| types/read.rs | `set_both` | 560 | 4 |  |
| types/read.rs | `binders` | 566 | 33 |  |
| types/read.rs | `enumeration` | 605 | 35 |  |
| types/scope.rs | `types_of` | 10 | 55 |  |
| types/scope.rs | `merge_types` | 68 | 27 |  |
| types/scope.rs | `scope_types` | 99 | 56 |  |
| types/scope.rs | `angle` | 159 | 17 |  |
| types/scope.rs | `param_names` | 179 | 18 |  |
| types/scope.rs | `type_args` | 199 | 18 |  |
| types/scope.rs | `with_params` | 220 | 24 |  |
| voices.rs | `behind` | 56 | 36 |  |
| voices.rs | `parts` | 99 | 7 |  |
| voices.rs | `rounded` | 251 | 4 |  |
| voices.rs | `by_line` | 258 | 30 |  |

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
| tie.rs | `results` | 514 | 3 | reader::Rust::results |
| tie.rs | `tests` | 520 | 38 | reader::Rust::tests |
| tie.rs | `erasures` | 560 | 3 | reader::Rust::erasures |
| tie.rs | `tie` | 566 | 469 | reader::Rust::tie |
| source.rs | `rust_files` | 6 | 14 |  |
| source.rs | `scan` | 45 | 55 |  |
| tie.rs | `traits` | 33 | 14 |  |
| tie.rs | `bounds_of` | 50 | 11 |  |
| tie.rs | `head_bounds` | 65 | 13 |  |
| tie.rs | `variants` | 95 | 25 |  |
| tie.rs | `shape_below` | 123 | 22 |  |
| tie.rs | `fields` | 147 | 18 |  |
| tie.rs | `balanced` | 167 | 12 |  |
| tie.rs | `pieces` | 182 | 20 |  |
| tie.rs | `as_model` | 204 | 52 |  |
| tie.rs | `fits` | 263 | 25 |  |
| tie.rs | `implemented` | 293 | 12 |  |
| tie.rs | `words` | 307 | 13 |  |
| tie.rs | `read` | 322 | 90 |  |
| tie.rs | `calls` | 418 | 8 |  |
| tie.rs | `forgot` | 459 | 31 |  |
| tie.rs | `result_of` | 493 | 13 |  |

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
| main.rs | `run_command` | 488 | 340 |  |
