# How much of Mitra its model covers

Written by `scripts/coverage.py --write`. A function is covered when a
`// mitra: implements …` comment ties it to the model. Tests are left out.

| Crate | Functions | Tied | Share | Lines in functions | In tied ones | Share | Code lines | Model lines | Model / code |
|---|---|---|---|---|---|---|---|---|---|
| mitra-ds | 8 | 6 | 75% | 118 | 60 | 51% | 146 | 62 | 42% |
| mitra-cli | 52 | 7 | 13% | 1407 | 133 | 9% | 1493 | 16 | 1% |
| mitra-core | 338 | 15 | 4% | 11415 | 2254 | 20% | 12052 | 156 | 1% |
| all | 398 | 28 | 7% | 12940 | 2447 | 19% | 13691 | 234 | 2% |

- mitra-ds: 62 lines of model over 146 lines of code, in 2 subjects with 3 ensures and 0 requires; apart from it, 0 lines of point claims and 45 tests pinned in the code.
- mitra-cli: 16 lines of model over 1493 lines of code, in 2 subjects with 2 ensures and 0 requires; apart from it, 58 lines of point claims and 3 tests pinned in the code.
- mitra-core: 156 lines of model over 12052 lines of code, in 7 subjects with 16 ensures and 4 requires; apart from it, 67 lines of point claims and 0 tests pinned in the code.

## mitra-ds

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| culprits.rs | `rank` | 58 | 22 | culprits::rank |
| lib.rs | `judged` | 46 | 7 | mass::Mass::judged |
| lib.rs | `is_valid` | 56 | 5 | mass::Mass::is_valid |
| lib.rs | `conflict` | 64 | 3 | mass::Mass::conflict |
| lib.rs | `dempster` | 83 | 15 | mass::Mass::dempster |
| lib.rs | `yager` | 102 | 8 | mass::Mass::yager |
| culprits.rs | `weigh` | 87 | 51 |  |
| lib.rs | `agreement` | 70 | 7 |  |

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
| main.rs | `run_command` | 219 | 335 |  |
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
| tie.rs | `tests` | 350 | 37 |  |
| tie.rs | `tie` | 388 | 249 |  |

## mitra-core

| File | Function | Line | Lines | Tied to |
|---|---|---|---|---|
| ast.rs | `check` | 497 | 417 | types::Model::check |
| eval.rs | `as_str` | 28 | 8 | report::Logic::as_str |
| eval.rs | `unary` | 1493 | 12 | eval::unary |
| eval.rs | `binary` | 1508 | 63 | eval::binary |
| eval.rs | `evaluate` | 1826 | 1336 | verdicts::evaluate |
| lib.rs | `load` | 78 | 7 | types::load |
| lib.rs | `read` | 87 | 48 | types::read |
| lib.rs | `run` | 391 | 3 | verdicts::run |
| parse.rs | `model` | 3591 | 3 | reader::model |
| plan.rs | `plan` | 17 | 272 | advice::plan |
| print.rs | `negative` | 101 | 7 | print::negative |
| print.rs | `level` | 115 | 9 | print::level |
| prove.rs | `derive` | 478 | 45 | verdicts::derive |
| report.rs | `word` | 194 | 11 | report::word |
| set.rs | `quote` | 1121 | 13 | print::quote |
| ast.rs | `set` | 92 | 6 |  |
| ast.rs | `is_spec` | 117 | 3 |  |
| ast.rs | `domain` | 121 | 3 |  |
| ast.rs | `is_structural` | 171 | 6 |  |
| ast.rs | `spec_of_classes` | 330 | 22 |  |
| ast.rs | `conforms` | 356 | 4 |  |
| ast.rs | `named` | 363 | 28 |  |
| ast.rs | `named_recursive` | 395 | 16 |  |
| ast.rs | `record` | 411 | 3 |  |
| ast.rs | `function` | 415 | 4 |  |
| ast.rs | `ensure` | 420 | 3 |  |
| ast.rs | `captured` | 425 | 4 |  |
| ast.rs | `invariant` | 432 | 17 |  |
| ast.rs | `declared_function` | 451 | 3 |  |
| ast.rs | `scored` | 456 | 8 |  |
| ast.rs | `subjects_of` | 466 | 14 |  |
| ast.rs | `claim` | 480 | 3 |  |
| ast.rs | `covers` | 485 | 9 |  |
| ast.rs | `arities` | 935 | 9 |  |
| ast.rs | `passed` | 946 | 13 |  |
| ast.rs | `descents` | 966 | 61 |  |
| ast.rs | `part` | 967 | 7 |  |
| ast.rs | `walk` | 1029 | 26 |  |
| ast.rs | `arms` | 1057 | 21 |  |
| ast.rs | `calls` | 1080 | 21 |  |
| ast.rs | `single` | 1103 | 5 |  |
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
| lib.rs | `new` | 52 | 6 |  |
| lib.rs | `fmt` | 61 | 3 |  |
| lib.rs | `points` | 142 | 84 |  |
| lib.rs | `determinism` | 232 | 24 |  |
| lib.rs | `invariants` | 260 | 40 |  |
| lib.rs | `tie` | 302 | 16 |  |
| lib.rs | `tie_uses` | 321 | 23 |  |
| lib.rs | `tie_records` | 347 | 34 |  |
| lib.rs | `read_evidence` | 384 | 4 |  |
| next.rs | `next` | 16 | 385 |  |
| next.rs | `ask` | 391 | 17 |  |
| next.rs | `vouch` | 410 | 4 |  |
| parse.rs | `imports_from` | 25 | 3 |  |
| parse.rs | `library` | 30 | 3 |  |
| parse.rs | `module` | 35 | 18 |  |
| parse.rs | `lex` | 82 | 67 |  |
| parse.rs | `earlier` | 154 | 31 |  |
| parse.rs | `unquote` | 279 | 15 |  |
| parse.rs | `join` | 296 | 4 |  |
| parse.rs | `bin` | 392 | 3 |  |
| parse.rs | `line` | 397 | 4 |  |
| parse.rs | `err` | 401 | 3 |  |
| parse.rs | `peek` | 404 | 3 |  |
| parse.rs | `peek_at` | 407 | 3 |  |
| parse.rs | `kind` | 410 | 3 |  |
| parse.rs | `kind_at` | 413 | 3 |  |
| parse.rs | `is` | 416 | 3 |  |
| parse.rs | `eat` | 419 | 5 |  |
| parse.rs | `expect` | 424 | 12 |  |
| parse.rs | `name` | 436 | 9 |  |
| parse.rs | `path` | 446 | 7 |  |
| parse.rs | `qualified` | 453 | 6 |  |
| parse.rs | `resolve` | 460 | 19 |  |
| parse.rs | `find` | 482 | 18 |  |
| parse.rs | `old` | 501 | 9 |  |
| parse.rs | `next_old` | 510 | 5 |  |
| parse.rs | `at_end` | 515 | 3 |  |
| parse.rs | `is_name` | 518 | 3 |  |
| parse.rs | `ident` | 522 | 9 |  |
| parse.rs | `word` | 533 | 6 |  |
| parse.rs | `local` | 541 | 6 |  |
| parse.rs | `is_keyword` | 547 | 3 |  |
| parse.rs | `eat_keyword` | 550 | 5 |  |
| parse.rs | `number` | 559 | 12 |  |
| parse.rs | `signed` | 572 | 16 |  |
| parse.rs | `numeral` | 589 | 12 |  |
| parse.rs | `values` | 602 | 8 |  |
| parse.rs | `value` | 612 | 10 |  |
| parse.rs | `value_inner` | 623 | 86 |  |
| parse.rs | `bound` | 714 | 20 |  |
| parse.rs | `set` | 738 | 7 |  |
| parse.rs | `set_inter` | 746 | 12 |  |
| parse.rs | `sets_in_parentheses` | 759 | 13 |  |
| parse.rs | `set_atom` | 772 | 212 |  |
| parse.rs | `variants` | 1005 | 70 |  |
| parse.rs | `both` | 1077 | 12 |  |
| parse.rs | `set_both` | 1090 | 4 |  |
| parse.rs | `binders` | 1095 | 33 |  |
| parse.rs | `args` | 1133 | 60 |  |
| parse.rs | `expr` | 1200 | 7 |  |
| parse.rs | `or` | 1207 | 7 |  |
| parse.rs | `and` | 1214 | 7 |  |
| parse.rs | `compare` | 1221 | 25 |  |
| parse.rs | `sum` | 1247 | 11 |  |
| parse.rs | `product` | 1259 | 11 |  |
| parse.rs | `unary` | 1270 | 42 |  |
| parse.rs | `method` | 1318 | 61 |  |
| parse.rs | `exprs` | 1380 | 11 |  |
| parse.rs | `block` | 1392 | 6 |  |
| parse.rs | `sequence` | 1399 | 16 |  |
| parse.rs | `bars` | 1417 | 11 |  |
| parse.rs | `primary` | 1428 | 275 |  |
| parse.rs | `trust` | 1718 | 10 |  |
| parse.rs | `state` | 1732 | 20 |  |
| parse.rs | `define` | 1753 | 13 |  |
| parse.rs | `declare` | 1767 | 11 |  |
| parse.rs | `type_params` | 1779 | 17 |  |
| parse.rs | `type_tokens` | 1797 | 15 |  |
| parse.rs | `enumeration` | 1814 | 35 |  |
| parse.rs | `items` | 1851 | 243 |  |
| parse.rs | `function` | 2109 | 117 |  |
| parse.rs | `sig` | 2236 | 11 |  |
| parse.rs | `inputs` | 2250 | 27 |  |
| parse.rs | `claim` | 2280 | 212 |  |
| parse.rs | `equivalence` | 2514 | 124 |  |
| parse.rs | `represented` | 2647 | 87 |  |
| parse.rs | `import` | 2738 | 66 |  |
| parse.rs | `instance` | 2812 | 92 |  |
| parse.rs | `evidence` | 2910 | 182 |  |
| parse.rs | `finish` | 3102 | 176 |  |
| parse.rs | `requirements` | 3294 | 140 |  |
| parse.rs | `reliance` | 3448 | 123 |  |
| parse.rs | `declared` | 3597 | 30 |  |
| parse.rs | `types_of` | 3632 | 55 |  |
| parse.rs | `merge_types` | 3690 | 27 |  |
| parse.rs | `scope_types` | 3721 | 56 |  |
| parse.rs | `impls` | 3784 | 99 |  |
| parse.rs | `read` | 3886 | 34 |  |
| parse.rs | `evidence` | 3922 | 11 |  |
| parse.rs | `evidence_alone` | 3935 | 50 |  |
| parse.rs | `free` | 3992 | 60 |  |
| parse.rs | `unwritten` | 4054 | 6 |  |
| parse.rs | `records_in` | 4061 | 32 |  |
| parse.rs | `written` | 4096 | 20 |  |
| parse.rs | `stamp` | 4119 | 46 |  |
| parse.rs | `tied` | 4168 | 4 |  |
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
| types.rs | `check` | 14 | 29 |  |
| types.rs | `plain` | 45 | 4 |  |
| types.rs | `show` | 50 | 7 |  |
| types.rs | `unknown` | 58 | 3 |  |
| types.rs | `numbers` | 62 | 3 |  |
| types.rs | `needs` | 67 | 11 |  |
| types.rs | `call` | 80 | 8 |  |
| types.rs | `of` | 90 | 210 |  |
