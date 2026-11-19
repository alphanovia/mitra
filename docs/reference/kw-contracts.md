# Keywords: contracts

Part of the [keyword reference](keywords.md). A contract is a role
several pieces can fill: signatures, and what every piece that fills it
owes. Why it is shaped this way: [design/contracts.md](../design/contracts.md).

## contract

`contract Name { … }` holds `fn` signatures, `ensure` lines and claims,
and nothing else. It is written outside any block. Its functions are
named `Name::f` and its promises `Name::promise`. They are nobody's
code: no tie is asked for them, and a function with a body is refused
(no default method). `&self`, `&mut self` and `self` are any value.

With nothing to fill it, a promise of a contract is assumed on its own
word, as a promise of a boundary is.

```mitra
contract Link {
    fn send(&self, bytes: List<Int>) -> Bool;
    ensure ordered = "What arrives, arrives in the order it was sent.";
}
```
```text
Link::ordered: holds assuming [itself]
  says            "What arrives, arrives in the order it was sent."
```

## impl for

`impl Contract for Type { … }`, in a `subject` or a `boundary`, as Rust
writes it. The type is given each function of the contract
(`radio::Ble::send`), each promise (`radio::Ble::ordered`) and each
claim, as if written in an `impl Type { … }`
([kw-records.md](kw-records.md#impl)). Its braces hold what decomposes
the promises for this implementer and nothing else: `assume` lines in a
boundary, claims in a subject.

A contract is not built or bought: its implementers are. In a boundary
the functions are the world's and the claims of the contract are
assumptions; in a subject the functions are code to tie and the claims
are owed.

Each implementer is a group of its own. Its promises rest on its own
claims and assumptions, not on those of the block around it. The
promise of the contract rests on the same promise of every implementer.

```mitra
contract Link {
    fn send(&self, bytes: List<Int>) -> Bool;
    ensure ordered = "What arrives, arrives in the order it was sent.";
}
boundary radio {
    record Ble;
    record Wifi { channel: 1..=2 }
    impl Link for Ble { }
    impl Link for Wifi {
        assume wifi_sends_nothing(self) { self.send([]) }
    }
}
```
```text
radio::Ble::ordered: holds assuming [itself]
radio::Wifi::ordered: holds assuming [radio::Wifi::wifi_sends_nothing, radio::Wifi::ordered::captured]
  rests on        `radio::Wifi::wifi_sends_nothing`
Link::ordered: holds assuming [radio::Ble::ordered, radio::Wifi::wifi_sends_nothing, radio::Wifi::ordered::captured]
  requires        `radio::Ble::ordered`, `radio::Wifi::ordered`
```

Evidence names an implementer by naming its function. A claim written
in the contract is checked for each implementer apart, so a report says
which one broke the promise.

```mitra
contract Shape {
    fn area(&self) -> Int;
    ensure sized = "The area is never negative.";
    claim area_is_positive(self) { self.area() >= 0 }
}
subject shapes {
    record Square { side: 0..=2 }
    record Dot;
    impl Shape for Square { }
    impl Shape for Dot { }
}
```
```evidence
evidence a { for shapes::Square::area given self = shapes::Square(2) gives 4 }
evidence b { for shapes::Square::area given self = shapes::Square(1) gives -1 }
```
```text
shapes::Square::area_is_positive: fails
  fails on        `self = shapes::Square(1)`
shapes::Square::sized: fails
shapes::Dot::sized: holds assuming [shapes::Dot::area::correct, shapes::Dot::area_is_positive, shapes::Dot::sized::captured]
Shape::sized: fails
  fails with      `shapes::Square::sized`
```

What `mitra build` lists as owed follows: each implementer has its own
functions, claims and promises.

## impl Contract

`x: impl Contract` as the type of an input: any value. What the
function may lean on is what the contract promises, and a `require`
says so. The block is then judged against the contract, with every
implementer behind it.

A function of a contract is called as any function of the world is: a
spec cannot compute it, so the function that takes the value says
`uses Contract::f` ([kw-declarations.md](kw-declarations.md#uses)). A
method call `x.f()` on such a value is the function of the contract.

```mitra
contract Link {
    fn send(&self, bytes: List<Int>) -> Bool;
    ensure ordered = "What arrives, arrives in the order it was sent.";
}
boundary radio {
    record Ble;
    impl Link for Ble { }
}
subject sync {
    require transport = Link::ordered;
    ensure delivered = "Data pushed is received whole or reported lost.";
    fn push(l: impl Link, data: List<Int>) -> Bool uses Link::send;
}
```
```text
sync::transport: holds assuming [radio::Ble::ordered]
  needs           `Link::ordered`
sync::delivered: holds assuming [sync::push::correct, radio::Ble::ordered, sync::delivered::captured]
```

A block that states its intent and takes an `impl Contract` without a
`require` on a promise of that contract gets M106
([evaluation.md](evaluation.md)).

```mitra
contract Link {
    fn send(&self, bytes: List<Int>) -> Bool;
    ensure ordered = "What arrives, arrives in the order it was sent.";
}
subject sync {
    ensure delivered = "Data pushed is received whole or reported lost.";
    fn push(l: impl Link, data: List<Int>) -> Bool;
}
```
```text
`sync` takes a value bound by `Link`, and no `require` in it names a promise of `Link` (`Link::ordered`).
```

## Not in the language

- No associated types, no default methods, no trait objects, no blanket
  impls. Type parameters and bounds: [kw-generics.md](kw-generics.md).
- A contract and its implementers are in one store: a contract is not
  brought from another store by `import`.
- No verdict per implementer on the user of a contract: `sync::delivered`
  has one verdict. An observation on `sync::push` cannot say which link
  it was given.

## In Rust

Each method of `impl Trait for Type` is tied by its own comment,
`// mitra: implements radio::Ble::send`, like any method
([evidence.md](evidence.md)). The trait is not tied, and nothing checks that
the Rust `impl` is of that trait.
