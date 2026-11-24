//! Red team of finding 9 (A8, A9): remembered calls, and the budgets of
//! steps and depth through calls. A chain through `if` is linear; deep
//! bodies through calls stop at the budget with a finding, not an abort.

use mitra_lang::agree::{agree, Agreement, Settings};
use mitra_lang::check::check;
use mitra_lang::parse::parse;
use mitra_lang::simulation::Budget;

/// `n` functions, each calling the one before in both branches of an `if`.
fn chain(n: usize) -> String {
    let mut out = vec![
        "subject s {".to_string(),
        "    fn f0(x: integer) -> y: integer { x + 1 }".to_string(),
    ];
    for i in 1..=n {
        out.push(format!(
            "    fn f{i}(x: integer) -> y: integer {{ if x < 0 {{ f{0}(x) }} else {{ f{0}(x) + 1 }} }} where {{ claim c: 0 <= y || x < 0; }}",
            i - 1
        ));
    }
    out.push("}".to_string());
    out.join("\n")
}

/// `n` functions, each a body `depth` levels deep calling the one before.
fn deep(n: usize, depth: usize) -> String {
    let mut out = vec!["subject s {".to_string()];
    for i in 0..n {
        let mut body = match i {
            0 => "x".to_string(),
            _ => format!("f{}(x)", i - 1),
        };
        for _ in 0..depth {
            body = format!("({body} + 1)");
        }
        out.push(format!(
            "    fn f{i}(x: integer) -> y: integer {{ {body} }} where {{ claim c: x < y; }}"
        ));
    }
    out.push("}".to_string());
    out.join("\n")
}

#[test]
fn a_chain_of_20_through_if_is_linear() {
    let model = parse(&chain(20)).unwrap();
    assert_eq!(check(&model), vec![]);
    let start = std::time::Instant::now();
    let (agreed, _) = agree(&model, &Settings::default());
    let took = start.elapsed();
    // 11.6 s in release before; now a few ms (debug: well under 2 s).
    assert!(took.as_secs_f64() < 2.0, "{took:?}");
    assert_eq!(agreed.len(), 40);
    // And 60 levels, which would have taken centuries.
    let start = std::time::Instant::now();
    agree(&parse(&chain(60)).unwrap(), &Settings::default());
    assert!(start.elapsed().as_secs_f64() < 5.0, "{:?}", start.elapsed());
}

/// Run on a thread of 256 MiB: the parser, in a debug build, needs about
/// 20 KiB a level of parentheses (a finding of its own), more than the
/// budget's 2 MiB for 1,000 levels allows.
fn on_a_thread<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap()
}

#[test]
fn deep_bodies_through_calls_stop_at_the_depth_budget() {
    // 40 functions of 1,000 levels each aborted the CLI before.
    let (agreed, findings) = on_a_thread(|| {
        let model = parse(&deep(40, 1000)).unwrap();
        assert_eq!(check(&model), vec![]);
        agree(&model, &Settings::default())
    });
    let stopped: Vec<_> = agreed
        .iter()
        .filter_map(|a| match &a.agreement {
            Agreement::Unknown { why } if why.contains("depth budget") => Some(why),
            _ => None,
        })
        .collect();
    assert!(!stopped.is_empty(), "{findings:?}");
    assert!(stopped[0]
        .contains("the depth budget, 1000 levels of bodies through calls, reached at `s::f"));
    // A007 says it, with the same words.
    assert!(findings
        .iter()
        .any(|f| f.code.to_string() == "A007" && f.message.contains("depth budget")));
}

#[test]
fn under_the_depth_budget_nothing_stops() {
    // 4 functions of 200 levels: 800 levels at most, under 1,000.
    let (agreed, _) = on_a_thread(|| agree(&parse(&deep(4, 200)).unwrap(), &Settings::default()));
    assert!(agreed
        .iter()
        .all(|a| !matches!(&a.agreement, Agreement::Unknown { why } if why.contains("budget"))));
}

#[test]
fn the_step_budget_is_a_setting() {
    // Only the last function has a claim: nothing is remembered before
    // it, so its first point needs a call of each of the six before it.
    let text = chain(6).replace(" where { claim c: 0 <= y || x < 0; }", "");
    let text = text.replacen(
        "f5(x) + 1 } }",
        "f5(x) + 1 } } where { claim c: 0 <= y || x < 0; }",
        1,
    );
    let model = parse(&text).unwrap();
    assert_eq!(check(&model), vec![]);
    let few = Settings {
        budget: Budget {
            steps: 3,
            depth: 1_000,
        },
        ..Settings::default()
    };
    let (agreed, _) = agree(&model, &few);
    let why = agreed.iter().find_map(|a| match &a.agreement {
        Agreement::Unknown { why } if why.contains("step budget") => Some(why.clone()),
        _ => None,
    });
    assert_eq!(
        why.as_deref(),
        Some("the step budget, 3 calls, reached at `s::f2`"),
        "{agreed:#?}"
    );
    // Never a failure: a budget reached decides nothing.
    assert!(agreed
        .iter()
        .all(|a| !matches!(a.agreement, Agreement::Fails { .. })));
}

/// A chain of `n` calls, each at another argument (no call is
/// remembered), read only where `x * x == 2`, which no whole
/// number keeps but bounds cannot tell: by bounds the chain is read, and
/// past 1,000 levels it reaches the depth budget; exactly, never.
fn guarded(n: usize) -> String {
    let mut out = vec![
        "subject s {".to_string(),
        "    fn g0(x: integer) -> y: integer { x }".to_string(),
    ];
    for i in 1..=n {
        out.push(format!(
            "    fn g{i}(x: integer) -> y: integer {{ g{}(x + 1) }}",
            i - 1
        ));
    }
    out.push(format!(
        "    fn top(x: integer) -> y: integer {{ if x * x == 2 {{ g{n}(x) }} else {{ x }} }} where {{ claim c: y == x; }}"
    ));
    out.push("}".to_string());
    out.join("\n")
}

/// Finding 24: a budget reached by bounds is said, not left for points
/// to cover in silence.
#[test]
fn a_budget_reached_by_bounds_is_said() {
    let (agreed, findings) = on_a_thread(|| {
        let model = parse(&guarded(1_100)).unwrap();
        assert_eq!(check(&model), vec![]);
        agree(&model, &Settings::default())
    });
    let why = agreed.iter().find_map(|a| match &a.agreement {
        Agreement::Unknown { why } if a.against.contains("`c`") => Some(why.clone()),
        _ => None,
    });
    let why = why.unwrap_or_else(|| panic!("{agreed:?}"));
    assert!(
        why.starts_with("not decided by bounds, which stopped at the depth budget, 1000 levels of bodies through calls"),
        "{why}"
    );
    assert!(findings
        .iter()
        .any(|f| f.code.to_string() == "A007" && f.message.contains("which stopped at")));
}

/// Finding 24: a text deeper than the parser's budget is P011, never a
/// stack that overflows; the budget is a setting.
#[test]
fn a_deep_text_is_p011() {
    use mitra_lang::parse::{parse_with, Settings as ParseSettings};
    let nested = |n: usize| {
        let mut body = "x".to_string();
        for _ in 0..n {
            body = format!("({body} + 1)");
        }
        format!("subject s {{ fn f(x: integer) -> y: integer {{ {body} }} }}")
    };
    let e = on_a_thread(move || parse(&nested(100_000)).unwrap_err());
    assert_eq!(e.code.to_string(), "P011");
    assert!(
        e.message.contains("deeper than 2000 levels"),
        "{}",
        e.message
    );
    let small = ParseSettings { depth: 10 };
    assert!(parse_with(&nested(8), &small).is_ok());
    assert_eq!(
        parse_with(&nested(10), &small)
            .unwrap_err()
            .code
            .to_string(),
        "P011"
    );
    let nots = format!(
        "subject s {{ fn f(x: bool) -> y: bool {{ {}x }} }}",
        "!".repeat(20)
    );
    assert_eq!(
        parse_with(&nots, &small).unwrap_err().code.to_string(),
        "P011"
    );
    let types = "subject s { fn f(x: ((((((((((integer,),),),),),),),),),)) -> y: integer { 0 } }";
    assert_eq!(
        parse_with(types, &small).unwrap_err().code.to_string(),
        "P011"
    );
}
