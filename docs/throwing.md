# The throwing toy

`tests/verdict.rs`. One object, a ball. One statement, `falls`:
thrown up, it goes up then falls down. One tie, `applies`: this is an
object the statement is about. `lands` is all of the two. A throw is
evidence about `lands`, by my senses, believed at 0.9. A statement and
a tie are expected at one half.

Read by `model/verdict.mitra`: every throw starts true, states spread
by the six rules, and confidence is counted after.

| what happens | `falls` | the tie | verdict | suspects |
|---|---|---|---|---|
| one throw falls | true 0.91 | true 0.91 | true 0.91 | |
| one throw does not fall | unknown | unknown | false | the tie and `falls` cannot both be true |
| it falls, seen by two voices | true 0.95 | true 0.95 | true 0.95 | |
| it falls, said twice by one voice | true 0.91 | true 0.91 | true 0.91 | |
| a ball falls, a balloon does not | true 0.91 | ball's true 0.91, balloon's false 0.91 | false | |
| it does not fall, and physics says `falls` a hundred times | true 1.00 | false 0.91 | false | |

- A throw that falls makes `lands` true, and a true whole makes each
  part true.
- A throw that does not fall makes `lands` false. With nothing else
  known, that refutes neither part: they are listed as suspects.
- With a ball that falls, `falls` is true, so the balloon's failure
  has one part left to be false: its tie.
- A voice counts once for what it says of one piece.
- A review is evidence that says many yes at once. It makes `falls`
  true, so the failing throw falls on the tie.

## A latent statement

`gravity` has no tie and nothing to throw at. "The ball falls" is all
of `gravity` and `mass ball`; "the balloon falls" is all of `gravity`
and `mass balloon`. The ball falls and the balloon does not: `gravity`
is true through the ball, and `mass balloon` is false. Two objects
tell gravity from a mass.

## The shape of the graph

`orbit` and `tide` are each all of `gravity` and one more piece, and
the sky is both.

- One sighting of the sky makes all five true, each as sure as that
  one report: `gravity` is reached by two paths and counted once.
- A sighting that fails names what stands on nothing: `gravity`,
  `moon`, `sea`. A part that is itself a whole stands for its parts.
- With `gravity` and `moon` seen, `orbit` is true, so `tide` is false
  and then `sea`.

## A doubted relation

A relation with a base rate under 1 is assumed like evidence. What
follows through it is no surer than it. Against other evidence it is
among what may be given up, and it comes first when it counts for
least.

## Forty objects and a feather

Forty objects fall, each with its tie and its voice. `falls` follows
from each throw alone, so forty reports would have to be given up for
it to fall: its count is 36. The feather does not fall, and its tie is
false.
