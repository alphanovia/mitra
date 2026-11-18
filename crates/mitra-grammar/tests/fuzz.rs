//! The hand-written parser against the reference grammar.
//!
//! Every golden file and every generated program must be accepted by the
//! LALRPOP grammar, by the LL(1) grammar and by `mitra_core`, and must
//! round-trip through the printer. Seeds are fixed, so runs are repeatable.

use mitra_core::{parse_evidence, parse_model, print_evidence, print_model, Evidence, Model};
use mitra_grammar::{accepts_evidence, accepts_model, gen, ll1::Grammar};
use std::{fs, path::Path, sync::OnceLock};

/// Generated programs per run: seeds 0..SEEDS.
const SEEDS: u64 = 500;

fn model_without_lines(mut m: Model) -> Model {
    m.records.iter_mut().for_each(|x| x.line = 0);
    m.functions.iter_mut().for_each(|x| x.line = 0);
    m.claims.iter_mut().for_each(|x| x.line = 0);
    m.tests.iter_mut().for_each(|x| {
        x.line = 0;
        x.judgment.line = 0;
    });
    m
}

fn evidence_without_lines(mut e: Vec<Evidence>) -> Vec<Evidence> {
    e.iter_mut().for_each(|x| x.line = 0);
    e
}

/// One program, checked every way. Returns (grammar accepts, parser
/// accepts, round trip holds); panics with the reason on the first failure
/// only through the callers' asserts.
#[derive(Debug, PartialEq)]
struct Outcome {
    grammar: Result<(), String>,
    ll1: Result<(), String>,
    parses: Result<(), String>,
    round_trips: Result<(), String>,
}

fn check(model: &str, evidence: &str) -> Outcome {
    static LL1: OnceLock<Grammar> = OnceLock::new();
    let g = LL1.get_or_init(Grammar::load);
    let both = |a: Result<(), String>, b: Result<(), String>| a.and(b);
    let grammar = both(accepts_model(model), accepts_evidence(evidence));
    let ll1 = both(g.accepts("Model", model), g.accepts("Evidences", evidence));
    let parsed = parse_model(model)
        .map_err(|e| format!("{e:?}"))
        .and_then(|m| Ok((m, parse_evidence(evidence).map_err(|e| format!("{e:?}"))?)));
    let parses = parsed.as_ref().map(|_| ()).map_err(|e| e.clone());
    let round_trips = parsed.and_then(|(m, e)| {
        let (m, e) = (model_without_lines(m), evidence_without_lines(e));
        let m2 = parse_model(&print_model(&m)).map_err(|x| format!("reprint: {x:?}"))?;
        let e2 =
            parse_evidence(&print_evidence(&e, None)).map_err(|x| format!("reprint: {x:?}"))?;
        if model_without_lines(m2) != m || evidence_without_lines(e2) != e {
            return Err("parse → print → parse changed the AST".into());
        }
        Ok(())
    });
    Outcome {
        grammar,
        ll1,
        parses,
        round_trips,
    }
}

fn all_ok(o: &Outcome) -> bool {
    o.grammar.is_ok() && o.ll1.is_ok() && o.parses.is_ok() && o.round_trips.is_ok()
}

#[test]
fn golden_files_are_in_the_grammar() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden");
    let mut stack = vec![root];
    let mut n = 0;
    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let src = fs::read_to_string(&path).unwrap();
            let o = match path.file_name().unwrap().to_str().unwrap() {
                "model.mitra" => check(&src, ""),
                f if f.ends_with(".mitra") => check("", &src),
                _ => continue,
            };
            assert!(all_ok(&o), "{}: {o:?}", path.display());
            n += 1;
        }
    }
    assert!(n > 20);
}

/// Runs the generated programs and returns them as Mitra evidence. A
/// failure becomes `= false` evidence, so the dogfood claims can fail.
fn fuzz() -> String {
    let mut evidence = String::new();
    for seed in 0..SEEDS {
        let mut r = gen::Rng::new(seed);
        let (model, ev) = (gen::model(&mut r), gen::evidences(&mut r));
        let o = check(&model, &ev);
        if !all_ok(&o) {
            eprintln!("seed {seed}: {o:?}\n--- model\n{model}--- evidence\n{ev}");
        }
        for (subject, ok) in [
            ("grammar_accepts", o.grammar.is_ok()),
            ("ll1_accepts", o.ll1.is_ok()),
            ("parses", o.parses.is_ok()),
            ("round_trips", o.round_trips.is_ok()),
        ] {
            evidence += &format!("evidence {subject}({seed}) = {ok} at 1\n");
        }
    }
    evidence
}

/// Dogfood: the fuzz results are evidence for `mitra/parser.mitra`.
#[test]
fn parser_matches_grammar() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("mitra");
    let model = fs::read_to_string(dir.join("parser.mitra")).unwrap();
    let expected = fs::read_to_string(dir.join("expected.txt")).unwrap();
    let report = mitra_core::run(&model, &fuzz()).unwrap();
    assert_eq!(report.to_text(), expected);
}
