//! Round 9, a differential fuzz of agree on finite domains: every
//! `Holds` is checked at every point, every `Fails` at its point, every
//! A004 at every value of what the claims read. Ignored: run with
//! `cargo test --release --test redteam9_fuzz -- --ignored`.

mod common;

use common::Rng;
use mitra_core::agree::{agree, Agreement, Settings};
use mitra_core::check::check;
use mitra_core::expression::eval;
use mitra_core::model::calls;
use mitra_core::parse::parse;
use mitra_core::simulation::{Exact, Value};
use std::collections::BTreeMap;
use std::sync::Arc;

fn atom(r: &mut Rng, with_r: bool) -> String {
    let mut atoms = vec!["a", "b", "p.0", "p.1", "true", "false"];
    if with_r {
        atoms.extend(["r", "r"]);
    }
    r.pick(&atoms).to_string()
}

fn expr(r: &mut Rng, depth: usize, with_r: bool) -> String {
    if depth == 0 || r.below(4) == 0 {
        return atom(r, with_r);
    }
    match r.below(5) {
        0 => format!("!({})", expr(r, depth - 1, with_r)),
        1 => format!(
            "({} && {})",
            expr(r, depth - 1, with_r),
            expr(r, depth - 1, with_r)
        ),
        2 => format!(
            "({} || {})",
            expr(r, depth - 1, with_r),
            expr(r, depth - 1, with_r)
        ),
        3 => format!(
            "({} == {})",
            expr(r, depth - 1, with_r),
            expr(r, depth - 1, with_r)
        ),
        _ => format!(
            "(if {} {{ {} }} else {{ {} }})",
            expr(r, depth - 1, with_r),
            expr(r, depth - 1, with_r),
            expr(r, depth - 1, with_r)
        ),
    }
}

fn b(x: bool) -> &'static str {
    if x {
        "true"
    } else {
        "false"
    }
}

fn points() -> Vec<[bool; 4]> {
    let mut out = vec![];
    for i in 0..16u32 {
        out.push([i & 1 != 0, i & 2 != 0, i & 4 != 0, i & 8 != 0]);
    }
    out
}

fn values(p: [bool; 4]) -> BTreeMap<String, Value> {
    let tuple = match mitra_core::simulation::tuple(vec![
        mitra_core::simulation::Type::Truth,
        mitra_core::simulation::Type::Truth,
    ]) {
        mitra_core::simulation::Type::Record(r) => r,
        _ => unreachable!(),
    };
    BTreeMap::from([
        ("a".to_string(), Value::Truth(p[0])),
        ("b".to_string(), Value::Truth(p[1])),
        (
            "p".to_string(),
            Value::Record(tuple, vec![Value::Truth(p[2]), Value::Truth(p[3])]),
        ),
    ])
}

#[test]
#[ignore]
fn agree_on_finite_domains() {
    let mut false_holds = vec![];
    for seed in 0..3000u64 {
        let mut r = Rng::new(seed);
        let body = expr(&mut r, 3, false);
        let claims: Vec<String> = (0..1 + r.below(2)).map(|_| expr(&mut r, 3, true)).collect();
        // The body's value at some points, as examples, a point at times twice.
        let text0 = format!(
            "subject s {{ fn f(a: bool, b: bool, p: (bool, bool)) -> r: bool {{ {body} }} }}"
        );
        let m0 = parse(&text0).unwrap();
        assert_eq!(check(&m0), vec![], "{text0}");
        let exact = Exact {
            calls: Arc::new(calls(&m0).unwrap()),
        };
        let body_e = m0.parts[0].functions[0].body.clone().unwrap();
        let mut rows = vec![];
        let dups = seed % 2 == 0;
        for _ in 0..r.below(8) {
            let p = points()[r.below(16)];
            let v = eval(&exact, &values(p), &body_e).unwrap();
            let Value::Truth(v) = v else { unreachable!() };
            let row = format!(
                "({}, {}, ({}, {})) -> {}",
                b(p[0]),
                b(p[1]),
                b(p[2]),
                b(p[3]),
                b(v)
            );
            rows.push(row.clone());
            if dups && r.coin() {
                rows.push(row);
            }
        }
        let claims_text: Vec<String> = claims
            .iter()
            .enumerate()
            .map(|(i, c)| format!("claim c{i} = {c};"))
            .collect();
        let examples = match rows.is_empty() {
            true => String::new(),
            false => format!("examples {{ {} }}", rows.join(", ")),
        };
        let text = format!(
            "subject s {{ fn f(a: bool, b: bool, p: (bool, bool)) -> r: bool {{ {body} }} where {{ {} {examples} }} }}",
            claims_text.join(" ")
        );
        let model = parse(&text).unwrap();
        assert_eq!(check(&model), vec![], "{text}");
        let (agreed, _) = agree(&model, &Settings::default());
        let f = &model.parts[0].functions[0];
        // Claims no value keeps together: none of the 32 values of what
        // they read keeps them all.
        let clash = agreed.iter().any(|a| {
            a.against == "its claims together" && matches!(a.agreement, Agreement::Fails { .. })
        });
        if clash {
            for p in points() {
                for rv in [false, true] {
                    let mut vs = values(p);
                    vs.insert("r".to_string(), Value::Truth(rv));
                    let kept = f
                        .claims
                        .iter()
                        .all(|c| eval(&exact, &vs, &c.holds) == Ok(Value::Truth(true)));
                    assert!(!kept, "seed {seed}: kept at {p:?}, r = {rv}: {text}");
                }
            }
        }
        for a in &agreed {
            if !a.against.starts_with("claim `") {
                continue;
            }
            let name = a
                .against
                .trim_start_matches("claim `")
                .trim_end_matches('`');
            let c = f.claims.iter().find(|c| c.name == name).unwrap();
            // The truth at every point, the body read as `r`.
            let at_all: Vec<bool> = points()
                .into_iter()
                .map(|p| {
                    let mut vs = values(p);
                    let rv = eval(&exact, &vs, &body_e).unwrap();
                    vs.insert("r".to_string(), rv);
                    eval(&exact, &vs, &c.holds) == Ok(Value::Truth(true))
                })
                .collect();
            match &a.agreement {
                Agreement::Holds if at_all.iter().any(|t| !t) => {
                    let has_dup = rows.iter().enumerate().any(|(i, x)| rows[..i].contains(x));
                    false_holds.push(format!("dup={has_dup} seed {seed}: {text}"));
                }
                _ => {}
            }
        }
    }
    let without: Vec<&String> = false_holds
        .iter()
        .filter(|s| s.starts_with("dup=false"))
        .collect();
    eprintln!(
        "{} false holds, {} without a duplicated row",
        false_holds.len(),
        without.len()
    );
    for s in false_holds.iter().take(3) {
        eprintln!("{s}");
    }
    assert!(without.is_empty(), "{:?}", without);
}

fn num(r: &mut Rng, depth: usize, with_r: bool) -> String {
    if depth == 0 || r.below(3) == 0 {
        let mut atoms = vec!["x", "y", "0.0", "1.0", "2.0", "-1.0", "1/2"];
        if with_r {
            atoms.extend(["r", "r"]);
        }
        return r.pick(&atoms).to_string();
    }
    match r.below(6) {
        0 => format!(
            "({} + {})",
            num(r, depth - 1, with_r),
            num(r, depth - 1, with_r)
        ),
        1 => format!(
            "({} - {})",
            num(r, depth - 1, with_r),
            num(r, depth - 1, with_r)
        ),
        2 => format!(
            "({} * {})",
            num(r, depth - 1, with_r),
            num(r, depth - 1, with_r)
        ),
        3 => format!(
            "({} / {})",
            num(r, depth - 1, with_r),
            num(r, depth - 1, with_r)
        ),
        4 => format!(
            "(if {} {{ {} }} else {{ {} }})",
            truth(r, depth - 1, with_r),
            num(r, depth - 1, with_r),
            num(r, depth - 1, with_r)
        ),
        _ => format!(
            "(let z = {} in {})",
            num(r, depth - 1, with_r),
            num(r, depth - 1, with_r)
        ),
    }
}

fn truth(r: &mut Rng, depth: usize, with_r: bool) -> String {
    match r.below(7) {
        0 => format!("({} < {})", num(r, depth, with_r), num(r, depth, with_r)),
        1 => format!("({} <= {})", num(r, depth, with_r), num(r, depth, with_r)),
        2 => format!("({} == {})", num(r, depth, with_r), num(r, depth, with_r)),
        3 => format!(
            "within({}, {}, {})",
            num(r, depth, with_r),
            num(r, depth, with_r),
            num(r, depth, with_r)
        ),
        4 if depth > 0 => format!("!({})", truth(r, depth - 1, with_r)),
        5 if depth > 0 => format!(
            "({} || {})",
            truth(r, depth - 1, with_r),
            truth(r, depth - 1, with_r)
        ),
        _ if depth > 0 => format!(
            "({} && {})",
            truth(r, depth - 1, with_r),
            truth(r, depth - 1, with_r)
        ),
        _ => format!("({} < {})", num(r, 0, with_r), num(r, 0, with_r)),
    }
}

#[test]
#[ignore]
fn agree_on_numbers_by_bounds() {
    use mitra_core::simulation::Rational;
    let grid: Vec<Value> = (-12..=12)
        .map(|n| Value::Real(Rational::new(n, 4).unwrap()))
        .collect();
    let mut bad = vec![];
    for seed in 0..20000u64 {
        let mut r = Rng::new(seed);
        let body = num(&mut r, 3, false);
        let claim = truth(&mut r, 2, true);
        let text = format!(
            "subject s {{ fn f(x: Real, y: Real) -> r: Real {{ {body} }} where {{ claim c = {claim}; }} }}"
        );
        let Ok(model) = parse(&text) else { continue };
        if !check(&model).is_empty() {
            continue;
        }
        let exact = Exact {
            calls: Arc::new(calls(&model).unwrap()),
        };
        let (agreed, _) = agree(
            &model,
            &Settings {
                points: 4,
                ..Settings::default()
            },
        );
        let f = &model.parts[0].functions[0];
        let body_e = f.body.clone().unwrap();
        for a in &agreed {
            // A claim's Holds: true everywhere. (Its claims together hold
            // where one point keeps them.)
            if a.agreement != Agreement::Holds || a.against != "claim `c`" {
                continue;
            }
            for x in &grid {
                for y in &grid {
                    let mut vs = BTreeMap::from([
                        ("x".to_string(), x.clone()),
                        ("y".to_string(), y.clone()),
                    ]);
                    let rv = eval(&exact, &vs, &body_e);
                    let got = match rv {
                        Ok(v) => {
                            vs.insert("r".to_string(), v);
                            eval(&exact, &vs, &f.claims[0].holds)
                        }
                        // The body has no value: the claim holds only if it does not read r.
                        Err(_) => {
                            let e = mitra_core::expression::substitute(
                                &f.claims[0].holds,
                                "r",
                                &body_e,
                            );
                            eval(&exact, &vs, &e)
                        }
                    };
                    if got != Ok(Value::Truth(true)) {
                        bad.push(format!(
                            "seed {seed} at x={x:?} y={y:?}: {text} gives {got:?}"
                        ));
                    }
                }
            }
        }
    }
    for b in bad.iter().take(3) {
        eprintln!("{b}");
    }
    assert!(bad.is_empty(), "{} false holds", bad.len());
}

/// Token-level mutations of models that check accepts: whatever check
/// accepts, agree and explain read without a panic.
#[test]
#[ignore]
fn accepted_models_are_read_without_a_panic() {
    use mitra_core::evidence::Ledger;
    use mitra_core::explain::{explain, Settings as ExplainSettings};
    // The red team's probes, when a folder of them is given (`PROBES`),
    // and the models.
    let dir = std::env::var("PROBES").unwrap_or_default();
    let mut corpus: Vec<String> = vec![];
    for entry in std::fs::read_dir(&dir).into_iter().flatten() {
        let p = entry.unwrap().path();
        if p.extension().is_some_and(|x| x == "mitra") {
            let text = std::fs::read_to_string(&p).unwrap();
            if text.len() < 3000 {
                corpus.push(text);
            }
        }
    }
    for name in ["fall", "div"] {
        corpus.push(
            std::fs::read_to_string(format!(
                "{}/models/{name}.mitra",
                env!("CARGO_MANIFEST_DIR")
            ))
            .unwrap(),
        );
    }
    let words: Vec<String> = corpus
        .iter()
        .flat_map(|t| t.split_whitespace().map(str::to_string).collect::<Vec<_>>())
        .collect();
    let (mut accepted, mut panics) = (0, vec![]);
    // `SEEDS=a..b` for part of the run; each model slower than a second
    // is said.
    let (from, to) = std::env::var("SEEDS")
        .ok()
        .and_then(|s| {
            let (a, b) = s.split_once("..")?;
            Some((a.parse().ok()?, b.parse().ok()?))
        })
        .unwrap_or((0u64, 20000u64));
    for seed in from..to {
        let mut r = Rng::new(seed);
        let base = r.pick(&corpus).clone();
        let mut toks: Vec<String> = base.split_whitespace().map(str::to_string).collect();
        for _ in 0..1 + r.below(3) {
            let i = r.below(toks.len());
            match r.below(4) {
                0 => {
                    toks.remove(i);
                }
                1 => toks.insert(i, r.pick(&words).clone()),
                2 => toks[i] = r.pick(&words).clone(),
                _ => {
                    let j = r.below(toks.len());
                    toks.swap(i, j);
                }
            }
            if toks.is_empty() {
                break;
            }
        }
        let text = toks.join(" ");
        let Ok(model) = parse(&text) else { continue };
        if !check(&model).is_empty() {
            continue;
        }
        accepted += 1;
        if std::env::var("TRACE").is_ok() {
            eprintln!("start {seed}: {text}");
        }
        let start = std::time::Instant::now();
        let outcome = std::panic::catch_unwind(|| {
            let _ = agree(
                &model,
                &Settings {
                    points: 16,
                    ..Settings::default()
                },
            );
            let _ = explain(&model, &Ledger::default(), &ExplainSettings::default());
        });
        if start.elapsed().as_secs() >= 1 {
            eprintln!("seed {seed}: {:?}", start.elapsed());
            if std::env::var("SHOW").is_ok() {
                eprintln!("{text}");
            }
        }
        if outcome.is_err() {
            panics.push(text);
        }
    }
    eprintln!("{accepted} accepted, {} panics", panics.len());
    for p in panics.iter().take(3) {
        eprintln!("PANIC ON: {p}");
    }
    assert!(panics.is_empty());
}

/// Each body within the stated depth, the chain of calls over it: check
/// passes on an 8 MiB stack (the CLI's main thread), and agree aborts.
#[test]
#[ignore]
fn depth_composes_through_calls() {
    let k: usize = std::env::var("K")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(40);
    let d: usize = std::env::var("D")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(1000);
    let mib: usize = std::env::var("MIB")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(8);
    let mut out = vec!["subject s {".to_string()];
    for i in 0..k {
        let start = if i == 0 {
            "x".to_string()
        } else {
            format!("f{}(x)", i - 1)
        };
        out.push(format!(
            "fn f{i}(x: Real) -> y: Real {{ {start}{} }}",
            " + 1.0".repeat(d)
        ));
    }
    out.push(format!(
        "fn top(x: Real) -> y: Real {{ f{}(x) }} where {{ claim c = y == y; }}",
        k - 1
    ));
    out.push("}".to_string());
    let text = out.join("\n");
    let stage = std::env::var("STAGE").unwrap_or_default();
    std::thread::Builder::new()
        .stack_size(mib << 20)
        .spawn(move || {
            let model = parse(&text).unwrap();
            assert_eq!(check(&model), vec![]);
            eprintln!("parsed and checked");
            if stage == "agree" {
                let (agreed, _) = agree(&model, &Settings::default());
                eprintln!("agreed: {}", agreed.len());
            }
        })
        .unwrap()
        .join()
        .unwrap();
}
