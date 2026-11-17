# Vision

## One sentence

Mitra is a formally verified trust engine: given a model of claims, the
assumptions they rest on and the evidence brought for them, it computes a trust
value that programs can use as a policy input.

*Tests check, Lean proves, Mitra weighs.*

## What Mitra is first

A **math language for trust**, in the same family as subjective logic,
Bayesian reasoning or propositional logic. The first thing to test is whether
this language is right: whether small, well-understood situations can be
written in it naturally and give the expected numbers. Everything else (CI
gates, agent harnesses, apps) is built on top later.

## Layers

```text
  text syntax   docstrings   Markdown blocks   (projections)
        \            |            /
         v           v           v
   (2) parser: text  ->  AST
                     |
                     v
   (1) semantic model / AST  --evaluate-->  opinion per claim
                     |
                     v
   (3) incremental use and storage
```

1. **Semantic model.** The core. An AST of claims, evidence, sources and
   operators, with a precise meaning given in subjective logic
   ([language.md](language.md)). Evaluation is a pure function from model to
   opinions.
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

1. **Uncertainty is not distrust.** "No evidence yet" and "evidence against"
   give different results. This is why the value is a subjective-logic opinion
   (belief, disbelief, uncertainty), not a single probability.
2. **Precise semantics.** Every construct maps to a defined subjective-logic
   operator, so results can be computed by hand and checked.
3. **Specified by example.** Toy models with hand-computed results
   ([examples.md](examples.md)) are the first tests.
4. **Proven.** The implementation is to be written with Lean 4 and Verus, so
   the trust engine can itself be trusted.

## Not now

- Extracting models from code, tests or agent traces.
- Integrations (CI, test frameworks, harnesses) and any user interface.
- Learning or calibrating weights from data.

## Glossary

- **Claim:** a proposition that may be true or false.
- **Evidence:** observations for (`+r`) or against (`-s`) a claim.
- **Source:** who or what produced evidence, with its own reliability.
- **Assumption:** a claim whose opinion is given, not computed.
- **Opinion:** Mitra's value for a claim: belief *b*, disbelief *d*,
  uncertainty *u* with *b + d + u = 1*, plus a base rate *a*. Its projected
  probability *P = b + a·u* is the single number a policy can use.
