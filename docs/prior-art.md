# Prior art

Mitra sits between safety assurance cases, uncertain-reasoning calculi and
proof assistants. None of these alone does what Mitra aims to do.

| Work | What it gives | How Mitra relates |
|---|---|---|
| **Assurance cases: GSN, Claims-Arguments-Evidence (CAE)** | Graphical structure for arguing that a system is safe, used in safety-critical industries. | Closest in spirit. Mitra borrows the claim/evidence structure but makes it computable and aims at everyday software. |
| **Assurance 2.0** (Bloomfield and Rushby) | Assurance cases with explicit confidence assessment and defeaters. | The most direct precedent for scoring an argument. Worth reading before SP1. |
| **Subjective logic** (Jøsang) | Opinions as belief, disbelief and uncertainty, with operators for conjunction, fusion and discounting. | Working hypothesis for Mitra's calculus. Its published worked examples are a ready source of reference models with known outputs. |
| **Dempster-Shafer theory** | Belief functions over sets of hypotheses; subjective logic's foundation. | Alternative to evaluate in SP1. |
| **Bayesian networks** | Probabilistic graphical models with well-understood inference. | Alternative to evaluate in SP1. Expressive, but needs priors and conditional tables that users rarely have, and collapses "unknown" into a probability. |
| **Lean 4, Verus** | Proof assistant, and verification of Rust code. | Used to build Mitra itself. Later, a Lean proof is also one of the strongest kinds of evidence a model can contain. |
| **SLSA, in-toto** | Signed attestations about how software was built. | Possible future evidence sources. Out of scope for v1. |
