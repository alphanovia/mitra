//! Empty sets (finding 17, C9). A rule's set whose range holds no point,
//! `1..0` or `3..3`, is an error of check (M034). One whose condition
//! leaves no point is known only once its points are listed: explain
//! says so (X013) and gives the all, or the any, nothing from its
//! points: never `explained` from no evidence.

use mitra_lang::check::check;
use mitra_lang::evidence::Ledger;
use mitra_lang::explain::{explain, Settings, Verdict};
use mitra_lang::judgement::Prop;
use mitra_lang::parse::parse;
use proptest::prelude::*;

/// A model of one rule: `f::c |- all x in (a: lo..hi) where cond`.
fn model(quantifier: &str, lo: i64, hi: i64, cond: &str) -> String {
    let such = match cond {
        "" => String::new(),
        c => format!(" where {c}"),
    };
    format!(
        "subject s {{\n    fn f(a: integer) -> b: integer {{ a }} where {{\n        claim c: b == a;\n        rule r: f::c |- {quantifier} x in (a: {lo}..{hi}){such}: correct(f, x);\n    }}\n}}"
    )
}

fn codes(text: &str) -> (Vec<String>, Vec<String>, bool) {
    let m = parse(text).unwrap();
    let errors: Vec<String> = check(&m).iter().map(|e| e.code.to_string()).collect();
    if !errors.is_empty() {
        return (errors, vec![], false);
    }
    let e = explain(&m, &Ledger::default(), &Settings::default());
    let findings = e.findings.iter().map(|f| f.code.to_string()).collect();
    let explained = e.verdicts.iter().any(|v| {
        matches!(v.prop, Prop::All { .. } | Prop::Any { .. }) && v.verdict == Verdict::Explained
    });
    (vec![], findings, explained)
}

#[test]
fn a_backwards_range_is_an_error() {
    assert_eq!(codes(&model("all", 1, 0, "")).0, ["M034"]);
    assert_eq!(codes(&model("any", 5, 5, "")).0, ["M034"]);
    assert_eq!(codes(&model("all", 0, 1, "")).0, Vec::<String>::new());
}

#[test]
fn a_condition_no_point_meets_is_a_finding() {
    let (errors, findings, explained) = codes(&model("all", 0, 3, "a > 5"));
    assert_eq!(errors, Vec::<String>::new());
    assert_eq!(findings, ["X013"]);
    assert!(!explained);
    let (_, findings, explained) = codes(&model("any", 0, 3, "a > 5"));
    assert_eq!(findings, ["X013"]);
    assert!(!explained);
}

/// The model alone says the all: with no evidence it is unsupported, and
/// no X013 where a point is in the set.
#[test]
fn a_set_with_points_is_left_as_it_was() {
    let (errors, findings, explained) = codes(&model("all", 0, 3, "a > 1"));
    assert_eq!(errors, Vec::<String>::new());
    assert_eq!(findings, Vec::<String>::new());
    assert!(!explained);
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]
    /// M034 exactly when the range holds no point; else X013 exactly
    /// when the condition leaves none; and with no evidence an all or an
    /// any is never explained.
    #[test]
    fn empty_is_said_once_and_never_explained(
        all in any::<bool>(),
        lo in -4i64..6,
        hi in -4i64..6,
        bar in -6i64..8,
        cond in 0usize..3,
    ) {
        let (text, holds) = match cond {
            0 => (String::new(), (lo..hi).count()),
            1 => (format!("a > {bar}"), (lo..hi).filter(|a| *a > bar).count()),
            _ => (format!("a < {bar}"), (lo..hi).filter(|a| *a < bar).count()),
        };
        let quantifier = if all { "all" } else { "any" };
        let (errors, findings, explained) = codes(&model(quantifier, lo, hi, &text));
        if lo >= hi {
            prop_assert_eq!(errors, vec!["M034".to_string()]);
        } else {
            prop_assert!(errors.is_empty());
            let want: Vec<String> = match holds {
                0 => vec!["X013".to_string()],
                _ => vec![],
            };
            prop_assert_eq!(findings, want);
        }
        prop_assert!(!explained);
    }
}
