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

- One statement per line, led by a keyword (`subject`, `spec`, `claim`,
  `profile`, `policy`, `evidence`), so a parser knows what it reads from the
  first token. The only exception: the regions of a `profile` follow it on
  indented lines.
- Blank lines are ignored; `#` starts a comment that runs to the end of the
  line.
- Names: a letter or `_`, then letters, digits or `_`; keywords are not
  names.
- Integers and numbers: an optional `-`, digits, an optional `.` (`1`, `-5`,
  `0.8`). A `-` right after a value, a name, `)` or `]` is subtraction.
- Strings: `"` … `"`, no escapes.
- Symbols, longest match first: `->` `==` `!=` `<=` `>=` `&&` `||` `(` `)`
  `:` `,` `@` `=` `{` `}` `[` `]` `|` `&` `\` `!` `<` `>` `+` `-` `*`.
- Keywords: `subject spec claim evidence profile policy correct wrong
  matches from holds fails if then else let in true false bool int string
  any otherwise weight _`.

## Grammar

```ebnf
model      = { statement } ;
statement  = subject | spec | claim | profile | policy ;

subject    = "subject" name params "->" set ;
spec       = "spec" name params "->" set "=" expr ;
params     = "(" name ":" set { "," name ":" set } ")" ;
claim      = "claim" name ":" name pattern
             ( "correct" | "matches" expr )
             [ "from" name { "," name } ] ;
profile    = "profile" name "for" name ":" { NEWLINE INDENT region } ;
region     = ( name "in" set | "otherwise" ) "weight" number ;
policy     = "policy" "strict"
           | "policy" name "=" ( "strict" | "coverage" "(" name ")" ) ;

evidences  = { evidence } ;
evidence   = "evidence" name pattern judgment "@" number ;
judgment   = "correct" | "wrong" | "holds" | "fails" | "->" value ;

pattern    = "(" position { "," position } ")" ;
position   = "_" | value | name [ "in" set ] ;

set        = set-atom { ( "|" | "&" | "\" ) set-atom } ;
set-atom   = "bool" | "int" | "string" | "any"
           | "{" [ value { "," value } ] "}"
           | "[" integer "," integer "]"
           | "(" set ")" ;

expr       = "if" expr "then" expr "else" expr
           | "let" name "=" expr "in" expr
           | or ;
or         = and { "||" and } ;
and        = compare { "&&" compare } ;
compare    = sum { ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) sum } ;
sum        = product { ( "+" | "-" ) product } ;
product    = unary { "*" unary } ;
unary      = ( "!" | "-" ) unary | value | name | call | "(" expr ")" ;
call       = name "(" expr { "," expr } ")" ;

value      = "true" | "false" | integer | string ;
```

The model file holds `model`; the evidence file holds `evidences`.

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
subject  not(a: bool) -> bool
subject  minus(a: int, b: int) -> int
```

A function, task or process under review. Each parameter has a type (a
set); its inputs are the tuples of the product of those types. The set after
`->` is its result type. A subject is **opaque**: Mitra never runs it, it
only weighs evidence about it.

### spec

```mitra
spec     neg(a: bool) -> bool = !a
spec     step(x: int) -> int = if x > 0 then x - 1 else let y = -x in y * 2
spec     difference(a: int, b: int) -> int = a - b
```

A **definition**: a total, pure function Mitra evaluates. Its body is an
expression: values, the parameters, `let`, `if`, `!`, `-`, `+`, `-`, `*`,
comparisons (`<` `<=` `>` `>=` on integers, `==` `!=` on any values), `&&`,
`||`, and calls of specs defined above it (so no recursion). Integer
arithmetic is 64-bit and an overflow is an error. A spec is true by
definition: evidence does not apply to it (see Evaluation).

### claim

```mitra
claim    not_correct:   not(_)       correct
claim    not_spec:      not(_)       matches neg
claim    neg_twice:     same(a)      matches neg(neg(a))
claim    id_small:      id(x in [0, 100]) correct
claim    not_not_same:  not_not(_)   matches same from not_spec, not_not_shape, neg_twice
claim    minus_shape:   minus(a in [0, 2], b in [0, 2]) matches add(a, negate(b))
```

The pattern picks the claim's **domain**, one position per parameter: `_`
or a bare name is the parameter's whole type, a value is that value alone,
`x in <set>` is that set. The domain is the product of the positions, and
must lie inside the function's domain. A name binds that input for the
expression after `matches`.

- `correct`: the subject is correct on every input of the domain, as people
  judge it. Only for subjects.
- `matches e`: on every input x of the domain, f(x) = e. A bare function
  name `g` means `g(x)`. `e` may call specs and subjects.
- `from c1, c2, …`: a **derivation**. Premises must be claims defined
  above. Mitra checks the implication "premises ⇒ claim" when it can (see
  Evaluation), and the claim holds on its whole domain when every premise
  holds and the implication is not refuted.

### evidence

```mitra
evidence not(true)   -> false @ 1
evidence id(7)          wrong @ 1
evidence id(x in [0, 100]) correct @ 1
evidence not_not_shape(_) holds @ 1
```

- `-> v`: the subject was run on one input and returned `v`. Used by
  `matches` claims.
- `correct` / `wrong`: people judged the subject's results on these inputs.
  A single value is a test; a set is evidence for every input in it, such as
  a proof. Used by `correct` claims.
- `holds` / `fails`: the named claim holds (or fails) on each of these
  inputs, for example from a review or a proof.

The confidence after `@` is `1` (fully believed) or `0` (fully distrusted).
**Values in between are rejected for now.**

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

Evidence at `@ 0` is ignored. Evidence at `@ 1` on a **spec** is not used
and is reported as `contested`: a spec is a definition, so evidence cannot
disprove it.

For each claim, in model order, with D its domain, Mitra gathers the inputs
where the claim is right and where it is wrong:

1. `holds` / `fails` evidence on the claim, within D.
2. For a `correct` claim: `correct` / `wrong` evidence on its subject.
3. For a `matches` claim, on each input x (every input of D if D is finite,
   otherwise every observed input in D): evaluate both sides, a subject
   call giving each output observed for that input. Equal values make x
   right, different values make x wrong. With no observation, x is not
   judged. A claim on specs alone over a finite domain is thus decided by
   computation.

An input both right and wrong is a **contradiction** and counts on neither
side. Then:

- *refuted* = wrong \ right.
- *covered* = right \ wrong; if the claim has `from`, every premise holds
  and the implication is not refuted, *covered* = D \ wrong instead. A
  failing premise only removes the derived support; a direct counterexample
  still refutes the claim.
- **logic** is `fails` if *refuted* is not empty, `holds` if *covered* = D,
  and `uncertain` otherwise.
- **belief** is 0 when the claim fails, and otherwise depends on the policy.

**Checking an implication.** For a `matches` claim with `from` and a
finite domain, Mitra evaluates both sides on every input, treating each
premise `g(pattern) matches e` as a definition: a call `g(y)` with y in the
premise's domain is replaced by `e`. Specs are evaluated as usual. The
implication is:

- `checked` if both sides have a value and agree on every input;
- `refuted` if they differ on some input: the premises imply something else,
  so the derivation is not used;
- `unchecked` otherwise (a subject call no premise defines, an infinite
  domain, or a `correct` claim). An unchecked implication is taken as
  stated by the author, and the output says so.

The check depends only on the model, not on evidence.

**Measure under a profile** (for `coverage`). μ(S) = Σ over regions of
weight / total × share, where the share of a region R is:

- if R is finite: |S ∩ R| / |R|, every value weighing the same;
- if R is infinite: 1 if S contains all of R, else 0. A finite sample of an
  infinite region, or any part short of the whole region, weighs nothing.

Then belief = μ(covered) / μ(D). When μ(D) = 0, belief is 1 if the claim
holds and 0 otherwise. Profiles only apply to functions with one parameter;
for several, μ is uniform over the tuples, with the same rule for finite and
infinite domains.

## Output

One line per claim and policy, claims in model order, then policies in
model order:

```text
verdict id_correct [by_usage] = { logic: uncertain, covered: [0, 100], domain: any, belief: 0.8 }
verdict id_correct [by_usage] = { logic: fails, refuted: {7}, domain: any, belief: 0 }
verdict minus_spec [strict] = { logic: holds, covered: ([0, 2], [0, 2]), domain: ([0, 2], [0, 2]), belief: 1, implication: checked }
```

`covered` is shown unless the claim fails, in which case `refuted` is shown.
A claim with `from` ends with `implication: checked|unchecked|refuted`.
Sets of tuples print as the domain's own form when equal to it (`([0, 2],
[0, 2])`), as a list of tuples when finite (`{(1, -1)}`), or as a union of
boxes. Beliefs are rounded to 4 decimals. Then one line per piece of evidence on a
spec:

```text
contested evidence line 4: `neg` is a spec, true by definition: evidence does not apply to it
```

A spec referring to a name it does not bind, a type error, an overflow or a
value outside a function's domain or result type stops evaluation with an
error.

## Golden set

`tests/golden/<set>/<policy>/<case>/` holds `model.mitra`,
`evidence.mitra` and the hand-written `expected.txt`. `cargo test` checks
every case, and checks that adding `tests/golden/<set>/noise.mitra`
(evidence at `@ 0`) to any case changes nothing. Evidence is `@ 1` unless
marked.

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

### `not_matches`: a subject against a spec, under `strict`

```mitra
spec     neg(a: bool) -> bool = !a
subject  not(a: bool) -> bool
claim    not_spec:    not(_)    matches neg
claim    not_on_true: not(true) matches neg
```

| Case | Evidence | not_spec | not_on_true |
|---|---|---|---|
| 01 | none | uncertain, covered {} | uncertain |
| 02 | `not(true) -> false`, `not(false) -> true` | holds | holds |
| 03 | `not(true) -> false` | uncertain, covered {true} | holds |
| 04 | `not(false) -> false` | fails, refuted {false} | uncertain |
| 05 | `not(true) -> false`, `not(false) -> true` and `-> false` | uncertain, covered {true} | holds |
| 06 | `not(_) correct` (a judgment, not an output) | uncertain | uncertain |
| 07 | case 02, plus `neg(true) -> true` | holds | holds, and the evidence on `neg` is contested |
| 08 | only `@ 0` evidence | uncertain | uncertain |

### `compose`: a derivation mixing specs and subjects, under `strict`

```mitra
spec     neg(a: bool) -> bool = !a
spec     same(a: bool) -> bool = a
subject  not(a: bool) -> bool
subject  not_not(a: bool) -> bool

claim    neg_twice:     same(a) matches neg(neg(a))
claim    not_spec:      not(_) matches neg
claim    not_not_shape: not_not(a) matches not(not(a))
claim    not_not_same:  not_not(_) matches same from not_spec, not_not_shape, neg_twice
```

`neg_twice` is about specs only: it holds by computation in every case.
The implication of `not_not_same` is `checked` in every case.
"Review" is `evidence not_not_shape(_) holds`.

| Case | Evidence | not_spec | not_not_shape | not_not_same |
|---|---|---|---|---|
| 01 | none | uncertain | uncertain | uncertain |
| 02 | `not` on both inputs, review | holds | holds | holds (derived, no evidence on `not_not`) |
| 03 | `not(true) -> false`, review | uncertain, covered {true} | holds | uncertain |
| 04 | `not(true) -> true`, `not(false) -> true`, review | fails, refuted {true} | holds | uncertain |
| 05 | case 02, plus `not_not(true) -> false` | holds | uncertain, covered {false} | fails, refuted {true} |
| 06 | outputs of `not` and `not_not` on both inputs | holds | holds | holds |
| 07 | only `@ 0` evidence | uncertain | uncertain | uncertain |

In case 05 the counterexample on `not_not` also contradicts the review of
`not_not_shape` on `true`, so that premise is no longer known to hold.

### `minus`: tuples and a checked derivation, under `strict`

```mitra
spec     plus(a: int, b: int) -> int = a + b
spec     opposite(b: int) -> int = -b
spec     difference(a: int, b: int) -> int = a - b
subject  add(a: int, b: int) -> int
subject  negate(b: int) -> int
subject  minus(a: int, b: int) -> int

claim    add_spec:    add(a in [0, 2], b in [-2, 0]) matches plus
claim    negate_spec: negate(b in [0, 2]) matches opposite
claim    minus_shape: minus(a in [0, 2], b in [0, 2]) matches add(a, negate(b))
claim    minus_spec:  minus(a in [0, 2], b in [0, 2]) matches difference from add_spec, negate_spec, minus_shape
```

"Leaves" are the outputs of `add` on its 9 inputs and of `negate` on its 3,
plus a review, `evidence minus_shape(_, _) holds`. There is never evidence
on `minus` itself.

| Case | Evidence | add_spec | minus_shape | minus_spec | implication |
|---|---|---|---|---|---|
| 01 | none | uncertain | uncertain | uncertain | checked |
| 02 | leaves | holds | holds | holds | checked |
| 03 | leaves without `add(2, -2)` | uncertain, 8 of 9 covered | holds | uncertain | checked |
| 04 | leaves with `add(1, -1) -> 1` | fails, refuted {(1, -1)} | holds | uncertain | checked |
| 05 | leaves; the shape is `add(b, negate(a))` | holds | holds | uncertain | refuted |
| 06 | leaves without the review; `from add_spec, negate_spec` only | holds | uncertain | holds | unchecked |
| 07 | only `@ 0` evidence | uncertain | uncertain | uncertain | checked |

`negate_spec` holds whenever the leaves are there. Case 06 shows the risk
of an unchecked implication: `minus_spec` holds only because the author
said so.

