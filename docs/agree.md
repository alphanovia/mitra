# Agree on core2

`src/agree.rs` answers the first question with no evidence: is the model
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
| a function's example | each later one at the same point | the values they give |
| a function's claims | each other | bounds: all of them false whatever the values |
| a contract's claim, read of what fits | every value of its arguments | bounds, then points |
| an example of the model | what the model computes | `Exact`, through calls, the world's values as rows |

A comparison **holds** when bounds say surely true over all of what it
reads, or when its domain is finite and every point was tried. It
**fails** only at a point `Exact` confirms. Else it is **unknown**, and
says how far it looked: the number of points, or the function with no
value there. Points come in order: the function's examples, then small
numbers (0, 1, −1, 2, −2, ½) and the constants the expressions write,
with their neighbours, the smallest combinations first, up to
`Settings::points`.

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
| A008 | two examples at one point that differ |

## What it cannot do yet (findings for you)

- **No splitting.** Bisection needs bounded inputs, and core2's numbers
  are all of ℤ or ℚ: `Ranges` has a number in `[low, high]` or anywhere,
  with no half-line, so `(−∞, ∞)` cannot be cut in two. A claim like
  `!(h <= 20) || v2 < 400` is unknown, though it holds. Either half-lines
  in `Ranges` (an inner change) or range types (main's `0..=9`) would let
  agree split and show it.
- **Bounds read each name on its own.** `0 < q && q < 0` is not seen as
  false, nor `0 <= x * x` as true: interval reasoning has no relations.
- **No solver**, by your choice: points are tried, not found.
- Record claims are not compared: a record has no body.
