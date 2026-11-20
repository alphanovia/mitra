# The throwing toy

What the core computes today on the smallest model we could think of,
row by row, and whether each row reads right. Pinned in
`tests/throwing.rs`. Nothing in the calculus was changed for it.

## The toy, in today's pieces

| Piece | Kind | Says |
|---|---|---|
| `falls` | statement | thrown up, it goes up then falls down |
| `applies` | tie | this model is a good model of this object |
| `lands` | a fourth piece | this object, thrown, falls: `falls` and `applies` together |
| `throw` | evidence about `lands` | I threw it and saw it fall, or not; believed as far as I trust my senses |

One relation: `lands` rests on [`applies`, `falls`]. The fourth piece
is needed because evidence is about one piece, and a throw tests the
statement and the tie together.

## 1. One throw

`a`, `f`: base rates of the tie and the statement. `s`: trust in my
senses. An opinion is written (b, d); the rest is uncertainty.

| a | f | s | I saw | `lands` today | `applies`, `falls` today | Reads right | Expected |
|---|---|---|---|---|---|---|---|
| 0 | 0 | 1 | fall | (1, 0) | unmoved, P 0 | **no** | both (1, 0): the pair is true, so each part is |
| 0 | 0 | 1 | no fall | (0, 1) | unmoved | **half** | right, plus "one of `applies`, `falls` is wrong" |
| 1 | 1 | 1 | fall | (1, 0) | unmoved, P 1 | yes | |
| 1 | 1 | 1 | no fall | (0, 0) | unmoved, P 1 | **no** | `lands` (0, 1), and a conflict between the three, unranked |
| 1 | 1 | 0 | no fall | (0, 0) | unmoved | yes | senses not trusted: nothing moves |
| 0 | 0 | 0 | fall | (0, 0) | unmoved | yes | |
| 0.5 | 0.5 | 0.9 | fall | (0.9, 0) | unmoved, P 0.5 | **no** | both (0.9, 0), P 0.95 |
| 0.5 | 0.5 | 0.9 | no fall | (0, 0) | unmoved | **no** | `lands` (0, 0.9); suspects: `applies` 0.5, `falls` 0.5, then `throw` 0.9 |
| 0.9 | 0.5 | 0.6 | no fall | (0, 0) | unmoved | **no** | `lands` (0, 0.6); suspects: `falls` 0.5, `throw` 0.6, `applies` 0.9 |
| 0 | 0 | 0.6 | fall | (0.6, 0) | unmoved, P 0 | **no** | both (0.6, 0) |

Three faults, all in how a whole and its parts stand:

1. **Nothing flows down.** Belief goes from parts to the whole and
   never back. A throw that succeeds for sure leaves the statement and
   the tie unbelieved.
2. **A "no" is dropped.** When the parts have a base rate above 0, a
   piece is read from its parts (their product reads higher than
   disbelief does), and the evidence against it disappears. With
   base rates at 0 it is kept. A sign error of the rule "its own
   evidence or what it rests on, whichever is more".
3. **No conflict is derived.** `conflicts` finds declared exclusions
   and opposite evidence on one piece. A failed throw is neither.

The non-identifiable pair does behave alike: `applies` and `falls`
never move differently from one throw. Only their base rates tell
them apart, which is what we agreed.

One more, smaller: `lands` reads P 0 when both its parts read P 1 and
nothing was thrown. Its base rate is its own (0) and not that of its
parts. The claim `base_rate_survives` says so on purpose; for a piece
that only rests on others it looks wrong.

## 2. The same throw again

Senses at 0.6, bases 0. Belief in `lands`:

| Throws | By me alone | By different people |
|---|---|---|
| 1 | 0.6 | 0.6 |
| 2 | 0.6 | 0.75 |
| 4 | 0.6 | 0.86 |
| 6 | 0.6 | 0.9 |

Reads right for the pair. Open, as before: repeating a throw myself
adds nothing at all, which is the "same voice" end of independence. A
second throw by me is a second act, though. Voice and act are one
thing today.

## 3. A ball and a balloon, one statement

`ball lands` rests on [`ball applies`, `falls`], `balloon lands` on
[`balloon applies`, `falls`]. The ball falls, the balloon does not.

| a | s | Today | Expected |
|---|---|---|---|
| 0 | 1 | wholes (1, 0) and (0, 1); no part moves | `falls` (1, 0) through the ball, so `balloon applies` (0, 1): the tie is wrong, for sure |
| 0 | 0.9 | wholes (0.9, 0) and (0, 0.9); no part moves | `falls` (0.9, 0), `balloon applies` (0, 0.81) |
| 0.5 | 0.9 | `balloon lands` dropped to (0, 0) | the same as the row above |

## 4. One object, two statements

`lands` rests on [`applies`, `falls`], `bangs` on [`applies`,
`noise`]. It falls, and makes no noise. Today and expected are the
mirror of scenario 3: `applies` should be believed through the fall,
and `noise` should take the blame at 0.81. Today no part moves.

So sharing does separate the pair, but only once belief flows down:
the 0.81 is "how sure I am it did not, times how far the other part
is believed on other evidence".

## What it says about the relation record

- The toy uses **one kind only: rests on, all of**. Nothing declares
  an exclusion.
- The conflict {`applies`, `falls`, `throw`} **cannot be derived on
  `core` today**. It could be, from the same records: evidence saying
  no about a piece that rests on others means the evidence and those
  others cannot all be right.
- Is "cannot all be right" ever declared with no evidence? The one
  case I see, two statements that contradict on paper, is computed by
  the seam (their expressions disagree at some input), so it is
  evidence too. `Exclude` may not need to be a declared relation.
- "Any of": several supports of one piece are fused evidence, or
  several `rests on` of which the better counts. No third kind needed
  for the toy.

## Two rules that would make the tables read right

Not coded. For you to accept, change or replace.

- **Yes flows down.** Evidence that believes a piece believes each
  piece it rests on as much. (Check: the pair recomputed from its
  parts would then read 0.81 for one throw at 0.9, the throw counted
  twice. The whole must keep its own evidence.)
- **No is shared.** Evidence that disbelieves a piece at `d` gives
  each part `d` times the belief of the other parts from other
  evidence, and reports the parts and itself as a conflict, least
  believed first. Its own evidence against a piece is never dropped.
