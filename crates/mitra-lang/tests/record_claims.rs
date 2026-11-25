//! A record's claims are compared where the model makes its values, and
//! a contract's claim every impl breaks is said once, at the contract
//! (finding 10b, the user's go on points 1 and 3, 2026-11-25).

use mitra_lang::agree::{agree, Agreement, Settings};
use mitra_lang::check::check;
use mitra_lang::model::Model;
use mitra_lang::parse::parse;

fn model(text: &str) -> Model {
    let m = parse(text).unwrap_or_else(|e| panic!("{e}"));
    assert_eq!(check(&m), vec![], "{text}");
    m
}
fn said(text: &str) -> Vec<String> {
    let (_, findings) = agree(&model(text), &Settings::default());
    findings.iter().map(|f| f.to_string()).collect()
}

const SPAN: &str = "subject Span {
    record { lo: integer, hi: integer } where {
        claim ordered: self.lo <= self.hi;
    }
    fn make(a: integer) -> r: Span { Span { lo: a, hi: a - 1 } }
    fn unit() -> r: Span { Span { lo: 0, hi: 1 } }
    fn point(a: integer) -> r: Span { Span { lo: a, hi: a } }
    fn widen(self, d: integer) -> r: Span { Span { lo: self.lo, hi: self.hi + d } }
    fn low(self) -> l: integer { self.lo }
}";

/// Where a function gives the record, its claims are read of what the
/// body gives; where it only reads one, nothing is said.
#[test]
fn a_record_claim_is_read_where_a_value_is_made() {
    let (agreed, _) = agree(&model(SPAN), &Settings::default());
    let of = |f: &str| -> Vec<(&str, &Agreement)> {
        agreed
            .iter()
            .filter(|a| a.at.to_string() == format!("Span::{f}"))
            .map(|a| (a.against.as_str(), &a.agreement))
            .collect()
    };
    assert!(
        matches!(
            of("make")[..],
            [(
                "claim `Span::ordered` of what it gives",
                Agreement::Fails { .. }
            )]
        ),
        "{:?}",
        of("make")
    );
    assert!(
        matches!(
            of("unit")[..],
            [("claim `Span::ordered` of what it gives", Agreement::Holds)]
        ),
        "{:?}",
        of("unit")
    );
    // Bounds lose that both fields are `a`: unknown, and said (A007).
    assert!(
        matches!(of("point")[..], [(_, Agreement::Unknown { .. })]),
        "{:?}",
        of("point")
    );
    // `widen` breaks it for d < 0: `self` here is what it was given.
    assert!(
        matches!(of("widen")[..], [(_, Agreement::Fails { .. })]),
        "{:?}",
        of("widen")
    );
    assert!(of("low").is_empty());
    let out = said(SPAN);
    assert!(
        out.iter().any(|f| f.starts_with(
            "finding[A001]: the body breaks claim `Span::ordered` of what it gives at a = "
        ) && f.ends_with("--> Span::make")),
        "{out:?}"
    );
}

/// A record of a nested part, by its whole name; a function elsewhere
/// that makes one is read too.
#[test]
fn a_nested_record_is_read_where_another_part_makes_it() {
    let out = said(
        "subject geo {
            subject record Range { lo: integer, hi: integer } where { claim ordered: self.lo <= self.hi; }
        }
        subject make {
            fn flip(a: integer, b: integer) -> r: geo::Range { geo::Range { lo: b, hi: a } }
        }",
    );
    assert!(
        out.iter().any(
            |f| f.contains("claim `geo::Range::ordered` of what it gives")
                && f.contains("--> make::flip")
        ),
        "{out:?}"
    );
}

const MONOTONE: &str = "contract Monotone<T: Number> {
    fn f(x: T) -> y: T;
    claim rises(a: T, b: T): !(a <= b) || f(a) <= f(b);
}
subject A {
    fn down(p: integer) -> q: integer { 0 - p }
    impl Monotone<integer> for down;
}
subject B {
    fn neg(p: integer) -> q: integer { 0 - 2 * p }
    impl Monotone<integer> for neg;
}";

/// Every impl breaks the law: one finding, at the contract.
#[test]
fn a_law_every_impl_breaks_is_said_once() {
    let out = said(MONOTONE);
    assert_eq!(out.len(), 1, "{out:?}");
    assert_eq!(
        out[0],
        "finding[A005]: every impl of `Monotone` breaks it: `A::down` false at a = 0, b = 1; `B::neg` false at a = 0, b = 1; the claim, or every impl, is to look at\n --> Monotone::rises"
    );
    // One impl that keeps it: each that breaks it is said, at the impl.
    let out = said(&format!(
        "{MONOTONE}\nsubject C {{\n    fn same(p: integer) -> q: integer {{ 7 }}\n    impl Monotone<integer> for same;\n}}"
    ));
    let at: Vec<&str> = out.iter().filter_map(|f| f.split("--> ").nth(1)).collect();
    assert_eq!(
        at,
        ["A::down::Monotone::rises", "B::neg::Monotone::rises"],
        "{out:?}"
    );
    // One impl alone: said at it, as before.
    let one = MONOTONE.split("subject B").next().unwrap();
    let out = said(one);
    assert_eq!(out.len(), 1, "{out:?}");
    assert!(out[0].ends_with("--> A::down::Monotone::rises"), "{out:?}");
}

/// What a function takes is read in its ranges: a value made out of
/// them is not its.
#[test]
fn a_record_claim_is_read_in_the_ranges_of_what_is_taken() {
    let (agreed, _) = agree(
        &model(
            "subject Span {
                record { lo: integer, hi: integer } where { claim ordered: self.lo <= self.hi; }
                fn small(a: 0..=9) -> r: Span { if a < 10 { Span { lo: 0, hi: 1 } } else { Span { lo: 1, hi: 0 } } }
            }",
        ),
        &Settings::default(),
    );
    assert!(
        matches!(agreed[..], [ref a] if a.agreement == Agreement::Holds),
        "{agreed:?}"
    );
}

/// What takes a record counts on its claims: what relates two of its
/// fields is decided as linear facts.
#[test]
fn what_takes_a_record_counts_on_its_claims() {
    let (agreed, findings) = agree(
        &model(
            "subject Span {
                record { lo: integer, hi: integer } where { claim ordered: self.lo <= self.hi; }
                fn width(self) -> w: integer { self.hi - self.lo } where { claim positive: w >= 0; }
                fn width_of(s: Span) -> w: integer { s.hi - s.lo } where { claim positive: w >= 0; }
                fn widen(self, d: integer) -> r: Span { Span { lo: self.lo, hi: self.hi + d } }
            }",
        ),
        &Settings::default(),
    );
    let of = |f: &str| {
        agreed
            .iter()
            .find(|a| a.at.to_string() == format!("Span::{f}"))
            .map(|a| a.agreement.clone())
    };
    assert_eq!(of("width"), Some(Agreement::Holds));
    assert_eq!(of("width_of"), Some(Agreement::Holds));
    // `self` in the promise is what `widen` takes, not what it gives.
    let shown: Vec<String> = findings.iter().map(|f| f.to_string()).collect();
    assert!(
        shown
            .iter()
            .any(|f| f.contains("at self = Span { lo: 0, hi: 0 }, d = -1")
                && f.ends_with("--> Span::widen")),
        "{shown:?}"
    );
}

const ROWS: &str = "environment Ruler {
    use Span;
    fn measure() -> s: Span;
}
subject Span {
    use Ruler::measure;
    record { lo: integer, hi: integer } where { claim ordered: self.lo <= self.hi; }
    subject record Pair { a: Span, b: Span }
    fn width_of(s: Span) -> w: integer { s.hi - s.lo } where {
        examples { (Span { lo: 3, hi: 1 }) -> -2, (Span { lo: 1, hi: 3 }) -> 2 }
    }
    fn read() -> w: integer { width_of(measure()) } where {
        examples { given Ruler::measure() -> Span { lo: 5, hi: 0 } when () then -5 }
    }
    fn first(p: Pair) -> s: Span { p.a } where {
        examples { (Pair { a: Span { lo: 0, hi: 1 }, b: Span { lo: 2, hi: 1 } }) -> Span { lo: 0, hi: 1 } }
    }
}
examples {
    given Ruler::measure() -> Span { lo: 9, hi: 8 } when Span::read() then 1
}";

/// Every value written in a row, at any depth, against its record's
/// claims: A012 where it breaks one.
#[test]
fn a_value_written_in_a_row_is_checked() {
    let out = said(ROWS);
    let a012: Vec<&String> = out.iter().filter(|f| f.contains("A012")).collect();
    let expect = [
        "finding[A012]: example 0: `p` = Span { lo: 2, hi: 1 } breaks record claim `Span::ordered`\n --> Span::first",
        "finding[A012]: example 0: `Ruler::measure()` = Span { lo: 5, hi: 0 } breaks record claim `Span::ordered`\n --> Span::read",
        "finding[A012]: example 0: `s` = Span { lo: 3, hi: 1 } breaks record claim `Span::ordered`\n --> Span::width_of",
        "finding[A012]: `Ruler::measure()` = Span { lo: 9, hi: 8 } breaks record claim `Span::ordered`\n --> examples::0",
    ];
    // `first` gives a Span its Pair holds: kept, as the Pair was checked
    // where it entered.
    assert!(
        !out.iter()
            .any(|f| f.contains("A007") && f.contains("Span::first")),
        "{out:?}"
    );
    assert_eq!(a012, expect.iter().collect::<Vec<_>>(), "{out:?}");
}

/// A measured value is checked too (X014), by who gave it: the world,
/// the test, or the system.
#[test]
fn a_measured_value_is_checked() {
    use mitra_lang::evidence::{Ledger, Measurement};
    use mitra_lang::explain::{explain, Settings as Explaining};
    use mitra_lang::model::Path;
    use mitra_lang::simulation::Value;
    let m = model(
        "environment Ruler at \"src/ruler.rs\" {
            use Span;
            fn measure() -> s: Span;
        }
        subject Span at \"src/span.rs\" {
            use Ruler::measure;
            record { lo: integer, hi: integer } where { claim ordered: self.lo <= self.hi; }
            fn width_of(s: Span) -> w: integer { s.hi - s.lo }
            fn read() -> w: integer { width_of(measure()) }
            fn flip(s: Span) -> r: Span { Span { lo: s.hi, hi: s.lo } }
        }",
    );
    let span = |lo: i64, hi: i64| {
        let r = match &m.parts.iter().find(|p| p.name == "Span").unwrap().functions[2]
            .gives
            .1
        {
            mitra_lang::simulation::Type::Record(r) => r.clone(),
            t => panic!("{t:?}"),
        };
        Value::Record(r, vec![Value::Int(lo.into()), Value::Int(hi.into())])
    };
    let p = |s: &str| Path::to(&s.split("::").collect::<Vec<_>>());
    let measured = |inputs: Vec<(Path, Value)>, outputs: Vec<(Path, Value)>, seen| Measurement {
        inputs,
        outputs,
        seen,
        count: 1,
        by: "test:t".to_string(),
        effects: Default::default(),
    };
    let ledger = Ledger {
        measurements: vec![
            measured(
                vec![(p("Ruler::measure::s"), span(5, 0))],
                vec![(p("Span::read::w"), Value::Int((-5).into()))],
                true,
            ),
            measured(
                vec![(p("Span::flip::s"), span(3, 1))],
                vec![(p("Span::flip::r"), span(1, 3))],
                true,
            ),
            measured(
                vec![(p("Span::flip::s"), span(1, 3))],
                vec![(p("Span::flip::r"), span(3, 1))],
                true,
            ),
            measured(
                vec![(p("Span::flip::s"), span(1, 3))],
                vec![(p("Span::flip::r"), span(3, 1))],
                false,
            ),
        ],
        reviews: vec![],
    };
    let e = explain(&m, &ledger, &Explaining::default());
    assert_eq!(mitra_lang::evidence::check(&m, &ledger), vec![]);
    let x014: Vec<String> = e
        .findings
        .iter()
        .filter(|f| f.code.to_string() == "X014")
        .map(|f| format!("{}: {}", f.at, f.message))
        .collect();
    assert_eq!(
        x014,
        [
            "measurements::0: the world gave `Ruler::measure::s` holding Span { lo: 5, hi: 0 }, which breaks record claim `Span::ordered`: the environment, or the claim, is wrong",
            "measurements::1: the test gave `Span::flip::s` holding Span { lo: 3, hi: 1 }, which breaks record claim `Span::ordered`: the test, or the claim, is wrong",
            "measurements::2: the system gave `Span::flip::r` holding Span { lo: 3, hi: 1 }, which breaks record claim `Span::ordered`: a drift between the model and the code, or the claim is wrong",
            "measurements::3: the test expected `Span::flip::r` holding Span { lo: 3, hi: 1 }, which breaks record claim `Span::ordered`: a drift between the model and the code, or the claim is wrong",
        ]
    );
}

/// The promise is what a function's claims are read under: without it,
/// a point it rules out would break them.
#[test]
fn a_claim_is_read_where_the_promise_holds() {
    let (agreed, _) = agree(
        &model(
            "subject Flags {
                record { a: bool, b: bool } where { claim implies: !self.a || self.b; }
                fn holds(s: Flags) -> r: bool { !s.a || s.b } where { claim true_: r; }
                fn same(self) -> r: Flags { Flags { a: self.a, b: self.b } }
            }",
        ),
        &Settings::default(),
    );
    let of = |f: &str| {
        agreed
            .iter()
            .find(|a| a.at.to_string() == format!("Flags::{f}"))
            .map(|a| a.agreement.clone())
    };
    assert_eq!(of("holds"), Some(Agreement::Holds), "{agreed:?}");
    assert_eq!(of("same"), Some(Agreement::Holds), "{agreed:?}");
}

/// Records in a sequence, in a variant, known in part, and in a call
/// `then` lists: each checked.
#[test]
fn records_held_anywhere_in_a_row_are_checked() {
    let out = said(
        "environment Log {
            use Span;
            fn write(s: Span) -> ok: ();
        }
        subject Span {
            use Option from \"std\";
            use Log::write;
            record { lo: integer, hi: integer, .. } where { claim ordered: self.lo <= self.hi; }
            fn count(xs: [Span]) -> n: integer { xs.len() } where {
                examples { ([Span { lo: 4, hi: 2, .. }]) -> 1 }
            }
            fn some(o: Option<Span>) -> n: integer { match o { Some(s) => 1, None => 0 } } where {
                examples { (Some(Span { lo: 6, hi: 5, .. })) -> 1 }
            }
            fn log(s: Span) -> n: integer { write(s); 0 } where {
                examples { when (Span { lo: 0, hi: 1, .. }) then 0, Log::write(Span { lo: 8, hi: 7, .. }) }
            }
        }",
    );
    for (bad, at) in [
        ("lo: 4, hi: 2", "Span::count"),
        ("lo: 6, hi: 5", "Span::some"),
        ("lo: 8, hi: 7", "Span::log"),
    ] {
        assert!(
            out.iter()
                .any(|f| f.contains("A012") && f.contains(bad) && f.ends_with(at)),
            "{bad}: {out:?}"
        );
    }
}

/// What a row of the model says a function gives is checked too.
#[test]
fn what_a_model_row_expects_is_checked() {
    let out = said(
        "subject Span {
            record { lo: integer, hi: integer } where { claim ordered: self.lo <= self.hi; }
            fn flip(s: Span) -> r: Span { Span { lo: s.hi, hi: s.lo } }
        }
        examples {
            when Span::flip(Span { lo: 1, hi: 3 }) then Span { lo: 3, hi: 1 }
        }",
    );
    assert!(
        out.iter().any(|f| f.contains("A012")
            && f.contains("Span { lo: 3, hi: 1 }")
            && f.ends_with("examples::0")),
        "{out:?}"
    );
}

/// A record a body makes and passes on is read where it is made: what
/// takes it counts on its claims (the coordinator's hole, 13:42).
#[test]
fn a_record_made_on_the_way_is_read_where_it_is_made() {
    let text = "subject Span {
        use Option from \"std\";
        record { lo: integer, hi: integer } where { claim ordered: self.lo <= self.hi; }
        fn width_of(s: Span) -> w: integer { s.hi - s.lo }
        fn arg(a: integer) -> w: integer { width_of(Span { lo: a, hi: a - 1 }) }
        fn bound(a: integer) -> w: integer { let b = a + 1; width_of(Span { lo: b, hi: a }) }
        fn guarded(a: integer, b: integer) -> w: integer { if a <= b { width_of(Span { lo: a, hi: b }) } else { 0 } }
        fn update(s: Span) -> w: integer { width_of(Span { hi: s.lo - 1, ..s }) }
        fn arm(o: Option<integer>) -> w: integer { match o { Some(x) => width_of(Span { lo: x, hi: x }), None => 0 } }
        fn given(a: integer) -> s: Span { if a < 0 { Span { lo: 0, hi: 0 } } else { Span { lo: a, hi: a + 1 } } }
    }";
    let (agreed, _) = agree(&model(text), &Settings::default());
    let of = |f: &str| -> Vec<(String, Agreement)> {
        agreed
            .iter()
            .filter(|a| a.at.to_string() == format!("Span::{f}"))
            .map(|a| (a.against.clone(), a.agreement.clone()))
            .collect()
    };
    let made = "claim `Span::ordered` of the `Span` it makes".to_string();
    assert!(
        matches!(&of("arg")[..], [(m, Agreement::Fails { .. })] if *m == made),
        "{:?}",
        of("arg")
    );
    assert!(
        matches!(&of("bound")[..], [(m, Agreement::Fails { .. })] if *m == made),
        "{:?}",
        of("bound")
    );
    // Under its `if`: only where `a <= b`.
    assert_eq!(of("guarded"), [(made.clone(), Agreement::Holds)]);
    assert!(
        matches!(&of("update")[..], [(m, Agreement::Fails { .. })] if *m == made),
        "{:?}",
        of("update")
    );
    // Of a name a `match` binds: not read, and said.
    assert!(
        matches!(&of("arm")[..], [(m, Agreement::Unknown { why })] if *m == made && why.contains("`x`")),
        "{:?}",
        of("arm")
    );
    // What it gives is read once, as what it gives.
    let given = of("given");
    assert_eq!(given.len(), 1, "{given:?}");
    assert_eq!(given[0].0, "claim `Span::ordered` of what it gives");
}

/// Red team: `let`s, the `else` of an `if`, arms and records inside
/// records, each read where it stands.
#[test]
fn where_a_record_is_made_is_read_exactly() {
    let text = "subject Span {
        use Option from \"std\";
        record { lo: integer, hi: integer } where { claim ordered: self.lo <= self.hi; }
        subject record Pair { a: Span, n: integer }
        fn width_of(s: Span) -> w: integer { s.hi - s.lo }
        fn let_holds(a: integer) -> w: integer { let b = a + 1; width_of(Span { lo: a, hi: b }) }
        fn rebound(o: Option<integer>) -> w: integer { match o { Some(x) => { let x = 1; width_of(Span { lo: x, hi: x }) }, None => 0 } }
        fn otherwise(a: integer, b: integer) -> w: integer { if b < a { 0 } else { width_of(Span { lo: a, hi: b }) } }
        fn pick(o: Option<integer>) -> s: Span { match o { Some(x) => Span { lo: x, hi: x }, None => Span { lo: 0, hi: 0 } } }
        fn named(a: integer) -> s: Span { let b = a; Span { lo: b, hi: b } }
        fn pair(a: integer) -> p: Pair { Pair { a: Span { lo: a, hi: a - 1 }, n: 0 } }
    }";
    let (agreed, _) = agree(&model(text), &Settings::default());
    let of = |f: &str| -> Vec<(String, Agreement)> {
        agreed
            .iter()
            .filter(|a| a.at.to_string() == format!("Span::{f}"))
            .map(|a| (a.against.clone(), a.agreement.clone()))
            .collect()
    };
    let made = "claim `Span::ordered` of the `Span` it makes".to_string();
    assert_eq!(of("let_holds"), [(made.clone(), Agreement::Holds)]);
    assert_eq!(of("rebound"), [(made.clone(), Agreement::Holds)]);
    assert_eq!(of("otherwise"), [(made.clone(), Agreement::Holds)]);
    // What its arms and its `let` give is what it gives: read once.
    for f in ["pick", "named"] {
        let said = of(f);
        assert_eq!(said.len(), 1, "{f}: {said:?}");
        assert_eq!(said[0].0, "claim `Span::ordered` of what it gives");
    }
    assert!(
        matches!(&of("pair")[..], [(m, Agreement::Fails { .. })] if *m == made),
        "{:?}",
        of("pair")
    );
}
