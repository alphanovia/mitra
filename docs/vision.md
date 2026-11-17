# Vision

## One sentence

Mitra is a formally verified trust engine: given a model of claims, the
assumptions they rest on and the evidence brought for them, it computes a trust
value that programs can use as a policy input.

## Positioning

*Tests check, Lean proves, Mitra weighs.* Tests and Lean proofs each produce
evidence. Mitra combines that evidence, along with the assumptions it rests on,
into one computed measure of how far a claim can be trusted.

## Problem

When we build software, or let an agent build it, we accumulate claims about
what it does and evidence that those claims hold. That evidence varies a lot in
strength, it depends on assumptions nobody wrote down, and some of it is
missing. There is no formal, computable way to say how much a claim is earned,
or which piece of evidence would raise that the most.

## Primary user

**A program making a policy decision.** For example a CI gate, a release
script or, later, an agent harness that needs a number and a reason to decide
whether to proceed. Humans see Mitra's output through such programs; v1 ships
one tiny demo app for that purpose only.

Concrete scenarios are in [use-cases.md](use-cases.md).

## Principles

1. **The engine is the product.** Integrations come after the core is proven.
2. **Uncertainty is not distrust.** "No evidence yet" and "evidence against"
   must give different outputs, so a policy can tell them apart.
3. **Explainable.** Every trust value comes with the element that limits it.
4. **Proven, not just tested.** The calculus's properties are proven in Lean 4
   and the implementation is verified with Verus. A trust engine that cannot be
   trusted defeats its own purpose.
5. **Specified by example.** A corpus of small models with expected outputs is
   the executable definition of correct behaviour.

## Goals for v1

- Model claims, assumptions and evidence, and the support between them.
- Compute a trust value for any claim in a model, deterministically.
- Explain which element limits a claim's trust value.
- Pass a reference corpus of nominal and corner-case models.
- Prove the calculus's key properties in Lean 4 and verify the implementation
  in Verus.
- Show a score to a human through a minimal demo app.

## Non-goals for v1

- The **product use case**: extracting claims from code, specs or test suites.
- The **agent use case**: a harness that watches or gates an agent.
- Integrations with CI, test frameworks, proof assistants or attestation
  formats (SLSA, in-toto).
- A polished user interface. The demo app exists to show the core, nothing more.
- Learning or calibrating evidence weights from data.
- Distributed or multi-party trust (who trusts whom across organisations).

## After v1 (not planned in detail)

- **v2, product case:** derive models from a codebase's specs, tests and proofs.
- **v3, agent case:** a harness that builds a model from an agent's actions and
  gates it on Mitra's output.

## Glossary

- **Claim:** a statement that may be true or false, such as "`sort` returns a
  sorted permutation of its input".
- **Assumption:** a claim accepted without evidence in this model, such as
  "the compiler is correct". Its trust is given, not computed.
- **Evidence:** an observation brought for or against a claim, such as a test
  run, a proof, a review. Each has a kind and a strength.
- **Support link:** how claims and evidence combine: a claim may need all of
  its sub-claims (conjunction), any of them (alternatives), or be backed by
  several independent pieces of evidence (fusion).
- **Defeater:** evidence or a claim that undermines another claim.
- **Trust value:** Mitra's output for a claim. The exact shape is decided in
  spike SP1 (see the roadmap); the working hypothesis is a subjective-logic
  opinion (belief, disbelief, uncertainty) plus a single projected probability
  for policies that want one number.
- **Model:** a set of claims, assumptions, evidence and links, the input to
  Mitra.
