# The sweep: objects that fall at the same speed

What `cargo run --release --example sweep` prints, with how to read
it. The experiment is "objects fall at the same speed". The model
holds only gravity and mass, and no air: `gravity` is a latent
statement shared by every object, `mass k` is the statement tied to
object k, and "object k falls at the same speed" is all of
[`gravity`, `mass k`]. Some objects pass every measurement; a feather
fails every one.

Each measurement is evidence about the whole of its object, believed
at 0.9. **Each measurement has a voice of its own**, so the counts
add, except in the one table that says otherwise. Everything else
starts at one half. An opinion is written (yes, no) and what it
reads at.

## How to read it

- **Gravity** gets a yes from every passing measurement of every
  object: it grows with objects times measurements. One feather takes
  almost nothing from it once a few objects pass.
- **The mass of an object** gets only its own measurements: it does
  not depend on how many other objects there are.
- **The feather's mass** takes nearly all of each of its failures, the
  more so as gravity is believed through the others. A hundred
  objects and one feather read: gravity 0.99, the feather's mass 0.37
  after one failure, 0.11 after ten.
- **The feather's measurements** stay believed (0.87): they were
  trusted at 0.9 and something less believed is there to blame.
- **One voice per object** counts once however many measurements:
  ten or a hundred read as one.
- **Noisy objects.** On an object that passes 19 of 20, the one
  failing measurement takes more (0.53 of a no, read at 0.71) than
  the mass (0.30, still 0.93): the measurement is the odd one. On an
  object that passes 12 of 20, the mass and each failing measurement
  both read 0.74. On the feather, 0 of 20, the mass goes to 0.06 and
  the measurements stay at 0.87: the object is the odd one.
- **Many objects failing.** Gravity stays high as long as some
  objects pass (0.83 with 8 of 10 failing) and falls only when none
  does (0.04). As written, any pass says yes to gravity, so a failure
  can only be the object's own piece.

## What it cannot say

1. **"The model is missing something."** The blame lands on the piece
   of the failing object, here called `mass`. Nothing in the model
   stands for "gravity and mass are all there is", so nothing can be
   doubted for it. With such a piece shared by every object it would
   behave like `gravity` does here: believed through every pass.
2. **"The measurement of this object is unreliable"**, as one thing.
   Each failing measurement is doubted on its own. A piece for the
   method on one object would sit in the same whole as its mass, and
   the two could not be told apart by these measurements alone.
3. **Eight objects of ten failing is not alarming to it.** That reads
   wrong to me: a person would doubt the model long before the last
   object fails.

### What gravity reads at

Rows: objects that pass. Columns: measurements per object.

| objects | 1 | 2 | 5 | 10 | 20 | 50 | 100 |
|---|---|---|---|---|---|---|---|
| 1 | 0.62 | 0.68 | 0.79 | 0.86 | 0.92 | 0.96 | 0.98 |
| 2 | 0.71 | 0.79 | 0.88 | 0.93 | 0.96 | 0.98 | 0.99 |
| 5 | 0.83 | 0.90 | 0.95 | 0.98 | 0.99 | 0.99 | 1.00 |
| 10 | 0.90 | 0.95 | 0.98 | 0.99 | 0.99 | 1.00 | 1.00 |
| 20 | 0.95 | 0.97 | 0.99 | 0.99 | 1.00 | 1.00 | 1.00 |
| 50 | 0.98 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |
| 100 | 0.99 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |

### What the mass of an object that passes reads at

Rows: objects that pass. Columns: measurements per object.

| objects | 1 | 2 | 5 | 10 | 20 | 50 | 100 |
|---|---|---|---|---|---|---|---|
| 1 | 0.66 | 0.74 | 0.85 | 0.91 | 0.95 | 0.98 | 0.99 |
| 2 | 0.66 | 0.74 | 0.85 | 0.91 | 0.95 | 0.98 | 0.99 |
| 5 | 0.66 | 0.74 | 0.85 | 0.91 | 0.95 | 0.98 | 0.99 |
| 10 | 0.66 | 0.74 | 0.85 | 0.91 | 0.95 | 0.98 | 0.99 |
| 20 | 0.66 | 0.74 | 0.85 | 0.91 | 0.95 | 0.98 | 0.99 |
| 50 | 0.66 | 0.74 | 0.85 | 0.91 | 0.95 | 0.98 | 0.99 |
| 100 | 0.66 | 0.74 | 0.85 | 0.91 | 0.95 | 0.98 | 0.99 |

### What the mass of the feather reads at

Rows: objects that pass. Columns: measurements per object.

| objects | 1 | 2 | 5 | 10 | 20 | 50 | 100 |
|---|---|---|---|---|---|---|---|
| 1 | 0.43 | 0.36 | 0.22 | 0.13 | 0.07 | 0.03 | 0.01 |
| 2 | 0.42 | 0.34 | 0.20 | 0.12 | 0.06 | 0.03 | 0.01 |
| 5 | 0.40 | 0.31 | 0.19 | 0.11 | 0.06 | 0.03 | 0.01 |
| 10 | 0.38 | 0.30 | 0.18 | 0.11 | 0.06 | 0.03 | 0.01 |
| 20 | 0.38 | 0.29 | 0.18 | 0.11 | 0.06 | 0.03 | 0.01 |
| 50 | 0.37 | 0.29 | 0.18 | 0.11 | 0.06 | 0.03 | 0.01 |
| 100 | 0.37 | 0.29 | 0.17 | 0.11 | 0.06 | 0.03 | 0.01 |

### What one measurement of the feather reads at

Rows: objects that pass. Columns: measurements per object.

| objects | 1 | 2 | 5 | 10 | 20 | 50 | 100 |
|---|---|---|---|---|---|---|---|
| 1 | 0.89 | 0.88 | 0.88 | 0.88 | 0.88 | 0.87 | 0.87 |
| 2 | 0.88 | 0.88 | 0.88 | 0.88 | 0.87 | 0.87 | 0.87 |
| 5 | 0.88 | 0.88 | 0.87 | 0.87 | 0.87 | 0.87 | 0.87 |
| 10 | 0.88 | 0.88 | 0.87 | 0.87 | 0.87 | 0.87 | 0.87 |
| 20 | 0.88 | 0.87 | 0.87 | 0.87 | 0.87 | 0.87 | 0.87 |
| 50 | 0.87 | 0.87 | 0.87 | 0.87 | 0.87 | 0.87 | 0.87 |
| 100 | 0.87 | 0.87 | 0.87 | 0.87 | 0.87 | 0.87 | 0.87 |

### The counts, at 10 measurements per object

| objects that pass | gravity | mass of one that passes | mass of the feather | one measurement of the feather |
|---|---|---|---|---|
| 1 | (9.0, 0.64) 0.86 | (9.0, 0.00) 0.91 | (0.0, 6.00) 0.13 | (0.0, 0.05) 0.88 |
| 2 | (18.0, 0.38) 0.93 | (9.0, 0.00) 0.91 | (0.0, 6.63) 0.12 | (0.0, 0.06) 0.88 |
| 5 | (45.0, 0.17) 0.98 | (9.0, 0.00) 0.91 | (0.0, 7.11) 0.11 | (0.0, 0.06) 0.87 |
| 10 | (90.0, 0.09) 0.99 | (9.0, 0.00) 0.91 | (0.0, 7.30) 0.11 | (0.0, 0.06) 0.87 |
| 20 | (180.0, 0.04) 0.99 | (9.0, 0.00) 0.91 | (0.0, 7.40) 0.11 | (0.0, 0.06) 0.87 |
| 50 | (450.0, 0.02) 1.00 | (9.0, 0.00) 0.91 | (0.0, 7.46) 0.11 | (0.0, 0.06) 0.87 |
| 100 | (900.0, 0.01) 1.00 | (9.0, 0.00) 0.91 | (0.0, 7.48) 0.11 | (0.0, 0.06) 0.87 |

### One voice per object instead (10 objects pass)

| measurements per object | gravity | mass of one that passes | mass of the feather |
|---|---|---|---|
| 1 | (9.0, 0.06) 0.90 | (0.9, 0.00) 0.66 | (0.0, 0.60) 0.38 |
| 10 | (9.0, 0.06) 0.90 | (0.9, 0.00) 0.66 | (0.0, 0.60) 0.38 |
| 100 | (9.0, 0.06) 0.90 | (0.9, 0.00) 0.66 | (0.0, 0.60) 0.38 |

### Noisy objects: 20 measurements each

16 objects pass 19 of 20, 3 odd ones pass 12 of 20, the feather 0 of 20.

| piece | counts and what it reads at |
|---|---|
| `gravity` | (306.0, 0.72) 0.99 |
| `mass o0` | (17.1, 0.30) 0.93 |
| `mass odd0` | (10.8, 3.10) 0.74 |
| `mass feather` | (0.0, 14.88) 0.06 |
| `drop o0 0` | (0.0, 0.00) 0.90 |
| `drop o0 19` | (0.0, 0.53) 0.71 |
| `drop odd0 0` | (0.0, 0.00) 0.90 |
| `drop odd0 19` | (0.0, 0.43) 0.74 |
| `drop feather 0` | (0.0, 0.06) 0.87 |

### How many of the objects fail (10 objects, 10 measurements each)

| objects that fail every time | gravity | mass of one that passes | mass of one that fails |
|---|---|---|---|
| 0 | (90.0, 0.00) 0.99 | (9.0, 0.00) 0.91 | none |
| 1 | (81.0, 0.10) 0.99 | (9.0, 0.00) 0.91 | (0.0, 7.28) 0.11 |
| 2 | (72.0, 0.22) 0.98 | (9.0, 0.00) 0.91 | (0.0, 7.25) 0.11 |
| 5 | (45.0, 0.84) 0.96 | (9.0, 0.00) 0.91 | (0.0, 7.11) 0.11 |
| 8 | (18.0, 3.00) 0.83 | (9.0, 0.00) 0.91 | (0.0, 6.63) 0.12 |
| 10 | (0.0, 21.43) 0.04 | none | (0.0, 2.14) 0.24 |

real	0m3.477s
user	0m3.506s
sys	0m0.064s
