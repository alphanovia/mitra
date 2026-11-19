# Design: contracts

Status: **built** (2026-11-19, increment 71). The word `contract` is the
user's, over `trait`. What is on main is in
[the reference](../reference/kw-contracts.md); this page says why, and
what was chosen alone.

## What it is

A contract is a role several pieces can fill: signatures, and the
promises every piece that fills it owes. It is the only thing a user of
the role may lean on. It is neither built nor bought: a boundary can
fill a contract (a BLE or a WiFi link, neither written from the model)
as well as a subject.

```
contract Link {
    fn send(&self, bytes: List<Int>) -> Result<(), String>;
    fn receive(&self) -> Option<List<Int>>;
    ensure ordered = "What arrives, arrives in the order it was sent.";
}

boundary radio {
    record Ble;
    record Wifi;
    impl Link for Ble  { assume ble_keeps_order(l: Ble) { … } }
    impl Link for Wifi { }
}

subject sync {
    require transport = Link::ordered;
    ensure delivered = "Data pushed is received whole or reported lost.";
    fn push(l: impl Link, data: List<Int>) -> Result<(), String>
        uses Link::send;
}
```

Rust's shape is kept for `impl Contract for Type { }` and for `x: impl
Contract`; only the declaring keyword differs. Nothing in it is Rust's
alone: an interface in another language fills the same role.

## What it means

- `contract Link { … }` declares the functions `Link::send`,
  `Link::receive` and the promise `Link::ordered`. They belong to
  nobody's code: no tie is asked for them.
- `impl Link for Ble { … }` gives `Ble` each function of the contract
  (`radio::Ble::send`) and each promise (`radio::Ble::ordered`), as if
  written in an `impl Ble { … }`. Its braces hold what decomposes the
  promises for this implementer: `assume` lines in a boundary, claims
  in a subject. Nothing else.
- Each implementer is a group of its own: its promises rest on its own
  claims and assumptions, not on those of the block around it. So each
  is its own suspect, and evidence names the implementer by naming its
  function: `for radio::Ble::send given … gives …`.
- `Link::ordered` rests on the same promise of every implementer. With
  none it is assumed on its own word.
- `require transport = Link::ordered;` names it as any promise:
  `sync::transport: holds assuming [radio::Ble::ordered, …]`.
- `x: impl Link` takes any value. The function may only lean on what
  the contract promises: that is what the `require` says.

What `build` owes per implementer follows: each implementer has its own
functions and promises in the build file and in what is owed.

## Kept out

The user wants the type system to stop growing. No associated types, no
default methods (a function with a body in a contract is refused), no
trait objects, no blanket impls, no bound beyond `impl Contract` (a
bounded type parameter is the generics proposal).

## Chosen alone, the smaller option each time

Numbered in [the review](../review/2026-11-18.md), increment 71.

- A verdict on the user of a contract is not printed per implementer:
  `sync::delivered` has one verdict, and names the implementers'
  promises it assumes.
- An observation names its implementer through the implementer's own
  function. An observation on `sync::push` cannot say which link it was
  given: a value of `impl Link` cannot be written.
- The Rust tie: each method of `impl Trait for Type` is tied by its own
  comment, `// mitra: implements radio::Ble::send`. The trait itself is
  not tied, and nothing checks that the Rust `impl` is of that trait.
- A function of a contract is of the world as far as a spec goes: a
  body cannot call it, the function says `uses Link::send`. The
  contract may be filled by a boundary, and a spec never performs IO.
- A claim written in a contract is owed by each implementer (a claim
  in a subject, an assumption in a boundary). The contract itself keeps
  only its promises.
- A contract is brought from another store by `import store::Contract;`
  (increment 73). Mitra's own model uses it: `Reader` in mitra-lang,
  filled by mitra-rust, required by mitra-cli. The code has the same
  seam since increment 74: the trait `mitra_lang::Reader`, `impl` for
  `mitra_rust::Rust`.
- `impl Link` is read as `Any`: the type system gains nothing. The
  interface `mitra build` writes therefore prints `Any`.
