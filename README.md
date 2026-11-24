# Mitra

**Model as code, trust by evidence.**

Mitra is a small language for saying what a system should do, and a
tool that tells you how far the evidence you have supports it.

You write a *model*: the parts of your system, the functions they
expose, and the claims you make about them. You tie the model to your
code. Then Mitra reads your tests and reviews as *evidence* and answers
three questions:

| Command | Question |
|---|---|
| `mitra check` | Is the model coherent on its own? |
| `mitra explain` | Does the model explain what the tests saw? |
| `mitra blame` | When it does not, where should you look? |

A proof is the strongest evidence Mitra knows, but it is never
required. A claim with one passing test is weakly supported; with a
hundred, strongly; with a proof, surely. Mitra says which, with a
number.

## The loop

Mitra is made for working in increments with a coding agent.

1. **Settle the spec** with your agent, as a model. `mitra check` tells
   you whether it is coherent: whether a function's examples break its
   own claims, whether two claims cannot both hold.
2. **Code the system and its tests** with your agent. `mitra explain`
   tells you whether the model explains the test results on the code.
3. **When something disagrees**, a bug, or a gap between the model and
   the code, `mitra blame` tells you where to look first.
4. **Settle the spec change** with your agent, and start the next
   increment.

## A taste

A division that gives 0 when dividing by 0. The model
([examples/div/model.mitra](examples/div/model.mitra)):

```mitra
subject calc {
    require inputs: "a and b are whole numbers from 0 to 9";

    fn div(a: integer, b: integer) -> q: integer where {
        claim spec: if b == 0 { q == 0 } else { q * b <= a && a < (q + 1) * b };
        claim bounded: b == 0 || q <= a;
        examples { (6, 2) -> 3, (7, 2) -> 3, (9, 3) -> 3, (5, 0) -> 0 }
        rule everywhere: div::spec |- all x in (a: 0..10, b: 0..10): correct(div, x);
    }

    ensure divides: "div gives a / b rounded down, and 0 when b is 0"
        from (div::spec, inputs);
}
```

The code says which function it implements, and each test says what it
measures ([examples/div/src/lib.rs](examples/div/src/lib.rs)):

```rust
// mitra: implements calc::div
pub fn div(a: u32, b: u32) -> u32 {
    if b == 0 { 0 } else { a / b }
}

// mitra: measures calc::div (6, 2) -> 3
#[test]
fn div_by_two() { assert_eq!(div(6, 2), 3); }
```

`mitra check` finds the model coherent: its examples keep its claims.

```text
check: well formed; agree: 9 comparisons, 9 hold, 0 findings
```

Now suppose the code is wrong, `a / b + 1`, and two tests fail.
`mitra explain` says the model no longer explains the system, and
`mitra blame` puts the tie between the model and `src/lib.rs` first:

```text
contradicted calc::div at src/lib.rs/div 0.22 (b 0.00, d 0.55, u 0.45: 0 for, 2 against, …)
diagnosis 1 (doubt 0.25): drop calc::div at src/lib.rs/div (only the model says it); …
```

Read it as: "the code at `src/lib.rs/div` is probably not the `div` of
the model". [Getting started](docs/getting-started.md) walks through
this example step by step.

## Where to go next

- [Getting started](docs/getting-started.md): build Mitra, run it on
  the example, break it, and write your first model.
- [Language reference](docs/reference/README.md): from the big ideas
  down to every form, then every error code.
- [Design notes](docs/design/): why each choice was made.

## Status

Mitra is early. What works today:

- the language of [the reference](docs/reference/README.md): parts,
  functions, claims, examples, rules and proofs, records, enums,
  sequences, text, loops, trees, contracts, and `use … from` across the
  project's folders;
- ties to Rust code, and evidence from `cargo test` and from a file of
  reviews and measurements written by hand;
- the three commands, with error messages in the style of `rustc`.

Not yet: other languages than Rust, colours and JSON output. Each lands in the
reference when it lands in the code.

A VS Code grammar is in [editors/vscode](editors/vscode).

## Build

```text
cargo build --release
target/release/mitra check examples/div/model.mitra
```

The workspace: `crates/mitra-lang` (the language; reads no file, knows
no code language), `crates/mitra-store` (the disk: the project, its
`mitra.toml`), `crates/mitra-rust` (Rust code and `cargo test`),
`crates/mitra-cli` (the `mitra` command), and `crates/mitra` with
`crates/mitra-derive` (the test macros, a dev-dependency of yours).

MIT licence.
