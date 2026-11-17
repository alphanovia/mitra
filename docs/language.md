# Language spec (draft 0)

Every construct maps to a subjective-logic opinion or operator (Jøsang,
*Subjective Logic*, 2016). This page is the meaning; the syntax is one
projection of it and may change.

## Values

An **opinion** about a claim *x* is ω = (b, d, u, a): belief, disbelief,
uncertainty, base rate, with b, d, u ≥ 0 and b + d + u = 1. Its **projected
probability** is P = b + a·u. All opinions are binomial (a claim is true or
false).

## Constructs

| # | Construct | Syntax | Meaning |
|---|---|---|---|
| 1 | Claim | `claim x "text"` | Declares proposition *x*. With nothing else said, its opinion is **vacuous**: (0, 0, 1, a). |
| 2 | Base rate | `base x = 0.3` | Sets *a* for *x*, the prior used when there is no evidence. Default 0.5. |
| 3 | Assumption | `assume x = (b, d, u)` | Gives *x*'s opinion directly. Use for things accepted rather than measured. `(1, 0, 0)` is a dogmatic "true". |
| 4 | Evidence | `evidence x : +r -s` | *r* observations for, *s* against. Maps to b = r/(r+s+2), d = s/(r+s+2), u = 2/(r+s+2). |
| 5 | Source | `source S : +r -s` or `source S = (b, d, u)` | Declares a source and the opinion that it is reliable, from its track record or given directly. |
| 6 | Attributed evidence | `evidence x : +r -s from S` | Evidence reported by *S*, **discounted** by S's reliability: with p = P(S), the result is (p·b, p·d, 1 − p·(b+d), a). Evidence with no `from` is first-hand and not discounted. |
| 7 | Accumulation | several `evidence` lines on one claim | Combined by **cumulative fusion** ⊕, assumed independent: b = (b₁u₂ + b₂u₁)/k, d = (d₁u₂ + d₂u₁)/k, u = u₁u₂/k, with k = u₁ + u₂ − u₁u₂. For first-hand evidence this equals adding the *r*s and *s*s. |
| 8 | Conjunction | `x = y and z` (chains: `y and z and w`) | Binomial multiplication. P(x) = P(y)·P(z); disbelief d = d_y + d_z − d_y·d_z; a = a_y·a_z. Full formula in Jøsang 2016, §7.1. |
| 9 | Disjunction | `x = y or z` | Binomial comultiplication, the dual of 8. b = b_y + b_z − b_y·b_z; a = a_y + a_z − a_y·a_z. |
| 10 | Negation | `not y` | Complement: (d, b, u, 1 − a). |
| 11 | Defeater | `x = y unless z` | Sugar for `y and not z`: *z* is a reason to doubt *x*. |
| 12 | Query | `query x` | Evaluate and report *x*'s opinion and P. |

`#` starts a comment. Identifiers are `[a-z_][a-z0-9_]*`.

## Rules

- A claim is either **derived** (exactly one `x = …` definition) or **basic**
  (evidence and/or an assumption). Mixing both on one claim is an error in
  draft 0.
- Definitions must not form a cycle.
- Assumption and evidence on the same basic claim are fused with ⊕.
- Evaluation is deterministic and order-independent: the order of lines does
  not change any result.

## Embedding

The same text is valid wherever it is placed:

````markdown
```mitra
evidence tests_pass : +12 -0 from ci
```
````

```rust
/// ```mitra
/// claim parse_total "parse() never panics on UTF-8 input"
/// evidence parse_total : +10000 from fuzzer
/// ```
fn parse(input: &str) -> Ast { … }
```

## Open questions

- Should a claim's dependence on assumptions be explicit (`assumes`) rather
  than written as a conjunction?
- Is fusion's independence assumption acceptable by default, or should
  dependent evidence be declared (averaging fusion)?
- Exact rationals or fixed point for the implementation, so Lean and Verus can
  reason about results exactly.
