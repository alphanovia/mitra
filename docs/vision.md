# Vision

## One sentence

Mitra is a formally verified trust engine: given a model of claims, the
assumptions they rest on and the evidence brought for them, it computes, for each
claim, a verdict (pass, fail or uncertain) and a confidence that programs can
use as a policy input.

*Tests check, Lean proves, Mitra weighs.*

## What Mitra is first

A **math language for trust**: claims that read like functions over
mathematical values, guards that narrow them, and evidence that records what
was checked, the verdict and the source's confidence. The first thing to test
is whether this language is right: whether small, well-understood situations
can be written in it naturally and give the verdicts a careful person would
reach ([validation.md](validation.md)). The confidence calculus is still open;
subjective logic was the first candidate and is now only prior art. Everything else (CI
gates, agent harnesses, apps) is built on top later.

## Layers

```text
  text syntax   docstrings   Markdown blocks   (projections)
        \            |            /
         v           v           v
   (2) parser: text  ->  AST
                     |
                     v
   (1) semantic model / AST  --evaluate-->  verdict + confidence
                     |
                     v
   (3) incremental use and storage
```

1. **Semantic model.** The core. An AST of claims, evidence, sources and
   guards ([language.md](language.md)). Evaluation is a pure function from
   model to a verdict and confidence per claim.
2. **Parsing.** Text to AST. The text syntax is one projection of the model,
   not the model itself; others (JSON, an API) can exist alongside it.
3. **Incremental use and storage.** Add evidence to a stored model and
   re-evaluate only what changed.

They are built and tested in that order.

## Why a language

- **Embeddable.** A few lines fit in a docstring next to the code they talk
  about, or in a fenced `mitra` block in a skill or harness instructions.
- **LLM-friendly.** LLMs read and write small regular languages well, so an
  agent can state its own claims and evidence, and a harness can check them.
- **Reviewable.** A model is text that can be diffed, reviewed and versioned.

## Principles

1. **Never a single score.** Every claim gets a verdict (pass, fail,
   uncertain) and a confidence. "No evidence yet" and "evidence against" are
   different answers.
2. **Counterexamples decide.** Claims are universal: one trusted failure
   refutes a claim, while passes only build confidence.
3. **Precise semantics.** Every rule can be computed by hand and checked.
4. **Specified by example.** Toy models with hand-computed results
   ([examples.md](examples.md)) are the first tests.
5. **Proven.** The implementation is to be written with Lean 4 and Verus, so
   the trust engine can itself be trusted.

## Not now

- Extracting models from code, tests or agent traces.
- Integrations (CI, test frameworks, harnesses) and any user interface.
- Learning or calibrating confidences from data.

## Glossary

- **Subject:** what evidence observes from outside: a CLI, a function, an agent.
- **Claim:** a universal statement about a subject, like a function over
  mathematical values, optionally split into guarded branches.
- **Evidence:** what was checked (a claim on given inputs, or on all inputs),
  the verdict (pass or fail) and the source's confidence in it.
- **Assumption:** a claim accepted at a stated confidence, not computed. Every
  result lists the assumptions it rests on.
- **Verdict and confidence:** Mitra's output for a claim: pass, fail or
  uncertain, with how confident Mitra is in that verdict.
