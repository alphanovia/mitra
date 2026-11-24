# Agree on core2

`crates/mitra-lang/src/agree.rs` answers the first question with no evidence: is the model
sound on itself, and if not, where? It reads a model `check` accepts and
compares what the model says twice.

```rust
pub struct Settings { pub points: usize }        // default 64
pub enum Agreement { Holds, Fails { at: Vec<(String, Value)>, gives: Option<Value> }, Unknown { why: String } }
pub struct Agreed { pub at: Path, pub against: String, pub agreement: Agreement }
pub fn agree(model: &Model, settings: &Settings) -> (Vec<Agreed>, Vec<Finding>)
```

## The comparisons

| Of | Against | Reading |
|---|---|---|
| a function's body | each of its claims | bounds over all it takes, then points |
| a function's body | each of its examples | `Exact` at the example's point |
| a function's example | each of its claims | `Exact`, the example's value as what it gives |
| a function's claims (one or more) | some value that keeps them all | bounds, then points: its examples, what its body gives, candidates |
| a contract's claim, read of what fits | every value of its arguments | bounds, then points |
| an example of the model | what the model computes | `Exact`, through calls, the world's values given first (`Calls::given`) |
| a value of the world an example of the model gives | the function's own examples at that point | the values they give |

A comparison **holds** when bounds say surely true over all of what it
reads, or when its domain is finite and every point was tried. It
**fails** at a point `Exact` confirms, or, for claims no value keeps
together, at no point: by bounds, or at every point of a finite domain.
Else it is **unknown**, and says how far it looked: the number of
points, or the function with no value there. Every comparison of the
table is counted (div: 9, four examples against two claims and the
claims together).

Points come in order, each once: the function's examples, then small
numbers (0, 1, −1, 2, −2, ½) and the constants the expressions write,
and those of the functions they call, however far, with their
neighbours, the smallest combinations first, up to `Settings::points`.
A function that takes nothing has one point, `()`. A record or a tuple
is tried at every value when it has at most `Settings::points` (two
truths: four), else at the i-th value of each field together (the
"diagonal": `(0, 0)`, `(1, 1)`, …).

**An opaque record** (`record Key;`) has values no one writes, and
simulation reads its type as a record of no field: one value, so two
keys would be equal. Agree reads no bounds over a comparison that reads
one, there or through the functions it calls, and makes none of its
values: such a comparison is unknown unless an example decides it.

**Inputs no output reads**: an input of an example of the model is read
when it is what an output's function takes, a value of the world the
computation calls for, or what that function takes. Any other is A009.

## Findings (suit A)

| Code | What |
|---|---|
| A001 | a body that breaks a claim |
| A002 | a body that disagrees with an example |
| A003 | an example that breaks a claim |
| A004 | claims no value keeps together, by bounds |
| A005 | a contract claim a function that fits breaks |
| A006 | an example of the model the model does not give |
| A007 | a comparison not decided: nothing is dropped quietly |
| A008 | retired: two examples at one point that differ are now the error M024 |
| A009 | an input of a model example no output reads |
| A010 | hint: a comparison that holds given the claims of a function with no body |

## Your choices, flagged (say no to undo)

1. **The claims of a function are compared together, one claim or
   more**: shown kept at a point, or clashing by bounds or at every point
   of a finite domain, else unknown (A007). A function whose claims no
   point tried keeps is said, not passed over.
2. **A comparison that reads an opaque record is not read by bounds**,
   and agree makes none of its values: unknown, unless an example
   decides it. The other way, a hidden field in the record's type, would
   change the parser's types.
3. **An input of an example of the model no output reads is a finding**
   (A009), as explain's X005 is for a measurement.
4. **A value of the world an example of the model gives is compared with
   the function's own examples there** (A006 when they differ); the
   example of the model's value is the one read.

## What it cannot do yet (findings for you)

- **Cuts only at zero.** Bounds read each of the first `splits` numbers
  (a setting, 4) below zero and above it, so `0 <= x * x` holds and
  `0 < q && q < 0` keeps nothing. A cut elsewhere, or bisection, would
  decide `!(h <= 20) || v2 < 400`; today it is unknown.
- **Sequences are tried up to `length` long** (a setting, 3): `[]`, then
  for each length the values worth trying of what it holds, in turn and
  repeated. A claim false only on longer ones is unknown (A007). By
  bounds, a sequence keeps its length and one span for all it holds, and
  the span of each place it surely holds: `p[0] < p[1]` of `[a, a + 1]`
  is read place by place only at a point.
- **Text is tried from what the model writes**: `""`, each text the
  expressions write, each two joined, and their prefixes (finding 3,
  step 3). A claim false only on another text is unknown (A007). By
  bounds, a text keeps its length in bytes, and the text itself when
  it is known.
- **Range types are read as premises.** `fn f(a: 0..=9)` reads each
  claim as `!(0 <= a && a <= 9) || claim`, and the range of what it
  gives as one more comparison, `its range \`q: 0..=9\``: a body that
  leaves it is A001. When the ranges hold at most `points` numbers
  together, every one is tried: main's `div(a: 0..=9, b: 0..=9)` needs
  100 points, more than the default 64, so it is unknown (A007)
  until `points` is raised.
- **A function with no body gives what its claims promise.** By bounds,
  its call is any value its claims keep (`200.0 <= w`: `[200, ∞)`), and no
  fault: the world gives a value. A comparison that holds so is given those
  claims: A010 says which. `!p || c` reads `c` where `p` keeps its names,
  so `!(1.0 <= h) || 200.0 <= h * light()` holds, given `bright`.
- **Comparisons read together one name at a time.** Among the claims
  joined by `&&`, each comparison of a name with what bounds give of the
  other side cuts that name (`5 < q && q < 3` keeps nothing). Relations
  between two names (`x < z && z < x`) are not kept: no octagons. On
  `rational` a strict end stays in: `1.0 < q && q < 1.0` is unknown.
- **No solver**, by your choice: points are tried, not found.
- Record claims are not compared: a record has no body.
- A proposition written in a rule's `{ … }` term is read as a claim by
  `check` (M006), and not compared by agree.
- A contract claim false for every function (`claim lie(a: T): a < a`)
  is said at each impl (A005, "fix the function"), never at the
  contract: a contract is checked once, by type, where it is written.
- Cost: `Exact` computes both sides of an `if`, `&&` and `||`, and does
  not remember calls (inner): a chain of 12 functions, each calling the
  one before in both branches of an `if`, takes about 1 s, and each level
  doubles it; written with `let`, 15 ms. Agree has no budget of steps.
