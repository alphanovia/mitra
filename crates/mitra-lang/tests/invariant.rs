//! `by invariant` (the user's choice, 2026-11-24 13:30): a claim of a
//! function whose body is a loop, shown as kept by the loop. Written
//! alone, the claim itself is the invariant; `by invariant n` names the
//! turns, 0 at the start, one more each turn, and the length of what a
//! `fold` folds at its end. The start and a turn are each read as a
//! claim: by bounds, the value carried any value that keeps the claim,
//! and as linear facts (`src/linear.rs`), which relate two values.

use mitra_lang::agree::{agree, Settings};
use mitra_lang::check::check;
use mitra_lang::linear::holds;
use mitra_lang::parse::parse;
use mitra_lang::print::print;

fn model(functions: &str) -> String {
    format!("subject calc {{\n{functions}\n}}")
}

fn errors(text: &str) -> Vec<String> {
    check(&parse(text).unwrap())
        .iter()
        .map(|e| e.to_string())
        .collect()
}

fn findings(text: &str) -> Vec<String> {
    let m = parse(text).unwrap();
    assert_eq!(check(&m), vec![], "{text}");
    agree(&m, &Settings::default())
        .1
        .iter()
        .map(|f| f.to_string())
        .collect()
}

const SQUARES: &str = "    fn squares(xs: [integer]) -> s: integer {
        xs.fold(s: 0) { x -> s + x * x }
    } where {
        claim positive: 0 <= s by invariant;
    }";

const COUNT: &str = "    fn count(xs: [integer]) -> c: integer {
        xs.fold(c: 0) { x -> c + 1 }
    } where {
        claim counted: c == n by invariant n;
        claim below: c <= n by invariant n;
        examples { ([4, 5]) -> 2 }
    }";

const PAIR: &str = "    fn pair(xs: [integer]) -> p: (k: integer, t: integer) {
        xs.fold((k: 0, t: 0)) { x -> (k + 1, t + 1) }
    } where {
        claim same: p.k == p.t by invariant;
        claim k: p.k == n by invariant n;
    }";

const HALVE: &str = "    fn halve(a: integer) -> b: integer {
        until(b: a * a, b <= 1) { b / 2 }
    } where {
        claim kept: 0 <= b by invariant;
    }";

const FIVES: &str = "    fn fives(xs: [integer]) -> c: integer {
        xs.fold(c: 0) { x -> 5 }
    } where {
        claim most: c <= 5 * n by invariant n;
        claim more: c + 1 <= n * 5 + 1 by invariant n;
    }";

const DOWN: &str = "    fn down(a: integer) -> b: integer {
        until(b: 10, b <= 1) { b - 1 }
    } where {
        claim kept: 0 <= b by invariant;
    }";

const BELOW: &str = "    fn below(a: integer) -> b: integer {
        until(b: a, b <= 1) { b - 1 }
    } where {
        claim kept: b <= a by invariant;
    }";

/// Each holds by its invariant, and none is decided without it. `fives`
/// needs `0 <= n`; `down`, that the loop is not done on a turn; `below`,
/// linear facts at the start, `a <= a`.
#[test]
fn a_claim_kept_by_its_loop_holds() {
    for f in [SQUARES, COUNT, PAIR, HALVE, FIVES, DOWN, BELOW] {
        assert_eq!(findings(&model(f)), Vec::<String>::new(), "{f}");
        let alone = f
            .replace(" by invariant n;", ";")
            .replace(" by invariant;", ";")
            .replace("c == n", "c == xs.len()")
            .replace("c <= n", "c <= xs.len()")
            .replace("p.k == n", "p.k == xs.len()")
            .replace("5 * n", "5 * xs.len()")
            .replace("n * 5", "xs.len() * 5");
        let found = findings(&model(&alone));
        assert!(
            !found.is_empty() && found.iter().all(|f| f.starts_with("finding[A007]")),
            "{alone}: {found:?}"
        );
    }
}

/// Of what the function gives, `n` is the length of the sequence folded:
/// an example is read so.
#[test]
fn n_is_the_length_folded_at_the_end() {
    let wrong = COUNT.replace("([4, 5]) -> 2", "([4, 5]) -> 3");
    let found = findings(&model(&wrong));
    assert!(
        found.iter().any(|f| f.starts_with(
            "finding[A003]: example 0 against claim `counted`: false at xs = [4, 5], giving 3"
        )),
        "{found:?}"
    );
}

/// What the loop does not keep stays unknown, and says why: its start,
/// or a turn. False where the body breaks it.
#[test]
fn a_claim_its_loop_does_not_keep_says_why() {
    // True, but no invariant: false at the start.
    let f = "    fn count(xs: [integer]) -> c: integer {
        xs.fold(c: 0) { x -> c + 1 }
    } where {
        claim counted: c == xs.len() by invariant;
    }";
    let found = findings(&model(f));
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("; `by invariant`: false at the loop's start, at xs = ["),
        "{found:?}"
    );
    // True, but not kept by a turn alone.
    let f = "    fn count(xs: [integer]) -> c: integer {
        xs.fold(c: 0) { x -> c + 1 }
    } where {
        claim within: c <= xs.len() by invariant;
    }";
    let found = findings(&model(f));
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains("; `by invariant`: a turn is not decided: "),
        "{found:?}"
    );
    // False: the body breaks it.
    let f = SQUARES.replace("0 <= s by invariant", "s <= 3 by invariant");
    let found = findings(&model(&f));
    assert!(
        found[0].starts_with("finding[A001]: the body breaks claim `positive` at xs = ["),
        "{found:?}"
    );
}

/// M033: a claim `by invariant` of what is not a loop's function.
#[test]
fn by_invariant_is_of_a_loop() {
    let cases = [
        (
            "    fn f(a: integer) -> b: integer { a } where { claim c: b == a by invariant; }",
            "error[M033]: the body of `f` is no loop: `by invariant` is on a function whose body is `xs.fold(…) { … }` or `until(…) { … }`\n --> calc::f::c",
        ),
        (
            "    fn f(a: integer) -> b: integer { until(b: a, b <= 1) { b / 2 } } where { claim c: b <= n by invariant n; }",
            "error[M033]: `by invariant n` counts the turns of a `fold`; an `until` has no count to end at: write `by invariant`\n --> calc::f::c",
        ),
        (
            "    fn f(xs: [integer], n: integer) -> c: integer { xs.fold(c: 0) { x -> c + 1 } } where { claim c: c == n by invariant n; }",
            "error[M033]: `n` is already a name of `f` or of its loop: the turns take a name of their own\n --> calc::f::c",
        ),
        (
            "    fn f(xs: [integer]) -> c: integer { xs.fold(c: 0) { x -> c + 1 } } where { claim c: c == x by invariant x; }",
            "error[M033]: `x` is already a name of `f` or of its loop: the turns take a name of their own\n --> calc::f::c",
        ),
    ];
    for (f, error) in cases {
        assert_eq!(errors(&model(f)), [error], "{f}");
    }
    let record = "subject record R { a: integer } where { claim c: self.a == 0 by invariant; }";
    assert_eq!(
        errors(record),
        ["error[M033]: a claim of a record is not shown `by invariant`: only a function's whose body is a loop\n --> R::R::c"]
    );
}

/// Printed as written: `by invariant n`, the claim without the length.
#[test]
fn by_invariant_prints_as_written() {
    let text = model(&[SQUARES, COUNT, PAIR, HALVE].join("\n"));
    let printed = print(&parse(&text).unwrap());
    assert!(
        printed.contains("claim counted: c == n by invariant n;"),
        "{printed}"
    );
    assert!(
        printed.contains("claim positive: 0 <= s by invariant;"),
        "{printed}"
    );
    assert_eq!(parse(&printed).unwrap(), parse(&text).unwrap());
}

/// Linear facts: true of any values, or from what is known, by a
/// number; never more.
#[test]
fn linear_facts_decide_only_what_follows() {
    let e = |s: &str| {
        parse(&format!(
            "subject s {{ fn f(a: integer, b: integer, c: integer, xs: [integer]) -> r: bool {{ {s} }} }}"
        ))
        .unwrap()
        .parts[0]
            .functions[0]
            .body
            .clone()
            .unwrap()
    };
    assert!(holds(&[], &e("a + 1 == 1 + a")));
    assert!(holds(&[], &e("a <= a || b < 0")));
    assert!(holds(&[e("a == b")], &e("a + 1 == b + 1")));
    assert!(holds(&[e("a <= b")], &e("a + 1 <= b + 2")));
    assert!(holds(&[e("a < b")], &e("a <= b")));
    assert!(holds(
        &[e("a == xs.len()")],
        &e("(xs ++ [1, 2]).len() == a + 2")
    ));
    assert!(!holds(&[], &e("a == b")));
    assert!(!holds(&[e("a <= b")], &e("b <= a")));
    assert!(!holds(&[e("a <= b")], &e("a < b")));
    assert!(!holds(&[e("a <= b")], &e("a + 1 <= b")));
    assert!(!holds(&[e("a == b * c")], &e("a == c * b")));
    assert!(!holds(&[e("a == b || a == c")], &e("a == b")));
    assert!(!holds(&[e("a == b")], &e("a == c")));
    assert!(!holds(&[], &e("a / 2 <= a")));
    assert!(holds(&[], &e("2 * a == a + a")));
    assert!(holds(&[e("0 <= a")], &e("0 <= 3 * a")));
    assert!(!holds(&[e("0 <= a")], &e("0 <= -3 * a")));
    assert!(!holds(&[], &e("2 * a == a")));
    assert!(!holds(&[e("a <= b")], &e("b == a")));
    assert!(!holds(&[e("!(a == b)")], &e("a == b")));
}
