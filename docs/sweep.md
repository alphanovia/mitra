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
starts at one half. The weight is 0.2. An opinion is written (yes,
no) and what it reads at.

## How to read it

- **Gravity** gets a yes from every passing measurement of every
  object. With a weight of 0.2 it is settled almost at once: 0.88
  after one object measured once, 0.99 after ten.
- **The mass of an object** gets only its own measurements: 0.91
  after one, 0.99 after ten, whatever the other objects.
- **The feather's mass** takes nearly all of each of its failures. A
  hundred objects and one feather read: gravity 1.00, the feather's
  mass 0.16 after one failure, 0.02 after ten.
- **The feather's measurements** stay believed (0.83): they were
  trusted at 0.9 and something less believed is there to blame.
- **One voice per object** counts once however many measurements:
  ten or a hundred read as one.
- **Noisy objects.** On an object that passes 19 of 20, the one
  failing measurement takes the no and reads 0.19, and the mass stays
  at 0.99: the measurement is the odd one. On an object that passes
  12 of 20, the mass still reads 0.94 and each failing measurement
  0.22: its measurements are what is doubted. On the feather, 0 of 20,
  the mass goes to 0.01 and the measurements stay at 0.83: the object
  is the odd one.
- **Many objects failing.** Gravity stays high as long as some
  objects pass (0.98 with 8 of 10 failing) and falls only when none
  does (0.02). As written, any pass says yes to gravity, so a failure
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

## With a weight of 2 instead

The same sweep, for the three pieces that matter, at 10 measurements
per object:

| objects that pass | gravity, W 0.2 | gravity, W 2 | feather's mass, W 0.2 | feather's mass, W 2 |
|---|---|---|---|---|
| 1 | 0.98 | 0.86 | 0.02 | 0.13 |
| 10 | 1.00 | 0.99 | 0.02 | 0.11 |
| 100 | 1.00 | 1.00 | 0.02 | 0.11 |

And at one measurement per object: gravity 0.88 against 0.62 with one
object, 0.99 against 0.90 with ten; the feather's mass 0.17 against
0.38 with ten objects. The noisy object that passes 19 of 20 keeps
its failing measurement at 0.71 with W 2, against 0.19 here.

### What gravity reads at

Rows: objects that pass. Columns: measurements per object.

| objects | 1 | 2 | 5 | 10 | 20 | 50 | 100 |
|---|---|---|---|---|---|---|---|
| 1 | 0.88 | 0.93 | 0.96 | 0.98 | 0.99 | 1.00 | 1.00 |
| 2 | 0.94 | 0.97 | 0.98 | 0.99 | 1.00 | 1.00 | 1.00 |
| 5 | 0.98 | 0.99 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 |
| 10 | 0.99 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |
| 20 | 0.99 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |
| 50 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |
| 100 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 | 1.00 |

### What the mass of an object that passes reads at

Rows: objects that pass. Columns: measurements per object.

| objects | 1 | 2 | 5 | 10 | 20 | 50 | 100 |
|---|---|---|---|---|---|---|---|
| 1 | 0.91 | 0.95 | 0.98 | 0.99 | 0.99 | 1.00 | 1.00 |
| 2 | 0.91 | 0.95 | 0.98 | 0.99 | 0.99 | 1.00 | 1.00 |
| 5 | 0.91 | 0.95 | 0.98 | 0.99 | 0.99 | 1.00 | 1.00 |
| 10 | 0.91 | 0.95 | 0.98 | 0.99 | 0.99 | 1.00 | 1.00 |
| 20 | 0.91 | 0.95 | 0.98 | 0.99 | 0.99 | 1.00 | 1.00 |
| 50 | 0.91 | 0.95 | 0.98 | 0.99 | 0.99 | 1.00 | 1.00 |
| 100 | 0.91 | 0.95 | 0.98 | 0.99 | 0.99 | 1.00 | 1.00 |

### What the mass of the feather reads at

Rows: objects that pass. Columns: measurements per object.

| objects | 1 | 2 | 5 | 10 | 20 | 50 | 100 |
|---|---|---|---|---|---|---|---|
| 1 | 0.21 | 0.12 | 0.05 | 0.02 | 0.01 | 0.00 | 0.00 |
| 2 | 0.19 | 0.11 | 0.05 | 0.02 | 0.01 | 0.00 | 0.00 |
| 5 | 0.17 | 0.10 | 0.04 | 0.02 | 0.01 | 0.00 | 0.00 |
| 10 | 0.17 | 0.10 | 0.04 | 0.02 | 0.01 | 0.00 | 0.00 |
| 20 | 0.16 | 0.10 | 0.04 | 0.02 | 0.01 | 0.00 | 0.00 |
| 50 | 0.16 | 0.10 | 0.04 | 0.02 | 0.01 | 0.00 | 0.00 |
| 100 | 0.16 | 0.09 | 0.04 | 0.02 | 0.01 | 0.00 | 0.00 |

### What one measurement of the feather reads at

Rows: objects that pass. Columns: measurements per object.

| objects | 1 | 2 | 5 | 10 | 20 | 50 | 100 |
|---|---|---|---|---|---|---|---|
| 1 | 0.84 | 0.84 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 |
| 2 | 0.84 | 0.84 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 |
| 5 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 |
| 10 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 |
| 20 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 |
| 50 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 |
| 100 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 | 0.83 |

### The counts, at 10 measurements per object

| objects that pass | gravity | mass of one that passes | mass of the feather | one measurement of the feather |
|---|---|---|---|---|
| 1 | (9.0, 0.08) 0.98 | (9.0, 0.00) 0.99 | (0.0, 4.03) 0.02 | (0.0, 0.02) 0.83 |
| 2 | (18.0, 0.04) 0.99 | (9.0, 0.00) 0.99 | (0.0, 4.15) 0.02 | (0.0, 0.02) 0.83 |
| 5 | (45.0, 0.02) 1.00 | (9.0, 0.00) 0.99 | (0.0, 4.23) 0.02 | (0.0, 0.02) 0.83 |
| 10 | (90.0, 0.01) 1.00 | (9.0, 0.00) 0.99 | (0.0, 4.26) 0.02 | (0.0, 0.02) 0.83 |
| 20 | (180.0, 0.00) 1.00 | (9.0, 0.00) 0.99 | (0.0, 4.27) 0.02 | (0.0, 0.02) 0.83 |
| 50 | (450.0, 0.00) 1.00 | (9.0, 0.00) 0.99 | (0.0, 4.28) 0.02 | (0.0, 0.02) 0.83 |
| 100 | (900.0, 0.00) 1.00 | (9.0, 0.00) 0.99 | (0.0, 4.28) 0.02 | (0.0, 0.02) 0.83 |

### One voice per object instead (10 objects pass)

| measurements per object | gravity | mass of one that passes | mass of the feather |
|---|---|---|---|
| 1 | (9.0, 0.01) 0.99 | (0.9, 0.00) 0.91 | (0.0, 0.40) 0.17 |
| 10 | (9.0, 0.01) 0.99 | (0.9, 0.00) 0.91 | (0.0, 0.40) 0.17 |
| 100 | (9.0, 0.01) 0.99 | (0.9, 0.00) 0.91 | (0.0, 0.40) 0.17 |

### Noisy objects: 20 measurements each

16 objects pass 19 of 20, 3 odd ones pass 12 of 20, the feather 0 of 20.

| piece | counts and what it reads at |
|---|---|
| `gravity` | (306.0, 0.11) 1.00 |
| `mass o0` | (17.1, 0.05) 0.99 |
| `mass odd0` | (10.8, 0.56) 0.94 |
| `mass feather` | (0.0, 8.56) 0.01 |
| `drop o0 0` | (0.0, 0.00) 0.90 |
| `drop o0 19` | (0.0, 0.73) 0.19 |
| `drop odd0 0` | (0.0, 0.00) 0.90 |
| `drop odd0 19` | (0.0, 0.64) 0.22 |
| `drop feather 0` | (0.0, 0.02) 0.83 |

### How many of the objects fail (10 objects, 10 measurements each)

| objects that fail every time | gravity | mass of one that passes | mass of one that fails |
|---|---|---|---|
| 0 | (90.0, 0.00) 1.00 | (9.0, 0.00) 0.99 | none |
| 1 | (81.0, 0.01) 1.00 | (9.0, 0.00) 0.99 | (0.0, 4.26) 0.02 |
| 2 | (72.0, 0.02) 1.00 | (9.0, 0.00) 0.99 | (0.0, 4.25) 0.02 |
| 5 | (45.0, 0.09) 1.00 | (9.0, 0.00) 0.99 | (0.0, 4.23) 0.02 |
| 8 | (18.0, 0.34) 0.98 | (9.0, 0.00) 0.99 | (0.0, 4.15) 0.02 |
| 10 | (0.0, 6.38) 0.02 | none | (0.0, 0.64) 0.12 |
