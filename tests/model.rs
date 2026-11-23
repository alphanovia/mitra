//! The model on the fall (`docs/model.md`). The stone, in the system,
//! falls; the sun, outside it, gives light. The model is checked, read
//! for its levels, propositions and leaves, judged on takings, and its claims
//! run in simulation, all on `expression` as it is.

mod common;

use common::{apply, let_in};
use mitra_core::catalogue::{Suit, CATALOGUE};
use mitra_core::check::check;
use mitra_core::expression::{eval, Expression};
use mitra_core::judgement::{self, Opinion, Prop, State, Taking};
use mitra_core::location::Location;
use mitra_core::model::{
    calls, contract_claims, judgement, leaves, levels, props, Claim, Contract, Ensure, Example,
    Function, Implements, Kind, Law, Level, Model, ModelExample, Part, Path, Record, Require, Rule,
    Signature, Source, Term, Tie, Use,
};
use mitra_core::simulation::{
    self, tuple, Bound, Bounds, Exact, Op, Ranges, Rational, Type, Value,
};
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

type E = Expression<Op>;

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

#[test]
fn the_fall_is_well_formed() {
    assert_eq!(check(&fall()), vec![]);
}

/// The errors of a model, each as its code and where: `M002 at a::b`.
fn errors(model: &Model) -> Vec<String> {
    let errors = check(model).into_iter();
    errors.map(|e| format!("{} at {}", e.code, e.at)).collect()
}

#[test]
fn levels_are_read_not_declared() {
    let expected = vec![
        (path("sun::light"), Level::Opaque),
        (path("stone::drop"), Level::Partial),
        (path("stone::speed"), Level::Total),
    ];
    assert_eq!(levels(&fall()), expected);
}

/// `{pre} about {post}`, about `about`.
fn implies(about: &str, pre: &[&str], post: &str) -> (Path, Prop) {
    let prop = Prop::Implies {
        pre: pre.iter().map(|p| Prop::named(p)).collect(),
        subject: about.to_string(),
        post: vec![Prop::named(post)],
    };
    (path(about), prop)
}
fn named(p: &str) -> (Path, Prop) {
    (path(p), Prop::named(p))
}

/// Each proposition is named by its path, a rule too: what a rule says,
/// judgement knows by that name.
#[test]
fn each_proposition_is_about_its_path() {
    let expected = vec![
        named("stone::air"),
        named("stone::drop::falls"),
        named("stone::drop::lands"),
        named("stone::drop::slow"),
        named("stone::drop::slows"),
        named("stone::lands_slow"),
        implies(
            "stone::lands_slow",
            &["stone::speed::slow", "stone::drop::falls"],
            "stone::lands_slow",
        ),
        named("stone::light"),
        implies("stone::light", &["sun::shines"], "stone::light"),
        named("stone::speed::slow"),
        named("sun::shines"),
        (
            path("stone::drop"),
            Prop::named("stone::drop at lab/tower/stone"),
        ),
    ];
    assert_eq!(props(&fall()), expected);
    let known = judgement(&fall());
    let names = |ps: &[&str]| ps.iter().map(|p| Prop::named(p)).collect::<Vec<_>>();
    let (falls, speed, slow) = (
        "stone::drop::falls",
        "stone::speed::slow",
        "stone::drop::slow",
    );
    let lands = Prop::named("stone::drop::lands");
    let (slows_pre, lands_pre) = (
        names(&["stone::drop::slows", falls, speed]),
        names(&["stone::drop::lands", "stone::drop::slows", falls, speed]),
    );
    assert_eq!(known.premises(&slows()), Some(slows_pre));
    assert_eq!(known.premises(&lands), Some(lands_pre));
    assert_eq!(known.posts(&lands), Some(&names(&[slow])[..]));
    assert_eq!(known.premises(&Prop::named(falls)), None);
}

/// What only a taking holds: the hypothesis `air`, what the sun
/// ensures, the claims, the assumed rule, what binds `light` to the
/// sun and gives `lands_slow` from its list, and the tie. Not `light`,
/// `lands_slow` or `drop::slow`, which follow, nor `lands`, which is
/// shown.
#[test]
fn leaves_are_what_only_a_taking_holds() {
    let expected = vec![
        named("stone::air"),
        named("stone::drop::falls"),
        named("stone::drop::lands"),
        named("stone::drop::slows"),
        implies(
            "stone::lands_slow",
            &["stone::speed::slow", "stone::drop::falls"],
            "stone::lands_slow",
        ),
        implies("stone::light", &["sun::shines"], "stone::light"),
        named("stone::speed::slow"),
        named("sun::shines"),
        (
            path("stone::drop"),
            Prop::named("stone::drop at lab/tower/stone"),
        ),
    ];
    assert_eq!(leaves(&fall()), expected);
}

/// An opinion of `prop`, taken true once by `by`.
fn taken(prop: Prop, by: &str) -> judgement::Value {
    let taking = Taking {
        prop: prop.clone(),
        value: true,
        count: 1,
        by: by.to_string(),
    };
    judgement::Value::Opinion(Opinion::taken(prop, vec![taking]).unwrap())
}
fn opinion<F: std::fmt::Debug>(v: Result<judgement::Value, F>) -> Opinion {
    match v {
        Ok(judgement::Value::Opinion(o)) => o,
        other => panic!("{other:?}"),
    }
}

/// Judgement reads the propositions as they come: `lands_slow` follows from
/// its list, taken by a reviewer, and the claims, taken by a machine and
/// a timer. And the proof of `lands` gives `drop::slow`, on takings of
/// its premises.
#[test]
fn the_fall_is_judged_on_takings() {
    let props = props(&fall());
    let (_, list) = implies(
        "stone::lands_slow",
        &["stone::speed::slow", "stone::drop::falls"],
        "stone::lands_slow",
    );
    assert!(props.iter().any(|(_, p)| *p == list));
    let values = BTreeMap::from([
        ("t".to_string(), taken(list.clone(), "human:reviewer")),
        (
            "slow".to_string(),
            taken(Prop::named("stone::speed::slow"), "machine:ranges"),
        ),
        (
            "falls".to_string(),
            taken(Prop::named("stone::drop::falls"), "machine:timer"),
        ),
    ]);
    let args = ["t", "slow", "falls"]
        .iter()
        .map(|n| Expression::Name(n.to_string()));
    let ponens = apply(judgement::Op::Ponens { t: list, post: 0 }, args.collect());
    let known = judgement(&fall());
    let lands_slow = opinion(eval(&known, &values, &ponens));
    assert_eq!(lands_slow.about(), &Prop::named("stone::lands_slow"));
    assert_eq!(lands_slow.state(), State::True);

    let model = fall();
    let lands = &model.parts[1].functions[0].rules[1];
    let values = BTreeMap::from([
        (
            "stone::drop::slows".to_string(),
            taken(slows(), "human:reviewer"),
        ),
        (
            "stone::drop::falls".to_string(),
            taken(Prop::named("stone::drop::falls"), "machine:timer"),
        ),
        (
            "stone::speed::slow".to_string(),
            taken(Prop::named("stone::speed::slow"), "machine:ranges"),
        ),
    ]);
    let slow = opinion(eval(&known, &values, lands.proof.as_ref().unwrap()));
    assert_eq!(slow.about(), &Prop::named("stone::drop::slow"));
    assert_eq!(slow.state(), State::True);

    // A run sees the stone land fast. Modus tollens on `slows` blames
    // the assumed rule itself, on the run and the two other takings.
    let mut values = values;
    let run = Taking {
        prop: Prop::named("stone::drop::slow"),
        value: false,
        count: 1,
        by: "machine:run".to_string(),
    };
    let fast = Opinion::taken(run.prop.clone(), vec![run]).unwrap();
    values.insert("fast".to_string(), judgement::Value::Opinion(fast));
    let tollens = judgement::Op::Tollens {
        t: slows(),
        post: 0,
        blame: 0,
    };
    let reads = ["fast", "stone::drop::falls", "stone::speed::slow"];
    let blamed = apply(tollens, reads.iter().map(|n| read(n)).collect());
    let blamed = opinion(eval(&known, &values, &blamed));
    assert_eq!(blamed.about(), &slows());
    assert_eq!(blamed.state(), State::False);
    let rest = blamed.rests_against().first().unwrap();
    let by: BTreeSet<&str> = rest.iter().map(|t| t.by.as_str()).collect();
    assert_eq!(
        by,
        BTreeSet::from(["machine:run", "machine:timer", "machine:ranges"])
    );
}

/// What the leaves hold, through the rules judgement knows: a rule and
/// its pres held give its posts.
fn held(model: &Model) -> BTreeSet<Prop> {
    let known = judgement(model);
    let all: Vec<Prop> = props(model).into_iter().map(|(_, p)| p).collect();
    let mut held: BTreeSet<Prop> = leaves(model).into_iter().map(|(_, p)| p).collect();
    loop {
        let before = held.len();
        for t in &all {
            if let (Some(premises), Some(posts)) = (known.premises(t), known.posts(t)) {
                if premises.iter().all(|p| held.contains(p)) {
                    held.extend(posts.iter().cloned());
                }
            }
        }
        if held.len() == before {
            return held;
        }
    }
}

/// The leaves of a well-formed model hold all of it. A proof concludes
/// nothing in the model: the checker takes the rule it shows, so a
/// proved rule is a leaf, and what uses it is held: here `gentle`, given
/// by `soft: lands, drop::slow |- drop::gentle` from the proved `lands`.
/// And in the fall with a rule given by a rule.
#[test]
fn the_leaves_hold_everything() {
    let mut model = fall();
    let drop = &mut model.parts[1].functions[0];
    drop.claims.push(claim("gentle", under("v", 30)));
    let premises = ["stone::drop::lands", "stone::drop::slow"];
    drop.rules
        .push(rule("soft", &premises, "stone::drop::gentle"));
    assert_eq!(check(&model), vec![]);
    let lands = (
        path("stone::drop::lands"),
        Prop::named("stone::drop::lands"),
    );
    assert!(leaves(&model).contains(&lands));
    let mut law = fall();
    with_law(&mut law);
    for model in [fall(), model, law] {
        let all: BTreeSet<Prop> = props(&model).into_iter().map(|(_, p)| p).collect();
        assert_eq!(held(&model), all);
    }
}

/// A claim of a function with a body runs on it: `speed::slow` holds
/// for every height up to 20 m, by bounds. A claim of an opaque one
/// runs on a measurement: `drop::falls` holds at 5 m and 9.9 m/s.
#[test]
fn claims_run_in_simulation() {
    let model = fall();
    let (drop, speed) = (&model.parts[1].functions[0], &model.parts[1].functions[1]);
    let body = speed.body.clone().unwrap();
    let slow = let_in(Op::Let, "v2", body, speed.claims[0].holds.clone());
    let low = Bounds::between(
        mitra_core::simulation::Numbers::Real,
        Rational::integer(0),
        Rational::integer(20),
    )
    .unwrap();
    let heights = BTreeMap::from([("h".to_string(), low)]);
    assert_eq!(
        eval(&Ranges::default(), &heights, &slow).unwrap().surely(),
        Some(true)
    );
    let measured = BTreeMap::from([
        ("h".to_string(), Value::Real(Rational::integer(5))),
        ("v".to_string(), Value::Real(Rational::new(99, 10).unwrap())),
    ]);
    assert_eq!(
        eval(&Exact::default(), &measured, &drop.claims[0].holds),
        Ok(Value::Truth(true))
    );
}

/// A record's claims read it as `self`. A tuple is a record with no
/// name, its fields `0`, `1`, … in order.
#[test]
fn records_and_tuples() {
    let mut model = fall();
    let h = apply(Op::Field("h".to_string()), vec![name("self")]);
    model.parts[1].records = vec![
        Record {
            name: "Drop".to_string(),
            fields: Some(vec![
                ("h".to_string(), Type::Real),
                ("v".to_string(), Type::Real),
            ]),
            claims: vec![claim("down", apply(Op::AtMost, vec![real(0), h]))],
            rules: vec![],
        },
        Record {
            name: "Weather".to_string(),
            fields: None,
            claims: vec![],
            rules: vec![],
        },
    ];
    let pair = tuple(vec![Type::Real, Type::Real]);
    let mut both = function("both", "h", ("o", pair.clone()));
    let first = apply(Op::Field("0".to_string()), vec![name("o")]);
    both.claims = vec![claim("first", apply(Op::Equal, vec![first, name("h")]))];
    model.parts[1].functions.push(both);
    assert_eq!(check(&model), vec![]);
    let levels = levels(&model);
    assert!(levels.contains(&(path("stone::Drop"), Level::Total)));
    assert!(levels.contains(&(path("stone::Weather"), Level::Opaque)));
    assert!(levels.contains(&(path("stone::both"), Level::Partial)));
    let Type::Record(mut wrong) = pair else {
        unreachable!()
    };
    wrong.fields[1].0 = "x".to_string();
    model.parts[1].functions[2].gives.1 = Type::Record(wrong);
    assert_eq!(errors(&model), ["M003 at stone::both::o"]);
}

/// The errors of the fall with one change. Reading its propositions and
/// leaves ends all the same.
fn changed(change: impl FnOnce(&mut Model)) -> Vec<String> {
    let mut model = fall();
    change(&mut model);
    let _ = (
        props(&model),
        leaves(&model),
        levels(&model),
        judgement(&model),
    );
    errors(&model)
}

/// A change to a model.
type Change = Box<dyn FnOnce(&mut Model)>;

/// Each code of the model is given, where its declaration is, by the
/// fall with one change. The cases give every code the catalogue lists
/// for the model.
#[test]
fn each_error_is_found() {
    let ghost = |fields: &[&str]| simulation::Record {
        name: "Ghost".to_string(),
        fields: fields.iter().map(|f| (f.to_string(), Type::Real)).collect(),
    };
    let cases: Vec<(&str, Change)> = vec![
        (
            "M001 at stone::still air",
            Box::new(|m| m.parts[1].requires[1].name = "still air".to_string()),
        ),
        (
            "M002 at stone::drop::falls",
            Box::new(|m| {
                let falls = m.parts[1].functions[0].claims[0].clone();
                m.parts[1].functions[0].claims.push(falls);
            }),
        ),
        (
            "M003 at stone::Drop::h",
            Box::new(move |m| {
                m.parts[1].records.push(Record {
                    name: "Drop".to_string(),
                    fields: Some(vec![("h".to_string(), Type::Record(ghost(&[])))]),
                    claims: vec![],
                    rules: vec![],
                })
            }),
        ),
        (
            "M004 at stone::speed::slow",
            Box::new(move |m| {
                let made = apply(Op::Make(ghost(&["h"])), vec![real(1)]);
                let slow = &mut m.parts[1].functions[1].claims[0].holds;
                *slow = let_in(Op::Let, "made", made, slow.clone());
            }),
        ),
        (
            "M005 at stone::speed",
            Box::new(|m| m.parts[1].functions[1].body = Some(under("h", 1))),
        ),
        (
            "M006 at stone::drop::falls",
            Box::new(|m| m.parts[1].functions[0].claims[0].holds = name("v")),
        ),
        (
            "M007 at stone::drop::slows",
            Box::new(|m| m.parts[1].functions[0].rules[0].premises[0] = at("stone::drop::fell")),
        ),
        (
            "M008 at stone::drop::shining",
            Box::new(|m| {
                let shining = rule("shining", &["stone::air"], "sun::shines");
                m.parts[1].functions[0].rules.push(shining);
            }),
        ),
        (
            "M009 at stone::light",
            Box::new(|m| m.parts[1].requires[0].from = Source::Ensure(path("sun::light"))),
        ),
        (
            "M010 at stone::drop::falls",
            Box::new(|m| m.ties[0].item = path("stone::drop::falls")),
        ),
        (
            "M011 at stone::lands_slow",
            Box::new(|m| m.parts[1].ensures[0].from.push(vec![])),
        ),
        (
            "M012 at stone::drop::lands",
            Box::new(|m| m.parts[1].functions[0].rules[1].conclusion = at("stone::air")),
        ),
        (
            "M013 at stone::lands_slow",
            Box::new(|m| {
                let itself = vec![path("stone::lands_slow")];
                m.parts[1].ensures[0].from.push(itself);
            }),
        ),
        (
            "M014 at stone::drop::lands",
            Box::new(|m| {
                let lands = at("stone::drop::lands");
                m.parts[1].functions[0].rules[1].premises.push(lands);
            }),
        ),
        (
            "M015 at stone::speed",
            Box::new(|m| {
                let row = Example {
                    takes: vec![Value::Truth(true)],
                    gives: Value::Real(Rational::integer(0)),
                };
                m.parts[1].functions[1].examples.push(row);
            }),
        ),
        (
            "M016 at stone",
            Box::new(|m| m.parts[1].uses.push(uses("sun", &["lamp"]))),
        ),
        (
            "M017 at stone::speed",
            Box::new(|m| m.parts[1].functions[1].body = Some(times_light("sun::lamp"))),
        ),
        (
            "M018 at stone::speed",
            Box::new(|m| m.parts[1].functions[1].body = Some(times_light("sun::light"))),
        ),
        (
            "M019 at stone::speed",
            Box::new(|m| {
                m.parts[1].functions[1].body = Some(call("stone::speed", vec![name("h")]))
            }),
        ),
        (
            "M020 at stone::speed::Monotone",
            Box::new(|m| {
                m.parts[1]
                    .implements
                    .push(fits("Monotone", Type::Real, "speed"))
            }),
        ),
        (
            "M021 at examples::0",
            Box::new(|m| m.examples.push(example(&[], &[("stone::speed::h", 5)]))),
        ),
        (
            "M022 at examples::0",
            Box::new(|m| m.examples.push(example(&[], &[("stone::speed::v2", 98)]))),
        ),
        (
            "M023 at stone::drop::everywhere",
            Box::new(|m| {
                let mut everywhere = rule("everywhere", &[], "stone::drop::slow");
                everywhere.conclusion = Term::Prop(correct("stone::drop", Some(vec![])));
                m.parts[1].functions[0].rules.push(everywhere);
            }),
        ),
        (
            "M024 at stone::speed",
            Box::new(|m| {
                let at = m.parts[1].functions[1].examples[0].takes.clone();
                m.parts[1].functions[1].examples.push(Example {
                    takes: at,
                    gives: Value::Real(Rational::integer(99)),
                })
            }),
        ),
    ];
    let mut given = Vec::new();
    for (expected, change) in cases {
        assert_eq!(changed(change), [expected]);
        given.push(expected[..4].to_string());
    }
    // Range types (finding 11), on a model of whole numbers.
    let digits = |text: &str| -> Vec<String> {
        let model = mitra_core::parse::parse(text).unwrap();
        check(&model).iter().map(|e| e.to_string()).collect()
    };
    assert_eq!(
        digits("subject d {\n    fn f(a: 9..=0) -> b: Int { a }\n}"),
        ["error[M025]: `a: 9..=0` holds no number: a range is written low end first, `0..=9`\n --> d::f"]
    );
    assert_eq!(
        digits("subject d {\n    fn f(a: 0..=9) -> b: 0..=9 where { examples { (12) -> 1, (1) -> 10 } }\n}"),
        [
            "error[M026]: example 0 has `a` = 12, outside its range 0..=9\n --> d::f",
            "error[M026]: example 1 has `b` = 10, outside its range 0..=9\n --> d::f",
        ]
    );
    assert_eq!(
        digits("subject d {\n    fn f(a: 0..=9) -> b: Int { a }\n}\nexamples { (d::f::a = 10) -> d::f::b = 10 }"),
        ["error[M026]: `d::f::a` = 10, outside its range 0..=9\n --> examples::0"]
    );
    given.extend(["M025".to_string(), "M026".to_string()]);
    let listed = CATALOGUE
        .iter()
        .filter(|e| e.code.suit == Suit::Model && !e.retired);
    let listed: Vec<String> = listed.map(|e| e.code.to_string()).collect();
    assert_eq!(given, listed);
    // A term that names a function is no proposition.
    let function = changed(|m| m.parts[1].functions[0].rules[0].premises[0] = at("stone::speed"));
    assert_eq!(function, ["M007 at stone::drop::slows"]);
    // Two records named alike in two parts: a record's name is once in
    // the model. A name twice among what a function takes.
    let records = changed(|m| {
        let calm = Record {
            name: "Calm".to_string(),
            fields: Some(vec![]),
            claims: vec![],
            rules: vec![],
        };
        m.parts[0].records.push(calm.clone());
        m.parts[1].records.push(calm);
    });
    assert_eq!(records, ["M002 at stone::Calm"]);
    let takes = changed(|m| {
        let h = m.parts[1].functions[0].takes[0].clone();
        m.parts[1].functions[0].takes.push(h);
    });
    assert_eq!(takes, ["M002 at stone::drop::h"]);
    // An expression written in a rule is read as a claim.
    let holds = changed(|m| {
        let mut low = rule("low", &[], "stone::drop::slow");
        low.premises.push(Term::Holds(name("h")));
        m.parts[1].functions[0].rules.push(low);
    });
    assert_eq!(holds, ["M006 at stone::drop::low"]);
    // A function that takes a type no record declares: its body is not
    // checked against it too.
    let takes = changed(move |m| m.parts[1].functions[1].takes[0].1 = Type::Record(ghost(&[])));
    assert_eq!(takes, ["M003 at stone::speed::h"]);
}

/// A name in a proof.
fn read(x: &str) -> Expression<judgement::Op> {
    Expression::Name(x.to_string())
}
/// `name = [premises] => conclusion`, assumed.
fn rule(name: &str, premises: &[&str], conclusion: &str) -> Rule {
    Rule {
        name: name.to_string(),
        premises: premises.iter().map(|p| at(p)).collect(),
        conclusion: at(conclusion),
        proof: None,
    }
}

/// A proof shows its rule only when it argues for the conclusion. The
/// type of an opinion names its proposition, not its side: reductio and
/// modus tollens give opinions of a proposition that never hold it true. So
/// each proof is also run once, on what it reads taken true, and must
/// give the conclusion true. A rule whose proof does not is not shown:
/// M012, and the checker does not take it.
#[test]
fn a_proof_argues_for_its_conclusion() {
    let lands = path("stone::drop::lands");
    let mut model = fall();
    let reductio = judgement::Op::Reductio {
        of: Prop::named("stone::drop::slow"),
    };
    let proof = apply(reductio, vec![read("stone::drop::falls")]);
    model.parts[1].functions[0].rules[1].proof = Some(proof);
    assert_eq!(errors(&model), ["M012 at stone::drop::lands"]);
    assert!(leaves(&model).iter().any(|(at, _)| *at == lands));

    // From `drop::slow`, got by an assumed rule, tollens on `slows` can
    // only say that `falls` is false.
    let mut model = fall();
    let upon = Prop::named("stone::drop::upon");
    let ponens = judgement::Op::Ponens { t: upon, post: 0 };
    let slow = apply(
        ponens,
        vec![read("stone::drop::upon"), read("stone::speed::slow")],
    );
    let tollens = judgement::Op::Tollens {
        t: slows(),
        post: 0,
        blame: 1,
    };
    let against = vec![slow, read("stone::drop::slows"), read("stone::speed::slow")];
    let premises = [
        "stone::drop::upon",
        "stone::drop::slows",
        "stone::speed::slow",
    ];
    let mut back = rule("back", &premises, "stone::drop::falls");
    back.proof = Some(apply(tollens, against));
    let rules = &mut model.parts[1].functions[0].rules;
    rules.push(rule("upon", &["stone::speed::slow"], "stone::drop::slow"));
    rules.push(back);
    let back = path("stone::drop::back");
    assert_eq!(errors(&model), ["M012 at stone::drop::back"]);
    assert!(leaves(&model).iter().any(|(at, _)| *at == back));

    // An expression written as a conclusion is its own proposition, which no
    // proof gives: to show it, name it as a claim.
    let low = || Term::Holds(apply(Op::AtMost, vec![name("h"), real(20)]));
    let reductio = judgement::Op::Reductio {
        of: Prop::named("stone::speed::same::then"),
    };
    for proof in [read("0"), apply(reductio, vec![read("0")])] {
        let mut model = fall();
        model.parts[1].functions[1].rules.push(Rule {
            name: "same".to_string(),
            premises: vec![low()],
            conclusion: low(),
            proof: Some(proof),
        });
        assert_eq!(errors(&model), ["M012 at stone::speed::same"]);
    }
}

/// Nothing rests on itself. A conclusion rests on its rule's premises
/// and on the rule, a require on its ensure, and an ensure on what its
/// lists name. On a cycle no proposition would be a leaf, and only a
/// taking of what is no leaf would hold any of it. And no rule is named
/// through itself: what it says would be said in terms of itself.
#[test]
fn nothing_rests_on_itself() {
    // Twice through `again`, reported once.
    let again = changed(|m| {
        let slow = "stone::drop::slow";
        let again = rule("again", &[slow, slow], slow);
        m.parts[1].functions[0].rules.push(again);
    });
    assert_eq!(again, ["M013 at stone::drop::again"]);
    let both = changed(|m| {
        let rules = &mut m.parts[1].functions[0].rules;
        rules.push(rule("up", &["stone::drop::falls"], "stone::speed::slow"));
        rules.push(rule("down", &["stone::speed::slow"], "stone::drop::falls"));
    });
    assert_eq!(
        both,
        ["M013 at stone::drop::down", "M013 at stone::drop::up"]
    );
    // `lands` is a premise of itself, and shown by itself. A rule that
    // only names it is not on the cycle: judgement still knows it, and
    // `slows`.
    let mut model = fall();
    let rules = &mut model.parts[1].functions[0].rules;
    let names = [
        "stone::drop::lands",
        "stone::drop::falls",
        "stone::speed::slow",
    ];
    let ponens = judgement::Op::Ponens {
        t: Prop::named(names[0]),
        post: 0,
    };
    rules[1].premises = names.iter().map(|n| at(n)).collect();
    rules[1].proof = Some(apply(ponens, names.iter().map(|n| read(n)).collect()));
    rules.push(rule("far", &[names[0]], "stone::drop::slow"));
    assert_eq!(errors(&model), ["M014 at stone::drop::lands"]);
    let known = judgement(&model);
    let knows = |p: &str| known.premises(&Prop::named(p)).is_some();
    assert!(!knows("stone::drop::lands"));
    assert!(knows("stone::drop::far") && knows("stone::drop::slows"));
    // `slows` names `lands`, which names `slows`.
    let both = changed(|m| m.parts[1].functions[0].rules[0].premises[0] = at("stone::drop::lands"));
    assert_eq!(
        both,
        ["M014 at stone::drop::lands", "M014 at stone::drop::slows"]
    );
    // Across parts: the stone's promise rests on the sun's, which rests
    // on the stone's. Assume–guarantee in a circle.
    let circle = changed(|m| {
        m.parts[0].requires.push(Require {
            name: "landing".to_string(),
            from: Source::Ensure(path("stone::lands_slow")),
        });
        m.parts[0].ensures[0].from.push(vec![path("sun::landing")]);
        m.parts[1].ensures[0].from[0].push(path("stone::light"));
    });
    let expected = [
        "M013 at stone::lands_slow",
        "M013 at stone::light",
        "M013 at sun::landing",
        "M013 at sun::shines",
    ];
    assert_eq!(circle, expected);
}

/// A rule is named through itself whatever else is wrong with it:
/// `slows` names `lands`, which names `slows`, and a term of `slows`
/// names nothing. All three are found at once.
#[test]
fn a_cycle_is_found_through_a_broken_rule() {
    let found = changed(|m| {
        let slows = &mut m.parts[1].functions[0].rules[0].premises;
        slows.push(at("stone::drop::lands"));
        slows.push(at("stone::drop::fell"));
    });
    let expected = [
        "M007 at stone::drop::slows",
        "M014 at stone::drop::lands",
        "M014 at stone::drop::slows",
    ];
    assert_eq!(found, expected);
}

/// The fall, and a rule that gives a rule, `law: drop::falls |-
/// drop::slows`. `by_law: law, drop::falls, speed::slow |- drop::slow`
/// is shown by applying `slows`, which `law` gives: the proof never
/// reads `slows`.
fn with_law(m: &mut Model) {
    let (falls, slow) = ("stone::drop::falls", "stone::speed::slow");
    let law = judgement::Op::Ponens {
        t: Prop::named("stone::drop::law"),
        post: 0,
    };
    let given = apply(law, vec![read("stone::drop::law"), read(falls)]);
    let ponens = judgement::Op::Ponens {
        t: slows(),
        post: 0,
    };
    let premises = ["stone::drop::law", falls, slow];
    let mut by_law = rule("by_law", &premises, "stone::drop::slow");
    by_law.proof = Some(apply(ponens, vec![given, read(falls), read(slow)]));
    let rules = &mut m.parts[1].functions[0].rules;
    rules.push(rule("law", &[falls], "stone::drop::slows"));
    rules.push(by_law);
}

/// A rule given by a rule is used like any other. When a run sees the
/// stone land fast, modus tollens blames `slows`, then the law that gave
/// it, each on the run and the takings of the other premises.
#[test]
fn a_rule_given_by_a_rule() {
    let mut model = fall();
    with_law(&mut model);
    assert_eq!(check(&model), vec![]);
    // `slows`, which `law` gives, is no leaf; `by_law`, proved, is one,
    // which the checker takes.
    let leaves = leaves(&model);
    let leaf = |at: &str| leaves.iter().any(|(p, _)| *p == path(at));
    assert!(!leaf("stone::drop::slows") && leaf("stone::drop::by_law"));
    let (falls, slow) = ("stone::drop::falls", "stone::speed::slow");
    let law = Prop::named("stone::drop::law");
    let run = Taking {
        prop: Prop::named("stone::drop::slow"),
        value: false,
        count: 1,
        by: "machine:run".to_string(),
    };
    let fast = Opinion::taken(run.prop.clone(), vec![run]).unwrap();
    let values = BTreeMap::from([
        (
            "stone::drop::law".to_string(),
            taken(law.clone(), "human:reviewer"),
        ),
        (
            falls.to_string(),
            taken(Prop::named(falls), "machine:timer"),
        ),
        (slow.to_string(), taken(Prop::named(slow), "machine:ranges")),
        ("fast".to_string(), judgement::Value::Opinion(fast)),
    ]);
    let known = judgement(&model);
    let proof = model.parts[1].functions[0].rules[3].proof.as_ref().unwrap();
    let given = opinion(eval(&known, &values, proof));
    let expected = (&Prop::named("stone::drop::slow"), State::True);
    assert_eq!((given.about(), given.state()), expected);
    let blame = |t: Prop, args: Vec<Expression<judgement::Op>>| {
        let op = judgement::Op::Tollens {
            t,
            post: 0,
            blame: 0,
        };
        apply(op, args)
    };
    let wrong = blame(slows(), vec![read("fast"), read(falls), read(slow)]);
    let wrong = blame(law.clone(), vec![wrong, read(falls)]);
    let blamed = opinion(eval(&known, &values, &wrong));
    assert_eq!((blamed.about(), blamed.state()), (&law, State::False));
    assert_eq!(blamed.rests_against().len(), 1);
    let rest = blamed.rests_against().first().unwrap();
    let by: BTreeSet<&str> = rest.iter().map(|t| t.by.as_str()).collect();
    let expected = BTreeSet::from(["machine:run", "machine:timer", "machine:ranges"]);
    assert_eq!(by, expected);
}

/// A proof that applies a rule judgement does not know is not checked:
/// that rule is wrong where it is declared, and reported there, once.
/// `slows`, which `law` gives, named through `lands`: the proofs of
/// `lands` and `by_law` apply it. And `x`, a premise of which names
/// nothing, applied by modus tollens inside a `let` in the proof of
/// `lands`; and `y`, whose conclusion names nothing. Judgement knows
/// neither.
#[test]
fn a_proof_on_a_rule_judgement_does_not_know_is_not_checked() {
    let cycle = changed(|m| {
        with_law(m);
        let slows = &mut m.parts[1].functions[0].rules[0].premises;
        slows.push(at("stone::drop::lands"));
    });
    assert_eq!(
        cycle,
        ["M014 at stone::drop::lands", "M014 at stone::drop::slows"]
    );
    let mut model = fall();
    let (falls, slow) = ("stone::drop::falls", "stone::speed::slow");
    let rules = &mut model.parts[1].functions[0].rules;
    rules.push(rule("x", &[falls, "stone::drop::fell"], slow));
    rules.push(rule("y", &[falls], "stone::drop::fell"));
    let x = judgement::Op::Tollens {
        t: Prop::named("stone::drop::x"),
        post: 0,
        blame: 0,
    };
    let doubt = apply(x, vec![read(slow), read(falls)]);
    let lands = &mut rules[1];
    let proof = lands.proof.take().unwrap();
    lands.proof = Some(let_in(judgement::Op::Let, "doubt", doubt, proof));
    let expected = ["M007 at stone::drop::x", "M007 at stone::drop::y"];
    assert_eq!(errors(&model), expected);
    let known = judgement(&model);
    for rule in ["stone::drop::x", "stone::drop::y"] {
        assert_eq!(known.premises(&Prop::named(rule)), None);
    }
}

/// A name is a word, so that a path reads one way: a require named
/// `drop::falls` would give the proposition of the claim `drop::falls`, and
/// one named `drop at lab/tower/stone` that of the tie.
#[test]
fn names_are_words() {
    for bad in [
        "drop::falls",
        "drop at lab/tower/stone",
        "",
        "2nd",
        "still air",
    ] {
        let found = changed(|m| m.parts[1].requires[1].name = bad.to_string());
        assert_eq!(found, [format!("M001 at stone::{bad}")]);
    }
    for good in ["still_air", "_air", "air2", "été"] {
        let found = changed(|m| m.parts[1].requires[1].name = good.to_string());
        assert!(found.is_empty());
    }
    let take = changed(|m| {
        let odd = function("odd", "the h", ("v", Type::Real));
        m.parts[1].functions.push(odd);
    });
    assert_eq!(take, ["M001 at stone::odd::the h"]);
}

/// The parts and leaves of a proposition.
fn size(p: &Prop) -> usize {
    match p {
        Prop::Named(_) => 1,
        Prop::Not(x) => 1 + size(x),
        Prop::Implies { pre, post, .. } => 1 + pre.iter().chain(post).map(size).sum::<usize>(),
        Prop::Correct { .. } => 1,
        Prop::All { body, .. } | Prop::Any { body, .. } => 1 + size(body),
    }
}

/// A rule is a premise by its name, and judgement holds what it says
/// once. So however rules use rules, each proposition and each
/// definition stays as small as the rule is written, and a long chain is
/// read in time that grows with it. Here `r_i` gives `c_{i+1}` from
/// `c_i`, assumed, and `s_i` gives `c_{i+2}` from `r_i`, `r_{i+1}` and
/// `c_i`, shown by `r_i` then `r_{i+1}`. And `t_i` and `u_i` each take
/// both `t_{i-1}` and `u_{i-1}`: written out in place, `t_64` would hold
/// 2⁶³ copies of `r_0`.
#[test]
fn rules_sharing_rules_stay_small() {
    let (n, m) = (5000, 64);
    let c = |i: usize| format!("stone::step::c{i}");
    let r = |i: usize| format!("stone::step::r{i}");
    let mut step = function("step", "h", ("v", Type::Real));
    for i in 0..=n {
        let low = apply(Op::AtMost, vec![name("h"), real(i as i64)]);
        step.claims.push(claim(&format!("c{i}"), low));
    }
    for i in 0..n {
        step.rules.push(rule(&format!("r{i}"), &[&c(i)], &c(i + 1)));
    }
    for i in 0..n - 1 {
        let ponens = |i: usize| judgement::Op::Ponens {
            t: Prop::named(&r(i)),
            post: 0,
        };
        let first = apply(ponens(i), vec![read(&r(i)), read(&c(i))]);
        let premises = [&r(i)[..], &r(i + 1), &c(i)];
        let mut s = rule(&format!("s{i}"), &premises, &c(i + 2));
        s.proof = Some(apply(ponens(i + 1), vec![read(&r(i + 1)), first]));
        step.rules.push(s);
    }
    let tu = |i: usize| match i {
        0 => [r(0), r(1)],
        i => ["t", "u"].map(|x| format!("stone::step::{x}{i}")),
    };
    for i in 1..=m {
        let below = tu(i - 1);
        let below = [&below[0][..], &below[1]];
        step.rules.push(rule(&format!("t{i}"), &below, &c(i)));
        step.rules.push(rule(&format!("u{i}"), &below, &c(i)));
    }
    let mut model = fall();
    model.parts[1].functions.push(step);
    let start = std::time::Instant::now();
    assert_eq!(check(&model), vec![]);
    let props = props(&model);
    assert_eq!(props.len(), 12 + (n + 1) + n + (n - 1) + 2 * m);
    assert!(props.iter().all(|(_, p)| size(p) <= 4));
    let known = judgement(&model);
    for (_, p) in props.iter().filter(|(at, _)| at.names.len() == 4) {
        let premises = known.premises(p).unwrap_or_default();
        let posts = known.posts(p).unwrap_or_default();
        assert!(premises.iter().chain(posts).all(|x| size(x) == 1));
    }
    // The fall's nine, `c0`, each `r_i`, `s_i`, `t_i` and `u_i`: a shown
    // rule is taken by the checker.
    assert_eq!(leaves(&model).len(), 9 + 1 + n + (n - 1) + 2 * m);
    assert!(start.elapsed() < std::time::Duration::from_secs(20));
}

/// Examples say what a function gives at some points: `5 -> 98.1` for
/// `speed` in the fall, 2·9.81·5. Each row takes what the function takes,
/// in order, and gives what it gives, and makes no record but as
/// declared. A function with examples and no body is partial: here
/// `sun::light`, giving 1000 W/m².
#[test]
fn examples_are_of_their_function() {
    let real = |n: i64, d: i64| Value::Real(Rational::new(n, d).unwrap());
    let row = |takes: Vec<Value>, gives: Value| Example { takes, gives };
    let mut model = fall();
    model.parts[0].functions[0]
        .examples
        .push(row(vec![], real(1000, 1)));
    assert_eq!(check(&model), vec![]);
    let level = |at: &str| levels(&model).into_iter().find(|(p, _)| *p == path(at));
    assert_eq!(
        level("sun::light"),
        Some((path("sun::light"), Level::Partial))
    );
    assert_eq!(
        level("stone::speed"),
        Some((path("stone::speed"), Level::Total))
    );
    let speed = |r: Example| changed(move |m| m.parts[1].functions[1].examples.push(r));
    let wrong = ["M015 at stone::speed"];
    assert_eq!(speed(row(vec![], real(1, 1))), wrong);
    assert_eq!(speed(row(vec![real(5, 1), real(5, 1)], real(1, 1))), wrong);
    assert_eq!(speed(row(vec![Value::Int(5.into())], real(1, 1))), wrong);
    assert_eq!(speed(row(vec![real(5, 1)], Value::Truth(true))), wrong);
    // A value of an opaque record is made by no one.
    let calm = changed(|m| {
        m.parts[1].records.push(Record {
            name: "Calm".to_string(),
            fields: None,
            claims: vec![],
            rules: vec![],
        });
        let opaque = simulation::Record {
            name: "Calm".to_string(),
            fields: vec![],
        };
        let mut still = function("still", "h", ("v", Type::Real));
        still.takes[0].1 = Type::Record(opaque.clone());
        let value = Value::Record(opaque.clone(), vec![]);
        still.examples.push(row(vec![value.clone()], real(0, 1)));
        // Nor within a tuple.
        let mut pair = function("calm_pair", "h", ("v", Type::Real));
        let Type::Record(one) = tuple(vec![Type::Record(opaque)]) else {
            unreachable!()
        };
        pair.takes[0].1 = Type::Record(one.clone());
        pair.examples
            .push(row(vec![Value::Record(one, vec![value])], real(0, 1)));
        m.parts[1].functions.extend([still, pair]);
    });
    assert_eq!(calm, ["M004 at stone::calm_pair", "M004 at stone::still"]);
}

/// A record is made only as declared: not one no record declares, not
/// one with other fields, not an opaque one, and one with no name only
/// as a tuple, its fields `0`, `1`, ….
#[test]
fn records_are_made_as_declared() {
    let drop = simulation::Record {
        name: "Drop".to_string(),
        fields: vec![("h".to_string(), Type::Real), ("v".to_string(), Type::Real)],
    };
    let declared = vec![
        Record {
            name: "Drop".to_string(),
            fields: Some(drop.fields.clone()),
            claims: vec![],
            rules: vec![],
        },
        Record {
            name: "Weather".to_string(),
            fields: None,
            claims: vec![],
            rules: vec![],
        },
    ];
    let make = |r: &simulation::Record| {
        let args = r.fields.iter().map(|_| real(1)).collect();
        apply(Op::Make(r.clone()), args)
    };
    let made = |r: simulation::Record| {
        let (declared, make) = (declared.clone(), make(&r));
        changed(move |m| {
            m.parts[1].records = declared;
            let slow = &mut m.parts[1].functions[1].claims[0].holds;
            *slow = let_in(Op::Let, "made", make, slow.clone());
        })
    };
    let wrong = ["M004 at stone::speed::slow"];
    let named = |name: &str, fields: &[&str]| simulation::Record {
        name: name.to_string(),
        fields: fields.iter().map(|f| (f.to_string(), Type::Real)).collect(),
    };
    assert!(made(drop.clone()).is_empty());
    assert!(made(named("", &["0", "1"])).is_empty());
    assert_eq!(made(named("Ghost", &["h", "v"])), wrong);
    assert_eq!(made(named("Drop", &["h"])), wrong);
    assert_eq!(made(named("Weather", &[])), wrong);
    assert_eq!(made(named("", &["a"])), wrong);
    let ghost = make(&named("Ghost", &["h", "v"]));
    let body =
        changed(|m| m.parts[1].functions[1].body = Some(let_in(Op::Let, "made", ghost, two_g_h())));
    assert_eq!(body, ["M004 at stone::speed"]);
}

/// Parts meet only where one requires what another ensures. A rule or
/// a list names only its own part's propositions, a proof reads only
/// its rule's premises, and a require names another part's ensure.
/// Else the sun's promise would be used with no proposition to blame
/// for it.
#[test]
fn parts_meet_only_through_requires() {
    let shining = changed(|m| {
        let shining = rule("shining", &["stone::air"], "sun::shines");
        m.parts[1].functions[0].rules.push(shining);
    });
    assert_eq!(shining, ["M008 at stone::drop::shining"]);
    let list = changed(|m| m.parts[1].ensures[0].from[0].push(path("sun::shines")));
    assert_eq!(list, ["M008 at stone::lands_slow"]);
    let own = changed(|m| m.parts[1].requires[0].from = Source::Ensure(path("stone::lands_slow")));
    assert_eq!(own, ["M009 at stone::light"]);
    // A proof that reads a rule, even to drop it: a premise is fine, the
    // sun's rule is not.
    let cite = |rule_path: &'static str| {
        changed(move |m| {
            let positive = apply(Op::AtMost, vec![real(0), name("w")]);
            let light = &mut m.parts[0].functions[0];
            light.claims.push(claim("positive", positive));
            let bright = rule("bright", &["sun::shines"], "sun::light::positive");
            light.rules.push(bright);
            let lands = &mut m.parts[1].functions[0].rules[1];
            let proof = lands.proof.take().unwrap();
            lands.proof = Some(let_in(judgement::Op::Let, "unused", read(rule_path), proof));
        })
    };
    assert!(cite("stone::drop::slows").is_empty());
    assert_eq!(cite("sun::light::bright"), ["M012 at stone::drop::lands"]);
}

/// A tie that names nothing gives no proposition, and a list given twice
/// gives one.
#[test]
fn props_are_given_once_of_what_is_there() {
    let mut model = fall();
    model.ties[0].item = path("stone::nowhere");
    assert_eq!(props(&model).len(), 11);
    let mut model = fall();
    let list = model.parts[1].ensures[0].from[0].clone();
    model.parts[1].ensures[0].from.push(list);
    assert_eq!(props(&model), props(&fall()));
}

fn call(f: &str, args: Vec<E>) -> E {
    apply(Op::Call(f.to_string()), args)
}
fn uses(part: &str, names: &[&str]) -> Use {
    Use::Names {
        part: part.to_string(),
        names: names.iter().map(|n| n.to_string()).collect(),
    }
}
/// 2·g·h times what `f` gives.
fn times_light(f: &str) -> E {
    apply(Op::Mul, vec![two_g_h(), call(f, vec![])])
}
fn fits(contract: &str, t: Type, function: &str) -> Implements {
    Implements {
        contract: contract.to_string(),
        types: vec![t],
        functions: vec![function.to_string()],
    }
}
fn number(n: i64) -> Value {
    Value::Real(Rational::integer(n))
}
fn example(inputs: &[(&str, i64)], outputs: &[(&str, i64)]) -> ModelExample {
    let side = |ports: &[(&str, i64)]| ports.iter().map(|(p, n)| (path(p), number(*n))).collect();
    ModelExample {
        inputs: side(inputs),
        outputs: side(outputs),
    }
}
/// `correct(f, point)`, or `correct(f, x)` for `x` bound around it.
fn correct(f: &str, point: Option<Vec<Value>>) -> Prop {
    Prop::Correct {
        function: f.to_string(),
        at: match point {
            Some(p) => judgement::Term::Point(p),
            None => judgement::Term::Var("x".to_string()),
        },
    }
}
/// `T: Number`.
fn number_param() -> Type {
    Type::Param {
        name: "T".to_string(),
        bound: Bound::Number,
    }
}
/// `contract Monotone<T: Number> { fn f(x: T) -> y: T; claim rises(a: T,
/// b: T) = !(a <= b) || f(a) <= f(b); }`
fn monotone() -> Contract {
    let t = number_param();
    let f = |x: &str| call("Monotone::f", vec![name(x)]);
    let le = |a: E, b: E| apply(Op::AtMost, vec![a, b]);
    let rises = apply(
        Op::Or,
        vec![
            apply(Op::Not, vec![le(name("a"), name("b"))]),
            le(f("a"), f("b")),
        ],
    );
    Contract {
        name: "Monotone".to_string(),
        params: vec![("T".to_string(), Bound::Number)],
        functions: vec![Signature {
            name: "f".to_string(),
            takes: vec![("x".to_string(), t.clone())],
            gives: ("y".to_string(), t.clone()),
        }],
        claims: vec![Law {
            name: "rises".to_string(),
            over: vec![("a".to_string(), t.clone()), ("b".to_string(), t)],
            holds: rises,
        }],
    }
}

/// `use`: the stone reads the sun's light, and its speed is computed
/// through the call. A use names another part, and what it lists is
/// there; a record of another part is named only through one too.
#[test]
fn parts_read_each_other_through_uses() {
    let lit = |m: &mut Model| m.parts[1].functions[1].body = Some(times_light("sun::light"));
    let used = changed(|m| {
        lit(m);
        m.parts[1].uses.push(uses("sun", &["light"]));
    });
    assert_eq!(used, Vec::<String>::new());
    let glob = changed(|m| {
        lit(m);
        m.parts[1].uses.push(Use::Glob {
            part: "sun".to_string(),
        });
    });
    assert_eq!(glob, Vec::<String>::new());
    // Another name of the sun is not this one; and its own part needs none.
    let other = changed(|m| {
        lit(m);
        m.parts[1].uses.push(uses("sun", &["shines"]));
    });
    assert_eq!(other, ["M016 at stone", "M018 at stone::speed"]);
    let own = changed(|m| {
        m.parts[1].uses.push(Use::Glob {
            part: "stone".to_string(),
        })
    });
    assert_eq!(own, ["M016 at stone"]);
    let nowhere = changed(|m| {
        m.parts[1].uses.push(Use::Glob {
            part: "moon".to_string(),
        })
    });
    assert_eq!(nowhere, ["M016 at stone"]);
    let inside =
        changed(|m| m.parts[1].functions[1].body = Some(call("stone::drop", vec![name("h")])));
    assert_eq!(inside, Vec::<String>::new());
    // Through a claim, a rule's expression, a type and a record made.
    let ray = Record {
        name: "Ray".to_string(),
        fields: Some(vec![("w".to_string(), Type::Real)]),
        claims: vec![],
        rules: vec![],
    };
    let ray_type = Type::Record(simulation::Record {
        name: "Ray".to_string(),
        fields: vec![("w".to_string(), Type::Real)],
    });
    let read = |m: &mut Model| {
        m.parts[0].records.push(ray.clone());
        let bright = apply(Op::Less, vec![real(0), call("sun::light", vec![])]);
        m.parts[1].functions[1]
            .claims
            .push(claim("bright", bright.clone()));
        let mut lit = rule("lit", &[], "stone::speed::slow");
        lit.premises.push(Term::Holds(bright));
        m.parts[1].functions[1].rules.push(lit);
        m.parts[1].functions[0]
            .takes
            .push(("r".to_string(), ray_type.clone()));
        let make = apply(
            Op::Make(match ray_type.clone() {
                Type::Record(r) => r,
                _ => unreachable!(),
            }),
            vec![real(1)],
        );
        let made = apply(
            Op::Less,
            vec![real(0), apply(Op::Field("w".to_string()), vec![make])],
        );
        m.parts[1].functions[0].claims.push(claim("made", made));
    };
    let unused = changed(read);
    let expected = [
        "M018 at stone::drop::made",
        "M018 at stone::drop::r",
        "M018 at stone::speed::bright",
        "M018 at stone::speed::lit",
    ];
    assert_eq!(unused, expected);
    let used = changed(|m| {
        read(m);
        m.parts[1].uses.push(uses("sun", &["light", "Ray"]));
    });
    assert_eq!(used, Vec::<String>::new());
}

/// Calls never go round, through bodies: every function on a cycle is
/// named. A claim may call its own function: claims do not run inside
/// bodies.
#[test]
fn no_function_calls_itself() {
    let round = changed(|m| {
        m.parts[1].functions[1].body = Some(call("stone::drop", vec![name("h")]));
        m.parts[1].functions[0].body = Some(call("stone::speed", vec![name("h")]));
    });
    assert_eq!(round, ["M019 at stone::drop", "M019 at stone::speed"]);
    let claimed = changed(|m| {
        let again = apply(
            Op::Equal,
            vec![call("stone::speed", vec![name("h")]), name("v2")],
        );
        m.parts[1].functions[1].claims.push(claim("again", again));
    });
    assert_eq!(claimed, Vec::<String>::new());
    // The library: what the model computes, through calls; refused names
    // when a function goes round.
    let mut model = fall();
    model.parts[1].uses.push(uses("sun", &["light"]));
    model.parts[1].functions[1].body = Some(times_light("sun::light"));
    model.parts[0].functions[0].examples.push(Example {
        takes: vec![],
        gives: number(2),
    });
    let library = Arc::new(calls(&model).unwrap());
    let exact = Exact { calls: library };
    let speed = call("stone::speed", vec![real(5)]);
    let gives = eval(&exact, &BTreeMap::new(), &speed);
    assert_eq!(gives, Ok(Value::Real(Rational::new(1962, 10).unwrap())));
    model.parts[1].functions[0].body = Some(call("stone::drop", vec![name("h")]));
    assert_eq!(
        calls(&model).err(),
        Some(BTreeSet::from(["stone::drop".to_string()]))
    );
}

/// A contract, Rust's trait: written once over `T: Number`, checked
/// once, and an impl only fits it. Its claim is then a claim about the
/// function that fits, with its calls made calls of that function.
#[test]
fn contracts_are_checked_once_and_fitted() {
    let with = |change: Box<dyn FnOnce(&mut Contract)>, fit: Option<Implements>| {
        changed(move |m| {
            let mut c = monotone();
            change(&mut c);
            m.contracts.push(c);
            m.parts[1].implements.extend(fit);
        })
    };
    let fine = || Some(fits("Monotone", Type::Real, "speed"));
    assert_eq!(with(Box::new(|_| {}), fine()), Vec::<String>::new());
    // Its claim, read of `speed`.
    let mut model = fall();
    model.contracts.push(monotone());
    model.parts[1]
        .implements
        .push(fits("Monotone", Type::Real, "speed"));
    let claims = contract_claims(&model);
    assert_eq!(claims.len(), 1);
    assert_eq!(claims[0].at, path("stone::speed::Monotone::rises"));
    assert_eq!(
        claims[0].over,
        vec![("a".to_string(), Type::Real), ("b".to_string(), Type::Real)]
    );
    let reads_speed = format!("{:?}", claims[0].holds);
    assert!(reads_speed.contains("stone::speed") && !reads_speed.contains("Monotone::f"));
    let rises = Prop::named("stone::speed::Monotone::rises");
    assert!(props(&model).contains(&(path("stone::speed::Monotone::rises"), rises)));
    // Run at two points, it holds: speed rises with h.
    let exact = Exact {
        calls: Arc::new(calls(&model).unwrap()),
    };
    let points = BTreeMap::from([("a".to_string(), number(1)), ("b".to_string(), number(2))]);
    assert_eq!(
        eval(&exact, &points, &claims[0].holds),
        Ok(Value::Truth(true))
    );
    // Errors in the contract are found once, at the contract.
    let not_truth = with(
        Box::new(|c| c.claims[0].holds = call("Monotone::f", vec![name("a")])),
        fine(),
    );
    assert_eq!(not_truth, ["M006 at Monotone::rises"]);
    let outside = with(
        Box::new(|c| {
            c.claims[0].holds = apply(Op::Less, vec![real(0), call("sun::light", vec![])])
        }),
        fine(),
    );
    assert_eq!(outside, ["M017 at Monotone::rises"]);
    let any = with(
        Box::new(|c| {
            c.functions[0].takes[0].1 = Type::Param {
                name: "T".to_string(),
                bound: Bound::Any,
            }
        }),
        fine(),
    );
    assert_eq!(any, ["M003 at Monotone::f::x"]);
    let named = with(
        Box::new(|c| {
            c.name = "sun".to_string();
            c.claims.clear();
        }),
        None,
    );
    assert_eq!(named, ["M002 at sun"]);
    // A parameter outside a contract.
    let open = changed(|m| m.parts[1].functions[1].takes[0].1 = number_param());
    assert_eq!(open, ["M003 at stone::speed::h"]);
    // An impl only fits: the bound, the count, the functions and their types.
    for (fit, why) in [
        (fits("Monotone", Type::Truth, "speed"), "bound"),
        (
            Implements {
                types: vec![],
                ..fits("Monotone", Type::Real, "speed")
            },
            "count",
        ),
        (fits("Monotone", Type::Int, "speed"), "types"),
        (fits("Monotone", Type::Real, "nowhere"), "function"),
        (
            Implements {
                functions: vec![],
                ..fits("Monotone", Type::Real, "speed")
            },
            "functions",
        ),
    ] {
        let at = match fit.functions.is_empty() {
            true => "M020 at stone::Monotone",
            false => "M020 at stone::speed::Monotone",
        };
        let at = if why == "function" {
            "M020 at stone::nowhere::Monotone"
        } else {
            at
        };
        assert_eq!(with(Box::new(|_| {}), Some(fit)), [at], "{why}");
    }
    // An impl declared twice, or beside a claim of its name.
    let twice = with(Box::new(|_| {}), fine());
    assert!(twice.is_empty());
    let twice = changed(|m| {
        m.contracts.push(monotone());
        m.parts[1]
            .implements
            .push(fits("Monotone", Type::Real, "speed"));
        m.parts[1]
            .implements
            .push(fits("Monotone", Type::Real, "speed"));
    });
    assert_eq!(twice, ["M002 at stone::speed::Monotone"]);
    // A function that fits says more than its name: it is no longer opaque.
    let mut model = fall();
    model.contracts.push(monotone());
    model.parts[0].functions[0]
        .takes
        .push(("t".to_string(), Type::Real));
    model.parts[0]
        .implements
        .push(fits("Monotone", Type::Real, "light"));
    assert_eq!(check(&model), vec![]);
    assert_eq!(levels(&model)[0], (path("sun::light"), Level::Partial));
}

/// Examples of the model as a whole, end to end: inputs on what
/// functions take, or on what one with no body gives; outputs on what
/// functions give, each argument given.
#[test]
fn model_examples_name_inputs_and_outputs() {
    let lit = |m: &mut Model| {
        m.parts[1].uses.push(uses("sun", &["light"]));
        m.parts[1].functions[1].body = Some(times_light("sun::light"));
    };
    let fine = changed(|m| {
        lit(m);
        m.examples.push(example(
            &[("sun::light::w", 2), ("stone::speed::h", 5)],
            &[("stone::speed::v2", 196)],
        ));
    });
    assert_eq!(fine, Vec::<String>::new());
    // An output of its own: the sun's light, which takes nothing.
    let lit_up = vec![("sun::light::w", 2)];
    for (inputs, outputs, expected) in [
        // An input the model computes; an output it takes; a name of nothing.
        (
            vec![("stone::speed::v2", 1), ("stone::speed::h", 1)],
            lit_up.clone(),
            "M021",
        ),
        (
            vec![("stone::speed::h", 1)],
            vec![("stone::speed::h", 1)],
            "M021",
        ),
        (vec![("stone::speed::x", 1)], lit_up.clone(), "M021"),
        (vec![("stone::speed", 1)], lit_up.clone(), "M021"),
        (
            vec![("stone::speed::h", 1), ("stone::speed::h", 2)],
            lit_up.clone(),
            "M021",
        ),
        // No output; a port on both sides.
        (vec![("stone::speed::h", 1)], vec![], "M021"),
        (lit_up.clone(), lit_up.clone(), "M021"),
        // What a function with no body gives, at no point.
        (vec![("stone::drop::v", 1)], lit_up.clone(), "M022"),
    ] {
        let got = changed(|m| {
            lit(m);
            m.examples.push(example(&inputs, &outputs));
        });
        // One error or more, each of the code: an output that is what a
        // function takes is on both sides too.
        let wanted = format!("{expected} at examples::0");
        assert!(
            !got.is_empty() && got.iter().all(|g| g == &wanted),
            "{inputs:?} {outputs:?}: {got:?}"
        );
    }
    let typed = changed(|m| {
        m.examples.push(ModelExample {
            inputs: vec![(path("stone::speed::h"), Value::Truth(true))],
            outputs: vec![(path("sun::light::w"), number(2))],
        });
    });
    assert_eq!(typed, ["M022 at examples::0"]);
}

/// A rule writes where functions of its part are correct: at a point,
/// or over a set, as an all or an any. Such a proposition is itself,
/// whichever rule writes it.
#[test]
fn rules_write_where_functions_are_correct() {
    let set = |kind: Value| judgement::Set {
        ranges: vec![("h".to_string(), kind.clone(), kind)],
        such: None,
    };
    let everywhere = |over: judgement::Set, body: Prop| Prop::All {
        name: "x".to_string(),
        over,
        body: Box::new(body),
    };
    let with = |conclusion: Prop| {
        changed(move |m| {
            let mut r = rule("everywhere", &["stone::speed::slow"], "stone::drop::slow");
            r.conclusion = Term::Prop(conclusion);
            m.parts[1].functions[1].rules.push(r);
        })
    };
    let fine = everywhere(set(number(0)), correct("stone::speed", None));
    assert_eq!(with(fine.clone()), Vec::<String>::new());
    let at_point = correct("stone::speed", Some(vec![number(5)]));
    assert_eq!(with(at_point), Vec::<String>::new());
    let ints = everywhere(set(Value::Int(0.into())), correct("stone::speed", None));
    assert_eq!(with(ints), ["M023 at stone::speed::everywhere"]);
    let free = correct("stone::speed", None);
    assert_eq!(with(free), ["M023 at stone::speed::everywhere"]);
    let sun = correct("sun::light", Some(vec![]));
    assert_eq!(with(sun), ["M008 at stone::speed::everywhere"]);
    let nothing = correct("stone::nothing", Some(vec![]));
    assert_eq!(with(nothing), ["M023 at stone::speed::everywhere"]);
    assert_eq!(
        with(Prop::named("stone::speed::slow")),
        ["M023 at stone::speed::everywhere"]
    );
    // The proposition is itself, a leaf when nothing gives it.
    let mut model = fall();
    let mut r = rule("represented", &[], "stone::speed::slow");
    r.premises.push(Term::Prop(fine.clone()));
    model.parts[1].functions[1].rules.push(r);
    assert_eq!(check(&model), vec![]);
    assert!(props(&model).contains(&(path("stone::speed::represented"), fine.clone())));
    assert!(leaves(&model).iter().any(|(_, p)| *p == fine));
    // Two rules writing it, one from the other: a cycle through it.
    let round = changed(|m| {
        let mut a = rule("a", &[], "stone::speed::slow");
        a.premises.push(Term::Prop(fine.clone()));
        let mut b = rule("b", &["stone::speed::slow"], "stone::speed::slow");
        b.conclusion = Term::Prop(fine.clone());
        m.parts[1].functions[1].rules.extend([a, b]);
    });
    assert!(
        round.iter().all(|e| e.starts_with("M013")) && !round.is_empty(),
        "{round:?}"
    );
}
