//! Print the generated program for a seed: `cargo run -p mitra-grammar --example sample -- 7`
fn main() {
    let seed = std::env::args().nth(1).and_then(|s| s.parse().ok()).unwrap_or(0);
    let mut r = mitra_grammar::gen::Rng::new(seed);
    println!("-- model\n{}-- evidence\n{}", mitra_grammar::gen::model(&mut r), mitra_grammar::gen::evidences(&mut r));
}
