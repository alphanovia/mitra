# 7. Contracts and use

From [examples/language/contracts.mitra](../../examples/language/contracts.mitra).

## Contracts

A **contract** says what several functions promise alike, as a Rust
trait. Its claims are written once, and hold of every function that
fits it.

```mitra
contract Monotone<T: Number> {
    fn f(x: T) -> y: T;
    claim rises(a: T, b: T): !(a <= b) || f(a) <= f(b);
}
```

`impl` says a function fits it:

```mitra
    impl Monotone<rational> for with_tax;
```

`check` then reads `rises` of `with_tax` (A005 when the function breaks
it). `T: Number` means `T` is `integer` or `rational`.

A part whose methods fulfil a contract says so with `impl Name;`
inside it: see [examples/assistant/assistant.mitra](../../examples/assistant/assistant.mitra).

## Use

A part reads another part's functions and records through `use`, as in
Rust:

```mitra
subject Shop {
    use Prices::with_tax;
```

`use Prices::{with_tax, or_zero};` names several, `use Prices::*;` all,
`use Prices;` the whole part. A part needs no `use` for its own names, nor
for a part's type. Without a `use`, a name from another part is written
whole: `Prices::with_tax(a)`.

A part used and not declared in the model's file is read from the files
of the project that declare it. The project is every `.mitra` file under
its root: the folder of `mitra.toml`, up from the model's folder, else
the root of the git work tree, else that folder; what the root's
`.gitignore` leaves out is not read. A folder is a name space: its files
may each hold a piece of one part. When two folders declare a part,
`from` names the one meant, by the end of its path, whole names only:

```text
    use Store::get from "memory";        // memory/store.mitra
    use Store::get from "memory/store";  // the same file
```

None found is P010; two folders and no `from`, P014.

## The standard model

`std` is Mitra's own small model, shipped in the binary, one more folder
of the project. Today it holds two data parts, as in Rust's core library:

```text
subject enum Option<T> { None, Some(T) }
subject enum Result<T, E> { Ok(T), Err(E) }
```

```mitra
    use Option from "std";
```

```mitra
    fn or_zero(p: Option<rational>) -> q: rational {
        match p { Some(x) => x, None => 0.0 }
    } where {
        examples { (Some(3.0)) -> 3.0, (None) -> 0.0 }
    }
```
