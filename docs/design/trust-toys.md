# Trust: the next toys

A note to read before anything more is built. The first toy, one
claim at one point with one test, is on main (`examples/trust/`, 13
cases). Its rules:

1. A test owes two things: that it passes, and that it is correct.
2. Unreviewed, it is believed at the bias of its kind. Reviewed, as
   far as its reviewer said.
3. A claim holds when what it stands on reaches the policy.

Below, four more toys. For each: the model, what main says today
(run on 207567f, bias 0.5, policy 0.9), what the trust system must
then do, and whether that agrees with the three rules.

## 1. A derivation from an easy claim

```mitra
subject clock {
    fn next(h: 0..=23) -> 0..=23;
    claim wraps() { next(23) == 0 }
    claim moves() { next(23) != 23 }
}
```

One test, tied to `wraps` and reviewed at 1. `moves` has no evidence.

```text
clock::wraps: holds
  confidence      1, the policy asks 0.9
  on              test:midnight_wraps passes, reviewed at 1 by agent:a/b/c
clock::moves: holds
  confidence      1, the policy asks 0.9
  on              test:midnight_wraps passes, reviewed at 1 by agent:a/b/c
```

**Requirement.** A derived claim is believed as what it follows
from, owes nothing of its own, and one review lifts both. Main does
this already.

**Against the rules.** Agrees. One thing to change in the report:
`moves` should read `from clock::wraps 1`, not name the test as if it
were its own. And `mitra next` could say how many claims each owed
review would lift: that is the snowball.

## 2. A claim one point cannot prove

```mitra
subject math {
    fn double(n: Nat) -> Nat;
    claim doubles(n: Nat) { double(n) == n + n }
}
fn small(n: Nat) -> Bool { n < 1000 }
```

With one test at `n = 3`, reviewed at 1:

```text
math::doubles: unverified
  passes on       `math::double(3)`
  no evidence on  n in 0..=2 | 4..
```

No confidence can lift it: the test is fully believed and says
nothing of the other numbers. So a verdict has two sides, **how far
the evidence is believed** and **how much of the claim it covers**.
The first toy only had the first.

### 2a. A sample

```evidence
evidence test { for math::doubles given n in small sampled 3000 by "machine:cargo" source "test:sweep" }
```
```text
math::doubles: unverified
  sampled on      3000 drawn from (n in small) (test:sweep), trust 0.95 at tolerance 0.001
  no evidence on  n in 0..
```

**Requirement.** A sample is a test, so it owes the same two things,
and a third: what its count gives. Its claim is believed at

```text
passes (1) x correct (bias, or its review) x count (0.95 for 3000 at tolerance 0.001)
```

and holds when that reaches the policy. Proposed lines:

```text
math::doubles: holds
  confidence      0.95, the policy asks 0.9
  on              test:sweep passes 3000 draws (0.95), reviewed at 1 by agent:a/b/c
```

**Against the rules.** Agrees with 1 and 2, with one more factor.
Two things it adds:

- A reviewed sample can never reach 1. At a policy of 0.99 this one
  needs 4,603 draws. The tolerance becomes part of the policy.
- This sample draws under 1,000 and the claim is over every number.
  That step is not the sample's to make: it is a claim of its own,
  "what holds under 1,000 holds everywhere", which is toy 2b. Today
  main ignores the sample for that reason, even with `counts = true`.

### 2b. Other properties of the function

```mitra
import std::claims::{uniform};
subject math {
    fn double(n: Nat) -> Nat;
    claim double_uniform = uniform(double);
    claim doubles(n: Nat) { double(n) == n + n }
}
```

The test at `n = 3` reviewed at 1, and a review of `double_uniform`
at 0.9: the code treats every number the same way.

```text
math::double_uniform: holds
math::doubles: holds
  confidence      1, the policy asks 0.9
  on              test:three passes, reviewed at 1 by agent:a/b/c
  passes on       `math::double(3)`
  assumed on      every input, by `math::double_uniform`
```

**Requirement.** The review of the property is evidence like any
other, believed as far as its author said, and the claim is the
product of the point and the property:

```text
math::double_uniform: holds
  confidence      0.9
  on              review by agent:a/b/c, sure at 0.9
math::doubles: holds
  confidence      0.9, the policy asks 0.9
  on              test:three passes, reviewed at 1 by agent:a/b/c
  from            `math::double_uniform` 0.9
```

**Against the rules.** Main **contradicts** them today: the 0.9 is
dropped and `doubles` is believed at 1. Only tests are weighed. To
agree, a review of a claim must carry its author's certainty, your
sentence "the author of the evidence is giving his opinion on how
certain he is". Two questions it opens:

- A review that states no certainty counts as 1 today. Nothing
  should be implicit: either a review must say it, or it takes a
  bias of its kind.
- A review owes no second review. The chain stops at a stated
  opinion; another reviewer is a second opinion to combine, which is
  the case of two reviews that disagree (golden 8, wrong today).

### 2c. A proof

There is an act `proof` in the ledger and no prover wired to it. As
evidence it reads:

```evidence
evidence proof { for math::doubles holds act proof by "machine:lean" source "proof:Double.lean#doubles" }
```
```text
math::doubles: holds
  by              proof (machine:lean); doubt 0.01
```

**Requirement.** A proof owes the same two things as a test. That it
passes: the checker accepted it, a machine's word. That it is
correct: the statement proved is the claim of the model, and the
definitions it used are the model's. The second is a review, or a
bias for proofs nobody read.

```text
math::doubles: holds
  confidence      1, the policy asks 0.9
  on              proof:Double.lean#doubles checked, reviewed at 1 by human:dana
```

**Against the rules.** Agrees exactly, and it covers the whole claim
with no third factor. Unreviewed it would read `unverified` at the
bias of proofs, which looks harsh and is the same rule: nobody
checked that the theorem is the claim.

## Side by side

| Toy | Covers the claim by | Owes | Agrees with the first toy |
|---|---|---|---|
| first | the point is the claim | passes, correct | — |
| 1 derivation | logic | nothing of its own | yes; the report should say `from` |
| 2a sample | its count, up to the tolerance | passes, correct, enough draws | yes, one more factor; never 1 |
| 2b property | a claim about the function | point: passes, correct; property: a stated opinion | **no, today**: the opinion on the property is not weighed |
| 2c proof | the proof itself | checked, and that it proves this claim | yes |

## What this asks to be built, in order

1. Weigh a review of a claim by its stated certainty (2b). It is the
   one contradiction, and 2a's step from a small domain needs it.
2. Weigh a sample: correctness times count (2a), and let it hold its
   claim, as you decided.
3. Weigh a proof like a test (2c), with a `[bias]` per kind: `test`,
   `sample`, `proof`.
4. Say `from` on a derived claim, and count in `mitra next` what each
   owed review lifts (1).
5. Two opinions on one piece of evidence, and two claims that
   disagree (goldens 8 and 13, both wrong today).
