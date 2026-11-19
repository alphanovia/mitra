# Design: generics, with a contract as a bound

Status: **built** (2026-11-19, increment 72). The user asked for
Rust's grammar and meaning, "with a possible contract constraint on the
type". What is on main is in the reference
([kw-generics.md](../reference/kw-generics.md)); this page says why.

## What was there

`fn first<T>(xs: List<T>) -> T`, `record Pair<A, B> { … }` and `enum
Tree<T> { … }` were read already: each parameter is any value. No
bound, no `impl<T>`, no `where`, no parameter on a contract.

## What is added, as Rust writes it

```
contract Link { fn send(&self, bytes: List<Int>) -> Bool; }
contract Store<K, V> { fn get(&self, key: K) -> Option<V>; }

subject sync {
    fn push<L: Link>(l: L, data: List<Int>) -> Bool uses Link::send;
    fn pull<L>(l: L) -> List<Int> where L: Link;
    record Stack<T> { items: List<T> }
    impl<T> Stack<T> { fn top(&self) -> Option<T>; }
    record Cache;
    impl Store<Int, String> for Cache { }
}
```

- A bound, `<L: Link>`, `<L: A + B>` or `where L: Link`, names
  contracts and nothing else.
- `x: impl Link` is the same as a parameter bound by `Link`, as in
  Rust.
- `impl<T> Stack<T> { … }`: each function of it has the parameter, and
  `self` is a `Stack<T>`. `impl<T> Contract for Stack<T> { … }` too.
- `contract Store<K, V> { … }`: an implementer says which types,
  `impl Store<Int, String> for Cache`, and is given the functions with
  them.

## What it means

Inside its function a type parameter is opaque, known by the contracts
that bound it; the user settled it (2026-11-19): "T opaque record yes".
It is the type `record T;` already was: nothing stated. The type
system gains no new kind of type. For a caller it is any value. A bound
says what the function leans on, exactly as `impl Contract` does: the
block is judged against the contract, and a `require` on one of its
promises is expected (M106 without).

The instance is chosen at the call, as in Rust: nothing is written for
it, and any value fits.

## Evidence

A test runs at one instance. It is recorded as what it is, a point: an
output for inputs of that instance. The report lists the instances
seen ("Instance list yes"). Nothing lets one instance stand for the
others. An implied piece `f::parametric`, the assumption that
would, is not built: every input type that holds a type parameter is
infinite today, so no finite evidence covers it with or without that
assumption. It would matter once a domain can be finite up to the
choice of `T`.

## Kept out

No specialisation, no macros, no associated types, no const generics,
no lifetimes, no higher-ranked bounds, no trait objects. No parameter
on a `subject` or a `boundary`. No bound on a `record` or an `enum`.

## Chosen alone, the smaller option each time

Numbered in [the review](../review/2026-11-18.md), increments 72 to 74.

- What a generic function gives back is any value to the reader, so
  the rule is not followed through a call. A claim of an `impl<T>`
  reads `T` as any value.
- A trait that bounds an input in the code alone is a detail of the
  code: noted by the tie, no difference. The user: "any T doesn't mean
  literally any T, it means any suitable T." A contract of the model
  the code does not bound by is a difference; traits are matched to
  contracts by name.
- No bound on a `record` or an `enum`, no parameter on a `subject` or a
  `boundary`.
