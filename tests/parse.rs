//! The text of a model (`docs/design/syntax.md`): `parse` reads what `print`
//! writes. The fall and div of `examples/` parse, check, and come back the
//! same; every form of the language comes back the same; random
//! expressions come back the same; errors say where, as rustc does.

mod common;

use common::apply;
use mitra_core::catalogue::{P001, P002, P003, P004, P005};
use mitra_core::check::check;
use mitra_core::expression::{Arg, Expression};
use mitra_core::judgement::{self, Prop, Set};
use mitra_core::location::Location;
use mitra_core::model::{
    Claim, Ensure, Example, Function, Kind, Model, Part, Path, Record, Require, Rule, Source, Term,
    Tie,
};
use mitra_core::parse::{parse, parse_prop, parse_row};
use mitra_core::print::{print, print_prop, real as real_text};
use mitra_core::simulation::{self, tuple, Op, Rational, Type, Value};
use proptest::prelude::*;

type E = Expression<Op>;

// ---------------------------------------------------------------- the fall, by hand (as tests/model.rs)

fn name(x: &str) -> E {
    Expression::Name(x.to_string())
}
fn real(n: i64) -> E {
    apply(Op::Real(Rational::integer(n)), vec![])
}
fn ratio(n: i64, d: i64) -> E {
    apply(Op::Real(Rational::new(n, d).unwrap()), vec![])
}
fn path(p: &str) -> Path {
    Path::to(&p.split("::").collect::<Vec<_>>())
}
fn at(p: &str) -> Term {
    Term::At(path(p))
}
fn claim(name: &str, holds: E) -> Claim {
    Claim {
        name: name.to_string(),
        holds,
        induction: None,
    }
}
fn function(name: &str, takes: &str, gives: (&str, Type)) -> Function {
    Function {
        name: name.to_string(),
        takes: vec![(takes.to_string(), Type::Real)],
        gives: (gives.0.to_string(), gives.1),
        body: None,
        examples: vec![],
        claims: vec![],
        rules: vec![],
        ranges: Default::default(),
    }
}

/// 2·g·h, with g = 9.81 m/s².
fn two_g_h() -> E {
    let two_g = apply(Op::Mul, vec![real(2), ratio(981, 100)]);
    apply(Op::Mul, vec![two_g, name("h")])
}
/// `!(h <= 20) || x < max`: from 20 m or less, `x` stays under `max`.
fn under(x: &str, max: i64) -> E {
    let low = apply(Op::AtMost, vec![name("h"), real(20)]);
    let under = apply(Op::Less, vec![name(x), real(max)]);
    apply(Op::Or, vec![apply(Op::Not, vec![low]), under])
}

/// The rule `slows`, as judgement reads it: by its name.
fn slows() -> Prop {
    Prop::named("stone::drop::slows")
}

/// The sun, outside the system. It gives light, how is not known, and
/// ensures that it shines, which only someone taking it holds.
fn sun() -> Part {
    let mut light = function("light", "", ("w", Type::Real));
    light.takes.clear();
    Part {
        inner: vec![],
        file: None,
        enums: vec![],
        name: "sun".to_string(),
        kind: Kind::Environment,
        uses: vec![],
        implements: vec![],
        records: vec![],
        functions: vec![light],
        requires: vec![],
        ensures: vec![Ensure {
            name: "shines".to_string(),
            prose: "at least 200 W/m² at noon".to_string(),
            from: vec![],
        }],
    }
}

/// The stone, in the system. `drop` is how it falls from `h` metres at
/// `v` metres per second: nature does it, how is not known. `speed` is
/// the model's own square of that speed.
fn stone() -> Part {
    let mut drop = function("drop", "h", ("v", Type::Real));
    let v_squared = apply(Op::Mul, vec![name("v"), name("v")]);
    // v² = 2·g·h, within 2 m²/s². From 20 m or less, under 21 m/s.
    let falls = apply(Op::Within, vec![v_squared, two_g_h(), real(2)]);
    drop.claims = vec![claim("falls", falls), claim("slow", under("v", 21))];
    // Arithmetic, which no rule of judgement does: assumed.
    let assumed = Rule {
        name: "slows".to_string(),
        premises: vec![at("stone::drop::falls"), at("stone::speed::slow")],
        conclusion: at("stone::drop::slow"),
        proof: None,
        short: None,
    };
    // The same, shown: from `slows` and its premises, the conclusion,
    // by modus ponens.
    let names = [
        "stone::drop::slows",
        "stone::drop::falls",
        "stone::speed::slow",
    ];
    let ponens = judgement::Op::Ponens {
        t: slows(),
        post: 0,
    };
    let lands = Rule {
        name: "lands".to_string(),
        premises: names.iter().map(|n| at(n)).collect(),
        conclusion: at("stone::drop::slow"),
        proof: Some(apply(
            ponens,
            names
                .iter()
                .map(|n| Expression::Name(n.to_string()))
                .collect(),
        )),
        short: None,
    };
    drop.rules = vec![assumed, lands];
    let mut speed = function("speed", "h", ("v2", Type::Real));
    speed.body = Some(two_g_h());
    speed.claims = vec![claim("slow", under("v2", 400))];
    // What it gives at 5 m: 2·9.81·5.
    let real = |n, d| Value::Real(Rational::new(n, d).unwrap());
    speed.examples = vec![Example {
        takes: vec![real(5, 1)],
        gives: real(981, 10),
    }];
    Part {
        inner: vec![],
        file: None,
        enums: vec![],
        name: "stone".to_string(),
        kind: Kind::Subject,
        uses: vec![],
        implements: vec![],
        records: vec![],
        functions: vec![drop, speed],
        requires: vec![
            Require {
                name: "light".to_string(),
                from: Source::Ensure(path("sun::shines")),
            },
            Require {
                name: "air".to_string(),
                from: Source::Prose("still air".to_string()),
            },
        ],
        ensures: vec![Ensure {
            name: "lands_slow".to_string(),
            prose: "lands slowly".to_string(),
            from: vec![vec![path("stone::speed::slow"), path("stone::drop::falls")]],
        }],
    }
}

/// The fall: the stone and the sun, and where the drop is.
fn fall() -> Model {
    Model {
        parts: vec![sun(), stone()],
        contracts: vec![],
        examples: vec![],
        ties: vec![Tie {
            item: path("stone::drop"),
            at: Location::at(&["lab", "tower", "stone"]),
        }],
    }
}

// ---------------------------------------------------- the models of `examples/`

fn file(name: &str) -> String {
    let path = format!("{}/examples/{name}", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("no {path}"))
}

fn read(text: &str) -> Model {
    parse(text).unwrap_or_else(|e| panic!("{}", e.render("text", text)))
}

/// `parse(print(m))` is `m`, and printing again writes the same text.
fn round_trip(m: &Model) {
    let text = print(m);
    let again = read(&text);
    assert_eq!(&again, m, "the text read back differs:\n{text}");
    assert_eq!(print(&again), text);
}

#[test]
fn the_fall_reads_as_built_by_hand() {
    let m = read(&file("fall/fall.mitra"));
    assert_eq!(m, fall());
    assert_eq!(check(&m), vec![]);
    round_trip(&m);
}

#[test]
fn div_reads_checks_and_comes_back() {
    let m = read(&file("div/tied.mitra"));
    assert_eq!(check(&m), vec![]);
    round_trip(&m);
    let div = &m.parts[0].functions[0];
    assert_eq!(
        div.takes,
        vec![("a".to_string(), Type::Int), ("b".to_string(), Type::Int)]
    );
    assert_eq!(div.examples.len(), 4);
    let int = |n: i64| Value::Int(n.into());
    assert_eq!(
        div.examples[0],
        Example {
            takes: vec![int(6), int(2)],
            gives: int(3)
        }
    );
    assert_eq!(
        m.ties,
        vec![Tie {
            item: path("calc::div"),
            at: Location::at(&["src", "lib.rs", "div"])
        }]
    );
}

/// The printed fall, as the user reads it.
#[test]
fn the_fall_prints_as_written() {
    let expected = r#"environment sun {
    fn light() -> w: rational;

    ensure shines: "at least 200 W/m² at noon";
}

subject stone {
    require light: sun::shines;
    require air: "still air";

    fn drop(h: rational) -> v: rational at "lab/tower/stone" where {
        claim falls: within(v * v, 2.0 * 9.81 * h, 2.0);
        claim slow: !(h <= 20.0) || v < 21.0;
        rule slows: drop::falls, speed::slow |- drop::slow;
        rule lands: drop::slows, drop::falls, speed::slow |- drop::slow
            by ponens(drop::slows, drop::falls, speed::slow);
    }

    fn speed(h: rational) -> v2: rational { 2.0 * 9.81 * h } where {
        claim slow: !(h <= 20.0) || v2 < 400.0;
        examples { (5.0) -> 98.1 }
    }

    ensure lands_slow: "lands slowly" from (speed::slow, drop::falls);
}
"#;
    assert_eq!(print(&fall()), expected);
}

// ---------------------------------------------------------------- every form

/// Every form the language has, once at least.
const EVERY_FORM: &str = r#"
contract Monotone<T: Number> {
    fn f(x: T) -> y: T;
    claim rises(a: T, b: T): !(a <= b) || f(a) <= f(b);
}

contract Same<T, U> {
    fn g(x: T) -> y: T;
    fn h(x: U) -> y: U;
    claim keeps(a: T, b: U): g(a) == a && Same::h(b) == b;
}

environment record Photon;
environment record Beam where {
    claim lit: true;
}
environment lab at "lab", "lab/annex" {
    fn clock() -> t: integer;
    fn dim() -> d: integer;
    fn shine(t: integer) -> w: rational;
    ensure lit: "the lamp is on" from ();
}

subject record Point at "src/geo.rs/Point" { x: rational, y: rational } where {
    claim right: 0.0 <= self.x && self.y >= -1/3;
    rule upper: Point::right, { self.x < 10.0 } |- Point::right;
}
subject record Empty {}
subject record Pair { p: Point, n: (integer, bool) }
subject geo at "src/geo.rs" {
    use lab::{clock, dim};
    use lab::*;
    use lab::shine;
    require lamp: lab::lit;
    require geo: "a require named as its part";
    fn flip(p: Point) -> q: Point { Point { y: p.x, x: p.y } } where {
        claim back: q.x == p.y && q.y != p.x;
        claim pair: (p.x, 1, true).0 == p.x;
        claim one: (3,) > 2;
        claim unit: () == ();
        claim cond: if (Point { x: 0.0, y: 0.0 }) == p { true } else { q.x - -2.5 <= 1/7 };
        claim let_: let d = q.x - p.y in let e = d * d in e == 0.0 || e / 2.0 < 1.0;
        claim nested: if p.x < 0.0 { false } else if p.y < 0.0 { false } else { true };
        claim sweep: (0..10).all(|i| (i..i + 1).all(|j| j - i == 0));
        claim tol: within(p.x, q.y, 0.001);
        claim paren: (1 + 2) * 3 == 9 && !(1 < 2 || false) == false;
        rule same: flip::back |- flip::pair;
        rule nothing: |- { true };
        rule shown: flip::same, flip::back |- flip::pair
            by ponens[flip::same](flip::same, flip::back);
        rule again: flip::same, flip::back |- flip::pair
            by let s = flip::same in ponens(s, flip::back);
        rule against: flip::same, flip::pair, { false } |- flip::back
            by tollens[flip::same, 0, 1](flip::pair, 2);
        rule rest: flip::back |- flip::pair by reductio[!flip::back](flip::back);
        rule over: flip::back |- flip::pair
            by cases[all x in (a: 0..2, b: -1..1) where b != 0: correct(flip, x)](flip::back, flip::back);
        rule points: flip::back |- flip::pair
            by instance[any x in (a: 0.0..1.0): {flip::back, lab::lit} flip {correct(flip, (Point { x: 0.5, y: 1.0 }))}](witness[all x in (n: 0..1): flip::back](flip::back, point(0)), point(0.5));
        rule any_point: flip::back |- flip::pair
            by intro[all x in (a: 0..3): correct(flip, x)](|p| instance[all x in (a: 0..3): correct(flip, x)](reductio[correct(flip, x in (a: 0..3) where a != 1)](flip::back), p));
        examples {
            (Point { y: 2.0, x: 1.0 }) -> Point { x: 2.0, y: 1.0 },
            (Point { x: 0.0, y: 0.0 }) -> Point { x: 0.0, y: 0.0 },
        }
    }
    fn pair(a: integer, b: (integer, bool)) -> c: Pair;
    fn constant() -> c: rational { (1.5,) }
    fn later(d: integer) -> t: integer { clock() + d } where {
        claim after: t == lab::clock() + d && shine(t) <= shine(t + 1) && (later(d), 1).0 == t;
        rule everywhere: later::after |- all x in (d: 0..3): correct(later, x);
        rule here: (later::after), !later::after, ({later::after} later {later::after}) |- correct(later, (1));
        rule any_: later::after |- any x in (d: 0..3) where d != 1: !correct(crate::lab::clock, x);
    }
    fn rise(h: rational) -> v: rational { geo::rise2(h) * 2.0 }
    fn rise2(h: rational) -> v: rational { h }
    impl Monotone<rational> for rise;
    impl Same<integer, rational> for (later, rise);
    ensure ok: "flip flips" from (flip::back, flip::pair) | (lamp) | (crate::geo);
}

examples {
    (geo::later::d = 1, lab::clock::t = 5) -> geo::later::t = 6,
    (geo::rise::h = 1.0) -> (geo::rise::v = 2.0, geo::rise2::v = 1.0),
}

tie geo::flip::back at "src/geo.rs/flip_back";
tie nowhere at "";
"#;

#[test]
fn every_form_comes_back() {
    let m = read(EVERY_FORM);
    round_trip(&m);
    // Ties: in the block of what they tie, in order, and at the end.
    let ties: Vec<String> = m
        .ties
        .iter()
        .map(|t| format!("{} at {}", t.item, t.at.path.join("/")))
        .collect();
    assert_eq!(
        ties,
        [
            "lab at lab",
            "lab at lab/annex",
            "Point::Point at src/geo.rs/Point",
            "geo at src/geo.rs",
            "geo::flip::back at src/geo.rs/flip_back",
            "nowhere at ",
        ]
    );
}

#[test]
fn short_paths_are_from_their_part() {
    let m = read(EVERY_FORM);
    let geo = m.parts.iter().find(|p| p.name == "geo").unwrap();
    assert_eq!(geo.requires[0].from, Source::Ensure(path("lab::lit")));
    // `crate::geo` is whole: a part, not geo's own require `geo`.
    assert_eq!(geo.requires[1].name, "geo");
    assert_eq!(
        geo.ensures[0].from,
        vec![
            vec![path("geo::flip::back"), path("geo::flip::pair")],
            vec![path("geo::lamp")],
            vec![path("geo")],
        ]
    );
    let rules = &geo.functions[0].rules;
    let again = rules[3].proof.as_ref().unwrap();
    // `let s = … in ponens(s, …)`: `s` is the name the let binds.
    let Expression::Apply(judgement::Op::Let, args) = again else {
        panic!("{again:?}")
    };
    assert_eq!(
        args[0].body,
        Expression::Name("geo::flip::same".to_string())
    );
    let Expression::Apply(judgement::Op::Ponens { t, post: 0 }, inner) = &args[1].body else {
        panic!()
    };
    assert_eq!(t, &Prop::named("s"));
    assert_eq!(inner[0].body, Expression::Name("s".to_string()));
    // A premise written in place is read by its place.
    let against = rules[4].proof.as_ref().unwrap();
    let Expression::Apply(
        judgement::Op::Tollens {
            post: 0, blame: 1, ..
        },
        read,
    ) = against
    else {
        panic!()
    };
    assert_eq!(read[1].body, Expression::Name("2".to_string()));
    // A record's type is as declared, a tuple's what it holds.
    let pair = &m.parts.iter().find(|p| p.name == "Pair").unwrap().records[0];
    let point = simulation::Record {
        args: vec![],
        name: "Point".to_string(),
        fields: vec![("x".to_string(), Type::Real), ("y".to_string(), Type::Real)],
    };
    assert_eq!(
        pair.fields,
        Some(vec![
            ("p".to_string(), Type::Record(point.clone())),
            ("n".to_string(), tuple(vec![Type::Int, Type::Truth])),
        ])
    );
    // `Point { y: …, x: … }` is made in the declared order.
    let flip = &geo.functions[0];
    let body = flip.body.as_ref().unwrap();
    let Expression::Apply(Op::Make(made), args) = body else {
        panic!()
    };
    assert_eq!(made, &point);
    assert_eq!(
        args[0].body,
        apply(
            Op::Field("y".to_string()),
            vec![Expression::Name("p".to_string())]
        )
    );
}

/// What the text gets wrong on purpose is found by `check`, not by `parse`.
#[test]
fn the_text_keeps_what_check_finds() {
    let m = read(EVERY_FORM);
    let codes: Vec<String> = check(&m)
        .iter()
        .map(|e| format!("{} at {}", e.code, e.at))
        .collect();
    assert!(codes.contains(&"M010 at nowhere".to_string()), "{codes:?}");
    assert!(codes.contains(&"M011 at lab::lit".to_string()), "{codes:?}");
}

/// Uses, calls, contracts, impls, examples of the model, and
/// propositions as rule terms read as the model layer writes them.
#[test]
fn uses_calls_and_contracts() {
    use mitra_core::model::{Implements, ModelExample, Use};
    let m = read(EVERY_FORM);
    let geo = m.parts.iter().find(|p| p.name == "geo").unwrap();
    assert_eq!(
        geo.uses,
        vec![
            Use::Names {
                part: "lab".to_string(),
                names: vec!["clock".to_string(), "dim".to_string()],
                from: None,
            },
            Use::Glob {
                part: "lab".to_string(),
                from: None,
            },
            Use::Names {
                part: "lab".to_string(),
                names: vec!["shine".to_string()],
                from: None,
            },
        ]
    );
    let later = geo.functions.iter().find(|f| f.name == "later").unwrap();
    let Some(Expression::Apply(Op::Add, args)) = &later.body else {
        panic!()
    };
    assert_eq!(
        args[0].body,
        apply(Op::Call("lab::clock".to_string()), vec![])
    );
    let rise = geo.functions.iter().find(|f| f.name == "rise").unwrap();
    let Some(Expression::Apply(Op::Mul, args)) = &rise.body else {
        panic!()
    };
    let Expression::Apply(Op::Call(f), _) = &args[0].body else {
        panic!()
    };
    assert_eq!(f, "geo::rise2");
    let same = &m.contracts[1];
    assert_eq!(same.params.len(), 2);
    let Expression::Apply(Op::And, both) = &same.claims[0].holds else {
        panic!()
    };
    let Expression::Apply(Op::Equal, eq) = &both[0].body else {
        panic!()
    };
    let Expression::Apply(Op::Call(g), _) = &eq[0].body else {
        panic!()
    };
    assert_eq!(g, "Same::g");
    assert_eq!(
        geo.implements[1],
        Implements {
            contract: "Same".to_string(),
            types: vec![Type::Int, Type::Real],
            functions: vec!["later".to_string(), "rise".to_string()],
        }
    );
    let int = |n: i64| Value::Int(n.into());
    assert_eq!(
        m.examples[0],
        ModelExample {
            within: None,
            inputs: vec![
                (path("geo::later::d"), int(1)),
                (path("lab::clock::t"), int(5))
            ],
            outputs: vec![(path("geo::later::t"), int(6))],
        }
    );
    let rule = &later.rules[0];
    let Term::Prop(Prop::All { body, .. }) = &rule.conclusion else {
        panic!("{:?}", rule.conclusion)
    };
    assert_eq!(
        **body,
        Prop::Correct {
            function: "geo::later".to_string(),
            at: judgement::Term::Var("x".to_string())
        }
    );
    let here = &later.rules[1];
    assert_eq!(
        here.premises[0],
        Term::Prop(Prop::named("geo::later::after"))
    );
    let codes: Vec<String> = check(&m)
        .iter()
        .map(|e| format!("{} at {}", e.code, e.at))
        .collect();
    assert!(
        !codes
            .iter()
            .any(|c| c.starts_with("M017") || c.starts_with("M018") || c.starts_with("M020")),
        "{codes:?}"
    );
}

/// An intro binds its point; a proposition about that point writes its set.
#[test]
fn intro_binds_its_point() {
    let m = read(EVERY_FORM);
    let rule = m.parts.iter().find(|p| p.name == "geo").unwrap().functions[0]
        .rules
        .iter()
        .find(|r| r.name == "any_point")
        .unwrap();
    let Some(Expression::Apply(judgement::Op::Intro { q }, args)) = &rule.proof else {
        panic!()
    };
    assert!(matches!(q, Prop::All { .. }));
    assert_eq!(args[0].binds, vec!["p".to_string()]);
    let Expression::Apply(judgement::Op::Instance { .. }, inner) = &args[0].body else {
        panic!()
    };
    assert_eq!(inner[1].body, Expression::Name("p".to_string()));
    let Expression::Apply(judgement::Op::Reductio { of }, _) = &inner[0].body else {
        panic!()
    };
    let Prop::Correct {
        at: judgement::Term::Arbitrary { name, over },
        ..
    } = of
    else {
        panic!("{of:?}")
    };
    assert_eq!(name, "x");
    assert!(over.such.is_some());
}

#[test]
fn sugar_reads_as_rust_writes_it() {
    // `a > b` is `b < a`, `a >= b` is `b <= a`, `a != b` is `!(a == b)`.
    let m = read("subject s { fn f(a: integer, b: integer) -> c: bool where { claim g: a > b && a >= b && a != b; } }");
    let holds = &m.parts[0].functions[0].claims[0].holds;
    let (a, b) = (
        Expression::Name("a".to_string()),
        Expression::Name("b".to_string()),
    );
    let less = apply(Op::Less, vec![b.clone(), a.clone()]);
    let most = apply(Op::AtMost, vec![b.clone(), a.clone()]);
    let differ = apply(Op::Not, vec![apply(Op::Equal, vec![a, b])]);
    let expected = apply(Op::And, vec![apply(Op::And, vec![less, most]), differ]);
    assert_eq!(holds, &expected);
    assert!(print(&m).contains("claim g: b < a && b <= a && a != b;"));
}

#[test]
fn numbers_read_and_print_exactly() {
    let r = |n: i64, d: i64| Rational::new(n, d).unwrap();
    assert_eq!(real_text(&r(981, 100)), "9.81");
    assert_eq!(real_text(&r(2, 1)), "2.0");
    assert_eq!(real_text(&r(-1, 2)), "-0.5");
    assert_eq!(real_text(&r(1, 3)), "1/3");
    assert_eq!(real_text(&r(-7, 6)), "-7/6");
    assert_eq!(real_text(&r(1, 1000)), "0.001");
    let m = Model {
        parts: vec![],
        contracts: vec![],
        examples: vec![],
        ties: vec![],
    };
    let (takes, gives) = parse_row("(1/3, -2, 0.25, true) -> (1, 2.5)", &m).unwrap();
    assert_eq!(
        takes,
        vec![
            Value::Real(r(1, 3)),
            Value::Int((-2).into()),
            Value::Real(r(1, 4)),
            Value::Truth(true)
        ]
    );
    let Value::Record(pair, _) = gives else {
        panic!()
    };
    assert_eq!(Type::Record(pair), tuple(vec![Type::Int, Type::Real]));
}

#[test]
fn a_proposition_reads_alone() {
    let m = read(&file("div/tied.mitra"));
    let text = "all x in (a: 0..10, b: 0..10) where b != 0: correct(calc::div, x)";
    let p = parse_prop(text, &m).unwrap();
    let Prop::All { name, over, body } = &p else {
        panic!()
    };
    assert_eq!(name, "x");
    let int = |n: i64| Value::Int(n.into());
    assert_eq!(
        over.ranges,
        vec![
            ("a".to_string(), int(0), int(10)),
            ("b".to_string(), int(0), int(10))
        ]
    );
    assert!(over.well_formed());
    assert_eq!(
        **body,
        Prop::Correct {
            function: "calc::div".to_string(),
            at: judgement::Term::Var("x".to_string())
        }
    );
    assert_eq!(print_prop(&p), text);
}

// ---------------------------------------------------------------- errors

fn fails(text: &str) -> (String, usize, usize, String) {
    let e = parse(text).expect_err("should not read");
    (e.code.to_string(), e.at.line, e.at.column, e.message)
}

#[test]
fn errors_say_where() {
    let (code, line, column, message) =
        fails("environment sun {\n    fn light() -> w: rational }\n");
    assert_eq!((code.as_str(), line, column), ("P003", 2, 31));
    assert_eq!(message, "expected `;`, found `}`");
    let text = "environment sun {\n    fn light() -> w: rational }\n";
    let e = parse(text).unwrap_err();
    assert_eq!(
        e.render("sun.mitra", text),
        "error[P003]: expected `;`, found `}`\n --> sun.mitra:2:31\n  |\n2 |     fn light() -> w: rational }\n  |                               ^\n"
    );
    assert_eq!(
        e.to_string(),
        "error[P003]: expected `;`, found `}`\n --> 2:31"
    );
    assert_eq!(fails("subject s { # }").0, P001.to_string());
    assert_eq!(
        fails("subject s { require a: \"open; }").0,
        P002.to_string()
    );
    assert_eq!(
        fails("subject s { fn f(x: integer) -> y: integer where { claim c: (x, zz).0 == x; } }").0,
        P004.to_string()
    );
    assert_eq!(
        fails("subject s { fn f() -> y: rational { 1/0 } }").0,
        P005.to_string()
    );
    assert_eq!(
        fails("subject s { fn f(x: integer) -> y: integer where { claim c: -x == 0; } }").0,
        P003.to_string()
    );
    assert_eq!(
        fails("subject s { fn f(x: integer) -> y: integer where { claim c: x < 1 < 2; } }").0,
        P003.to_string()
    );
    assert_eq!(
        fails("subject record R where { examples { () -> 1 } }\nsubject s { }").0,
        P003.to_string()
    );
    assert_eq!(fails("subject fn { }").3, "expected a name, found `fn`");
}

#[test]
fn names_take_a_colon() {
    // Every naming form reads `name: …`; `=` is not read, and no hint
    // speaks of it.
    for text in [
        "subject s { require a = \"open\"; }",
        "subject s { ensure e = \"open\"; }",
        "subject record R { x: integer } where { claim c = true; }\nsubject s { }",
        "subject s { fn f(x: integer) -> y: integer { x } where { claim c = true; } }",
        "contract C<T> { fn f(x: T) -> y: T; claim c(a: T) = f(a) == f(a); }",
    ] {
        let (code, _, _, message) = fails(text);
        assert_eq!(
            (code.as_str(), message.as_str()),
            ("P003", "expected `:`, found `=`"),
            "{text}"
        );
    }
    let m = read("subject s { require a: \"open\"; ensure e: \"shut\"; }");
    assert_eq!(m.parts[0].requires[0].name, "a");
    assert_eq!(m.parts[0].ensures[0].name, "e");
    round_trip(&m);
}

#[test]
fn a_proof_writes_no_state() {
    // `state(…)` and `&&` on states are judgement's own (finding 27): no
    // proof could give an opinion through them, so the text has neither,
    // and no hint speaks of them.
    let rule = |by: &str| {
        format!("subject s {{ fn f(x: integer) -> y: bool {{ x < 3 }} where {{ claim c: y; rule r: s::f::c |- s::f::c by {by}; }} }}")
    };
    let (code, _, _, message) = fails(&rule("state(s::f::c)"));
    assert_eq!(
        (code.as_str(), message.as_str()),
        ("P003", "expected `;`, found `(`")
    );
    let (code, _, _, message) = fails(&rule("s::f::c && s::f::c"));
    assert_eq!(
        (code.as_str(), message.as_str()),
        ("P003", "expected `;`, found `&&`")
    );
    // `state` is no word of the language any more: a premise may be named so.
    let m = read(&rule("let state = s::f::c in state"));
    round_trip(&m);
    // Built from Rust, they print as no text.
    let r = judgement::Op::StateOf;
    let e = apply(r, vec![Expression::Name("s::f::c".to_string())]);
    let mut m = read(&rule("s::f::c"));
    m.parts[0].functions[0].rules[0].proof = Some(e);
    assert!(print(&m).contains("__malformed__"), "{}", print(&m));
}

// ---------------------------------------------------------------- random expressions

/// A random expression on `a: Int`, `r: Real`, `t: bool`: every
/// operation of simulation, tuples of what checks, never a named record.
fn expression(depth: u32) -> BoxedStrategy<E> {
    let int = (-20i64..20).prop_map(|n| apply(Op::Int(n.into()), vec![]));
    let real = (-50i64..50, 1i64..13)
        .prop_map(|(n, d)| apply(Op::Real(Rational::new(n, d).unwrap()), vec![]));
    let truth = any::<bool>().prop_map(|b| apply(Op::Truth(b), vec![]));
    let name = prop_oneof![Just("a"), Just("r"), Just("t"), Just("x")]
        .prop_map(|n| Expression::Name(n.to_string()));
    let leaf = prop_oneof![int, real, truth, name].boxed();
    leaf.prop_recursive(depth, 64, 4, |inner| {
        let two = |op: Op, inner: BoxedStrategy<E>| {
            (inner.clone(), inner).prop_map(move |(a, b)| apply(op.clone(), vec![a, b]))
        };
        prop_oneof![
            inner.clone().prop_map(|a| apply(Op::Not, vec![a])),
            two(Op::And, inner.clone()),
            two(Op::Or, inner.clone()),
            two(Op::Add, inner.clone()),
            two(Op::Sub, inner.clone()),
            two(Op::Mul, inner.clone()),
            two(Op::Div, inner.clone()),
            two(Op::Rem, inner.clone()),
            two(Op::Less, inner.clone()),
            two(Op::AtMost, inner.clone()),
            two(Op::Equal, inner.clone()),
            (inner.clone(), inner.clone(), inner.clone())
                .prop_map(|(a, b, c)| apply(Op::Within, vec![a, b, c])),
            (inner.clone(), inner.clone(), inner.clone())
                .prop_map(|(a, b, c)| apply(Op::If, vec![a, b, c])),
            (inner.clone(), inner.clone()).prop_map(|(a, b)| Expression::Apply(
                Op::Let,
                vec![
                    Arg {
                        binds: vec![],
                        body: a
                    },
                    Arg {
                        binds: vec!["x".to_string()],
                        body: b
                    }
                ]
            )),
            (inner.clone(), inner.clone(), inner.clone()).prop_map(|(a, b, c)| Expression::Apply(
                Op::All,
                vec![
                    Arg {
                        binds: vec![],
                        body: a
                    },
                    Arg {
                        binds: vec![],
                        body: b
                    },
                    Arg {
                        binds: vec!["x".to_string()],
                        body: c
                    }
                ]
            )),
            // A tuple of constants, read back by a field.
            (-5i64..5, any::<bool>(), 0usize..2).prop_map(|(n, b, i)| {
                let items = vec![
                    apply(Op::Int(n.into()), vec![]),
                    apply(Op::Truth(b), vec![]),
                ];
                let Type::Record(r) = tuple(vec![Type::Int, Type::Truth]) else {
                    unreachable!()
                };
                apply(Op::Field(i.to_string()), vec![apply(Op::Make(r), items)])
            }),
        ]
    })
    .boxed()
}

fn in_a_model(holds: E) -> Model {
    let f = Function {
        name: "f".to_string(),
        takes: vec![
            ("a".to_string(), Type::Int),
            ("r".to_string(), Type::Real),
            ("t".to_string(), Type::Truth),
        ],
        gives: ("g".to_string(), Type::Int),
        body: None,
        examples: vec![],
        claims: vec![claim("c", holds)],
        rules: vec![],
        ranges: Default::default(),
    };
    Model {
        parts: vec![Part {
            inner: vec![],
            file: None,
            enums: vec![],
            name: "s".to_string(),
            kind: Kind::Subject,
            uses: vec![],
            implements: vec![],
            records: vec![],
            functions: vec![f],
            requires: vec![],
            ensures: vec![],
        }],
        contracts: vec![],
        examples: vec![],
        ties: vec![],
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]
    #[test]
    fn random_expressions_come_back(e in expression(6)) {
        let m = in_a_model(e);
        let text = print(&m);
        let again = parse(&text).map_err(|e| e.render("random", &text));
        prop_assert_eq!(again, Ok(m), "{}", text);
    }
}

/// Ties written apart come back in the blocks of what they tie: the
/// same ties, in another order.
#[test]
fn ties_come_back_in_their_blocks() {
    let mut m = fall();
    m.ties.insert(
        0,
        Tie {
            item: path("sun"),
            at: Location::at(&["sky"]),
        },
    );
    m.ties.push(Tie {
        item: path("stone::air"),
        at: Location::at(&["lab"]),
    });
    let again = read(&print(&m));
    assert_eq!(again.ties.len(), m.ties.len());
    for t in &m.ties {
        assert!(again.ties.contains(t));
    }
    assert!(print(&m).contains("environment sun at \"sky\" {"));
    assert!(print(&m).ends_with("tie stone::air at \"lab\";\n"));
    let _ = (
        Set {
            ranges: vec![],
            such: None,
        },
        Record {
            params: vec![],
            name: String::new(),
            fields: None,
            claims: vec![],
            rules: vec![],
        },
    );
}
