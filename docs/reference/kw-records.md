# Keywords: records and enums

Part of the [language reference](language.md); index in
[keywords.md](keywords.md). Every block fenced `mitra` is a whole model,
`evidence` is evidence for it, `text` is what `mitra eval` prints for
the two. A test runs them all.

## record

A value with named fields. Written `Point(1, 2)`, read `p.x`. Type
parameters: `record Box<T> { item: T }`. Written in a subject or a
boundary it belongs to it, like a function: `Point` inside, `geo::Point`
elsewhere, or `Point` alone where only one block has the name and
nothing outside any block does. The same for an enum and its variants.

```mitra
record Point { x: 0..=9, y: 0..=9 }
fn flip(p: Point) -> Point { Point(p.y, p.x) }
claim flips() { flip(Point(1, 2)).x == 2 }
```
```text
flips: holds

```

A record may leave its contents out, as a `fn` leaves out its body.
`record Model;` states nothing of it. `record Claim { name: String, .. }`
states the fields that matter and says there are others. Such a value
is passed around and its stated fields are read, but none can be
written: not in the model, not in evidence (`holds` or `fails` on the
whole function, no `given` on that input).

```mitra
record Model;
record Claim { name: String, .. }
subject check {
    fn ok(m: Model) -> Bool;
    fn named(c: Claim) -> String { c.name }
}
```
```evidence
evidence r { for check::ok holds source "review:me" }
```
```text
check::ok::correct: holds
check::named::spec: unverified
  no evidence on  c in Claim
```

A record is tied to its struct like a function to its code:
`// mitra: implements check::Claim` above the `struct`, see
[the tie](evidence.md#the-tie-to-the-code).

A claim on a record, calling no code, is true of every value the code
builds. An output observed on a function that returns the record is
evidence for it or against it; against, the code of that function is a
candidate with the claim and the evidence.

```mitra
subject geo {
    record Point { x: 0..=9, y: 0..=9 }
    fn origin() -> Point;
    claim on_the_diagonal(p: Point) { p.x == p.y }
}
```
```evidence
evidence seen { for geo::origin gives Point(0, 1) }
```
```text
geo::on_the_diagonal: fails
  fails on        `p = geo::Point(0, 1)`
```

## impl

`impl R { … }`, as Rust writes it, in the subject or boundary of what it
holds. A `fn` or a `claim` there is named `R::name`. `self` (also
`&self`, `&mut self`) is an input of type `R`, and `Self` is `R`. A
function taking `self` is a method, called `p.flip()` or
`Point::flip(p)`; one without is called `Point::new(0, 1)`. A claim on
`self` alone is a claim on the record, as above. Tied by
`// mitra: implements geo::Point::flip` above the method of the code.

```mitra
subject geo {
    record Point { x: 0..=9, y: 0..=9 }
    impl Point {
        fn new(x: 0..=9, y: 0..=9) -> Self;
        fn flip(&self) -> Self { Point(self.y, self.x) }
        claim flips_back(self) { self.flip().flip() == self }
        claim on_the_diagonal(self) { self.x == self.y }
    }
    fn mirror(p: Point) -> Point;
    claim mirror_flips(p: Point) { mirror(p) == p.flip() }
}
```
```evidence
evidence seen { for Point::new given x = 0, y = 1 gives Point(0, 1) }
```
```text
geo::Point::flips_back: holds assuming [geo::Point::flip::spec]
```
```text
geo::Point::on_the_diagonal: fails
  fails on        `self = geo::Point(0, 1)`
```

`p.name(…)` is the method of the record `p` is declared as; on any
other expression, the only method of that name.

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
