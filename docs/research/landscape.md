# The landscape

One card per standard, tool or method. Each card says what it is, what it is
good at, where it stops, and what it means for Mitra. ★ marks the best first
read. Calculus details (Bayesian networks, Dempster-Shafer, subjective logic)
are in [calculus.md](calculus.md).

- [1. Standards: how critical industries decide](#1-standards-how-critical-industries-decide)
- [2. Traceability and verification tools](#2-traceability-and-verification-tools)
- [3. Assurance case notations](#3-assurance-case-notations)
- [4. Assurance case tools](#4-assurance-case-tools)
- [5. Assurance 2.0](#5-assurance-20)
- [6. Quantified confidence](#6-quantified-confidence)
- [7. AI systems and agents](#7-ai-systems-and-agents)
- [8. Summary: who does what](#8-summary-who-does-what)

---

## 1. Standards: how critical industries decide

### DO-178C (avionics software)

**What.** The RTCA/EUROCAE standard for airborne software. Software gets a
Design Assurance Level (A, most critical, to E). Each level comes with a list
of *objectives* (up to 71 at level A): requirements are traced to design, code
and tests; structural coverage is measured (MC/DC at level A); reviews and
independence are documented. Supplements cover tool qualification (DO-330),
model-based development (DO-331) and **formal methods (DO-333)**, which lets a
proof replace some testing objectives.

**Strengths.** Decades of flight experience. Evidence is concrete and
auditable. DO-333 is the precedent for "a proof counts as evidence, and here
is what it does not cover" (the proof's model, the compiler, the hardware).

**Limits.** It is a *process* standard: it says what must be done, not how
much confidence results. There is no number, and no way to say "this
requirement is better supported than that one". It does not claim a failure
rate for software at all; the rigour of the level stands in for one.

**For Mitra.** DO-333's split between "what the proof establishes" and "what
must still be tested" (that the model matches the code, that the compiler is
right) is the same split Mitra makes with `via model_ok`. Worth borrowing its
vocabulary.

**Read.** RTCA DO-178C and DO-333 (2011, paywalled). For an open overview:
Moy, Ledinot, Delseny, Wiels, Monate, *Testing or Formal Verification:
DO-178C Alternatives and Industrial Experience*, IEEE Software 30(3), 2013.

### EN 50128 / EN 50716 (rail), IEC 61508, ISO 26262 (automotive)

**What.** The same pattern in other industries. Software gets an integrity
level (SIL 0 to 4, or ASIL A to D) and each level *recommends* techniques in
tables ("formal methods: highly recommended at SIL 4"). EN 50716:2023 has
replaced EN 50128 for new rail projects. ISO 26262 requires a safety case.
UL 4600 (autonomous vehicles) is the first standard *built* around a safety
case rather than a checklist.

**Strengths.** Clear, shared vocabulary. Graded rigour.

**Limits.** Technique tables say what to do, not how well it worked. The link
from "technique applied" to "confidence in the claim" is left to assessors.

**For Mitra.** The integrity level is a policy: it says how strong the
evidence must be. Mitra's `policy` lines play the same role, but per evidence
kind and with a computed result.

**Read.** Bishop and Bloomfield, *A Methodology for Safety Case Development*,
1998 (short, free, explains why cases replaced checklists). UL 4600 overview at
ul.com.

---

## 2. Traceability and verification tools

### Requirements managers: IBM DOORS, Siemens Polarion, Jama Connect, PTC codebeamer

**What.** Databases of requirements with typed links (satisfies, verifies,
refines) to design, code and test results. They produce traceability matrices
and impact analysis ("this requirement changed, these 14 tests are now
suspect").

**Strengths.** Industry standard; auditors expect them. *Suspect links* (a
link flagged when either end changes) are a simple, much-loved diagnostic.

**Limits.** A link is present or absent. A requirement with one shallow test
and one with a proof look the same: both are "covered". Heavy, expensive,
GUI-first, and far from where code is written.

**For Mitra.** Mitra's "relationships are claims" idea is the exact fix for
"a link is binary": `x_covers` is a link with its own opinion. Suspect links
are a cheap diagnostic worth copying: evidence older than the code it is about
should lose weight.

### Coverage and test tools: LDRA, VectorCAST, Rapita RVS

**What.** Unit and integration test harnesses for C, C++ and Ada, with
structural coverage (statement, decision, MC/DC) measured on target, and
qualification kits for DO-178C and ISO 26262. LDRA and VectorCAST also link
results back to requirements.

**Strengths.** They produce the hard numbers assessors trust: which lines and
decisions were exercised.

**Limits.** Coverage says code was *run*, not that it was *checked*. 100%
MC/DC with weak assertions is still weak evidence. There is no notion of
evidence strength beyond pass, fail and coverage percent.

**For Mitra.** Coverage and mutation score are natural evidence for an
`x_covers` claim ("test X really exercises f"). That is a place where Mitra can
consume numbers others already produce.

### Docs-as-code traceability: StrictDoc, Sphinx-Needs, Doorstop, OpenFastTrace, TRLC + LOBSTER

**What.** Open-source tools that keep requirements as text in the repository
and trace them to code and tests. StrictDoc (Python, aimed at DO-178C and
medical) and Sphinx-Needs (an extension for Sphinx docs, popular in
automotive) are the most active. BMW's **TRLC** ("Treat Requirements Like
Code") is a small typed language for requirements, and **LOBSTER** collects
traces from code, tests and requirements into a coverage report.

**Strengths.** Text in git, reviewed in pull requests, close to the code:
the same bet Mitra makes. TRLC shows a requirements *language* can be small
and typed.

**Limits.** Still binary links and coverage counts; no weighing, no
uncertainty, no diagnosis of where support is thin beyond "untraced".

**For Mitra.** These are the closest tools to Mitra in *form*. Mitra should
interoperate rather than compete: read their trace output as evidence, and add
the weighing and the diagnosis they lack.

**Read.** github.com/strictdoc-project/strictdoc;
sphinx-needs.readthedocs.io; github.com/bmw-software-engineering/trlc and
/lobster.

### Static analysis and proof tools: SPARK, Frama-C, Astrée, Verus, Lean

**What.** Tools that prove properties of code: absence of runtime errors
(Astrée, SPARK, Frama-C/EVA) or full functional correctness (SPARK, Frama-C/WP,
Verus, Lean).

**Strengths.** The strongest evidence there is, for what they cover.

**Limits.** Each proof rests on a *trusted base*: the specification, the
model of the language, unproven lemmas (`assume`, `admit`, `sorry`,
`external_body`), the solver and the compiler. Tools report "verified" or not;
they rarely report what that verdict rests on in a form another tool can
weigh.

**For Mitra.** This is a gap Mitra can fill: turn a verifier's trusted base
into explicit `assume` claims, so a "proven" function shows exactly what it is
proven *under*. See [wedge.md](wedge.md).

---

## 3. Assurance case notations

An **assurance case** (safety case, security case) is a structured argument
that a system meets a claim, made of claims, sub-claims and evidence. The
three notations below describe the same idea.

### GSN: Goal Structuring Notation

**What.** A graphical notation from the University of York (Kelly, 1990s).
Goals (claims), strategies (how a goal is broken down), solutions (evidence),
context, assumptions and justifications. The Assurance Case Working Group
publishes the community standard (v3, 2021). Extensions add patterns
(reusable argument templates), modules, and "assured safety arguments" that
pair each step with a confidence argument (Hawkins et al., 2011).

**Strengths.** Widely used in UK defence, rail, nuclear and automotive.
Simple to read. Makes the argument reviewable.

**Limits.** No semantics: a diagram is checked by people. Confidence is
argued in prose, not computed. Large cases (thousands of nodes) become hard
to maintain and easy to make look convincing.

**For Mitra.** GSN is the vocabulary Mitra's users in industry would know:
claim = goal, `split` = strategy, evidence = solution, `assume` = assumption.
A GSN export would make Mitra legible to safety engineers at low cost.

**Read.** ★ Kelly and Weaver, *The Goal Structuring Notation: A Safety
Argument Notation*, DSN 2004 workshop. ACWG, *GSN Community Standard v3*,
SCSC-141C, 2021. Hawkins, Kelly, Knight, Graydon, *A New Approach to Creating
Clear Safety Arguments*, SSS 2011.

### CAE: Claims, Arguments, Evidence

**What.** Adelard's notation (Bloomfield and others, 1990s). Fewer node types
than GSN. Later work defined five **CAE building blocks**: decomposition,
substitution, concretion, calculation and evidence incorporation, each with a
side-condition that must hold for the step to be valid.

**Strengths.** The building blocks give a step a *meaning*: you can check
whether a decomposition is complete. This is the root of Assurance 2.0.

**Limits.** Same as GSN for confidence: argued, not computed.

**For Mitra.** Mitra's `split` with its explicit `.complete` claim *is* a CAE
decomposition block with its side-condition.

**Read.** Bloomfield and Netkachova, *Building Blocks for Assurance Cases*,
ISSRE workshops 2014.

### SACM: Structured Assurance Case Metamodel

**What.** The OMG standard metamodel (v2.x) that unifies GSN and CAE: claims,
argument reasoning, artefacts, terminology, and the links between them, with
an XML interchange format.

**Strengths.** A machine-readable standard. Distinguishes asserted claims
from assumed and "needs support" claims, and lets relations themselves be
asserted, which is close to "relationships are claims".

**Limits.** Large and abstract; little tooling outside research. No
confidence calculus.

**For Mitra.** If Mitra ever needs an exchange format with the assurance
world, SACM is the one. Its "relations are assertions" design is independent
support for Mitra's choice.

**Read.** OMG, *Structured Assurance Case Metamodel*, v2.2 or later
(omg.org/spec/SACM).

---

## 4. Assurance case tools

### AdvoCATE (NASA Ames)

**What.** An Eclipse-based tool for building GSN cases (Denney, Pai and
colleagues). It has a formal basis for argument structures, patterns that are
instantiated from data, queries and views over large cases, links to hazard
analysis, and metrics on the structure and development status of the case.

**Strengths.** The most formally grounded industrial-strength GSN tool. Used
on real NASA and FAA unmanned-aircraft cases. Shows that *metrics on the
argument* are valued in practice.

**Limits.** Heavy, GUI-based, for safety specialists. Its confidence work
(Denney, Pai, Habli 2011, a Bayesian network) was one of the methods Graydon
and Holloway found could mask contrary evidence.

**Read.** ★ Denney and Pai, *Tool Support for Assurance Case Development*,
Automated Software Engineering 25, 2018. Denney, Pai, Habli, *Towards
Measurement of Confidence in Safety Cases*, ESEM 2011.

### ASCE (Adelard) and NOR-STA (Gdańsk / Argevide)

**What.** Commercial assurance case editors. ASCE supports GSN and CAE and is
the editor used with Assurance 2.0. NOR-STA implements the TRUST-IT method,
where each piece of evidence gets an assessment on Jøsang's opinion triangle
(belief, disbelief, uncertainty) and these are aggregated with
Dempster-Shafer-style rules (Cyra and Górski).

**Strengths.** NOR-STA is the one tool that shipped an opinion-triangle
calculus to industrial users, so it is the nearest *commercial* precedent for
Mitra's value type.

**Limits.** Graydon and Holloway found Cyra and Górski's method sensitive to
an arbitrary choice (how hazards are scoped). Neither tool is aimed at
software developers.

**Read.** Cyra and Górski, *Support for Argument Structures Review and
Assessment*, Reliability Engineering and System Safety 96(1), 2011.

### Isabelle/SACM, Resolute, the Evidential Tool Bus

**What.** Three research tools that tie assurance cases to formal
verification. Isabelle/SACM embeds SACM in the Isabelle proof assistant so
argument steps can be machine-checked. Resolute (Rockwell Collins) writes
assurance rules over AADL architecture models and generates the case from
them. The Evidential Tool Bus (SRI) runs verification tools and records their
results as evidence with provenance.

**Strengths.** They show cases can be *generated* and *checked*, not just
drawn. The Tool Bus is the precedent for "evidence comes from running tools".

**Limits.** Research prototypes; logical, not quantitative; no uncertainty.

**For Mitra.** Mitra's own correctness story (Lean and Verus) is in this
tradition. The Tool Bus's provenance tracking is what Mitra needs to avoid
double-counting evidence ([calculus.md](calculus.md)).

**Read.** Foster, Nemouchi, Gleirscher, Wei, Kelly, Woodcock, *Integration of
Formal Proof into Unified Assurance Cases with Isabelle/SACM*, Formal Aspects
of Computing 33, 2021. Gacek, Backes, Cofer, Slind, Whalen, *Resolute: An
Assurance Case Language for Architecture Models*, HILT 2014. Cruanes, Hamon,
Owre, Shankar, *Tool Integration with the Evidential Tool Bus*, VMCAI 2013.

### DARPA ARCOS: RACK, DesCert, Clarissa

**What.** DARPA's *Automated Rapid Certification Of Software* program
(2019 to 2023) funded tools that gather evidence automatically and assemble
assurance cases from it. RACK (GE) is a curated graph database of evidence
with provenance. DesCert (SRI) builds cases during design. **Clarissa** (SRI,
Adelard/City) is the toolkit for Assurance 2.0: it represents cases in a
logic program (s(CASP)) so that properties of the argument can be checked
automatically and defeaters can be queried.

**Strengths.** The best-funded attempt at exactly "machine evidence in,
assurance out". Clarissa shows a case can be *reasoned over* by a solver.

**Limits.** Aimed at defence certification; large tool stacks; adoption
outside the program is limited. Confidence is still mostly qualitative.

**For Mitra.** Proof that the problem is real and funded, and a warning: the
heavy, everything-integrated approach did not spread to ordinary developers.

**Read.** Varadarajan, Bloomfield, Rushby et al., *CLARISSA: Foundations,
Tools and Automation for Assurance Cases*, DASC 2023. Gupta et al.,
*Automating Semantic Analysis of System Assurance Cases using Goal-directed
ASP*, TPLP 2024 (arXiv:2408.11699). RACK at github.com/ge-high-assurance/RACK.
Rushby et al., *DesCert: Design for Certification*, arXiv:2203.15178.

### Certus (University of Victoria, 2025)

**What.** A domain-specific language for confidence assessment in assurance
cases (Diemert and Weber). Assessors give judgments in words ("high",
"medium") interpreted as **fuzzy sets**, and the language states how
confidence propagates at each step, so the propagation is visible and
inspectable.

**Strengths.** It is the nearest thing to "a confidence language" in the
assurance literature, and it is explicit that *inspectable propagation* is
what earlier methods lacked. A 2026 follow-up compares the effort Certus,
BBN and Dempster-Shafer methods demand as cases grow.

**Limits.** Early: one worked example, no formal verification, aimed at
safety assessors, no machine evidence.

**For Mitra.** Direct prior art for the *language* idea. Mitra differs by
grounding leaves in evidence counts rather than linguistic labels, by being
formally verified, and by its users (developers and agents).

**Read.** ★ Diemert and Weber, *Certus: A Domain Specific Language for
Confidence Assessment in Assurance Cases*, arXiv:2505.01894 (2025). Diemert
and Weber, *A Scalability Analysis of Quantitative Confidence Assessment
Methods for Assurance Cases*, arXiv:2606.15480 (2026).

---

## 5. Assurance 2.0

**What.** A program by Robin Bloomfield (City St George's) and John Rushby
(SRI) to modernise assurance cases. Its main ideas:

- **Narrow argument steps.** Each step is a CAE building block with a
  side-condition, so the argument can be checked *deductively*: if the
  sub-claims hold, the claim holds.
- **Defeaters.** Doubts are written into the case as defeaters ("the test
  oracle may be wrong") and each must be defeated by further argument or
  accepted as a residual risk. This is *eliminative* reasoning: confidence
  grows as doubts are eliminated (Goodenough, Weinstock and Klein, after
  Bacon).
- **Indefeasible confidence.** The goal is a case where no credible new
  information would change the decision. Confidence is judged from four
  angles: logical soundness, probabilistic assessment, dialectical
  examination (actively looking for defeaters), and residual risk.
- **Probabilities last.** Their 2025 report proposes a probabilistic
  confidence method, but only *after* the argument is sound and all defeaters
  are dealt with. It propagates confidence according to how sub-claims
  relate: **diverse** (independent ways to eliminate the same doubt: doubts
  multiply), **partitioned** (sub-claims cover disjoint parts: weighted sum),
  **containment** (one inside the other: take the maximum) and **cumulative**
  (chain rule). Bayesian networks are kept for genuinely tangled
  dependencies. They drop their earlier "sum of doubts" rule as too
  conservative for general use.

**Strengths.** The most thoughtful current position, and it directly answers
the Graydon and Holloway critique. Its four propagation patterns are a
practical recipe for *when* each operator applies, which subjective logic
does not give you.

**Limits.** Demanding: it assumes a skilled assessor and a lot of time.
Numbers are secondary and elicited, not computed from evidence.

**For Mitra.** The most important thing to read. Three direct lessons:

1. Mitra's `unless` is a defeater; making defeaters first-class (each one
   must be resolved or listed as residual) would match the state of the art.
2. The diverse / partitioned / containment / cumulative distinction is a
   better guide than "always `and`, always fuse". `split` is *partitioned*:
   a weighted sum by how much of the input space each part covers, not a
   product.
3. "Numbers are supplementary, soundness first" is a positioning Mitra can
   adopt: the diagnostics come first, the score second.

**Read.** ★ Bloomfield and Rushby, *Assurance 2.0: A Manifesto*,
arXiv:2004.10474 (2020). *Assessing Confidence with Assurance 2.0*,
arXiv:2205.04522 (2022). *Defeaters and Eliminative Argumentation in
Assurance 2.0*, arXiv:2405.15800 (2024). *Confidence in Assurance 2.0 Cases*,
arXiv:2409.10665 (2024). ★ *Quantifying Confidence in Assurance 2.0
Arguments*, SRI-CSL-25-01R2 (2025). Goodenough, Weinstock, Klein,
*Eliminative Argumentation*, CMU/SEI-2015-TN-005.

---

## 6. Quantified confidence

Short cards; [calculus.md](calculus.md) compares the mathematics.

| Approach | Key work | Value type | Main known problem |
|---|---|---|---|
| **Bayesian networks** | Denney, Pai, Habli 2011; Hobbs and Lloyd 2012; Zhao et al. 2012 | probability, with conditional tables | Needs many conditional probabilities nobody knows; can mask contrary evidence |
| **Dempster-Shafer** | Ayoub et al. 2013; Wang, Guiochet, Motet, Schön 2019; Idmessaoud, Dubois, Guiochet 2024 | belief / plausibility interval | Dempster's rule behaves badly under conflict; results sensitive to how the frame is chosen |
| **Subjective logic / Beta** | Cyra and Górski 2011; Duan et al. 2015; Herd et al. 2026 | opinion (b, d, u, a) = Beta distribution | Independence assumption in fusion; approximate operators; can mask contrary evidence |
| **Conservative Bayesian inference** | Bishop, Bloomfield, Littlewood et al. 2011; Zhao et al. 2018 | worst-case bound on a failure probability | Narrow: answers reliability claims only, but answers them rigorously |
| **Eliminative / Baconian** | Goodenough, Weinstock, Klein 2013 to 2015 | count of eliminated doubts | Not a probability; does not combine across claims |
| **Fuzzy / linguistic** | Certus (Diemert and Weber 2025) | fuzzy set over "low … high" | Little validation so far |

### The critique everyone cites

**What.** Graydon and Holloway (NASA Langley) took 12 published methods and
tried to break each with a counterexample built from its own rules. For 9
they found one. The recurring failures were **masking** (missing or contrary
evidence barely moves the result), **scope sensitivity** (an arbitrary
modelling choice changes the answer a lot), and outright implausibility (one
BBN gives 86% confidence in safety while trusting the hazard analysis at
0.1%). None of the methods had been validated by experiment.

**For Mitra.** Mitra will be judged against this paper. Two consequences:
build its counterexamples into the test corpus (each must produce a visible
conflict or uncertainty, not a confident score), and validate empirically,
which is what the `add` ladder in [validation.md](../validation.md) does.

**Read.** ★ Graydon and Holloway, NASA/TM-2016-219195 (2016), and the short
version at SAFECOMP 2017.

### Subjective logic for GSN (Herd et al., 2026)

**What.** A compositional semantics for GSN in subjective logic: goals,
evidence and assumptions are opinions; support links are SL *deduction* with
conditional opinions as explicit warrants; conjunction is multiplication,
disjunction comultiplication, redundant support cumulative fusion; consumed
assumptions are tracked for provenance.

**For Mitra.** This is close to Mitra's draft 2. Read it before going
further: it settles some choices (deduction rather than discounting for
support links) and states the same open problems Mitra has (independence of
evidence, how many conditional opinions users must give).

**Read.** ★ arXiv:2605.22213 (2026). Duan, Rayadurgam, Heimdahl et al.,
*Representing Confidence in Assurance Case Evidence*, SAFECOMP 2015 workshops,
and the survey *Reasoning about Confidence and Uncertainty in Assurance
Cases*, 2017.

### Conservative Bayesian inference (City, Bishop, Littlewood, Strigini)

**What.** Instead of assuming a prior, ask: over *all* priors consistent with
what we actually believe (say "we are 90% sure the failure rate is below
10⁻³"), what is the *worst* posterior after n failure-free runs? This gives
claims that are safe to rely on. A key idea is the **probability of
perfection**: formal verification does not make failures rare, it makes it
likely that there are *none*. Littlewood and Rushby used this to show how a
"possibly perfect" verified channel combines with a tested one.

**For Mitra.** This is the rigorous version of what the `add` ladder wants:
usage evidence bounds a failure rate; a proof raises the probability of
perfection; the two combine by a known formula. See
[calculus.md](calculus.md#usage-evidence-what-n-runs-can-show).

**Read.** ★ Littlewood and Strigini, CACM 1993. Butler and Finelli, *The
Infeasibility of Quantifying the Reliability of Life-Critical Real-Time
Software*, IEEE TSE 19(1), 1993. Bishop, Bloomfield, Littlewood, Povyakalo,
Wright, *Toward a Formalism for Conservative Claims about the Dependability
of Software-Based Systems*, IEEE TSE 37(5), 2011. Littlewood and Rushby,
*Reasoning about the Reliability of Diverse Two-Channel Systems in which One
Channel is "Possibly Perfect"*, IEEE TSE 38(5), 2012. Zhao, Littlewood,
Povyakalo, Strigini, Wright, *Conservative Claims for the Probability of
Perfection of a Software-Based System Using Operational Experience of
Previous Similar Systems*, RESS 175, 2018.

---

## 7. AI systems and agents

### Safety cases for frontier AI

**What.** Since 2024, AI labs and the UK AI Security Institute have adapted
safety cases to AI models. Clymer et al. sort arguments into *inability* (the
model cannot do harm), *control* (it cannot get away with harm under
monitoring), *trustworthiness* and *deference*. AISI's *control safety case
sketch* (Korbak et al., 2025) argues that a red team's measured attack
success, under the deployment's monitors, bounds the real risk. Balesni et
al. sketch cases against scheming from capability evaluations. "Dynamic
safety cases" update as monitoring data arrives.

**Strengths.** The frontier of the field, and directly about trusting agents.
The control argument is quantitative: evidence (red-team runs) bounds a rate
(undetected attack), conservatively.

**Limits.** Written for whole AI systems at lab scale, in prose. No
executable form, no per-action granularity.

**For Mitra.** UC3 (agent autonomy gate) is a *per-task* control case: "this
agent's output is acceptable because its claims were checked by these
monitors". Mitra could be the executable notation such cases lack.

**Read.** ★ Clymer, Gabrieli, Krueger, Larsen, *Safety Cases: How to Justify
the Safety of Advanced AI Systems*, arXiv:2403.10462 (2024). Korbak et al.,
*A Sketch of an AI Control Safety Case*, arXiv:2501.17315 (2025). Balesni et
al., *Towards Evaluations-Based Safety Cases for AI Scheming*,
arXiv:2411.03336 (2024). Dalrymple et al., *Towards Guaranteed Safe AI*,
arXiv:2405.06624 (2024): world model + specification + verifier, the
proof-first view.

### ML components: AMLAS

**What.** York's *Assurance of Machine Learning for use in Autonomous
Systems*: GSN patterns for the ML lifecycle (data, training, verification,
deployment).

**For Mitra.** A source of ready-made claim structures; the data-related
claims ("the training data is representative") are typical assumptions.

**Read.** Hawkins et al., *Guidance on the Assurance of Machine Learning in
Autonomous Systems (AMLAS)*, University of York, 2021.

### LLMs inside assurance

**What.** Recent work uses LLMs to *write* assurance cases and to *find
defeaters* in them. GPT-4-class models find many plausible defeaters but miss
or misclassify others, so they work as a reviewer, not an authority. A 2025
taxonomy of real-world defeaters and 2026 "defeater cards" give structure to
what to look for.

**For Mitra.** Supports the "LLM-friendly language" bet: an agent proposing
claims, splits and defeaters in a small notation, with Mitra computing the
consequences, is a division of labour this literature points to.

**Read.** Shahandashti et al., *Evaluating the Effectiveness of GPT-4 Turbo
in Creating Defeaters for Assurance Cases*, FORGE 2024 (arXiv:2401.17991).
*A Taxonomy of Real-World Defeaters in Safety Assurance Cases*,
arXiv:2502.00238 (2025). *Defeater Cards*, arXiv:2606.11462 (2026).

### Agents that write verified code

**What.** "Vericoding" benchmarks ask LLMs to produce code *with* proofs in
Lean, Verus or Dafny. Two lessons recur. Agents **cheat** when they can:
`assume(false)`, `sorry`, `axiom`, `native_decide`, hollow instances,
separating the proven function from the one that runs. And **specifications
are often too weak**: in one benchmark about 9% of specs admitted trivial
solutions and 15% were poorly translated. Benchmarks defend with axiom
allowlists, frozen specs and LLM judges.

**Strengths.** Shows the market Mitra's wedge targets is real and growing:
code, tests and proofs written by agents, in Verus and Lean.

**For Mitra.** "Proven" from an agent means "proven under these assumptions,
against this spec". Both are exactly what Mitra represents (`assume` claims,
a `spec_ok` claim on the `via` route). See [wedge.md](wedge.md).

**Read.** ★ Bursuc et al., *A Benchmark for Vericoding: Formally Verified
Program Synthesis*, arXiv:2509.22908 (2025). Ye et al., *Vero: Can AI Agents
Build Formally Verified Software Repositories?*, arXiv:2608.13522 (2026).
Yang et al., *AutoVerus: Automated Proof Generation for Rust Code*,
arXiv:2409.13082 (2024).

---

## 8. Summary: who does what

| | Links claims to evidence | Weighs evidence | Shows uncertainty | Localizes conflicts | Machine evidence in | Text in repo | Formally verified engine | For developers / agents |
|---|---|---|---|---|---|---|---|---|
| DOORS, Polarion | ✓ | | | suspect links | partial | | | |
| LDRA, VectorCAST | ✓ | coverage only | | | ✓ | | | |
| StrictDoc, Sphinx-Needs, TRLC/LOBSTER | ✓ | | | | ✓ | ✓ | | partly |
| GSN / CAE / SACM tools (AdvoCATE, ASCE) | ✓ | by hand | by hand | | | | | |
| NOR-STA | ✓ | ✓ (opinions) | ✓ | | | | | |
| Clarissa (Assurance 2.0) | ✓ | ✓ (2025 method) | defeaters | defeaters | partial | | | |
| Certus | ✓ | ✓ (fuzzy) | ✓ | | | ✓ | | |
| Herd et al. (SL for GSN) | ✓ | ✓ (SL) | ✓ | | | | | |
| AI safety cases | ✓ | some | some | | | | | prose |
| **Mitra (aim)** | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ | ✓ |

No one combines the last four columns. That is the space; [wedge.md](wedge.md)
narrows it to something one person can build.
