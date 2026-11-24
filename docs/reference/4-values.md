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

`(a, b)` is a tuple, read with `t.0`, `t.1`. There is no tuple of one
value: `(x)` is just `x`.

A record has named fields:

```mitra
subject record Point { x: rational, y: rational }
```

Built with `Point { x: 3.0, y: 4.0 }`, read with `p.x`, changed into a
new value with `Point { x: 0.0, ..p }`.

## `let` and `if`

```mitra
    fn far(p: Point) -> d: rational {
        let dx = p.x in let dy = p.y in dx * dx + dy * dy
    } where {
```

`let x = e in body` names a value inside one expression; what follows
`in` is its whole scope. This is OCaml's form, not Rust's `let x = e;`,
because a Mitra body is one expression.

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

A sequence never changes. Loops over sequences (`fold`) are coming.

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
`within(a, b, tol)` is true when `a` and `b` differ by at most `tol`.
