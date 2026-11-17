# The seed

The first code: an embeddable library, `mitra-core`, and a thin CLI, `mitra`,
that reads a claim model and an evidence list and writes Mitra's result.

```sh
cargo run -p mitra-cli -- eval claims.mitra evidence.mitra -o result.json
```

## Inputs

One claim or one piece of evidence per line. `#` starts a comment.

```mitra
claim not_correct(a: bool): not(a) = !a
```

```mitra
evidence test(t_true)          : not_correct(true)  = pass @ 0.9
evidence usage(cli_session_43) : not_correct(false) = fail @ 0.95
```

`not` is the subject under trust; `!a` is mathematics. Parameters are untyped,
except `: bool`, which tells Mitra the domain is finite. The claim body is
parsed into the AST but not evaluated yet: evidence states pass or fail.

## Output

For each claim: a verdict, two supports (`p` for pass, `f` for fail), its
coverage, and every instance (the claim on concrete inputs) with its own
verdict and the evidence it rests on. For a `bool` domain all instances are
listed, so unchecked ones show up.

```json
{
  "policy": { "tau": 0.5, "sample_weight": 0.1 },
  "claims": [
    {
      "claim": "not_correct(a: bool)",
      "verdict": "uncertain", "p": 0, "f": 0, "coverage": "1/2",
      "instances": [
        { "instance": "not_correct(false)", "verdict": "uncertain", "p": 0, "f": 0,
          "evidence": [] },
        { "instance": "not_correct(true)", "verdict": "pass", "p": 1, "f": 0,
          "evidence": ["test(t_true) pass @ 1"] }
      ]
    }
  ]
}
```

## Rules (placeholders for the calculus)

1. **Instance.** p = the most confident pass on those inputs, f = the most
   confident fail.
2. **Verdict** (instance or claim): pass if p ≥ τ and f < τ; fail if f ≥ τ
   and p < τ; conflict if both; uncertain otherwise. τ = 0.5.
3. **Claim, fail support.** f = the highest instance f: one counterexample
   refutes a universal claim.
4. **Claim, pass support, finite domain** (every parameter `bool`). The claim
   is the conjunction of its instances: p = the lowest instance p, and an
   unchecked instance has p = 0. No weights.
5. **Claim, pass support, unbounded domain.** Instances are only samples:
   p = 1 − ∏(1 − w·pᵢ), with w = `sample_weight` (0.1), a stated policy.
6. **Coverage.** Instances decided (pass or fail) over the domain size, or
   over `unbounded`.

These replace rules 1 to 5 of [language.md](language.md) draft 3 for now.

## Validation

`tests/golden/<claim>/<case>/` holds `claims.mitra`, `evidence.mitra` and
the `expected.json` Mitra must produce; `cargo test` checks every case. For
the `not` claim: no evidence, one pass, both inputs passing, a trusted and a
doubtful counterexample, a conflict, and evidence at the extremes (1.0 fully
believed, 0.0 fully distrusted) for pass and fail. `id/` has one case on
an unbounded domain, to pin down the sampling rule.

At 0.0, evidence counts for nothing: a distrusted counterexample leaves a
fully tested claim at pass. At 1.0, one fail decides the claim (f = 1), and
one pass decides its instance (pass @ 1); the claim passes once every
instance does.

## Later

Guards, typed values, `observe`, `(*)` evidence, policies in the text. The
core is kept small and free of dependencies so it can be proven with Verus and
Lean 4, as planned; that is not done yet.
