//! Finding 39 B (the user's go, 2026-11-26 13:36): an example is one
//! run. Everything it states is used; on a function with a body it
//! holds or fails, saying why, never unknown: A002, A013, A014, A015,
//! A016, A017. An input known only in part is not one run, and is read
//! as a claim.

use mitra_lang::agree::{agree, Agreement, Settings};
use mitra_lang::check::check;
use mitra_lang::parse::parse;

const DISK: &str = "environment Disk {
    fn small(n: integer) -> r: integer;
    fn big(n: integer) -> r: integer;
    fn neg(n: integer) -> r: integer where { claim below: r < 0; }
    fn known(n: integer) -> r: integer where { examples { (3) -> 7 } }
}
subject R {
    use Disk::small;
    use Disk::big;
    use Disk::neg;
    use Disk::known;
    fn twice(n: integer) -> m: integer { n * 2 }
";

fn findings(functions: &str) -> Vec<String> {
    let text = format!("{DISK}{functions}\n}}");
    let m = parse(&text).unwrap();
    assert_eq!(check(&m), vec![], "{text}");
    agree(&m, &Settings::default())
        .1
        .iter()
        .map(|f| f.to_string())
        .collect()
}

/// The row of the locate report: the test the wrong way round sends the
/// run to `big`, which the example does not give.
// mitra: supports agree::examples
#[test]
fn a_row_down_another_path_fails() {
    let found = findings("    fn route(n: integer) -> r: integer { if n > 10 { small(n) } else { big(n) } } where {\n        examples { given Disk::small(3) -> 30 when (3) then 30 }\n    }");
    assert_eq!(
        found,
        [
            "finding[A014]: example 0: fails: no value for `Disk::big(3)`: the run calls it, and nothing gives it there; give it in the example (`given Disk::big(3) -> …`), or give `Disk::big` a body\n --> R::route",
            "finding[A015]: example 0: `Disk::small(3)` is given but never called: the run calls `Disk::big(3)`\n --> R::route",
        ]
    );
    // The right way round: it holds, and nothing is said.
    let right = findings("    fn route(n: integer) -> r: integer { if n < 10 { small(n) } else { big(n) } } where {\n        examples { given Disk::small(3) -> 30 when (3) then 30 }\n    }");
    assert_eq!(right, Vec::<String>::new());
    // Other arguments to the same function: no value at those.
    let off = findings("    fn route(n: integer) -> r: integer { small(n + 1) } where {\n        examples { given Disk::small(3) -> 30 when (3) then 30 }\n    }");
    assert!(off.iter().any(|f| f.starts_with("finding[A014]: example 0: fails: no value for `Disk::small(4)`")), "{off:?}");
    assert!(off.iter().any(|f| f.starts_with("finding[A015]")), "{off:?}");
}

/// What gives a value: a given, `_` any argument, or the callee's own
/// example at that input. A callee with a body needs none (a given of it
/// is M022).
#[test]
fn what_gives_a_value() {
    assert_eq!(
        findings("    fn a(n: integer) -> r: integer { small(n) } where {\n        examples { given Disk::small(_) -> 5 when (3) then 5 }\n    }"),
        Vec::<String>::new()
    );
    assert_eq!(
        findings("    fn b(n: integer) -> r: integer { known(n) } where {\n        examples { (3) -> 7 }\n    }"),
        Vec::<String>::new()
    );
    assert_eq!(
        findings("    fn c(n: integer) -> r: integer { twice(n) } where {\n        examples { (3) -> 6 }\n    }"),
        Vec::<String>::new()
    );
}

/// A callee's claim that keeps the run from the value expected: A017.
/// One that allows it: no value, A014.
// mitra: supports agree::examples
#[test]
fn a_callee_claim_refuses_the_value() {
    assert_eq!(
        findings("    fn e(n: integer) -> r: integer { neg(n) } where {\n        examples { (3) -> 30 }\n    }"),
        ["finding[A017]: example 0: fails: `Disk::neg(3)` has no value here, and the claim `Disk::neg::below` keeps the run from giving 30\n --> R::e"]
    );
    let found = findings("    fn e(n: integer) -> r: integer { neg(n) } where {\n        examples { (3) -> -30 }\n    }");
    assert!(found.iter().all(|f| f.starts_with("finding[A014]")) && !found.is_empty(), "{found:?}");
}

/// No example of a function with a body is unknown, but one whose input
/// is known only in part.
// mitra: supports agree::examples
#[test]
fn an_example_is_never_unknown() {
    let text = format!("{DISK}    fn f(n: integer) -> r: integer {{ big(n) + small(n) }} where {{\n        examples {{ (1) -> 2, given Disk::big(1) -> 1 when (1) then 2 }}\n    }}\n}}");
    let m = parse(&text).unwrap();
    let (agreed, _) = agree(&m, &Settings::default());
    assert!(agreed
        .iter()
        .filter(|a| a.against.starts_with("example"))
        .all(|a| !matches!(a.agreement, Agreement::Unknown { .. })));
}

/// A record known in part, `S { a: 1, .. }`: not one run, read as a
/// claim, so a value it needs and lacks is unknown.
#[test]
fn an_input_known_in_part_is_not_one_run() {
    let text = "subject S {\n    record { a: integer, b: integer, .. }\n    fn sum(s: S) -> t: integer { s.a + s.b } where {\n        examples { (S { a: 1, b: 2, .. }) -> 3 }\n    }\n    fn rest(s: S) -> t: S { s } where {\n        examples { (S { a: 1, b: 2, .. }) -> S { a: 1, b: 2, .. } }\n    }\n}";
    let m = parse(text).unwrap();
    assert_eq!(check(&m), vec![], "{text}");
    let (_, found) = agree(&m, &Settings::default());
    let found: Vec<String> = found.iter().map(|f| f.to_string()).collect();
    assert!(found.iter().all(|f| !f.starts_with("finding[A014]")), "{found:?}");
}

/// A row of the whole model is one run too: its outputs fail as a
/// function's examples do, and a given it never calls is A015.
// mitra: supports agree::examples
#[test]
fn a_row_of_the_model_is_one_run() {
    let text = format!("{DISK}    fn route(n: integer) -> r: integer {{ if n > 10 {{ small(n) }} else {{ big(n) }} }}\n}}\nexamples {{\n    given Disk::small(3) -> 30 when R::route(3) then 30\n}}");
    let m = parse(&text).unwrap();
    assert_eq!(check(&m), vec![], "{text}");
    let (_, found) = agree(&m, &Settings::default());
    let found: Vec<String> = found.iter().map(|f| f.to_string()).collect();
    assert!(found.iter().any(|f| f.starts_with("finding[A014]: `R::route::r`: fails: no value for `Disk::big(3)`")), "{found:?}");
    assert!(found.iter().any(|f| f.starts_with("finding[A015]: the example: `Disk::small(3)` is given but never called: the run calls `Disk::big(3)`")), "{found:?}");
    assert!(!found.iter().any(|f| f.starts_with("finding[A007]") || f.starts_with("finding[A009]")), "{found:?}");
}

/// What is not one run stays unknown: an input known in part, or a value
/// given known in part whose fields not stated the run reads. And the
/// call named is the one with no value, the last of its function.
#[test]
fn what_is_not_one_run_stays_unknown() {
    let text = "subject S {\n    record { a: integer, b: integer, .. }\n}\nenvironment Disk {\n    fn small(n: integer) -> r: integer;\n    fn get(n: integer) -> s: S;\n}\nsubject R {\n    use Disk::small;\n    use Disk::get;\n    fn f(s: S) -> r: integer { small(s.a) } where {\n        examples { (S { a: 1, b: 2, .. }) -> 3 }\n    }\n    fn g(n: integer) -> r: bool { get(n) == get(n) } where {\n        examples { given Disk::get(1) -> S { a: 1, b: 2, .. } when (1) then true }\n    }\n    fn h(n: integer) -> r: integer { small(n) + small(n + 1) } where {\n        examples { given Disk::small(3) -> 1 when (3) then 2 }\n    }\n}";
    let m = parse(text).unwrap();
    assert_eq!(check(&m), vec![], "{text}");
    let (_, found) = agree(&m, &Settings::default());
    let found: Vec<String> = found.iter().map(|f| f.to_string()).collect();
    assert_eq!(
        found,
        [
            "finding[A007]: example 0: unknown, no value for `Disk::small` here\n --> R::f",
            "finding[A007]: example 0: unknown, no value for the fields of `S` the model does not state (`..`) here\n --> R::g",
            "finding[A014]: example 0: fails: no value for `Disk::small(4)`: the run calls it, and nothing gives it there; give it in the example (`given Disk::small(4) -> …`), or give `Disk::small` a body\n --> R::h",
        ]
    );
}

const SHOP: &str = "environment Shop {
    fn price(n: integer) -> r: integer where { claim positive: r > 0; claim cap: n > 100 || r < 1000; }
}
subject R {
    use Shop::price;
    fn total(n: integer) -> t: integer { price(n) * 2 }
";

fn shop(rest: &str) -> Vec<String> {
    let text = format!("{SHOP}{rest}");
    let m = parse(&text).unwrap();
    assert_eq!(check(&m), vec![], "{text}");
    agree(&m, &Settings::default())
        .1
        .iter()
        .map(|f| f.to_string())
        .collect()
}

/// Finding 39 F: a value given that a claim of its function refuses is
/// said, naming both, in each form of example; one a claim cannot decide
/// (`_` where the claim reads it) is quiet.
// mitra: supports agree::givens_kept
#[test]
fn a_given_the_callee_rules_out_is_said() {
    assert_eq!(
        shop("    fn f(n: integer) -> t: integer { total(n) } where {\n        examples { given Shop::price(3) -> -5 when (3) then -10 }\n    }\n}"),
        ["finding[A018]: example 0: `given Shop::price(3) -> -5` breaks claim `Shop::price::positive`: the world given is one the model rules out\n --> R::f"]
    );
    assert_eq!(
        shop("}\nexamples {\n    given Shop::price(3) -> -5 when R::total(3) then -10\n}"),
        ["finding[A018]: the example: `given Shop::price(3) -> -5` breaks claim `Shop::price::positive`: the world given is one the model rules out\n --> examples::0"]
    );
    assert_eq!(
        shop("}\nexamples {\n    (R::total::n = 3, Shop::price::n = 3, Shop::price::r = -5) -> R::total::t = -10\n}"),
        ["finding[A018]: `Shop::price::r`: `given Shop::price(3) -> -5` breaks claim `Shop::price::positive`: the world given is one the model rules out\n --> examples::0"]
    );
    // Its input read too: at 3, 5000 breaks `cap`; at 200 it keeps it.
    let found = shop("    fn f(n: integer) -> t: integer { total(n) } where {\n        examples { given Shop::price(3) -> 5000 when (3) then 10000, given Shop::price(200) -> 5000 when (200) then 10000 }\n    }\n}");
    assert_eq!(found, ["finding[A018]: example 0: `given Shop::price(3) -> 5000` breaks claim `Shop::price::cap`: the world given is one the model rules out\n --> R::f"]);
    // `_`: `cap` reads `n`, not known, and is quiet; `positive` is not.
    let found = shop("    fn f(n: integer) -> t: integer { total(n) } where {\n        examples { given Shop::price(_) -> 5000 when (3) then 10000, given Shop::price(_) -> 0 when (4) then 0 }\n    }\n}");
    assert_eq!(found, ["finding[A018]: example 1: `given Shop::price(_) -> 0` breaks claim `Shop::price::positive`: the world given is one the model rules out\n --> R::f"]);
}
