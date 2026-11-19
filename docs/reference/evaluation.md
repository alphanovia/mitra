# Evaluation and output

Part of the [language reference](language.md); says what the code does
today, and changes in the same commit as the code.

## Evaluation

Every piece of evidence counts, whatever its trust. For each claim, with
D the inputs it speaks of, Mitra gathers where the claim is right and
where it is wrong:

1. Evidence on the claim itself (`holds`, `fails`), within D.
2. For `correct(f(…))`: `holds` and `fails` evidence on `f`.
3. For any other claim, on each input (all of D when finite, otherwise
   the inputs the observations speak of): compute the expression, a call
   of the code giving the output observed there (or what a vouched claim
   or a spec gives, see [claims.md](claims.md)). True is right, false is wrong, no value
   is not judged. `&&` and `||` are decided by one side when it settles
   them.

An input both right and wrong is a **contradiction**, and so are two
outputs for one input of a subject: an error, and the input counts on
neither side. Then:

- *refuted*: wrong and not right.
- *tested*: right and not wrong.
- *assumed*: what other claims give: the inputs a `uniform` or
  `represented` claim covers from a passing input, and the inputs the
  specs or the claims on what it reaches imply.
- *missed*: the rest of D.

The **verdict** is `fails` if something is refuted; `unverified` if
something is missed; `holds` if nothing is missed and every claim the
assumed inputs rest on holds; `holds assuming [a, b]` when those claims
are not verified or not vouched for. There is no number on a claim.

**Uniform.** For `u = uniform(f, g)`, the class of an input is `g` of
it. A class with a passing input and a failing one is a conflict: `u`
fails, whatever vouches for it (M004). Otherwise, for every other claim
on `f`, the inputs of a class with a passing input are assumed; when
nobody vouches for `u`, such a claim only `holds assuming [u]`. While `u`
holds, a failing input refutes its class. This is done input by input up
to 10 000 inputs; on a larger set every value of `g`'s result type needs
a passing input. A claim that calls several functions is sorted by one
class per call.

**Measures.** `rate(…)`, `precision(f)`, `recall(f)` take the value of
their measurement (`gives rate = 0.9`) when there is one; without one
they are counted from the observed outputs when these cover the whole
set. Two measurements that differ, or one the observed outputs exclude,
are error M005.

**Requirements.** An `ensure` of a subject holds as far as every claim
of the subject, its capture (`name::captured`) and what it requires:
`unverified` with no claim in the subject, `holds assuming […]` while
some are not settled or the capture is not vouched (M103), `fails` when
one of its claims fails or evidence against it stands. In a boundary it
rests the same way on the `assume` lines of its block; with none it is
assumed: `holds assuming [itself]`.

**Errors and candidates.** An error is something that can't all be true.
Its candidates are the pieces of which at least one is wrong: evidence,
the code of the functions involved (and of what they use), and what the
model states (the claim, the signature, an assumption, a `uses` clause).
Code has no trust; what the model states is trusted at 1 unless it says
`trust n`. Evidence outside a signature is kept and reported as M006,
with the evidence and the signature as candidates. A boundary is never a
candidate. An assumption prints as `` `clock::noon` as assumed ``, a
`uses` clause as `` `timer::left::uses` as declared ``. For evidence against
an `ensure`, the candidates are its text (no trust), the word that the
claims say it (the trust of its vouch), what it requires, and the
evidence; never the code.

**Most plausibly wrong.** For each error Mitra computes, for every
candidate, the belief and the plausibility that it is wrong, with
Dempster's rule (`mitra_ds::culprits`): a piece trusted at `t` puts `t`
on "this piece is right"; each error says its pieces are not all right.
Several pieces can be wrong at once, and errors that share a piece are
computed together. When the pieces trusted at 1 can't all be right, the
doubt is shared equally between them. More than 20 pieces to weigh are
not ranked.

**Suspects and root cause.** Through the `uses` clauses, a function that
calls a function with a failing claim has it as a suspect (`look first
at …`). A function with a failing claim, no suspect and a `uses only`
that holds is a root cause.

## Output

`eval` prints each claim, in model order, with the lines that say
something:

```text
calc::div::spec: unverified
  passes on       `calc::div(6, 2)`
  assumed on      89 inputs, by `calc::div_rest`
  no evidence on  (a, b) in {(0, 0), (1, 0), (2, 0), (3, 0), (4, 0), (5, 0), (6, 0), (7, 0), (8, 0), (9, 0)}
calc::div_small: holds
```

`fails on`, `passes on`, `assumed on … by` and `no evidence on`. Up to 3
inputs print as calls, more as a set named after the inputs, and a set
longer than 160 characters as its size. Names are whole:
`subject::claim`, `subject::f::spec` for the claim a body adds,
`subject::f::uses`, `subject::f::tied`.

Then each error and each warning, like a compiler's:

```text
error[M001]: evidence refutes a claim
  --> model.mitra:4  fn div(a: 0..=9, b: 0..=9) -> 0..=9 { if b == 0 { 0 } else { a / b } }

  `calc::div::spec` fails on `calc::div(8, 4)`.
  The evidence that says so:

    output    evidence.mitra:3  evidence bug_report { for div given a = 8, b = 4 gives 4 trust 0.8 }

  The code, the claim or the evidence is wrong. Below, the trust given to each
  piece, and what is left of it once they are weighed against each other:

                                                      trust  left
    possible  the code of `calc::div`, model.mitra:4  none   0.00 to 0.71
    likely    evidence.mitra:3 (bug_report)           0.8    0.29 to 0.36

  The lower what is left, the more plausibly the piece is the wrong one.
  A range means the evidence does not settle it.

  help: if the evidence is right, fix `calc::div`; if not, correct the evidence
```

`left` is what remains of the trust in a piece: 1 minus the plausibility
that it is wrong, to 1 minus the belief. The word is `certain` (belief
at least 0.995), `likely` (at least 0.5), `possible` (plausibility at
least 0.5), else `unlikely`. At most 5 pieces of evidence are quoted.

| Code | Severity | Headline |
|---|---|---|
| `M001` | error | evidence refutes a claim |
| `M002` | error | evidence denies an assumption (`uniform`, `represented`, `uses`) |
| `M003` | error | two pieces of evidence contradict each other |
| `M004` | error | an assumption is contradicted by the evidence: a pass and a failure in one class |
| `M005` | error | a measurement does not fit another measurement of the same measure, or the observed outputs |
| `M006` | error | evidence does not fit the model: inputs or an output outside the signature of its function |
| `M007` | error | a claim contradicts the specs of the functions it calls, with no evidence needed |
| `M101` | warning | a claim is not verified: some inputs have no evidence |
| `M102` | warning | a claim holds only under assumptions |
| `M103` | warning | an assumption is not vouched for |
| `M104` | warning | evidence that can't be used: it is about a helper, which is computed |
| `M105` | warning | a `require` on a promise of a block that nothing beside it leans on |
| `M106` | warning | a function `uses` a block that makes promises, and no `require` beside it names one (only where the block states an `ensure` or a `require`) |

The codes are stable. `mitra explain <code>` says what one means and how
to fix it.
