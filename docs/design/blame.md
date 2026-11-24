# Blame on core2

`crates/mitra-lang/src/blame.rs` answers the third question: which evidence is wrong,
given the model explains the system? It reads what explain gives
(`docs/design/explain.md`).

```rust
pub struct Settings { pub diagnoses: usize, pub tries: usize }   // 10; 100,000 a group
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
  a rule explain wrote). Measurements explain read as one (`same`) are
  one suspect, by the first one's place.
- **Held, not suspected: the model's claims.** "Given the model explains
  the system" (your question 3). Their contradictions are question 2's.
- **A diagnosis** is a smallest set of suspects that meets every
  conflict: dropping it leaves none (Reiter's minimal hitting sets,
  breadth first, so the smallest come first).
- **Groups**: conflicts that share no suspect are apart. Each group's
  diagnoses are found on their own, and a diagnosis of all is one of
  each group's, best first: twelve failed tests of twelve functions give
  their first 10 diagnoses in under a millisecond.
- **Ranked** by size, then by the most trusted kind each drops (flagged),
  then the next: what only the model says, then measurements, reviews,
  the rules explain wrote, and proofs last. The numbers layer ranks by
  doubt instead.
- **Budget**: `Settings::diagnoses` diagnoses, and `Settings::tries` sets
  tried in each group (a set queued counts). Blame looks for one more:
  when there is one, or it stops at the budget, `complete` is false and
  B002 says "the first N diagnoses only".
- **Cases**: an all by cases read one taking a point. A diagnosis that
  drops it and not another that says the same there may leave the
  conflict: B003 says so.

On main's div, a test that saw `div(7, 2) = 4` with one that saw
`div(6, 2) = 3`: the first diagnosis drops the failed test, whole; the
second, the tie of div and the rule `everywhere`, which only the model
says.

## Findings (suit B)

| Code | What |
|---|---|
| B001 | a conflict the model's claims alone hold (a guard: today's explain never makes one) |
| B002 | more diagnoses than blame looked for |
| B003 | a diagnosis a proof by cases may outlive |

## Your choices, flagged (say no to undo)

1. The model's claims are held; everything else can be doubted.
2. A measurement or a review is one suspect, whole.
3. The kind order: model, measurement, review, written rule, proof,
   compared from the most trusted kind a diagnosis drops.
