# The Mitra language reference

Read it in order: each chapter starts from what the one before taught.
Every example is taken from a file in [examples/](../../examples), and
a test checks that those files pass `mitra check`.

1. [The ideas](1-ideas.md): model, system, evidence; the three
   questions; verdicts and trust numbers.
2. [Parts](2-parts.md): subjects and environments, requires and
   ensures, parts inside parts, a part as a type.
3. [Functions and claims](3-functions.md): opaque functions and bodies,
   claims, examples, ranges.
4. [Values](4-values.md): numbers, records, enums and `match`,
   sequences, text, loops, trees, `let` and `if`.
5. [Ties and evidence](5-ties-and-evidence.md): tying the model to code,
   tests that measure, reviews and measurements by hand.
6. [Rules and proofs](6-rules-and-proofs.md): `p, q |- c`, proofs,
   propositions over many points.
7. [Contracts and use](7-contracts-and-use.md): what several functions
   promise alike, reading names from another part, the standard model.
8. [The commands](8-commands.md): `check`, `explain`, `blame`, their
   options and exit codes.
9. [Codes](codes/README.md): every error and finding, with an example
   and the fix.
