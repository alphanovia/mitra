//! The dry run (branch claude/project-thread-p45kjs): machine types, the
//! trace of a run as a value, route claims and rows, symbolic runs, the
//! model checker (`mc`) and the search for needed promises (`search`).
//! Each test is one behaviour, said in its name.

use mitra_lang::agree::{agree, Settings};
use mitra_lang::catalogue::Code;
use mitra_lang::check::check;
use mitra_lang::model::Model;
use mitra_lang::parse::parse;
use mitra_lang::search::{self, Verdict};

fn model(text: &str) -> Model {
    let model = parse(text).unwrap_or_else(|e| panic!("{e:?}\n{text}"));
    assert_eq!(check(&model), vec![], "{text}");
    model
}

fn findings(text: &str) -> Vec<String> {
    let (_, found) = agree(&model(text), &Settings::default());
    found.iter().map(|f| f.to_string()).collect()
}

fn verdict(report: &search::Report, claim: &str) -> Verdict {
    report
        .outcomes
        .iter()
        .find(|o| o.claim == claim)
        .unwrap_or_else(|| panic!("no claim `{claim}`: {:?}", report.outcomes))
        .verdict
        .clone()
}

fn code_of(text: &str) -> Vec<Code> {
    let m = parse(text).unwrap();
    check(&m).iter().map(|e| e.code).collect()
}

const DISK: &str = "environment Disk {
    fn write(name: [~char], text: [~char]) -> ok: bool;
    fn exists(name: [~char]) -> b: bool;
}
";

// Machine types.

#[test]
fn a_machine_sum_that_overflows_gives_no_value() {
    let text = "subject N {\n    fn add(a: i64, b: i64) -> c: i64 { a + b } where {\n        claim more: a <= c || b < 0;\n    }\n}";
    let report = search::mc(&model(text), &search::Settings::default(), None);
    assert_eq!(verdict(&report, "more"), Verdict::Fails, "{report:?}");
    let shown: Vec<String> = report.findings.iter().map(|f| f.to_string()).collect();
    assert!(shown[0].contains("9223372036854775807"), "{shown:?}");
}

#[test]
fn a_machine_type_is_not_an_integer() {
    let text = "subject N {\n    fn f(a: i32) -> c: integer { a } where { claim t: true; }\n}";
    assert!(parse(text).is_err() || !check(&parse(text).unwrap()).is_empty());
    let widened = "subject N {\n    fn f(a: i32) -> c: integer { integer::from(a) } where { claim t: c == integer::from(a); }\n}";
    model(widened);
}

#[test]
fn every_u8_is_tried_when_points_allow() {
    let text = "subject N {\n    fn half(a: u8) -> c: u8 { a / 2 } where {\n        claim below: c <= a;\n        examples { (255) -> 127 }\n    }\n}";
    let all = Settings { points: 256, ..Settings::default() };
    let (_, found) = agree(&model(text), &all);
    assert!(found.is_empty(), "{found:?}");
    // Fewer points: small values and the ends only, and the claim unknown.
    let found = findings(text);
    assert!(found[0].starts_with("finding[A007]"), "{found:?}");
}

#[test]
fn a_literal_out_of_its_machine_type_is_refused() {
    let text = "subject N {\n    fn f(a: u8) -> c: u8 { a + 300 } where { claim t: true; }\n}";
    assert!(parse(text).is_err() || !check(&parse(text).unwrap()).is_empty());
}

// Routes: claims and rows.

#[test]
fn a_route_row_holds_and_breaks() {
    let text = format!(
        "{DISK}subject Notes {{\n    use Disk::{{write, exists}};\n    fn save(text: [~char]) -> n: integer {{\n        if Disk::exists(\"notes.txt\") {{ 0 }} else {{ let _w = Disk::write(\"notes.txt\", text); text.len() }}\n    }} where {{\n        examples {{\n            given Disk::exists(_) -> false, Disk::write(_, _) -> true when (\"hi\") then 2, once Disk::write(_, \"hi\"),\n            given Disk::exists(_) -> true when (\"hi\") then 0, once Disk::write(_, _)\n        }}\n    }}\n}}"
    );
    let found = findings(&text);
    assert_eq!(found.len(), 1, "{found:?}");
    assert!(found[0].starts_with("finding[A023]: example 1"), "{found:?}");
}

#[test]
fn a_route_claim_on_a_bodiless_function_is_m035() {
    let text = format!("{DISK}subject W {{\n    use Disk::write;\n    fn d(t: [~char]) -> n: integer where {{ claim q: never Disk::write(_, _); }}\n}}");
    assert_eq!(code_of(&text).iter().map(|c| c.to_string()).collect::<Vec<_>>(), ["M035"]);
}

#[test]
fn a_route_naming_no_function_is_m036_alone() {
    let text = format!("{DISK}subject W {{\n    fn d(t: integer) -> n: integer {{ t }} where {{ claim q: never Disk::nope(_, _); }}\n}}");
    assert_eq!(code_of(&text).iter().map(|c| c.to_string()).collect::<Vec<_>>(), ["M036"]);
}

#[test]
fn a_binder_named_once_is_p018() {
    let text = format!("{DISK}subject W {{\n    use Disk::write;\n    fn d(t: integer) -> n: integer {{ t }} where {{ claim q: Disk::write(f, _) only after Disk::exists(_); }}\n}}");
    let e = parse(&text).unwrap_err();
    assert_eq!(e.code.to_string(), "P018");
}

#[test]
fn a_route_counting_an_unreached_function_is_a022() {
    let text = format!("{DISK}subject W {{\n    use Disk::write;\n    fn d(t: integer) -> n: integer {{ t }} where {{ claim q: never Disk::write(_, _) || n == 0; }}\n}}");
    assert!(findings(&text).iter().any(|f| f.starts_with("finding[A022]")));
}

#[test]
fn a_route_claim_is_left_to_mc_by_check() {
    let text = "subject W {\n    fn d(t: integer) -> n: integer { t } where { claim q: calls(W::d).len() == 0; }\n}";
    let found = findings(text);
    assert!(found.iter().any(|f| f.contains("`mitra mc` reads every path")), "{found:?}");
}

// The model checker.

const NOTES: &str = "subject Notes {
    use Disk::{write, exists};
    fn save(text: [~char]) -> n: integer {
        if Disk::exists(\"notes.txt\") { 0 } else { let _w = Disk::write(\"notes.txt\", text); 1 }
    } where {
        claim writes_once: at most 1 Disk::write(_, _);
        claim order: Disk::write(f, _) only after Disk::exists(f) -> false;
        claim wrote: n == 0 || calls(Disk::write).any { c -> c.ok };
    }
}";

#[test]
fn mc_holds_on_every_path_and_finds_a_counterexample_row() {
    let m = model(&format!("{DISK}{NOTES}"));
    let report = search::mc(&m, &search::Settings::default(), None);
    assert_eq!(verdict(&report, "writes_once"), Verdict::Holds);
    assert_eq!(verdict(&report, "order"), Verdict::Holds);
    assert_eq!(verdict(&report, "wrote"), Verdict::Fails);
    let shown: Vec<String> = report.findings.iter().map(|f| f.to_string()).collect();
    assert!(
        shown.iter().any(|f| f.starts_with("finding[A024]") && f.contains("Disk::write(\"notes.txt\", ") && f.contains("-> false")),
        "{shown:?}"
    );
}

#[test]
fn a_promise_rules_paths_out() {
    let promised = DISK.replace(
        "fn write(name: [~char], text: [~char]) -> ok: bool;",
        "fn write(name: [~char], text: [~char]) -> ok: bool where { claim works: ok; }",
    );
    let m = model(&format!("{promised}{NOTES}"));
    let report = search::mc(&m, &search::Settings::default(), None);
    assert_eq!(verdict(&report, "wrote"), Verdict::Holds, "{report:?}");
    let o = report.outcomes.iter().find(|o| o.claim == "wrote").unwrap();
    assert!(o.excluded > 0);
}

#[test]
fn a_promise_no_path_keeps_is_said() {
    let promised = DISK.replace(
        "fn exists(name: [~char]) -> b: bool;",
        "fn exists(name: [~char]) -> b: bool where { claim never: b && !b; }",
    );
    let m = model(&format!("{promised}{NOTES}"));
    let report = search::mc(&m, &search::Settings::default(), None);
    match verdict(&report, "wrote") {
        Verdict::Unknown { why } => assert!(why.contains("`Disk::exists::never` is false"), "{why}"),
        v => panic!("{v:?}"),
    }
}

// The search for needed promises.

#[test]
fn search_finds_a_needed_promise() {
    let text = "environment Clock {
    fn now() -> t: integer;
}
subject Timer {
    use Clock::now;
    fn left(deadline: integer) -> l: integer { deadline - Clock::now() } where {
        claim within: l <= deadline;
        examples { given Clock::now() -> 3 when (10) then 7 }
    }
}";
    let report = search::search(&model(text), &search::Settings::default(), true);
    let shown: Vec<String> = report.findings.iter().map(|f| f.to_string()).collect();
    assert!(
        shown.iter().any(|f| f.starts_with("finding[A025]") && f.contains("needs `Clock::now()` to give at least 0 (with it promised, it holds")),
        "{shown:?}"
    );
    let stated = text.replace("fn now() -> t: integer;", "fn now() -> t: integer where { claim after_start: t >= 0; }");
    let report = search::search(&model(&stated), &search::Settings::default(), true);
    assert!(report.findings.iter().all(|f| f.code.to_string() != "A025"), "{:?}", report.findings);
}

// Red team, round 1.

#[test]
fn whole_literals_in_a_sequence_of_a_record_settle_to_its_machine_type() {
    let text = "subject P { record { x: u8, ys: [i16] } }\nsubject N {\n    fn f(p: P) -> c: u8 { p.x / 2 } where { examples { (P { x: 9, ys: [1, -2] }) -> 4 } }\n}";
    assert_eq!(findings(text), Vec::<String>::new());
}

#[test]
fn a_claim_no_example_runs_is_said_by_search() {
    let text = "subject N {\n    fn f(a: integer) -> c: integer { a } where { claim same: c == a; }\n}";
    let report = search::search(&model(text), &search::Settings::default(), true);
    match verdict(&report, "same") {
        Verdict::Unknown { why } => assert!(why.contains("no example runs it"), "{why}"),
        v => panic!("{v:?}"),
    }
}

// Round 2.

#[test]
fn steps_find_a_lost_update_among_interleavings() {
    let text = "subject enum Event { ReadA, WriteA, ReadB, WriteB }
subject record Shared { count: u8, a: u8, b: u8, done: u8 }
subject Counter {
    fn start() -> s: Shared { Shared { count: 0, a: 0, b: 0, done: 0 } }
    fn step(s: Shared, e: Event) -> t: Shared {
        match e {
            ReadA => Shared { count: s.count, a: s.count, b: s.b, done: s.done },
            WriteA => Shared { count: s.a + 1, a: s.a, b: s.b, done: s.done + 1 },
            ReadB => Shared { count: s.count, a: s.a, b: s.count, done: s.done },
            WriteB => Shared { count: s.b + 1, a: s.a, b: s.b, done: s.done + 1 },
        }
    } where {
        claim none_lost: t.count == t.done;
    }
}";
    let m = model(text);
    let settings = search::Settings { steps: 3, ..search::Settings::default() };
    let report = search::mc_steps(&m, &settings, "Counter::step", Some("Counter::start")).unwrap();
    assert_eq!(verdict(&report, "none_lost"), Verdict::Fails);
    let shown = report.findings[0].to_string();
    assert!(shown.contains("the steps: e1 = "), "{shown}");
    // Two steps cannot lose an update: each write follows a read of 0.
    let two = search::Settings { steps: 2, ..search::Settings::default() };
    let report = search::mc_steps(&m, &two, "Counter::step", Some("Counter::start")).unwrap();
    assert_eq!(verdict(&report, "none_lost"), Verdict::Fails, "WriteA twice from 0 is a lost update too");
}

#[test]
fn search_halves_to_the_exact_bound_of_a_machine_number() {
    let text = "environment Sensor { fn read() -> v: u8; }
subject Gauge {
    use Sensor::read;
    fn pct(x: u8) -> p: u8 { Sensor::read() / 2 } where {
        claim small: p <= 1;
        examples { given Sensor::read() -> 2 when (0) then 1 }
    }
}";
    let report = search::search(&model(text), &search::Settings::default(), true);
    let shown: Vec<String> = report.findings.iter().map(|f| f.to_string()).collect();
    assert!(shown.iter().any(|f| f.contains("needs `Sensor::read()` to give at most 3 ")), "{shown:?}");
}

#[test]
fn search_reaches_a_bound_no_sample_holds_at() {
    let text = "environment Clock { fn now() -> t: integer; }
subject Timer {
    use Clock::now;
    fn left(deadline: integer) -> l: integer { deadline - Clock::now() } where {
        claim margin: l <= deadline - 100;
        examples { given Clock::now() -> 300 when (1000) then 700 }
    }
}";
    let report = search::search(&model(text), &search::Settings::default(), true);
    let shown: Vec<String> = report.findings.iter().map(|f| f.to_string()).collect();
    assert!(shown.iter().any(|f| f.contains("needs `Clock::now()` to give at least 100 ")), "{shown:?}");
}

#[test]
fn a_relation_of_two_results_is_listed_by_points() {
    let text = "environment Clock { fn now() -> t: integer; fn zone() -> z: integer; }
subject Timer {
    use Clock::{now, zone};
    fn left(deadline: integer) -> l: integer { deadline - Clock::now() - Clock::zone() } where {
        claim within: l <= deadline;
        examples { given Clock::now() -> 3, Clock::zone() -> 0 when (10) then 7 }
    }
}";
    let report = search::search(&model(text), &search::Settings::default(), true);
    let shown: Vec<String> = report.findings.iter().map(|f| f.to_string()).collect();
    assert!(shown.iter().any(|f| f.contains("only if not (")), "{shown:?}");
}

// Round 3.

#[test]
fn an_arm_that_binds_nothing_runs_once() {
    let text = "subject enum Look { Found(integer), Missing }
environment Disk {
    fn look(name: [~char]) -> l: Look;
    fn write(name: [~char], text: [~char]) -> ok: bool;
}
subject N {
    use Disk::{look, write};
    fn f(a: [~char]) -> n: integer {
        match Disk::look(a) { Look::Found(k) => k, Look::Missing => { let _w = Disk::write(a, a); 0 } }
    } where {
        examples { given Disk::look(_) -> Look::Missing, Disk::write(_, _) -> true when (\"a\") then 0, once Disk::write(_, _) }
    }
}";
    let found = findings(text);
    assert!(found.is_empty(), "{found:?}");
}

#[test]
fn an_empty_sequence_in_a_model_row_settles_for_its_routes() {
    let text = format!("{DISK}subject N {{
    use Disk::write;
    fn f(xs: [[~char]]) -> n: integer {{ let ys = xs ++ [\"b\"]; ys.fold(n: 0) {{ y -> {{ let _w = Disk::write(y, y); n + 1 }} }} }}
}}
examples {{ given Disk::write(_, _) -> true when N::f([]) then 1, once Disk::write(\"b\", _) }}");
    let found = findings(&text);
    assert!(found.iter().all(|f| !f.contains("examples::0")), "{found:?}");
}

#[test]
fn search_finds_what_a_row_needs_to_have_a_value() {
    let text = "environment Sensor { fn read() -> v: u8; }
subject Gauge {
    use Sensor::read;
    fn twice(x: u8) -> p: u8 { Sensor::read() * 2 } where {
        examples { given Sensor::read() -> 2 when (0) then 4 }
    }
}";
    let report = search::search(&model(text), &search::Settings::default(), true);
    let shown: Vec<String> = report.findings.iter().map(|f| f.to_string()).collect();
    assert!(shown.iter().any(|f| f.contains("the run needs `Sensor::read()` to give at most 127 ")), "{shown:?}");
    assert!(report.outcomes.is_empty(), "the run is no claim: {:?}", report.outcomes);
}

// The model's arithmetic: `[model]` in mitra.toml (the user's S1, S2).

fn findings_with(text: &str, how: mitra_lang::simulation::Arithmetic) -> Vec<String> {
    let mut m = model(text);
    m.arithmetic = how;
    let (_, found) = agree(&m, &Settings::default());
    found.iter().map(|f| f.to_string()).collect()
}

fn overflow(o: mitra_lang::simulation::Overflow) -> mitra_lang::simulation::Arithmetic {
    mitra_lang::simulation::Arithmetic { overflow: o, ..Default::default() }
}

fn floor() -> mitra_lang::simulation::Arithmetic {
    let division = mitra_lang::simulation::Division::Floor;
    mitra_lang::simulation::Arithmetic { division, ..Default::default() }
}

#[test]
fn a_sum_past_u8_fails_wraps_or_clamps_as_the_model_says() {
    use mitra_lang::simulation::Overflow::{Clamp, Fail, Wrap};
    let row = |gives: &str| {
        format!("subject N {{\n    fn add(a: u8, b: u8) -> c: u8 {{ a + b }} where {{\n        examples {{ (200, 100) -> {gives} }}\n    }}\n}}")
    };
    assert_eq!(findings_with(&row("44"), overflow(Wrap)), Vec::<String>::new());
    assert_eq!(findings_with(&row("255"), overflow(Clamp)), Vec::<String>::new());
    assert_ne!(findings_with(&row("44"), overflow(Fail)), Vec::<String>::new());
    assert_ne!(findings_with(&row("255"), overflow(Wrap)), Vec::<String>::new());
}

#[test]
fn a_literal_past_its_type_fails_even_where_the_model_wraps() {
    use mitra_lang::simulation::Overflow::Wrap;
    let text = "subject N {\n    fn f(a: u8) -> c: u8 { a } where {\n        examples { (300) -> 44 }\n    }\n}";
    assert!(parse(text).is_err() || !check(&parse(text).unwrap()).is_empty() || !findings_with(text, overflow(Wrap)).is_empty());
}

#[test]
fn bounds_follow_the_overflow_setting() {
    use mitra_lang::simulation::Overflow::{Clamp, Fail, Wrap};
    // A sum of two u8 at most 255: clamped, always; wrapped, always, and
    // never 255 itself past it; failing, it may have no value.
    let text = "subject N {\n    fn add(a: u8, b: u8) -> c: u8 { a + b } where {\n        claim more: a <= c;\n    }\n}";
    let clamp = findings_with(text, overflow(Clamp));
    assert!(clamp.iter().all(|f| f.contains("unknown")), "{clamp:?}");
    let wrap = findings_with(text, overflow(Wrap));
    assert!(wrap.iter().any(|f| !f.contains("unknown")), "{wrap:?}");
    let fail = findings_with(text, overflow(Fail));
    assert!(fail.iter().any(|f| !f.contains("unknown")), "{fail:?}");
    // Bounds alone: clamped, `a + 1` is at least 1.
    let one = "subject N {\n    fn up(a: u8) -> c: u8 { a + 1 } where {\n        claim some: 1 <= c;\n    }\n}";
    assert_eq!(findings_with(one, overflow(Clamp)), Vec::<String>::new());
    assert_ne!(findings_with(one, overflow(Wrap)), Vec::<String>::new());
}

#[test]
fn wrapping_keeps_a_span_that_goes_around_once() {
    use mitra_lang::simulation::Overflow::Wrap;
    // 255 + 1..=4 wraps to 0..=3 whole: bounds decide the claim.
    let text = "subject N {\n    fn f(b: u8) -> c: u8 { 255 + (b / 64 + 1) } where {\n        claim small: c <= 3;\n    }\n}";
    assert_eq!(findings_with(text, overflow(Wrap)), Vec::<String>::new());
}

#[test]
fn division_rounds_toward_zero_or_down_as_the_model_says() {
    let row = |q: &str, r: &str| {
        format!("subject N {{\n    fn div(a: integer, b: integer) -> q: (integer, integer) {{ (a / b, a % b) }} where {{\n        examples {{ (-7, 2) -> ({q}, {r}) }}\n    }}\n}}")
    };
    assert_eq!(findings(&row("-3", "-1")), Vec::<String>::new());
    assert_eq!(findings_with(&row("-4", "1"), floor()), Vec::<String>::new());
    assert_ne!(findings_with(&row("-3", "-1"), floor()), Vec::<String>::new());
}

#[test]
fn a_floored_remainder_takes_the_sign_of_the_divisor_in_bounds() {
    let text = "subject N {\n    fn m(a: integer) -> r: integer { a % 3 } where {\n        claim kept: 0 <= r && r < 3;\n    }\n}";
    assert_eq!(findings_with(text, floor()), Vec::<String>::new());
    assert_ne!(findings(text), Vec::<String>::new());
}

#[test]
fn machine_division_floors_too() {
    let text = "subject N {\n    fn half(a: i8) -> c: i8 { a / 2 } where {\n        examples { (-7) -> -4 }\n    }\n}";
    assert_eq!(findings_with(text, floor()), Vec::<String>::new());
    assert_ne!(findings(text), Vec::<String>::new());
}

// Narrowing (the user's S3): `u8::narrow(x)` has no value past the
// range, as a division by zero; `Convert::try_u8` in std gives a Result.

#[test]
fn narrowing_past_the_range_gives_no_value() {
    let text = "subject N {\n    fn twice(b: u8) -> c: u8 { u8::narrow(integer::from(b) * 2) } where {\n        examples { (100) -> 200, (200) -> 144 }\n    }\n}";
    let found = findings(text);
    assert_eq!(found.len(), 1, "{found:?}");
    // The reason (400 is past `u8`) is not said yet: report choice N3.
    assert!(found[0].contains("no value at b = 200"), "{found:?}");
}

#[test]
fn narrowing_a_span_inside_the_range_is_decided_by_bounds() {
    let text = "subject N {\n    fn half(b: u8) -> c: u8 { u8::narrow(integer::from(b) / 2) } where {\n        claim small: c <= 127;\n    }\n}";
    assert_eq!(findings(text), Vec::<String>::new());
}

#[test]
fn try_from_in_std_gives_ok_or_the_integer_that_did_not_fit() {
    let text = "subject Port {\n    use Convert from \"std\";\n    use Result from \"std\";\n    fn port(n: integer) -> p: Result<u16, integer> { Convert::try_u16(n) } where {\n        examples { (80) -> Ok(80), (65536) -> Err(65536), (-1) -> Err(-1) }\n    }\n}";
    assert_eq!(findings(text), Vec::<String>::new());
}

#[test]
fn narrow_takes_an_integer_only() {
    let text = "subject N {\n    fn f(b: u16) -> c: u8 { u8::narrow(b) } where { claim t: true; }\n}";
    assert!(!code_of(text).is_empty());
}
