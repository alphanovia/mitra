//! Agree (`src/agree.rs`): the model against itself, with no evidence.
//! Every code of suit `A` is given, each by one change to a model that
//! agrees.

use mitra_lang::agree::{agree, Agreement, Settings};
use mitra_lang::catalogue::{Suit, CATALOGUE};
use mitra_lang::check::check;
use mitra_lang::model::Model;
use mitra_lang::parse::parse;

fn model(text: &str) -> Model {
    let model = parse(text).unwrap();
    assert_eq!(check(&model), vec![], "{text}");
    model
}

const DIV: &str = r#"
subject calc {
    fn div(a: integer, b: integer) -> q: integer where {
        claim spec: if b == 0 { q == 0 } else { q * b <= a && a < (q + 1) * b };
        claim bounded: b == 0 || q <= a;
        examples { (6, 2) -> 3, (7, 2) -> 3, (9, 3) -> 3, (5, 0) -> 0 }
    }
}
"#;

const FALL: &str = r#"
environment sun {
    fn light() -> w: rational;
}
subject stone {
    use sun::light;
    fn speed(h: rational) -> v2: rational { 2.0 * 9.81 * h * light() } where {
        claim slow: !(h <= 20.0) || v2 < 400.0;
    }
    fn square(h: rational) -> s: rational { h * h } where {
        claim small: !(h <= 20.0) || !(0.0 <= h) || s <= 400.0;
        examples { (5.0) -> 25.0 }
    }
    fn lit(on: bool) -> off: bool { !on } where {
        claim flips: off != on;
    }
}
contract Monotone<T: Number> {
    fn f(x: T) -> y: T;
    claim rises(a: T, b: T): !(a <= b) || f(a) <= f(b);
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
    // of the model, and no claim bounds: unknown, and said. `small` holds
    // since finding 8: its conclusion read where h is from 0 to 20.
    assert_eq!(
        shown,
        [
            "finding[A007]: claim `slow`: unknown, no value for `sun::light` at some of the 16 points tried\n --> stone::speed",
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
        "stone::square claim `small`",
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
            // A claim false whatever the values.
            changed("claim bounded: b == 0 || q <= a;", "claim never: 1 < 0;"),
            "A004 at calc::div",
        ),
        (
            changed(
                "    fn lit(",
                "    impl Monotone<rational> for square;\n    fn lit(",
            ),
            "A005 at stone::square::Monotone::rises",
        ),
        (
            changed("stone::speed::v2 = 196.2", "stone::speed::v2 = 98.1"),
            "A006 at examples::0",
        ),
        (
            changed("(sun::light::w = 2.0, ", "("),
            "A014 at examples::0",
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
    // A hint: what holds given an environment's claim.
    let text = FALL
        .replace(
            "fn light() -> w: rational;",
            "fn light() -> w: rational where { claim bright: 0.0 <= w; }",
        )
        .replace(
            "claim slow: !(h <= 20.0) || v2 < 400.0;",
            "claim slow: !(0.0 <= h) || 0.0 <= v2;",
        );
    let (agreed, _) = agree(&model(&text), &Settings::default());
    let hints: Vec<String> = mitra_lang::agree::hints(&agreed)
        .iter()
        .map(|h| h.to_string())
        .collect();
    assert_eq!(
        hints,
        ["hint[A010]: claim `slow` holds, given `sun::light::bright`\n --> stone::speed"]
    );
    given.insert("A010".to_string());
    // A call `then` lists that the run does not make.
    let text = FALL.replace(
        "fn light() -> w: rational;",
        "fn light() -> w: rational;\n    fn mark(n: integer) -> ok: ();",
    )
        + "\nexamples {\n    given sun::light() -> 2.0 when stone::speed(5.0) then sun::mark(1)\n}";
    let (_, found) = agree(&model(&text), &Settings::default());
    let found: Vec<String> = found.iter().map(|f| f.to_string()).collect();
    assert!(
        found.iter().any(|f| f.starts_with(
            "finding[A011]: the call `sun::mark(1)`: the run does not make it; it calls `sun::light()`"
        )),
        "{found:?}"
    );
    given.insert("A011".to_string());
    // A value written in a row that breaks its record's claims (more in
    // tests/record_claims.rs).
    let (_, found) = agree(
        &model("subject S {\n    record { lo: integer, hi: integer } where { claim ordered: self.lo <= self.hi; }\n    fn lo(s: S) -> l: integer { s.lo } where { examples { (S { lo: 3, hi: 1 }) -> 3 } }\n}"),
        &Settings::default(),
    );
    let found: Vec<String> = found.iter().map(|f| f.to_string()).collect();
    assert_eq!(found, ["finding[A012]: example 0: `s` = S { lo: 3, hi: 1 } breaks record claim `S::ordered`\n --> S::lo"]);
    given.insert("A012".to_string());
    // A loop that never ends (more in tests/endless.rs).
    let (_, found) = agree(
        &model("subject S {\n    fn halve(n: integer) -> k: integer { until(k: n, k <= 1) { k } } where { examples { (4) -> 1 } }\n}"),
        &Settings::default(),
    );
    let found: Vec<String> = found.iter().map(|f| f.to_string()).collect();
    assert_eq!(found, ["finding[A013]: example 0: fails, at n = 4, the `until` in `S::halve` never ends: what it carries, 4, came back after 1 turn\n --> S::halve"]);
    given.insert("A013".to_string());
    // An example is one run (finding 39 B, more in tests/one_run.rs).
    let disk = "environment Disk {\n    fn small(n: integer) -> r: integer;\n    fn big(n: integer) -> r: integer where { claim below: r < 0; }\n}\nsubject R {\n    use Disk::small;\n    use Disk::big;\n    fn route(n: integer) -> r: integer { if n > 10 { small(n) } else { big(n) } } where {\n        examples { given Disk::small(3) -> 30 when (3) then 30 }\n    }\n    fn count(n: integer) -> c: integer { until((k: n, s: 0), k <= 1) { (k, s + 1) }.s } where { examples { (4) -> 2 } }\n}";
    let small = Settings {
        budget: mitra_lang::simulation::Budget { steps: 100, ..Default::default() },
        ..Settings::default()
    };
    let (_, found) = agree(&model(disk), &small);
    let found: Vec<String> = found.iter().map(|f| f.to_string()).collect();
    assert_eq!(found, [
        "finding[A016]: example 0: fails, at n = 4, the step budget, 100 turns of `until`, reached before the run gave a value\n --> R::count",
        "finding[A015]: example 0: `Disk::small(3)` is given but never called: the run calls `Disk::big(3)`\n --> R::route",
        "finding[A017]: example 0: fails: `Disk::big(3)` has no value here, and the claim `Disk::big::below` keeps the run from giving 30\n --> R::route",
    ]);
    let (_, found) = agree(&model(&disk.replace(" where { claim below: r < 0; }", ";")), &small);
    assert!(found.iter().any(|f| f.to_string().starts_with("finding[A014]: example 0: fails: no value for `Disk::big(3)`")));
    // A claim not decided stays unknown.
    let (_, found) = agree(
        &model("subject s {\n    fn count(n: integer) -> c: integer { until((k: n, s: 0), k <= 1) { (k, s + 1) }.s } where {\n        claim some: c >= 0;\n    }\n}"),
        &small,
    );
    assert!(found.iter().any(|f| f.to_string().starts_with("finding[A007]: claim `some`: unknown")), "{found:?}");
    given.insert("A007".to_string());
    for code in ["A014", "A015", "A016", "A017"] {
        given.insert(code.to_string());
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
    let (_, findings) = agree(
        &model(&text),
        &Settings {
            points: 0,
            ..Settings::default()
        },
    );
    assert!(findings
        .iter()
        .any(|f| f.message.ends_with("nor at 0 points")));
}
