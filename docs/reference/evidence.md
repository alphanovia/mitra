# Evidence, ties and tests

Part of the [language reference](language.md); says what the code does
today, and changes in the same commit as the code.

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
