# Frozen

What the user has frozen: for each item, its meaning, its purpose and
its code with a test. The language has not converged, so no model file
is frozen.

## Location

**Meaning.** An address of something in the real system: a file, then
what is in it. It is known by its path and nothing else. It is not
right or wrong, and nothing is believed of it. A location is within
another when its path goes on from the other's (a path prefix); every
location is within itself.

**Purpose.** The tether between the model and the real system.

**Code.** `src/location.rs`. **Test.** `tests/location.rs`.
