//! A small seeded random source for the generators: proptest gives the
//! seed, so every failure names the seed that replays it.

pub struct Rng(u64);

#[allow(dead_code)]
impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }
    /// SplitMix64.
    pub fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }
    /// A number from 0 to `n - 1`.
    pub fn below(&mut self, n: usize) -> usize {
        (self.next() % n.max(1) as u64) as usize
    }
    pub fn coin(&mut self) -> bool {
        self.below(2) == 0
    }
    pub fn pick<'a, T>(&mut self, xs: &'a [T]) -> &'a T {
        &xs[self.below(xs.len())]
    }
}

use mitra_core::expression::Expression;

/// `let name = value; body`.
#[allow(dead_code)]
pub fn let_in<Op>(name: &str, value: Expression<Op>, body: Expression<Op>) -> Expression<Op> {
    Expression::Let(name.to_string(), Box::new(value), Box::new(body))
}

/// `e` with every `Let` written out: each name a `Let` binds replaced by
/// what it binds. Inner `Let`s go first, so no name is ever caught.
#[allow(dead_code)]
pub fn inline<Op: Clone>(e: &Expression<Op>) -> Expression<Op> {
    match e {
        Expression::Input(name) => Expression::Input(name.clone()),
        Expression::Apply(op, args) => {
            Expression::Apply(op.clone(), args.iter().map(inline).collect())
        }
        Expression::Let(name, value, body) => put(&inline(body), name, &inline(value)),
    }
}

/// `e`, which has no `Let`, with `with` in place of every `name`.
fn put<Op: Clone>(e: &Expression<Op>, name: &str, with: &Expression<Op>) -> Expression<Op> {
    match e {
        Expression::Input(n) if n == name => with.clone(),
        Expression::Apply(op, args) => Expression::Apply(
            op.clone(),
            args.iter().map(|a| put(a, name, with)).collect(),
        ),
        other => other.clone(),
    }
}
