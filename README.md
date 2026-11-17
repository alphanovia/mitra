# Mitra

**Mitra is a formally verified trust engine: given a model of claims, the
assumptions they rest on and the evidence brought for them, it computes, for each
claim, a verdict (pass, fail or uncertain) and a confidence that programs can
use as a policy input.**

*Tests check, Lean proves, Mitra weighs.* Tests and Lean proofs each produce
evidence. Mitra combines that evidence, along with the assumptions it rests on,
into a verdict and a confidence for each claim.

> Status: early. The code evaluates claims, specs and derivations against
> evidence at confidence 0 or 1, under two policies ([language reference](docs/reference/language.md)).

## The idea

At its heart Mitra is a small **language for trust**. A **subject** is a
function under review, a **spec** is a definition Mitra can evaluate, a
**claim** says a subject is correct (or matches a spec) on some of its inputs,
or follows from other claims,
and **evidence** is a run of the subject judged correct or wrong, with a
confidence. A **policy** turns evidence into a belief. Mitra gives every
claim a verdict: what logic proves, which inputs are covered, and how much
the policy believes it.

```mitra
subject  not(a: bool) -> bool
claim    not_correct: not(_)    correct
claim    not_on_true: not(true) correct

policy   strict

evidence not(true)  correct @ 1
evidence not(false) wrong   @ 1
```

```text
verdict not_correct [strict] = { logic: fails, refuted: {false}, domain: bool, belief: 0 }
verdict not_on_true [strict] = { logic: holds, covered: {true}, domain: {true}, belief: 1 }
```

Never a single score: logic, coverage and belief are kept apart. One trusted
counterexample refutes a claim; on an infinite domain, tests alone never
prove it. The calculus for confidences between 0 and 1 is still open.

The core is a **semantic model** (an AST). The text above is only one
projection of it, small enough to live in a code docstring or a Markdown block
in an agent skill, and regular enough for an LLM to read and write.

## Documentation

Two sets, kept apart:

**Language reference**: what the code does today, updated with every change
to it.

| Doc | What it covers |
|---|---|
| [docs/reference/language.md](docs/reference/language.md) | Grammar, semantics, output, the CLI and the golden set |

**General docs**: purpose, positioning and research. These describe
direction, not the current language.

| Doc | What it covers |
|---|---|
| [docs/vision.md](docs/vision.md) | What Mitra is, how it is layered, principles, non-goals |
| [docs/use-cases.md](docs/use-cases.md) | Where Mitra is headed: checking a model, gating a merge, gating an agent |
| [docs/validation.md](docs/validation.md) | How Mitra will be validated: a ladder of toy `add` CLIs scored against ground truth |
| [docs/prior-art.md](docs/prior-art.md) | Related work and how Mitra differs |
| [docs/research/](docs/research/README.md) | Deeper reading: assurance tools, confidence calculi and the open calculus candidates, Mitra's wedge |
| [docs/scoping-method.md](docs/scoping-method.md) | INVEST rules used to scope work |

## License

MIT, see [LICENSE](LICENSE).
