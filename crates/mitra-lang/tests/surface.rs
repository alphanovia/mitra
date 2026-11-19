//! The language: each folder of `examples/reworked/` holds
//! `model.mitra`, `evidence.mitra` and the report Mitra must give
//! (`MITRA_RECORD=1 cargo test` writes it again).

use std::{fs, path::Path};

#[test]
fn examples() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/reworked");
    let mut seen = 0;
    for entry in fs::read_dir(root).unwrap() {
        let dir = entry.unwrap().path();
        let read = |f: &str| fs::read_to_string(dir.join(f)).unwrap();
        // The test does the work of mitra-store: it reads the modules the
        // model imports and hands them over.
        let model = read("model.mitra");
        let mut handed = mitra_lang::Handed::default();
        for name in mitra_lang::parse::imports(&model) {
            let file = format!("{}.mitra", name.replace("::", "/"));
            handed.modules.insert(name, read(&file));
        }
        let name = "model.mitra".to_string();
        let files = vec![mitra_lang::Module { name, text: model }];
        let model = mitra_lang::Modules::new(files, handed);
        let report = mitra_lang::load_modules(&model, &read("evidence.mitra"))
            .and_then(|m| mitra_lang::evaluate(&m))
            .unwrap_or_else(|(_, e)| panic!("{}: {e}", dir.display()));
        if std::env::var("MITRA_RECORD").is_ok() {
            fs::write(dir.join("expected.txt"), report.to_text()).unwrap();
        }
        assert_eq!(report.to_text(), read("expected.txt"), "{}", dir.display());
        seen += 1;
    }
    assert!(seen >= 3);
}

fn verdicts(model: &str, evidence: &str) -> String {
    mitra_lang::run(model, evidence)
        .unwrap_or_else(|(_, e)| panic!("{e}"))
        .to_text()
}

fn refused(model: &str, evidence: &str) -> String {
    match mitra_lang::run(model, evidence) {
        Err((_, e)) => e.to_string(),
        Ok(r) => panic!("accepted: {}", r.to_text()),
    }
}

const HALF: &str = "subject math {
    fn half(n: 0..=5) -> 0..=2;
    claim even_exact(n: 0..=5) when n == 0 || n == 2 || n == 4 { half(n) + half(n) == n }
}";

#[test]
fn when_on_inputs_takes_them_out_of_the_claim() {
    let e = "evidence a { for half given n = 0 gives 0 }
evidence b { for half given n = 2 gives 1 }";
    let out = verdicts(HALF, e);
    assert!(out.contains("no evidence on  `n = 4`"), "{out}");
    let e = format!("{e}\nevidence c {{ for math::half given n = 4 gives 2 }}");
    assert_eq!(
        verdicts(HALF, &e)
            .lines()
            .find(|l| l.contains("even_exact")),
        Some("math::even_exact: holds")
    );
}

#[test]
fn evidence_names_what_it_is_for() {
    let e = "evidence a { for half given m = 0 gives 0 }";
    assert_eq!(refused(HALF, e), "line 1: `math::half` has no input `m`");
    let e = "evidence a { for third given n = 0 gives 0 }";
    // A piece the model does not have: set aside and shown, not refused.
    let out = verdicts(HALF, e);
    assert!(out.contains("warning[M107]"), "{out}");
    assert!(
        out.contains("The model has no `third`. It judges nothing."),
        "{out}"
    );
    // New evidence is refused on it: only what is in a ledger is kept.
    let said = mitra_lang::read_evidence(&mitra_lang::Modules::alone(HALF), e).unwrap_err();
    assert_eq!(
        said.to_string(),
        "line 1: the model has no function or claim `third`"
    );
    let e = "evidence a { for half given n = 0 }";
    assert!(refused(HALF, e).contains("says nothing"));
}

#[test]
fn evidence_is_written_in_blocks_only() {
    let e = "// a comment
evidence a { for half given n = 0 gives 0 source \"x\" }
evidence b { for half given n in {2, 4} holds }
evidence c { for even_exact given n = 2 fails trust 0.5 }";
    let out = verdicts(HALF, e);
    assert!(out.contains("\nmath::even_exact: fails"), "{out}");
    let e = refused(HALF, "evidence math::half(0) = 0 at 1");
    assert!(e.contains("line 1: expected an `evidence` block"), "{e}");
    let e = refused("opaque f(a: Bool) -> Bool", "");
    assert!(e.contains("line 1: this syntax is no longer read"), "{e}");
}

#[test]
fn a_body_in_a_subject_is_a_spec_and_a_claim() {
    let m = "subject s { fn double(n: 0..=2) -> 0..=4 { n + n } }";
    let e = "evidence a { for double given n = 2 gives 3 }";
    let out = verdicts(m, e);
    assert!(out.starts_with("s::double::spec: fails"), "{out}");
}

#[test]
fn assume_is_taken_as_true_until_evidence_says_otherwise() {
    let m = "boundary clock { fn now() -> 0..=9; assume starts_at_zero() { now() == 0 } }";
    let first = |e: &str| verdicts(m, e).lines().next().map(String::from);
    // Nothing but its own word: said so, and no warning for it.
    assert_eq!(
        first("").as_deref(),
        Some("clock::starts_at_zero: holds assuming [itself]")
    );
    assert!(!verdicts(m, "").contains("warning"), "{}", verdicts(m, ""));
    // Observed: it holds.
    let seen = "evidence seen { for now gives 0 }";
    assert_eq!(first(seen).as_deref(), Some("clock::starts_at_zero: holds"));
    let out = verdicts(m, "evidence late { for now gives 3 }");
    assert!(out.contains("error[M"), "{out}");
}

#[test]
fn a_boundary_may_answer_differently_each_time() {
    let m = "boundary dice { fn roll() -> 1..=6; }\n\
             subject game { claim small() { dice::roll() <= 3 } }";
    let two = "evidence a { for dice::roll gives 1 }\nevidence b { for dice::roll gives 2 }";
    let out = verdicts(m, two);
    assert!(out.starts_with("game::small: holds"), "{out}");
    assert!(!out.contains("M003"), "{out}");
    // True of one answer and not of another: the claim is about each.
    let out = verdicts(
        m,
        &format!("{two}\nevidence c {{ for dice::roll gives 5 }}"),
    );
    assert!(out.starts_with("game::small: fails"), "{out}");
    assert!(!out.contains("the code of `dice::roll`"), "{out}");
    // A subject is claimed deterministic: two outputs on one input
    // refute that, unless the model says it is not.
    let m = m.replace("boundary", "subject");
    let out = verdicts(&m, two);
    assert!(out.contains("dice::roll::deterministic: fails"), "{out}");
    assert!(out.contains("the code of `dice::roll`"), "{out}");
    let free = m.replace(
        "fn roll() -> 1..=6;",
        "fn roll() -> 1..=6; claim free = !deterministic(roll);",
    );
    let out = verdicts(&free, two);
    assert!(
        out.contains("dice::free: holds") && !out.contains("error["),
        "{out}"
    );
    let named = m.replace(
        "fn roll() -> 1..=6;",
        "fn roll() -> 1..=6; claim same = deterministic(roll) trust 0.9;",
    );
    let out = verdicts(&named, "evidence r { for dice::same holds }");
    assert!(
        out.contains("dice::same: holds") && !out.contains("roll::deterministic"),
        "{out}"
    );
}

#[test]
fn a_claim_with_parameters_is_written_once() {
    let m = "claim commutes(f, T) { given a: T, b: T then f(a, b) == f(b, a) }\n\
             subject s { fn add(a: 0..=1, b: 0..=1) -> 0..=2;\n\
             claim add_commutes = commutes(add, 0..=1); }";
    let model = mitra_lang::load(m, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    // With the claim every function of a subject has: its code is correct.
    let names: Vec<&str> = model.claims.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "s::add::correct",
            "s::add::deterministic",
            "s::add_commutes"
        ]
    );
    let bad = m.replace("commutes(add, 0..=1)", "commutes(add)");
    let e = mitra_lang::load(&bad, "").unwrap_err().1.message;
    assert!(e.contains("takes 2 parameter(s), got 1"), "{e}");
    let bad = m.replace("= commutes(add", "= swaps(add");
    let e = mitra_lang::load(&bad, "").unwrap_err().1.message;
    assert!(e.contains("no claim with parameters `swaps`"), "{e}");
}

#[test]
fn an_inline_function_and_the_elements_that_fit() {
    let m = "subject s { claim c() { (|a, b| a - b)(3, 1) == 2 && (|a, b| a - b)(1, 3) == -2 }\n\
             claim d(a: 0..=3, b: 0..=3) { (|a, b| a - b)(b, a) == b - a }\n\
             claim e() { { x in {1, 2, 3, 4} | x > 2 } == {3, 4} } }";
    let out = verdicts(m, "");
    assert!(
        out.starts_with("s::c: holds\ns::d: holds\ns::e: holds"),
        "{out}"
    );
}

#[test]
fn a_test_block_is_refused() {
    let m = format!("{HALF}\ntest t {{ for half given n = 4 gives 2 }}");
    let e = mitra_lang::load(&m, "").unwrap_err().1.message;
    assert!(e.contains("a `test` block is not read any more"), "{e}");
}

/// Evidence on a point claim is an output of the code it calls.
#[test]
fn a_point_claim_carries_its_evidence_to_the_code() {
    let m = "subject s { fn half(n: 0..=9) -> 0..=4 { n / 2 }\n\
             claim half_right() { given n: 0..=9 then correct(half(n)) }\n\
             claim four() { half(4) == 2 } }";
    let out = verdicts(m, "evidence t { for four holds }");
    assert!(out.contains("s::four: holds"), "{out}");
    assert!(
        out.contains("s::half_right: unverified\n  passes on       `s::half(4)`"),
        "{out}"
    );
}

#[test]
fn import_takes_what_it_names() {
    let m = "import std::claims::{commutative};\n\
             subject s { fn add(a: 0..=1, b: 0..=1) -> 0..=2;\n\
             claim add_commutes = commutative(add, 0..=1); }";
    let model = mitra_lang::load(m, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    assert_eq!(model.claims.len(), 3);
    let error = |m: &str| mitra_lang::load(m, "").unwrap_err().1.message;
    // Not named, not there.
    let e = error(&m.replace("{commutative}", "{neutral}"));
    assert!(e.contains("no claim with parameters `commutative`"), "{e}");
    let e = error("import std::claims::{nothing};\nsubject s { }");
    assert!(e.contains("`std::claims` declares no `nothing`"), "{e}");
    let e = error("import std::nowhere::x;\nsubject s { }");
    assert!(e.contains("no module `std::nowhere` in `std`"), "{e}");
}

#[test]
fn represented_says_nothing_outside_its_set() {
    let m = "import std::claims::{represented};\n\
             subject s { fn half(n: 0..=9) -> 0..=4 { n / 2 }\n\
             claim low = represented(half, 0..=4);\n\
             claim high = represented(half, { n | n > 4 }); }";
    // One tested input stands for its set only, and the claim that says
    // so is an assumption until someone vouches for it.
    let out = verdicts(m, "evidence a { for half given n = 2 gives 1 }");
    assert!(
        out.contains("assumed on      n in 0..=1 | 3..=4, by `s::low`"),
        "{out}"
    );
    assert!(out.contains("no evidence on  n in 5..=9"), "{out}");
    let both = "evidence a { for half given n = 2 gives 1 }\n\
                evidence b { for half given n = 8 gives 4 }";
    let out = verdicts(m, both);
    assert!(
        out.starts_with("s::half::spec: holds assuming [s::low, s::high]"),
        "{out}"
    );
    let vouched = format!(
        "{both}\nevidence c {{ for low holds source \"review:me\" }}\n\
         evidence d {{ for high holds source \"review:me\" }}"
    );
    assert!(verdicts(m, &vouched).starts_with("s::half::spec: holds\n"));
    // Two inputs of one set that do not behave the same deny the claim.
    let out = verdicts(
        m,
        &format!("{vouched}\nevidence e {{ for half given n = 6 gives 0 }}"),
    );
    assert!(out.contains("s::high: fails"), "{out}");
    assert!(out.contains("error[M002]"), "{out}");
    assert!(out.contains("do not behave the same"), "{out}");
    // Not imported, not there.
    let e = mitra_lang::load(&m.replace("import std::claims::{represented};", ""), "");
    assert!(e
        .unwrap_err()
        .1
        .message
        .contains("no claim with parameters `represented`"));
}

#[test]
fn correct_is_a_value_with_three_states() {
    let m = "subject s { fn add(a: 0..=2, b: 0..=2) -> 0..=4;\n\
             claim add_ok(a: 0..=2, b: 0..=2) { correct(add(a, b)) }\n\
             claim mirror(a: 0..=2, b: 0..=2) { correct(add(a, b)) <=> correct(add(b, a)) } }";
    let seen = "evidence t { for s::add given a = 0, b = 1 holds }";
    // No value where nothing judges the call: the claim is not judged there.
    let out = verdicts(m, seen);
    assert!(
        out.contains("s::mirror: unverified\n  no evidence on"),
        "{out}"
    );
    // Vouched for, it carries the judgement to the call nothing judges.
    let vouched = format!("{seen}\nevidence r {{ for s::mirror holds source \"review:me\" }}");
    let out = verdicts(m, &vouched);
    assert!(
        out.contains("assumed on      `s::add(1, 0)`, by `s::mirror`"),
        "{out}"
    );
    // Both judged, and differently: the claim is false there.
    let out = verdicts(
        m,
        &format!("{seen}\nevidence u {{ for s::add given a = 1, b = 0 fails }}"),
    );
    assert!(out.contains("s::mirror: fails"), "{out}");
}

#[test]
fn the_shape_of_a_set_is_read_whatever_its_name() {
    // The text of `represented`, under another name and with `==`.
    let m = "claim alike(f, S) { given p: S, q: S then correct(f(p)) == correct(f(q)) }\n\
             subject s { fn half(n: Int) -> Int { n / 2 }\n\
             claim low = alike(half, 0..=4); }";
    let out = verdicts(m, "evidence a { for s::half given n = 2 gives 1 }");
    assert!(
        out.contains("assumed on      n in 0..=1 | 3..=4, by `s::low`"),
        "{out}"
    );
    // `uniform` is library text too: not there until imported.
    let e = mitra_lang::load(
        "subject s { fn f(n: Int) -> Int;\nclaim u = uniform(f); }",
        "",
    );
    assert!(e.unwrap_err().1.message.contains("or import it"));
    let m = "import std::claims::{uniform};\nsubject s { fn f(n: Int) -> Int;\n\
             claim u = uniform(f);\nclaim v = uniform(f, f, f); }";
    let e = mitra_lang::load(m, "").unwrap_err().1.message;
    assert!(
        e.contains("`uniform` takes 1 or 2 parameter(s), got 3"),
        "{e}"
    );
}

/// An assumption and a `uses` clause are suspects of their own kind.
#[test]
fn assumptions_and_uses_clauses_are_suspects() {
    use mitra_lang::Piece;
    let m = "boundary clock { fn now() -> 0..=9;\nassume noon() trust 0.7 { now() == 5 } }\n\
             subject timer { fn left() -> 0..=9 uses clock::now;\n\
             claim left_fits() { left() + clock::now() <= 9 } }";
    let pieces = |evidence: &str| -> Vec<Piece> {
        let report = mitra_lang::run(m, evidence).unwrap_or_else(|(_, e)| panic!("{e}"));
        let all = report
            .problems
            .iter()
            .flat_map(|p| p.candidates.clone().unwrap());
        all.map(|c| c.piece).collect()
    };
    // A claim fails on what the assumption gave: the assumption is a
    // candidate, with the code.
    let got = pieces("evidence l { for timer::left gives 6 }");
    assert!(
        got.contains(&Piece::Assumption("clock::noon".into())),
        "{got:?}"
    );
    assert!(got.contains(&Piece::Code("timer::left".into())), "{got:?}");
    // An observation against the assumption.
    let got = pieces("evidence n { for clock::now gives 7 }");
    assert!(
        got.contains(&Piece::Assumption("clock::noon".into())),
        "{got:?}"
    );
    // The reader denies the `uses` clause.
    let got = pieces("evidence u { for timer::left::uses fails }");
    assert!(got.contains(&Piece::Uses("timer::left".into())), "{got:?}");
    assert!(
        !got.iter()
            .any(|p| matches!(p, Piece::Evidence(n) if *n > 1)),
        "{got:?}"
    );
}

/// The code of a subject is claimed correct without a line for it.
#[test]
fn a_function_of_a_subject_is_claimed_correct() {
    let names = |m: &str| -> Vec<String> {
        let model = mitra_lang::load(m, "").unwrap_or_else(|(_, e)| panic!("{e}"));
        model.claims.iter().map(|c| c.name.clone()).collect()
    };
    // No body: the implied claim. A body: the spec says what correct is.
    let m = "subject s { fn f(a: 0..=1) -> Int;\nfn g(a: 0..=1) -> Int { a } }";
    assert_eq!(
        names(m),
        [
            "s::f::correct",
            "s::f::deterministic",
            "s::g::spec",
            "s::g::deterministic"
        ]
    );
    // Not on a boundary. With one word or an unnamed block, named as written.
    let m = "boundary b { fn f(a: 0..=1) -> Int; }\nboundary fn now() -> Int;\n\
             subject fn h(a: 0..=1) -> Int;\nsubject { fn k() -> Int; }";
    assert_eq!(
        names(m),
        [
            "h::correct",
            "h::deterministic",
            "k::correct",
            "k::deterministic"
        ]
    );
    // The model's own claim replaces it: nothing is claimed on 0.
    let m = "subject s { fn div(a: 0..=2, b: 0..=2) -> Int;\n\
             claim div_ok(a, b: 1..=2) { correct(div(a, b)) } }";
    assert_eq!(names(m), ["s::div::deterministic", "s::div_ok"]);
    // A point pins one call: it narrows nothing.
    let m = "subject s { fn f(a: 0..=1) -> Int;\nclaim t() { correct(f(1)) } }";
    assert_eq!(names(m), ["s::f::correct", "s::f::deterministic", "s::t"]);
    // Evidence on the function is evidence for it.
    let out = verdicts(m, "evidence e { for s::f holds }");
    assert!(out.starts_with("s::f::correct: holds\n"), "{out}");
}

/// What an unnamed subject or boundary holds is imported by its name.
#[test]
fn import_reaches_into_unnamed_blocks() {
    let lib = "subject fn div(a: 0..=4, b: 1..=2) -> 0..=4;\n\
               subject { fn half(x: 0..=4) -> 0..=2;\nfn other() -> Int;\n\
               claim half_small(x) { half(x) <= 2 } }\n\
               boundary { fn now() -> Int; }";
    let mut handed = mitra_lang::Handed::default();
    handed.modules.insert("lib".into(), lib.into());
    let m = "import lib::{div, half, half_small, now};\n\
             claim t() { div(4, 2) == half(4) + now() }";
    let name = "m".to_string();
    let text = m.to_string();
    let files = mitra_lang::Modules::new(vec![mitra_lang::Module { name, text }], handed);
    let model = mitra_lang::load_modules(&files, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    let names: Vec<&str> = model.claims.iter().map(|c| c.name.as_str()).collect();
    assert_eq!(
        names,
        [
            "half_small",
            "div::correct",
            "half::correct",
            "div::deterministic",
            "half::deterministic",
            "t"
        ]
    );
    assert_eq!(model.boundaries, ["now"]);
}

/// A requirement in prose holds as far as the claims of its subject,
/// the word that they say it, and what it requires.
#[test]
fn a_requirement_in_prose_rests_on_its_claims_and_a_vouch() {
    use mitra_lang::Piece;
    let m = "boundary mailer { ensure delivers = \"Mail arrives.\";\n\
             fn send(to: 0..=1) -> Bool; }\n\
             subject signup { ensure confirms = \"A new user gets a mail.\";\n\
             require delivery = mailer::delivers;\n\
             fn register(u: 0..=1) -> Bool uses mailer::send;\n\
             claim sent(u) when register(u) { mailer::send(u) } }";
    let run = |evidence: &str| mitra_lang::run(m, evidence).unwrap_or_else(|(_, e)| panic!("{e}"));
    let seen = "evidence a { for signup::register given u = 0 gives true }\n\
                evidence b { for signup::register given u = 1 gives false }\n\
                evidence c { for mailer::send given to = 0 gives true }\n\
                evidence r { for signup::register holds }\n";
    // Nothing known: it holds only assuming all it rests on.
    let text = run("").to_text();
    assert!(
        text.contains("mailer::delivers: holds assuming [itself]"),
        "{text}"
    );
    assert!(
        text.contains("signup::confirms::captured: unverified"),
        "{text}"
    );
    assert!(
        text.contains("signup::confirms: holds assuming ["),
        "{text}"
    );
    // The claims hold, nobody vouched.
    let text = run(seen).to_text();
    assert!(
        text.contains(
            "signup::confirms: holds assuming [mailer::delivers, signup::confirms::captured]"
        ),
        "{text}"
    );
    // Vouched: only the boundary's promise is left assumed.
    let vouched =
        format!("{seen}evidence v {{ for signup::confirms::captured holds trust 0.8 }}\n");
    let text = run(&vouched).to_text();
    assert!(
        text.contains("signup::confirms: holds assuming [mailer::delivers]"),
        "{text}"
    );
    // A ticket against the requirement: the text, the vouch, what it
    // requires and the ticket are the candidates, never the code.
    let report = run(&format!(
        "{vouched}evidence bug {{ for signup::confirms fails trust 0.9 }}\n"
    ));
    assert!(report.to_text().contains("signup::confirms: fails"));
    let got: Vec<Piece> = report
        .problems
        .iter()
        .flat_map(|p| p.candidates.clone().unwrap_or_default())
        .map(|c| c.piece)
        .collect();
    for want in [
        Piece::Requirement("signup::confirms".into()),
        Piece::Capture("signup::confirms".into()),
        Piece::Requirement("signup::delivery".into()),
        Piece::Requirement("mailer::delivers".into()),
    ] {
        assert!(got.contains(&want), "{want:?} not in {got:?}");
    }
    assert!(!got.iter().any(|p| matches!(p, Piece::Code(_))), "{got:?}");
    // A claim of the subject fails: so does the requirement.
    let text = run(
        "evidence a { for signup::register given u = 0 gives true }\n\
                    evidence c { for mailer::send given to = 0 gives false }\n",
    )
    .to_text();
    assert!(text.contains("signup::confirms: fails"), "{text}");
    // The need has a name of its own, and says what its promise says.
    let text = run("").to_text();
    assert!(
        text.contains("signup::delivery: holds assuming [mailer::delivers]"),
        "{text}"
    );
}

/// A subject with a requirement and no claim says nothing of it; a
/// `require` names an `ensure`.
#[test]
fn a_requirement_with_no_claim_is_unverified() {
    let text = mitra_lang::run("subject s { ensure works = \"It works.\"; }", "")
        .unwrap_or_else(|(_, e)| panic!("{e}"))
        .to_text();
    assert!(text.contains("s::works: unverified"), "{text}");
    let refused = |m: &str| mitra_lang::run(m, "").err().expect("refused").1.to_string();
    let e = refused("subject s { require n = t::nothing; fn f() -> Bool; }");
    assert!(e.contains("no `ensure` is named"), "{e}");
    // The forms without `=` are refused, with the form to write.
    let e = refused("subject s { ensure works \"It works.\"; }");
    assert!(e.contains("ensure works = "), "{e}");
    let e = refused("subject s { require t::x; }");
    assert!(e.contains("require <name> = t::x;"), "{e}");
    refused("ensure works = \"It works.\";");
    // A need with nobody to promise it yet: assumed.
    let text = mitra_lang::run(
        "subject s { ensure works = \"It works.\";\nrequire power = \"There is power.\"; }",
        "",
    )
    .unwrap_or_else(|(_, e)| panic!("{e}"))
    .to_text();
    assert!(text.contains("s::power: holds assuming [itself]"), "{text}");
    // A word straight on a requirement does not stand in for its claims
    // and its capture.
    let text = mitra_lang::run(
        "subject s { ensure works = \"It works.\";\nfn f() -> Bool; }",
        "evidence v { for s::works holds }",
    )
    .unwrap_or_else(|(_, e)| panic!("{e}"))
    .to_text();
    assert!(
        text.contains("s::works: holds assuming [s::f::correct, s::works::captured]"),
        "{text}"
    );
    // Evidence against an assumed one names it, not what holds it.
    let text = mitra_lang::run(
        "subject s { ensure works = \"It works.\";\nrequire power = \"There is power.\"; }",
        "evidence x { for s::power fails trust 0.8 }",
    )
    .unwrap_or_else(|(_, e)| panic!("{e}"))
    .to_text();
    assert!(text.contains("On `s::power`, these two"), "{text}");
    assert!(!text.contains("::prose"), "{text}");
}

/// A promise of the world rests on the assumptions of its boundary, with
/// a capture of its own; `trust n` is read on `ensure` and `require`.
#[test]
fn a_promise_of_the_world_rests_on_its_assumptions() {
    use mitra_lang::Piece;
    let m = "boundary mailer { ensure delivers = \"Mail arrives.\" trust 0.9;\n\
             fn send(to: 0..=1) -> Bool;\n\
             assume up() trust 0.7 { rate(t in 0..=1, send(t)) >= 0.5 } }\n\
             subject signup { ensure confirms = \"A new user gets a mail.\";\n\
             require delivery = mailer::delivers trust 0.8;\n\
             fn register(u: 0..=1) -> Bool uses mailer::send; }";
    let run = |evidence: &str| mitra_lang::run(m, evidence).unwrap_or_else(|(_, e)| panic!("{e}"));
    let text = run("").to_text();
    let want = "mailer::delivers: holds assuming [mailer::up, mailer::delivers::captured]";
    assert!(text.contains(want), "{text}");
    let pieces = |evidence: &str| -> Vec<Piece> {
        let report = run(evidence);
        let all = report.problems.iter();
        all.flat_map(|p| p.candidates.clone().unwrap_or_default())
            .map(|c| c.piece)
            .collect()
    };
    // Evidence against the promise: its text, its capture, the assumption.
    let got = pieces("evidence t { for mailer::delivers fails trust 0.9 }");
    for want in [
        Piece::Requirement("mailer::delivers".into()),
        Piece::Capture("mailer::delivers".into()),
        Piece::Assumption("mailer::up".into()),
    ] {
        assert!(got.contains(&want), "{want:?} not in {got:?}");
    }
    // The text and the link of a named need carry the trust they say.
    let text = run("evidence t { for signup::confirms fails trust 0.9 }").to_text();
    let line = |part: &str| text.lines().find(|l| l.contains(part)).unwrap().to_string();
    assert!(
        line("is what `signup::delivery` needs").contains(" 0.8 "),
        "{text}"
    );
    assert!(
        line("the text of `mailer::delivers`").contains(" 0.9 "),
        "{text}"
    );
    assert!(
        line("the text of `signup::confirms`").contains(" none "),
        "{text}"
    );
}

/// What a block requires and what its functions use are compared.
#[test]
fn require_and_uses_are_compared() {
    let world =
        "boundary mailer { ensure delivers = \"Mail arrives.\";\nfn send(to: 0..=1) -> Bool; }\n\
                 boundary clock { ensure ticks = \"Time moves.\";\nfn now() -> 0..=9; }\n";
    let codes = |subject: &str| -> Vec<&'static str> {
        let m = format!("{world}{subject}");
        let report = mitra_lang::run(&m, "").unwrap_or_else(|(_, e)| panic!("{e}"));
        report.model.lints.iter().map(|l| l.code).collect()
    };
    // A need nothing uses, and a use nothing requires.
    let got = codes(
        "subject s { ensure works = \"It works.\";\nrequire time = clock::ticks;\n\
         fn f(u: 0..=1) -> Bool uses mailer::send; }",
    );
    assert_eq!(got, ["M105", "M106"]);
    // The two agree.
    let got = codes(
        "subject s { ensure works = \"It works.\";\nrequire mail = mailer::delivers;\n\
         fn f(u: 0..=1) -> Bool uses mailer::send; }",
    );
    assert!(got.is_empty(), "{got:?}");
    // A block that states no intent is not held to it; a need in prose
    // has nobody to use.
    let got = codes("subject s { fn f(u: 0..=1) -> Bool uses mailer::send; }");
    assert!(got.is_empty(), "{got:?}");
    let got = codes("subject s { require power = \"There is power.\";\nfn f(u: 0..=1) -> Bool; }");
    assert!(got.is_empty(), "{got:?}");
    // A need is justified by anything that leans on the block: a call
    // through another subject, a call in a claim, one of its types.
    let through = "subject mid { fn relay(u: 0..=1) -> Bool uses mailer::send; }\n\
                   subject s { require mail = mailer::delivers;\n\
                   fn f(u: 0..=1) -> Bool uses mid::relay; }";
    assert!(codes(through).is_empty());
    let called = "subject s { require mail = mailer::delivers;\n\
                  fn f(u: 0..=1) -> Bool;\n\
                  claim c(u: 0..=1) { f(u) == mailer::send(u) } }";
    assert!(codes(called).is_empty());
    let typed = "subject data { ensure kept = \"Kept.\"; record Row { id: 0..=1 } }\n\
                 subject s { require rows = data::kept; fn f(r: data::Row) -> Bool; }";
    assert!(codes(typed).is_empty());
    let apart = "subject data { ensure kept = \"Kept.\"; record Row { id: 0..=1 } }\n\
                 subject s { require rows = data::kept; fn f(u: 0..=1) -> Bool; }";
    assert_eq!(codes(apart), ["M105"]);
    assert!(mitra_lang::explain("M105").is_some() && mitra_lang::explain("M106").is_some());
}

/// A record may leave its contents out: `record R;`, or `..` after the
/// fields that matter. No value of it can be written.
#[test]
fn a_record_may_leave_parts_unstated() {
    let m = "record Model;\nrecord Claim { name: String, .. }\n\
             subject check { fn ok(m: Model) -> Bool;\n\
             fn named(c: Claim) -> String { c.name } }";
    let run = |e: &str| mitra_lang::run(m, e).unwrap_or_else(|(_, e)| panic!("{e}"));
    let text = run("evidence r { for check::ok holds }").to_text();
    assert!(text.contains("check::ok::correct: holds"), "{text}");
    assert!(text.contains("check::named::spec: unverified"), "{text}");
    let refused = |m: &str, e: &str| mitra_lang::run(m, e).err().expect("refused").1.to_string();
    let e = refused(m, "evidence r { for check::ok given m = Model() holds }");
    assert!(e.contains("no value of it can be written"), "{e}");
    let e = refused(
        "record M { a: Int, .. }\nsubject s { fn f() -> M { M(1) } }",
        "",
    );
    assert!(e.contains("no value of it can be written"), "{e}");
    // The tie of a record is a claim once evidence names it.
    let text = run("evidence t { for Claim::tied fails }").to_text();
    assert!(text.contains("Claim::tied: fails"), "{text}");
    assert!(
        text.contains("The record `Claim` and its struct do not agree"),
        "{text}"
    );
}

/// A record or an enum written in a subject is named by it: short inside,
/// `subject::Name` elsewhere, or short where one subject has the name.
#[test]
fn a_record_belongs_to_its_subject() {
    let m = "enum Piece { Claim(name: String), Code }\n\
             subject model { record Claim { name: String, line: 0..=3 }\n\
             enum Kind { Point, Wide(n: 0..=2) }\n\
             fn first() -> Claim;\n\
             fn kind(c: Claim) -> Kind;\n\
             claim line_from_one(c: Claim) trust 0.8 { c.line >= 1 } }\n\
             subject other { fn pass(c: model::Claim) -> Kind;\n\
             claim seen() { pass(model::Claim(\"a\", 1)) == Wide(1) }\n\
             fn piece(p: Piece) -> Bool { p is Claim } }";
    let run = |e: &str| mitra_lang::run(m, e).unwrap_or_else(|(_, e)| panic!("{e}"));
    // Short names in evidence are those of the function's subject.
    let text = run(
        "evidence a { for model::kind given c = Claim(\"a\", 0) gives Point }\n\
                    evidence b { for other::pass given c = model::Claim(\"a\", 1) gives Wide(1) }",
    )
    .to_text();
    assert!(text.contains("other::seen: holds"), "{text}");
    // A claim on a record is true of every value the code builds: an
    // output that fits it is evidence for it, one that breaks it against.
    let text = run("evidence a { for model::first gives Claim(\"a\", 2) }").to_text();
    assert!(
        text.contains("model::line_from_one: unverified\n  passes on"),
        "{text}"
    );
    let report = run("evidence a { for model::first gives Claim(\"a\", 0) trust 0.9 }");
    let text = report.to_text();
    assert!(text.contains("model::line_from_one: fails"), "{text}");
    let all = report.problems.iter();
    let got: Vec<mitra_lang::Piece> = all
        .flat_map(|p| p.candidates.clone().unwrap_or_default())
        .map(|c| c.piece)
        .collect();
    use mitra_lang::Piece::{Claim, Code};
    assert!(got.contains(&Code("model::first".into())), "{got:?}");
    assert!(
        got.contains(&Claim("model::line_from_one".into())),
        "{got:?}"
    );
}

/// `impl R { … }` is Rust's: `self` is an `R`, `Self` the record, a
/// function is `R::name`, called as a method or by its path.
#[test]
fn an_impl_is_read_as_rust_writes_it() {
    let m = "subject geo { record Point { x: 0..=2, y: 0..=2 }\n\
             impl Point { fn new(x: 0..=2, y: 0..=2) -> Self;\n\
             fn flip(&self) -> Self { Point(self.y, self.x) }\n\
             fn sum(&mut self) -> Int;\n\
             fn meets(&self, o: &Point) -> Bool;\n\
             claim flat(self) { self.x == self.y }\n\
             claim back(self) { self.flip().flip() == Self::flip(Point::flip(self)) } }\n\
             claim sums(p: Point) { p.sum() == Point::sum(p.flip()) } }\n\
             subject other { claim far(p: geo::Point) { p.sum() == geo::Point::new(p.x, p.y).sum() } }";
    let run = |e: &str| mitra_lang::run(m, e).unwrap_or_else(|(_, e)| panic!("{e}"));
    let text = run(
        "evidence a { for Point::new given x = 0, y = 1 gives Point(0, 1) }\n\
                    evidence b { for geo::Point::sum given self = Point(1, 2) gives 3 }\n\
                    evidence c { for geo::Point::sum given self = Point(2, 1) gives 3 }",
    )
    .to_text();
    assert!(
        text.contains("geo::Point::new::correct: unverified"),
        "{text}"
    );
    assert!(
        text.contains("geo::Point::back: holds assuming [geo::Point::flip::spec]"),
        "{text}"
    );
    assert!(
        text.contains("geo::Point::flat: fails\n  fails on        `self = geo::Point(0, 1)`"),
        "{text}"
    );
    assert!(text.contains("geo::sums: unverified"), "{text}");
    let refused = |model: &str, want: &str| {
        let e = mitra_lang::run(model, "").map(|_| ()).unwrap_err().1;
        assert!(e.to_string().contains(want), "{e}");
    };
    refused(
        "record A { x: Int } claim c(a: A) { a.size() == 1 }",
        "no method `size`",
    );
    // The method is that of the type of the expression, whatever it is:
    // an input, a call, a field, a `let`, a value written out.
    let two = "subject s { record A { x: 0..=1 } record B { a: A }\n\
               impl A { fn size(self) -> Int; fn next(self) -> Self; }\n\
               impl B { fn size(self) -> Int; fn first(self) -> A; }\n\
               fn pick(b: B) -> A;\n";
    let reads = |claim: &str| {
        let m = format!("{two}{claim} }}");
        mitra_lang::load(&m, "").unwrap_or_else(|(_, e)| panic!("{claim}: {e}"))
    };
    reads("claim c(b: B) { pick(b).size() == b.a.size() }");
    reads("claim c(b: B) { b.first().next().size() == A(1).size() }");
    reads("claim c(b: B) { let a = b.first(); a.size() == b.size() }");
    let m = reads("claim c(b: B) { pick(b).size() == 1 }");
    let c = m.claims.iter().find(|c| c.name == "s::c").unwrap();
    assert!(format!("{:?}", c.target).contains("s::A::size"), "{c:?}");
    refused(
        &format!("{two} claim c(b: B) {{ b.next() == b.a }} }}"),
        "`b` is `s::B`, which has no method `next`",
    );
    // Where the type is not known, the name must be enough.
    refused(
        &format!("{two} claim c(xs: List<Any>) {{ xs[0].size() == 1 }} }}"),
        "a method of several records",
    );
    refused(
        "impl A<T { }",
        "an `impl` is `impl R { … }`, `impl<T> R<T> { … }`",
    );
}

/// A word of the language is a name where nothing else can stand: a
/// field, after a dot, an input when the word only starts a
/// declaration. Elsewhere `r#word`, as in Rust.
#[test]
fn a_keyword_is_a_name_where_it_cannot_be_confused() {
    let m = "subject s { record Verdict { claim: String, match: 0..=2, r#in: Bool }\n\
             fn name(evidence: Verdict, r#if: 0..=1) -> String { let claim = evidence.claim; claim }\n\
             fn level(v: Verdict) -> Int { v.match + (if v.r#in { 1 } else { 0 }) }\n\
             claim c(claim: Verdict) { name(claim, 0) == claim.claim }\n\
             claim d() { level(Verdict(\"a\", 2, true)) == 3 } }";
    let e =
        "evidence e { for s::name given evidence = Verdict(\"a\", 1, true), r#if = 0 gives \"a\" }";
    let text = mitra_lang::run(m, e)
        .unwrap_or_else(|(_, e)| panic!("{e}"))
        .to_text();
    assert!(
        text.contains("s::d: holds assuming [s::level::spec]"),
        "{text}"
    );
    assert!(
        text.contains("passes on       `claim = s::Verdict(\"a\", 1, true)`"),
        "{text}"
    );
    let e = mitra_lang::run("subject fn f(match: Int) -> Int;", "")
        .map(|_| ())
        .unwrap_err()
        .1;
    assert!(
        e.to_string().contains("as an input, write `r#match`"),
        "{e}"
    );
}

/// The type of every expression is worked out at `check`: what can
/// never fit its place is refused, also on code with no body.
#[test]
fn an_expression_must_fit_its_place() {
    let refused = |model: &str, want: &str| {
        let e = mitra_lang::load(model, "").map(|_| ()).unwrap_err().1;
        assert!(e.to_string().contains(want), "{e}");
    };
    let g = "record P { x: Int } subject fn g(a: 0..=2) -> Int; subject fn p(a: Int) -> P;\n\
             subject fn h(s: String) -> Int;\n";
    let claim = |body: &str| format!("{g}claim c(a: 0..=2) {{ {body} }}");
    refused(
        &claim("g(a) + \"x\" == 1"),
        "`+` takes `Int | Float64`, and `\"x\"` is",
    );
    refused(
        &claim("g(a) + 1.5 > 0.0"),
        "`+` takes two integers or two floats",
    );
    refused(
        &claim("g(a).size == 1"),
        "`g(a)` is `Int`, which has no `.size`",
    );
    refused(&claim("p(a).y == 1"), "`p(a)` is `P`, which has no `.y`");
    refused(&claim("!p(a).x"), "`!` takes `Bool`, and `p(a).x` is `Int`");
    refused(
        &claim("h(g(a)) == 1"),
        "`h`, as `s`, takes `String`, and `g(a)` is `Int`",
    );
    refused(&claim("g(a) + 1"), "`c` is `Int`, never true or false");
    refused(
        &claim("if g(a) { true } else { false }"),
        "`if` takes `Bool`",
    );
    refused(
        &format!("{g}fn k(a: Int) -> String {{ g(1) + a }} claim c() {{ k(1) == \"\" }}"),
        "`k` gives `Int`, never its result type `String`",
    );
    // What may fit is read: a wider type, a value of another variant.
    let ok = claim("g(a) + a >= 0 && p(g(a)).x == a && h(\"s\") == 1");
    mitra_lang::load(&ok, "").unwrap_or_else(|(_, e)| panic!("{e}"));
}

/// A sample says what it drew from and how many: it covers nothing when
/// it passes, and refutes the claim on no stated input when it fails.
#[test]
fn a_sample_covers_nothing_and_can_refute() {
    let m = "fn small(a: Int) -> Bool { a >= 0 && a < 100 }\n\
             subject fn half(a: Int) -> Int;\n\
             claim halves(a: Int) { half(a) * 2 <= a }";
    let run = |e: &str| mitra_lang::run(m, e).map(|r| r.to_text());
    let text = run("evidence t { for halves given a in small sampled 1000 }\n\
                    evidence p { for half given a = 6 gives 3 }")
    .unwrap();
    let want = "halves: unverified\n  passes on       `a = 6`\n  \
                sampled on      1000 drawn from (a in small) (t)\n";
    assert!(text.contains(want), "{text}");
    // A failing sample does not contradict the point that passes.
    let text = run(
        "evidence t { for halves given a in small sampled 1000 fails }\n\
                    evidence p { for half given a = 6 gives 3 }",
    )
    .unwrap();
    let want =
        "halves: fails\n  fails on        one of 1000 drawn from (a in small), not stated (t)\n";
    assert!(text.contains(want), "{text}");
    assert!(
        text.contains("error[M001]") && !text.contains("M003"),
        "{text}"
    );
    assert!(
        text.contains("fails on an input a sample drew and does not state"),
        "{text}"
    );
    for (e, why) in [
        (
            "evidence t { for halves given a in small holds }",
            "write `sampled 1000`",
        ),
        (
            "evidence t { for halves given a = 1 sampled 5 }",
            "drawn from a set",
        ),
        ("evidence t { for halves sampled 0 }", "how many inputs"),
    ] {
        let err = run(e).unwrap_err().1.message;
        assert!(err.contains(why), "{err}");
    }
    // Written back as read.
    let (model, _) = mitra_lang::parse::model(m).unwrap();
    let e = "evidence e { for halves given a in small sampled 7 fails source \"x\" }\n";
    let read = mitra_lang::read_evidence(&mitra_lang::Modules::alone(m), e).unwrap();
    assert_eq!(mitra_lang::print::print_evidence(&read, Some(&model)), e);
}

/// With no `uses` written, a body says what the function calls.
#[test]
fn uses_is_implied_by_a_body() {
    let m = "subject s { fn half(a: Int) -> Int;\n\
             fn quarter(a: Int) -> Int { half(half(a)) }\n\
             fn same(a: Int) -> Int { a }\n\
             fn eighth(a: Int) -> Int uses half { half(quarter(a)) } }";
    let model = mitra_lang::load(m, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    let uses = |f: &str| {
        model
            .claim(&format!("s::{f}::uses"))
            .map(|c| match &c.target {
                mitra_lang::Target::Uses { used, only } => (used.clone(), *only),
                _ => panic!("not a uses claim"),
            })
    };
    assert_eq!(uses("quarter"), Some((vec!["s::half".to_string()], true)));
    // Nothing called, nothing claimed; what is written stands.
    assert_eq!(uses("same"), None);
    assert_eq!(uses("eighth"), Some((vec!["s::half".to_string()], false)));
    // A reader of the code may speak of a function that states none.
    let e = "evidence tie { for s::half::uses fails source \"reader\" }";
    let text = mitra_lang::run(m, e)
        .unwrap_or_else(|(_, e)| panic!("{e}"))
        .to_text();
    assert!(text.contains("s::half::uses: fails"), "{text}");
}

/// A helper a failing claim reads, and the tie of the function to its
/// code, are pieces that can be the wrong one: named, and weighed with
/// the trust given to them.
#[test]
fn a_helper_and_a_tie_are_candidates() {
    let m = |trust: &str| {
        format!(
            "fn small(a: Int) -> Bool {trust}{{ a >= 0 && a < 100 }}\n\
             subject s {{ fn half(a: Int) -> Int;\n\
             claim halves(a: Int) {{ !small(a) || half(a) * 2 <= a }} }}"
        )
    };
    let run = |m: &str, e: &str| mitra_lang::run(m, e).unwrap_or_else(|(_, e)| panic!("{e}"));
    let e = |trust: &str| {
        format!(
            "evidence p {{ for s::half given a = 6 gives 4 }}\n\
             evidence tie {{ for s::half::tied holds {trust}source \"reader:rust src/lib.rs:3; \
             not compared: a is Int in the model\" }}"
        )
    };
    // Trusted in full: the code alone is doubted, the two are named.
    let text = run(&m(""), &e("")).to_text();
    for want in [
        "certain   the code of `s::half`",
        "trusted   the helper `small` as written, model.mitra:1\n",
        "trusted   that `s::half` and its code agree (not compared: a is Int in the model), model.mitra:2\n",
    ] {
        assert!(text.contains(want), "{text}");
    }
    // With less trust they share the doubt.
    let text = run(&m("trust 0.7 "), &e("trust 0.8 ")).to_text();
    for want in [
        "likely    the code of `s::half`",
        "unlikely  the helper `small` as written, model.mitra:1",
        "0.7    0.70 to 1.00",
        "0.8    0.80 to 1.00",
    ] {
        assert!(text.contains(want), "{text}");
    }
    assert!(!text.contains("trusted "), "{text}");
    // No tie, no such piece.
    let text = run(&m(""), "evidence p { for s::half given a = 6 gives 4 }").to_text();
    assert!(!text.contains("its code agree"), "{text}");
}
