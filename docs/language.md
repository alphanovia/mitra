# Language spec (draft 2)

Mitra makes the graph of trust explicit and puts a number on every node.
Every construct maps to a subjective-logic opinion or operator (Jøsang,
*Subjective Logic*, 2016). This page is the meaning; the syntax is one
projection of it and may change.

## The core: claims and opinions

- A **claim** is a binary proposition about a subject: "function `f` is
  correct", "this task output is correct", "the agent only edited `src/`".
  It is true or false.
- Mitra's value for a claim is an **opinion** ω = (b, d, u, a): belief,
  disbelief, uncertainty and base rate, with b + d + u = 1. Uncertainty lives
  in the opinion, not in the claim. The **projected probability**
  P = b + a·u is the single number a policy can use.
- **"Partially true"** is never a value. Either split the claim into parts
  ("`f` is correct for n < 1000" and "for n ≥ 1000"), or put the threshold in
  the claim ("`f` is correct on at least 99% of inputs").

## Relationships are claims

"Test X verifies `f`" is not an edge with a weight. It is made of claims that
can each be wrong and each get a score:

1. `x_ok`: test X is itself correct;
2. `x_covers`: test X really exercises what `f` should do;
3. `f_ok`: `f` is correct.

X's result reaches `f_ok` only **through** `x_ok and x_covers`: the weaker
those are, the more of X's evidence turns into uncertainty about `f_ok`.

## Constructs

| # | Construct | Syntax | Meaning |
|---|---|---|---|
| 1 | Claim | `claim c "text"` | A binary claim. With nothing else said its opinion is **vacuous**: (0, 0, 1, a). |
| 2 | Base rate | `base c = 0.3` | Sets *a*, the prior used where evidence is missing. Default 0.5. |
| 3 | Assumption | `assume c = (b, d, u)` or `assume c` | A claim accepted without evidence: the regress stops here. The opinion is set by policy; bare `assume c` is vacuous, so only its base rate counts. Assumptions are always reported with a result. |
| 4 | Evidence | `evidence c : kind +r -s` | *r* observations for, *s* against, giving (r, s, 2) / (r + s + 2). `kind` is one of `test`, `usage`, `review`, `agent`. |
| 5 | Proof | `evidence c : proof` | A checked proof: the dogmatic opinion (1, 0, 0). Its real strength comes from what it is discounted by (rule 6). |
| 6 | Via | `evidence c : … via s` | The evidence only counts as far as claim *s* holds (the test is right, the agent is reliable, the spec is faithful). With p = P(s): (b, d, u) becomes (p·b, p·d, 1 − p·(b + d)). Without `via`, the kind's policy applies (see below); with neither, evidence is first-hand. |
| 7 | Accumulation | several `evidence` lines on one claim | **Cumulative fusion** ⊕, assumed independent: b = (b₁u₂ + b₂u₁)/k, d = (d₁u₂ + d₂u₁)/k, u = u₁u₂/k, k = u₁ + u₂ − u₁u₂. |
| 8 | And | `c = p and q` (chains) | Binomial multiplication: P(c) = P(p)·P(q), a = a_p·a_q (Jøsang 2016, §7.1). |
| 9 | Or, not | `c = p or q`, `not p` | Comultiplication; complement (d, b, u, 1 − a). |
| 10 | Unless | `c = p unless q` | `p and not q`: *q* is a reason to doubt *c* (a defeater). |
| 11 | Split | `split c = p, q` | Sugar for `c = p and q and c.complete`, creating the claim `c.complete`: "the parts cover all of *c*". It must be assumed or evidenced like any claim, else it stays vacuous. |
| 12 | Policy | `policy kind = (b, d, u)` | How much you trust a kind of evidence. Creates the assumption `policy.kind`, used as the default `via` for that kind. |
| 13 | Queries | `query c`, `uncertain`, `next c`, `conflicts` | See **Diagnostics**. |

`#` starts a comment. Names are `[a-z_][a-z0-9_.]*`.

## Policy: trust per evidence kind

The main thing a user configures is how far each kind of evidence is trusted.
A policy is an assumption like any other, so every result that depends on it
lists it under "rests on". A starting profile:

```mitra
policy proof  = (0.95, 0, 0.05)   # checked, but only as good as its spec
policy test   = (0.80, 0, 0.20)
policy usage  = (0.70, 0, 0.30)   # telemetry, bug reports
policy review = (0.60, 0, 0.40)
policy agent  = (0.50, 0, 0.50)   # an agent's own report
```

An explicit `via` replaces the policy for that line, so a specific test can be
trusted more or less than tests in general. A kind with no policy is
first-hand.

## Diagnostics

A single score is not the point. Mitra is useful when it shows **where** the
graph is uncertain or wrong.

| Query | Answers | Definition |
|---|---|---|
| `query c` | How much can I trust *c*, and on what? | c's opinion and P, its weakest link (first entry of `next c`), and every assumption reachable from it. |
| `uncertain` | Where do I not know enough? | Every claim with its uncertainty *u*, highest first. |
| `next c` | What evidence would help most? | Each leaf under *c* (assumption or claim with evidence), ranked by how much P(c) rises if that leaf were certain (set to (1, 0, 0)). Ties in name order. |
| `conflicts` | Where is something wrong? | Every claim whose evidence lines disagree, with degree of conflict con = \|P₁ − P₂\| · (1 − u₁) · (1 − u₂) above 0.1, and its **suspects**. |

**Localizing a fault.** When evidence lines on a claim conflict, at least one
of them is wrong, so at least one claim on its `via` route is false (or the
claim itself is). Suspects are the claims and assumptions on the routes of
the conflicting lines, plus the claim itself, ranked by P, lowest first: the
least trusted link is the most likely culprit. Conflict matters because
fusion can hide it: a strong proof and a failing test still fuse into a
fairly high score (toy model 6).

Example output:

```text
f_ok   (0.514, 0, 0.486)  P = 0.757
  weakest:  x_covers   (P would be 0.821 if it were certain)
  rests on: x_ok, x_covers
```

## Rules

- A claim is **derived** (one `c = …` definition) or **basic** (evidence
  and/or an assumption), never both in this draft.
- No cycles, through definitions or through `via`.
- Evaluation is deterministic and independent of line order.

## Embedding

The text is valid wherever it is placed: a fenced ` ```mitra ` block in
Markdown (skills, harness instructions), or a docstring:

```rust
/// ```mitra
/// claim parse_ok "parse() never panics on UTF-8 input"
/// evidence parse_ok : test +10000 via fuzzer_ok
/// ```
fn parse(input: &str) -> Ast { … }
```

## Open questions

- Should `via` combine with the kind's policy (both apply) rather than
  replace it?
- Should suspects be ranked by something richer than P, such as how much the
  conflict drops if the suspect were false?
- Fusion assumes independent evidence. When should dependent evidence (two
  tests sharing a fixture) use averaging fusion instead?
- Exact rationals or fixed point, so Lean and Verus can reason about results
  exactly?
