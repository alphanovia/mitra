# Design: composition and tests

Status: **proposal**. Implemented so far: derived claims
(`:= by derive [...]`), whose implication is checked on finite domains, and
`test` declarations, `uniform` and `uses` claims ([reference](../reference/language.md)). Syntax below
not in the reference is a sketch. Verdicts and trust in evidence are in
[semantics.md](semantics.md).

## 1. Composition is implication

`claim c : … := by derive [p, q]` is two things:

- an **implication** `I : p ∧ q → c`, itself a claim, and
- **modus ponens**: if I and every premise hold, c holds.

| | Definitional | Empirical |
|---|---|---|
| When | the subject's shape is known, and the premises relate each part to a spec | only the dependencies are known, not how they combine |
| Example | `minus(a, b) = add(a, negate(b))` with specs for `add` and `negate` | `fetch` uses `parse` and `http`; nobody wrote down how |
| How I is decided | **checked**: substitute and compare, by evaluation (finite domains, done) or a prover (later) | **evidenced**, like any claim: review, tests of the whole |
| Can evidence refute I? | no, it is computed | yes: on an input where every premise holds but c has a counterexample |

A refuted empirical implication is a diagnosis: the parts are fine, the way
they are put together is not.

## 2. What we can know about a subject's structure

From least to most knowledge:

| Claim | Meaning | Use |
|---|---|---|
| `correct(f(x))` | f does the right thing, as people judge it | today |
| `f(x) = g(x)` | f equals the spec g | today |
| `uses f [g, h]` | f calls g and h | **impact**: a failing claim on g makes f *suspect* (flagged, not failed): implemented |
| `only uses f [g, h]` | f depends on nothing else | **frame**: evidence and changes elsewhere are irrelevant to f |
| `f(x) = h(g(x))` | f's shape | **derivation**: the implication becomes definitional and is checked |

With `only uses` but no shape, premises on g and h do not determine f. The
gap is its own claim, `glue_correct`: "f combines g and h correctly". Then
`c := by derive [p_g, p_h, glue_correct]` is sound, and the glue claim is
where review or end-to-end tests are needed.

Structural claims get evidence like any other claim: a parser reading the
code (near certain), or a human review (less so).

## 3. Tests are subjects too

Today a `test` declares the evidence its result stands for, and that
evidence counts at 1. But a test is code, so what it proves depends on
claims about the test itself:

```lean
opaque t_small : Bool                        -- a test: pass or fail
claim t_ex : forall x in 0..=10, exercises(t_small, add(x))   -- sketch
claim t_judge : judges(t_small, add, add_spec)               -- sketch
```

A pass then **derives** evidence on `add` over `0..=10`, resting on `t_ex`
and `t_judge`: what it supports is `holds assuming` them until they have
evidence of their own. A buggy or
vacuous test is then a named assumption that can take the blame, instead
of silently counting as proof.

`t_ex` says which inputs the test runs, not that one run stands for more.
That **uniformity hypothesis** is a claim on the subject, `uniform f by
classes`, which is implemented
([reference](../reference/language.md#claim)).

A failure gives only "wrong **somewhere** in S", which Mitra cannot record
yet: a failed test over a set gives no evidence today.

## 4. Open questions

1. **Infinite domains.** Checking definitional implications there needs a
   prover (SMT, Lean). Until then, only finite domains are checked.
2. **Existential failures.** How "wrong somewhere in S" shows in the
   output, and how a later pass on part of S narrows it.
3. **Suspects.** How far suspicion spreads through `uses`, and how much
   weight it gets in the opinion ([semantics](semantics.md#4-opinion-which-piece-is-wrong)).
4. **Naming the implication** of a derived claim, so evidence can target it.
