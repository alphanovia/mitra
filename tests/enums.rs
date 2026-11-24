//! Red team of enums, `match` and type parameters on records (finding 3,
//! step 1; your "ok", 19:56). Each guard has a test that fails when it
//! is taken out: exhaustive `match`, the arm a variant takes, a span per
//! variant by bounds, the type of `None` told from where it goes, the
//! declarations no value fits, and the text read back as printed.

use mitra_core::agree::{agree, Agreement, Settings};
use mitra_core::check::check;
use mitra_core::evidence::check as check_evidence;
use mitra_core::explain::{explain, Settings as ExplainSettings};
use mitra_core::parse::parse;
use mitra_core::print::print;
use mitra_core::reader::read_store;

/// The first parse error, rendered without the source line.
fn refused(text: &str) -> String {
    parse(text)
        .map(|_| "parsed".to_string())
        .unwrap_or_else(|e| e.to_string())
}

fn errors(text: &str) -> Vec<String> {
    check(&parse(text).unwrap())
        .iter()
        .map(|e| e.to_string())
        .collect()
}

/// Each comparison, `path: outcome` with the point of a failure.
fn agreed(text: &str) -> Vec<String> {
    let model = parse(text).unwrap();
    assert_eq!(check(&model), vec![], "{text}");
    let (agreed, _) = agree(&model, &Settings::default());
    agreed
        .iter()
        .map(|a| {
            let outcome = match &a.agreement {
                Agreement::Holds => "holds".to_string(),
                Agreement::Fails { at, .. } => {
                    let at: Vec<String> = at
                        .iter()
                        .map(|(n, v)| format!("{n} = {}", mitra_core::print::value(v)))
                        .collect();
                    format!("fails at {}", at.join(", "))
                }
                Agreement::Unknown { why } => format!("unknown: {why}"),
            };
            format!("{} {}: {outcome}", a.at, a.against)
        })
        .collect()
}

const HALF: &str = "subject enum Option<T> { None, Some(T) }
subject calc {
    fn half(n: 0..=9) -> h: Option<integer> { if n % 2 == 0 { Some(n / 2) } else { None } } where {
        claim back: match h { Some(q) => q * 2 == n, None => n % 2 == 1 };
        examples { (4) -> Some(2), (3) -> None }
    }
}";

#[test]
fn a_claim_on_an_option_is_decided_on_every_point() {
    let out = agreed(HALF);
    assert!(out.iter().all(|o| o.ends("holds")), "{out:?}");
    // The wrong arm taken, or `None` read as `Some`: a failure at a point.
    let wrong = HALF.replace("None => n % 2 == 1", "None => n % 2 == 0");
    let out = agreed(&wrong);
    assert!(
        out.iter()
            .any(|o| o.contains("claim `back`: fails at n = ")),
        "{out:?}"
    );
    let wrong = HALF.replace("Some(q) => q * 2 == n", "Some(q) => q == n");
    let out = agreed(&wrong);
    assert!(
        out.iter()
            .any(|o| o.contains("claim `back`: fails at n = ")),
        "{out:?}"
    );
}

#[test]
fn an_example_row_of_a_variant_is_compared() {
    let wrong = HALF.replace("(4) -> Some(2)", "(4) -> Some(3)");
    let out = agreed(&wrong);
    assert!(out.iter().any(|o| o.contains("fails")), "{out:?}");
    let wrong = HALF.replace("(3) -> None", "(3) -> Some(1)");
    let out = agreed(&wrong);
    assert!(out.iter().any(|o| o.contains("fails")), "{out:?}");
}

/// Ranges must keep each variant that may be: a unit enum whose claim is
/// broken on one variant only.
#[test]
fn every_variant_is_tried() {
    let text = "subject enum Light { Red, Amber, Green }
subject road {
    fn go(l: Light) -> g: bool { match l { Green => true, _ => false } } where {
        claim never_red: match l { Red => !g, Amber => !g, Green => g };
        claim wrong: match l { Amber => g, _ => true };
    }
}";
    let out = agreed(text);
    assert!(
        out.iter()
            .any(|o| o.contains("never_red") && o.ends("holds")),
        "{out:?}"
    );
    assert!(
        out.iter()
            .any(|o| o.contains("wrong") && o.contains("fails at l = Amber")),
        "{out:?}"
    );
}

#[test]
fn a_match_gives_each_variant_one_arm() {
    let light = |arms: &str| {
        refused(&format!(
            "subject enum Light {{ Red, Amber, Green }} subject s {{ fn go(l: Light) -> g: bool {{ match l {{ {arms} }} }} }}"
        ))
    };
    assert!(light("Green => true, Amber => false").contains("error[P006]: `match` misses `Red`"));
    assert!(light("Red => true, Red => false, _ => true").contains("`Red` is matched twice"));
    assert!(
        light("Red => true, Amber => true, Green => true, _ => false")
            .contains("`_` matches nothing")
    );
    assert!(light("_ => true, _ => false").contains("`_` is written twice"));
    assert_eq!(light("Red => true, _ => false"), "parsed");
}

#[test]
fn variants_are_named_and_given_what_they_hold() {
    let f = |body: &str| {
        refused(&format!(
            "subject enum Option<T> {{ None, Some(T) }} subject enum Shape {{ None, Dot(rational) }} subject s {{ fn f(x: integer) -> y: Option<integer> {{ {body} }} }}"
        ))
    };
    assert!(
        f("Some(x, x)").contains("error[P007]"),
        "{}",
        f("Some(x, x)")
    );
    assert!(f("Some").contains("error[P007]"), "{}", f("Some"));
    // No such variant: read as a call of a function the model lacks.
    let text = "subject enum Option<T> { None, Some(T) }\nsubject s { fn f(x: integer) -> y: Option<integer> { Nothing(x) } }";
    assert!(
        errors(text).iter().any(|e| e.starts_with("error[M")),
        "{:?}",
        errors(text)
    );
    // Two enums have `None`: it is written with its enum.
    assert!(f("None").contains("error[P007]"), "{}", f("None"));
    assert_eq!(f("Option::None"), "parsed");
    assert_eq!(f("Option::Some(x)"), "parsed");
    // `Some(true)` is an `Option<bool>`: check says the body is of another type.
    let text = "subject enum Option<T> { None, Some(T) }\nsubject s { fn f(x: integer) -> y: Option<integer> { Some(true) } }";
    assert!(
        errors(text)[0].contains("it should be Option<integer>"),
        "{:?}",
        errors(text)
    );
}

#[test]
fn types_are_given_their_arguments() {
    let f = |ty: &str| {
        refused(&format!(
            "subject enum Option<T> {{ None, Some(T) }} subject record N<T: Number> {{ v: T }} subject s {{ fn f(x: {ty}) -> y: integer {{ 0 }} }}"
        ))
    };
    assert!(f("Option").contains("error[P008]"));
    assert!(f("Option<integer, integer>").contains("error[P008]"));
    assert!(f("N<bool>").contains("error[P008]"), "{}", f("N<bool>"));
    assert!(
        f("integer<integer>").contains("error[P008]") || f("integer<integer>").contains("error")
    );
    assert_eq!(f("N<rational>"), "parsed");
    assert_eq!(f("Option<N<integer>>"), "parsed");
}

#[test]
fn the_type_of_none_is_told_from_where_it_goes() {
    let f = |body: &str| {
        refused(&format!(
            "subject enum Option<T> {{ None, Some(T) }} subject s {{ fn g(o: Option<integer>) -> b: bool {{ true }} fn f(x: integer) -> y: bool {{ {body} }} }}"
        ))
    };
    assert!(f("None == None").contains("error[P009]: the type of `None` cannot be told here"));
    assert_eq!(f("None == Some(x)"), "parsed");
    assert_eq!(f("Some(x) == None"), "parsed");
    assert_eq!(f("g(None)"), "parsed");
    assert_eq!(f("let o = Some(x) in o == None"), "parsed");
    assert_eq!(f("if x < 0 { None == Some(1) } else { true }"), "parsed");
    // A generic record told by its fields, or not.
    let r = |body: &str| {
        refused(&format!(
            "subject enum Option<T> {{ None, Some(T) }} subject record Box<T> {{ v: Option<T> }} subject s {{ fn f(x: integer) -> y: bool {{ {body} }} }}"
        ))
    };
    assert!(r("Box { v: None } == Box { v: None }").contains("error[P009]"));
    assert_eq!(r("Box { v: Some(x) } == Box { v: None }"), "parsed");
}

#[test]
fn declarations_no_value_fits_are_m027() {
    let out = errors(
        "subject enum Never {}
subject record Box<T> { n: integer }
subject enum Tree { Leaf, Node(Tree, integer, Tree) }
subject d {
    fn f(t: Tree, b: Box<integer>) -> e: integer { 0 }
}",
    );
    assert_eq!(out.len(), 3, "{out:?}");
    assert!(out.iter().all(|e| e.starts_with("error[M027]")), "{out:?}");
}

#[test]
fn a_model_reads_back_as_printed() {
    let text = "subject enum Option<T> { None, Some(T) }
subject enum Light { Red, Amber, Green }
subject record Pair<T> { a: T, b: T }
subject s {
    fn f(p: Pair<integer>, l: Light) -> o: Option<integer> {
        match l { Red => None, Amber => Some(p.a), _ => Some(p.b) }
    } where {
        claim c: match o { Some(v) => v == p.a || v == p.b, None => l == Light::Red };
        examples { (Pair { a: 1, b: 2 }, Amber) -> Some(1), (Pair { a: 1, b: 2 }, Red) -> None }
    }
}";
    let model = parse(text).unwrap();
    assert_eq!(check(&model), vec![]);
    let printed = print(&model);
    let again = parse(&printed).unwrap_or_else(|e| panic!("{e}\n{printed}"));
    assert_eq!(model, again, "{printed}");
    assert_eq!(print(&again), printed);
    // `v == p.a || v == p.b` is relational: bounds leave it unknown,
    // which A007 says; nothing fails.
    assert!(agreed(text).iter().all(|o| !o.contains("fails")));
}

#[test]
fn a_measured_variant_is_evidence() {
    let text = "subject enum Option<T> { None, Some(T) }
subject calc {
    fn half(n: 0..=9) -> h: Option<integer> at \"src/lib.rs/half\" { if n % 2 == 0 { Some(n / 2) } else { None } }
}";
    let model = parse(text).unwrap();
    assert_eq!(check(&model), vec![]);
    let store = "measure calc::half (3) -> None by \"test:odd\";\nmeasure calc::half (4) -> Some(3) by \"test:even\";";
    let ledger = read_store("s", store, &model, &model.ties).unwrap().0;
    assert_eq!(check_evidence(&model, &ledger), vec![]);
    let e = explain(&model, &ledger, &ExplainSettings::default());
    let shown: Vec<String> = e.verdicts.iter().map(|v| format!("{v:?}")).collect();
    let at = |n: &str| {
        shown
            .iter()
            .find(|v| v.contains(&format!("[Int({n})]")))
            .cloned()
    };
    assert!(
        at("3").is_some_and(|v| v.contains("Explained")),
        "{shown:#?}"
    );
    assert!(
        at("4").is_some_and(|v| v.contains("Contradicted")),
        "{shown:#?}"
    );
}

trait Ends {
    fn ends(&self, s: &str) -> bool;
}
impl Ends for String {
    fn ends(&self, s: &str) -> bool {
        self.ends_with(s)
    }
}
