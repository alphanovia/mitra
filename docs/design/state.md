# Design: state and sequences of calls

Status: **proposal**. Built: B (subjects without parameters, `()`), and
golden cases for sections 1 and 3A with traces written by hand. Decided
(2026-11-18): traces (A), by hand first; invariants by hand; nothing from
"not proposed" until the trials ask for it.

The examples are in the first syntax, which is no longer read.
Since then, in the [language](../reference/language.md): S5 has a
construct, the `boundary`, whose functions may answer differently each
time, with `assume` for what is taken as true of it; a test is tied to a
claim, so S1 reads "a tied test checks one point claim, not a sequence".
Sequences and simulations are deferred.

## 1. What works today: state is a value

A `&mut` is one more input and one more output. No language change.

| Rust | Mitra |
|---|---|
| `fn deposit(&mut self, n: u64)` | `opaque deposit(a: Account, n: 0..) -> Account` |
| `fn withdraw(&mut self, n: u64) -> bool` | `opaque withdraw(a: Account, n: 0..) -> (Account, Bool)` |
| `fn take_value(name: &str, rest: &mut Vec<String>) -> Option<String>` | `opaque take_value(name: String, rest: List(String)) -> (Option(String), List(String))` |
| a file read and written | its content, as a `String` in and a `String` out |

```lean
record Account(balance: 0.., open: Bool)
def withdraw_spec(a: Account, n: 0..) -> (Account, Bool) :=
  if a.open && n <= a.balance then (Account(a.balance - n, true), true) else (a, false)
claim withdraw_is : forall a, forall n, withdraw(a, n) = withdraw_spec(a, n)
def withdraw_class(a: Account, n: 0..) -> {"closed", "covered", "overdraw"} :=
  if !a.open then "closed" else if n <= a.balance then "covered" else "overdraw"
claim withdraw_uniform : uniform withdraw by withdraw_class
```

Tried on main: specs, classes, verdicts and `mitra next` behave as for any
function (`no evidence on class "closed"`).

## 2. What does not work (tried on main)

| # | Gap | What happens |
|---|---|---|
| S1 | A test over a sequence | `test t : withdraw(deposit(Account(0, true), 5), 3) = …` is refused: a test names one call on values. Tests of stateful code are nearly all sequences, so today they stay outside the model. |
| S2 | A subject without parameters | `opaque open_account() -> Account` is refused. |
| S3 | A property over two calls | `withdraw(deposit(a, n), n).1 == a.open` is accepted, but `uniform` does not reach a call inside a call, so one test stands for one input. |
| S4 | An invariant | "the balance never goes below 0, whatever the calls" can't be concluded from one claim per operation. |
| S5 | State outside the value | time, randomness, the network, another process writing the file. |

## 3. Proposal

### A. Sequence tests are traces (answers S1)

The Rust test runs as it is. The harness records every call of a modelled
function made during the test: input state, arguments, output.

```
-- Rust: a.deposit(5); assert!(a.withdraw(3)); assert_eq!(a.balance, 2)
@[source "test:t_seq"] evidence deposit(Account(0, true), 5) = Account(5, true) at 1
@[source "test:t_seq"] evidence withdraw(Account(5, true), 3) = (Account(2, true), true) at 1
```

Each line is plain evidence of the kind Mitra already reads. The assert at
the end judges the last line; the lines before it are observed outputs.

- The test passes: every line is evidence of what the code returned. A line
  agrees or disagrees with the spec of its function, like any observed
  output. One sequence test gives one piece of evidence per call.
- The test fails: the lines are compared with the specs. The first line
  that disagrees names the function, with certainty. No line disagrees:
  the functions did what the specs say and the test expected something
  else, so the candidates are the test and the specs.
- A function without a spec: its lines are outputs nobody judged. They
  count only for claims that compare outputs (`f(x) = g(x)`, properties).

Cost: an integration piece, not core. A Rust macro or a wrapper prints
`mitra: f(args) = output` lines, `mitra ingest` reads them; the state must
print as a Mitra value. It also closes the dogfooding gap "a failed test
gives no output".

Alternative A': let a test be any expression with nested calls. No
instrumentation, but the states in the middle are not observed: a pass
says nothing on each call, a failure only lists every function called as
candidates.

### B. Subjects without parameters (S2)

`opaque open_account() -> Account`, `correct(open_account())`,
`open_account() = Account(0, true)`. Grammar only.

### C. Invariants (S3, S4)

Written with what exists, plus one new claim form:

```lean
def valid(a: Account) -> Bool := a.open || a.balance == 0
claim account_valid : invariant valid on Account
```

It reads: every modelled function that returns an `Account` returns a
valid one when the `Account`s it was given are valid. Mitra expands it to
one property claim per function (`open_account`, `deposit`, `withdraw`),
each judged as usual with its own classes; `account_valid` holds when all
of them hold, and then `valid` may be assumed of every state reached
through these functions. A trace line with an invalid output fails it and
names the function.

Not proposed: temporal operators (eventually, until), concurrency.

### D. The outside world (S5)

No construct. What a function reads from outside is an input, what it
writes is an output. What can't be listed that way (another process, time
passing) is an assumption: a claim vouched by review.

## 4. Order, if you agree

1. B, then a golden case for section 1 (small).
2. A: the trace lines in `ingest`, then the Rust side, tried on
   `take_value` and the store in mitra-cli.
3. C.

## 5. Decisions for you

1. Sequence tests: traces (A) or nested calls (A')?
2. Is the Rust instrumentation in scope now, or do you write trace lines
   by hand in the trials first?
3. `invariant p on T` as a claim form, or leave invariants as hand-written
   property claims per function?
4. Is anything in "not proposed" needed for the agile organisation model
   (a team's state from sprint to sprint)?
