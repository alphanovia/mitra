# Design: requirements and traceability

Status: **proposal**, not implemented. A layer on top of the core: it must
not shape the core.

## Claims are per property, not per function

A claim states one property precisely enough to be judged. A function can
carry many claims (one per input region, one per spec), a claim can span
several functions (`minus(a, b) = add(a, negate(b))`), and most helper
functions carry none: they are covered by the claims of their callers.

## A requirement is a prose proposition

Mitra can't evaluate prose, so a requirement is opaque, like a subject: an
id and its text. It is linked to the claims that together establish it,
with the same form as a derived claim:

```lean
-- sketch
requirement req12 "Checkout never charges a customer twice" :=
  by derive [charge_once, retry_idempotent]

claim charge_once : forall order in Orders, correct(charge(order))
claim retry_idempotent : forall order in Orders, correct(retry(order))
```

`req12` holds when the claims hold **and** the implication holds: that
these claims really say what the prose means. Mitra can't check that
implication, since one side is prose. It is an empirical claim
([composition](composition.md#1-composition-is-implication)) and gets
evidence like any other, usually a review:

```lean
@[source "review:alice", rev "3f2a1c"] evidence req12.derive at 1   -- sketch
```

That is the validation gap traceability tools skip: a link between a
requirement and a test says nothing about whether the test checks what the
requirement means. Here the link itself is a reviewed assumption.

## What it gives

- **Coverage per requirement**: not "a test is linked", but the
  requirement's verdict, with what is assumed and what is missed.
- **Impact of change**: when evidence goes stale, the requirements that
  lost support are listed.
- **Orphans both ways**: requirements with no claim, and claims or subjects
  with no requirement above them.
- **Root cause upward**: a bug report shows which requirements it breaks,
  not just which function.

This is the traceability that DO-178C and ISO 26262 demand, and that
assurance cases (GSN) draw by hand.

## Open questions

1. How requirements refine into sub-requirements before reaching claims.
2. How evidence names the implication (`req12.derive` is a placeholder;
   see composition's open question 4).
3. Whether structural claims (`uses`) are needed to tie claims
   to the code the requirement is about.
