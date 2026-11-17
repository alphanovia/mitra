# Design: the calculus

Status: **spec**, not implemented. The code handles confidence 0 and 1 only,
with the `strict` and `coverage` policies
([reference](../reference/language.md#policy)). This page says what a
calculus for the confidences in between must do. Dempster–Shafer is no
longer a given: it is one candidate, scored below.

The order is: axioms, then the smallest representation that can state them,
then the operators, then how they are checked. A calculus is chosen by
passing the axioms, not the other way round.

## 1. Axioms

On one input, evidence is a judgment (correct or wrong) at a confidence c
in [0, 1]. Combining all evidence on an input gives its state; aggregating
states over a claim's domain gives the verdict.

**Extremes** (the rules today's golden set already checks):

- **E1.** Correct at 1 on every input of the domain: the claim holds, belief 1.
- **E2.** Wrong at 1 on one input: the claim fails.
- **E3.** No evidence: uncertain, nothing believed either way.
- **E4.** Correct at 1 and wrong at 1 on the same input: that input is
  unknown, and the contradiction is reported.
- **E5.** Evidence at 0 changes nothing.

**Structure:**

- **A1. Order-free.** The result depends on the set of evidence, not on its
  order or grouping. The store is append-only and merged by union, so any
  order can occur.
- **A2. Monotone.** More evidence that an input is correct never lowers its
  belief correct, and never raises its belief wrong. The same holds the
  other way round for evidence that it is wrong.
- **A3. Conflict stays visible.** Opposite evidence is never averaged into a
  confident middle. The state carries a conflict measure, and it is reported.
- **A4. Ignorance is not doubt.** No evidence and balanced evidence are
  different states.
- **A5. Counterexamples decide.** Belief that a claim fails is at least the
  strongest belief wrong on any one input. Belief that it holds needs the
  domain covered.
- **A6. No double counting.** The same observation, recorded twice (the same
  source on the same input and revision), counts once.
- **A7. Discounting.** Evidence from a source trusted at t is unchanged at
  t = 1, worth nothing at t = 0, and monotone in t.
- **A8. Continuous.** A small change in a confidence gives a small change in
  the result: no jump near total conflict.

## 2. Representation

Per input: a state (b, d, u), meaning belief correct, belief wrong and
unknown, with b + d + u = 1, plus the conflict k met so far. All values are
exact rationals, so the axioms can be proved, not just tested.

The confidence c of a piece of evidence is **the probability that the
observation is valid**: the run happened as recorded and its judgment is
right. Correct at c gives (c, 0, 1 − c).

Per claim and policy: the logic (holds, fails, uncertain), the covered and
refuted sets, belief holds, belief fails, and the conflict. This is today's
verdict plus belief fails and conflict.

## 3. Operators

| Operator | Takes | Gives |
|---|---|---|
| `judge` | a judgment and c | a state |
| `combine` | all states on one input | one state and its conflict |
| `discount` | a state and a source trust t | (t·b, t·d, 1 − t·(b + d)) |
| `aggregate` | the states over a domain and a profile μ | belief holds, belief fails |
| `derive` | belief in the implication and the premises | a lower bound on belief in the conclusion |

A6 is met by provenance, not by the arithmetic: evidence is deduplicated by
source, input and revision before `combine`.

For `aggregate`, the current proposal is: belief holds = ∫ b dμ / μ(D),
belief fails = max d. Whether a universal claim (correct everywhere) and a
statistical one (rarely wrong in use) need different aggregates is open
([research](../research/calculus.md#usage-evidence-what-n-runs-can-show)).
So is the rule that turns an in-between state into a logic word.

## 4. How it is checked

1. **Golden cases.** E1 to E5 are the golden set at 0 and 1, which exists.
   Every candidate must reproduce it unchanged. Known failures from the
   literature (Zadeh's example, Graydon and Holloway's masking cases) are
   added as golden cases.
2. **Properties.** Each axiom is a property test over random states, in the
   way `mitra-ds` is tested today, and is judged by Mitra from those runs.
3. **Proofs.** The axioms are proved on exact rationals, with Verus for the
   Rust code and Lean for the algebra (for example associativity).

**Plan:** rename `mitra-ds` to `mitra-calculus`, whose claims are these
axioms, one per operator, instead of "is Dempster–Shafer". Not started.

## 5. Candidates against the axioms

The table covers `combine` on the binary frame {correct, wrong}. ✓ means the
property is known to hold, ✗ that it is known to fail, and ? that it has not
been checked.

| | Dempster | Yager | Subjective logic, cumulative fusion | Yager grouped by side |
|---|---|---|---|---|
| E1, E2, E3, E5 | ✓ | ✓ | ✓ | ✓ |
| E4 contradiction at 1 | ✗ undefined | ✓ unknown | ✗ certain 50/50 | ✓ unknown |
| A1 order-free | ✓ | ✗ not associative | ✓ | ✓ |
| A2 monotone | ✓ | ✓ | ✓ | ✓ |
| A3 conflict visible | ✗ renormalised away | ✓ moved to unknown | ✗ averaged | ✓ |
| A4 ignorance ≠ doubt | ✓ | ✓ | ✓ | ✓ |
| A6 no double counting | by provenance | by provenance | by provenance | by provenance |
| A7 discounting | ✓ (Shafer's) | ✓ (Shafer's) | ✓ native | ✓ (Shafer's) |
| A8 continuous | ✗ near total conflict | ✓ | ✗ at u = 0 | ✓ |

**Yager grouped by side** combines all evidence on an input in one step.
The correct side gives B = 1 − Π(1 − cᵢ) and the wrong side gives
D = 1 − Π(1 − cⱼ). Then b = B(1 − D), d = D(1 − B), conflict k = B·D, and
u = 1 − b − d. On two pieces of evidence it equals Yager's rule. Because it
reads the whole set at once, it is order-free. It is the leading candidate;
the ✓ marks in its column are hand checks, so they are proof obligations,
not results.

Subjective logic remains the strongest rival for what is not in this
table. It handles source trust and base rates natively, and its opinions
are Beta distributions ([research](../research/calculus.md)). Its fusion is
what fails E4 and A3.

## 6. Why earlier confidence measures failed

Statistical testing, reliability growth models and confidence scores for
assurance cases never reached everyday development, for three reasons:

1. **They treated the code as a black box.** A developer who knows the code
   assumes that one passing input stands for a whole interval: a
   **uniformity hypothesis** (Gaudel's testing theory). Without it, high
   confidence takes thousands of runs (Littlewood and Strigini; Butler and
   Finelli). The numbers came out absurdly pessimistic, and were ignored.
2. **Their inputs were unknowable.** They needed an operational profile and
   independent failures, which nobody had, so they could not be calibrated.
3. **They gave numbers nobody could act on.** "0.97" says neither where to
   look nor what to test next.

Mitra makes the hypothesis explicit instead. A test's claim that it
exercises a set of inputs (`t_small` exercises `add` on `0..=10`) already
carries it: one pass stands for the set. A separate claim on the subject,
`uniform add on S`, would let any test or review reuse the hypothesis, and
would take the blame when a bug lands inside S. Whether that is worth a new
concept is left for the [Rust trial](../trials.md#t1-a-small-rust-project-built-with-mitra)
to settle. Mitra also answers with what to do next (`mitra next`), not with
a score alone.
