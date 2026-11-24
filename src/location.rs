//! An address in the real system (`docs/design/frozen.md`).

/// An address in the real system: a file, then what is in it, or an
/// object, then a physical part of it. It is not right or wrong. What
/// it promises is that an actor can verify that what it names exists.
// mitra: implements location::Location
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Location {
    pub path: Vec<String>,
}

impl Location {
    pub fn at(path: &[&str]) -> Location {
        Location {
            path: path.iter().map(|p| p.to_string()).collect(),
        }
    }
}

/// `a` is within `b`: its path goes on from the other's.
// mitra: implements location::within
pub fn within(a: &Location, b: &Location) -> bool {
    a.path.starts_with(&b.path)
}
