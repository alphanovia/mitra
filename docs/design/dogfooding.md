# Design: dogfooding

Status: **plan**, with a first slice of Track 1 implemented. Two tracks
that use Mitra on Mitra.

## Track 1: the calculus crate, judged by Mitra

A standalone crate implements the calculus ([calculus.md](calculus.md)),
and Mitra judges it from its own test runs. If the verdicts help while the
crate is built and maintained, that is the first sign Mitra is useful.

**What exists.** `crates/mitra-ds` has masses, Dempster's rule and
Yager's rule, with property tests over numbered cases (random pairs, and
the golden-set extremes). Its store `crates/mitra-ds/.mitra/` declares the
claims and the `test`s. `scripts/dogfood-ds.sh` runs the tests and ingests
the results, tagged with the git hash of the crate's committed code (it
refuses uncommitted changes and stops when the tests don't build). It then
evaluates with `--rev`, so only evidence on that code counts. `cargo test`
checks the verdicts.

**Next.** Rename it `mitra-calculus`, with one claim per axiom and operator
instead of "is Dempster–Shafer". Evidence then comes from three sources,
from weakest to strongest: unit tests (points), property tests (samples)
and Verus proofs (the whole domain).

**Circularity.** Mitra would judge the code it computes with, so a bug in
the crate could hide itself. The anchors outside the loop are Verus (a
proof does not depend on Mitra's calculus) and the golden set at 0 and 1,
which does not use the crate.

**Prerequisites:** rationals (cases stand in for real masses today), and
models in doc comments next to the code.

## Track 2: Mitra's semantics written in Mitra

Write Mitra's own rules as Mitra specs: set operations, the verdict rule
(contradiction, covered, refuted, logic) and the calculus. This gives an
**executable reference semantics** and the start of a **standard library**.
The Rust core is then claimed to match it:

```lean
opaque rust_union (s t : Value) : Value
def union (s t : Value) : Value := …
claim core_union : ∀ s, ∀ t, rust_union s t = union s t
```

with evidence from the Rust tests and, later, Verus.

Self-hosting the parser is not possible yet: specs are total and first
order, with no lists or recursion. What applies now is **bootstrapping
trust**: each release's model of itself is evaluated by the previous
release, and the fixed point is reached when both give the same verdicts.

## Order

1. Rationals and richer specs (needed by both tracks).
2. Track 2: the verdict rule and set operations as specs. Cheap, and it
   checks the language on a real example.
3. Track 1: `mitra-calculus` against the axioms, then confidences between
   0 and 1 in `mitra-core`.
