# The throwing toy

The smallest model we could think of, used to settle one relation and
its computation. Every row below is what the core computes, pinned in
`tests/throwing.rs`; the toy is also claims of `model.mitra`.

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

- *discount*: what the evidence says counts as far as the evidence is
  believed. `moved(opinion(e), e.value)`.
- *fuse*: several pieces of evidence about one piece add up as far as
  their voices are independent.

**`All`**: a whole to its parts, here the pair to its tie and its
statement. Declared: `Related(All, [lands, applies, falls])`.

| Direction | Operation | Formula |
|---|---|---|
| up | conjunction | `b = b1·b2`, `d = d1 + d2 - d1·d2` |
| down, a yes | what believes the whole believes each part as much | part gets `(b, 0)` from each evidence about the whole, with its voice |
| down, a no | shared between the suspects as disbelief | each suspect gets `(0, d · share)`, see below |
| the whole itself | what is said of it directly, or its parts together | whichever is more certain (less `u`) |

The **conflict is computed, not declared**: evidence saying no about
a whole means its parts and that evidence cannot all be right. They
are the suspects, listed the least believed first (by `P = b + a·u`).
A relation believed under 1 is a suspect too.

**How a no is shared.** Each suspect takes the chance that it alone
is wrong, given that they are not all right, with `p` what each was
expected to be (`P`) before the failure:

```text
share(x) = (1 - p(x)) · product of p(others) / (1 - product of p(all))
```

When all were expected at 1 the shares are equal. This is not an
operator of subjective logic. The base rates are what it reads when
nothing else is known: they are where "I trust my senses more than my
model or my tie" is said. The evidence takes its share too, so a
failed throw is itself believed less afterwards, and the pair is
disbelieved as far as the throw is still believed.

## 1. One throw

`a`, `f`: base rates of the tie and the statement. `s`: trust in my
senses. An opinion is written (b, d).

It falls:

| a | f | s | `lands` | `applies` and `falls` |
|---|---|---|---|---|
| 0 | 0 | 1 | (1, 0) | (1, 0) |
| 1 | 1 | 1 | (1, 0) | (1, 0) |
| 0 | 0 | 0 | (0, 0) | unmoved |
| 0.5 | 0.5 | 0.9 | (0.9, 0) | (0.9, 0), P 0.95 |
| 0 | 0 | 0.6 | (0.6, 0) | (0.6, 0) |

It does not fall. Disbelief put on each suspect, and where each reads
afterwards (P):

| a | f | s | d `applies` | d `falls` | d `throw` | P after: tie, statement, throw | Suspects, least believed first |
|---|---|---|---|---|---|---|---|
| 0.5 | 0.5 | 0.5 | 0.143 | 0.143 | 0.143 | 0.43, 0.43, 0.43 | no order |
| 1 | 1 | 1 | 0.333 | 0.333 | 0.333 | 0.67, 0.67, 0.67 | no order |
| 0.5 | 0.5 | 0.9 | 0.290 | 0.290 | 0.032 | 0.35, 0.35, 0.87 | `applies`, `falls`, `throw` |
| 0.5 | 0.5 | 1 | 0.333 | 0.333 | 0 | 0.33, 0.33, 1 | `applies`, `falls`, `throw` |
| 0.9 | 0.9 | 0.5 | 0.076 | 0.076 | 0.681 | 0.83, 0.83, 0.16 | `throw`, `applies`, `falls` |
| 1 | 1 | 0 | 0 | 0 | 1 | 1, 1, 0 | none: senses not trusted |
| 0.9 | 0.5 | 0.6 | 0.041 | 0.370 | 0.247 | 0.86, 0.32, 0.45 | `falls`, `throw`, `applies` |
| 0 | 0 | 1 | 0 | 0 | 0 | 0, 0, 1 | `applies`, `falls`, `throw` |
| 0 | 0 | 0.6 | 0 | 0 | 0 | 0, 0, 0.6 | `applies`, `falls`, `throw` |

An equal start gives equal shares. Senses trusted more leave the tie
and the statement more disbelieved than the throw; senses trusted less
turn it round. The tie and the statement move alike when they start
alike: one throw cannot tell them apart, only their base rates can.

## 2. The same throw again

Senses at 0.6, bases 0. Belief in the pair, and in each part:

| Throws | By me alone | By different people |
|---|---|---|
| 1 | 0.6 | 0.6 |
| 2 | 0.6 | 0.75 |
| 4 | 0.6 | 0.86 |
| 6 | 0.6 | 0.9 |

## 3. A ball and a balloon, one statement

Two pairs share `falls`. The ball falls, the balloon does not.

| a | s | `falls` | `ball applies` | `balloon applies` | `throw balloon` | Suspects |
|---|---|---|---|---|---|---|
| 0 | 1 | (1, 0) | (1, 0) | (0, 1) | unmoved | the balloon's tie first |
| 0 | 0.9 | (0.9, 0) | (0.9, 0) | (0, 0.81) | unmoved | the balloon's tie first |
| 0.5 | 0.9 | (0.815, 0.004), P 0.91 | (0.9, 0) | (0, 0.747) | (0, 0.083) | the balloon's tie, the throw, then `falls` |

`falls` was expected at 0.95 after the ball, so it takes almost none
of the balloon's no (a share of 0.04); the balloon's tie takes 0.75.

## 4. One object, two statements

Two pairs share `applies`. It falls, and makes no noise. The mirror,
with the same numbers: `applies` stays believed through the fall,
`noise` takes the no, and `noise` is the first suspect.

## 5. A relation that is itself said

A relation of a model is a statement whose content is the relation:
`Statement(Piece("because"), Related(All, [falls, gravity]))`. What is
computed on it counts as far as that statement is believed. Believed
at 1 it is plain structure, which is what scenarios 1 to 4 use.

The toy: `gravity` ("things are pulled down") is a latent statement,
with no tie and nothing to throw at. `because` says that `falls` is
all of [`gravity`]: it falls because of gravity, and only because of
it. I see the object fall, for sure.

| `because` believed at | `falls` | `gravity` |
|---|---|---|
| 1 | (1, 0) | (1, 0) |
| 0.5 | (1, 0) | (0.5, 0) |
| 0 | (1, 0) | unmoved |
| not said how far | (1, 0) | unmoved: nothing is assumed |

And a relation that can be doubted is a suspect: a failed throw with
the pair itself believed at 0.5 lists the pair first, then the tie and
the statement, then the throw.

Evidence was already this for `about`: an `Evidence` is a piece that
says something about another piece, and it counts as far as it is
believed. So both relations of the core are now said by pieces.

## What still reads odd

1. **The pair is not the product of its parts after a yes.** One
   throw at 0.9 gives the pair 0.9 and each part 0.9; multiplied back
   they would give 0.81, the one throw counted twice. The pair keeps
   0.9 because what is said of it directly is more certain. The rule
   "whichever is more certain" is mine.
2. **Two suspects expected wrong take no blame.** With the tie and
   the statement both at base rate 0, a failed throw puts disbelief on
   neither: the chance that one of them *alone* is wrong is 0, since
   the other is expected wrong too. They stay at P 0 with d 0. The
   shares add up to 1 only when at most one suspect is expected wrong.
3. **A throw trusted at 1 is still doubted when all were at 1.** With
   everything expected at 1 and a failure, each takes a third, the
   throw included. Nothing else can be said, but a "1" did move.
4. **A pair with no evidence reads at its own base rate**, 0, even
   when both parts are expected at 1.
5. **Repeating a throw myself adds nothing.** Voice and act are one
   thing today.
6. **A shared no costs a little belief on a believed piece.** In
   scenario 3 at 0.5, `falls` goes from b 0.9 to 0.815: its small
   share of the no comes from my own voice, and one voice counts once,
   so it is averaged with my yes instead of added.
7. **What each was expected to be is read before any no.** With two
   failures in one model, each is shared as if the other had not
   happened.

No longer odd: "a no alone disbelieves nothing" and "blame uses
belief, not expectation" are both gone with the sharing rule.

## Open on relations

- A statement with no tie (a latent part of the system) is linked by
  a relation between statements. Scenario 5 does it with `All`, read as
  "because, and only because". Whether "because" deserves a relation
  of its own, with its own computation, is not settled.
- `Exclude` is still in the record and the toy never uses it. The one
  case I see for it, two statements that contradict on paper, could be
  computed by the seam instead.
- A tie without a statement cannot be forbidden by the records as they
  are: a tie names a statement by its piece.
