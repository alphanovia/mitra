# Engine tests

Each folder is a case: `model.mitra`, `evidence.mitra` and the
`expected.txt` the engine must produce. They are written in the
**internal form** a model is translated to before it is evaluated. It
is not the Mitra language and the tool refuses it; the language and its
examples are in [`docs/reference/reworked.md`](../../docs/reference/reworked.md)
and [`examples/reworked/`](../../examples/reworked).

`MITRA_RECORD=1 cargo test -p mitra-core --test golden` records the
outputs again.
