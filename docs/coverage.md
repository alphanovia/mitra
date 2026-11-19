# How much of Mitra its model covers

Written by `scripts/coverage.py --write`. A function is covered when a
`// mitra: implements …` comment ties it to the model. Tests are left out.

| Crate | Functions | Tied | Share | Lines in functions | In tied ones | Share | Code lines | Model lines | Model / code |
|---|---|---|---|---|---|---|---|---|---|
| mitra-lang | 405 | 41 | 10% | 13836 | 2708 | 20% | 14647 | 924 | 6% |
| mitra-store | 21 | 11 | 52% | 322 | 158 | 49% | 354 | 95 | 27% |
| mitra-rust | 23 | 7 | 30% | 937 | 533 | 57% | 975 | 42 | 4% |
| mitra-cli | 23 | 2 | 9% | 658 | 60 | 9% | 697 | 56 | 8% |
| all | 472 | 61 | 13% | 15753 | 3459 | 22% | 16673 | 1117 | 7% |

- mitra-lang: 924 lines of model over 14647 lines of code, in 20 subjects with 71 ensures and 8 requires; apart from it, 97 lines of point claims and 24 tests pinned in the code.
- mitra-store: 95 lines of model over 354 lines of code, in 4 subjects with 20 ensures and 8 requires; apart from it, 0 lines of point claims and 0 tests pinned in the code.
- mitra-rust: 42 lines of model over 975 lines of code, in 2 subjects with 3 ensures and 3 requires; apart from it, 28 lines of point claims and 0 tests pinned in the code.
- mitra-cli: 56 lines of model over 697 lines of code, in 1 subject with 10 ensures and 18 requires; apart from it, 0 lines of point claims and 4 tests pinned in the code.

## mitra-lang

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| ast.rs | `by_machine` | 256 | 3 | ledger::Evidence::by_machine |
| ast.rs | `block_of` | 410 | 6 | model::Model::block_of |
| ast.rs | `check` | 628 | 417 | types::Model::check |
| build.rs | `build` | 37 | 61 | build::build |
| build.rs | `lock` | 123 | 3 | build::Built::lock |
| build.rs | `count` | 129 | 3 | build::Built::count |
| build.rs | `interface` | 138 | 76 | build::interface |
| eval.rs | `as_str` | 29 | 8 | report::Logic::as_str |
| eval.rs | `unary` | 1495 | 12 | eval::unary |
| eval.rs | `binary` | 1510 | 63 | eval::binary |
| eval.rs | `evaluate` | 1828 | 1336 | verdicts::evaluate |
| fingerprint.rs | `hash` | 12 | 8 | fingerprint::hash |
| fingerprint.rs | `all` | 87 | 3 | fingerprint::all |
| fingerprint.rs | `of` | 94 | 9 | fingerprint::of |
| frontend.rs | `results` | 30 | 1 | Reader::results |
| frontend.rs | `tests` | 34 | 1 | Reader::tests |
| frontend.rs | `tie` | 38 | 1 | Reader::tie |
| levels.rs | `left` | 24 | 27 | levels::left |
| lib.rs | `load` | 90 | 3 | types::load |
| lib.rs | `load_modules` | 97 | 7 | types::load_modules |
| lib.rs | `run` | 448 | 3 | verdicts::run |
| modules.rs | `alone` | 32 | 4 | modules::Modules::alone |
| modules.rs | `new` | 40 | 20 | modules::Modules::new |
| modules.rs | `text` | 69 | 3 | modules::Modules::text |
| modules.rs | `place` | 75 | 6 | modules::Modules::place |
| modules.rs | `locate` | 85 | 6 | modules::Modules::locate |
| parse.rs | `imports` | 185 | 26 | modules::imports |
| parse.rs | `model` | 4144 | 3 | reader::model |
| parse.rs | `entries` | 5361 | 49 | ledger::entries |
| plan.rs | `plan` | 17 | 272 | advice::plan |
| print.rs | `negative` | 104 | 7 | print::negative |
| print.rs | `level` | 118 | 9 | print::level |
| prove.rs | `derive` | 478 | 45 | verdicts::derive |
| report.rs | `word` | 219 | 11 | report::word |
| results.rs | `from_tests` | 62 | 88 | results::from_tests |
| results.rs | `short` | 160 | 4 | results::short |
| results.rs | `attr` | 168 | 30 | results::attr |
| results.rs | `strip` | 201 | 11 | results::strip |
| results.rs | `junit` | 214 | 28 | results::junit |
| set.rs | `quote` | 1146 | 13 | print::quote |
| trust/culprits.rs | `rank` | 59 | 22 | culprits::rank |
| ast.rs | `set` | 93 | 6 |  |
| ast.rs | `is_spec` | 119 | 3 |  |
| ast.rs | `domain` | 123 | 3 |  |
| ast.rs | `is_structural` | 174 | 6 |  |
| ast.rs | `machine` | 262 | 3 |  |
| ast.rs | `bare_ensures` | 403 | 4 |  |
| ast.rs | `spec_of_classes` | 448 | 22 |  |
| ast.rs | `conforms` | 474 | 4 |  |
| ast.rs | `wholes` | 482 | 14 |  |
| ast.rs | `named` | 500 | 22 |  |
| ast.rs | `named_recursive` | 526 | 16 |  |
| ast.rs | `record` | 542 | 3 |  |
| ast.rs | `function` | 546 | 4 |  |
| ast.rs | `ensure` | 551 | 3 |  |
| ast.rs | `captured` | 556 | 4 |  |
| ast.rs | `invariant` | 563 | 17 |  |
| ast.rs | `declared_function` | 582 | 3 |  |
| ast.rs | `scored` | 587 | 8 |  |
| ast.rs | `subjects_of` | 597 | 14 |  |
| ast.rs | `claim` | 611 | 3 |  |
| ast.rs | `covers` | 616 | 9 |  |
| ast.rs | `arities` | 1066 | 9 |  |
| ast.rs | `passed` | 1077 | 13 |  |
| ast.rs | `descents` | 1097 | 61 |  |
| ast.rs | `part` | 1098 | 7 |  |
| ast.rs | `walk` | 1160 | 26 |  |
| ast.rs | `arms` | 1188 | 21 |  |
| ast.rs | `calls` | 1211 | 21 |  |
| ast.rs | `single` | 1234 | 5 |  |
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
| eval.rs | `represented` | 3306 | 23 |  |
| eval.rs | `computed` | 3332 | 22 |  |
| eval.rs | `rank` | 3357 | 146 |  |
| eval.rs | `brief` | 3513 | 14 |  |
| eval.rs | `warnings` | 3538 | 13 |  |
| eval.rs | `own_word` | 3557 | 3 |  |
| fingerprint.rs | `said` | 22 | 60 |  |
| instances.rs | `pieces` | 12 | 19 |  |
| instances.rs | `within` | 34 | 15 |  |
| instances.rs | `type_of` | 51 | 23 |  |
| instances.rs | `given` | 77 | 19 |  |
| instances.rs | `unify` | 99 | 62 |  |
| instances.rs | `seen` | 167 | 42 |  |
| levels.rs | `diagnosis` | 64 | 44 |  |
| lib.rs | `new` | 63 | 6 |  |
| lib.rs | `fmt` | 72 | 3 |  |
| lib.rs | `read` | 105 | 74 |  |
| lib.rs | `points` | 187 | 84 |  |
| lib.rs | `determinism` | 277 | 24 |  |
| lib.rs | `invariants` | 305 | 40 |  |
| lib.rs | `tie` | 347 | 16 |  |
| lib.rs | `tie_uses` | 366 | 23 |  |
| lib.rs | `tie_records` | 392 | 45 |  |
| lib.rs | `read_evidence` | 441 | 4 |  |
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
| parse.rs | `tidy` | 479 | 18 |  |
| parse.rs | `bin` | 609 | 3 |  |
| parse.rs | `line` | 614 | 4 |  |
| parse.rs | `err` | 618 | 3 |  |
| parse.rs | `peek` | 621 | 3 |  |
| parse.rs | `peek_at` | 624 | 3 |  |
| parse.rs | `kind` | 627 | 3 |  |
| parse.rs | `kind_at` | 630 | 3 |  |
| parse.rs | `is` | 633 | 3 |  |
| parse.rs | `eat` | 636 | 5 |  |
| parse.rs | `expect` | 641 | 12 |  |
| parse.rs | `name` | 653 | 9 |  |
| parse.rs | `path` | 663 | 7 |  |
| parse.rs | `qualified` | 670 | 6 |  |
| parse.rs | `resolve` | 677 | 19 |  |
| parse.rs | `find` | 699 | 18 |  |
| parse.rs | `old` | 718 | 9 |  |
| parse.rs | `next_old` | 727 | 5 |  |
| parse.rs | `at_end` | 732 | 3 |  |
| parse.rs | `is_name` | 735 | 3 |  |
| parse.rs | `ident` | 739 | 9 |  |
| parse.rs | `word` | 750 | 6 |  |
| parse.rs | `local` | 758 | 6 |  |
| parse.rs | `is_keyword` | 764 | 3 |  |
| parse.rs | `eat_keyword` | 767 | 5 |  |
| parse.rs | `number` | 776 | 12 |  |
| parse.rs | `signed` | 789 | 16 |  |
| parse.rs | `numeral` | 806 | 12 |  |
| parse.rs | `values` | 819 | 8 |  |
| parse.rs | `value` | 829 | 10 |  |
| parse.rs | `value_inner` | 840 | 86 |  |
| parse.rs | `bound` | 931 | 20 |  |
| parse.rs | `set` | 955 | 7 |  |
| parse.rs | `set_inter` | 963 | 12 |  |
| parse.rs | `sets_in_parentheses` | 976 | 13 |  |
| parse.rs | `set_atom` | 989 | 220 |  |
| parse.rs | `variants` | 1232 | 70 |  |
| parse.rs | `both` | 1304 | 12 |  |
| parse.rs | `set_both` | 1317 | 4 |  |
| parse.rs | `binders` | 1322 | 33 |  |
| parse.rs | `args` | 1360 | 60 |  |
| parse.rs | `expr` | 1427 | 7 |  |
| parse.rs | `or` | 1434 | 7 |  |
| parse.rs | `and` | 1441 | 7 |  |
| parse.rs | `compare` | 1448 | 25 |  |
| parse.rs | `sum` | 1474 | 11 |  |
| parse.rs | `product` | 1486 | 11 |  |
| parse.rs | `unary` | 1497 | 42 |  |
| parse.rs | `method` | 1545 | 86 |  |
| parse.rs | `exprs` | 1636 | 11 |  |
| parse.rs | `block` | 1648 | 6 |  |
| parse.rs | `sequence` | 1655 | 16 |  |
| parse.rs | `bars` | 1673 | 11 |  |
| parse.rs | `primary` | 1684 | 275 |  |
| parse.rs | `trust` | 1974 | 10 |  |
| parse.rs | `state` | 1988 | 20 |  |
| parse.rs | `define` | 2009 | 13 |  |
| parse.rs | `declare` | 2023 | 12 |  |
| parse.rs | `type_params` | 2036 | 22 |  |
| parse.rs | `bounded_by` | 2059 | 23 |  |
| parse.rs | `unbounded` | 2084 | 8 |  |
| parse.rs | `type_tokens` | 2093 | 15 |  |
| parse.rs | `enumeration` | 2110 | 35 |  |
| parse.rs | `items` | 2147 | 267 |  |
| parse.rs | `function` | 2429 | 213 |  |
| parse.rs | `sig` | 2660 | 11 |  |
| parse.rs | `inputs` | 2674 | 27 |  |
| parse.rs | `claim` | 2704 | 237 |  |
| parse.rs | `equivalence` | 2964 | 124 |  |
| parse.rs | `represented` | 3097 | 87 |  |
| parse.rs | `import` | 3188 | 109 |  |
| parse.rs | `instance` | 3309 | 92 |  |
| parse.rs | `evidence` | 3407 | 185 |  |
| parse.rs | `finish` | 3602 | 176 |  |
| parse.rs | `requirements` | 3794 | 144 |  |
| parse.rs | `reliance` | 3954 | 166 |  |
| parse.rs | `model_with` | 4149 | 3 |  |
| parse.rs | `declared` | 4155 | 30 |  |
| parse.rs | `types_of` | 4190 | 55 |  |
| parse.rs | `merge_types` | 4248 | 27 |  |
| parse.rs | `scope_types` | 4279 | 56 |  |
| parse.rs | `angle` | 4339 | 17 |  |
| parse.rs | `param_names` | 4359 | 18 |  |
| parse.rs | `type_args` | 4379 | 18 |  |
| parse.rs | `with_params` | 4400 | 24 |  |
| parse.rs | `impls` | 4430 | 190 |  |
| parse.rs | `closing` | 4646 | 16 |  |
| parse.rs | `impl_types` | 4666 | 90 |  |
| parse.rs | `given_types` | 4762 | 18 |  |
| parse.rs | `contracts` | 4788 | 348 |  |
| parse.rs | `bring_contracts` | 5150 | 118 |  |
| parse.rs | `read` | 5275 | 60 |  |
| parse.rs | `evidence` | 5339 | 12 |  |
| parse.rs | `evidence_alone` | 5353 | 3 |  |
| parse.rs | `blocks` | 5414 | 84 |  |
| parse.rs | `free` | 5507 | 60 |  |
| parse.rs | `unwritten` | 5569 | 6 |  |
| parse.rs | `records_in` | 5576 | 32 |  |
| parse.rs | `written` | 5611 | 20 |  |
| parse.rs | `stamp` | 5634 | 59 |  |
| parse.rs | `tied` | 5697 | 4 |  |
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
| report.rs | `verdicts` | 403 | 161 |  |
| report.rs | `quoted` | 570 | 13 |  |
| report.rs | `piece` | 585 | 109 |  |
| report.rs | `ranking` | 698 | 87 |  |
| report.rs | `error` | 791 | 229 |  |
| report.rs | `lints` | 1024 | 13 |  |
| report.rs | `warnings` | 1038 | 208 |  |
| report.rs | `to_text` | 1151 | 3 |  |
| report.rs | `to_text_with` | 1156 | 7 |  |
| report.rs | `model_text_at` | 1167 | 20 |  |
| report.rs | `problem_texts` | 1189 | 16 |  |
| report.rs | `model_warnings` | 1207 | 3 |  |
| report.rs | `to_text_at` | 1214 | 47 |  |
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
| report.rs | `libtest` | 11 | 17 | reader::libtest |
| source.rs | `mitra_line` | 24 | 8 | source::mitra_line |
| source.rs | `fn_below` | 105 | 35 | source::fn_below |
| source.rs | `is_tie` | 144 | 3 | source::is_tie |
| tie.rs | `results` | 435 | 3 | reader::Rust::results |
| tie.rs | `tests` | 441 | 38 | reader::Rust::tests |
| tie.rs | `tie` | 482 | 429 | reader::Rust::tie |
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

## mitra-cli

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| main.rs | `conform` | 94 | 52 | commands::conform |
| main.rs | `check_rev` | 361 | 8 | commands::check_rev |
| main.rs | `build` | 37 | 53 |  |
| main.rs | `assess` | 149 | 36 |  |
| main.rs | `diagnose` | 189 | 28 |  |
| main.rs | `in_place` | 219 | 7 |  |
| main.rs | `shown` | 228 | 8 |  |
| main.rs | `main` | 255 | 17 |  |
| main.rs | `inputs` | 277 | 14 |  |
| main.rs | `reader` | 297 | 3 |  |
| main.rs | `read` | 301 | 3 |  |
| main.rs | `located` | 306 | 9 |  |
| main.rs | `relative` | 317 | 7 |  |
| main.rs | `model` | 332 | 3 |  |
| main.rs | `evidence` | 335 | 3 |  |
| main.rs | `model` | 347 | 3 |  |
| main.rs | `evidence` | 350 | 6 |  |
| main.rs | `evaluate` | 374 | 25 |  |
| main.rs | `take_rev` | 402 | 18 |  |
| main.rs | `take_value` | 422 | 11 |  |
| main.rs | `take_display` | 436 | 20 |  |
| main.rs | `run` | 458 | 3 |  |
| main.rs | `run_command` | 463 | 321 |  |
