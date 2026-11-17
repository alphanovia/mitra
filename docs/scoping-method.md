# How Mitra is scoped

Every piece of work on Mitra, from the project description down to a single
backlog item, is checked against the same rules before it is accepted. The goal
is a project that is coherent, sized to the time available, and provably done.

## INVEST

Each scope item must pass all six checks. An item that fails one is split,
reworded or dropped, never accepted "for now".

| Letter | Check | Question to ask | Fails when |
|---|---|---|---|
| **I** | Independent | Can this be built, tested and shipped without waiting on another open item? | It only works once another item lands. Merge them or reorder so dependencies are explicit and one-way. |
| **N** | Negotiable | Does it describe the outcome, leaving the "how" open? | It prescribes an algorithm, data structure or library that isn't itself the requirement. |
| **V** | Valuable | Can a named user (see the vision) do something after it lands that they could not before? | Its only value is "enables later work" with no observable result of its own. |
| **E** | Estimable | Can we put a size on it today (see sizing below)? | There is an open research question. Turn that into a time-boxed spike first. |
| **S** | Small | Is it size S or M? | It is L or larger. Split it along a user-visible seam. |
| **T** | Testable | Are the acceptance criteria checkable by an automated test or a scripted demo? | Criteria use words like "fast", "secure", "robust" without a number or a scenario. |

## Sizing

Sizes are in focused working time, so they stay meaningful regardless of how
many hours a week go into the project.

| Size | Focused time | Allowed in a milestone? |
|---|---|---|
| S | up to 1 day | yes |
| M | 2 to 3 days | yes |
| L | about a week | no, split it |
| XL | more | no, it is an epic |

A **spike** is a time-boxed investigation (S or M) whose deliverable is a short
written decision in `docs/decisions/`, not code. Spikes are how unknowns become
estimable.

## Milestones

A milestone is a set of items that together give a user something whole. Each
milestone has:

- a one-sentence outcome, phrased as what a user can now do;
- an exit demo: a script or test that proves the outcome end to end;
- a size budget, the sum of its items, which must fit the time available.

Only the next milestone is planned in detail. Later ones stay as one-line
outcomes until the previous milestone ships.

## Project-level checks

The same idea applied to the project as a whole, checked in the vision doc:

- **Coherent:** one problem statement, one primary user. Everything in scope
  traces back to it.
- **Bounded:** an explicit non-goals list, as important as the goals.
- **Achievable:** milestone 1 fits in the stated time budget with margin.
- **Testable:** "v1 is done" is defined by exit demos, not by a feature count.

## Definition of done (per item)

- Acceptance criteria pass as automated tests in CI.
- Public API is documented, with one example.
- Any design choice made along the way is recorded in `docs/decisions/`.
