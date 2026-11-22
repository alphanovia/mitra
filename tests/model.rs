//! The model on the fall (`docs/model.md`). The stone, in the system,
//! falls; the sun, outside it, gives light. The model is checked, read
//! for its levels, pieces and leaves, judged on takings, and its claims
//! run in simulation, all on `expression` as it is.

mod common;

use common::{apply, let_in};
use mitra_core::expression::{eval, Expression};
use mitra_core::judgement::{self, Judgement, Opinion, Piece, State, Taking};
use mitra_core::location::Location;
use mitra_core::model::{
    check, leaves, levels, pieces, Claim, Ensure, Function, Kind, Level, Model, Part, Path,
    Problem, Prop, Record, Require, Rule, Source, Tie,
};
use mitra_core::simulation::{self, tuple, Bounds, Exact, Op, Ranges, Rational, Type, Value};
use std::collections::BTreeMap;

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
fn at(p: &str) -> Prop {
    Prop::At(path(p))
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

/// The rule `slows`, as judgement reads it.
fn slows() -> Piece {
    Piece::Implies {
        pre: vec![
            Piece::named("stone::drop::falls"),
            Piece::named("stone::speed::slow"),
        ],
        subject: "stone::drop::slows".to_string(),
        post: vec![Piece::named("stone::drop::slow")],
    }
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
    // `slows`, then its premises, give its conclusion: modus ponens.
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

#[test]
fn the_fall_is_well_formed() {
    assert_eq!(check(&fall()), vec![]);
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
fn implies(about: &str, pre: &[&str], post: &str) -> (Path, Piece) {
    let piece = Piece::Implies {
        pre: pre.iter().map(|p| Piece::named(p)).collect(),
        subject: about.to_string(),
        post: vec![Piece::named(post)],
    };
    (path(about), piece)
}
fn named(p: &str) -> (Path, Piece) {
    (path(p), Piece::named(p))
}

#[test]
fn each_proposition_is_a_piece_about_its_path() {
    let mut lands = slows();
    if let Piece::Implies { pre, subject, .. } = &mut lands {
        pre.insert(0, slows());
        *subject = "stone::drop::lands".to_string();
    }
    let expected = vec![
        named("stone::air"),
        named("stone::drop::falls"),
        (path("stone::drop::lands"), lands),
        named("stone::drop::slow"),
        (path("stone::drop::slows"), slows()),
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
            Piece::named("stone::drop at lab/tower/stone"),
        ),
    ];
    assert_eq!(pieces(&fall()), expected);
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
        (path("stone::drop::slows"), slows()),
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
            Piece::named("stone::drop at lab/tower/stone"),
        ),
    ];
    assert_eq!(leaves(&fall()), expected);
}

/// An opinion of `piece`, taken true once by `by`.
fn taken(piece: Piece, by: &str) -> judgement::Value {
    let taking = Taking {
        piece: piece.clone(),
        value: true,
        count: 1,
        by: by.to_string(),
    };
    judgement::Value::Opinion(Opinion::taken(piece, vec![taking]).unwrap())
}
fn opinion<F: std::fmt::Debug>(v: Result<judgement::Value, F>) -> Opinion {
    match v {
        Ok(judgement::Value::Opinion(o)) => o,
        other => panic!("{other:?}"),
    }
}

/// Judgement reads the pieces as they come: `lands_slow` follows from
/// its list, taken by a reviewer, and the claims, taken by a machine and
/// a timer. And the proof of `lands` gives `drop::slow`, on takings of
/// its premises.
#[test]
fn the_fall_is_judged_on_takings() {
    let pieces = pieces(&fall());
    let (_, list) = implies(
        "stone::lands_slow",
        &["stone::speed::slow", "stone::drop::falls"],
        "stone::lands_slow",
    );
    assert!(pieces.iter().any(|(_, p)| *p == list));
    let values = BTreeMap::from([
        ("t".to_string(), taken(list.clone(), "human:reviewer")),
        (
            "slow".to_string(),
            taken(Piece::named("stone::speed::slow"), "machine:ranges"),
        ),
        (
            "falls".to_string(),
            taken(Piece::named("stone::drop::falls"), "machine:timer"),
        ),
    ]);
    let args = ["t", "slow", "falls"]
        .iter()
        .map(|n| Expression::Name(n.to_string()));
    let ponens = apply(judgement::Op::Ponens { t: list, post: 0 }, args.collect());
    let lands_slow = opinion(eval(&Judgement, &values, &ponens));
    assert_eq!(lands_slow.about(), &Piece::named("stone::lands_slow"));
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
            taken(Piece::named("stone::drop::falls"), "machine:timer"),
        ),
        (
            "stone::speed::slow".to_string(),
            taken(Piece::named("stone::speed::slow"), "machine:ranges"),
        ),
    ]);
    let slow = opinion(eval(&Judgement, &values, lands.proof.as_ref().unwrap()));
    assert_eq!(slow.about(), &Piece::named("stone::drop::slow"));
    assert_eq!(slow.state(), State::True);
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
    let low = Bounds::between(Rational::integer(0), Rational::integer(20)).unwrap();
    let heights = BTreeMap::from([("h".to_string(), low)]);
    assert_eq!(eval(&Ranges, &heights, &slow).unwrap().surely(), Some(true));
    let measured = BTreeMap::from([
        ("h".to_string(), Value::Real(Rational::integer(5))),
        ("v".to_string(), Value::Real(Rational::new(99, 10).unwrap())),
    ]);
    assert_eq!(
        eval(&Exact, &measured, &drop.claims[0].holds),
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
    assert_eq!(check(&model), vec![Problem::Mistyped(path("stone::both"))]);
}

/// The problems of the fall with one change. Reading its pieces and
/// leaves ends all the same.
fn changed(change: impl FnOnce(&mut Model)) -> Vec<Problem> {
    let mut model = fall();
    change(&mut model);
    let _ = (pieces(&model), leaves(&model), levels(&model));
    check(&model)
}

#[test]
fn each_problem_is_found() {
    let unknown = |at: &str, names: &str| Problem::Unknown {
        at: path(at),
        names: path(names),
    };
    let falls = changed(|m| {
        let falls = m.parts[1].functions[0].claims[0].clone();
        m.parts[1].functions[0].claims.push(falls);
    });
    assert_eq!(falls, vec![Problem::Twice(path("stone::drop::falls"))]);
    let fell = changed(|m| m.parts[1].functions[0].rules[0].premises[0] = at("stone::drop::fell"));
    assert_eq!(
        fell,
        vec![unknown("stone::drop::slows", "stone::drop::fell")]
    );
    let light = changed(|m| m.parts[1].requires[0].from = Source::Ensure(path("sun::light")));
    assert_eq!(light, vec![unknown("stone::light", "sun::light")]);
    let tie = changed(|m| m.ties[0].item = path("stone::drop::falls"));
    assert_eq!(
        tie,
        vec![unknown("stone::drop::falls", "stone::drop::falls")]
    );
    let number = changed(|m| m.parts[1].functions[0].claims[0].holds = name("v"));
    assert_eq!(number, vec![Problem::Mistyped(path("stone::drop::falls"))]);
    let body = changed(|m| m.parts[1].functions[1].body = Some(under("h", 1)));
    assert_eq!(body, vec![Problem::Mistyped(path("stone::speed"))]);
    let proof = changed(|m| m.parts[1].functions[0].rules[1].conclusion = at("stone::drop::falls"));
    assert_eq!(proof, vec![Problem::Mistyped(path("stone::drop::lands"))]);
    let ghost = Type::Record(simulation::Record {
        name: "Ghost".to_string(),
        fields: vec![],
    });
    let record = changed(|m| {
        m.parts[1].records.push(Record {
            name: "Drop".to_string(),
            fields: Some(vec![("h".to_string(), ghost)]),
            claims: vec![],
            rules: vec![],
        })
    });
    assert_eq!(record, vec![Problem::Mistyped(path("stone::Drop"))]);
    let empty = changed(|m| m.parts[1].ensures[0].from.push(vec![]));
    assert_eq!(empty, vec![Problem::Empty(path("stone::lands_slow"))]);
    let cycle = changed(|m| {
        m.parts[1].functions[0].rules[0]
            .premises
            .push(at("stone::drop::slows"))
    });
    assert_eq!(cycle, vec![Problem::Circular(path("stone::drop::slows"))]);
}
