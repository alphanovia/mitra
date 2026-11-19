# Keywords

Part of the [language reference](language.md). One entry per keyword,
each with a model that runs. `cargo test -p mitra-core --test reference`
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
| `record` | a value with named fields | [kw-records.md](kw-records.md#record) |
| `enum` | a value that is one of several variants | [kw-records.md](kw-records.md#enum) |
| `ensure` | what a subject promises, in plain words | [kw-requirements.md](kw-requirements.md#ensure) |
| `require` | a promise made elsewhere that this one needs | [kw-requirements.md](kw-requirements.md#require) |
| `import` | declarations of another file or of `std` | [kw-declarations.md](kw-declarations.md#import) |
| `//` | a comment | [kw-declarations.md](kw-declarations.md#comments) |
| `claim` | what must be true for every input | [kw-claims.md](kw-claims.md#claim) |
| `when` | only where a condition holds | [kw-claims.md](kw-claims.md#when) |
| `given`, `then` | a claim as a contract | [kw-claims.md](kw-claims.md#given-then) |
| `trust` | how much a statement or a piece of evidence is trusted | [kw-claims.md](kw-claims.md#trust) |
| `correct` | the code is right, as judged | [kw-claims.md](kw-claims.md#correct) |
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
| `Bool`, `bool`, `Int`, `i64`, `u8`, `Float64`, `f64`, `String`, `Value`, `()` | plain types | [kw-types.md](kw-types.md#bool-int-float64-string-value) |
| `0..=9`, `1..` | ranges | [kw-types.md](kw-types.md#ranges) |
| `{"a", "b"}` | sets of values | [kw-types.md](kw-types.md#sets-of-values) |
| `A \\| B`, `A & B`, `A \\ B` | union, both, minus | [kw-types.md](kw-types.md#union-both-minus) |
| `(A, B)` | tuples | [kw-types.md](kw-types.md#tuples) |
| `List`, `Vec`, `Set`, `Map` | collections | [kw-types.md](kw-types.md#list-set-map) |
| `Option`, `Result` | enums of `std::rust` | [kw-types.md](kw-types.md#option-result) |
| `<T>` | type parameters | [kw-types.md](kw-types.md#type-parameters) |
| `fn(A) -> B` | function types | [kw-types.md](kw-types.md#function-types) |
| `evidence` | one thing someone saw or judged | [kw-evidence.md](kw-evidence.md#evidence) |
| `for` | what the evidence is on | [kw-evidence.md](kw-evidence.md#for) |
| `given` | the inputs of the evidence | [kw-evidence.md](kw-evidence.md#given) |
| `holds`, `fails` | right or wrong, true or false | [kw-evidence.md](kw-evidence.md#holds-fails) |
| `gives` | the output observed, or a measurement | [kw-evidence.md](kw-evidence.md#gives) |
| `trust` | how much a piece of evidence is believed | [kw-evidence.md](kw-evidence.md#trust) |
| `source` | where the evidence comes from | [kw-evidence.md](kw-evidence.md#source) |
| `rev` | the revision of the code | [kw-evidence.md](kw-evidence.md#rev) |

Not keywords, and refused with advice: `test`, `opaque`, `def`,
`forall`, `derive`. What the language does not read yet is listed in
[language.md](language.md), "Not yet".
