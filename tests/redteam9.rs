//! Red team, round 9: agree, check (uses, calls, contracts, examples of
//! the model, rule terms) and evidence. Each test asserts what the docs
//! promise; each failed on f1cbd14, a defect reproduced, and passes since
//! the fixes. The section "after the fixes" pins what they added; the
//! last keeps attacks that held.

use mitra_core::agree::{agree, Agreed, Agreement, Settings};
use mitra_core::catalogue::Finding;
use mitra_core::check::check;
use mitra_core::evidence::{self, Ledger, Measurement, Review};
use mitra_core::explain::{explain, Verdict};
use mitra_core::judgement::Prop;
use mitra_core::model::{Model, Path};
use mitra_core::parse::parse;
use mitra_core::simulation::{self, Value};

fn model(text: &str) -> Model {
    let model = parse(text).unwrap();
    assert_eq!(check(&model), vec![], "{text}");
    model
}
fn path(p: &str) -> Path {
    Path::to(&p.split("::").collect::<Vec<_>>())
}
fn agreed(text: &str) -> (Vec<Agreed>, Vec<Finding>) {
    agree(&model(text), &Settings::default())
}
/// The outcome of the comparison at `at` against `against`.
fn outcome(agreed: &[Agreed], at: &str, against: &str) -> Agreement {
    let found = agreed
        .iter()
        .find(|a| a.at.to_string() == at && a.against == against);
    found
        .unwrap_or_else(|| panic!("no comparison of {at} against {against}: {agreed:?}"))
        .agreement
        .clone()
}
fn codes(findings: &[Finding]) -> Vec<String> {
    findings
        .iter()
        .map(|f| format!("{} at {}", f.code, f.at))
        .collect()
}

// ------------------------------------------------------------ agree

/// D1. "Holds ... when its domain is finite and every point was tried."
/// Two equal example rows are both counted as tried, so the count of
/// points reaches the size of the domain with one point never tried.
/// `!r` is false at p = (true, false), where the body gives true.
#[test]
fn d1_a_repeated_example_is_not_a_second_point() {
    let text = r#"
subject s {
    fn f(p: (bool, bool)) -> r: bool { p.0 && !p.1 } where {
        claim never: !r;
        examples { ((false, true)) -> false, ((false, true)) -> false }
    }
}
"#;
    let (agreed, _) = agreed(text);
    assert_ne!(
        outcome(&agreed, "s::f", "claim `never`"),
        Agreement::Holds,
        "false at p = (true, false), never tried"
    );
}

/// D1, with the default 64 points: six truths, 64 values, one row twice.
/// `!r` is false only where all six are true, the last point in order.
#[test]
fn d1_at_the_default_settings_too() {
    let text = r#"
subject s {
    fn f(a: bool, b: bool, c: bool, d: bool, e: bool, g: bool) -> r: bool { a && b && c && d && e && g } where {
        claim never: !r;
        examples { (false, false, false, false, false, false) -> false, (false, false, false, false, false, false) -> false }
    }
}
"#;
    let (agreed, _) = agreed(text);
    assert_ne!(outcome(&agreed, "s::f", "claim `never`"), Agreement::Holds);
}

/// D2. An opaque record has unknown values ("no value of it is made"),
/// but its type is a record with no field, which simulation reads as a
/// type of one value: `a == b` is surely true by bounds, and the domain
/// is "finite", one point. Two keys need not be equal: a false Holds,
/// and a false A004 on `!(a == b)`, which two distinct keys keep.
#[test]
fn d2_an_opaque_record_is_not_one_value() {
    let text = r#"
subject s {
    record Key;
    fn same(a: Key, b: Key) -> r: bool { a == b } where {
        claim always: r;
    }
    fn swap(a: Key, b: Key) -> r: bool where {
        claim distinct: !(a == b);
        claim told: r;
    }
}
"#;
    let (agreed, findings) = agreed(text);
    assert_ne!(
        outcome(&agreed, "s::same", "claim `always`"),
        Agreement::Holds
    );
    assert!(
        !codes(&findings).contains(&"A004 at s::swap".to_string()),
        "{findings:?}"
    );
}

/// D3. A function that takes nothing has one point, `()`, and agree never
/// tries it: no candidate is made when there is no argument, so a claim
/// the exact reading refutes at the only point is unknown "at 0 points".
/// Same for a contract claim over no argument.
#[test]
fn d3_a_function_of_nothing_is_tried_at_its_one_point() {
    let text = r#"
subject s {
    fn light() -> w: rational where {
        examples { () -> -5.0 }
    }
    fn g() -> y: rational { light() } where {
        claim pos: 0.0 < y;
    }
    fn z() -> y: rational { 1.0 / 0.0 } where {
        claim pos: 0.0 < y;
    }
}
"#;
    let (_, findings) = agreed(text);
    assert_eq!(codes(&findings), ["A001 at s::g", "A001 at s::z"]);
}

/// D4. A004 is looked for only when a function has two claims or more:
/// one claim false whatever the values, on a function with no body and
/// no example, gives no comparison and no finding at all.
#[test]
fn d4_one_claim_no_value_keeps_is_found() {
    let text = r#"
subject s {
    fn drop(h: rational) -> v: rational where {
        claim never: 1.0 < 0.0;
    }
}
"#;
    let (agreed, findings) = agreed(text);
    assert_eq!(codes(&findings), ["A004 at s::drop"], "{agreed:?}");
}

/// D5. An impl of a contract with no function, `impl Bad<Real> for ();`,
/// fits (M020 is silent), is declared any number of times (no M002), and
/// agree reads none of its claims: a contract claim false everywhere is
/// dropped quietly.
///
/// Fixed: M020, an impl names the functions its claims are read of.
#[test]
fn d5_an_impl_for_no_function_is_not_dropped() {
    let text = r#"
contract Bad<T: Number> {
    claim never(a: T, b: T): a < b && b < a;
}
subject s {
    impl Bad<rational> for ();
    impl Bad<rational> for ();
}
"#;
    let parsed = parse(text).unwrap();
    let errors = check(&parsed);
    if errors.is_empty() {
        let (agreed, findings) = agree(&parsed, &Settings::default());
        assert!(
            !agreed.is_empty() && !findings.is_empty(),
            "accepted, and nothing compared: {agreed:?}"
        );
    }
    let errors: Vec<String> = errors
        .iter()
        .map(|e| format!("{} at {}", e.code, e.at))
        .collect();
    // Each impl, said once: the same error twice is one.
    assert_eq!(errors, ["M020 at s::Bad"]);
}

/// D6. An example of the model with no output is accepted and compares
/// nothing; V006 refuses the same shape in a measurement.
#[test]
fn d6_an_example_of_the_model_states_an_output() {
    let text = r#"
subject stone {
    fn speed(h: rational) -> v2: rational { 2.0 * h }
}
examples {
    (stone::speed::h = 5.0) -> ()
}
"#;
    let errors: Vec<String> = check(&parse(text).unwrap())
        .iter()
        .map(|e| e.code.to_string())
        .collect();
    assert!(
        errors.contains(&"M021".to_string()) || errors.contains(&"M022".to_string()),
        "{errors:?}"
    );
}

/// D7 (doc). "Every comparison the table lists gives Holds, Fails or a
/// finding": `its claims together` is pushed only when it fails, and two
/// examples at one point only when they differ. Div has two claims, so
/// nine comparisons by the table; agree counts eight, and the CLI prints
/// that count.
#[test]
fn d7_every_comparison_of_the_table_is_counted() {
    let text = std::fs::read_to_string(format!("{}/models/div.mitra", env!("CARGO_MANIFEST_DIR")))
        .unwrap();
    let (agreed, _) = agreed(&text);
    let together = agreed.iter().any(|a| a.against == "its claims together");
    assert!(
        together,
        "{} comparisons, none of the claims together",
        agreed.len()
    );
}

// ------------------------------------------------------------ check

/// D8. M018: "a part ... names another part's record (in a type ...)
/// only through a use". `impl C<Ray> for up;` names the sun's record in
/// a type, with no use; agree then makes values of it.
#[test]
fn d8_a_record_named_by_an_impl_is_used() {
    let text = r#"
contract C<T> {
    fn f(x: rational) -> y: rational;
    claim c(a: T, b: T): a == b || f(1.0) <= f(2.0);
}
environment sun { record Ray { w: rational } }
subject stone {
    fn up(x: rational) -> y: rational { x + 1.0 }
    impl C<Ray> for up;
}
"#;
    let errors: Vec<String> = check(&parse(text).unwrap())
        .iter()
        .map(|e| e.code.to_string())
        .collect();
    assert!(errors.contains(&"M018".to_string()), "{errors:?}");
}

/// N1. M016: "A use names another part, and functions or records of it."
/// `use sun::{};` names none, and a name twice is taken (Rust: E0252).
#[test]
fn n1_a_use_names_something_once() {
    for u in ["use sun::{};", "use sun::{light, light};"] {
        let text = format!(
            "environment sun {{ fn light() -> w: rational; }}\nsubject stone {{ {u} fn f(h: rational) -> v: rational {{ h }} }}"
        );
        let errors = check(&parse(&text).unwrap());
        assert!(!errors.is_empty(), "{u} accepted");
    }
}

// ------------------------------------------------------------ evidence

fn div() -> Model {
    let path = format!("{}/models/div.mitra", env!("CARGO_MANIFEST_DIR"));
    model(&std::fs::read_to_string(path).unwrap())
}
fn int(n: i64) -> Value {
    Value::Int(n.into())
}
fn ports(these: &[(&str, Value)]) -> Vec<(Path, Value)> {
    these.iter().map(|(p, v)| (path(p), v.clone())).collect()
}

/// D9. V007 checks only that a voice is not empty. A review by `model`
/// (explain's voice for what the model alone says) or by
/// `machine:mitra-check` (the checker's) is accepted, and explain then
/// reads it as the model's own: the same review by a person explains the
/// spec, by `model` it does not.
#[test]
fn d9_a_voice_of_mitra_is_no_voice_of_evidence() {
    let model = div();
    let spec = Prop::Named("calc::div::spec".to_string());
    let verdict = |by: &str| {
        let ledger = Ledger {
            measurements: vec![],
            reviews: vec![Review {
                prop: path("calc::div::spec"),
                holds: true,
                by: by.to_string(),
            }],
        };
        let errors = evidence::check(&model, &ledger);
        let e = explain(&model, &ledger, &Default::default());
        let v = e
            .verdicts
            .iter()
            .find(|v| v.prop == spec)
            .unwrap()
            .verdict
            .clone();
        (errors, v)
    };
    assert_eq!(verdict("human:ana"), (vec![], Verdict::Explained));
    let (errors, v) = verdict("model");
    assert!(
        !errors.is_empty(),
        "a review by `model` is accepted, and explain gives {v:?}"
    );
    let (errors, _) = verdict("machine:mitra-check");
    assert!(
        !errors.is_empty(),
        "a review in the checker's voice is accepted"
    );
}

/// N2. One measurement says div gave q = 3 (an input: what a function
/// with no body gave) and q = 4 (an output, seen). V001 checks "each once
/// on its side", so a port on both sides passes; explain then reads the
/// measurement against itself.
#[test]
fn n2_a_port_is_an_input_or_an_output() {
    let model = div();
    let ledger = Ledger {
        measurements: vec![Measurement {
            inputs: ports(&[
                ("calc::div::a", int(6)),
                ("calc::div::b", int(2)),
                ("calc::div::q", int(3)),
            ]),
            outputs: ports(&[("calc::div::q", int(4))]),
            seen: true,
            count: 1,
            by: "human:ana".to_string(),
        }],
        reviews: vec![],
    };
    assert!(!evidence::check(&model, &ledger).is_empty());
}

/// N3. V003 is `has_type`: a value of an opaque record has the type, so a
/// measurement gives one, though M004 refuses it in an example ("an
/// opaque one never").
#[test]
fn n3_no_value_of_an_opaque_record_is_measured() {
    let text = r#"
subject s {
    record Key;
    fn mk(n: integer) -> k: Key at "src/lib.rs/mk";
}
"#;
    let model = model(text);
    let key = simulation::Record {
        args: vec![],
        name: "Key".to_string(),
        fields: vec![],
    };
    let ledger = Ledger {
        measurements: vec![Measurement {
            inputs: ports(&[("s::mk::n", int(1))]),
            outputs: vec![(path("s::mk::k"), Value::Record(key.clone(), vec![]))],
            seen: true,
            count: 1,
            by: "human:ana".to_string(),
        }],
        reviews: vec![],
    };
    assert!(!evidence::check(&model, &ledger).is_empty());
    // The same value as an example of the model is refused.
    let example = text.replace(
        "fn mk(n: integer) -> k: Key at \"src/lib.rs/mk\";",
        "fn mk(n: integer) -> k: Key at \"src/lib.rs/mk\" where { examples { (1) -> Key {} } }",
    );
    let errors: Vec<String> = check(&parse(&example).unwrap())
        .iter()
        .map(|e| e.code.to_string())
        .collect();
    assert_eq!(errors, ["M004"]);
}

// ------------------------------------------------------------ after the fixes

/// D1, the other half: a record of few values is tried at every one of
/// them, not only "the i-th value of each field together".
#[test]
fn after_a_small_record_is_tried_at_every_value() {
    let text = r#"
subject s {
    fn f(p: (bool, bool)) -> r: bool { p.0 && !p.1 } where {
        claim never: !r;
    }
}
"#;
    let (agreed, _) = agreed(text);
    let Agreement::Fails { at, .. } = outcome(&agreed, "s::f", "claim `never`") else {
        panic!("{agreed:?}");
    };
    assert_eq!(mitra_core::print::value(&at[0].1), "(true, false)");
}

/// D2, through a call: bounds read a function's type when it has no body
/// or claim, so an opaque record it gives is one value there too.
#[test]
fn after_an_opaque_record_through_a_call_is_not_one_value() {
    let text = r#"
subject s {
    record Key;
    fn mk(n: integer) -> k: Key;
    fn g(n: integer) -> r: bool { mk(n) == mk(n + 1) } where {
        claim c: r;
    }
}
"#;
    let (agreed, _) = agreed(text);
    assert_ne!(outcome(&agreed, "s::g", "claim `c`"), Agreement::Holds);
}

/// D3: a point of nothing prints `()`, and a claim with no value there
/// says so.
#[test]
fn after_a_point_of_nothing_prints() {
    let text = r#"
subject s {
    fn z() -> y: rational { 1.0 / 0.0 } where {
        claim pos: 0.0 < y;
    }
}
"#;
    let (_, findings) = agreed(text);
    let messages: Vec<&str> = findings.iter().map(|f| f.message.as_str()).collect();
    assert_eq!(
        messages,
        ["the body breaks claim `pos` at (), where it has no value"]
    );
}

/// D4 and D7: some value keeps a function's claims together, shown at a
/// point (an example, what the body gives, a candidate); or none, at
/// every point of a finite domain; or unknown, said.
#[test]
fn after_claims_together_hold_fail_or_say_why() {
    let text = r#"
subject s {
    fn lit(on: bool) -> off: bool { !on } where {
        claim flips: off != on;
    }
    fn never(on: bool) -> off: bool where {
        claim a: off;
        claim b: !off;
    }
    record Key;
    fn keys(a: Key) -> r: bool where {
        claim c: r;
    }
}
"#;
    let (agreed, findings) = agreed(text);
    assert_eq!(
        outcome(&agreed, "s::lit", "its claims together"),
        Agreement::Holds
    );
    assert!(matches!(
        outcome(&agreed, "s::never", "its claims together"),
        Agreement::Fails { .. }
    ));
    assert!(matches!(
        outcome(&agreed, "s::keys", "its claims together"),
        Agreement::Unknown { .. }
    ));
    assert_eq!(codes(&findings), ["A007 at s::keys", "A004 at s::never"]);
}

/// Contract candidates: the constants of the functions a claim calls are
/// tried, so `g` breaks `rises` at 7.5, where it drops to 0.
#[test]
fn after_a_contract_claim_tries_the_constants_of_what_it_calls() {
    let text = r#"
contract Monotone<T: Number> {
    fn f(x: T) -> y: T;
    claim rises(a: T, b: T): !(a <= b) || f(a) <= f(b);
}
subject s {
    fn g(x: rational) -> y: rational { if x < 7.5 { x } else { 0.0 } }
    impl Monotone<rational> for g;
    fn inv(x: rational) -> y: rational { 1.0 / x }
    impl Monotone<rational> for inv;
}
"#;
    let (_, findings) = agreed(text);
    let shown: Vec<String> = findings
        .iter()
        .map(|f| format!("{} at {}: {}", f.code, f.at, f.message))
        .collect();
    assert_eq!(
        shown,
        [
            "A005 at s::g::Monotone::rises: false at a = 1.0, b = 7.5",
            "A005 at s::inv::Monotone::rises: no value at a = 0.0, b = 0.0",
        ]
    );
}

/// Ignored inputs: an input of an example of the model no output reads is
/// a finding (A009), and a value of the world given there is compared
/// with the function's own example at that point.
#[test]
fn after_an_input_nothing_reads_is_said() {
    let text = r#"
environment sun {
    fn light() -> w: rational where {
        examples { () -> 850.0 }
    }
}
subject stone {
    use sun::light;
    fn speed(h: rational) -> v: rational { h * light() }
    fn half(h: rational) -> v: rational { speed(5.0) / 2.0 }
    fn other(h: rational) -> v: rational { h }
}
examples {
    (sun::light::w = 900.0, stone::half::h = 1.0, stone::speed::h = 1.0, stone::other::h = 2.0) -> stone::half::v = 2250.0
}
"#;
    let (agreed, findings) = agreed(text);
    let shown: Vec<String> = findings
        .iter()
        .map(|f| format!("{} {}", f.code, f.message))
        .collect();
    assert_eq!(
        shown,
        [
            "A006 `sun::light::w` against example 0 of `sun::light`: the model gives 850.0 at ()",
            "A009 `stone::other::h` is given and no output reads it: no output calls `stone::other`",
            "A009 `stone::speed::h` is given and no output reads it: the model computes what `stone::speed` takes where it is called",
        ]
    );
    // The world's value given is the one read.
    assert_eq!(
        outcome(&agreed, "examples::0", "`stone::half::v`"),
        Agreement::Holds
    );
}

/// Ambiguous call: a name alone two glob uses read names two functions;
/// M017 says which, not "no function".
#[test]
fn after_an_ambiguous_call_says_which() {
    let text = r#"
environment sun { fn light() -> w: rational; }
environment moon { fn light() -> w: rational; }
subject stone {
    use sun::*;
    use moon::*;
    fn speed(h: rational) -> v: rational { h * light() }
}
"#;
    let errors: Vec<String> = check(&parse(text).unwrap())
        .iter()
        .map(|e| format!("{} {}", e.code, e.message))
        .collect();
    assert_eq!(
        errors,
        ["M017 `light` names several functions its uses read, `moon::light`, `sun::light`: write which"]
    );
}

/// D10, the rule half: a point written in a rule makes records as
/// declared (M004), as an example does.
#[test]
fn after_a_rule_point_makes_no_opaque_record() {
    let text = r#"
subject s {
    record Key;
    fn has(k: Key) -> r: bool where {
        claim c: r;
        rule r1: has::c |- correct(has, (Key {}));
    }
}
"#;
    let errors: Vec<String> = check(&parse(text).unwrap())
        .iter()
        .map(|e| e.code.to_string())
        .collect();
    assert!(errors.contains(&"M004".to_string()), "{errors:?}");
}

/// D9, the other half: a voice is of a kind, `human:`, `agent:`,
/// `machine:` or `test:`, with a name.
#[test]
fn after_a_voice_is_of_a_kind() {
    let model = div();
    for (by, refused) in [
        ("ana", true),
        ("human:", true),
        ("robot:r2", true),
        ("human:ana", false),
        ("agent:claude", false),
        ("machine:verus", false),
        ("test:tests::div_by_two", false),
    ] {
        let ledger = Ledger {
            measurements: vec![],
            reviews: vec![Review {
                prop: path("calc::div::spec"),
                holds: true,
                by: by.to_string(),
            }],
        };
        let errors = evidence::check(&model, &ledger);
        assert_eq!(!errors.is_empty(), refused, "{by}: {errors:?}");
    }
}

// ------------------------------------------------------------ cost

/// C1, held since finding 9: `Exact` computes only the side `if` takes,
/// and calls are remembered. A chain of 12 functions, each calling the
/// one before in both branches of an `if`, three claims each, took about
/// 1 s, each level doubling it; now a few ms, as with `let`.
#[test]
fn c1_a_chain_through_if_doubles_at_each_level() {
    let chain = |n: usize, shared: bool| {
        let mut out = vec!["subject p {".to_string()];
        for i in 0..n {
            let body = match (i, shared) {
                (0, _) => "x * 2.0 + y".to_string(),
                (_, false) => format!(
                    "if x < 0.0 {{ f{0}(x, y) }} else {{ f{0}(x, y) + 1.0 }}",
                    i - 1
                ),
                (_, true) => format!(
                    "let a = f{}(x, y) in if x < 0.0 {{ a }} else {{ a + 1.0 }}",
                    i - 1
                ),
            };
            out.push(format!(
                "fn f{i}(x: rational, y: rational) -> r: rational {{ {body} }} where {{ \
                 claim a: !(0.0 <= x) || !(0.0 <= y) || 0.0 <= r; \
                 claim b: !(x == 0.0 && y == 0.0) || r <= 1000000.0; \
                 claim c: r == r; }}"
            ));
        }
        out.push("}".to_string());
        model(&out.join("\n"))
    };
    for n in [8, 10, 12] {
        for shared in [true, false] {
            let m = chain(n, shared);
            let start = std::time::Instant::now();
            agree(&m, &Settings::default());
            eprintln!("{n} functions, let {shared}: {:?}", start.elapsed());
        }
    }
    let m = chain(12, false);
    let start = std::time::Instant::now();
    agree(&m, &Settings::default());
    assert!(start.elapsed().as_millis() < 100, "{:?}", start.elapsed());
}

// ------------------------------------------------------------ held

/// Held: a finite domain with distinct rows is decided only when every
/// point was tried (3,000 random bool bodies and claims, tests/redteam9_fuzz.rs),
/// and a claim with a division the guard protects holds where it should.
#[test]
fn held_a_guarded_division_and_a_failure_with_no_value() {
    let text = r#"
subject s {
    fn inv(x: rational) -> y: rational { 1.0 / x } where {
        claim back: x == 0.0 || x * y == 1.0;
        claim pos: !(0.0 < x) || 0.0 < y;
        claim never: !(x == 0.0) || 0.0 < y;
    }
}
"#;
    let (agreed, findings) = agreed(text);
    // `never` reads y where the body has no value: a failure at x = 0.
    assert!(matches!(
        outcome(&agreed, "s::inv", "claim `never`"),
        Agreement::Fails { .. }
    ));
    assert_eq!(
        codes(&findings),
        ["A001 at s::inv", "A007 at s::inv", "A007 at s::inv"]
    );
}
