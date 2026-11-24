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

`by …` gives a proof. Mitra's checker runs it once, and if it shows the
rule, the rule is held by the checker, the strongest evidence there
is:

```mitra
        rule lands: drop::slows, drop::falls, speed::slow |- drop::slow
            by ponens(drop::slows, drop::falls, speed::slow);
```

`ponens(r, p, q)` is modus ponens: rule `r` applied to premises `p` and
`q`. A proof that does not show its rule is an error (M012).

The other proof steps, for reference, each with its classical name:
`tollens` (from a false conclusion, a false premise), `reductio` (by
contradiction), `cases` (an all, one case per point), `instance` (an all
at one point), `witness` (an any from one point), `intro` (an all from
any point). Their forms are in the
[design notes](../design/syntax.md).

A claim of a function over a tree can also be shown, with
`claim c: … by induction t;`: see [Trees](4-values.md#trees).

### Short proofs

A proof of one step can name the step alone. Mitra puts the rule's
premises where they fit, and keeps the proof the checker takes. From
[examples/language/proofs.mitra](../../examples/language/proofs.mitra):

```mitra
        rule five: half::up, correct(half, (4)) |- correct(half, (5)) by ponens;
        // Modus tollens: wrong at 5, so wrong at 4 too, or `up` is false.
        rule four: half::up, !correct(half, (5)) |- !correct(half, (4)) by tollens;
        // In full, the same proof.
        rule four_full: half::up, !correct(half, (5)) |- !correct(half, (4))
            by !tollens[half::up, 0, 1](!1, half::up);
```

`by ponens`, `by tollens`, `by cases`, `by instance`, `by witness`,
`by intro` and `by reductio` each take no argument. Mitra tries each
premise, as it is and negated, in each place the step takes, and the
points the rule writes for `instance` and `witness`. When no proof
fits, or two different ones do, it is an error (M032) that lists them:
write the one you mean in full. A premise written in place is read by
its place, `0`, `1`, …, as in `!1` above.

Rust has no proofs; Lean's tactics `apply`, `cases` and `exact` are the
same idea: name the step, the tool finds the arguments.

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

Ranges leave out the end, as Rust's `0..10`. A condition narrows a set:
`all x in (a: 0..10, b: 0..10) where b != 0: …`.

A term of a rule can also be a `bool` expression written in place, in
braces: `{ v < 21.0 }`.
