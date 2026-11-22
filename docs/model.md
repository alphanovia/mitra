# The model layer on core2

`src/model.rs` is what you write about a system, as Rust data. It has no
syntax yet: the language is decided later. It runs nothing and judges
nothing. It checks that a model is well formed and reads three things from
it: how far each item is specified, the pieces judgement reasons about, and
which of them only a taking can hold.

`src/expression.rs` did not change for it (`git diff eca4b67 --
src/expression.rs` is empty). `src/simulation.rs` gained one helper,
`tuple`.

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
        rule lands = [drop::slows, drop::falls, speed::slow] => drop::slow
            by ponens(drop::slows, drop::falls, speed::slow);          // shown
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
  true. The proof of `lands`, run on takings of its premises, gives
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
enum Prop { At(Path), Holds(Expression<simulation::Op>) }          // a proposition
struct Require { name, from: Source }    enum Source { Prose(String), Ensure(Path) }
struct Ensure { name, prose: String, from: Vec<Vec<Path>> }        // any one list, all of it
struct Tie { item: Path, at: Location }
struct Path { names: Vec<String> }                                 // stone::drop::falls
```

`check(model) -> Vec<Problem>`, `levels(model)`, `pieces(model)` and
`leaves(model)`. A `Problem` is `Twice`, `Unknown`, `Mistyped`, `Empty` or
`Circular`, each with the path of the declaration at fault.

## Checks

- Each name is declared once in its place. Record names are unique in the
  whole model, because a record's type is named by it.
- Each type is a truth, a number, a tuple, or the type of a declared record.
- A body checks to the type its function gives, from what it takes.
- A claim checks to a truth. In a function it reads what the function takes
  and gives. In a record it reads the record as `self`.
- A path in a rule or in an ensure's list names a claim, rule, require or
  ensure. A require's `from` names an ensure. A tie names a part, record or
  function.
- An expression written in a rule checks to a truth, read as a claim of the
  rule's record or function.
- A proof checks to an opinion of its rule's conclusion. It reads the
  rule's premises as opinions: one at a path by its path, and one written
  in place by its place, `0`, `1`, ….
- No rule is among its own premises or conclusion, through other rules.
- No list of an ensure's `from` is empty.

## Pieces

| In the model | The piece | About |
|---|---|---|
| claim, require, ensure at `p` | `p` | `p` |
| rule at `r` | `{premises} r {conclusion}` | `r` |
| an expression written in rule `r` | `r::0`, `r::1`, … or `r::then` | `r` |
| require `q` from ensure `e` | `{e} q {q}` | `q` |
| each list `l` of ensure `e` | `{l} e {e}` | `e` |
| tie of `i` at `place` | `i at place` | `i` |

A leaf is a piece nothing concludes. A rule concludes its conclusion. A
require from an ensure is concluded by it, and an ensure by its lists. A
rule with a proof is concluded by its proof. A tie is always a leaf.

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
*Gain*: what you must answer for (subjects) is apart from what you only
assume of the world (environments), in one model.
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

**5. A rule's proof is checked by its type.** A rule says that its premises
give its conclusion. Its proof is a judgement expression from opinions of
the premises to an opinion of the conclusion. If it type-checks, the rule is
shown. Without a proof, the rule is assumed. This is the Curry–Howard
correspondence: a proposition is a type, and a proof is a value of it.
*Example*: `lands` is shown by `ponens(slows, falls, speed::slow)`; `slows`
is assumed, since no rule of judgement does arithmetic.
*Gain*: a proof is checked by the same core as everything else, with
nothing new to trust.
*Give up*: a proof reads only its premises. To use another rule, make it a
premise.
*Elsewhere*: Lean, Coq and Agda check proofs exactly so. An unproved rule
is Lean's `sorry`, or an axiom.

**6. Requires and ensures: what a part needs, and what it promises.** A
require comes from words (a hypothesis) or from another part's ensure. An
ensure follows from any one of its lists, each held together. With no list,
only someone taking it holds it.
*Example*: `stone::light` comes from `sun::shines`, and `lands_slow` follows
from `speed::slow` and `drop::falls`.
*Gain*: parts connect only through what one promises and the other needs,
and each link is a piece that can be blamed.
*Give up*: a link from an ensure to a require is a judgement that the
words match, and someone must take it.
*Elsewhere*: assume–guarantee contracts (Benveniste and others, 2018). The
lists are the "or of ands" of assurance cases (GSN).

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
*Elsewhere*: traceability links in requirements tools, and Rust's
`// mitra: implements` comments on main.

## Beyond what was named (going ahead; say no to undo)

1. **Paths are absolute**, from the part: `stone::drop::falls`, not
   `drop::falls` inside the stone. The language can resolve short ones later.
2. **A record's claims read it as `self`**, Rust's form, as in `self.h >= 0`.
3. **Ensures can be named in a rule or a list**, beside claims, rules and
   requires.
4. **An expression written in a rule is its own piece**, named by the rule
   and its place. Two rules never share one. To chain, name it as a claim.
5. **A proof reads its premises by path**, or by place when written in place.
6. **The subject of a rule's piece is the rule.** For a require from an
   ensure it is the require, and for an ensure's list the ensure.
7. **An opaque record's type has no field to read.**
8. **A tie names a part, record or function**, and its piece is
   `item at place`.
9. **Tuples are records with no name**, their fields `0`, `1`, … in order:
   `simulation::tuple(types)`. `(a, b)` is `Make` and `t.0` is `Field("0")`.
   No new operation.

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
measurements, cycles across parts (they need a fixpoint, in diagnose).
