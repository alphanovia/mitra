# Numbers: the world and the machine

A proposal to read and react to. Nothing here is built. Six toy models,
each written twice: as main reads it today (run by hand on 409f745),
and as it could read with the two kinds of numbers apart.

Asked (2026-11-20): "we need to define natural arithmetic to represent
the world, so that we could model quantization for exemple, or analog
to digital. And we need to have a small type that captures computer
and language operations and assumptions, which is enough to build
common types." "Those too should be explicitly apart."

## What main does today

- Integers are the world's. `Int`, `Nat` and ranges are exact, and
  `UInt8` is only a name for `0..=255`. `fn bump(c: UInt8) -> UInt8
  { c + 1 }` is refused: "`bump(255)` = 256 is outside its result
  type". There is no way to say that an addition wraps.
- Floats are the machine's. `claim tenths() { 0.1 + 0.2 == 0.3 }`
  **fails** on main: `Float64` arithmetic in a model is the host's.
- The two do not mix: `sample(v) * 3.3` is refused, an integer times
  a float.

So the split exists already, by accident and unsaid: integers on one
side, floats on the other.

## Proposed

**The world.** `Int`, `Nat`, ranges, and `Real`. Operations are exact.
A literal `0.1` is the real number one tenth. `Real` is computed as
exact fractions; no square root or sine until one is needed.

**The machine.** One built-in, `Bits<N>`: N bits and nothing more. The
common types are written in a library, in Mitra, as a boundary: the
machine is bought, so what it does is an `assume`.

```text
// std::machine, a sketch
boundary machine {
    record UInt8 { bits: Bits<8> }
    impl UInt8 {
        fn of(n: 0..=255) -> Self;            // the way in
        fn value(&self) -> 0..=255;           // the way out: the number it stands for
        fn wrapping_add(&self, o: Self) -> Self;
        fn checked_add(&self, o: Self) -> Option<Self>;
        assume reads_back(n: 0..=255) { UInt8::of(n).value() == n }
        assume wraps(self, o: Self) { self.wrapping_add(o).value() == (self.value() + o.value()) % 256 }
        assume checks(self, o: Self) { self.checked_add(o) is None <=> self.value() + o.value() > 255 }
    }
}
```

**Apart.** A machine value has no `+`, `<` or literal. It crosses to
the world by `value` and back by `of`, both named. What a language adds
is in its own library: `std::rust` would say that `u8 + u8` is
`checked_add` and a panic on `None` in a debug build.

## The toys

### 1. A counter that can overflow

Shows: who says what happens at 255.

```mitra
// today, runs: the wrap is two claims on an opaque function
subject counter {
    fn bump(c: UInt8) -> UInt8;
    claim steps(c: 0..=254) { bump(c) == c + 1 }
    claim wraps() { bump(255) == 0 }
}
```
```text
// proposed: the body names the operation, the wrap follows from the machine
subject counter {
    fn bump(c: UInt8) -> UInt8 { c.wrapping_add(UInt8::of(1)) }
    claim wraps() { bump(UInt8::of(255)).value() == 0 }
}
```

### 2. A checksum

Shows: a wrap that is wanted, over a list.

```mitra
// today, runs: the sum is the world's, cut by hand (there is no `%`)
fn rem(n: Nat, m: 1..) -> Nat { n - n / m * m }
subject sum {
    fn checksum(data: List<UInt8>) -> UInt8;
    claim folds(data: List<UInt8>) { checksum(data) == rem(fold(b in data, let s = 0; s + b), 256) }
}
```
```text
// proposed: the body is what the machine does, the claim is what the world sees
subject sum {
    fn checksum(data: List<UInt8>) -> UInt8 { fold(b in data, let s = UInt8::of(0); s.wrapping_add(b)) }
    claim folds(data: List<UInt8>) { checksum(data).value() == fold(b in data, let s = 0; s + b.value()) % 256 }
}
```

### 3. An average

Shows: a bug the model finds with no test. The claim is the world's
answer; the body is the classic `(a + b) / 2`.

```mitra
// today, runs: only a test at the top can find the overflow
subject stats {
    fn mean(a: Int32, b: Int32) -> Int32;
    claim exact(a: Int32, b: Int32) { mean(a, b) == (a + b) / 2 }
}
```
```text
// proposed: the claim contradicts the body at (2147483647, 2147483647), read from the model alone
subject stats {
    fn mean(a: Int32, b: Int32) -> Int32 { a.wrapping_add(b).div(Int32::of(2)) }
    claim exact(a: Int32, b: Int32) { mean(a, b).value() == (a.value() + b.value()) / 2 }
}
```

### 4. Analog to digital

Shows: a quantity of the world read into 12 bits, and how far off.

```mitra
// today, runs: two opaque functions; the subtraction is the host's floats
subject adc {
    fn sample(v: Float64) -> 0..=4095;
    fn volts(code: 0..=4095) -> Float64;
    claim half_step(v: Float64) when v >= 0.0 && v <= 3.3 {
        volts(sample(v)) - v <= 0.0005 && v - volts(sample(v)) <= 0.0005
    }
}
```
```text
// proposed: the voltage is a Real, the code is 12 bits, the error is exact
subject adc {
    fn sample(v: Real) -> UInt12;
    fn volts(code: UInt12) -> Real { code.value() * 3.3 / 4095 }
    claim half_step(v: Real) when 0 <= v && v <= 3.3 { abs(v - volts(sample(v))) <= 3.3 / 4095 / 2 }
}
```

### 5. Money in cents

Shows: a model that needs no machine number at all.

```mitra
// today, runs, and would not change
subject bill {
    fn split(total: Nat, n: 1..=10) -> (Nat, Nat);
    claim no_cent_lost(total: Nat, n: 1..=10) {
        let s = split(total, n);
        s.0 * n + s.1 == total && s.1 < n
    }
}
```

The machine enters only at the tie: the code's `u64` is read as `Nat`,
an erasure (`range`, cost 0.05) that says the total fits.

### 6. A float mean with a tolerance

Shows: what a float promises, said once in the library.

```text
// today: not expressible. `mean(0.1, 0.2) == 0.15` fails on main, on the host's rounding
// proposed
subject stats {
    fn mean(a: Float64, b: Float64) -> Float64 { a.add(b).div(Float64::of(2)) }
    claim close(a: Float64, b: Float64) when abs(a.value()) < 1e300 && abs(b.value()) < 1e300 {
        abs(mean(a, b).value() - (a.value() + b.value()) / 2) <= abs(a.value() + b.value()) / 1e15
    }
}
// in std::machine: assume add_rounds(a, b) { a.add(b).value() == nearest(a.value() + b.value()) }
```

## What to react to

1. **No `+` on a machine value**, only named operations. Explicit, and
   heavy: toy 3 reads worse than today.
2. **The machine is a boundary.** Every claim through `wrapping_add`
   then reads `holds assuming [machine::wraps]` until someone vouches
   for the library once.
3. **`UInt8` changes meaning**: today a range of the world, proposed
   a machine record. The range stays, written `0..=255`.
4. **`Bits<N>` is not used by any of the six toys.** `of` and `value`
   carry them all. It would matter for masks, shifts and the layout
   of a float.
5. **`Real` as exact fractions**: enough for toys 4 and 6, and it
   makes `0.1 + 0.2 == 0.3` hold.
6. Toys 1, 2, 3 and 5 can be a rung today, in the first form. Toys 4
   and 6 wait for `Real`.
