# The model layer on core2

`src/model.rs` is what you write about a system, as Rust data. It has no
syntax yet: the language is decided later. It checks that a model is well
formed and reads three things from it: how far each item is specified, the
pieces judgement reasons about, and which of them only a taking can hold.
It runs nothing, but each rule's proof, once (choice 5).

`src/expression.rs` did not change for it (`git diff eca4b67 --
src/expression.rs` is empty), nor did `src/judgement.rs`.
`src/simulation.rs` gained one helper, `tuple`.

## The fall

The test model is `tests/model.rs`. Here it is in an illustrative syntax, not
a proposal, with paths shortened inside a part:

```text
environment sun {
    fn light() -> w: Real;                            // opaque: how is not known
    ensure shines = "at least 200 W/m² at noon";      // only a taking holds it
}
subject stone {
    require light = sun::shines;                      // the sun's ensure gives it
    require air = "still air";                        // a hypothesis
    fn drop(h: Real) -> v: Real {                     // opaque: nature does it
        claim falls = within(v * v, 2 * 9.81 * h, 2);
        claim slow = !(h <= 20) || v < 21;
        rule slows = [drop::falls, speed::slow] => drop::slow;          // assumed
        rule lands = [drop::falls, speed::slow] => drop::slow
            by ponens(drop::slows, drop::falls, speed::slow);   // shown, using slows
    }
    fn speed(h: Real) -> v2: Real = 2 * 9.81 * h {    // total: it has a body
        claim slow = !(h <= 20) || v2 < 400;
    }
    ensure lands_slow = "lands slowly" from (speed::slow, drop::falls);
}
tie stone::drop at lab/tower/stone;
```

What the tests show on it:

- `check` finds nothing wrong. Each kind of problem is found when the model
  is changed to have it.
- Levels: `sun::light` is opaque, `stone::drop` partial, `stone::speed` total.
- Pieces: 12, each with the path it is about (below).
- Leaves: 8. They are `air`, `shines`, the claims `falls` and `speed::slow`,
  the assumed rule `slows`, what binds `light` to `shines`, what gives
  `lands_slow` from its list, and the tie. `light`, `lands_slow` and
  `drop::slow` follow from them. `lands` is shown by its proof.
- Judgement reads the pieces as they are. Taking the list of `lands_slow`
  (a reviewer) and its two claims (a machine, a timer) gives `lands_slow`
  true. The proof of `lands`, run on takings of what it reads, gives
  `drop::slow` true.
- Simulation runs the claims as they are. `speed::slow` holds for every
  height up to 20 m, by bounds, with the body read in place of `v2`.
  `drop::falls` holds on one measured fall, 5 m at 9.9 m/s.

## Data

```rust
struct Model { parts: Vec<Part>, ties: Vec<Tie> }
struct Part { name, kind: Kind /* Subject | Environment */, records, functions, requires, ensures }
struct Record { name, fields: Option<Vec<(String, Type)>> /* None: opaque */, claims, rules }
struct Function { name, takes: Vec<(String, Type)>, gives: (String, Type),
                  body: Option<Expression<simulation::Op>> /* None: opaque */, claims, rules }
struct Claim { name, holds: Expression<simulation::Op> }          // must be a truth
struct Rule { name, premises: Vec<Prop>, conclusion: Prop,
              proof: Option<Expression<judgement::Op>> /* None: assumed */ }
enum Prop { At(Path), Holds(Expression<simulation::Op>) }  // a claim, require or ensure, or written in place
struct Require { name, from: Source }    enum Source { Prose(String), Ensure(Path) }
struct Ensure { name, prose: String, from: Vec<Vec<Path>> }        // any one list, all of it
struct Tie { item: Path, at: Location }
struct Path { names: Vec<String> }                                 // stone::drop::falls
```

`check(model) -> Vec<Problem>`, `levels(model)`, `pieces(model)` and
`leaves(model)`. A `Problem` is `Twice`, `Unknown`, `Mistyped`, `Empty`,
`Circular` or `Misnamed`, each with the path of the declaration at fault.

## Checks

- Each name is a word (letters, digits and `_`, not first a digit), declared
  once in its place. Record names are unique in the whole model, because a
  record's type is named by it.
- Each type is a truth, a number, a tuple, or the type of a declared record.
  A record is made only with its declared fields, and an opaque one never.
- A body checks to the type its function gives, from what it takes.
- A claim checks to a truth. In a function it reads what the function takes
  and gives. In a record it reads the record as `self`.
- Parts meet only through requires. A rule's premises and conclusion name
  claims, requires or ensures of its part. A list of an ensure names claims,
  rules, requires or ensures of its part. A require's `from` names an ensure
  of another part. A tie names a part, record or function.
- An expression written in a rule checks to a truth, read as a claim of the
  rule's record or function.
- A proof shows its rule (choice 5).
- Nothing rests on itself. A conclusion rests on its rule's premises and on
  the rule, a rule on the rules its proof uses, a require on its ensure, and
  an ensure on what its lists name. Each declaration on a cycle, across
  parts too, is `Circular`.
- No list of an ensure's `from` is empty.

## Pieces

| In the model | The piece | About |
|---|---|---|
| claim, require, ensure at `p` | `p` | `p` |
| rule at `r` | `{premises} r {conclusion}` | `r` |
| an expression written in rule `r` | `r::0`, `r::1`, … or `r::then` | `r` |
| require `q` from ensure `e` | `{e} q {q}` | `q` |
| each list `l` of ensure `e` (a rule in it is its piece) | `{l} e {e}` | `e` |
| tie of `i` at `place` | `i at place` | `i` |

A piece is given once. A leaf is a piece nothing concludes. A rule concludes
its conclusion. A require from an ensure is concluded by it, and an ensure by
its lists. A rule is concluded by its proof when the proof shows it. A tie is
always a leaf. This holds in a model `check` accepts: on a cycle, nothing
would be a leaf.

## The choices, in plain words

**1. A model is data, and it does not run.** You write parts, records,
functions and propositions. Nothing is evaluated or judged by writing them.
*Example*: `drop` has no body, and its claim `falls` is still well formed.
*Gain*: the same model is checked, run by simulation and judged, each by its
own reading, and the code never bends to one of them.
*Give up*: a model says nothing until something runs or judges it.
*Elsewhere*: SysML and AADL models are data read by tools. Ada/SPARK
contracts are part of the code itself.

**2. Subject or environment.** A subject is in the system: what you build,
or buy and use. An environment is outside it, and gives the system flows
or takes them.
*Example*: the stone is a subject, and the sun, which gives light, an
environment.
*Gain*, later: diagnose can tell a fault in what you answer for (a bug to
fix) from one in what you assume of the world (a belief to revise).
*Today*: a label. No check and no reading uses it yet.
*Give up*: the line is yours to draw, and moving it changes the model.
*Elsewhere*: systems engineering draws the same "system boundary"; SysML
calls the outside "actors" and "external systems".

**3. How far something is specified is read, never declared.** A record or
function is opaque (names and types only), partial (claims or rules, but no
body or fields) or total (a body, or fields).
*Example*: `drop` is partial, and `speed`, with its body, total.
*Gain*: the level cannot be wrong: it is what the model says.
*Give up*: you cannot mark something "total, details elsewhere".
*Elsewhere*: Ada separates a package's specification from its body. Here
the level says which of the two you have.

**4. A claim is an expression that must be true.** It is written in
simulation, so it can run, and named, so judgement can reason about it.
*Example*: `falls = within(v * v, 2 * 9.81 * h, 2)`.
*Gain*: one claim is both a test (run it on a measurement) and a
proposition (judge it on takings).
*Give up*: a claim cannot say what simulation cannot write yet: no lists,
no time, no distributions.
*Elsewhere*: postconditions and invariants in Ada/SPARK, Dafny and Eiffel.

**5. A proof is checked by its type, and by one run.** A rule says that its
premises give its conclusion. Its proof is a judgement expression: it reads
opinions of the premises, and of the rules of its part, and gives an opinion
of the conclusion. Its type must be an opinion of the conclusion. But an
opinion can argue against its piece (modus tollens) or come from a conflict
(reductio). So the proof is also run once, with all it reads taken true, and
it must give the conclusion true. Then the rule is shown. Without a proof,
it is assumed.
*Example*: `lands` is shown by `ponens(slows, falls, speed::slow)`; `slows`
is assumed, since no rule of judgement does arithmetic. `reductio(falls)`
has the type of a proof of `lands`, but run on `falls` true it gives
`drop::slow` unknown: rejected.
*Gain*: a proof is checked by the same core as everything else, with
nothing new to trust.
*Give up*: an expression written as a conclusion cannot be shown. It is its
own piece, and no proof gives it. To show it, name it as a claim.
*Elsewhere*: the Curry–Howard correspondence (a proposition is a type, a
proof is a value of it), with one difference. In Lean, Coq and Agda, the
type of a proof says that the proposition holds, so the type is enough.
Here `Opinion(p)` says what is believed of `p`, for or against, hence the
run. An unproved rule is Lean's `sorry`, or an axiom.

**6. Requires and ensures: what a part needs, and what it promises.** A
require comes from words (a hypothesis) or from another part's ensure. An
ensure follows from any one of its lists, each held together. With no list,
only someone taking it holds it. Rules and lists name only their own part's
items, so parts meet only through requires.
*Example*: `stone::light` comes from `sun::shines`. A stone rule cannot name
`sun::shines`: it names `stone::light`.
*Gain*: each link between parts is a piece that can be blamed.
*Give up*: a link from an ensure to a require is a judgement that the
words match, and someone must take it. And a circle of requires and ensures
across parts, which assume–guarantee reasoning allows when an argument over
time backs it, is a problem here.
*Elsewhere*: assume–guarantee contracts (Benveniste and others, 2018). The
lists are the AND/OR refinement of goal models (KAOS).

**7. What is assumed is read, never declared.** There is no `assume`. A leaf
is a piece nothing in the model concludes: only a taking holds it.
*Example*: `air` (words only), `shines` (no list), `slows` (no proof).
*Gain*: nothing is assumed quietly: every leaf is listed, and each is where
evidence must come in.
*Give up*: a long list of leaves, even for things everyone believes.
*Elsewhere*: Lean's `#print axioms` lists what a proof rests on.

**8. A tie says where an item is in the real system.** One belief: that it
is what is there, and that the model says all that matters of it there.
*Example*: `stone::drop` is the drop in the tower, `lab/tower/stone`.
*Gain*: a tie can be wrong, so it can be blamed, like any piece.
*Give up*: a tie names one place; a part spread over several needs several.
*Elsewhere*: traceability links in requirements tools, and the
`// mitra: implements` comments that main reads in Rust code.

**9. A rule is used by its name, not as a premise.** A rule's premises and
conclusion are claims, requires, ensures or expressions written in place:
propositions about the system. A proof uses another rule by its path.
*Example*: the proof of `lands` uses `slows`; the premises of `lands` are
`falls` and `speed::slow`.
*Gain*: a rule's piece holds only its own premises and conclusion. Before,
a rule named as a premise was copied whole into the piece, with the rules
it named in turn: 36 rules gave one piece of 655,358 parts, and a chain of
580 rules overflowed the stack. Now 5,000 rules are checked and read in
about a second, in a debug build.
*Give up*: a premise cannot be a rule ("if `slows` holds, then …"). A list
of an ensure still can name one.
*Elsewhere*: Lean and Coq. A theorem states something about objects, and
its proof uses other theorems by name.

## Beyond what was named (going ahead; say no to undo)

1. **Paths are absolute**, from the part: `stone::drop::falls`, not
   `drop::falls` inside the stone. The language can resolve short ones later.
2. **A record's claims read it as `self`**, Rust's form, as in
   `0.0 <= self.h`.
3. **Ensures can be named in a rule or a list**, beside claims and requires,
   and rules in a list.
4. **An expression written in a rule is its own piece**, named by the rule
   and its place. Two rules never share one. To chain, name it as a claim.
5. **A proof reads its premises by path**, or by place when written in
   place, and the rules of its part by path.
6. **The subject of a rule's piece is the rule.** For a require from an
   ensure it is the require, and for an ensure's list the ensure.
7. **An opaque record's type has no field to read**, and no value of it
   is made.
8. **A tie names a part, record or function**, and its piece is
   `item at place`.
9. **Tuples are records with no name**, their fields `0`, `1`, … in order:
   `simulation::tuple(types)`. `(a, b)` is `Make` and `t.0` is `Field("0")`.
   No new operation.
10. **Parts meet only through requires** (choice 6).
11. **Names are words**, so that a path reads one way. `Misnamed` is new.
12. **A cycle is a problem anywhere**, across parts too (choice 6).

## Contracts: not built

A contract is a named bundle of claims, rules, requires and ensures that
several items promise (main's `contract C` and `impl C for T`). Its claims
must be read in each item that promises it, with that item's names and
types. There are two ways, and the choice is yours:

- **Checked in each item** (like C++ templates). `Implements { contract,
  item }` in the model, and each claim of the contract checked again in the
  item's scope. About 40 lines, no change to simulation. A contract is
  only checked where it is used.
- **Type parameters** (like Rust traits). The contract is checked once,
  over types it names but does not fix. This needs type variables in the
  simulation standard and in its `gives`. It is bigger, and it changes
  simulation, not expression.

Not in this round: syntax, nesting of parts, contracts, evidence and
measurements, a reading of circular assume–guarantee (for diagnose).

## Red team (round 4)

A fresh agent attacked the first version (326388c). Each finding has a test
in `tests/model.rs`:

| Found | Now | Test |
|---|---|---|
| A proof by reductio or tollens type-checked, and its rule counted as shown | a proof is also run once (choice 5) | `a_proof_argues_for_its_conclusion` |
| Cycles not through a rule's own path passed, and vanished from the leaves | every cycle of what rests on what is `Circular` | `nothing_rests_on_itself` |
| A name with `::` or ` at ` made two pieces one | names are words | `names_are_words` |
| Pieces grew exponentially; 580 chained rules overflowed the stack | rules used by name (choice 9); cycles found without the call stack | `many_rules_stay_flat` |
| A claim could make a record no one declares | records made as declared | `records_are_made_as_declared` |
| "Parts connect only through requires" was false | now checked | `parts_meet_only_through_requires` |
| A tie to nothing gave a piece; a list given twice gave two; leaves trusted a rejected proof | fixed | `pieces_are_given_once_of_what_is_there` |
| `kind` is never read | said so (choice 2) | |
| `self.h >= 0` cannot be written; "Rust's" comments; GSN has no "or" | fixed above | |

Fifteen bugs seeded in the fixes were all caught by these tests.
