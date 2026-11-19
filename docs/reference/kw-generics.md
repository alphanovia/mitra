# Keywords: type parameters and bounds

Part of the [keyword reference](keywords.md). Rust's grammar, kept
narrow. Why: [design/generics.md](../design/generics.md).

A type parameter is any value, bound or not: `T` is read as `Any`
([kw-types.md](kw-types.md#type-parameters)). The instance is chosen at
the call and nothing is written for it. What a parameter adds is what a
reader and the tie to the code see: which inputs are the same type, and
which contract a value is known by.

## Bounds

`fn f<L: Link>(l: L)`, `<L: Link + Other>`, and `where L: Link` after
the result. A bound names contracts
([kw-contracts.md](kw-contracts.md)) and nothing else.
`l: impl Link` is the same thing with no name for the type, as in Rust.

A bound says what the function leans on. The block is judged against
the contract: a `require` on one of its promises is expected, and M106
says so when a block that states its intent has none.

```mitra
contract Link {
    fn send(&self, bytes: List<Int>) -> Bool;
    ensure ordered = "What arrives, arrives in the order it was sent.";
}
subject sync {
    require transport = Link::ordered;
    ensure delivered = "Data pushed is received whole or reported lost.";
    fn push<L: Link>(l: L, data: List<Int>) -> Bool uses Link::send;
    fn pull<L>(l: &L) -> List<Int> where L: Link;
}
```
```text
sync::transport: holds assuming [Link::ordered]
sync::delivered: holds assuming [sync::push::correct, sync::pull::correct, Link::ordered, sync::delivered::captured]
```

## impl with parameters

`impl<T> R<T> { … }`: each function has the parameters of the `impl`,
before its own, and `self` is an `R<T>`. `Self` is `R<T>`. A claim
written in it is over every `T`. `impl R<Int> { … }` gives functions
of one instance only. A bound may be written on the `impl`:
`impl<L: Link> R<L> { … }`.

```mitra
record Pair<A, B> { first: A, second: B }
impl<A, B> Pair<A, B> {
    fn swap(&self) -> Pair<B, A> { Pair(self.second, self.first) }
    fn with<C>(&self, third: C) -> Pair<A, C> { Pair(self.first, third) }
}
claim swaps() { Pair(1, "a").swap() == Pair("a", 1) && Pair(1, 2).with(true).second }
```
```text
swaps: holds
```

## Contracts with parameters

`contract Store<K, V> { … }`: each function of the contract has the
parameters. An implementer says which types,
`impl Store<Int, String> for Cache { }`, and is given the functions
with them; as many types as the contract has parameters.
`impl<T> Store<Int, T> for Shelf<T> { }` fills it for every `T`. In a
bound, `S: Store<Int, String>`, only the name of the contract is read.

```mitra
import std::rust::{Option};
contract Store<K, V> {
    fn get(&self, key: K) -> Option<V>;
    ensure kept = "What was put is what is got.";
}
subject cache {
    record Names { size: 0..=1 }
    impl Store<0..=1, String> for Names { }
}
```
```evidence
evidence a { for cache::Names::get given self = cache::Names(0), key = 1 gives None }
```
```text
cache::Names::get::correct: unverified
  no evidence on  (self, key) in (cache::Names, 0..=1)
Store::kept: holds assuming [cache::Names::get::correct, cache::Names::kept::captured]
  requires        `cache::Names::kept`
```

## Evidence at an instance

A test runs at one instance of `T`. It is recorded as what it is: an
output for those inputs. Nothing lets one instance stand for the
others.

```mitra
subject fn first<T>(xs: List<T>) -> T;
```
```evidence
evidence ints { for first given xs = [1, 2] gives 1 }
evidence words { for first given xs = ["a"] gives "a" }
```
```text
first::correct: unverified
  no evidence on  xs in List<Any>
```

## Not in the language

- A parameter is not opaque: a body may do with a `T` what it may do
  with any value, bound or not. The bound is what the block declares it
  leans on, not what the reader enforces.
- No bound on a `record` or an `enum`. No parameter on a `subject` or a
  `boundary`.
- No specialisation, no associated types, no const generics, no
  lifetimes, no higher-ranked bounds, no trait objects, no macros.
- No `f::parametric`: no assumption that one instance stands for all.

## In Rust

`mitra tie` compares, for each input the model binds by a contract, the
traits that bound it in the code: on its type parameter (`<L: Link>`,
`where L: Link`, or on the `impl`), or written `impl Link` or
`&dyn Link`. The code may bound it by more. A trait is matched to a
contract by name. `record R<T> { … }` is compared with its `struct`
field by field. The interface `mitra build` writes prints functions
with their parameters and bounds. A difference reads: ``input `l` is
bound by `Link` in the model, by `Clone` in the code``.
