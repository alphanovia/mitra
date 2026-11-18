# The reworked language (prototype)

What the code reads today in the reworked syntax, and nothing more. It is
a front end: each declaration is translated to the
[earlier syntax](language.md), and the same engine gives the verdicts,
the errors and the ranking. Both syntaxes are read; a file is in one or
the other. Examples: [`examples/reworked/`](../../examples/reworked).

One rule for every declaration: `keyword name(inputs) -> Output { body }`.

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

`bool`, `i8`…`i64`, `u8`…`u32`, `u64`/`usize` (`0..`), `f64`, `String`,
and Mitra's `Bool`, `Int`, `Float64`. Ranges `0..=9`, `1..`. Sets of values
`{"a", "b"}`. Tuples `(A, B)`, unit `()`. `Vec<T>`/`List<T>`, `Set<T>`,
`Map<K, V>`, `Option<T>` and `Result<T, E>` once declared as enums.
Function types `fn(A) -> B`. `A | B`, `A & B`, `A \ B`.

### Expressions

`+ - * /`, `== != < <= > >=`, `&& || !`, `a <=> b` (both true or both
false), `x in xs`, `x is Variant`, `r.field`, `t.0`, `xs[i]`,
`if c { a } else { b }`, `let x = e; rest`,
`match e { A => …, B(x) => …, _ => … }`, lists `[1, 2]`, sets `{1, 2}`,
maps `{k: v}`, calls `f(a, b)`, `rate(x in S, cond)`, `precision(f)`,
`recall(f)`, `false_positive_rate(f)`, `all`/`any`/`filter`/`map(x in xs, e)`,
`fold(x in xs, let acc = start; step)`.

### Expressions added to the earlier ones

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

A line in the earlier syntax is still read in the same file: `mitra add`
and `mitra ingest` append such lines.

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
appends one line per tied test that ran: `evidence div_small at 1` when
it passed, `evidence not div_small at 1` when it failed, with source
`test:div_by_two`.

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

The earlier form, the model in `// mitra:` comments, is still read, in
either syntax. `mitra add 'evidence e { for f … }'` accepts a block.

## Not yet

- Reports quote claims and evidence as you wrote them, and `mitra next`
  asks for evidence in this syntax, but verdict lines and stored evidence
  still use the generated names (`loader::load::spec`, `c::case`) and the
  earlier line form.
- Names that are keywords of the earlier syntax can't be used: `by`, `at`,
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
