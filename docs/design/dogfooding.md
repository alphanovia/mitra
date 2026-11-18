# Design: dogfooding

Status: **plan**, with a first slice of Track 1 implemented. Two tracks
that use Mitra on Mitra.

## Track 1: the Dempster-Shafer crate, judged by Mitra

A standalone crate implements the combination rules Mitra will use to say
which evidence is most plausibly wrong
([semantics.md](semantics.md#4-opinion-which-piece-is-wrong)), and Mitra
judges it from its own test runs. If the verdicts help while the
crate is built and maintained, that is the first sign Mitra is useful.

**What exists.** `crates/mitra-ds` has masses, Dempster's rule and
Yager's rule. Its model sits in `mitra:` comments next to the code: one
subject and one claim per public function (a mass is three `Float64`s),
and one `test` above each unit test, for the inputs it runs. The store
`crates/mitra-ds/.mitra/` holds the evidence.
`scripts/dogfood-ds.sh` runs the tests and ingests the results, tagged
with the git hash of the crate's committed code (it refuses uncommitted
changes and stops when the tests don't build). It then evaluates with
`--rev`, so only evidence on that code counts. `cargo test` checks the
verdicts. Each function also has a `uniform` claim, whose classes follow
the branches of its code; with a review vouching for them, every claim
holds from one passing test per class. A review is recorded by hand, with
`mitra add 'evidence yager_uniform at 1' --source review:<who> --rev <hash>`,
and goes stale when the code changes. What the model can't say yet is
listed in the review list (2026-11-18, increments 8 and 9).

**Next.** Masses over a frame of possible culprits instead of three
numbers, once the semantics are agreed. Evidence then comes from three
sources, from weakest to strongest: unit tests (points), property tests
(samples) and Verus proofs (the whole domain).

**Circularity.** Mitra would judge the code it computes with, so a bug in
the crate could hide itself. Verdicts do not use the crate, only the ranking
of culprits does; the other anchor is Verus.

**Prerequisites:** sampled evidence (property tests), records (a mass as
one value), and a proof source for infinite domains.

## Track 2: Mitra's semantics written in Mitra

Write Mitra's own rules as Mitra specs: set operations, the verdict rule
(tested, assumed, missed, the verdict). This gives an
**executable reference semantics** and the start of a **standard library**.
The Rust core is then claimed to match it:

```lean
opaque rust_union (s t : Value) : Value
def union (s t : Value) : Value := …
claim core_union : forall s, forall t, rust_union s t = union s t
```

with evidence from the Rust tests and, later, Verus.

Self-hosting the parser is not possible yet: specs are total and first
order, with no lists or recursion. What applies now is **bootstrapping
trust**: each release's model of itself is evaluated by the previous
release, and the fixed point is reached when both give the same verdicts.

## Order

1. Richer specs: records, lists (needed by both tracks).
2. Track 2: the verdict rule and set operations as specs. Cheap, and it
   checks the language on a real example.
3. Track 1: the opinion on evidence in `mitra-core`, computed with the
   crate.
