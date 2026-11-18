# Mitra

**Mitra is a static analyser for trust: you write down what your software
claims, what you assume and what evidence you have, and it tells you where
they contradict each other and how much of each claim is tested, assumed
or unsupported.**

*Tests check, Lean proves, Mitra weighs.* Tests and Lean proofs each produce
evidence. Mitra judges each claim against that evidence, and when things
do not add up, weighs which piece is most plausibly wrong.

> Status: early. Mitra reads a model kept aside from the code, checks
> that the Rust code is tied to it, turns test reports into evidence,
> gives each claim a verdict, lists the evidence still owed, and on an
> error ranks what is most plausibly wrong
> ([language reference](docs/reference/reworked.md)).

## The idea

At its heart Mitra is a small **language for trust**. A **subject** is
real code under review: its functions are opaque, or carry a body that
says what the code should return. A **boundary** is the environment,
which may answer differently each time. A **claim** is a property of a
subject, an **assumption** is a claim taken as true until observed, and
**evidence** is what a test, a review or a report says, with the trust
its author puts in it. Mitra never runs the code. It gives every claim a
verdict without a number (`holds`, `holds assuming`, `unverified`,
`fails`), with the inputs tested, assumed and missed.

```rust
import std::claims::{represented};

subject calc {
    fn div(a: 0..=9, b: 0..=9) -> 0..=9 { if b == 0 { 0 } else { a / b } }

    claim div_rest = represented(div, { (a, b) | b != 0 });
    claim div_small() { div(6, 2) == 3 }
    claim by_zero() { div(1, 0) == 0 }
}

evidence unit_test { for div_small holds source "test:div_by_two" }
evidence review { for div_rest holds source "review:ana" trust 0.9 }
```

One test passed, and a reviewer vouches that every division by a
non-zero number behaves like the others:

```text
calc::div::spec: unverified
  passes on       `calc::div(6, 2)`
  assumed on      89 inputs, by `calc::div_rest`
  no evidence on  (a, b) in {(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0)}
calc::div_small: holds
calc::by_zero: holds assuming [calc::div::spec]
calc::div_rest: holds
```

Nothing tested the division by zero, and `mitra plan` lists the tests
owed. Then a bug report arrives, trusted at 0.8:

```rust
evidence bug_report { for div given a = 8, b = 4 gives 4 trust 0.8 }
```

```text
error[M001]: evidence refutes a claim
  --> model.mitra:4  fn div(a: 0..=9, b: 0..=9) -> 0..=9 { if b == 0 { 0 } else { a / b } }

  `calc::div::spec` fails on `calc::div(8, 4)`.
  The evidence that says so:

    output    evidence.mitra:3  evidence bug_report { for div given a = 8, b = 4 gives 4 trust 0.8 }

  The code, the claim or the evidence is wrong. Below, the trust given to each
  piece, and what is left of it once they are weighed against each other:

                                                      trust  left
    possible  the code of `calc::div`, model.mitra:4  none   0.00 to 0.71
    likely    evidence.mitra:3 (bug_report)           0.8    0.29 to 0.36
```

A second error says the review can't stand with the report: the code did
not behave the same on two inputs the reviewer called alike. Everything
is a suspect: the code, the claim, the signature, the assumption and the
evidence. `left` is what remains of the trust in each piece once they are
weighed against each other ([semantics](docs/design/semantics.md)). The
example is [`examples/reworked/readme`](examples/reworked/readme).

## Who it is for

Work where agents write the model and the code, and a person gives the
intent and reads the model and the report.

- **The agent models and codes.** A person gives intent and some
  requirements; a second agent diagnoses with Mitra. Mitra catches intent
  that drifted and work left without evidence.
- **The person writes the model as a scaffold.** Mitra checks that the
  code is tied to it and covered. The person's effort goes to the
  architecture.

The model lives aside from the code. The code points at it with one
comment per function (`// mitra: implements calc::div`) and per test
(`// mitra: evidence for div_small`); `mitra tie` and `mitra ingest` turn
both into evidence. Without them Mitra still works: the core reads only
a model and evidence, and nothing in it is specific to a language.

## Documentation

Two sets, kept apart:

**Language reference**: what the code does today, updated with every change
to it.

| Doc | What it covers |
|---|---|
| [docs/reference/reworked.md](docs/reference/reworked.md) | The language and the tool: model, evidence, ties, tests, commands, evaluation, output and error codes |

**General docs**: purpose, positioning and research. These describe
direction, not the current language.

| Doc | What it covers |
|---|---|
| [docs/vision.md](docs/vision.md) | What Mitra is, how it is layered, principles, non-goals |
| [docs/design/semantics.md](docs/design/semantics.md) | Design: verdicts without numbers, opinions on evidence, the loop they are judged against, and what the code does differently today |
| [docs/design/composition.md](docs/design/composition.md) | Earlier proposal, in part built another way: composition as implication, structure claims, tests as subjects |
| [docs/design/dogfooding.md](docs/design/dogfooding.md) | Mitra on Mitra: three crates with a model aside, tied to the code and judged from their own tests |
| [docs/design/requirements.md](docs/design/requirements.md) | Proposal: prose requirements as opaque propositions derived from claims, for traceability |
| [docs/design/elaboration.md](docs/design/elaboration.md) | Proposal: Mitra elaborates partial knowledge into claims, splits, specs and proof obligations |
| [docs/review/2026-11-18.md](docs/review/2026-11-18.md) | Review list: every decision taken alone, open questions, how to test |
| [docs/design/lean-syntax.md](docs/design/lean-syntax.md) | History: the first syntax (inspired by Lean 4), no longer read |
| [docs/design/state.md](docs/design/state.md) | Proposal: state as a value, traces, invariants |
| [docs/design/workflow.md](docs/design/workflow.md) | The loop: model, plan, code, tie, ingest, eval; and what is proposed (bugs, diagnosis, usage) |
| [docs/use-cases.md](docs/use-cases.md) | Where Mitra is headed: checking a model, gating a merge, gating an agent |
| [docs/validation.md](docs/validation.md) | How Mitra will be validated: a ladder of toy `add` CLIs scored against ground truth |
| [docs/trials.md](docs/trials.md) | Field trials, done by hand: a Rust project, an agile organisation judged from news, an agent judged against a skill |
| [docs/prior-art.md](docs/prior-art.md) | Related work and how Mitra differs |
| [docs/research/](docs/research/README.md) | Deeper reading: assurance tools, confidence calculi, Mitra's wedge |
| [docs/scoping-method.md](docs/scoping-method.md) | INVEST rules used to scope work |

## License

MIT, see [LICENSE](LICENSE).
