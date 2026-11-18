# Field trials

Three trials of Mitra on real work, run by hand by the project's author to
learn what Mitra is worth. The [add ladder](validation.md) checks that
Mitra computes the right verdicts on a toy whose truth is known; the trials
check whether it **helps** when the truth is not known. Mitra's code is not
changed for a trial: what is missing is written down as a finding.

Each trial answers the [core hypothesis](vision.md#core-hypothesis): most systems can't be
proven, so what is worth having is good-enough evidence management.

## The three trials

### T1: a small Rust project, built with Mitra

The developer writes a small Rust program, with its claims, `test`s and
store kept up to date as the code grows, the way mitra-ds is
([dogfooding](design/dogfooding.md)).

- **Stresses:** the dev loop (`add`, `ingest`, `next`, `--rev`), claims on
  code that changes, and the cost of keeping the model current.
- **Questions:** Does `next` point at the code worth testing? Does a verdict
  ever change what the developer does? How long does the model take to
  write and to keep up, compared with the code? Are the classes of a
  `uniform` claim easy to write and to review, and do they break where
  the bugs are?
- **Success:** at least one decision (what to test, what to ship) is made
  differently, and better, because of a verdict. **Failure:** the model is
  written once and then left behind, or only repeats what `cargo test` says.
- **Likely gaps:** subjects are not linked to code, so staleness is per
  revision, not per function; no records or lists; confidence is 0 or 1;
  most failed tests give no evidence; no `uses` claims, so a bug can't be
  traced through calls ([composition](design/composition.md)).

### T2: an abstract model of an agile organisation, judged from news

The subject is a system that is not code: an organisation working in an
agile way. The model states what such an organisation should achieve
(claims); the evidence is what news reports say happened.

- **Stresses:** modelling without code, evidence that is second-hand,
  partial and sometimes contradictory, and sources of uneven quality.
- **Questions:** Can the model be written so that a news report is evidence
  for or against a claim, not just a story? Do contradicting reports show
  up as contested, as they should? Does the verdict match what a careful
  reader concludes from the same reports?
- **Success:** the verdicts and the `contested` lines say something a
  reader can check and agree with, and `next` names the evidence that would
  settle a question. **Failure:** every claim needs a judgment call that
  Mitra cannot record, so the verdict is the modeller's opinion with extra
  steps.
- **Likely gaps:** confidence 0 or 1 only (a news report is rarely
  certain); no weighting of sources (`source` is recorded but not used);
  inputs are numbers, strings and booleans, so organisations, teams or
  years become names or numbered cases; no time: evidence can't age except
  by `--rev`.

### T3: a skill and its spec, judging an agent that uses it

The subject is an agent following a skill. The spec says what the skill
should make the agent do; the evidence is the agent's runs.

- **Stresses:** a subject that is not deterministic, outputs that are judged
  rather than computed, and the spec as a `def` versus a description.
- **Questions:** Can the skill's promise be written as claims over a set of
  tasks? Is one run of a task evidence, or does it take several? Does the
  verdict tell which part of the skill fails, and on which tasks?
- **Success:** the verdict ranks the tasks or parts of the skill that need
  work, in a way that matches reading the transcripts. **Failure:** every
  task is uncertain because a run proves nothing at confidence 1, or the
  spec can't be written in Mitra and lives only in prose.
- **Likely gaps:** confidence 0 or 1, while an agent passes a task some of
  the time; a failed run on a set gives no evidence (Mitra can't tell which
  input was wrong); a spec must be a total pure `def`, so "follows the
  skill" is a subject's `correct`, judged by someone; no notion of a judge
  (person or LLM) and how much to trust it.

## Where the outputs go

```text
trials/
  rust/          the project, with its .mitra/ store
  agile/         .mitra/ store, and sources/ with the reports used
  skill/         the skill, .mitra/ store, and runs/ with transcripts
  <trial>/FINDINGS.md
```

`FINDINGS.md` is short and dated, one entry per finding:

- **What happened:** the model, evidence or command, and Mitra's output.
- **Expected:** what a person would have concluded.
- **Kind:** missing feature, wrong verdict, confusing output, too costly,
  or worked well.
- **Cost:** time spent on the model versus on the work itself.

## How findings feed back

Each finding is reviewed together. A wrong verdict becomes a golden case
([reference](reference/language.md#golden-set)) before it is fixed. A
missing feature goes to the design doc it belongs to, or to a new one, and
is built only once the language change is settled. A finding that recurs
across trials ranks higher. A trial that fails in the way listed above is a
result, not a setback: it is evidence for the
[kill criterion](vision.md#what-success-looks-like).
