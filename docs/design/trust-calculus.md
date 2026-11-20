# Trust: the calculus

A note to read before any code. It replaces the product of the first
step with opinions of subjective logic, one operator at a time, each
shown on the 22 goldens of `examples/trust/`.

Decided (2026-11-20): opinions (b, d, u, a). "a is what is set by
settings or given by author of the evidence. b, d, u are computed on
the network of evidence and all things that can be judged." Reading
A: the policy decides the word. "Be cautious about the operations."

## An opinion

About anything that can be judged: a test is correct, a review is
right, a claim holds.

| | Means |
|---|---|
| b | belief: evidence that it is so |
| d | disbelief: evidence that it is not |
| u | uncertainty: no evidence either way; b + d + u = 1 |
| a | the base rate: what to expect where nothing is known |
| P = b + a·u | the number shown, and compared with the policy |

With no evidence an opinion is (0, 0, 1, a) and P = a. This is where
the bias lives: an unreviewed test is (0, 0, 1, 0.6), so P = 0.6.

## The operators

Six, in the order a verdict is computed. Operators 1, 2, 4 and the
two ends of 3 are Jøsang's. The middle of 3 is mine and is flagged.

**1. A review is a statement, weighed by how sure its author is.**
"Correct, sure at c" gives the test (c, 0, 1 - c). "Wrong, sure at c"
gives (0, c, 1 - c). The c is what the author wrote, or the base rate
of that kind of reviewer when they wrote none. This is discounting:
the statement is cut down to what its source is believed for.

So "I do not know" (c = 0) is (0, 0, 1): nothing changes. And "it is
wrong" is disbelief. The product could not tell them apart.

**2. A test speaks for its claim as far as it is correct.** A test
that passes says "the claim holds here"; one that fails says "it does
not". Either is cut down to P of the test: the claim gets (P, 0,
1 - P) from a pass and (0, P, 1 - P) from a failure. A test believed
wrong says nothing of the claim: it does not make the claim false.

**3. Two opinions on the same thing are fused, as far as they are
independent.** With independence i from `voices`, 0 for the same
voice and 1 for two kinds:

```text
k = u1 + u2 - i·u1·u2
b = (b1·u2 + b2·u1) / k      d = (d1·u2 + d2·u1) / k      u = (2 - i)·u1·u2 / k
```

At i = 1 this is cumulative fusion: two independent opinions add up.
At i = 0 it is averaging fusion: the same voice twice counts once.
Between the two it is **my interpolation**, exact at both ends and
not from the literature; the order of fusion can matter there. When
both are certain (u = 0) and disagree, the result is their average
and Mitra reports a conflict.

**4. What needs several things is their conjunction.** A claim that
follows from two claims, or reads two tested points, is believed as
b1·b2. One piece of evidence is counted once in a verdict, however
many paths lead to it: this is the false independence to avoid.

**5. The base rate of a claim is 0.** A claim nobody brought evidence
for has P = 0: nothing is assumed. So for a claim P = b. A setting,
to move if you want claims to start higher.

**6. The word.** `holds` when P reaches the policy. `fails` when
d > 0 and d ≥ b: the evidence against is at least as strong as the
evidence for. Otherwise `unverified`. When both b and d are present
Mitra also raises the conflict as an error, with the pieces ranked
by 1 - P, the least believed first.

## On the 22 goldens

Bias 0.5 and policy 0.9 as recorded, unless said. **Bold** changes.

| Case | Today | With the calculus |
|---|---|---|
| 1 passing, no review | unverified 0.5 | same |
| 2 reviewed at 1 | holds 1 | same |
| 3 reviewed at 0.8 | unverified 0.8 | **holds 0.9**: 0.8 + 0.5 × 0.2 |
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
| 17 policy 0, bias 0 | unverified | same: holding needs b > 0 |
| 18 policy 1, reviewed at 0.99 | unverified 0.99 | **unverified 0.995** |
| 19 policy 1, reviewed at 1 | holds 1 | same |
| 21 failing at bias 0 | unverified | same |
| 22 failing, reviewed as wrong | unverified | same |

Seven change. One to look at twice: **12**. A review that says
"correct, but I am only 0.3 sure" used to lower the test under its
bias. Now it raises it a little, to 0.65. To lower a test a reviewer
says it is wrong. I think that is the right reading of a review and
it is a change of behaviour.

## Default settings, proposed

```toml
[bias]            # the base rate a, per kind
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

The review rates are how sure a review counts when its author wrote
no number. What they give for one passing test:

| The test is | P | Word at 0.9 |
|---|---|---|
| not reviewed | 0.6 | unverified |
| reviewed by an agent, no number | 0.92 | holds |
| reviewed by a cold agent | 0.98 | holds |
| reviewed by a person | 0.996 | holds |
| reviewed "wrong" by a cold agent | 0.03 | unverified |

A hold is easy, as you wanted: any review that does not say "wrong"
is enough. Only the 0.6, the 0.9 and "very high" for cold and human
are yours; the other numbers are mine.

## The experiment: unreviewed tests alone

How many passing tests nobody reviewed, on the same claim, make it
hold at 0.9. Each is (bias, 0, 1 - bias) for the claim; they are
fused at their independence.

| Bias | Fully independent | Then |
|---|---|---|
| 0.5 | 0.5, 0.67, 0.75, 0.8 … | holds at 9 tests |
| 0.6 | 0.6, 0.75, 0.82, 0.86, 0.88, 0.9 | holds at 6 tests |
| 0.8 | 0.8, 0.89, 0.92 | holds at 3 tests |

At bias 0.6, by independence:

| Independence | 1 test | 2 | 4 | 8 | Ever holds |
|---|---|---|---|---|---|
| 1 (two kinds of voice) | 0.6 | 0.75 | 0.86 | 0.92 | at 6 |
| 0.75 | 0.6 | 0.71 | 0.79 | 0.83 | no |
| 0.5 | 0.6 | 0.67 | 0.72 | 0.74 | no |
| 0.25 | 0.6 | 0.63 | 0.66 | 0.67 | no |
| 0 (one voice) | 0.6 | 0.6 | 0.6 | 0.6 | no |

So one agent writing ten tests never gets past 0.6 without a review,
and six tests hold a claim only if six different kinds of author
wrote them. The ceilings in the middle rows come from my
interpolation: they look reasonable, tests that share an author share
its blind spots, and they are not derived from anything.

## Open, for you

1. **Independence as a number.** `voices` orders distance: same,
   session, model, vendor, kind. Proposed: 0, 0.25, 0.5, 0.75, 1. A
   voice nobody named is independent of nothing.
2. **A review by the test's own author.** Proposed: it adds nothing,
   independence 0. Then a test nobody signed can still be reviewed,
   which assumes its author is someone else.
3. **The interpolation** in operator 3. The alternative is two
   levels only: independent or not.
4. **`fails` at d ≥ b.** One unreviewed failing test still fails its
   claim. A failing test against a reviewed passing one no longer
   does, and shows as a conflict.
5. **With no `[bias]` in the settings**: these defaults, or bias 1
   as today.

## To build after your read

The opinion type and the six operators as functions of their own,
each with claims at the extremes (b = 0, u = 0, i = 0 and 1). Then
`confidence.rs` on top of them, the seven goldens recorded again, and
the experiment as a golden of its own. Reviews of claims, samples and
proofs come after, as in [trust-toys.md](trust-toys.md).
