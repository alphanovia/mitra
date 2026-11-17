# Mitra language reference

This page describes exactly what the code implements today and nothing
more. When the code changes, this page changes in the same commit. Design
ideas and open questions live in the [general docs](../../README.md#documentation).

## Running Mitra

```sh
mitra eval model.mitra evidence.mitra [-o result.txt]
```

`mitra` (crate `mitra-cli`) reads a model file and an evidence file, calls
the library `mitra-core`, and writes the result to `result.txt`, or to
standard output without `-o`. On an error it prints `file: line N: message`
and exits with status 2.

## Lexical rules

- One statement per line. Blank lines are ignored; `#` starts a comment that
  runs to the end of the line.
- Names: a letter or `_`, then letters, digits or `_`.
- Numbers: digits with an optional `.` (`1`, `0`, `1.0`).
- Symbols: `(` `)` `,` `:` `->` `@`.
- Every statement starts with a keyword (`subject`, `claim`, `evidence`), so
  a parser knows what it reads from the first token, with no lookahead
  beyond one token.

## Grammar

```text
model     = { subject | claim }
evidence  = { evidence-statement }

subject   = "subject" name "(" [ param { "," param } ] ")" "->" "bool"
param     = name ":" "bool"
claim     = "claim" name ":" name "(" [ pattern { "," pattern } ] ")" "correct"
pattern   = "_" | "true" | "false"

evidence-statement
          = "evidence" name "(" [ bool { "," bool } ] ")" ( "correct" | "wrong" ) "@" number
bool      = "true" | "false"
```

The only type is `bool`.

## Statements

### subject

```mitra
subject  not(a: bool) -> bool
```

A function, task or process under review. Mitra never runs it; it only
weighs evidence about it. Subject names are unique.

### claim

```mitra
claim    not_correct: not(_)    correct
claim    not_on_true: not(true) correct
```

The subject is correct on every input the pattern matches. `_` matches any
value, `true` and `false` match themselves. `not(_)` covers both inputs of
`not`; `not(true)` covers one. The pattern has one position per parameter of
the subject. Claim names are unique.

### evidence

```mitra
evidence not(true)  correct @ 1
evidence not(false) wrong   @ 1
```

The subject was run on these inputs, and its result was judged `correct` or
`wrong`, with a confidence after `@`: `1` is fully believed, `0` is fully
distrusted. **Only `0` and `1` are accepted for now**; any value in between
is an error until a calculus is chosen.

## Evaluation

1. Evidence at `@ 0` is ignored.
2. For each input a claim covers, the evidence at `@ 1` on that input says
   it is *correct*, *wrong*, *nothing* (no such evidence) or *both* (a
   contradiction).
3. If any covered input is *wrong*, the claim **fails @ 1**.
4. Otherwise, if every covered input is *correct*, the claim **holds @ 1**.
5. Otherwise (an input with nothing, or a contradiction), the claim is
   **uncertain @ 0**.

## Output

One line per claim, in model order, names padded to the same width:

```text
not_correct  fails @ 1
not_on_true  holds @ 1
```

`holds`, `fails` or `uncertain` is the verdict. The number after `@` is the
certainty of the verdict: `1` certain, `0` no idea. What it means between 0
and 1 is not defined yet.

## Golden set

`tests/golden/<subject>/<case>/` holds `model.mitra`, `evidence.mitra` and
the hand-written `expected.txt`. `cargo test` checks every case, and also
checks that adding `@ 0` evidence of both kinds on every input of `not` to
any case leaves its result unchanged.

Model for every case in `tests/golden/not/`:

```mitra
subject  not(a: bool) -> bool
claim    not_correct: not(_)    correct
claim    not_on_true: not(true) correct
```

| Case | Evidence | not_correct | not_on_true |
|---|---|---|---|
| 01 | none | uncertain @ 0 | uncertain @ 0 |
| 02 | true correct, false correct | holds @ 1 | holds @ 1 |
| 03 | true correct | uncertain @ 0 | holds @ 1 |
| 04 | false wrong | fails @ 1 | uncertain @ 0 |
| 05 | true correct, false wrong | fails @ 1 | holds @ 1 |
| 06 | true correct, false correct and wrong | uncertain @ 0 | holds @ 1 |
| 07 | true correct and wrong, false correct | uncertain @ 0 | uncertain @ 0 |
| 08 | true wrong, false correct and wrong | fails @ 1 | fails @ 1 |
| 09 | only `@ 0` evidence | uncertain @ 0 | uncertain @ 0 |
| 10 | case 02, plus false wrong `@ 0` | holds @ 1 | holds @ 1 |

All evidence is `@ 1` unless marked.
