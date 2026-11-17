# Roadmap

v1 builds and proves Mitra's core engine. The product and agent use cases come
after v1 (see [vision.md](vision.md)). Every item below follows
[scoping-method.md](scoping-method.md): sizes are S (up to a day) or M (2 to 3
days), and each row records its INVEST check.

**How proofs fit in.** Proofs are part of each item, not a later phase. An
operator item is done when its definition and key property are in Lean 4, its
Rust implementation is verified against that definition with Verus, and its
reference models pass. If, after the spikes, an item no longer fits in M, it is
split into a Lean part and a Verus part.

## Milestone 0: Decide (spikes)

**Outcome:** the open questions that block estimation are answered and
recorded. **Exit demo:** three decision records merged in `docs/decisions/`.

| ID | Item | Size | Done when |
|---|---|---|---|
| SP1 | Choose the calculus and the shape of the trust value | M | A decision record compares subjective logic, Dempster-Shafer and a Bayesian network on five hand-worked models (no evidence, one test, conflicting evidence, two independent tests, a broken assumption), and picks one. |
| SP2 | Choose a number representation that both Lean and Verus can reason about | S | A decision record picks one (exact rationals, fixed point or bounded integers), with a 20-line Verus proof-of-concept of one operator using it. Floats are ruled out unless shown provable. |
| SP3 | Choose how the Lean definitions and the Verus implementation stay in step | S | A decision record describes the link (for example: Lean is the reference, the corpus is evaluated by both, outputs must match exactly), with one operator done end to end. |

INVEST: each spike is independent of the others, negotiable in method, valuable
because it makes M1 estimable, sized, and testable through its written
deliverable.

## Milestone 1: A program can score a claim

**Outcome:** a Rust program can load a model and get a proven trust value for
any claim in it. **Exit demo:** `cargo test` runs the reference corpus (at
least 30 models, nominal and corner cases) and every output matches the Lean
reference exactly.

| ID | Item | Size | Depends on | Acceptance criteria (summary) |
|---|---|---|---|---|
| 1.1 | Run the reference corpus against the engine | M | SP1–3 | A model file with one claim and an expected value is loaded, scored and compared; CI runs Lean, Verus and the corpus on every push. |
| 1.2 | Score a claim from direct evidence | M | 1.1 | Evidence for, evidence against and no evidence give three distinct, expected values; no evidence gives maximal uncertainty (proven). |
| 1.3 | Fuse independent evidence for one claim | M | 1.2 | Two agreeing tests raise belief; result is independent of evidence order (proven); conflicting evidence raises disbelief, not just uncertainty. |
| 1.4 | Require all sub-claims (conjunction) | S | 1.2 | A claim needing A and B is never trusted more than its weakest sub-claim (proven); corpus covers one weak leg. |
| 1.5 | Accept any of several arguments (alternatives) | S | 1.2 | A claim with two alternative arguments is trusted at least as much as its best one (proven). |
| 1.6 | Discount by assumptions | M | 1.2 | Lowering an assumption's trust never raises any claim that rests on it (proven); a fully distrusted assumption leaves its claims at maximal uncertainty. |
| 1.7 | Apply defeaters | M | 1.2 | Adding a defeater never raises a claim's trust (proven); corpus covers a defeater that is itself weakly supported. |
| 1.8 | Reject malformed models with clear errors | S | 1.1 | Unknown references, cycles and out-of-range values each produce a specific error naming the element; evaluation of valid models terminates (proven). |

INVEST notes:

- **Independent:** 1.3 to 1.7 depend only on 1.2 and can be built in any order.
- **Negotiable:** criteria fix behaviour and properties, not formulas; the
  formulas follow from SP1.
- **Valuable:** each adds a modelling construct a program can use immediately.
- **Estimable / Small:** all S or M once the spikes are done.
- **Testable:** every item adds corpus models with exact expected values, and
  names the property it proves.

## Milestone 2: A human can see why

**Outcome:** a program and a person can see not only a claim's trust value but
what limits it. **Exit demo:** the demo app opens three corpus models and shows
each score with its limiting element.

| ID | Item | Size | Depends on | Acceptance criteria (summary) |
|---|---|---|---|---|
| 2.1 | Name the element that limits a claim | M | M1 | For each corpus model, the reported limiting element is the one whose improvement raises the claim the most (checked against brute force on the corpus). |
| 2.2 | Expose the engine as a CLI with machine-readable output | S | M1 | `mitra eval model.toml --claim X` prints the trust value and limiting element as JSON; a non-zero exit code when a policy threshold passed as a flag is not met. |
| 2.3 | Show a model's scores in a minimal demo app | M | 2.1, 2.2 | Opening a model shows each claim's score and limiting element; nothing is editable. |

## Not in v1

Listed so they are not drifted into: the product case (deriving models from
code, tests and specs), the agent harness, CI or test-framework integrations,
attestation formats, learning weights from data, multi-party trust, and any UI
beyond 2.3. See [vision.md](vision.md#non-goals-for-v1).

## Size budget

| Milestone | Items | Budget (focused days) |
|---|---|---|
| M0 | 3 | 3 to 5 |
| M1 | 8 | 15 to 21 |
| M2 | 3 | 5 to 7 |
| **v1** | **14** | **23 to 33** |

At a few focused hours a week this is several months; check it against the
time actually available before committing to M1, and cut M2.3 first if it
does not fit.
