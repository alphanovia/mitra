# Mitra

**Mitra is a formally verified trust engine: given a model of claims, the
assumptions they rest on and the evidence brought for them, it computes, for each
claim, a verdict (pass, fail or uncertain) and a confidence that programs can
use as a policy input.**

*Tests check, Lean proves, Mitra weighs.* Tests and Lean proofs each produce
evidence. Mitra combines that evidence, along with the assumptions it rests on,
into a verdict and a confidence for each claim.

> Status: design. Nothing is implemented yet.

## The idea

At its heart Mitra is a small **math language for trust**. Claims read like
functions over mathematical values, guards narrow them to the inputs where
they should hold, and evidence (tests, proofs, usage, reviews, agent reports)
records what was checked, the verdict, and how confident the source is.

```mitra
subject add "the CLI `add A B`"

claim add_correct(a, b):
  fits     when -2^63 <= a + b < 2^63 : add(a, b) = a + b
  overflow otherwise                  : add(a, b) = error

evidence proof(verus_add) : add_correct.fits(*)              = pass @ 0.95
evidence usage(user)      : add_correct.overflow(2^62, 2^62) = fail @ 0.95
```

```text
add_correct            fail @ 0.95
  add_correct.fits       pass @ 0.95     proven (verus_add)
  add_correct.overflow   fail @ 0.95     counterexample: (2^62, 2^62)
```

Every claim gets a verdict **and** a confidence, never a single score. One
trusted counterexample refutes a claim; passes build confidence gradually.
Mitra's value is in the diagnostics: where the graph is uncertain, where a
counterexample contradicts it, and which evidence to gather next.

The core is a **semantic model** (an AST). The text above is only one
projection of it, small enough to live in a code docstring or a Markdown block
in an agent skill, and regular enough for an LLM to read and write.

## Documentation

| Doc | What it covers |
|---|---|
| [docs/vision.md](docs/vision.md) | What Mitra is, how it is layered, principles, non-goals |
| [docs/language.md](docs/language.md) | One-page language spec: claims, guards, evidence, verdicts, diagnostics |
| [docs/examples.md](docs/examples.md) | Toy models on the `add` CLI with verdicts worked out by hand |
| [docs/validation.md](docs/validation.md) | How Mitra is validated: a ladder of toy `add` CLIs scored against ground truth |
| [docs/use-cases.md](docs/use-cases.md) | Where Mitra is headed: checking a model, gating a merge, gating an agent |
| [docs/prior-art.md](docs/prior-art.md) | Related work and how Mitra differs |
| [docs/research/](docs/research/README.md) | Deeper reading: assurance tools, confidence calculi, Mitra's wedge |
| [docs/scoping-method.md](docs/scoping-method.md) | INVEST rules used to scope work |

## License

MIT, see [LICENSE](LICENSE).
