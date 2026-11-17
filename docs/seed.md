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

```json
{
  "tau": 0.5,
  "claims": [
    { "claim": "not_correct", "verdict": "fail", "p": 0.09, "f": 0.95, "exhaustive": false, "evidence": 2 }
  ]
}
```

Per claim: a verdict and two supports, `p` for pass and `f` for fail.

## Rules (placeholders for the calculus)

Per input, keep the most confident pass and the most confident fail.

1. **Fail support.** `f` = the highest fail confidence: one counterexample
   refutes a universal claim.
2. **Pass support.** Passes on distinct inputs combine as
   `p = 1 − ∏(1 − w·cᵢ)`, with `w` per evidence kind (0.1 for every kind for now).
3. **Exhaustive.** If every parameter is `bool` and every input has a pass
   with confidence ≥ τ, the claim was checked everywhere: `p` = max(p, the
   weakest of those passes), and `exhaustive` is true.
4. **Verdict.** pass if p ≥ τ and f < τ; fail if f ≥ τ and p < τ; conflict if
   both; uncertain otherwise. τ = 0.5.

These replace rules 1 to 5 of [language.md](language.md) draft 3 for now.

## Validation

`tests/golden/<claim>/<case>/` holds `claims.mitra`, `evidence.mitra` and
the `expected.json` Mitra must produce; `cargo test` checks every case. For
the `not` claim: no evidence, one pass, both inputs passing, a trusted and a
doubtful counterexample, a conflict, and evidence at the extremes (1.0 fully
believed, 0.0 fully distrusted) for pass and fail.

At 0.0, evidence counts for nothing: a distrusted counterexample leaves a
fully tested claim at pass. At 1.0, one fail decides (f = 1), while one pass
on one input is still only p = 0.1; both inputs at 1.0 give pass @ 1.

## Later

Guards, typed values, `observe`, `(*)` evidence, policies in the text. The
core is kept small and free of dependencies so it can be proven with Verus and
Lean 4, as planned; that is not done yet.
