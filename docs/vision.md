# Vision

## One sentence

Mitra is a static analyser for trust: you write down what your software
claims, what you assume and what evidence you have, and it tells you where
they contradict each other and how much of each claim is tested, assumed
or unsupported.

It is quality engineering made checkable. Like a compiler, Mitra does not
run the code: it reads the claims, assumptions and evidence, and reports
what can't all be true as an error, with the pieces in conflict. Its
verdicts (logic, coverage and belief) are meant as policy inputs for
programs, and the analyser itself is to be formally verified.

*Tests check, Lean proves, Mitra weighs.*

## Core hypothesis

Most software can't be proven correct, and won't be. What a project needs is
**good-enough evidence management**: knowing what the tests, reviews and
usage actually establish, and where that is not much. A proof is just the
strongest evidence, at confidence 1 over its whole domain. Mitra's value is
on the unproven code.

Earlier attempts to measure confidence in software failed because they
treated code as a black box, needed inputs nobody had, and gave numbers
nobody could act on ([calculus](design/calculus.md#6-why-earlier-confidence-measures-failed)).
Mitra writes down the hypotheses developers already rely on, such as "the
code behaves the same on all these inputs, so one passing test stands for
them". That is why tests are cheaper than proofs and usually accepted: the
assumption is checked by reading the code. Written down, it lets Mitra say
what is tested, what is only assumed, what is missing, and, on a bug,
which assumption broke.

## What Mitra is first

A **language for trust**: subjects under review, claims that they are
correct on some of their inputs, assumptions such as uniformity, and
evidence that records runs judged correct or wrong with a confidence (see the
[language reference](reference/language.md) for what exists today). The first thing to test
is whether this language is right: whether small, well-understood situations
can be written in it naturally and give the verdicts a careful person would
reach ([validation.md](validation.md), [trials](trials.md)). The
confidence calculus is specified by axioms before it is chosen
([calculus](design/calculus.md)). Everything else (CI gates, agent
harnesses, apps) is built on top later.

## What success looks like

Mitra succeeds if it makes software more reliable for less effort:

- **Faster root cause.** After a bug report, the faulty part is near the
  top of Mitra's diagnosis.
- **Fewer tests for the same confidence.** Mitra points at the evidence that
  matters, so fewer tests reach a good coverage.
- **Fewer escaped defects.** A claim Mitra believes strongly rarely turns
  out wrong, and its belief can be trusted as a number.

How each one is measured is in [validation.md](validation.md#success-criteria).

**Kill criterion.** On unproven code, Mitra's verdicts must beat the
developer's own guess about what is reliable. If they don't, or if
dogfooding has not changed one real decision by the time `mitra-ds` is
annotated ([design/dogfooding.md](design/dogfooding.md)), rethink the
approach.

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
   re-evaluate. A first store exists (`.mitra/`); re-evaluating only what
   changed does not.

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
   the analyser can itself be trusted.

## Specs and intent

A spec is a definition, so evidence cannot disprove it: Mitra flags such
evidence as contested. If a person really means "the spec is wrong", that is
a validation question, not a verification one: the spec does not match what
they intend. The way to say it is a claim that treats the spec itself as
the thing under review, with its own evidence (for example
`claim neg_intent : forall a, correct (neg a)`, which is an error today because
`neg` is a spec).

## Not now

- Extracting models from code, tests or agent traces.
- Integrations beyond reading test reports (CI plugins, agent harnesses),
  and any user interface.
- Learning or calibrating confidences from data.

## Glossary

- **Subject:** a function, task or process under review. Mitra never runs it.
- **Spec:** a definition Mitra can evaluate; true by definition.
- **Claim:** a subject is correct on a set of its inputs, or matches a spec
  there. A claim can be derived from other claims.
- **Evidence:** a run of the subject on given inputs, judged correct or
  wrong, with a confidence from 0 (distrusted) to 1 (fully believed): the
  probability that the observation is valid.
- **Test:** a test of the code, declared with the evidence its result
  stands for; `mitra ingest` turns its results into evidence.
- **Uniformity claim:** the assumption that a subject behaves the same on
  every input of a class, so that one judged input stands for the class.
  It counts once evidence, such as a review, vouches for it.
- **Conflict:** claims and evidence that can't all be true, such as a
  pass and a failure in one class of a uniformity claim. Reported like a
  compiler error.
- **Policy:** how evidence becomes belief, chosen by configuration so
  policies can be compared (`strict`, `coverage` of a usage profile).
- **Verdict:** Mitra's output for a claim under a policy: the logic (holds,
  fails or uncertain), the inputs covered or refuted, and a belief.
