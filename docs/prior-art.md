# Prior art

Mitra sits between safety assurance cases, uncertain-reasoning calculi and
proof assistants. None of these alone does what Mitra aims to do.

| Work | What it gives | How Mitra relates |
|---|---|---|
| **Assurance cases: GSN, Claims-Arguments-Evidence (CAE)** | Graphical structure for arguing that a system is safe, used in safety-critical industries. | Closest in spirit. Mitra borrows the claim/evidence structure but makes it computable and aims at everyday software. |
| **Assurance 2.0** (Bloomfield and Rushby) | Assurance cases with explicit confidence assessment and defeaters. | The most direct precedent for scoring an argument. Worth reading. |
| **Subjective logic** (Jøsang) | Opinions as belief, disbelief and uncertainty, with operators for conjunction, fusion and discounting. | Considered for the opinion on evidence ([semantics](design/semantics.md)): strong on source trust, weak on contradiction. Its published worked examples are a ready source of reference models with known outputs. |
| **Dempster-Shafer theory** | Belief functions over sets of hypotheses; subjective logic's foundation. | Used to say which evidence or assumption is most plausibly wrong ([semantics](design/semantics.md)); Dempster's and Yager's rules are in `crates/mitra-ds`. |
| **Bayesian networks** | Probabilistic graphical models with well-understood inference. | Considered as an alternative. Expressive, but needs priors and conditional tables that users rarely have, and collapses "unknown" into a probability. |
| **Lean 4, Verus** | Proof assistant, and verification of Rust code. | To verify Mitra itself. A proof is also the strongest kind of evidence a model can contain. Lean 4 inspires Mitra's syntax. |
| **SLSA, in-toto** | Signed attestations about how software was built. | Possible future evidence sources. Not in scope yet. |

For the full survey, critiques of subjective logic and the proposed wedge,
see [research/](research/README.md).
