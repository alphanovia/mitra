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
- `next` prints first the evidence that can't all be true (see Evaluation:
  contradictions, uniformity) and each root cause, then
  each claim that fails with the inputs refuted. Those on a subject that
  uses nothing failing come first, as the likely root causes; the others
  are followed by `look first at …` (see Evaluation, suspects), and so is
  a claim that holds on a suspect subject. Then it lists each uncertain claim with the inputs it is missing (its
  domain minus the inputs covered), the premises of a derived claim that
  fail (`derived: premise fails: p`) or are uncertain (`derived: waits on
  p`), and what to add: the evidence lines to write when at most 5 inputs are
  missing on a subject (`evidence f 2 = <output> at 1` for an `=` claim:
  run `f 2` and fill in its output), else how many inputs are missing
  (counted without listing them), or that the set is infinite and only
  evidence over a set can cover it. Inputs with contradicting evidence
  count as missing too. A missing set longer than 100 characters is summed
  up. When the subject has a `uniform` claim, `next` lists instead the
  classes with no passing input, and says if the `uniform` claim fails or
  is not vouched for. For an uncertain `uniform` claim it prints the
  evidence line a reviewer adds.
  It prints `nothing uncertain` when it has nothing to say.
- `--rev R` on `check`, `eval` and `next` counts only evidence recorded at
  `R`: evidence with another `rev`, or with none, is **stale** and left out
  (otherwise an old pass would outlive a newer failure that is left out).
  The number left out is printed on standard error.
  Revisions are compared as text, and any other revision is stale, even
  if the subject did not change.
- Without file names, `check`, `eval` and `next` use the **store**: the nearest
  `.mitra/` folder at or above the current one, with `model.mitra` and
  `evidence.mitra`.
- In a store, the model is `model.mitra` followed by the **Rust model
  lines**: in every `.rs` file under `src/` and `tests/` beside `.mitra/`
  (files in path order), each comment line `/// mitra: …` or `// mitra: …`
  gives one model line, the text after `mitra:`. The comment must be the
  whole line, in code: text inside a string or a `/* */` comment is not
  read. Symbolic links are not followed. Errors there name the Rust file
  and line. Consecutive comment lines form a block. An `opaque f` or
  `test t` must start its line, and the first item below its block must be
  `fn f` or `fn t` (attributes, comments, blank lines and `pub`, `const`,
  `async`, `unsafe`, `extern` may come before `fn`), or it is an error:

  ```rust
  /// mitra: opaque flip (a : Bool) : Bool
  /// mitra: claim flip_correct : forall a, correct (flip a)
  pub fn flip(a: bool) -> bool { !a }

  // mitra: test t_true : correct (flip true)
  #[test]
  fn t_true() { assert!(!flip(true)) }
  ```

  `add` and `ingest` read the same model.
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
  is left out, with a note.
- A revision given with `--rev` must not be empty, start with `-`, or hold
  quotes or line breaks.
  `ingest` prints how many results passed and failed, how many evidence
  lines it added, failed tests that gave no evidence, and the tests with no
  `test` declaration.

## Lexical rules

The syntax is a small, fixed subset inspired by Lean 4
([design](../design/lean-syntax.md)), written in ASCII. Every command starts
with a keyword (`opaque`, `def`, `claim`, `test`, `profile`, `policy`,
`evidence`), so a parser knows what it reads from the first token. Layout is
free: line breaks and indentation mean nothing.

- `--` starts a comment that runs to the end of the line.
- Names: a letter or `_`, then letters, digits or `_`; keywords are not
  names.
- Numbers: digits (`nat`), or digits, `.` and digits (`dec`: `0.8`, `1.0`).
  A negative number is `-` then digits (spaces allowed between them); where a single argument is expected
  it is written `(-5)` or `(-1.5)`. `1..5` reads as `1`, `..`, `5`.
- Strings: `"` … `"` on one line, no escapes.
- Symbols, longest match first: `@[` `..=` `:=` `=>` `==` `!=` `<=` `>=`
  `&&` `||` `..` `(` `)` `:` `;` `,` `=` `{` `}` `[` `]` `|` `&` `!` `<` `>`
  `+` `-` `*` `\`.
- Keywords: `opaque def claim test evidence profile policy correct at by
  derive if then else let true false forall in not uniform uses only Bool Int Int8 Int16
  Int32 Int64 UInt8 UInt16 UInt32 Float64 String Value Real for strict
  coverage _`. `Real` is reserved.
- Lean's Unicode symbols are accepted as aliases: `∀` for `forall`, `∈` for
  `in`, `¬` for `not`, `∪` for `|`, `∩` for `&`. Mitra prints ASCII.

## Grammar

```ebnf
model      = { command } ;
command    = opaque | def | claim | test | profile | policy ;

opaque     = "opaque" name binders ":" set ;
def        = "def" name binders ":" set ":=" expr ;
binders    = binder { binder } ;
binder     = "(" name { name } ":" set ")" ;
claim      = "claim" name ":" ( "uniform" name "by" name
             | [ "only" ] "uses" name "[" name { "," name } "]"
             | { forall }
               ( "correct" "(" name args ")" | name args "=" expr )
               [ ":=" "by" "derive" "[" name { "," name } "]" ] ) ;
test       = "test" name ":" { forall } judgment ;
profile    = "profile" name "for" name { "|" region "=>" number } ;
region     = "_" | name "in" set ;
policy     = "policy" ( "strict" | name ":="
             ( "strict" | "coverage" ( name | "uniform" ) ) ) ;

evidences  = { evidence } ;
evidence   = [ attributes ] "evidence" { forall } judgment "at" number ;
attributes = "@[" name string { "," name string } "]" ;
judgment   = "correct" "(" name args ")"
           | "not" ( "correct" "(" name args ")" | name args )
           | name args [ "=" value ] ;

forall     = "forall" name [ "in" set ] "," ;
args       = { name | arg-value } ;
arg-value  = "true" | "false" | number | string | "(" "-" number ")" ;

set        = inter { "|" inter } ;
inter      = set-atom { ( "&" | "\" ) set-atom } ;
set-atom   = "Bool" | "Int" | "Int8" | "Int16" | "Int32" | "Int64"
           | "UInt8" | "UInt16" | "UInt32" | "Float64" | "String" | "Value"
           | "{" [ value { "," value } ] "}"
           | "(" set ")"
           | bound ( "..=" bound | ".." [ bound ] )
           | ( "..=" | ".." ) bound ;
bound      = [ "-" ] nat ;

expr       = "if" expr "then" expr "else" expr
           | "let" name ":=" expr ";" expr
           | or ;
or         = and { "||" and } ;
and        = compare { "&&" compare } ;
compare    = sum { ( "==" | "!=" | "<" | "<=" | ">" | ">=" ) sum } ;
sum        = product { ( "+" | "-" ) product } ;
product    = unary { "*" unary } ;
unary      = ( "!" | "-" ) unary | name { atom } | atom ;
atom       = name | "true" | "false" | number | string | "(" expr ")" ;

value      = "true" | "false" | [ "-" ] number | string ;
number     = nat | dec ;
```

The model file holds `model`; the evidence file holds `evidences`.
Function application is by juxtaposition, as in Lean: `add a (negate b)`.

Beyond the grammar, the parser checks:

- every name in `args` is bound by a `forall` before it, and every `forall`
  name is distinct and used exactly once;
- `correct (f args)` and a claim's `f args = e` have at least one argument;
- integer values fit in 64 bits (`-9223372036854775808` to
  `9223372036854775807`); range bounds may be larger, up to 2^126 either
  way, since `Int` is unbounded, but inputs past 64 bits can't be listed
  or computed with;
- `dec` numbers are finite 64-bit floats;
- a range has an upper bound after `..=`, and `..` alone is not a range;
  ranges are over integers only.

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
| `Int` | every integer, unbounded (ℤ) |
| `Int8` … `Int64`, `UInt8` … `UInt32` | the fixed-width integers |
| `Float64` | every 64-bit float, including NaN, the infinities and `-0.0` |
| `String` | every string |
| `Value` | everything: booleans, integers, floats, strings and any other value |
| `{1, 2.5, "a", true}` | exactly these values (`1` is an integer, `1.0` a float) |
| `0..=100`, `0..100` | the integers from 0 to 100, with or without 100 |
| `5..`, `..=5`, `..5` | the integers from 5 up, up to 5, below 5 |
| `A \| B`, `A & B`, `A \ B` | union, intersection, difference |

`&` and `\` bind tighter than `|`, and each applies left to right; use
parentheses to group. `Int` is infinite, so a claim over all of it can't be
covered by tests, only by evidence over a set (a proof). Mitra computes
with 64-bit integers for now: an integer value outside them is an error,
never a wrap.

Results print in a normal form: `Value`, `Bool`, `Int`, `Float64`,
`String`, `{…}` for single values, `a..=b`, `a..` and `..=b` for ranges (a
range equal to a fixed-width type prints its name, such as `Int8`),
`T \ {…}` for a type minus some values, joined by ` | `. The empty set
prints `{}`.

Floats in a set are compared exactly: `NaN` is one value, and `0.0` and
`-0.0` are two. NaN and the infinities have no literal: they can only come
out of float arithmetic in a spec.

## Commands

### opaque: a subject

```lean
opaque flip (a : Bool) : Bool
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
comparisons (`<` `<=` `>` `>=` on numbers, `==` `!=` on any values), `&&`,
`||`, and calls of specs defined above it (so no recursion). Arithmetic and
comparisons need two integers or two floats; there is no conversion.
Integer arithmetic is 64-bit and an overflow is an error. Float arithmetic
and `<` `<=` `>` `>=` follow IEEE 754; `==` and `!=` compare values exactly
(`NaN == NaN` is true, `0.0 == -0.0` is false). A spec is true by
definition: evidence does not apply to it (see Evaluation).

### claim

```lean
claim flip_correct : forall a, correct (flip a)
claim flip_spec : forall a, flip a = neg a
claim id_small : forall x in 0..=100, correct (id x)
claim flip_on_true : correct (flip true)
claim minus_shape : forall a in 0..=2, forall b in 0..=2, minus a b = add a (negate b)
claim flip_not_same : forall a, flip_not a = same a := by derive [flip_spec, flip_not_shape, neg_twice]
```

The arguments of the call pick the claim's **domain**, one per parameter: a
name bound by `forall x,` is the parameter's whole type, by `forall x in S,` the set
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
- `uniform f by g`: a **uniformity claim**, the assumption a developer
  makes when a few tests are taken to cover a function. `g` is a spec with
  the same parameters as the subject `f`; its values are the **classes**
  of inputs, and the claim says `f` behaves the same on every input of a
  class: right on all of them or wrong on all of them. So one judged input
  stands for its class (see Evaluation).

  ```lean
  opaque div (a b : Int) : Int
  def div_class (a b : Int) : {"zero", "rest"} := if b == 0 then "zero" else "rest"
  claim div_uniform : uniform div by div_class
  ```

  `g`'s result type has at most 64 values, and a subject has at most one
  uniformity claim. Mitra can't check it: it counts once evidence vouches
  for it, `evidence div_uniform at 1`, for example from a review that
  the classes follow the branches of the code.
- `uses f [g, h]`: the subject `f` calls the subjects `g` and `h`.
  `only uses f [g, h]` adds that it calls no other subject. Neither
  changes a verdict; they tell Mitra where to look when `f` fails (see
  Evaluation, suspects). Like uniformity, Mitra can't check them:
  evidence vouches for them (`evidence c at 1`) or denies them (`not c`).
  Without evidence, a claim may raise suspicion but never trust: an
  unvouched `uses` or `only uses` still gives suspects, unless denied,
  and only a vouched `only uses` gives a root cause.

  ```lean
  claim dempster_uses : only uses dempster [is_valid, conflict]
  ```

A `uniform` or `uses` claim can't be a premise of `by derive`.

### evidence

```lean
evidence flip true = false at 1
evidence not correct (id 7) at 1
evidence forall x in 0..=100, correct (id x) at 1
evidence flip_not_shape at 1
```

- `f args = v`: the subject was run on one input and returned `v`, which
  must lie in its result type. Used by `=` claims.
- `correct (f args)` / `not correct (f args)`: people judged the subject's
  results on these inputs correct (or wrong). A single input is a test; a
  set is evidence for every input in it, such as a proof. Used by `correct`
  claims.
- `c` / `not c`, with `c` a claim: the claim holds (or fails) on its whole
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
test t_true : correct (flip true)
test t_small : forall x in 0..=10, correct (add x 1)
test t_one : add 1 2 = 3
```

A test of the code and the evidence its result stands for; the judgment is
written as in `evidence`, without a confidence. `mitra ingest` turns a
result into evidence at 1:

- a pass gives the judgment;
- a failure gives `not correct (…)` for a `correct (…)` test, and `not c …`
  for a `c …` test, when the test is about a single input. Otherwise it
  gives nothing: a failed test over a set says the code is wrong somewhere
  in it, which Mitra cannot express yet; a failed `f args = v` test says
  the output was something else, but not what. If the subject's result
  type is `Bool`, it gives `f args =` the other value. Otherwise it gives
  `not c args` for each `=` claim `c` on `f` whose other side is `v` on
  that input (the claim is wrong there), and nothing if there is none. A failed negative test
  (`not correct …`, `not c …`) proves nothing, since it may have failed for
  another reason.

Test names share the namespace of functions and claims.

### profile

```lean
profile usage for id
  | x in 0..=100 => 0.8
  | x in String => 0.1
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

1. Evidence on the claim itself (`c` / `not c`), within D.
2. For a `correct` claim: `correct` / `not correct` evidence on its subject.
3. For an `=` claim, on each input x (every input of D if D is finite,
   otherwise every observed input in D): evaluate both sides, a subject
   call giving each output observed for that input. Equal values make x
   right, different values make x wrong. With no observation, x is not
   judged. A claim on specs alone over a finite domain is thus decided by
   computation.

Evidence on a claim (`c` or `not c`) speaks of the claim's domain only,
whatever inputs it names.

An input both right and wrong is a **contradiction**: the pass and the
failure can't both be true. So are two different outputs for one input,
and an input judged `correct` and `not correct`, on any subject, with or
without a claim. A contradicted input counts on neither side, for every
claim saying the same of the subject, and each pair of pieces of evidence
that contradict is reported once as a conflict, with the inputs they
share. Then:

- *refuted* = wrong \ right.
- *covered* = right \ wrong; if the claim has `by derive`, every premise holds
  and the implication is not refuted, *covered* = D \ wrong instead. A
  failing premise only removes the derived support; a direct counterexample
  still refutes the claim.
- **logic** is `fails` if *refuted* is not empty, `holds` if *covered* = D,
  and `uncertain` otherwise.
- **belief** is 0 when the claim fails, and otherwise depends on the policy.

**Uniformity.** Let `u : uniform f by g` be a claim. Mitra looks at the
single inputs judged for `f`, one **statement** at a time. `correct` is
one statement. Each `= e` is another: claims on `f` with the same `e`
share their inputs, whatever their domains. The inputs of a statement are
those of the evidence in rules 1 to 3 above, also outside a claim's
domain (an observed output is compared with `e` wherever the claim's
arguments allow). Evidence over a set counts input by input when the set
has at most 10 000 inputs. A contradicted input is neither a pass nor a
failure, and nothing is assumed for its class. The class of
an input x is `g x`.

- **Conflict.** If, for one statement, a class holds an input that passed
  and one that failed, `u`, the pass and the failure can't all be true. `u` **fails**, whatever
  vouches for it; its *refuted* shows the two inputs, and the output ends
  with the conflict and both pieces of evidence. Mitra does not choose
  which of the three is wrong.
- Otherwise `u` holds if evidence on `u` covers its whole domain, like any
  claim, and is uncertain if not.
- While `u` **holds**, for every other claim c on `f`: an input of D in a
  class with a passing input is **assumed**, and counts as covered for
  c's logic and belief; an input in a class with a failing input is
  refuted. On a finite D (up to 10 000 inputs) this is done input by
  input. On a larger D the classes Mitra can't place: c holds once every
  value of `g`'s result type has a passing input, and a failing class
  refutes only its own input.
- While `u` fails or is uncertain, it changes no other verdict.
- Nothing is assumed for a claim whose statement has a failure over a set
  too large to read input by input (its class is unknown), nor for an `=`
  claim whose other side calls a subject: `u` says nothing of that one.
- Mitra trusts `g`'s result type on inputs it never computes: a value no
  input gives is a class no test can fill.

**Suspects.** For each subject `f`, Mitra follows the `uses` claims that
do not fail, from `f` to what it uses, and on from there. Each subject
reached that has a failing claim (other than a `uses` claim) is a
**suspect** for `f`, printed after the verdicts. Verdicts do not change.

**Root cause.** A subject `f` with a failing `correct` or `=` claim, no
suspect, and an `only uses` claim that holds (evidence covers it) is a
root cause: the fault is in `f` itself. There is none when a subject it
uses is judged `not correct` on some input, even outside every claim.
The subjects it uses are those of all its `uses` claims that do not
fail. If some `correct` or `=` claim on one of them does not hold, or
one of them has no such claim, the output adds that the fault may be
where no claim covers them.

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
id_correct [by_usage] := { logic := .uncertain, covered := 0..=100, domain := Value, belief := 0.8 }
id_correct [by_usage] := { logic := .fails, refuted := {7}, domain := Value, belief := 0 }
minus_spec [strict] := { logic := .holds, covered := (0..=2, 0..=2), domain := (0..=2, 0..=2), belief := 1, implication := .checked }
```

`covered` is shown unless the claim fails, in which case `refuted` is shown.
`covered` holds what evidence shows directly. When a uniformity claim that
holds adds to it, `assumed := {"rest"} by div_uniform` follows: the classes
with a passing input (on a finite domain, only those it meets).
A claim with `by derive` ends with `implication := .checked`, `.unchecked`
or `.refuted`. Sets of tuples print as the domain's own form when equal to
it (`(0..=2, 0..=2)`), as a list of tuples when finite (`{(1, -1)}`),
or as a union of products, each written as a tuple of sets:
`(0..=2, {0}) | (3..=4, Int)`. Beliefs are rounded to 4 decimals. Then one line
per piece of evidence on a spec:

```text
contested evidence line 4: `neg` is a spec, true by definition: evidence does not apply to it
```

Then each conflict, with the evidence lines as Mitra reads them: first
the contradictions on an input, then those within a class:

```text
conflict: can't both be true, on {false} of `flip`:
  evidence line 3: evidence correct (flip false) at 1
  evidence line 4: evidence not correct (flip false) at 1
conflict: `div_uniform` can't hold with both, in class "rest" of `div_class`:
  evidence line 4: evidence correct (div 6 3) at 1
  evidence line 5: evidence not correct (div 4 2) at 1
```

Then each suspect and each root cause:

```text
suspect: `dempster` uses `conflict`, where conflict_spec_ok fails: look there first
root cause: `quarter` fails and only uses `half`, where nothing fails: the fault is in `quarter` itself
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

### `flip`, under `strict` and `by_usage := coverage uniform`

```lean
opaque flip (a : Bool) : Bool
claim flip_correct : forall a, correct (flip a)
claim flip_on_true : correct (flip true)
```

| Case | Evidence | flip_correct | flip_on_true | belief by_usage (flip_correct) |
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
claim id_correct : forall x, correct (id x)
claim id_small : forall x in 0..=100, correct (id x)
profile usage for id
  | x in 0..=100 => 0.8
  | x in String => 0.1
  | _ => 0.1
```

| Case | Evidence | id_correct | by_usage | id_small | by_usage |
|---|---|---|---|---|---|
| 01 | none | uncertain, covered {} | 0 | uncertain | 0 |
| 02 | `id 1`, `id 42` correct | uncertain, covered {1, 42} | 0.0158 | uncertain | 0.0198 |
| 03 | correct on `0..=100` | uncertain, covered 0..=100 | 0.8 | holds | 1 |
| 04 | correct on `0..=100` and on `String` | uncertain | 0.9 | holds | 1 |
| 05 | `id 1` correct, `id 7` wrong | fails, refuted {7} | 0 | fails, refuted {7} | 0 |
| 06 | correct on `forall x` | holds, covered Value | 1 | holds | 1 |
| 07 | correct on `0..=100`, `id 5` wrong | uncertain, covered 0..=4 \| 6..=100 | 0.7921 | uncertain | 0.9901 |
| 08 | `id "a"` correct | uncertain, covered {"a"} | 0 | uncertain, covered {} | 0 |
| 09 | only `at 0` evidence | uncertain | 0 | uncertain | 0 |

On `Value`, tests never make `id_correct` hold: only evidence on the whole
domain does (case 06). Under `strict`, belief is 1 exactly where the claim
holds.

### `flip_matches`: a subject against a spec, under `strict`

```lean
def neg (a : Bool) : Bool := !a
opaque flip (a : Bool) : Bool
claim flip_spec : forall a, flip a = neg a
claim flip_on_true : flip true = neg true
```

| Case | Evidence | flip_spec | flip_on_true |
|---|---|---|---|
| 01 | none | uncertain, covered {} | uncertain |
| 02 | `flip true = false`, `flip false = true` | holds | holds |
| 03 | `flip true = false` | uncertain, covered {true} | holds |
| 04 | `flip false = false` | fails, refuted {false} | uncertain |
| 05 | `flip true = false`, `flip false = true` and `= false` | uncertain, covered {true} | holds |
| 06 | `forall a, correct (flip a)` (a judgment, not an output) | uncertain | uncertain |
| 07 | case 02, plus `neg true = true` | holds | holds, and the evidence on `neg` is contested |
| 08 | only `at 0` evidence | uncertain | uncertain |

### `compose`: a derivation mixing specs and subjects, under `strict`

```lean
def neg (a : Bool) : Bool := !a
def same (a : Bool) : Bool := a
opaque flip (a : Bool) : Bool
opaque flip_not (a : Bool) : Bool
claim neg_twice : forall a, same a = neg (neg a)
claim flip_spec : forall a, flip a = neg a
claim flip_not_shape : forall a, flip_not a = flip (flip a)
claim flip_not_same : forall a, flip_not a = same a := by derive [flip_spec, flip_not_shape, neg_twice]
```

`neg_twice` is about specs only: it holds by computation in every case.
The implication of `flip_not_same` is `checked` in every case.
"Review" is `evidence flip_not_shape at 1`.

| Case | Evidence | flip_spec | flip_not_shape | flip_not_same |
|---|---|---|---|---|
| 01 | none | uncertain | uncertain | uncertain |
| 02 | `flip` on both inputs, review | holds | holds | holds (derived, no evidence on `flip_not`) |
| 03 | `flip true = false`, review | uncertain, covered {true} | holds | uncertain |
| 04 | `flip true = true`, `flip false = true`, review | fails, refuted {true} | holds | uncertain |
| 05 | case 02, plus `flip_not true = false` | holds | uncertain, covered {false} | fails, refuted {true} |
| 06 | outputs of `flip` and `flip_not` on both inputs | holds | holds | holds |
| 07 | only `at 0` evidence | uncertain | uncertain | uncertain |

In case 05 the counterexample on `flip_not` also contradicts the review of
`flip_not_shape` on `true`, so that premise is no longer known to hold.

### `minus`: tuples and a checked derivation, under `strict`

```lean
def plus (a b : Int) : Int := a + b
def opposite (b : Int) : Int := -b
def difference (a b : Int) : Int := a - b
opaque add (a b : Int) : Int
opaque negate (b : Int) : Int
opaque minus (a b : Int) : Int
claim add_spec : forall a in 0..=2, forall b in -2..=0, add a b = plus a b
claim negate_spec : forall b in 0..=2, negate b = opposite b
claim minus_shape : forall a in 0..=2, forall b in 0..=2, minus a b = add a (negate b)
claim minus_spec : forall a in 0..=2, forall b in 0..=2, minus a b = difference a b := by derive [add_spec, negate_spec, minus_shape]
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

### `div`: a uniformity claim, under `strict`

```lean
opaque div (a b : Int) : Int
def div_class (a b : Int) : {"zero", "rest"} := if b == 0 then "zero" else "rest"
claim div_correct : forall a, forall b, correct (div a b)
claim div_small : forall a in 0..=1, forall b in 1..=2, correct (div a b)
claim div_uniform : uniform div by div_class
```

| Case | Evidence | div_correct | div_small | div_uniform |
|---|---|---|---|---|
| 01 | `div 6 3` and `div 1 0` pass; nothing vouches | uncertain, covered 2 inputs | uncertain | uncertain |
| 02 | vouched; `div 6 3` passes | uncertain, assumed "rest" | holds, assumed "rest" | holds |
| 03 | vouched; `div 6 3` and `div 1 0` pass | holds, assumed "rest", "zero" | holds | holds |
| 04 | vouched; `div 6 3` passes, `div 4 2` fails | fails, refuted {(4, 2)} | uncertain | fails, with a conflict |
| 05 | vouched; `div 4 2` fails | fails, refuted {(4, 2)} | fails, refuted its whole domain | holds |
| 06 | vouched; `div 6 3` passes; `div` fails somewhere on `(10.., 10..)` | fails | uncertain: nothing assumed | holds |
| 07 | vouched; `div 6 3` passes; `not div_correct 4 2` | fails, refuted {(4, 2)} | uncertain | fails, with a conflict |
| 08 | vouched; `div 6 3` passes and fails; `div 4 2` passes | uncertain, covered {(4, 2)}; a conflict on (6, 3) | uncertain: class "rest" not relied on | holds |
| 09 | vouched; `div_small` reviewed | uncertain, assumed "rest" | holds | holds |
| 10 | vouched; `div 6 3` fails; `div_correct 6 3` reviewed | uncertain; a conflict on (6, 3) | uncertain: the failure stands for nothing | holds |

### `uses`: suspects, under `strict`

```lean
opaque half (x : Int) : Int
opaque quarter (x : Int) : Int
opaque eighth (x : Int) : Int
claim half_correct : forall x in 0..=1, correct (half x)
claim quarter_correct : forall x in 0..=1, correct (quarter x)
claim eighth_correct : forall x in 0..=1, correct (eighth x)
claim quarter_uses : uses quarter [half]
claim eighth_uses : uses eighth [quarter]
```

| Case | Evidence | Suspects |
|---|---|---|
| 01 | `half 1` and `quarter 1` fail; `eighth` passes | `half` for `quarter`; `half` and `quarter` for `eighth`, which holds |
| 02 | the same failures; `not quarter_uses` | only `quarter` for `eighth` |

Cases 03 to 05 have `claim quarter_uses : only uses quarter [half]` and a
failing `quarter`:

| Case | Evidence | Root cause |
|---|---|---|
| 03 | vouched; `half` holds | `quarter` itself |
| 04 | not vouched; `half` holds | none |
| 05 | vouched; `half` tested on 0 only | `quarter` itself, or where no claim covers `half` |
| 06 | vouched; `half` holds, but fails on 7, outside its claim | none |
| 07 | vouched; a second claim `uses quarter [eighth]`, `eighth` without a claim | `quarter` itself, or where no claim covers them |

### `numbers`: integer types, ranges and floats, under `strict`

```lean
opaque inc (x : Int64) : Int64
def halve (x : Float64) : Float64 := x * 0.5
opaque half (x : Float64) : Float64
claim inc_correct : forall x, correct (inc x)
claim inc_small : forall x in 0..100, correct (inc x)
claim half_spec : forall x in {1.0, -3.0}, half x = halve x
```

| Case | Evidence | inc_correct | inc_small | half_spec |
|---|---|---|---|---|
| 01 | `inc` on `0..=99`; `half 1.0 = 0.5`, `half (-3.0) = -1.5` | uncertain, covered 0..=99 of Int64 | holds | holds |
| 02 | `half (-3.0) = -1.0`; `inc` on `-5..0` at 0 | uncertain | uncertain | fails, refuted {-3.0} |
