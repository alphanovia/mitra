# Toy models

Six small models with results worked out by hand from
[language.md](language.md), and checked with exact fractions. They are the
first tests: an implementation must reproduce them. Base rates are 0.5 unless
stated.

## 1. Nothing known

```mitra
claim f_ok "f is correct"
query f_ok
```

ω = **(0, 0, 1)**, P = **0.5**. No evidence is full uncertainty, not distrust.

## 2. One first-hand test

```mitra
claim f_ok "f is correct"
evidence f_ok : test +3        # three passing cases, run by us
query f_ok
```

(3, 0, 2)/5 → ω = **(0.6, 0, 0.4)**, P = 0.6 + 0.5·0.4 = **0.8**.

## 3. Test X verifies f

The relationship is made of claims, and X's result is discounted by them.

```mitra
claim f_ok     "f is correct"
claim x_ok     "test X is correct"
claim x_covers "test X exercises f's specified behaviour"
claim x_good   "test X is a reliable witness for f"

assume x_ok     = (0.8, 0, 0.2)     # reviewed test code
assume x_covers = (0.6, 0, 0.4)     # covers the main paths only
x_good = x_ok and x_covers

evidence f_ok : test +5 via x_good  # X's 5 cases pass
query f_ok
```

- `x_good`: P = P(x_ok)·P(x_covers) = 0.9 · 0.8 = 0.72.
- X's result alone: (5, 0, 2)/7 = (0.714, 0, 0.286).
- Through `x_good`: b = 0.72 · 5/7 = 0.514, d = 0, u = 0.486.

```text
f_ok   (0.514, 0, 0.486)  P = 0.757
  weakest:  x_covers   (P would be 0.821 if it were certain; x_ok: 0.786)
  rests on: x_ok, x_covers
```

Five passing cases first-hand would give P = 0.857. Doubt about the test turns
part of that into uncertainty, and the output says that improving coverage
helps more than re-reviewing the test.

## 4. Splitting a claim

`f` is proven for small inputs and only tested for large ones. The builder
asks "is `f` correct?" and gets one score per part, plus the parent.

```mitra
claim f_ok       "f is correct"
claim f_small    "f is correct for n < 1000"
claim f_large    "f is correct for n >= 1000"
claim spec_ok    "the Lean spec says what f should do"

split f_ok = f_small, f_large
assume f_ok.complete = (0.95, 0, 0.05)   # the two ranges cover every input

assume spec_ok = (0.9, 0, 0.1)
evidence f_small : proof via spec_ok
evidence f_large : test +3
query f_ok
```

- `f_small`: proof (1, 0, 0) through P(spec_ok) = 0.95 → (0.95, 0, 0.05), P = 0.975.
- `f_large`: (0.6, 0, 0.4), P = 0.8.
- `f_ok` = `f_small and f_large and f_ok.complete`: P = 0.975 · 0.8 · 0.975 = 0.760, a = 1/8.

```text
f_ok   (0.726, 0, 0.274)  P = 0.760
  weakest:  f_large    (P would be 0.951; spec_ok: 0.780; f_ok.complete: 0.780)
  rests on: spec_ok, f_ok.complete
```

The proven half is not the problem; three tests on large inputs are.

## 5. Is the agent's output correct?

An agent reports success. Its report is discounted by its track record; the
harness also runs tests itself.

```mitra
claim task_ok  "the task output is correct"
claim agent_ok "this agent's reports are reliable"

evidence agent_ok : review +8 -2      # right 8 times, wrong twice
evidence task_ok  : agent +1 via agent_ok
evidence task_ok  : test +4           # harness ran 4 tests itself
query task_ok
```

- `agent_ok`: (8, 2, 2)/12, P = 2/3 + 0.5 · 1/6 = 3/4.
- Agent's report: (1, 0, 2)/3 through 3/4 → (1/4, 0, 3/4).
- Harness tests: (4, 0, 2)/6 = (2/3, 0, 1/3).
- Fusion: k = 3/4 + 1/3 − 1/4 = 5/6; b = (1/12 + 1/2)/(5/6) = 7/10, u = (1/4)/(5/6) = 3/10.

```text
task_ok   (0.7, 0, 0.3)  P = 0.85
  weakest:  agent_ok   (P would be 0.857 if it were certain)
  rests on: (no assumptions)
```

**Variant:** the harness's run fails twice instead (`test -2`). Fusing
(1/4, 0, 3/4) with (0, 1/2, 1/2) gives ω = **(1/7, 3/7, 3/7)**, P = **0.357**:
real disbelief, not just doubt, which a harness can treat as "stop", where
high uncertainty alone would mean "gather more evidence".

## 6. A bug report arrives

`f_small` is proven in Lean. The proof counts only as far as the spec is right
and the Lean model of `f` matches the Rust code.

```mitra
claim f_small    "f is correct for n < 1000"
claim spec_ok    "the Lean spec says what f should do"
claim model_ok   "the Lean model of f matches the Rust code"
claim proof_good "the Lean proof is a reliable witness for f"

assume spec_ok  = (0.9, 0, 0.1)
assume model_ok = (0.7, 0, 0.3)      # hand-translated from Rust
proof_good = spec_ok and model_ok
evidence f_small : proof via proof_good
```

**Before:** P(proof_good) = 0.95 · 0.85 = 0.8075, so f_small =
(0.8075, 0, 0.1925), P = 0.904.

**Then** a user reports a crash at n = 500, and a failing regression test
reproduces it:

```mitra
evidence f_small : test -1           # fails on n = 500
conflicts
```

- Failing test: (0, 1, 2)/3 = (0, 0.333, 0.667), P = 0.333.
- Fused: f_small = (646, 77, 154)/877 ≈ (0.737, 0.088, 0.176), P = **0.824**.
  The score barely moves: one failure against a proof.
- Conflict: \|0.904 − 0.333\| · (1 − 0.1925) · (1 − 0.667) = **0.154**, above 0.1.

```text
conflict on f_small   con = 0.154
  for:     proof via proof_good   P = 0.904
  against: test (first-hand)      P = 0.333
  suspects: model_ok (0.85), spec_ok (0.95), f_small
```

The aggregate score still looks healthy; the conflict query does not. It
points at the Lean model not matching the Rust code as the most likely root
cause, then the spec, before blaming the code itself. Fixing the model and
re-proving, or a second failing case, resolves it either way.
