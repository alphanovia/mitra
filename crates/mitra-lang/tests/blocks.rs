//! Blocks, as in Rust (BDD step 2, user's choice B): `{ let x = a; b }`
//! names a value for the rest of the block, `e; r` runs `e` for what it
//! does and gives `r`, and `let x = a in b` is no longer read.

use mitra_lang::agree::agree;
use mitra_lang::check::check;
use mitra_lang::parse::parse;
use mitra_lang::print::print;

const MODEL: &str = "environment Disk {
    fn write(path: [~char], body: [~char]) -> ok: ();
}
subject Notes {
    use Disk::write;
    fn save(n: [~char]) -> r: integer { write(\"a.txt\", n); n.len() } where {
        examples { (\"ab\") -> 2 }
    }
    fn touch(n: [~char]) -> r: () { write(\"a.txt\", n); } where {
        examples { (\"ab\") -> () }
    }
    fn far(x: integer) -> d: integer { let a = x; let b = { let c = a; c * c }; a + b } where {
        claim p: { let z = d - x; z * 1 == z };
        examples { (2) -> 6, (3) -> 12 }
    }
    fn sum(xs: [integer]) -> s: integer { xs.fold(t: 0) { x -> let y = x * 2; t + y } } where {
        examples { ([1, 2]) -> 6 }
    }
    fn pick(x: integer) -> d: integer { if x < 0 { let y = 0 - x; y } else { x } } where {
        examples { (-2) -> 2, (3) -> 3 }
    }
}";

fn shown(text: &str) -> Vec<String> {
    match parse(text) {
        Ok(m) => {
            let mut all: Vec<String> = check(&m).iter().map(|e| e.to_string()).collect();
            all.extend(
                agree(&m, &Default::default())
                    .1
                    .iter()
                    .map(|f| f.to_string()),
            );
            all
        }
        Err(e) => vec![e.to_string()],
    }
}

#[test]
fn blocks_read_and_their_rows_hold() {
    let shown = shown(MODEL);
    assert!(
        !shown
            .iter()
            .any(|f| f.starts_with("error") || f.contains("A002")),
        "{shown:?}"
    );
}

#[test]
fn a_block_prints_as_written_and_reads_back() {
    let m = parse(MODEL).unwrap();
    let printed = print(&m);
    assert_eq!(parse(&printed).unwrap(), m, "{printed}");
    for written in [
        "{ write(\"a.txt\", n); n.len() }",
        "{ write(\"a.txt\", n); }",
        "{ let a = x; let b = { let c = a; c * c }; a + b }",
        "claim p: { let z = d - x; z * 1 == z };",
        "if x < 0 { let y = 0 - x; y } else { x }",
        "xs.fold(t: 0) { x -> let y = x * 2; t + y }",
    ] {
        assert!(printed.contains(written), "{written} in {printed}");
    }
}

#[test]
fn a_dropped_value_that_is_not_unit_is_p016() {
    let e = shown("subject N {\n    fn f(x: integer) -> y: integer { x + 1; x }\n}");
    assert!(
        e[0].starts_with("error[P016]: `;` drops what this gives, of type `integer`"),
        "{e:?}"
    );
    // An environment's answer dropped, too.
    let e = shown("environment E {\n    fn r() -> v: integer;\n}\nsubject M {\n    use E::r;\n    fn h(x: integer) -> y: integer { r(); x }\n}");
    assert!(e[0].starts_with("error[P016]"), "{e:?}");
}

#[test]
fn the_old_let_says_how_to_write_it() {
    for text in [
        "subject N {\n    fn g(x: integer) -> y: integer { let a = 1 in a }\n}",
        "subject N {\n    fn g(x: integer) -> y: integer { x } where {\n        claim c: let a = y in a == x;\n    }\n}",
    ] {
        let e = shown(text);
        assert!(
            e[0].starts_with("error[P003]") && e[0].contains("{ let"),
            "{e:?}"
        );
    }
    // A `let` needs `;`, then a last expression.
    let e = shown("subject N {\n    fn g(x: integer) -> y: integer { let a = 1; }\n}");
    assert!(
        e[0].starts_with("error[P003]: expected an expression after `let`"),
        "{e:?}"
    );
}

#[test]
fn a_proof_names_in_a_block() {
    let text = "subject s {
    fn f(x: integer) -> y: integer { x } where {
        claim c: y == x;
        claim d: y - x == 0;
        rule r: f::c |- f::d;
        rule again: f::r, f::c |- f::d by { let s = f::r; ponens(s, f::c) };
        rule twice: f::r, f::c |- f::d by { let s = f::r; let t = f::c; ponens(s, t) };
    }
}";
    let m = parse(text).unwrap();
    let printed = print(&m);
    assert!(
        printed.contains("by { let s = f::r; let t = f::c; ponens(s, t) }"),
        "{printed}"
    );
    assert!(
        printed.contains("by { let s = f::r; ponens(s, f::c) }"),
        "{printed}"
    );
    assert_eq!(parse(&printed).unwrap(), m);
    let old = text.replace(
        "by { let s = f::r; ponens(s, f::c) }",
        "by let s = f::r in ponens(s, f::c)",
    );
    let e = parse(&old).unwrap_err().to_string();
    assert!(e.starts_with("error[P003]") && e.contains("{ let"), "{e}");
}
