# Design: dogfooding

Status: **plan**; a first slice of Track 1 is implemented (`crates/mitra-ds`,
see below). Two tracks that use Mitra on Mitra.

## Track 1, "core": annotate another language

A standalone crate, **`mitra-ds`**: Dempster–Shafer on the binary frame
{correct, wrong}. It holds masses (b, d, u), Dempster's rule, Yager's rule
and the conflict mass ([composition.md](composition.md#4-what-a-confidence-means-and-the-calculus)).
`mitra-core` uses it for confidences between 0 and 1.

The crate carries its Mitra model in its doc comments:

```rust
/// ```mitra
/// spec  yager(m1: mass, m2: mass) -> mass = …
/// claim yager_impl: combine_yager(_) matches yager
/// claim yager_sums_to_one: combine_yager(m) matches total(m) == 1
/// ```
pub fn combine_yager(m1: Mass, m2: Mass) -> Mass { … }
```

Evidence comes from three sources, from weakest to strongest:

- unit tests: points;
- proptest: samples;
- Verus proofs: the whole domain.

Mitra reads the comments and the results and gives the diagnostics. If they
help while building and maintaining the crate, that is the first proof that
Mitra is useful.

**Circularity.** Mitra would judge the code it computes with. A bug in
`mitra-ds` could hide itself. The anchor outside the loop is **Verus**: a
proof does not depend on Mitra's calculus. Golden sets at `@ 0` and `@ 1`
do not use `mitra-ds` either.

**Prerequisites:** tuples (functions of two masses), rationals (masses are
fractions), and specs that can express the combination rules.

### What exists (2026-11-18)

`crates/mitra-ds` has masses, Dempster's rule and Yager's rule, and
property tests over numbered cases (random pairs, and the golden-set
extremes). Its store `crates/mitra-ds/.mitra/` declares the claims and the
`test`s; `scripts/dogfood-ds.sh` runs the tests and ingests the results,
tagged with the git hash of the crate's committed code (it refuses
uncommitted changes and stops when the tests don't build), then evaluates
with `--rev` so only evidence on that code counts. `cargo test` checks the
verdicts. Not done yet: rationals (cases stand in
for real masses), doc-comment models, Verus, and `mitra-core` using the
crate.

## Track 2, "pure Mitra": Mitra's semantics written in Mitra

Write Mitra's own rules as Mitra specs:

- set operations (union, difference, measure under a profile);
- the verdict rule (contradiction, covered, refuted, logic);
- the DS combination.

This gives two things. The first is an **executable reference semantics**:
the meaning of Mitra, in Mitra, evaluated by Mitra. The second is the start
of a **standard library**. The Rust core is then claimed to match it:

```mitra
claim core_union: rust_union(s, t) matches union(s, t)
```

with evidence from the Rust tests and, later, Verus.

## How languages usually bootstrap, and what applies here

A compiler is usually bootstrapped in three steps:

1. stage 0 is written in a host language;
2. stage 1 is the compiler written in its own language and built by stage 0;
3. stage 2 is built by stage 1.

The language has reached a fixed point when stage 2 equals stage 1.

Mitra cannot do that yet. Its spec language is total and first order, with
no lists and no general recursion, so a parser cannot be written in it.
Self-hosting the parser is a later option.

What does apply now is **bootstrapping trust**. Each release's Mitra model
of itself is evaluated by the previous release. The fixed point is reached
when release n and release n−1 give the same verdicts on release n's model.

## Order

1. Tuples, rationals and richer specs (needed by both tracks).
2. Track 2: the verdict rule and set operations as specs. This is cheap, and
   it checks the language on a real example.
3. Track 1: `mitra-ds`, then intermediate confidences in `mitra-core`.
