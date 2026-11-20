//! A location in the system. It is not right or wrong, and nothing is
//! believed of it.

/// Known by its path, from the outside in: a file, then what is in it.
// mitra: implements place::Place
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Place {
    pub path: Vec<String>,
}

impl Place {
    pub fn at(path: &[&str]) -> Place {
        Place {
            path: path.iter().map(|p| p.to_string()).collect(),
        }
    }
}

/// `a` is within `b`: its path goes on from the other's.
// mitra: implements place::within
pub fn within(a: &Place, b: &Place) -> bool {
    a.path.starts_with(&b.path)
}
