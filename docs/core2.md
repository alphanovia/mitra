# core2: the expression type system

Three files. `src/expression.rs` is the core to freeze. `src/simulation.rs`
and `src/judgement.rs` are two standards on it. An expression never mixes
the two: `Expression<simulation::Op>` and `Expression<judgement::Op>` are
different Rust types.

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
| S2 | **Preservation and progress**, from S1 by induction: `check(Γ, e) = Some(T)` and `fits(Γ, values)` ⇒ `eval(values, e)` is `Ok(v)` with `has_type(v, T)`, or `Err(Op(_))`, never `Err(Stuck)`. The induction keeps each name in scope standing for a value of its type or an `Op` fault. | random well-typed expressions with `let`, `all` and a two-name binder, each reading |
| S3 | **Totality**: `check` and `eval` never panic, on any expression and values. They recurse once per level, as do `substitute` and the derived `Clone`, `PartialEq` and drop, and a binder's body runs inside its reading. So depth is bounded by the stack: on a 2 MiB thread, 1,000 levels in a debug build and 2,000 in release hold for all of them, where each `all` counts as three. Time and memory are not bounded by the size of an expression: an operation may run an argument many times (`all`, once per number), and each `let` may square a number. | random ill-typed expressions and values, names bound anywhere; 1,000 levels (2,000 in release) of `if`, of ponens, of `let`s in value and in body position, each also substituted, renaming at every level; nested `all`. Measured on 2 MiB: about 1,350 to 1,600 levels in debug, 2,300 to 4,800 in release |
| S4 | **Determinism**: `eval` is a function of reading, values and expression. | evaluating twice |
| S5 | **Covering**, per operation: typed as in S1, each bounded argument of its type or an `Op` fault, each exact one in `γ` of its bounds at its type, and each body run in `Exact` at values in `γ` of those run in `Ranges` giving an outcome in `γ` of what that gives ⇒ `covers(t, Ranges, Exact)`. Then by induction: `check(Γ, e) = Some(T)`, `fits(Ranges, Γ, B)`, each exact input in `γ` of its bounds at its type ⇒ `covers(T, eval(Ranges, B, e), eval(Exact, C, e))`. On points it loses nothing: when each bound is `Bounds::exactly` of the exact input, the bounded outcome is `Bounds::exactly` of the exact one, or the same fault. | each operation with fault arguments; `all` over uncertain ends; random expressions end to end; random expressions on points; `all` on numbers, each read |
| S6 | **Substitution**, in the core: when `e₁` checks to `T₁` and `e₂` to `T₂` with `x : T₁`, `substitute(e₂, x, e₁)` checks to `T₂`; on every reading, when `e₁` gives `v`, `e₂` with `x = v` gives what `substitute(e₂, x, e₁)` gives. **Let shares**, in `Exact`, `Ranges` and judgement: `let x = e₁; e₂` gives what `substitute(e₂, x, e₁)` gives, faults included, and `e₁` is evaluated once. **Let chooses once**, in a reading of sets: `let` gives a subset of what writing it out gives, so S6 fails there, by design. | random expressions whose binders bind the names of `e₁`; random expressions with lets and shadowing, written out, every reading, typed and wild; a count of operations applied; sets, random |
| J1 | **Rules preserve truth**: each set an opinion of `x` rests on, for or against, entails `x`, or `!x`, where `t = {p} s {q}` means `t → (∧p → ∧q)`, and a name `r` defined `{p} r {q}` the same. | every valuation of every random derivation, its rules written in place or known by name |
| J2 | **State is read from rests**: `state(not o) = not state(o)`, `state(join a b) = join(state a, state b)`. | random opinions |
| J3 | **Rests are the smallest, and all**: no rest holds another. Keeping only the takings in a set `S`, a proposition follows only if one of its rests lies in `S`, unless a `Reductio` lies below another; and, with no `Reductio`, whenever one does. A reductio's rest leaves out the taking it finds wrong. Below another reductio, a taking added can take a conclusion away: the inner one keeps only its smallest blame. | random derivations, random `S`; the case found by the red team |

## simulation.rs

- Types `Truth | Int | Real | Record { name, fields }`. Records are types of
  their own, so a record named "Real" is not `Real`.
- Values `Truth(bool) | Int(ℤ) | Real(ℚ) | Record(record, values)`. `Rational`
  is always in lowest terms, so equality is exact.
- Operations, in Rust's form: literals, `! && ||`, `+ - *` on two `Int` or two
  `Real`, `/` on `Real` (no value at 0), `< <=`, `==` on any one type,
  `within(a, b, tol)`, `if`, `R { .. }`, `r.x`, and two that bind a name:
  `let x = a; b`, and `(lo..hi).all(|x| p)` on two `Int` and a truth, `hi`
  left out, which reads `p` at each number in turn as `&&` does.
- Readings: `Exact`, and `Ranges`, which works on bounds: a truth may be
  true, false or both, a number lies in `[low, high]` or anywhere, a record is
  bounded field by field, and any of them may fault. The γ is `covers(T, b,
  x)`: `x` is a value of type `T` that lies in `b`, or a fault where `b` may
  fault or is one. Bounds carry no type; the expression's type says which.
- `all` by bounds: when both ends are one number each, `p` is read at each
  number in turn; else once, with `x` anywhere the range may reach, and true
  is allowed when the range may be empty.

## judgement.rs

- Types `State | Opinion(Prop)`. A proposition is `Named(n) | Not(x) |
  Implies { pre, subject, post }`, where the last is `t = {p} s {q}`.
- Rules known by name. `Judgement::with(definitions)` holds a
  `Definition { pre, post }` for each name, once, and `Named(r)` is then the
  rule `{pre} r {post}`. A name it does not know is no rule: a ponens on it
  has no type. `with` refuses the names defined through themselves, with
  `Err(names)`: a definition that names itself, or names a rule whose
  definition does, and so on, through negations and implications written
  in place too. `Judgement::default()` knows none. An opinion rests on
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
- Numbers sit only on takings, and no rule touches one.
- Costs: a ponens over `n` premises, each with two ways, rests on `2ⁿ` sets,
  each a different smallest way: inherent. A ponens copies its ways as they
  grow: quadratic in its premises (2,000 premises: 0.4 s). A reductio reads
  every pair of a way for and a way against.

## The choices in the core, in plain words

Each choice below is what it is, a tiny example, what we gain, what we give
up, and what other languages do.

**1. A standard is the language.** It lists the types and the operations,
and for each operation the type it gives from the types it takes, and the
names it binds. The core knows none of them.
*Example*: in simulation, `+` takes two `Int` and gives an `Int`; `let x =
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
for functions, where bounding a function is hard.
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

## Choices taken on Rust's form

`!x`, `&&`, `||`, `if`, `R { .. }`, `r.x`, `let x = e₁; e₂`, and
`(lo..hi).all(|x| p)` for "for all" over a range, `hi` left out, as Rust's
ranges and `Iterator::all`. `within(a, b, tol)` stays as you named it, where
Rust would write `(a - b).abs() <= tol`. Unlike Rust, a fault in what a `let`
binds counts only where it is read (5), and `&&`, `||` and `if` compute the
side they drop (8): the outcome is Rust's, the time is not. No
conversion is implicit: `Int` and `Real` never mix, and a conversion would
be an operation.

## Beyond what was named (going ahead; say no to undo)

1. Core: a **Standard / Reading** split, with constants as operations that take nothing. This lets one expression be read exactly and by bounds without copying it.
2. Core: **`Fault::Op` vs `Fault::Stuck`**, and operations see their arguments' faults, so `if` is not special.
3. **Int = ℤ, Real = ℚ, unbounded** (num-bigint). Nothing overflows. Machine numbers (i64, Int8) are not in this round.
4. **`/` on Real**, the one fault.
5. **`Prop::Not`**, with `not` on opinions (`!x`). There is no `and` on opinions, because a step's pre is already a conjunction.
6. **Taking `{ prop, value, count, by }`**. An opinion carries its proposition (`about`), so its value says what it is about.
7. Removed after the red team: **bias**, and **`yes` / `no`** (the least count to give up). No rule reads either, and the least count is a smallest hitting set: exponential, and it crashed on 20,000 takings. Numbers come back when the logic is settled.
8. Binding trees: the pair is named **`Arg { binds, body }`**, not `Bound`, since `Bounds` already means ranges in simulation.
9. **`Reading::bind`**, a second method beside `apply`, for operations that bind: plain arguments come evaluated, `Stuck` stands in for the others, and `run` evaluates them. `apply` is unchanged for every other operation.
10. **`let` in judgement too**, as it was before, now as one of its operations.
11. The **reading of sets with `choose`** lives in the tests only, to show `let` choosing once; no standard here has `choose`.

Not in this round: an `Option` type, `Int → Real` conversion, units, diagnosis, streams.
