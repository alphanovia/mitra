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

- One statement per line, led by a keyword (`subject`, `claim`, `profile`,
  `policy`, `evidence`), so a parser knows what it reads from the first
  token. The only exception: the regions of a `profile` follow it on
  indented lines.
- Blank lines are ignored; `#` starts a comment that runs to the end of the
  line.
- Names: a letter or `_`, then letters, digits or `_`.
- Numbers: an optional `-`, digits, an optional `.` (`1`, `-5`, `0.8`).
- Strings: `"` … `"`, no escapes.
- Symbols: `(` `)` `,` `:` `->` `@` `=` `{` `}` `[` `]` `|` `&` `\`.

## Grammar

```text
model     = { subject | claim | profile | policy }
evidence  = { "evidence" name pattern ( "correct" | "wrong" ) "@" number }

subject   = "subject" name "(" name ":" set ")" "->" set
claim     = "claim" name ":" name pattern "correct"
profile   = "profile" name "for" name ":" { NEWLINE INDENT region }
region    = ( name "in" set | "otherwise" ) "weight" number
policy    = "policy" "strict"
          | "policy" name "=" ( "strict" | "coverage" "(" name ")" )

pattern   = "(" ( "_" | value | name "in" set ) ")"
set       = set-atom { ( "|" | "&" | "\" ) set-atom }
set-atom  = "bool" | "int" | "string" | "any"
          | "{" [ value { "," value } ] "}"
          | "[" integer "," integer "]"
          | "(" set ")"
value     = "true" | "false" | integer | string
```

## Sets

Types are sets of values, and every group of inputs is a set.

| Set | Contains |
|---|---|
| `bool` | `true`, `false` |
| `int` | every integer (64-bit literals, but no upper or lower bound) |
| `string` | every string |
| `any` | everything: booleans, integers, strings and any other value |
| `{1, "a", true}` | exactly these values |
| `[0, 100]` | the integers from 0 to 100, both included |
| `A \| B`, `A & B`, `A \ B` | union, intersection, difference |

Set operators have equal precedence and apply left to right; use
parentheses to group. Results print in a normal form: `any`, `bool`, `int`,
`string`, `{…}` for single values, `[a, b]` for ranges, `T \ {…}` for a type
minus some values, joined by ` | `. The empty set prints `{}`.

## Statements

### subject

```mitra
subject  id(x: any) -> any
```

A function, task or process under review, with **one parameter** for now.
The set after `:` is its domain. The set after `->` is read but not used
yet. Mitra never runs a subject.

### claim

```mitra
claim    id_correct: id(_)             correct
claim    id_small:   id(x in [0, 100]) correct
claim    not_on_true: not(true)        correct
```

The subject is correct on every input of the pattern: `_` is the whole
domain, a value is that input alone, `x in <set>` is that set. This set is
the claim's **domain**. It must lie inside the subject's domain.

### evidence

```mitra
evidence id(7)             wrong   @ 1
evidence id(x in [0, 100]) correct @ 1
```

The subject's results on the pattern's inputs were judged `correct` or
`wrong`. A single value is a test; a set is evidence for every input in it,
such as a proof. The confidence after `@` is `1` (fully believed) or `0`
(fully distrusted). **Values in between are rejected for now.**

### profile

```mitra
profile  usage for id:
  x in [0, 100] weight 0.8
  x in string   weight 0.1
  otherwise     weight 0.1
```

How much each part of a subject's domain matters, for `coverage` policies.
Regions are matched in order: a value belongs to the first region that
contains it, and `otherwise` takes the rest of the domain. Weights are
divided by their total. The built-in profile `uniform` has a single region,
the whole domain.

### policy

```mitra
policy   strict
policy   by_usage = coverage(usage)
```

A policy turns the evidence into a belief. Every claim gets one verdict per
policy. With no policy in the model, `strict` is used. The kinds are:

- `strict`: belief is 1 if the claim holds, else 0.
- `coverage(profile)`: belief is the share of the claim's domain that is
  covered, measured with the profile. A profile for another subject is
  replaced by `uniform`.

## Evaluation

For each claim, with D its domain:

1. Evidence at `@ 0` is ignored.
2. *Correct* is the union of the sets of `correct @ 1` evidence on the
   subject, within D; *wrong* likewise for `wrong @ 1`.
3. An input in both is a **contradiction** and counts on neither side:
   *covered* = correct \ wrong, *refuted* = wrong \ correct.
4. **logic** is `fails` if *refuted* is not empty, `holds` if *covered* = D,
   and `uncertain` otherwise.
5. **belief** is 0 when the claim fails, and otherwise depends on the policy.

**Measure under a profile** (for `coverage`). μ(S) = Σ over regions of
weight / total × share, where the share of a region R is:

- if R is finite: |S ∩ R| / |R|, every value weighing the same;
- if R is infinite: 1 if S contains all of R, else 0. A finite sample of an
  infinite region, or any part short of the whole region, weighs nothing.

Then belief = μ(covered) / μ(D). When μ(D) = 0, belief is 1 if the claim
holds and 0 otherwise.

## Output

One line per claim and policy, claims in model order, then policies in
model order:

```text
verdict id_correct [by_usage] = { logic: uncertain, covered: [0, 100], domain: any, belief: 0.8 }
verdict id_correct [by_usage] = { logic: fails, refuted: {7}, domain: any, belief: 0 }
```

`covered` is shown unless the claim fails, in which case `refuted` is shown.
Beliefs are rounded to 4 decimals.

## Golden set

`tests/golden/<subject>/<policy>/<case>/` holds `model.mitra`,
`evidence.mitra` and the hand-written `expected.txt`. `cargo test` checks
every case, and checks that adding `tests/golden/<subject>/noise.mitra`
(`correct @ 0` and `wrong @ 0` on the whole domain) to any case changes
nothing. Evidence is `@ 1` unless marked.

### `not`, under `strict` and `by_usage = coverage(uniform)`

```mitra
subject  not(a: bool) -> bool
claim    not_correct: not(_)    correct
claim    not_on_true: not(true) correct
```

| Case | Evidence | not_correct | not_on_true | belief by_usage (not_correct) |
|---|---|---|---|---|
| 01 | none | uncertain, covered {} | uncertain, covered {} | 0 |
| 02 | true correct, false correct | holds | holds | 1 |
| 03 | true correct | uncertain, covered {true} | holds | 0.5 |
| 04 | false wrong | fails, refuted {false} | uncertain | 0 |
| 05 | true correct, false wrong | fails, refuted {false} | holds | 0 |
| 06 | true correct, false correct and wrong | uncertain, covered {true} | holds | 0.5 |
| 07 | true correct and wrong, false correct | uncertain, covered {false} | uncertain | 0.5 |
| 08 | true wrong, false correct and wrong | fails, refuted {true} | fails, refuted {true} | 0 |
| 09 | only `@ 0` evidence | uncertain | uncertain | 0 |
| 10 | case 02, plus false wrong `@ 0` | holds | holds | 1 |

Under `strict`, belief is 1 exactly where the claim holds.

### `id`, under `strict` and `by_usage = coverage(usage)`

```mitra
subject  id(x: any) -> any
claim    id_correct: id(_)             correct
claim    id_small:   id(x in [0, 100]) correct

profile  usage for id:
  x in [0, 100] weight 0.8
  x in string   weight 0.1
  otherwise     weight 0.1
```

| Case | Evidence | id_correct | by_usage | id_small | by_usage |
|---|---|---|---|---|---|
| 01 | none | uncertain, covered {} | 0 | uncertain | 0 |
| 02 | `id(1)`, `id(42)` correct | uncertain, covered {1, 42} | 0.0158 | uncertain | 0.0198 |
| 03 | correct on `[0, 100]` | uncertain, covered [0, 100] | 0.8 | holds | 1 |
| 04 | correct on `[0, 100]` and on `string` | uncertain | 0.9 | holds | 1 |
| 05 | `id(1)` correct, `id(7)` wrong | fails, refuted {7} | 0 | fails, refuted {7} | 0 |
| 06 | correct on `_` | holds, covered any | 1 | holds | 1 |
| 07 | correct on `[0, 100]`, `id(5)` wrong | uncertain, covered [0, 4] \| [6, 100] | 0.7921 | uncertain | 0.9901 |
| 08 | `id("a")` correct | uncertain, covered {"a"} | 0 | uncertain, covered {} | 0 |
| 09 | only `@ 0` evidence | uncertain | 0 | uncertain | 0 |

On `any`, tests never make `id_correct` hold: only evidence on the whole
domain does (case 06). Under `strict`, belief is 1 exactly where the claim
holds.
