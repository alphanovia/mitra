//! The language of intent: what a requirement in prose rests on, promises of
//! the world, `require` against `uses`.

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
