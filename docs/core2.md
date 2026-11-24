# core2: the expression type system

Four files. `src/expression.rs` is the core to freeze. `src/simulation.rs`
and `src/judgement.rs` are two standards on it. An expression never mixes
the two: `Expression<simulation::Op>` and `Expression<judgement::Op>` are
different Rust types. Judgement reads simulation: the condition of a set
of points is a simulation expression. `src/tactic.rs` writes proofs that
judgement checks.

## expression.rs

```rust
enum Expression<Op> { Name(String),                    // what the nearest argument around it that binds it is given, or an input
    Apply(Op, Vec<Arg<Op>>) }                          // a constant: Apply(op, [])
struct Arg<Op> { binds: Vec<String>, body: Expression<Op> }  // an argument, and the names it binds, in scope in body only
trait Standard { type Type; type Op;                   // a language
    fn binds(&self, op, i, before: &[Type]) -> Vec<Type> { vec![] }  // the types of the names argument i binds
    fn gives(&self, op, takes: &[Type]) -> Option<Type>; }
trait Reading { type Standard; type Value; type Fault; // a meaning of it
    fn has_type(&self, v: &Value, of: &Type) -> bool;
    fn apply(&self, op, args: &[Outcome]) -> Outcome;  // no argument binds a name
    fn bind(&self, op, args: &[Outcome], run: &mut dyn FnMut(usize, &[Outcome]) -> Outcome) -> Outcome { Stuck } }
enum Fault<F> { Op(F), Stuck }          type Outcome<V, F> = Result<V, Fault<F>>
fn check(s, Γ: &BTreeMap<String, Type>, e) -> Option<Type>      // Γ ⊢ e : T
fn eval(r, values: &BTreeMap<String, Value>, e) -> Outcome
fn fits(r, Γ, values) -> bool                                   // each input has a value of its type
fn strict(args) -> Result<Vec<Value>, Fault>                    // all values, or the first fault
fn substitute(e, x, with) -> Expression                         // with in place of each free x, nothing caught
```

**Binding trees.** An expression is a name, or an operation on arguments,
each an expression and the names the operation binds in it (Harper, PFPL,
ch. 1). `let x = e₁; e₂` is `Apply(Let, [e₁, x.e₂])`, and
`(lo..hi).all(|x| p)` is `Apply(All, [lo, hi, x.p])`. Neither is in the
core: each is an operation of a standard.

**Typing judgement.** `Γ ⊢ Name(x) : Γ(x)`. For `Apply(op, [e₁ … eₙ])`, in
order: argument `i` binds names of the types `binds(op, i, [T₁ … Tᵢ₋₁])`,
as many as it has, and `eᵢ` checks to `Tᵢ` with them in scope; then the
whole has type `gives(op, [T₁ … Tₙ])`. A name stands for the nearest
argument around it that binds it, or else for the input of that name. An
argument with other names than its operation binds there does not check.

**Evaluation.** Every argument that binds no name is evaluated first, left
to right. If no argument binds a name, the reading's `apply` gets all the
outcomes. Else its `bind` gets them, with `Stuck` in the place of each
argument that binds names, and `run(i, values)` to evaluate argument `i`
with its names standing for `values`, as often as it needs.

**Substitution.** `substitute(e₂, x, e₁)` writes `e₁` in place of each `x`
free in `e₂`. An argument of `e₂` that binds `x` is left alone; one that
binds a name free in `e₁` around an `x` has that name renamed first (`y` to
`y'`, adding `'` until it is new), so no name of `e₁` is caught.

**Readings are functions.** `has_type`, `apply` and `bind` give the same
answer for the same arguments: a reading keeps no state that changes one.
S4 and S6 rest on it.

**Value typing.** `has_type(v, T)` belongs to the reading. A truth or a
number is of its kind. A record carries its record type, and each field must
be of its type; an opinion carries the proposition it is about. The reading
computes what a value carries, so preservation still checks it.

**Faults.** `Op(f)` is the reading's own "no value here" (a division by zero).
`Stuck` is evaluation going wrong (a missing input, an operation given values
it does not take). An operation sees each argument's outcome, fault included,
so `if`, `&&` and `||` drop the branch they do not take, fault and all. They
are ordinary operations, not special forms: both branches are computed (choice
8 below). Every other operation that binds no name is `strict`.

## Soundness statements

| | Statement | Tested by |
|---|---|---|
| S1 | **Agreement**, per reading and operation: `gives(op, ts) = Some(t)`, with argument `i` binding names of the types `binds(op, i, …)`, and each argument a value of its type or an `Op` fault, whenever its names stand for values of their types or `Op` faults ⇒ the operation gives a value of type `t` or an `Op` fault, and `bind` runs an argument only at such values. | depth-1 expressions over random inputs, every operation; `all` over random ends and bodies |
| S2 | **Preservation and progress**, from S1 by induction: `check(Γ, e) = Some(T)` and `fits(Γ, values)` ⇒ `eval(values, e)` is `Ok(v)` with `has_type(v, T)`, or `Err(Op(_))`, never `Err(Stuck)`. The induction keeps each name in scope standing for a value of its type or an `Op` fault. A call runs a body `Calls::with` checked, on values of its argument types; a call of a function with no body, at a point no row has, is the `Op` fault `Unknown`. | random well-typed expressions with `let`, `all` and a two-name binder, each reading; each calling a random library of three functions, with bodies (calling those before), rows, or nothing |
| S3 | **Totality**: `check` and `eval` never panic, on any expression and values. They recurse once per level, as do `substitute` and the derived `Clone`, `PartialEq` and drop, and a binder's body runs inside its reading. So depth is bounded by the stack: on a 2 MiB thread, 1,000 levels in a debug build and 2,000 in release hold for all of them, where each `all` counts as three and each call as two, and a set's condition runs inside the proof that reads it, so its levels add to the proof's, as a callee's body runs inside its call. Calls never go round, so evaluation ends. Time and memory are not bounded by the size of an expression, nor of the library: an operation may run an argument many times (`all`, once per number), each `let` may square a number, and each call runs its callee's body again, so `n` functions each calling the next twice run the last `2ⁿ` times. | random ill-typed expressions and values, names bound anywhere; 1,000 levels (2,000 in release) of `if`, of ponens, of `let`s in value and in body position, each also substituted, renaming at every level; nested `all`; chains of calls, alone and under a `let`. Measured on 2 MiB: about 1,350 to 1,600 levels in debug, 2,300 to 4,800 in release; a chain of calls, about 950 calls in debug, 2,800 in release |
| S4 | **Determinism**: `eval` is a function of reading, values and expression. | evaluating twice |
| S5 | **Covering**, per operation: typed as in S1, each bounded argument of its type or an `Op` fault, each exact one in `γ` of its bounds at its type, and each body run in `Exact` at values in `γ` of those run in `Ranges` giving an outcome in `γ` of what that gives ⇒ `covers(t, Ranges, Exact)`. Then by induction: `check(Γ, e) = Some(T)`, `fits(Ranges, Γ, B)`, each exact input in `γ` of its bounds at its type ⇒ `covers(T, eval(Ranges, B, e), eval(Exact, C, e))`. On points it loses nothing: when each bound is `Bounds::exactly` of the exact input, the bounded outcome is `Bounds::exactly` of the exact one, or the same fault, when every function called has a body. Of one without, `Exact` has a row or no value, and `Ranges` knows only the type. | each operation with fault arguments; `all` over uncertain ends; random expressions end to end; random expressions on points; `all` on numbers, each read |
| S6 | **Substitution**, in the core: when `e₁` checks to `T₁` and `e₂` to `T₂` with `x : T₁`, `substitute(e₂, x, e₁)` checks to `T₂`; on every reading, when `e₁` gives `v`, `e₂` with `x = v` gives what `substitute(e₂, x, e₁)` gives. **Let shares**, in `Exact`, `Ranges` and judgement: `let x = e₁; e₂` gives what `substitute(e₂, x, e₁)` gives, faults included, and `e₁` is evaluated once. **Let chooses once**, in a reading of sets: `let` gives a subset of what writing it out gives, so S6 fails there, by design. | random expressions whose binders bind the names of `e₁`; random expressions with lets and shadowing, written out, every reading, typed and wild; a count of operations applied; sets, random |
| J1 | **Rules preserve truth**: each set an opinion of `x` rests on, for or against, entails `x`, or `!x`, where `t = {p} s {q}` means `t → (∧p → ∧q)`, a name `r` defined `{p} r {q}` the same, an all is the conjunction of its body at each point of its set, and an any the disjunction. For a derivation whose inputs are what can be given: opinions `Opinion::taken` makes and points `Point::of` makes. Inside an `intro`, the point stands for each point of its set, and only what the intro proves leaves it. | every valuation of every closed step of every random derivation, its rules written in place or known by name; half the worlds over points, with alls and anys over four sets (one empty, one with a condition), nested, a name bound again, rules under an all, intros, nested, with instances, witnesses and rules at the point, each instance and each set's points listed by hand |
| J2 | **State is read from rests**: `state(not o) = not state(o)`, `state(join a b) = join(state a, state b)`. | random opinions |
| J3 | **Rests are the smallest, and all**: no rest holds another. Keeping only the takings in a set `S`, a proposition follows only if one of its rests lies in `S`, unless a `Reductio` lies below another; and, with no `Reductio`, whenever one does. A reductio's rest leaves out the taking it finds wrong. Below another reductio, a taking added can take a conclusion away: the inner one keeps only its smallest blame. | random derivations, random `S`; the case found by the red team |

## simulation.rs

- Types `Truth | integer | rational | Record { name, fields }`. Records are types of
  their own, so a record named "rational" is not `rational`.
- Values `Truth(bool) | Int(ℤ) | Real(ℚ) | Record(record, values)`. `Rational`
  is always in lowest terms, so equality is exact.
- Operations, in Rust's form: literals, `! && ||`, `+ - *` on two `integer` or two
  `rational`, `/` on `rational` (no value at 0), `< <=`, `==` on any one type,
  `within(a, b, tol)`, `if`, `R { .. }`, `r.x`, and two that bind a name:
  `let x = a; b`, and `(lo..hi).all(|x| p)` on two `integer` and a truth, `hi`
  left out, which reads `p` at each number in turn as `&&` does.
- Readings: `Exact`, which computes only the side of `if`, `&&` and `||`
  it takes, and `Ranges`, which works on bounds: a truth may be
  true, false or both, a number lies from a low end to a high end, either
  of which may be missing (`q >= 1` is `[1, ∞)`), a record is
  bounded field by field, and any of them may fault. The γ is `covers(T, b,
  x)`: `x` is a value of type `T` that lies in `b`, or a fault where `b` may
  fault or is one. Bounds carry no type; the expression's type says which.
- `all` by bounds: when both ends are one number each, `p` is read at each
  number in turn; else once, with `x` anywhere the range may reach, and true
  is allowed when the range may be empty.
- Calls (`use`): `sun::light()` calls a function of the model by its path,
  typed by its signature. `Calls::with` takes the library once: what each
  function takes and gives, its body, or else its rows (examples). It refuses,
  by name, a function that calls itself however far, one that calls a
  function the library lacks, a body not of its type, a row not of its types,
  two arguments of one name, two rows at one point that give different values, and whatever calls one it refuses. The standard
  and both readings hold the same library. `Exact` runs the body, or reads what the reading
  was given at the arguments (`Calls::given`: a world value a measurement or
  an example of the model saw, for a function with no body, of its types),
  or else the row there; with neither, the call has no value:
  `Undefined::Unknown(path)`, an `Op` fault, so S2 holds. `Ranges` runs the
  body, or else allows any value of the type, and a fault. A call is strict,
  as Rust's: a fault in an argument is the call's, read or not.
- Type parameters, `T` and `T: Number` (`Type::Param`), for contracts: a type
  named but not fixed. `T: Number` has `+ - *`, `< <=`, `within` and `==`;
  `T` has `==` only. No literal is of a parameter (`x + 1` is refused), no
  value is either, and a function whose types name one has no body and no
  rows: its contract says the rest. `Type::given` fixes them.

## judgement.rs

- Types `State | Opinion(Prop) | Point(Term)`. A proposition is `Named(n) | Not(x) |
  Implies { pre, subject, post } | Correct { function, at } | All { name,
  over, body } | Any { name, over, body }`, where `Implies` is `t = {p} s
  {q}`.
- Points. `correct(f, at)`: the function `f`, known by its path, gives
  what it should at `at`. Judgement does not know `f`; the model does. `at`
  is a term: `Point([6, 2])`, the values of `f`'s arguments in order, as in
  a row of its examples; `Var(x)`, the name an all or an any around it
  binds; or `Arbitrary { name, over }`, the point an `intro` binds, any
  point of `over`. `all x in S: p` holds when `p` holds at every point of
  `S`, `x` standing for the point; `any x in S: p`, at some point.
- A point is a value of a proof, and its type is the point itself:
  `Point([6, 2])` writes `[6, 2]`, of type `Point([6, 2])`. `Instance` and
  `Witness` take it as their second argument, so a `let` can share it and
  an `intro` can give it.
- Sets. `Set { ranges: [(a, 0, 10), (b, 0, 10)], such: Some(b != 0) }` is
  the points `[a, b]` with `a` and `b` in `0..10`, `hi` left out as in Rust,
  where `b != 0`. `such` is read by `Exact`, each name standing for its
  number; a fault is not true. A set names nothing outside itself. A set of
  whole numbers lists its points, in Rust's order of tuples; one of reals
  only says whether a point is in it.
- Well formed: each name a term uses is bound by an all or an any around
  it, and each set is (names distinct, each range two `integer` or two `rational`,
  `such` a truth). Judgement reasons about no other: `Opinion::taken`
  refuses one, and no rule gives one. `not`, `join`, `state` and
  `reductio` may name one in the type of what they take, which no value
  fits.
- Dependent types, syntactically: the type of an opinion names its
  proposition, points and bound names included, as `Opinion(correct(div,
  [6, 2]))`. Two types are equal only when written the same. A rule at a
  point writes the point in place of the name (`Prop::at`), so the types of
  its arguments say which point.
- Rules known by name. `Judgement::with(definitions)` holds a
  `Definition { pre, post }` for each name, once, and `Named(r)` is then the
  rule `{pre} r {post}`. A name it does not know is no rule: a ponens on it
  has no type. `with` refuses the names defined through themselves, with
  `Err(names)`: a definition that names itself, or names a rule whose
  definition does, and so on, through negations and implications written
  in place too. `with` does not ask that a definition be well formed: one
  that is not is never applied. `Judgement::default()` knows none. An opinion rests on
  takings under the definitions of the judgement that made it, and does
  not say which: read it only with that judgement, and make it again
  after a rule changes.
- Values: a `State` (Belnap), or an `Opinion { about, rests_for,
  rests_against }`. The rests are the smallest sets of takings, where a taking
  is `{ prop, value, count, by }`. The state is read from which rests are
  empty. Counts stay on the takings; nothing reads them yet.
- The rules, whose types are the rules (premises of `t` = `t, p₁ … pₙ`,
  read by `premises(t)` and `posts(t)` from `t`, or from its definition):
  - `Ponens { t, post }`: opinions of every premise give one of `q`.
  - `Tollens { t, post, blame }`: `q`, then every premise but `blame`, give an opinion against premise `blame`.
  - `Reductio { of }`: an opinion of `x` gives one of `of`.
  - `Not` and `Join` on opinions; `And`, `Not` and `Join` on states; `StateOf` gives an opinion's state.
  - `let x = a; b`, to use one derivation twice.
  - `Instance { q }`: from an opinion of `q`, an all or an any, then a
    point of its set, an opinion of its body at the point. An all true
    makes it true; an any false makes it false. Down.
  - `Witness { q }`: from an opinion of the body at a point, then the
    point, one of `q`. True there makes an any true; false there makes an
    all false. Up, from one point.
  - `Cases { q }`: from an opinion of the body at each point of the set, in
    order, one of `q`: all the points tell. An all is true when every point
    is, false when one is; an any is true when one is, false when every
    one is. Over an empty set, an all is true and an any false, from
    nothing. Up, from every point.
  - `Intro { q }`: `intro(q, |x| p)`, from `p`, an opinion of the body of
    `q` at `x`, the point the intro binds, one of `q`. True at any point
    makes an all true; false at any point makes an any false. Up, from any
    point: one proof for every point, whatever the size of the set.
  - No rule goes up from some of the points: that would be induction. An
    all over a set too large to list is taken (a review), refuted (a
    witness), or proved by an intro from what holds on all of it.
- The point an intro binds. Only an intro makes it, and only for its
  argument: `Point` keeps its term private, and `Point::of` writes a point
  in place. Nothing taken is about it (`Opinion::taken` refuses), nor any
  rule known by name. An instance or a witness at it needs its set within
  the other's (`Set::within`, by writing: as many ranges, each within the
  other's, of the same kind, and the other's `such` none or the same).
  Inside the intro, what follows is about any point of the set: of an
  empty set, there is none, and what follows there says nothing, as after
  Lean's `intro x` on an empty type. Only what the intro proves leaves it,
  and that holds of an empty set too: an all is true, an any false. The
  point is fresh: an intro of a proposition that already writes it, as an
  instance at it can, is refused, since a point is told by its name and
  set only and the two would be caught as one.
- Counts sit only on takings, and no rule touches one.
- Costs: `Cases` reads every point of its set's ranges, in the set or not,
  when it is checked as when it runs: checking a proof is no longer linear
  in its size, and `0..1000 × 0..1000` is a million points. `such` runs once
  per point read. Running it adds each case's takings once: 4,000 points of
  one test each take milliseconds (the red team found it quadratic, 6 s;
  fixed). A ponens over `n` premises, each with two ways, rests on `2ⁿ` sets,
  each a different smallest way: inherent. A ponens copies its ways as they
  grow: quadratic in its premises (2,000 premises: 0.4 s). A reductio reads
  every pair of a way for and a way against.

## tactic.rs

A tactic writes a proof, and judgement checks it like any other. It is
trusted with nothing: a wrong proof does not check, whatever wrote it, and
a checked proof is a taking by the checker (`docs/model.md`, choice 5).
`cases(q, case)` writes `Cases { q }` with one argument per point of `q`'s
set, in order: `case(goal, point)` writes a proof of `goal`, the body at
that point. `intro(q, name, body)` writes `Intro { q }`: `body(goal, at)`
writes a proof of `goal`, the body at `at`, the point it binds, which the
proof calls `name`, as Lean's `intro x`. One proof, whatever the size of
the set: on `0..2⁴⁰ × 1..2⁴⁰`, "quot is correct everywhere" rests on two
takings, "div is correct everywhere" and "quot is correct wherever div
is". An intro does not split a set by its condition: it reads only what
holds on all of its set, so div's two classes need `cases` (or a taking).

On main's `div(a, b)`, `a` and `b` in `0..10`, "div is correct
everywhere" rests on 100 tests by cases; or on 4 takings: a review of each
class (`b != 0`, `b == 0`) as a rule, "correct at one point gives correct on
the class", and one test in each. `cases` then takes each point from its
class by `Instance`. A failing second test in a class refutes its review,
by `Witness` then `Tollens`, naming both tests (`tests/tactic.rs`).

## The choices in the core, in plain words

Each choice below is what it is, a tiny example, what we gain, what we give
up, and what other languages do.

**1. A standard is the language.** It lists the types and the operations,
and for each operation the type it gives from the types it takes, and the
names it binds. The core knows none of them.
*Example*: in simulation, `+` takes two `integer` and gives an `integer`; `let x =
e₁; e₂` binds `x` in `e₂`, with the type of `e₁`.
*Gain*: one core and one checker for every language Mitra needs (simulation,
judgement, others later). A new operation is a few lines in a standard.
*Give up*: the core cannot help a standard beyond what it declares: no
function that works for every type, no type worked out from how a name is
used. Each operation's types are listed by hand.
*Elsewhere*: most languages build their types and operators in (Rust's
`i32 + i32`). Logics and SMT solvers split them out like this: a signature,
or a theory (SMT-LIB's integers, reals, arrays), over a fixed core. So do
MLIR's dialects in compilers.

**2. A reading is a meaning.** It says what the values are, which value has
which type, and what each operation gives on values. One expression can
have several readings.
*Example*: `x + 1` with `x = 2` gives `3` in `Exact`; with `x` in `[0, 5]` it
gives `[1, 6]` in `Ranges`. The same expression, checked once.
*Gain*: a model is written once, then run, bounded or judged; covering (S5)
says one reading is a safe summary of another.
*Give up*: every operation needs a meaning in every reading, so the work is
operations times readings; and the Rust compiler cannot check that a reading
keeps to the types. The tests do (S1).
*Elsewhere*: an ordinary program has one meaning, running it. Static
analysers give the same program a second one (abstract interpretation:
Astrée, Frama-C). Haskell's "tagless final" style writes one program
for several interpreters.

**3. Two kinds of "no value": `Op` and `Stuck`.** An `Op` fault is the
operation's own: it has no value here. `Stuck` is evaluation going wrong: the
expression or its inputs do not fit.
*Example*: `1 / x` at `x = 0` is `Op(DivisionByZero)`, a fact about the
model. `y + 1` with no `y` is `Stuck`, a mistake in the model or the tool.
*Gain*: S2 can promise that a checked expression on inputs that fit is never
`Stuck`, while `Op` faults stay, as findings.
*Give up*: two cases to handle, not one; and a reading must never use `Stuck`
for a real "no value".
*Elsewhere*: "well-typed programs cannot go wrong" (Milner, 1978): type
errors are found before running, division by zero stays a run-time error.
Rust: compile errors, then panics. Python finds both while running
(`TypeError`, `ZeroDivisionError`), so nothing is promised before.

**4. Binders: arguments carry the names they bind.** `let x = e₁; e₂` gives
the name `x` to what `e₁` gives, for `e₂`; `(lo..hi).all(|x| p)` gives `x`
each number in turn, for `p`. The core handles scope, shadowing and
substitution once, for all such operations. The standard says the types of
the names; each reading says how often to run the body, and with what.
*Example*: `let` is `Apply(Let, [e₁, x.e₂])` and `all` is `Apply(All, [lo,
hi, x.p])`; neither is in the core.
*Gain*: a new binder (a sum over a range, "for all" over a list, a stream's
`rec`) is an operation of a standard; the core stays as it is. Scope rules
cannot differ between `let` and `all`.
*Give up*: the core is bigger (substitution with renaming); expressions are
wordier to build by hand; every reading defines every binder; a body runs
inside its reading, which costs stack. The depth bound drops from 1,500 to
1,000 levels in a debug build, and an `all` counts as three. A chain of
`let`s used to take no stack at all.
*Elsewhere*: most languages build in a fixed few binders (`let`, functions,
`match`). The general form is Harper's abstract binding trees. Proof
assistants (Lean, Coq, Isabelle) get the same headroom another way: a few
binders built in, the function `fun x => p` among them, and every other
binder an operation on a function, as `∃ x, p` is `Exists (fun x => p)`. That way needs functions as
values, which we do not have (6).

**5. `let` chooses once.** When a value may be one of several (a set, later
a distribution), `let x = e₁; e₂` picks one of what `e₁` may be, and every
`x` in `e₂` is that one.
*Example*: `let x = choose(0, 1); x - x` gives `{0}`. Written out,
`choose(0, 1) - choose(0, 1)` gives `{-1, 0, 1}`: each `choose` picks on its
own.
*Gain*: a name means one thing wherever it is read, as in maths ("let x be
the throw of a die; then x − x = 0") and in everyday code (`let x =
rand(); x - x` is 0).
*Give up*: in such a reading a `let` is not its value written out, so S6
holds only where `let` shares one outcome. A reading of sets runs the body
once per choice, which multiplies the work.
*Elsewhere*: probabilistic languages (Church, WebPPL, Pyro) and the language
Curry work this way ("call-time choice"). `Ranges` is not a reading of
choices: it shares the whole bound, so `x - x` with `x` in `[0, 1]` gives
`[-1, 1]`. Still sound, but it forgets that both `x` are one (the
"dependency problem" of interval arithmetic). The reading of sets is in the
tests only (`tests/expression.rs`).

*A fault in what a `let` binds counts only where `x` is read: kept.* In a
reading that shares, `let` stays exactly its value written out, faults
included (S6), and `let q = 1 / x; if x != 0 { q } else { 0 }` has a value
at 0. What we give up: unlike Rust, which panics at the `let`, a `let` whose
value is never read hides its fault.

**6. First-order: values are data, never functions.** You cannot pass a
function, return one, or keep one in a record.
*Example*: `(lo..hi).all(|x| p)` looks like a Rust closure, but `p` is an
argument that binds `x`, run by `all` itself. It is not a value you could
pass on.
*Gain*: types stay simple (no function types); with no function to call
itself, evaluation always ends; and `Ranges` and judgement need no meaning
for functions, where bounding a function is hard. A function of the model
is called by its path (12), never passed.
*Give up*: no helper of your own that takes a function (`map(xs, f)`). Each
such pattern must be an operation of the standard, with a binder.
*Elsewhere*: Rust, Python, JavaScript and Haskell pass functions as values.
SQL, Excel formulas (before `LAMBDA`), Datalog, and Lustre (used for
avionics software) do not.

**7. Readings keep no state.** A reading's methods are functions: the same
arguments always give the same answer. A reading may not count, print or
draw a random number in a way that changes an answer.
*Example*: if `+` gave another answer on its tenth call, `let x = a + b; x *
x` would differ from `(a + b) * (a + b)`, and one expression evaluated twice
would differ.
*Gain*: S4 (the same answer every time) and S6 (sharing); the order of
evaluation does not matter, and results can be cached or computed in
parallel.
*Give up*: no side effects in a reading. Chance goes in the values (sets,
distributions), never drawn while evaluating.
*Elsewhere*: Haskell makes every function pure and puts effects in types.
PostgreSQL lets a function be declared `IMMUTABLE`, so it can be cached.
Excel formulas are pure, but for "volatile" ones such as `RAND()` and
`NOW()`. Rust and Python allow side effects anywhere.

**8. Every argument is computed before its operation reads it**, but those
that bind names, which the operation runs itself.
*Example*: `if x == 0 { 0 } else { 1 / x }` at `x = 0` computes `1 / x`, gets
a fault, and drops it: the outcome is `0`, as in Rust.
*Gain*: one order everywhere, simple to state and to test; the outcome is
Rust's, faults included.
*Give up*: time. A guard saves no work: `false && (0..10^15).all(|i| …)`
runs the whole range, where Rust would skip it. Skipping would need a reading
to ask for each argument when it needs it: one method in place of `apply` and
`bind`, and more stack per level.
*Elsewhere*: most languages compute arguments first ("call by value"), with
`&&`, `||` and `if` built in as exceptions. Haskell computes nothing until it
is needed ("lazy").

**9. Dependent types, by writing.** The type of an opinion says what it
is about, down to the point. Two types are equal only when they are
written the same.
*Example*: `Instance` of `all x in D: correct(div, x)` at `[6, 2]` has type
`Opinion(correct(div, [6, 2]))`. A ponens that needs the test at `[7, 3]`
does not take it.
*Gain*: the checker knows which point each opinion is about, so a proof
cannot pass off one point's test for another's. One checker for all.
*Give up*: equality is only by writing: `all x in S: p` and the same with
`y` are two propositions, and nothing is computed in a type (`[12 / 2, 2]`
could not even be written: a point holds values only). Checking a proof
may run `such` and list points.
*Elsewhere*: Lean, Coq and Agda have full dependent types, where types
equal up to computation are the same (`2 + 2` and `4`). LambdaPi (Löh,
McBride and Swierstra, 2010) is the smallest such checker. Dependent ML
(`int(n)`) and TypeScript's literal types (`let n: 5`) put values in
types, with no computation.

**10. Tactics outside the kernel.** Judgement checks proofs; tactics write
them.
*Example*: `cases(q, …)` writes div's proof by cases, 100 arguments; the
kernel lists the 100 points itself and checks each argument against its
point.
*Gain*: a bug in a tactic cannot make a wrong proof pass. A new tactic
changes nothing in the kernel, which stays small enough to red-team.
*Give up*: a proof a tactic writes can be large (one argument per point),
and the kernel does again what the tactic did.
*Elsewhere*: Lean and Coq: tactics write terms, a small kernel checks them
(the "de Bruijn criterion"). Isabelle and HOL: only the kernel's functions
make theorems (the LCF style).

**11. The any point.** `intro` proves an all from one proof at a point
that stands for every point of the set.
*Example*: `intro(all x in D: correct(quot, x), |p| ponens(chain at p,
div at p))`: if div is correct on all of `D` and quot is wherever div is,
quot is, for 2⁸⁰ points, in one step.
*Gain*: a set too large to list is proved, not only taken or refuted, and
the proof stays small. A chain of rules over a set becomes one proof.
*Give up*: the point is special. Only an intro makes it, nothing taken can
be about it, and what is derived inside is about any point: over an empty
set, about none. Its set must be written within the sets it reads.
*Elsewhere*: Lean's `intro x` and Coq's `intros`, the ∀-introduction of
natural deduction: prove `p(x)` for a fresh `x`, then `∀x. p(x)`. The
"fresh" condition is what the private `Point` keeps here, with the refusal of an intro whose proposition already writes its point.

**12. Calls by path, never round.** A function of one part can call a
function of another by its path, as Rust's `use`; a call takes the values
of its arguments first; no function calls itself, however far.
*Example*: `stone::speed(h)` reads `sun::light()`. `Exact` runs light's
body, or reads its row for `()`; if it has neither, the call has no value,
named: "no value for sun::light here". `Ranges` runs the body, or allows any
value of light's type.
*Gain*: the model computes across parts, so a test of A's inputs can state
B's outputs. With no cycle, evaluation still always ends, and a function is
checked once, when the library is made, not at each call.
*Give up*: no recursion: a function that needs it is written as an `all`
over a range, or left opaque with rows. By bounds, a function with no body
is known only by its type: its rows and claims are not read there (yet).
*Elsewhere*: Rust, Python and most languages allow recursion. Lustre and
SCADE (avionics), Excel and SQL views refuse cycles, for the same reason:
every evaluation ends and its cost is known.

**13. Type parameters.** A contract can name a type it does not fix: `T`,
with `==` only, or `T: Number`, a whole number or a real, with `+ - *` and
`<`. It is checked once with the parameter; each use fixes it.
*Example*: `contract Monotone<T: Number> { fn f(x: T) -> T; x <= y implies
f(x) <= f(y) }` is checked once; `impl Monotone<rational> for speed` checks only
that `speed` fits (takes and gives a `rational`), and the claim becomes one
about `speed`.
*Gain*: one property written once for every function that has it; an error
is reported at the contract, not at each use.
*Give up*: nothing can be computed on `T` itself: no value is of it, so a
contract's claims are proved by rules, or checked at each use. No literal is
of `T` (`x + 1` is refused): a number written in a contract must be given.
*Elsewhere*: Rust's generics with trait bounds (`T: Ord`), Haskell's type
classes, Java's interfaces. `Number` is built in here; Rust's standard
library has no such trait (crates such as `num-traits` add one).

## Choices taken on Rust's form

`!x`, `&&`, `||`, `if`, `R { .. }`, `r.x`, `let x = e₁; e₂`, and
`(lo..hi).all(|x| p)` for "for all" over a range, `hi` left out, as Rust's
ranges and `Iterator::all`; `sun::light()` for a call by path; `T` and
`T: Number` for type parameters, though Rust's standard library has no
`Number`. `within(a, b, tol)` stays as you named it, where
Rust would write `(a - b).abs() <= tol`. Unlike Rust, a fault in what a `let`
binds counts only where it is read (5), and `&&`, `||` and `if` compute the
side they drop (8): the outcome is Rust's, the time is not. No
conversion is implicit: `integer` and `rational` never mix, and a conversion would
be an operation.

## Beyond what was named (going ahead; say no to undo)

1. Core: a **Standard / Reading** split, with constants as operations that take nothing. This lets one expression be read exactly and by bounds without copying it.
2. Core: **`Fault::Op` vs `Fault::Stuck`**, and operations see their arguments' faults, so `if` is not special.
3. **integer = ℤ, rational = ℚ, unbounded** (num-bigint). Nothing overflows. Machine numbers (i64, Int8) are not in this round.
4. **`/` on rational**, the one fault.
5. **`Prop::Not`**, with `not` on opinions (`!x`). There is no `and` on opinions, because a step's pre is already a conjunction.
6. **Taking `{ prop, value, count, by }`**. An opinion carries its proposition (`about`), so its value says what it is about.
7. Removed after the red team: **bias**, and **`yes` / `no`** (the least count to give up). No rule reads either, and the least count is a smallest hitting set: exponential, and it crashed on 20,000 takings. Numbers come back when the logic is settled.
8. Binding trees: the pair is named **`Arg { binds, body }`**, not `Bound`, since `Bounds` already means ranges in simulation.
9. **`Reading::bind`**, a second method beside `apply`, for operations that bind: plain arguments come evaluated, `Stuck` stands in for the others, and `run` evaluates them. `apply` is unchanged for every other operation.
10. **`let` in judgement too**, as it was before, now as one of its operations.
11. The **reading of sets with `choose`** lives in the tests only, to show `let` choosing once; no standard here has `choose`.
12. A **point is the list of a function's arguments**, `[6, 2]`, as in a row of examples, not one tuple value.
13. A **set's ranges are named**, `(a, 0, 10)`, `hi` left out as in Rust, and `such` reads those names. A fault in `such` is not true.
14. **`Cases` gives all the cases tell**, both sides: an all is also false when one case is, as `Witness` says.
15. **Substitution in a proposition is its own** (`Prop::at`): a point holds no name, so nothing can be caught, and core's `substitute` works on expressions only.
16. **Only well formed propositions** (`Prop::well_formed`): `Opinion::taken` and every rule refuse the others.
17. `simulation::Value` is now **ordered and hashed**, and `simulation::Op` ordered, to keep propositions in sets; a `Set` orders its `such` part by part, in judgement.rs, since expression.rs does not change.
18. **Points are values of proofs**, of a type that is the point itself (`Type::Point`), written by `Op::Point`: `Instance` and `Witness` now take the point as an argument, not a parameter, so that an intro can give one.
19. **`Point` keeps its term private**: only an intro makes the any point, so what is given to a proof is a point written in place.
20. **`Set::within` is by writing**: a set with a condition is within another only if that one has none or the same. Whether one condition holds wherever another does is not asked.
21. **Inside an intro, anything may follow at the point**, witnesses and rules included: only what the intro proves leaves it.
22. **The library travels with the standard**: `Simulation`, `Exact` and `Ranges` each hold the same `Arc<Calls>`; `::default()` has no functions.
23. **A call with no value is named**: `Undefined::Unknown(path)`, an `Op` fault beside the division by zero.
24. **By bounds, a function with no body gives any value of its type, and may fault**: its rows and claims are not read there.
25. **A function whose types name a parameter is bare** (no body, no rows): only its contract speaks of it.
26. **An intro's point is fresh**: an intro of a proposition that already writes the point it would bind is refused (red team round 7 caught the diagonal taken for the square).
27. **A function gives one value at a point**: `Calls::with` refuses two rows at one point that differ (check says M024). Beside a body, rows are kept and never read: `agree` compares them with the body (A002). What a reading is given (`Calls::given`) is read before the rows, never before a body.
28. **Half-lines** (finding 6): a span of numbers has a low end and a high end, either of which may be missing; zero times no end is zero.
29. **Comparisons read together, numbers cut at zero** (finding 7): `Ranges::kept` cuts each name by the comparisons joined by `&&`, one name at a time, strict on `integer` only; agree reads its first `splits` numbers (a setting, 4) below and above zero.
30. **Calls remembered, `if` lazy, budgets** (finding 9): `Exact` and `Ranges` remember a call's outcome at its arguments; `Exact` computes only the side of `if`, `&&`, `||` it takes (`Reading::needed`, in `expression`); a computation past `Budget { steps, depth }` (settings, a million calls and 1,000 levels) gives `Undefined::Budget`: agree says unknown (A007), explain X009.
33. **Values known in part** (finding 2): `Location { depth: 2, .. }`, Rust's struct pattern (flagged: a pattern stands for a value here), written in rows, examples and measurements of a declared record; a field not known may be of an opaque record. Reading it, or an `==` only it decides, gives `Undefined::Unknown("Location::first")`; bounds read it as any value of its type. Agree tries records whose opaque fields are not known, and says unknown where only they could tell; explain takes nothing there and says X010.
32. **Whole division and range types** (finding 11): `/` on two `integer` rounds toward zero and `%` takes the sign of its left side, as Rust's `i64`; no value at zero; `/` on two `rational` stays exact; no mix. A span says its numbers (`Numbers::Int` or `rational`). A range type `a: 0..=9` is an `integer` that is checked (your option 1): a call outside a range of what it takes, or a body giving outside the range it gives, has no value (`Undefined::Outside`); agree reads a claim as `!(in ranges) || claim`, checks the body keeps its gives range (A001, `its range`), and tries every point when the ranges hold at most `points` of them; check refuses an empty range (M025) and an example outside one (M026); a measured value outside one was refused (V008), now evidence against (choice 34). `implied` reads a conclusion, and the lets it reads, only where the premises hold when every premise is read first.
31. **A function with no body gives what its claims promise** (finding 8): `Callee.promises` holds its claims; by bounds, a call of it is what `Ranges::kept` keeps of its gives name under them, and never a fault (the world gives a value); a call of it inside its own claims is only its type. `covers` lets in any bounds for a value `Exact` does not know (`Undefined::Unknown`). Agree reads `!p || c` as `c` where `p` keeps its names, after lets; every let, premise and conclusion must be fault-free by bounds. A result read so names the claims it rests on (`Agreed.given`, hint A010).
34. **A measured value outside its range** (your answer, 18:05): evidence of a drift between the model and the code, not an error. Explain says X011 and takes `correct(f, x)` false by that measurement, through f's ties, so blame lists the measurement and the ties; the caller of an opaque value outside its range does not read it as a row. A test that expected such a value and failed takes nothing. V008 is retired; an example row outside a range stays M026 (the model contradicting itself).

35. **Enums and generic records** (your "ok", 19:56): `simulation::Enum { name, args, variants }`, `Type::Enum`, `Value::Variant(enum, index, values)`; `Op::Variant(enum, i)` makes one, `Op::Match(enum)` reads one, its arguments the value then one arm per variant in the enum's order, each arm binding what its variant holds. Exact computes only the arm taken; by bounds a span of an enum is a span per variant that may be there (`Span::Variant`), and an arm is read only where its variant may be. A `Record` and an `Enum` carry their type arguments (`args`); `Type::learn` reads them from a value. Agree tries every variant. Check refuses an enum with no variant, a parameter nothing uses, and a type that holds itself (M027); the parser follows a record or enum inside itself once, then leaves it empty (a cut at the first repeat, not at a depth: `Node(Tree, integer, Tree)` would double at each level). The type of a variant or a generic record its values do not tell (`None`, `Box { v: None }`) is a hole the parser fills from where it goes: a result, a field, a call's argument (by the callee's signature), the other side of `==` or of an `if`; one still open is P009. Settling reads an argument twice only when a hole is still in it (reading all twice doubled the time at each level of parentheses).

36. **Files and Mitra's standard model** (your "ok", 19:56): `use std::core::Option;` reads the part `core` of `std/core.mitra`, shipped in the binary (`parse::STD`), with `Option<T>` and `Result<T, E>` as ordinary enums; nothing in it is built in. A part a file uses and does not declare is read from `<part>.mitra` beside the model (`parse::load`, the CLI), once, whichever file uses it; a part found nowhere is P010, at the `use`, and an error in a file read is given with that file. `parse` reads `std::` only. Each part keeps the file it came from (`Part.file`); `print` writes the model's own file, its uses of std as `use std::…`. A type of std named with no `use` is M003, which says the `use` to write. Names of types stay global (finding 13).

37. **Budgets are said, never silent** (finding 24, your "ok"): the parser has a depth budget, `parse::Settings::depth` (2000, twice simulation's depth), a setting, past which a text is P011 at the place it stopped, not a crash; the CLI reads on a 256 MiB stack. Bounds that stop at simulation's budget are a fault `Budget` carried by `join`, never absorbed into "may fault": agree then says A007 "not decided by bounds, which stopped at …" (and "not false by bounds, which stopped at …" for claims together). The two proptest seeds of the judgement tests were the generator's (finding 22): it built an intro over a point its proposition already writes, which the kernel refuses (choice 26); it no longer does, and both seeds are a test.

38. **A proof writes no state** (finding 27, your "option 1"): `state(…)` and `&&` on states left the proof text, the parser and print. They stay in judgement (`Op::StateOf`, `Op::And`), where explain, numbers and the verdicts read what is known of an opinion; no proof can give an opinion through them, so `check` refused every proof that wrote them (M012). Built from Rust, they print as `__malformed__`, as an operation on other arguments does. `!` on an opinion stays; `|-` stays for rules.

39. **Sequences, `[T]`, the minimal set** (finding 3, step 2; your "ok", 20:24): `Type::Seq`, `Value::Seq(T, values)`, `Op::Seq(T)` for `[a, b]`, `Op::Len` for `xs.len()`, `Op::Index` for `xs[i]`, and `==` place by place. `xs[i]` with no place `i` is a fault, `Undefined::Index`, as division by zero ("the index is 2, and the sequence holds 2"). By bounds, `Span::Seq` keeps its length (an `integer` span from 0), a span every value lies in, and the span of each first place it surely holds, so that a sequence known at a point is known exactly (S3, "points are exact"). `[]` is told from where it goes, as `None` (P009). Agree tries sequences up to `Settings::length` long (a setting, 3). A record holding a sequence of itself stays M027 until trees.

40. **Text, `[~char]`** (finding 3, step 3; your "go"): `Type::Encoded(T)` (only `Encoded(Char)` parses, else P008), `Type::Char`, `Type::Byte`; `Value::Text(String)` (UTF-8, as Rust's `str`) and `Value::Byte(u8)`; `Op::Text` for `"abc"`, `Op::Join` for `++`, `Op::StartsWith`, `Op::Decoded`. `Len` and `Index` read text in bytes ("the index is 3, and the text holds 2 bytes"). `++` and `starts_with` take two texts or two sequences of one type; `<` and `<=` take two texts or two `char` (byte order); `byte` has `==` only. `.decoded()` types on `[T]` only (itself): on text, check says it waits for the grapheme decoder. By bounds, `Span::Text(length, known)`, `Span::Byte(known)`, `Span::Char`; a join adds lengths and keeps the places surely held. Agree tries `""`, the model's texts, their joins and prefixes; `char` and `byte` have no value written, so none is tried. A byte prints as Rust's byte literal, `b'A'`, shown only. The dogfood's `std::String` is gone: String is `[~char]`.

41. **`integer` and `rational`** (your "go", 10:21): the names `Int` and `Real` are gone from the text, the printer and every message; `Type::Int` and `Type::Real` keep their names inside. The old names are now names of nothing (M003), with no hint (no refusal hints for retired forms).

Not in this round: `integer → rational` conversion, units, diagnosis, streams, recursion, functions as values, and an all or an any written in the model (next, in `model.rs`).
