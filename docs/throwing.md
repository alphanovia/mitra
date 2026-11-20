# The throwing toy

The smallest model we could think of, used to settle the relations
and their computation. Every row below is what the core computes,
pinned in `tests/throwing.rs`; the toy is also claims of
`model.mitra`. The sweep over many objects is in [sweep.md](sweep.md).

## An opinion is counts

`(r, s, a)`: `r` counts of yes, `s` counts of no, `a` the base rate.
One setting for the whole system, the weight `W = 0.2`.

```text
reads at = (r + a·W) / (r + s + W)
```

With no evidence it reads at its base rate. This is a Beta: how often
the piece is right, with `r + a·W` on the yes side and
`s + (1 - a)·W` on the no side. One half as the neutral start is the
usual one for it. A small weight says "it is either right or wrong, I
do not know which", so the first observation nearly settles it:

| One sure yes on one half reads | W |
|---|---|
| 0.67 | 2 |
| 0.75 | 1 |
| 0.92 | 0.2 |
| 0.99 | 0.02 |

## The toy

| Piece | Kind | Says |
|---|---|---|
| `falls` | statement | thrown up, it goes up then falls down |
| `applies` | tie | this model is a good model of this object |
| `lands` | the pair | `applies` and `falls` together: this object, thrown, falls |
| `throw` | evidence about the pair | I threw it and saw it fall, or not; believed as far as I trust my senses |

## Two relations, named after what is computed on them

**`about`**: evidence to the piece it speaks of. It is the field
`Evidence.about`, not a declared relation.

- *discount*: the counts of what the evidence says, times what the
  evidence reads at. A fall seen at 0.6 is 0.6 of a yes, and never
  any no.
- *fuse*: different voices add their counts. One voice counts once:
  its strongest yes and its strongest no.

**`All`**: a whole to its parts. Declared:
`Related(All, [lands, applies, falls])`, itself said by a statement,
and counted as far as that statement is believed.

| Direction | What is computed |
|---|---|
| up | each part is a Beta, taken as independent; the whole is the Beta with the mean and the variance of their product |
| down, a yes | the same count of yes on each part. This is exact: seeing all right once is one yes for each |
| down, a no | a failure is seeing that the suspects are not all right. Each suspect gets the count of no that brings it to its Bayesian mean |
| the whole itself | its parts together, or what is said of it directly when that leaves less uncertain |

The suspects of a failure are the parts, the evidence that says no,
and the relation when it is doubted. With `q` the chance that all the
other suspects are right, from before any no:

```text
mean after = (mean - q · mean of the square) / (1 - q · mean)
no added   = (r + a·W) · (1 / mean after - 1 / mean)
```

The mean is Bayes for one observation. Putting all of the move into
the count of no, and none into the yes, is my choice: it keeps
"a no never becomes a yes" for the shared no too.

## 1. One throw

`a`, `f`: base rates of the tie and the statement. `s`: trust in my
senses.

It falls (the last column is what the tie read at with W = 2):

| a | f | s | tie and statement | read at | the pair reads at | with W 2 |
|---|---|---|---|---|---|---|
| 0.5 | 0.5 | 0.5 | (0.5, 0) | 0.86 | 0.74 | 0.60 |
| 0.5 | 0.5 | 0.9 | (0.9, 0) | 0.91 | 0.83 | 0.66 |
| 0.5 | 0.5 | 1 | (1, 0) | 0.92 | 0.84 | 0.67 |
| 0.9 | 0.9 | 0.5 | (0.5, 0) | 0.97 | 0.94 | 0.92 |
| 1 | 1 | 1 | (1, 0) | 1 | 1 | 1 |
| 0 | 0 | 1 | (1, 0) | 0.83 | 0.69 | 0.33 |
| 0.5 | 0.5 | 0 | (0, 0) | 0.50 | 0.25 | 0.50 |

It does not fall. The count of no on each, and what each reads at:

| a | f | s | no on tie | no on statement | no on throw | read at: tie, statement, throw | with W 2 | Suspects, least believed first |
|---|---|---|---|---|---|---|---|---|
| 0.5 | 0.5 | 0.5 | 0.03 | 0.03 | 0.03 | 0.44, 0.44, 0.44 | 0.48, 0.48, 0.48 | no order |
| 1 | 1 | 1 | 0.33 | 0.33 | 0.33 | 0.38, 0.38, 0.38 | 0.86, 0.86, 0.86 | no order |
| 0.5 | 0.5 | 0.9 | 0.06 | 0.06 | 0.01 | 0.38, 0.38, 0.88 | 0.45, 0.45, 0.89 | tie, statement, throw |
| 0.5 | 0.5 | 1 | 0.08 | 0.08 | 0 | 0.36, 0.36, 1 | 0.44, 0.44, 1 | tie, statement, throw |
| 0.9 | 0.9 | 0.5 | 0.01 | 0.01 | 0.26 | 0.84, 0.84, 0.22 | 0.88, 0.88, 0.39 | throw, tie, statement |
| 0.9 | 0.5 | 0.6 | 0.01 | 0.09 | 0.05 | 0.87, 0.35, 0.48 | 0.89, 0.44, 0.55 | statement, throw, tie |
| 1 | 1 | 0 | 0 | 0 | 0 | 1, 1, 0 | 1, 1, 0 | none: senses not trusted |
| 0 | 0 | 1 | 0 | 0 | 0 | 0, 0, 1 | 0, 0, 1 | tie, statement, throw |

A yes nearly settles the tie and the statement: 0.5 to 0.92. A miss
does not, and should not: with a sure miss the tie and the statement
go to 0.36, close to the one third that "one of the two is wrong, I
do not know which" gives. An equal start gives equal shares. Senses
trusted more put the no on the tie and the statement; senses trusted
less, on the throw.

## 2. The same throw again

Senses at 0.6, the tie and the statement at one half. What the tie
(and the statement) read at:

| Throws | By me alone | By different people |
|---|---|---|
| 1 | (0.6, 0) 0.88 | (0.6, 0) 0.88 |
| 2 | (0.6, 0) 0.88 | (1.2, 0) 0.93 |
| 6 | (0.6, 0) 0.88 | (3.6, 0) 0.97 |
| 10 | (0.6, 0) 0.88 | (6, 0) 0.98 |

## 3. A ball and a balloon, one statement

Two pairs share `falls`. The ball falls, the balloon does not. All
at one half.

| s | `falls` | `ball applies` | `balloon applies` | `throw balloon` | Suspects |
|---|---|---|---|---|---|
| 0.9 | (0.9, 0.04) 0.88 | (0.9, 0) 0.91 | (0, 0.27) 0.21 | (0, 0.01) 0.84 | the balloon's tie, the throw, `falls` |
| 1 | (1, 0.04) 0.89 | (1, 0) 0.92 | (0, 0.48) 0.15 | unmoved | the balloon's tie, `falls`, the throw |

One ball is now enough to speak for `falls`: the balloon's tie takes
almost all of the no.

## 4. One object, two statements

Two pairs share `applies`. It falls, and makes no noise. The mirror,
with the same numbers: `noise` takes the most, and is the first
suspect.

## 5. A latent statement: gravity and mass

`gravity` is latent: no tie, nothing to throw at. `mass ball` is the
statement tied to the ball. "The ball falls" is all of
[`gravity`, `mass ball`], and a throw is evidence about that. Senses
at 0.9, all at one half, **each throw with a voice of its own**.

| Seen | `gravity` | `mass ball` | `mass balloon` |
|---|---|---|---|
| the ball falls | (0.9, 0) 0.91 | (0.9, 0) 0.91 | |
| the ball does not | (0, 0.06) 0.38 | (0, 0.06) 0.38 | |
| both fall | (1.8, 0) 0.95 | (0.9, 0) 0.91 | (0.9, 0) 0.91 |
| the ball falls, the balloon does not | (0.9, 0.04) 0.88 | (0.9, 0) 0.91 | (0, 0.27) 0.21 |
| neither falls | more no than either mass | | |

With one object, gravity and its mass cannot be told apart. With two,
gravity is seen twice and each mass once, and a balloon that does not
fall is put on its own mass first.

A relation is itself said. The same toy with the relation believed at
0.5 gives gravity half the yes; with no base rate for the relation,
it relates nothing and gravity stays where it was.

## On main's goldens

One claim, one test, at most two reviews, the claim expected at 0.

| Case | W 2 | W 0.2 | Before counts |
|---|---|---|---|
| passing, no review | 0.20 | 0.71 | 0.5 |
| passing, reviewed at 1 | 0.25 | 0.82 | holds 1 |
| passing, reviewed at 0.8 | 0.24 | 0.82 | holds 0.9 |
| passing, reviewed as wrong | 0.14 | 0.29 | 0 |
| failing, reviewed as wrong | fails | fails | unverified |

**No claim holds at policy 0.9 on one test**, reviewed or not: the
test itself reads 0.92 at best and gives its claim that much of a
yes, 0.82. Three unreviewed tests by three voices hold it. The cases
that held and no longer do: 2, 3, 15, 19. `tests/goldens.rs` records
all of them.

## What reads odd

1. **A chain loses at each step.** A sure review, a test, a claim:
   0.92, then 0.82. One reviewed test does not hold a claim at 0.9.
2. **A base rate of 0 means "expected wrong"**, lightly now: one sure
   yes brings it to 0.83. A piece at 0 takes no blame (it is wrong
   already). "Nothing assumed" is one half, and a piece with no base
   rate still gets 0.
3. **One sure "wrong" does not silence a test.** A test reviewed as
   wrong at 1 reads 0.08 and still gives its claim that much of a yes;
   a failing test reviewed as wrong still fails its claim, by 0.08.
4. **One voice counts once, by its strongest yes and its strongest
   no.** Two different throws by me are one. Voice and act are one
   thing today; the gravity toy gives each throw its own voice.
5. **Each failure is shared from what was expected before any no.**
   Two failures are each shared as if the other had not happened, and
   their counts add.
6. **The counts of a whole are not counts of anything.** They are the
   Beta that fits a product: a pair after one yes at 0.5 shows
   (0.51, 0.05). Only what it reads at means something.
7. **A whole switches between its parts and what is said of it.**
   After a miss, `lands` reads from the direct no (0) or from its
   parts, depending on which has more counts.
8. **A sure throw is doubted when all were sure.** Everything at 1 and
   a miss: each takes a third of a no, and falls from 1 to 0.38.
9. **A small weight makes one observation loud.** One doubtful yes
   (senses at 0.5) already reads 0.86. What stops a wrong first
   observation from settling a piece is only a later no.

Gone with counts: "the pair is not the product of its parts" (it is
now), "a pair with no evidence reads at 0 when its parts are expected
at 1", and "blame uses belief, not expectation".

## Open on relations

- A tie without a statement cannot be forbidden by the records as they
  are: a tie names a statement by its piece.
- The latent toy needs no new relation: `All` is enough. It may later
  read as a contract in the syntax.
