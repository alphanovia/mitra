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

A **language for trust**: subjects (the real code under review),
boundaries (the environment), claims on them, assumptions such as "one
tested input stands for this set", and evidence from tests, reviews and
reports, with the trust its author puts in it (see the
[language reference](reference/language.md) for what exists today). The first thing to test
is whether this language is right: whether small, well-understood situations
can be written in it naturally and give the verdicts a careful person would
reach ([validation.md](validation.md), [trials](trials.md)). Everything
else (CI gates, agent harnesses, apps) is built on top later.

**No number on the model.** A claim holds, fails, or can't be decided,
possibly "holds assuming" a dependency nobody gave evidence for. Numbers are kept for
the evidence: when something does not add up, Mitra says which piece is
most plausibly wrong ([semantics](design/semantics.md)).

## Who it is for

Modelling used to be double work: write the model, then the code. With
agents doing the writing it is no longer a human cost, and the model
becomes the thing a person reads.

- **The agent models and codes.** A person gives the intent and some
  requirements. An agent writes the model and the code, a second agent
  diagnoses with Mitra, and the person reads the model and the report.
  Mitra is there to catch intent that drifted and sloppy work: a
  requirement with no claim, a claim with no evidence, a test weakened
  until it passes.
- **The person writes the model as a scaffold** of the project. Mitra
  checks that the code produced is tied to it and covered. Human effort
  goes to the architecture.

The agent learns while it codes, so it must be able to say the model is
wrong: it does so with evidence, and Mitra shows the conflict with the
model's own statement as a candidate.

**The model lives aside from the code**, as in model-based design. It is
checked alone; the code is checked against it; when the model changes the
code may be rewritten. The code keeps one comment per function and per
test that names what it implements or supports.

**Everything is a suspect.** A claim, an assumption, a signature, a
`uses` clause, a piece of evidence and the code can each be the piece
that is wrong, and each is a candidate in an error.

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
The first three exist: `mitra tie` compares the signatures and calls of
the Rust code with the model, `mitra ingest` turns test reports into
evidence for the claims the tests name, and `mitra plan` lists the
evidence a model owes before any code exists. The core stays agnostic:
it works with no reader, and Rust is the only language invested in for
now.

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
  the language   (a projection)
         |
         v
   (2) parser: text  ->  AST
                     |
                     v
   (1) semantic model / AST  --evaluate-->  verdicts, errors, warnings
                     |
                     v
   (3) incremental use and storage
```

1. **Semantic model.** The core. An AST of subjects, boundaries, claims
   and evidence.
   Evaluation is a pure function from the model and the ledger to a
   verdict per claim.
2. **Parsing.** Text to AST. The text syntax is one projection of the model,
   not the model itself. One is read: [the language](reference/language.md).
3. **Incremental use and storage.** Add evidence to a stored model and
   re-evaluate. A first store exists (`.mitra/`); re-evaluating only what
   changed does not.

They are built and tested in that order.

## Why a language

- **Readable alone.** A model is a few files a person can review without
  the code, and small enough for a fenced block in a skill or a harness.
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
   examples under `examples/reworked/` and the engine tests under `tests/golden/`) are the
   tests.
5. **Proven.** The implementation is to be written with Lean 4 and Verus, so
   the analyser can itself be trusted.

## Specs and intent

A body on a function of a subject says what the code should return, and
adds the claim that it does. Evidence can refute that claim; the code,
the claim and the evidence are then the candidates. A claim that the
bodies themselves contradict is an error with no evidence needed. Whether
a body says what a person intends is a validation question: it is
settled by reading the model, which is why the model must read alone.

## Not now

- Readers for other languages than Rust, and agent traces as evidence.
- Simulations, sequences of calls, distributions on what a boundary
  answers; uniformity proven from the branches of a body.
- Integrations beyond reading test reports (CI plugins, agent harnesses),
  and any user interface.
- Learning the trust in a source from data.

## Glossary

- **Subject:** real code under review, and a namespace for its functions
  and claims. Mitra never runs it.
- **Boundary:** the environment (network, clock, a person). It may answer
  differently each time and is never the piece to fix.
- **Spec:** the body of a function in the model: what the code should
  return. Mitra computes it.
- **Claim:** a property of a subject, true on every input it names. A
  point claim pins one call (`div(6, 2) == 3`).
- **Assumption:** a claim taken as true without evidence. What rests on
  it `holds assuming` it.
- **Evidence:** what was observed or judged, on a function or a claim,
  with the trust its author puts in it, from 0 to 1.
- **Ledger:** the append-only record of all evidence.
- **Tie:** a comment in the code naming the function of the model it
  implements, or the claim a test is evidence for.
- **Representation claim:** the assumption that one tested input stands
  for a set (`represented`) or for its class (`uniform`). It counts once
  someone vouches for it.
- **Conflict:** claims and evidence that can't all be true. Reported like
  a compiler error.
- **Verdict:** holds, holds assuming, unverified or fails, with the
  inputs tested, assumed and missed.
- **Opinion:** on an error, how plausibly each piece is the wrong one.
