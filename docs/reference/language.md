# Mitra language reference

This page describes exactly what the code implements today and nothing
more. When the code changes, this page changes in the same commit. Design
ideas and open questions live in the [general docs](../../README.md#documentation).

## Running Mitra

```sh
mitra init
mitra check [model.mitra evidence.mitra] [--rev <commit>]
mitra eval  [model.mitra evidence.mitra] [--rev <commit>] [-o result.txt]
mitra next  [model.mitra evidence.mitra] [--rev <commit>]
mitra add   '<evidence>' [--source <who>] [--rev <commit>]
mitra ingest <report> [--format libtest|junit] [--rev <commit>]
```

`mitra` (crate `mitra-cli`) is a thin wrapper over the library
`mitra-core`. On an error it prints `file: line N: message` and exits with
status 2.

- `eval` writes one verdict per claim and policy (see Output) to
  `result.txt`, or to standard output without `-o`. The result file may not
  start with `-` or end in `.mitra`, so a store file is never overwritten.
- `check` parses, checks and evaluates both files, prints
  `ok: F functions, C claims, T tests, E evidence`, then any contested
  evidence.
- `next` lists each uncertain claim with the inputs it is missing (its
  domain minus the inputs covered), the premises of a derived claim that
  fail (`derived: premise fails: p`) or are uncertain (`derived: waits on
  p`), and what to add: the evidence lines to write when at most 5 inputs are
  missing on a subject (`evidence f 2 = <output> at 1` for an `=` claim:
  run `f 2` and fill in its output), else how many inputs are missing
  (counted without listing them), or that the set is infinite and only
  evidence over a set can cover it. Inputs with contradicting evidence
  count as missing too.
  It prints `nothing uncertain` when every claim holds or fails.
- `--rev R` on `check`, `eval` and `next` leaves out **stale** evidence:
  evidence whose `rev` attribute is set and differs from `R`. Evidence with
  no `rev` always counts. The number left out is printed on standard error.
  Revisions are compared as text, and any other revision is stale, even
  if the subject did not change.
- Without file names, `check`, `eval` and `next` use the **store**: the nearest
  `.mitra/` folder at or above the current one, with `model.mitra` and
  `evidence.mitra`.
- A `.mitra` that is not a folder is an error.
- `init` creates `.mitra/` with an empty model, an empty evidence file and a
  `.gitattributes` that merges `evidence.mitra` with git's `union` driver.
  It never overwrites an existing store.
- `add` parses the evidence given, checks it against the store's model and
  evidence, and appends it to `evidence.mitra`, printed in normal form with
  its `source` and `rev` attributes. The store must check and evaluate
  without error before, and with the new evidence after; otherwise nothing
  is written. Errors in the new text are reported as `evidence to add: line
  N: …`, counted from its first line. `--source` and `--rev` may each be
  given once, and not when the text already has that attribute.
  The evidence file is only ever appended to, so two branches that add
  evidence merge as the union of their lines.
- `ingest` reads a test report and appends, for each test the model
  declares with `test` (see below), the evidence its result gives, with
  `source "test:<name>"` and the `--rev` given. Formats: `libtest` (the
  text `cargo test` prints, lines `test <path> ... ok|FAILED`, optionally
  followed by a time, as with `--report-time`; ` - should panic` is dropped
  from the path) and `junit` (JUnit XML: `<testcase name="…">` or `name='…'`,
  failed if it holds `<failure` or `<error`, left out if `<skipped`; CDATA,
  `<system-out>` and `<system-err>` are ignored). Without `--format`, a
  report starting with `<` (after a byte order mark) is JUnit. A test's
  name is the last `::` segment of its path. Any other line or element is
  ignored without a message. A test that appears more than once in a report
  is left out, with a note. `--rev` must not be empty.
  `ingest` prints how many results passed and failed, how many evidence
  lines it added, failed tests that gave no evidence, and the tests with no
  `test` declaration.

## Lexical rules

The syntax is a small, fixed subset inspired by Lean 4
([design](../design/lean-syntax.md)). Every command starts with a keyword
(`opaque`, `def`, `claim`, `test`, `profile`, `policy`, `evidence`), so a parser
knows what it reads from the first token. Layout is free: line breaks and
indentation mean nothing.

- `--` starts a comment that runs to the end of the line.
- Names: a letter or `_`, then letters, digits or `_`; keywords are not
  names.
- Numbers: digits, with an optional `.` and digits (`1`, `0.8`). A negative
  integer is `-` then digits; where a single argument is expected it is
  written `(-5)`.
- Strings: `"` … `"` on one line, no escapes.
- Symbols, longest match first: `@[` `:=` `=>` `==` `!=` `<=` `>=` `&&` `||` `(`
  `)` `:` `;` `,` `=` `{` `}` `[` `]` `|` `!` `<` `>` `+` `-` `*` `\` `∀`
  `∈` `∪` `∩` `¬`.
- Keywords: `opaque def claim test evidence profile policy correct at by derive
  if then else let true false Bool Int String Value Icc for strict coverage
  _`.

## Grammar

```ebnf
model      = { command } ;
command    = opaque | def | claim | test | profile | policy ;

opaque     = "opaque" name binders ":" set ;
def        = "def" name binders ":" set ":=" expr ;
binders    = binder { binder } ;
binder     = "(" name { name } ":" set ")" ;
claim      = "claim" name ":" { forall }
             ( "correct" "(" name args ")" | name args "=" expr )
             [ ":=" "by" "derive" "[" name { "," name } "]" ] ;
test       = "test" name ":" { forall } judgment ;
profile    = "profile" name "for" name { "|" region "=>" number } ;
region     = "_" | name "∈" set ;
policy     = "policy" ( "strict" | name ":=" ( "strict" | "coverage" name ) ) ;

evidences  = { evidence } ;
evidence   = [ attributes ] "evidence" { forall } judgment "at" number ;
attributes = "@[" name string { "," name string } "]" ;
judgment   = "correct" "(" name args ")"
           | "¬" ( "correct" "(" name args ")" | name args )
           | name args [ "=" value ] ;

forall     = "∀" name [ "∈" set ] "," ;
args       = { name | arg-value } ;
arg-value  = "true" | "false" | nat | string | "(" "-" nat ")" ;

set        = inter { "∪" inter } ;
inter      = set-atom { ( "∩" | "\" ) set-atom } ;
set-atom   = "Bool" | "Int" | "String" | "Value"
           | "{" [ value { "," value } ] "}"
           | "Icc" int-atom int-atom
           | "(" set ")" ;
int-atom   = nat | "(" "-" nat ")" ;

expr       = "if" expr "then" expr "else" expr
           | "let" name ":=" expr ";" expr
           | or ;
or         = and { "||" and } ;
and        = compare { "&&" compare } ;
compare    = sum { ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) sum } ;
sum        = product { ( "+" | "-" ) product } ;
product    = unary { "*" unary } ;
unary      = ( "!" | "-" ) unary | name { atom } | atom ;
atom       = name | "true" | "false" | nat | string | "(" expr ")" ;

value      = "true" | "false" | [ "-" ] nat | string ;
```

The model file holds `model`; the evidence file holds `evidences`. `nat`
is a number without `.`; `number` may have one. Function application is by
juxtaposition, as in Lean: `add a (negate b)`.

Beyond the grammar, the parser checks:

- every name in `args` is bound by a `∀` before it, and every `∀` name is
  distinct and used exactly once;
- `correct (f args)` and a claim's `f args = e` have at least one argument;
- integers fit in 64 bits (`-9223372036854775808` to
  `9223372036854775807`), and numbers are finite.

### Grammar checks

The crate `crates/mitra-grammar` (not published) checks this grammar:

- `src/mitra.lalrpop` is the grammar above for LALRPOP. LALRPOP refuses
  any conflict, so a clean build proves the grammar is LR(1).
- `src/mitra.ll1` is the same grammar, left-factored. A test computes
  FIRST and FOLLOW sets and checks that it is LL(1).
- `src/gen.rs` writes random valid programs from the grammar. `cargo test`
  runs seeds 0 to 499: each program must be accepted by both grammars and
  by the parser, and parse → print → parse must give the same AST
  (`print_model`, `print_evidence` in mitra-core). Every golden file is
  checked the same way.
- `mitra/parser.mitra` states these results as Mitra claims; the fuzz run
  is their evidence, and `mitra/expected.txt` is the verdict.

`tree-sitter-mitra/` is a tree-sitter grammar for editors, with corpus
tests taken from the golden set. It accepts a little more than the grammar
above (model and evidence commands in one file, `if`/`let` as operands), as
editor grammars usually do.

## Sets

Types are sets of values, and every group of inputs is a set.

| Set | Contains |
|---|---|
| `Bool` | `true`, `false` |
| `Int` | every integer (64-bit literals, but no upper or lower bound) |
| `String` | every string |
| `Value` | everything: booleans, integers, strings and any other value |
| `{1, "a", true}` | exactly these values |
| `Icc 0 100` | the integers from 0 to 100, both included |
| `A ∪ B`, `A ∩ B`, `A \ B` | union, intersection, difference |

As in Lean, `∩` and `\` bind tighter than `∪`, and each applies left to
right; use parentheses to group. Results print in a normal form: `Value`,
`Bool`, `Int`, `String`, `{…}` for single values, `Icc a b` for ranges,
`T \ {…}` for a type minus some values, joined by ` ∪ `. The empty set
prints `{}`. Sets that can only be computed print `Iic b` (up to b) and
`Ici a` (from a); they cannot be written.

## Commands

### opaque: a subject

```lean
opaque not (a : Bool) : Bool
opaque minus (a b : Int) : Int
```

A function, task or process under review. Each parameter has a type (a
set); its inputs are the tuples of the product of those types. The set
after the last `:` is its result type. A subject is **opaque**: Mitra never
runs it, it only weighs evidence about it.

### def: a spec

```lean
def neg (a : Bool) : Bool := !a
def step (x : Int) : Int := if x > 0 then x - 1 else let y := -x; y * 2
def difference (a b : Int) : Int := a - b
```

A **definition**: a total, pure function Mitra evaluates. Its body is an
expression: values, the parameters, `let`, `if`, `!`, `-`, `+`, `-`, `*`,
comparisons (`<` `<=` `>` `>=` on integers, `==` `!=` on any values), `&&`,
`||`, and calls of specs defined above it (so no recursion). Integer
arithmetic is 64-bit and an overflow is an error. A spec is true by
definition: evidence does not apply to it (see Evaluation).

### claim

```lean
claim not_correct : ∀ a, correct (not a)
claim not_spec : ∀ a, not a = neg a
claim id_small : ∀ x ∈ Icc 0 100, correct (id x)
claim not_on_true : correct (not true)
claim minus_shape : ∀ a ∈ Icc 0 2, ∀ b ∈ Icc 0 2, minus a b = add a (negate b)
claim not_not_same : ∀ a, not_not a = same a := by derive [not_spec, not_not_shape, neg_twice]
```

The arguments of the call pick the claim's **domain**, one per parameter: a
name bound by `∀ x,` is the parameter's whole type, by `∀ x ∈ S,` the set
S, and a value is that value alone. The domain is the product of the
arguments, and must lie inside the function's domain. Bound names can be
used in the expression after `=`.

- `correct (f args)`: the subject is correct on every input of the domain,
  as people judge it. Only for subjects.
- `f args = e`: on every input x of the domain, f(x) = e. `e` may call
  specs and subjects. A bare function name `g` as `e` means `g` applied to
  the same inputs.
- `:= by derive [c1, c2, …]`: a **derivation**. Premises must be claims
  defined above. Mitra checks the implication "premises ⇒ claim" when it
  can (see Evaluation), and the claim holds on its whole domain when every
  premise holds and the implication is not refuted.

### evidence

```lean
evidence not true = false at 1
evidence ¬ correct (id 7) at 1
evidence ∀ x ∈ Icc 0 100, correct (id x) at 1
evidence not_not_shape at 1
```

- `f args = v`: the subject was run on one input and returned `v`, which
  must lie in its result type. Used by `=` claims.
- `correct (f args)` / `¬ correct (f args)`: people judged the subject's
  results on these inputs correct (or wrong). A single input is a test; a
  set is evidence for every input in it, such as a proof. Used by `correct`
  claims.
- `c` / `¬ c`, with `c` a claim: the claim holds (or fails) on its whole
  domain, for example from a review or a proof. `c args` restricts it to
  those inputs.

The confidence after `at` is `1` (fully believed) or `0` (fully
distrusted). **Values in between are rejected for now.**

Attributes record where evidence comes from, and do not change verdicts:

```lean
@[source "ci", rev "3f2a1c"] evidence add 1 2 = 3 at 1
```

- `source`: who or what observed it (a CI job, a reviewer, a test).
- `rev`: the revision of the code it was observed on.

Each may appear once; other names are rejected.

### test

```lean
test t_true : correct (not true)
test t_small : ∀ x ∈ Icc 0 10, correct (add x 1)
test t_one : add 1 2 = 3
```

A test of the code and the evidence its result stands for; the judgment is
written as in `evidence`, without a confidence. `mitra ingest` turns a
result into evidence at 1:

- a pass gives the judgment;
- a failure gives `¬ correct (…)` for a `correct (…)` test, and `¬ c …`
  for a `c …` test, when the test is about a single input. Otherwise it
  gives nothing: a failed test over a set says the code is wrong somewhere
  in it, which Mitra cannot express yet; a failed `f args = v` test says
  the output was something else, but not what; and a failed negative test
  (`¬ correct …`, `¬ c …`) proves nothing, since it may have failed for
  another reason.

Test names share the namespace of functions and claims.

### profile

```lean
profile usage for id
  | x ∈ Icc 0 100 => 0.8
  | x ∈ String => 0.1
  | _ => 0.1
```

How much each part of a subject's domain matters, for `coverage` policies.
Regions are matched in order: a value belongs to the first region that
contains it, and `_` takes the rest of the domain. Weights are divided by
their total. The built-in profile `uniform` has a single region, the whole
domain.

### policy

```lean
policy strict
policy by_usage := coverage usage
```

A policy turns the evidence into a belief. Every claim gets one verdict per
policy. With no policy in the model, `strict` is used. The kinds are:

- `strict`: belief is 1 if the claim holds, else 0.
- `coverage profile`: belief is the share of the claim's domain that is
  covered, measured with the profile. A profile for another subject is
  replaced by `uniform`.

## Evaluation

Evidence `at 0` is ignored. Evidence `at 1` on a **spec** is not used
and is reported as `contested`: a spec is a definition, so evidence cannot
disprove it.

For each claim, in model order, with D its domain, Mitra gathers the inputs
where the claim is right and where it is wrong:

1. Evidence on the claim itself (`c` / `¬ c`), within D.
2. For a `correct` claim: `correct` / `¬ correct` evidence on its subject.
3. For an `=` claim, on each input x (every input of D if D is finite,
   otherwise every observed input in D): evaluate both sides, a subject
   call giving each output observed for that input. Equal values make x
   right, different values make x wrong. With no observation, x is not
   judged. A claim on specs alone over a finite domain is thus decided by
   computation.

An input both right and wrong is a **contradiction** and counts on neither
side. Then:

- *refuted* = wrong \ right.
- *covered* = right \ wrong; if the claim has `by derive`, every premise holds
  and the implication is not refuted, *covered* = D \ wrong instead. A
  failing premise only removes the derived support; a direct counterexample
  still refutes the claim.
- **logic** is `fails` if *refuted* is not empty, `holds` if *covered* = D,
  and `uncertain` otherwise.
- **belief** is 0 when the claim fails, and otherwise depends on the policy.

**Checking an implication.** For an `=` claim with `by derive` and a
finite domain, Mitra evaluates both sides on every input, treating each
premise `g args = e` as a definition: a call `g y` with y in the
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
model order, written as Lean structure instances:

```lean
id_correct [by_usage] := { logic := .uncertain, covered := Icc 0 100, domain := Value, belief := 0.8 }
id_correct [by_usage] := { logic := .fails, refuted := {7}, domain := Value, belief := 0 }
minus_spec [strict] := { logic := .holds, covered := Icc 0 2 ×ˢ Icc 0 2, domain := Icc 0 2 ×ˢ Icc 0 2, belief := 1, implication := .checked }
```

`covered` is shown unless the claim fails, in which case `refuted` is shown.
A claim with `by derive` ends with `implication := .checked`, `.unchecked`
or `.refuted`. Sets of tuples print as the domain's own form when equal to
it (`Icc 0 2 ×ˢ Icc 0 2`), as a list of tuples when finite (`{(1, -1)}`),
or as a union of products. Beliefs are rounded to 4 decimals. Then one line
per piece of evidence on a spec:

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
(evidence `at 0`) to any case changes nothing. Evidence is `at 1` unless
marked.

### `not`, under `strict` and `by_usage := coverage uniform`

```lean
opaque not (a : Bool) : Bool
claim not_correct : ∀ a, correct (not a)
claim not_on_true : correct (not true)
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
| 09 | only `at 0` evidence | uncertain | uncertain | 0 |
| 10 | case 02, plus false wrong `at 0` | holds | holds | 1 |

Under `strict`, belief is 1 exactly where the claim holds.

### `id`, under `strict` and `by_usage := coverage usage`

```lean
opaque id (x : Value) : Value
claim id_correct : ∀ x, correct (id x)
claim id_small : ∀ x ∈ Icc 0 100, correct (id x)
profile usage for id
  | x ∈ Icc 0 100 => 0.8
  | x ∈ String => 0.1
  | _ => 0.1
```

| Case | Evidence | id_correct | by_usage | id_small | by_usage |
|---|---|---|---|---|---|
| 01 | none | uncertain, covered {} | 0 | uncertain | 0 |
| 02 | `id 1`, `id 42` correct | uncertain, covered {1, 42} | 0.0158 | uncertain | 0.0198 |
| 03 | correct on `Icc 0 100` | uncertain, covered Icc 0 100 | 0.8 | holds | 1 |
| 04 | correct on `Icc 0 100` and on `String` | uncertain | 0.9 | holds | 1 |
| 05 | `id 1` correct, `id 7` wrong | fails, refuted {7} | 0 | fails, refuted {7} | 0 |
| 06 | correct on `∀ x` | holds, covered Value | 1 | holds | 1 |
| 07 | correct on `Icc 0 100`, `id 5` wrong | uncertain, covered Icc 0 4 ∪ Icc 6 100 | 0.7921 | uncertain | 0.9901 |
| 08 | `id "a"` correct | uncertain, covered {"a"} | 0 | uncertain, covered {} | 0 |
| 09 | only `at 0` evidence | uncertain | 0 | uncertain | 0 |

On `Value`, tests never make `id_correct` hold: only evidence on the whole
domain does (case 06). Under `strict`, belief is 1 exactly where the claim
holds.

### `not_matches`: a subject against a spec, under `strict`

```lean
def neg (a : Bool) : Bool := !a
opaque not (a : Bool) : Bool
claim not_spec : ∀ a, not a = neg a
claim not_on_true : not true = neg true
```

| Case | Evidence | not_spec | not_on_true |
|---|---|---|---|
| 01 | none | uncertain, covered {} | uncertain |
| 02 | `not true = false`, `not false = true` | holds | holds |
| 03 | `not true = false` | uncertain, covered {true} | holds |
| 04 | `not false = false` | fails, refuted {false} | uncertain |
| 05 | `not true = false`, `not false = true` and `= false` | uncertain, covered {true} | holds |
| 06 | `∀ a, correct (not a)` (a judgment, not an output) | uncertain | uncertain |
| 07 | case 02, plus `neg true = true` | holds | holds, and the evidence on `neg` is contested |
| 08 | only `at 0` evidence | uncertain | uncertain |

### `compose`: a derivation mixing specs and subjects, under `strict`

```lean
def neg (a : Bool) : Bool := !a
def same (a : Bool) : Bool := a
opaque not (a : Bool) : Bool
opaque not_not (a : Bool) : Bool
claim neg_twice : ∀ a, same a = neg (neg a)
claim not_spec : ∀ a, not a = neg a
claim not_not_shape : ∀ a, not_not a = not (not a)
claim not_not_same : ∀ a, not_not a = same a := by derive [not_spec, not_not_shape, neg_twice]
```

`neg_twice` is about specs only: it holds by computation in every case.
The implication of `not_not_same` is `checked` in every case.
"Review" is `evidence not_not_shape at 1`.

| Case | Evidence | not_spec | not_not_shape | not_not_same |
|---|---|---|---|---|
| 01 | none | uncertain | uncertain | uncertain |
| 02 | `not` on both inputs, review | holds | holds | holds (derived, no evidence on `not_not`) |
| 03 | `not true = false`, review | uncertain, covered {true} | holds | uncertain |
| 04 | `not true = true`, `not false = true`, review | fails, refuted {true} | holds | uncertain |
| 05 | case 02, plus `not_not true = false` | holds | uncertain, covered {false} | fails, refuted {true} |
| 06 | outputs of `not` and `not_not` on both inputs | holds | holds | holds |
| 07 | only `at 0` evidence | uncertain | uncertain | uncertain |

In case 05 the counterexample on `not_not` also contradicts the review of
`not_not_shape` on `true`, so that premise is no longer known to hold.

### `minus`: tuples and a checked derivation, under `strict`

```lean
def plus (a b : Int) : Int := a + b
def opposite (b : Int) : Int := -b
def difference (a b : Int) : Int := a - b
opaque add (a b : Int) : Int
opaque negate (b : Int) : Int
opaque minus (a b : Int) : Int
claim add_spec : ∀ a ∈ Icc 0 2, ∀ b ∈ Icc (-2) 0, add a b = plus a b
claim negate_spec : ∀ b ∈ Icc 0 2, negate b = opposite b
claim minus_shape : ∀ a ∈ Icc 0 2, ∀ b ∈ Icc 0 2, minus a b = add a (negate b)
claim minus_spec : ∀ a ∈ Icc 0 2, ∀ b ∈ Icc 0 2, minus a b = difference a b := by derive [add_spec, negate_spec, minus_shape]
```

"Leaves" are the outputs of `add` on its 9 inputs and of `negate` on its 3,
plus a review, `evidence minus_shape at 1`. There is never evidence
on `minus` itself.

| Case | Evidence | add_spec | minus_shape | minus_spec | implication |
|---|---|---|---|---|---|
| 01 | none | uncertain | uncertain | uncertain | checked |
| 02 | leaves | holds | holds | holds | checked |
| 03 | leaves without `add 2 (-2)` | uncertain, 8 of 9 covered | holds | uncertain | checked |
| 04 | leaves with `add 1 (-1) = 1` | fails, refuted {(1, -1)} | holds | uncertain | checked |
| 05 | leaves; the shape is `add b (negate a)` | holds | holds | uncertain | refuted |
| 06 | leaves without the review; `by derive [add_spec, negate_spec]` only | holds | uncertain | holds | unchecked |
| 07 | only `at 0` evidence | uncertain | uncertain | uncertain | checked |

`negate_spec` holds whenever the leaves are there. Case 06 shows the risk
of an unchecked implication: `minus_spec` holds only because the author
said so.

