# 4. Values

Mitra's expressions follow Rust's, with a few differences flagged below.
Every example here is from
[examples/language/values.mitra](../../examples/language/values.mitra).

## Built-in types

| Type | Values | Note |
|---|---|---|
| `bool` | `true`, `false` | |
| `integer` | `3`, `-3` | any size, exact |
| `rational` | `9.81`, `1/3` | an exact fraction, never rounded |
| `char`, `byte` | | one letter; one byte of text |
| `[~char]` | `"abc"` | text |
| `0..=9` | | an `integer` checked to be in a range |

No conversion is ever implicit: `2` is an `integer`, `2.0` a `rational`,
and `2 + 2.0` is an error. Machine types (`u32`, `i64`, `f64`) come
later; for now a model speaks of exact numbers.

## Tuples and records

`(a, b)` is a tuple, read with `t.0`, `t.1`. Its places may have
labels: `(sum: 3, n: 2)`, of type `(sum: integer, n: integer)`, read
with `t.sum` or `t.0`. There is no tuple of one value: `(x)` is just
`x`.

A record has named fields:

```mitra
subject record Point { x: rational, y: rational }
```

Built with `Point { x: 3.0, y: 4.0 }`, read with `p.x`, changed into a
new value with `Point { x: 0.0, ..p }`.

A record that ends with `..` has fields the model does not state: the
code holds more than the model says (a cache, a file handle).

```mitra
subject Thermostat {
    record { target: integer, on: bool, .. }

    fn heats(self) -> b: bool { self.on } where {
        examples { (Thermostat { target: 20, on: true, .. }) -> true }
    }
}
```

- No whole value of it is written: only one known in part, `Thermostat
  { target: 20, on: true, .. }`, in an example, a row or the store. A
  function that makes one has no body, or makes it from another,
  `Thermostat { on: false, ..t }`, which keeps the rest.
- Its stated fields are read as usual. `a == b` of two is not known:
  the fields not stated may differ.
- An example or a measurement is compared on the fields it states.
- `..` comes last. It is Rust's struct pattern, `Point { x, .. }`, used
  in a type.

## Blocks, `let` and `if`

```mitra
    fn far(p: Point) -> d: rational {
        let dx = p.x;
        let dy = p.y;
        dx * dx + dy * dy
    } where {
```

A block is Rust's: `{ let x = e; … last }`. Each `let x = e;` names a
value for the rest of the block, and the block gives its last
expression. A body is a block, and so is each branch of an `if`; any
other place writes one in braces: `claim c: { let d = v - w; d * d < 1.0 };`.

`e;` drops what `e` gives, which must be `()`: it is written for what it
does, a call to the environment. Dropping another value is an error
(P016), where Rust only warns for some types: name it with `let` to use
it. A block that ends with `;` gives `()`. `let x = e in b` is no longer
read; the error says how to write it.

`if c { a } else { b }` is Rust's, and always has an `else`.

## Enums and `match`

An enum is one of its variants, each holding values in order:

```mitra
subject enum Shape { Dot(Point), Square(rational) }
```

```mitra
    fn area(s: Shape) -> a: rational {
        match s { Dot(p) => 0.0, Square(side) => side * side }
    } where {
        examples { (Shape::Square(3.0)) -> 9.0 }
    }
```

A `match` has an arm for every variant, or `_` for the rest (P006). No
guards and no nested patterns yet. `Option<T>` and `Result<T, E>` come
from the standard model ([chapter 7](7-contracts-and-use.md)).

## Trees

An enum may hold itself, as long as one variant does not: that one
ends a value. Unlike Rust, no `Box` is needed, for a model has no
memory layout (as in Haskell, OCaml or Lean).

```mitra
subject enum Tree { Leaf, Node(Tree, integer, Tree) }
```

A function may call itself on a part a `match` takes apart from one of
its inputs, the same input for every call: each call is on a smaller
tree, so it ends (M019 for any other call to itself). A claim on such a
function can be shown `by induction` on that input:

```mitra
    fn size(t: Tree) -> n: integer {
        match t { Leaf => 0, Node(l, _, r) => size(l) + 1 + size(r) }
    } where {
        claim counted: 0 <= n by induction t;
        examples { (Leaf) -> 0, (Node(Leaf, 5, Node(Leaf, 6, Leaf))) -> 2 }
    }
```

`mitra check` then reads `size(l)` and `size(r)` as numbers the claim
already holds of (each `0 <= …`), and decides the claim for every
variant at once by bounds: `Leaf` gives 0, `Node` gives a sum of
numbers at 0 or more, plus 1. A claim it cannot decide this way is
unknown (A007), never held. It also tries small trees as points, up to
`[agree] height` levels (3 by default; [chapter 8](8-commands.md#mitratoml)).

## Sequences

`[integer]` is a sequence of integers, written `[7, 8]`. `xs.len()` is
its length and `xs[i]` its value at place `i`, from 0. Reading past
the end has no value, as Rust would panic.

```mitra
    fn first_or(xs: [integer], other: integer) -> y: integer {
        if xs.len() == 0 { other } else { xs[0] }
    } where {
        examples { ([7, 8], 0) -> 7, ([], 5) -> 5 }
    }
```

A sequence never changes.

## Loops

A loop carries values from one turn to the next, and gives them back
at the end. There are two loops.

`xs.fold(start) { x -> next }` goes over a sequence: for each `x`, the
block gives the next carried values. Carrying one value, it is named
and given back bare:

```mitra
    fn total(xs: [integer]) -> t: integer { xs.fold(sum: 0) { x -> sum + x } } where {
        examples { ([1, 2, 3]) -> 6, ([]) -> 0 }
    }
```

Carrying more, they are a tuple with labels, and the block gives the
next tuple in the same order:

```mitra
    fn mean(xs: [integer]) -> m: integer {
        let c = xs.fold((sum: 0, n: 0)) { x -> (sum + x, n + 1) };
        if c.n == 0 { 0 } else { c.sum / c.n }
    } where {
```

`until(start, done) { next }` is the while loop: it stops at the first
carried values where `done` holds.

```mitra
    fn halvings(n: integer) -> s: integer {
        until((k: n, steps: 0), k <= 1) { (k / 2, steps + 1) }.steps
    } where {
```

Mitra remembers what an `until` has carried. When a turn carries
values it had already carried, every one known, the loop can only go
round again: it stops at once with no value, and `check` says so in an
A013 finding that names the loop, the input and the values that came
back. For example, a loop that forgets to halve, `{ k }` in place of
`{ k / 2 }`:

```text
finding[A013]: example 0: fails, at n = 4, the `until` in `s::halve`
never ends: what it carries, 4, came back after 1 turn
```

An example fails so; a claim that reaches the loop is unknown.

A loop whose values never repeat, such as one that counts its turns
while it waits, is stopped by Mitra's step budget (1,000,000 turns):
an example fails there (A016), and a claim is unknown (A007). A block
of the wrong type is M005, with a note on what the loop carries.

### Claims kept by a loop

A claim of a function whose body is a loop can be shown `by invariant`:
it holds where the loop starts, and each turn keeps it, so it holds at
the end. From
[examples/language/invariants.mitra](../../examples/language/invariants.mitra):

```mitra
    fn squares(xs: [integer]) -> s: integer {
        xs.fold(s: 0) { x -> s + x * x }
    } where {
        claim positive: 0 <= s by invariant;
```

Many claims are not true at the start: `c == xs.len()` is false while
`c` is 0 and nothing is counted. `by invariant n` names the turns: `n`
is 0 at the start, one more each turn, and the length of the sequence
at the end, so this claim says `c == xs.len()`:

```mitra
    fn count(xs: [integer]) -> c: integer {
        xs.fold(c: 0) { x -> c + 1 }
    } where {
        claim counted: c == n by invariant n;
```

`n` is the claim's own name for the loop's counter, the `i` of a `for` loop. Only a
`fold` has it: an `until` has no count to end at. A claim `by invariant`
of a function whose body is no loop is M033. What the start and a turn
do not decide stays unknown (A007), and the finding says which one.

Rust has no invariants; Dafny writes them with `invariant` on the loop.

## Text

Text is `[~char]`: a sequence of letters, kept as bytes, like Rust's
`str`. `s.len()` counts bytes. `a ++ b` joins two texts (`+` is only
for numbers), and `a.starts_with(b)` reads a prefix.

```mitra
    fn greet(name: [~char]) -> g: [~char] { "hello " ++ name } where {
        examples { ("ana") -> "hello ana" }
    }
```

## Operators

`! && || + - * / % == != < <= > >=`, as Rust. `/` on two integers
rounds toward zero, `%` is on integers only, and division by zero has
no value (a claim with no value does not hold). `a < b < c` is refused.
`within(a, b, tol)` is true when `a` and `b` differ by at most `tol`. A
model function named `within` comes first ([chapter 2](2-parts.md#kinds-of-names)).
