# Cases kept aside

Nine engine tests that the language can't state. They are in the first
syntax, which nothing reads any more, with the result they had. Kept so
that what they tested is not lost; see `docs/review/2026-11-18.md`,
increment 36.

| Case | What it tested | Why it is aside |
|---|---|---|
| `compose/01`, `03`, `07`, `minus/01`, `03`, `07` | A claim `:= by derive [p, q]`: it `holds assuming` its unverified premises when Mitra checks that they imply it | The language has no `derive`. A claim follows from the specs of the functions it calls (`examples/reworked/clock`), or from a vouched claim `f(…) == e` (`examples/reworked/vouched`), not from named premises |
| `minus/06` | A derivation Mitra can't check is taken as stated | The same |
| `uses/04` | An `only uses` claim nobody vouched for names no root cause | A `uses` clause is taken as declared: it always has evidence |
| `uses/07` | Two `uses` claims on one function, one not vouched for | A function has one `uses` clause |
