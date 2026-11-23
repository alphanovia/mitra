//! Red team of sequences, `[T]`, the minimal set (finding 3, step 2;
//! your "ok", 20:24): `[1, 2]`, `[]` told from where it goes, `xs.len()`,
//! `xs[i]` with no value out of range, and `==`. Each guard has a test
//! that fails when it is taken out.

use mitra_core::agree::{agree, Agreement, Settings};
use mitra_core::check::check;
use mitra_core::evidence::check as check_evidence;
use mitra_core::explain::{explain, Settings as ExplainSettings};
use mitra_core::parse::parse;
use mitra_core::print::print;
use mitra_core::reader::read_store;

/// The first parse error, rendered without the source line.
fn refused(text: &str) -> String {
    parse(text)
        .map(|_| "parsed".to_string())
        .unwrap_or_else(|e| e.to_string())
}

fn errors(text: &str) -> Vec<String> {
    check(&parse(text).unwrap())
        .iter()
        .map(|e| e.to_string())
        .collect()
}

/// Each comparison, `path against: outcome` with the point of a failure.
fn agreed(text: &str, settings: &Settings) -> Vec<String> {
    let model = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![], "{text}");
    let (agreed, _) = agree(&model, settings);
    agreed
        .iter()
        .map(|a| {
            let outcome = match &a.agreement {
                Agreement::Holds => "holds".to_string(),
                Agreement::Fails { at, .. } => {
                    let at: Vec<String> = at
                        .iter()
                        .map(|(n, v)| format!("{n} = {}", mitra_core::print::value(v)))
                        .collect();
                    format!("fails at {}", at.join(", "))
                }
                Agreement::Unknown { why } => format!("unknown: {why}"),
            };
            format!("{} {}: {outcome}", a.at, a.against)
        })
        .collect()
}

const FIRST: &str = "subject list {
    fn first(xs: [Int]) -> f: Int { if xs.len() == 0 { 0 } else { xs[0] } } where {
        claim head: xs.len() == 0 || f == xs[0];
        examples { ([]) -> 0, ([3, 4]) -> 3 }
    }
    fn pair(a: 0..=3) -> p: [Int] { [a, a + 1] } where {
        claim two: p.len() == 2;
        claim up: p[0] < p[1];
    }
}";

#[test]
fn claims_on_sequences_are_decided() {
    let out = agreed(FIRST, &Settings::default());
    assert!(out.iter().all(|o| !o.contains("fails")), "{out:?}");
    // On a range, every point: `up` holds; `two` holds by bounds.
    assert!(
        out.iter()
            .any(|o| o.contains("claim `up`") && o.ends_with("holds")),
        "{out:?}"
    );
    assert!(
        out.iter()
            .any(|o| o.contains("claim `two`") && o.ends_with("holds")),
        "{out:?}"
    );
    // The wrong place read: a failure at a point.
    let wrong = FIRST.replace("{ xs[0] } }", "{ xs[xs.len() - 1] } }");
    let out = agreed(&wrong, &Settings::default());
    assert!(
        out.iter()
            .any(|o| o.contains("claim `head`: fails at xs = [")),
        "{out:?}"
    );
}

#[test]
fn a_place_out_of_range_has_no_value() {
    // `xs[0]` of `[]`: no value, as Rust's panics; the body breaks the
    // claim there.
    let wrong = FIRST.replace("if xs.len() == 0 { 0 } else { xs[0] }", "xs[0]");
    let out = agreed(&wrong, &Settings::default());
    assert!(
        out.iter()
            .any(|o| o.contains("example 0: fails at xs = []")),
        "{out:?}"
    );
    // Read where it has no value, a claim is broken there.
    let wrong = wrong.replace(
        "claim head: xs.len() == 0 || f == xs[0];",
        "claim head: f == f;",
    );
    let out = agreed(&wrong, &Settings::default());
    assert!(
        out.iter()
            .any(|o| o.contains("claim `head`: fails at xs = []")),
        "{out:?}"
    );
    // A place past every length is no value at all, at the first point.
    let text = "subject s { fn f(a: 0..=3) -> b: Int { [a, a][2] } where { claim c: b == 0; } }";
    let out = agreed(text, &Settings::default());
    assert!(
        out.iter().any(|o| o.contains("claim `c`: fails at a = 0")),
        "{out:?}"
    );
}

#[test]
fn every_length_is_tried_up_to_the_setting() {
    // A claim false only on three values: found with the default length 3,
    // unknown with length 2.
    let text =
        "subject s { fn f(xs: [bool]) -> n: Int { xs.len() } where { claim short: n < 3; } }";
    let out = agreed(text, &Settings::default());
    assert!(out.iter().any(|o| o.contains("fails at xs = [")), "{out:?}");
    let short = Settings {
        length: 2,
        ..Settings::default()
    };
    let out = agreed(text, &short);
    assert!(
        out.iter()
            .any(|o| o.contains("claim `short`: unknown") && o.contains("nor at 7 points")),
        "{out:?}"
    );
}

#[test]
fn equal_sequences_are_equal_place_by_place() {
    let text = "subject s {
    fn same(a: 0..=2, b: 0..=2) -> e: bool { [a, b] == [b, a] } where {
        claim c: e == (a == b);
    }
    fn none(a: 0..=2) -> e: bool { [a] == [] } where {
        claim c: !e;
    }
}";
    let out = agreed(text, &Settings::default());
    assert!(out.iter().all(|o| o.ends_with("holds")), "{out:?}");
    let wrong = text.replace("claim c: !e;", "claim c: e;");
    let out = agreed(&wrong, &Settings::default());
    assert!(out.iter().any(|o| o.contains("fails")), "{out:?}");
}

#[test]
fn the_type_of_an_empty_sequence_is_told_from_where_it_goes() {
    let f = |body: &str| {
        refused(&format!(
            "subject s {{ enum Option<T> {{ None, Some(T) }} fn g(xs: [Int]) -> b: bool {{ true }} fn f(x: Int) -> y: bool {{ {body} }} }}"
        ))
    };
    assert!(
        f("[] == []").contains("error[P009]: the type of `[]` cannot be told here"),
        "{}",
        f("[] == []")
    );
    assert_eq!(f("[] == [x]"), "parsed");
    assert_eq!(f("[x] == []"), "parsed");
    assert_eq!(f("g([])"), "parsed");
    assert_eq!(f("[None] == [Some(x)]"), "parsed");
    assert!(f("[None] == [None]").contains("error[P009]"));
    // A row: `[]` from the type the function takes.
    let text =
        "subject s { fn f(xs: [Int]) -> n: Int { xs.len() } where { examples { ([]) -> 0 } } }";
    assert_eq!(check(&parse(text).unwrap()), vec![]);
    // Given where no sequence goes, it is of the wrong type (M015), not of
    // a type not told; `None` likewise.
    for row in ["([]) -> 0", "(None) -> 0"] {
        let text = format!(
            "subject s {{ enum Option<T> {{ None, Some(T) }} fn f(x: Int) -> y: Int {{ x }} where {{ examples {{ {row} }} }} }}"
        );
        let out = errors(&text);
        assert_eq!(out.len(), 1, "{out:?}");
        assert!(out[0].starts_with("error[M015]"), "{out:?}");
    }
}

#[test]
fn a_sequence_holds_values_of_one_type() {
    let text = "subject s { fn f(x: Int) -> y: [Int] { [x, true] } }";
    let out = errors(text);
    assert!(out.iter().any(|e| e.starts_with("error[M005]")), "{out:?}");
    let text = "subject s { fn f(x: Int) -> y: Int { x.len() } }";
    let out = errors(text);
    assert!(
        out.iter().any(|e| e.contains("`.len()` reads a sequence")),
        "{out:?}"
    );
    let text = "subject s { fn f(xs: [Int]) -> y: Int { xs.len } }";
    let out = errors(text);
    assert!(
        out.iter()
            .any(|e| e.contains("the length of a sequence is `.len()`")),
        "{out:?}"
    );
    let text = "subject s { fn f(xs: [Int]) -> y: Int { xs[true] } }";
    let out = errors(text);
    assert!(
        out.iter().any(|e| e.contains("`xs[i]` reads a sequence")),
        "{out:?}"
    );
}

#[test]
fn a_model_of_sequences_reads_back_as_printed() {
    let text = "subject s {
    enum Option<T> { None, Some(T) }
    record P { x: Int, ys: [Real] }
    fn f(p: P, xss: [[Int]], o: [Option<Int>]) -> b: bool {
        p.ys.len() == xss[0].len() && xss[1][0] == 3 && o == [None, Some(p.x)] && [] == p.ys
    } where {
        examples { (P { x: 1, ys: [] }, [[], [3]], [None, Some(1)]) -> true }
    }
}";
    let model = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&model), vec![]);
    let printed = print(&model);
    let again = parse(&printed).unwrap_or_else(|e| panic!("{e}\n{printed}"));
    assert_eq!(model, again, "{printed}");
    assert_eq!(print(&again), printed);
    let out = agreed(text, &Settings::default());
    assert!(out.iter().all(|o| !o.contains("fails")), "{out:?}");
}

#[test]
fn a_measured_sequence_is_evidence() {
    let text = "subject list {
    fn first(xs: [Int]) -> f: Int at \"src/lib.rs/first\" { if xs.len() == 0 { 0 } else { xs[0] } }
}";
    let model = parse(text).unwrap();
    assert_eq!(check(&model), vec![]);
    let store = "measure list::first ([]) -> 0 by \"test:empty\";\nmeasure list::first ([5, 6]) -> 6 by \"test:two\";";
    let ledger = read_store("s", store, &model, &model.ties).unwrap().0;
    assert_eq!(check_evidence(&model, &ledger), vec![]);
    let e = explain(&model, &ledger, &ExplainSettings::default());
    let shown: Vec<String> = e.verdicts.iter().map(|v| format!("{v:?}")).collect();
    assert!(
        shown
            .iter()
            .any(|v| v.contains("Seq(Int, [])") && v.contains("Explained")),
        "{shown:#?}"
    );
    assert!(
        shown
            .iter()
            .any(|v| v.contains("Int(5)") && v.contains("Contradicted")),
        "{shown:#?}"
    );
}
