# Explain on core2

`src/explain.rs` answers the second question: does the model explain
the system, assuming the evidence, and if not, where? It reads a model
`check` accepts and a ledger (`docs/design/evidence.md`); what `evidence::check`
refuses is left out, and reported there.

```rust
pub struct Settings { pub rounds: usize, pub cases: usize, pub rests: usize }   // 64, 1,024, 1,024
pub enum Verdict { Explained, Contradicted { against: Rests }, Unsupported }
pub struct Verdicted { pub at: Path, pub prop: Prop, pub verdict: Verdict }
pub struct Explanation { pub verdicts: Vec<Verdicted>, pub opinions: BTreeMap<Prop, Opinion>,
                         pub sources: BTreeMap<Taking, Source>, pub same: BTreeMap<usize, Vec<usize>>,
                         pub alike: BTreeMap<Taking, Vec<Taking>>,
                         pub findings: Vec<Finding>, pub hints: Vec<Hint> }
pub fn explain(model: &Model, ledger: &Ledger, settings: &Settings) -> Explanation
```

## 1. Takings

| What | Taken | By, count |
|---|---|---|
| each leaf of the model (`leaves`): claims, ties, rules with no proof, requires in words, ensures with no list, the links between them | true | `model`, 0 |
| a rule its proof shows | true | `machine:mitra-check`, 1 |
| a review of a claim, rule, require or ensure | `holds` | its voice, 1 |
| a review of a tied item | each of its ties, `holds` | its voice, 1 |
| a measurement at `x`, where the model gives a value `v` there (a body, or an example, through calls) | `correct(f, x)`: seen, `y == v`; not seen, false when `y == v` | its voice, its count |
| the same, where the model gives no value but `f` has claims | `correct(f, x)`, seen only: false when a claim breaks, true when all hold (a failed test there: X002) | its voice, its count |
| the same, each claim `c` of `f`, seen only | `f::c at x`: `c` read with what the system gave | its voice, its count |

`y` is the value the test states, `x` the arguments of the function it
measured. The world's values a measurement gives as inputs (what a
function with no body gave) are given to the library (`Calls::given`),
read before its examples.

Measurements that say the same (inputs and outputs in any order, seen
or not, voice) are one, their counts added: `same` lists them by the
first one's place. Two that differ elsewhere but say one thing at one
point, by one voice and count, are one taking, read once: X008.

## 2. Rules explain writes

Each is taken by the checker, `machine:mitra-check`, count 1, and can be
blamed like any taking:

- `f at x, by its ties`: the ties of `f` and of every function its body
  calls, however far (a function's own tie or its part's), give
  `correct(f, x)`. With no value to compare, its claims decided: the
  ties of what they call instead.
- `f::c at x, by its ties`: the claim `c` and the ties of `f` and of what
  `c` calls give `f::c at x`.

So a failed test makes its tie, the claim that breaks there, and the
rule explain wrote all suspects, through modus tollens.

## 3. Judging

Every rule (the model's, the links of requires and ensures, those
explain writes) is applied forward (ponens, when every premise follows
true from something) and back (tollens, from a post false and every
premise but one), and each all or any written in the model at the points
opinions are about: down to each (instance), up from each (witness: an
all false, an any true). Until nothing changes, within `Settings::rounds`.

- **A negation is kept with what it negates**: `!p` true is `p` false,
  so a rule that concludes `!p` meets a test that says `p`.
- **A round reads what it began with** and joins what it gives at its
  end: one link of a chain a round. A rule is read again only when the
  side it reads changed (forward, its premises true; back, a post false
  or the other premises true).
- **A rule whose premises rest on more than `Settings::rests` sets
  together is left out** (flagged), with X006 naming it: each way to
  have every premise is one more set the conclusion rests on, and
  judgement keeps the smallest by comparing each with each. An ensure
  from 7 claims, each reviewed by 3 voices, rests on 4⁷ sets.

An all is true, or an any false, by its cases when its ranges hold at
most `Settings::cases` points (its condition is read after: a few points
in large ranges are not listed) and the ledger says so at every one.
**Each point by the first taking that says so** (flagged): the ways to
have every point multiply, so on div's 100 points with two ways each
there would be 2¹⁰⁰. The others at a point are kept in `alike`, and
blame says when a diagnosis may leave the all to them (B003).

## 4. Verdicts

For each proposition of the model, then each a measurement wrote:

- **Contradicted**, with the sets of takings it follows false from;
- **Explained**, when it follows true from a set with nothing by `model`:
  evidence and checks alone;
- **Unsupported**, otherwise: only what the model alone says holds it, or
  nothing.

A claim over all of ℤ or ℚ is never explained by tests: they explain it
at their points (`f::c at x`), and how much that says is the numbers
layer's (corroboration).

## Findings (suit X)

| Code | What |
|---|---|
| X001 | a measurement nothing in the model says what to expect of |
| X002 | a failed test whose expected value the model does not give |
| X003 | a claim a measurement does not decide |
| X004 | an output the model cannot compute from the inputs |
| X005 | an input nothing measured reads |
| X006 | judging stopped before it was done, or a rule left out (`rests`) |
| X007 | hint: an opaque function measured the same each time |
| X008 | two measurements that say one thing, by one voice and count |
| X009 | a budget of steps or depth reached |
| X010 | a value known in part, where only what is not known could tell |
| X011 | a measured value outside the range of its type: a taking against the function there, by its ties |

A hint is below a finding: something that may help.

## Your choices, flagged (say no to undo)

1. **What the model alone says is taken**, by the voice `model`, count 0,
   so that a failed test can blame it (Reiter's "assumed correct"). It
   weighs nothing, and a verdict built on it alone is unsupported.
2. **The ties are premises of what a measurement says of the model**: a
   measurement is through its ties, so they share its blame.
3. **A failed test at a point where the model gives the value it
   expected says the function is not correct there.** Where the model
   gives another value, nothing is known of what the system gave: a
   finding (X002), no taking.
4. **The model gives no value (a fault, say a division by zero) where a
   test saw one**: the function is not correct there.
5. **One taking a point in a proof by cases**, the first.
6. **A review of a proposition is always read**: by its own verdict, so
   "a review nothing uses" is not a finding.
7. **Measurements that say the same are one**, their counts added: the
   same run written twice says nothing new apart, and blame doubts them
   together.
8. **A rule whose premises rest on too many sets is left out**
   (`Settings::rests`, 1,024), said by X006: the kernel's smallest sets
   are found by comparing each with each.

## Not in this round (findings for you)

- What an opaque function was seen to give feeds only the measurement
  that gave it (end-to-end, as an input). Fed across measurements, one
  set at one time, it could stand for the world everywhere.
- An "or" a test cannot split (two ways from A to B) gives no hint: a
  failed test blames every tie it may have gone through.
- A contract's claims, a record's claims and expressions written in a
  rule are not read at a measured point.
- Two measurements at one point under different world values (light at
  2.0, then 3.0) are one point: `correct(f, x)` holds only `f`'s arguments.
- Corroboration: tests explain a claim at their points, never as a whole
  (numbers layer).
- Cost, measured after red team round 8 (release): 300 tests through a
  chain of 50 calls, 30 failing, 0.25 s; an all of 100 points by cases,
  a few ms; a set of 5 points in 10,000 × 10,000 ranges, not listed.
