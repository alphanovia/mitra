//! Red team, round 8: explain, blame, numbers and the command line.
//!
//! Each `x*`, `b*`, `n*` and `c*` test is the reproduction of one finding
//! the red team made on f1cbd14, each failing there, and asserts what the
//! docs promise since the fixes. `held_*` tests are what was attacked and
//! held. `cost_*` tests are ignored (slow, or large in memory): `cargo
//! test --release --test redteam8 -- --ignored --nocapture` prints sizes
//! and timings.
//!
//! The CLI tests write only under `target/redteam8` of this tree.

use mitra_lang::blame::{blame, Settings as BSettings, Suspect};
use mitra_lang::check::check;
use mitra_lang::evidence::{self, Ledger, Measurement, Review};
use mitra_lang::explain::{explain, Explanation, Settings, Source, Verdict, MODEL};
use mitra_lang::judgement::{Opinion, Prop, Taking, Term};
use mitra_lang::model::{Model, Path};
use mitra_lang::numbers::{fused, Numbers, Settings as NSettings};
use mitra_lang::parse::parse;
use mitra_lang::simulation::{Rational, Value};
use std::collections::BTreeSet;
use std::time::Instant;

// ------------------------------------------------------------------ helpers

fn model(src: &str) -> Model {
    let m = parse(src).unwrap();
    assert_eq!(check(&m), vec![], "the model checks");
    m
}
fn div() -> Model {
    let p = format!(
        "{}/examples/div/tied.mitra",
        concat!(env!("CARGO_MANIFEST_DIR"), "/../..")
    );
    model(&std::fs::read_to_string(p).unwrap())
}
fn path(p: &str) -> Path {
    Path::to(&p.split("::").collect::<Vec<_>>())
}
fn int(n: i64) -> Value {
    Value::Int(n.into())
}
fn real(n: i64) -> Value {
    Value::Real(Rational::new(n, 1).unwrap())
}
fn ports(these: &[(&str, Value)]) -> Vec<(Path, Value)> {
    these.iter().map(|(p, v)| (path(p), v.clone())).collect()
}
fn measured(
    inputs: &[(&str, Value)],
    outputs: &[(&str, Value)],
    seen: bool,
    by: &str,
) -> Measurement {
    Measurement {
        inputs: ports(inputs),
        outputs: ports(outputs),
        seen,
        count: 1,
        by: by.to_string(),
    }
}
fn divided(a: i64, b: i64, q: i64, by: &str) -> Measurement {
    measured(
        &[("calc::div::a", int(a)), ("calc::div::b", int(b))],
        &[("calc::div::q", int(q))],
        true,
        by,
    )
}
fn right(a: i64, b: i64) -> i64 {
    if b == 0 {
        0
    } else {
        a / b
    }
}
fn correct(f: &str, at: &[Value]) -> Prop {
    Prop::Correct {
        function: f.to_string(),
        at: Term::Point(at.to_vec()),
    }
}
fn named(n: &str) -> Prop {
    Prop::Named(n.to_string())
}
fn of(e: &Explanation, prop: &Prop) -> Verdict {
    e.verdicts
        .iter()
        .find(|v| &v.prop == prop)
        .unwrap_or_else(|| panic!("no verdict on {prop:?}"))
        .verdict
        .clone()
}
fn codes(e: &Explanation) -> Vec<String> {
    e.findings
        .iter()
        .map(|f| format!("{} at {}", f.code, f.at))
        .collect()
}
fn ledger(measurements: Vec<Measurement>) -> Ledger {
    Ledger {
        measurements,
        reviews: vec![],
    }
}
fn explained(model: &Model, ledger: &Ledger) -> Explanation {
    assert_eq!(evidence::check(model, ledger), vec![], "the ledger reads");
    explain(model, ledger, &Settings::default())
}
/// Each diagnosis meets every conflict, and no smaller set does.
fn sound(conflicts: &[BTreeSet<Suspect>], d: &BTreeSet<Suspect>) -> bool {
    let meets = |s: &BTreeSet<Suspect>| conflicts.iter().all(|c| !c.is_disjoint(s));
    meets(d)
        && d.iter().all(|s| {
            let mut less = d.clone();
            less.remove(s);
            !meets(&less)
        })
}
fn peak() -> String {
    std::fs::read_to_string("/proc/self/status")
        .unwrap_or_default()
        .lines()
        .find(|l| l.starts_with("VmHWM"))
        .unwrap_or("")
        .to_string()
}

// ------------------------------------------------------------- explain (X)

/// DEFECT, claim 1. A rule may conclude `!p` (syntax.md: a rule term
/// `!p`), but an opinion of `!p` is never joined with the opinion of
/// `p`: the model says div is not correct at (6, 2), a passing test says
/// it is, and explain calls it Explained with nothing in conflict.
#[test]
fn x1_a_negated_conclusion_is_never_set_against_its_proposition() {
    let model = model(
        r#"
subject calc {
    fn div(a: integer, b: integer) -> q: integer at "src/lib.rs/div" where {
        claim spec: if b == 0 { q == 0 } else { q * b <= a && a < (q + 1) * b };
        examples { (6, 2) -> 3 }
        rule never: div::spec |- !correct(div, (6, 2));
    }
}
"#,
    );
    let e = explained(&model, &ledger(vec![divided(6, 2, 3, "test:t")]));
    let at = correct("calc::div", &[int(6), int(2)]);
    // It follows false from {spec, never}: Contradicted, and blame has a
    // conflict to work on. Today: Explained, and no conflict.
    assert!(
        matches!(of(&e, &at), Verdict::Contradicted { .. }),
        "correct(div, (6, 2)) is {:?}, though the rule `never` says it is false",
        of(&e, &at)
    );
    let b = blame(&model, &e, &BSettings::default());
    assert!(!b.conflicts.is_empty(), "blame sees no conflict");
}

const WORLD: &str = r#"
environment sun {
    fn light() -> w: rational at "sky/sun";
    fn ray(k: integer) -> r: rational at "sky/ray";
}
subject stone {
    use sun::{light, ray};
    fn lit(h: rational) -> l: rational at "src/stone.rs/lit" where {
        claim bright: l <= light();
    }
    fn noise(a: integer) -> n: integer at "src/stone.rs/noise";
    fn beam(k: integer) -> s: rational at "src/stone.rs/beam" { ray(k) * 2.0 }
}
"#;

/// DEFECT, claim 2. A failed test of an opaque function that has claims
/// makes no taking and no finding: it vanishes. The same test of an
/// opaque function with no claims is X002 ("a failed test whose expected
/// value the model does not give").
#[test]
fn x2_a_failed_test_of_an_opaque_function_with_claims_vanishes() {
    let model = model(WORLD);
    let m = measured(
        &[("sun::light::w", real(2)), ("stone::lit::h", real(5))],
        &[("stone::lit::l", real(1))],
        false,
        "test:t",
    );
    let e = explained(&model, &ledger(vec![m]));
    let used = e.sources.values().any(|s| *s == Source::Measurement(0));
    assert!(
        used || !e.findings.is_empty(),
        "measurements::0 gave no taking and no finding: {:?}",
        codes(&e)
    );
}

/// DEFECT, claims 1 and 2. An opaque function's output may be given as
/// an input (V001 allows what a function with no body gave) and measured
/// as an output in the same measurement. The input is read as a row of
/// the function, so the model "expects" what the measurement itself
/// says: `correct(noise, (1))` is Explained with no X001, though nothing
/// in the model says what noise gives. With 5 in and 6 out, it is
/// Contradicted, again with no finding.
///
/// Fixed: V001 refuses a port on both sides, and explain leaves the
/// measurement out.
#[test]
fn x3_a_measurement_supplies_its_own_expected_value() {
    let model = model(WORLD);
    let at = correct("stone::noise", &[int(1)]);
    for out in [5, 6] {
        let m = measured(
            &[("stone::noise::a", int(1)), ("stone::noise::n", int(5))],
            &[("stone::noise::n", int(out))],
            true,
            "test:t",
        );
        let given = ledger(vec![m]);
        let refused: Vec<String> = evidence::check(&model, &given)
            .iter()
            .map(|e| e.code.to_string())
            .collect();
        assert_eq!(refused, vec!["V001"]);
        let e = explain(&model, &given, &Settings::default());
        let verdict = e
            .verdicts
            .iter()
            .find(|v| v.prop == at)
            .map(|v| v.verdict.clone());
        assert!(verdict.is_none(), "output {out}: verdict {verdict:?}");
    }
}

/// NIT, claim 2 (a false finding). An opaque function given as an input
/// needs its arguments given too (V006); they are read, as the row's
/// point, yet each is reported X005 "nothing this measurement measures
/// reads it".
#[test]
fn x4_x005_on_the_arguments_of_an_input_row() {
    let model = model(WORLD);
    let m = measured(
        &[
            ("stone::beam::k", int(1)),
            ("sun::ray::k", int(1)),
            ("sun::ray::r", real(3)),
        ],
        &[("stone::beam::s", real(6))],
        true,
        "test:t",
    );
    let e = explained(&model, &ledger(vec![m]));
    // The row was read: beam is explained from it, at its point in the
    // world the run read (finding 16).
    let at = Prop::Correct {
        function: "stone::beam".to_string(),
        at: Term::Given {
            point: vec![int(1)],
            world: mitra_lang::judgement::World(vec![(
                "sun::ray".to_string(),
                vec![int(1)],
                real(3),
            )]),
        },
    };
    assert_eq!(of(&e, &at), Verdict::Explained);
    assert_eq!(codes(&e), Vec::<String>::new(), "{:?}", e.findings);
}

/// DEFECT, claims 1 and 3. `f at x, by its ties` is documented as the
/// ties of f "and of every function it calls"; the code also adds the
/// ties of functions only f's claims call. f's body is `a + 1`; its claim
/// calls g. A failed test of f then blames g's tie, alone, as a
/// diagnosis, and contradicts it.
#[test]
fn x5_a_body_failure_blames_the_tie_of_a_function_only_a_claim_calls() {
    let model = model(
        r#"
subject s {
    fn f(a: integer) -> b: integer at "src/f.rs/f" { a + 1 } where {
        claim c: b <= g(a) + 10;
    }
    fn g(a: integer) -> b: integer at "src/g.rs/g" { a * 2 }
}
"#,
    );
    let m = measured(
        &[("s::f::a", int(1))],
        &[("s::f::b", int(5))],
        true,
        "test:f",
    );
    let e = explained(&model, &ledger(vec![m]));
    let g_tie = named("s::g at src/g.rs/g");
    assert_eq!(
        of(&e, &g_tie),
        Verdict::Unsupported,
        "g's tie is not in question"
    );
    let b = blame(&model, &e, &BSettings::default());
    let alone = BTreeSet::from([Suspect::Taking(Taking {
        prop: g_tie,
        value: true,
        count: 0,
        by: MODEL.to_string(),
    })]);
    assert!(
        !b.diagnoses.iter().any(|d| d.suspects == alone),
        "dropping g's tie is offered as a diagnosis of f's failure"
    );
}

// ---------------------------------------------------------------- blame (B)

/// DEFECT, claims 3 and 4. Two measurements that say the same (same
/// point, value, count and voice: a store with one line twice, say)
/// make one `Taking`; `sources` keeps only the first. (a) Blame's first
/// diagnosis drops measurements::0, but with it gone the conflict is
/// still there, through measurements::1. (b) Numbers count one run, not
/// two.
#[test]
fn b1_identical_measurements_merge_into_one_suspect() {
    let model = div();
    let wrong = || divided(7, 2, 4, "human:ana");
    let given = ledger(vec![wrong(), wrong(), divided(6, 2, 3, "test:ok")]);
    let e = explained(&model, &given);
    let b = blame(&model, &e, &BSettings::default());
    let first = &b.diagnoses[0].suspects;
    let mut rest = given.clone();
    // Fixed: the same measurement twice is one suspect, `same` says which.
    let mut dropped: Vec<usize> = first
        .iter()
        .filter_map(|s| match s {
            Suspect::Measurement(i) => Some(e.same.get(i).cloned().unwrap_or(vec![*i])),
            _ => None,
        })
        .flatten()
        .collect();
    dropped.sort();
    for i in dropped.iter().rev() {
        rest.measurements.remove(*i);
    }
    let after = blame(&model, &explained(&model, &rest), &BSettings::default());
    let n = Numbers::of(&e, &NSettings::default());
    let against = n.prop(&correct("calc::div", &[int(7), int(2)])).disbelief;
    let two_runs = fused(
        &[
            Taking {
                prop: named("p"),
                value: false,
                count: 1,
                by: "human:ana".into(),
            },
            Taking {
                prop: named("p"),
                value: false,
                count: 1,
                by: "human:ana".into(),
            },
        ],
        &NSettings::default(),
    )
    .disbelief;
    assert!(
        after.conflicts.is_empty() && (against - two_runs).abs() < 1e-9,
        "dropping {first:?} leaves {} conflicts; disbelief {against}, two runs give {two_runs}",
        after.conflicts.len()
    );
}

/// DEFECT, claim 3. "A diagnosis ... dropping it leaves none". An all by
/// its cases rests on the first measurement at each point (explain's
/// choice 5), so a second measurement at a point is in no conflict.
/// Blame offers {measurements::0, the all}; drop both and judge again:
/// the conflict is back, through measurements::2.
#[test]
fn b2_a_case_diagnosis_does_not_resolve_when_a_point_is_measured_twice() {
    let model = model(
        r#"
subject calc {
    fn div(a: integer, b: integer) -> q: integer at "src/lib.rs/div" where {
        claim spec: if b == 0 { q == 0 } else { q * b <= a && a < (q + 1) * b };
        claim bounded: b == 0 || q <= a;
        examples { (0, 1) -> 0, (1, 1) -> 1 }
        rule r: all x in (a: 0..2, b: 1..2): correct(div, x) |- div::bounded;
    }
}
"#,
    );
    let given = Ledger {
        measurements: vec![
            divided(0, 1, 0, "test:a"),
            divided(1, 1, 1, "test:b"),
            divided(0, 1, 0, "test:c"),
        ],
        reviews: vec![Review {
            from: None,
            seeing: vec![],
            prop: path("calc::div::bounded"),
            holds: false,
            by: "human:z".to_string(),
        }],
    };
    let e = explained(&model, &given);
    let b = blame(&model, &e, &BSettings::default());
    let all = e
        .verdicts
        .iter()
        .find(|v| matches!(v.prop, Prop::All { .. }))
        .unwrap()
        .prop
        .clone();
    let the_all = Suspect::Taking(Taking {
        prop: all,
        value: true,
        count: 0,
        by: MODEL.to_string(),
    });
    let d = BTreeSet::from([Suspect::Measurement(0), the_all.clone()]);
    assert!(
        b.diagnoses.iter().any(|x| x.suspects == d),
        "{:?}",
        b.diagnoses
    );
    // Apply it: drop measurements::0 (test:a) from the ledger; the all
    // stays a leaf of the model, so only conflicts without it count.
    let mut rest = given.clone();
    rest.measurements.remove(0);
    let b2 = blame(&model, &explained(&model, &rest), &BSettings::default());
    let left: Vec<_> = b2
        .conflicts
        .iter()
        .filter(|c| !c.contains(&the_all))
        .collect();
    assert!(!left.is_empty(), "the conflict stands, through test:c");
    // Fixed by saying so: every way to have each point would be too many
    // (explain's choice 5), so blame names the taking a diagnosis may
    // drop alone.
    let said: Vec<&str> = b
        .findings
        .iter()
        .filter(|f| f.code.to_string() == "B003")
        .map(|f| f.message.as_str())
        .collect();
    assert_eq!(said.len(), 1, "{:?}", b.findings);
    assert!(
        said[0].contains("by test:a") && said[0].contains("by test:c"),
        "{}",
        said[0]
    );
}

/// DEFECT, claims 3 and 5. Twelve failed tests, each of its own tied
/// function, are twelve conflicts with nothing in common: every
/// diagnosis has twelve suspects, and the breadth-first search runs out
/// of `tries` (100,000) on the 3^11 sets below. Blame gives no diagnosis
/// at all ("blame: 12 conflicts, the first 0 diagnoses"), after 2.7 s
/// and 1.7 GB. Ignored for the memory; `b3_small` shows the same with a
/// smaller budget.
#[test]
#[ignore]
fn b3_independent_failures_give_no_diagnosis() {
    let (model, given) = independent(12);
    let e = explained(&model, &given);
    let t = Instant::now();
    let b = blame(&model, &e, &BSettings::default());
    println!(
        "12 independent failures: blame {:.2?}, {} conflicts, {} diagnoses, {}",
        t.elapsed(),
        b.conflicts.len(),
        b.diagnoses.len(),
        peak()
    );
    assert!(!b.diagnoses.is_empty(), "no diagnosis");
}

/// The same, cheap: seven independent failures and a budget of 1,000
/// tries (3^6 = 729 sets of six come first).
#[test]
fn b3_small_independent_failures_give_no_diagnosis() {
    let (model, given) = independent(7);
    let e = explained(&model, &given);
    let b = blame(
        &model,
        &e,
        &BSettings {
            tries: 1000,
            ..BSettings::default()
        },
    );
    assert_eq!(b.conflicts.len(), 7);
    assert!(
        !b.diagnoses.is_empty(),
        "7 disjoint conflicts of 3, 2,187 diagnoses of 7: none found"
    );
}

fn independent(k: usize) -> (Model, Ledger) {
    let mut src = String::from("subject s {\n");
    for i in 0..k {
        src.push_str(&format!(
            "    fn f{i}(a: integer) -> b: integer at \"src/lib.rs/f{i}\" {{ a + 1 }}\n"
        ));
    }
    src.push_str("}\n");
    let given = ledger(
        (0..k)
            .map(|i| {
                measured(
                    &[(&format!("s::f{i}::a"), int(1))],
                    &[(&format!("s::f{i}::b"), int(5))],
                    true,
                    &format!("test:f{i}"),
                )
            })
            .collect(),
    );
    (model(&src), given)
}

/// NIT, claim 3 (`complete` is honest). With exactly as many diagnoses
/// as the budget, blame stops at the next set it pops and says
/// `complete: false`, B002 "more may exist", though there are no more.
#[test]
fn b4_complete_is_false_when_the_budget_meets_exactly_all() {
    // Conflicts {a, b}, {a, c}, {c, d}: three diagnoses.
    let t = |p: &str, value: bool, by: &str| Taking {
        prop: named(p),
        value,
        count: 1,
        by: by.to_string(),
    };
    let (a, b, c, d) = (
        t("p1", true, "test:a"),
        t("p1", false, "test:b"),
        t("p2", true, "test:c"),
        t("p2", false, "test:d"),
    );
    let (a3, c3) = (t("p3", true, "test:a"), t("p3", false, "test:c"));
    let mut e = Explanation::default();
    for (p, ts) in [
        ("p1", vec![a.clone(), b.clone()]),
        ("p2", vec![c.clone(), d.clone()]),
        ("p3", vec![a3.clone(), c3.clone()]),
    ] {
        e.opinions
            .insert(named(p), Opinion::taken(named(p), ts).unwrap());
    }
    for (x, i) in [(a, 0), (b, 1), (c, 2), (d, 3), (a3, 0), (c3, 2)] {
        e.sources.insert(x, Source::Measurement(i));
    }
    let model = div();
    let all = blame(&model, &e, &BSettings::default());
    assert_eq!((all.diagnoses.len(), all.complete), (3, true));
    let three = blame(
        &model,
        &e,
        &BSettings {
            diagnoses: 3,
            ..BSettings::default()
        },
    );
    assert_eq!(three.diagnoses.len(), 3);
    assert!(three.complete, "all 3 found, yet {:?}", three.findings);
}

/// DOC, claim 3 (ranking). "Ranked by size, then by kind: ... and proofs
/// last." Kinds are compared as sorted lists, smallest first, so a
/// diagnosis {model, proof} ([0, 4]) comes before {measurement,
/// measurement} ([1, 1]): one that drops a proof is not last.
#[test]
fn b5_a_diagnosis_that_drops_a_proof_ranks_before_two_measurements() {
    let t = |p: &str, value: bool, by: &str, count: u64| Taking {
        prop: named(p),
        value,
        count,
        by: by.to_string(),
    };
    let model_taking = t("calc::inputs", true, MODEL, 0);
    let proof = t("calc::div::everywhere", true, "machine:mitra-check", 1);
    let m1 = t("calc::inputs", false, "test:one", 1);
    let m2 = t("calc::div::everywhere", false, "test:two", 1);
    let mut e = Explanation::default();
    for (p, ts) in [
        ("calc::inputs", vec![model_taking.clone(), m1.clone()]),
        ("calc::div::everywhere", vec![proof.clone(), m2.clone()]),
    ] {
        e.opinions
            .insert(named(p), Opinion::taken(named(p), ts).unwrap());
    }
    e.sources.insert(model_taking, Source::Model);
    e.sources.insert(proof, Source::Proof);
    e.sources.insert(m1, Source::Measurement(0));
    e.sources.insert(m2, Source::Measurement(1));
    let b = blame(&div(), &e, &BSettings::default());
    let has_proof = |d: &BTreeSet<Suspect>| d.iter().any(|s| mitra_lang::blame::kind(&e, s) == 4);
    let first_proof = b
        .diagnoses
        .iter()
        .position(|d| has_proof(&d.suspects))
        .unwrap();
    let last_plain = b
        .diagnoses
        .iter()
        .rposition(|d| !has_proof(&d.suspects))
        .unwrap();
    assert!(
        first_proof > last_plain,
        "order: {:?}",
        b.diagnoses
            .iter()
            .map(|d| d
                .suspects
                .iter()
                .map(|s| mitra_lang::blame::kind(&e, s))
                .collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );
}

// -------------------------------------------------------------- numbers (N)

/// NIT, claim 4. b + d + u = 1 fails, and doubt leaves [0, 1], when a
/// voice's weight times its count overflows f64 (any weight above about
/// 1.8e308 / count, or infinity): r / (r + s + W) is inf / inf. Settings
/// are not checked (also: a negative weight, a base rate outside [0, 1]).
#[test]
fn n1_a_huge_voice_weight_gives_nan() {
    let model = div();
    let e = explained(
        &model,
        &ledger(vec![
            divided(7, 2, 4, "human:ana"),
            divided(6, 2, 3, "test:b"),
        ]),
    );
    let s = NSettings {
        voices: [("human:ana".to_string(), f64::MAX)].into(),
        ..NSettings::default()
    };
    let inf = NSettings {
        voices: [("human:ana".to_string(), f64::INFINITY)].into(),
        ..NSettings::default()
    };
    let one = fused(
        &[Taking {
            prop: named("p"),
            value: true,
            count: 2,
            by: "human:ana".into(),
        }],
        &s,
    );
    let n = Numbers::of(&e, &inf);
    let p = n.prop(&correct("calc::div", &[int(7), int(2)]));
    let doubts: Vec<f64> = e
        .sources
        .keys()
        .filter(|t| t.by == "human:ana")
        .map(|t| n.taking_doubt(t))
        .collect();
    let ok = |x: f64| (0.0..=1.0).contains(&x);
    assert!(
        ok(one.belief) && ok(p.disbelief) && doubts.iter().all(|d| ok(*d)),
        "fused {one:?}; prop {p:?}; doubts {doubts:?}"
    );
}

/// NIT, claims 1 and 4. A voice of the ledger may be `model` or
/// `machine:mitra-check` (V007 checks only that there is one). A store
/// measurement `by "model"` with count 50 is then Unsupported ("only the
/// model says it") while numbers give it a belief of 0.96.
///
/// Fixed: V007 refuses Mitra's own voices in a ledger.
#[test]
fn n2_a_ledger_voice_named_model_is_read_as_the_model() {
    let model = div();
    for voice in [MODEL, "machine:mitra-check"] {
        let mut m = divided(6, 2, 3, voice);
        m.count = 50;
        let refused: Vec<String> = evidence::check(&model, &ledger(vec![m]))
            .iter()
            .map(|e| e.code.to_string())
            .collect();
        assert_eq!(refused, vec!["V007"], "{voice}");
    }
}

// -------------------------------------------------------------- the CLI (C)

fn mitra(cwd: &std::path::Path, args: &[&str]) -> (i32, String) {
    let out = std::process::Command::new(env!("CARGO_BIN_EXE_mitra"))
        .current_dir(cwd)
        .args(args)
        .output()
        .unwrap();
    let text =
        String::from_utf8_lossy(&out.stdout).to_string() + &String::from_utf8_lossy(&out.stderr);
    (out.status.code().unwrap_or(-1), text)
}
/// A project under `target/redteam8/<name>`: the example's code and
/// report, the model with its tie written in it too.
fn project(name: &str) -> std::path::PathBuf {
    let root = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../.."));
    let dir = root.join("target").join("redteam8").join(name);
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("proj/src")).unwrap();
    let ex = root.join("examples/div");
    std::fs::copy(ex.join("src/lib.rs"), dir.join("proj/src/lib.rs")).unwrap();
    std::fs::copy(ex.join("report.txt"), dir.join("proj/report.txt")).unwrap();
    let text = std::fs::read_to_string(ex.join("model.mitra"))
        .unwrap()
        .replace(
            "fn div(a: integer, b: integer) -> q: integer where {",
            "fn div(a: integer, b: integer) -> q: integer at \"src/lib.rs/div\" where {",
        );
    std::fs::write(dir.join("proj/model.mitra"), text).unwrap();
    // The project's root: tie paths are read from it.
    std::fs::write(dir.join("proj/mitra.toml"), "").unwrap();
    std::fs::write(
        dir.join("proj/store.mitra"),
        "measure calc::div (7, 2) -> 4 by \"human:ana\";\n",
    )
    .unwrap();
    dir
}

/// DEFECT, claim 6. Code paths are made relative to the model's folder
/// with `strip_prefix` on the paths as typed, so `./proj/model.mitra`
/// or an absolute model path ties div at `proj/src/lib.rs/div`: a second
/// tie beside the model's `src/lib.rs/div`, and one more diagnosis.
#[test]
fn c1_the_spelling_of_a_path_changes_the_ties() {
    let dir = project("c1");
    let abs = dir.join("proj/model.mitra");
    let abs = abs.to_str().unwrap();
    let rest = [
        "--code",
        "proj/src",
        "--report",
        "proj/report.txt",
        "--store",
        "proj/store.mitra",
    ];
    let mut outs = vec![];
    for m in ["proj/model.mitra", "./proj/model.mitra", abs] {
        let mut args = vec!["blame", m];
        args.extend(rest);
        let (code, out) = mitra(&dir, &args);
        assert_eq!(code, 0, "{out}");
        let ties = out
            .lines()
            .find(|l| l.starts_with("evidence:"))
            .unwrap()
            .to_string();
        let tail = out
            .lines()
            .find(|l| l.starts_with("blame:"))
            .unwrap()
            .to_string();
        outs.push(format!("{m}: {ties} / {tail}"));
    }
    assert!(
        outs.iter()
            .all(|o| o.contains("1 ties") && o.contains("3 diagnoses")),
        "{outs:#?}"
    );
}

/// DEFECT, claim 6 (nothing given is dropped quietly). A second
/// `--store` replaces the first, which is never read, with no word.
#[test]
fn c2_a_second_store_drops_the_first() {
    let dir = project("c2");
    let p = dir.join("proj");
    std::fs::write(
        p.join("other.mitra"),
        "measure calc::div (9, 3) -> 3 by \"human:bo\";\n",
    )
    .unwrap();
    let (code, out) = mitra(
        &p,
        &[
            "explain",
            "model.mitra",
            "--store",
            "store.mitra",
            "--store",
            "other.mitra",
        ],
    );
    let refused = code == 2;
    let both = out.contains("evidence: 2 measurements");
    assert!(refused || both, "exit {code}: {out}");
}

/// DEFECT, claim 6. `--code` with `// mitra: measures` tests and no
/// `--report`: the reader reads the points, but without a report it
/// makes no measurement and no finding; explain says 0 measurements and
/// exits 0.
#[test]
fn c3_measuring_tests_without_a_report_vanish() {
    let dir = project("c3");
    let p = dir.join("proj");
    let (code, out) = mitra(&p, &["explain", "model.mitra", "--code", "src"]);
    assert_eq!(code, 0, "{out}");
    assert!(
        out.contains("finding[R") || !out.contains("evidence: 0 measurements"),
        "three measuring tests, no word: {out}"
    );
}

/// NIT, claim 6. A stray positional argument, or an unknown flag with no
/// value, is reported as "`x` needs a value" (exit 2 is right; the text
/// is not).
#[test]
fn c4_a_stray_argument_is_said_to_need_a_value() {
    let dir = project("c4");
    let p = dir.join("proj");
    for stray in ["extra", "--bogus"] {
        let (code, out) = mitra(&p, &["explain", "model.mitra", stray]);
        assert_eq!(code, 2);
        assert!(!out.contains("needs a value"), "{stray}: {out}");
    }
}

// ---------------------------------------------------------- what held

/// Claim 4 held for valid settings: on 60 seeded ledgers of div (counts
/// to 200, failed tests, reviews), every verdict's b + d + u = 1, each
/// in [0, 1], and every ranked doubt in [0, 1], under four settings.
#[test]
fn held_numbers_invariants() {
    let model = div();
    let mut checked = 0;
    for seed in 0..60u64 {
        let given = seeded(seed);
        let e = explained(&model, &given);
        let b = blame(&model, &e, &BSettings::default());
        for s in [
            NSettings::default(),
            NSettings {
                prior_weight: 0.0,
                ..NSettings::default()
            },
            NSettings {
                base_rate: 0.0,
                ..NSettings::default()
            },
            NSettings {
                base_rate: 1.0,
                ..NSettings::default()
            },
        ] {
            let n = Numbers::of(&e, &s);
            for v in &e.verdicts {
                let x = n.prop(&v.prop);
                let sum = x.belief + x.disbelief + x.uncertainty;
                assert!((sum - 1.0).abs() < 1e-9, "seed {seed}: {x:?}");
                for y in [x.belief, x.disbelief, x.uncertainty, x.expected] {
                    assert!((-1e-12..=1.0 + 1e-12).contains(&y), "seed {seed}: {x:?}");
                }
                checked += 1;
            }
            for (_, doubt) in n.rank(&b) {
                assert!((0.0..=1.0).contains(&doubt));
            }
        }
    }
    assert!(checked > 1000);
}

/// Claim 3 held on seeded ledgers: every diagnosis meets every conflict
/// and is smallest, none drops a claim of the model, sizes never fall,
/// and `complete` is true when the search ran out.
#[test]
fn held_blame_diagnoses_are_hitting_sets() {
    let model = div();
    for seed in 0..60u64 {
        let e = explained(&model, &seeded(seed));
        let b = blame(&model, &e, &BSettings::default());
        let mut last = 0;
        for d in &b.diagnoses {
            assert!(
                sound(&b.conflicts, &d.suspects),
                "seed {seed}: {:?}",
                d.suspects
            );
            assert!(d.suspects.len() >= last);
            last = d.suspects.len();
            for t in &d.drop {
                let claim = matches!(&t.prop, Prop::Named(n) if n == "calc::div::spec" || n == "calc::div::bounded");
                assert!(!(claim && t.by == MODEL), "seed {seed}: drops {t:?}");
            }
        }
        // Explained: some rest with nothing the model alone says.
        for v in &e.verdicts {
            if v.verdict == Verdict::Explained {
                let o = &e.opinions[&v.prop];
                assert!(o
                    .rests_for()
                    .iter()
                    .any(|r| r.iter().all(|t| t.by != MODEL)));
            }
        }
    }
}

fn seeded(seed: u64) -> Ledger {
    let mut x = seed
        .wrapping_mul(6364136223846793005)
        .wrapping_add(1442695040888963407);
    let mut next = move || {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (x >> 33) as i64
    };
    let mut given = Ledger::default();
    for k in 0..(3 + next() % 6) {
        let (a, b) = (next() % 10, next() % 10);
        let q = if next() % 3 == 0 {
            right(a, b) + 1
        } else {
            right(a, b)
        };
        let mut m = divided(a, b, q, &format!("test:{k}"));
        m.count = 1 + (next() % 200) as u64;
        m.seen = next() % 5 != 0;
        given.measurements.push(m);
    }
    if next() % 2 == 0 {
        given.reviews.push(Review {
            from: None,
            seeing: vec![],
            prop: path("calc::div::spec"),
            holds: next() % 2 == 0,
            by: "human:r".into(),
        });
    }
    given
}

/// Claim 6 held: exit codes as documented.
#[test]
fn held_cli_exit_codes() {
    let dir = project("held");
    let p = dir.join("proj");
    std::fs::write(
        p.join("bad.mitra"),
        "measure calc::div (7, 2) -> 4 by \"human:a\"\n",
    )
    .unwrap();
    std::fs::write(
        p.join("zero.mitra"),
        "measure calc::div (7, 2) -> 4 count 0 by \"human:a\";\n",
    )
    .unwrap();
    std::fs::write(
        p.join("broken.mitra"),
        "subject s { fn f(a: integer) -> b: integer }",
    )
    .unwrap();
    for (args, want) in [
        (vec![], 2),
        (vec!["judge", "model.mitra"], 2),
        (vec!["explain"], 2),
        (vec!["explain", "nothere.mitra"], 2),
        (vec!["check", "model.mitra", "--code", "src"], 2),
        (vec!["explain", "model.mitra", "--code"], 2),
        (vec!["explain", "model.mitra", "--report", "nothere.txt"], 2),
        (vec!["check", "broken.mitra"], 1),
        (vec!["explain", "model.mitra", "--store", "bad.mitra"], 1),
        (vec!["explain", "model.mitra", "--store", "zero.mitra"], 1),
        (vec!["check", "model.mitra"], 0),
        (
            vec![
                "blame",
                "model.mitra",
                "--code",
                "src",
                "--report",
                "report.txt",
                "--store",
                "store.mitra",
            ],
            0,
        ),
    ] {
        let (code, out) = mitra(&p, &args);
        assert_eq!(code, want, "{args:?}: {out}");
    }
}

// ------------------------------------------------------------ cost (claim 5)

/// DEFECT, claim 5. An ensure from n claims, each reviewed by r voices:
/// the ensure rests on (r + 1)^n sets, rebuilt and joined every round
/// (`smallest` is quadratic within a least taking). Measured on this
/// machine: 6x2 with the ensure reviewed false 0.3 s; 6x3 2.2 s; 7x3
/// with no review against 6.3 s; 7x3 with one 33 s; 8x2 10.5 s; 8x3
/// 466 s (262,186 rests).
#[test]
#[ignore]
fn cost_ensure_from_reviewed_claims() {
    let (n, r) = (7, 3);
    let mut src = String::from("subject s {\n");
    for i in 0..n {
        src.push_str(&format!(
            "    fn f{i}(a: integer) -> b: integer at \"src/s.rs/f{i}\" {{ a + 1 }} where {{ claim c: a < b; }}\n"
        ));
    }
    let list: Vec<String> = (0..n).map(|i| format!("f{i}::c")).collect();
    src.push_str(&format!(
        "    ensure ok: \"good\" from ({});\n}}\n",
        list.join(", ")
    ));
    let model = model(&src);
    let mut given = Ledger::default();
    for i in 0..n {
        for k in 0..r {
            given.reviews.push(Review {
                from: None,
                seeing: vec![],
                prop: path(&format!("s::f{i}::c")),
                holds: true,
                by: format!("human:r{k}"),
            });
        }
    }
    given.reviews.push(Review {
        from: None,
        seeing: vec![],
        prop: path("s::ok"),
        holds: false,
        by: "human:z".to_string(),
    });
    let t = Instant::now();
    let e = explained(&model, &given);
    let took = t.elapsed();
    println!("ensure from {n} claims, {r} reviews each: explain {took:.2?}");
    assert!(took.as_secs() < 5, "{took:?}");
    let _ = e;
}

/// DEFECT, claim 5. 50 functions, each calling the one before (one
/// part), a claim each, 300 unit tests of which 30 fail: explain 18.8 s
/// (chains of 20: 0.34 s, 30: 2.3 s, 40: 7.4 s; 5 parts of 10: 0.12 s;
/// no failure: 0.09 s). Every round clones every premise's whole opinion
/// (its rests against too) for every rule, written rules included.
#[test]
#[ignore]
fn cost_deep_call_chain_with_failures() {
    let per = 50;
    let mut src = String::from("subject p {\n");
    for i in 0..per {
        let body = if i == 0 {
            "a + 1".to_string()
        } else {
            format!("f{}(a) + 1", i - 1)
        };
        src.push_str(&format!(
            "    fn f{i}(a: integer) -> b: integer at \"src/p.rs/f{i}\" {{ {body} }} where {{ claim c: a < b; }}\n"
        ));
    }
    src.push_str("}\n");
    let model = model(&src);
    let mut given = Ledger::default();
    let mut k = 0;
    for i in 0..per {
        for a in 0..6i64 {
            let b = a + i as i64 + 1 + i64::from(k % 10 == 3);
            given.measurements.push(measured(
                &[(&format!("p::f{i}::a"), int(a))],
                &[(&format!("p::f{i}::b"), int(b))],
                true,
                &format!("test:t{k}"),
            ));
            k += 1;
        }
    }
    let t = Instant::now();
    let _e = explained(&model, &given);
    let took = t.elapsed();
    println!("chain of {per}, {k} tests, 30 failing: explain {took:.2?}");
    assert!(took.as_secs() < 5, "{took:?}");
}

/// DEFECT, claim 5. `cases` stops after `Settings::cases + 1` points of
/// the set, but `Set::points` reads every point of the ranges: a set of
/// 5 points, `(a: 0..10000, b: 0..10000) where a == b && a < 5`, reads
/// 10^8, with an empty ledger. 1000: 0.4 s; 3000: 3.5 s; 10000: 39 s.
#[test]
#[ignore]
fn cost_a_sparse_set_with_no_evidence() {
    let n = 10000;
    let src = format!(
        r#"
subject calc {{
    fn div(a: integer, b: integer) -> q: integer at "src/lib.rs/div" where {{
        claim spec: if b == 0 {{ q == 0 }} else {{ q * b <= a && a < (q + 1) * b }};
        rule diagonal: div::spec |- all x in (a: 0..{n}, b: 0..{n}) where a == b && a < 5: correct(div, x);
    }}
}}
"#
    );
    let model = model(&src);
    let t = Instant::now();
    let _e = explain(&model, &Ledger::default(), &Settings::default());
    let took = t.elapsed();
    println!("a set of 5 points in {n}x{n}, empty ledger: explain {took:.2?}");
    assert!(took.as_secs() < 5, "{took:?}");
}

/// DEFECT, claim 5. Blame's `tries` bounds the sets popped, not those
/// queued: each pop queues one set per suspect of a conflict. Six parts,
/// each a chain of 8 tied functions whose top fails at two points:
/// 0 diagnoses, 6 s, 4.8 GB peak. With an all rule and an ensure in
/// each part (24 conflicts): 0 diagnoses, 11 s, 10.8 GB; six parts of 5:
/// 10 diagnoses, 4.9 s, 4.6 GB; eight parts of 3: 0 diagnoses, 6 s,
/// 5.3 GB. Run alone.
#[test]
#[ignore]
fn cost_blame_memory_on_independent_modules() {
    let (k, d) = (6, 8);
    let mut src = String::new();
    for p in 0..k {
        src.push_str(&format!("subject p{p} {{\n"));
        for i in 0..d {
            let body = if i == 0 {
                "a + 1".to_string()
            } else {
                format!("f{}(a) + 1", i - 1)
            };
            src.push_str(&format!(
                "    fn f{i}(a: integer) -> b: integer at \"src/p{p}.rs/f{i}\" {{ {body} }} where {{ claim c: a < b; }}\n"
            ));
        }
        src.push_str("}\n");
    }
    let model = model(&src);
    let mut given = Ledger::default();
    for p in 0..k {
        for a in 0..2i64 {
            given.measurements.push(measured(
                &[(&format!("p{p}::f{}::a", d - 1), int(a))],
                &[(&format!("p{p}::f{}::b", d - 1), int(a + d as i64 + 1))],
                true,
                &format!("test:p{p}_{a}"),
            ));
        }
    }
    let e = explained(&model, &given);
    let t = Instant::now();
    let b = blame(&model, &e, &BSettings::default());
    println!(
        "{k} modules, chains of {d}: blame {:.2?}, {} conflicts, {} diagnoses, {}",
        t.elapsed(),
        b.conflicts.len(),
        b.diagnoses.len(),
        peak()
    );
    assert!(!b.diagnoses.is_empty());
}
