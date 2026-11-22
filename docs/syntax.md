# The text of a model on core2

`src/parse.rs` reads it, `src/print.rs` writes it, and `parse(print(m))`
is `m`, ties aside (below). It is the data of `docs/model.md`, written
down: nothing here adds to the model. The models of `models/` are in it:
`fall.mitra` (the stone and the sun) and `div.mitra` (main's div).

Spellings follow Rust. Where a choice is not Rust's, or Rust has no
form for it, it is **flagged**: a proposal, yours to change.

## A whole model

```text
environment sun {
    fn light() -> w: Real;                                 // opaque
    ensure shines = "at least 200 W/m² at noon";
}

subject stone {
    require light = sun::shines;                           // from an ensure
    require air = "still air";                             // in words
    record Drop at "src/drop.rs/Drop" { h: Real, v: Real } where {
        claim up = 0.0 <= self.h;
    }
    fn drop(h: Real) -> v: Real at "lab/tower/stone" where {
        claim falls = within(v * v, 2.0 * 9.81 * h, 2.0);
        rule slows: drop::falls, speed::slow |- drop::slow;
        rule lands: drop::slows, drop::falls, speed::slow |- drop::slow
            by ponens(drop::slows, drop::falls, speed::slow);
    }
    fn speed(h: Real) -> v2: Real { 2.0 * 9.81 * h } where {
        claim slow = !(h <= 20.0) || v2 < 400.0;
        examples { (5.0) -> 98.1 }
    }
    ensure lands_slow = "lands slowly" from (speed::slow, drop::falls) | (air);
}
```

## Each form

| Form | Model | Rust? |
|---|---|---|
| `subject s { … }`, `environment e { … }` | `Part`, `Kind` | flagged (Rust has `mod`) |
| `require q = sun::shines;` / `= "words";` | `Require`, `Source` | flagged |
| `ensure e = "words" from (a, b) \| (c);` | `Ensure`, its lists | flagged |
| `record R;` opaque, `record R { x: Real }`, `record R {}` | `Record` | Rust's `struct` form, keyword flagged |
| `fn f(x: Real) -> y: Real;` opaque, `… { body }` | `Function` | Rust, but the named result is flagged |
| `where { claim …; rule …; examples { … } }` after the signature or body | claims, rules, examples | flagged |
| `claim c = expression;` | `Claim` | flagged |
| `rule r: p, q \|- c by proof;` | `Rule` | your spelling |
| `{ expression }` as a term of a rule | `Term::Holds` | Rust's block, flagged |
| `examples { (6, 2) -> 3, (7, 0) -> 0 }` | `Example` rows | flagged |
| `at "a/b/c"` after a name, `at "x", "y"` for two | `Tie` | your choice: in the block |
| `tie a::b at "x";` at the end | a tie to what has no block | flagged |
| `bool`, `Int`, `Real`, `(Int, bool)`, `(Real,)`, `()`, `R` | `Type` | Rust, but `Int`/`Real` flagged |
| `true`, `3`, `-3`, `9.81`, `1/3` | constants | Rust, but `1/3` flagged |
| `! && \|\| + - * / < <= == within(a, b, tol)` | the operations | Rust (`within` is a call) |
| `a > b`, `a >= b`, `a != b` | `b < a`, `b <= a`, `!(a == b)` | Rust, read only as that |
| `if c { a } else { b }`, `else if` | `If` | Rust |
| `R { x: 1.0, y: 2.0 }`, `(a, b)`, `r.x`, `t.0` | `Make`, `Field` | Rust |
| `let x = a in b` | `Let` | your choice, not Rust's `;` |
| `(lo..hi).all(\|x\| p)` | `All` | Rust |
| `ponens(r, p, q)`, `ponens[t, 1](…)`, `tollens[t, 0, 2](…)`, `reductio[p](…)`, `cases[q](…)`, `instance[q, (3, 4)](…)`, `witness[q, (3, 4)](…)`, `join(a, b)`, `state(a)`, `!a`, `a && b`, `let x = a in b` | a proof, `judgement::Op` | flagged |
| `all x in (a: 0..10, b: 0..10) where b != 0: p`, `any …`, `!p`, `{p, q} s {r}`, `correct(f, (6, 2))`, `correct(f, x)` | `Prop` | flagged |
| `// …` | nothing: comments are not kept | Rust |

## The choices, in plain words

**1. `let x = a in b`** (your choice). *What*: name a value inside one
expression. *Example*: `claim c = let d = v - w in d * d < 1.0;`.
*Gain*: one expression, no statements: what follows `in` is the whole
scope. *Cost*: Rust writes `{ let d = v - w; d * d < 1.0 }`, so a Rust
reader meets a new word. *Elsewhere*: OCaml, Haskell and F# write `let … in`.

**2. A tie in the block of what it ties** (your choice). *What*: `at
"place"` after the name of a part, record or function. *Example*: `fn
drop(h: Real) -> v: Real at "lab/tower/stone" where { … }`. *Gain*: you
read where a function is next to what it says. *Cost*: a tie to
something with no block (a claim, a name of nothing) goes at the end,
`tie a::b at "x";`; and ties come back in the order of the blocks, not
as written (flagged). *Elsewhere*: Rust's `#[path = "…"]` on a module,
Java annotations.

**3. A named result, `-> v: Real`** (flagged). *What*: what a function
gives has a name. *Example*: `claim falls = within(v * v, …)` reads `v`.
*Gain*: claims read the result by name, as the model needs (`gives:
(String, Type)`). *Cost*: not Rust, where the result has no name.
*Elsewhere*: Go (`func f() (v float64)`), Ada/SPARK (`F'Result`),
Dafny (`returns (v: real)`).

**4. `where { … }` for what is said of a record or function**
(flagged). *What*: claims, rules and examples, in a block after the
signature, or after the body. *Example*: `fn speed(h: Real) -> v2: Real
{ 2.0 * 9.81 * h } where { claim slow = …; }`. *Gain*: the body stays
what Rust writes, and what is said of it is apart. *Cost*: Rust's
`where` is for bounds on types, before the body. *Elsewhere*: Haskell's
`where` after a definition; Dafny puts `ensures` between signature and
body.

**5. `rule r: p, q |- c by proof;`** (your spelling). *What*: premises,
turnstile, conclusion, then the proof if there is one. A term is a path,
or an expression in braces, `{ v < 21.0 }`, written in place (flagged:
Rust's block). *Gain*: the logic is read as in a textbook. *Cost*: `|-`
is not Rust. *Elsewhere*: the sequent of logic; Lean writes `theorem r
(hp : p) (hq : q) : c := proof`.

**6. Short paths, from the part** (flagged). *What*: inside a part, a
path whose first name is one of its records, functions, requires or
ensures goes from the part: `drop::falls` in `stone` is
`stone::drop::falls`. Anything else is whole: `sun::shines`. `crate::`
first makes a path whole when it would read as one from the part.
*Gain*: what model.md writes. *Cost*: a part and an item of one name
need `crate::`. *Elsewhere*: Rust resolves `self::` and `crate::` the same way.

**7. Examples are rows, always in parentheses** (flagged). *What*:
`examples { (6, 2) -> 3 }`: what the function takes, in order, then what
it gives. One argument is still `(5.0) -> 98.1`. *Gain*: one form
whatever the arity; a tuple argument is `((1, 2)) -> 3`. *Cost*: `(5.0)`
is not a tuple here, unlike everywhere else. *Elsewhere*: test tables
in Go and pytest's `parametrize` are rows of the same shape.

**8. `Int`, `Real` and `1/3`** (flagged). *What*: core2's numbers are
exact, so the types are named for maths, not for machines; `2` is an
Int and `2.0` a Real, as in Rust. A real no decimal writes is printed
`1/3`, no spaces: a constant, not a division (`1.0 / 3.0` is one).
*Gain*: every number comes back exactly. *Cost*: `1/3` and `1 / 3` mean
different things. *Elsewhere*: Lisp and Scheme write exact fractions
`1/3`; Rust has `i64` and `f64`.

**9. Proofs: an operation's parameters in `[…]`** (flagged). *What*:
`ponens[t, post](args)`, and `ponens(r, …)` when the rule is the first
argument and the post is 0, as model.md writes it. A premise written in
place is read by its place, `0`, `1`, …. *Gain*: every judgement
operation has one spelling. *Cost*: brackets are not Rust's call form.
*Elsewhere*: Rust's turbofish `f::<T>(x)` gives parameters apart from
arguments, as do Lean's `@f T x`.

**10. Quantifiers: `all x in (a: 0..10, b: 0..10) where b != 0: p`**
(flagged). *What*: as `judgement::Prop::All`: a name for the point, the
ranges (`hi` left out, as Rust's `0..10`), the condition, the body.
*Gain*: reads as maths. *Cost*: only in a proof's parameters and in a
store of reviews today: a rule cannot name one until the model has it.
*Elsewhere*: Lean `∀ x ∈ s, p x`; Dafny `forall x | 0 <= x < 10 :: p`.

**11. Rust's own rules kept**. `if c { … }` takes no record literal in
`c` unless it is in parentheses. One comparison at most, `a < b < c`
refused. `-` only before a number: there is no negation of an
expression in core2. `a > b` is read as `b < a` and printed so.

## Errors

A problem in the text is `error[P…]`, suit `P` of the catalogue
(flagged), at line and column, as rustc writes it; only the first is
given. `check` then finds what is wrong with the model read.

```text
error[P003]: expected `;`, found `}`
 --> sun.mitra:2:27
  |
2 |     fn light() -> w: Real }
  |                           ^
```

## What cannot be written

- A name that is not a word (M001), or a keyword (`subject`, `fn`,
  `at`, `in`, `crate`, …, `parse::KEYWORDS`).
- A proposition named by a tie, `stone::drop at lab/tower`, in a proof.
- An operation applied to other arguments than it takes, printed
  `__malformed__`, which does not read back.
- A location with a `/` inside one of its names.
