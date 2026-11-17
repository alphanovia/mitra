# Mitra

**Mitra is a formally verified trust engine: given a model of claims, the
assumptions they rest on and the evidence brought for them, it computes a trust
value that programs can use as a policy input.**

*Tests check, Lean proves, Mitra weighs.* Tests and Lean proofs each produce
evidence. Mitra combines that evidence, along with the assumptions it rests on,
into one computed measure of how far a claim can be trusted.

> Status: design. Nothing is implemented yet.

## The idea

At its heart Mitra is a small **math language for trust**, in the family of
subjective logic, Bayesian reasoning and classical logic. You write claims,
evidence and how they combine; Mitra computes, for each claim, how much it is
believed, disbelieved, and still unknown.

```mitra
claim sort_ok   "sort returns a sorted permutation"
claim ordered   "output is ordered"
claim permuted  "output is a permutation of the input"

assume   ordered  = (0.9, 0, 0.1)      # Lean proof, modulo the spec
evidence permuted : +3                 # three passing example tests
sort_ok = ordered and permuted

query sort_ok                          # => (0.68, 0, 0.32), P = 0.76
```

The core is a **semantic model** (an AST). The text above is only one
projection of it, small enough to live in a code docstring or a Markdown block
in an agent skill, and regular enough for an LLM to read and write.

## Documentation

| Doc | What it covers |
|---|---|
| [docs/vision.md](docs/vision.md) | What Mitra is, how it is layered, principles, non-goals |
| [docs/language.md](docs/language.md) | One-page language spec: each construct and its subjective-logic meaning |
| [docs/examples.md](docs/examples.md) | Five toy models with scores worked out by hand |
| [docs/use-cases.md](docs/use-cases.md) | Where Mitra is headed: checking a model, gating a merge, gating an agent |
| [docs/prior-art.md](docs/prior-art.md) | Related work and how Mitra differs |
| [docs/scoping-method.md](docs/scoping-method.md) | INVEST rules used to scope work |

## License

MIT, see [LICENSE](LICENSE).
