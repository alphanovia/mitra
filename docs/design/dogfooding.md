# Design: dogfooding

Status: Track 1 implemented on two crates; Track 2 is a plan, with a
first slice (the operators of specs). Two tracks
that use Mitra on Mitra.

## Track 1: the Dempster-Shafer crate, judged by Mitra

A standalone crate implements the combination rules Mitra will use to say
which evidence is most plausibly wrong
([semantics.md](semantics.md#4-opinion-which-piece-is-wrong)), and Mitra
judges it from its own test runs. If the verdicts help while the
crate is built and maintained, that is the first sign Mitra is useful.

**What exists.** The model is aside from the code, in each crate's
`.mitra/` folder, one file per subject, starting at `model.mitra`, in the
[language](../reference/language.md), with the same types as the Rust
code: a struct is a record, `Option` and `Result` are enums, a `Vec` is
a list. The code keeps one comment per modelled function or struct,
`// mitra: implements claims::Claim`, and `mitra tie` checks that the two
agree (fields, inputs, result, calls). Each subject says what it promises
(`ensure`), what it needs (`require`), then the records, functions and
claims that serve those. A test names the claim it checks:
`// mitra: evidence for ledger::vouched_holds`.

The four crates have a store. `mitra-lang` models what a model is
(function, claim, intent, evidence, the model itself), then what Mitra
does with one (reader, checks, verdicts, report, ranking of culprits);
it has no boundary, since it reads no file. `mitra-store` models the
files of a model and the ledger file, behind the boundary `disk`.
`mitra-rust` models the readers of `cargo test` reports and of ties, behind the
boundary `code`. `mitra-cli` models what the caller is taken to say
(boundary `caller`) and the check of a revision. Most of
`mitra-lang` is not modelled: its types are recursive (`Expr`, `Set`) or
its functions keep state. The gaps are listed in the review list
(2026-11-18, increment 22).

`scripts/dogfood.sh <crate>` runs the tests, ingests the results and
runs `mitra tie`, all tagged with the git hash of the crate's committed code (it refuses
uncommitted changes and stops when the tests don't build). It then
evaluates with `--rev`, so only evidence on that code counts. `cargo
test` checks the verdicts of the three stores. A review is recorded with
`scripts/vouch.sh <crate> review:<who>`, which vouches for every
`uniform` claim at that hash (a `uses` clause is taken as declared, and
checked by `mitra tie`), and goes stale when the code
changes. `scripts/breaks.py <crate>` is the benchmark: it breaks the code
one way at a time (`scripts/breaks/<crate>.txt`) and prints what `cargo
test` and Mitra say.

**Next.** Masses over a frame of possible culprits instead of three
numbers, once the semantics are agreed. Evidence then comes from three
sources, from weakest to strongest: unit tests (points), property tests
(samples) and Verus proofs (the whole domain).

**Circularity.** Mitra would judge the code it computes with, so a bug in
the crate could hide itself. Verdicts do not use the crate, only the ranking
of culprits does; the other anchor is Verus.

**Prerequisites:** sampled evidence (property tests) and a proof source
for infinite domains.

## Track 2: Mitra's semantics written in Mitra

Write Mitra's own rules as Mitra specs: set operations, the verdict rule
(tested, assumed, missed, the verdict). This gives an
**executable reference semantics** and the start of a **standard library**.
The Rust core is then claimed to match it:

```rust
subject set {
    fn union(s: Any, t: Any) -> Any { … }   // the spec; the claim is set::union::spec
}
```

with `// mitra: implements set::union` above the Rust function, and
evidence from the Rust tests and, later, Verus. The first slice exists
in `crates/mitra-lang/.mitra/operators.mitra`.

Self-hosting the parser is not tried: specs are total, and recursion is
structural only. What applies now is **bootstrapping
trust**: each release's model of itself is evaluated by the previous
release, and the fixed point is reached when both give the same verdicts.

## Order

1. Richer specs: records, lists (done).
2. Track 2: the verdict rule and set operations as specs. Cheap, and it
   checks the language on a real example.
3. Track 1: the opinion on evidence in `mitra-lang`, computed with the
   crate.
