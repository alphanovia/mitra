# Mitra language reference

The one language Mitra reads, and its commands. The reference says what
the code does today, and nothing more; it changes in the same commit as
the code. Examples: [`examples/reworked/`](../../examples/reworked).

One rule for every declaration: `keyword name(inputs) -> Output { body }`.

The reference is in eight files, each short enough to read in one go:

| File | What it holds |
|---|---|
| [language.md](language.md) | This page: a project start to end, the model, types, expressions, what is not read |
| [claims.md](claims.md) | What a claim rests on without evidence, boundaries, `correct(…)`, `represented`, `uniform` |
| [evidence.md](evidence.md) | Evidence blocks, the tie to the code, tests |
| [levels.md](levels.md) | The four levels: `build`, `conform`, `assess`, `diagnose`, and the build files |
| [commands.md](commands.md) | The commands under them, and `mitra plan` |
| [ledger.md](ledger.md) | How the evidence file is kept: what a run replaces, stale and orphaned evidence, `mitra remove` |
| [stores.md](stores.md) | A store that leans on another one |
| [settings.md](settings.md) | The numbers the trust of a tie and of a sample come from |
| [evaluation.md](evaluation.md) | How verdicts are worked out, the output, the codes |

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

Without a store, name the files: `mitra eval model.mitra
evidence.mitra`, or the model alone to check it before any evidence
exists: `mitra check model.mitra`.

A subject may say what it promises in plain words, `ensure name =
"text";`; a reviewer then vouches that its claims say the same
([kw-requirements.md](kw-requirements.md)).

For one entry per keyword, each with a model that runs, see
[keywords.md](keywords.md).

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
| `subject { … }`, `subject fn f(a: T) -> U;` | The same with no name: what it holds is named as written (`f`). The second form is one function. Several per file. |
| `boundary name { … }`, `boundary { … }`, `boundary fn f(…);` | The environment. Same rules. Its functions may answer differently each time, and are never the piece to fix (see Boundaries). |
| `fn f(a: T) -> U;` in a subject or boundary | No body: the code is opaque. Without `-> U` the result is `()`. Outside both it is refused: write `subject fn` or `boundary fn`. In a subject, Mitra adds the claim `name::f::correct`, "the code is right on every input", unless the model has its own claim `correct(f(…))` with inputs, and `name::f::deterministic`, "the same input always gives the same output", unless the model has its own `deterministic(f)` or `!deterministic(f)`. |
| `fn f(a: T) -> U { e }` in a subject or boundary | Opaque code **and** its specification: Mitra adds the claim `name::f::spec`, "the code returns what the body returns". |
| `fn f(a: T) -> U { e }` outside | A helper for the model: computed, no code behind it, no claim, never a suspect. It needs a body. |
| `fn f(…) -> U uses g, h;` | What the code calls; `uses only g, h` when it calls nothing else. Adds the claim `name::f::uses`, taken as declared (evidence with source `declared`). |
| `claim c(a: T, b: U) when cond { e }` | For every input where `cond` holds, `e` is true. `when` is optional. A type may be left out (`claim c(a, b)`): it is the type of the function the input is passed to. |
| `claim c(a: T) trust 0.8 { e }` | The trust given to the statement itself, 1 when not said. In an error the claim is a candidate like evidence. Also on `assume`, and after a `uses` list. |
| `claim c { given a: T, b: U  when cond  then e }` | The same claim, as a contract. |
| `claim c(a: T) { correct(f(a)) }` | The code is right on these inputs, as judged by a test or a person. No `when` with it yet. Needed only to claim less than every input, or outside a subject. |
| `claim c = uniform(f);` | Every input of `f` behaves the same: one tested input stands for all. With `import std::claims::{uniform};`, as for the next two. |
| `claim c = uniform(f, \|a, b\| b == 0);` | The same within each class. The class is a Bool, or typed: `\|m\| -> {"x", "y"} { m.kind }`. A function name also works: `uniform(f, g)`. |
| `claim c(f, T) { given a: T then f(a) == a }` | A claim with parameters: names without a type, and a contract as body. It is not a claim by itself. |
| `claim d = c(g, 0..=9) trust 0.9;` | That claim with `f` and `T` replaced: a function, a set, a value or an inline function `\|a, b\| b > 0`. Written above its use. |
| `import geometry::{Point, origin};` | The named declarations of `geometry.mitra`, beside this file, as if written here (`a::b::x` reads `a/b.mitra`). One file is one module. Nothing else of the file comes, except what it imports itself. |
| `import std::claims::{commutative};` | The same from the modules shipped with Mitra; `mitra explain std::claims` prints one. `std::claims`: `commutative`, `associative`, `idempotent`, `neutral`, `inverse`, `monotone`, `within`, `represented`, `uniform`. `std::rust`: `Option<T> { None, Some(value: T) }`, `Result<T, E> { Ok(value: T), Err(error: E) }`. |
| `claim c = represented(f, { (a, b) \| b != 0 });` | One tested input of the set stands for all of it; nothing is said outside. With `import std::claims::{represented};`. A set per input also works: `represented(abs, 0..=9)`, `represented(div, (0..=9, 1..=9))`. Several on one function. |
| `assume c() { e }` | A claim taken as true without evidence (evidence with source `assumed`): `holds assuming [itself]` until the boundary is observed. Evidence against it is an error. |
| `ensure name = "text";` | In a subject: what it promises, in plain words. Mitra never reads the text. `s::name` rests on every claim of the subject, on `s::name::captured` (a reviewer's word that the claims say what the text says) and on what the subject requires. In a boundary it rests the same way on the `assume` lines of the block; with none, it is assumed. `trust n` before the `;`. [More](kw-requirements.md). |
| `require need = other::name;` | In a subject or a boundary: `s::need` holds as far as the `ensure` it names, and every `ensure` of the block rests on it. `require need = "text";` when nobody promises it yet: assumed. A name no `ensure` has is refused. |
| `record R { a: T, b: U }`, `record Box<T> { item: T }` | Values are written `R(1, 2)`. |
| `record R;`, `record R { a: T, .. }` | Contents left out, all or some: fields stated are read, no value can be written. Tied to its `struct` by `// mitra: implements subject::R`. |
| `impl R { fn m(self) -> T; claim c(self) { … } }` | Rust's form: `s::R::m`, `self` an `R`, `Self` the record; called `r.m()` or `R::m(r)`. |
| `contract C { fn f(&self) -> T; ensure p = "…"; }`, `impl C for R { }`, `x: impl C` | A role and what every type that fills it owes; each implementer is a group of its own. [More](kw-contracts.md). |
| `fn f<L: Link>(l: L)`, `where L: Link`, `impl<T> R<T> { … }`, `contract C<K> { … }` | Type parameters and bounds, as Rust writes them; a bound names contracts. [More](kw-generics.md). |
| `subject s { record R { … } }` | A record or an enum written in a subject or boundary belongs to it: `R` inside, `s::R` elsewhere. A claim on a record alone (`claim c(r: R) { … }`, no call) is judged on every `R` an observed output shows. |
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
integer), `Float64`, `Any` (everything). Ranges `0..=9`, `1..`. Sets of values
`{"a", "b"}`. Tuples `(A, B)`, unit `()`. `Vec<T>`/`List<T>`, `Set<T>`,
`Map<K, V>`, `Option<T>` and `Result<T, E>` once declared as enums.
Function types `fn(A) -> B`. `A | B`, `A & B`, `A \ B`.

Mitra computes with 64-bit integers: a value outside them is an error,
never a wrap. Verdicts print sets in a normal form, with Mitra's own
names: `Bool`, `Int`, `Float64`, `0..=9`, `{1, 2}`, `List<Int>`,
`Option<Int>`, `(0..=2, {0}) | (3..=4, Int)`.

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

`mitra check` works out the type of every expression from the text,
also in claims on code with no body, and refuses what can never fit
its place: `g(a) + "x"`, `g(a).size` on an `Int`, an input a function
never takes, a claim that is never true or false, a body that never
gives its result type. An integer and a float do not mix. What may
fit (a wider type) is read.

## What is no longer read

The syntax of before the rework (`opaque`, `def … :=`, `claim c : forall
…`, `evidence … at 1`, `-- comments`) is refused, with what to write
instead. Nothing in the tool reads or writes it any more: one reader
(`crates/mitra-lang/src/parse.rs`) builds the model from the language,
and the engine's tests (`tests/golden`) are written in it.

A `// mitra:` comment is one of these two ties. A model written in
comments is refused.

## Not yet

- No formal grammar of the language is checked yet.
- Reserved words are names only as fields, or written `r#word`
  ([names](kw-records.md#names-and-r)): the types (`Bool`, `Int`, `String`,
  `List`, `Set`, `Map`, `Any`, …) and `fn`, `claim`, `evidence`,
  `record`, `enum`, `match`, `if`, `else`, `let`, `in`, `true`, `false`,
  `correct`, `rate`, `precision`, `recall`, `false_positive_rate`, `len`,
  `keys`.
- A vouched claim that reads `correct(…)` settles other calls only on a
  finite set of inputs; `represented` and `uniform` have no such limit.
- Several `uniform` claims on one function (several `represented` work).
- Uniformity proven from a spec's branches (later).
- One name for the whole inputs of a function (`given x`, then `f(x)`)
  is read only in the shape of `represented` and `uniform`. An inline
  function is not a value. Two claims with parameters may share a name
  when they take a different number of parameters.
- A body can't call a boundary: a spec is computed, and a boundary is not.
  It can call the code of a subject (`fn sum3(..) { add(add(a, b), c) }`
  with `add` not modelled): the body is then not computed, and claims
  on it follow from the claims on `add` (claims.md).
- Distributions on what a boundary answers, sequences. Simulations: later.
- An imported file is read from the
  folder of the model; in a store, put modules in a subfolder
  (`.mitra/lib/pair.mitra`, `import lib::pair::Pair;`), since every file
  directly in `.mitra/` is already part of the model.
- An error inside an imported file is reported on the `import` line,
  with the module and its line in the message.
- The reader reads text, not Rust's syntax tree: macros, trait methods
  and two functions of the same name can mislead it.
- No trust on a signature (it is 1).
- A subject inside a subject.
