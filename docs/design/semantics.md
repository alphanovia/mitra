# Design: semantics

Status: **design, not implemented**. It replaces the confidence calculus
(belief numbers on claims). Parts marked *decided* are the user's words
(2026-11-18); parts marked *proposed* are open for review. What the code
does today is in the [reference](../reference/language.md), and the
differences are listed in section 6.

## 1. Two sides

*Decided.* The core is the **model** and the **ledger of evidence**.

- **Model side: no probability.** Like a static analyser, Mitra gives each
  node of the model a verdict from a short list. The model holds, fails,
  or can't be decided.
- **Evidence side: opinion.** A piece of evidence may be wrong, and its
  author says how much they trust it (its **bias**). Mitra computes an
  opinion only when something does not add up, to say which piece is most
  plausibly wrong. The opinion is never about how true the model is.

## 2. Verdicts

*Decided:* a node is unverified, holds against the evidence, fails against
the evidence, or holds assuming its uncertain dependencies hold. Failures
are errors; where Mitra is not sure there is an error, it warns.

*Proposed* form:

| Verdict | Meaning | Reported as |
|---|---|---|
| `holds` | every input is tested or proven, nothing is assumed | nothing |
| `holds assuming [A, B]` | every input is tested or follows from A and B, which Mitra can't decide | note; warning if A or B is not vouched for |
| `unverified` | some inputs are missed | warning |
| `fails` | evidence refutes the claim | error |
| `inconsistent` | on a set of claims and evidence that can't all be true | error |

Coverage is three **sets**, never a percentage: *tested*, *assumed* and
*missed*. A claim holds only when nothing is missed.

```text
div_correct : holds assuming [div_uniform]
  tested   {(1, 0), (6, 3)}
  assumed  every other input, by div_uniform (review:alice)
  missed   nothing

warning: div_correct is unverified
  tested   {(6, 3)}
  assumed  class "rest", by div_uniform (review:alice)
  missed   class "zero"
```

Dependencies are the claims a verdict rests on: a uniformity claim, the
premises of `by derive`, what `uses` names.

*Proposed:* anything resting on a review stays `holds assuming`, so
reviews remain visible. Only tests and proofs give plain `holds`. A
suspect (`f` holds, but uses `g`, which fails) is a warning on `f`.

## 3. Errors

*Decided.* An error report has three parts: what is not possible, the
evidence failing the model, and the plausibility of each piece being
wrong, against each other and against the model being wrong.

*Proposed:* every error is a set that can't all be true. `fails` is the
smallest case: one claim against the evidence refuting it.

```text
error: div_correct fails on {(4, 2)}
  1. claim div_correct                         the code is wrong   0.80 to 1.00
  2. evidence line 7: not correct (div 4 2)    bug report, at 0.8  0.00 to 0.20

error: inconsistent: these can't all be true
  1. claim div_uniform                         review:alice, at 0.9   0.28 to 0.35
  2. evidence line 4: correct (div 6 3)        test t_six, at 0.99    0.03 to 0.04
  3. evidence line 7: not correct (div 4 2)    bug report, at 0.8     0.62 to 0.70
  the biases together say nothing is wrong (conflict 0.71)
```

Each range is the belief and the plausibility that the line is the wrong
one (section 4). The numbers above are Dempster's rule on these biases.

*Proposed:* "the model being wrong" is weighed through its reviews: an
assumption's bias is the bias of the review vouching for it. A claim
that the code is correct has no bias of its own, so it takes whatever
the evidence leaves.

## 4. Opinion: which piece is wrong

*Decided.* Dempster-Shafer stays, for this computation. It must handle
several pieces of evidence fighting each other, a bug query giving the
plausibility of each node being broken, and staleness or the place of
the last diff as weights. It is not expected to be simple.

*Proposed.* The frame is the set of things that could be wrong: ledger
entries, assumptions, and subjects (the code). Each source below is a
mass on a subset of the frame, which is the native case of
Dempster-Shafer:

| Source | Mass |
|---|---|
| An inconsistent set S | 1 on "something in S is wrong" |
| The bias b of an entry e | b on "e is not the wrong one" |
| Stale evidence (its subject changed since) | its bias is discounted |
| The last diff touched subjects D | some weight on "something in D is wrong" |
| A bug report on `f`, at bias b | b on "`f`, or something it uses, is broken" |

Combining them gives, for each entry, assumption and subject, a belief
and a plausibility of being the culprit. Errors print them (section 3);
a bug query ranks subjects by them.

`mitra-ds` keeps its role and its dogfooding. What goes away is the mass
per input (correct, wrong, unknown) that measured the truth of a claim.

What the combination must satisfy, kept from the retired calculus spec:
order-free (the ledger is merged by union); the same observation
recorded twice counts once (by source, input and revision); a bias of 0
changes nothing; small changes in a bias give small changes in the
result.

## 5. The loop these semantics are judged against

Each verdict must lead to one obvious action. If a report leaves the user
not knowing what to do, the semantics are wrong there.

| Mitra says | The user does |
|---|---|
| `unverified`, missed M | adds a test in M, or states the assumption that lets existing tests stand for M |
| `holds assuming [A]`, A not vouched for | reviews A and records the review, or tests more until A is not needed |
| `holds assuming [A]`, A vouched for | nothing; A is what the next reviewer reads first |
| `holds` | nothing, until the code changes and the evidence goes stale |
| `fails` | reads the ranking: fixes the code, or corrects the evidence |
| `inconsistent` | reads the ranking: drops or splits the assumption, fixes the code, or corrects the evidence |
| a bug arrives | records it (`mitra bug`), and reads the subjects ranked by plausibility of being broken |

The loop: write the model, run the tests, ingest, read the verdicts, act
on the first error or warning, repeat.

## 6. What the code does differently today

The build increment is this list.

| Today | With these semantics |
|---|---|
| A verdict per claim and policy: `{ logic, covered, refuted, assumed, domain, belief }` | One verdict per claim: the enum, with tested, assumed and missed |
| `uncertain` | `unverified` |
| `holds` with an `assumed` field | `holds assuming [..]` |
| A uniformity claim not vouched for counts for nothing | open, question 1 |
| A pass and a failure on one input: the input counts on neither side, a conflict is printed, the verdict comes from the rest | `inconsistent` error |
| A pass and a failure in one class: the uniformity claim `fails` | `inconsistent` error naming the claim and both pieces of evidence |
| Evidence counts only at confidence 1; other values are ignored | the number is a bias, used only in the opinion (question 2) |
| `policy`, `profile`, `belief`, `coverage` as a ratio | removed (question 3) |
| `suspect` and `root cause` lines | warnings, and sources for the opinion |
| No plausibility anywhere; `mitra-core` does not use `mitra-ds` | errors carry belief and plausibility per piece |
| `--rev R`: evidence at another revision is ignored | staleness is a weight |
| No bug command | `mitra bug`, ranking subjects |

## 7. Open questions

1. **An assumption nobody vouched for.** "Holds assuming the uncertain
   dependency holds" reads as: `holds assuming [A]` even when A is
   unverified, with a warning on A. Today an unvouched uniformity claim
   gives nothing. Which one?
2. **Does bias change a verdict?** Proposed: no. Every ledger entry counts
   for the verdict; bias matters only when ranking culprits. Then an entry
   at 0 is the one exception (ignored), or is removed from the language.
3. **Policies and profiles.** Without belief or coverage ratios they have
   no use left. Remove them?
4. **The verdict of an assumption itself.** A vouched uniformity claim is
   neither tested nor proven. Proposed: it prints as `assumed
   (review:alice)`, and never as plain `holds` unless a backend or a
   proof checks it.
5. **One culprit or several.** With one wrong thing per frame, two
   unrelated inconsistent sets are in total conflict. The frame may have
   to be per connected group of errors, or range over sets of culprits.
6. **Which rule.** Dempster hides conflict by renormalising; Yager moves
   it to "unknown". Here conflict has a meaning (the biases claim nothing
   is wrong), so it should be shown. To settle on examples.
7. **Weights.** How much a diff or an age discounts, and who sets it.

## Why not a confidence in the claim

Statistical testing, reliability growth models and confidence scores for
assurance cases never reached everyday development. They treated the code
as a black box, so the numbers were absurdly pessimistic; they needed
operational profiles nobody had; and "0.97" says neither where to look
nor what to do next. Mitra instead writes down the hypothesis a developer
relies on (one passing input stands for its class) as an assumption a
reviewer can read, and keeps numbers for the one question where they
help: which piece to doubt first.
