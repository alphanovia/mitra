# How much of Mitra its model covers

Written by `scripts/coverage.py --write`. A function is covered when a
`// mitra: implements …` comment ties it to the model. Tests are left out.

| Crate | Functions | Tied | Share | Lines in functions | In tied ones | Share | Code lines | Model lines | Model / code |
|---|---|---|---|---|---|---|---|---|---|
| mitra-ds | 8 | 6 | 75% | 118 | 60 | 51% | 146 | 164 | 112% |
| mitra-cli | 50 | 7 | 14% | 1238 | 133 | 11% | 1322 | 74 | 6% |
| mitra-core | 324 | 15 | 5% | 10468 | 2174 | 21% | 11084 | 215 | 2% |
| all | 382 | 28 | 7% | 11824 | 2367 | 20% | 12552 | 453 | 4% |

## mitra-ds

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| culprits.rs | `rank` | 57 | 22 | culprits::rank |
| lib.rs | `judged` | 44 | 7 | mass::judged |
| lib.rs | `is_valid` | 54 | 5 | mass::is_valid |
| lib.rs | `conflict` | 62 | 3 | mass::conflict |
| lib.rs | `dempster` | 81 | 15 | mass::dempster |
| lib.rs | `yager` | 100 | 8 | mass::yager |
| culprits.rs | `weigh` | 86 | 51 |  |
| lib.rs | `agreement` | 68 | 7 |  |

## mitra-cli

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| report.rs | `short` | 20 | 4 | report::short |
| report.rs | `libtest` | 26 | 17 | report::libtest |
| report.rs | `attr` | 48 | 30 | report::attr |
| report.rs | `strip` | 81 | 11 | report::strip |
| report.rs | `junit` | 94 | 28 | report::junit |
| source.rs | `mitra_line` | 119 | 8 | source::mitra_line |
| source.rs | `fn_below` | 200 | 35 | source::fn_below |
| main.rs | `main` | 39 | 17 |  |
| main.rs | `inputs` | 61 | 14 |  |
| main.rs | `read` | 79 | 3 |  |
| main.rs | `located` | 84 | 9 |  |
| main.rs | `relative` | 95 | 7 |  |
| main.rs | `model` | 110 | 3 |  |
| main.rs | `evidence` | 113 | 3 |  |
| main.rs | `check_rev` | 120 | 8 |  |
| main.rs | `evaluate` | 133 | 25 |  |
| main.rs | `take_rev` | 161 | 15 |  |
| main.rs | `take_value` | 178 | 11 |  |
| main.rs | `take_display` | 192 | 20 |  |
| main.rs | `run` | 214 | 3 |  |
| main.rs | `run_command` | 219 | 278 |  |
| source.rs | `file` | 18 | 19 |  |
| source.rs | `add_file` | 40 | 6 |  |
| source.rs | `push_lines` | 47 | 7 |  |
| source.rs | `add_rust` | 57 | 24 |  |
| source.rs | `place` | 86 | 6 |  |
| source.rs | `locate` | 93 | 6 |  |
| source.rs | `rust_files` | 101 | 14 |  |
| source.rs | `scan` | 140 | 55 |  |
| source.rs | `is_tie` | 238 | 3 |  |
| store.rs | `find` | 27 | 12 |  |
| store.rs | `of` | 42 | 6 |  |
| store.rs | `init` | 50 | 14 |  |
| store.rs | `root` | 66 | 3 |  |
| store.rs | `model_path` | 70 | 3 |  |
| store.rs | `source` | 75 | 17 |  |
| store.rs | `evidence_path` | 95 | 3 |  |
| store.rs | `add` | 101 | 20 |  |
| store.rs | `model` | 123 | 8 |  |
| store.rs | `append` | 136 | 35 |  |
| tie.rs | `shape_below` | 37 | 22 |  |
| tie.rs | `fields` | 61 | 18 |  |
| tie.rs | `balanced` | 81 | 12 |  |
| tie.rs | `pieces` | 96 | 20 |  |
| tie.rs | `as_model` | 118 | 45 |  |
| tie.rs | `fits` | 168 | 25 |  |
| tie.rs | `read` | 198 | 62 |  |
| tie.rs | `calls` | 263 | 8 |  |
| tie.rs | `tests` | 294 | 34 |  |
| tie.rs | `tie` | 329 | 182 |  |

## mitra-core

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| ast.rs | `check` | 479 | 415 | types::check |
| eval.rs | `as_str` | 28 | 8 | report::as_str |
| eval.rs | `unary` | 1486 | 12 | eval::unary |
| eval.rs | `binary` | 1501 | 63 | eval::binary |
| eval.rs | `evaluate` | 1819 | 1276 | verdicts::evaluate |
| lib.rs | `load` | 77 | 7 | types::load |
| lib.rs | `read` | 86 | 46 | types::read |
| lib.rs | `run` | 335 | 3 | verdicts::run |
| parse.rs | `model` | 3235 | 3 | reader::model |
| plan.rs | `plan` | 17 | 256 | advice::plan |
| print.rs | `negative` | 93 | 7 | print::negative |
| print.rs | `level` | 107 | 9 | print::level |
| prove.rs | `derive` | 478 | 45 | verdicts::derive |
| report.rs | `word` | 193 | 11 | report::word |
| set.rs | `quote` | 1121 | 13 | print::quote |
| ast.rs | `set` | 92 | 6 |  |
| ast.rs | `is_spec` | 117 | 3 |  |
| ast.rs | `domain` | 121 | 3 |  |
| ast.rs | `is_structural` | 171 | 6 |  |
| ast.rs | `spec_of_classes` | 312 | 22 |  |
| ast.rs | `conforms` | 338 | 4 |  |
| ast.rs | `named` | 345 | 28 |  |
| ast.rs | `named_recursive` | 377 | 16 |  |
| ast.rs | `record` | 393 | 3 |  |
| ast.rs | `function` | 397 | 4 |  |
| ast.rs | `ensure` | 402 | 3 |  |
| ast.rs | `captured` | 407 | 4 |  |
| ast.rs | `invariant` | 414 | 17 |  |
| ast.rs | `declared_function` | 433 | 3 |  |
| ast.rs | `scored` | 438 | 8 |  |
| ast.rs | `subjects_of` | 448 | 14 |  |
| ast.rs | `claim` | 462 | 3 |  |
| ast.rs | `covers` | 467 | 9 |  |
| ast.rs | `arities` | 914 | 9 |  |
| ast.rs | `passed` | 925 | 13 |  |
| ast.rs | `descents` | 945 | 61 |  |
| ast.rs | `part` | 946 | 7 |  |
| ast.rs | `walk` | 1008 | 26 |  |
| ast.rs | `arms` | 1036 | 21 |  |
| ast.rs | `calls` | 1059 | 21 |  |
| ast.rs | `single` | 1082 | 5 |  |
| eval.rs | `as_str` | 53 | 7 |  |
| eval.rs | `text` | 191 | 14 |  |
| eval.rs | `declared_claim` | 241 | 7 |  |
| eval.rs | `code` | 315 | 11 |  |
| eval.rs | `default` | 338 | 6 |  |
| eval.rs | `rules` | 422 | 82 |  |
| eval.rs | `relations` | 514 | 22 |  |
| eval.rs | `close` | 548 | 64 |  |
| eval.rs | `eval` | 616 | 295 |  |
| eval.rs | `call` | 922 | 38 |  |
| eval.rs | `call_inner` | 961 | 104 |  |
| eval.rs | `target` | 1070 | 23 |  |
| eval.rs | `compare` | 1096 | 34 |  |
| eval.rs | `measure` | 1135 | 36 |  |
| eval.rs | `count` | 1175 | 91 |  |
| eval.rs | `candidates` | 1272 | 47 |  |
| eval.rs | `judged` | 1323 | 43 |  |
| eval.rs | `forced` | 1372 | 53 |  |
| eval.rs | `counter` | 1432 | 11 |  |
| eval.rs | `implication` | 1446 | 25 |  |
| eval.rs | `product` | 1474 | 10 |  |
| eval.rs | `class_text` | 1581 | 19 |  |
| eval.rs | `computable` | 1630 | 23 |  |
| eval.rs | `subject_calls` | 1656 | 34 |  |
| eval.rs | `sorted` | 1693 | 86 |  |
| eval.rs | `listed` | 1811 | 6 |  |
| eval.rs | `represented` | 3225 | 23 |  |
| eval.rs | `computed` | 3251 | 22 |  |
| eval.rs | `rank` | 3276 | 136 |  |
| eval.rs | `brief` | 3421 | 14 |  |
| eval.rs | `warnings` | 3446 | 10 |  |
| eval.rs | `own_word` | 3462 | 3 |  |
| lib.rs | `new` | 51 | 6 |  |
| lib.rs | `fmt` | 60 | 3 |  |
| lib.rs | `points` | 139 | 84 |  |
| lib.rs | `invariants` | 230 | 40 |  |
| lib.rs | `tie` | 272 | 16 |  |
| lib.rs | `tie_records` | 291 | 34 |  |
| lib.rs | `read_evidence` | 328 | 4 |  |
| next.rs | `next` | 16 | 370 |  |
| next.rs | `ask` | 375 | 17 |  |
| next.rs | `vouch` | 394 | 4 |  |
| parse.rs | `imports_from` | 25 | 3 |  |
| parse.rs | `library` | 30 | 3 |  |
| parse.rs | `module` | 35 | 18 |  |
| parse.rs | `lex` | 82 | 64 |  |
| parse.rs | `earlier` | 150 | 31 |  |
| parse.rs | `unquote` | 268 | 15 |  |
| parse.rs | `join` | 285 | 4 |  |
| parse.rs | `bin` | 375 | 3 |  |
| parse.rs | `line` | 380 | 4 |  |
| parse.rs | `err` | 384 | 3 |  |
| parse.rs | `peek` | 387 | 3 |  |
| parse.rs | `peek_at` | 390 | 3 |  |
| parse.rs | `kind` | 393 | 3 |  |
| parse.rs | `kind_at` | 396 | 3 |  |
| parse.rs | `is` | 399 | 3 |  |
| parse.rs | `eat` | 402 | 5 |  |
| parse.rs | `expect` | 407 | 12 |  |
| parse.rs | `name` | 419 | 9 |  |
| parse.rs | `path` | 429 | 7 |  |
| parse.rs | `qualified` | 436 | 6 |  |
| parse.rs | `resolve` | 443 | 19 |  |
| parse.rs | `find` | 465 | 18 |  |
| parse.rs | `old` | 484 | 9 |  |
| parse.rs | `next_old` | 493 | 5 |  |
| parse.rs | `at_end` | 498 | 3 |  |
| parse.rs | `is_name` | 501 | 3 |  |
| parse.rs | `ident` | 505 | 9 |  |
| parse.rs | `is_keyword` | 514 | 3 |  |
| parse.rs | `eat_keyword` | 517 | 5 |  |
| parse.rs | `number` | 526 | 12 |  |
| parse.rs | `signed` | 539 | 16 |  |
| parse.rs | `numeral` | 556 | 12 |  |
| parse.rs | `values` | 569 | 8 |  |
| parse.rs | `value` | 579 | 10 |  |
| parse.rs | `value_inner` | 590 | 86 |  |
| parse.rs | `bound` | 681 | 20 |  |
| parse.rs | `set` | 705 | 7 |  |
| parse.rs | `set_inter` | 713 | 12 |  |
| parse.rs | `sets_in_parentheses` | 726 | 13 |  |
| parse.rs | `set_atom` | 739 | 212 |  |
| parse.rs | `variants` | 972 | 70 |  |
| parse.rs | `both` | 1044 | 12 |  |
| parse.rs | `set_both` | 1057 | 4 |  |
| parse.rs | `binders` | 1062 | 27 |  |
| parse.rs | `args` | 1092 | 60 |  |
| parse.rs | `expr` | 1159 | 7 |  |
| parse.rs | `or` | 1166 | 7 |  |
| parse.rs | `and` | 1173 | 7 |  |
| parse.rs | `compare` | 1180 | 25 |  |
| parse.rs | `sum` | 1206 | 11 |  |
| parse.rs | `product` | 1218 | 11 |  |
| parse.rs | `unary` | 1229 | 34 |  |
| parse.rs | `exprs` | 1265 | 11 |  |
| parse.rs | `block` | 1277 | 6 |  |
| parse.rs | `sequence` | 1284 | 13 |  |
| parse.rs | `bars` | 1298 | 11 |  |
| parse.rs | `primary` | 1309 | 254 |  |
| parse.rs | `trust` | 1577 | 10 |  |
| parse.rs | `state` | 1591 | 20 |  |
| parse.rs | `define` | 1612 | 13 |  |
| parse.rs | `declare` | 1626 | 11 |  |
| parse.rs | `type_params` | 1638 | 17 |  |
| parse.rs | `type_tokens` | 1656 | 15 |  |
| parse.rs | `enumeration` | 1673 | 35 |  |
| parse.rs | `items` | 1710 | 243 |  |
| parse.rs | `function` | 1968 | 94 |  |
| parse.rs | `sig` | 2068 | 11 |  |
| parse.rs | `inputs` | 2082 | 22 |  |
| parse.rs | `claim` | 2105 | 190 |  |
| parse.rs | `equivalence` | 2316 | 124 |  |
| parse.rs | `represented` | 2449 | 87 |  |
| parse.rs | `import` | 2540 | 66 |  |
| parse.rs | `instance` | 2614 | 92 |  |
| parse.rs | `evidence` | 2712 | 127 |  |
| parse.rs | `finish` | 2843 | 134 |  |
| parse.rs | `requirements` | 2990 | 140 |  |
| parse.rs | `reliance` | 3144 | 74 |  |
| parse.rs | `declared` | 3241 | 26 |  |
| parse.rs | `types_of` | 3272 | 55 |  |
| parse.rs | `merge_types` | 3330 | 27 |  |
| parse.rs | `scope_types` | 3361 | 56 |  |
| parse.rs | `read` | 3420 | 27 |  |
| parse.rs | `evidence` | 3449 | 11 |  |
| parse.rs | `evidence_alone` | 3462 | 43 |  |
| parse.rs | `free` | 3511 | 60 |  |
| parse.rs | `unwritten` | 3573 | 6 |  |
| parse.rs | `records_in` | 3580 | 32 |  |
| parse.rs | `written` | 3615 | 20 |  |
| parse.rs | `stamp` | 3638 | 46 |  |
| parse.rs | `tied` | 3687 | 4 |  |
| plan.rs | `owed` | 62 | 158 |  |
| plan.rs | `point` | 226 | 6 |  |
| plan.rs | `subject_call` | 233 | 3 |  |
| plan.rs | `calls_no_subject` | 237 | 5 |  |
| plan.rs | `class_set` | 244 | 24 |  |
| plan.rs | `observed` | 270 | 8 |  |
| plan.rs | `set_text` | 280 | 11 |  |
| print.rs | `print_claim` | 8 | 3 |  |
| print.rs | `print_evidence` | 14 | 49 |  |
| print.rs | `pattern` | 65 | 19 |  |
| print.rs | `call` | 87 | 4 |  |
| print.rs | `expr` | 117 | 3 |  |
| print.rs | `contract` | 122 | 112 |  |
| print.rs | `at` | 238 | 99 |  |
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
| report.rs | `explain` | 182 | 4 |  |
| report.rs | `headline` | 187 | 3 |  |
| report.rs | `claim` | 213 | 3 |  |
| report.rs | `verdict` | 217 | 4 |  |
| report.rs | `at_claim` | 223 | 8 |  |
| report.rs | `evidence_text` | 232 | 16 |  |
| report.rs | `entry` | 251 | 8 |  |
| report.rs | `declared` | 262 | 5 |  |
| report.rs | `logic` | 269 | 4 |  |
| report.rs | `stated` | 275 | 6 |  |
| report.rs | `evidence_at` | 283 | 6 |  |
| report.rs | `role` | 290 | 15 |  |
| report.rs | `inputs` | 307 | 34 |  |
| report.rs | `missed` | 344 | 21 |  |
| report.rs | `verdicts` | 368 | 112 |  |
| report.rs | `quoted` | 482 | 13 |  |
| report.rs | `piece` | 497 | 83 |  |
| report.rs | `ranking` | 583 | 79 |  |
| report.rs | `error` | 666 | 220 |  |
| report.rs | `warnings` | 889 | 150 |  |
| report.rs | `to_text` | 983 | 3 |  |
| report.rs | `to_text_with` | 988 | 7 |  |
| report.rs | `to_text_at` | 999 | 47 |  |
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
| set.rs | `minus` | 822 | 3 |  |
| set.rs | `is_empty` | 826 | 9 |  |
| set.rs | `is_subset` | 836 | 3 |  |
| set.rs | `contains` | 840 | 13 |  |
| set.rs | `shapes` | 855 | 3 |  |
| set.rs | `conforms` | 860 | 33 |  |
| set.rs | `values` | 896 | 29 |  |
| set.rs | `single` | 927 | 6 |  |
| set.rs | `count` | 935 | 30 |  |
| set.rs | `fmt` | 971 | 134 |  |
| set.rs | `n_is_set` | 1115 | 3 |  |
| set.rs | `paren` | 1136 | 8 |  |
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
