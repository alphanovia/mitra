# The throwing toy

The smallest model we could think of, used to settle the relations
and their computation. Every row below is what the core computes,
pinned in `tests/throwing.rs`; the toy is also claims of
`model.mitra`. The sweep over many objects is in [sweep.md](sweep.md).

## An opinion is counts

`(r, s, a)`: `r` counts of yes, `s` counts of no, `a` the base rate.
One setting for the whole system, the weight `W = 2`.

```text
reads at = (r + a·W) / (r + s + W)
```

With no evidence it reads at its base rate. One yes on one half reads
2/3; eight read 0.9. This is a Beta: how often the piece is right,
with `r + a·W` on the yes side and `s + (1 - a)·W` on the no side.

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

It falls:

| a | f | s | tie and statement | read at | the pair reads at |
|---|---|---|---|---|---|
| 0.5 | 0.5 | 0.5 | (0.5, 0) | 0.60 | 0.36 |
| 0.5 | 0.5 | 0.9 | (0.9, 0) | 0.66 | 0.43 |
| 0.5 | 0.5 | 1 | (1, 0) | 0.67 | 0.44 |
| 0.9 | 0.9 | 0.5 | (0.5, 0) | 0.92 | 0.85 |
| 1 | 1 | 1 | (1, 0) | 1 | 1 |
| 0 | 0 | 1 | (1, 0) | 0.33 | 0.11 |
| 0.5 | 0.5 | 0 | (0, 0) | 0.50 | 0.25 |

It does not fall. The count of no on each, and what each reads at:

| a | f | s | no on tie | no on statement | no on throw | read at: tie, statement, throw | Suspects, least believed first |
|---|---|---|---|---|---|---|---|
| 0.5 | 0.5 | 0.5 | 0.10 | 0.10 | 0.10 | 0.48, 0.48, 0.48 | no order |
| 1 | 1 | 1 | 0.33 | 0.33 | 0.33 | 0.86, 0.86, 0.86 | no order |
| 0.5 | 0.5 | 0.9 | 0.21 | 0.21 | 0.02 | 0.45, 0.45, 0.89 | tie, statement, throw |
| 0.5 | 0.5 | 1 | 0.25 | 0.25 | 0 | 0.44, 0.44, 1 | tie, statement, throw |
| 0.9 | 0.9 | 0.5 | 0.05 | 0.05 | 0.59 | 0.88, 0.88, 0.39 | throw, tie, statement |
| 0.9 | 0.5 | 0.6 | 0.03 | 0.28 | 0.18 | 0.89, 0.44, 0.55 | statement, throw, tie |
| 1 | 1 | 0 | 0 | 0 | 0 | 1, 1, 0 | none: senses not trusted |
| 0 | 0 | 1 | 0 | 0 | 0 | 0, 0, 1 | tie, statement, throw |

An equal start gives equal shares. Senses trusted more put the no on
the tie and the statement; senses trusted less, on the throw. One
miss moves little: from 0.5 to 0.45.

## 2. The same throw again

Senses at 0.6, the tie and the statement at one half. What the tie
(and the statement) read at:

| Throws | By me alone | By different people |
|---|---|---|
| 1 | (0.6, 0) 0.62 | (0.6, 0) 0.62 |
| 2 | (0.6, 0) 0.62 | (1.2, 0) 0.69 |
| 6 | (0.6, 0) 0.62 | (3.6, 0) 0.82 |
| 10 | (0.6, 0) 0.62 | (6, 0) 0.88 |
| 40 | (0.6, 0) 0.62 | (24, 0) 0.96 |

## 3. A ball and a balloon, one statement

Two pairs share `falls`. The ball falls, the balloon does not. All
at one half.

| s | `falls` | `ball applies` | `balloon applies` | `throw balloon` | Suspects |
|---|---|---|---|---|---|
| 0.9 | (0.9, 0.17) 0.62 | (0.9, 0) 0.66 | (0, 0.32) 0.43 | (0, 0.03) 0.89 | the balloon's tie, `falls`, the throw |
| 1 | (1, 0.2) 0.63 | (1, 0) 0.67 | (0, 0.4) 0.42 | unmoved | the balloon's tie, `falls`, the throw |

The balloon's tie takes about twice what `falls` takes, because the
ball speaks for `falls`. With one ball the statement is still doubted
a fair bit: it takes many balls to clear it (see the sweep).

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
| the ball falls | (0.9, 0) 0.66 | (0.9, 0) 0.66 | |
| the ball does not | (0, 0.21) 0.45 | (0, 0.21) 0.45 | |
| both fall | (1.8, 0) 0.74 | (0.9, 0) 0.66 | (0.9, 0) 0.66 |
| the ball falls, the balloon does not | (0.9, 0.17) 0.62 | (0.9, 0) 0.66 | (0, 0.32) 0.43 |
| neither falls | more no than either mass | | |

With one object, gravity and its mass cannot be told apart. With two,
gravity is seen twice and each mass once, and a balloon that does not
fall is put on its own mass first.

A relation is itself said. The same toy with the relation believed at
0.5 gives gravity half the yes; with no base rate for the relation,
it relates nothing and gravity stays where it was.

## On main's goldens

One claim, one test, at most two reviews, the claim expected at 0.
One test is one yes against a weight of two, so **no claim holds at
policy 0.9**: a passing test reviewed at 1 reads 0.25, unreviewed 0.2.
Five cases that held are now unverified (2, 3, 5, 15, 19), and a
failing test reviewed as wrong (22) still fails its claim by a third
of a no. `tests/goldens.rs` records all of them. Thirty unreviewed
tests by thirty voices hold a claim at 0.9; from a base rate of one
half it takes eight sure ones.

## What reads odd

1. **One observation is small against W = 2.** See the goldens. The
   policy, the weight, or what one test counts for has to move.
2. **A base rate of 0 is strong**: "expected wrong, with the weight of
   two observations". A piece at 0 takes no blame (it is wrong
   already) and one sure yes brings it to 1/3 only. "Nothing assumed"
   is one half, and a piece with no base rate still gets 0.
3. **One sure "wrong" does not silence a test.** A test reviewed as
   wrong at 1 reads 1/3 and still gives its claim a third of a yes.
4. **One voice counts once, by its strongest yes and its strongest
   no.** Two different throws by me are one. Voice and act are one
   thing today; the gravity toy gives each throw its own voice.
5. **Each failure is shared from what was expected before any no.**
   Two failures are each shared as if the other had not happened, and
   their counts add.
6. **The counts of a whole are not counts of anything.** They are the
   Beta that fits a product: a pair after one yes at 0.5 shows
   (0.67, 0.59). Only what it reads at means something.
7. **A whole switches between its parts and what is said of it.**
   After a miss, `lands` reads 0.20 from its parts in one row and 0
   from the direct no in another, depending on which has more counts.
8. **A sure throw is doubted when all were sure.** Everything at 1 and
   a miss: each takes a third of a no.

Gone with counts: "the pair is not the product of its parts" (it is
now), "a pair with no evidence reads at 0 when its parts are expected
at 1", and "blame uses belief, not expectation".

## Open on relations

- `Exclude` is still in the record and the toys never use it.
- A tie without a statement cannot be forbidden by the records as they
  are: a tie names a statement by its piece.
- The latent toy needs no new relation: `All` is enough. It may later
  read as a contract in the syntax.
