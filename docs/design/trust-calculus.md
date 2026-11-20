# Trust: pieces, opinions, evidence

A note to read before any code. It states the innermost part of
Mitra and the calculus on it, shown on the 22 goldens of
`examples/trust/`. The same thing as a model, to read beside it:
[pieces.mitra](pieces.mitra) (a draft: Mitra reads it and its twelve
claims hold by computation; no code is tied to it).

Agreed (2026-11-20): "the atomic thing is a piece, as in a piece of
knowledge about a system." "On a piece, we put a trust (b,d,u,a)
which is what is believed about this piece." "evidence is a piece.
the content of evidence is an opinion about another piece." "a is
what is set by settings or given by author of the evidence. b, d, u
are computed." Reading A: the policy decides the word.

## Two levels

| Level | What is there | Written as |
|---|---|---|
| values | what a model computes with: numbers, records, results | types, `fn`, expressions |
| pieces | what a model is made of: things said, right or wrong | declarations, named by path |

`Piece` is not a type of values and not a top type like Java's
`Object`: no function of a model takes one. A claim, an ensure, a
capture, an assume, a tie and a piece of evidence are pieces. In the
engine a piece is a node with an id, in a module that knows nothing
of syntax. This is the coordinator's answer to your last question
and mine too; you have not confirmed it.

## The three things

```mitra
record Piece;                                             // opaque: known by its name
record Opinion { b: Float64, d: Float64, u: Float64, a: Float64 }
record Evidence { about: Piece, value: Opinion, by: String }
```

| | Means |
|---|---|
| b | belief: evidence that the piece is right |
| d | disbelief: evidence that it is wrong |
| u | uncertainty: no evidence either way; b + d + u = 1 |
| a | the base rate: what to expect where nothing is known |
| P = b + a·u | the number shown, `projected` |

An evidence `e` carries two opinions, and they are not the same:
`e.value` is what it **says** about `e.about`; `opinion(e)` is how
far `e` **itself is believed**. A test run says "yes" about its
claim; whether the test is correct is the opinion of that evidence.
A review of the test is evidence about the evidence. "Passing and
correct" is this, for every kind.

And one more thing between pieces, a **relation**: this piece *rests
on* those; these *cannot all be right*.

## The operations

```text
piece_id(x)             its name
opinion(known, x)       what is believed of x, from all that is known
trusted(known, x, p)    opinion(x) reaches the policy p
```

`opinion(x)` is computed in six steps. Steps 1, 2, 4 and the two
ends of 3 are Jøsang's subjective logic. The middle of 3 is mine.

**1. With nothing, the base rate.** `nothing(a) = (0, 0, 1, a)`, so
P = a. The `a` of a piece comes from the settings by its kind (a
test 0.6, a claim 0) or from its author: the certainty a reviewer
states is the base rate of their review. "Sure at 0.8" is a review
whose opinion is `nothing(0.8)` until someone reviews the review.

**2. Evidence moves an opinion as far as it is believed.**
`moved(opinion(e), e.value)`: what `e` says, multiplied by P of `e`.
A "yes" from evidence believed at 0.8 is (0.8, 0, 0.2); a "no" is
(0, 0.8, 0.2); anything from evidence believed at 0 is (0, 0, 1) and
moves nothing. So "I do not know" is not "it is wrong", and a test
believed wrong does not make its claim false.

**3. Several evidences about one piece are fused, as far as their
voices are independent.** With independence i, 0 for one voice and 1
for two kinds:

```text
k = u1 + u2 - i·u1·u2
b = (b1·u2 + b2·u1) / k      d = (d1·u2 + d2·u1) / k      u = (2 - i)·u1·u2 / k
```

At i = 1 this is cumulative fusion: they add up. At i = 0 it is
averaging fusion: the same voice twice counts once. Between the two
it is **my interpolation**, exact at both ends and not from the
literature; the order of fusion can matter there.

**4. A piece that rests on others is all of them.** `all(x, y)`
believes b1·b2. One piece is counted once, however many paths lead
to it.

**5. Pieces that cannot all be right.** When each is believed
(b1·d2 + d1·b2 > 0 after fusion, or two claims that give two answers)
Mitra reports the conflict with the pieces ordered by 1 - P, the
least believed first. Nothing is chosen silently.

**6. The word.** `trusted`: P reaches the policy, and b > 0. The
claim reads `holds` when trusted, `fails` when d > 0 and d ≥ b,
otherwise `unverified`.

### One golden, step by step (case 3: bias 0.5, policy 0.9)

```text
r   review  { about: t,     value: yes }   opinion(r) = nothing(0.8)                          P 0.8
t   test    { about: wraps, value: yes }   opinion(t) = nothing(0.5) + moved(r) = (0.8, 0, 0.2, a 0.5)   P 0.9
wraps       a claim                        opinion    = nothing(0)   + moved(t) = (0.9, 0, 0.1, a 0)     P 0.9, trusted
```

## On the 22 goldens

Bias 0.5 and policy 0.9 as recorded, unless said. **Bold** changes.

| Case | Today | With the calculus |
|---|---|---|
| 1 passing, no review | unverified 0.5 | same |
| 2 reviewed at 1 | holds 1 | same |
| 3 reviewed at 0.8 | unverified 0.8 | **holds 0.9** |
| 11 reviewed at 0.5 | unverified 0.5 | **unverified 0.75** |
| 12 reviewed at 0.3 | unverified 0.3 | **unverified 0.65** |
| 20 reviewed at 0 | unverified 0 | **unverified 0.5**: "I do not know" is no review |
| 10 reviewed as wrong at 1 | unverified 0 | same |
| 4 failing, no review | fails 0.5 | same; d = 0.5 |
| 9 failing, reviewed correct | fails 1 | same |
| 5 policy 0.4 | holds 0.5 | same |
| 6 derived, one point reviewed | 1, 0.5, promise 0.5 | same, until reviews of claims are weighed |
| 7 no bias | holds, no number | depends on the default you choose |
| 8 two reviews disagree, both sure | holds 1 | **unverified 0.5, and a conflict naming both reviews** |
| 13 two claims disagree | "two outputs", the code blamed | **a conflict: `stops` and its unreviewed test (0.5) first, `wraps` and its reviewed test (1) last** |
| 14 bias 0 | unverified 0 | same |
| 15 bias 1 | holds 1 | same |
| 16 policy 0 | holds 0.5 | same |
| 17 policy 0, bias 0 | unverified | same: trusted needs b > 0 |
| 18 policy 1, reviewed at 0.99 | unverified 0.99 | **unverified 0.995** |
| 19 policy 1, reviewed at 1 | holds 1 | same |
| 21 failing at bias 0 | unverified | same |
| 22 failing, reviewed as wrong | unverified | same |

Seven change. One to look at twice: **12**. A review "correct, but
only 0.3 sure" used to lower the test under its bias. Now it raises
it a little. To lower a test, a reviewer says it is wrong.

## Default settings, proposed

```toml
[bias]            # the base rate a, per kind of piece
test = 0.6        # "most of the test written are correct before being reviewed"
claim = 0         # nothing assumed of a claim with no evidence
review.human = 0.99
review.cold = 0.95        # an agent with the model and the code only
review.agent = 0.8        # an agent with the author's context
review.unknown = 0.5
proof = 0.99

[policy]
holds = 0.9
```

A review rate is used when its author states no certainty. One
passing test then reads 0.6 unreviewed (unverified), 0.92 reviewed by
an agent, 0.98 by a cold agent, 0.996 by a person (all hold), and
0.03 reviewed "wrong" by a cold agent. A hold is easy, as you wanted.
Only the 0.6, the 0.9 and "very high" are yours.

## The experiment: unreviewed tests alone

How many passing tests nobody reviewed make one claim hold at 0.9.

| Bias | Fully independent | Holds at |
|---|---|---|
| 0.5 | 0.5, 0.67, 0.75, 0.8 … | 9 tests |
| 0.6 | 0.6, 0.75, 0.82, 0.86, 0.88, 0.9 | 6 tests |
| 0.8 | 0.8, 0.89, 0.92 | 3 tests |

| At bias 0.6, independence | 1 test | 2 | 4 | 8 | Ever holds |
|---|---|---|---|---|---|
| 1 (two kinds of voice) | 0.6 | 0.75 | 0.86 | 0.92 | at 6 |
| 0.75 | 0.6 | 0.71 | 0.79 | 0.83 | no |
| 0.5 | 0.6 | 0.67 | 0.72 | 0.74 | no |
| 0.25 | 0.6 | 0.63 | 0.66 | 0.67 | no |
| 0 (one voice) | 0.6 | 0.6 | 0.6 | 0.6 | no |

One agent writing ten tests never gets past 0.6 without a review.
The ceilings of the middle rows come from my interpolation and are
not derived from anything.

## Open, for you

1. **Piece as a level apart**, not a type: confirm or not.
2. **Independence as a number.** `voices` orders distance: same,
   session, model, vendor, kind. Proposed: 0, 0.25, 0.5, 0.75, 1. A
   voice nobody named is independent of nothing.
3. **A review by the test's own author**: proposed, independence 0,
   it adds nothing.
4. **The interpolation** in step 3. The alternative is two levels
   only: independent or not.
5. **`fails` at d ≥ b.** One unreviewed failing test still fails its
   claim. A failing test against a reviewed passing one no longer
   does, and shows as a conflict.
6. **Evidence on a piece and what it rests on**: two arguments for
   one piece. Proposed for now: the better one, as on main, not fused.
7. **With no `[bias]` in the settings**: these defaults, or bias 1
   as today.

## To build after your read

The module of pieces first, alone: piece, opinion, evidence,
relation and the functions of `pieces.mitra`, tied to that model.
Then `confidence.rs` on top of it, the seven goldens recorded again,
and the experiment as a golden. Where `voices` and a `trust` subject
go is yours to pick, and nothing moves before.
