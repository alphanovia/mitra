# Evidence with an act and a voice

Status: **built** (2026-11-19, increment 76). Reference:
[kw-acts.md](../reference/kw-acts.md), [settings.md](../reference/settings.md).

The user: "we do need to rework evidence and have a type on it. I
think the diagnostic needs to state if the product holds only on
untrustworthy agent review or not." "A better structure on evidence
with source, and type of act. This way checking the evidence against
the claim becomes easier and we can have a policy system on top of
build with a default rule. By default it shouldn't be too hard to get
the holding verdict." And: "evidence doesn't have to be typed in how
we feed them, but they are typed on the claim they support. A claim of
uniform can't be hold by a point test."

## Shape

```
evidence e1 { for fee given zone = 0, weight = 3 gives 12
              act test by "machine:cargo" written "agent:anthropic/opus-5.5/s-8f2a" }
evidence e2 { for shipping::never_free::captured holds act review by "human:ana" }
evidence e3 { for fee holds act review by "agent:anthropic/fable-5.2/s-11c0" of "test:t_fee" cold }
```

- `act`: what was done. `test` (a point), `sample`, `exhaustive` (a
  set run through), `proof`, `review`, `measure`, `tie`, `report` (of
  a failure).
- `by`: who did it. `written`: who wrote what a machine ran; that is
  the voice, since a machine that runs a test is not its author.
- `cold`: done with the model and the code only, without the context
  of whoever wrote them.
- `of "source"`: a review of that piece of evidence. It judges nothing
  of the claim; it is a second voice on the evidence. The user: "Can
  you have evidence of review of a test tie to reinforce a test
  evidence?"

A voice is `agent:vendor/model/session`, `human:name` or
`machine:tool`. "Claude is not enough": the model and the session are
part of the name.

## What an act can say

Checked when the evidence is read, and refused with the reason:

| Act | Can say |
|---|---|
| `test` | an output or `holds`/`fails` at one input; `passes` |
| `exhaustive` | `holds`/`fails` over the set it states |
| `sample` | how many were drawn, passed or not |
| `review`, `proof` | `holds`/`fails` of anything, a capture and a tie included |
| `measure` | a rate |
| `tie` | `f::tied`, `f::uses` |
| `report` | `fails` only |

Evidence that states no act is read as before, and its act is told
from its shape.

## No number on `holds`

The verdict is unchanged. Under it, a line says on whose word it
rests, how far apart the voices are (same, session, model, vendor,
kind), and a doubt: how likely it is that every word behind it is
wrong. The user rejected a number for "how true" and proposed "how
likely I'm wrong". The doubt is the product of the error rate of each
kind of voice behind the claim (`[doubt]` in the settings); a promise
is doubted as its most doubted part.

`mitra assess` ends with the count the user asked for: how many
promises hold, on what kind of word, and which a person's review would
lift the most.

## Policy

`[policy]` in the settings, on top of the verdicts: what a claim or a
promise must rest on. By default nothing is asked: any act that can
hold the claim makes it hold, whoever did it. A piece that holds and
lacks what is asked reads `holds`, with `below policy` under it.
