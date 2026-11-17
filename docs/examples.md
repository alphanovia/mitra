# Toy models

Five small models with their results worked out by hand from the formulas in
[language.md](language.md). They are the first tests: an implementation must
reproduce these numbers exactly. All base rates are the default 0.5.

## 1. No evidence (vacuous)

```mitra
claim x "the feature works"
query x
```

Nothing is known, so ω = **(0, 0, 1)**, P = **0.5**: the base rate.
*Checks:* no evidence means full uncertainty, not distrust.

## 2. One piece of evidence

```mitra
claim x "the feature works"
evidence x : +3          # three passing tests
query x
```

r = 3, s = 0, so b = 3/5, d = 0, u = 2/5.
ω = **(0.6, 0, 0.4)**, P = 0.6 + 0.5·0.4 = **0.8**.

## 3. Two independent pieces, one conflicting

```mitra
claim x "the feature works"
evidence x : +3          # unit tests
evidence x : +1 -1       # manual check: one pass, one failure
query x
```

ω₁ = (3/5, 0, 2/5), ω₂ = (1/4, 1/4, 1/2).
k = 2/5 + 1/2 − 1/5 = 7/10.
b = (3/5·1/2 + 1/4·2/5)/k = (2/5)/(7/10) = 4/7, d = (1/4·2/5)/k = 1/7,
u = (1/5)/(7/10) = 2/7.
ω = **(4/7, 1/7, 2/7) ≈ (0.571, 0.143, 0.286)**, P = 4/7 + 1/7 = **5/7 ≈ 0.714**.
*Checks:* same as pooling the evidence (+4 −1); conflict shows up as
disbelief, not only as lower belief.

## 4. Evidence from an imperfect source

```mitra
claim x "all tests pass"
source agent : +8 -2           # agent was right 8 times, wrong twice
evidence x : +10 from agent    # agent reports 10 passing tests
query x
```

Source: (8/12, 2/12, 2/12), P(agent) = 2/3 + 0.5·1/6 = 3/4.
Reported evidence: (10/12, 0, 2/12) = (5/6, 0, 1/6).
Discounted: b = 3/4·5/6 = 5/8, d = 0, u = 1 − 5/8 = 3/8.
ω = **(0.625, 0, 0.375)**, P = 5/8 + 3/16 = **13/16 = 0.8125**.
*Checks:* the agent's 10 tests first-hand would give P ≈ 0.917; going through
an imperfect source turns some belief into uncertainty, not disbelief.

## 5. Conjunction

```mitra
claim sort_ok  "sort returns a sorted permutation"
claim ordered  "output is ordered"
claim permuted "output is a permutation"
assume   ordered  = (0.9, 0, 0.1)
evidence permuted : +3
sort_ok = ordered and permuted
query sort_ok
```

ω_ordered = (0.9, 0, 0.1), ω_permuted = (0.6, 0, 0.4), both a = 0.5.
With k = 1 − a_y·a_z = 3/4:
b = 0.9·0.6 + (0.5·0.5·0.9·0.4 + 0.5·0.5·0.1·0.6)/k = 0.54 + 0.105/0.75 = 0.68,
d = 0 + 0 − 0 = 0,
u = 0.1·0.4 + (0.5·0.9·0.4 + 0.5·0.1·0.6)/k = 0.04 + 0.21/0.75 = 0.32,
a = 0.25.
ω = **(0.68, 0, 0.32)**, P = 0.68 + 0.25·0.32 = **0.76**.
*Checks:* P = P(ordered)·P(permuted) = 0.95·0.8; the result is no more
trusted than its weaker part, and the weak part is `permuted`.
