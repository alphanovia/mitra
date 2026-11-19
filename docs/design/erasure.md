# Erasures on a tie, the weight of a sample, and settings

Status: **built** (2026-11-19, increment 75). Reference:
[evidence.md](../reference/evidence.md), [settings.md](../reference/settings.md).

The user: "what we need is an erasure profile on ties. You take the
code signature, you apply erasure operation to get to model, if you
can't then it's a drift to report. If you can you have a strength that
is measured based on how many mutation you had to do." Then: "make
those parameters of the build and put them on a setting file".

## An erasure

A named operation of a reader that forgets one detail of the code. It
only forgets: it never adds an input, a variant or a failure. A tie
holds when a path of erasures leads from the code's signature to the
model's; where no erasure applies, the path stops and the difference
is reported as before.

The profile is the reader's. Rust's:

| Name | Forgets | Cost |
|---|---|---|
| `reference` | `&`, `&mut`, `mut`, a lifetime | 0 |
| `pointer` | `Box`, `Rc`, `Arc` | 0.01 |
| `owned` | `Cow<str>`, `PathBuf` | 0.01 |
| `conversion` | `impl Into<T>`, `impl AsRef<T>` | 0.02 |
| `sequence` | `impl Iterator<Item = T>` read as a list | 0.03 |
| `width` | the width of the numbers of a signature | 0.02 |
| `bound` | a trait that bounds an input in the code alone | 0.02 |
| `range` | a type the model states narrower than the code can (`0..=5` for `i64`) | 0.05 |
| `any` | a type the model leaves as any value | 0.05 |
| `error` | the type of the error of a `Result` | 0.05 |
| `field` | a field of a struct the record does not state, without `..` | 0.05 |

Not erasures, so still differences: another number of inputs, another
name of an input, another type, a failure the model does not state, a
variant the model does not state, a call the model does not state.

## Strength

The distance of a tie is the number of erasures on its path. Its trust
is the product of `1 - cost` over them, written on the evidence the
reader brings:

```
evidence tie { for bank::open::tied holds trust 0.93
               source "reader:rust src/lib.rs:4; erased 2: width, bound(l: Ord)" }
```

It is the trust of the tie as a piece: it ranks the tie among the
suspects of a contradiction and changes no verdict. No distance fails
a tie.

## A sample

`sampled n` that passed, with tolerance `eps`: if the claim failed on
more than `eps` of its domain, `n` draws would have missed it with
chance `(1 - eps)^n`. Its trust is `1 - (1 - eps)^n`: 0.63 at
`n = 1000`, `eps = 0.001`; it grows less and less with `n`.

A claim whose inputs without evidence are all inside a passing sample
reads `holds assuming [c::sample]`, with the trust of the sample in
the report. One setting turns this off.

## Settings

`.mitra/settings.toml`, beside the model, read by mitra-store and
handed to the core and to the reader. Absent, the defaults.

```toml
[sample]
tolerance = 0.001
counts = true

[erasure]
width = 0.02
field = 0.05
```

`mitra build` writes the settings in force, defaults included, to
`.mitra/build/settings.toml`.
