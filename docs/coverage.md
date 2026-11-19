# How much of Mitra its model covers

Written by `scripts/coverage.py --write`. A function is covered when a
`// mitra: implements …` comment ties it to the model. Tests are left out.

| Crate | Functions | Tied | Share | Lines in functions | In tied ones | Share | Code lines | Model lines | Model / code |
|---|---|---|---|---|---|---|---|---|---|
| mitra-lang | 411 | 46 | 11% | 13966 | 2808 | 20% | 14793 | 983 | 7% |
| mitra-store | 22 | 12 | 55% | 333 | 169 | 51% | 365 | 99 | 27% |
| mitra-rust | 26 | 8 | 31% | 1024 | 576 | 56% | 1076 | 42 | 4% |
| mitra-cli | 23 | 2 | 9% | 671 | 60 | 9% | 710 | 56 | 8% |
| all | 482 | 68 | 14% | 15994 | 3613 | 23% | 16944 | 1180 | 7% |

- mitra-lang: 983 lines of model over 14793 lines of code, in 21 subjects with 77 ensures and 8 requires; apart from it, 97 lines of point claims and 24 tests pinned in the code.
- mitra-store: 99 lines of model over 365 lines of code, in 4 subjects with 21 ensures and 8 requires; apart from it, 0 lines of point claims and 0 tests pinned in the code.
- mitra-rust: 42 lines of model over 1076 lines of code, in 2 subjects with 3 ensures and 3 requires; apart from it, 28 lines of point claims and 0 tests pinned in the code.
- mitra-cli: 56 lines of model over 710 lines of code, in 1 subject with 10 ensures and 18 requires; apart from it, 0 lines of point claims and 4 tests pinned in the code.

## mitra-lang

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| ast.rs | `by_machine` | 259 | 3 | ledger::Evidence::by_machine |
| ast.rs | `block_of` | 415 | 6 | model::Model::block_of |
| ast.rs | `check` | 633 | 417 | types::Model::check |
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
| levels.rs | `left` | 24 | 27 | levels::left |
| lib.rs | `load` | 92 | 3 | types::load |
| lib.rs | `load_modules` | 99 | 7 | types::load_modules |
| lib.rs | `run` | 450 | 3 | verdicts::run |
| modules.rs | `alone` | 32 | 4 | modules::Modules::alone |
| modules.rs | `new` | 40 | 20 | modules::Modules::new |
| modules.rs | `text` | 69 | 3 | modules::Modules::text |
| modules.rs | `place` | 75 | 6 | modules::Modules::place |
| modules.rs | `locate` | 85 | 6 | modules::Modules::locate |
| parse.rs | `imports` | 187 | 26 | modules::imports |
| parse.rs | `model` | 4147 | 3 | reader::model |
| parse.rs | `entries` | 5366 | 49 | ledger::entries |
| plan.rs | `plan` | 17 | 272 | advice::plan |
| print.rs | `negative` | 106 | 7 | print::negative |
| print.rs | `level` | 120 | 9 | print::level |
| prove.rs | `derive` | 478 | 45 | verdicts::derive |
| report.rs | `word` | 219 | 11 | report::word |
| results.rs | `from_tests` | 62 | 88 | results::from_tests |
| results.rs | `short` | 160 | 4 | results::short |
| results.rs | `attr` | 168 | 30 | results::attr |
| results.rs | `strip` | 201 | 11 | results::strip |
| results.rs | `junit` | 214 | 28 | results::junit |
| set.rs | `quote` | 1146 | 13 | print::quote |
| settings.rs | `read` | 37 | 57 | settings::Settings::read |
| settings.rs | `sample` | 99 | 3 | settings::Settings::sample |
| settings.rs | `strength` | 106 | 3 | settings::Settings::strength |
| settings.rs | `written` | 113 | 11 | settings::Settings::written |
| trust/culprits.rs | `rank` | 59 | 22 | culprits::rank |
| ast.rs | `set` | 93 | 6 |  |
| ast.rs | `is_spec` | 119 | 3 |  |
| ast.rs | `domain` | 123 | 3 |  |
| ast.rs | `is_structural` | 174 | 6 |  |
| ast.rs | `machine` | 265 | 3 |  |
| ast.rs | `bare_ensures` | 408 | 4 |  |
| ast.rs | `spec_of_classes` | 453 | 22 |  |
| ast.rs | `conforms` | 479 | 4 |  |
| ast.rs | `wholes` | 487 | 14 |  |
| ast.rs | `named` | 505 | 22 |  |
| ast.rs | `named_recursive` | 531 | 16 |  |
| ast.rs | `record` | 547 | 3 |  |
| ast.rs | `function` | 551 | 4 |  |
| ast.rs | `ensure` | 556 | 3 |  |
| ast.rs | `captured` | 561 | 4 |  |
| ast.rs | `invariant` | 568 | 17 |  |
| ast.rs | `declared_function` | 587 | 3 |  |
| ast.rs | `scored` | 592 | 8 |  |
| ast.rs | `subjects_of` | 602 | 14 |  |
| ast.rs | `claim` | 616 | 3 |  |
| ast.rs | `covers` | 621 | 9 |  |
| ast.rs | `arities` | 1071 | 9 |  |
| ast.rs | `passed` | 1082 | 13 |  |
| ast.rs | `descents` | 1102 | 61 |  |
| ast.rs | `part` | 1103 | 7 |  |
| ast.rs | `walk` | 1165 | 26 |  |
| ast.rs | `arms` | 1193 | 21 |  |
| ast.rs | `calls` | 1216 | 21 |  |
| ast.rs | `single` | 1239 | 5 |  |
| build.rs | `hidden` | 101 | 5 |  |
| build.rs | `lines` | 109 | 11 |  |
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
| instances.rs | `pieces` | 12 | 19 |  |
| instances.rs | `within` | 34 | 15 |  |
| instances.rs | `type_of` | 51 | 23 |  |
| instances.rs | `given` | 77 | 19 |  |
| instances.rs | `unify` | 99 | 62 |  |
| instances.rs | `seen` | 167 | 42 |  |
| levels.rs | `diagnosis` | 64 | 44 |  |
| lib.rs | `new` | 65 | 6 |  |
| lib.rs | `fmt` | 74 | 3 |  |
| lib.rs | `read` | 107 | 74 |  |
| lib.rs | `points` | 189 | 84 |  |
| lib.rs | `determinism` | 279 | 24 |  |
| lib.rs | `invariants` | 307 | 40 |  |
| lib.rs | `tie` | 349 | 16 |  |
| lib.rs | `tie_uses` | 368 | 23 |  |
| lib.rs | `tie_records` | 394 | 45 |  |
| lib.rs | `read_evidence` | 443 | 4 |  |
| modules.rs | `files` | 63 | 3 |  |
| modules.rs | `first` | 92 | 3 |  |
| modules.rs | `in_order` | 100 | 50 |  |
| next.rs | `next` | 16 | 385 |  |
| next.rs | `ask` | 391 | 17 |  |
| next.rs | `vouch` | 410 | 4 |  |
| parse.rs | `library` | 36 | 3 |  |
| parse.rs | `module` | 41 | 15 |  |
| parse.rs | `from_store` | 61 | 74 |  |
| parse.rs | `imports_of` | 142 | 31 |  |
| parse.rs | `of_store` | 177 | 6 |  |
| parse.rs | `lex` | 260 | 67 |  |
| parse.rs | `earlier` | 332 | 31 |  |
| parse.rs | `unquote` | 457 | 15 |  |
| parse.rs | `join` | 474 | 4 |  |
| parse.rs | `tidy` | 481 | 18 |  |
| parse.rs | `bin` | 611 | 3 |  |
| parse.rs | `line` | 616 | 4 |  |
| parse.rs | `err` | 620 | 3 |  |
| parse.rs | `peek` | 623 | 3 |  |
| parse.rs | `peek_at` | 626 | 3 |  |
| parse.rs | `kind` | 629 | 3 |  |
| parse.rs | `kind_at` | 632 | 3 |  |
| parse.rs | `is` | 635 | 3 |  |
| parse.rs | `eat` | 638 | 5 |  |
| parse.rs | `expect` | 643 | 12 |  |
| parse.rs | `name` | 655 | 9 |  |
| parse.rs | `path` | 665 | 7 |  |
| parse.rs | `qualified` | 672 | 6 |  |
| parse.rs | `resolve` | 679 | 19 |  |
| parse.rs | `find` | 701 | 18 |  |
| parse.rs | `old` | 720 | 9 |  |
| parse.rs | `next_old` | 729 | 5 |  |
| parse.rs | `at_end` | 734 | 3 |  |
| parse.rs | `is_name` | 737 | 3 |  |
| parse.rs | `ident` | 741 | 9 |  |
| parse.rs | `word` | 752 | 6 |  |
| parse.rs | `local` | 760 | 6 |  |
| parse.rs | `is_keyword` | 766 | 3 |  |
| parse.rs | `eat_keyword` | 769 | 5 |  |
| parse.rs | `number` | 778 | 12 |  |
| parse.rs | `signed` | 791 | 16 |  |
| parse.rs | `numeral` | 808 | 12 |  |
| parse.rs | `values` | 821 | 8 |  |
| parse.rs | `value` | 831 | 10 |  |
| parse.rs | `value_inner` | 842 | 86 |  |
| parse.rs | `bound` | 933 | 20 |  |
| parse.rs | `set` | 957 | 7 |  |
| parse.rs | `set_inter` | 965 | 12 |  |
| parse.rs | `sets_in_parentheses` | 978 | 13 |  |
| parse.rs | `set_atom` | 991 | 220 |  |
| parse.rs | `variants` | 1234 | 70 |  |
| parse.rs | `both` | 1306 | 12 |  |
| parse.rs | `set_both` | 1319 | 4 |  |
| parse.rs | `binders` | 1324 | 33 |  |
| parse.rs | `args` | 1362 | 60 |  |
| parse.rs | `expr` | 1429 | 7 |  |
| parse.rs | `or` | 1436 | 7 |  |
| parse.rs | `and` | 1443 | 7 |  |
| parse.rs | `compare` | 1450 | 25 |  |
| parse.rs | `sum` | 1476 | 11 |  |
| parse.rs | `product` | 1488 | 11 |  |
| parse.rs | `unary` | 1499 | 42 |  |
| parse.rs | `method` | 1547 | 86 |  |
| parse.rs | `exprs` | 1638 | 11 |  |
| parse.rs | `block` | 1650 | 6 |  |
| parse.rs | `sequence` | 1657 | 16 |  |
| parse.rs | `bars` | 1675 | 11 |  |
| parse.rs | `primary` | 1686 | 275 |  |
| parse.rs | `trust` | 1976 | 10 |  |
| parse.rs | `state` | 1990 | 20 |  |
| parse.rs | `define` | 2011 | 13 |  |
| parse.rs | `declare` | 2025 | 12 |  |
| parse.rs | `type_params` | 2038 | 22 |  |
| parse.rs | `bounded_by` | 2061 | 23 |  |
| parse.rs | `unbounded` | 2086 | 8 |  |
| parse.rs | `type_tokens` | 2095 | 15 |  |
| parse.rs | `enumeration` | 2112 | 35 |  |
| parse.rs | `items` | 2149 | 267 |  |
| parse.rs | `function` | 2431 | 213 |  |
| parse.rs | `sig` | 2662 | 11 |  |
| parse.rs | `inputs` | 2676 | 27 |  |
| parse.rs | `claim` | 2706 | 237 |  |
| parse.rs | `equivalence` | 2966 | 124 |  |
| parse.rs | `represented` | 3099 | 87 |  |
| parse.rs | `import` | 3190 | 109 |  |
| parse.rs | `instance` | 3311 | 92 |  |
| parse.rs | `evidence` | 3409 | 186 |  |
| parse.rs | `finish` | 3605 | 176 |  |
| parse.rs | `requirements` | 3797 | 144 |  |
| parse.rs | `reliance` | 3957 | 166 |  |
| parse.rs | `model_with` | 4152 | 5 |  |
| parse.rs | `declared` | 4160 | 30 |  |
| parse.rs | `types_of` | 4195 | 55 |  |
| parse.rs | `merge_types` | 4253 | 27 |  |
| parse.rs | `scope_types` | 4284 | 56 |  |
| parse.rs | `angle` | 4344 | 17 |  |
| parse.rs | `param_names` | 4364 | 18 |  |
| parse.rs | `type_args` | 4384 | 18 |  |
| parse.rs | `with_params` | 4405 | 24 |  |
| parse.rs | `impls` | 4435 | 190 |  |
| parse.rs | `closing` | 4651 | 16 |  |
| parse.rs | `impl_types` | 4671 | 90 |  |
| parse.rs | `given_types` | 4767 | 18 |  |
| parse.rs | `contracts` | 4793 | 348 |  |
| parse.rs | `bring_contracts` | 5155 | 118 |  |
| parse.rs | `read` | 5280 | 60 |  |
| parse.rs | `evidence` | 5344 | 12 |  |
| parse.rs | `evidence_alone` | 5358 | 3 |  |
| parse.rs | `blocks` | 5419 | 84 |  |
| parse.rs | `free` | 5512 | 60 |  |
| parse.rs | `unwritten` | 5574 | 6 |  |
| parse.rs | `records_in` | 5581 | 32 |  |
| parse.rs | `written` | 5616 | 20 |  |
| parse.rs | `stamp` | 5639 | 59 |  |
| parse.rs | `tied` | 5702 | 4 |  |
| plan.rs | `owed` | 79 | 158 |  |
| plan.rs | `point` | 243 | 6 |  |
| plan.rs | `subject_call` | 250 | 3 |  |
| plan.rs | `calls_no_subject` | 254 | 5 |  |
| plan.rs | `class_set` | 261 | 24 |  |
| plan.rs | `observed` | 287 | 8 |  |
| plan.rs | `set_text` | 297 | 11 |  |
| print.rs | `print_claim` | 8 | 3 |  |
| print.rs | `print_evidence` | 14 | 62 |  |
| print.rs | `pattern` | 78 | 19 |  |
| print.rs | `call` | 100 | 4 |  |
| print.rs | `expr` | 130 | 3 |  |
| print.rs | `contract` | 135 | 112 |  |
| print.rs | `at` | 251 | 99 |  |
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
| report.rs | `verdicts` | 403 | 171 |  |
| report.rs | `quoted` | 580 | 13 |  |
| report.rs | `piece` | 595 | 109 |  |
| report.rs | `ranking` | 708 | 87 |  |
| report.rs | `error` | 801 | 229 |  |
| report.rs | `lints` | 1034 | 13 |  |
| report.rs | `warnings` | 1048 | 216 |  |
| report.rs | `to_text` | 1170 | 3 |  |
| report.rs | `to_text_with` | 1175 | 7 |  |
| report.rs | `model_text_at` | 1186 | 20 |  |
| report.rs | `problem_texts` | 1208 | 16 |  |
| report.rs | `model_warnings` | 1226 | 3 |  |
| report.rs | `to_text_at` | 1233 | 47 |  |
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
| settings.rs | `default` | 22 | 7 |  |
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
| types.rs | `check` | 14 | 29 |  |
| types.rs | `inside` | 46 | 11 |  |
| types.rs | `contract` | 59 | 5 |  |
| types.rs | `plain` | 66 | 4 |  |
| types.rs | `show` | 71 | 7 |  |
| types.rs | `unknown` | 79 | 4 |  |
| types.rs | `numbers` | 87 | 3 |  |
| types.rs | `needs` | 92 | 11 |  |
| types.rs | `call` | 105 | 8 |  |
| types.rs | `of` | 115 | 233 |  |

## mitra-store

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| source.rs | `read` | 10 | 3 | files::read |
| source.rs | `file` | 16 | 3 | files::file |
| store.rs | `of` | 86 | 6 | store::Store::of |
| store.rs | `model_path` | 115 | 3 | store::Store::model_path |
| store.rs | `source` | 123 | 14 | store::Store::source |
| store.rs | `settings` | 141 | 9 | store::Store::settings |
| store.rs | `evidence_path` | 152 | 3 | store::Store::evidence_path |
| store.rs | `add` | 159 | 16 | ledger::Store::add |
| store.rs | `write_build` | 243 | 9 | store::Store::write_build |
| store.rs | `replace` | 258 | 41 | keeping::Store::replace |
| store.rs | `remove` | 305 | 27 | keeping::Store::remove |
| store.rs | `append` | 348 | 35 | ledger::Store::append |
| source.rs | `files` | 22 | 42 |  |
| source.rs | `other_store` | 69 | 24 |  |
| store.rs | `said` | 48 | 8 |  |
| store.rs | `find` | 70 | 12 |  |
| store.rs | `init` | 94 | 14 |  |
| store.rs | `root` | 110 | 3 |  |
| store.rs | `stamped` | 180 | 25 |  |
| store.rs | `without` | 207 | 15 |  |
| store.rs | `rewrite` | 226 | 13 |  |
| store.rs | `model` | 334 | 8 |  |

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
| main.rs | `conform` | 109 | 52 | commands::conform |
| main.rs | `check_rev` | 376 | 8 | commands::check_rev |
| main.rs | `build` | 37 | 66 |  |
| main.rs | `assess` | 164 | 36 |  |
| main.rs | `diagnose` | 204 | 28 |  |
| main.rs | `in_place` | 234 | 7 |  |
| main.rs | `shown` | 243 | 8 |  |
| main.rs | `main` | 270 | 17 |  |
| main.rs | `inputs` | 292 | 14 |  |
| main.rs | `reader` | 312 | 3 |  |
| main.rs | `read` | 316 | 3 |  |
| main.rs | `located` | 321 | 9 |  |
| main.rs | `relative` | 332 | 7 |  |
| main.rs | `model` | 347 | 3 |  |
| main.rs | `evidence` | 350 | 3 |  |
| main.rs | `model` | 362 | 3 |  |
| main.rs | `evidence` | 365 | 6 |  |
| main.rs | `evaluate` | 389 | 25 |  |
| main.rs | `take_rev` | 417 | 18 |  |
| main.rs | `take_value` | 437 | 11 |  |
| main.rs | `take_display` | 451 | 20 |  |
| main.rs | `run` | 473 | 3 |  |
| main.rs | `run_command` | 478 | 321 |  |
