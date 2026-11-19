# How much of Mitra its model covers

Written by `scripts/coverage.py --write`. A function is covered when a
`// mitra: implements …` comment ties it to the model. Tests are left out.

| Crate | Functions | Tied | Share | Lines in functions | In tied ones | Share | Code lines | Model lines | Model / code |
|---|---|---|---|---|---|---|---|---|---|
| mitra-lang | 381 | 38 | 10% | 12557 | 2686 | 21% | 13320 | 858 | 6% |
| mitra-store | 21 | 11 | 52% | 322 | 158 | 49% | 354 | 95 | 27% |
| mitra-rust | 19 | 6 | 32% | 784 | 435 | 55% | 824 | 53 | 6% |
| mitra-cli | 23 | 1 | 4% | 676 | 8 | 1% | 714 | 46 | 6% |
| all | 444 | 56 | 13% | 14339 | 3287 | 23% | 15212 | 1052 | 7% |

- mitra-lang: 858 lines of model over 13320 lines of code, in 18 subjects with 58 ensures and 8 requires; apart from it, 97 lines of point claims and 24 tests pinned in the code.
- mitra-store: 95 lines of model over 354 lines of code, in 4 subjects with 20 ensures and 8 requires; apart from it, 0 lines of point claims and 0 tests pinned in the code.
- mitra-rust: 53 lines of model over 824 lines of code, in 3 subjects with 8 ensures and 3 requires; apart from it, 28 lines of point claims and 0 tests pinned in the code.
- mitra-cli: 46 lines of model over 714 lines of code, in 1 subject with 9 ensures and 12 requires; apart from it, 0 lines of point claims and 4 tests pinned in the code.

## mitra-lang

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| ast.rs | `by_machine` | 256 | 3 | ledger::Evidence::by_machine |
| ast.rs | `block_of` | 384 | 6 | model::Model::block_of |
| ast.rs | `check` | 602 | 417 | types::Model::check |
| build.rs | `build` | 37 | 60 | build::build |
| build.rs | `lock` | 122 | 3 | build::Built::lock |
| build.rs | `count` | 128 | 3 | build::Built::count |
| build.rs | `interface` | 137 | 58 | build::interface |
| eval.rs | `as_str` | 29 | 8 | report::Logic::as_str |
| eval.rs | `unary` | 1495 | 12 | eval::unary |
| eval.rs | `binary` | 1510 | 63 | eval::binary |
| eval.rs | `evaluate` | 1828 | 1336 | verdicts::evaluate |
| fingerprint.rs | `hash` | 12 | 8 | fingerprint::hash |
| fingerprint.rs | `all` | 87 | 3 | fingerprint::all |
| fingerprint.rs | `of` | 94 | 9 | fingerprint::of |
| levels.rs | `left` | 24 | 27 | levels::left |
| lib.rs | `load` | 87 | 3 | types::load |
| lib.rs | `load_modules` | 94 | 7 | types::load_modules |
| lib.rs | `run` | 445 | 3 | verdicts::run |
| modules.rs | `alone` | 32 | 4 | modules::Modules::alone |
| modules.rs | `new` | 40 | 20 | modules::Modules::new |
| modules.rs | `text` | 69 | 3 | modules::Modules::text |
| modules.rs | `place` | 75 | 6 | modules::Modules::place |
| modules.rs | `locate` | 85 | 6 | modules::Modules::locate |
| parse.rs | `imports` | 185 | 26 | modules::imports |
| parse.rs | `model` | 3855 | 3 | reader::model |
| parse.rs | `entries` | 4235 | 49 | ledger::entries |
| plan.rs | `plan` | 17 | 272 | advice::plan |
| print.rs | `negative` | 104 | 7 | print::negative |
| print.rs | `level` | 118 | 9 | print::level |
| prove.rs | `derive` | 478 | 45 | verdicts::derive |
| report.rs | `word` | 218 | 11 | report::word |
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
| ast.rs | `machine` | 262 | 3 |  |
| ast.rs | `bare_ensures` | 377 | 4 |  |
| ast.rs | `spec_of_classes` | 422 | 22 |  |
| ast.rs | `conforms` | 448 | 4 |  |
| ast.rs | `wholes` | 456 | 14 |  |
| ast.rs | `named` | 474 | 22 |  |
| ast.rs | `named_recursive` | 500 | 16 |  |
| ast.rs | `record` | 516 | 3 |  |
| ast.rs | `function` | 520 | 4 |  |
| ast.rs | `ensure` | 525 | 3 |  |
| ast.rs | `captured` | 530 | 4 |  |
| ast.rs | `invariant` | 537 | 17 |  |
| ast.rs | `declared_function` | 556 | 3 |  |
| ast.rs | `scored` | 561 | 8 |  |
| ast.rs | `subjects_of` | 571 | 14 |  |
| ast.rs | `claim` | 585 | 3 |  |
| ast.rs | `covers` | 590 | 9 |  |
| ast.rs | `arities` | 1040 | 9 |  |
| ast.rs | `passed` | 1051 | 13 |  |
| ast.rs | `descents` | 1071 | 61 |  |
| ast.rs | `part` | 1072 | 7 |  |
| ast.rs | `walk` | 1134 | 26 |  |
| ast.rs | `arms` | 1162 | 21 |  |
| ast.rs | `calls` | 1185 | 21 |  |
| ast.rs | `single` | 1208 | 5 |  |
| build.rs | `hidden` | 100 | 5 |  |
| build.rs | `lines` | 108 | 11 |  |
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
| eval.rs | `warnings` | 3538 | 13 |  |
| eval.rs | `own_word` | 3557 | 3 |  |
| fingerprint.rs | `said` | 22 | 60 |  |
| levels.rs | `diagnosis` | 64 | 44 |  |
| lib.rs | `new` | 60 | 6 |  |
| lib.rs | `fmt` | 69 | 3 |  |
| lib.rs | `read` | 102 | 74 |  |
| lib.rs | `points` | 184 | 84 |  |
| lib.rs | `determinism` | 274 | 24 |  |
| lib.rs | `invariants` | 302 | 40 |  |
| lib.rs | `tie` | 344 | 16 |  |
| lib.rs | `tie_uses` | 363 | 23 |  |
| lib.rs | `tie_records` | 389 | 45 |  |
| lib.rs | `read_evidence` | 438 | 4 |  |
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
| parse.rs | `lex` | 258 | 67 |  |
| parse.rs | `earlier` | 330 | 31 |  |
| parse.rs | `unquote` | 455 | 15 |  |
| parse.rs | `join` | 472 | 4 |  |
| parse.rs | `bin` | 581 | 3 |  |
| parse.rs | `line` | 586 | 4 |  |
| parse.rs | `err` | 590 | 3 |  |
| parse.rs | `peek` | 593 | 3 |  |
| parse.rs | `peek_at` | 596 | 3 |  |
| parse.rs | `kind` | 599 | 3 |  |
| parse.rs | `kind_at` | 602 | 3 |  |
| parse.rs | `is` | 605 | 3 |  |
| parse.rs | `eat` | 608 | 5 |  |
| parse.rs | `expect` | 613 | 12 |  |
| parse.rs | `name` | 625 | 9 |  |
| parse.rs | `path` | 635 | 7 |  |
| parse.rs | `qualified` | 642 | 6 |  |
| parse.rs | `resolve` | 649 | 19 |  |
| parse.rs | `find` | 671 | 18 |  |
| parse.rs | `old` | 690 | 9 |  |
| parse.rs | `next_old` | 699 | 5 |  |
| parse.rs | `at_end` | 704 | 3 |  |
| parse.rs | `is_name` | 707 | 3 |  |
| parse.rs | `ident` | 711 | 9 |  |
| parse.rs | `word` | 722 | 6 |  |
| parse.rs | `local` | 730 | 6 |  |
| parse.rs | `is_keyword` | 736 | 3 |  |
| parse.rs | `eat_keyword` | 739 | 5 |  |
| parse.rs | `number` | 748 | 12 |  |
| parse.rs | `signed` | 761 | 16 |  |
| parse.rs | `numeral` | 778 | 12 |  |
| parse.rs | `values` | 791 | 8 |  |
| parse.rs | `value` | 801 | 10 |  |
| parse.rs | `value_inner` | 812 | 86 |  |
| parse.rs | `bound` | 903 | 20 |  |
| parse.rs | `set` | 927 | 7 |  |
| parse.rs | `set_inter` | 935 | 12 |  |
| parse.rs | `sets_in_parentheses` | 948 | 13 |  |
| parse.rs | `set_atom` | 961 | 220 |  |
| parse.rs | `variants` | 1204 | 70 |  |
| parse.rs | `both` | 1276 | 12 |  |
| parse.rs | `set_both` | 1289 | 4 |  |
| parse.rs | `binders` | 1294 | 33 |  |
| parse.rs | `args` | 1332 | 60 |  |
| parse.rs | `expr` | 1399 | 7 |  |
| parse.rs | `or` | 1406 | 7 |  |
| parse.rs | `and` | 1413 | 7 |  |
| parse.rs | `compare` | 1420 | 25 |  |
| parse.rs | `sum` | 1446 | 11 |  |
| parse.rs | `product` | 1458 | 11 |  |
| parse.rs | `unary` | 1469 | 42 |  |
| parse.rs | `method` | 1517 | 61 |  |
| parse.rs | `exprs` | 1579 | 11 |  |
| parse.rs | `block` | 1591 | 6 |  |
| parse.rs | `sequence` | 1598 | 16 |  |
| parse.rs | `bars` | 1616 | 11 |  |
| parse.rs | `primary` | 1627 | 275 |  |
| parse.rs | `trust` | 1917 | 10 |  |
| parse.rs | `state` | 1931 | 20 |  |
| parse.rs | `define` | 1952 | 13 |  |
| parse.rs | `declare` | 1966 | 12 |  |
| parse.rs | `type_params` | 1979 | 17 |  |
| parse.rs | `type_tokens` | 1997 | 15 |  |
| parse.rs | `enumeration` | 2014 | 35 |  |
| parse.rs | `items` | 2051 | 257 |  |
| parse.rs | `function` | 2323 | 117 |  |
| parse.rs | `sig` | 2450 | 11 |  |
| parse.rs | `inputs` | 2464 | 27 |  |
| parse.rs | `claim` | 2494 | 212 |  |
| parse.rs | `equivalence` | 2728 | 124 |  |
| parse.rs | `represented` | 2861 | 87 |  |
| parse.rs | `import` | 2952 | 109 |  |
| parse.rs | `instance` | 3073 | 92 |  |
| parse.rs | `evidence` | 3171 | 185 |  |
| parse.rs | `finish` | 3366 | 176 |  |
| parse.rs | `requirements` | 3558 | 140 |  |
| parse.rs | `reliance` | 3712 | 123 |  |
| parse.rs | `model_with` | 3860 | 3 |  |
| parse.rs | `declared` | 3866 | 30 |  |
| parse.rs | `types_of` | 3901 | 55 |  |
| parse.rs | `merge_types` | 3959 | 27 |  |
| parse.rs | `scope_types` | 3990 | 56 |  |
| parse.rs | `impls` | 4053 | 99 |  |
| parse.rs | `read` | 4155 | 54 |  |
| parse.rs | `evidence` | 4213 | 12 |  |
| parse.rs | `evidence_alone` | 4227 | 3 |  |
| parse.rs | `blocks` | 4288 | 84 |  |
| parse.rs | `free` | 4381 | 60 |  |
| parse.rs | `unwritten` | 4443 | 6 |  |
| parse.rs | `records_in` | 4450 | 32 |  |
| parse.rs | `written` | 4485 | 20 |  |
| parse.rs | `stamp` | 4508 | 59 |  |
| parse.rs | `tied` | 4571 | 4 |  |
| plan.rs | `owed` | 79 | 158 |  |
| plan.rs | `point` | 243 | 6 |  |
| plan.rs | `subject_call` | 250 | 3 |  |
| plan.rs | `calls_no_subject` | 254 | 5 |  |
| plan.rs | `class_set` | 261 | 24 |  |
| plan.rs | `observed` | 287 | 8 |  |
| plan.rs | `set_text` | 297 | 11 |  |
| print.rs | `print_claim` | 8 | 3 |  |
| print.rs | `print_evidence` | 14 | 60 |  |
| print.rs | `pattern` | 76 | 19 |  |
| print.rs | `call` | 98 | 4 |  |
| print.rs | `expr` | 128 | 3 |  |
| print.rs | `contract` | 133 | 112 |  |
| print.rs | `at` | 249 | 99 |  |
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
| report.rs | `explain` | 207 | 4 |  |
| report.rs | `headline` | 212 | 3 |  |
| report.rs | `claim` | 238 | 3 |  |
| report.rs | `verdict` | 242 | 4 |  |
| report.rs | `at_claim` | 248 | 8 |  |
| report.rs | `evidence_text` | 257 | 16 |  |
| report.rs | `entry` | 276 | 8 |  |
| report.rs | `declared` | 287 | 5 |  |
| report.rs | `logic` | 294 | 4 |  |
| report.rs | `stated` | 300 | 6 |  |
| report.rs | `evidence_at` | 308 | 6 |  |
| report.rs | `role` | 315 | 16 |  |
| report.rs | `inputs` | 333 | 41 |  |
| report.rs | `missed` | 378 | 21 |  |
| report.rs | `verdicts` | 402 | 149 |  |
| report.rs | `quoted` | 555 | 13 |  |
| report.rs | `piece` | 570 | 109 |  |
| report.rs | `ranking` | 683 | 87 |  |
| report.rs | `error` | 776 | 229 |  |
| report.rs | `lints` | 1009 | 13 |  |
| report.rs | `warnings` | 1023 | 208 |  |
| report.rs | `to_text` | 1136 | 3 |  |
| report.rs | `to_text_with` | 1141 | 7 |  |
| report.rs | `model_text_at` | 1152 | 20 |  |
| report.rs | `problem_texts` | 1174 | 16 |  |
| report.rs | `model_warnings` | 1192 | 3 |  |
| report.rs | `to_text_at` | 1199 | 47 |  |
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
| store.rs | `of` | 86 | 6 | store::Store::of |
| store.rs | `model_path` | 115 | 3 | store::Store::model_path |
| store.rs | `source` | 123 | 12 | store::Store::source |
| store.rs | `evidence_path` | 137 | 3 | store::Store::evidence_path |
| store.rs | `add` | 144 | 16 | ledger::Store::add |
| store.rs | `write_build` | 228 | 9 | store::Store::write_build |
| store.rs | `replace` | 243 | 41 | keeping::Store::replace |
| store.rs | `remove` | 290 | 27 | keeping::Store::remove |
| store.rs | `append` | 333 | 35 | ledger::Store::append |
| source.rs | `files` | 22 | 42 |  |
| source.rs | `other_store` | 69 | 24 |  |
| store.rs | `said` | 48 | 8 |  |
| store.rs | `find` | 70 | 12 |  |
| store.rs | `init` | 94 | 14 |  |
| store.rs | `root` | 110 | 3 |  |
| store.rs | `stamped` | 165 | 25 |  |
| store.rs | `without` | 192 | 15 |  |
| store.rs | `rewrite` | 211 | 13 |  |
| store.rs | `model` | 319 | 8 |  |

## mitra-rust

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| report.rs | `libtest` | 11 | 17 | report::libtest |
| source.rs | `mitra_line` | 24 | 8 | source::mitra_line |
| source.rs | `fn_below` | 105 | 35 | source::fn_below |
| source.rs | `is_tie` | 144 | 3 | source::is_tie |
| tie.rs | `tests` | 374 | 37 | ties::tests |
| tie.rs | `tie` | 413 | 335 | ties::tie |
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
| main.rs | `check_rev` | 378 | 8 | commands::check_rev |
| main.rs | `build` | 36 | 53 |  |
| main.rs | `conform` | 92 | 47 |  |
| main.rs | `assess` | 142 | 36 |  |
| main.rs | `diagnose` | 182 | 28 |  |
| main.rs | `in_place` | 212 | 7 |  |
| main.rs | `shown` | 221 | 8 |  |
| main.rs | `main` | 248 | 17 |  |
| main.rs | `inputs` | 270 | 14 |  |
| main.rs | `read` | 288 | 3 |  |
| main.rs | `ties_only` | 295 | 26 |  |
| main.rs | `located` | 323 | 9 |  |
| main.rs | `relative` | 334 | 7 |  |
| main.rs | `model` | 349 | 3 |  |
| main.rs | `evidence` | 352 | 3 |  |
| main.rs | `model` | 364 | 3 |  |
| main.rs | `evidence` | 367 | 6 |  |
| main.rs | `evaluate` | 391 | 25 |  |
| main.rs | `take_rev` | 419 | 18 |  |
| main.rs | `take_value` | 439 | 11 |  |
| main.rs | `take_display` | 453 | 20 |  |
| main.rs | `run` | 475 | 3 |  |
| main.rs | `run_command` | 480 | 321 |  |
