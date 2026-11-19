// mitra: implements Model
#[derive(Debug, Clone)]
pub struct Model {
    pub claims: Vec<Claim>,
}

// mitra: implements Claim
#[derive(Debug, Clone)]
pub struct Claim {
    pub name: String,
    /// Where it is written.
    pub line: usize,
    pub from: Vec<String>,
}

// The model says `y: Int`.
// mitra: implements Point
pub struct Point {
    pub x: i64,
    pub y: String,
}

// The model states no `b`, and has no `..`.
// mitra: implements Wide
pub struct Wide {
    pub a: i64,
    pub b: i64,
}

// mitra: implements check::named
pub fn named(c: &Claim) -> String {
    c.name.clone()
}

// mitra: implements check::ok
pub fn ok(m: &Model) -> bool {
    !m.claims.is_empty()
}
