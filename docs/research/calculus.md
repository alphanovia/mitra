# The calculus: what the literature supports

Mitra's draft 2 defines every construct in subjective logic (SL). This page
asks whether that is the right calculus, what is known to go wrong with it,
and which choices published work supports. It ends with recommendations.

## What Mitra needs from a calculus

1. **Leaves from evidence.** Turn "12 passes, 1 failure" or "a checked proof"
   into a value, and keep *no evidence* distinct from *mixed evidence*.
2. **Propagation.** Combine values through `and`, `split`, `via` and several
   evidence lines on one claim.
3. **Diagnostics.** Support `uncertain`, `next` and `conflicts` with a clear
   meaning.
4. **Verifiable.** Simple enough to prove in Lean and Verus, ideally with
   exact arithmetic.

## The candidates

| Calculus | Value | Ignorance ≠ 0.5? | Propagation | Verdict for Mitra |
|---|---|---|---|---|
| Point probability (plain Bayes) | P ∈ [0,1] | no | exact, needs full joint model | too little: hides "unknown" |
| Bayesian network | P with conditional tables | no | exact | sound, but users must fill tables; Graydon and Holloway broke three |
| **Beta / second-order Bayes** | distribution over P (mean, spread) | yes (spread) | exact for evidence, approximate for `and` | sound basis for leaves |
| Subjective logic | (b, d, u, a), equivalent to a Beta | yes (u) | fixed operator set, several approximate | good *representation*, mixed operators |
| Dempster-Shafer | belief/plausibility interval | yes | Dempster's rule | conflict handling notoriously poor (Zadeh's example) |
| Conservative Bayesian inference | worst-case bound on a failure rate | yes (by bounding) | for reliability claims | rigorous for usage and test evidence |
| Eliminative (Baconian) | doubts eliminated / total | n/a | counting | good for defeaters, not a probability |
| Fuzzy / linguistic (Certus) | fuzzy label | partly | min/max-style rules | easy to elicit, weak semantics |

**Key fact.** A binomial SL opinion with prior weight W = 2 is *exactly* a
Beta distribution: r passes and s failures give Beta(r + 2a, s + 2(1 − a)),
with b = r/(r+s+2), d = s/(r+s+2), u = 2/(r+s+2). So the leaves of Mitra are
already ordinary Bayesian statistics. The debate is about the **operators**,
not the value type.

## Subjective logic: what is known to go wrong

Each point says what it means for Mitra concretely.

1. **Fusion assumes independent evidence.** Cumulative fusion adds evidence
   counts. Two tests sharing a fixture, or an agent repeating the same claim
   twice, are counted twice. This is the most common practical failure and is
   named as an open problem even by SL's proponents (Herd et al. 2026).
   *Mitra:* toy models with the same source fused twice will look more
   certain than they are.

2. **Discounting (`via`) has no probabilistic derivation.** "Trust
   transitivity" scales belief by the trustor's P. It is a reasonable
   heuristic, there are several variants in the SL literature, and it is not
   what Bayes' rule gives for an unreliable witness. *Mitra:* the open
   question "should `via` stack with policy or replace it?" has no principled
   answer inside SL, which is a sign the operator is underspecified.

3. **Multiplication (`and`) is an approximation.** The product of two Beta
   variables is not a Beta. SL picks a Beta with the right mean and an
   approximate spread. Kaplan and Ivanovska (2018) showed a second-order
   Bayesian network propagates uncertainty more accurately than SL operators
   on the same graph. *Mitra:* fine for toy models; formal proofs of the
   engine will be about an approximation, which must be said.

4. **Masking.** A dogmatic opinion (a proof, u = 0) dominates fusion, and
   strong evidence absorbs a few contrary observations. Graydon and Holloway
   list exactly this failure against Duan et al.'s Beta/SL method.
   *Mitra:* toy model 6 (bug report against a proof: 0.904 → 0.824) is this
   failure. The `conflicts` query is the right remedy; it must be part of
   every result, not an optional query.

5. **Base rate dominates when u is large.** P = b + a·u, so with little
   evidence the answer is mostly the base rate, which the user chose.
   *Mitra:* always show u next to P (already done) and list base rates among
   the assumptions a result rests on.

6. **Formal objections to fusion.** Dezert, Tchamova, Han and Tacnet (2014)
   argue SL's consensus rule mishandles agreeing mass, is undefined for
   Bayesian (u = 0) inputs, and that the opinion-to-Beta mappings across SL
   versions are inconsistent. They conclude they "cannot reasonably recommend"
   SL for fusion. Jøsang has answered parts of this, and the debate is partly
   between the Dezert-Smarandache school and SL; but the Bayesian-input case
   matters for Mitra, because a proof is a u = 0 opinion.

7. **The conflict measure is a heuristic.** SL's conflict degree has no
   probabilistic meaning. *Mitra:* better-founded options exist (see
   diagnostics below).

**Summary.** Keep SL's *representation* (b, d, u, a): it is a Beta
distribution and reads well. Do not adopt its *operator set* wholesale.

## Usage evidence: what n runs can show

This is the most important point for the `add` ladder.

**Universal versus statistical claims.** "`add` is correct for every pair of
small integers" is a *universal* claim. "`add` fails on fewer than 1 in 1,000
runs of normal use" is a *statistical* claim. Usage and random testing are
samples, so they bear on the statistical claim only.

**The numbers.** After n runs without failure, the 95% upper bound on the
failure probability is about 3/n (the "rule of three"). To claim a failure
rate below 10⁻³ at 99% confidence takes about 4,600 failure-free runs
(n = ln 0.01 / ln 0.999). Littlewood and Strigini (1993) and Butler and
Finelli (1993) showed that this is why ultra-high reliability can never be
demonstrated by testing alone.

**What SL says instead.** With the default mapping, 18 passes give u = 0.1
and P ≈ 0.95. Read as a Beta, that is "the next run probably succeeds", not
"the code is correct". If the claim is universal, 18 passes say very little.

**How people get to "it is correct for small numbers" anyway.** They assume
the region is an *equivalence class*: the code treats all small inputs the
same way, so a few samples stand for all of them. That is the
equivalence-partitioning assumption from testing textbooks. Mitra can make it
explicit:

```mitra
claim small_ok      "add is correct for every small pair"
claim small_uniform "add handles all small pairs by the same path"
assume small_uniform = (0.9, 0, 0.1)        # from reading the code
evidence small_ok : usage +18 via small_uniform
```

Now the result honestly rests on `small_uniform`, and `next` can point at it:
reading the code, a proof, or branch coverage would raise it.

**How proofs fit.** Conservative Bayesian inference treats a proof as raising
the **probability of perfection** (no faults at all in what it covers), and
usage as bounding the failure rate if imperfect. The two combine by a known
formula (Littlewood and Rushby 2012; Zhao et al. 2018), which is the
principled version of "a proof plus a little usage beats a lot of usage".

## Composition: pick the operator by the relation

Bloomfield and Rushby (2025) give a recipe for how sub-claims combine, which
SL lacks. Mapped to Mitra:

| Relation between sub-claims | Example | Combination | Mitra construct |
|---|---|---|---|
| **Partition** of a universal claim | correct on small *and* on overflow | all must hold: product | `split` (as now) |
| **Partition** of a statistical claim | failure rate over the whole profile | weighted sum by each part's share of the profile | `split` with weights (new) |
| **Diversity** | a proof and a test of the same property | doubts multiply: 1 − (1−P₁)(1−P₂), if truly independent | several `evidence` lines |
| **Containment** | a proof of a stronger property | take the stronger | (not yet expressible) |
| **Cumulative** | each step relies on the previous | chain rule | `via` chains |

## Diagnostics: better-founded versions exist

- **`conflicts` → model-based diagnosis.** Reiter (1987) and de Kleer and
  Williams (1987) formalised "something is wrong, which component?" as
  *conflict sets* (sets of assumptions that cannot all be true given the
  observations) and *diagnoses* (minimal sets whose failure explains
  everything), ranked by prior probability. Mitra's suspect list is a
  one-route version of this; the full theory handles several conflicts at
  once and is well understood and verifiable.
- **Bayesian conflict measure.** Jensen et al. (1991) define data conflict as
  log(∏ P(eᵢ) / P(e)): positive when observations are less likely together
  than apart. A principled replacement for SL's conflict degree.
- **`next` → value of information / sensitivity analysis.** Standard in
  Bayesian networks. Mitra's "set the leaf to certain and see how P moves" is
  a crude sensitivity; expected change in P from one more piece of evidence of
  a given kind is the decision-theoretic version, and is cheap for Beta leaves.
- **Spectrum-based fault localization** (Ochiai, Tarantula) ranks code by how
  often it appears in failing versus passing runs. If usage evidence records
  which region or code path ran, Mitra gets this for free.

## Recommendations

What the literature supports, in priority order:

1. **Keep the graph calculus-neutral.** Claims, evidence, assumptions,
   `split`, `via` and defeaters are the language. Every serious proposal
   (GSN, SACM, Assurance 2.0, Herd et al.) separates structure from numbers.
2. **Leaves: Beta from counts, shown as (b, d, u).** Well founded, exact in
   rationals, readable. This is SL's representation and need not change.
3. **Give `via` a probabilistic meaning.** Model a test or monitor as a noisy
   detector: x_covers becomes its *detection probability* (the chance it
   fails if f is wrong), x_ok its chance of a correct verdict. Bayes' rule then
   says how much a pass or a failure moves f_ok, and the "stack or replace"
   question disappears. SL deduction (Herd et al.) is the SL-native
   alternative.
4. **Type the claims: universal or statistical.** Usage supports statistical
   claims; proofs and exhaustive tests support universal ones; an explicit
   uniformity assumption lifts one to the other.
5. **Track provenance.** Each evidence line records its source; lines sharing
   a source are not fused as independent.
6. **Choose `split` semantics by claim type** (product for universal,
   weighted sum for statistical), following Assurance 2.0's patterns.
7. **Conflicts first.** Report conflicts with every result, and build the
   suspect list on model-based diagnosis.
8. **Test against the known failures.** Encode Graydon and Holloway's
   counterexamples as toy models; each must show a conflict or high
   uncertainty, never a confident score.

None of this makes the work larger at the start: the leaves and the syntax
stay; what changes is the meaning of `via`, `split` and fusion, which are
still on paper.

## References

- Jøsang, *Subjective Logic: A Formalism for Reasoning Under Uncertainty*,
  Springer, 2016.
- ★ Dezert, Tchamova, Han, Tacnet, *Can We Trust Subjective Logic for
  Information Fusion?*, FUSION 2014.
- Kaplan and Ivanovska, *Efficient Belief Propagation in Second-Order
  Bayesian Networks for Singly-Connected Graphs*, Int. J. Approximate
  Reasoning 93, 2018.
- ★ Herd, Kelly, Sabsch, Gauerhof, *Towards a Compositional Semantics for
  Quantitative Confidence Assessment in Assurance Arguments*,
  arXiv:2605.22213, 2026.
- Shafer, *A Mathematical Theory of Evidence*, Princeton, 1976. Zadeh's
  critique of Dempster's rule: *A Simple View of the Dempster-Shafer Theory of
  Evidence*, AI Magazine 7(2), 1986.
- Wang, Guiochet, Motet, Schön, *Safety Case Confidence Propagation Based on
  Dempster-Shafer Theory*, Int. J. Approximate Reasoning 107, 2019.
- ★ Graydon and Holloway, NASA/TM-2016-219195, 2016.
- ★ Bloomfield and Rushby, *Quantifying Confidence in Assurance 2.0
  Arguments*, SRI-CSL-25-01R2, 2025.
- ★ Littlewood and Strigini, *Validation of Ultra-High Dependability for
  Software-Based Systems*, CACM 36(11), 1993.
- Butler and Finelli, *The Infeasibility of Quantifying the Reliability of
  Life-Critical Real-Time Software*, IEEE TSE 19(1), 1993.
- Littlewood and Rushby, IEEE TSE 38(5), 2012 ("possibly perfect").
- Zhao, Littlewood, Povyakalo, Strigini, Wright, RESS 175, 2018.
- Reiter, *A Theory of Diagnosis from First Principles*, Artificial
  Intelligence 32(1), 1987. de Kleer and Williams, *Diagnosing Multiple
  Faults*, Artificial Intelligence 32(1), 1987.
- Jensen, Chamberlain, Nordahl, Jensen, *Analysis in HUGIN of Data Conflict*,
  UAI 1990.
- Abreu, Zoeteweij, van Gemund, *On the Accuracy of Spectrum-Based Fault
  Localization*, TAIC PART 2007.
