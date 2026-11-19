# How much of Mitra its model covers

Written by `scripts/coverage.py --write`. A function is covered when a
`// mitra: implements …` comment ties it to the model. Tests are left out.

| Crate | Functions | Tied | Share | Lines in functions | In tied ones | Share | Code lines | Model lines | Model / code |
|---|---|---|---|---|---|---|---|---|---|
| mitra-lang | 351 | 19 | 5% | 11655 | 2261 | 19% | 12331 | 532 | 4% |
| mitra-store | 13 | 8 | 62% | 160 | 85 | 53% | 178 | 71 | 40% |
| mitra-rust | 22 | 9 | 41% | 745 | 419 | 56% | 791 | 59 | 7% |
| mitra-cli | 15 | 1 | 7% | 499 | 8 | 2% | 521 | 28 | 5% |
| all | 401 | 37 | 9% | 13059 | 2773 | 21% | 13821 | 690 | 5% |

- mitra-lang: 532 lines of model over 12331 lines of code, in 14 subjects with 41 ensures and 5 requires; apart from it, 67 lines of point claims and 10 tests pinned in the code.
- mitra-store: 71 lines of model over 178 lines of code, in 4 subjects with 17 ensures and 7 requires; apart from it, 0 lines of point claims and 0 tests pinned in the code.
- mitra-rust: 59 lines of model over 791 lines of code, in 3 subjects with 8 ensures and 3 requires; apart from it, 58 lines of point claims and 0 tests pinned in the code.
- mitra-cli: 28 lines of model over 521 lines of code, in 1 subject with 7 ensures and 3 requires; apart from it, 0 lines of point claims and 3 tests pinned in the code.

## mitra-lang

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| ast.rs | `check` | 517 | 417 | types::Model::check |
| eval.rs | `as_str` | 28 | 8 | report::Logic::as_str |
| eval.rs | `unary` | 1493 | 12 | eval::unary |
| eval.rs | `binary` | 1508 | 63 | eval::binary |
| eval.rs | `evaluate` | 1826 | 1336 | verdicts::evaluate |
| lib.rs | `load` | 83 | 3 | types::load |
| lib.rs | `load_modules` | 90 | 7 | types::load_modules |
| lib.rs | `run` | 403 | 3 | verdicts::run |
| modules.rs | `alone` | 32 | 4 | modules::Modules::alone |
| modules.rs | `new` | 40 | 20 | modules::Modules::new |
| modules.rs | `place` | 74 | 6 | modules::Modules::place |
| parse.rs | `model` | 3650 | 3 | reader::model |
| plan.rs | `plan` | 17 | 272 | advice::plan |
| print.rs | `negative` | 101 | 7 | print::negative |
| print.rs | `level` | 115 | 9 | print::level |
| prove.rs | `derive` | 478 | 45 | verdicts::derive |
| report.rs | `word` | 194 | 11 | report::word |
| set.rs | `quote` | 1139 | 13 | print::quote |
| trust/culprits.rs | `rank` | 58 | 22 | culprits::rank |
| ast.rs | `set` | 93 | 6 |  |
| ast.rs | `is_spec` | 119 | 3 |  |
| ast.rs | `domain` | 123 | 3 |  |
| ast.rs | `is_structural` | 173 | 6 |  |
| ast.rs | `spec_of_classes` | 337 | 22 |  |
| ast.rs | `conforms` | 363 | 4 |  |
| ast.rs | `wholes` | 371 | 14 |  |
| ast.rs | `named` | 389 | 22 |  |
| ast.rs | `named_recursive` | 415 | 16 |  |
| ast.rs | `record` | 431 | 3 |  |
| ast.rs | `function` | 435 | 4 |  |
| ast.rs | `ensure` | 440 | 3 |  |
| ast.rs | `captured` | 445 | 4 |  |
| ast.rs | `invariant` | 452 | 17 |  |
| ast.rs | `declared_function` | 471 | 3 |  |
| ast.rs | `scored` | 476 | 8 |  |
| ast.rs | `subjects_of` | 486 | 14 |  |
| ast.rs | `claim` | 500 | 3 |  |
| ast.rs | `covers` | 505 | 9 |  |
| ast.rs | `arities` | 955 | 9 |  |
| ast.rs | `passed` | 966 | 13 |  |
| ast.rs | `descents` | 986 | 61 |  |
| ast.rs | `part` | 987 | 7 |  |
| ast.rs | `walk` | 1049 | 26 |  |
| ast.rs | `arms` | 1077 | 21 |  |
| ast.rs | `calls` | 1100 | 21 |  |
| ast.rs | `single` | 1123 | 5 |  |
| eval.rs | `as_str` | 53 | 7 |  |
| eval.rs | `text` | 191 | 14 |  |
| eval.rs | `declared_claim` | 248 | 7 |  |
| eval.rs | `code` | 322 | 11 |  |
| eval.rs | `default` | 345 | 6 |  |
| eval.rs | `rules` | 429 | 82 |  |
| eval.rs | `relations` | 521 | 22 |  |
| eval.rs | `close` | 555 | 64 |  |
| eval.rs | `eval` | 623 | 295 |  |
| eval.rs | `call` | 929 | 38 |  |
| eval.rs | `call_inner` | 968 | 104 |  |
| eval.rs | `target` | 1077 | 23 |  |
| eval.rs | `compare` | 1103 | 34 |  |
| eval.rs | `measure` | 1142 | 36 |  |
| eval.rs | `count` | 1182 | 91 |  |
| eval.rs | `candidates` | 1279 | 47 |  |
| eval.rs | `judged` | 1330 | 43 |  |
| eval.rs | `forced` | 1379 | 53 |  |
| eval.rs | `counter` | 1439 | 11 |  |
| eval.rs | `implication` | 1453 | 25 |  |
| eval.rs | `product` | 1481 | 10 |  |
| eval.rs | `class_text` | 1588 | 19 |  |
| eval.rs | `computable` | 1637 | 23 |  |
| eval.rs | `subject_calls` | 1663 | 34 |  |
| eval.rs | `sorted` | 1700 | 86 |  |
| eval.rs | `listed` | 1818 | 6 |  |
| eval.rs | `represented` | 3304 | 23 |  |
| eval.rs | `computed` | 3330 | 22 |  |
| eval.rs | `rank` | 3355 | 146 |  |
| eval.rs | `brief` | 3511 | 14 |  |
| eval.rs | `warnings` | 3536 | 10 |  |
| eval.rs | `own_word` | 3552 | 3 |  |
| lib.rs | `new` | 56 | 6 |  |
| lib.rs | `fmt` | 65 | 3 |  |
| lib.rs | `read` | 98 | 49 |  |
| lib.rs | `points` | 154 | 84 |  |
| lib.rs | `determinism` | 244 | 24 |  |
| lib.rs | `invariants` | 272 | 40 |  |
| lib.rs | `tie` | 314 | 16 |  |
| lib.rs | `tie_uses` | 333 | 23 |  |
| lib.rs | `tie_records` | 359 | 34 |  |
| lib.rs | `read_evidence` | 396 | 4 |  |
| modules.rs | `files` | 63 | 3 |  |
| modules.rs | `text` | 68 | 3 |  |
| modules.rs | `locate` | 83 | 6 |  |
| modules.rs | `first` | 90 | 3 |  |
| modules.rs | `in_order` | 98 | 50 |  |
| next.rs | `next` | 16 | 385 |  |
| next.rs | `ask` | 391 | 17 |  |
| next.rs | `vouch` | 410 | 4 |  |
| parse.rs | `library` | 31 | 3 |  |
| parse.rs | `module` | 36 | 15 |  |
| parse.rs | `imports` | 54 | 26 |  |
| parse.rs | `lex` | 110 | 67 |  |
| parse.rs | `earlier` | 182 | 31 |  |
| parse.rs | `unquote` | 307 | 15 |  |
| parse.rs | `join` | 324 | 4 |  |
| parse.rs | `bin` | 427 | 3 |  |
| parse.rs | `line` | 432 | 4 |  |
| parse.rs | `err` | 436 | 3 |  |
| parse.rs | `peek` | 439 | 3 |  |
| parse.rs | `peek_at` | 442 | 3 |  |
| parse.rs | `kind` | 445 | 3 |  |
| parse.rs | `kind_at` | 448 | 3 |  |
| parse.rs | `is` | 451 | 3 |  |
| parse.rs | `eat` | 454 | 5 |  |
| parse.rs | `expect` | 459 | 12 |  |
| parse.rs | `name` | 471 | 9 |  |
| parse.rs | `path` | 481 | 7 |  |
| parse.rs | `qualified` | 488 | 6 |  |
| parse.rs | `resolve` | 495 | 19 |  |
| parse.rs | `find` | 517 | 18 |  |
| parse.rs | `old` | 536 | 9 |  |
| parse.rs | `next_old` | 545 | 5 |  |
| parse.rs | `at_end` | 550 | 3 |  |
| parse.rs | `is_name` | 553 | 3 |  |
| parse.rs | `ident` | 557 | 9 |  |
| parse.rs | `word` | 568 | 6 |  |
| parse.rs | `local` | 576 | 6 |  |
| parse.rs | `is_keyword` | 582 | 3 |  |
| parse.rs | `eat_keyword` | 585 | 5 |  |
| parse.rs | `number` | 594 | 12 |  |
| parse.rs | `signed` | 607 | 16 |  |
| parse.rs | `numeral` | 624 | 12 |  |
| parse.rs | `values` | 637 | 8 |  |
| parse.rs | `value` | 647 | 10 |  |
| parse.rs | `value_inner` | 658 | 86 |  |
| parse.rs | `bound` | 749 | 20 |  |
| parse.rs | `set` | 773 | 7 |  |
| parse.rs | `set_inter` | 781 | 12 |  |
| parse.rs | `sets_in_parentheses` | 794 | 13 |  |
| parse.rs | `set_atom` | 807 | 220 |  |
| parse.rs | `variants` | 1050 | 70 |  |
| parse.rs | `both` | 1122 | 12 |  |
| parse.rs | `set_both` | 1135 | 4 |  |
| parse.rs | `binders` | 1140 | 33 |  |
| parse.rs | `args` | 1178 | 60 |  |
| parse.rs | `expr` | 1245 | 7 |  |
| parse.rs | `or` | 1252 | 7 |  |
| parse.rs | `and` | 1259 | 7 |  |
| parse.rs | `compare` | 1266 | 25 |  |
| parse.rs | `sum` | 1292 | 11 |  |
| parse.rs | `product` | 1304 | 11 |  |
| parse.rs | `unary` | 1315 | 42 |  |
| parse.rs | `method` | 1363 | 61 |  |
| parse.rs | `exprs` | 1425 | 11 |  |
| parse.rs | `block` | 1437 | 6 |  |
| parse.rs | `sequence` | 1444 | 16 |  |
| parse.rs | `bars` | 1462 | 11 |  |
| parse.rs | `primary` | 1473 | 275 |  |
| parse.rs | `trust` | 1763 | 10 |  |
| parse.rs | `state` | 1777 | 20 |  |
| parse.rs | `define` | 1798 | 13 |  |
| parse.rs | `declare` | 1812 | 11 |  |
| parse.rs | `type_params` | 1824 | 17 |  |
| parse.rs | `type_tokens` | 1842 | 15 |  |
| parse.rs | `enumeration` | 1859 | 35 |  |
| parse.rs | `items` | 1896 | 243 |  |
| parse.rs | `function` | 2154 | 117 |  |
| parse.rs | `sig` | 2281 | 11 |  |
| parse.rs | `inputs` | 2295 | 27 |  |
| parse.rs | `claim` | 2325 | 212 |  |
| parse.rs | `equivalence` | 2559 | 124 |  |
| parse.rs | `represented` | 2692 | 87 |  |
| parse.rs | `import` | 2783 | 78 |  |
| parse.rs | `instance` | 2871 | 92 |  |
| parse.rs | `evidence` | 2969 | 182 |  |
| parse.rs | `finish` | 3161 | 176 |  |
| parse.rs | `requirements` | 3353 | 140 |  |
| parse.rs | `reliance` | 3507 | 123 |  |
| parse.rs | `model_with` | 3655 | 3 |  |
| parse.rs | `declared` | 3661 | 30 |  |
| parse.rs | `types_of` | 3696 | 55 |  |
| parse.rs | `merge_types` | 3754 | 27 |  |
| parse.rs | `scope_types` | 3785 | 56 |  |
| parse.rs | `impls` | 3848 | 99 |  |
| parse.rs | `read` | 3950 | 35 |  |
| parse.rs | `evidence` | 3987 | 11 |  |
| parse.rs | `evidence_alone` | 4000 | 50 |  |
| parse.rs | `free` | 4057 | 60 |  |
| parse.rs | `unwritten` | 4119 | 6 |  |
| parse.rs | `records_in` | 4126 | 32 |  |
| parse.rs | `written` | 4161 | 20 |  |
| parse.rs | `stamp` | 4184 | 46 |  |
| parse.rs | `tied` | 4233 | 4 |  |
| plan.rs | `owed` | 79 | 158 |  |
| plan.rs | `point` | 243 | 6 |  |
| plan.rs | `subject_call` | 250 | 3 |  |
| plan.rs | `calls_no_subject` | 254 | 5 |  |
| plan.rs | `class_set` | 261 | 24 |  |
| plan.rs | `observed` | 287 | 8 |  |
| plan.rs | `set_text` | 297 | 11 |  |
| print.rs | `print_claim` | 8 | 3 |  |
| print.rs | `print_evidence` | 14 | 57 |  |
| print.rs | `pattern` | 73 | 19 |  |
| print.rs | `call` | 95 | 4 |  |
| print.rs | `expr` | 125 | 3 |  |
| print.rs | `contract` | 130 | 112 |  |
| print.rs | `at` | 246 | 99 |  |
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
| report.rs | `model` | 16 | 1 |  |
| report.rs | `evidence` | 17 | 1 |  |
| report.rs | `model` | 27 | 3 |  |
| report.rs | `evidence` | 30 | 3 |  |
| report.rs | `explain` | 183 | 4 |  |
| report.rs | `headline` | 188 | 3 |  |
| report.rs | `claim` | 214 | 3 |  |
| report.rs | `verdict` | 218 | 4 |  |
| report.rs | `at_claim` | 224 | 8 |  |
| report.rs | `evidence_text` | 233 | 16 |  |
| report.rs | `entry` | 252 | 8 |  |
| report.rs | `declared` | 263 | 5 |  |
| report.rs | `logic` | 270 | 4 |  |
| report.rs | `stated` | 276 | 6 |  |
| report.rs | `evidence_at` | 284 | 6 |  |
| report.rs | `role` | 291 | 16 |  |
| report.rs | `inputs` | 309 | 41 |  |
| report.rs | `missed` | 354 | 21 |  |
| report.rs | `verdicts` | 378 | 149 |  |
| report.rs | `quoted` | 531 | 13 |  |
| report.rs | `piece` | 546 | 109 |  |
| report.rs | `ranking` | 659 | 87 |  |
| report.rs | `error` | 752 | 226 |  |
| report.rs | `warnings` | 981 | 150 |  |
| report.rs | `to_text` | 1075 | 3 |  |
| report.rs | `to_text_with` | 1080 | 7 |  |
| report.rs | `to_text_at` | 1091 | 47 |  |
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
| set.rs | `interval` | 735 | 11 |  |
| set.rs | `union` | 747 | 19 |  |
| set.rs | `complement` | 768 | 23 |  |
| set.rs | `intersect` | 792 | 13 |  |
| set.rs | `complement_scalars` | 809 | 12 |  |
| set.rs | `minus` | 822 | 14 |  |
| set.rs | `is_empty` | 839 | 9 |  |
| set.rs | `is_subset` | 849 | 3 |  |
| set.rs | `contains` | 853 | 13 |  |
| set.rs | `shapes` | 868 | 3 |  |
| set.rs | `conforms` | 873 | 33 |  |
| set.rs | `values` | 909 | 29 |  |
| set.rs | `single` | 940 | 6 |  |
| set.rs | `count` | 948 | 30 |  |
| set.rs | `fmt` | 984 | 138 |  |
| set.rs | `n_is_set` | 1133 | 3 |  |
| set.rs | `paren` | 1154 | 8 |  |
| trust/culprits.rs | `weigh` | 87 | 51 |  |
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
| types.rs | `plain` | 45 | 4 |  |
| types.rs | `show` | 50 | 7 |  |
| types.rs | `unknown` | 58 | 4 |  |
| types.rs | `numbers` | 66 | 3 |  |
| types.rs | `needs` | 71 | 11 |  |
| types.rs | `call` | 84 | 8 |  |
| types.rs | `of` | 94 | 210 |  |

## mitra-store

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| source.rs | `read` | 10 | 3 | files::read |
| source.rs | `file` | 16 | 3 | files::file |
| store.rs | `of` | 45 | 6 | store::Store::of |
| store.rs | `model_path` | 74 | 3 | store::Store::model_path |
| store.rs | `source` | 82 | 12 | store::Store::source |
| store.rs | `evidence_path` | 96 | 3 | store::Store::evidence_path |
| store.rs | `add` | 103 | 20 | ledger::Store::add |
| store.rs | `append` | 139 | 35 | ledger::Store::append |
| source.rs | `files` | 22 | 38 |  |
| store.rs | `find` | 29 | 12 |  |
| store.rs | `init` | 53 | 14 |  |
| store.rs | `root` | 69 | 3 |  |
| store.rs | `model` | 125 | 8 |  |

## mitra-rust

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| report.rs | `short` | 21 | 4 | report::short |
| report.rs | `libtest` | 27 | 17 | report::libtest |
| report.rs | `attr` | 49 | 30 | report::attr |
| report.rs | `strip` | 82 | 11 | report::strip |
| report.rs | `junit` | 95 | 28 | report::junit |
| source.rs | `mitra_line` | 24 | 8 | source::mitra_line |
| source.rs | `fn_below` | 105 | 35 | source::fn_below |
| tie.rs | `tests` | 353 | 37 | ties::tests |
| tie.rs | `tie` | 392 | 249 | ties::tie |
| source.rs | `rust_files` | 6 | 14 |  |
| source.rs | `scan` | 45 | 55 |  |
| source.rs | `is_tie` | 143 | 3 |  |
| tie.rs | `shape_below` | 40 | 22 |  |
| tie.rs | `fields` | 64 | 18 |  |
| tie.rs | `balanced` | 84 | 12 |  |
| tie.rs | `pieces` | 99 | 20 |  |
| tie.rs | `as_model` | 121 | 45 |  |
| tie.rs | `fits` | 171 | 25 |  |
| tie.rs | `implemented` | 201 | 12 |  |
| tie.rs | `words` | 215 | 13 |  |
| tie.rs | `read` | 230 | 79 |  |
| tie.rs | `calls` | 314 | 8 |  |

## mitra-cli

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| main.rs | `check_rev` | 150 | 8 | commands::check_rev |
| main.rs | `main` | 38 | 17 |  |
| main.rs | `inputs` | 60 | 14 |  |
| main.rs | `read` | 78 | 3 |  |
| main.rs | `ties_only` | 85 | 26 |  |
| main.rs | `located` | 113 | 9 |  |
| main.rs | `relative` | 124 | 7 |  |
| main.rs | `model` | 139 | 3 |  |
| main.rs | `evidence` | 142 | 3 |  |
| main.rs | `evaluate` | 163 | 25 |  |
| main.rs | `take_rev` | 191 | 15 |  |
| main.rs | `take_value` | 208 | 11 |  |
| main.rs | `take_display` | 222 | 20 |  |
| main.rs | `run` | 244 | 3 |  |
| main.rs | `run_command` | 249 | 335 |  |
