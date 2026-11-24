# Codes

Every error and finding Mitra gives has a page here: what it means, an
example, and the fix. The letter says which step found it.

## P (parse): the text of the model

- [P001](P001.md): a character that starts nothing
- [P002](P002.md): a string that is not closed
- [P003](P003.md): not what the language has here
- [P004](P004.md): a tuple or record whose type cannot be told
- [P005](P005.md): a fraction with zero below
- [P006](P006.md): a `match` that misses a variant, or names one twice
- [P007](P007.md): an unknown variant, or one given the wrong number of values
- [P008](P008.md): a type given the wrong type arguments
- [P009](P009.md): a value whose type cannot be told here
- [P010](P010.md): a part no file of the project declares
- [P011](P011.md): a text nested past the parser's depth budget
- [P012](P012.md): a part reopened as another kind
- [P013](P013.md): a method call on a value whose part has no such method
- [P014](P014.md): a part a `use` names that two folders declare
- [P015](P015.md): a `mitra.toml` Mitra cannot read

## M (model): names, types and structure (`check`, before agree)

- [M001](M001.md): a name is not a word
- [M002](M002.md): a name is declared twice
- [M003](M003.md): a type that no record declares
- [M004](M004.md): a record made other than as declared
- [M005](M005.md): a body not of the type its function gives
- [M006](M006.md): a claim that is not a truth
- [M007](M007.md): a name that names no proposition
- [M008](M008.md): a proposition of another part
- [M009](M009.md): a require from no ensure of another part
- [M010](M010.md): a tie to no part, record or function
- [M011](M011.md): an empty list of an ensure
- [M012](M012.md): a proof that does not show its rule
- [M013](M013.md): a declaration that rests on itself
- [M014](M014.md): a rule named through itself
- [M015](M015.md): an example not of its function's types
- [M016](M016.md): a use that names nothing of another part, or a name twice
- [M017](M017.md): a call of no function
- [M018](M018.md): a function or record of another part, not used
- [M019](M019.md): a function that calls itself, not on a part of its input
- [M020](M020.md): an impl that does not fit its contract, or names no function
- [M021](M021.md): a model example that names no input or output, or states no output
- [M022](M022.md): a model example not of its types, or missing an input
- [M023](M023.md): a correct, all or any a rule cannot write
- [M024](M024.md): two examples at one point that differ
- [M025](M025.md): a range type that holds no number
- [M026](M026.md): an example outside the range of its type
- [M027](M027.md): an enum or a record declared so that no value fits it
- [M028](M028.md): a part holding parts that holds more than parts, requires, ensures and examples
- [M029](M029.md): an ensure of a part holding parts that rests on what is not its parts' ensures
- [M030](M030.md): a `self` that is not the first of a method of a part with a record
- [M031](M031.md): a claim `by induction` on what is not an input that is a tree
- [M032](M032.md): a proof written as its step alone that no proof, or two, fill in
- [M033](M033.md): a claim `by invariant` of what is not a loop's function

## A (agree): the model compared with itself (`check`)

- [A001](A001.md): a body that breaks a claim
- [A002](A002.md): a body that disagrees with an example
- [A003](A003.md): an example that breaks a claim
- [A004](A004.md): claims no value keeps together
- [A005](A005.md): a contract claim a function that fits breaks
- [A006](A006.md): an example of the model the model does not give
- [A007](A007.md): a comparison not decided
- [A008](A008.md): two examples at one point that differ
- [A009](A009.md): an input of a model example no output reads
- [A010](A010.md): a comparison that holds given the claims of a function with no body

## R (reader): `mitra:` comments in code and the test report

- [R001](R001.md): a `mitra:` comment the reader cannot read
- [R002](R002.md): a tie comment on no item, or to nothing of the model
- [R003](R003.md): a `measures` comment on no test
- [R004](R004.md): a test that measures nothing the reader knows
- [R005](R005.md): a measuring test that did not run
- [R006](R006.md): what names nothing of the model, or is not of its types
- [R007](R007.md): a measurement through no tie
- [R008](R008.md): a review seeing a test that did not pass
- [R009](R009.md): a test binary that stopped before its result
- [R010](R010.md): a test Mitra could not read in the report
- [R011](R011.md): a field or variant on one side of a tie only
- [R012](R012.md): a field whose Rust type holds values its model type does not
- [R013](R013.md): a Rust form Mitra cannot write yet
- [README](README.md): # Codes

## V (evidence): measurements and reviews

- [V001](V001.md): a measurement that names no input or output
- [V002](V002.md): a measurement through no tie
- [V003](V003.md): a measured value not of its type, or of an opaque record
- [V004](V004.md): a measurement of no run
- [V005](V005.md): a review of nothing a review can name
- [V006](V006.md): a measurement missing an input, or with no output
- [V007](V007.md): a voice missing, not of a kind, Mitra's own, or saying twice
- [V008](V008.md): a measured value outside the range of its type

## X (explain): the model against the evidence

- [X001](X001.md): a measurement nothing in the model says what to expect of
- [X002](X002.md): a failed test whose expected value the model does not give
- [X003](X003.md): a claim a measurement does not decide
- [X004](X004.md): an output the model cannot compute from the inputs
- [X005](X005.md): an input nothing measured reads
- [X006](X006.md): judging stopped before it was done
- [X007](X007.md): an opaque function measured the same each time
- [X008](X008.md): two measurements that say one thing, by one voice and count
- [X009](X009.md): a budget of steps or depth reached
- [X010](X010.md): a value known in part, where only what is not known could tell
- [X011](X011.md): a measured value outside the range of its type
- [X012](X012.md): a link, or a piece said only in words, no voice reviewed

## B (blame): diagnoses

- [B001](B001.md): a conflict the model's claims alone hold
- [B002](B002.md): more diagnoses than blame looked for
- [B003](B003.md): a diagnosis a proof by cases may outlive

