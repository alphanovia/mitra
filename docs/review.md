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

## Increment 100: an opinion is counts (2026-11-20)

**Asked.** "let's try beta for opinion (r, s, a), with W a global
setting"; "let's try the beta representation, with bayesian network
operators"; then the latent toy reshaped ("gravity is latent, mass of
an object is the model tied to the object") and swept ("from one
object to a 100, with 100 measurement each", "for each object, you
can decide of a rate of success of the measurement"). Notes:
`docs/throwing.md`, `docs/sweep.md`.

**Decisions taken alone.**

605. `W` is `piece::weight()`, a function that returns 2. It becomes a
     setting when a layer holds settings.
606. `yes()` is one count of yes, `no()` one count of no.
607. One voice counts once by its strongest yes and its strongest no
     (`fuse` at 0); the average would halve a yes heard beside a small
     no from the same voice.
608. `all` matches mean and variance of the product, never with fewer
     counts than the base rate alone; one part surely right gives the
     other, one surely wrong gives no yes.
609. A whole has the base rate of its parts together; a base rate
     written for a whole is not read when it reads from its parts.
610. A whole reads from its parts unless what is said of it directly
     has more counts.
611. A shared no moves only the count of no of each suspect, to the
     Bayesian mean; the variance is not matched there.
612. Each failure is shared from what each suspect read as before any
     no, the yes about the same whole included.
613. `all` has no body in the model: its claims are tested, not
     computed by Mitra.
614. One walk remembers what it worked out (relations, what was read
     before, the failures). A hundred objects with a hundred
     measurements take about a second.
615. The goldens are recorded again at what counts give; none holds at
     0.9 (see the note).
616. In the sweep each measurement has its own voice, and a success
     rate is a fixed number of passes, not a draw.

**Then (12:41).** "I want the first observation to collapse the model
into certainty very fast." The weight is 0.2, as the coordinator
proposed: one sure yes on one half reads 0.92. Tables and goldens are
recorded again, with a column at 2 where it helps.

617. The W 2 columns are the numbers of the commit before; the weight
     is one function and is not swept by the code.

## Increment 101: two trials, completeness and a growing weight (2026-11-20)

**Asked.** "On a given tie, we have a piece of system, and the
statements we attached to it. So we have an opinion on each statement,
and on the completeness of the relations." "let's try this on our two
foundation exemples"; "try the W being scaled with the amount of
evidence in the model too". Note: `docs/trials.md`. Nothing is added
to the model: both are examples over the core as it is.

**Decisions taken alone.**

618. Completeness is written by hand in each trial model, one piece
     per tie; the core does not imply it. To imply it, a model must
     hold its ties, which is a change for you to see first.
619. In the one-throw trial the pair is all of [falls, complete]: the
     tie piece `applies` is left out, since the two say nearly the
     same.
620. The scale is tried both ways: evidence about the mass itself, and
     a prediction on the tie.
621. `with_weight` lets a trial work under another weight. It is in
     the code and not in the model, for trials only; a real setting
     would be a field of the model.
622. The schedule is 0.2 + 0.2 · ln(1 + evidence): it reaches the
     earlier weight of 2 at ten thousand pieces of evidence.
623. The fitted prior per kind is not tried: a piece has no kind.

**Then (12:50).** "in the build we would need something like an
influence optimisation pass to find the right W ?" Two passes tried in
`examples/weight.rs`, read in `docs/trials.md`.

624. Both passes search a grid of ten weights from 0.02 to 20.
625. The likelihood is the Beta-binomial one on the counts each piece
     has under that weight, over the pieces with a base rate strictly
     between 0 and 1.
626. The influence of a piece of evidence is how far the reading of
     the parts of its whole moves with and without it; epsilon is 0.01.

**Then (12:52).** "on a W fitting, can we compute a sort of bound with
that technique ?" Yes: section "Bounds" of `docs/trials.md`.

627. The interval of a piece is from the 5th to the 95th hundredth of
     its Beta, worked out in the example, not in the core.
628. The weights allowed are those within 2 of the best log-likelihood,
     over 40 weights from 0.01 to 50.
629. `w` is not added to `Opinion`: it changes the record and every
     claim on it, and is left for the user's word.

## Increment 102: reviews in the mix (2026-11-20)

**Asked.** "I'm not convinced by w per piece either... the author of
the model would need a know to say I trust this or statement per
experience, or I'm unsure. Which is what review evidence is for no ?
So how about we put review evidence in the mix". No `w` per piece.
`examples/review.rs`, read in `docs/trials.md` (was complete.md).

**Decisions taken alone.**

630. A review is plain evidence whose value carries a count of yes;
     nothing is added to the core.
631. Reviewers are believed at 0.9, "centuries" at 0.99 with 1000 yes,
     "I weighed it" with 1, 10 or 100 yes. These numbers are mine.
632. The Beta maths of the trials (interval, ln of gamma) is shared in
     `examples/common/`, still outside the core.

**Then (13:04).** "you can review completeness too... a test where the
reviewer is very good on the model, and unsure on the masses. Then we
try a bad reviewer, saying feather is complete, and overconfident on
everything." Section "A good reviewer and a bad one" of
`docs/trials.md`.

633. The good reviewer: 1000 yes on gravity, 1 yes on each mass and
     half a yes on each completeness (user 13:05: unsure of both),
     believed at 0.99. The bad one: 100 yes on everything, believed
     at 0.9. Each review is a piece of its own.
634. Found, not fixed: the evidence that says yes about a part is not
     a suspect of a failure, so a review is never doubted by what
     fails. To change it is a change of the failure rule, for the
     user to decide.

## Increment 103: a no that travels back to a review (2026-11-20)

**Asked.** "the review could be confident on stone and less so on a
leaf"; and the smallest rule that lets a no reach the evidence that
vouched for a part, shown on the bad reviewer, stopped for the user's
read.

**Done.** `examples/review.rs` sections 4 and 5; a trial rule
`vouched` in `src/model.rs`, off unless `model::with_vouching` asks;
`docs/trials.md`, "A good reviewer and a bad one". No test, no model
change: 69 tests as before.

**Decided alone.**

635. Completeness reviews by the good reviewer: 20 yes for the stone,
     5 for the ball, half a yes for the leaf and the feather. The
     stone fails 1 in 10, the leaf 4 in 10, the feather always.
636. The rule: evidence that said yes about a part of what failed gets
     a no, by Bayes on "the failure is more likely if this evidence is
     wrong", the part read with all of its count against none of it.
     Its yes count is untouched, as in `blamed`.
637. The rule takes the evidence as the only one of its voice about
     that part, and judges each review alone: no credit shared across
     one reviewer's reviews.
638. The rule is a thread-local switch in the code and is not in
     `model/model.mitra`: the model does not hold it until the user
     keeps it.
639. Found, not fixed: a discredited review of a large count still
     gives a large yes (100 yes at 0.17 is 17 yes), and the
     measurements stay blamed since failures are shared from the
     opinions before any no.

## Increment 104: two independent voices (2026-11-20)

**Asked.** "let's try to independent voice on the review then, one
cautious uncertain good reviewer, and one bad overconfident reviewer.
How is independence modelled in bayesian network?"

**Done.** `examples/review.rs` section 6; the trial rule `vouched`
now judges a piece once on all the evidence standing on it;
`docs/trials.md`, "Two independent voices". No test, no model change.

**Decided alone.**

640. A voice is one piece, `reliable: <voice>`, and each of its
     reviews is evidence whose `piece` is that one. No new relation:
     the review and the reliability are not told apart, so a voice is
     right or wrong as a whole.
641. `vouched` takes the piece, not one evidence: L(wrong) removes all
     its counts from all the suspects at once. For a piece behind one
     review nothing changes (section 5 reads the same).
642. Both voices start at 0.9: the model is not told which is good.
643. Found, not fixed: the rule only takes credit away, so a cautious
     voice alone is discredited by the feather (0.34) whatever else
     it got right; and the parts do not follow the voice's discredit,
     since failures are shared once, from the opinions before any no.

## Increment 105: what was kept goes into the core (2026-11-20)

**Asked.** "I think we should agree on beta représentation, the
completeness, and all the things we got good result so far"; then
"W = 0.2 is not that perfect right? Let's keep W a given setting to
the code model".

**Done.** `model/model.mitra`, `src/model.rs`, `src/model/{piece,
evidence,tie}.rs`; 74 tests (69 before). 30 promises (26), 57 claims
(46). `mitra conform` as before: 49 ties agree, the same 5 differ.

**Decided alone.**

644. The weight is a field of `Model`, and a parameter `w` of what
     reads an opinion (`projected`, `uncertainty`, `reaches`, `moved`,
     `all`). `fn weight()` and the trial switch `with_weight` are
     gone. `Model` has no default: `Model::empty(weight)` in Rust.
     Tests and examples name 0.2 themselves.
645. Shown: `belief`, `disbelief`, `uncertainty` as three functions,
     not a record; `interval(o, w)` is bodiless in the model, and its
     numerics (gamma, incomplete Beta, bisection; 80 lines) are in
     `src/model/piece.rs`.
646. Completeness: `Model.ties`; `tie::complete(t)` is a
     `Statement<Complete>` named `complete: <tie's name>`; it is a part
     of every whole whose relation names the tie's piece
     (`model::parts`). Its base rate is written like any other: with
     none it is 0, expected wrong, and so is the whole.
647. Not implied: the whole of a tie and its statement. The relation
     that makes `lands` all of [applies, falls] is still written.
648. The rule of increment 103 is always on (`no_reaches_what_vouched`)
     and its switch is gone. One recorded test moved: the scale that
     weighed the feather is doubted (0.90 to 0.68) and the feather's
     mass reads 0.62, was 0.68. No golden moved.
649. A review as evidence with a count needed no code: the promise
     `evidence::count_is_firmness` and two claims.
650. `docs/trials.md` keeps its tables as recorded, with a note.
651. Still a trial, not in the core: a voice as one piece behind its
     reviews (it needs nothing more from the core than 648).

## Increment 106: voices, iterated (2026-11-20)

**Asked.** "Ok, iterate on voice", with the design relayed from the
chat: reliability implied per voice; independent by default; a shared
source as a declared parent; credit from passes; a discredited voice
stops holding up the parts.

**Done.** `examples/voice.rs` and `docs/trials.md`, "Voices,
iterated". The core is not touched: 74 tests as before.

**Decided alone.**

652. The trial is a pass outside the core, over the public functions
     only: judge the voices, discount their word, call `opinion`. No
     switch in the core this time.
653. The word of a voice is discounted as a mix of the word and of
     nothing (the trust discount of subjective logic, in counts), not
     by scaling counts. Without this a discredited voice keeps holding
     up the parts. `moved` in the core still scales.
654. A voice is judged per whole, passes and failures in one
     likelihood, not per failure: repeated failures of one whole have
     one cause.
655. A source is weighed by its voices together; the word of a voice
     holds at (voice) times (source). One source per voice here.
656. The reliability of a voice is a number in the trial, not yet a
     `Statement` in the model, and its prior is given by hand (0.9).
     A voice with none is not doubted as a voice.
657. Found: the vouching rule kept in the core (648) and this pass do
     the same job in two ways. If voices are kept, 648 is the one to
     drop or to redo on this pattern.

## Increment 107: value as quantities, types and operations (2026-11-20)

**Asked.** "A value is a quantity with a type. Each type are assigned
operations to transform or convert quantities. An expression is a
nested chain of operation or expression that can be evaluated?";
then "let's do the small core and the standard table".

**Done.** `model/value.mitra` and `src/value.rs` rewritten;
`model/standard.mitra` and `src/standard.rs` new; the seam passes
what applies operations. 89 tests (74). 37 promises (30), 72 claims
(57). `mitra conform`: 70 ties agree, the same 5 differ.

**Decided alone.**

658. The core: `Type { name }`, `Quantity` (opaque), `Value { quantity,
     of }`, `Operation { name, takes, gives }`, `Expression = Given |
     Input(name, of) | Apply(op, args)`, `check(e)`, `eval(e, inputs,
     by)`. An input carries its type, so `check` needs no list of
     inputs.
659. What applies operations is a contract, `Applies`, given to
     `eval`. It is declared before the types it speaks of, so it names
     them as parameters: `Applies<Operation, Value, Quantity>`. Main's
     reader cannot tie a Rust trait, so the trait carries no tie.
660. `Quantity` is opaque in the model. In Rust it is a carrier of
     three shapes (a truth, a whole number, several quantities) that
     `src/value.rs` never reads; `tests/wall.rs` checks it. A generic
     carrier would be purer and spreads a type parameter to the seam.
661. `eval` checks the type of each value an operation is applied to,
     and gives the result the type the operation says: `typed` holds
     even on what was not checked.
662. `==` and `if` are one operation per type (`equal(of)`,
     `choose(of)`). `if` computes both branches: a branch with no
     value leaves the `if` with none. Before, the branch not taken
     was not computed.
663. A record is a `Shape`: its name is its type, `make(shape)` and
     `field(shape, place)` are its operations, and a table applies
     them only for the shapes it was given.
664. The seam reads `standard` for what a truth is.
665. Not built: resolving each operation once. Each `Apply` is matched
     by name and types when evaluated. Measured
     (`examples/bench.rs`): a million evaluations of a five-operation
     expression take 0.57 s, 0.33 s before. A type is a name held as
     text, copied per value; that is most of the difference.
666. The model cannot yet declare an operation with a body: a model's
     own operation is a Rust `Applies`. That is where the `operator`
     keyword would come in.

## Increment 108: voices in the core, the format of the model files (2026-11-20)

**Asked.** "vouching seems like a workaround. Which one is closer to
textbook bayesian network modeling?", then yes to moving the voice
pass into the core. On the format: a short header per file, a subject
of tests at the end, "What is Prose? ... Why not having a record
Claim?". On completeness: "better than random guess by default but
not very strong. 0.6 maybe or 0.7".

**Done.** The vouching rule is removed. `model::judged`, `settled`,
`evidence::reliable`, `evidence::held`; `Model.completeness`;
`Claim` for `Formal`, `Prose` removed; headers; `subject tests`.
97 tests (89). 41 promises, 78 claims. `mitra conform`: 77 ties
agree, 4 differ (the known ones).

**Decided alone.**

667. A voice is judged only if its reliability has a base rate in the
     model. With none it is not doubted and its word counts whole
     (`unjudged_is_not_doubted`). The other reading, "no base rate is
     0, so the voice says nothing", would silence every model that
     does not name its voices.
668. `opinion` judges the voices first. `judged(model)` gives how far
     each word holds; `settled(model, judged)` is the model with the
     words discounted and no voice left to doubt.
669. Until nothing moves: rounds repeat until no voice moves by more
     than one billionth, at most 200. It is expectation and
     maximisation in turn, in the manner of Dawid and Skene; it is not
     their model, which has a confusion table per annotator and labels
     that are independent items. Here a voice is one number and the
     items are the parts of wholes.
670. A source is declared with the relation the core already has: the
     reliability of a voice is all of [the source]. No new field.
671. `Model.completeness` is the base rate of a completeness nothing
     writes; a written one wins. Tests name 0.6. At 0.7 the toys read
     the same but for the completeness itself (0.55 and 0.65 after a
     failing throw).
672. `Prose` was a content nothing used: removed with its promise.
     `Formal` is renamed `Claim`: a claim is a `Statement<Claim>`.
     The subject keeps the name `claims`, since `claim` is a word of
     the language.
673. `subject tests` at the end of each file holds the claims, each
     function called by its subject's name. Main accepts several
     subjects of that name and still reports one warning. The claims
     of `evidence` stay in their subject: `evidence` is a word of the
     language, and `evidence::moved(..)` in an expression is refused
     ("expected an expression"; the raw form `r#evidence::` is not
     known either).
674. Not checked: whether main still counts a moved claim as serving
     its promise. The build raises no new warning, and the claims are
     now named `tests::<claim>` in the ties of the Rust tests.

## Increment 109: a tie lists its statements and implies its whole (2026-11-20)

**Asked.** "how about we expect the author to make a tie link to one
or several statement, and we put completeness by default? what about
voice, what under reliable() means? ... is the graph structure
changing the computation?"

**Done.** `Tie { piece, statements, place }`, `tie::whole`,
`model::written`; the toy claims of `model.mitra` use ties;
`examples/tied.rs`; `docs/trials.md`, "The toys written with ties".
101 tests (97). 42 promises. `mitra conform`: 80 ties agree, 4 differ.

**Decided alone.**

675. The whole of a tie is a piece named `whole: <tie's name>`
     (`tie::whole`). Evidence at the place is about it.
676. The relation a tie implies is structure: believed at 1, with no
     base rate to write. The doubt is carried by the tie, which is a
     part.
677. A relation written by hand is still read, beside the implied
     ones. The toy "it falls and makes no noise" keeps one: its two
     statements are observed apart, and the whole of a tie is all its
     statements together.
678. A tie has a base rate like any piece, 0 with none written. In the
     gravity toy it starts at one half, which adds a suspect the toy
     did not have.
679. The reliability of a voice is not a written relation. Every piece
     of evidence by a doubted voice has it as a parent, applied when
     the model is read (`settled`).
680. The older trial examples are not rewritten with ties.
