# Trials

What the toys read as, with `model/verdict.mitra`: a state for each
piece, found by logic from the evidence taken as true, then how sure
it is. The trials of the counts calculus that came before are in
[trials-counts.md](trials-counts.md).

## The speed of a fall

`cargo run --example falling`, tests in `tests/falling.rs`. Read by
`verdict`: a state for each piece, found by logic from the evidence
taken as true, then how sure it is. One statement, `speed`: after
falling a height h, the speed v satisfies v² = 2·g·h within 2 m²/s².
A stone and a feather are each tied to it. A measurement is evidence
about three pieces together: the object's tie, the claim at that
height and speed, and the tie's completeness. `speed::uniform` says
the measured falls stand for all, and one review says yes to it. A
statement and a tie are expected at one half, a completeness at 0.6,
a measurement is believed at 0.9.

| measured | says |
|---|---|
| stone: 5 m, 9.9 m/s | yes |
| stone: 10 m, 14 m/s | yes |
| stone: 20 m, 19.8 m/s | yes |
| feather: 5 m, 2 m/s | no |
| feather: 10 m, 2.1 m/s | no |

| after | verdict | `speed` | at the stone's 5 m | at the feather's 5 m | stone's tie | its completeness | feather's tie | its completeness |
|---|---|---|---|---|---|---|---|---|
| one fall of the stone | true 0.91 | true 0.91 | true 0.91 | unknown | true 0.91 | true 0.93 | unknown | unknown |
| the stone alone | true 0.91 | true 0.91 | true 0.95 | unknown | true 0.97 | true 0.97 | unknown | unknown |
| the feather alone | false | unknown | unknown | unknown | unknown | unknown | unknown | unknown |
| both | false | true 0.91 | true 0.95 | true 0.91 | true 0.97 | true 0.97 | unknown | unknown |

the feather alone: cannot all be true: feather applies, speed @ h = Many([Whole(5), Whole(1)]), v = Many([Whole(2), Whole(1)]), complete: feather applies
the feather alone: cannot all be true: feather applies, speed @ h = Many([Whole(10), Whole(1)]), v = Many([Whole(21), Whole(10)]), complete: feather applies
both: cannot all be true: feather applies, complete: feather applies

One fall of the stone makes its tie, its completeness and the claim
at that height true, and with `uniform` the claim itself. Three falls
make the tie surer (0.97): three reports would have to be given up.
The claim stays at 0.91: every way to it goes through the one review
of `uniform`.

The feather after the stone is not a conflict. The claim is true, so
it is true at the feather's heights; each of the feather's
measurements is false; so the feather's tie and its completeness
cannot both be true. They are listed, and nothing says which. The
verdict is false: something tied in this model is wrong. The feather
alone leaves the claim unknown, and names three suspects.

### The same claim read by bounds

`claims::bounded` reads the claim with each input known to lie in a
range. Here a whole range is worth ten observations, a number given
by who asks, so the claim over the margins is surer than at a point.

| the stone, 5 m at 9.9 m/s | `speed` there |
|---|---|
| read at the point | true 0.91 |
| read over its margins | true 0.99 |

Not shown: a statement for the air.

## A function and its test

`cargo run --example double`, tests in `tests/function.rs`. The
function `double(n)` has the body `n * 2` and a tie to its code. A
test at 3 is evidence about the tie, `double::correct` at 3 and the
tie's completeness together. `uniform` is reviewed by one voice.

| | verdict | `double::correct` | at 3 | at 7 | `double::uniform` | the tie | its completeness |
|---|---|---|---|---|---|---|---|
| no `uniform`, a test at 3 | unknown | unknown | true 0.91 | unknown | unknown | true 0.91 | true 0.93 |
| no `uniform`, then a run at 7 gives 15 | false | false 0.91 | true 0.91 | false 0.91 | unknown | true 0.91 | true 0.93 |
| `uniform` not reviewed, a test at 3 | unknown | unknown | true 0.91 | unknown | unknown | true 0.91 | true 0.93 |
| `uniform` reviewed, a test at 3 | true 0.91 | true 0.91 | true 0.91 | unknown | true 0.91 | true 0.91 | true 0.93 |
| then a run at 7 gives 15 | conflict | conflict | conflict | conflict | true 0.91 | conflict | conflict |
| tests at 1, 2, 3 and 4, `uniform` reviewed | true 0.91 | true 0.91 | true 0.95 | unknown | true 0.91 | true 0.97 | true 0.98 |
| then a run at 7 gives 15 | conflict | conflict | conflict | conflict | true 0.91 | conflict | conflict |

After four tests and the run at 7, what may be given up, the lightest first:
- the run at 7: then `double::correct` is true 0.91
- the review of uniform: then `double::correct` is false 0.91
- the run at 1, the run at 2, the run at 3, the run at 4: then `double::correct` is unknown

- Without `uniform`, passing tests never make `double::correct` true:
  it is unknown, and so is the verdict. One failing run makes it
  false, because the tie and its completeness are true by the test
  at 3.
- With `uniform` reviewed, one passing test makes it true, as sure as
  the review (0.91). Four tests make the tie surer (0.97) and leave
  the claim at 0.91.
- With `uniform` reviewed, a failing run is a conflict: by `uniform`
  the claim is true at 7, and the run says it is not. Everything the
  conflict reaches is marked. What may be given up is listed, and
  each choice leaves a plain state.
- A verdict that holds shows how sure it is: its least sure
  statement.
