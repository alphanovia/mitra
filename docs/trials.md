# Trials: completeness, the weight, and reviews

Both on the two foundation examples, the one-throw toy and the
gravity sweep, at weight 0.2. Printed by
`cargo run --release --example complete`, `--example weight` and
`--example review`.
The tables are as recorded when each trial ran. Since increment 105
the core holds what was kept: the weight given by the model,
completeness implied per tie, a review as evidence with a count, and
a failure that doubts what vouched for its parts. That last rule is
now always on, so the review tables read a little differently when
run today, and the columns "without the rule" are history.

Since increment 108 that rule is gone again: the core judges each
voice instead (the last section, "Voices, iterated", now in
`model::judged`), and `examples/voice.rs` prints the same tables from
the core.

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

## Reviews in the mix

No weight per piece. What an author knows from experience is said as
evidence: a review of a statement or of a tie, signed by a voice,
that carries a count. "Centuries of knowledge about gravity" is a
review of 1000 yes; "I checked the environment" is a review of the
tie. It counts as far as its reviewer is believed. Nothing changes in
the core for this: evidence already says counts.

**Reads right.**

- **An unreviewed piece still collapses on its first evidence**: 0.91
  after one throw, and a new object measured once reads 0.91 beside a
  gravity that holds a thousand yes.
- **A reviewed piece stands, and the no goes to what was not
  reviewed.** One throw that does not fall, the statement reviewed:
  the statement stays at 0.99 and the tie goes to 0.17. Both reviewed:
  both stay at 0.98 and the throw itself is doubted, 0.29.
- **Gravity reviewed stands against failures**: 1.00 with one, three
  or ten objects failing every time, where unreviewed it fell to 0.12,
  0.05 and 0.02. The failing objects' own pieces take the no.
- **The interval follows.** One throw: 0.91, right nine times out of
  ten between 0.40 and 1.00. Reviewed with 10 yes: 0.99, between 0.94
  and 1.00. A review is what closes the interval.
- **The feather weighed.** With its mass reviewed (10 yes) and gravity
  reviewed, the mass stands at 0.91 and its measurements are doubted,
  0.22 each: nothing else is left to blame. With a completeness piece
  on the tie, that piece would take it (see above).

**Reads odd.**

1. **A firm review cannot be overturned from inside.** Gravity with a
   thousand yes takes 0.01 of a no from a hundred failed measurements
   on ten objects. As long as some unreviewed piece can take the
   blame, it does. That is what was asked of "centuries", and it is
   also how a wrong firm review would survive.
2. **The count is the author's number.** 10 or 100 changes what gets
   blamed (section 3: with 1 yes the mass falls to 0.24, with 10 it
   stands). It is the old bias under a better name: it is signed,
   it is evidence, and it can itself be reviewed, but nothing
   calibrates it.
3. **A piece with a base rate and no evidence has a wide interval.**
   The throw is trusted at 0.9 and its interval is 0.02 to 1.00: with
   a small weight, a base rate alone is "probably right, I have never
   checked". Senses reviewed would close it.
4. **A review and its reviewer.** The review counts as far as the
   reviewer is believed, and that is again a base rate (0.9, 0.99).
   The chain has to stop on a number somebody states.

Each cell: (yes, no), what it reads at, and [where it is right nine times out of ten].

### 1. One throw

A review is believed at 0.9, the throw too.

| review of the statement | review of the tie | I saw | `falls` | `applies` | `throw` |
|---|---|---|---|---|---|
| none | none | fall | (0.9, 0.00) 0.91 [0.40, 1.00] | (0.9, 0.00) 0.91 [0.40, 1.00] | (0.0, 0.00) 0.90 [0.02, 1.00] |
| none | none | no fall | (0.0, 0.06) 0.38 [0.00, 1.00] | (0.0, 0.06) 0.38 [0.00, 1.00] | (0.0, 0.01) 0.88 [0.01, 1.00] |
| 10 yes | none | fall | (9.9, 0.00) 0.99 [0.94, 1.00] | (0.9, 0.00) 0.91 [0.40, 1.00] | (0.0, 0.00) 0.90 [0.02, 1.00] |
| 10 yes | none | no fall | (9.0, 0.01) 0.99 [0.93, 1.00] | (0.0, 0.40) 0.17 [0.00, 0.92] | (0.0, 0.02) 0.83 [0.00, 1.00] |
| 10 yes | 10 yes | fall | (9.9, 0.00) 0.99 [0.94, 1.00] | (9.9, 0.00) 0.99 [0.94, 1.00] | (0.0, 0.00) 0.90 [0.02, 1.00] |
| 10 yes | 10 yes | no fall | (9.0, 0.07) 0.98 [0.90, 1.00] | (9.0, 0.07) 0.98 [0.90, 1.00] | (0.0, 0.43) 0.29 [0.00, 0.98] |
| 100 yes | none | fall | (90.9, 0.00) 1.00 [0.99, 1.00] | (0.9, 0.00) 0.91 [0.40, 1.00] | (0.0, 0.00) 0.90 [0.02, 1.00] |
| 100 yes | none | no fall | (90.0, 0.00) 1.00 [0.99, 1.00] | (0.0, 0.43) 0.16 [0.00, 0.91] | (0.0, 0.02) 0.83 [0.00, 1.00] |

### 2. Gravity reviewed or not, against failures

10 measurements per object. "Centuries" is a review of 1000 yes about gravity, believed at 0.99.

| objects that pass | that fail | gravity not reviewed | gravity reviewed | mass of one that fails, gravity not reviewed | gravity reviewed |
|---|---|---|---|---|---|
| 0 | 1 | (0.0, 0.64) 0.12 [0.00, 0.76] | (990.0, 0.00) 1.00 [1.00, 1.00] | (0.0, 0.64) 0.12 [0.00, 0.76] | (0.0, 4.28) 0.02 [0.00, 0.14] |
| 0 | 3 | (0.0, 1.91) 0.05 [0.00, 0.30] | (990.0, 0.00) 1.00 [1.00, 1.00] | (0.0, 0.64) 0.12 [0.00, 0.76] | (0.0, 4.28) 0.02 [0.00, 0.14] |
| 0 | 10 | (0.0, 6.38) 0.02 [0.00, 0.09] | (990.0, 0.01) 1.00 [1.00, 1.00] | (0.0, 0.64) 0.12 [0.00, 0.76] | (0.0, 4.28) 0.02 [0.00, 0.14] |
| 2 | 8 | (18.0, 0.34) 0.98 [0.91, 1.00] | (1008.0, 0.01) 1.00 [1.00, 1.00] | (0.0, 4.15) 0.02 [0.00, 0.14] | (0.0, 4.28) 0.02 [0.00, 0.14] |
| 9 | 1 | (81.0, 0.01) 1.00 [0.99, 1.00] | (1071.0, 0.00) 1.00 [1.00, 1.00] | (0.0, 4.26) 0.02 [0.00, 0.14] | (0.0, 4.28) 0.02 [0.00, 0.14] |

### 3. The feather's own statement reviewed too

Gravity reviewed (1000 yes), 3 objects pass, the feather fails, 10 measurements each. "I weighed it" is a review about the feather's mass, believed at 0.9.

| review of the feather's mass | `gravity` | `mass f0` | one of its measurements | mass of a new object measured once |
|---|---|---|---|---|
| none | (1017.9, 0.00) 1.00 [1.00, 1.00] | (0.0, 4.28) 0.02 [0.00, 0.14] | (0.0, 0.02) 0.83 [0.00, 1.00] | (0.9, 0.00) 0.91 [0.40, 1.00] |
| 1 yes | (1017.9, 0.00) 1.00 [1.00, 1.00] | (0.9, 3.00) 0.24 [0.02, 0.62] | (0.0, 0.14) 0.53 [0.00, 1.00] | (0.9, 0.00) 0.91 [0.40, 1.00] |
| 10 yes | (1017.9, 0.01) 1.00 [1.00, 1.00] | (9.0, 0.81) 0.91 [0.73, 1.00] | (0.0, 0.60) 0.22 [0.00, 0.93] | (0.9, 0.00) 0.91 [0.40, 1.00] |
| 100 yes | (1017.9, 0.01) 1.00 [1.00, 1.00] | (90.0, 0.10) 1.00 [0.99, 1.00] | (0.0, 0.93) 0.16 [0.00, 0.77] | (0.9, 0.00) 0.91 [0.40, 1.00] |

## A good reviewer and a bad one

Asked: a reviewer good on the model and unsure about masses and
completeness, "confident on stone and less so on a leaf"; then a bad
reviewer, overconfident on everything, who says the feather is
complete. Run with `cargo run --release --example review`.

### 4. A good reviewer

A stone, a ball, a leaf and a feather, 10 measurements each. Each tie has its completeness: "same speed" is all of [gravity, mass, complete].

The reviewer is believed at 0.99. 1000 yes on gravity, 1 yes on each mass. On completeness, by object, not having looked at the air: 20 yes for the stone, 5 for the ball, half a yes for the leaf and for the feather.

| object | passes | its mass | its completeness | one failing measurement |
|---|---|---|---|---|
| stone | 9 of 10 | (9.1, 0.08) 0.98 [0.90, 1.00] | (27.9, 0.03) 1.00 [0.97, 1.00] | (0.0, 0.53) 0.25 [0.00, 0.95] |
| ball | 10 of 10 | (10.0, 0.00) 0.99 [0.94, 1.00] | (14.0, 0.00) 0.99 [0.96, 1.00] | none |
| leaf | 6 of 10 | (6.4, 0.37) 0.93 [0.74, 1.00] | (5.9, 0.40) 0.92 [0.72, 1.00] | (0.0, 0.34) 0.33 [0.00, 1.00] |
| feather | 0 of 10 | (1.0, 1.33) 0.43 [0.05, 0.89] | (0.5, 2.17) 0.21 [0.00, 0.64] | (0.0, 0.06) 0.70 [0.00, 1.00] |

`gravity`: (1012.5, 0.01) 1.00 [1.00, 1.00]

Read plainly. The stone's single failure goes to the measurement
(0.25), not to the stone: 20 yes on its completeness and nine passes
outweigh it. The leaf's four failures are shared: its mass and its
completeness lose a little, each failing measurement is doubted. The
feather's ten failures land on its completeness first (0.21), then on
its mass (0.43), and its measurements are believed (0.70). Gravity is
untouched. The count on a completeness review is what decides where a
failure goes.

### 5. A bad reviewer, and a no that travels back

The bad reviewer is believed at 0.9 and says 100 yes on gravity, on every mass and on every completeness, the feather's too. The stone, the ball and the leaf pass every time, the feather never.

The trial rule: evidence that said yes about a part of what failed is judged by the failure. Each cell is without the rule, then with it.

| measurements per object | review of the feather's completeness | review of the stone's completeness | review of gravity | `complete feather` | `mass feather` | one of the feather's measurements | `gravity` |
|---|---|---|---|---|---|---|---|
| 1 | 0.90 / 0.63 [0.00, 1.00] | 0.90 / 0.90 [0.02, 1.00] | 0.90 / 0.87 [0.01, 1.00] | 1.00 / 1.00 [0.99, 1.00] | 1.00 / 1.00 [0.99, 1.00] | 0.17 / 0.17 [0.00, 0.82] | 1.00 / 1.00 [0.99, 1.00] |
| 3 | 0.90 / 0.39 [0.00, 1.00] | 0.90 / 0.90 [0.02, 1.00] | 0.90 / 0.87 [0.01, 1.00] | 1.00 / 1.00 [0.98, 1.00] | 1.00 / 1.00 [0.98, 1.00] | 0.17 / 0.17 [0.00, 0.82] | 1.00 / 1.00 [0.99, 1.00] |
| 10 | 0.90 / 0.17 [0.00, 0.80] | 0.90 / 0.90 [0.02, 1.00] | 0.90 / 0.88 [0.01, 1.00] | 1.00 / 0.99 [0.94, 1.00] | 1.00 / 0.99 [0.94, 1.00] | 0.17 / 0.17 [0.00, 0.82] | 1.00 / 1.00 [0.99, 1.00] |
| 30 | 0.90 / 0.06 [0.00, 0.34] | 0.90 / 0.90 [0.02, 1.00] | 0.90 / 0.88 [0.01, 1.00] | 1.00 / 0.94 [0.77, 1.00] | 1.00 / 0.94 [0.77, 1.00] | 0.17 / 0.17 [0.00, 0.81] | 1.00 / 1.00 [0.99, 1.00] |
| 100 | 0.90 / 0.02 [0.00, 0.11] | 0.90 / 0.90 [0.02, 1.00] | 0.90 / 0.89 [0.01, 1.00] | 0.99 / 0.66 [0.23, 0.97] | 0.99 / 0.66 [0.23, 0.97] | 0.17 / 0.17 [0.00, 0.80] | 1.00 / 1.00 [1.00, 1.00] |

### The trial rule

Off unless asked (`model::with_vouching`), not in the model. When a
whole fails, each piece that evidence stands on, when that evidence
said yes about parts of what failed, is judged by the failure:

```text
mean after = mean · L(right) / (mean · L(right) + (1 - mean) · L(wrong))
L          = 1 - the chance that all the suspects are right
no added   = α / after - α / mean
```

`mean` is the belief in that piece before any failure. L(right) reads
each part with all of the count the piece stands behind, L(wrong) with
none of it. A failure is more likely when the piece is wrong, since
the parts then stand on less. A piece behind one review is judged on
that review; a piece behind many is judged once on all of them.

Read plainly.

- The review that vouched for the feather's completeness loses credit
  fast: 0.63 after one failure, 0.39 after three, 0.17 after ten.
- Only that review is doubted. The same reviewer's review of the
  stone's completeness stays at 0.90, and the review of gravity barely
  moves, because the passes of the other objects speak for gravity.
  Each review is a piece of its own: there is no shared credit for the
  reviewer's voice.
- The reviews of the feather's mass fall the same way as the one of its
  completeness: nothing tells the two parts apart.
- The part itself falls slowly: `complete feather` still reads 0.99
  after ten failures and 0.66 after a hundred. A review of 100 yes
  believed at 0.17 still gives 17 yes, against a no that grows by a
  fraction per failure. Discounting scales the count; it does not
  cancel it.
- The measurements are still blamed (0.17): every failure is shared
  from the opinions before any no, so the doubt on the review does not
  come back to clear the measurement.

So the rule answers "a bad reviewer is never doubted", and shows two
things it does not answer: an overconfident count survives its own
discredit, and a reviewer is not one thing across reviews.

## Two independent voices

Asked: "let's try to independent voice on the review then, one
cautious uncertain good reviewer, and one bad overconfident reviewer.
How is independence modelled in bayesian network?"

In a Bayesian network independence is the absence of a shared parent.
Each review is a child of two things: the truth of what it reviews,
and the reliability of who wrote it. Two reviews by one voice share
the reliability parent, so they are tied: what discredits one
discredits the other. Two voices have each their own reliability, so
given the truth their words are independent, and their counts add.
Two voices that learnt from the same source would need a parent in
common; that is not modelled here.

In this trial a voice is one piece, `reliable: <voice>`, and every
review by that voice is evidence standing on that one piece. Nothing
else changes: the walk already adds different voices and discounts
evidence by the piece it stands on, and the trial rule judges that
piece once per failure on all it vouched for.

### 6. Two independent voices

The cautious reviewer and the overconfident one review the same four objects. Each voice has one piece, its reliability, behind all its reviews, and both start believed at 0.9. The cautious one says 1000 yes on gravity, 1 on each mass, and on completeness 20 for the stone, 5 for the ball, half a yes for the leaf and the feather. The overconfident one says 100 yes on everything.

The stone, the ball and the leaf pass every time, the feather never. With the trial rule.

| measurements per object | cautious voice | overconfident voice | `complete feather` | `mass feather` | `complete stone` | one of the feather's measurements | `gravity` |
|---|---|---|---|---|---|---|---|
| 1 | 0.90 [0.02, 1.00] | 0.75 [0.00, 1.00] | 1.00 [0.99, 1.00] | 1.00 [0.99, 1.00] | 1.00 [0.99, 1.00] | 0.17 [0.00, 0.80] | 1.00 [1.00, 1.00] |
| 3 | 0.90 [0.02, 1.00] | 0.56 [0.00, 1.00] | 1.00 [0.99, 1.00] | 1.00 [0.99, 1.00] | 1.00 [0.99, 1.00] | 0.17 [0.00, 0.80] | 1.00 [1.00, 1.00] |
| 10 | 0.89 [0.01, 1.00] | 0.30 [0.00, 0.99] | 0.99 [0.97, 1.00] | 0.99 [0.97, 1.00] | 1.00 [0.99, 1.00] | 0.17 [0.00, 0.80] | 1.00 [1.00, 1.00] |
| 30 | 0.89 [0.01, 1.00] | 0.13 [0.00, 0.65] | 0.97 [0.88, 1.00] | 0.97 [0.89, 1.00] | 1.00 [0.99, 1.00] | 0.17 [0.00, 0.80] | 1.00 [1.00, 1.00] |
| 100 | 0.88 [0.01, 1.00] | 0.04 [0.00, 0.23] | 0.82 [0.52, 0.99] | 0.83 [0.55, 0.99] | 1.00 [0.99, 1.00] | 0.17 [0.00, 0.80] | 1.00 [1.00, 1.00] |

The same at 10 measurements, by who reviews, and without the rule.

| who reviews | cautious voice | overconfident voice | `complete feather` | `mass feather` | `complete stone` | one of the feather's measurements | `gravity` |
|---|---|---|---|---|---|---|---|
| cautious | 0.34 [0.00, 1.00] |  | 0.11 [0.00, 0.49] | 0.24 [0.00, 0.77] | 0.99 [0.96, 1.00] | 0.71 [0.00, 1.00] | 1.00 [1.00, 1.00] |
| overconfident |  | 0.12 [0.00, 0.62] | 0.98 [0.92, 1.00] | 0.98 [0.92, 1.00] | 1.00 [0.97, 1.00] | 0.17 [0.00, 0.82] | 1.00 [0.98, 1.00] |
| both | 0.89 [0.01, 1.00] | 0.30 [0.00, 0.99] | 0.99 [0.97, 1.00] | 0.99 [0.97, 1.00] | 1.00 [0.99, 1.00] | 0.17 [0.00, 0.80] | 1.00 [1.00, 1.00] |
| both, without the rule | 0.90 | 0.90 | 1.00 | 1.00 | 1.00 | 0.17 | 1.00 |

Noisy objects, 10 measurements each, both voices, with the rule.

| object | passes | its mass | its completeness | one failing measurement |
|---|---|---|---|---|
| stone | 9 of 10 | 1.00 [0.98, 1.00] | 1.00 [0.99, 1.00] | 0.16 [0.00, 0.79] |
| ball | 10 of 10 | 1.00 [0.98, 1.00] | 1.00 [0.99, 1.00] | none |
| leaf | 6 of 10 | 1.00 [0.98, 1.00] | 1.00 [0.98, 1.00] | 0.17 [0.00, 0.80] |
| feather | 0 of 10 | 0.99 [0.97, 1.00] | 0.99 [0.97, 1.00] | 0.17 [0.00, 0.80] |

cautious voice 0.89 [0.01, 1.00], overconfident voice 0.29 [0.00, 0.98], `gravity` 1.00 [1.00, 1.00]

Read plainly.

- The overconfident voice is found: 0.75 after one failure of the
  feather, 0.30 after ten, 0.04 after a hundred. It falls slower than
  its lone review did (0.63, 0.17), because the cautious voice also
  spoke for the feather's parts, so they stand on something without
  it.
- Its word on the stone drops with it, since it is the same piece:
  at ten failures its 100 yes on the stone count as 30. `complete
  stone` still reads 1.00, held by the cautious voice's 20 yes and the
  stone's own passes.
- The cautious voice is hardly touched when both speak (0.90 to 0.88):
  its half a yes on the feather changes little of what the parts
  stand on.
- Alone, the cautious voice is discredited to 0.34 by the feather: a
  half yes is still a yes that failed, and the voice starts on a weak
  belief (the base rate 0.9 weighs W = 0.2). Its 1000 yes on gravity
  and 20 on the stone, confirmed by thirty passes, earn it nothing:
  the rule only takes credit away.
- The blame on the feather does not follow the discredit. With both
  voices `complete feather` reads 0.99 at ten failures and 0.82 at a
  hundred, and each of its measurements is blamed (0.17), where the
  cautious voice alone puts the blame on completeness (0.11) and
  believes the measurements (0.71). Two causes, both seen before:
  failures are shared from the opinions before any no, where the
  overconfident voice is still believed at 0.9; and 100 yes believed
  at 0.30 are still 30 yes.
- On the noisy objects the overconfident voice drowns the cautious
  one: every part reads 1.00 and every failing measurement is blamed.

So the voice as one piece does what was hoped for the voice, and the
parts do not benefit yet. What would let them: share the failures a
second time, from the opinions after the voices are judged; and let a
pass speak for the voice that vouched for it. Neither is tried.

## Voices, iterated

Asked: "Ok, iterate on voice". The design from the chat: a
reliability per voice, the parent of all that voice says; voices
independent unless a shared source is declared; a voice gains credit
when what it vouched for passes; and a discredited voice stops
holding up the parts.

This trial is a pass outside the core (`examples/voice.rs`,
`cargo run --release --example voice`): it judges the voices,
discounts their word, and hands the model to the core's `opinion`.
The core is not changed.

Three things differ from the trial before.

**The word of a voice is a mix, not a scaled count.** A voice is
reliable or not. If it is, its word counts whole; if not, it is
nothing. So 100 yes from a voice believed at 0.3 read as 0.65 on a
piece at one half, where scaled counts gave 30 yes and 0.997. A belief
never exceeds the trust in who says it: 100 yes from a voice at 0.9
read 0.95. This is the trust discount of subjective logic, written in
counts. The count scaling agreed for `about` is untouched: it still
applies to how far one piece of evidence is believed on its own.

**A voice is judged per whole, passes and failures together.** With
`p` the chance that the parts of a whole are all right before anything
is seen of it:

```text
L = p · (every failing measurement is wrong) + (1 - p) · (every passing one is wrong)
```

read once with the voice's word taken whole, once without it. The
ratio of the two, over all the wholes, moves the odds of the voice.
Ten failures of one whole are one fact with one cause, not ten: a
voice that said "probably" is not destroyed by repetition, and a voice
that said "surely" is. Passes give credit the same way, as far as the
voice's word was needed to expect them.

**A source is a shared parent.** The word of a voice holds when the
voice is reliable and its source is. Voices on one source are judged
together: with `L` the ratio of a voice and `a` its chance, the source
is weighed by the product of `a · L + 1 - a` over its voices.
Independence is the absence of such a parent, and is the default.

### 1. A cautious voice and an overconfident one

Both start at 0.9. The cautious one says 1000 yes on gravity, 1 on each mass, and on completeness 20 for the stone, 5 for the ball, half a yes for the leaf and the feather. The overconfident one says 100 yes on everything. The stone, the ball and the leaf pass every time, the feather never.

| measurements per object | cautious voice | overconfident voice | `complete feather` | `mass feather` | `complete stone` | one of the feather's measurements | `gravity` |
|---|---|---|---|---|---|---|---|
| 1 | 0.90 | 0.84 | 0.85 [0.34, 1.00] | 0.87 [0.42, 1.00] | 0.97 [0.84, 1.00] | 0.60 [0.00, 1.00] | 0.97 [0.85, 1.00] |
| 3 | 0.98 | 0.20 | 0.46 [0.01, 0.97] | 0.67 [0.10, 1.00] | 0.99 [0.93, 1.00] | 0.71 [0.00, 1.00] | 0.99 [0.95, 1.00] |
| 10 | 0.98 | 0.15 | 0.22 [0.00, 0.66] | 0.41 [0.04, 0.87] | 0.99 [0.96, 1.00] | 0.70 [0.00, 1.00] | 1.00 [0.98, 1.00] |
| 30 | 0.98 | 0.14 | 0.09 [0.00, 0.29] | 0.20 [0.01, 0.52] | 1.00 [0.98, 1.00] | 0.70 [0.00, 1.00] | 1.00 [0.99, 1.00] |
| 100 | 0.98 | 0.13 | 0.03 [0.00, 0.10] | 0.07 [0.00, 0.20] | 1.00 [0.99, 1.00] | 0.70 [0.00, 1.00] | 1.00 [1.00, 1.00] |

### 2. By who reviews, 10 measurements per object

| who reviews | cautious voice | overconfident voice | `complete feather` | `mass feather` | `complete stone` | one of the feather's measurements | `gravity` |
|---|---|---|---|---|---|---|---|
| cautious | 0.99 |  | 0.21 [0.00, 0.64] | 0.42 [0.04, 0.88] | 0.99 [0.97, 1.00] | 0.71 [0.00, 1.00] | 1.00 [0.98, 1.00] |
| cautious, silent on the leaf's and the feather's completeness | 0.99 |  | 0.03 [0.00, 0.21] | 0.70 [0.12, 1.00] | 1.00 [0.97, 1.00] | 0.84 [0.00, 1.00] | 1.00 [0.99, 1.00] |
| overconfident |  | 0.68 | 0.24 [0.00, 0.74] | 0.24 [0.00, 0.74] | 0.99 [0.94, 1.00] | 0.76 [0.00, 1.00] | 0.99 [0.97, 1.00] |
| both | 0.98 | 0.15 | 0.22 [0.00, 0.66] | 0.41 [0.04, 0.87] | 0.99 [0.96, 1.00] | 0.70 [0.00, 1.00] | 1.00 [0.98, 1.00] |

### 3. Noisy objects, both voices, 10 measurements each

| object | passes | its mass | its completeness | one failing measurement |
|---|---|---|---|---|
| stone | 9 of 10 | 0.98 [0.90, 1.00] | 0.99 [0.94, 1.00] | 0.28 [0.00, 0.98] |
| ball | 10 of 10 | 0.99 [0.94, 1.00] | 0.99 [0.95, 1.00] | none |
| leaf | 6 of 10 | 0.93 [0.74, 1.00] | 0.93 [0.72, 1.00] | 0.35 [0.00, 1.00] |
| feather | 0 of 10 | 0.41 [0.04, 0.87] | 0.22 [0.00, 0.66] | 0.70 [0.00, 1.00] |

cautious voice 0.98, overconfident voice 0.15, `gravity` 0.99 [0.96, 1.00]

### 4. Two agents, independent or sharing a source

Agent A says 100 yes on gravity and on the mass and the completeness of the ball and of the feather. Agent B says 100 yes on the mass and the completeness of the stone, and of nothing else. The ball passes 10 times, the feather fails 10 times, the stone is never measured. Each agent starts at 0.9, and so does the source when they share one: a word then starts at 0.81.

| declared | A's word holds at | B's word holds at | the source | `complete stone` | `complete feather` | one of the feather's measurements |
|---|---|---|---|---|---|---|
| independent | 0.22 | 0.90 |  | 0.95 [0.67, 1.00] | 0.14 [0.00, 0.74] | 0.86 [0.00, 1.00] |
| one source | 0.12 | 0.48 | 0.53 | 0.74 [0.00, 1.00] | 0.13 [0.00, 0.75] | 0.87 [0.00, 1.00] |

Read plainly.

- The overconfident voice is found from three failures on (0.20,
  then 0.15), and the cautious one gains from the passes it vouched
  for (0.90 to 0.98). After one measurement per object neither has
  moved much.
- The blame moves to where it belongs: the feather's completeness
  reads 0.22 after ten failures and 0.03 after a hundred, its mass
  0.41 then 0.07, and its measurements are believed (0.70). With both
  voices the model reads nearly as with the cautious one alone.
- The overconfident voice's word on the stone drops with it. The
  stone still reads 0.99, held by the cautious voice and its passes.
- A voice that keeps silent where it does not know does best: the
  cautious voice silent on the feather's completeness puts all the
  blame there (0.03) and leaves the mass at 0.70.
- The overconfident voice alone keeps 0.68: three objects confirm its
  word and one contradicts it, and nothing else explains the passes.
  It does not fall further with more failures, because one cause
  explains them all. With all evidence from one bad voice, nothing
  better can be expected.
- Two agents: declared independent, the failure of the feather costs
  A its word (0.22) and leaves B's word on the stone whole (0.90).
  On one source, the source is doubted (0.53), and B's word on the
  stone, which nothing tested, falls from 0.81 to 0.48: `complete
  stone` reads 0.74 instead of 0.95.

What is rough.

- Voices are judged in three rounds, each reading the others as
  judged so far. It settles here; nothing says it always does.
- A voice is judged before any no is shared, so a no on another whole
  does not inform it.
- The core still shares a no per failure, so a part falls with the
  number of failures (0.22, 0.09, 0.03) while the voice does not.
- Voice priors of 0.9 and a source prior of 0.9 are picked by hand.

## The toys written with ties

Asked: "how about we expect the author to make a tie link to one or
several statement, and we put completeness by default?"

A tie now links a place to one or several statements. It implies its
completeness (0.6 unless written) and its whole, which is all of
[the tie, its statements, its completeness] and is what an observation
at the place is about. Nobody writes `lands` any more. A shared
statement such as gravity is listed by the tie of each object.
`cargo run --release --example tied` reads each toy both ways.

### 1. One throw

The throw is believed at 0.9; `falls` and `applies` start at one half. By hand, `lands` is all of [applies, falls]. Tied, the tie `applies` lists `falls`, and its completeness starts at 0.6.

| written | I saw | `falls` | `applies` | completeness | `throw` |
|---|---|---|---|---|---|
| by hand | fall | 0.91 | 0.91 |  | 0.90 |
| by hand | no fall | 0.38 | 0.38 |  | 0.88 |
| with a tie | fall | 0.91 | 0.91 | 0.93 | 0.90 |
| with a tie | no fall | 0.43 | 0.43 | 0.55 | 0.89 |

### 2. Gravity, objects that pass and a feather that never does

10 measurements per object. By hand, "same speed" is all of [gravity, its mass]. Tied, each object has a tie that lists gravity and its mass; the tie starts at one half and its completeness at 0.6.

| written | objects that pass | `gravity` | mass of one that passes | the feather's mass | the feather's tie | the feather's completeness | one of the feather's measurements |
|---|---|---|---|---|---|---|---|
| by hand | 1 | 0.98 | 0.99 | 0.02 |  |  | 0.83 |
| by hand | 3 | 1.00 | 0.99 | 0.02 |  |  | 0.83 |
| by hand | 9 | 1.00 | 0.99 | 0.02 |  |  | 0.83 |
| by hand | 30 | 1.00 | 0.99 | 0.02 |  |  | 0.83 |
| with ties | 1 | 0.99 | 0.99 | 0.20 | 0.20 | 0.31 | 0.89 |
| with ties | 3 | 1.00 | 0.99 | 0.20 | 0.20 | 0.31 | 0.89 |
| with ties | 9 | 1.00 | 0.99 | 0.20 | 0.20 | 0.31 | 0.89 |
| with ties | 30 | 1.00 | 0.99 | 0.20 | 0.20 | 0.31 | 0.89 |

### 3. The cautious and the overconfident reviewer

A stone, a ball and a leaf pass 10 times, the feather fails 10 times. Both voices start at 0.9. By hand, each object has a completeness piece at one half. Tied, the completeness is implied at 0.6 and the tie starts at one half. The overconfident voice says 100 yes on everything, ties too.

| written | cautious voice | overconfident voice | `gravity` | the feather's mass | the feather's tie | the feather's completeness | the stone's completeness | one of the feather's measurements |
|---|---|---|---|---|---|---|---|---|
| by hand | 0.98 | 0.15 | 1.00 | 0.41 |  | 0.22 | 0.99 | 0.70 |
| with ties | 0.96 | 0.41 | 1.00 | 0.61 | 0.11 | 0.52 | 0.99 | 0.81 |

Read plainly.

- One throw. A fall reads the same. A miss blames the statement and
  the tie a little less (0.43, was 0.38), because the completeness
  takes a share of the no.
- Gravity. Nothing changes for gravity or for what passes. The
  feather's ten failures are now shared by three suspects of its own
  instead of one: its mass reads 0.20 (was 0.02), its tie 0.20, its
  completeness 0.31. Its measurements are believed a little more
  (0.89, was 0.83). Nothing tells the three apart: only evidence
  about one of them alone would, a scale for the mass.
- Reviewers. The blame lands on the feather's tie (0.11): the
  cautious voice spoke for the mass and, a little, for the
  completeness, and nobody but the overconfident voice spoke for the
  tie. The overconfident voice is doubted less (0.41, was 0.15),
  because the tie at one half already explains much of the failure
  without it. The cautious voice and the stone are unchanged.

Not rewritten: the older trials above (`complete`, `weight`, `review`
and `sweep` examples) still write their relations by hand, which the
core still reads; and the toy with two statements observed apart
(it falls, it makes no noise), since the whole of a tie is all its
statements at once.
