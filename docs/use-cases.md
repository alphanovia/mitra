# Use cases

Three concrete situations Mitra is for. They are ordered by when Mitra can
serve them: UC1 is what comes first, UC2 and UC3 are the reasons the engine
exists and shape its design now, but are built later. Who writes the
model in each (an agent, or a person as a scaffold) is in
[vision.md](vision.md#who-it-is-for).

---

## UC1: Check what a trust model computes (first)

**Actor:** a developer of Mitra, or of a program that will use Mitra.
**Goal:** write a small model, see each verdict and why, and be sure it is
right.

**Scenario.** The developer models the `add` CLI from
[validation.md](validation.md): a claim that `add` is correct on inputs whose
sum fits in 64 bits, another for inputs that overflow, a Verus proof of the
arithmetic, and one test of the overflow path.

**What Mitra should return.** The first claim holds because it is proven;
the overflow claim is unverified, because one test says little about
every overflowing input unless someone vouches that it stands for them;
and `mitra plan` lists the overflow path as evidence owed (a property
test or a proof).

**Why it matters.** This is how the language is validated: small models
whose correct verdicts are known. Every case in the golden set
([language reference](reference/language.md#golden-set)) and every example under `examples/reworked/` is an instance of UC1, and a
demo app would be UC1 with a screen.

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
merge     if  holds
block     if  fails or inconsistent
ask human otherwise
```

**What Mitra returns.** Uncertain: nothing contradicts the claim, but little
supports it independently, and the integration test rests on an unchecked
assumption. CI routes the change to a human and reports
the limiting element: "assumption `staging_token_lifetime` is unverified".
Checking that one assumption would let the change merge on its own.

**What this needs beyond the language:** a policy on verdicts, and a tool
that knows what a change touched. The model itself is written by the
agent with the change; `mitra tie` and `mitra ingest` already turn its
code and tests into evidence.

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

**What Mitra returns.** The first claim passes, the other two are unverified
rather than failed. Because nothing contradicts them, the policy's answer is not "stop" but "get more evidence": the harness
runs the tests itself, which is the cheapest action that raises trust the
most. If the harness's own run fails, that is a trusted counterexample: the
claim fails, the agent is stopped, and the agent's track record drops for the next decision.

**Why this shapes the language now:** it is the case that needs "unverified" and
"fails" to be different answers, and evidence from the agent itself to carry a
lower confidence than evidence from the harness. Toy models of this shape are written
now even though the harness itself comes later.

---

## What the three have in common

| | UC1 | UC2 | UC3 |
|---|---|---|---|
| Who asks Mitra | developer, demo app | CI pipeline | agent harness |
| Where the model comes from | written by a person or an agent | written by the agent with the change | derived from an agent's actions |
| What the answer drives | understanding, tests | merge / human / block | continue / gather / human / stop |
| Needs the limiting element | to explain | to tell the author what to add | to pick the next action |
| When | first | later | later |

The engine is the same in all three. Only the source of the model and the
policy on top of the verdict change, which is why the language comes first.
