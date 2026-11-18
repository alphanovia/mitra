# Mitra

**Mitra is a static analyser for trust: you write down what your software
claims, what you assume and what evidence you have, and it tells you where
they contradict each other and how much of each claim is tested, assumed
or unsupported.**

*Tests check, Lean proves, Mitra weighs.* Tests and Lean proofs each produce
evidence. Mitra judges each claim against that evidence, and when things
do not add up, weighs which piece is most plausibly wrong.

> Status: early. The code evaluates claims, specs, derivations and
> uniformity assumptions against evidence, ranks what is most plausibly
> wrong on an error, reads the model from comments in Rust code, and
> keeps a project's evidence in a store fed by test reports
> ([language reference](docs/reference/language.md)).

## The idea

At its heart Mitra is a small **language for trust**. A **subject** is a
function under review, a **spec** is a definition Mitra can evaluate, a
**claim** says a subject is correct (or matches a spec) on some of its inputs,
or follows from other claims, a **uniformity claim** states the classes of
inputs on which a subject is assumed to behave the same,
and **evidence** is a run of the subject judged correct or wrong, with the
trust its author puts in it. Mitra gives every claim a verdict without a
number (`holds`, `holds assuming`, `unverified`, `fails`), with the inputs
tested, assumed and missed. On an error it lists the pieces involved and
how plausibly each is the wrong one.

```lean
opaque flip(a: Bool) -> Bool
claim flip_correct : forall a, correct(flip(a))
claim flip_on_true : correct(flip(true))

evidence correct(flip(true)) at 1
evidence not correct(flip(false)) at 0.8
```

```text
flip_correct: fails
  fails on        `flip(false)`
  passes on       `flip(true)`
flip_on_true: holds

error[M001]: evidence refutes a claim
  --> model.mitra:2  claim flip_correct : forall a, correct(flip(a))

  `flip_correct` fails on `flip(false)`.
  The evidence that says so:

    fails     evidence.mitra:2  evidence not correct(flip(false)) at 0.8

  The code, the claim or the evidence is wrong. Below, the trust given to each
  piece, and what is left of it once they are weighed against each other:

                                                 trust  left
    likely    the code of `flip`, model.mitra:1  none   0.00 to 0.20
    unlikely  evidence.mitra:2                   0.8    0.80 to 1.00

  The lower what is left, the more plausibly the piece is the wrong one.
  A range means the evidence does not settle it.

  help: if the evidence is right, fix `flip`; if not, correct the evidence
```

One counterexample refutes a claim; on an infinite domain, tests alone
never prove it. `left` is what remains of the trust in each piece once
they are weighed against each other ([semantics](docs/design/semantics.md)).

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
| [docs/design/semantics.md](docs/design/semantics.md) | Design: verdicts without numbers, opinions on evidence, the loop they are judged against, and what the code does differently today |
| [docs/design/composition.md](docs/design/composition.md) | Proposal: composition as implication, structure claims, tests as subjects |
| [docs/design/dogfooding.md](docs/design/dogfooding.md) | Plan: Mitra on Mitra: the Dempster-Shafer crate judged from its own tests, and Mitra's semantics written in Mitra |
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
