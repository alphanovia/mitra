# Claims without evidence, boundaries, `correct`, `represented`, `uniform`

Part of the [language reference](language.md); says what the code does
today, and changes in the same commit as the code.

## What a claim rests on without evidence on it

A claim that calls a function of a subject needs the output of that call.
Where none was observed, Mitra looks for it in this order.

1. **A vouched claim of the shape `f(a, b) == e`** (either side, each
   input a different name or a written value, the evidence on the whole
   claim and no evidence against it): `e` is worked out from what was
   observed, and stands for the output. The vouch is then among the
   evidence of the claim that read it, so a wrong result names it.

   ```rust
   claim add_commutes(a: 0..=2, b: 0..=2) { add(a, b) == add(b, a) }
   claim more_one(a: 0..=1) { add(1, a) == a + 1 }   // reads add(1, 0)
   evidence t0 { for add given a = 0, b = 1 gives 1 }
   evidence r { for add_commutes holds source "review:me" trust 0.9 }
   ```

   `more_one` holds: `add(1, 0)` comes from `add(0, 1)`. A claim never
   uses the vouch on itself. `examples/reworked/vouched`.

2. **The specs of the functions it calls** (`name::f::spec`, declared
   above the claim): when they imply the claim on every input, the claim
   is `assumed on` those inputs and holds as far as the spec claims do
   (`holds assuming [clock::back::spec]` when one is not verified). When
   the specs say the claim is false on an input, that is error M007: the
   claim fails there, with the claim and the specs as candidates, and no
   evidence needed. `examples/reworked/clock`.

3. **The claims on the functions it reaches**, for a claim with inputs.
   A function is reached when the claim calls it, when the body of a
   reached function calls it, or when a reached function declares it
   with `uses`. The claims above that name only reached functions are
   the candidates, vouched or not. When some of them imply the claim on
   every input, it is `assumed on` its whole domain, and holds as far as
   they do. Mitra keeps the fewest that are enough and prints them.

   ```rust
   fn add(a: Int, b: Int) -> Int;
   claim add_commutes = commutative(add, Int);
   fn sum3(a: Int, b: Int, c: Int) -> Int { add(add(a, b), c) }
   claim sum3_swaps(a: Int, b: Int, c: Int) { sum3(a, b, c) == sum3(b, a, c) }
   ```

   `sum3_swaps: holds assuming [add_commutes, sum3::spec]`, on every
   integer. A premise that fails stops it. `uses` alone carries nothing:
   it puts the claims on `add` in reach, and a body or a claim such as
   `plus(a, b) == add(a, b)` says how the two are tied. The reasoning
   treats code as unknown: equal inputs give equal outputs, the premises
   are tried on the terms at hand, and nothing is known of `+` or `<` on
   unknown values. It may miss a consequence, never invent one. Records,
   lists and `match` on unknown values are not followed.
   `examples/reworked/arithmetic`.

## Boundaries

- Two different outputs on one input of a boundary function both stand:
  no contradiction. A claim that reads it is judged on every answer seen,
  and fails if one answer breaks it.
- A boundary function is never a candidate in an error: there is no code
  of yours to fix. The candidates are the subjects, the claim, the
  evidence and the assumptions.
- An `assume` of the shape `f(…) == e` gives the output where none was
  observed, as a vouched claim does. Every claim that read it is
  `holds assuming [the assumption]`, with warning M102. The assumption
  itself prints `holds assuming [itself]`, without a warning, until
  observations cover it; then it holds, or fails.

`examples/reworked/network`.

## `correct(…)` as a value, `represented` and `uniform`

`correct(f(a, b))` alone in a claim is the claim on the code. Inside a
larger expression it is a value with three states: `true` when evidence
judges that call right (a `holds` on the function, or an observed output
that a spec of the function gives), `false` when wrong, and no value
when nothing says. Where it has no value the claim is not judged.

```rust
claim mirror(a: 0..=2, b: 0..=2) { correct(add(a, b)) <=> correct(add(b, a)) }
```

- Both calls judged, and differently: `mirror` fails there.
- `mirror` vouched for on the whole claim: it settles the calls nothing
  judges. `add(0, 1)` judged right makes `add(1, 0)` right: a claim on
  the code of `add` prints it under `assumed on …, by mirror`. This
  works on a finite set of inputs, for calls written with the claim's
  inputs (`add(b, a)`, not `add(a + 1, b)`).

`std::claims` writes `represented` and `uniform` with it; `x` and `y`
stand for the whole inputs of `f`:

```rust
claim represented(f, S) { given x: S, y: S then correct(f(x)) <=> correct(f(y)) }
claim uniform(f) { given x, y then correct(f(x)) <=> correct(f(y)) }
claim uniform(f, case) { given x, y when case(x) == case(y) then correct(f(x)) <=> correct(f(y)) }
```

Mitra reads a claim of this shape, whatever its name, as a set or as
classes, and works on them whole: this is the only place where one name
may stand for all the inputs of a function, and it also works on types
too large to list. The shape is exactly: two inputs, both with the same
set or both with none, an optional `when k(x) == k(y)`, and
`correct(f(x)) <=> correct(f(y))` (or `==`).

Nobody can compute these claims: someone vouches for them (`evidence r {
for div_rest holds source "review:me" }`). For a claim on the code of
`f` (`correct(f(…))`, or a spec) and each `represented` claim on `f`:

- an input of the set passes and none fails: the rest of the set is
  `assumed on …, by` the claim. Not vouched for, the claim is among what
  the verdict assumes (`holds assuming [div_rest]`) and has warning M103;
- an input passes and another fails: the `represented` claim fails with
  error M002, quoting the two, and stands for nothing;
- no input of the set is tested: nothing. `mitra plan` then owes
  `any one point of { (a, b) | b != 0 }`.

The set of `represented` is `{ (a, b) | cond }` over the inputs of the
function (a finite set of inputs only), or a set per input: `0..=9`,
`(0..=9, 1..=9)`, on any type. Several `represented` on one function;
one `uniform`, which sorts every input. `examples/reworked/represented`.
