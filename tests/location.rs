//! Location (`docs/design/frozen.md`): within is a path prefix.

use mitra_core::location::{within, Location};

#[test]
fn a_place_is_within_what_its_path_goes_on_from() {
    let (file, inner) = (Location::at(&["a.rs"]), Location::at(&["a.rs", "f"]));
    assert!(within(&inner, &file) && within(&file, &file) && !within(&file, &inner));
}
