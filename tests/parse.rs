//! The text of a model (`docs/syntax.md`): `parse` reads what `print`
//! writes. The fall and div of `models/` parse, check, and come back the
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
        name: "sun".to_string(),
        kind: Kind::Environment,
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
        name: "stone".to_string(),
        kind: Kind::Subject,
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
        ties: vec![Tie {
            item: path("stone::drop"),
            at: Location::at(&["lab", "tower", "stone"]),
        }],
    }
}

// ---------------------------------------------------------------- the models of `models/`

fn file(name: &str) -> String {
    let path = format!("{}/models/{name}", env!("CARGO_MANIFEST_DIR"));
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
    let m = read(&file("fall.mitra"));
    assert_eq!(m, fall());
    assert_eq!(check(&m), vec![]);
    round_trip(&m);
}

#[test]
fn div_reads_checks_and_comes_back() {
    let m = read(&file("div.mitra"));
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
    fn light() -> w: Real;

    ensure shines = "at least 200 W/m² at noon";
}

subject stone {
    require light = sun::shines;
    require air = "still air";

    fn drop(h: Real) -> v: Real at "lab/tower/stone" where {
        claim falls = within(v * v, 2.0 * 9.81 * h, 2.0);
        claim slow = !(h <= 20.0) || v < 21.0;
        rule slows: drop::falls, speed::slow |- drop::slow;
        rule lands: drop::slows, drop::falls, speed::slow |- drop::slow
            by ponens(drop::slows, drop::falls, speed::slow);
    }

    fn speed(h: Real) -> v2: Real { 2.0 * 9.81 * h } where {
        claim slow = !(h <= 20.0) || v2 < 400.0;
        examples { (5.0) -> 98.1 }
    }

    ensure lands_slow = "lands slowly" from (speed::slow, drop::falls);
}
"#;
    assert_eq!(print(&fall()), expected);
}

// ---------------------------------------------------------------- every form

/// Every form the language has, once at least.
const EVERY_FORM: &str = r#"
environment lab at "lab", "lab/annex" {
    record Photon;
    record Beam where {
        claim lit = true;
    }
    fn clock() -> t: Int;
    ensure lit = "the lamp is on" from ();
}

subject geo at "src/geo.rs" {
    require lamp = lab::lit;
    require geo = "a require named as its part";
    record Point at "src/geo.rs/Point" { x: Real, y: Real } where {
        claim right = 0.0 <= self.x && self.y >= -1/3;
        rule upper: Point::right, { self.x < 10.0 } |- Point::right;
    }
    record Empty {}
    record Pair { p: Point, n: (Int, bool) }
    fn flip(p: Point) -> q: Point { Point { y: p.x, x: p.y } } where {
        claim back = q.x == p.y && q.y != p.x;
        claim pair = (p.x, 1, true).0 == p.x;
        claim one = (3,).0 > 2;
        claim unit = () == ();
        claim cond = if (Point { x: 0.0, y: 0.0 }) == p { true } else { q.x - -2.5 <= 1/7 };
        claim let_ = let d = q.x - p.y in let e = d * d in e == 0.0 || e / 2.0 < 1.0;
        claim nested = if p.x < 0.0 { false } else if p.y < 0.0 { false } else { true };
        claim sweep = (0..10).all(|i| (i..i + 1).all(|j| j - i == 0));
        claim tol = within(p.x, q.y, 0.001);
        claim paren = (1 + 2) * 3 == 9 && !(1 < 2 || false) == false;
        rule same: flip::back |- flip::pair;
        rule nothing: |- { true };
        rule shown: flip::same, flip::back |- flip::pair
            by ponens[flip::same](flip::same, flip::back);
        rule again: flip::same, flip::back |- flip::pair
            by let s = flip::same in ponens(s, flip::back);
        rule against: flip::same, flip::pair, { false } |- flip::back
            by tollens[flip::same, 0, 1](flip::pair, 2) && !state(join(flip::same, flip::same));
        rule rest: flip::back |- flip::pair by reductio[!flip::back](flip::back);
        rule over: flip::back |- flip::pair
            by cases[all x in (a: 0..2, b: -1..1) where b != 0: correct(flip, x)](flip::back, flip::back);
        rule points: flip::back |- flip::pair
            by instance[any x in (a: 0.0..1.0): {flip::back, lab::lit} flip {correct(flip, (Point { x: 0.5, y: 1.0 }))}, (0.5)](witness[all x in (n: 0..1): flip::back, (0)](flip::back));
        examples {
            (Point { y: 2.0, x: 1.0 }) -> Point { x: 2.0, y: 1.0 },
            (Point { x: 0.0, y: 0.0 }) -> Point { x: 0.0, y: 0.0 },
        }
    }
    fn pair(a: Int, b: (Int, bool)) -> c: Pair;
    fn constant() -> c: (Real,) { (1.5,) }
    ensure ok = "flip flips" from (flip::back, flip::pair) | (lamp) | (crate::geo);
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
            "geo at src/geo.rs",
            "geo::Point at src/geo.rs/Point",
            "geo::flip::back at src/geo.rs/flip_back",
            "nowhere at ",
        ]
    );
}

#[test]
fn short_paths_are_from_their_part() {
    let m = read(EVERY_FORM);
    let geo = &m.parts[1];
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
    let Expression::Apply(judgement::Op::And, args) = against else {
        panic!()
    };
    let Expression::Apply(
        judgement::Op::Tollens {
            post: 0, blame: 1, ..
        },
        read,
    ) = &args[0].body
    else {
        panic!()
    };
    assert_eq!(read[1].body, Expression::Name("2".to_string()));
    // A record's type is as declared, a tuple's what it holds.
    let pair = &geo.records[2];
    let point = simulation::Record {
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

#[test]
fn sugar_reads_as_rust_writes_it() {
    // `a > b` is `b < a`, `a >= b` is `b <= a`, `a != b` is `!(a == b)`.
    let m = read("subject s { fn f(a: Int, b: Int) -> c: bool where { claim g = a > b && a >= b && a != b; } }");
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
    assert!(print(&m).contains("claim g = b < a && b <= a && a != b;"));
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
    let m = read(&file("div.mitra"));
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
    let (code, line, column, message) = fails("environment sun {\n    fn light() -> w: Real }\n");
    assert_eq!((code.as_str(), line, column), ("P003", 2, 27));
    assert_eq!(message, "expected `;`, found `}`");
    let text = "environment sun {\n    fn light() -> w: Real }\n";
    let e = parse(text).unwrap_err();
    assert_eq!(
        e.render("sun.mitra", text),
        "error[P003]: expected `;`, found `}`\n --> sun.mitra:2:27\n  |\n2 |     fn light() -> w: Real }\n  |                           ^\n"
    );
    assert_eq!(
        e.to_string(),
        "error[P003]: expected `;`, found `}`\n --> 2:27"
    );
    assert_eq!(fails("subject s { # }").0, P001.to_string());
    assert_eq!(
        fails("subject s { require a = \"open; }").0,
        P002.to_string()
    );
    assert_eq!(
        fails("subject s { fn f(x: Int) -> y: Int where { claim c = (x, zz).0 == x; } }").0,
        P004.to_string()
    );
    assert_eq!(
        fails("subject s { fn f() -> y: Real { 1/0 } }").0,
        P005.to_string()
    );
    assert_eq!(
        fails("subject s { fn f(x: Int) -> y: Int where { claim c = -x == 0; } }").0,
        P003.to_string()
    );
    assert_eq!(
        fails("subject s { fn f(x: Int) -> y: Int where { claim c = x < 1 < 2; } }").0,
        P003.to_string()
    );
    assert_eq!(
        fails("subject s { record R where { examples { () -> 1 } } }").0,
        P003.to_string()
    );
    assert_eq!(fails("subject fn { }").3, "expected a name, found `fn`");
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
    };
    Model {
        parts: vec![Part {
            name: "s".to_string(),
            kind: Kind::Subject,
            records: vec![],
            functions: vec![f],
            requires: vec![],
            ensures: vec![],
        }],
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
            name: String::new(),
            fields: None,
            claims: vec![],
            rules: vec![],
        },
    );
}
