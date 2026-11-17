# Prior art

Mitra sits between safety assurance cases, uncertain-reasoning calculi and
proof assistants. None of these alone does what Mitra aims to do.

| Work | What it gives | How Mitra relates |
|---|---|---|
| **Assurance cases: GSN, Claims-Arguments-Evidence (CAE)** | Graphical structure for arguing that a system is safe, used in safety-critical industries. | Closest in spirit. Mitra borrows the claim/evidence structure but makes it computable and aims at everyday software. |
| **Assurance 2.0** (Bloomfield and Rushby) | Assurance cases with explicit confidence assessment and defeaters. | The most direct precedent for scoring an argument. Worth reading. |
| **Subjective logic** (Jøsang) | Opinions as belief, disbelief and uncertainty, with operators for conjunction, fusion and discounting. | The calculus Mitra's semantics are defined in (draft 0). Its published worked examples are a ready source of reference models with known outputs. |
| **Dempster-Shafer theory** | Belief functions over sets of hypotheses; subjective logic's foundation. | Considered as an alternative calculus. |
| **Bayesian networks** | Probabilistic graphical models with well-understood inference. | Considered as an alternative calculus. Expressive, but needs priors and conditional tables that users rarely have, and collapses "unknown" into a probability. |
| **Lean 4, Verus** | Proof assistant, and verification of Rust code. | Used to build Mitra itself. Later, a Lean proof is also one of the strongest kinds of evidence a model can contain. |
| **SLSA, in-toto** | Signed attestations about how software was built. | Possible future evidence sources. Not in scope yet. |

For the full survey, critiques of subjective logic and the proposed wedge,
see [research/](research/README.md).
