# Design: composition, tests and confidence

Status: **proposal**, not implemented. What the code does today is in the
[language reference](../reference/language.md). Syntax below is a sketch.

## 1. Composition is implication

Today `claim C: … from P1, P2` means "if P1 and P2 hold, C holds", and Mitra
takes that on trust. It is really two things:

- an **implication** `I: P1 ∧ P2 ⇒ C`, which is itself a claim, and
- **modus ponens**: if I and every premise hold, C holds.

Two kinds of implication:

| | Definitional | Empirical |
|---|---|---|
| When | The shape of the subject is known and the premises relate each part to a spec | Only the dependencies are known, not how they are combined |
| Example | `not_not(a) matches not(not(a))`, `not matches neg`, `neg(neg(a)) = same(a)` | `fetch` uses `parse` and `http`; nobody wrote down how |
| How I is decided | **Checked**: substitute the specs into the shape and compare with the target, by evaluation (finite domains) or a prover (later) | **Evidenced**, like any claim: review, tests of the whole |
| Can evidence refute I? | No, it is computed, like a spec | Yes: on an input where every premise holds but C has a counterexample |

A refuted empirical implication is a diagnosis: the parts are fine, the way
they are put together is not.

## 2. What we can know about a subject's structure

From least to most knowledge:

| Claim | Meaning | Use |
|---|---|---|
| `claim c: f(_) correct` | f does the right thing, as people judge it | today |
| `claim c: f(_) matches g` | f equals the spec g | today |
| `claim u: f uses {g, h}` | f calls g and h | **impact**: a refuted claim on g makes claims on f *suspect* (flagged, not failed) |
| `claim o: f only uses {g, h}` | f depends on nothing else | **frame**: evidence and changes elsewhere are irrelevant to f |
| `claim s: f(x) matches h(g(x))` | f's shape | **derivation**: the implication becomes definitional and is checked |

With `only uses` but no shape, premises on g and h do not determine f.
The gap is made explicit as its own claim, `glue_correct`: "f combines g and
h correctly". Then `C from P_g, P_h, glue_correct` is a sound derivation, and
the glue claim is where review or end-to-end tests are needed.

Structural claims (`uses`, `only uses`, shape) get evidence like any other
claim: a parser reading the code or its AST (near certain), or a human
review (less so).

## 3. Tests are subjects too

A test is code, so what it proves depends on claims about the test itself:

```mitra
subject  t_small() -> bool                        # a test: returns pass or fail
claim    t_ex:    t_small exercises add(x in S)   # it runs add on the inputs S
claim    t_judge: t_small judges add by add_spec  # it compares with add_spec
evidence t_small() -> true @ 1                    # one run: it passed
```

A pass then **derives** evidence: `add(x in S) matches add_spec`, with
confidence = belief that `t_ex` and `t_judge` hold. A buggy or vacuous test
lowers that belief instead of silently counting as proof.

A failure derives the opposite, but only "wrong **somewhere** in S": an
existential counterexample. It refutes any universal claim covering S
without saying where (see open questions).

## 4. What a confidence means, and the calculus

**Confidence c is the probability that the observation is valid**: the run
happened as recorded and the judgment of its result is right.

Per input x, each piece of evidence is a Dempster–Shafer mass on
{correct, wrong}: `correct @ c` gives m(correct) = c, m(unknown) = 1 − c.
Evidence on the same input combines by Dempster's rule with **Yager's
variant**: the conflicting mass goes to *unknown* and is also reported as a
conflict. Each input ends with (b, d, u): belief correct, belief wrong,
unknown, b + d + u = 1.

A claim on domain D, under a profile μ:

- **belief it holds** = ∫_D b dμ / μ(D)
- **belief it fails** = max over x of d(x): one counterexample is enough
- the rest is uncertainty
- a derived claim gets at least b(I) × Π b(Pᵢ) from modus ponens

At `@ 0` and `@ 1` this gives exactly today's golden sets: `@ 1` makes b or
d equal 1 on its inputs, `@ 0` adds nothing, two opposite `@ 1` meet in full
conflict and leave the input unknown. **Strict logic counts only `@ 1`**:
`holds` when b = 1 on all of D, `fails` when d = 1 somewhere.

## 5. Tuples

```mitra
subject  minus(a: int, b: int) -> int
claim    minus_shape: minus(a, b) matches add(a, negate(b))
evidence minus(5, 3) -> 2 @ 1
```

Domains become products; sets of tuples are unions of boxes such as
`(a in [0, 10], b in int)`; patterns work per position (`minus(_, 0)`). A
profile on tuples is a product of per-parameter profiles unless a joint one
is given. `()` is the empty tuple, for tests with no input.

## 6. Open questions

1. **Checking definitional implications on infinite domains** needs a
   prover (SMT, Lean). Until then, only finite domains are checked.
2. **Existential failures.** How a "wrong somewhere in S" shows in the
   output, and how a later pass on part of S narrows it.
3. **Independence.** Combining evidence assumes independent sources. Two
   runs of the same test should not count twice: we need provenance.
4. **From (b, d, u) to a verdict word** when nothing is at 0 or 1: a
   threshold per policy, or no word at all?
5. **Suspect propagation** through `uses`: how far, and does it lower
   belief or only flag?
6. **Naming the implicit implication** of a `from`, so evidence can target
   it (`C.from`?).
