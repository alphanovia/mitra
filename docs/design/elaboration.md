# Design: elaboration

Status: **proposal**. Built since, in part: claims that follow from the
specs of what they call (section 1, derived claims), `mitra plan`, which
lists the evidence owed per claim (section 4, without generating stubs),
and library claims (`std::claims`) an agent can instantiate (section 5).
Not built: splitting, mining, obligations for Lean and Verus.

Developers give partial knowledge, such as a few claims, a shape or a
dependency. Mitra **elaborates** it: it works out what follows from that
knowledge, what is missing, and what to write next. Helping to write the
spec or the proof is a feature in its own right.

## 1. Saturation

Apply the rules until nothing new comes out (a Datalog-style closure):

- **Derived claims.** Run modus ponens on every implication whose premises
  hold.
- **Suspects.** If a claim on `g` is refuted and `f uses g`, every claim on
  `f` becomes suspect.
- **Refuted implications.** If every premise holds on x and the conclusion
  has a counterexample at x, the implication is refuted.
- **Glue gaps.** When `f only uses {g, h}` but no shape is known, Mitra
  creates the missing `glue_correct` claim.

The output is the model the developer wrote, plus everything implied by it,
each item marked with where it came from.

## 2. Splitting claims around counterexamples

When a counterexample refutes a universal claim, Mitra proposes to split the
claim's domain: the region around the counterexample, and the rest. Each
part gets its own claim and its own evidence. It can also propose the
"honest alternative", the claim that describes what the subject actually
does. This is close to Imandra's region decomposition.

## 3. Mining specs from evidence

Observed outputs often follow a pattern. Mitra can propose a spec that fits
every observation and is refuted by none, in the style of Daikon's likely
invariants. A proposed spec is only a draft: it becomes a claim once a
person accepts it.

## 4. Generating obligations

For each claim that is unverified, Mitra can write the stub for the evidence
it needs, aimed at the inputs that are still uncovered:

- a **Lean** theorem statement for a definitional implication;
- a **Verus** `ensures` clause on the Rust function;
- a **proptest** generator restricted to the uncovered set, or to the region
  around a counterexample.

## 5. LLMs propose, Mitra checks

An LLM is good at drafting claims, specs, shapes and test stubs. Mitra
gives each draft a verdict like any other claim, so a wrong draft shows up
as refuted or unverified, not as quiet trust.
