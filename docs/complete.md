# Trials: completeness, and how to choose the weight

Both on the two foundation examples, the one-throw toy and the
gravity sweep, at weight 0.2. Printed by
`cargo run --release --example complete` and `--example weight`.
Nothing here is in the core yet: the pieces are written by hand in
each model.

## Completeness

For each tie, one more piece, `complete`: "the statements on this tie
are all that matter for it". A prediction on a tie is all of
[its statements..., `complete`]. An opinion is written (yes, no) and
what it reads at.

**Reads right.**

- The feather, with a scale that says yes about its mass and
  completeness starting at one half: the mass stays at 0.68 and
  `complete feather` goes to 0.03. Gravity stays at 1. That is "mass
  and gravity are right, something is missing for this object".
- The objects that pass get a yes on their completeness with each
  measurement: 0.99 after ten.

**Reads odd.**

1. **The scale must speak of the mass itself.** If weighing is one
   more prediction on the tie, all of [mass, complete], its yes goes
   to both, and the failed falls are shared equally again: 0.35 each.
   So the rule "a prediction on a tie is all of its statements and
   its completeness" cannot hold for every observation, or evidence
   about one statement alone must be allowed beside it.
2. **Without the scale nothing tells the mass from completeness**:
   0.12 each. Only their starting rates do. With completeness starting
   at 0.9 the mass takes the blame (0.04) and completeness reads 0.54;
   with the scale too, the ten failures still weigh more than one
   weighing: mass 0.36, completeness 0.20.
3. **It does not lower gravity when 8 of 10 objects fail**: 0.99, and
   0.96 with 9 of 10. Each failure is put on that object's own mass
   and completeness. A completeness per tie is one piece per object,
   so nothing adds up across objects. What would add up is one piece
   shared by all of them.
4. **In the one-throw toy, `complete` says nearly what the tie says**
   ("this model is a good model of this object"), and computes exactly
   like it. With both in the pair there would be three pieces a throw
   cannot tell apart.

### 1. One throw: `lands` is all of [falls, complete]

| complete starts at | senses | I saw | `falls` | `complete` | `throw` |
|---|---|---|---|---|---|
| 0.5 | 0.9 | fall | (0.90, 0.00) 0.91 | (0.90, 0.00) 0.91 | (0.00, 0.00) 0.90 |
| 0.5 | 0.9 | no fall | (0.00, 0.06) 0.38 | (0.00, 0.06) 0.38 | (0.00, 0.01) 0.88 |
| 0.5 | 1 | no fall | (0.00, 0.08) 0.36 | (0.00, 0.08) 0.36 | (0.00, 0.00) 1.00 |
| 0.9 | 0.9 | fall | (0.90, 0.00) 0.91 | (0.90, 0.00) 0.98 | (0.00, 0.00) 0.90 |
| 0.9 | 0.9 | no fall | (0.00, 0.26) 0.22 | (0.00, 0.01) 0.84 | (0.00, 0.01) 0.84 |
| 0.9 | 1 | no fall | (0.00, 0.43) 0.16 | (0.00, 0.02) 0.83 | (0.00, 0.00) 1.00 |

### 2. The feather, completeness starting at 0.5

10 objects pass, the feather fails, 10 measurements each.

| the scale | `gravity` | `mass feather` | `complete feather` | `mass o0` | `complete o0` |
|---|---|---|---|---|---|
| no scale | (90.00, 0.00) 1.00 | (0.00, 0.64) 0.12 | (0.00, 0.64) 0.12 | (9.00, 0.00) 0.99 | (9.00, 0.00) 0.99 |
| says yes about the mass | (90.00, 0.01) 1.00 | (0.90, 0.37) 0.68 | (0.00, 2.72) 0.03 | (9.00, 0.00) 0.99 | (9.00, 0.00) 0.99 |
| is a prediction on the tie | (90.00, 0.03) 1.00 | (0.90, 1.76) 0.35 | (0.90, 1.76) 0.35 | (9.00, 0.00) 0.99 | (9.00, 0.00) 0.99 |

### 2. The feather, completeness starting at 0.9

10 objects pass, the feather fails, 10 measurements each.

| the scale | `gravity` | `mass feather` | `complete feather` | `mass o0` | `complete o0` |
|---|---|---|---|---|---|
| no scale | (90.00, 0.01) 1.00 | (0.00, 2.61) 0.04 | (0.00, 0.13) 0.54 | (9.00, 0.00) 0.99 | (9.00, 0.00) 1.00 |
| says yes about the mass | (90.00, 0.03) 1.00 | (0.90, 1.68) 0.36 | (0.00, 0.69) 0.20 | (9.00, 0.00) 0.99 | (9.00, 0.00) 1.00 |
| is a prediction on the tie | (90.00, 0.04) 1.00 | (0.90, 2.64) 0.27 | (0.90, 0.41) 0.72 | (9.00, 0.00) 0.99 | (9.00, 0.00) 1.00 |

### 3. How many of 10 objects fail (10 measurements each, completeness at 0.5)

| objects that fail | `gravity` | `mass` of one that fails | `complete` of one that fails | `complete` of one that passes |
|---|---|---|---|---|
| 0 | (90.00, 0.00) 1.00 | none | none | (9.00, 0.00) 0.99 |
| 1 | (81.00, 0.00) 1.00 | (0.00, 0.64) 0.12 | (0.00, 0.64) 0.12 | (9.00, 0.00) 0.99 |
| 2 | (72.00, 0.01) 1.00 | (0.00, 0.64) 0.12 | (0.00, 0.64) 0.12 | (9.00, 0.00) 0.99 |
| 5 | (45.00, 0.03) 1.00 | (0.00, 0.64) 0.12 | (0.00, 0.64) 0.12 | (9.00, 0.00) 0.99 |
| 8 | (18.00, 0.12) 0.99 | (0.00, 0.63) 0.12 | (0.00, 0.63) 0.12 | (9.00, 0.00) 0.99 |
| 9 | (9.00, 0.26) 0.96 | (0.00, 0.63) 0.12 | (0.00, 0.63) 0.12 | (9.00, 0.00) 0.99 |
| 10 | (0.00, 2.36) 0.04 | (0.00, 0.24) 0.23 | (0.00, 0.24) 0.23 | none |

## A weight that grows with the evidence

```text
W = 0.2 + 0.2 · ln(1 + pieces of evidence in the model)
```

0.34 with one piece of evidence, 1.1 with a hundred, 2 with ten
thousand. Each cell below is the fixed weight, then the scaled one.

**The fixed weight reads better on both examples.**

- With little evidence the two are close: one throw gives 0.92
  against 0.87.
- With much evidence the scaled weight slows down pieces that the
  evidence is not about. A new object measured once reads 0.91 in a
  small model and 0.72 in a model of a hundred objects, only because
  the model is big. Being nuanced where much is known is already what
  counts do: a piece with 900 yes is not moved by one no, whatever
  the weight.
- It blurs the noisy objects: the one failing measurement of an
  object that passes 19 of 20 reads 0.64 instead of 0.19.
- It helps in one place: with 8 of 10 objects failing, gravity reads
  0.89 instead of 0.98, and 0.70 instead of 0.92 with 9 of 10.

**Not tried: a prior and weight per kind of piece, fitted from the
pieces of that kind already judged.** It is not cheap here: a piece
has no kind in the core, and the weight is one number. It is the
version that matches "nuanced later": the weight would grow for
masses because many masses were judged, and not for a piece of a new
kind.

Each cell: fixed weight 0.2 / scaled weight.

### 1. One throw (one piece of evidence: W = 0.34)

| senses | I saw | tie and statement | `throw` |
|---|---|---|---|
| 1 | fall | 0.92 / 0.87 | 1.00 / 1.00 |
| 0.9 | fall | 0.91 / 0.86 | 0.90 / 0.90 |
| 0.5 | fall | 0.86 / 0.80 | 0.50 / 0.50 |
| 1 | no fall | 0.36 / 0.38 | 1.00 / 1.00 |
| 0.9 | no fall | 0.38 / 0.39 | 0.88 / 0.88 |

### 2. Objects that pass and one that never does

| objects that pass | measurements each | W scaled | `gravity` | mass of one that passes | mass of the one that fails |
|---|---|---|---|---|---|
| 1 | 1 | 0.42 | 0.88 / 0.80 | 0.91 / 0.84 | 0.21 / 0.29 |
| 10 | 1 | 0.70 | 0.99 / 0.96 | 0.91 / 0.78 | 0.17 / 0.27 |
| 100 | 1 | 1.12 | 1.00 / 0.99 | 0.91 / 0.72 | 0.16 / 0.31 |
| 1 | 10 | 0.81 | 0.98 / 0.93 | 0.99 / 0.96 | 0.02 / 0.06 |
| 10 | 10 | 1.14 | 1.00 / 0.99 | 0.99 / 0.94 | 0.02 / 0.07 |
| 100 | 10 | 1.58 | 1.00 / 1.00 | 0.99 / 0.93 | 0.02 / 0.09 |
| 100 | 100 | 2.04 | 1.00 / 1.00 | 1.00 / 0.99 | 0.00 / 0.01 |

### 3. Noisy objects, 20 measurements each

W scaled: 1.40

| piece | reads at |
|---|---|
| mass of one that passes 19 of 20 | 0.99 / 0.95 |
| its failing measurement | 0.19 / 0.64 |
| mass of one that passes 12 of 20 | 0.94 / 0.78 |
| one of its failing measurements | 0.22 / 0.67 |
| mass of the feather, 0 of 20 | 0.01 / 0.04 |
| one of the feather's measurements | 0.83 / 0.87 |

### 4. How many of 10 objects fail (10 measurements each: W = 1.12)

| objects that fail | `gravity` | mass of one that fails |
|---|---|---|
| 1 | 1.00 / 0.99 | 0.02 / 0.07 |
| 5 | 1.00 / 0.98 | 0.02 / 0.07 |
| 8 | 0.98 / 0.89 | 0.02 / 0.07 |
| 9 | 0.92 / 0.70 | 0.02 / 0.08 |
| 10 | 0.02 / 0.03 | 0.12 / 0.19 |

## Two passes that choose the weight

"Collapse as much as possible each time, while every evidence keeps
an influence."

**Neither pass reads better than the fixed 0.2, and the two wishes
pull the weight in opposite directions.**

- **(a) Most likely weight.** With one throw it picks the largest
  weight of the grid, 20, and nothing collapses (0.52): one
  observation says nothing about how sharp pieces are. With many
  pieces that are each all yes or all no it picks the smallest, 0.02,
  and everything collapses. So it is cautious at the start and sharp
  at the end, the reverse of what was asked for the start. It is also
  the only one that would notice a model where pieces are really
  mixed, and none of these examples is one.
- **(b) Every evidence moves a reading by at least 0.01.** With one
  measurement per piece the smallest weight does it. With ten
  measurements on a piece it needs a weight of 5: the tenth yes can
  only move a reading if the first nine have not settled it. Then the
  collapse is gone: a mass with ten yes reads 0.82, and the failing
  measurement of a good object is no longer blamed (0.87).
- A reading that has collapsed cannot be moved by one more agreeing
  observation, whatever the weight. But **in counts every piece of
  evidence always has its influence**: it adds its count, and a later
  no weighs against all of them. A way to have both wishes is to keep
  the weight small for the reading and to judge influence on the
  counts, not on the reading.

(a) the weight under which the counts are the most likely; (b) the smallest weight under which every piece of evidence moves a reading by at least 0.01. Both over the grid [0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0].

| case | piece | fixed 0.2 | scaled | (a) most likely | (b) influence |
|---|---|---|---|---|---|
| **one throw, it falls** | the weight | 0.2 | 0.34 | 20 | 0.02 (least move 0.489) |
| | `applies` | 0.91 | 0.86 | 0.52 | 0.99 |
| **one throw, it does not** | the weight | 0.2 | 0.34 | 20 | 0.02 (least move 0.142) |
| | `applies` | 0.38 | 0.39 | 0.49 | 0.36 |
| **10 objects pass once, one fails** | the weight | 0.2 | 0.70 | 0.1 | 0.02 (least move 0.400) |
| | `gravity` | 0.99 | 0.96 | 0.99 | 1.00 |
| | `mass o0` | 0.91 | 0.78 | 0.95 | 0.99 |
| | `mass f0` | 0.17 | 0.27 | 0.13 | 0.10 |
| **3 objects pass 10 times, one fails** | the weight | 0.2 | 0.94 | 0.02 | 5 (least move 0.012) |
| | `gravity` | 1.00 | 0.98 | 1.00 | 0.91 |
| | `mass o0` | 0.99 | 0.95 | 1.00 | 0.82 |
| | `mass f0` | 0.02 | 0.06 | 0.01 | 0.21 |
| **2 of 10 objects pass, 5 times each** | the weight | 0.2 | 0.99 | 0.5 | 0.5 (least move 0.016) |
| | `gravity` | 0.96 | 0.83 | 0.90 | 0.90 |
| | `mass o0` | 0.98 | 0.91 | 0.95 | 0.95 |
| | `mass f0` | 0.05 | 0.12 | 0.08 | 0.08 |
| **noisy: 9 of 10, 6 of 10, 0 of 10** | the weight | 0.2 | 0.99 | 0.02 | 5 (least move 0.013) |
| | `mass o0` | 0.98 | 0.92 | 1.00 | 0.78 |
| | `drop o0 9` | 0.25 | 0.64 | 0.03 | 0.87 |
| | `mass odd` | 0.91 | 0.76 | 0.99 | 0.63 |
| | `drop odd 9` | 0.28 | 0.68 | 0.04 | 0.87 |
| | `mass feather` | 0.02 | 0.07 | 0.01 | 0.21 |

## Bounds

Two bounds come with counts, and both say when a reading can be
relied on.

- **Per piece: where it is right nine times out of ten.** One throw
  that falls reads 0.91, and the interval is 0.40 to 1.00: the reading
  collapsed, what is known did not. After ten measurements a mass
  reads 0.99, from 0.94 to 1.00. The interval is what a small weight
  hides, and it costs nothing more than the counts.
- **For the fit: the weights the counts allow.** With one measurement
  per piece every weight from 0.01 to 50 is allowed: the counts say
  nothing about the weight, and words flip inside the range (a mass
  holds at one end and is unverified at the other). With ten
  measurements per piece the range closes to 0.01 to 0.5, and no word
  flips in it. The fit bounds the weight from above once pieces have
  been measured several times, and 0.2 is inside every range found.
- A word that flips inside the range is a word that depends on the
  weight and not on the evidence. That is a thing a build could say.

**Not done: `w` in the opinion.** (r, s, a, w) with a global default
works with every operator as it is: each piece reads with its own
weight, and the failure rule already takes each suspect as its own
Beta. It changes the record of the core and every claim written on
it, and it asks where a piece gets its weight: the list of base rates
would hold (piece, a, w). I have left it for your word.

For each piece: what it reads at under 0.2 with the interval where it is right nine times out of ten; then the weights the counts allow (within 2 of the best log-likelihood, over 40 weights from 0.01 to 50), what the piece reads at both ends, and its word at policy 0.9.

| case | piece | at 0.2, nine times out of ten | weights allowed | reads at both ends | word at both ends |
|---|---|---|---|---|---|
| **one throw, it falls** | `applies` | 0.91, from 0.40 to 1.00 | 0.01 to 50.00, best 50.00 | 0.99 and 0.51 | holds and unverified **flips** |
| **one throw, it does not** | `applies` | 0.38, from 0.00 to 1.00 | 0.01 to 50.00, best 50.00 | 0.36 and 0.50 | fails and fails |
| **10 objects pass once, one fails** | `gravity` | 0.99, from 0.93 to 1.00 | 0.01 to 7.00, best 0.07 | 1.00 and 0.77 | holds and unverified **flips** |
|  | `mass o0` | 0.91, from 0.40 to 1.00 | 0.01 to 7.00, best 0.07 | 0.99 and 0.56 | holds and unverified **flips** |
|  | `mass f0` | 0.17, from 0.00 to 0.92 | 0.01 to 7.00, best 0.07 | 0.10 and 0.47 | fails and fails |
| **3 objects pass 10 times, one fails** | `gravity` | 1.00, from 0.97 to 1.00 | 0.01 to 0.51, best 0.01 | 1.00 and 0.99 | holds and holds |
|  | `mass o0` | 0.99, from 0.94 to 1.00 | 0.01 to 0.51, best 0.01 | 1.00 and 0.97 | holds and holds |
|  | `mass f0` | 0.02, from 0.00 to 0.14 | 0.01 to 0.51, best 0.01 | 0.01 and 0.04 | fails and fails |
| **2 of 10 objects pass, 5 times each** | `gravity` | 0.96, from 0.82 to 1.00 | 0.01 to 50.00, best 0.51 | 1.00 and 0.49 | holds and fails **flips** |
|  | `mass o0` | 0.98, from 0.87 to 1.00 | 0.01 to 50.00, best 0.51 | 1.00 and 0.54 | holds and unverified **flips** |
|  | `mass f0` | 0.05, from 0.00 to 0.29 | 0.01 to 50.00, best 0.51 | 0.02 and 0.48 | fails and fails |
| **noisy: 9 of 10, 6 of 10, 0 of 10** | `mass o0` | 0.98, from 0.88 to 1.00 | 0.01 to 1.22, best 0.01 | 1.00 and 0.90 | holds and holds |
|  | `drop o0 9` | 0.25, from 0.00 to 0.96 | 0.01 to 1.22, best 0.01 | 0.02 and 0.69 | fails and fails |
|  | `mass odd` | 0.91, from 0.68 to 1.00 | 0.01 to 1.22, best 0.01 | 0.99 and 0.74 | holds and unverified **flips** |
|  | `drop odd 9` | 0.28, from 0.00 to 0.98 | 0.01 to 1.22, best 0.01 | 0.02 and 0.72 | fails and fails |
|  | `mass feather` | 0.02, from 0.00 to 0.14 | 0.01 to 1.22, best 0.01 | 0.01 and 0.08 | fails and fails |
