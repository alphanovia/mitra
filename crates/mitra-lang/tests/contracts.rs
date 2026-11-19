//! Contracts: `contract C { … }`, `impl C for T { … }` and `x: impl C`.

use mitra_lang::Side;

const LINK: &str = r#"
contract Link {
    fn send(&self, bytes: List<Int>) -> Bool;
    ensure ordered = "What arrives, arrives in the order it was sent.";
    claim sends_nothing(self) { self.send([]) }
}
"#;

fn model(rest: &str) -> mitra_lang::Model {
    let read = mitra_lang::load(&format!("{LINK}{rest}"), "");
    read.unwrap_or_else(|e| panic!("{}", e.1.message))
}

fn refused(src: &str) -> String {
    mitra_lang::load(src, "").unwrap_err().1.message
}

fn report(rest: &str, evidence: &str) -> String {
    let run = mitra_lang::run(&format!("{LINK}{rest}"), evidence);
    run.unwrap_or_else(|(_, e)| panic!("{}", e.message))
        .to_text()
}

fn from(m: &mitra_lang::Model, claim: &str) -> Vec<String> {
    m.claim(claim)
        .unwrap_or_else(|| panic!("no {claim}"))
        .from
        .clone()
}

#[test]
fn a_contract_is_a_side_of_its_own() {
    let m = model("");
    assert_eq!(
        m.block_of("Link::send").map(|b| b.side),
        Some(Side::Contract)
    );
    // Its functions are nobody's code, and `self` is any value.
    assert!(m.boundaries.contains(&"Link::send".to_string()));
    let send = m.functions.iter().find(|f| f.name == "Link::send").unwrap();
    assert_eq!(send.params[0].1.to_string(), "Any");
    // Unfilled, its promise is assumed on its own word; its claim is of
    // its implementers, and it has none.
    assert!(report("", "").contains("Link::ordered: holds assuming [itself]"));
    assert!(m.claim("Link::sends_nothing").is_none());
}

#[test]
fn an_implementer_is_given_the_contract() {
    let m = model(
        "boundary radio { record Ble; record Wifi;
            ensure powered = \"There is power.\";
            assume always_on() { true }
            impl Link for Ble { assume ble_too(self) { self.send([1]) } }
            impl Link for Wifi { } }",
    );
    for f in ["radio::Ble::send", "radio::Wifi::send"] {
        assert!(m.boundaries.contains(&f.to_string()), "{f}");
    }
    // Each implementer is a group of its own: its promise rests on the
    // claim of the contract and on what its braces say, not on the
    // block around it.
    let ble = from(&m, "radio::Ble::ordered");
    assert!(ble.contains(&"radio::Ble::sends_nothing".to_string()));
    assert!(ble.contains(&"radio::Ble::ble_too".to_string()));
    assert!(!ble.contains(&"radio::always_on".to_string()));
    assert!(!from(&m, "radio::Wifi::ordered").contains(&"radio::Ble::ble_too".to_string()));
    assert!(!from(&m, "radio::powered").contains(&"radio::Ble::ble_too".to_string()));
    // The promise of the contract rests on the same of each.
    assert_eq!(
        from(&m, "Link::ordered"),
        ["radio::Ble::ordered", "radio::Wifi::ordered"]
    );
}

#[test]
fn in_a_subject_the_contract_is_owed() {
    let rest = "subject wires { record Cable { length: 1..=2 }
        impl Link for Cable { } }";
    let m = model(rest);
    assert!(!m.boundaries.contains(&"wires::Cable::send".to_string()));
    assert!(m.claim("wires::Cable::send::correct").is_some());
    // One implementer breaks the promise: the report says which.
    let seen = "evidence a { for wires::Cable::send given self = wires::Cable(1), bytes = [] gives false }";
    let text = report(rest, seen);
    for want in [
        "wires::Cable::sends_nothing: fails",
        "wires::Cable::ordered: fails",
        "Link::ordered: fails\n",
        "fails with      `wires::Cable::ordered`",
    ] {
        assert!(text.contains(want), "no `{want}` in\n{text}");
    }
}

#[test]
fn a_user_is_judged_against_the_contract() {
    let user = "subject sync {
        require transport = Link::ordered;
        ensure delivered = \"Data pushed is received whole or reported lost.\";
        fn push(l: impl Link, data: List<Int>) -> Bool uses Link::send;
        claim pushes(l: impl Link) { l.send([]) == Link::send(l, []) } }";
    let rest = format!("boundary radio {{ record Ble; impl Link for Ble {{ }} }}\n{user}");
    let m = model(&rest);
    let push = m.functions.iter().find(|f| f.name == "sync::push").unwrap();
    assert_eq!(push.params[0].1.to_string(), "Any");
    assert!(m.lints.is_empty(), "{:?}", m.lints);
    assert_eq!(from(&m, "sync::transport"), ["Link::ordered"]);
    // Without the `require`, the block leans on what it does not name.
    let bare = "subject sync { ensure delivered = \"Whole or lost.\";
        fn push(l: impl Link, data: List<Int>) -> Bool; }";
    let lints = model(bare).lints;
    assert_eq!(lints.len(), 1, "{lints:?}");
    assert_eq!(lints[0].code, "M106");
    assert!(lints[0].message.contains("takes an `impl Link`"));
    // Said once where a function also `uses` the contract.
    let both = "subject sync { ensure delivered = \"Whole or lost.\";
        fn push(l: impl Link, data: List<Int>) -> Bool uses Link::send; }";
    assert_eq!(model(both).lints.len(), 1);
}

#[test]
fn what_is_kept_out_is_refused() {
    let no = [
        (
            "contract C { fn f(&self) -> Int { 1 } }",
            "no default method",
        ),
        ("contract C { record R; }", "a contract holds"),
        (
            "subject s { record R; impl D for R { } }",
            "`D` is not a contract",
        ),
        (
            "contract C { fn f(&self) -> Int; }\nrecord R;\nimpl C for R { }",
            "is written in a `subject { … }` or a `boundary { … }`",
        ),
        (
            "contract C { fn f(&self) -> Int; }
             subject s { record R; impl C for R { fn g(&self) -> Int; } }",
            "holds only what decomposes the promises",
        ),
        (
            "contract C { fn f(&self) -> Int; }\nsubject s { record R; impl C { } }",
            "a type fills it with `impl C for Type { … }`",
        ),
        (
            "contract C { fn f(&self) -> Int; }\ncontract C { fn g(&self) -> Int; }",
            "declared twice",
        ),
        (
            "contract C { fn f(&self) -> Int; }
             subject s { record R; impl C for R { } impl C for R { } }",
            "`s::R` fills `C` twice",
        ),
    ];
    for (src, want) in no {
        let got = refused(src);
        assert!(got.contains(want), "{src}\n  gave `{got}`, not `{want}`");
    }
}

const FILLED: &str = "contract C { fn f(&self) -> Int; ensure p = \"x\"; \
                      claim c(self) { self.f() >= 0 } } \
                      subject s { record R; impl C for R { } }";

// mitra: evidence for contracts::implementer_is_given_the_contract
#[test]
fn implementer_is_given_the_contract() {
    let m = mitra_lang::load(FILLED, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    let names: Vec<&str> = m.ensures.iter().map(|x| x.name.as_str()).collect();
    assert_eq!(names, ["s::R::p", "C::p"]);
    assert_eq!(m.ensures[0].claims, ["s::R::f::correct", "s::R::c"]);
    assert!(m.ensures[1].claims.is_empty() && m.ensures[0].requires.is_empty());
    assert_eq!(m.ensures[1].requires, ["s::R::p"]);
}

// mitra: evidence for contracts::unfilled_is_assumed
#[test]
fn unfilled_is_assumed() {
    let alone = "contract C { fn f(&self) -> Int; ensure p = \"x\"; }";
    let r = mitra_lang::run(alone, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    let all: Vec<(&str, mitra_lang::Logic)> =
        (r.verdicts.iter().map(|v| (v.claim.as_str(), v.logic))).collect();
    assert_eq!(all, [("C::p", mitra_lang::Logic::Assuming)]);
}

// mitra: evidence for contracts::contract_is_neither_built_nor_bought
#[test]
fn contract_is_neither_built_nor_bought() {
    let m = mitra_lang::load(FILLED, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    let sides: Vec<Side> = m.blocks.iter().map(|b| b.side).collect();
    assert_eq!(sides, [Side::Contract, Side::Subject]);
}
