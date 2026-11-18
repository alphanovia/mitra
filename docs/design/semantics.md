# Design: semantics

Status: **implemented in part** (2026-11-18). Built: the verdicts, the
errors, and the ranking of what is most plausibly wrong, as the
[reference](../reference/language.md) describes them. Not built:
staleness and the last diff as weights, and the bug query (section 6).
Parts marked *decided* are the user's words (2026-11-18); parts marked
*proposed* are open for review.

## 1. Two sides

*Decided.* The core is the **model** and the **ledger of evidence**.

- **Model side: no probability.** Like a static analyser, Mitra gives each
  node of the model a verdict from a short list. The model holds, fails,
  or can't be decided.
- **Evidence side: opinion.** A piece of evidence may be wrong, and its
  author says how much they trust it (its **bias**). Mitra computes an
  opinion only when something does not add up, to say which piece is most
  plausibly wrong. The opinion is never about how true the model is.
- **One kind of evidence.** A test result, a review, a proof and a bug
  report are sugar over one raw form: a judgment on a claim or a subject,
  over some inputs, with a source and a bias. The kind never changes a
  verdict rule. A review is plain evidence.

## 2. Verdicts

*Decided:* a node is unverified, holds against the evidence, fails against
the evidence, or holds assuming its uncertain dependencies hold. Failures
are errors; where Mitra is not sure there is an error, it warns.

*Proposed* form:

| Verdict | Meaning | Reported as |
|---|---|---|
| `holds` | evidence covers every input, directly or through dependencies that hold | nothing |
| `holds assuming [A, B]` | the same, if A and B hold, and they are unverified | warning on A and B |
| `unverified` | some inputs are missed | warning |
| `fails` | evidence refutes the claim | error |
| `inconsistent` | on a set of claims and evidence that can't all be true | error |

Coverage is three **sets**, never a percentage: *tested*, *assumed* and
*missed*. A claim holds only when nothing is missed.

```text
div_correct : holds
  tested   {(1, 0), (6, 3)}
  assumed  every other input, by div_uniform (evidence line 1, review:alice)
  missed   nothing

div_correct : holds assuming [div_uniform]
  tested   {(1, 0), (6, 3)}
  assumed  every other input, by div_uniform
  missed   nothing
warning: div_uniform is unverified: no evidence on it

warning: div_correct is unverified
  tested   {(6, 3)}
  assumed  class "rest", by div_uniform (review:alice)
  missed   class "zero"
```

Dependencies are the claims a verdict rests on: a uniformity claim, the
premises of `by derive`, what `uses` names.

*Decided:* an assumption with evidence, such as a review, holds like any
claim, and what rests on it can be plain `holds`. `holds assuming [A]` is
for a dependency A with no evidence at all. The *assumed* set keeps the
review visible: it names the claim and the evidence it rests on.

*Proposed:* a suspect (`f` holds, but uses `g`, which fails) is a warning
on `f`.

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

*Proposed:* "the model being wrong" is weighed through its evidence: an
assumption's bias is the bias of the evidence on it, usually a review. A claim
that the code is correct has no bias of its own, so it takes whatever
the evidence leaves.

## 4. Opinion: which piece is wrong

*Decided.*

- The computation weighs the **candidate culprits**, and nothing else.
  What a number could mean on an unverified claim is left for later.
- **Bias never changes a verdict**: either the model holds or it does
  not. Bias only changes what we think is wrong, like a person reasoning
  about a bug from what is tested and enduring, what just changed, and so
  on.
- An entry at **bias 0** is still evidence for the verdict. If an error
  is raised, it is likely near the top of the candidates, because its
  author does not trust it. That is expected behaviour, not a rule.
- Candidates are **ranked by plausibility**.
- On a big model and ledger, many things can have a small non-zero value.
  The tool has a setting for **how many candidates an error shows** and a
  **threshold** under which they are hidden: one culprit at 0.99 and a
  hundred at 0.01 should show only the likely one. This is where
  thresholds live: on the display of an error, in the tool, not in the
  core.
- On a model where all the evidence agrees and everything is verified,
  the computation collapses to **fully trusting**: nothing is suspect.
- It should be correct and consistent all the time, if possible.
- Dempster-Shafer stays, for this computation. It must handle several
  pieces of evidence fighting each other, a bug query giving the
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

Staleness and the last diff are **inputs** to this computation: the
incremental tool works them out from git, and the core never reads git.

Combining them gives, for each entry, assumption and subject, a belief
and a plausibility of being the culprit. Errors print them (section 3);
a bug query ranks subjects by them.

`mitra-ds` keeps its role and its dogfooding. What goes away is the mass
per input (correct, wrong, unknown) that measured the truth of a claim.

*Proposed:* the properties the ranking must satisfy. Each becomes a
test, and later a proof.

- **P1. No error, no suspect.** Without an error or a bug report, no
  opinion is computed and nothing is doubted, whatever the biases.
- **P2. Distrust shows.** In an error, an entry at bias 0 has
  plausibility 1 of being the wrong one. It is expected near the top,
  not required to be first.
- **P3. Full trust shifts the blame.** If every entry against a claim
  has bias 1, the claim is the culprit, with belief 1. If every piece of
  an error is trusted at 1, assumption included, Mitra says so and ranks
  nothing: it never divides by zero.
- **P4. Only candidates are doubted.** Belief of being wrong is 0 for
  anything outside every inconsistent set and unreachable from every bug
  report.
- **P5. Monotone.** Lowering an entry's bias never lowers its rank.
- **P6. Order-free.** The result depends on the set of entries, not on
  their order or grouping: the ledger is merged by union.
- **P7. No double counting.** The same observation recorded twice (same
  source, input and revision) counts once.
- **P8. Continuous.** A small change in a bias gives a small change in
  the result.

## 5. The loop these semantics are judged against

Each verdict must lead to one obvious action. If a report leaves the user
not knowing what to do, the semantics are wrong there.

| Mitra says | The user does |
|---|---|
| `unverified`, missed M | adds a test in M, or states the assumption that lets existing tests stand for M |
| `holds assuming [A]` | reviews A and records the review as evidence, or tests more until A is not needed |
| `holds` | nothing, until the code changes and the evidence goes stale |
| `fails` | reads the ranking: fixes the code, or corrects the evidence |
| `inconsistent` | reads the ranking: drops or splits the assumption, fixes the code, or corrects the evidence |
| a bug arrives | records it (`mitra bug`), and reads the subjects ranked by plausibility of being broken |

The loop: write the model, run the tests, ingest, read the verdicts, act
on the first error or warning, repeat.

**Not in the core** (*decided*). Treating warnings as errors, requiring
some coverage of a domain, or allowing a share of unverified claims are
choices of whoever runs the tool. They belong to the tool and its
integrations, and are deferred. There is no policy on the opinion score.

## 6. What is built, and what is not

Built as designed: one verdict per claim (`holds`, `holds assuming`,
`unverified`, `fails`) with tested, assumed and missed; errors for a
failing claim, a contradiction and a conflict in a class, each with its
evidence and its pieces ranked by plausibility; bias used only there;
one kind of evidence; no `policy`, `profile` or belief; a display limit
and threshold in the tool; exit status 1 on errors.

Built differently from the proposals above:

- **Several pieces can be wrong at once.** The frame is not "which one
  piece is wrong" but every way of calling each piece right or wrong.
  With one culprit only, a failure that both refutes a claim and
  conflicts with a pass was blamed with certainty, which is wrong.
  Errors that share a piece are computed together (question 1).
- **The code of a subject is a piece**, next to evidence and assumptions,
  and so is the code of what it uses. This is how `uses` enters the
  ranking: a failing callee takes the blame from its caller.

Not built, and whose layer they belong to:

| Missing | Layer |
|---|---|
| Staleness of evidence and the last diff as weights | the incremental tool: it reads git and hands the core a weight per piece. The core takes no such input yet; `--rev R` still leaves out evidence from another revision |
| `mitra bug`, ranking subjects from a report | the tool; the ranking it needs is the one errors use |
| Warnings as errors, coverage thresholds | the tool and its integrations |
| A warning for evidence that no claim covers | the core |

## 7. Open questions

1. **One culprit or several.** Built with several (section 6): it costs
   2^k steps for k pieces with a bias strictly between 0 and 1 in one
   group of errors, capped at 20. Is that the right trade?
2. **Which rule.** Dempster hides conflict by renormalising; Yager moves
   it to "unknown". Built with Dempster, the conflict always shown. It
   jumps near total conflict (P8 fails there), and at total conflict
   there is no ranking at all.
3. **Weights.** How much a diff or an age discounts, and who sets it.
   Proposed: fixed defaults in the tool (stale evidence halves its bias;
   the last diff gets a small fixed weight), tuned on the mitra-ds
   benchmark, not set in the model.
4. **Display defaults.** Proposed: at most 5 candidates, none under a
   plausibility of 0.05, and one line saying how many were hidden and how
   to see them all.

5. **`holds assuming` with no evidence at all.** A derived claim whose
   implication is checked `holds assuming` its premises even when nothing
   was tested (golden `compose/01`). Is that the wanted reading?
6. **A rate is an error.** A test that passes sometimes gives a pass and
   a failure on one input: an error, not a frequency. Trials T2 and T3
   will meet this.

## Why not a confidence in the claim

Statistical testing, reliability growth models and confidence scores for
assurance cases never reached everyday development. They treated the code
as a black box, so the numbers were absurdly pessimistic; they needed
operational profiles nobody had; and "0.97" says neither where to look
nor what to do next. Mitra instead writes down the hypothesis a developer
relies on (one passing input stands for its class) as an assumption a
reviewer can read, and keeps numbers for the one question where they
help: which piece to doubt first.
