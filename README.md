# Mitra

**Mitra is a trust engine, to be formally verified: given a model of
claims and the evidence brought for them, it computes, for each claim, a
verdict (holds, fails or uncertain, the inputs covered, and a belief) that
programs can use as a policy input.**

*Tests check, Lean proves, Mitra weighs.* Tests and Lean proofs each produce
evidence. Mitra combines that evidence into a verdict for each claim.

> Status: early. The code evaluates claims, specs and derivations against
> evidence at confidence 0 or 1, under two policies, and keeps a project's
> evidence in a store fed by test reports
> ([language reference](docs/reference/language.md)).

## The idea

At its heart Mitra is a small **language for trust**. A **subject** is a
function under review, a **spec** is a definition Mitra can evaluate, a
**claim** says a subject is correct (or matches a spec) on some of its inputs,
or follows from other claims,
and **evidence** is a run of the subject judged correct or wrong, with a
confidence. A **policy** turns evidence into a belief. Mitra gives every
claim a verdict: what logic proves, which inputs are covered, and how much
the policy believes it.

```lean
opaque not (a : Bool) : Bool
claim not_correct : ∀ a, correct (not a)
claim not_on_true : correct (not true)

policy strict

evidence correct (not true) at 1
evidence ¬ correct (not false) at 1
```

```lean
not_correct [strict] := { logic := .fails, refuted := {false}, domain := Bool, belief := 0 }
not_on_true [strict] := { logic := .holds, covered := {true}, domain := {true}, belief := 1 }
```

Never a single score: logic, coverage and belief are kept apart. One trusted
counterexample refutes a claim; on an infinite domain, tests alone never
prove it. Confidences between 0 and 1 are specified by axioms first
([calculus](docs/design/calculus.md)); the maths that meets them is not
chosen yet.

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
| [docs/design/calculus.md](docs/design/calculus.md) | Spec: the axioms a confidence calculus must meet, the operators, candidates scored, why earlier confidence measures failed |
| [docs/design/composition.md](docs/design/composition.md) | Proposal: composition as implication, structure claims, tests as subjects |
| [docs/design/dogfooding.md](docs/design/dogfooding.md) | Plan: Mitra on Mitra: the calculus crate judged from its own tests, and Mitra's semantics written in Mitra |
| [docs/design/requirements.md](docs/design/requirements.md) | Proposal: prose requirements as opaque propositions derived from claims, for traceability |
| [docs/design/elaboration.md](docs/design/elaboration.md) | Proposal: Mitra elaborates partial knowledge into claims, splits, specs and proof obligations |
| [docs/review/2026-11-18.md](docs/review/2026-11-18.md) | Review list from the overnight prototype run: decisions, open questions, how to test |
| [docs/design/lean-syntax.md](docs/design/lean-syntax.md) | Decision: a standalone language with a fixed LL(1) syntax inspired by Lean 4 (implemented) |
| [docs/design/workflow.md](docs/design/workflow.md) | The developer loop: what exists (store, ingest, next) and what is proposed (bugs, diagnosis, usage) |
| [docs/use-cases.md](docs/use-cases.md) | Where Mitra is headed: checking a model, gating a merge, gating an agent |
| [docs/validation.md](docs/validation.md) | How Mitra will be validated: a ladder of toy `add` CLIs scored against ground truth |
| [docs/trials.md](docs/trials.md) | Field trials, done by hand: a Rust project, an agile organisation judged from news, an agent judged against a skill |
| [docs/prior-art.md](docs/prior-art.md) | Related work and how Mitra differs |
| [docs/research/](docs/research/README.md) | Deeper reading: assurance tools, confidence calculi, Mitra's wedge |
| [docs/scoping-method.md](docs/scoping-method.md) | INVEST rules used to scope work |

## License

MIT, see [LICENSE](LICENSE).
