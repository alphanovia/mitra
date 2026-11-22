# Blame on core2

`src/blame.rs` answers the third question: which evidence is wrong,
given the model explains the system? It reads what explain gives
(`docs/explain.md`).

```rust
pub struct Settings { pub diagnoses: usize, pub tries: usize }   // defaults 10, 100,000
pub enum Suspect { Measurement(usize), Review(usize), Taking(Taking) }
pub struct Diagnosis { pub suspects: BTreeSet<Suspect>, pub drop: BTreeSet<Taking> }
pub struct Blame { pub conflicts: Vec<BTreeSet<Suspect>>, pub diagnoses: Vec<Diagnosis>,
                   pub complete: bool, pub findings: Vec<Finding> }
pub fn blame(model: &Model, explanation: &Explanation, settings: &Settings) -> Blame
```

- **A conflict**: where a proposition follows true from one set of
  takings and false from another, the suspects of both cannot all hold.
  Only the smallest conflicts are kept.
- **A suspect** is a measurement or a review of the ledger, whole (all it
  says is dropped together), or one taking of the model's own (a tie, a
  rule with no proof, a require in words, …) or of the checker (a proof,
  a rule explain wrote).
- **Held, not suspected: the model's claims.** "Given the model explains
  the system" (your question 3). Their contradictions are question 2's.
- **A diagnosis** is a smallest set of suspects that meets every
  conflict: dropping it leaves none (Reiter's minimal hitting sets,
  breadth first, so the smallest come first).
- **Ranked** by size, then by kind (flagged): what only the model says,
  then measurements, reviews, the rules explain wrote, and proofs last.
  The numbers layer ranks by doubt instead.
- **Budget**: `Settings::diagnoses` diagnoses and `Settings::tries` sets
  tried. When blame stops there, `complete` is false and B002 says "the
  first N diagnoses only".

On main's div, a test that saw `div(7, 2) = 4` with one that saw
`div(6, 2) = 3`: the first diagnosis drops the failed test, whole; the
second, the tie of div and the rule `everywhere`, which only the model
says.

## Findings (suit B)

| Code | What |
|---|---|
| B001 | a conflict the model's claims alone hold (a guard: today's explain never makes one) |
| B002 | more diagnoses than blame looked for |

## Your choices, flagged (say no to undo)

1. The model's claims are held; everything else can be doubted.
2. A measurement or a review is one suspect, whole.
3. The kind order: model, measurement, review, written rule, proof.
