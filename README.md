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
subjective logic, Bayesian reasoning and classical logic. You write binary
claims, the evidence for them (tests, proofs, usage, reviews, agent reports),
the assumptions they rest on, and how they relate; Mitra computes, for each
claim, how much it is believed, disbelieved, and still unknown.

```mitra
claim f_ok     "f is correct"
claim x_ok     "test X is correct"
claim x_covers "test X exercises f's specified behaviour"
claim x_good   "test X is a reliable witness for f"

assume x_ok     = (0.8, 0, 0.2)
assume x_covers = (0.6, 0, 0.4)
x_good = x_ok and x_covers

evidence f_ok : test +5 via x_good
query f_ok      # => (0.514, 0, 0.486), P = 0.757, weakest: x_covers
```

"Test X verifies `f`" is not an edge with a weight: the test being right and
the test covering `f` are claims too, each with its own score. Mitra makes this
graph explicit, puts a number on every claim, and points at the weakest link.

The core is a **semantic model** (an AST). The text above is only one
projection of it, small enough to live in a code docstring or a Markdown block
in an agent skill, and regular enough for an LLM to read and write.

## Documentation

| Doc | What it covers |
|---|---|
| [docs/vision.md](docs/vision.md) | What Mitra is, how it is layered, principles, non-goals |
| [docs/language.md](docs/language.md) | One-page language spec: each construct and its subjective-logic meaning |
| [docs/examples.md](docs/examples.md) | Five toy models with scores worked out by hand |
| [docs/validation.md](docs/validation.md) | How Mitra is validated: a ladder of toy `add` CLIs scored against ground truth |
| [docs/use-cases.md](docs/use-cases.md) | Where Mitra is headed: checking a model, gating a merge, gating an agent |
| [docs/prior-art.md](docs/prior-art.md) | Related work and how Mitra differs |
| [docs/scoping-method.md](docs/scoping-method.md) | INVEST rules used to scope work |

## License

MIT, see [LICENSE](LICENSE).
