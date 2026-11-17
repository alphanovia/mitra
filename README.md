# Mitra

**Mitra is a formally verified trust engine: given a model of claims, the
assumptions they rest on and the evidence brought for them, it computes a trust
value that programs can use as a policy input.**

*Tests check, Lean proves, Mitra weighs.* Tests and Lean proofs each produce
evidence. Mitra combines that evidence, along with the assumptions it rests on,
into one computed measure of how far a claim can be trusted.

> Status: design. Nothing is implemented yet. This repository currently holds
> the project's scope and plan.

## Why

Software is built on claims: "this function sorts", "the agent only edited
`src/`", "we tested the login flow". Each claim rests on assumptions and is
backed by evidence of very different strength, from a machine-checked proof to
a passing test to someone's word. Today that reasoning lives in people's heads.
Mitra makes it explicit and computable, so a program can ask "how far can I
trust this claim?" and act on the answer.

The long-term uses are judging whether a product (including vibe-coded ones)
really does what was intended, and giving an agent harness a model of whether
an agent is doing what it was asked. **Version 1 deliberately covers neither.**
It builds and proves the core engine, the way a trust calculus such as
subjective logic is validated: by modelling small, well-understood situations
and checking the output.

## What v1 is

- A small **model format** for claims, assumptions, evidence and how they
  support or undermine each other.
- A **calculus** that turns a model into a trust value, with an explanation of
  which element limits it.
- A **reference corpus** of small nominal and corner-case models with expected
  outputs. These are Mitra's tests and its specification by example.
- **Proofs**: the calculus's properties stated and proven in Lean 4, and the
  Rust implementation verified with Verus.
- A **tiny demo app** that loads a model and shows the score to a human.

See [docs/use-cases.md](docs/use-cases.md) for concrete examples,
[docs/vision.md](docs/vision.md) for goals and non-goals, and
[docs/roadmap.md](docs/roadmap.md) for the scoped, INVEST-checked plan.

## Documentation

| Doc | What it covers |
|---|---|
| [docs/vision.md](docs/vision.md) | Problem, users, goals, non-goals, glossary |
| [docs/use-cases.md](docs/use-cases.md) | Three concrete use cases: checking a model (v1), gating a merge, gating an agent |
| [docs/roadmap.md](docs/roadmap.md) | Milestones and v1 scope items, each INVEST-checked |
| [docs/prior-art.md](docs/prior-art.md) | Related work and how Mitra differs |
| [docs/scoping-method.md](docs/scoping-method.md) | INVEST rules, sizing, definition of done |
| [docs/decisions/](docs/decisions/) | Decision records produced by spikes |

## License

MIT, see [LICENSE](LICENSE).
