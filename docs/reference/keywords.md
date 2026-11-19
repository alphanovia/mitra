# Keywords

Part of the [language reference](language.md). One entry per keyword,
each with a model that runs. `cargo test -p mitra-lang --test reference`
reads and evaluates every example, and checks the output shown.

To try one: copy the model into `model.mitra`, the evidence into
`evidence.mitra`, then `mitra eval model.mitra evidence.mitra` (or
`mitra check model.mitra` for the model alone).

| Keyword | What it is | Entry |
|---|---|---|
| `subject`, `subject fn` | real code under review | [kw-declarations.md](kw-declarations.md#subject) |
| `boundary`, `boundary fn` | the environment | [kw-declarations.md](kw-declarations.md#boundary) |
| `fn` | a function: code, code with its spec, or a helper of the model | [kw-declarations.md](kw-declarations.md#fn) |
| `uses`, `only` | what the code of a function calls | [kw-declarations.md](kw-declarations.md#uses) |
| `record`, `record R;`, `..` | a value with named fields, stated in full, in part or not at all | [kw-records.md](kw-records.md#record) |
| `impl`, `self`, `Self` | the functions and claims of a record, as Rust writes them | [kw-records.md](kw-records.md#impl) |
| `contract` | a role: signatures and what every piece that fills it owes | [kw-contracts.md](kw-contracts.md#contract) |
| `impl … for` | a type fills a contract, in a subject or a boundary | [kw-contracts.md](kw-contracts.md#impl-for) |
| `impl Contract` | as a type: any value, known by what the contract promises | [kw-contracts.md](kw-contracts.md#impl-contract) |
| `r#word` | a word of the language used as a name | [kw-records.md](kw-records.md#names-and-r) |
| `passes` | a test passed on inputs it does not state | [kw-evidence.md](kw-evidence.md#passes) |
| `act`, `by`, `written`, `cold`, `of` | what was done and by whom | [kw-acts.md](kw-acts.md) |
| `sampled` | a test ran on inputs drawn from a stated domain | [kw-evidence.md](kw-evidence.md#sampled) |
| `enum` | a value that is one of several variants | [kw-records.md](kw-records.md#enum) |
| `ensure` | what a subject promises, in plain words | [kw-requirements.md](kw-requirements.md#ensure) |
| `require` | a named need: a promise made elsewhere, or one nobody makes yet | [kw-requirements.md](kw-requirements.md#require) |
| `import` | declarations of another file or of `std` | [kw-declarations.md](kw-declarations.md#import) |
| `//` | a comment | [kw-declarations.md](kw-declarations.md#comments) |
| `claim` | what must be true for every input | [kw-claims.md](kw-claims.md#claim) |
| `when` | only where a condition holds | [kw-claims.md](kw-claims.md#when) |
| `given`, `then` | a claim as a contract | [kw-claims.md](kw-claims.md#given-then) |
| `trust` | how much a statement or a piece of evidence is trusted | [kw-claims.md](kw-claims.md#trust) |
| `correct` | the code is right, as judged | [kw-claims.md](kw-claims.md#correct) |
| `deterministic` | the same input always gives the same output | [kw-claims.md](kw-claims.md#deterministic) |
| `assume` | a claim on a boundary, true without evidence | [kw-claims.md](kw-claims.md#assume) |
| `claim c(f, T)` | a claim with parameters | [kw-library.md](kw-library.md#claims-with-parameters) |
| `commutative`, `associative`, `idempotent`, `neutral`, `inverse`, `monotone`, `within` | claims shipped with Mitra | [kw-library.md](kw-library.md#stdclaims) |
| `uniform` | one tested input stands for its class | [kw-library.md](kw-library.md#uniform) |
| `represented` | one tested input stands for a set | [kw-library.md](kw-library.md#represented) |
| `+ - * / == != < <= > >= && \\|\\| ! <=>` | operators | [kw-expressions.md](kw-expressions.md#operators) |
| `if`, `else` | a choice | [kw-expressions.md](kw-expressions.md#if-else) |
| `let` | a name for a value | [kw-expressions.md](kw-expressions.md#let) |
| `match`, `_` | by variant of an enum | [kw-expressions.md](kw-expressions.md#match) |
| `is` | whether a value is a variant | [kw-expressions.md](kw-expressions.md#is) |
| `in` | membership, and each element of | [kw-expressions.md](kw-expressions.md#in) |
| `len`, `keys`, `[ ]`, `{ }` | lists, sets, maps, tuples | [kw-expressions.md](kw-expressions.md#lists-sets-maps) |
| `all`, `any`, `filter`, `map`, `fold` | over each element | [kw-expressions.md](kw-expressions.md#all-any-filter-map-fold) |
| `rate` | the share of values where a condition holds | [kw-expressions.md](kw-expressions.md#rate) |
| `precision`, `recall`, `false_positive_rate` | measures of a true/false function | [kw-expressions.md](kw-expressions.md#precision-recall-false_positive_rate) |
| `\\|a, b\\| e` | an inline function | [kw-expressions.md](kw-expressions.md#inline-functions) |
| `true`, `false` | the two values of `Bool` | [kw-expressions.md](kw-expressions.md#true-false) |
| `Bool`, `bool`, `Int`, `i64`, `u8`, `Float64`, `f64`, `String`, `Any`, `()` | plain types | [kw-types.md](kw-types.md#bool-int-float64-string-any) |
| `0..=9`, `1..` | ranges | [kw-types.md](kw-types.md#ranges) |
| `{"a", "b"}` | sets of values | [kw-types.md](kw-types.md#sets-of-values) |
| `A \\| B`, `A & B`, `A \\ B` | union, both, minus | [kw-types.md](kw-types.md#union-both-minus) |
| `(A, B)` | tuples | [kw-types.md](kw-types.md#tuples) |
| `List`, `Vec`, `Set`, `Map` | collections | [kw-types.md](kw-types.md#list-set-map) |
| `Option`, `Result` | enums of `std::rust` | [kw-types.md](kw-types.md#option-result) |
| `<T>` | type parameters | [kw-types.md](kw-types.md#type-parameters) |
| `T: Contract`, `where` | a type parameter known by the contracts it fills | [kw-generics.md](kw-generics.md#bounds) |
| `impl<T> R<T>` | the functions of a record with parameters | [kw-generics.md](kw-generics.md#impl-with-parameters) |
| `contract C<K>` | a contract with type parameters | [kw-generics.md](kw-generics.md#contracts-with-parameters) |
| `fn(A) -> B` | function types | [kw-types.md](kw-types.md#function-types) |
| `evidence` | one thing someone saw or judged | [kw-evidence.md](kw-evidence.md#evidence) |
| `for` | what the evidence is on | [kw-evidence.md](kw-evidence.md#for) |
| `given` | the inputs of the evidence | [kw-evidence.md](kw-evidence.md#given) |
| `holds`, `fails` | right or wrong, true or false | [kw-evidence.md](kw-evidence.md#holds-fails) |
| `gives` | the output observed, or a measurement | [kw-evidence.md](kw-evidence.md#gives) |
| `trust` | how much a piece of evidence is believed | [kw-evidence.md](kw-evidence.md#trust) |
| `source` | where the evidence comes from | [kw-evidence.md](kw-evidence.md#source) |
| `rev` | the revision of the code | [kw-evidence.md](kw-evidence.md#rev) |
| `seen` | the fingerprint of the piece when a person said it | [kw-evidence.md](kw-evidence.md#seen) |

Not keywords, and refused with advice: `test`, `opaque`, `def`,
`forall`, `derive`. What the language does not read yet is listed in
[language.md](language.md), "Not yet".
