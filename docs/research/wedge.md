# The wedge

Where can one developer build something people would pick over the prior art
in [landscape.md](landscape.md)? This page proposes an answer and says what it
means for validation.

## Where the prior art is strong: don't compete there

- **Certification.** DOORS, Polarion, LDRA and VectorCAST own regulated
  traceability, and anything used for certification needs tool
  qualification. Out of reach for a solo project, and not where the need is.
- **Assurance cases for safety engineers.** GSN tools, Assurance 2.0 with
  Clarissa, Certus and Herd et al.'s SL semantics serve assessors well, and
  that community moves slowly. A new calculus there is one more paper.
- **AI lab safety cases.** Prose arguments about whole models, written by
  research teams.

## Where nobody is

The table at the end of [landscape.md](landscape.md) shows the gap: no tool
combines **weighing evidence**, **localizing conflicts**, **reading machine
evidence**, and **living as text next to the code**, for **developers and
agents** rather than assessors.

The people in that gap are growing fast: developers who have an agent write
code, tests and increasingly *proofs* (Verus, Lean, Dafny). They have a
specific, unanswered question: **"Everything is green. What do I actually
know?"** The vericoding research shows why that question is hard. Agents
reach "verified" with `assume`, `sorry`, `external_body` or a spec too weak
to mean anything; benchmarks found about 9% of specs admitted trivial
solutions and 15% were mistranslated.

## Proposed wedge

> **Mitra is the evidence ledger for code written with agents: it reads the
> claims next to the code and the results of tests and proofs, and shows what
> is proven, what is only tested, what is assumed, and where to look next.**

Concretely, the first useful Mitra:

1. **Reads** ` ```mitra ` blocks in docstrings and Markdown (the language as
   designed).
2. **Ingests** machine evidence: test results (`cargo test`, JUnit XML), and
   verifier results from Verus and Lean, turning every trusted-base item
   (`assume`, `admit`, `sorry`, `axiom`, `external_body`) into an explicit
   `assume` claim.
3. **Answers** three questions, as text a person or an agent can read:
   - *What does this rest on?* Every assumption under a claim, including the
     ones the agent introduced.
   - *Where is it thin?* The uncertainty map: claims that are only tested, or
     only used, or not covered at all.
   - *What is wrong?* Conflicts between evidence lines, with ranked suspects
     (a failing test against a proven function points at the spec or the
     model first).

**Why someone would choose it over the prior art:**

| They have today | Mitra adds |
|---|---|
| "Verified ✓" from Verus or Lean | *verified under what*: the trusted base, listed and weighed |
| Green tests and a coverage % | which claims those tests actually support, and how strongly |
| Traceability (StrictDoc, LOBSTER) | a computed answer, not just links, and a diagnosis when evidence disagrees |
| An agent's report "done, tests pass" | the agent's own claims checked against independent evidence |
| Assurance case tools | lives in the repo, written by agents, no GUI, runs in CI |

And one thing no competitor can claim: **the engine itself is proven**, so the
tool that tells you what to trust does not itself need to be taken on trust.

**Why it is small enough for one person.** No GUI, no editor, no
certification. One language, one evaluator, a handful of evidence readers
(test results, Verus and Lean markers). The trusted-base extraction is
mostly pattern matching. The agent use case comes for free: an agent writes
the ` ```mitra ` blocks, and a harness asks Mitra the same three questions.

**Both use cases fit.** "Can I trust what I built or vibe-coded" is the
ledger on a repository. "Can I trust the agent" is the ledger on one agent
task: the agent's claims, the harness's checks, and what the result rests on.

**Positioning line, unchanged:** *Tests check, Lean proves, Mitra weighs.*
A sharper tagline for this wedge: *"Everything is green. Mitra tells you what
that actually means."*

### Alternatives considered

- **Agent harness gate first (UC3).** Valuable, but the harness space is
  crowded and integration-heavy, and the gate is only as good as the ledger
  underneath. Better as the second step.
- **A better assurance-case calculus.** Intellectually closest to the
  literature, but it competes with Assurance 2.0 and Certus for a small,
  slow audience.

### Risks

- **Verus and Lean users are few.** The ledger also works with plain tests and
  coverage; proofs are where it is most distinctive, not its only input.
- **Numbers that look authoritative.** Graydon and Holloway's lesson. Lead
  with the diagnostics (what rests on what, where it is thin, what conflicts)
  and treat the score as secondary, as Assurance 2.0 does.

## What this means for the `add` ladder

The ladder in [validation.md](../validation.md) is a good test, and the
research suggests five additions:

1. **Say which claims are universal.** Usage alone can bound a failure rate,
   never prove correctness ([calculus.md](calculus.md#usage-evidence-what-n-runs-can-show)).
   R0's "confident `small` is correct after enough usage" is only reachable
   through an explicit assumption that small inputs behave alike. That is not
   a weakness: Mitra showing that assumption is exactly what a careful human
   does implicitly.
2. **Add a cheating rung.** R4 with a Verus proof that uses `assume` or
   `external_body`, or a spec too weak to rule out overflow. Mitra must report
   "proven under assumption" and point at it. This tests the wedge directly.
3. **Add a masking rung.** R4 with a parsing bug injected. The score may stay
   high, but `conflicts` must fire and name the parser before the proof.
   Encode Graydon and Holloway's counterexamples the same way.
4. **Compare with a baseline.** Run the same rungs through plain traceability
   (a coverage report of which regions have a passing test). Mitra is worth
   building only if its diagnosis beats that, for example by finding the
   overflow bug in R1 sooner.
5. **Question the 18-observation threshold.** It comes from SL's prior weight
   (u = 2/(n+2)). It is a fine first threshold, but it is a statement about
   the next run, not about correctness, so the "decided" criterion should
   differ for universal and statistical claims.

## Reading for this page

- Bursuc et al., *A Benchmark for Vericoding*, arXiv:2509.22908 (2025).
- Ye et al., *Vero*, arXiv:2608.13522 (2026).
- Graydon and Holloway, NASA/TM-2016-219195 (2016).
- Bloomfield and Rushby, *Assurance 2.0: A Manifesto*, arXiv:2004.10474.
