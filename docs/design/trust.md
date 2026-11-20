# The trust system

A proposal to settle. Nothing in "to decide" or "to build" is coded.
The examples are the real report of the clock rung
(`ladder/1-clock/model/expected.txt`).

## Decided

Your words, 2026-11-20 unless dated.

- **A sample can hold a claim.** "Yes sample can hold a claim. It's a
  legitimate test."
- **A verdict carries a number.** "It holds when we are satisfied
  about the evidences given that it holds. The number mean the
  evidence is good enough to believe that it holds or fails." It is
  not how true the claim is.
- **Same structure as culprit ranking.** An argument is a set of
  pieces that must all be right; several arguments for one claim
  reinforce each other. "Yes let's try the same structure."
- **Evidence is typed on the claim it supports.** A test at a point
  cannot hold a claim over a domain. A review is "a proof with low
  confidence". (2026-11-19)
- **Voices.** Evidence says what was done and by whom: an agent, a
  cold agent, a person, a machine. Each kind has an error rate in the
  settings (agent 0.5, cold 0.1, person 0.05, unknown 0.5), and a
  policy says what a claim and a promise need. (on main)
- **Everything can be a suspect**, and nothing is assumed silently.
- Tentative, yours: "Maybe in the report we put the assuming on the
  claim supported by evidence, and we put a plain hold on derived
  ones?"

## What main does today

Three numbers that never meet:

| Number | Where it comes from | What it does |
|---|---|---|
| `trust n` | written on evidence, a claim, a tie | ranks suspects in an error; 1 when not said |
| the trust of a sample | its count and the tolerance | printed under the claim; holds nothing |
| `doubt` | the voices behind a verdict | printed on the `by` line; compared with the policy |

The verdict word comes from none of them. On the clock:

```text
clock::next::correct: unverified
  passes on       `clock::next(23)`
  no evidence on  h in 0..=22
clock::steps: holds
  by              exhaustive (agent), test (agent); doubt 0.3
clock::one_more: holds
  by              review (agent), sample (agent); doubt 0.3
clock::jumps::captured: holds
  by              review (agent); doubt 0.3
clock::jumps: holds assuming [clock::next::correct, clock::back::correct, clock::after::correct]
  by              exhaustive (agent), review (agent), sample (agent), test (agent); doubt 1
  rests on        next::correct, back::correct, after::correct, steps, midnight, round_trip, stays_put, one_more
```

- Without the review, `one_more` is `unverified` under a sample of
  3,000 trusted at 0.95.
- The promise is at doubt 1 whatever is done: it assumes the `correct`
  of three functions that nothing can support.
- In an error every piece is trusted at 1, so the suspects are each
  left at 0.50: the doubt of a voice is not used.

## To decide

In each mock-up a piece of evidence is believed as far as its voice:
0.70 for this agent (the report's doubt 0.3 turned around), 0.95 for
the sample of 3,000.

### 1. The wording

```text
First                                   Second
clock::steps: holds assuming 0.70       clock::steps: holds, confidence 0.70
  by  exhaustive test, agent (0.70)       on    exhaustive test, agent (0.70)
clock::jumps: holds                     clock::jumps: follows, confidence 0.34
  rests on  stays_put 0.70, one_more      from  stays_put (0.70), one_more
    0.70, captured 0.70; together 0.34          (0.70), captured (0.70)
```

- First: the number is on the claim evidence supports; a derived
  claim is a plain `holds` with the product under it. Your tentative
  idea. `assuming` is also the word of a boundary's `assume`.
- Second: `holds` where evidence is on the claim, `follows` where it
  is logic. `holds assuming [x]` stays only for what leans on an
  `assume` nobody checked.

### 2. Below the policy

Say the policy asks 0.9 of a claim.

```text
A   clock::steps: unverified, evidence 0.70, policy asks 0.9
B   clock::steps: holds, confidence 0.70
      below policy: asks 0.9
```

- A: the word is `holds` only when someone was satisfied, your
  sentence. On the clock today every line would read `unverified`,
  since an agent alone signed everything.
- B: the word says what the evidence says, the policy is a second
  line. Nothing flips when a setting moves.

### 3. One voice, counted once or multiplied

Two places.

Inside a claim, `one_more` has a sample (0.95, the test written by the
agent, 0.70) and the same agent's review (0.70):

```text
multiplied    two arguments, 0.67 and 0.70, reinforce: 1 - 0.33 * 0.30 = 0.90
counted once  both stand on one agent: 0.70
```

Across a promise, `jumps` follows from three pieces, each 0.70 and
each the same agent's:

```text
multiplied    0.70 * 0.70 * 0.70 = 0.34
counted once  0.70
```

- Multiplied is the structure of culprit ranking as it is. It
  rewards an agent for reviewing its own tests, and punishes a
  promise for having many claims.
- Counted once is what `doubt` does today: one factor per voice that
  is its own. A second voice (a cold agent, a person) is what moves
  the number.

### 4. What a promise rests on

Today: every claim of its subject and the `correct` of every
function.

```text
a   every claim of its subject, no `correct`
      clock::jumps  from  steps, midnight, round_trip, stays_put, one_more, captured
b   the claims it names
      ensure jumps = "Moving n hours at once gives what n single steps give." by stays_put, one_more;
      clock::jumps  from  stays_put, one_more, captured
```

- a: nothing to write. One failing claim still fails every promise of
  its subject.
- b: one more clause, proposed syntax. A failure fails the promises
  it serves, and the capture review reads two claims, not five. The
  cold agents refused every capture because the promises listed no
  claims.
- In both, `f::correct` stays a line of the report, owed, and no
  promise leans on it.

## To build once settled

1. **One number.** `trust`, the trust of a sample and `doubt` become
   the one confidence, computed as decided in 3, printed as decided
   in 1, compared with the policy as decided in 2.
2. **A sample holds its claim**, at the confidence its count gives
   times its author's.
3. **A promise rests on** what 4 decides.
4. **Diagnose**, from rungs 1 and 2 (12 bug reports, none diagnosed
   sensibly):
   - the bug report is a suspect with a voice, `unknown` unless it
     says who;
   - a witness makes `fails`: one observed output against a claim at
     that input is a failure, not `holds -> unverified`;
   - the code of the function is named among the suspects;
   - the suspects are weighed by their voices, not all at 1;
   - an output that differs from a test's on the same input is first
     a wrong output, and only then "two outputs for one input".
5. **A boundary's `assume`**: one good answer counts as a sample of
   one (today it makes the assume hold); one bad answer makes it fail
   (today it stays unverified).
6. One failing test that pins four outputs fails the function whose
   output differed, not the four.

Each is checked on the two rungs: their expected reports change with
it, and their 13 breaks say whether the culprits make sense.
