# Mitra language reference

This page describes exactly what the code implements today and nothing
more. When the code changes, this page changes in the same commit. Design
ideas and open questions live in the [general docs](../../README.md#documentation).

A reworked syntax (`subject`, `boundary`, `fn`, evidence blocks) is being
tried as a prototype on the same engine: see [reworked.md](reworked.md).

## Running Mitra

```sh
mitra init
mitra check [model.mitra evidence.mitra] [--rev <commit>]
mitra eval  [model.mitra evidence.mitra] [--rev <commit>] [--candidates <n>]
            [--threshold <p>] [--all] [-o result.txt]
mitra next  [model.mitra evidence.mitra] [--rev <commit>]
mitra add   '<evidence>' [--source <who>] [--rev <commit>]
mitra ingest <report> [--format libtest|junit] [--rev <commit>]
mitra tie   [--rev <commit>]
```

`mitra` (crate `mitra-cli`) is a thin wrapper over the library
`mitra-core`. When it can't read or check its input it prints `file: line
N: message` and exits with status 2. For a common mistake a `help:` line
follows, with what to write instead: an unquoted `mitra add` line, `_`
used as a wildcard, a claim named where a subject is expected.

- Without the two files, a command reads the store: the nearest `.mitra/`
  at or above the current folder. Naming a store's own `model.mitra` reads
  the same model, with the `mitra:` lines of the Rust sources beside it.
- In a store, every `.mitra` file beside `model.mitra` is model too, read
  after it in name order, except `evidence.mitra`.
- `tie` reads the Rust code of a store whose model is kept aside: see
  [reworked.md](reworked.md#the-tie-to-the-code).
- `eval` writes each claim's verdict, then the errors (see Output), to
  `result.txt`, or to standard output without `-o`. The result file may not
  start with `-` or end in `.mitra`, so a store file is never overwritten.
  It prints `mitra: E errors, W warnings` on standard error, and exits
  with status 1 when there is an error, 0 otherwise. A warning is a claim
  that is `unverified` or `holds assuming`, or evidence that can't be
  used.
- An error lists the pieces that may be wrong, most plausible first: at
  most 5, and none under a plausibility of 0.05. `--candidates <n>` and
  `--threshold <p>` change the two limits, and `--all` removes both.
- `explain <code>` describes a kind of error or warning (`mitra explain
  M004`). The codes are listed under Output.
- `check` parses, checks and evaluates both files, prints
  `ok: F functions, C claims, T tests, E evidence`, then any evidence
  that can't be used.
- `next` prints first the evidence that can't all be true (see Evaluation:
  contradictions, uniformity) and each root cause, then
  each claim that fails with the inputs refuted. Those on a subject that
  uses nothing failing come first, as the likely root causes; the others
  are followed by `look first at …` (see Evaluation, suspects), and so is
  a claim that holds on a suspect subject. A claim that `holds assuming`
  is listed with what it assumes. Then it lists each unverified claim
  with the inputs it misses, the premises of a derived claim that
  fail (`derived: premise fails: p`), and what to add: the evidence lines to write when at most 5 inputs are
  missing on a subject (`evidence f(2) = <output> at 1` for an `=` claim:
  run `f(2)` and fill in its output), else how many inputs are missing
  (counted without listing them), or that the set is infinite and only
  evidence over a set can cover it. Inputs with contradicting evidence
  count as missing too. A missing set longer than 100 characters is summed
  up. When the subject has a `uniform` claim, `next` lists instead the
  classes with no passing input, and says if the `uniform` claim fails or
  is not vouched for. For an unverified `uniform` or `uses` claim it
  prints the evidence line a reviewer adds.
  It prints `nothing to do` when it has nothing to say.
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
  /// mitra: opaque flip(a: Bool) -> Bool
  /// mitra: claim flip_correct : forall a, correct(flip(a))
  pub fn flip(a: bool) -> bool { !a }

  // mitra: test t_true : correct(flip(true))
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
  `test` declaration, the failed ones first and apart: no verdict shows
  them.

## Lexical rules

The syntax is a small, fixed subset inspired by Lean 4
([design](../design/lean-syntax.md)), written in ASCII, with parameters
and calls in parentheses: `f(a: Int, b: Int) -> Int` and `f(1, -2)`. Every command starts
with a keyword (`record`, `enum`, `opaque`, `def`, `claim`, `test`, `evidence`), so a parser knows what it reads from the first token. Layout is
free: line breaks and indentation mean nothing.

- `--` starts a comment that runs to the end of the line.
- Names: a letter or `_`, then letters, digits or `_`; keywords are not
  names.
- Numbers: digits (`nat`), or digits, `.` and digits (`dec`: `0.8`, `1.0`).
  A negative number is `-` then digits (spaces allowed between them).
  `1..5` reads as `1`, `..`, `5`.
- Strings: `"` … `"` on one line. Inside, `\n` is a line break, `\t` a tab,
  `\"` a quote and `\\` a backslash; no other escape.
- Symbols, longest match first: `@[` `..=` `:=` `=>` `->` `==` `!=` `<=` `>=`
  `&&` `||` `..` `.` `(` `)` `:` `;` `,` `=` `{` `}` `[` `]` `|` `&` `!` `<` `>`
  `+` `-` `*` `/` `\`.
- Keywords: `rate precision recall false_positive_rate record enum match opaque def claim test evidence correct at by derive
  if then else let true false forall in not uniform uses only len keys List Set Map
  Bool Int Int8 Int16 Int32 Int64 UInt8 UInt16 UInt32 Float64 String
  Value Real _`. `Real`
  and `_` are reserved.
- Lean's Unicode symbols are accepted as aliases: `∀` for `forall`, `∈` for
  `in`, `¬` for `not`, `∪` for `|`, `∩` for `&`. Mitra prints ASCII.

## Grammar

```ebnf
model      = { command } ;
command    = record | enum | opaque | def | claim | test ;

record     = "record" name [ type-params ] params ;
enum       = "enum" name [ type-params ] "{" variant { "," variant } "}" ;
type-params = "(" name { "," name } ")" ;
variant    = name [ params ] ;
opaque     = "opaque" name [ type-params ] fn-params "->" set ;
def        = "def" name [ type-params ] fn-params "->" set ":=" expr ;
fn-params  = params | "(" ")" ;
params     = "(" param { "," param } ")" ;
param      = name ":" set ;
claim      = "claim" name ":" ( "uniform" name [ "by" name ]
             | [ "only" ] "uses" name "[" name { "," name } "]"
             | { forall }
               ( "correct" "(" name args ")" | expr [ "=" expr ] )
               [ ":=" "by" "derive" "[" name { "," name } "]" ] ) ;
test       = "test" name ":" { forall } judgment ;

evidences  = { evidence } ;
evidence   = [ attributes ] "evidence" { forall } judgment "at" number ;
attributes = "@[" name string { "," name string } "]" ;
judgment   = "correct" "(" name args ")"
           | "not" ( "correct" "(" name args ")" | name args )
           | name args [ "=" value ] ;

forall     = "forall" name [ "in" set ] "," ;
args       = [ "(" [ arg { "," arg } ] ")" ] ;
arg        = name | value ;

set        = inter { "|" inter } ;
inter      = set-atom { ( "&" | "\" ) set-atom } ;
set-atom   = "Bool" | "Int" | "Int8" | "Int16" | "Int32" | "Int64"
           | "UInt8" | "UInt16" | "UInt32" | "Float64" | "String" | "Value"
           | "{" [ value { "," value } ] "}"
           | "(" [ set { "," set } ] ")" [ "->" set-atom ]
           | name [ "(" set { "," set } ")" ]
           | "List" "(" set ")" | "Set" "(" set ")"
           | "Map" "(" set [ "," set ] ")"
           | bound ( "..=" bound | ".." [ bound ] )
           | ( "..=" | ".." ) bound ;
bound      = [ "-" ] nat ;

expr       = "if" expr "then" expr "else" expr
           | "let" name ":=" expr ";" expr
           | or ;
or         = and { "||" and } ;
and        = compare { "&&" compare } ;
compare    = sum { ( "==" | "!=" | "<" | "<=" | ">" | ">=" | "in" ) sum } ;
sum        = product { ( "+" | "-" | "|" | "\" ) product } ;
product    = unary { ( "*" | "/" | "&" ) unary } ;
unary      = ( "!" | "-" ) unary
           | atom { "." ( name | nat ) | "[" expr "]" } ;
atom       = name [ "(" [ expr { "," expr } ] ")" ]
           | "true" | "false" | number | string
           | "(" [ expr { "," expr } ] ")"
           | "[" [ expr { "," expr } ] "]"
           | "len" "(" expr ")" | "keys" "(" expr ")"
           | "{" [ expr { "," expr } ] "}"
           | "{" ( ":" | expr ":" expr { "," expr ":" expr } ) "}"
           | "match" expr "{" arm { "," arm } "}"
           | measure ;
measure    = "rate" "(" name in set "," expr ")"
           | ( "precision" | "recall" | "false_positive_rate" ) "(" name ")" ;
arm        = ( "_" | name [ "(" name { "," name } ")" ] ) "=>" expr ;

value      = "true" | "false" | [ "-" ] number | string | "(" ")"
           | "(" value "," value { "," value } ")"
           | name [ "(" value { "," value } ")" ]
           | "[" [ value { "," value } ] "]"
           | "{" [ value { "," value } ] "}"
           | "{" ( ":" | value ":" value { "," value ":" value } ) "}" ;
number     = nat | dec ;
```

The model file holds `model`; the evidence file holds `evidences`.
A call lists its arguments in parentheses, separated by commas:
`add(a, negate(b))`. Evidence on a whole claim names it without
arguments: `evidence c at 1`.

Beyond the grammar, the parser checks:

- every name in `args` is bound by a `forall` before it, or is a variant
  without fields (`None`); a bound name wins over a variant of the same
  name; every `forall` name is distinct and used exactly once;
- `correct(f(args))`, a claim's `f(args) = e` and an output `f(args) = v`
  write the parentheses, `f()` for a subject without parameters; a name
  alone is a claim;
- a record has at least one field, and a variant without fields is
  written without `()`;
- integer values fit in 64 bits (`-9223372036854775808` to
  `9223372036854775807`); range bounds may be larger, up to 2^126 either
  way, since `Int` is unbounded, but inputs past 64 bits can't be listed
  or computed with;
- `dec` numbers are finite 64-bit floats;
- a range has an upper bound after `..=`, and `..` alone is not a range;
  ranges are over integers only;
- a name used alone as a set is a record or an enum declared above it,
  or a parameter of the enum being declared;
- an enum is declared above its first use and takes one set for each of
  its parameters; inside its own declaration it is written with the same
  parameters (`Tree(T)` in `enum Tree(T)`), and one variant at least does
  not hold it; a record may hold itself inside a list, a set, a map or
  an enum, not as a field alone;
- a value is nested 64 deep at most;
- type parameters come first, in parentheses of their own, and are told
  from fields by having no `:`: `record Pair(A, B)(first: A, second: B)`;
- an arm of a `match` names a variant, with one name for each field;
- `t.0.1` reads `0.1` as a number: write `(t.0).1`.

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
| `(A, B)` | the tuples of one value of A and one of B; two parts or more |
| `Number` | every value of the record `Number` (see `record`) |
| `Number(A, B)` | the records `Number` with a first field in A and a second in B |
| `List(A)` | the lists, of any length, whose elements are all in A |
| `Set(A)` | the set values whose elements are all in A |
| `Map(K, V)` | the map values with every key in K and every value in V |
| `Color`, `Option(A)`, `Result(A, B)` | every value of the enum, with these sets for its parameters (see `enum`) |
| `Some(A)`, `{None}` | the values of one variant: a variant is a record shape |
| `{(1, "a"), Number(1, "m"), [1, 2]}` | exactly these values |
| `{{1, 2}, {}, {"a": 1}, {:}}` | exactly these values: two sets and two maps |
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
prints `{}`. A set of tuples or records prints as products,
`Number(0..=2, {"m"})`, or as a product minus a few values; a set of
lists as `List(A)`, with `\ List(B)` for "at least one element outside
B" and `\ {[]}` for "not the empty list"; sets and maps print the same
way, `Set(A) \ {{1, 2}}`. A set of maps that is not "any key with any
value" prints `Map(P)`, with P its `(key, value)` pairs, and reads back. In reports a record's whole
set prints as its name, and so does an enum as the model writes it
(`Option(Int)`); a part of an enum prints by its variants, such as
`Some(Int \ {4})`.

`Value` holds every tuple, record and list too. A value is a tuple, a
record or a list by its form, whatever was declared: `Number(1, "m")` is
in `Value` and in `Number(Int, String)`, and it is in `Number` when the
fields fit the declaration.

Floats in a set are compared exactly: `NaN` is one value, and `0.0` and
`-0.0` are two. NaN and the infinities have no literal: they can only come
out of float arithmetic in a spec.

## Commands

### record: a group of named fields

```lean
record Number(value: Int, unit: String)
record Sprint(committed: Int, delivered: Int, goal_met: Bool)
```

A record is a type: `Number` is the set of the values `Number(v, u)`
with `v` in `Int` and `u` in `String`. Its name then works wherever a set
does, for a parameter, a result, a field of another record or the
elements of a list. A record is declared above its first use as a set,
and can't hold itself. Two records with the same fields are different
types.

### enum: one of several variants

```lean
enum Color { Red, Green, Blue }
enum Option(T) { None, Some(value: T) }
enum Result(T, E) { Ok(value: T), Err(error: E) }

opaque first(xs: List(Int)) -> Option(Int)
opaque parse(s: String) -> Result(Int, String)
```

An enum is a type made of variants. A variant has named fields, like a
record, or none: `Some(3)`, `None`. The parameters in parentheses are
sets, given at each use: `Option(Int)` is `{None} | Some(Int)`, and
`Option(0..=9)` is a subset of it. An enum without parameters is used by
its name alone, `Color`.

Variant names are shared by the whole model: two enums can't both have a
`None`, and a variant can't be named like a record, a function or a
claim. An enum is declared above its first use.

A record takes type parameters like an enum, and is then used with a set
for each: `Pair(Int, String)`.

```lean
record Pair(A, B)(first: A, second: B)
opaque swap(A, B)(p: Pair(A, B)) -> Pair(B, A)
def first(T)(xs: List(T)) -> Option(T) := if len(xs) == 0 then None else Some(xs[0])
```

A function takes type parameters too, before its own. In a function
each one is any value: Mitra does not check that the `T` going in is the
`T` coming out, and reports print `Pair(Value, Value)`.

A type may contain itself:

```lean
enum Expr { Lit(n: Int), Add(l: Expr, r: Expr), Neg(e: Expr) }
enum Tree(T) { Leaf, Node(left: Tree(T), value: T, right: Tree(T)) }
record Dir(name: String, children: List(Dir))
```

A value is checked against such a type all the way down: `Add(Lit(1), 2)`
is not an `Expr`. A set is not: inside itself the type reads `Value`, so
a set printed piece by piece may show `Add(Value, Value)`. Reports print
`Expr \ {Lit(1)}` where they can. Two types can't contain each other.

Values of an enum work like records in evidence and claims:
`evidence first([]) = None at 1`, `evidence parse("12") = Ok(12) at 1`,
`claim pick_red : pick(None) = Red`.

### opaque: a subject

```lean
opaque flip(a: Bool) -> Bool
opaque minus(a: Int, b: Int) -> Int
```

A function, task or process under review. Each parameter has a type (a
set); its inputs are the tuples of the product of those types. The set
after `->` is its result type.

```lean
opaque open_account() -> Account
opaque log(message: String) -> ()
claim open_correct : correct(open_account())
evidence log("started") = () at 1
```

A subject may have no parameter: it has one input, and is called as
`open_account()`. `()` is the unit type, with one value `()`, for a
function that returns nothing, as in Rust. A `def` takes `()` too.

A function type, `(Int) -> Int` or `(A, B) -> C` or `() -> Int`, holds
the functions **declared above it** that fit: the same number of
parameters, each accepting at least what the type says, and a result
within the type's. The name of a function is then a value, and a name
that holds a function is called like one:

```lean
def double(x: Int) -> Int := x * 2
opaque apply(f: (Int) -> Int, x: Int) -> Int
def each(T, U)(xs: List(T), f: (T) -> U) -> List(U) := map(x in xs, f(x))
claim apply_is : forall f, forall x, apply(f, x) = f(x)
evidence apply(double, 3) = 6 at 1
```

Such a type is a finite set, `{double, negate}`, so a claim over it can
be covered input by input. The result after `->` is one type: write
`(Int) -> (Int | String)` for a union. A subject can be passed as well as
a spec. Inside a function type, a type parameter accepts every function
(`(T) -> U` holds each function of one parameter).

A function that changes a state takes the state and returns it: `fn
withdraw(&mut self, n: u64) -> bool` is `opaque withdraw(a: Account, n:
0..) -> (Account, Bool)`. A test that makes several calls is recorded as
its trace, one line of evidence for each call (golden `state`). A subject is **opaque**: Mitra never
runs it, it only weighs evidence about it.

### def: a spec

```lean
def neg(a: Bool) -> Bool := !a
def step(x: Int) -> Int := if x > 0 then x - 1 else let y := -x; y * 2
def difference(a: Int, b: Int) -> Int := a - b
```

A **definition**: a total, pure function Mitra evaluates. Its body is an
expression: values, the parameters, `let`, `if`, `!`, `-`, `+`, `-`, `*`, `/`,
comparisons (`<` `<=` `>` `>=` on numbers, `==` `!=` on any values), `&&`,
`||`, and calls of specs defined above it. Arithmetic and
comparisons need two integers or two floats; there is no conversion.

A spec may also call itself, on a piece of one of its inputs, so that it
always ends:

```lean
def value(e: Expr) -> Int :=
  match e { Lit(n) => n, Add(l, r) => value(l) + value(r), Neg(x) => 0 - value(x) }
def total(d: Dir) -> 0.. := fold(c in d.children, let sum := d.size; sum + total(c))
```

A piece of an input is a name bound by a `match` on it, one of its
fields (`d.children`), an element (`xs[0]`, or `c` in `fold(c in
d.children, …)`), or a piece of a piece. Every call of the spec to itself
gives a piece of the same parameter, in that parameter's place. `def
down(n: Int) -> Int := down(n - 1)` is refused: `n - 1` is not a piece
of `n`. Two specs can't call each other.
Integer arithmetic is 64-bit and an overflow is an error; `/` on integers
rounds towards zero and dividing by zero is an error. Float arithmetic
and `<` `<=` `>` `>=` follow IEEE 754; `==` and `!=` compare values exactly
(`NaN == NaN` is true, `0.0 == -0.0` is false). A spec is true by
definition: evidence does not apply to it (see Evaluation).

Tuples, records and lists in a body:

```lean
def add_spec(l: Number, r: Number) -> Number := Number(l.value + r.value, l.unit)
def swap(t: (Int, String)) -> (String, Int) := (t.1, t.0)
def size(xs: List(Int)) -> {"empty", "one", "many"} :=
  if len(xs) == 0 then "empty" else if len(xs) == 1 then "one" else "many"
def first(xs: List(Int) \ {[]}) -> Int := xs[0]
```

- `(a, b)` builds a tuple, `Number(a, b)` a record, `[a, b]` a list (`[]`
  is the empty list). Building a record whose fields don't fit its
  declaration is an error.
- `r.field` reads a field of a record, `t.0` a position of a tuple (from
  0), `xs[i]` an element of a list (from 0), and `len(xs)` is its length.
  A missing field, or a position outside the tuple or the list, is an
  error.
- `==` and `!=` compare tuples, records and lists part by part. There is
  no concatenation yet.
- Over each element of a list or a set: `all(x in xs, cond)` and
  `any(x in xs, cond)` give a Bool (`all` is true and `any` false on an
  empty one), `filter(x in xs, cond)` keeps the elements where `cond` is
  true, `map(x in xs, e)` gives `e` for each, and `fold(x in xs, let acc
  := start; step)` starts from `start` and takes `step` as the next
  `acc`, element after element: `fold(x in xs, let sum := 0; sum + x)`.
  A list gives a list, in order; a set gives a set. They are written
  like calls: the five names mean this only with `x in …` as the first
  of two arguments.
- `match e { None => a, Some(v) => b, _ => c }` takes the first arm whose
  variant is the one of `e`, with a name for each field; `_` takes
  anything. No arm for a value is an error: Mitra does not check that the
  arms cover the enum. A variant is built like a record, `Some(x + 1)`,
  and a variant without fields is its name, `None`. A parameter or a
  `let` of the same name wins over the variant.
- `&&` and `||` stop at the first operand that settles them, as in Rust:
  `len(xs) == 0 || xs[0] > 3` is defined on the empty list.

Sets and maps are values too, as a Rust `BTreeSet` or `BTreeMap`: no
order, no repeated element, one value per key.

```lean
def add_spec(stock: Map(String, 0..), item: String, n: 0..) -> Map(String, 0..) :=
  stock | {item: (if item in stock then stock[item] else 0) + n}
def seen(xs: List(String), known: Set(String)) -> Bool := len(xs) == 0 || xs[0] in known
```

- `{a, b}` builds a set and `{k: v}` a map; `{}` is the empty set and
  `{:}` the empty map. A key written twice in a map is an error. Within
  braces in a type, `{{1, 2}}` is the type of the one set `{1, 2}`.
- `x in c` says whether `x` is an element of a set or of a list, or a key
  of a map. `len(c)` counts the elements or the entries.
- `m[k]` is the value at a key; a key that is not in the map is an error,
  so guard it with `k in m`. `keys(m)` is the set of the keys.
- On sets, `a | b`, `a & b` and `a \ b` are union, intersection and
  difference. On maps, `m | n` is `m` with the entries of `n`, which win,
  and `m \ s` is `m` without the keys in the set `s`. `|` and `\` bind
  like `+`, `&` like `*`.
- There is no loop over the elements, and no values-of-a-map yet.

```lean
def or_zero(o: Option(Int)) -> Int := match o { None => 0, Some(v) => v }
def sub_spec(a: UInt8, b: UInt8) -> Result(UInt8, String) :=
  if a < b then Err("underflow") else Ok(a - b)
```

### claim

```lean
claim flip_correct : forall a, correct(flip(a))
claim flip_spec : forall a, flip(a) = neg(a)
claim id_small : forall x in 0..=100, correct(id(x))
claim flip_on_true : correct(flip(true))
claim minus_shape : forall a in 0..=2, forall b in 0..=2, minus(a, b) = add(a, negate(b))
claim flip_not_same : forall a, flip_not(a) = same(a) := by derive [flip_spec, flip_not_shape, neg_twice]
```

The arguments of the call pick the claim's **domain**, one per parameter: a
name bound by `forall x,` is the parameter's whole type, by `forall x in S,` the set
S, and a value is that value alone. The domain is the product of the
arguments, and must lie inside the function's domain. Bound names can be
used in the expression after `=`.

- `correct(f(args))`: the subject is correct on every input of the domain,
  as people judge it. Only for subjects.
- `f(args) = e`: on every input x of the domain, f(x) = e. `e` may call
  specs and subjects. A bare function name `g` as `e` means `g` applied to
  the same inputs.
- any expression with a Bool value: a **property**, true on every input.
  It may call several subjects, nest calls and build values:

  ```lean
  claim sort_len : forall xs, len(sort(xs)) == len(xs)
  claim max_bounds : forall a, forall b, max(a, b) >= a && max(a, b) >= b
  claim add_assoc : forall a, forall b, forall c, add(add(a, b), c) == add(a, add(b, c))
  claim pick_same : forall c in Color, pick(Some(c)) == c
  claim pick_default : pick(None) == Red
  ```

  Its inputs are the names it binds, each used at least once. A name
  passed as it is to a function takes that parameter's type; any other
  needs its set, `forall c in Color,`. `a = b` at the top is `a == b`,
  and `f(x, y) == e` with one name per input is the claim `f(x, y) = e`
  above. Reports name an input by its names, `a = 5, b = 2`, and
  evidence on the claim lists them in order: `evidence max_bounds(5, 2) at 1`.
- a property on a **measure**: how often something holds, for functions
  that don't always answer the same way.

  ```lean
  opaque goal_met(team: {"payments", "search"}, sprint: Int) -> Bool
  claim adapts : rate(s in 11..=20, goal_met("payments", s)) >= 0.6

  opaque is_spam(mail: 1..=1000) -> Bool
  claim precise : precision(is_spam) >= 0.9
  claim few_false_alarms : false_positive_rate(is_spam) <= 0.02
  ```

  `rate(x in S, cond)` is the share of the values of S where `cond` is
  true, from 0 to 1. The condition under which something is measured is
  the set S. `precision(f)`, `recall(f)` and `false_positive_rate(f)`
  are for a subject `f` that answers true or false: the truth on an
  input is the answer when it was judged `correct`, and the other one
  when judged `not correct`. A measure is an expression: it can be
  compared, added, and used under a `forall`.

  A measure has a value in two ways (see Evaluation): a measurement
  recorded as evidence, or the observed outputs when they cover every
  value of S. Mitra does no statistics: how good a sample is goes into
  the trust given to the measurement.
- `:= by derive [c1, c2, …]`: a **derivation**. Premises must be claims
  defined above. Mitra checks the implication "premises ⇒ claim" when it
  can (see Evaluation), and the claim follows on its whole domain when no
  premise fails and the implication is not refuted.
- `uniform f by g`: a **uniformity claim**, the assumption a developer
  makes when a few tests are taken to cover a function. `g` is a spec with
  the same parameters as the subject `f`; its values are the **classes**
  of inputs, and the claim says `f` behaves the same on every input of a
  class: right on all of them or wrong on all of them. So one judged input
  stands for its class (see Evaluation).

  ```lean
  opaque div(a: Int, b: Int) -> Int
  def div_class(a: Int, b: Int) -> {"zero", "rest"} := if b == 0 then "zero" else "rest"
  claim div_uniform : uniform div by div_class
  ```

  `g`'s result type has at most 64 values, and a subject has at most one
  uniformity claim. Mitra can't check it: evidence vouches for it,
  `evidence div_uniform at 1`, for example from a review that the classes
  follow the branches of the code. Without evidence it is `unverified`,
  and what rests on it `holds assuming` it.

  `uniform f`, without `by`, is the same claim with a single class of
  every input: one judged input stands for all of them.

  `f` may also be a **property** claim, named by its claim name: the class
  spec then takes the property's bound names, and sorts the inputs of the
  property itself instead of one class per call. Use it when the classes
  of the subjects called can't all meet.

  ```lean
  claim same_sign : forall x, sign(x) * half(x) >= 0
  claim same_sign_uniform : uniform same_sign by sign_class
  ```

  ```lean
  opaque add(left: Int, right: Int) -> Int
  claim add_uniform : uniform add
  ```
- `uses f [g, h]`: the subject `f` calls the subjects `g` and `h`.
  `only uses f [g, h]` adds that it calls no other subject. Neither
  changes another claim's verdict; they tell Mitra where to look when `f`
  fails (see Evaluation, suspects and errors). Like uniformity, Mitra
  can't check them: evidence vouches for them (`evidence c at 1`) or
  denies them (`not c`). Without evidence, a `uses` or `only uses` claim
  still gives suspects, unless denied, but only a vouched `only uses`
  gives a root cause.

  ```lean
  claim dempster_uses : only uses dempster [is_valid, conflict]
  ```

A `uniform` or `uses` claim can't be a premise of `by derive`.

### evidence

```lean
evidence flip(true) = false at 1
evidence not correct(id(7)) at 1
evidence forall x in 0..=100, correct(id(x)) at 1
evidence flip_not_shape at 1
```

- `f(args) = v`: the subject was run on one input and returned `v`, which
  must lie in its result type. Used by `=` claims. Inputs and outputs can
  be tuples, records and lists:
  `evidence add(Number(1, "m"), Number(2, "m")) = Number(3, "m") at 1`,
  `evidence sort([3, 1, 2]) = [1, 2, 3] at 1`. An argument is a whole
  value or a bound name: `Number(x, "m")` with `x` bound is not accepted.
- `rate(s in 11..=20, goal_met("payments", s)) = 0.4`, `precision(is_spam)
  = 0.93`: a **measurement**, the value of a measure as someone measured
  it. It names every input (no `forall`), and it is matched with the
  measure of a claim as written, once the claim's names have their
  values.
- `correct(f(args))` / `not correct(f(args))`: people judged the subject's
  results on these inputs correct (or wrong). A single input is a test; a
  set is evidence for every input in it, such as a proof. Used by `correct`
  claims.
- `c` / `not c`, with `c` a claim: the claim holds (or fails) on its whole
  domain, for example from a review or a proof. `c(args)` restricts it to
  those inputs.

The number after `at` is the **bias**: how much its author trusts this
piece of evidence, from `0` to `1`. It never changes a verdict: every
piece of evidence counts, at any bias, even 0. It is used only when an
error is raised, to say which piece is most plausibly wrong (see
Evaluation, errors).

There is one kind of evidence. A test result, a review, a proof and a bug
report differ only by their `source`, and no rule depends on it.

Attributes record where evidence comes from, and do not change verdicts:

```lean
@[source "ci", rev "3f2a1c"] evidence add(1, 2) = 3 at 1
```

- `source`: who or what observed it (a CI job, a reviewer, a test).
- `rev`: the revision of the code it was observed on.

Each may appear once; other names are rejected.

### test

```lean
test t_true : correct(flip(true))
test t_small : forall x in 0..=10, correct(add(x, 1))
test t_one : add(1, 2) = 3
```

A test of the code and the evidence its result stands for; the judgment is
written as in `evidence`, without a bias. `mitra ingest` turns a
result into evidence at 1:

- a pass gives the judgment;
- a test `f(args) = v` gives the output its author **expects**: when a
  `correct` claim on `f` covers the input, a pass also gives
  `correct(f(args))` and a failure gives `not correct(f(args))`;
- a failure gives `not correct(…)` for a `correct(…)` test, and `not c …`
  for a `c …` test, when the test is about a single input. Otherwise it
  gives nothing: a failed test over a set says the code is wrong somewhere
  in it, which Mitra cannot express yet; a failed `f(args) = v` test says
  the output was something else, but not what. If the subject's result
  type is `Bool`, it gives `f(args) =` the other value. Otherwise it gives
  `not c(args)` for each `=` claim `c` on `f` whose other side is `v` on
  that input (the claim is wrong there), and nothing more if there is none. A failed negative test
  (`not correct …`, `not c …`) proves nothing, since it may have failed for
  another reason.

Test names share the namespace of functions and claims.

## Evaluation

Every piece of evidence counts, whatever its bias. Evidence on a **spec**
is not used and gives warning `M104`: a spec is a definition, so
evidence cannot disprove it.

For each claim, in model order, with D its domain, Mitra gathers the inputs
where the claim is right and where it is wrong:

1. Evidence on the claim itself (`c` / `not c`), within D.
2. For a `correct` claim: `correct` / `not correct` evidence on its subject.
3. For an `=` claim, on each input x (every input of D if D is finite,
   otherwise every observed input in D): evaluate both sides, a subject
   call giving the output observed for that input. Equal values make x
   right, different values make x wrong. With no observation, x is not
   judged. A claim on specs alone over a finite domain is thus decided by
   computation.
4. For a property, on each input x (every input of D if D is finite,
   otherwise the inputs the observations speak of: every way to give its
   names the observed inputs of the calls whose arguments are names or
   values): compute the expression, a subject call giving its observed
   output. True makes x right, false makes x wrong. `a && b` and `a || b`
   are decided by one side when it settles them, even if the other has no
   observation. When x is wrong, the error lists every output that was
   read, and the code of every subject called: they can't all be right.

A measure in a property takes the value of its measurement when there is
one. Without one it is counted from the observed outputs, when they cover
every value of the set; otherwise it has no value, the claim is not
judged there, and a note says what was counted so far. Two measurements
of one measure with different values, or a measurement outside what the
observed outputs leave possible (3 of 4 values seen true: at least
0.75), are an error (`M005`); the measure then has no value until that is
settled.

Evidence on a claim (`c` or `not c`) speaks of the claim's domain only,
whatever inputs it names.

An input both right and wrong is a **contradiction**: the pass and the
failure can't both be true. So are two different outputs for one input,
and an input judged `correct` and `not correct`, on any subject, with or
without a claim. A contradicted input counts on neither side, for every
claim saying the same of the subject, and each pair of pieces of evidence
that contradict is an error (see Errors). Then, for each claim:

- *refuted* = wrong \ right.
- *tested* = right \ wrong: what evidence covers directly.
- *assumed*: what follows from other claims. With `by derive`, when no
  premise fails and the implication is not refuted, every input of D
  that is neither tested nor wrong. With a uniformity claim on the
  subject, the inputs of the classes that passed (see Uniformity). A
  direct counterexample still refutes the claim.
- *missed* = D minus the three sets above.

The **verdict** is:

- `fails` if *refuted* is not empty;
- `unverified` if *missed* is not empty;
- `holds` if nothing is missed and every claim the *assumed* inputs rest
  on holds;
- `holds assuming [a, b]` if nothing is missed and the claims `a`, `b`
  it rests on are not verified. These are: an unverified premise; the
  claims a premise itself assumes; a uniformity claim nobody vouched for.

There is no number on a claim.

**Uniformity.** Let `u : uniform f by g` be a claim. Mitra looks at the
single inputs judged for `f`, one **statement** at a time. `correct` is
one statement. Each `= e` is another: claims on `f` with the same `e`
share their inputs, whatever their domains. The inputs of a statement are
those of the evidence in rules 1 to 3 above, also outside a claim's
domain (an observed output is compared with `e` wherever the claim's
arguments allow). Evidence over a set counts input by input when the set
has at most 10 000 inputs. A contradicted input is neither a pass nor a
failure, and nothing is assumed for its class. The class of
an input x is `g(x)`.

- **Conflict.** If, for one statement, a class holds an input that passed
  and one that failed, `u`, the pass and the failure can't all be true.
  `u` **fails**, whatever vouches for it; its *refuted* shows the two
  inputs, and an error lists the three pieces (see Errors).
- Otherwise `u` holds if evidence on `u` covers its whole domain, like any
  claim, and is unverified if not. It also fails if evidence denies it.
- While `u` **does not fail** and no evidence denies it on any input,
  for every other claim c on `f`: an input of D in a class with a passing
  input is **assumed**. If `u` is unverified, c can then only `hold
  assuming [u]`.
- While `u` **holds**, an input in a class with a failing input is
  refuted.
- On a finite D (up to 10 000 inputs) this is done input by input. On a
  larger D the classes Mitra can't place: the inputs are assumed once
  every value of `g`'s result type has a passing input, and a failing
  class refutes only its own input.
- Nothing is assumed for a claim whose statement has a failure over a set
  too large to read input by input (its class is unknown).
- **A claim that calls subjects**, as `add(l, r) = add(r, l)` or
  `f(x) = g(x)` with `g` a subject, is sorted by every call: its own,
  `f(x)`, and each call of a subject on its other side. The class of an
  input is then one class per call, each from the `uniform` claim of the
  subject called, on the arguments of that call: for `abs(x) = abs(-x)`
  with classes by sign, the input 2 is in `("pos", "neg")`. The rules
  above apply to these classes. All the `uniform` claims involved must
  not fail; the claim `holds assuming` those not vouched for; a failure
  stands for its class only when all are vouched for. A conflict is an
  error on the `uniform` claim of the claim's own subject, and the
  evidence vouching for the others is among its pieces.
  Nothing is assumed, and a `note:` says why, when a subject called has
  no `uniform` claim, or when an argument of a call is not computed from
  the claim's inputs alone (it calls a subject, or sits under `if` or
  `let`). On a domain too large to list, every combination of classes
  needs a passing input, also those no input can give.
- Mitra trusts `g`'s result type on inputs it never computes: a value no
  input gives is a class no test can fill.

**Errors.** An error is something that can't all be true. Each lists the
evidence involved, and the **pieces** of which at least one is wrong: a
piece of evidence, the code of a subject, or a claim.

| Error | Pieces, of which one is wrong |
|---|---|
| A `correct` or `=` claim on `f` fails | each piece of evidence refuting it, with the evidence vouching for the uniformity claim when it refutes only through its class; the code of `f`, of what `f` uses (see Suspects) and of the subjects its `=` side calls. For a claim on specs alone, the claim |
| A `uniform` or `uses` claim is denied by evidence | the denial; the claim |
| A contradiction on an input | the two pieces of evidence |
| A conflict in a class of `u` | every pass and every failure of the class, and each piece of evidence vouching for `u` on both inputs, or `u` itself when none does |

Claims that fail on the same evidence share one error. The same error
found twice, through an observation recorded twice, prints once.

**Most plausibly wrong.** For each error Mitra computes, for every piece,
the belief and the plausibility that it is wrong, with Dempster-Shafer
(`mitra_ds::culprits`):

- A piece of evidence at bias b puts mass b on "this piece is right" and
  1 − b on "unknown". The code of a subject and a claim have no bias:
  they say nothing.
- Each error is a constraint: its pieces are not all right. One error can
  give several constraints (one per refuting piece, per vouching piece,
  per pair of a pass and a failure in a class).
- The masses are combined by Dempster's rule under the constraints. The
  mass that would make every piece of some constraint right is the
  **conflict**: how much the biases say that nothing is wrong.
- *Plausibility* that a piece is wrong: the share of the remaining mass
  where it is not taken as right. *Belief*: the share where it must be
  wrong, because every other piece of one of its constraints is right.

Several pieces can be wrong at once. Errors that share a piece are
computed together. The same observation recorded twice (same judgment,
inputs, `source` and `rev`) is one piece, at the highest bias given.
Pieces are listed by plausibility, then belief, highest first.

**When the pieces trusted at 1 can't all be right**, Dempster's rule has
no answer: every way out contradicts a trust of 1. Mitra then takes the
limit of the rule as each trust of 1 goes down to 1 − e, e going to 0.
Only the ways with the fewest trusted pieces wrong count, so trusted
pieces that play the same part share the doubt equally: three pieces at
1 that can't all be right are each wrong at 1/3 (golden `uses/08`,
`bias/06`). A piece trusted below 1 that can take the blame still takes
it first. The error says that the doubt is shared.

If more than 20 pieces have to be weighed (those with a bias strictly
between 0 and 1, and those at 1 when they can't all be right), the
error says there are too many to rank.

Examples: a failure at bias 0.8 against a claim gives the code belief 0.8
and plausibility 1 of being wrong, and the failure 0 and 0.2. Evidence at
1 against a claim gives the code 1 and 1. A piece at bias 0 always has
plausibility 1.

**Suspects.** For each subject `f`, Mitra follows the `uses` claims that
do not fail, from `f` to what it uses, and on from there. Each subject
reached that has a failing claim (other than a `uses` claim) is a
**suspect** for `f`, printed after the errors. Verdicts do not change.
A `uses` claim needs no evidence for this: unverified, it still gives
suspects and pieces.

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
premise `g(args) = e` as a definition: a call `g(y)` with y in the
premise's domain is replaced by `e`. Specs are evaluated as usual. The
implication is:

- `checked` if both sides have a value and agree on every input;
- `refuted` if they differ on some input: the premises imply something else,
  so the derivation is not used;
- `unchecked` otherwise (a subject call no premise defines, an infinite
  domain, or a `correct` claim). An unchecked implication is taken as
  stated by the author, and the output says so.

The check depends only on the model, not on evidence.

## Output

`eval` prints each claim's verdict, then the errors, the warnings and the
notes, written like a compiler's diagnostics.

**Verdicts.** Each claim, in model order: its name and verdict, then
indented lines, each printed only when it says something:

```text
div_correct: unverified
  passes on       `div(6, 3)`
  assumed on      class "rest" of `div_class`, by `div_uniform`
  no evidence on  class "zero" of `div_class`
div_small: holds
  assumed on      class "rest" of `div_class`, by `div_uniform`
div_uniform: holds
```

- `fails on`: the inputs evidence refutes.
- `passes on`: the inputs evidence supports, unless it is the whole
  domain.
- `assumed on … : its premises imply it`: the inputs that follow from the
  premises of `by derive`; `from its premises, the implication not
  checked` when Mitra could not check it. When the derivation is not
  used: `not derived: a premise fails`, or `… its premises do not imply
  it`.
- `assumed on class … by u`: the classes with a passing input (on a
  finite domain, only those the claim meets), unless the claim fails or
  `u` can't be relied on. `which is not vouched for` when no evidence
  vouches for `u`. For `uniform f` without a spec: `assumed on every
  input, by u`. For a claim that calls subjects: ``assumed on classes
  ("pos", "neg") of `abs(x)`, `abs(-x)`, by u``, with every `uniform`
  claim relied on, and `not vouched for:` those that are not.
- `no evidence on`: the inputs missed. With a uniformity claim and a set
  too large to list, the classes without a passing input instead.

A `uniform` or `uses` claim has no such lines. Up to 3 inputs print as
calls (`` `div(4, 2)` ``); more print as a set, named after the parameters:
`(a, b) in (0..=2, 0..=2)`. A set of tuples prints as the domain's own
form when equal to it, as a list of tuples when finite (`{(1, -1)}`), or
as a union of products, each a tuple of sets:
`(0..=2, {0}) | (3..=4, Int)`. A set longer than 160 characters prints as
its size (`9 inputs`).

**Diagnostics.** After a blank line, each error, then each warning. A
diagnostic has a severity and a code, a headline, `-->` and the
`file:line` of what it is about with the claim as written, a description,
and `help:` with the next thing to do:

```text
error[M001]: evidence refutes a claim
  --> model.mitra:5  claim div_correct : forall a, forall b, correct(div(a, b))

  `div_correct` fails on `div(4, 2)`.
  The evidence that says so:

    fails     evidence.mitra:11  @[source "bug:42"] evidence not correct(div(4, 2)) at 0.8

  The code, the claim or the evidence is wrong. Below, the trust given to each
  piece, and what is left of it once they are weighed against each other:

                                                trust  left
    possible  the code of `div`, model.mitra:2  none   0.00 to 0.70
    likely    evidence.mitra:11 (bug:42)        0.8    0.30 to 0.38

  The lower what is left, the more plausibly the piece is the wrong one.
  A range means the evidence does not settle it.

  help: if the evidence is right, fix `div`; if not, correct the evidence

error[M004]: an assumption is contradicted by the evidence
  --> model.mitra:6  claim div_uniform : uniform div by div_class

  `div_uniform` says `div` behaves the same for every input of class "rest"
  of `div_class`. But in that class some inputs pass and others fail:

    vouches   evidence.mitra:9  @[source "review:alice"] evidence div_uniform at 0.9
    passes    evidence.mitra:10  @[source "test:t_six"] evidence correct(div(6, 3)) at 0.99
    fails     evidence.mitra:11  @[source "bug:42"] evidence not correct(div(4, 2)) at 0.8

  These can't all be right. Below, the trust given to each
  piece, and what is left of it once they are weighed against each other:

                                               trust  left
    likely    evidence.mitra:11 (bug:42)       0.8    0.30 to 0.38
    unlikely  evidence.mitra:9 (review:alice)  0.9    0.65 to 0.72
    1 less plausible not shown (`--all` shows them)

  The lower what is left, the more plausibly the piece is the wrong one.
  A range means the evidence does not settle it.

  note: the trust given says these are probably all right, which can't
  be: at least one is trusted more than it deserves

  help: check that `div(4, 2)` really fails; if it does, class "rest" is
  too wide: split it in `div_class`
```

| Code | Severity | Headline |
|---|---|---|
| `M001` | error | evidence refutes a claim |
| `M002` | error | evidence denies an assumption (a `uniform` or `uses` claim) |
| `M003` | error | two pieces of evidence contradict each other |
| `M004` | error | an assumption is contradicted by the evidence: a pass and a failure in one class |
| `M005` | error | a measurement does not fit another measurement of the same measure, or the observed outputs |
| `M006` | error | evidence does not fit the model: inputs or an output outside the signature of its function |
| `M101` | warning | a claim is not verified: some inputs have no evidence |
| `M102` | warning | a claim holds only under assumptions |
| `M103` | warning | an assumption is not vouched for |
| `M104` | warning | evidence that can't be used: it is about a spec |

**Every piece can be the wrong one.** The candidates of an error are the
evidence, the code of the subjects, and what the model states: the claim
itself (`the claim `c` itself`) and, for `M006`, the signature of the
function. What the model states is trusted at 1, like evidence without a
number, so it is doubted only when pieces trusted at 1 can't all be
right, or when its trust is lowered (in the
[reworked syntax](reworked.md): `claim c(…) trust 0.8 { … }`).

**Evidence outside a signature is kept.** Evidence on a function with
inputs outside its parameter types, or an output outside its result
type, is not refused: it is set aside, judges nothing, and is reported as
`M006` with the evidence and the signature as candidates. So a run of
the code can show that the model is wrong. Evidence with the wrong number
of inputs, or on an unknown name, is still refused.

`mitra explain <code>` prints what a code means and how to fix it. The
codes are stable: a code keeps its meaning.

- **Locations.** Lines of the model and of the evidence print as
  `file:line`, relative to the current directory when under it. For a
  model line in a Rust comment, it is the line of the `.rs` file. The
  library names the files `model.mitra` and `evidence.mitra` unless given
  a `Places` (`Report::to_text_at`).
- **Evidence.** Each piece an error rests on: what it says (`passes`,
  `fails`, `output`, `vouches`, `denies`), where it is, and its text. At
  most 5 are quoted, then `and N more`.
- **Which one is wrong.** The pieces of the error, most plausible first.
  Each line has a word, the piece, the `trust` given to it (`none` for
  code and claims) and what is `left` of that trust once the pieces are
  weighed against each other: 1 − plausibility of being wrong, to
  1 − belief of being wrong, as one number when the two are equal. The
  lower what is left, the more plausibly the piece is the wrong one. The
  word is `certain` (belief of being wrong at least 0.995), `likely` (at
  least 0.5), `possible` (plausibility at least 0.5), else `unlikely`.
  At most 5 pieces are listed, and none under plausibility 0.05 (see
  Running Mitra); `N less plausible not shown` counts the others whose
  plausibility is not 0.
- `note: the pieces trusted at 1 can't all be right`: the doubt is shared
  equally between them (see Evaluation).
- When more than 20 pieces would have to be weighed, the error says so
  and ranks nothing.
- `note: the trust given says these are probably all right`: on an error
  other than `M001`, when the pieces would all be right with a mass of at
  least 0.5 by their biases alone (the conflict). At least one is
  trusted more than it deserves.
- On `M001`: `help: look first at g` when the subject uses a subject that
  fails, and `note:` with the root cause when there is one (see
  Evaluation). A suspect that has no error of its own is a `note:` after
  the warnings.

The last line names `mitra explain`. `eval` prints the counts on standard
error: `mitra: E errors, W warnings`.

A spec referring to a name it does not bind, a type error, an overflow or a
value outside a function's domain or result type stops evaluation with an
error.

## Golden set

`tests/golden/<set>/<case>/` holds `model.mitra`, `evidence.mitra` and
`expected.txt`, the exact output. `cargo test` checks every case.
Evidence is `at 1` unless marked. Each failing claim below also gives an
error, with the code of its subject as the piece most plausibly wrong.

### `flip`

```lean
opaque flip(a: Bool) -> Bool
claim flip_correct : forall a, correct(flip(a))
claim flip_on_true : correct(flip(true))
```

| Case | Evidence | flip_correct | flip_on_true |
|---|---|---|---|
| 01 | none | unverified, missed Bool | unverified |
| 02 | true correct, false correct | holds | holds |
| 03 | true correct | unverified, missed {false} | holds |
| 04 | false wrong | fails, refuted {false} | unverified |
| 05 | true correct, false wrong | fails, refuted {false} | holds |
| 06 | true correct, false correct and wrong | unverified, missed {false}; error on false | holds |
| 07 | true correct and wrong, false correct | unverified, missed {true}; error on true | unverified |
| 08 | true wrong, false correct and wrong | fails, refuted {true}; error on false | fails |
| 09 | true correct `at 0`, false wrong `at 0` | fails: bias 0 still counts | holds |
| 10 | case 02, plus false wrong `at 0` | unverified; error on false, the piece at 0 is the one wrong | holds |

### `id`

```lean
opaque id(x: Value) -> Value
claim id_correct : forall x, correct(id(x))
claim id_small : forall x in 0..=100, correct(id(x))
```

| Case | Evidence | id_correct | id_small |
|---|---|---|---|
| 01 | none | unverified | unverified |
| 02 | `id(1)`, `id(42)` correct | unverified, tested {1, 42} | unverified |
| 03 | correct on `0..=100` | unverified, tested 0..=100 | holds |
| 04 | correct on `0..=100` and on `String` | unverified | holds |
| 05 | `id(1)` correct, `id(7)` wrong | fails, refuted {7} | fails, refuted {7}; one error for both |
| 06 | correct on `forall x` | holds | holds |
| 07 | correct on `0..=100`, `id(5)` wrong | unverified; error on 5 | unverified, missed {5} |
| 08 | `id("a")` correct | unverified, tested {"a"} | unverified, tested nothing |
| 09 | correct on `forall x` `at 0`, `id(7)` wrong `at 0` | unverified, missed {7}; error on 7 | unverified, missed {7} |

On `Value`, tests never make `id_correct` hold: only evidence on the whole
domain does (case 06).

### `flip_matches`: a subject against a spec

```lean
def neg(a: Bool) -> Bool := !a
opaque flip(a: Bool) -> Bool
claim flip_spec : forall a, flip(a) = neg(a)
claim flip_on_true : flip(true) = neg(true)
```

| Case | Evidence | flip_spec | flip_on_true |
|---|---|---|---|
| 01 | none | unverified | unverified |
| 02 | `flip(true) = false`, `flip(false) = true` | holds | holds |
| 03 | `flip(true) = false` | unverified, tested {true} | holds |
| 04 | `flip(false) = false` | fails, refuted {false} | unverified |
| 05 | `flip(true) = false`, `flip(false) = true` and `= false` | unverified, tested {true}; error on false | holds |
| 06 | `forall a, correct(flip(a))` (a judgment, not an output) | unverified | unverified |
| 07 | case 02, plus `neg(true) = true` | holds | holds, and the evidence on `neg` can't be used (`M104`) |
| 08 | `flip(false) = false` `at 0`, `neg(true) = true` `at 0` | fails | unverified |

### `compose`: a derivation mixing specs and subjects

```lean
def neg(a: Bool) -> Bool := !a
def same(a: Bool) -> Bool := a
opaque flip(a: Bool) -> Bool
opaque flip_not(a: Bool) -> Bool
claim neg_twice : forall a, same(a) = neg(neg(a))
claim flip_spec : forall a, flip(a) = neg(a)
claim flip_not_shape : forall a, flip_not(a) = flip(flip(a))
claim flip_not_same : forall a, flip_not(a) = same(a) := by derive [flip_spec, flip_not_shape, neg_twice]
```

`neg_twice` is about specs only: it holds by computation in every case.
The implication of `flip_not_same` is `checked` in every case.
"Review" is `evidence flip_not_shape at 1`.

| Case | Evidence | flip_spec | flip_not_shape | flip_not_same |
|---|---|---|---|---|
| 01 | none | unverified | unverified | holds assuming [flip_spec, flip_not_shape] |
| 02 | `flip` on both inputs, review | holds | holds | holds (derived, no evidence on `flip_not`) |
| 03 | `flip(true) = false`, review | unverified, tested {true} | holds | holds assuming [flip_spec] |
| 04 | `flip(true) = true`, `flip(false) = true`, review | fails, refuted {true} | holds | unverified: a premise fails |
| 05 | case 02, plus `flip_not(true) = false` | holds | unverified, tested {false} | fails, refuted {true} |
| 06 | outputs of `flip` and `flip_not` on both inputs | holds | holds | holds |
| 07 | `flip(true) = false` `at 0`, review `at 0` | unverified | holds | holds assuming [flip_spec] |

In case 05 the counterexample on `flip_not` also contradicts the review of
`flip_not_shape` on `true`: a second error.

### `minus`: tuples and a checked derivation

```lean
def plus(a: Int, b: Int) -> Int := a + b
def opposite(b: Int) -> Int := -b
def difference(a: Int, b: Int) -> Int := a - b
opaque add(a: Int, b: Int) -> Int
opaque negate(b: Int) -> Int
opaque minus(a: Int, b: Int) -> Int
claim add_spec : forall a in 0..=2, forall b in -2..=0, add(a, b) = plus(a, b)
claim negate_spec : forall b in 0..=2, negate(b) = opposite(b)
claim minus_shape : forall a in 0..=2, forall b in 0..=2, minus(a, b) = add(a, negate(b))
claim minus_spec : forall a in 0..=2, forall b in 0..=2, minus(a, b) = difference(a, b) := by derive [add_spec, negate_spec, minus_shape]
```

"Leaves" are the outputs of `add` on its 9 inputs and of `negate` on its 3,
plus a review, `evidence minus_shape at 1`. There is never evidence
on `minus` itself.

| Case | Evidence | add_spec | minus_shape | minus_spec | implication |
|---|---|---|---|---|---|
| 01 | none | unverified | unverified | holds assuming all three premises | checked |
| 02 | leaves | holds | holds | holds | checked |
| 03 | leaves without `add(2, -2)` | unverified, missed {(2, -2)} | holds | holds assuming [add_spec] | checked |
| 04 | leaves with `add(1, -1) = 1` | fails, refuted {(1, -1)} | holds | unverified: a premise fails | checked |
| 05 | leaves; the shape is `add b (negate a)` | holds | holds | unverified: derivation not used | refuted |
| 06 | leaves without the review; `by derive [add_spec, negate_spec]` only | holds | unverified | holds | unchecked |
| 07 | `add(0, 0) = 0` `at 0`, review `at 0` | unverified | holds | holds assuming [add_spec, negate_spec] | checked |

`negate_spec` holds whenever the leaves are there. Case 06 shows the risk
of an unchecked implication: `minus_spec` holds only because the author
said so.

### `div`: a uniformity claim

```lean
opaque div(a: Int, b: Int) -> Int
def div_class(a: Int, b: Int) -> {"zero", "rest"} := if b == 0 then "zero" else "rest"
claim div_correct : forall a, forall b, correct(div(a, b))
claim div_small : forall a in 0..=1, forall b in 1..=2, correct(div(a, b))
claim div_uniform : uniform div by div_class
```

| Case | Evidence | div_correct | div_small | div_uniform |
|---|---|---|---|---|
| 01 | `div(6, 3)` and `div(1, 0)` pass; nothing vouches | holds assuming [div_uniform] | holds assuming [div_uniform] | unverified |
| 02 | vouched; `div(6, 3)` passes | unverified, missed class "zero" | holds, assumed "rest" | holds |
| 03 | vouched; `div(6, 3)` and `div(1, 0)` pass | holds, assumed "rest", "zero" | holds | holds |
| 04 | vouched; `div(6, 3)` passes, `div(4, 2)` fails | fails, refuted {(4, 2)} | unverified | fails; error in class "rest" |
| 05 | vouched; `div(4, 2)` fails | fails, refuted {(4, 2)} | fails, refuted its whole domain; its error also lists the review | holds |
| 06 | vouched; `div(6, 3)` passes; `div` fails somewhere on `(10.., 10..)` | fails | unverified: nothing assumed | holds |
| 07 | vouched; `div(6, 3)` passes; `not div_correct(4, 2)` | fails, refuted {(4, 2)} | unverified | fails; error in class "rest" |
| 08 | vouched; `div(6, 3)` passes and fails; `div(4, 2)` passes | unverified, tested {(4, 2)}; error on (6, 3) | unverified: class "rest" not relied on | holds |
| 09 | vouched; `div_small` reviewed | unverified, assumed "rest" | holds | holds |
| 10 | vouched; `div(6, 3)` fails; `div_correct(6, 3)` reviewed | unverified; error on (6, 3) | unverified: the failure stands for nothing | holds |

### `calls`: uniformity for claims that call subjects

Each case has its own model. 01 to 04:
`claim add_commutative : forall left, forall right, add(left, right) = add(right, left)`
with `claim add_uniform : uniform add`. 05 to 07:
`claim abs_even : forall x in -3..=3, abs(x) = abs(-x)` with
`uniform abs by sign` (classes "neg", "zero", "pos"). 08 and 09:
`claim f_g : forall x, f(x) = g(x)`, both subjects, with `uniform f`.

| Case | Evidence | The claim | The `uniform` claim |
|---|---|---|---|
| 01 | `add_commutative(1, 2)`; nothing vouches | holds assuming [add_uniform] | unverified |
| 02 | vouched; `add_commutative(1, 2)` | holds, assumed on every input | holds |
| 03 | vouched; outputs `add(1, 2) = 3`, `add(2, 1) = 3` | holds | holds |
| 04 | vouched; `add_commutative(1, 2)`; `not add_commutative(3, 4)` at 0.8 | fails | fails; error `M004` |
| 05 | vouched; `abs_even(2)` | unverified: assumed on ("pos", "neg"), nothing on -3..=0 | holds |
| 06 | vouched; `abs_even` on 2, -1 and 0 | holds | holds |
| 07 | vouched; `abs_even(2)`; `not abs_even(3)` at 0.8 | fails | fails; error `M004` |
| 08 | `f_uniform` vouched; `f_g(1)`; `g` has no `uniform` claim | unverified, with a note | holds |
| 09 | as 08, with `claim g_uniform : uniform g` not vouched | holds assuming [g_uniform] | f: holds, g: unverified |

### `same`: uniformity without classes

```lean
opaque add(left: Int, right: Int) -> Int
claim add_ok : forall left, forall right, correct(add(left, right))
claim add_uniform : uniform add
```

| Case | Evidence | add_ok | add_uniform |
|---|---|---|---|
| 01 | `add(1, 2)` passes; nothing vouches | holds assuming [add_uniform] | unverified |
| 02 | vouched; `add(1, 2)` passes | holds, assumed on every input | holds |
| 03 | vouched; no test | unverified | holds |
| 04 | vouched; `add(1, 2)` passes, `add(2, 2)` fails at 0.8 | fails on `add(2, 2)` | fails; error `M004` |

### `bias`: which piece is most plausibly wrong

```lean
opaque div(a: Int, b: Int) -> Int
def div_class(a: Int, b: Int) -> {"zero", "rest"} := if b == 0 then "zero" else "rest"
claim div_correct : forall a, forall b, correct(div(a, b))
claim div_uniform : uniform div by div_class
```

The numbers are computed by hand in each case's `evidence.mitra`.

| Case | Evidence | Most plausibly wrong `[belief, plausibility]` |
|---|---|---|
| 01 | `div(4, 2)` fails, at 0.8 | the code of `div` [0.80, 1.00]; the failure [0.00, 0.20] |
| 02 | review of `div_uniform` at 0.9; `div(6, 3)` passes at 0.99; `div(4, 2)` fails at 0.8 | for the class: the failure [0.62, 0.70], the review [0.28, 0.35], the pass hidden (0.03); conflict 0.71. For `div_correct`: the code [0.30, 1.00] |
| 03 | the same failure at 0.8 recorded twice, and another report at 0.5 | the code [0.90, 1.00]: the twin counts once |
| 04 | two sources report the same wrong output, at 0.9 and 0.1 | the code [0.91, 1.00]; both reports listed, whatever their order |
| 05 | review of `div_uniform` at 0; `div(4, 2)` fails, outside `div_small` | `div_small` fails; the code and the review both [0.00, 1.00] |
| 06 | vouched; `div(6, 3)` passes at 0.1, `div(8, 4)` passes, `div(4, 2)` fails | the review, the trusted pass and the failure share the doubt: [0.34, 0.34], [0.31, 0.31], [0.34, 0.34]; the distrusted pass [0.31, 0.93] |

### `uses`: suspects

```lean
opaque half(x: Int) -> Int
opaque quarter(x: Int) -> Int
opaque eighth(x: Int) -> Int
claim half_correct : forall x in 0..=1, correct(half(x))
claim quarter_correct : forall x in 0..=1, correct(quarter(x))
claim eighth_correct : forall x in 0..=1, correct(eighth(x))
claim quarter_uses : uses quarter [half]
claim eighth_uses : uses eighth [quarter]
```

| Case | Evidence | Suspects | Error on `quarter_correct` |
|---|---|---|---|
| 01 | `half(1)` and `quarter(1)` fail; `eighth` passes | `half` for `quarter`; `half` and `quarter` for `eighth`, which holds | the code of `half` [1.00, 1.00], then of `quarter` [0.00, 1.00] |
| 02 | the same failures; `not quarter_uses` | only `quarter` for `eighth` | the code of `quarter` [1.00, 1.00]; `quarter_uses` fails too |

Cases 03 to 07 have `claim quarter_uses : only uses quarter [half]` and a
failing `quarter`:

| Case | Evidence | Root cause |
|---|---|---|
| 03 | vouched; `half` holds | `quarter` itself |
| 04 | not vouched; `half` holds | none |
| 05 | vouched; `half` tested on 0 only | `quarter` itself, or where no claim covers `half` |
| 06 | vouched; `half` holds, but fails on 7, outside its claim | none |
| 07 | vouched; a second claim `uses quarter [eighth]`, `eighth` without a claim | `quarter` itself, or where no claim covers them |

### `types`: records, tuples and lists

Each case has its own model.

| Case | Model and evidence | Verdicts |
|---|---|---|
| 01 | `add` on `Number`, against the spec `add_spec`; the four outputs of the small domain; `uniform add` vouched, one `correct` | all hold |
| 02 | an output with the wrong unit, at 0.9 | `add_is` fails; error on the code and the evidence |
| 03 | one `correct` on records, nothing vouched | `add_is` unverified, `add_ok` holds assuming [add_uniform] |
| 04 | `sprint` returns a `Sprint`; `healthy` is claimed equal to `sprint(..).delivered >= sprint(..).committed`; sprint 3 slipped at 0.7 but `healthy` says true | both claims fail |
| 05 | `divmod` returns `(Int, Int)`, compared with a spec and with `(dm(a, b).0, 1)` | both fail on `divmod(3, 1)` |
| 06 | `sort` on `List(Int)` with `uniform sort by size` (empty, one, many); `[]` and `[3, 1, 2]` pass; `sort([7]) = [7]`; `total` on `List(Number)` | `sort_ok` unverified: class "one"; `sort_one` holds |
| 07 | a passing list in each class; evidence on every `ns` | `sort_ok` and `total_ok` hold |

### `enums`: variants with data, and `match`

Three models: `get` on `Option(Int)` with the classes "none" and "some"
(01 to 03), `sub` returning a `Result` (04, 05), and the enum `Color`
(06, 07).

| Case | Evidence | Verdicts |
|---|---|---|
| 01 | `get(None) = 0`, `get(Some(4)) = 4`, `get_uniform` vouched | `get_is` holds, both classes assumed |
| 02 | `get(Some(4)) = 5` | `get_is` fails on `get(Some(4))` |
| 03 | only `get(Some(4)) = 4` | unverified: class "none" |
| 04 | `sub(5, 3) = Ok(2)`, `sub(3, 5) = Err("underflow")`, uniformity vouched | `sub_is` holds |
| 05 | `sub(3, 5) = Ok(0)` | `sub_is` fails on the whole class `a < b`, 45 inputs |
| 06 | every `Color` observed; `pick(None) = Red`; `correct` over `Option(Color)` | all hold |
| 07 | one colour and two inputs of `pick` | unverified, the missing variants listed |

### `properties`: claims on any expression

| Case | Model and evidence | Verdicts |
|---|---|---|
| 01 | `len(sort(xs)) == len(xs)` with `uniform sort by size`; one output per class | `sort_len` holds |
| 02 | `sort([5]) = [5, 5]` at 0.9 | `sort_len` fails on `xs = [5]` |
| 03 | `total(sort(xs)) == total(xs)`; three outputs that can't all be right | fails; the code of `total` and `sort` and the three outputs are the candidates |
| 04 | associativity of `add`, `add(a, 0) == a`, bounds of `max` | inputs found from the observed calls; `max_bounds` fails on `a = 5, b = 2` |
| 05 | `pick(Some(c)) == c` over `Color`; `pick(None) == Red` | holds; fails on the statement itself |
| 06 | `slip_rate("red") > 0.5` with observed rates | one holds, one fails |

### `collections`: sets and maps as values

| Case | Model | Verdicts |
|---|---|---|
| 01 | a stock `Map(String, 0..)`, with a spec that adds to an item; `remove` takes a set of keys | `add_is` fails on the one output with a wrong count; the claim on `keys` passes on three inputs |
| 02 | `tags(xs: List(String)) -> Set(String)`, classes "none" and "some" | every claim holds; the set operators, checked on constants |
| 03 | `alarm(readings: Map(String, Level), muted: Set(String))` | fails on the input where the muted door still rings |

### `state`: a state as a value, and traces

| Case | Model | Verdicts |
|---|---|---|
| 01 | `deposit` and `withdraw` on an `Account`, with specs and classes; the trace of one passing test, three calls | nothing fails; each call counts for its own function |
| 02 | the same test, failing: the first `withdraw` returns a wrong balance | `withdraw_is` fails on that call and names the code of `withdraw`; `deposit` passes |
| 03 | `open_account() -> Account`, `log(message: String) -> ()` | the claims on `open_account` hold on its one input |

### `recursion`: types and specs that contain themselves

| Case | Model | Verdicts |
|---|---|---|
| 01 | `enum Expr`, a spec `value` that calls itself, classes by the outer variant | `eval_is` fails on `Neg(Neg(Lit(2)))`, passes on two others |
| 02 | `record Dir` with a list of `Dir`; `fold`, `map`, `filter`, `any`, `all` | `usage_is` holds by its classes; a claim on constants holds |
| 03 | a type inside itself through `Option`, a list, a set, a map, and as the parameter of `Tree` | nothing fails; the inputs are checked all the way down |

### `kit`: type parameters and function types

| Case | Model | Verdicts |
|---|---|---|
| 01 | `record Pair(A, B)`, a list `Node(T)` through `Option`, `swap(A, B)`, `first(T)` | a claim on constants holds; `Pair("a", 1)` is not a `Pair(Int, String)` |
| 02 | `apply(f: (Int) -> Int, x: Int)` against `f(x)`; `each` passing a spec and a subject | `apply_is` fails on `apply(negate, 3)` and passes on `apply(double, 3)` |

### `measures`: rates, precision, recall

One model of a team, `goal_met(team, sprint)`, with three claims on a
`rate` (cases 01 to 05), and one of a filter, `is_spam` (06).

| Case | Evidence | Verdicts |
|---|---|---|
| 01 | the rate over sprints 11 to 20 measured at 0.4, trust 0.7 | `adapts` (>= 0.6) fails; the measurement and the code are the candidates |
| 02 | measured at 0.8; a rate over `1..` measured at 0.9 | `adapts` and `ever` hold |
| 03 | three outputs only | all unverified; "2 of 4 inputs counted, 1 true" |
| 04 | the four outputs of sprints 1 to 4 | `steady` holds, by counting |
| 05 | three outputs true out of four, and the rate measured at 0.25 | error `M005`; `steady` unverified |
| 06 | four answers of `is_spam` and their judgments | `precise` and `few_false_alarms` fail, `finds_most` holds |

### `numbers`: integer types, ranges and floats

```lean
opaque inc(x: Int64) -> Int64
def halve(x: Float64) -> Float64 := x * 0.5
opaque half(x: Float64) -> Float64
claim inc_correct : forall x, correct(inc(x))
claim inc_small : forall x in 0..100, correct(inc(x))
claim half_spec : forall x in {1.0, -3.0}, half(x) = halve(x)
```

| Case | Evidence | inc_correct | inc_small | half_spec |
|---|---|---|---|---|
| 01 | `inc` on `0..=99`; `half(1.0) = 0.5`, `half(-3.0) = -1.5` | unverified, tested 0..=99 of Int64 | holds | holds |
| 02 | `half(-3.0) = -1.0`; `inc` wrong on `-5..0` at 0 | fails, refuted -5..=-1 | unverified | fails, refuted {-3.0} |
