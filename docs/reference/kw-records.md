# Keywords: records and enums

Part of the [language reference](language.md); index in
[keywords.md](keywords.md). Every block fenced `mitra` is a whole model,
`evidence` is evidence for it, `text` is what `mitra eval` prints for
the two. A test runs them all.

## record

A value with named fields. Written `Point(1, 2)`, read `p.x`. Global,
also when written inside a subject. Type parameters: `record Box<T> {
item: T }`.

```mitra
record Point { x: 0..=9, y: 0..=9 }
fn flip(p: Point) -> Point { Point(p.y, p.x) }
claim flips() { flip(Point(1, 2)).x == 2 }
```
```text
flips: holds

```

## enum

A value that is one of several variants, each with or without fields.
Unnamed fields are named `v0`, `v1`. Read with
[match](kw-expressions.md#match) and [is](kw-expressions.md#is).

```mitra
enum Shape { Dot, Circle(r: 1..=9), Rect(1..=9, 1..=9) }
fn area(s: Shape) -> Int {
    match s { Dot => 0, Circle(r) => 3 * r * r, Rect(w, h) => w * h }
}
claim areas() { area(Rect(2, 3)) == 6 && area(Dot) == 0 && Circle(1).r == 1 }
```
```text
areas: holds

```
