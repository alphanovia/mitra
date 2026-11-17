# Design: Mitra in Lean 4 syntax

Status: **proposal**, not implemented. Today's syntax is in the
[reference](../reference/language.md).

Direction: reuse Lean 4's grammar and syntax. Change only what we must, and
add only what the ledger needs (evidence, profiles, policies, verdicts).

## 1. Current constructs in Lean 4

| Mitra today | Lean 4 | Note |
|---|---|---|
| `# comment` | `-- comment`, `/- … -/` | |
| `subject f(a: int, b: int) -> int` | `opaque f (a b : Int) : Int` | `opaque`: declared, body unknown. Exactly a subject. |
| `spec f(a: int) -> int = e` | `def f (a : Int) : Int := e` | |
| `bool`, `int`, `string` | `Bool`, `Int`, `String` | |
| `any` | `Value` | Lean has no "any". Mitra provides `Value`, with coercions from `Bool`, `Int`, `String`. |
| `{1, "a"}` | `{1, "a"}` | Lean/Mathlib set literal. |
| `[0, 100]` | `Icc 0 100` | Mathlib's closed interval (`open Set`). |
| `A \| B`, `A & B`, `A \ B` | `A ∪ B`, `A ∩ B`, `A \ B` | Usual precedence instead of left to right. |
| `let x = v in e` | `let x := v; e` | |
| `if c then a else b`, `&& \|\| ! == != < + - *` | same | |
| `f(a, b)` in expressions | `f a b` | Application by juxtaposition. `-2` as an argument is `(-2)`, as in Lean, which removes today's context-sensitive `-`. |
| pattern `_` | `∀ a,` | |
| pattern value `true` | `true` applied directly | |
| pattern `x in S` | `∀ x ∈ S,` | Lean binder predicate. |
| tuples `(a, b)`, boxes | `∀ a ∈ S, ∀ b ∈ T,`; output `(1, -1)`, `S ×ˢ T` | |
| `claim n: f(p) matches e` | `claim n : ∀ …, f … = e …` | `matches` becomes `=`. |
| `claim n: f(p) correct` | `claim n : ∀ …, correct (f …)` | `correct (f a)`: "the call `f a` is correct". A Mitra form, not a Lean function. |
| `… from c1, c2` | `… := by derive [c1, c2]` | Reads as a proof; Mitra checks the implication. |

A `claim` is like a `theorem` without a proof: a proposition to be weighed.
A claim that is later proved in Lean is evidence at 1 on its whole domain.

## 2. What Lean lacks: ledger commands

```lean
-- Evidence: a judgment and a confidence. The judgment is a proposition.
evidence correct (not true) at 1          -- was: not(true) correct @ 1
evidence ¬ correct (not false) at 1       -- was: wrong
evidence add 0 (-2) = -2 at 1             -- was: add(0, -2) -> -2
evidence minus_shape at 1                 -- was: minus_shape(_, _) holds
evidence ¬ minus_shape at 1               -- was: fails
evidence ∀ x ∈ Icc 0 100, correct (id x) at 1   -- a proof over a set

-- Profile: weights over regions, as match arms.
profile usage for id
  | x ∈ Icc 0 100 => 0.8
  | x ∈ strings   => 0.1
  | _             => 0.1

-- Policy.
policy strict
policy by_usage := coverage usage
```

`at` replaces `@`, because `@` already means explicit arguments in Lean.
`at` is a Lean token that never continues a term, so the term before it
ends cleanly.

**Verdicts**, printed as Lean structure instances:

```lean
not_correct [strict] := { logic := .fails, refuted := {false}, domain := univ, belief := 0 }
```

## 3. Examples

### not (golden `not/strict/05`)

```lean
opaque not (a : Bool) : Bool

-- not is correct on every input.
claim not_correct : ∀ a, correct (not a)
-- not is correct on true, a subset of its inputs.
claim not_on_true : correct (not true)

policy strict

evidence correct (not true) at 1
evidence ¬ correct (not false) at 1
```

```lean
not_correct [strict] := { logic := .fails, refuted := {false}, domain := univ, belief := 0 }
not_on_true [strict] := { logic := .holds, covered := {true}, domain := {true}, belief := 1 }
```

### id (golden `id/by_usage`)

```lean
opaque id (x : Value) : Value

-- id is correct on every input.
claim id_correct : ∀ x, correct (id x)
-- id is correct on the integers from 0 to 100.
claim id_small : ∀ x ∈ Icc 0 100, correct (id x)

-- How id is used: mostly small integers, sometimes strings, rarely anything else.
profile usage for id
  | x ∈ Icc 0 100 => 0.8
  | x ∈ strings   => 0.1
  | _             => 0.1

policy by_usage := coverage usage

evidence ∀ x ∈ Icc 0 100, correct (id x) at 1
```

### minus (golden `minus/strict/03`)

```lean
def plus (a b : Int) : Int := a + b
def opposite (b : Int) : Int := -b
def difference (a b : Int) : Int := a - b

opaque add (a b : Int) : Int
opaque negate (b : Int) : Int
opaque minus (a b : Int) : Int

-- add, on the inputs minus uses.
claim add_spec : ∀ a ∈ Icc 0 2, ∀ b ∈ Icc (-2) 0, add a b = plus a b
claim negate_spec : ∀ b ∈ Icc 0 2, negate b = opposite b
-- How minus is built, from a review of its code.
claim minus_shape : ∀ a ∈ Icc 0 2, ∀ b ∈ Icc 0 2, minus a b = add a (negate b)
-- Derived: Mitra checks the implication by substituting the premises.
claim minus_spec : ∀ a ∈ Icc 0 2, ∀ b ∈ Icc 0 2, minus a b = difference a b :=
  by derive [add_spec, negate_spec, minus_shape]

policy strict

evidence add 0 (-2) = -2 at 1
evidence add 0 (-1) = -1 at 1
-- … the other add points, add 2 (-2) never observed …
evidence negate 0 = 0 at 1
evidence negate 1 = -1 at 1
evidence negate 2 = -2 at 1
evidence minus_shape at 1
```

`matches plus` (a spec name alone) becomes the explicit `= plus a b`.

## 4. Consequences: grammar, and where Mitra lives

Lean's grammar is not a fixed grammar. Notation is user-extensible, some
constructs depend on columns, and the parser backtracks. It is neither
LL(1) nor LR(1), and no Lean file can be checked by a grammar tool. Also,
`not` and `id` are Lean prelude names: in real Lean, our examples need a
namespace or other names.

| Option | What | For | Against |
|---|---|---|---|
| **A. Lean library (DSL)** | `import Mitra`; `claim`, `evidence`, `profile`, `policy` are Lean commands (`syntax` + `elab`). | Real Lean: specs run, claims are `Prop`s, a proof is direct evidence, Mathlib sets, Lean's editor support. | Mitra needs the Lean toolchain (heavy, slow to start). The engine moves to Lean or calls Rust through FFI. Name clashes with Lean's prelude. Verus side gets split. |
| **B. Fixed subset, own parser** | Mitra is a small, fixed, Lean-looking language, parsed by Rust. | Keeps the Rust core and today's tooling: LALRPOP/LL(1) checks, generator, tree-sitter. Fast and embeddable. | Looks like Lean but isn't, so users hit what Lean has and Mitra lacks. No proofs for free. |
| **C. B, kept inside A** | B's subset is chosen so that every Mitra file is also valid Lean once `import Mitra` defines the commands. | B now, A later with no syntax change. A Lean CI job can parse the golden files as a check. | Every syntax choice must work in both. Only choices a Lean prototype confirms (such as `at`) are safe. |

**Recommendation: C.** Define a fixed subset with these rules:

- **Commands:** `opaque`, `def`, `claim`, `evidence`, `profile`, `policy`. Each starts with a keyword and has no layout rule.
- **Terms:** application, `let`/`if`, the operators above, `∀ x ∈ S,`, set literals, and `Icc`, `∪`, `∩`, `\`.
- **No user notation, `match`, `fun`, or tactics.** The only exception is `by derive [...]`.

This subset can stay LL(1) after left-factoring, and LR(1). We keep the
grammar tooling (paused) and point it at the new syntax. A small Lean
prototype of `import Mitra` comes first, before any Rust change, to confirm
that the subset really is valid Lean.

Open: whether to use ASCII forms. Lean has `forall` for `∀`, but no ASCII
`∈`, `∪` or `∩`.
