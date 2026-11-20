//! How long a million evaluations take through the standard table.
//! `cargo run --release --example bench`

use mitra_core::standard::*;
use mitra_core::value::Expression::*;
use mitra_core::value::*;

fn main() {
    let x = || Input("x".to_string(), whole());
    let n = |k: i64| Given(number(k));
    // (x + 1) * 2 < 100 && !(x == 3)
    let e = Apply(
        and(),
        vec![
            Apply(
                less(),
                vec![
                    Apply(mul(), vec![Apply(add(), vec![x(), n(1)]), n(2)]),
                    n(100),
                ],
            ),
            Apply(not(), vec![Apply(equal(&whole()), vec![x(), n(3)])]),
        ],
    );
    let table = Table::default();
    let start = std::time::Instant::now();
    let mut yes_ = 0u64;
    for k in 0..1_000_000i64 {
        let inputs = [("x".to_string(), number(k % 97))];
        if eval(&e, &inputs, &table) == Some(yes()) {
            yes_ += 1;
        }
    }
    println!("{yes_} true, {:?} per million", start.elapsed());
}
