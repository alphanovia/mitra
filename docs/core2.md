# core2: the expression type system

Three files. `src/expression.rs` is the core to freeze. `src/simulation.rs`
and `src/judgement.rs` are two standards on it. An expression never mixes
the two: `Expression<simulation::Op>` and `Expression<judgement::Op>` are
different Rust types.

## expression.rs

```rust
enum Expression<Op> { Name(String),                                   // a name: bound by a let, or an input
    Apply(Op, Vec<Expression<Op>>),                                    // a constant: Apply(op, [])
    Let(String, Box<Expression<Op>>, Box<Expression<Op>>) }            // let x = e₁; e₂
trait Standard { type Type; type Op;                                   // a language
    fn gives(&self, op, takes: &[Type]) -> Option<Type>; }
trait Reading { type Standard; type Value; type Fault;                 // a meaning of it
    fn has_type(&self, v: &Value, of: &Type) -> bool;
    fn apply(&self, op, args: &[Outcome]) -> Outcome; }
enum Fault<F> { Op(F), Stuck }          type Outcome<V, F> = Result<V, Fault<F>>
fn check(s, Γ: &BTreeMap<String, Type>, e) -> Option<Type>             // Γ ⊢ e : T
fn eval(r, values: &BTreeMap<String, Value>, e) -> Outcome
fn fits(r, Γ, values) -> bool                                          // each input has a value of its type
fn strict(args) -> Result<Vec<Value>, Fault>                           // all values, or the first fault
```

**Typing judgement.** `Γ ⊢ Name(x) : Γ(x)`. If `Γ ⊢ eᵢ : Tᵢ` for each `i`
and `gives(op, [T₁ … Tₙ]) = T`, then `Γ ⊢ Apply(op, [e₁ … eₙ]) : T`. If
`Γ ⊢ e₁ : T₁` and `Γ, x : T₁ ⊢ e₂ : T₂`, then `Γ ⊢ Let(x, e₁, e₂) : T₂`:
`x` is in scope in `e₂` only, and the nearest `let` of a name shadows the
others and any input of that name.

**Let.** `Let(x, e₁, e₂)` is written as Rust's `let x = e₁; e₂`. It
evaluates `e₁` once and reads `e₂` with `x` standing for its outcome, fault
included. Unlike Rust, a fault of `e₁` is not raised at the `let`: it counts
only where `x` is read, as if `e₁` were written there. So a `let` never
changes what an expression gives: it shares.

**Readings are functions.** `has_type` and `apply` give the same answer for
the same arguments: a reading keeps no state that changes one. S4 and S6 rest
on it.

**Value typing.** `has_type(v, T)` belongs to the reading and reads what the
value is. Nothing stamps a type on a value, so preservation is not vacuous.

**Faults.** `Op(f)` is the reading's own "no value here" (a division by zero).
`Stuck` is evaluation going wrong (a missing input, an operation given values
it does not take). An operation sees each argument's outcome, fault included,
so `if`, `&&` and `||` drop the branch they do not take. They are ordinary
operations, not special forms. Every other operation is `strict`.

## Soundness statements

| | Statement | Tested by |
|---|---|---|
| S1 | **Agreement**, per reading and operation: `gives(op, ts) = Some(t)` and each argument is a value of its type or an `Op` fault ⇒ `apply` gives a value of type `t` or an `Op` fault. | depth-1 expressions over random inputs, every operation |
| S2 | **Preservation and progress**, from S1 by induction: `check(Γ, e) = Some(T)` and `fits(Γ, values)` ⇒ `eval(values, e)` is `Ok(v)` with `has_type(v, T)`, or `Err(Op(_))`, never `Err(Stuck)`. The induction keeps each name a `let` binds standing for a value of its type or an `Op` fault. | random well-typed expressions with lets, each reading |
| S3 | **Totality**: `check` and `eval` never panic, on any expression and values. They recurse once per level, except along a chain of `let`s each in the body of the one before, which takes no stack; the derived `Clone`, `PartialEq` and drop recurse on every level. So depth is bounded by the stack: on a 2 MiB thread, 1,500 levels in a debug build and 3,000 in release hold for all of them. Time and memory are not bounded by the size of an expression: each `let` may square a number. | random ill-typed expressions and values; 1,500 levels of `if`, of ponens and of `let`s in bound position; a chain of 10,000 `let`s |
| S4 | **Determinism**: `eval` is a function of reading, values and expression. | evaluating twice |
| S5 | **Covering**, per operation: `gives(op, ts) = Some(t)`, each bounded argument of its type or an `Op` fault, each exact argument in `γ` of its bounds at its type ⇒ `covers(t, Ranges.apply(op, b), Exact.apply(op, c))`. Then by induction: `check(Γ, e) = Some(T)`, `fits(Ranges, Γ, B)`, each exact input in `γ` of its bounds at its type ⇒ `covers(T, eval(Ranges, B, e), eval(Exact, C, e))`. On points it loses nothing: when each bound is `Bounds::exactly` of the exact input, the bounded outcome is `Bounds::exactly` of the exact one, or the same fault. | each operation with fault arguments; random expressions end to end; random expressions on points |
| S6 | **Sharing**: `eval(Let(x, e₁, e₂))` is `eval` of `e₂` with `e₁` written in place of `x`, on every reading, checked or not; and when `Let(x, e₁, e₂)` checks to `T`, so does that. `e₁` is evaluated once. | random expressions with lets and shadowing, written out, on every reading, typed and wild; a count of operations applied |
| J1 | **Rules preserve truth**: each set an opinion of `x` rests on, for or against, entails `x`, or `!x`, where `t = {p} s {q}` means `t → (∧p → ∧q)`. | every valuation of every random derivation |
| J2 | **State is read from rests**: `state(not o) = not state(o)`, `state(join a b) = join(state a, state b)`. | random opinions |
| J3 | **Rests are the smallest, and all**: no rest holds another. Keeping only the takings in a set `S`, a piece follows only if one of its rests lies in `S`, unless a `Reductio` lies below another; and, with no `Reductio`, whenever one does. A reductio's rest leaves out the taking it finds wrong. Below another reductio, a taking added can take a conclusion away: the inner one keeps only its smallest blame. | random derivations, random `S`; the case found by the red team |

## simulation.rs

- Types `Truth | Int | Real | Record { name, fields }`. Records are types of
  their own, so a record named "Real" is not `Real`.
- Values `Truth(bool) | Int(ℤ) | Real(ℚ) | Record(record, values)`. `Rational`
  is always in lowest terms, so equality is exact.
- Operations, in Rust's form: literals, `! && ||`, `+ - *` on two `Int` or two
  `Real`, `/` on `Real` (no value at 0), `< <=`, `==` on any one type,
  `within(a, b, tol)`, `if`, `R { .. }`, `r.x`.
- Readings: `Exact`, and `Ranges`, which works on bounds: a truth may be
  true, false or both, a number lies in `[low, high]` or anywhere, a record is
  bounded field by field, and any of them may fault. The γ is `covers(T, b,
  x)`: `x` is a value of type `T` that lies in `b`, or a fault where `b` may
  fault or is one. Bounds carry no type; the expression's type says which.

## judgement.rs

- Types `State | Opinion(Piece)`. A piece is `Named(n) | Not(x) |
  Implies { pre, subject, post }`, where the last is `t = {p} s {q}`.
- Values: a `State` (Belnap), or an `Opinion { about, rests_for,
  rests_against }`. The rests are the smallest sets of takings, where a taking
  is `{ piece, value, count, by }`. The state is read from which rests are
  empty. Counts stay on the takings; nothing reads them yet.
- The rules, whose types are the rules (premises of `t` = `t, p₁ … pₙ`):
  - `Ponens { t, post }`: opinions of every premise give one of `q`.
  - `Tollens { t, post, blame }`: `q`, then every premise but `blame`, give an opinion against premise `blame`.
  - `Reductio { of }`: an opinion of `x` gives one of `of`.
  - `Not` and `Join` on opinions; `And`, `Not` and `Join` on states; `StateOf` gives an opinion's state.
- Numbers sit only on takings, and no rule touches one.
- Costs: a ponens over `n` premises, each with two ways, rests on `2ⁿ` sets,
  each a different smallest way: inherent. A ponens copies its ways as they
  grow: quadratic in its premises (2,000 premises: 0.4 s). A reductio reads
  every pair of a way for and a way against.

## Choices taken on Rust's form

`!x`, `&&`, `||` short-circuit, `if` reads one branch, `R { .. }`, `r.x`,
`let x = e₁; e₂`. `within(a, b, tol)` stays as you named it, where Rust would
write `(a - b).abs() <= tol`. Unlike Rust, a fault in what a `let` binds
counts only where it is read (`let q = 1 / x; if x != 0 { q } else { 0 }`
has a value at 0). That keeps a `let` to sharing, and keeps bounds sound with
nothing new asked of a reading. No conversion is implicit: `Int` and `Real`
never mix, and a conversion would be an operation.

## Beyond what was named (going ahead; say no to undo)

1. Core: a **Standard / Reading** split, with constants as operations that take nothing. This lets one expression be read exactly and by bounds without copying it.
2. Core: **`Fault::Op` vs `Fault::Stuck`**, and operations see their arguments' faults, so `if` is not special.
3. **Int = ℤ, Real = ℚ, unbounded** (num-bigint). Nothing overflows. Machine numbers (i64, Int8) are not in this round.
4. **`/` on Real**, the one fault.
5. **`Piece::Not`**, with `not` on opinions (`!x`). There is no `and` on opinions, because a step's pre is already a conjunction.
6. **Taking `{ piece, value, count, by }`**. An opinion carries its piece (`about`), so its value says what it is about.
7. Removed after the red team: **bias**, and **`yes` / `no`** (the least count to give up). No rule reads either, and the least count is a smallest hitting set: exponential, and it crashed on 20,000 takings. Numbers come back when the logic is settled.

Not in this round: an `Option` type, `Int → Real` conversion, units, diagnosis.
