# Getting started

You need Rust (`cargo`). Everything below runs from a clone of this
repository.

## 1. Build Mitra

```text
cargo build --release
export PATH="$PWD/target/release:$PATH"
mitra check examples/div/model.mitra
```

```text
check: well formed; agree: 9 comparisons, 9 hold, 0 findings
```

## 2. The example project

`examples/div` is a tiny Rust project with a model beside it:

```text
examples/div/
  model.mitra       the model: what div should do
  src/lib.rs        the code, tied to the model, and three tests
  evidence.mitra    evidence written by hand: a review, a measurement
  Cargo.toml
```

Open [model.mitra](../examples/div/model.mitra). It says that `div`
takes two integers and gives `q`, with a claim `spec` (what `q` is) and
four examples. Open [src/lib.rs](../examples/div/src/lib.rs): the line
`// mitra: implements calc::div` ties the Rust function to the model,
and each `// mitra: measures calc::div (6, 2) -> 3` says what a test
measures.

## 3. Check the model

`mitra check` reads the model alone. It runs each example against each
claim, and the claims against each other.

```text
cd examples/div
mitra check model.mitra
```

```text
check: well formed; agree: 9 comparisons, 9 hold, 0 findings
```

Try breaking it: change the example `(6, 2) -> 3` to `(6, 2) -> 4` and
check again. Mitra finds that the example breaks the claim `spec`
(finding A003). Put it back.

## 4. Explain the tests

Run the tests and keep what `cargo test` printed, then ask Mitra
whether the model explains it:

```text
cargo test > report.txt 2>&1
mitra explain model.mitra --code src --report report.txt
```

Each line is a verdict on one thing the model says:

```text
explained    correct(calc::div, (6, 2)) 0.67 (b 0.33, d 0.00, u 0.67: 1 for, 0 against, test:tests::div_by_two)
unsupported  calc::div::bounded 0.50 (b 0.00, d 0.00, u 1.00: 0 for, 0 against, 1 only the model says)
…
explain: 17 propositions, 9 explained, 0 contradicted, 8 unsupported; 0 findings, 0 hints
```

- **explained**: the evidence supports it. Here, one passing test says
  `div(6, 2)` is right.
- **unsupported**: only the model says it. No test or review speaks of
  the claim `bounded` yet. That is not an error, it is a place where
  trust is missing.
- **contradicted**: the evidence goes against it.

The number after the verdict is how much to trust it, from 0 to 1. One
test gives 0.67; more tests at the same point raise it. The numbers in
brackets are explained in [the reference](reference/1-ideas.md#trust-numbers).

## 5. Break the code, and blame

Change `a / b` to `a / b + 1` in `src/lib.rs`, run the tests again,
and ask Mitra:

```text
cargo test > report.txt 2>&1
mitra explain model.mitra --code src --report report.txt
mitra blame model.mitra --code src --report report.txt
```

Two tests fail. `explain` now says `contradicted` for the tie and the
claim `spec`. `blame` lists the ways out of the contradiction, most
doubtful first:

```text
diagnosis 1 (doubt 0.25): drop calc::div at src/lib.rs/div (only the model says it); calc::div::everywhere (only the model says it)
diagnosis 2 (doubt 0.11): drop measurements::0 (test:tests::div_by_two, did not see calc::div::q = 3); measurements::2 (test:tests::div_by_three, did not see calc::div::q = 3)
…
```

Diagnosis 1 reads: the code at `src/lib.rs/div` is probably not the
model's `div`. Diagnosis 2 is the other way out: the two tests are
wrong. Mitra ranks the tie first because only the model says it, while
each test is a run. Put the code back.

## 6. Evidence written by hand

Not all evidence is a Rust test. [evidence.mitra](../examples/div/evidence.mitra)
holds a review and a measurement someone took by hand, and took wrong:

```text
measure calc::div (7, 2) -> 4 by "human:ana";
review calc::div::spec holds by "human:bo";
```

```text
mitra blame model.mitra --code src --report report.txt --store evidence.mitra
```

Blame doubts Ana's measurement first.

## 7. Your first model

Make a folder, and a file `model.mitra` in it:

```mitra
subject temperature {
    fn to_fahrenheit(c: rational) -> f: rational { c * 9.0 / 5.0 + 32.0 } where {
        examples { (0.0) -> 32.0, (100.0) -> 212.0 }
    }
}
```

```text
mitra check model.mitra
```

From here, the [language reference](reference/README.md) goes from the
big ideas to every form. The loop is always the same: write the model,
check it, tie the code, explain the tests, blame what disagrees.
