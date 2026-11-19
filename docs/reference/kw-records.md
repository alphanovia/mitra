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
`// mitra: implements Claim` above the `struct`, see
[the tie](evidence.md#the-tie-to-the-code).

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
