# Design: syntax inspired by Lean 4

Status: **implemented**. The [reference](../reference/language.md) has the
exact syntax.

Decision (2026-11-17): Mitra stays a standalone language and tool, with its
own Rust parser. It is not a Lean library. Lean 4 is the inspiration for
grammar and syntax, because LLMs know it well and there is no need to
reinvent notation. Mitra changes from Lean only where it must, and adds
only what the ledger needs: evidence, tests and verdicts.

## What comes from Lean

| Mitra | Lean 4 meaning it borrows |
|---|---|
| `opaque f …` | declared, body unknown: a subject |
| `def f … := e` | a definition: a spec |
| `claim c : forall x in S, …` | a theorem statement without a proof |
| `… := by derive [p, q]` | reads as a proof; Mitra checks the implication |
| `{1, "a"}`, `\` | set literals and difference |
| `let x := v; e`, `--` | local definitions, comments |
| `Int`, `Int64`, `Float64` | `Int` is ℤ; the others are fixed-width |

What Lean lacks, Mitra adds: `correct(f(a))`, `evidence … at c` (not `@`,
which means explicit arguments in Lean), `test`. Mitra files need not be valid Lean, so Lean's prelude names
(`id`) are free to use.

Where Lean reads badly or needs an editor, Mitra departs from it
(2026-11-17):

- **ASCII is canonical**: `forall`, `in`, `not`, `|` (union), `&`
  (intersection). Lean's `∀ ∈ ¬ ∪ ∩` are accepted as aliases, and Mitra
  prints ASCII. So `not` is a keyword, and the old `not` subject of the
  golden set is now `flip`.
- **Ranges, as in Rust**: `0..=10` (closed), `0..10` (half-open), `5..`,
  `..=5`, instead of Mathlib's `Icc 0 10`.
- **Parameters and calls with parentheses** (2026-11-18): `opaque
  judged(ok: Bool, c: Float64) -> Value` and `correct(judged(true, c))`,
  not Lean's `(ok : Bool) (c : Float64) : Value` and `judged true c`.
  More languages write it this way, Rust first, which is the language
  Mitra's models sit next to. One parenthesis group, commas, one type per
  parameter, `->` before the result type, and a negative argument needs
  no parentheses of its own: `div(4, -2)`. Kept as they were, because
  they are not calls: `claim c : uniform f by g`, `claim c : uses f [g,
  h]`, `:= by derive [p, q]`, `forall x in S,`, `evidence … at 1`, and
  evidence on a whole claim, `evidence c at 1`.
- **`Float64`, not `Real`**: a 64-bit float is not a real number (NaN,
  infinities, `-0.0`, rounding). `Real` is reserved for later, as a type
  for specs and proofs.

## Why a fixed subset

Lean's grammar is not fixed, so it is neither LL(1) nor LR(1):

- **Extensible.** Any file can add notation, so the grammar changes while a
  file is read.
- **Layout.** Some constructs depend on columns (`by` blocks, `match` arms).
- **Backtracking.** The parser tries alternatives, for example to tell
  `{x, y}` (a set) from `{ x := 1 }` (a structure).

Mitra takes a subset with no user notation, no layout rule and no
backtracking. Every command starts with a keyword. Not taken: `match`,
`fun`, tactics. `crates/mitra-grammar` checks that the grammar is LL(1) and
LR(1), and fuzzes the parser against it.

Verdicts print a product of sets as a tuple of sets, `(0..=2, -2..=0)`,
and a finite one as a set of tuples, `{(1, -1)}`.
