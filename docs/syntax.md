# The text of a model on core2

`src/parse.rs` reads it, `src/print.rs` writes it, and `parse(print(m))`
is `m`, ties aside (below). It is the data of `docs/model.md`, written
down: nothing here adds to the model. The models of `models/` are in it:
`fall.mitra` (the stone and the sun) and `div.mitra` (main's div).

Spellings follow Rust. Where a choice is not Rust's, or Rust has no
form for it, it is **flagged**: a proposal, yours to change.

A keyword added to or taken from the parser changes the VS Code grammar
in the same commit (`editors/vscode/README.md`); `tests/editor_grammar.rs`
fails when they drift.

## A whole model

```text
environment sun {
    fn light() -> w: rational;                                 // opaque
    ensure shines: "at least 200 W/m² at noon";
}

subject stone {
    require light: sun::shines;                           // from an ensure
    require air: "still air";                             // in words
    record Drop at "src/drop.rs/Drop" { h: rational, v: rational } where {
        claim up: 0.0 <= self.h;
    }
    fn drop(h: rational) -> v: rational at "lab/tower/stone" where {
        claim falls: within(v * v, 2.0 * 9.81 * h, 2.0);
        rule slows: drop::falls, speed::slow |- drop::slow;
        rule lands: drop::slows, drop::falls, speed::slow |- drop::slow
            by ponens(drop::slows, drop::falls, speed::slow);
    }
    fn speed(h: rational) -> v2: rational { 2.0 * 9.81 * h } where {
        claim slow: !(h <= 20.0) || v2 < 400.0;
        examples { (5.0) -> 98.1 }
    }
    ensure lands_slow: "lands slowly" from (speed::slow, drop::falls) | (air);
}
```

## Each form

| Form | Model | Rust? |
|---|---|---|
| `subject s { … }`, `environment e { … }` | `Part`, `Kind` | flagged (Rust has `mod`) |
| `require q: sun::shines;` / `require q: "words";` | `Require`, `Source` | flagged |
| `ensure e: "words" from (a, b) \| (c);` | `Ensure`, its lists | flagged |
| `record R;` opaque, `record R { x: rational }`, `record R {}` | `Record` | Rust's `struct` form, keyword flagged |
| `record Pair<T> { a: T, b: T }`, `enum Light { Red, Amber, Green }`, `enum Option<T> { None, Some(T) }` | `Record.params`, `Enum` | Rust's `struct` and `enum`; `T: Number` flagged |
| `Option<integer>`, `Pair<rational>` | a type given its parameters | Rust |
| `Some(x)`, `None`, `Light::Red`, `Option::Some(x)` | `Op::Variant` | Rust; a variant alone when one enum the part reads has it |
| `match o { Some(x) => x, None => 0 }`, `_ => …` for the rest | `Op::Match`, one arm per variant | Rust; no guards, no nested patterns, no `\|` in a pattern |
| `fn f(x: rational) -> y: rational;` opaque, `… { body }` | `Function` | Rust, but the named result is flagged |
| `where { claim …; rule …; examples { … } }` after the signature or body | claims, rules, examples | flagged |
| `claim c: expression;` | `Claim` | flagged: every naming form reads `name: …` (claim, require, ensure, rule, contract claim); `=` stays for `let x = e`, which binds a value, and for a port `path = value` in a model example row, which gives one |
| `rule r: p, q \|- c by proof;` | `Rule` | your spelling |
| `{ expression }` as a term of a rule | `Term::Holds` | Rust's block, flagged |
| `examples { (6, 2) -> 3, (7, 0) -> 0 }` | `Example` rows | flagged |
| `at "a/b/c"` after a name, `at "x", "y"` for two | `Tie` | your choice: in the block |
| `tie a::b at "x";` at the end | a tie to what has no block | flagged |
| `bool`, `integer`, `rational`, `(integer, bool)`, `(rational,)`, `()`, `R` | `Type` | Rust, but `integer`/`rational` flagged (Haskell's `Integer`, `Rational`): built-in primitives lowercase, parts and data types CamelCase; machine types later with Rust's names (`i64`, `u8`, `f64`) |
| `[integer]`, `[[rational]]`, `[Option<integer>]` | `Type::Seq`: a sequence, its length its own, never changed | Rust's slice form `[T]`, not `Vec<T>` |
| `[1, 2, 3]`, `[]`, `xs.len()`, `xs[i]`, `xs == ys` | `Op::Seq`, `Len`, `Index`, `Equal` | Rust; `xs[i]` out of range has no value, as Rust's panics |
| `[~char]`, `char`, `byte` | `Type::Encoded(Char)`: text, an encoded sequence of letters; `Type::Char`, `Type::Byte` | `[~T]` flagged; only `[~char]` today |
| `"abc"`, `s.len()`, `s[i]`, `s.decoded()` | `Op::Text`, `Len` and `Index` in bytes, `Decoded` | Rust's `str`: length and places in bytes |
| `a ++ b`, `a.starts_with(b)` | `Op::Join`, `Op::StartsWith`, on two texts or two sequences | `starts_with` Rust's; `++` flagged (Haskell's) |
| `fn f(a: 0..=9) -> q: 0..=9` | `Function.ranges`: an `integer` that is checked | Rust's range form; as a type Ada's `range 0 .. 9`, flagged |
| `true`, `3`, `-3`, `9.81`, `1/3` | constants | Rust, but `1/3` flagged |
| `Location { depth: 2, .. }` in a row | a value known in part (`Value::Partial`) | Rust's struct pattern, used for a value, flagged |
| `! && \|\| + - * / % < <= == within(a, b, tol)` | the operations | Rust (`within` is a call); `/` on two `integer` rounds toward zero, `%` on `integer` only |
| `a > b`, `a >= b`, `a != b` | `b < a`, `b <= a`, `!(a == b)` | Rust, read only as that |
| `if c { a } else { b }`, `else if` | `If` | Rust |
| `R { x: 1.0, y: 2.0 }`, `(a, b)`, `r.x`, `t.0` | `Make`, `Field` | Rust |
| `let x = a in b` | `Let` | your choice, not Rust's `;` |
| `(lo..hi).all(\|x\| p)` | `All` | Rust |
| `ponens(r, p, q)`, `ponens[t, 1](…)`, `tollens[t, 0, 2](…)`, `reductio[p](…)`, `cases[q](…)`, `instance[q](o, point(3, 4))`, `witness[q](o, p)`, `intro[q](|p| …)`, `point(3, 4)`, `join(a, b)`, `!a`, `let x = a in b` | a proof, `judgement::Op` | flagged |
| `all x in (a: 0..10, b: 0..10) where b != 0: p`, `any …`, `!p`, `{p, q} s {r}`, `correct(f, (6, 2))`, `correct(f, x)`, `correct(f, x in (a: 0..10))` | `Prop` | flagged |
| `use sun::{light, Ray};`, `use sun::light;`, `use sun::*;` in a part | `Use` | Rust |
| `use std::core::Option;`: Mitra's standard model (`std/core.mitra`, in the binary: `Option<T>`, `Result<T, E>`) | `Use` of the part `core`, `Part.file` | Rust's path; a std file holds one part (flagged) |
| a part used and not declared: read from `<part>.mitra` beside the model | `Part.file` | Rust's `mod` file rule, without `mod` (flagged) |
| `light(h)`, `sun::light(h)` | `Op::Call("sun::light")` | Rust |
| `contract Monotone<T: Number> { fn f(x: T) -> y: T; claim rises(a: T, b: T): …; }` | `Contract`, `Law` | Rust's trait, keyword and `Number` flagged |
| `impl Monotone<rational> for speed;`, `impl Pair<rational> for (f, g);` | `Implements` | Rust's form, `for (f, g)` flagged |
| `examples { (sun::light::w = 850.0, stone::speed::h = 5.0) -> stone::speed::v2 = 98.1 }` at the top | `ModelExample` | flagged |
| a rule term `all …`, `any …`, `!p`, `correct(f, …)`, `(p)` | `Term::Prop` | flagged |
| `// …` | nothing: comments are not kept | Rust |

## The choices, in plain words

**1. `let x = a in b`** (your choice). *What*: name a value inside one
expression. *Example*: `claim c: let d = v - w in d * d < 1.0;`.
*Gain*: one expression, no statements: what follows `in` is the whole
scope. *Cost*: Rust writes `{ let d = v - w; d * d < 1.0 }`, so a Rust
reader meets a new word. *Elsewhere*: OCaml, Haskell and F# write `let … in`.

**2. A tie in the block of what it ties** (your choice). *What*: `at
"place"` after the name of a part, record or function. *Example*: `fn
drop(h: rational) -> v: rational at "lab/tower/stone" where { … }`. *Gain*: you
read where a function is next to what it says. *Cost*: a tie to
something with no block (a claim, a name of nothing) goes at the end,
`tie a::b at "x";`; and ties come back in the order of the blocks, not
as written (flagged). *Elsewhere*: Rust's `#[path = "…"]` on a module,
Java annotations.

**3. A named result, `-> v: rational`** (flagged). *What*: what a function
gives has a name. *Example*: `claim falls: within(v * v, …)` reads `v`.
*Gain*: claims read the result by name, as the model needs (`gives:
(String, Type)`). *Cost*: not Rust, where the result has no name.
*Elsewhere*: Go (`func f() (v float64)`), Ada/SPARK (`F'Result`),
Dafny (`returns (v: real)`).

**4. `where { … }` for what is said of a record or function**
(flagged). *What*: claims, rules and examples, in a block after the
signature, or after the body. *Example*: `fn speed(h: rational) -> v2: rational
{ 2.0 * 9.81 * h } where { claim slow: …; }`. *Gain*: the body stays
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

**8. `integer`, `rational` and `1/3`** (flagged). *What*: core2's numbers are
exact, so the types are named for maths, not for machines; `2` is an
integer and `2.0` a rational, as in Rust. A real no decimal writes is printed
`1/3`, no spaces: a constant, not a division (`1.0 / 3.0` is one).
*Gain*: every number comes back exactly. *Cost*: `1/3` and `1 / 3` mean
different things. *Elsewhere*: Lisp and Scheme write exact fractions
`1/3`; Rust has `i64` and `f64`.

**9. Proofs: an operation's parameters in `[…]`** (flagged). *What*:
`ponens[t, post](args)`, and `ponens(r, …)` when the rule is the first
argument and the post is 0, as model.md writes it. A premise written in
place is read by its place, `0`, `1`, …. A point is `point(6, 2)`; `intro[q](|p| …)` binds the point `p`, as a
Rust closure does. *Gain*: every judgement
operation has one spelling. *Cost*: brackets are not Rust's call form.
*Elsewhere*: Rust's turbofish `f::<T>(x)` gives parameters apart from
arguments, as do Lean's `@f T x`.

**10. Quantifiers: `all x in (a: 0..10, b: 0..10) where b != 0: p`**
(flagged). *What*: as `judgement::Prop::All`: a name for the point, the
ranges (`hi` left out, as Rust's `0..10`), the condition, the body.
Inside an intro, a proposition about the point it binds, any point of
the set, writes that set: `correct(div, x in (a: 0..10, b: 0..10))`
(flagged: long, but it reads alone, with no scope to follow).
*Gain*: reads as maths. *Cost*: only in a proof's parameters and in a
store of reviews today: a rule cannot name one until the model has it.
*Elsewhere*: Lean `∀ x ∈ s, p x`; Dafny `forall x | 0 <= x < 10 :: p`.

**11. Calls, uses and contracts** (calls and `use` Rust's). *What*: a
call names a function whole, `sun::light(h)`, or by its name alone,
`light(h)`, when the part has it or one `use` reads it; in a contract,
its own functions by name. `impl C<rational> for f;` says `f` fits `C`; with
several functions, `for (f, g)`, in the contract's order (flagged: Rust
implements a trait for one type). *Gain*: what Rust writes. *Cost*: a
name two uses read must be written whole. *Elsewhere*: Rust, Python's
`from sun import light`.

**12. Examples of the model, and propositions in rules** (flagged).
`examples { (a::f::x = 1, …) -> b::g::y = 2 }` at the top level, each
port whole, several outputs in parentheses. A rule's term may be a
proposition, `all x in (a: 0..10): correct(div, x)`; one that would
read as a path or a block is put in parentheses, `(stone::air)`.

**13. Rust's own rules kept**. `if c { … }` takes no record literal in
`c` unless it is in parentheses. One comparison at most, `a < b < c`
refused. `-` only before a number: there is no negation of an
expression in core2. `a > b` is read as `b < a` and printed so.

**14. Enums, `match` and type parameters on records** (Rust's).
*What*: `enum Option<T> { None, Some(T) }` says a value is one of its
variants, each holding what it lists; `match o { Some(x) => x, None => 0
}` reads which one, an arm for each variant or `_` for the rest (P006).
`record Pair<T> { a: T, b: T }` names a type its fields use, given at
each use: `Pair<rational>`. `None` alone tells no `T`: Mitra reads it from
where it goes, a result, a field, an argument or the other side of `==`,
and never guesses (P009). *Gain*: a value that may be missing, or one of
a few cases, is said as Rust says it. *Cost*: no guards, no nested
patterns; an enum that holds itself (a list, a tree) waits (M027).
Differences from main, each Rust's form: `use`, not `import`; a variant
holds types in order, `Some(T)`, not named fields `Some(value: T)`; a
`match` that misses a variant is an error; a std file holds one part
(`subject core`). *Elsewhere*: Rust, Haskell's `data`, OCaml's variants, Swift's
`enum`.

**15. Sequences, `[T]`** (finding 3, step 2; your "ok", 20:24). *What*:
`[integer]` is a sequence of `integer`, of some length, written `[1, 2, 3]`;
`xs.len()` is its length, `xs[i]` the value at place `i` from 0, `xs ==
ys` compares place by place. `xs[i]` below 0 or at its length has no
value, as division by zero has none (Rust panics). `[]` alone tells no
`T`: as `None`, it is read from where it goes, else P009. *Gain*: a
list a function takes or gives is said, and a test row can write one
(`([3, 4]) -> 3`). *Cost*: only this minimal set: no `[v; n]`, `get`,
`push`, slices `xs[a..b]`, nor loops over a sequence (`fold`, `until`,
next). A record holding a sequence of itself is a tree, and waits
(M027). Agree tries sequences up to `Settings::length` long (3: empty,
one value, two or three to compare). The name is Rust's slice form, not
`Vec<T>`: a sequence is never changed, so a real `Vec` can be modelled
later. *Elsewhere*: Rust's `&[T]`, Haskell's and OCaml's lists, Lean's
`List`, Dafny's `seq<T>`.

**16. Text, `[~char]`** (finding 3, step 3; your "go"). *What*: `[~T]` is
an encoded sequence: a sequence of `T` kept as bytes. Text is
`[~char]`, written `"abc"`; a `char` is one letter as a reader sees it
(a grapheme), with no number. As Rust's `str`, `s.len()` counts bytes
and `s[i]` is the byte at place `i` (a `byte`, which has `==` only);
past the bytes it has no value. `a == b` is the same bytes, `a < b` and
`a <= b` compare byte by byte, which is the order of the letters' code
points. `a ++ b` joins and `a.starts_with(b)` reads a prefix, on two
texts or on two sequences of one type. `s.decoded()` gives the letters
as a `[char]`; on text it waits for the grapheme decoder, and today it
is check's error (M005, with a note); on a sequence `[T]` it is the
sequence itself. *Gain*: a name, a title, a path are said as text, with
the length a test sees; the model writes no `String` of its own. *Cost*:
no letter or byte is written yet (`'é'` later), no `type String =
[~char];`, no decoder (UAX #29, its Unicode version a setting), no
`from_bytes` nor `as_bytes`. Agree tries `""`, the texts the model
writes, each two joined, and their prefixes. `++` is Haskell's join
(`+` adds numbers only: `+` on two texts says to write `++`); `[~T]` is
ours. *Elsewhere*: Rust's `str` (bytes, `chars()`), Swift's `String`
(graphemes, `utf8` view), Lean's `String` (a list of `Char`), Dafny's
`string = seq<char>`.

**17. Lowercase primitives: `integer`, `rational`** (your "go", 10:21).
*What*: the built-in primitives are lowercase, `bool`, `char`, `byte`,
`integer` (a whole number of any size, was `Int`) and `rational` (an
exact fraction, was `Real`); a part or a data type is CamelCase. *Gain*:
a primitive reads apart from a type of the model, and the names say
what the numbers are: exact, not a machine's. *Cost*: two words to
write in full. A real number (`real`) and machine types (`i64`, `u8`,
`f64`, Rust's names) come later. *Elsewhere*: Haskell's `Integer` and
`Rational`, Lean's `Int` and `Rat`, Rust's lowercase primitives.

## Errors

A problem in the text is `error[P…]`, suit `P` of the catalogue
(flagged), at line and column, as rustc writes it; only the first is
given. `check` then finds what is wrong with the model read.

```text
error[P003]: expected `;`, found `}`
 --> sun.mitra:2:27
  |
2 |     fn light() -> w: rational }
  |                           ^
```

## What cannot be written

- A name that is not a word (M001), or a keyword (`subject`, `fn`,
  `at`, `in`, `crate`, …, `parse::KEYWORDS`).
- A proposition named by a tie, `stone::drop at lab/tower`, in a proof.
- An operation applied to other arguments than it takes, printed
  `__malformed__`, which does not read back.
- A location with a `/` inside one of its names.
- A text nested deeper than `parse::Settings::depth` levels (P011).
- In a proof, `state(a)` or `a && b`: what is known of an opinion is
  judgement's own (`judgement::Op::StateOf`, `And`), used by explain and
  numbers; no proof could give an opinion through it (finding 27).
