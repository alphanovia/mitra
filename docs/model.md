# The model layer on core2

`src/model.rs` is what you write about a system, as Rust data. It has no
syntax yet: the language is decided later. It checks that a model is well
formed and reads three things from it: how far each item is specified,
the propositions judgement reasons about, and which of them only a taking
can hold. It runs nothing, but each rule's proof, once (choice 5).

`src/expression.rs` did not change for it (`git diff eca4b67 --
src/expression.rs` is empty). `src/simulation.rs` gained one helper,
`tuple`, then calls by path and type parameters (`docs/core2.md`, choices
12 and 13). `src/judgement.rs` renamed `Piece` to `Prop`, and now knows
rules by name (choice 9).

## The fall

The test model is `tests/model.rs`. Here it is in an illustrative syntax, not
a proposal, with paths shortened inside a part. A rule is written
`name: premises |- conclusion`, the turnstile you are trying:

```text
environment sun {
    fn light() -> w: Real;                            // opaque: how is not known
    ensure shines = "at least 200 W/m² at noon";      // only a taking holds it
}
subject stone {
    require light = sun::shines;                      // the sun's ensure gives it
    require air = "still air";                        // a hypothesis
    fn drop(h: Real) -> v: Real {                     // opaque: nature does it
        claim falls: within(v * v, 2 * 9.81 * h, 2);
        claim slow: !(h <= 20) || v < 21;
        rule slows: drop::falls, speed::slow |- drop::slow;                // assumed
        rule lands: drop::slows, drop::falls, speed::slow |- drop::slow    // a rule as premise
            by ponens(drop::slows, drop::falls, speed::slow);              // shown: the checker takes it
    }
    fn speed(h: Real) -> v2: Real = 2 * 9.81 * h {    // total: it has a body
        claim slow: !(h <= 20) || v2 < 400;
        examples { 5.0 -> 98.1 }                      // what it gives at 5 m
    }
    ensure lands_slow = "lands slowly" from (speed::slow, drop::falls);
}
tie stone::drop at lab/tower/stone;
```

What the tests show on it:

- `check` finds nothing wrong. Each kind of problem is found when the model
  is changed to have it.
- Levels: `sun::light` is opaque, `stone::drop` partial, `stone::speed` total.
- Propositions: 12, each with the path it is about (below).
- Leaves: 9. They are `air`, `shines`, the claims `falls` and `speed::slow`,
  the assumed rule `slows`, the rule `lands`, shown by its proof, which the
  checker takes, what binds `light` to `shines`, what gives `lands_slow`
  from its list, and the tie. `light`, `lands_slow` and `drop::slow`
  follow from them.
- Judgement reads the propositions as they are, and knows each rule by its
  path. Taking the list of `lands_slow` (a reviewer) and its two claims (a
  machine, a timer) gives `lands_slow` true. The proof of `lands`, run on
  takings of what it reads, gives `drop::slow` true. A run that sees the
  stone land fast blames `slows` itself, by modus tollens.
- Simulation runs the claims as they are. `speed::slow` holds for every
  height up to 20 m, by bounds, with the body read in place of `v2`.
  `drop::falls` holds on one measured fall, 5 m at 9.9 m/s.

## Data

```rust
struct Model { parts: Vec<Part>, contracts: Vec<Contract>, examples: Vec<ModelExample>, ties: Vec<Tie> }
struct Part { name, kind: Kind /* Subject | Environment */, uses: Vec<Use>, records, functions,
              implements: Vec<Implements>, requires, ensures }
enum Use { Names { part, names: Vec<String> } /* use sun::{light, Ray}; */, Glob { part } /* use sun::*; */ }
struct Contract { name, params: Vec<(String, Bound)> /* T, T: Number */, functions: Vec<Signature>, claims: Vec<Law> }
struct Signature { name, takes: Vec<(String, Type)>, gives: (String, Type) }
struct Law { name, over: Vec<(String, Type)>, holds: Expression<simulation::Op> }   // true at every value of `over`
struct Implements { contract, types: Vec<Type>, functions: Vec<String> }            // impl Monotone<Real> for speed;
struct ModelExample { inputs: Vec<(Path, Value)>, outputs: Vec<(Path, Value)> }    // stone::speed::h = 5.0 -> stone::speed::v2 = 98.1
struct Record { name, fields: Option<Vec<(String, Type)>> /* None: opaque */, claims, rules }
struct Function { name, takes: Vec<(String, Type)>, gives: (String, Type),
                  body: Option<Expression<simulation::Op>> /* None: opaque */,
                  examples: Vec<Example>, claims, rules }
struct Example { takes: Vec<simulation::Value>, gives: simulation::Value }   // (6, 2) -> 3
struct Claim { name, holds: Expression<simulation::Op> }          // must be a truth
struct Rule { name, premises: Vec<Term>, conclusion: Term,
              proof: Option<Expression<judgement::Op>> /* None: assumed */ }
enum Term { At(Path), Holds(Expression<simulation::Op>), Prop(Prop) }  // by path, an expression, or correct/all/any in place
struct Require { name, from: Source }    enum Source { Prose(String), Ensure(Path) }
struct Ensure { name, prose: String, from: Vec<Vec<Path>> }        // any one list, all of it
struct Tie { item: Path, at: Location }
struct Path { names: Vec<String> }                                 // stone::drop::falls
```

`levels(model)`, `props(model)`, `judgement(model)` and `leaves(model)`
read the model. `calls(model)` gives its functions as simulation calls
them, and `contract_claims(model)` each contract's claim read of the
functions that fit it. `check(model) -> Vec<Error>`, in `src/check.rs`, says
where it is not well formed: each error has a code of the catalogue, the
path of the declaration to fix, and a message (below).

A `Term` is a proposition as a rule writes it: a path, or an expression.
`props(model)` gives each as judgement reads it, a `Prop`: `Named`,
`Not` or `Implies`. A rule is `Named` by its path, and `judgement(model)`
knows what it says: its premises give its conclusion.

## Checks

`check` gives every error it finds, not the first. Each check has its code,
and each code its page in `docs/reference`, with an example and the fix.

| Code | The check |
|---|---|
| M001 | Each name is a word: letters, digits and `_`, not first a digit. |
| M002 | Each name is declared once in its place. Record names are unique in the whole model, because a record's type is named by it. |
| M003 | Each type is a truth, a number, a tuple, or the type of a declared record. |
| M004 | A record is made only with its declared fields, and an opaque one never: in a body, a claim or an example. |
| M005 | A body checks to the type its function gives, from what it takes. |
| M006 | A claim checks to a truth. In a function it reads what the function takes and gives; in a record, the record as `self`. An expression written in a rule is read as a claim of its record or function. |
| M007 | A rule's premises and conclusion, and a list of an ensure, name claims, rules, requires or ensures. |
| M008 | … of their own part: parts meet only through requires. |
| M009 | A require's `from` names an ensure of another part. |
| M010 | A tie names a part, record or function. |
| M011 | No list of an ensure's `from` is empty. |
| M012 | A proof shows its rule (choice 5). |
| M013 | Nothing rests on itself. A conclusion rests on its rule's premises and on the rule, a require on its ensure, and an ensure on what its lists name. Each declaration on a cycle, across parts too, is reported. |
| M014 | No rule is named through itself: a term of it names it, or names a rule a term of which does, and so on (choice 9). Found even when a term of the rule names nothing. |
| M015 | An example takes one value for each name its function takes, in order, each of its type, and gives one of the type the function gives. |
| M016 | A use names another part, and functions or records of it. |
| M017 | A call names a function of the model; a contract's claim, one of the contract's. A body or claim with such a call is not also checked for its type. |
| M018 | Another part's function is called, and its record named, only through a use. |
| M019 | No function calls itself through its body, however far. Each on a cycle is reported. |
| M020 | An impl names a contract, gives a type for each parameter within its bound, and one function of its part for each of the contract's, of its types. |
| M021 | An example of the model names what functions take or give: inputs what they take, or what one with no body gives; outputs what they give; each once. |
| M022 | Each value of an example of the model is of its type, and every argument of a function it reads the output of is given. |
| M023 | A proposition written in a rule is `correct(f, point)` of a function of its part, or an all, an any or a `!` of those, with points and sets that fit what `f` takes. |
| M024 | Two examples of a function at one point give the same value. |
| M025 | Every range type holds a number: `0..=9`, not `9..=0`. |
| M026 | Every example, of a function or of the model, lies in the range types of what it names. |

## Errors and the catalogue

`src/catalogue.rs` is one list of codes for every layer. A code is a suit,
the layer that gives it, and a number: `M005`. The suits are `E` for the
expression core (none yet), `M` for the model as written (`check`), `A`
for the model against itself (agree), `V` for evidence, `X` for
explaining the system, and `B` for blame. A code is never taken out or
given to something else: one no longer given is marked retired, and stays
listed with its page. A test holds the list, the numbering and the pages
together.

An `Error { code, at, message }` says the model must be fixed before
anything reads it. A `Finding`, the same shape, will say what a reading
found in a well-formed model: a verdict, not a fault. Both are written as
rustc writes its errors:

```text
error[M002]: `falls` is declared twice
 --> stone::drop::falls
```

## Propositions, as judgement reads them

| In the model | The proposition | About |
|---|---|---|
| claim, require, ensure at `p` | `p` | `p` |
| rule at `r` | `r`, which judgement knows: `premises |- conclusion` | `r` |
| an expression written in rule `r` | `r::0`, `r::1`, … or `r::then` | `r` |
| require `q` from ensure `e` | `{e} q {q}` | `q` |
| each list `l` of ensure `e` | `{l} e {e}` | `e` |
| tie of `i` at `place` | `i at place` | `i` |
| `correct(f, x)`, or an all or an any of them, written in a rule `r` | itself, whichever rule writes it | `r` |
| claim `c` of contract `C`, for an impl of `C` for `f` | `f::C::c` | `f` |

A proposition is given once. A leaf is a proposition nothing concludes. A
rule judgement knows concludes its conclusion. A require from an ensure is concluded by it,
and an ensure by its lists. A rule is concluded by its proof when the proof
shows it. A tie is always a leaf. This holds in a model `check` accepts: on
a cycle, nothing would be a leaf.

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
*Today*: explain reads it for one hint only: what a subject's opaque function was seen to give may be copied as examples, not what the environment gave.
*Give up*: the line is yours to draw, and moving it changes the model.
*Elsewhere*: systems engineering draws the same "system boundary"; SysML
calls the outside "actors" and "external systems".

**3. How far something is specified is read, never declared.** A record or
function is opaque (names and types only), partial (claims, rules or
examples, but no body or fields) or total (a body, or fields).
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
opinions of the premises, and gives an opinion of the conclusion. Its type must be an opinion of the conclusion. But an
opinion can argue against its proposition (modus tollens) or come from a
conflict (reductio). So the proof is also run once, with all it reads taken
true, and it must give the conclusion true. Then the rule is shown, and the
checker takes it true: a taking like any other, by the voice
`machine:mitra-check`. Without a proof, it is assumed, and someone else
must take it. Either way the rule is a leaf (your choice: Mitra will read
other checkers, and a proof stays a suspect, the firmest one).
*Example*: `lands` is shown by `ponens(slows, falls, speed::slow)`; `slows`
is assumed, since no rule of judgement does arithmetic. `reductio(falls)`
has the type of a proof of `lands`, but run on `falls` true it gives
`drop::slow` unknown: rejected.
*Gain*: a proof is checked by the same core as everything else, with
nothing new to trust.
*Give up*: an expression written as a conclusion cannot be shown. It is its
own proposition, and no proof gives it. To show it, name it as a claim.
*Elsewhere*: the Curry–Howard correspondence (a proposition is a type, a
proof is a value of it), with one difference. In Lean, Coq and Agda, the
type of a proof says that the proposition holds, so the type is enough.
Here `Opinion(p)` says what is believed of `p`, for or against, hence the
run. An unproved rule is Lean's `sorry`, or an axiom. Lean trusts its
kernel; here the checker is a voice, so Lean or Verus can take a rule the
same way, and the numbers layer weighs each voice.

**6. Requires and ensures: what a part needs, and what it promises.** A
require comes from words (a hypothesis) or from another part's ensure. An
ensure follows from any one of its lists, each held together. With no list,
only someone taking it holds it. Rules and lists name only their own part's
items, so propositions of parts meet only through requires. Data meets
through `use` (choice 10).
*Example*: `stone::light` comes from `sun::shines`. A stone rule cannot name
`sun::shines`: it names `stone::light`.
*Gain*: each link between parts is a proposition that can be blamed.
*Give up*: a link from an ensure to a require is a judgement that the
words match, and someone must take it. And a circle of requires and ensures
across parts, which assume–guarantee reasoning allows when an argument over
time backs it, is a problem here.
*Elsewhere*: assume–guarantee contracts (Benveniste and others, 2018). The
lists are the AND/OR refinement of goal models (KAOS).

**7. What is assumed is read, never declared.** There is no `assume`. A leaf
is a proposition nothing in the model concludes: only a taking holds it.
*Example*: `air` (words only), `shines` (no list), `slows` (no proof),
`lands` (a proof: the checker takes it).
*Gain*: nothing is assumed quietly: every leaf is listed, and each is where
evidence must come in.
*Give up*: a long list of leaves, even for things everyone believes.
*Elsewhere*: Lean's `#print axioms` lists what a proof rests on.

**8. A tie says where an item is in the real system.** One belief: that it
is what is there, and that the model says all that matters of it there.
*Example*: `stone::drop` is the drop in the tower, `lab/tower/stone`.
*Gain*: a tie can be wrong, so it can be blamed, like any proposition.
*Give up*: a tie names one place; a part spread over several needs several.
*Elsewhere*: traceability links in requirements tools, and the
`// mitra: implements` comments that main reads in Rust code.

**9. A rule is a premise by its name, and judgement holds what it says,
once.** A rule's premises and conclusion are claims, rules, requires,
ensures or expressions written in place. A rule is the proposition named by
its path. Judgement holds its definition, `Definition { pre, post }`: its
premises give its conclusion. Ponens and tollens on `slows` look it up.
*Example*: `lands: slows, falls, speed::slow |- drop::slow`, shown by
`ponens(slows, falls, speed::slow)`. When a run sees the stone land fast,
`tollens` on `slows` blames `slows` itself.
*Gain*: rules use rules, and each is written once. In the first version, a
rule named as a premise was copied whole, with the rules it named in turn:
36 rules gave one proposition of 655,358 parts, and a chain of 580 rules
overflowed the stack. Now 5,000 chained rules, and 128 that each take the
two before (over 2⁶⁴ parts if written out), are checked and read in 1.5 s in a
debug build. The dependence stays visible: a proof reads only its
premises, so a rule it applies is one of them, or given by them (`law:
drop::falls |- drop::slows`), and blame reaches it.
*Give up*: no rule is named through itself, `r: r |- q`. J1 reads a rule
one way, as an implication that can be false: there `r` says `r → (r → q)`,
lawful but of no use, since only holding `r` lets you use it. Refused all
the same, so that a rule can also be read as exactly what it defines, true
when its premises give its conclusion. Read so, `r` with `q` false means
nothing (it is true exactly when it is false), and `a: b |- q` with
`b: a |- q` means two things (`a` or `b`). And an opinion does not say
under which definitions it was made: read it only with the judgement that
made it, and make it again after a rule changes.
*Elsewhere*: definitions in any language. Lean's `def` is unfolded by the
kernel only when it must. An ML `let` cannot name itself; `let rec` can, and
needs a meaning of its own, which judgement does not have.

## Beyond what was named (going ahead; say no to undo)

1. **Paths are absolute**, from the part: `stone::drop::falls`, not
   `drop::falls` inside the stone. The language can resolve short ones later.
2. **A record's claims read it as `self`**, Rust's form, as in
   `0.0 <= self.h`.
3. **Ensures can be named in a rule or a list**, beside claims and requires,
   and rules in a list.
4. **An expression written in a rule is its own proposition**, named by the rule
   and its place. Two rules never share one. To chain, name it as a claim.
5. **A proof reads its premises by path**, or by place when written in
   place, and nothing else.
6. **Only rules are known by name.** A require's link to an ensure and each
   list of an ensure stay implications written in place, `{e} q {q}` and
   `{l} e {e}`, the require or the ensure as subject: nothing names them, so
   nothing copies them.
7. **An opaque record's type has no field to read**, and no value of it
   is made.
8. **A tie names a part, record or function**, and its proposition is
   `item at place`.
9. **Tuples are records with no name**, their fields `0`, `1`, … in order:
   `simulation::tuple(types)`. `(a, b)` is `Make` and `t.0` is `Field("0")`.
   No new operation.
10. **Parts meet only through requires** (choice 6).
11. **Names are words**, so that a path reads one way (M001).
12. **A cycle is a problem anywhere**, across parts too (choice 6).
13. **A rule's conclusion may be a rule**, as in the first version: a rule
    that gives a rule.
14. **A proof that applies a rule judgement does not know is not
    checked.** That rule has a term that names nothing, or is named
    through itself: it is reported where it is, once, and the proof is
    checked when it is fixed.
15. **An error is at the narrowest declaration**: a type at the field or at
    what the function takes or gives (`stone::Drop::h`), a term at its rule,
    a list at its ensure.
16. **A body or a claim is checked for its type only when the types it
    reads are known**, so a missing record gives M003 alone, not M005 or
    M006 too.
17. **Examples are rows of values**, `Example { takes, gives }`, beside the
    body (your option B). A row gives no proposition yet: with binders it
    is the answer `Correct(f, x)` is read against at its point. A function
    with examples and no body is partial.
18. **The checker's voice is `machine:mitra-check`**, in the takings its
    proofs give. Nothing gives them yet: the evidence layer will.
19. **A use is `Use::Names` or `Use::Glob`**, and a part needs none for its
    own functions and records. A record of another part in a type, or made
    in an expression, needs one, as a call does.
20. **A contract's claim is a `Law`**, a claim with its own arguments
    (`over`), true at every value of them. Say no to name it otherwise.
21. **An impl is declared in the first function it names**, beside its
    claims: `stone::speed::Monotone`, so a claim of `speed` named
    `Monotone` is M002. Its claims are under it:
    `stone::speed::Monotone::rises`, one name deeper than a function's.
22. **Contracts are named like parts** (`Monotone::f` is a path), so a
    contract and a part do not share a name (M002).
23. **An error in an example of the model is at `examples::i`**, its
    place in the list: the example has no name.
24. **A `correct` written in a rule is the proposition itself**, the same
    for every rule that writes it, unlike an expression written in place.
    So a cycle through it is found (M013).
25. **A call of no function is reported alone** (M017): the body or claim
    is not also checked for its type.

## Uses, contracts and examples of the model

**10. A part reads another's functions and records through a `use`.** As
Rust's: `use sun::{light, Ray};` names some, `use sun::*;` all. A call is
written by its path, `sun::light()`, and simulation runs it (`docs/core2.md`,
choice 12): the body, or a row of its examples, or no value.
*Example*: `stone::speed` is `2 · g · h · sun::light()`. With `use
sun::light;` it is well formed; without, M018 says which use is missing.
*Gain*: the model computes across parts, so an example or a test can give
A's inputs and state B's outputs. Each link is written once, at the top of
the part, and can be blamed: a claim that reads `light()` rests on the sun
too (the explain layer).
*Give up*: a use is one more line per link. Records are named by their
name alone (they are unique in the model), so `use sun::Ray;` only says
that the stone may name it.
*Elsewhere*: Rust's `use`, Python's `import`, Java's `import`. Ada's
`with` clauses make the same links explicit for the same reason: a
package's dependencies are read from its first lines.

**11. A contract is a property written once, for every function that
fits it.** As a Rust trait with a bound: `contract Monotone<T: Number> {
fn f(x: T) -> y: T; claim rises(a: T, b: T): !(a <= b) || f(a) <= f(b);
}`. The contract is checked once, with `T` named but not fixed: its types,
and that each claim is a truth calling only the contract's functions
(`Monotone::f`). `impl Monotone<Real> for speed;` only checks that `speed`
fits: `Real` is a number, and `speed` takes and gives a `Real`. The claim
then becomes one about `speed`, `stone::speed::Monotone::rises`, read with
`speed` for `f` and `Real` for `T`, which agree checks like any claim.
Explain does not read it yet: measurements of `speed` leave it
unsupported, with no finding (a finding for you). An impl names one
function at least (M020, flagged).
*Example*: the stone's `speed` rises with the height; so does any other
function that fits `Monotone`, with no claim written again.
*Gain*: one property for many functions; a mistake in it is reported once,
at the contract (M006 at `Monotone::rises`), not at each use.
*Give up*: a contract claim has its own arguments (`a`, `b`), unlike a
function's claim, which reads what the function takes and gives. Nothing is
computed on `T` itself: no value is of it. A contract has functions and
claims only; rules, requires and ensures in a contract are not in this
round.
*Elsewhere*: Rust traits and Haskell type classes are checked once, as
here; C++ templates are checked at each use. Your first option, checking
each claim again in each item, was the C++ way.

**12. Examples of the model as a whole.** A row says what the model gives
across parts, as an end-to-end test would: `(sun::light::w = 850.0,
stone::speed::h = 5.0) -> stone::speed::v2 = 98.1`. Inputs are what
functions take, or what a function with no body gives (the world's value
there); outputs are what functions give. Through calls, the model computes
the outputs, and agree compares them, as it compares a function's rows.
*Gain*: the spec of an end-to-end test, written with the model's names; a
measurement through ties has the same shape, so the two can be compared.
*Give up*: the model must compute each output: every argument of its
function is given. An output the model can only reach through a function
it does not call is not computed: the example says so (M022).
*Elsewhere*: Gherkin's examples tables (Given/When/Then), and golden
tests: inputs and expected outputs of a whole program.

## Red team (round 4)

A fresh agent attacked the first version (326388c). Each finding has a test
in `tests/model.rs`:

| Found | Now | Test |
|---|---|---|
| A proof by reductio or tollens type-checked, and its rule counted as shown | a proof is also run once (choice 5) | `a_proof_argues_for_its_conclusion` |
| Cycles not through a rule's own path passed, and vanished from the leaves | every cycle of what rests on what is reported (M013) | `nothing_rests_on_itself` |
| A name with `::` or ` at ` made two propositions one | names are words | `names_are_words` |
| Propositions grew exponentially; 580 chained rules overflowed the stack | rules are premises by name, held once by judgement (choice 9); cycles found without the call stack | `rules_sharing_rules_stay_small` |
| A claim could make a record no one declares | records made as declared | `records_are_made_as_declared` |
| "Parts connect only through requires" was false | now checked | `parts_meet_only_through_requires` |
| A tie to nothing gave a proposition; a list given twice gave two; leaves trusted a rejected proof | fixed | `props_are_given_once_of_what_is_there` |
| `kind` is never read | said so (choice 2) | |
| `self.h >= 0` cannot be written; "Rust's" comments; GSN has no "or" | fixed above | |

Fifteen bugs seeded in the fixes were all caught by these tests.

## Red team (round 5): rules known by name

A fresh agent attacked choice 9 (7e9624a), the catalogue aside:

| Found | Now | Test |
|---|---|---|
| A cycle of names through a rule with a term that names nothing was not reported | cycles are found on every rule, its broken terms left out | `a_cycle_is_found_through_a_broken_rule` |
| A proof applying a rule given by a rule (`law` gives `slows`) got M012 when `slows` was named through itself: a cascade | a proof is not checked when it applies a rule judgement does not know (item 14) | `a_proof_on_a_rule_judgement_does_not_know_is_not_checked` |
| An opinion does not say under which definitions it was made | said so (choice 9, `Opinion`) | |
| Choice 9 gave a reason that J1 does not need | rewritten | |
| `a: b, p |- q`, proved, and `c: a |- p`: a cycle through the proof? | no: the proof shows the whole rule, its premises as hypotheses, so `a` rests on none of them | |
| A proved rule that another rule uses is held by no leaf, and no rule of judgement gives it | your choice: a proved rule is a leaf, which the checker takes (choice 5) | `the_leaves_hold_everything` |

Attacks that held, kept as tests: blame reaches the rule that gives a rule
(`a_rule_given_by_a_rule`); `with` refuses exactly the names on a cycle, on
3,000 random tables against brute force
(`with_refuses_exactly_the_names_on_a_cycle`); a negation or a name
judgement does not know is no rule (`a_rule_known_by_its_name`). Eleven
bugs seeded in the fixes were all caught by these tests.
