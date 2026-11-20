# The throwing toy

The smallest model we could think of, used to settle one relation and
its computation. Every row below is what the core computes, pinned in
`tests/throwing.rs`; the toy is also five claims of `model.mitra`.

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
| down, a no | shared: a part is disbelieved as far as the others are believed | part gets `(0, d · b of the other parts)`, their belief taken from other evidence |
| the whole itself | what is said of it directly, or its parts together | whichever is more certain (less `u`) |

The **conflict is computed, not declared**: evidence saying no about
a whole means its parts and that evidence cannot all be right. They
are listed, the least believed first (by `P = b + a·u`).

## 1. One throw

`a`, `f`: base rates of the tie and the statement. `s`: trust in my
senses. An opinion is written (b, d).

| a | f | s | I saw | `lands` | `applies` and `falls` | Suspects, least believed first |
|---|---|---|---|---|---|---|
| 0 | 0 | 1 | fall | (1, 0) | (1, 0) | |
| 0 | 0 | 1 | no fall | (0, 1) | unmoved | `applies` 0, `falls` 0, `throw` 1 |
| 1 | 1 | 1 | fall | (1, 0) | (1, 0) | |
| 1 | 1 | 1 | no fall | (0, 1) | unmoved | all three at 1: no order |
| 1 | 1 | 0 | no fall | (0, 0) | unmoved | none: senses not trusted |
| 0 | 0 | 0 | fall | (0, 0) | unmoved | |
| 0.5 | 0.5 | 0.9 | fall | (0.9, 0) | (0.9, 0), P 0.95 | |
| 0.5 | 0.5 | 0.9 | no fall | (0, 0.9) | unmoved | `applies` 0.5, `falls` 0.5, `throw` 0.9 |
| 0.9 | 0.5 | 0.6 | no fall | (0, 0.6) | unmoved | `falls` 0.5, `throw` 0.6, `applies` 0.9 |
| 0 | 0 | 0.6 | fall | (0.6, 0) | (0.6, 0) | |

All ten read right to me. The tie and the statement always move
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

| a | s | `falls` | `ball applies` | `balloon applies` | Suspects |
|---|---|---|---|---|---|
| 0 | 1 | (1, 0) | (1, 0) | (0, 1) | the balloon's tie first |
| 0 | 0.9 | (0.9, 0) | (0.9, 0) | (0, 0.81) | the balloon's tie first |
| 0.5 | 0.9 | (0.9, 0) | (0.9, 0) | (0, 0.81) | the balloon's tie, the throw, then `falls` |

## 4. One object, two statements

Two pairs share `applies`. It falls, and makes no noise. The mirror:
`applies` (s, 0) through the fall, `noise` (0, s·s), and `noise` is
the first suspect.

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
2. **A no alone disbelieves nothing.** With one object and one model,
   a failed throw leaves the tie and the statement exactly where they
   were and only names them as suspects. That is the
   non-identifiability we agreed on, and it means a claim that fails
   its only test is "suspect", not "disbelieved".
3. **Blame uses belief, not expectation.** A part with a high base
   rate and no evidence does not push blame on the other part. Using
   `P` there would make disbelief out of nothing but base rates.
4. **A pair with no evidence reads at its own base rate**, 0, even
   when both parts are expected at 1.
5. **Repeating a throw myself adds nothing.** Voice and act are one
   thing today.

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
