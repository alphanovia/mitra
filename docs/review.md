# Review notes of the `core` branch

Increments and decisions go on from `main`'s
`docs/review/2026-11-18.md` (increment 90, decision 556).

## Increment 91: layer 1, the two sides and the seam (2026-11-20)

**Asked.** "how about you start again from scratch on a new branch ?"
"when the branch has converged on a good code, we can rework main
around it." An orphan branch: no history and no file of `main`
(only `LICENSE` and `.gitignore` copied). One crate, `mitra-core`, no
dependency, 362 lines of code for 191 lines of model.

**Decisions taken alone.**

557. An orphan branch, not a branch off `main`: the tree starts empty
     and nothing is inherited by accident. The merge is to be decided.
558. One crate with three modules, not three crates. The wall is a
     test (`tests/wall.rs`). Crates when a layer needs them.
559. `Evidence` has a fourth field, `piece`: its own name. You agreed
     to `{ about, value, by }`; evidence is a piece, so it needs a name
     to be reviewed by, as `Claim` has one. `as_piece` is gone.
560. `Known` carries the base rate of each piece (`base`). Who sets it
     is an outer layer: settings by kind, or the certainty its author
     stated. A piece with none has 0.
561. `apart` knows two levels only, same voice or not. Reading a voice
     as agent, model, vendor is a later layer, and so is the
     interpolation between the two ends, which `fuse` already takes.
562. A new voice is as independent as it is from the nearest voice
     already heard on that piece.
563. A piece is believed on its own evidence or on what it rests on,
     whichever is more (not fused). What it stands on is a set of
     pieces, so one reached by two paths counts once.
564. A piece that leads back to itself (evidence about its own
     evidence, resting on itself) is taken with no evidence there.
565. `conflicts` is a function of its own and changes no opinion: the
     pieces of an `Exclude` that are each believed, and the evidence
     for and against one piece, least believed first.
566. An operator is strict: `&&` computes both sides. `eval` does not
     check: an expression that does not check can still have a value.
567. The evidence of `observe` is named after the claim, the inputs
     and the voice, so the same observation twice is one piece.
568. The verdict word (`holds`, `fails`, `unverified`) is not in the
     core. `tests/goldens.rs` writes it in six lines to compare with
     `main`.

**Still open, from the proposal.** A claim over many inputs (proposed:
a claim at one input is a piece, the whole rests on its points); how
far `machine:mitra` is believed (the seam test sets it to 1 by hand);
where a test of the code enters; the names of the variants; `Piece` as
a level apart; independence as a number; a review by the test's own
author.

**What to review.** The three model files, `ensure` lines first. Then
`src/piece/model.rs` (121 lines): `direct`, `support`, `together`.
Then `tests/goldens.rs` against the table of the calculus note.

**How to test.** `cargo test`. With `main`'s binary, `mitra conform`
in this folder.

## Increment 92: `Model` and `Voice` (2026-11-20)

**Asked.** "You didn't call that model, you called that 'Known'. Why
not model ?" "I saw an apart function taking string on each side,
when they really are about voice of an evidence. Why the erasure ?"

- `Known` is now `Model`. My reason for `Known` was that on `main` a
  model is the text that was written and the ledger is beside it. By
  your definition a model is all knowledge put together, evidence
  included, so `Model` is the right word and `Known` hid it.
- `record Voice { name: String }`; `Evidence.by`, `apart` and
  `observe` take a `Voice`. There was no reason for the `String`: the
  reading of a voice was left out and its type went with it.
- New claim `base_rate_survives`: `opinion(model, x).a == base(model,
  x)`, with `base` a function of the model. It was false for a piece
  believed through what it rests on (the product of the parts' rates
  came out); fixed in `opinion`.

**Decisions taken alone.** 569. A voice is its name and nothing else
yet. 570. `base` is public and tied, since a claim names it.

**Proposed, not confirmed and not in the code: the base rate by kind
of piece.** Sceptical about the system, trusting about evidence.

| Kind of piece | Base rate |
|---|---|
| a claim, an ensure, a capture | 0 |
| a test nobody reviewed | 0.6 |
| a review | the certainty its author states |
| a tie | near 1 |
| an assume | 0.5 |
| a bug report | 0.9 |
| what Mitra computes itself | 1, for now |

Kinds of piece and settings are a later layer; today a model carries
its base rates as given.

## Increment 93: three subjects in `piece.mitra` (2026-11-20)

**Asked.** "you need to split the subjects in it, because model and
evidence have there own rational for existing, so they shouldn't be
attached to the same ensure as piece." One file, three subjects, each
with its own promises, and a `require` where one needs another:

- `piece`: `everything_is_a_piece`, `opinion_is_four_numbers`,
  `trusted_is_the_policy`. Holds `Piece`, `Opinion`, and what reads
  one opinion alone (`projected`, `reaches`, `against`).
- `evidence` (requires `piece`): `evidence_is_a_piece`,
  `said_as_far_as_believed`, `same_voice_counts_once`,
  `disagreement_is_measured`. Holds `Voice`, `Evidence`, `apart`,
  `moved`, `fuse`, `conflict`.
- `model` (requires three promises of `evidence`):
  `only_from_the_model`, `rests_is_all`, `conflict_is_said`. Holds
  `Relation`, `Related`, `Model`, `base`, `opinion`, `trusted`,
  `conflicts`, `all`.

The code follows: `src/piece.rs`, `src/piece/evidence.rs`,
`src/piece/model.rs`. The seam now requires
`evidence::evidence_is_a_piece`, since what it makes is evidence.

**Decisions taken alone.** 571. Each function sits with the promise
it serves: `moved` and `fuse` with evidence, `all` with the model,
`reaches` with the piece. 572. Two promises are new,
`opinion_is_four_numbers` and `disagreement_is_measured`, since the
split left `Opinion` and `conflict` with no promise of their own.
573. Four small claims are new, one per promise that had none in its
subject.

## Increment 94: `model.mitra` with six subjects (2026-11-20)

**Asked.** "how about we rename piece.mitra, model.mitra, and we put
location, piece, and model in it ? so that we get evidence, statement
and tie as the core of the system". "statement is an interpretation of
the system, tie is a believe link between an interpretation and a
named part of the system."

`model/model.mitra` (342 lines; the ceiling of `main` is 1,000) holds
`place`, `piece`, `evidence`, `statement`, `tie`, `model`. It is also
the entry file of the store: that works. The code follows:
`src/model.rs` and `src/model/`. The seam fills the content of a
statement: `Formal { expression }` or `Prose { text }`.

**Decisions taken alone.**

574. `Statement<C>` is generic in its content, not `record Content;`.
     Opaque inside is what a type parameter means in Mitra, and the
     seam then writes `Statement<Formal>` with no table in between.
575. `Owner` is written above the subjects, because of a fault of
     `main`'s reader (a generic record naming an enum of its subject).
576. A tie names its statement by its piece, not by holding it: ties
     of formal and prose statements then sit in one list. Hence the
     true warning M105 on `tie`'s require of `statement`.
577. `Place { path: List<String> }` with `within`: the one thing a
     place can say alone. `concerned` reads ties at a place or within.
578. `Model` does not hold statements and ties yet: `opinion` needs
     only evidence, relations and base rates. A build would.
579. `stance(statement, word)` gives the base rate: 0 when owed, the
     giver's word when given. Nobody calls it yet.
580. No `Kind` of statement in the seam: claim, assume, ensure and a
     bought promise are the four corners of content and owner, said in
     a comment. `observe` treats owed and given alike.
581. Four claims in `model` build small models of evidence and
     relations and state the opinion that comes out.

## Increment 95: no owner on a statement (2026-11-20)

**Asked.** "let's remove owner from statement, let's try to have the
smallest model that make sense and is passing our tests. Later we can
move down things if we find that they are actually part of the core."
`Owner`, `stance`, the promise `owed_or_given` and their three claims
are gone, in the model, the seam, the code and the tests. A statement
is a piece and a content. Decisions 575, 579 and 580 no longer apply.

## Increment 96: the throwing toy (2026-11-20)

**Asked.** "let's use this toy exemple, and figure out how we describe
and compute opinions in different scenarios, with the extreme opinions
as a test of logic, and with middle as a test of what uncertainty does
to it". `docs/throwing.md` has the four scenarios; `tests/throwing.rs`
pins what the core gives today. No change to the calculus or to the
relation record. Seven of ten rows of the single throw read wrong:
nothing flows from a whole to its parts, a "no" is dropped when the
parts have a base rate, and no conflict is derived.

**Decisions taken alone.** 582. The pair a throw tests is a fourth
piece, `lands`, resting on the tie and the statement. 583. The wrong
rows are pinned as tests and not written as claims of the model: a
claim is a promise. 584. Two rules are proposed in the note, not
coded.

## Increment 97: the relation `All`, and the toy reads right (2026-11-20)

**Asked.** "Let's name this relation we have an exemple of, make the
computation right. I think relation semantic should be about the
trust operation we compute on them no ?" Two relations in the toy:
`about` (evidence to its piece: discount, then fuse), which is a field
and was already there, and the pair, renamed from `RestsOn` to `All`
after its operation. `docs/throwing.md` has the operators and the
tables; all four scenarios now read as expected.

**Decisions taken alone.**

585. The name is `All`, not `Pair`: the operation is a conjunction of
     any number of parts, and the pair is its case of two.
586. A yes flows down as the same belief, with the voice of the
     evidence, and is fused with what else is said of the part.
587. A no gives a part `d` times the product of the belief of the
     other parts, that belief taken without the whole in question and
     without any blame. Belief, not projected probability.
588. A whole reads what is said of it directly or its parts together,
     whichever has less uncertainty (it was: whichever projects
     higher, which dropped a no).
589. A conflict is derived from every evidence that disbelieves a
     whole, whether or not its parts are believed.
590. `record Piece { name: String }`: a piece was opaque, and the
     toy's claims need to name pieces. In the code it was a name
     already.
591. Not done: the same evidence reaching a part through two wholes
     is counted twice; flowing down more than one level is coded and
     not tested.

## Increment 98: a relation is said (2026-11-20)

**Asked.** "relation is structure, and we found the structure
conjunction, and discounting, or about ? Give me a name for both
relation types. Then a statement which a piece on content, can be
constructed as a belief about a relation taken as content". The names
are `All` and `about`. `Model.related` is now a list of
`Statement<Related>`: every relation has a piece, and what is computed
on it is discounted by the opinion of that piece. Scenario 5 of
`docs/throwing.md` is the toy: a latent `gravity` linked by `because`.

**Decisions taken alone.**

592. One list: every relation of a model is a statement. Structure is
     the case believed at 1, and it is written: a relation whose piece
     has no base rate relates nothing.
593. The discount is by `P` of the relation's statement, on the three
     directions of `All`: up, a yes down, a no down.
594. A relation believed under 1 is added to the suspects of a
     conflict it takes part in.
595. `about` stays a field of `Evidence` and is not a variant of
     `Relation`: an evidence already is a piece that says its relation.
596. The latent link is written with `All` ("because, and only
     because"). No new kind of relation.
597. Known cost: the belief of each relation is worked out again at
     every step of the walk. Fine for toys, to cache before real
     models.

## Increment 99: a no is shared (2026-11-20)

**Asked.** "if you trust your sense more, the logic would be to
distrust the model and tie more than your sense when ranking the
culprit of the gap"; "You don't have to stick to subjective logic
operators"; "we already proven we need the bias if we start the
evaluation at uncertain, so that we can express things like I trust my
sense more than my model or ties". A failed throw now puts disbelief
on the tie, the statement and the throw, shared by what each was
expected to be. Tables in `docs/throwing.md`.

**Decisions taken alone.**

598. The operator: `share(x) = (1 - p(x)) · product of p(others) /
     (1 - product of p(all))`, the chance that `x` alone is wrong given
     that not all are right. Not from subjective logic.
599. `p` is `P = b + a·u` read before any no is shared, so the base
     rates set the shares when nothing else is known.
600. All expected at 1: equal shares.
601. The evidence that says no is a suspect and takes its share: it is
     believed less afterwards, and the whole is disbelieved as far as
     it is still believed.
602. A relation believed at 1 takes no share; one believed under 1 is
     a suspect like the others.
603. The shares are not made to add up to 1: with two suspects
     expected wrong, nobody is blamed (oddity 2 of the note).
604. `share` is private code, stated by the promise `no_is_shared` and
     two claims; it is not a function of the model yet.
