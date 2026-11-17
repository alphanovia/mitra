# Use cases

Three concrete situations Mitra is for. They are ordered by when Mitra can
serve them: UC1 is what comes first, UC2 and UC3 are the reasons the engine
exists and shape its design now, but are built later.

The snippets use the draft syntax from [language.md](language.md).

---

## UC1: Check what a trust model computes (first)

**Actor:** a developer of Mitra, or of a program that will use Mitra.
**Goal:** write a small model, see the trust value and why, and be sure it is
right.

**Scenario.** The developer models a sorting function: the claim "`sort` is
correct" needs two sub-claims, "output is ordered" and "output is a
permutation of the input". The first has a Lean proof; the second only has
three example-based tests. The proof rests on the assumption "the Lean model of
`sort` matches the Rust code".

```mitra
claim sort_correct "sort returns a sorted permutation of its input"
claim ordered      "output is ordered"
claim permutation  "output is a permutation of the input"
claim model_ok     "the Lean model of sort matches the Rust code"

assume   ordered     = (1, 0, 0)     # Lean proof
assume   model_ok    = (0.8, 0, 0.2)
evidence permutation : +3            # three example tests
sort_correct = ordered and model_ok and permutation
```

**What Mitra returns.** `sort_correct` gets a moderate value with high
uncertainty, and the limiting element is `permutation`: three example tests
are thin. Raising it (say, a property-based test) is what helps most; a second
proof of `ordered` would barely move the result.

**Why it matters.** This is how the language is validated, the way subjective logic is
validated: small nominal and corner-case models whose expected outputs are
known, plus proven properties ("a conjunction is never trusted more than its
weakest leg"). Every toy model in [examples.md](examples.md) is an instance of UC1, and
a demo app would be UC1 with a screen.

---

## UC2: Gate a merge on the trust earned by a change (later, product case)

**Actor:** a CI pipeline.
**Goal:** decide automatically whether a change can merge, needs a human, or
is blocked, and say what evidence is missing.

**Scenario.** A pull request, partly vibe-coded, claims "token refresh no
longer logs users out". Evidence: unit tests written by the same agent that
wrote the code, an existing integration test that touches the flow, and no
review yet. Assumption: "staging uses the production token lifetime", which
nobody has checked.

**Policy, written by the team:**

```text
merge     if  P(claim) >= 0.90 and uncertainty <= 0.15
ask human if  uncertainty > 0.15
block     if  disbelief > 0.20
```

**What Mitra returns.** High uncertainty, low disbelief: nothing contradicts
the claim, but little supports it independently, and the unchecked assumption
discounts the integration test. CI routes the change to a human and reports
the limiting element: "assumption `staging_token_lifetime` is unverified".
Checking that one assumption would let the change merge on its own.

**What this needs beyond the language:** turning a change's tests, proofs and reviews
into a model automatically. That is the v2 product case.

---

## UC3: Decide how much autonomy to give an agent (later, agent case)

**Actor:** an agent harness.
**Goal:** after each step, choose between letting the agent continue, asking
the agent for more evidence, asking a human, or stopping it.

**Scenario.** An agent reports: "Done. I only changed files under `src/`, and
all tests pass." The harness turns this into claims, each with evidence of
different strength:

| Claim | Evidence | Strength |
|---|---|---|
| Only `src/` changed | Harness inspected the diff | Strong, independent of the agent |
| All tests pass | Agent's own report | Weak, discounted by the agent's track record |
| Task is done as asked | Agent's own report | Weak |

**What Mitra returns.** The first claim is trusted, the other two are
uncertain rather than distrusted. Because uncertainty is high but disbelief is
low, the policy's answer is not "stop" but "get more evidence": the harness
runs the tests itself, which is the cheapest action that raises trust the
most. If the harness's own run fails, disbelief jumps and the agent is
stopped, and the agent's track record drops for the next decision.

**Why this shapes the language now:** it is the case that needs "uncertainty is not
distrust" (vision principle 2) and discounting by a source's reliability
(`from` in [language.md](language.md)). Toy models of this shape are written
now even though the harness itself comes later.

---

## What the three have in common

| | UC1 | UC2 | UC3 |
|---|---|---|---|
| Who asks Mitra | developer, demo app | CI pipeline | agent harness |
| Where the model comes from | written by hand | derived from a change | derived from an agent's actions |
| What the answer drives | understanding, tests | merge / human / block | continue / gather / human / stop |
| Needs the limiting element | to explain | to tell the author what to add | to pick the next action |
| When | first | later | later |

The engine is the same in all three. Only the source of the model and the
policy on top of the trust value change, which is why the language comes first.
