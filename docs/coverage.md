# How much of Mitra its model covers

Written by `scripts/coverage.py --write`. A function is covered when a
`// mitra: implements …` comment ties it to the model. Tests are left out.

| Crate | Functions | Tied | Share | Lines in functions | In tied ones | Share | Code lines | Model lines | Model / code |
|---|---|---|---|---|---|---|---|---|---|
| mitra-ds | 8 | 6 | 75% | 118 | 60 | 51% | 146 | 161 | 110% |
| mitra-cli | 48 | 7 | 15% | 1119 | 133 | 12% | 1197 | 72 | 6% |
| mitra-core | 312 | 7 | 2% | 9296 | 123 | 1% | 10138 | 134 | 1% |
| all | 368 | 20 | 5% | 10533 | 316 | 3% | 11481 | 367 | 3% |

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
| main.rs | `run_command` | 219 | 274 |  |
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
| tie.rs | `balanced` | 25 | 12 |  |
| tie.rs | `pieces` | 40 | 20 |  |
| tie.rs | `as_model` | 62 | 45 |  |
| tie.rs | `fits` | 112 | 18 |  |
| tie.rs | `read` | 134 | 47 |  |
| tie.rs | `calls` | 184 | 8 |  |
| tie.rs | `tests` | 215 | 34 |  |
| tie.rs | `tie` | 250 | 129 |  |

## mitra-core

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| eval.rs | `as_str` | 28 | 8 | eval::as_str |
| eval.rs | `unary` | 1476 | 12 | eval::unary |
| eval.rs | `binary` | 1491 | 63 | eval::binary |
| print.rs | `negative` | 93 | 7 | print::negative |
| print.rs | `level` | 107 | 9 | print::level |
| report.rs | `word` | 160 | 11 | report::word |
| set.rs | `quote` | 1121 | 13 | set::quote |
| ast.rs | `set` | 88 | 6 |  |
| ast.rs | `is_spec` | 113 | 3 |  |
| ast.rs | `domain` | 117 | 3 |  |
| ast.rs | `is_structural` | 167 | 6 |  |
| ast.rs | `spec_of_classes` | 270 | 22 |  |
| ast.rs | `conforms` | 296 | 4 |  |
| ast.rs | `named` | 303 | 28 |  |
| ast.rs | `named_recursive` | 335 | 16 |  |
| ast.rs | `record` | 351 | 3 |  |
| ast.rs | `function` | 355 | 4 |  |
| ast.rs | `declared_function` | 360 | 3 |  |
| ast.rs | `scored` | 365 | 8 |  |
| ast.rs | `subjects_of` | 375 | 14 |  |
| ast.rs | `claim` | 389 | 3 |  |
| ast.rs | `covers` | 394 | 9 |  |
| ast.rs | `check` | 405 | 415 |  |
| ast.rs | `arities` | 840 | 9 |  |
| ast.rs | `passed` | 851 | 13 |  |
| ast.rs | `descents` | 871 | 61 |  |
| ast.rs | `part` | 872 | 7 |  |
| ast.rs | `walk` | 934 | 26 |  |
| ast.rs | `arms` | 962 | 21 |  |
| ast.rs | `calls` | 985 | 21 |  |
| ast.rs | `single` | 1008 | 5 |  |
| eval.rs | `as_str` | 53 | 7 |  |
| eval.rs | `text` | 190 | 14 |  |
| eval.rs | `declared_claim` | 234 | 7 |  |
| eval.rs | `code` | 306 | 11 |  |
| eval.rs | `default` | 329 | 6 |  |
| eval.rs | `rules` | 412 | 82 |  |
| eval.rs | `relations` | 504 | 22 |  |
| eval.rs | `close` | 538 | 64 |  |
| eval.rs | `eval` | 606 | 295 |  |
| eval.rs | `call` | 912 | 38 |  |
| eval.rs | `call_inner` | 951 | 104 |  |
| eval.rs | `target` | 1060 | 23 |  |
| eval.rs | `compare` | 1086 | 34 |  |
| eval.rs | `measure` | 1125 | 36 |  |
| eval.rs | `count` | 1165 | 91 |  |
| eval.rs | `candidates` | 1262 | 47 |  |
| eval.rs | `judged` | 1313 | 43 |  |
| eval.rs | `forced` | 1362 | 53 |  |
| eval.rs | `counter` | 1422 | 11 |  |
| eval.rs | `implication` | 1436 | 25 |  |
| eval.rs | `product` | 1464 | 10 |  |
| eval.rs | `class_text` | 1571 | 19 |  |
| eval.rs | `computable` | 1620 | 23 |  |
| eval.rs | `subject_calls` | 1646 | 34 |  |
| eval.rs | `sorted` | 1683 | 86 |  |
| eval.rs | `listed` | 1801 | 6 |  |
| eval.rs | `evaluate` | 1808 | 1211 |  |
| eval.rs | `represented` | 3134 | 23 |  |
| eval.rs | `computed` | 3160 | 22 |  |
| eval.rs | `rank` | 3185 | 129 |  |
| eval.rs | `brief` | 3321 | 14 |  |
| eval.rs | `warnings` | 3346 | 5 |  |
| eval.rs | `own_word` | 3356 | 3 |  |
| lib.rs | `new` | 50 | 6 |  |
| lib.rs | `fmt` | 59 | 3 |  |
| lib.rs | `load` | 74 | 7 |  |
| lib.rs | `read` | 82 | 44 |  |
| lib.rs | `points` | 133 | 84 |  |
| lib.rs | `tie` | 222 | 16 |  |
| lib.rs | `read_evidence` | 241 | 4 |  |
| lib.rs | `run` | 247 | 3 |  |
| next.rs | `next` | 16 | 308 |  |
| next.rs | `ask` | 333 | 17 |  |
| next.rs | `vouch` | 352 | 4 |  |
| parse.rs | `imports_from` | 25 | 3 |  |
| parse.rs | `library` | 30 | 3 |  |
| parse.rs | `module` | 35 | 18 |  |
| parse.rs | `lex` | 81 | 64 |  |
| parse.rs | `earlier` | 149 | 31 |  |
| parse.rs | `unquote` | 262 | 15 |  |
| parse.rs | `join` | 279 | 4 |  |
| parse.rs | `bin` | 361 | 3 |  |
| parse.rs | `line` | 366 | 4 |  |
| parse.rs | `err` | 370 | 3 |  |
| parse.rs | `peek` | 373 | 3 |  |
| parse.rs | `peek_at` | 376 | 3 |  |
| parse.rs | `kind` | 379 | 3 |  |
| parse.rs | `kind_at` | 382 | 3 |  |
| parse.rs | `is` | 385 | 3 |  |
| parse.rs | `eat` | 388 | 5 |  |
| parse.rs | `expect` | 393 | 12 |  |
| parse.rs | `name` | 405 | 9 |  |
| parse.rs | `path` | 415 | 7 |  |
| parse.rs | `qualified` | 422 | 6 |  |
| parse.rs | `resolve` | 429 | 19 |  |
| parse.rs | `find` | 451 | 18 |  |
| parse.rs | `old` | 470 | 9 |  |
| parse.rs | `next_old` | 479 | 5 |  |
| parse.rs | `at_end` | 484 | 3 |  |
| parse.rs | `is_name` | 487 | 3 |  |
| parse.rs | `ident` | 491 | 9 |  |
| parse.rs | `is_keyword` | 500 | 3 |  |
| parse.rs | `eat_keyword` | 503 | 5 |  |
| parse.rs | `number` | 512 | 12 |  |
| parse.rs | `signed` | 525 | 16 |  |
| parse.rs | `numeral` | 542 | 12 |  |
| parse.rs | `values` | 555 | 8 |  |
| parse.rs | `value` | 565 | 10 |  |
| parse.rs | `value_inner` | 576 | 83 |  |
| parse.rs | `bound` | 664 | 20 |  |
| parse.rs | `set` | 688 | 7 |  |
| parse.rs | `set_inter` | 696 | 12 |  |
| parse.rs | `sets_in_parentheses` | 709 | 13 |  |
| parse.rs | `set_atom` | 722 | 212 |  |
| parse.rs | `variants` | 955 | 69 |  |
| parse.rs | `both` | 1026 | 12 |  |
| parse.rs | `set_both` | 1039 | 4 |  |
| parse.rs | `binders` | 1044 | 27 |  |
| parse.rs | `args` | 1074 | 60 |  |
| parse.rs | `expr` | 1141 | 7 |  |
| parse.rs | `or` | 1148 | 7 |  |
| parse.rs | `and` | 1155 | 7 |  |
| parse.rs | `compare` | 1162 | 25 |  |
| parse.rs | `sum` | 1188 | 11 |  |
| parse.rs | `product` | 1200 | 11 |  |
| parse.rs | `unary` | 1211 | 34 |  |
| parse.rs | `exprs` | 1247 | 11 |  |
| parse.rs | `block` | 1259 | 6 |  |
| parse.rs | `sequence` | 1266 | 13 |  |
| parse.rs | `bars` | 1280 | 11 |  |
| parse.rs | `primary` | 1291 | 254 |  |
| parse.rs | `trust` | 1559 | 10 |  |
| parse.rs | `state` | 1573 | 17 |  |
| parse.rs | `define` | 1591 | 13 |  |
| parse.rs | `declare` | 1605 | 11 |  |
| parse.rs | `type_params` | 1617 | 17 |  |
| parse.rs | `type_tokens` | 1635 | 15 |  |
| parse.rs | `enumeration` | 1652 | 35 |  |
| parse.rs | `items` | 1689 | 159 |  |
| parse.rs | `function` | 1857 | 93 |  |
| parse.rs | `sig` | 1956 | 11 |  |
| parse.rs | `inputs` | 1970 | 22 |  |
| parse.rs | `claim` | 1993 | 190 |  |
| parse.rs | `equivalence` | 2204 | 124 |  |
| parse.rs | `represented` | 2337 | 87 |  |
| parse.rs | `import` | 2428 | 66 |  |
| parse.rs | `instance` | 2502 | 92 |  |
| parse.rs | `evidence` | 2600 | 127 |  |
| parse.rs | `finish` | 2731 | 110 |  |
| parse.rs | `model` | 2853 | 3 |  |
| parse.rs | `declared` | 2859 | 26 |  |
| parse.rs | `read` | 2887 | 21 |  |
| parse.rs | `evidence` | 2910 | 11 |  |
| parse.rs | `evidence_alone` | 2923 | 38 |  |
| parse.rs | `free` | 2966 | 60 |  |
| parse.rs | `records_in` | 3027 | 32 |  |
| parse.rs | `written` | 3062 | 20 |  |
| parse.rs | `stamp` | 3085 | 46 |  |
| parse.rs | `tied` | 3134 | 4 |  |
| plan.rs | `plan` | 16 | 25 |  |
| plan.rs | `owed` | 44 | 158 |  |
| plan.rs | `point` | 208 | 6 |  |
| plan.rs | `subject_call` | 215 | 3 |  |
| plan.rs | `calls_no_subject` | 219 | 5 |  |
| plan.rs | `class_set` | 226 | 24 |  |
| plan.rs | `observed` | 252 | 8 |  |
| plan.rs | `set_text` | 262 | 11 |  |
| print.rs | `print_claim` | 8 | 3 |  |
| print.rs | `print_evidence` | 14 | 49 |  |
| print.rs | `pattern` | 65 | 19 |  |
| print.rs | `call` | 87 | 4 |  |
| print.rs | `expr` | 117 | 3 |  |
| print.rs | `contract` | 122 | 92 |  |
| print.rs | `at` | 218 | 99 |  |
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
| prove.rs | `derive` | 477 | 45 |  |
| report.rs | `model` | 16 | 1 |  |
| report.rs | `evidence` | 17 | 1 |  |
| report.rs | `model` | 27 | 3 |  |
| report.rs | `evidence` | 30 | 3 |  |
| report.rs | `explain` | 149 | 4 |  |
| report.rs | `headline` | 154 | 3 |  |
| report.rs | `claim` | 180 | 3 |  |
| report.rs | `verdict` | 184 | 4 |  |
| report.rs | `at_claim` | 190 | 8 |  |
| report.rs | `evidence_text` | 199 | 16 |  |
| report.rs | `entry` | 218 | 8 |  |
| report.rs | `declared` | 229 | 5 |  |
| report.rs | `stated` | 236 | 6 |  |
| report.rs | `evidence_at` | 244 | 6 |  |
| report.rs | `role` | 251 | 15 |  |
| report.rs | `inputs` | 268 | 34 |  |
| report.rs | `missed` | 305 | 21 |  |
| report.rs | `verdicts` | 329 | 70 |  |
| report.rs | `quoted` | 401 | 13 |  |
| report.rs | `piece` | 416 | 46 |  |
| report.rs | `ranking` | 465 | 75 |  |
| report.rs | `error` | 544 | 198 |  |
| report.rs | `warnings` | 745 | 62 |  |
| report.rs | `to_text` | 811 | 3 |  |
| report.rs | `to_text_with` | 816 | 7 |  |
| report.rs | `to_text_at` | 827 | 47 |  |
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
