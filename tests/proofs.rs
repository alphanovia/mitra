//! Short proofs (finding 26; the form the user took, 2026-11-24): a
//! rule's proof written as its step alone, `by tollens`, its arguments
//! the rule's premises put where they fit. Mitra fills it in when one
//! proof of that step shows the rule; none, or two, is M032.

use mitra_core::check::check;
use mitra_core::evidence::Ledger;
use mitra_core::explain::{explain, Settings, Source};
use mitra_core::judgement::Prop;
use mitra_core::parse::parse;
use mitra_core::print::print;

fn model(rules: &str) -> String {
    format!(
        "subject calc {{
    fn half(n: 0..=9) -> h: 0..=9 {{ n / 2 }} where {{
        rule r2: correct(half, (1)) |- correct(half, (2));
{rules}
    }}
}}"
    )
}

fn errors(text: &str) -> Vec<String> {
    check(&parse(text).unwrap())
        .iter()
        .map(|e| e.to_string())
        .collect()
}

/// The rules the checker takes: shown by their proofs.
fn shown(text: &str) -> Vec<String> {
    let m = parse(text).unwrap();
    assert_eq!(errors(text), Vec::<String>::new(), "{text}");
    let e = explain(&m, &Ledger::default(), &Settings::default());
    let mut shown: Vec<String> = e
        .sources
        .iter()
        .filter(|(_, s)| matches!(s, Source::Proof))
        .filter_map(|(t, _)| match &t.prop {
            Prop::Named(n) => Some(n.clone()),
            _ => None,
        })
        .collect();
    shown.sort();
    shown
}

const STEPS: &str = "        rule p: half::r2, correct(half, (1)) |- correct(half, (2)) by ponens;
        rule t: half::r2, !correct(half, (2)) |- !correct(half, (1)) by tollens;
        rule c: correct(half, (0)), correct(half, (1)) |- all x in (n: 0..2): correct(half, x) by cases;
        rule i: all x in (n: 0..5): correct(half, x) |- correct(half, (3)) by instance;
        rule w: correct(half, (3)) |- any x in (n: 0..5): correct(half, x) by witness;
        rule n: all x in (n: 0..5): correct(half, x) |- all y in (n: 0..3): correct(half, y) by intro;
        rule d: correct(half, (1)), half::r2, !correct(half, (2)) |- !correct(half, (1)) by reductio;";

/// Each step alone shows its rule: the checker takes all seven, and
/// none without its `by`.
#[test]
fn each_step_alone_shows_its_rule() {
    let names = ["c", "d", "i", "n", "p", "t", "w"];
    let all: Vec<String> = names.iter().map(|n| format!("calc::half::{n}")).collect();
    assert_eq!(shown(&model(STEPS)), all);
    let assumed: String = STEPS
        .lines()
        .map(|l| l.split(" by ").next().unwrap().to_string() + ";\n")
        .collect();
    assert_eq!(shown(&model(&assumed)), Vec::<String>::new());
}

/// Written alone or in full, the same proof: tollens on `r2`, its
/// conclusion against, blaming premise 1.
#[test]
fn the_short_form_is_the_full_one() {
    let full = "        rule t: half::r2, !correct(half, (2)) |- !correct(half, (1))
            by !tollens[half::r2, 0, 1](!1, half::r2);";
    assert_eq!(shown(&model(full)), ["calc::half::t"]);
    let short = "        rule t: half::r2, !correct(half, (2)) |- !correct(half, (1)) by tollens;";
    assert_eq!(shown(&model(short)), ["calc::half::t"]);
}

/// A step no proof of the premises fills in is M032, which says to
/// write the proof in full; the checker takes nothing.
#[test]
fn a_step_that_shows_nothing_is_an_error() {
    for (rule, step) in [
        // r2 is missing: nothing gives (2).
        (
            "correct(half, (1)) |- correct(half, (2)) by ponens",
            "ponens",
        ),
        // Tollens argues against: it gives `!correct(half, (1))`.
        (
            "half::r2, !correct(half, (2)) |- correct(half, (1)) by tollens",
            "tollens",
        ),
        // A point is missing.
        (
            "correct(half, (0)) |- all x in (n: 0..2): correct(half, x) by cases",
            "cases",
        ),
        // Cases of what is no all or any.
        ("correct(half, (0)) |- correct(half, (1)) by cases", "cases"),
        // Outside the all's set.
        (
            "all x in (n: 0..5): correct(half, x) |- correct(half, (7)) by instance",
            "instance",
        ),
        (
            "correct(half, (7)) |- any x in (n: 0..5): correct(half, x) by witness",
            "witness",
        ),
        // The all's set is wider than the premise's.
        (
            "all x in (n: 0..3): correct(half, x) |- all y in (n: 0..5): correct(half, y) by intro",
            "intro",
        ),
        // Nothing conflicts.
        (
            "correct(half, (1)), half::r2 |- !correct(half, (1)) by reductio",
            "reductio",
        ),
    ] {
        let text = model(&format!("        rule bad: {rule};"));
        let errors = errors(&text);
        assert_eq!(errors.len(), 1, "{rule}: {errors:?}");
        assert!(
            errors[0].starts_with(&format!(
                "error[M032]: no `{step}` of the premises shows `bad`: write the proof in full, `by "
            )),
            "{rule}: {errors:?}"
        );
    }
}

/// Two proofs fill it in: M032 lists them, each as you would write it.
#[test]
fn a_step_two_proofs_fill_in_is_an_error() {
    let text = model(
        "        rule twice: correct(half, (3)), { h == 1 } |- any x in (n: 0..5): correct(half, x) by witness;
        rule both: correct(half, (1)), correct(half, (1)) |- any x in (n: 0..5): correct(half, x) by witness;",
    );
    let found = errors(&text);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(
        found[0].contains(
            "error[M032]: `by witness` shows `both` 2 ways: \
             `by witness[any x in (n: 0..5): correct(calc::half, x)](0, point(1))`, \
             `by witness[any x in (n: 0..5): correct(calc::half, x)](1, point(1))`; \
             write the one you mean"
        ),
        "{found:?}"
    );
    // Not shown: the checker does not choose.
    let m = parse(&text).unwrap();
    let e = explain(&m, &Ledger::default(), &Settings::default());
    let proved = |name: &str| {
        e.sources
            .iter()
            .any(|(t, s)| matches!(s, Source::Proof) && t.prop == Prop::Named(name.to_string()))
    };
    assert!(!proved("calc::half::both") && proved("calc::half::twice"));
    // Two in one place of a step.
    let text = model(
        "        rule pp: half::r2, correct(half, (1)), correct(half, (1)) |- correct(half, (2)) by ponens;",
    );
    let two = errors(&text);
    assert!(
        two[0].contains(
            "`by ponens` shows `pp` 2 ways: `by ponens(calc::half::r2, 1)`, `by ponens(calc::half::r2, 2)`"
        ),
        "{two:?}"
    );
    // The same premise twice by its path is one premise: one proof.
    let text = model(
        "        rule one: half::r2, half::r2, correct(half, (1)) |- correct(half, (2)) by ponens;",
    );
    assert_eq!(shown(&text), ["calc::half::one"]);
}

/// The short form prints as written.
#[test]
fn a_short_form_prints_as_written() {
    let text = model(STEPS);
    let printed = print(&parse(&text).unwrap());
    for step in [
        "ponens", "tollens", "cases", "instance", "witness", "intro", "reductio",
    ] {
        assert!(printed.contains(&format!(" by {step};\n")), "{printed}");
    }
    assert_eq!(parse(&printed).unwrap(), parse(&text).unwrap());
}
