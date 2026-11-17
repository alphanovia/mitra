# Vision

## One sentence

Mitra is a formally verified trust engine: given a model of claims, the
assumptions they rest on and the evidence brought for them, it computes, for each
claim, a verdict (logic, coverage and belief) that programs can
use as a policy input.

*Tests check, Lean proves, Mitra weighs.*

## What Mitra is first

A **language for trust**: subjects under review, claims that they are
correct on some of their inputs, and evidence that records runs judged
correct or wrong with a confidence (see the
[language reference](reference/language.md) for what exists today). The first thing to test
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
   (1) semantic model / AST  --evaluate-->  verdict (logic, coverage, belief)
                     |
                     v
   (3) incremental use and storage
```

1. **Semantic model.** The core. An AST of subjects, specs, claims and
   evidence.
   Evaluation is a pure function from model to a verdict per claim and
   policy.
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

1. **Never a single score.** Every claim gets its logic (holds, fails,
   uncertain), the inputs covered, and a belief. "No evidence yet" and "evidence against" are
   different answers.
2. **Counterexamples decide.** Claims are universal: one trusted failure
   refutes a claim, while passes only build confidence.
3. **Precise semantics.** Every rule can be computed by hand and checked.
4. **Specified by example.** Small models with hand-written results (the
   golden set in the [language reference](reference/language.md)) are the
   tests.
5. **Proven.** The implementation is to be written with Lean 4 and Verus, so
   the trust engine can itself be trusted.

## Specs and intent

A spec is a definition, so evidence cannot disprove it: Mitra flags such
evidence as contested. If a person really means "the spec is wrong", that is
a validation question, not a verification one: the spec does not match what
they intend. The way to say it is a claim that treats the spec itself as
the thing under review, with its own evidence (for example
`claim neg_intent: neg(_) correct` once specs can be judged that way).

## Not now

- Extracting models from code, tests or agent traces.
- Integrations (CI, test frameworks, harnesses) and any user interface.
- Learning or calibrating confidences from data.

## Glossary

- **Subject:** a function, task or process under review. Mitra never runs it.
- **Spec:** a definition Mitra can evaluate; true by definition.
- **Claim:** a subject is correct on a set of its inputs, or matches a spec
  there. A claim can be derived from other claims.
- **Evidence:** a run of the subject on given inputs, judged correct or
  wrong, with a confidence from 0 (distrusted) to 1 (fully believed).
- **Assumption** (planned): a claim accepted at a stated confidence, not
  computed.
- **Policy:** how evidence becomes belief, chosen by configuration so
  policies can be compared (`strict`, `coverage` of a usage profile).
- **Verdict:** Mitra's output for a claim under a policy: the logic (holds,
  fails or uncertain), the inputs covered or refuted, and a belief.
