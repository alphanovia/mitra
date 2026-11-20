# Keywords: acts and voices

Part of the [language reference](language.md); index in
[keywords.md](keywords.md). Every block fenced `mitra` is a whole model,
`evidence` is evidence for it, `text` is what `mitra eval` prints for
the two. A test runs them all.

Evidence may say what was done and by whom. A verdict then says on
whose word it rests. Nothing here changes a verdict.

## act

`act test`: what was done. It is checked against what the evidence
says and what it is on; an act that cannot say it is refused.

| Act | Can say |
|---|---|
| `test` | an output, or `holds`/`fails`, at one input; `passes` |
| `exhaustive` | `holds`/`fails` over the set it states with `given` |
| `sample` | `act sample 1000`: how many were drawn ([sampled](kw-evidence.md#sampled)) |
| `review`, `proof` | `holds`/`fails` of anything: a claim, a capture, a tie |
| `measure` | a rate |
| `tie` | `f::tied`, `f::uses` |
| `report` | `fails` only: someone met a failure |

On a claim over every input, `for shipping::positive holds act test`
is refused: ``a test is a point: over a set of inputs it is `act
exhaustive` or `act sample n`, and on a whole claim a review or a
proof``.

Without `act`, evidence is read as it always was, and its act is told
from its shape: a sample, a measure, a test (`gives`, `passes`, a
source `test:`), a tie (a source `reader:`), otherwise a review.

## by, written

`by "kind:…"`: who did it. `written "kind:…"`: who wrote what a
machine ran. A voice is `agent:vendor/model/session`, `human:name` or
`machine:tool`.

The voice behind evidence is `written` when there is one. A machine
that ran a test is not its author: a test with no `written` is the
word of a voice that is `unknown`.

```mitra
subject shipping {
    fn fee(zone: 0..=2, weight: 1..=3) -> Int;
    claim positive(zone, weight) { fee(zone, weight) > 0 }
    claim one() { fee(0, 3) == 12 }
}
```
```evidence
evidence t { for shipping::fee given zone = 0, weight = 3 gives 12 act test by "machine:cargo" written "agent:anthropic/opus-5.5/s-8f2a" source "test:t_fee" }
evidence r { for shipping::positive holds act review by "agent:anthropic/fable-5.2/s-11c0" cold }
```
```text
shipping::fee::correct: unverified
  no evidence on  (zone, weight) in (0..=2, 1..=3)
shipping::fee::deterministic: unverified
  no evidence on  `shipping::fee` giving one output per input
shipping::positive: holds
  by              review (agent:anthropic/fable-5.2/s-11c0 cold); doubt 0.15
shipping::one: holds
  by              test (agent:anthropic/opus-5.5/s-8f2a); doubt 0.3
```

The line `by` is under a verdict that holds when a voice behind it is
named:

- The acts, each with its voices.
- `voices a model apart`: how far apart the two furthest voices are:
  `same`, `session`, `model`, `vendor`, `kind` (an agent and a person).
  Two people, or two machines, are a `kind` apart.
- `doubt`: how likely it is that every word behind the claim is wrong.
  The product of the error rate of each kind of voice
  ([settings.md](settings.md#doubt)), one factor per voice that is its
  own: a model of a vendor, cold or not; a person; a machine; a proof;
  the unknown. A promise is doubted as its most doubted part; its
  capture, or an opaque function under it, with no word behind it is
  doubt 1.

A word on single inputs of a function speaks for `f::correct` and for
claims on single calls (`one`). It does not speak for a claim on every
input (`positive`).

## cold

`cold`: the act was done with the model and the code only, without the
context of whoever wrote them. A cold agent is a kind of voice of its
own, with its own error rate.

## of

`of "test:t_fee"`: a review of the evidence of that source, not of the
claim. It judges nothing; the evidence it names carries a second
voice. Its `trust` is how sure the reviewer is that the test is
correct, which a [bias](settings.md#a-bias-and-a-threshold) uses.

```mitra
subject shipping {
    fn fee(zone: 0..=2, weight: 1..=3) -> Int;
    claim one() { fee(0, 3) == 12 }
}
```
```evidence
evidence t { for shipping::fee given zone = 0, weight = 3 gives 12 act test by "machine:cargo" written "agent:anthropic/opus-5.5/s-8f2a" source "test:t_fee" }
evidence r { for shipping::fee holds act review by "agent:anthropic/fable-5.2/s-9" of "test:t_fee" cold }
```
```text
shipping::fee::correct: unverified
  no evidence on  (zone, weight) in (0..=2, 1..=3)
shipping::fee::deterministic: unverified
  no evidence on  `shipping::fee` giving one output per input
shipping::one: holds
  by              review (agent:anthropic/fable-5.2/s-9 cold), test (agent:anthropic/opus-5.5/s-8f2a); voices a model apart; doubt 0.05
```

## From the tools

- `mitra tie` writes `act tie by "machine:mitra-rust"`.
- `mitra ingest` writes `act test by "machine:cargo"` (or
  `machine:junit`), or the act the comment states: a test that loops
  over a whole set says `// mitra: evidence for steps given h in
  0..=22 act exhaustive`. Who wrote a test is said in its comment, after what
  it pins: `// mitra: evidence for fee given zone = 0, weight = 3 gives
  12 written "agent:anthropic/opus-5.5/s-8f2a" cold`.
- `mitra add` takes the block as written.
- `mitra assess` ends with how many promises hold and on what kind of
  word, and which a person's review would lift the most.

## What is not there

- A review that says `fails` of a piece of evidence is kept and shown
  nowhere yet: it does not set the evidence aside.
- Nothing checks that a voice is who it says.
- No act is inferred as `exhaustive`.
