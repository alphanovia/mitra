// Generates the LR(1) parser. LALRPOP fails the build on any conflict, so
// a clean build is the proof that the grammar is unambiguous and LR(1).
fn main() {
    lalrpop::process_src().unwrap();
}
