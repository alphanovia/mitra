# The reworked language (prototype, stage 1)

What the code reads today in the reworked syntax, and nothing more. It is
a front end: each declaration is translated to the
[earlier syntax](language.md), and the same engine gives the verdicts,
the errors and the ranking. Both syntaxes are read; a file is in one or
the other. Examples: [`examples/reworked/`](../../examples/reworked).

One rule for every declaration: `keyword name(inputs) -> Output { body }`.

## Model

```rust
enum NetError { Timeout, Refused }
enum Result<T, E> { Ok(T), Err(E) }
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
| `boundary name { … }` | The environment. Same namespace rule. Today it differs from a subject only by what you write in it (`assume`). |
| `fn f(a: T) -> U;` | No body: the code is opaque. Without `-> U` the result is `()`. |
| `fn f(a: T) -> U { e }` in a subject or boundary | Opaque code **and** its specification: Mitra adds the claim `name::f::spec`, "the code returns what the body returns". |
| `fn f(a: T) -> U { e }` outside | A helper for the model: computed, no code behind it, no claim. |
| `fn f(…) -> U uses g, h;` | What the code calls; `uses only g, h` when it calls nothing else. Adds the claim `name::f::uses`, taken as declared (evidence with source `declared`). |
| `claim c(a: T, b: U) when cond { e }` | For every input where `cond` holds, `e` is true. `when` is optional. A type may be left out (`claim c(a, b)`): it is the type of the function the input is passed to. |
| `claim c(a: T) trust 0.8 { e }` | The trust given to the statement itself, 1 when not said. In an error the claim is a candidate like evidence. Also on `assume`, and after a `uses` list. |
| `claim c { given a: T, b: U  when cond  then e }` | The same claim, as a contract. |
| `claim c(a: T) { correct(f(a)) }` | The code is right on these inputs, as judged by a test or a person. No `when` with it yet. |
| `claim c = uniform(f);` | Every input of `f` behaves the same: one tested input stands for all. |
| `claim c = uniform(f, \|a, b\| b == 0);` | The same within each class. The class is a Bool, or typed: `\|m\| -> {"x", "y"} { m.kind }`. A function name also works: `uniform(f, g)`. |
| `assume c() { e }` | A claim taken as true without evidence (evidence with source `assumed`). Evidence against it is an error. |
| `record R { a: T, b: U }`, `record Box<T> { item: T }` | Values are written `R(1, 2)`. |
| `enum E<T> { A, B(T), C(x: T, y: T) }` | Unnamed fields are named `v0`, `v1`. |
| `test t { for f  given a = 1  gives 2 }` | A test of the code, for `mitra ingest`: same clauses as evidence, without `trust`, `source`, `rev`. |

Records, enums, their variants and tests are global, also when written
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
`.mitra/*.mitra` file (not `evidence.mitra`), such as `tests.mitra`. It is
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
also lists the functions of the model with no tie and the tests with no
`fn` of their name; these are printed, not recorded.

A test of the model is the Rust test of the same name: `mitra ingest`
turns its result into evidence.

The earlier form, the model in `// mitra:` comments, is still read, in
either syntax. `mitra add 'evidence e { for f … }'` accepts a block.

## Not yet

- Reports quote claims and evidence as you wrote them, but verdict lines,
  `mitra next` and stored evidence still use the earlier syntax and the
  generated names (`loader::load::spec`, `c::case`).
- Names that are keywords of the earlier syntax can't be used: `by`, `at`,
  `from`, `not`, `def`, `forall`, `uniform`, `correct`, and others.
- Claims with parameters as a library (`represented`, your own),
  inline functions outside `uniform`, set comprehension, `correct` inside
  an expression, `mitra explain <claim>`.
- A spec standing in for its code in other claims, and a vouched property
  passing observations on.
- The reader reads text, not Rust's syntax tree: macros, trait methods
  and two functions of the same name can mislead it.
- No trust on a signature (it is 1).
- Boundaries that answer differently each time, distributions,
  `holds assuming [an assumption]`, simulations, sequences.
- A subject inside a subject.
