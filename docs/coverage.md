# How much of Mitra its model covers

Written by `scripts/coverage.py --write`. A function is covered when a
`// mitra: implements …` comment ties it to the model. Tests are left out.

| Crate | Functions | Tied | Share | Lines in functions | In tied ones | Share | Code lines | Model lines | Model / code |
|---|---|---|---|---|---|---|---|---|---|
| mitra-lang | 360 | 28 | 8% | 12013 | 2463 | 21% | 12728 | 672 | 5% |
| mitra-store | 14 | 8 | 57% | 188 | 85 | 45% | 206 | 72 | 35% |
| mitra-rust | 19 | 6 | 32% | 776 | 427 | 55% | 814 | 51 | 6% |
| mitra-cli | 15 | 1 | 7% | 418 | 8 | 2% | 440 | 37 | 8% |
| all | 408 | 43 | 11% | 13395 | 2983 | 22% | 14188 | 832 | 6% |

- mitra-lang: 672 lines of model over 12728 lines of code, in 15 subjects with 46 ensures and 6 requires; apart from it, 97 lines of point claims and 22 tests pinned in the code.
- mitra-store: 72 lines of model over 206 lines of code, in 4 subjects with 18 ensures and 7 requires; apart from it, 0 lines of point claims and 0 tests pinned in the code.
- mitra-rust: 51 lines of model over 814 lines of code, in 3 subjects with 8 ensures and 3 requires; apart from it, 28 lines of point claims and 0 tests pinned in the code.
- mitra-cli: 37 lines of model over 440 lines of code, in 1 subject with 6 ensures and 9 requires; apart from it, 0 lines of point claims and 3 tests pinned in the code.

## mitra-lang

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| ast.rs | `block_of` | 336 | 6 | model::Model::block_of |
| ast.rs | `check` | 554 | 417 | types::Model::check |
| eval.rs | `as_str` | 29 | 8 | report::Logic::as_str |
| eval.rs | `unary` | 1495 | 12 | eval::unary |
| eval.rs | `binary` | 1510 | 63 | eval::binary |
| eval.rs | `evaluate` | 1828 | 1336 | verdicts::evaluate |
| lib.rs | `load` | 84 | 3 | types::load |
| lib.rs | `load_modules` | 91 | 7 | types::load_modules |
| lib.rs | `run` | 416 | 3 | verdicts::run |
| modules.rs | `alone` | 32 | 4 | modules::Modules::alone |
| modules.rs | `new` | 40 | 20 | modules::Modules::new |
| modules.rs | `text` | 69 | 3 | modules::Modules::text |
| modules.rs | `place` | 75 | 6 | modules::Modules::place |
| modules.rs | `locate` | 85 | 6 | modules::Modules::locate |
| parse.rs | `imports` | 185 | 26 | modules::imports |
| parse.rs | `model` | 3834 | 3 | reader::model |
| plan.rs | `plan` | 17 | 272 | advice::plan |
| print.rs | `negative` | 101 | 7 | print::negative |
| print.rs | `level` | 115 | 9 | print::level |
| prove.rs | `derive` | 478 | 45 | verdicts::derive |
| report.rs | `word` | 194 | 11 | report::word |
| results.rs | `from_tests` | 62 | 88 | results::from_tests |
| results.rs | `short` | 160 | 4 | results::short |
| results.rs | `attr` | 168 | 30 | results::attr |
| results.rs | `strip` | 201 | 11 | results::strip |
| results.rs | `junit` | 214 | 28 | results::junit |
| set.rs | `quote` | 1139 | 13 | print::quote |
| trust/culprits.rs | `rank` | 59 | 22 | culprits::rank |
| ast.rs | `set` | 93 | 6 |  |
| ast.rs | `is_spec` | 119 | 3 |  |
| ast.rs | `domain` | 123 | 3 |  |
| ast.rs | `is_structural` | 174 | 6 |  |
| ast.rs | `spec_of_classes` | 374 | 22 |  |
| ast.rs | `conforms` | 400 | 4 |  |
| ast.rs | `wholes` | 408 | 14 |  |
| ast.rs | `named` | 426 | 22 |  |
| ast.rs | `named_recursive` | 452 | 16 |  |
| ast.rs | `record` | 468 | 3 |  |
| ast.rs | `function` | 472 | 4 |  |
| ast.rs | `ensure` | 477 | 3 |  |
| ast.rs | `captured` | 482 | 4 |  |
| ast.rs | `invariant` | 489 | 17 |  |
| ast.rs | `declared_function` | 508 | 3 |  |
| ast.rs | `scored` | 513 | 8 |  |
| ast.rs | `subjects_of` | 523 | 14 |  |
| ast.rs | `claim` | 537 | 3 |  |
| ast.rs | `covers` | 542 | 9 |  |
| ast.rs | `arities` | 992 | 9 |  |
| ast.rs | `passed` | 1003 | 13 |  |
| ast.rs | `descents` | 1023 | 61 |  |
| ast.rs | `part` | 1024 | 7 |  |
| ast.rs | `walk` | 1086 | 26 |  |
| ast.rs | `arms` | 1114 | 21 |  |
| ast.rs | `calls` | 1137 | 21 |  |
| ast.rs | `single` | 1160 | 5 |  |
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
| eval.rs | `represented` | 3306 | 23 |  |
| eval.rs | `computed` | 3332 | 22 |  |
| eval.rs | `rank` | 3357 | 146 |  |
| eval.rs | `brief` | 3513 | 14 |  |
| eval.rs | `warnings` | 3538 | 10 |  |
| eval.rs | `own_word` | 3554 | 3 |  |
| lib.rs | `new` | 57 | 6 |  |
| lib.rs | `fmt` | 66 | 3 |  |
| lib.rs | `read` | 99 | 49 |  |
| lib.rs | `points` | 155 | 84 |  |
| lib.rs | `determinism` | 245 | 24 |  |
| lib.rs | `invariants` | 273 | 40 |  |
| lib.rs | `tie` | 315 | 16 |  |
| lib.rs | `tie_uses` | 334 | 23 |  |
| lib.rs | `tie_records` | 360 | 45 |  |
| lib.rs | `read_evidence` | 409 | 4 |  |
| modules.rs | `files` | 63 | 3 |  |
| modules.rs | `first` | 92 | 3 |  |
| modules.rs | `in_order` | 100 | 50 |  |
| next.rs | `next` | 16 | 385 |  |
| next.rs | `ask` | 391 | 17 |  |
| next.rs | `vouch` | 410 | 4 |  |
| parse.rs | `library` | 34 | 3 |  |
| parse.rs | `module` | 39 | 15 |  |
| parse.rs | `from_store` | 59 | 74 |  |
| parse.rs | `imports_of` | 140 | 31 |  |
| parse.rs | `of_store` | 175 | 6 |  |
| parse.rs | `lex` | 241 | 67 |  |
| parse.rs | `earlier` | 313 | 31 |  |
| parse.rs | `unquote` | 438 | 15 |  |
| parse.rs | `join` | 455 | 4 |  |
| parse.rs | `bin` | 564 | 3 |  |
| parse.rs | `line` | 569 | 4 |  |
| parse.rs | `err` | 573 | 3 |  |
| parse.rs | `peek` | 576 | 3 |  |
| parse.rs | `peek_at` | 579 | 3 |  |
| parse.rs | `kind` | 582 | 3 |  |
| parse.rs | `kind_at` | 585 | 3 |  |
| parse.rs | `is` | 588 | 3 |  |
| parse.rs | `eat` | 591 | 5 |  |
| parse.rs | `expect` | 596 | 12 |  |
| parse.rs | `name` | 608 | 9 |  |
| parse.rs | `path` | 618 | 7 |  |
| parse.rs | `qualified` | 625 | 6 |  |
| parse.rs | `resolve` | 632 | 19 |  |
| parse.rs | `find` | 654 | 18 |  |
| parse.rs | `old` | 673 | 9 |  |
| parse.rs | `next_old` | 682 | 5 |  |
| parse.rs | `at_end` | 687 | 3 |  |
| parse.rs | `is_name` | 690 | 3 |  |
| parse.rs | `ident` | 694 | 9 |  |
| parse.rs | `word` | 705 | 6 |  |
| parse.rs | `local` | 713 | 6 |  |
| parse.rs | `is_keyword` | 719 | 3 |  |
| parse.rs | `eat_keyword` | 722 | 5 |  |
| parse.rs | `number` | 731 | 12 |  |
| parse.rs | `signed` | 744 | 16 |  |
| parse.rs | `numeral` | 761 | 12 |  |
| parse.rs | `values` | 774 | 8 |  |
| parse.rs | `value` | 784 | 10 |  |
| parse.rs | `value_inner` | 795 | 86 |  |
| parse.rs | `bound` | 886 | 20 |  |
| parse.rs | `set` | 910 | 7 |  |
| parse.rs | `set_inter` | 918 | 12 |  |
| parse.rs | `sets_in_parentheses` | 931 | 13 |  |
| parse.rs | `set_atom` | 944 | 220 |  |
| parse.rs | `variants` | 1187 | 70 |  |
| parse.rs | `both` | 1259 | 12 |  |
| parse.rs | `set_both` | 1272 | 4 |  |
| parse.rs | `binders` | 1277 | 33 |  |
| parse.rs | `args` | 1315 | 60 |  |
| parse.rs | `expr` | 1382 | 7 |  |
| parse.rs | `or` | 1389 | 7 |  |
| parse.rs | `and` | 1396 | 7 |  |
| parse.rs | `compare` | 1403 | 25 |  |
| parse.rs | `sum` | 1429 | 11 |  |
| parse.rs | `product` | 1441 | 11 |  |
| parse.rs | `unary` | 1452 | 42 |  |
| parse.rs | `method` | 1500 | 61 |  |
| parse.rs | `exprs` | 1562 | 11 |  |
| parse.rs | `block` | 1574 | 6 |  |
| parse.rs | `sequence` | 1581 | 16 |  |
| parse.rs | `bars` | 1599 | 11 |  |
| parse.rs | `primary` | 1610 | 275 |  |
| parse.rs | `trust` | 1900 | 10 |  |
| parse.rs | `state` | 1914 | 20 |  |
| parse.rs | `define` | 1935 | 13 |  |
| parse.rs | `declare` | 1949 | 11 |  |
| parse.rs | `type_params` | 1961 | 17 |  |
| parse.rs | `type_tokens` | 1979 | 15 |  |
| parse.rs | `enumeration` | 1996 | 35 |  |
| parse.rs | `items` | 2033 | 257 |  |
| parse.rs | `function` | 2305 | 117 |  |
| parse.rs | `sig` | 2432 | 11 |  |
| parse.rs | `inputs` | 2446 | 27 |  |
| parse.rs | `claim` | 2476 | 212 |  |
| parse.rs | `equivalence` | 2710 | 124 |  |
| parse.rs | `represented` | 2843 | 87 |  |
| parse.rs | `import` | 2934 | 109 |  |
| parse.rs | `instance` | 3055 | 92 |  |
| parse.rs | `evidence` | 3153 | 182 |  |
| parse.rs | `finish` | 3345 | 176 |  |
| parse.rs | `requirements` | 3537 | 140 |  |
| parse.rs | `reliance` | 3691 | 123 |  |
| parse.rs | `model_with` | 3839 | 3 |  |
| parse.rs | `declared` | 3845 | 30 |  |
| parse.rs | `types_of` | 3880 | 55 |  |
| parse.rs | `merge_types` | 3938 | 27 |  |
| parse.rs | `scope_types` | 3969 | 56 |  |
| parse.rs | `impls` | 4032 | 99 |  |
| parse.rs | `read` | 4134 | 54 |  |
| parse.rs | `evidence` | 4191 | 11 |  |
| parse.rs | `evidence_alone` | 4204 | 55 |  |
| parse.rs | `free` | 4267 | 60 |  |
| parse.rs | `unwritten` | 4329 | 6 |  |
| parse.rs | `records_in` | 4336 | 32 |  |
| parse.rs | `written` | 4371 | 20 |  |
| parse.rs | `stamp` | 4394 | 46 |  |
| parse.rs | `tied` | 4443 | 4 |  |
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
| source.rs | `files` | 22 | 42 |  |
| source.rs | `other_store` | 69 | 24 |  |
| store.rs | `find` | 29 | 12 |  |
| store.rs | `init` | 53 | 14 |  |
| store.rs | `root` | 69 | 3 |  |
| store.rs | `model` | 125 | 8 |  |

## mitra-rust

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| report.rs | `libtest` | 11 | 17 | report::libtest |
| source.rs | `mitra_line` | 24 | 8 | source::mitra_line |
| source.rs | `fn_below` | 105 | 35 | source::fn_below |
| source.rs | `is_tie` | 144 | 3 | source::is_tie |
| tie.rs | `tests` | 369 | 37 | ties::tests |
| tie.rs | `tie` | 408 | 327 | ties::tie |
| source.rs | `rust_files` | 6 | 14 |  |
| source.rs | `scan` | 45 | 55 |  |
| tie.rs | `variants` | 42 | 25 |  |
| tie.rs | `shape_below` | 70 | 22 |  |
| tie.rs | `fields` | 94 | 18 |  |
| tie.rs | `balanced` | 114 | 12 |  |
| tie.rs | `pieces` | 129 | 20 |  |
| tie.rs | `as_model` | 151 | 45 |  |
| tie.rs | `fits` | 201 | 25 |  |
| tie.rs | `implemented` | 231 | 12 |  |
| tie.rs | `words` | 245 | 13 |  |
| tie.rs | `read` | 260 | 80 |  |
| tie.rs | `calls` | 345 | 8 |  |

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
| main.rs | `run_command` | 249 | 254 |  |
