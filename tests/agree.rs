//! Agree (`src/agree.rs`): the model against itself, with no evidence.
//! Every code of suit `A` is given, each by one change to a model that
//! agrees.

use mitra_core::agree::{agree, Agreement, Settings};
use mitra_core::catalogue::{Suit, CATALOGUE};
use mitra_core::check::check;
use mitra_core::model::Model;
use mitra_core::parse::parse;

fn model(text: &str) -> Model {
    let model = parse(text).unwrap();
    assert_eq!(check(&model), vec![], "{text}");
    model
}

const DIV: &str = r#"
subject calc {
    fn div(a: Int, b: Int) -> q: Int where {
        claim spec = if b == 0 { q == 0 } else { q * b <= a && a < (q + 1) * b };
        claim bounded = b == 0 || q <= a;
        examples { (6, 2) -> 3, (7, 2) -> 3, (9, 3) -> 3, (5, 0) -> 0 }
    }
}
"#;

const FALL: &str = r#"
environment sun {
    fn light() -> w: Real;
}
subject stone {
    use sun::light;
    fn speed(h: Real) -> v2: Real { 2.0 * 9.81 * h * light() } where {
        claim slow = !(h <= 20.0) || v2 < 400.0;
    }
    fn square(h: Real) -> s: Real { h * h } where {
        claim small = !(h <= 20.0) || !(0.0 <= h) || s <= 400.0;
        examples { (5.0) -> 25.0 }
    }
    fn lit(on: bool) -> off: bool { !on } where {
        claim flips = off != on;
    }
}
contract Monotone<T: Number> {
    fn f(x: T) -> y: T;
    claim rises(a: T, b: T) = !(a <= b) || f(a) <= f(b);
}
examples { (sun::light::w = 2.0, stone::speed::h = 5.0) -> stone::speed::v2 = 196.2 }
"#;

/// Each finding as its code and where.
fn findings(model: &Model) -> Vec<String> {
    let (_, findings) = agree(model, &Settings::default());
    findings
        .iter()
        .map(|f| format!("{} at {}", f.code, f.at))
        .collect()
}

#[test]
fn div_agrees_with_itself() {
    let (agreed, findings) = agree(&model(DIV), &Settings::default());
    assert_eq!(findings, vec![]);
    // Nine comparisons: four examples against two claims, and the claims
    // together, kept at an example.
    assert_eq!(agreed.len(), 9);
    assert!(agreed.iter().all(|a| a.agreement == Agreement::Holds));
}

#[test]
fn the_fall_agrees_where_it_can_tell() {
    let fall = model(FALL);
    let (agreed, findings) = agree(&fall, &Settings::default());
    let shown: Vec<String> = findings.iter().map(|f| f.to_string()).collect();
    // `slow` reads the sun's light, which has no value but in an example
    // of the model; `small` holds for h up to 20 only by splitting, which
    // bounds over all the reals cannot do. Both unknown, and said.
    assert_eq!(
        shown,
        [
            "finding[A007]: claim `slow`: unknown, no value for `sun::light` at some of the 16 points tried\n --> stone::speed",
            "finding[A007]: claim `small`: unknown, not decided by bounds over all of what it reads, nor at 13 points\n --> stone::square",
        ]
    );
    // `flips` at both of its points; the example against the body and
    // the claim; each function's claims together, at a point that keeps
    // them; the example of the model, through the call.
    let holds: Vec<String> = agreed
        .iter()
        .filter(|a| a.agreement == Agreement::Holds)
        .map(|a| format!("{} {}", a.at, a.against))
        .collect();
    let expected = [
        "stone::speed its claims together",
        "stone::square example 0",
        "stone::square example 0 against claim `small`",
        "stone::square its claims together",
        "stone::lit claim `flips`",
        "stone::lit its claims together",
        "examples::0 `stone::speed::v2`",
    ];
    assert_eq!(holds, expected);
}

#[test]
fn each_finding_is_given() {
    let changed = |from: &str, to: &str| {
        assert!(FALL.contains(from) || DIV.contains(from), "{from}");
        let text = if FALL.contains(from) {
            FALL.replace(from, to)
        } else {
            DIV.replace(from, to)
        };
        findings(&model(&text))
    };
    let cases = [
        (changed("s <= 400.0", "s <= 300.0"), "A001 at stone::square"),
        (
            changed("(5.0) -> 25.0", "(5.0) -> 24.0"),
            "A002 at stone::square",
        ),
        (changed("(6, 2) -> 3", "(6, 2) -> 4"), "A003 at calc::div"),
        (
            // By bounds, only a claim false whatever the values: `0 < q`
            // with `q < 0` is not seen.
            changed("claim bounded = b == 0 || q <= a;", "claim never = 1 < 0;"),
            "A004 at calc::div",
        ),
        (
            changed(
                "    fn lit(",
                "    impl Monotone<Real> for square;\n    fn lit(",
            ),
            "A005 at stone::square::Monotone::rises",
        ),
        (
            changed("stone::speed::v2 = 196.2", "stone::speed::v2 = 98.1"),
            "A006 at examples::0",
        ),
        (
            changed("(sun::light::w = 2.0, ", "("),
            "A007 at examples::0",
        ),
        (
            changed(
                "stone::speed::h = 5.0)",
                "stone::speed::h = 5.0, stone::square::h = 1.0)",
            ),
            "A009 at examples::0",
        ),
    ];
    let mut given = std::collections::BTreeSet::new();
    for (got, expected) in cases {
        assert!(
            got.contains(&expected.to_string()),
            "{expected} not in {got:?}"
        );
        given.insert(expected[..4].to_string());
    }
    let listed = CATALOGUE
        .iter()
        .filter(|e| e.code.suit == Suit::Agree && !e.retired);
    let listed: std::collections::BTreeSet<String> = listed.map(|e| e.code.to_string()).collect();
    assert_eq!(given, listed);
}

/// A failure is at a point `Exact` confirms, named.
#[test]
fn a_failure_says_where() {
    let text = FALL.replace("s <= 400.0", "s <= 300.0");
    let (_, findings) = agree(&model(&text), &Settings::default());
    let messages: Vec<&str> = findings.iter().map(|f| f.message.as_str()).collect();
    assert!(
        messages.contains(&"the body breaks claim `small` at h = 20.0"),
        "{messages:?}"
    );
    // With no points to try, it is unknown, and says so.
    let (_, findings) = agree(&model(&text), &Settings { points: 0 });
    assert!(findings
        .iter()
        .any(|f| f.message.ends_with("nor at 0 points")));
}
