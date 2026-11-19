# Settings

Part of the reference. Design: [erasure.md](../design/erasure.md).

`.mitra/settings.toml`, beside the model. It holds the numbers the
trust of a tie and of a sample are computed from. With no file, the
defaults. `mitra build` writes the settings in force, defaults
included, to `.mitra/build/settings.toml`.

```toml
[sample]
tolerance = 0.001   # the share of its domain a sampled claim may fail on unseen
counts = true       # a passing sample lets a claim read `holds assuming [c::sample]`

[erasure]           # the cost of an erasure of the reader, from 0 to 1
width = 0.02
field = 0.05
```

The part of TOML that is read: the two tables, `key = value`, numbers,
`true`, `false`, comments. A table or a key that is not a setting is
refused with its line, and so is an erasure the reader does not have.

Settings never change a verdict from `holds` to `fails` or back. They
set the trust of two pieces, which ranks them among the suspects of a
contradiction.

## A sample

A passing `sampled n` is trusted for `1 - (1 - tolerance)^n`: had the
claim failed on more than `tolerance` of its domain, `n` draws would
have met a failure with that chance.

| n | 100 | 1000 | 3000 | 10000 |
|---|---|---|---|---|
| trust at 0.001 | 0.10 | 0.63 | 0.95 | 1.00 |

`counts = false`: a sample covers nothing, and the claim stays
`unverified` ([sampled](kw-evidence.md#sampled)).

## An erasure

What the reader of code may forget of a signature to reach the
model's. A tie that agrees names the erasures on its path and is
trusted for the product of `1 - cost` over them
([evidence.md](evidence.md)). The Rust reader's, with their costs:

| Name | Forgets | Cost |
|---|---|---|
| `reference` | `&`, `&mut`, `mut`, a lifetime | 0 |
| `pointer` | `Box`, `Rc`, `Arc` | 0.01 |
| `owned` | `Cow<str>`, `PathBuf` | 0.01 |
| `conversion` | `impl Into<T>`, `impl AsRef<T>` | 0.02 |
| `sequence` | `impl Iterator<Item = T>` read as a list | 0.03 |
| `width` | the width of the numbers of a signature, once | 0.02 |
| `bound` | a trait that bounds an input in the code alone | 0.02 |
| `range` | a type the model states narrower than the code can (`0..=5` for `i64`) | 0.05 |
| `any` | a type the model leaves as any value | 0.05 |
| `error` | the type of the error of a `Result` | 0.05 |
| `field` | a field of a struct the record does not state, without `..` | 0.05 |

A cost of 0 still names the erasure on the tie. A cost of 1 leaves the
tie no trust; it still agrees.

## What is not there

- No setting fails a tie on its distance, or a claim on the trust of
  its sample.
- One file per store; nothing is inherited from a folder above.
- A sample is not yet weighed against a failing point as a suspect:
  the failure wins.
