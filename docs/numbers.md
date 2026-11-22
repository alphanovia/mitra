# Numbers on core2

`src/numbers.rs` says how much the evidence says, after judging and
never in it: judgement and its verdicts do not change. Subjective Logic
(Jøsang, 2016) on counts.

```rust
pub struct Settings { pub base_rate: f64, pub prior_weight: f64, pub voices: BTreeMap<String, f64> }  // 0.5, 2, 1 each
pub struct Belief { pub belief: f64, pub disbelief: f64, pub uncertainty: f64, pub expected: f64 }
pub fn fused(takings, settings) -> Belief
pub struct Numbers<'e>   // Numbers::of(explanation, settings)
    .prop(p) -> Belief, .strongest_rests(p), .rest(r) -> f64
    .taking_doubt(t), .doubt(suspect), .diagnosis_doubt(d), .rank(blame) -> Vec<(Diagnosis, f64)>
```

- **One proposition, from its takings**: each count weighted by its
  voice (`Settings::voices`, 1 when not said), `r` for and `s` against:
  `b = r/(r+s+W)`, `d = s/(r+s+W)`, `u = W/(r+s+W)`, expected `b + a·u`.
  What only the model says counts 0: `u = 1`, expected `a`.
- **Any proposition, from its rests**: a rest is as strong as the product
  of its takings (a taking's proposition's belief, or its disbelief when
  taken false); the proposition's belief is its strongest rest for, its
  disbelief its strongest rest against; above 1 together, each in
  proportion. One operation for every rule of judgement, read from what
  they gave.
- **Doubt**: a taking is wrong with `1 −` the chance its proposition is as
  it says, every taking of it fused; a measurement or a review with its
  most doubtful taking; a diagnosis with the product of its suspects'.
  `rank` puts the most doubtful diagnosis first.

On div: one test at `(6, 2)` gives `correct(div, (6, 2))` a belief of ⅓
and an expected ⅔. A failed run at `(7, 2)` is doubted ⅓; with 97 runs
there saying 3, the one that saw 4 is doubted 0.98 and blamed first.

## Your choices, flagged (say no to undo)

1. **The product along a rest reads takings as independent.** Two rests
   sharing takings are not added (only the strongest counts), so nothing
   is counted twice across rests, but within a rest correlated takings
   are.
2. **A conjunction of many uncertain points is uncertain**: the all of
   div's 100 points by its cases, one run each, has belief (⅓)¹⁰⁰. This is
   Subjective Logic's conjunction. A large all needs many runs a point, a
   smaller prior weight, or another operation for cases: your call.
3. **A measurement's doubt is its weakest taking's**, not the product:
   what one run says at once is not independent.
4. **Counts are only on takings**; a rule explain writes counts one run
   of the checker.
