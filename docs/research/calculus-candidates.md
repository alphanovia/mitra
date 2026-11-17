# Calculus candidates (open design)

The [language reference](../reference/language.md) only defines evidence at
confidence 0 or 1, with belief set by a policy (`strict` or `coverage`). Any calculus for confidences in between must give the same
results at those extremes:

1. Correct @ 1 on every covered input: holds, fully certain.
2. Wrong @ 1 on one covered input: fails, fully certain.
3. No evidence: fully uncertain.
4. Contradictory evidence @ 1 on one input: fully uncertain there.
5. Evidence @ 0 changes nothing.

| | A. Subjective logic | B. Dempster–Shafer, Yager's rule | C. Probability intervals |
|---|---|---|---|
| State per input | opinion (b, d, u); evidence weight c/(1−c) | belief correct, belief wrong | [low, high] for P(correct) |
| Same input | add weights | Yager: conflict becomes uncertainty | intersect; empty resets to [0, 1] |
| All covered inputs | SL multiplication | product of beliefs; fails if any input believed wrong | Fréchet bounds |
| Rules 1, 2, 3, 5 | ✓ | ✓ | ✓ |
| Rule 4 | ✗ certain 50/50 | ✓ | ✓ |
| 0.9 and 0.8 correct | holds ≈ 0.72 | holds @ 0.72 | holds in [0.7, 1] |
| Weak point | contradiction | assumes independent sources | jumps: 0.5 vs 0.5 is a point, 0.6 vs 0.6 is "no idea" |

Leaning: B first, C as the comparator (no independence assumption). Still
open: the rule that turns an in-between state into holds, fails or
uncertain. Background in [calculus.md](calculus.md).
