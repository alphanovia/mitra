# Vision

## One sentence

Mitra is a static analyser for trust: you write down what your software
claims, what you assume and what evidence you have, and it tells you where
they contradict each other and how much of each claim is tested, assumed
or unsupported.

It is quality engineering made checkable. Like a compiler, Mitra does not
run the code: it reads the claims, assumptions and evidence, and reports
what can't all be true as an error, with the pieces in conflict. Its
verdicts are meant as inputs for programs (a merge gate, an agent
harness), and the analyser itself is to be formally verified.

*Tests check, Lean proves, Mitra weighs.*

## Core hypothesis

Most software can't be proven correct, and won't be. What a project needs is
**good-enough evidence management**: knowing what the tests, reviews and
usage actually establish, and where that is not much. A proof is just the
strongest evidence, over its whole domain. Mitra's value is on the
unproven code.

Earlier attempts to measure confidence in software failed because they
treated code as a black box, needed inputs nobody had, and gave numbers
nobody could act on ([semantics](design/semantics.md#why-not-a-confidence-in-the-claim)).
Mitra writes down the hypotheses developers already rely on, such as "the
code behaves the same on all these inputs, so one passing test stands for
them". That is why tests are cheaper than proofs and usually accepted: the
assumption is checked by reading the code. Written down, it lets Mitra say
what is tested, what is only assumed, what is missing, and, on a bug,
which assumption broke.

## What Mitra is first

A **language for trust**: subjects under review, claims that they are
correct on some of their inputs, assumptions such as uniformity, and
evidence that records runs judged correct or wrong, with the trust its
author puts in it (see the [language reference](reference/language.md)
for what exists today). The first thing to test
is whether this language is right: whether small, well-understood situations
can be written in it naturally and give the verdicts a careful person would
reach ([validation.md](validation.md), [trials](trials.md)). Everything
else (CI gates, agent harnesses, apps) is built on top later.

**No number on the model.** A claim holds, fails, or can't be decided,
possibly "holds assuming" a dependency nobody gave evidence for. Numbers are kept for
the evidence: when something does not add up, Mitra says which piece is
most plausibly wrong ([semantics](design/semantics.md)).

## The core, and what can be automated

The core of Mitra is the **pure model** and the **ledger of evidence**.
Everything is explicit: every claim and assumption is written in the
model, every piece of evidence is a line in the ledger, and a review is
evidence given by a human or an agent. Mitra infers nothing from the code.

On top of that core, language backends and other means can later make
part of the writing and the checking automatic: proposing claims from the
code, checking a `uses` claim against the call graph, turning test runs
into evidence. Each of them only writes model lines or ledger entries
that could have been written by hand, so the core never depends on them.
Reading `mitra:` comments from Rust sources and ingesting test reports
are the first two.

## What success looks like

Mitra succeeds if it makes software more reliable for less effort:

- **Faster root cause.** After a bug report, the faulty part is near the
  top of Mitra's diagnosis.
- **Fewer tests for the same confidence.** Mitra points at the evidence that
  matters, so fewer tests leave nothing missed.
- **Fewer escaped defects.** A claim that holds rarely turns out wrong,
  and when one does, the assumption that broke is named.

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
   (1) semantic model / AST  --evaluate-->  verdicts, errors, warnings
                     |
                     v
   (3) incremental use and storage
```

1. **Semantic model.** The core. An AST of subjects, specs, claims and
   evidence.
   Evaluation is a pure function from the model and the ledger to a
   verdict per claim.
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

1. **Never a score.** Every claim gets a verdict (holds, holds assuming,
   unverified, fails) and the inputs tested, assumed and missed. "No
   evidence yet" and "evidence against" are different answers.
2. **Counterexamples decide.** Claims are universal: one failure refutes
   a claim, while passes only cover inputs.
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

- Language backends that extract or check parts of a model from code,
  tests or agent traces (see "The core, and what can be automated").
- Integrations beyond reading test reports (CI plugins, agent harnesses),
  and any user interface.
- Learning the trust in a source from data.

## Glossary

- **Subject:** a function, task or process under review. Mitra never runs it.
- **Spec:** a definition Mitra can evaluate; true by definition.
- **Claim:** a subject is correct on a set of its inputs, or matches a spec
  there. A claim can be derived from other claims.
- **Evidence:** a run of the subject on given inputs, judged correct or
  wrong, with the trust its author puts in it, from 0 to 1 (its bias).
- **Ledger:** the append-only record of all evidence.
- **Test:** a test of the code, declared with the evidence its result
  stands for; `mitra ingest` turns its results into evidence.
- **Uniformity claim:** the assumption that a subject behaves the same on
  every input of a class, so that one judged input stands for the class.
  It counts once evidence, such as a review, vouches for it.
- **Conflict:** claims and evidence that can't all be true, such as a
  pass and a failure in one class of a uniformity claim. Reported like a
  compiler error.
- **Verdict:** Mitra's output for a claim: holds, holds assuming,
  unverified or fails, with the inputs tested, assumed and missed.
- **Opinion:** on an error, the belief and plausibility that each piece
  of evidence or assumption is the wrong one.
