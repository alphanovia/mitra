# Research: assurance tools and where Mitra fits

A reading guide to the prior art around Mitra, written to be read over time.
Every section ends with references; the ones marked ★ are the best first read.
These notes were written against an earlier draft of the language: its
constructs (`via`, `split`, `unless`, `assume`, the `conflicts` query, toy
model 6) are not in the current one. Code snippets are illustrative only;
the [language reference](../reference/language.md) is the language, and
[design/calculus.md](../design/calculus.md) is the current calculus plan.

| File | What it covers | Read time |
|---|---|---|
| [landscape.md](landscape.md) | The tools and methods, one card each: what it does, strengths, limits, references | 45 min |
| [calculus.md](calculus.md) | How confidence is computed in the literature, what is wrong with subjective logic, which choices are well supported | 30 min |
| [wedge.md](wedge.md) | Where a solo developer can win against all of the above, and what this means for the `add` ladder | 15 min |

## The short version

1. **Critical industries do not compute trust.** Avionics (DO-178C), rail
   (EN 50128 / EN 50716) and automotive (ISO 26262) run on *traceability*:
   every requirement links to code, tests and reviews, and coverage is
   measured. Confidence is a human judgment written into an *assurance case*
   (GSN, CAE, SACM). Tools draw and check those links; none of them weighs
   evidence.
2. **Quantified confidence is a 25-year research topic with no industrial
   uptake.** Bayesian networks, Dempster-Shafer and subjective logic have all
   been applied to assurance cases. NASA's review (Graydon and Holloway, 2016)
   found that every method they could test produced implausible results on
   some case, most often by **masking missing or contrary evidence**: a
   failing test barely dents the score. Mitra's toy model 6 shows exactly
   that, and its `conflicts` query is the right answer to it.
3. **The closest work is very close.** Assurance 2.0 (Bloomfield and Rushby)
   adds defeaters and, since 2025, a probabilistic confidence method. Certus
   (2025) is a DSL for confidence. Herd et al. (2026) give GSN a
   compositional subjective-logic semantics, almost Mitra's draft 2. None of
   them is formally verified, ingests machine evidence, or targets developers
   and agents.
4. **Subjective logic is defensible at the leaves, weak in the middle.** An
   opinion built from counts *is* a Beta distribution, which is sound. Its
   fusion assumes independence (double counting), its multiplication is an
   approximation, its discounting has no probabilistic derivation, and its
   fusion rules have been formally criticised (Dezert et al., 2014).
5. **The literature supports a different stance on usage evidence.** n runs
   without failure bound the failure *rate* (about 3/n at 95% confidence);
   they never establish that code is *correct*. Only proofs, exhaustive tests
   or a prior on "perfection" reach that. This matters directly for the `add`
   ladder.
6. **Proposed wedge:** Mitra as the *evidence ledger for code written with
   agents*: it reads claims next to the code, takes machine evidence (test
   runs, Verus and Lean results, including every `assume`, `admit` and
   `external_body`), and answers three questions: what is proven, what is
   only tested, and where to look next. See [wedge.md](wedge.md).

## Suggested reading order

If you read only five things:

1. ★ Graydon and Holloway, *An Investigation of Proposed Techniques for
   Quantifying Confidence in Assurance Arguments*, NASA/TM-2016-219195. Why
   every confidence number so far has failed.
2. ★ Bloomfield and Rushby, *Assurance 2.0: A Manifesto* (2020),
   arXiv:2004.10474. The modern view of assurance cases.
3. ★ Bloomfield and Rushby, *Quantifying Confidence in Assurance 2.0
   Arguments*, SRI-CSL-25-01R2 (2025). Their answer to Graydon and Holloway.
4. ★ Littlewood and Strigini, *Validation of Ultra-High Dependability for
   Software-Based Systems*, CACM 36(11), 1993. What testing and usage can and
   cannot show.
5. ★ Herd, Kelly, Sabsch, Gauerhof, *Towards a compositional semantics for
   quantitative confidence assessment in assurance arguments*,
   arXiv:2605.22213 (2026). The nearest neighbour to Mitra's language.
