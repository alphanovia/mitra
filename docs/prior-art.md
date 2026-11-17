# Prior art

Mitra sits between safety assurance cases, uncertain-reasoning calculi and
proof assistants. None of these alone does what Mitra aims to do.

| Work | What it gives | How Mitra relates |
|---|---|---|
| **Assurance cases: GSN, Claims-Arguments-Evidence (CAE)** | Graphical structure for arguing that a system is safe, used in safety-critical industries. | Closest in spirit. Mitra borrows the claim/evidence structure but makes it computable and aims at everyday software. |
| **Assurance 2.0** (Bloomfield and Rushby) | Assurance cases with explicit confidence assessment and defeaters. | The most direct precedent for scoring an argument. Worth reading. |
| **Subjective logic** (Jøsang) | Opinions as belief, disbelief and uncertainty, with operators for conjunction, fusion and discounting. | A candidate calculus, scored against Mitra's axioms in [calculus.md](design/calculus.md): strong on source trust, weak on contradiction. Its published worked examples are a ready source of reference models with known outputs. |
| **Dempster-Shafer theory** | Belief functions over sets of hypotheses; subjective logic's foundation. | A candidate calculus, scored in [calculus.md](design/calculus.md). Yager's variant is the current prototype (`crates/mitra-ds`). |
| **Bayesian networks** | Probabilistic graphical models with well-understood inference. | Considered as an alternative calculus. Expressive, but needs priors and conditional tables that users rarely have, and collapses "unknown" into a probability. |
| **Lean 4, Verus** | Proof assistant, and verification of Rust code. | To verify Mitra itself. A proof is also the strongest kind of evidence a model can contain. Lean 4 inspires Mitra's syntax. |
| **SLSA, in-toto** | Signed attestations about how software was built. | Possible future evidence sources. Not in scope yet. |

For the full survey, critiques of subjective logic and the proposed wedge,
see [research/](research/README.md).
