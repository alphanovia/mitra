# Numbers: the world and the machine

A proposal to read and react to. Nothing here is built. One toy, a
dishwasher on a small microcontroller, written twice: as main reads it
today (run by hand on 409f745), and as it could read with the two
kinds of numbers apart.

Asked (2026-11-20): "we need to define natural arithmetic to represent
the world, so that we could model quantization for exemple, or analog
to digital. And we need to have a small type that captures computer
and language operations and assumptions, which is enough to build
common types." "Those too should be explicitly apart." "a dishwasher
with temperature of water reading to test the Real to bits usage."
"And we have the motor on the other side."

## What main does today

- Integers are the world's. `Int`, `Nat` and ranges are exact, and
  `UInt8` is only a name for `0..=255`. `fn bump(c: UInt8) -> UInt8
  { c + 1 }` is refused: "`bump(255)` = 256 is outside its result
  type". There is no way to say that an addition wraps.
- Floats are the machine's. `claim tenths() { 0.1 + 0.2 == 0.3 }`
  **fails** on main: `Float64` arithmetic in a model is the host's.
- The two do not mix: an integer times a float is refused.

So the split exists already, by accident and unsaid.

## The dishwasher

The water has a temperature. A sensor and its converter give the
controller 10 bits. The controller answers with 8 bits, the duty of
the pump motor, which turns at some speed. Two promises cross the
whole chain: the pump stands still when the water is too hot, and it
runs at full speed at washing temperature.

```
water (degrees) --sensor--> 10 bits --control--> 8 bits --motor--> speed (rpm)
     world                        machine                        world
```

### Today, runs

The water is in tenths of a degree, since main has no `Real`.

```mitra
// The water, in tenths of a degree: main has no Real.
boundary sensor {
    ensure accurate = "Between 0 and 100 degrees the reading is within half a degree of the water.";
    fn read(water: 0..=1000) -> 0..=1023;
    assume within_half(water: 0..=1000) {
        degrees(read(water)) - water <= 5 && water - degrees(read(water)) <= 5
    }
}
boundary motor {
    ensure follows = "The pump turns at 3000 rpm at full duty, in proportion below, within 50 rpm; at duty 0 it stands still.";
    fn turn(duty: UInt8) -> 0..=3100;
    assume proportional(duty: UInt8) {
        turn(duty) - duty * 3000 / 255 <= 50 && duty * 3000 / 255 - turn(duty) <= 50
    }
    assume stands_still() { turn(0) == 0 }
}
// What a count means, in tenths of a degree.
fn degrees(code: 0..=1023) -> 0..=1000 { code * 1000 / 1023 }

subject washer {
    ensure safe = "The pump never runs when the water is at 76 degrees or more.";
    ensure washes = "From 56 to 74 degrees the pump runs at 2900 rpm or more.";
    require reading = sensor::accurate;
    require drive = motor::follows;

    // Counts: 563 is the first at 55 degrees or more, 767 the last under 75.
    fn control(code: 0..=1023) -> UInt8 {
        if code > 767 { 0 } else if code >= 563 { 255 } else { 64 }
    }

    claim thresholds() { degrees(563) >= 550 && degrees(562) < 550 && degrees(767) < 750 && degrees(768) >= 750 }
    claim stops_when_too_hot(water: 760..=1000) { motor::turn(control(sensor::read(water))) == 0 }
    claim full_when_hot(water: 560..=740) { motor::turn(control(sensor::read(water))) >= 2900 }
}
```

What main says of it:

- `washer::thresholds: holds`, computed: 563 counts is the first at 55
  degrees, 767 the last under 75.
- `stops_when_too_hot` and `full_when_hot`: **unverified**, "no
  evidence on water in 760..=1000". They follow from the two
  boundaries' assumes and the body of `control`, and Mitra does not
  make that step: an assume is not used to reason about what the
  boundary returns.
- Both promises read `holds assuming [control::spec,
  stops_when_too_hot, full_when_hot, sensor::within_half,
  motor::proportional, …]`: the sensor and the motor are named as
  what the promise leans on.

### Proposed

```text
boundary sensor {
    ensure accurate = "Between 0 and 100 degrees the reading is within half a degree of the water.";
    fn read(water: Real) -> UInt10;
    assume within_half(water: Real) when 0 <= water && water <= 100 { abs(degrees(read(water)) - water) <= 0.5 }
}
boundary motor {
    ensure follows = "The pump turns at 3000 rpm at full duty, in proportion below, within 50 rpm; at duty 0 it stands still.";
    fn turn(duty: UInt8) -> Real;
    assume proportional(duty: UInt8) { abs(turn(duty) - duty.value() * 3000 / 255) <= 50 }
    assume stands_still() { turn(UInt8::of(0)) == 0 }
}
// What a count means: from the machine to the world, exact.
fn degrees(code: UInt10) -> Real { code.value() * 100 / 1023 }

subject washer {
    ensure safe = "The pump never runs when the water is at 76 degrees or more.";
    ensure washes = "From 56 to 74 degrees the pump runs at 2900 rpm or more.";
    require reading = sensor::accurate;
    require drive = motor::follows;

    fn control(code: UInt10) -> UInt8 {
        if code.value() > 767 { UInt8::of(0) } else if code.value() >= 563 { UInt8::of(255) } else { UInt8::of(64) }
    }
    claim thresholds() { degrees(UInt10::of(563)) >= 55 && degrees(UInt10::of(562)) < 55 }
    claim stops_when_too_hot(water: Real) when 76 <= water && water <= 100 { motor::turn(control(sensor::read(water))) == 0 }
    claim full_when_hot(water: Real) when 56 <= water && water <= 74 { motor::turn(control(sensor::read(water))) >= 2900 }
}
```

What changes:

- **The water is a `Real`.** 76 degrees is `76`, half a degree is
  `0.5`, and the claim is over every temperature, not 1,001 of them.
  No test can cover it: the evidence is a sample on a rig, or the
  derivation main does not make today.
- **The two crossings are visible.** `degrees` and `value` go from
  bits to the world, `UInt8::of` goes back. Nothing else does.
- **For this controller the machine types add nothing.** It only
  compares and picks a constant, so no operation can go wrong, and
  `0..=1023` said as much.

### Where the machine type earns its place

A second version of the controller displays the temperature. It turns
counts into degrees on a 16 bit register:

```text
// today: the body is exact arithmetic, so the model agrees with itself and the bug is invisible
fn celsius(code: 0..=1023) -> 0..=100 { code * 100 / 1023 }

// proposed: the body says what the register does
fn celsius(code: UInt10) -> UInt16 { code.widen().wrapping_mul(UInt16::of(100)).div(UInt16::of(1023)) }
claim reads_degrees(code: UInt10) { celsius(code).value() == code.value() * 100 / 1023 }
```

`656 * 100` is 65,600, one turn past 16 bits. The claim contradicts
the body from 656 counts up, that is from 64 degrees: Mitra would say
so from the model alone, with no test, as it does today when a claim
contradicts a spec (M007).

## The pieces proposed

**The world.** `Int`, `Nat`, ranges, and `Real`. Operations are exact.
A literal `0.5` is one half. `Real` is computed as exact fractions; no
square root or sine until one is needed.

**The machine.** One built-in, `Bits<N>`. The common types are a
library written in Mitra, as a boundary: the machine is bought, so
what it does is an `assume`.

```text
// std::machine, a sketch
boundary machine {
    record UInt16 { bits: Bits<16> }
    impl UInt16 {
        fn of(n: 0..=65535) -> Self;          // the way in
        fn value(&self) -> 0..=65535;         // the way out: the number it stands for
        fn wrapping_mul(&self, o: Self) -> Self;
        fn checked_mul(&self, o: Self) -> Option<Self>;
        assume reads_back(n: 0..=65535) { UInt16::of(n).value() == n }
        assume wraps(self, o: Self) { self.wrapping_mul(o).value() == (self.value() * o.value()) % 65536 }
    }
}
```

What a language adds is in its own library: `std::rust` would say that
`u16 * u16` is `checked_mul` and a panic on `None` in a debug build.

## What to react to

1. **No operator on a machine value**, only named operations
   (`wrapping_mul`). Explicit, and heavy: `celsius` reads worse than
   the C it models. A comparison cannot go wrong, so `code > 767`
   could be allowed where arithmetic is not.
2. **The machine is a boundary.** A claim through `wrapping_mul` then
   reads `holds assuming [machine::wraps]` until someone vouches for
   the library once.
3. **`UInt8` changes meaning**: today a range of the world, proposed a
   machine record. The range stays, written `0..=255`.
4. **`Bits<N>` is not used by the dishwasher.** `of` and `value` carry
   it. It would matter for masks, shifts and the layout of a float.
5. **The step main does not make**: deriving `stops_when_too_hot` from
   the assumes. With `Real` no enumeration can stand in for it. This
   is a question of the calculus more than of the types.
6. Not shown, since the dishwasher has none: floats. On main
   `mean(0.1, 0.2) == 0.15` fails on the host's rounding; proposed, a
   float is a machine record whose `add` is assumed to round the real
   sum to the nearest.
