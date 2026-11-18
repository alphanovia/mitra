# Mitra language reference

The one language Mitra reads, and its commands. This page says what the
code does today, and nothing more; it changes in the same commit as the
code. Examples: [`examples/reworked/`](../../examples/reworked).

One rule for every declaration: `keyword name(inputs) -> Output { body }`.

## A project, start to end

```sh
mitra init                         # creates .mitra/ beside src/
$EDITOR .mitra/model.mitra         # the model, aside from the code
mitra eval                         # the model alone: errors, verdicts
mitra plan                         # the evidence owed: the tests to write
# write the code and the tests, with the two `// mitra:` comments
R=$(git rev-parse --short HEAD)
mitra tie --rev $R                 # code against model, as evidence
cargo test > report.txt 2>&1
mitra ingest report.txt --rev $R   # test results, as evidence
mitra eval --rev $R
mitra add 'evidence r { for div_rest holds source "review:me" }' --rev $R
```

Without a store, name the two files: `mitra eval model.mitra
evidence.mitra`.

## Commands

```sh
mitra init
mitra check [model.mitra evidence.mitra] [--rev <commit>]
mitra eval  [model.mitra evidence.mitra] [--rev <commit>] [--candidates <n>]
            [--threshold <p>] [--all] [-o result.txt]
mitra plan  [model.mitra evidence.mitra] [--rev <commit>]
mitra next  [model.mitra evidence.mitra] [--rev <commit>]
mitra add   '<evidence blocks>' [--source <who>] [--rev <commit>]
mitra ingest <report> [--format libtest|junit] [--rev <commit>]
mitra tie   [--rev <commit>]
mitra explain <code | claim | std::module>
```

When Mitra can't read or check its input it prints `file: line N:
message` and exits with status 2.

- **The store** is the nearest `.mitra/` at or above the current folder:
  `model.mitra`, every other `.mitra` file beside it (read after it, in
  name order) and `evidence.mitra`. `init` creates it, with a
  `.gitattributes` that merges `evidence.mitra` with git's `union`
  driver, and never overwrites one. The evidence file is only appended
  to, so two branches that add evidence merge as the union of their lines.
- **`check`** reads and evaluates, and prints `ok: F functions, C claims,
  E evidence`.
- **`eval`** prints each claim's verdict, then errors and warnings (see
  Output), to standard output or to the `-o` file (not a `.mitra` file).
  It prints `mitra: E errors, W warnings` on standard error and exits
  with 1 when there is an error. An error lists at most 5 candidates and
  none under a plausibility of 0.05; `--candidates`, `--threshold` and
  `--all` change that.
- **`plan`** lists the evidence each claim owes (see `mitra plan`).
- **`next`** prints the evidence that can't all be true, each failing
  claim (root causes first), each claim that `holds assuming` with what
  it assumes, then each unverified claim with what it misses and the
  evidence block to write, when at most 5 inputs are missing. With a
  `uniform` claim it lists the classes with no passing input.
- **`add`** takes evidence blocks as one argument, writes `--source` and
  `--rev` into each block that does not say them (twice is an error),
  checks the store with the new evidence, and appends the blocks as
  written. The store must evaluate without a read error before and
  after; otherwise nothing is written.
- **`ingest`** and **`tie`**: see Tests and The tie to the code. Report
  formats: `libtest` (lines `test <path> ... ok|FAILED` as `cargo test`
  prints them) and `junit` (`<testcase name="…">`, failed with
  `<failure` or `<error`, left out with `<skipped`); a report starting
  with `<` is JUnit. A test's name is the last `::` part of its path. A
  test that appears twice in a report is left out, with a note.
- **`explain`** describes an error code (`mitra explain M004`), prints a
  library module (`mitra explain std::claims`), or prints a claim as
  written, as read, the count of evidence on it and its verdict
  (`mitra explain div_rest`, the last part of the name is enough).
- **`--rev R`** on `check`, `eval`, `next`, `plan` counts only evidence
  recorded at `R`: any other, or none, is stale and left out, and the
  number left out is printed. Revisions are compared as text. A revision
  must not be empty, start with `-`, or hold quotes or line breaks.

## Model

```rust
import std::rust::{Result};
enum NetError { Timeout, Refused }
record Config { retries: 0..=5, verbose: bool }

boundary network {
    fn fetch(url: {"a", "b"}) -> Result<String, NetError>;
    assume mostly_up() { rate(u in {"a", "b"}, fetch(u) is Ok) >= 0.5 }
}

fn default_config() -> Config { Config(3, false) }

subject loader {
    fn parse(text: String) -> Result<Config, String>;
    fn load(url: {"a", "b"}) -> Config uses network::fetch, parse;

    claim falls_back(url: {"a", "b"}) when network::fetch(url) is Err {
        load(url) == default_config()
    }
}
```

| Declaration | Meaning |
|---|---|
| `subject name { … }` | Real code under review. A namespace: its functions and claims are `name::f`. Inside, a sibling is named without the prefix. |
| `boundary name { … }` | The environment. Same namespace rule. Its functions may answer differently each time, and are never the piece to fix (see Boundaries). |
| `fn f(a: T) -> U;` | No body: the code is opaque. Without `-> U` the result is `()`. |
| `fn f(a: T) -> U { e }` in a subject or boundary | Opaque code **and** its specification: Mitra adds the claim `name::f::spec`, "the code returns what the body returns". |
| `fn f(a: T) -> U { e }` outside | A helper for the model: computed, no code behind it, no claim. |
| `fn f(…) -> U uses g, h;` | What the code calls; `uses only g, h` when it calls nothing else. Adds the claim `name::f::uses`, taken as declared (evidence with source `declared`). |
| `claim c(a: T, b: U) when cond { e }` | For every input where `cond` holds, `e` is true. `when` is optional. A type may be left out (`claim c(a, b)`): it is the type of the function the input is passed to. |
| `claim c(a: T) trust 0.8 { e }` | The trust given to the statement itself, 1 when not said. In an error the claim is a candidate like evidence. Also on `assume`, and after a `uses` list. |
| `claim c { given a: T, b: U  when cond  then e }` | The same claim, as a contract. |
| `claim c(a: T) { correct(f(a)) }` | The code is right on these inputs, as judged by a test or a person. No `when` with it yet. |
| `claim c = uniform(f);` | Every input of `f` behaves the same: one tested input stands for all. With `import std::claims::{uniform};`, as for the next two. |
| `claim c = uniform(f, \|a, b\| b == 0);` | The same within each class. The class is a Bool, or typed: `\|m\| -> {"x", "y"} { m.kind }`. A function name also works: `uniform(f, g)`. |
| `claim c(f, T) { given a: T then f(a) == a }` | A claim with parameters: names without a type, and a contract as body. It is not a claim by itself. |
| `claim d = c(g, 0..=9) trust 0.9;` | That claim with `f` and `T` replaced: a function, a set, a value or an inline function `\|a, b\| b > 0`. Written above its use. |
| `import geometry::{Point, origin};` | The named declarations of `geometry.mitra`, beside this file, as if written here (`a::b::x` reads `a/b.mitra`). One file is one module. Nothing else of the file comes, except what it imports itself. |
| `import std::claims::{commutative};` | The same from the modules shipped with Mitra; `mitra explain std::claims` prints one. `std::claims`: `commutative`, `associative`, `idempotent`, `neutral`, `inverse`, `monotone`, `within`, `represented`, `uniform`. `std::rust`: `Option<T> { None, Some(value: T) }`, `Result<T, E> { Ok(value: T), Err(error: E) }`. |
| `claim c = represented(f, { (a, b) \| b != 0 });` | One tested input of the set stands for all of it; nothing is said outside. With `import std::claims::{represented};`. A set per input also works: `represented(abs, 0..=9)`, `represented(div, (0..=9, 1..=9))`. Several on one function. |
| `assume c() { e }` | A claim taken as true without evidence (evidence with source `assumed`): `holds assuming [itself]` until the boundary is observed. Evidence against it is an error. |
| `record R { a: T, b: U }`, `record Box<T> { item: T }` | Values are written `R(1, 2)`. |
| `enum E<T> { A, B(T), C(x: T, y: T) }` | Unnamed fields are named `v0`, `v1`. |
| `claim t() { f(6, 2) == 3 }` | A point claim: no input, one call pinned. It is what a test of the code checks (see Tests). |

Records, enums and their variants are global, also when written
inside a subject. `//` starts a comment.

Inside a body (of a `fn` or of a class `|a| …`), a call to a function of
a subject that has a body is a call to its spec: a spec is computed, it
never runs the code. A helper is declared above the bodies that call it.

`when` on the inputs alone (`when b != 0`) takes the refused inputs out of
the claim: they are not counted as tested, assumed or missed. This works
for a finite set of inputs. `when` on what the code returns
(`when fetch(url) is Err`) is an implication: where no output was
observed the claim is not judged.

### Types

A type is a set of values. `bool`, `i8`…`i64`, `u8`…`u32`,
`u64`/`usize` (`0..`), `f64`, `String`, and Mitra's `Bool`, `Int` (every
integer), `Float64`, `Value` (everything). Ranges `0..=9`, `1..`. Sets of values
`{"a", "b"}`. Tuples `(A, B)`, unit `()`. `Vec<T>`/`List<T>`, `Set<T>`,
`Map<K, V>`, `Option<T>` and `Result<T, E>` once declared as enums.
Function types `fn(A) -> B`. `A | B`, `A & B`, `A \ B`.

Mitra computes with 64-bit integers: a value outside them is an error,
never a wrap. Verdicts print sets in a normal form, with Mitra's own
names: `Bool`, `Int`, `Float64`, `0..=9`, `{1, 2}`, `List(Int)`,
`Option(Int)`, `(0..=2, {0}) | (3..=4, Int)`.

### Expressions

`+ - * /`, `== != < <= > >=`, `&& || !`, `a <=> b` (both true or both
false), `x in xs`, `x is Variant`, `r.field`, `t.0`, `xs[i]`,
`if c { a } else { b }`, `let x = e; rest`,
`match e { A => …, B(x) => …, _ => … }`, lists `[1, 2]`, sets `{1, 2}`,
maps `{k: v}`, calls `f(a, b)`, `rate(x in S, cond)`, `precision(f)`,
`recall(f)`, `false_positive_rate(f)`, `all`/`any`/`filter`/`map(x in xs, e)`,
`fold(x in xs, let acc = start; step)`.

| Expression | Meaning |
|---|---|
| `(\|a, b\| a - b)(3, 1)` | An inline function, on its inputs. It exists only applied: it is not a value. |
| `{ x in xs \| x > 2 }` | The elements of a list or a set where the condition holds. A union on the left goes in parentheses. |

## What a claim rests on without evidence on it

A claim that calls a function of a subject needs the output of that call.
Where none was observed, Mitra looks for it in this order.

1. **A vouched claim of the shape `f(a, b) == e`** (either side, each
   input a different name or a written value, the evidence on the whole
   claim and no evidence against it): `e` is worked out from what was
   observed, and stands for the output. The vouch is then among the
   evidence of the claim that read it, so a wrong result names it.

   ```rust
   claim add_commutes(a: 0..=2, b: 0..=2) { add(a, b) == add(b, a) }
   claim more_one(a: 0..=1) { add(1, a) == a + 1 }   // reads add(1, 0)
   evidence t0 { for add given a = 0, b = 1 gives 1 }
   evidence r { for add_commutes holds source "review:me" trust 0.9 }
   ```

   `more_one` holds: `add(1, 0)` comes from `add(0, 1)`. A claim never
   uses the vouch on itself. `examples/reworked/vouched`.

2. **The specs of the functions it calls** (`name::f::spec`, declared
   above the claim): when they imply the claim on every input, the claim
   is `assumed on` those inputs and holds as far as the spec claims do
   (`holds assuming [clock::back::spec]` when one is not verified). When
   the specs say the claim is false on an input, that is error M007: the
   claim fails there, with the claim and the specs as candidates, and no
   evidence needed. `examples/reworked/clock`.

## Boundaries

- Two different outputs on one input of a boundary function both stand:
  no contradiction. A claim that reads it is judged on every answer seen,
  and fails if one answer breaks it.
- A boundary function is never a candidate in an error: there is no code
  of yours to fix. The candidates are the subjects, the claim, the
  evidence and the assumptions.
- An `assume` of the shape `f(…) == e` gives the output where none was
  observed, as a vouched claim does. Every claim that read it is
  `holds assuming [the assumption]`, with warning M102. The assumption
  itself prints `holds assuming [itself]`, without a warning, until
  observations cover it; then it holds, or fails.

`examples/reworked/network`.

## `correct(…)` as a value, `represented` and `uniform`

`correct(f(a, b))` alone in a claim is the claim on the code. Inside a
larger expression it is a value with three states: `true` when evidence
judges that call right (a `holds` on the function, or an observed output
that a spec of the function gives), `false` when wrong, and no value
when nothing says. Where it has no value the claim is not judged.

```rust
claim mirror(a: 0..=2, b: 0..=2) { correct(add(a, b)) <=> correct(add(b, a)) }
```

- Both calls judged, and differently: `mirror` fails there.
- `mirror` vouched for on the whole claim: it settles the calls nothing
  judges. `add(0, 1)` judged right makes `add(1, 0)` right: a claim on
  the code of `add` prints it under `assumed on …, by mirror`. This
  works on a finite set of inputs, for calls written with the claim's
  inputs (`add(b, a)`, not `add(a + 1, b)`).

`std::claims` writes `represented` and `uniform` with it; `x` and `y`
stand for the whole inputs of `f`:

```rust
claim represented(f, S) { given x: S, y: S then correct(f(x)) <=> correct(f(y)) }
claim uniform(f) { given x, y then correct(f(x)) <=> correct(f(y)) }
claim uniform(f, case) { given x, y when case(x) == case(y) then correct(f(x)) <=> correct(f(y)) }
```

Mitra reads a claim of this shape, whatever its name, as a set or as
classes, and works on them whole: this is the only place where one name
may stand for all the inputs of a function, and it also works on types
too large to list. The shape is exactly: two inputs, both with the same
set or both with none, an optional `when k(x) == k(y)`, and
`correct(f(x)) <=> correct(f(y))` (or `==`).

Nobody can compute these claims: someone vouches for them (`evidence r {
for div_rest holds source "review:me" }`). For a claim on the code of
`f` (`correct(f(…))`, or a spec) and each `represented` claim on `f`:

- an input of the set passes and none fails: the rest of the set is
  `assumed on …, by` the claim. Not vouched for, the claim is among what
  the verdict assumes (`holds assuming [div_rest]`) and has warning M103;
- an input passes and another fails: the `represented` claim fails with
  error M002, quoting the two, and stands for nothing;
- no input of the set is tested: nothing. `mitra plan` then owes
  `any one point of { (a, b) | b != 0 }`.

The set of `represented` is `{ (a, b) | cond }` over the inputs of the
function (a finite set of inputs only), or a set per input: `0..=9`,
`(0..=9, 1..=9)`, on any type. Several `represented` on one function;
one `uniform`, which sorts every input. `examples/reworked/represented`.

## `mitra plan`: the evidence owed

`mitra plan` lists, for each claim that does not hold yet, the evidence
it still owes. With no evidence it is the list of tests to write, from
the model alone; with evidence, what is left. `nothing owed` when every
claim holds.

```
calc::div::spec owed:
  any one point of { (a, b) | !(b == 0) }
  any one point of { (a, b) | b == 0 }
  and `calc::div_classes`, which says one point is enough
calc::div_classes owed: a review that `calc::div` behaves the same within each class
calc::div_small owed:
  calc::div(6, 2), expected 3
  or nothing, once calc::div::spec holds
calc::add_commutes owed:
  calc::add(0, 0)
  calc::add(0, 1)
```

- A claim with a `uniform` claim on its function owes one point per class
  not yet tested; with `represented` claims, one point per set not yet
  tested, then the points no set covers.
- Otherwise it owes each call it reads and that was not observed, when
  there are at most 12 points; `expected` is what the spec or the claim
  itself says the output is (`f(…) == e`, with `e` computed).
- A larger or infinite set owes "every point, or a claim one point can
  stand for".
- A claim that holds only through other claims (`holds assuming`) still
  lists its own points, with "or nothing, once … holds".
- `uniform`, `uses` and ties owe a review or `mitra tie`. An assumption
  owes nothing. A failing claim owes nothing: `mitra eval` says why.

A pinned point is a claim: `claim div_small() { div(6, 2) == 3 }`.
`examples/reworked/plan`.

## Evaluation

Every piece of evidence counts, whatever its trust. For each claim, with
D the inputs it speaks of, Mitra gathers where the claim is right and
where it is wrong:

1. Evidence on the claim itself (`holds`, `fails`), within D.
2. For `correct(f(…))`: `holds` and `fails` evidence on `f`.
3. For any other claim, on each input (all of D when finite, otherwise
   the inputs the observations speak of): compute the expression, a call
   of the code giving the output observed there (or what a vouched claim
   or a spec gives, see above). True is right, false is wrong, no value
   is not judged. `&&` and `||` are decided by one side when it settles
   them.

An input both right and wrong is a **contradiction**, and so are two
outputs for one input of a subject: an error, and the input counts on
neither side. Then:

- *refuted*: wrong and not right.
- *tested*: right and not wrong.
- *assumed*: what other claims give: the inputs a `uniform` or
  `represented` claim covers from a passing input, and the inputs the
  specs imply.
- *missed*: the rest of D.

The **verdict** is `fails` if something is refuted; `unverified` if
something is missed; `holds` if nothing is missed and every claim the
assumed inputs rest on holds; `holds assuming [a, b]` when those claims
are not verified or not vouched for. There is no number on a claim.

**Uniform.** For `u = uniform(f, g)`, the class of an input is `g` of
it. A class with a passing input and a failing one is a conflict: `u`
fails, whatever vouches for it (M004). Otherwise, for every other claim
on `f`, the inputs of a class with a passing input are assumed; when
nobody vouches for `u`, such a claim only `holds assuming [u]`. While `u`
holds, a failing input refutes its class. This is done input by input up
to 10 000 inputs; on a larger set every value of `g`'s result type needs
a passing input. A claim that calls several functions is sorted by one
class per call.

**Measures.** `rate(…)`, `precision(f)`, `recall(f)` take the value of
their measurement (`gives rate = 0.9`) when there is one; without one
they are counted from the observed outputs when these cover the whole
set. Two measurements that differ, or one the observed outputs exclude,
are error M005.

**Errors and candidates.** An error is something that can't all be true.
Its candidates are the pieces of which at least one is wrong: evidence,
the code of the functions involved (and of what they use), and what the
model states (the claim, the signature, an assumption, a `uses` clause).
Code has no trust; what the model states is trusted at 1 unless it says
`trust n`. Evidence outside a signature is kept and reported as M006,
with the evidence and the signature as candidates. A boundary is never a
candidate.

**Most plausibly wrong.** For each error Mitra computes, for every
candidate, the belief and the plausibility that it is wrong, with
Dempster's rule (`mitra_ds::culprits`): a piece trusted at `t` puts `t`
on "this piece is right"; each error says its pieces are not all right.
Several pieces can be wrong at once, and errors that share a piece are
computed together. When the pieces trusted at 1 can't all be right, the
doubt is shared equally between them. More than 20 pieces to weigh are
not ranked.

**Suspects and root cause.** Through the `uses` clauses, a function that
calls a function with a failing claim has it as a suspect (`look first
at …`). A function with a failing claim, no suspect and a `uses only`
that holds is a root cause.

## Output

`eval` prints each claim, in model order, with the lines that say
something:

```text
calc::div::spec: unverified
  passes on       `calc::div(6, 2)`
  assumed on      89 inputs, by `calc::div_rest`
  no evidence on  (a, b) in {(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0)}
calc::div_small: holds
```

`fails on`, `passes on`, `assumed on … by` and `no evidence on`. Up to 3
inputs print as calls, more as a set named after the inputs, and a set
longer than 160 characters as its size. Names are whole:
`subject::claim`, `subject::f::spec` for the claim a body adds,
`subject::f::uses`, `subject::f::tied`.

Then each error and each warning, like a compiler's:

```text
error[M001]: evidence refutes a claim
  --> model.mitra:4  fn div(a: 0..=9, b: 0..=9) -> 0..=9 { if b == 0 { 0 } else { a / b } }

  `calc::div::spec` fails on `calc::div(8, 4)`.
  The evidence that says so:

    output    evidence.mitra:3  evidence bug_report { for div given a = 8, b = 4 gives 4 trust 0.8 }

  The code, the claim or the evidence is wrong. Below, the trust given to each
  piece, and what is left of it once they are weighed against each other:

                                                      trust  left
    possible  the code of `calc::div`, model.mitra:4  none   0.00 to 0.71
    likely    evidence.mitra:3 (bug_report)           0.8    0.29 to 0.36

  The lower what is left, the more plausibly the piece is the wrong one.
  A range means the evidence does not settle it.

  help: if the evidence is right, fix `calc::div`; if not, correct the evidence
```

`left` is what remains of the trust in a piece: 1 minus the plausibility
that it is wrong, to 1 minus the belief. The word is `certain` (belief
at least 0.995), `likely` (at least 0.5), `possible` (plausibility at
least 0.5), else `unlikely`. At most 5 pieces of evidence are quoted.

| Code | Severity | Headline |
|---|---|---|
| `M001` | error | evidence refutes a claim |
| `M002` | error | evidence denies an assumption (`uniform`, `represented`, `uses`) |
| `M003` | error | two pieces of evidence contradict each other |
| `M004` | error | an assumption is contradicted by the evidence: a pass and a failure in one class |
| `M005` | error | a measurement does not fit another measurement of the same measure, or the observed outputs |
| `M006` | error | evidence does not fit the model: inputs or an output outside the signature of its function |
| `M007` | error | a claim contradicts the specs of the functions it calls, with no evidence needed |
| `M101` | warning | a claim is not verified: some inputs have no evidence |
| `M102` | warning | a claim holds only under assumptions |
| `M103` | warning | an assumption is not vouched for |
| `M104` | warning | evidence that can't be used: it is about a helper, which is computed |

The codes are stable. `mitra explain <code>` says what one means and how
to fix it.

## Evidence

```rust
evidence a_is_down {
    for network::fetch
    given url = "a"
    gives Err(Timeout)
    trust 0.9
    source "trace:run_1"
}
```

| Clause | Meaning |
|---|---|
| `for name` | A function or a claim. The last part of the name is enough when only one has it (`for fetch`). |
| `given a = value, b in Set` | The inputs, by name. An input left out is every value. |
| `holds` / `fails` | On a function: the code is right / wrong on these inputs. On a claim: the claim is true / false on these inputs; without `given`, someone vouches for the whole claim. |
| `gives value` | The output observed, on a function with one value per input. |
| `gives rate = 0.9` | A measurement, on a claim that holds one measure. |
| `trust n` | 0 to 1, default 1. |
| `source "…"` | Default: the name of the block. |
| `rev "…"` | The revision of the code. |

Evidence is written in blocks only. `mitra add`, `mitra ingest` and
`mitra tie` append blocks to `evidence.mitra`. `//` starts a comment.

## The tie to the code

The model lives aside from the code: `.mitra/model.mitra`, and any other
`.mitra/*.mitra` file (not `evidence.mitra`), such as `points.mitra`. It is
checked alone, with no code present. The code points at it:

```rust
// mitra: implements bank::withdraw
pub fn withdraw(&self, n: u64) -> (Account, bool) { … }
```

`mitra tie [--rev R]` reads `src/` and `tests/`, compares each tied `fn`
with the function of the model, and appends what it finds as evidence
(source `reader:rust file:line`, with the reason when they differ):

| It compares | Agreement | Evidence on |
|---|---|---|
| Inputs | The same number (`self` counts as one), the same names, and the same types where the model's type is a plain one (`bool`, `f64`, `String`, a record, `Vec<T>`, `Option<T>`, a tuple; every integer type is `Int`). A narrower type in the model (`0..=5`, `{"a", "b"}`) is not compared. | `f::tied` |
| Result | The same rule. | `f::tied` |
| Calls | For `uses g, h`: the body calls the `fn` tied to each. For `uses only`: it calls no other tied `fn`. A function with no tie is not checked. | `f::uses` |

`f::tied` is a claim that exists once evidence names it: without a
reader, a reviewer brings the same evidence
(`evidence e { for withdraw::tied holds source "review:me" }`). When it
fails, the candidates are the evidence, the signature in the model and
the code. A tie to a function the model does not have is refused. It
also lists the functions of the model with no tie; these are printed,
not recorded.

## Tests

The model has no tests, only claims. A test of the code is evidence for
the claim its comment names:

```rust
// in the model
claim div_small() { div(6, 2) == 3 }

// in the code, above the test
// mitra: evidence for div_small
#[test]
fn div_by_two() { assert_eq!(div(6, 2), 3); }
```

`cargo test > report.txt 2>&1`, then `mitra ingest report.txt --rev R`
appends one block per tied test that ran, `evidence test { for
calc::div_small holds source "test:div_by_two" rev "R" }`, with `fails`
when it failed.

- The test's name is free. The claim is named in full
  (`calc::div_small`) or by its last part when only one claim has it.
  A comment that names no claim of the model is refused.
- Any claim can be named. On a point claim the result also says
  something of the code:

  | Claim | Test passed | Test failed |
  |---|---|---|
  | `f(6, 2) == 3` | The output `f(6, 2) = 3` was observed, for every claim that reads it. | The specs that give 3 there are refuted; on a `bool` result, the other output was observed. |
  | `correct(f(6, 2))` | The code is right on that call. | The code is wrong on that call. |

  When a claim `correct(f(…))` exists on `f`, the first row also judges
  the call right or wrong.
- Several tests may name one claim, and one test several claims (one
  comment each).
- `ingest` lists the tests of the report tied to no claim, the failed
  ones apart: no verdict shows them.
- `mitra plan` lists the point claims with no evidence yet: the tests
  left to write.
- `test t { … }` in the model is refused, with this section's advice.

## What is no longer read

The syntax of before the rework (`opaque`, `def … :=`, `claim c : forall
…`, `evidence … at 1`, `-- comments`) is refused, with what to write
instead. Inside, Mitra still translates a model to that form before
evaluating it: it is not a language to write in, and the engine's own
tests (`tests/golden`) are kept in it.

A `// mitra:` comment is one of these two ties. A model written in
comments is refused.

## Not yet

- Some generated names still show: `c::case` (the classes of a `uniform`
  claim written inline) and `f::body` in `mitra explain`, and a few
  messages of the checker use the internal form's words.
- No formal grammar of the language is checked yet.
- Names that are keywords of the internal form can't be used: `by`, `at`,
  `from`, `not`, `def`, `forall`, `uniform`, `correct`, and others.
- A vouched claim that reads `correct(…)` settles other calls only on a
  finite set of inputs; `represented` and `uniform` have no such limit.
- Several `uniform` claims on one function (several `represented` work).
- Uniformity proven from a spec's branches (later).
- One name for the whole inputs of a function (`given x`, then `f(x)`)
  is read only in the shape of `represented` and `uniform`. An inline
  function is not a value. Two claims with parameters may share a name
  when they take a different number of parameters.
- A body can't call a boundary: a spec is computed, and a boundary is not.
- Distributions on what a boundary answers, sequences. Simulations: later.
- An imported file is read from the
  folder of the model; in a store, put modules in a subfolder
  (`.mitra/lib/pair.mitra`, `import lib::pair::Pair;`), since every file
  directly in `.mitra/` is already part of the model.
- An error inside an imported file is reported on the `import` line.
- The reader reads text, not Rust's syntax tree: macros, trait methods
  and two functions of the same name can mislead it.
- No trust on a signature (it is 1).
- A subject inside a subject.
