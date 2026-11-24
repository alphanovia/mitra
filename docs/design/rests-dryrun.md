# Dry run: branch and bound for rests (finding 25)

Branch `exp/rests-bnb`, from main a9d1b03. Nothing here is on main. With no
environment variable set, Mitra behaves and prints exactly as on main (all 425
tests pass, dogfood output byte for byte the same).

## How rests work today (checked in the code)

- A rest is one set of takings a proposition follows from. Judgement keeps
  every smallest one (`judgement::Opinion`, `smallest`, `all_of`).
- A rule applied forward gives its conclusion one rest per way of taking one
  rest of each premise (`all_of`). The rule's own link counts as a premise.
- Before applying, explain multiplies the premises' counts. Past
  `[explain] rests` (1,024) the rule is dropped for good (X006). This count is
  taken BEFORE supersets are dropped, so it can overstate a lot.
- Readers:
  - verdict (`explain::verdict`): Explained needs only that SOME rest has no
    `model` taking; Contradicted needs only that some rest against exists.
    Neither needs the list.
  - numbers (`Numbers::prop`): the strongest rest (Subjective Logic product of
    its takings). Shared evidence counts once because the product runs over
    the set. Needs the best, not the list.
  - blame (`blame::blame`): every rest for paired with every rest against is a
    conflict; diagnoses are the smallest hitting sets. Needs ALL of them to be
    complete.

So the coordinator's explanation is right: whole sets are kept so shared
evidence counts once, and blame reads them. Add: blame needs every one, not
the best.

## What the prototype does

- `MITRA_REAL_COUNT=1`: the cap reads the real number of smallest sets
  (`judgement::real_ways`, work bounded at 16 x cap) instead of the product.
- `MITRA_DRYRUN=1`: logs every forward rule application and, for each,
  compares today's listing with `Numbers::strongest_ways`: branch and bound
  that never lists the ways. Bound = product of the expected values
  (b + a u) of the takings so far, times the best each remaining premise
  could add, already-taken takings counting 1. This bound is safe because
  Subjective Logic multiplies expected values exactly and a belief never
  exceeds its expected value (checked on 200,000 random cases). A complete
  way counts only if it is a smallest set (checked in polynomial time), as
  `all_of` keeps. Variants: k best, and independent groups.

## Results

Dogfood (`mitra explain crates/mitra-cli/src/main.mitra --code crates --store
core2.store.mitra`) and all 15 example models: 477 rule applications.

| check | result |
| --- | --- |
| branch and bound = today, where today lists them (202 + 275) | 477/477 equal, best and top 3 |
| independent groups = today | 476/477: wrong once, examples/language/proofs `calc::half::low`, 0.143 vs 0.333 |
| time, all uncapped applications in dogfood | listing 95 ms, branch and bound 16 ms |
| capped `agree::no_false_holds` (product 1,500) | real count 520: under the cap. B&B 0.6 ms, 80 nodes |
| capped `cli::answers` (product up to 23.9 M) | B&B 189 ms, 1,072 nodes; raising the cap to 30 M ran out of memory (> 8 GB) |

What the two X006 hide on the dogfood: nothing visible. Both rules are links
only the model states, so every rest through them holds a `model` taking: the
verdict cannot become Explained that way. Their best rest is 0.0006 and
0.0002, against 0.33 from the direct review already there. Verdicts and
numbers stay the same; only the warning goes.

Blame: unchanged on the dogfood (the capped conclusions have no rest against).
In general a best-only or top-k list loses conflicts, so blame would miss
diagnoses: top-k does not fix it.

## Why groups fail

Mitra's strength is Subjective Logic's product, not a plain product: adding a
taking can RAISE the belief (32 % of random cases), because uncertainty turns
into belief as base rates multiply. So the best of each group, put together,
is not always the best overall. Groups are exact only for a plain product.
Same reason rules out k-shortest paths in -log space and dominance pruning by
belief. (In no dogfood or example case did dropping supersets drop a stronger
rest, but in theory it can.)

## Options

| option | exact? | dogfood X006 | cost |
| --- | --- | --- | --- |
| A. count real sets for the cap | yes | 2 -> 1 | ~30 lines |
| B. raise the cap | yes | 2 -> 1 at 2,000; never 0 (memory) | a setting |
| C. drop supersets | already done | - | - |
| D. k strongest (finding 17) | no: verdicts, numbers downstream, blame | 2 -> 0 | small |
| E. groups | no (shown) | - | - |
| F. keep a capped conclusion as an unexpanded product; verdict by existence, numbers by B&B, blame by a "still conflicts?" check | yes | 2 -> 0 | new form of rest in judgement, numbers, blame |
| G. compiled forms (BDD / d-DNNF, as ProbLog) | yes | 2 -> 0 | F generalised; later |
