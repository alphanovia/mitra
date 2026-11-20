# Several stores

A project may hold several stores, one per crate or package, each with
its own `.mitra/` folder, model and ledger. A store may lean on what
another one declares, without restating it.

## Bringing from another store

```
import std::rust::{Vec};
import mitra_lang::results::{Outcome, a_failure_is_kept};

subject report {
    require failure_kept = results::a_failure_is_kept;
    fn libtest(text: String) -> Vec<results::Outcome>;
}
```

`import store::block::{names};` is Rust's `use crate_name::module::Item`
(decision 371). It is read when no file `store/block.mitra` sits beside
the model:

- `store` is a folder beside the folder of this store which holds a
  `.mitra/model.mitra`. `mitra_lang` finds `mitra_lang/` or `mitra-lang/`.
- `block` is a `subject` or a `boundary` of that store, in any of its
  files.
- each name is a `record`, an `enum` or an `ensure` of that block.

What comes is a `boundary` of the same name holding those declarations
alone. Another store is bought here, not built:

- a promise brought is assumed, as any promise of a boundary:
  `results::a_failure_is_kept: holds assuming [itself]`. Whether it
  holds is judged in its own store, by its own ledger. `require` then
  names it as it names any promise.
- a record or an enum brought is a type of this model, written
  `results::Outcome`, or `Outcome` where no other block has that name.
  Its code is tied in its own store: `mitra tie` does not ask for it
  here.
- the functions, claims and requires of the block stay there.

A record whose fields name a type of that store which is not brought
comes without its fields, as `record Model;`. An enum in the same case
is refused: bring the type it holds with it.

## What is refused

| Written | Said |
|---|---|
| a name that is no record, enum or `ensure` of the block | ``its block `results` has no record, enum or `ensure` named `first` `` |
| a block that store does not have | ``that store has no block `nowhere` `` |
| a block named as a block of this model | ``is a block of this model too`` |
| one block name brought from two stores | ``was brought from `there` already`` |

## Not done

- Nothing checks that the other store's promise holds there: its ledger
  is not read. Run `mitra eval` in each store.
- No renaming: two stores with a block of the same name cannot both be
  brought from, and neither can a block named like one of this model.
- A store is found beside this one only; no path, no registry.

## Bringing a contract

```
import mitra_lang::Reader;
import mitra_lang::results::{Outcome, Tied};

subject reader {
    record Rust;
    impl Reader<Path> for Rust { … }
}
```

`import store::Contract;` (or `import store::{A, B};`) brings a
`contract` of the model of another store, whole: its functions, its
promises and its claims ([kw-contracts.md](kw-contracts.md)). It is
then filled here, leaned on here, or both.

- Filled here, its promise rests on the implementers of this store.
  Leaned on and not filled, it is assumed, as any promise of another
  store: `Reader::reads_only: holds assuming [itself]`.
- A type the contract names, `results::Outcome`, is the one brought by
  `import store::results::{Outcome};`. Not brought, it is any value.
- The implementers of the other store do not come.

Mitra's own model uses it, and its code has the same seam: mitra-lang
states `Reader` (the trait `mitra_lang::Reader`), mitra-rust fills it
for Rust (`mitra_rust::Rust`), mitra-cli requires its promises and
calls a reader only through the trait.
