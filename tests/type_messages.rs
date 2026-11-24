//! The message of a type error (M005, M006), as rustc gives one: the
//! expression, the innermost part that has no type underlined with the
//! type of each argument, what the operation takes, and a help when a
//! whole number is the cause. Mitra converts no number.

use mitra_core::check::check;
use mitra_core::parse::parse;
use proptest::prelude::*;

/// Every error of the model's text, as printed.
fn errors(text: &str) -> Vec<String> {
    let model = parse(text).unwrap_or_else(|e| panic!("{}", e.render("t", text)));
    check(&model).iter().map(|e| e.to_string()).collect()
}

fn one(text: &str) -> String {
    let all = errors(text);
    assert_eq!(all.len(), 1, "{all:#?}");
    all[0].clone()
}

#[test]
fn a_whole_number_beside_a_type_parameter() {
    let text = "contract Positive<T: Number> {
    fn f(x: T) -> y: T;
    claim pos(a: T): 0 <= f(a);
}";
    assert_eq!(
        one(text),
        "error[M006]: `pos` is not a truth
 --> Positive::pos
  |
  | 0 <= f(a)
  | ^^^^^^^^^ `0` is integer and `f(a)` is T
  = note: `<=` compares two numbers of one type: integer, rational, or one `T: Number`
  = help: Mitra converts no number; a T comes from a function that gives one, declared beside the others"
    );
}

#[test]
fn the_contract_s_own_function_gives_the_zero() {
    let text = "contract Positive<T: Number> {
    fn zero() -> z: T;
    fn f(x: T) -> y: T;
    claim pos(a: T): zero() <= f(a);
}";
    assert_eq!(errors(text), Vec::<String>::new());
}

#[test]
fn a_whole_number_beside_a_real() {
    let text = "subject s {
    fn g(x: rational) -> y: rational { x } where {
        claim c: y <= 1;
    }
}";
    assert_eq!(
        one(text),
        "error[M006]: `c` is not a truth
 --> s::g::c
  |
  | y <= 1
  | ^^^^^^ `y` is rational and `1` is integer
  = note: `<=` compares two numbers of one type: integer, rational, or one `T: Number`
  = help: Mitra converts no number; write `1.0` for a rational"
    );
}

#[test]
fn the_innermost_part_is_underlined() {
    let text = "contract C<T: Number> {
    fn f(x: T) -> y: T;
    claim a(x: T): let z = f(x) + 1 in f(x) <= z;
}";
    let e = one(text);
    assert!(
        e.contains(
            "  | let z = f(x) + 1 in f(x) <= z\n  |         ^^^^^^^^ `f(x)` is T and `1` is integer"
        ),
        "{e}"
    );
}

#[test]
fn a_call_and_a_record_name_what_does_not_fit() {
    let text = "subject s {
    record P { x: integer, y: rational }
    fn g(x: integer) -> y: rational { 1.0 } where {
        claim e: g(1.0) == y;
        claim h: P { x: 1, y: 2 }.y == y;
    }
}";
    let all = errors(text);
    assert_eq!(all.len(), 2, "{all:#?}");
    assert!(all[0].contains(
        "^^^^^^ `1.0` is rational where `x` is integer\n  = note: `g` takes (x: integer)"
    ));
    assert!(all[1].contains(
        "`2` is integer where `y` is rational\n  = note: `P` takes { x: integer, y: rational }"
    ));
    assert!(all[1].ends_with("= help: Mitra converts no number; write `2.0` for a rational"));
}

#[test]
fn a_body_of_the_wrong_type_says_both() {
    let text = "subject s {
    fn k(x: rational) -> y: integer { x / 2.0 }
}";
    assert_eq!(
        one(text),
        "error[M005]: the body is not of the type of `y`
 --> s::k
  |
  | x / 2.0
  | ^^^^^^^ this is rational
  = note: it should be integer"
    );
}

// ------------------------------------------------------------- red team

/// A random expression over `i: Int`, `r: Real` and `t: T`, typed or not.
fn expr() -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        Just("i".to_string()),
        Just("r".to_string()),
        Just("t".to_string()),
        Just("f(t)".to_string()),
        (0..4i32).prop_map(|n| n.to_string()),
        (0..4i32).prop_map(|n| format!("{n}.5")),
        Just("true".to_string()),
    ];
    leaf.prop_recursive(4, 24, 3, |inner| {
        prop_oneof![
            (inner.clone(), inner.clone(), 0..5usize).prop_map(|(a, b, o)| {
                let op = ["+", "-", "*", "<=", "=="][o];
                format!("({a} {op} {b})")
            }),
            (inner.clone(), inner.clone(), inner.clone())
                .prop_map(|(c, a, b)| format!("(if {c} {{ {a} }} else {{ {b} }})")),
            inner.clone().prop_map(|a| format!("f({a})")),
            (inner.clone(), inner).prop_map(|(a, b)| format!("(let z = {a} in {b})")),
        ]
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(512))]

    /// No panic; each type error underlines text of the expression it
    /// shows, and says what is wrong in the label.
    #[test]
    fn every_type_error_shows_and_underlines(e in expr()) {
        let text = format!(
            "contract C<T: Number> {{\n    fn f(x: T) -> y: T;\n    claim c(i: integer, r: rational, t: T): {e};\n}}"
        );
        let Ok(model) = parse(&text) else { return Ok(()) };
        for error in check(&model) {
            if error.code.to_string() != "M006" {
                continue;
            }
            let shown = error.to_string();
            let lines: Vec<&str> = shown.lines().collect();
            prop_assert!(lines.len() >= 5, "{}", shown);
            let expression = lines[3].strip_prefix("  | ").unwrap();
            let carets = lines[4].strip_prefix("  | ").unwrap();
            let start = carets.find('^').unwrap();
            let width = carets[start..].find(' ').unwrap_or(carets.len() - start);
            let underlined: String = expression.chars().skip(start).take(width).collect();
            prop_assert_eq!(underlined.chars().count(), width, "{}", shown);
            let label = &carets[start + width..];
            prop_assert!(label.contains(" is "), "{}", shown);
            prop_assert!(lines[5..].iter().all(|l| l.starts_with("  = ")), "{}", shown);
        }
    }
}

/// The primitives are lowercase (your "go", 10:21): `integer` and
/// `rational`; `Int` and `Real` name nothing, with no hint.
#[test]
fn numbers_are_integer_and_rational() {
    let text = "subject s { fn f(x: integer, y: rational) -> z: (integer, rational) { (x, y) } }";
    let model = mitra_core::parse::parse(text).unwrap();
    assert_eq!(mitra_core::check::check(&model), vec![]);
    assert_eq!(
        mitra_core::print::print(&model).contains("x: integer, y: rational"),
        true
    );
    for old in ["Int", "Real"] {
        let text = format!("subject s {{ fn f(x: {old}) -> y: bool {{ true }} }}");
        let out: Vec<String> = mitra_core::check::check(&mitra_core::parse::parse(&text).unwrap())
            .iter()
            .map(|e| e.to_string())
            .collect();
        assert_eq!(
            out,
            vec!["error[M003]: the type of `x` names no record or enum of the model as declared\n --> s::f::x"]
        );
    }
}
