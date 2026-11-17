# Language spec (draft 1)

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
| 6 | Via | `evidence c : … via s` | The evidence only counts as far as claim *s* holds (the test is right, the agent is reliable, the spec is faithful). With p = P(s): (b, d, u) becomes (p·b, p·d, 1 − p·(b + d)). Without `via`, evidence is first-hand. |
| 7 | Accumulation | several `evidence` lines on one claim | **Cumulative fusion** ⊕, assumed independent: b = (b₁u₂ + b₂u₁)/k, d = (d₁u₂ + d₂u₁)/k, u = u₁u₂/k, k = u₁ + u₂ − u₁u₂. |
| 8 | And | `c = p and q` (chains) | Binomial multiplication: P(c) = P(p)·P(q), a = a_p·a_q (Jøsang 2016, §7.1). |
| 9 | Or, not | `c = p or q`, `not p` | Comultiplication; complement (d, b, u, 1 − a). |
| 10 | Unless | `c = p unless q` | `p and not q`: *q* is a reason to doubt *c* (a defeater). |
| 11 | Split | `split c = p, q` | Sugar for `c = p and q and c.complete`, creating the claim `c.complete`: "the parts cover all of *c*". It must be assumed or evidenced like any claim, else it stays vacuous. |
| 12 | Query | `query c` | Reports c's opinion and P, its **weakest link**, and the assumptions it **rests on**. |

`#` starts a comment. Names are `[a-z_][a-z0-9_.]*`.

## Query output

```text
f_ok   (0.514, 0, 0.486)  P = 0.757
  weakest:  x_covers   (P would be 0.821 if it were certain)
  rests on: x_ok, x_covers
```

- **Weakest link:** among the leaves under the queried claim (assumptions and
  claims with evidence), the one that raises P the most when set to (1, 0, 0).
  Ties go to the first in name order. It is where new evidence or a check
  pays off most.
- **Rests on:** every assumption reachable from the claim.

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

- Should each evidence kind have a default `via` set by policy (for example,
  every `agent` report discounted by the agent's track record)?
- Fusion assumes independent evidence. When should dependent evidence (two
  tests sharing a fixture) use averaging fusion instead?
- Exact rationals or fixed point, so Lean and Verus can reason about results
  exactly?
