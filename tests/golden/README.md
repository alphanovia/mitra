# Engine tests

Each folder is a case: `model.mitra`, `evidence.mitra` and the
`expected.txt` the tool must print. They are written in the Mitra
language ([`docs/reference/reworked.md`](../../docs/reference/reworked.md)).

`MITRA_RECORD=1 cargo test -p mitra-core --test golden` records the
outputs again.

Nine cases whose meaning the language cannot keep are in
[`tests/aside/`](../aside/README.md), with what each tested.
