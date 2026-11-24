//! Red team of finding 7 (A2): agree reads a name's comparisons together
//! (`Ranges::kept`) and cuts numbers at zero. Both must stay sound: what
//! agree says holds is true at every point tried, and what it says no
//! value keeps is kept at none.

use mitra_lang::agree::{agree, Agreement, Settings};
use mitra_lang::catalogue::A004;
use mitra_lang::check::check;
use mitra_lang::expression::eval;
use mitra_lang::model::calls;
use mitra_lang::parse::parse;
use mitra_lang::simulation::{Exact, Value};
use num_bigint::BigInt;
use proptest::prelude::*;
use std::collections::BTreeMap;
use std::sync::Arc;

/// A random term over `x`, `z` and, in a claim, `y`: all `Int`.
fn term(names: &'static [&'static str]) -> impl Strategy<Value = String> {
    let leaf = prop_oneof![
        prop::sample::select(names).prop_map(str::to_string),
        (-3i32..4).prop_map(|n| if n < 0 {
            format!("(0 - {})", -n)
        } else {
            n.to_string()
        }),
    ];
    leaf.prop_recursive(3, 12, 2, |inner| {
        (inner.clone(), inner, 0..3usize)
            .prop_map(|(a, b, o)| format!("({a} {} {b})", ["+", "-", "*"][o]))
    })
}

/// A comparison, often of a name or a square with a small number: where
/// the cuts decide.
fn cmp() -> impl Strategy<Value = String> {
    let side = prop_oneof![
        term(&["x", "z", "y"]),
        prop::sample::select(&["x", "z", "y"][..]).prop_map(|n| format!("{n} * {n}")),
        prop::sample::select(&["x", "z", "y"][..]).prop_map(str::to_string),
    ];
    let small = (-3i32..4).prop_map(|n| {
        if n < 0 {
            format!("(0 - {})", -n)
        } else {
            n.to_string()
        }
    });
    let pair = prop_oneof![
        (side.clone(), small.clone()),
        (small, side.clone()).prop_map(|(a, b)| (a, b)),
        (side.clone(), side),
    ];
    (pair, 0..3usize).prop_map(|((a, b), o)| format!("{a} {} {b}", ["<", "<=", "=="][o]))
}

fn claim() -> impl Strategy<Value = String> {
    prop_oneof![
        cmp(),
        (cmp(), cmp()).prop_map(|(a, b)| format!("{a} && {b}")),
    ]
}

/// Every point of `x`, `z`, `y` from -6 to 6.
fn points() -> impl Iterator<Item = BTreeMap<String, Value>> {
    let n = |i: i64| Value::Int(BigInt::from(i));
    (-6..=6).flat_map(move |x| {
        (-6..=6).flat_map(move |z| {
            (-6..=6).map(move |y| {
                BTreeMap::from([
                    ("x".to_string(), n(x)),
                    ("z".to_string(), n(z)),
                    ("y".to_string(), n(y)),
                ])
            })
        })
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(3000))]

    #[test]
    fn what_agree_decides_by_bounds_is_true_at_points(
        body in proptest::option::of(term(&["x", "z"])),
        claims in prop::collection::vec(claim(), 1..3),
    ) {
        let body_text = body.as_ref().map_or(";".to_string(), |b| format!(" {{ {b} }}"));
        let mut text = format!("subject s {{\n    fn f(x: integer, z: integer) -> y: integer{body_text}");
        text = text.trim_end_matches(';').to_string();
        text.push_str(" where {\n");
        for (i, c) in claims.iter().enumerate() {
            text.push_str(&format!("        claim c{i} = {c};\n"));
        }
        text.push_str("    }\n}\n");
        let Ok(model) = parse(&text) else { return Ok(()) };
        if !check(&model).is_empty() {
            return Ok(());
        }
        // No points: what agree decides, it decides by bounds alone.
        let settings = Settings { points: 0, ..Settings::default() };
        let (agreed, findings) = agree(&model, &settings);
        let exact = Exact { calls: Arc::new(calls(&model).unwrap()) };
        let f = &model.parts[0].functions[0];
        let at = |p: &BTreeMap<String, Value>| -> BTreeMap<String, Value> {
            let mut p = p.clone();
            if let Some(body) = &f.body {
                match eval(&exact, &p, body) {
                    Ok(v) => _ = p.insert("y".to_string(), v),
                    Err(_) => _ = p.remove("y"),
                }
            }
            p
        };
        for a in &agreed {
            if a.agreement != Agreement::Holds || !a.against.starts_with("claim `") {
                continue;
            }
            let name = a.against.trim_start_matches("claim `").trim_end_matches('`');
            let c = f.claims.iter().find(|c| c.name == name).unwrap();
            for p in points().filter(|p| p["y"] == Value::Int(BigInt::from(0)) || f.body.is_none()) {
                let p = at(&p);
                if !p.contains_key("y") { continue; }
                prop_assert_eq!(eval(&exact, &p, &c.holds), Ok(Value::Truth(true)), "{}\nat {:?}", text, p);
            }
        }
        if findings.iter().any(|x| x.code == A004) {
            for p in points() {
                let all = f.claims.iter().all(|c| eval(&exact, &p, &c.holds) == Ok(Value::Truth(true)));
                prop_assert!(!all, "A004, yet all hold at {:?}\n{}", p, text);
            }
        }
    }
}

#[test]
fn the_two_examples_of_finding_7() {
    let text = "subject s {
    fn f(x: integer) -> q: integer where {
        claim pos: 0 < q;
        claim neg: q < 0;
    }
    fn sq(x: rational) -> y: rational { x * x } where {
        claim never_negative: 0.0 <= y;
    }
}";
    let (agreed, findings) = agree(&parse(text).unwrap(), &Settings::default());
    let codes: Vec<String> = findings
        .iter()
        .map(|f| format!("{} at {}", f.code, f.at))
        .collect();
    assert_eq!(codes, ["A004 at s::f"]);
    assert!(agreed
        .iter()
        .any(|a| a.against == "claim `never_negative`" && a.agreement == Agreement::Holds));
}

#[test]
fn a_real_strict_bound_stays_open_to_its_end() {
    // On Real, `1.0 < q && q < 1.0` keeps no value; bounds keep the end
    // 1 in, so they cannot tell, and agree says unknown, not A004. (At 0
    // the cut at zero tells: `0.0 < q && q < 0.0` is A004.)
    let text = "subject s {
    fn f(x: rational) -> q: rational where {
        claim pos: 1.0 < q;
        claim neg: q < 1.0;
    }
}";
    let (_, findings) = agree(&parse(text).unwrap(), &Settings::default());
    let codes: Vec<String> = findings.iter().map(|f| f.code.to_string()).collect();
    assert_eq!(codes, ["A007"]);
}

#[test]
fn cuts_away_from_zero() {
    // `5 < q && q < 3`: the cut at zero does not decide it; the
    // comparisons read together do.
    let text = "subject s {
    fn f(x: integer) -> q: integer where {
        claim a: 5 < q;
        claim b: q < 3;
    }
}";
    let (_, findings) = agree(
        &parse(text).unwrap(),
        &Settings {
            points: 0,
            ..Settings::default()
        },
    );
    let codes: Vec<String> = findings.iter().map(|f| f.code.to_string()).collect();
    assert_eq!(codes, ["A004"]);
}
