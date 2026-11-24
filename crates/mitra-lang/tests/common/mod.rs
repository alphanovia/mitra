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

use mitra_lang::expression::{substitute, Arg, Expression};

/// `op` on arguments that bind no name.
#[allow(dead_code)]
pub fn apply<Op>(op: Op, args: Vec<Expression<Op>>) -> Expression<Op> {
    let args = args.into_iter().map(|body| Arg {
        binds: vec![],
        body,
    });
    Expression::Apply(op, args.collect())
}

/// `let name = value; body`, where `op` is the standard's `let`.
#[allow(dead_code)]
pub fn let_in<Op>(
    op: Op,
    name: &str,
    value: Expression<Op>,
    body: Expression<Op>,
) -> Expression<Op> {
    let value = Arg {
        binds: vec![],
        body: value,
    };
    let body = Arg {
        binds: vec![name.to_string()],
        body,
    };
    Expression::Apply(op, vec![value, body])
}

/// `e` with every `let` written out: what each binds written in place of
/// its name, by `substitute`. `is_let` says which operation is `let`.
#[allow(dead_code)]
pub fn inline<Op: Clone>(e: &Expression<Op>, is_let: &impl Fn(&Op) -> bool) -> Expression<Op> {
    match e {
        Expression::Name(_) => e.clone(),
        Expression::Apply(op, args) => match args.as_slice() {
            [value, body] if is_let(op) && value.binds.is_empty() && body.binds.len() == 1 => {
                let value = inline(&value.body, is_let);
                substitute(&inline(&body.body, is_let), &body.binds[0], &value)
            }
            _ => {
                let args = args.iter().map(|arg| Arg {
                    binds: arg.binds.clone(),
                    body: inline(&arg.body, is_let),
                });
                Expression::Apply(op.clone(), args.collect())
            }
        },
    }
}
