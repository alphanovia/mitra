# Design: syntax inspired by Lean 4

Status: **implemented**. The [reference](../reference/language.md) has the
exact syntax.

Decision (2026-11-17): Mitra stays a standalone language and tool, with its
own Rust parser. It is not a Lean library. Lean 4 is the inspiration for
grammar and syntax, because LLMs know it well and there is no need to
reinvent notation. Mitra changes from Lean only where it must, and adds
only what the ledger needs: evidence, tests, profiles, policies and verdicts.

## What comes from Lean

| Mitra | Lean 4 meaning it borrows |
|---|---|
| `opaque f (a b : Int) : Int` | declared, body unknown: a subject |
| `def f (a : Int) : Int := e` | a definition: a spec |
| `claim c : ∀ x ∈ S, …` | a theorem statement without a proof |
| `… := by derive [p, q]` | reads as a proof; Mitra checks the implication |
| `Icc a b`, `∪ ∩ \`, `{1, "a"}`, `×ˢ` | Mathlib sets |
| `f a b`, `(-2)`, `let x := v; e`, `--` | application, literals, comments |
| `{ logic := .holds, … }` | verdicts print as structure instances |

What Lean lacks, Mitra adds: `correct (f a)`, `evidence … at c` (not `@`,
which means explicit arguments in Lean), `test`, `profile … | x ∈ S => w`,
`policy`. Mitra files need not be valid Lean, so Lean's prelude names
(`not`, `id`) are free to use.

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

Open: ASCII forms. Lean has `forall` for `∀`, but no ASCII `∈`, `∪` or `∩`.
