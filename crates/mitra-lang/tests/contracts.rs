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
    assert!(lints[0].message.contains("takes a value bound by `Link`"));
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
            "is not a function of the contract",
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

fn bounds(m: &mitra_lang::Model) -> Vec<(&str, &str, Vec<&str>)> {
    let mut all = Vec::new();
    for b in &m.bounds {
        let contracts = b.contracts.iter().map(String::as_str).collect();
        all.push((b.function.as_str(), b.input.as_str(), contracts));
    }
    all
}

#[test]
fn a_bound_names_contracts() {
    let m = model(
        "contract Other { fn stop(&self) -> Bool; }
         subject sync {
            require transport = Link::ordered;
            fn push<L: Link>(l: L, n: Int) -> Bool uses Link::send;
            fn pull<L, M>(l: &L, m: &mut M, n: List<L>) -> Bool where L: Link + Other, M: Other;
            fn both(a: impl Link + Other, b: impl Link) -> Bool; }",
    );
    // A bound parameter is any value; what it is bound by is kept by input.
    let push = m.functions.iter().find(|f| f.name == "sync::push").unwrap();
    assert_eq!(push.params[0].1.to_string(), "Any");
    assert_eq!(
        push.written.as_deref(),
        Some("<L: Link>(l: L, n: Int) -> Bool")
    );
    assert_eq!(
        bounds(&m),
        [
            ("sync::push", "l", vec!["Link"]),
            ("sync::pull", "l", vec!["Link", "Other"]),
            ("sync::pull", "m", vec!["Other"]),
            ("sync::both", "a", vec!["Link", "Other"]),
            ("sync::both", "b", vec!["Link"]),
        ]
    );
    // A bound leans on the contract, as `impl Contract` does: `Other`
    // promises nothing, `Link` is required.
    assert!(m.lints.is_empty(), "{:?}", m.lints);
    let bare = "subject sync { ensure e = \"x\"; fn push<L>(l: L) -> Bool where L: Link; }";
    let lints = model(bare).lints;
    assert_eq!(lints.len(), 1, "{lints:?}");
    assert!(lints[0].message.contains("takes a value bound by `Link`"));
}

#[test]
fn an_impl_has_type_parameters() {
    let m = model(
        "subject s { record Stack<T> { items: List<T> }
            impl<T> Stack<T> {
                fn top(&self) -> T;
                fn zip<U: Link>(&self, other: Stack<U>, with: U) -> Stack<(T, U)>;
                fn same(&self) -> Self;
                claim kept(self) { self.same().items == self.items }
            }
            record Line<T> { items: List<T> }
            impl Line<Int> { fn sum(&self) -> Int; } }",
    );
    let written = |f: &str| {
        let f = m.functions.iter().find(|g| g.name == f).unwrap();
        f.written.clone().unwrap_or_default()
    };
    // Each function has the parameter of its `impl`, before its own.
    assert_eq!(written("s::Stack::top"), "<T>(self: s::Stack<T>) -> T");
    assert_eq!(
        written("s::Stack::zip"),
        "<T, U: Link>(self: s::Stack<T>, other: s::Stack<U>, with: U) -> s::Stack<(T, U)>"
    );
    assert_eq!(
        written("s::Stack::same"),
        "<T>(self: s::Stack<T>) -> s::Stack<T>"
    );
    assert_eq!(bounds(&m), [("s::Stack::zip", "with", vec!["Link"])]);
    // A claim of it is over every `T`.
    assert!(m.claim("s::Stack::kept").is_some());
    // `impl R<Int>`: the functions of one instance.
    let sum = m
        .functions
        .iter()
        .find(|f| f.name == "s::Line::sum")
        .unwrap();
    assert_eq!(sum.written, None);
    assert_eq!(m.named(&sum.params[0].1.to_string()), "s::Line<Int>");
}

const STORE: &str = "import std::core::{Option};
    contract Store<K, V> {
        fn get(&self, key: K) -> Option<V>;
        fn put(&self, key: K, value: V) -> Self;
        ensure kept = \"What is put is got.\";
        claim got(self, k: K, v: V) { self.put(k, v).get(k) == Some(v) }
    }";

#[test]
fn a_contract_has_type_parameters() {
    let src = format!(
        "{STORE}
         subject s {{ record Cache; record Shelf<T> {{ item: T }}
            impl Store<Int, String> for Cache {{ }}
            impl<T> Store<Int, T> for Shelf<T> {{ }}
            fn read<S: Store<Int, String>>(s: S) -> Int uses Store::get; }}"
    );
    let m = mitra_lang::load(&src, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    let f = |name: &str| m.functions.iter().find(|g| g.name == name).unwrap();
    // The contract: its parameters on each function.
    let get = f("Store::get").written.clone().unwrap_or_default();
    assert_eq!(get, "<K, V>(self: Any, key: K) -> Option<V>");
    // An implementer is given the functions with the types it names.
    let get = f("s::Cache::get");
    assert_eq!(get.written, None);
    assert_eq!(get.params[1].1.to_string(), "Int");
    assert_eq!(m.named(&get.range.to_string()), "Option<String>");
    let get = f("s::Shelf::get").written.clone().unwrap_or_default();
    assert_eq!(get, "<T>(self: s::Shelf<T>, key: Int) -> Option<T>");
    assert!(m.claim("s::Cache::got").is_some() && m.claim("s::Shelf::got").is_some());
    assert_eq!(
        from(&m, "Store::kept"),
        ["s::Cache::kept", "s::Shelf::kept"]
    );
    assert_eq!(bounds(&m), [("s::read", "s", vec!["Store"])]);
    // As many types as the contract has parameters.
    let short = format!("{STORE} subject s {{ record Cache; impl Store<Int> for Cache {{ }} }}");
    let said = refused(&short);
    assert!(said.contains("`Store` takes 2 type(s)"), "{said}");
}

#[test]
fn what_generics_leave_out_is_refused() {
    let no = [
        (
            "subject s { fn f<T: Missing>(x: T) -> T; }",
            "`Missing` is not a contract: a bound names a contract",
        ),
        (
            "contract C { fn f(&self) -> Int; }\nsubject s { fn f<T>(x: T) -> T where U: C; }",
            "`f` has no type parameter `U`",
        ),
        (
            "contract C { fn f(&self) -> Int; }\nsubject s { record R<T: C> { x: T } }",
            "a bound is written on a function, an `impl` or a contract",
        ),
        (
            "contract C { fn f(&self) -> Int; }\nsubject s { fn f(x: impl C + D) -> Int; }",
            "`D` is not a contract",
        ),
        (
            "subject s { record R; impl<T R { fn f(&self) -> Int; } }",
            "an `impl` is `impl R { … }`, `impl<T> R<T> { … }`",
        ),
    ];
    for (src, want) in no {
        let got = refused(src);
        assert!(got.contains(want), "{src}\n  gave `{got}`, not `{want}`");
    }
}

// mitra: evidence for contracts::bound_is_kept_by_input
#[test]
fn bound_is_kept_by_input() {
    let src = "contract C { fn f(&self) -> Int; } \
               subject s { fn g<T: C>(x: T, y: impl C) -> Int; }";
    let m = mitra_lang::load(src, "").unwrap_or_else(|(_, e)| panic!("{e}"));
    assert_eq!(
        bounds(&m),
        [("s::g", "x", vec!["C"]), ("s::g", "y", vec!["C"])]
    );
}

// mitra: evidence for contracts::parameter_is_opaque
#[test]
fn parameter_is_opaque() {
    let two = "contract A { fn a(&self) -> Bool; } contract B { fn b(&self) -> Bool; }";
    // Inside its function, a type parameter is a type of its own: only
    // what its bounds promise is known of it.
    let no = [
        ("fn f<T>(x: T) -> Int { x + 1 }", "`x` is `T`"),
        (
            "fn f<T>(x: T) -> Int { x.size }",
            "`x` is `T`, which has no `.size`",
        ),
        (
            "fn h(a: Int) -> Int { a } fn f<T>(x: T) -> Int { h(x) }",
            "takes `Int`, and `x` is `T`",
        ),
        (
            "record P<A, B> { first: A, second: B }
             impl<A, B> P<A, B> { fn f(&self) -> Int { self.first + 1 } }",
            "`self.first` is `A`",
        ),
        (
            "claim c(x: impl A) { x.b() }",
            "`x` is `impl A`, which has no method `b`",
        ),
        (
            "claim c(x: impl A) { B::b(x) }",
            "`x` is `impl A`, which is not bound by `B`",
        ),
        ("claim c(x: impl A) { x + 1 == 2 }", "`x` is `impl A`"),
    ];
    for (src, want) in no {
        let got = refused(&format!("{two}\n{src}"));
        assert!(got.contains(want), "{src}\n  gave `{got}`, not `{want}`");
    }
    // What stays: passing it on, comparing it, and what its bounds give.
    let yes = "fn id<T>(x: T) -> T { x }
               fn same<T>(x: T, y: T) -> Bool { id(x) == y }
               claim c() { same(1, 1) && id(\"a\") == \"a\" }
               claim d(x: impl A + B) { x.a() == A::a(x) && x.b() == B::b(x) }";
    let text = mitra_lang::run(&format!("{two}\n{yes}"), "")
        .unwrap()
        .to_text();
    assert!(text.contains("c: holds"), "{text}");
    // A caller still gives any value.
    let m = mitra_lang::load("fn id<T>(x: T) -> T { x }", "").unwrap();
    assert_eq!(m.functions[0].params[0].1.to_string(), "Any");
    assert_eq!(m.generics.len(), 1);
}

#[test]
fn the_report_says_where_a_generic_was_seen() {
    let model = "import std::core::{Option};
        subject s {
            fn first<T>(xs: List<T>) -> Option<T>;
            record Pair<A, B> { first: A, second: B }
            impl<A, B> Pair<A, B> { fn swap(&self) -> Pair<B, A>; }
            fn plain(n: Int) -> Int;
        }";
    let seen = "evidence a { for s::first given xs = [1, 2] gives Some(1) }
        evidence b { for s::first given xs = [\"a\"] gives Some(\"a\") }
        evidence c { for s::first given xs = [] gives None }
        evidence d { for s::Pair::swap given self = s::Pair(1, \"a\") gives s::Pair(\"a\", 1) }
        evidence e { for s::plain given n = 1 gives 1 }";
    let text = mitra_lang::run(model, seen).unwrap().to_text();
    for want in [
        "s::first::correct: unverified\n  seen at         T = Int; T = String\n",
        "  no evidence at  any other type\ns::first::deterministic",
        "s::Pair::swap::correct: unverified\n  seen at         A = Int, B = String\n",
        "s::plain::correct: unverified\n  no evidence on  n in Int\ns::plain::deterministic",
    ] {
        assert!(text.contains(want), "no `{want}` in\n{text}");
    }
    // With no evidence, nothing is said of instances.
    let text = mitra_lang::run(model, "").unwrap().to_text();
    assert!(!text.contains("seen at"), "{text}");
}

#[test]
fn an_implementer_says_what_it_requires_and_uses() {
    let m = model(
        "boundary disk { ensure whole = \"A file is read whole.\"; fn read(p: String) -> String; }
         subject wires { record Cable;
            impl Link for Cable {
                require files = disk::whole;
                fn send(&self, bytes: List<Int>) -> Bool uses disk::read;
            } }",
    );
    // The need is the implementer's, and its promise rests on it.
    assert_eq!(from(&m, "wires::Cable::files"), ["disk::whole"]);
    assert!(from(&m, "wires::Cable::ordered").contains(&"wires::Cable::files".to_string()));
    assert!(m.claim("wires::Cable::send::uses").is_some());
    assert!(m.lints.is_empty(), "{:?}", m.lints);
    // Written again, a function is the contract's, as the contract writes it.
    let other = format!(
        "{LINK} subject wires {{ record Cable;
            impl Link for Cable {{ fn send(&self, bytes: List<Bool>) -> Bool; }} }}"
    );
    assert!(refused(&other).contains("is not a function of the contract"));
}
