# 6. Rules and proofs

## Rules

A **rule** says that some statements give another:

```text
rule name: premise, premise |- conclusion;
```

Read `|-` as "gives". From [examples/fall/fall.mitra](../../examples/fall/fall.mitra):

```mitra
        rule slows: drop::falls, speed::slow |- drop::slow;
```

"If the stone falls as `falls` says, and `speed::slow` holds, then
`drop::slow` holds." This rule has no proof: it is **assumed**, like a
claim nobody tested. Mitra reports it as resting only on the model,
until evidence or a proof holds it.

A rule works both ways when Mitra judges the evidence: premises true
make the conclusion true; a conclusion seen false makes some premise
suspect. That is how `blame` finds a culprit.

## Proofs

`by …` after a rule gives its proof. Mitra's checker runs it once, and
if it shows the rule, the rule is held by the checker, the strongest
evidence there is. Most often you only name the step, and Mitra finds
its arguments among the premises. From
[examples/language/proofs.mitra](../../examples/language/proofs.mitra):

```mitra
        rule up: correct(half, (4)) |- correct(half, (5));
        rule five: half::up, correct(half, (4)) |- correct(half, (5)) by ponens;
```

`up` is assumed: "where `half` is right at 4, it is right at 5". `five`
is shown: `up` applied to its premise, which is modus ponens.

| Step | Shows | Example in proofs.mitra |
|---|---|---|
| `ponens` | a rule applied to its premises | `five` |
| `tollens` | a premise false, from a conclusion seen false | `four` |
| `cases` | an `all`, one case per point | `low` |
| `instance` | an `all` at one of its points | `three` |
| `witness` | an `any`, from one point | |
| `intro` | an `all`, from any point | |
| `reductio` | by contradiction | |

```mitra
        rule four: half::up, !correct(half, (5)) |- !correct(half, (4)) by tollens;
```

When no proof fits, or two different ones do, it is an error (M032)
that lists them; a proof that does not show its rule is M012.

Lean's tactics `apply`, `cases` and `exact` are the same idea: name the
step, the tool finds the arguments. Rust has no proofs.

### In full

A step can also be written with its arguments, when you want to say
exactly which. From [examples/fall/fall.mitra](../../examples/fall/fall.mitra):

```mitra
        rule lands: drop::slows, drop::falls, speed::slow |- drop::slow
            by ponens(drop::slows, drop::falls, speed::slow);
```

`ponens(r, p, q)` applies rule `r` to premises `p` and `q`. Every full
form is in the [design notes](../design/syntax.md).

### Proofs on trees and loops, and none at all

A claim of a function over a tree can be shown with
`claim c: … by induction t;` (see [Trees](4-values.md#trees)), and a
claim of a function whose body is a loop with `by invariant` (see
[Claims kept by a loop](4-values.md#claims-kept-by-a-loop)).

A proof is never required. A model where every rule is assumed is a
fine model: its rules are hypotheses the evidence will test.

## Propositions over many points

A rule can speak of where a function is correct, at one point or over
a set. From [examples/div/model.mitra](../../examples/div/model.mitra):

```mitra
        rule everywhere: div::spec |- all x in (a: 0..10, b: 0..10): correct(div, x);
```

- `correct(div, (6, 2))`: `div` gives what the model says at `(6, 2)`.
- `all x in (a: 0..10, b: 0..10): correct(div, x)`: at each of the 100
  points.
- `any x in (…): p`: at one of them at least.

Ranges leave out the end, as Rust's `0..10`; a range with no number in
it, such as `3..3`, is an error (M034). A condition narrows a set:
`all x in (a: 0..10, b: 0..10) where b != 0: …`. A condition that
leaves no point is a finding (X013): the `all` or `any` then learns
nothing from its points.

A condition may call a model function that has a body. The set is then
defined by the model's body, not by what the tied code does. From
[examples/language/sets.mitra](../../examples/language/sets.mitra),
where `f(a) < 5` keeps the points 0 to 3:

```mitra
subject Sets {
    fn f(a: integer) -> b: integer { a + 1 }
    fn g(a: 0..=9) -> b: 0..=9 { a } where {
        claim c: 0 <= b;
        // The points 0 to 3: those where f(a) < 5.
        rule small: g::c |- all p in (a: 0..10) where f(a) < 5: correct(g, p);
        // Proved by its four points, which check lists through f.
        rule listed: correct(g, (0)), correct(g, (1)), correct(g, (2)), correct(g, (3))
            |- all p in (a: 0..10) where f(a) < 5: correct(g, p) by cases;
    }
}
```

A function with no body, or one that calls such, is M023: write the
bound by hand, `(a: 0..4)`. Each point is read within the budget
(`[budget]` in `mitra.toml`); a point not read to its end is X009, and
the `all` or `any` then learns nothing from its points.

A term of a rule can also be a `bool` expression written in place, in
braces: `{ v < 21.0 }`.
