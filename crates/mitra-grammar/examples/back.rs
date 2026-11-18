//! Print a generated model as written and as the printer gives it back,
//! and what differs when that is read again:
//! `cargo run -p mitra-grammar --example back -- 7`
fn main() {
    let seed = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);
    let mut r = mitra_grammar::gen::Rng::new(seed);
    let src = mitra_grammar::gen::model(&mut r);
    println!("{src}");
    let m = match mitra_core::parse_model(&src) {
        Ok(m) => m,
        Err(e) => return println!("-- error\n{e:?}"),
    };
    let printed = mitra_core::print_model(&m);
    println!("-- printed\n{printed}");
    match mitra_core::parse_model(&printed) {
        Ok(again) => {
            let twice = mitra_core::print_model(&again);
            println!("-- same text again: {}", twice == printed);
            println!(
                "-- same functions: {}",
                again
                    .functions
                    .iter()
                    .zip(&m.functions)
                    .all(|(a, b)| a.params == b.params && a.range == b.range && a.body == b.body)
            );
            println!(
                "-- same records: {}",
                again
                    .records
                    .iter()
                    .zip(&m.records)
                    .all(|(a, b)| a.fields == b.fields)
            );
            println!(
                "-- same deep: {} {}",
                again.deep == m.deep,
                again.recursive == m.recursive
            );
        }
        Err(e) => println!("-- error on the printed text\n{e:?}"),
    }
}
