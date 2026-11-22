//! The model on the fall (`docs/model.md`). The stone, in the system,
//! falls; the sun, outside it, gives light. The model is checked, read
//! for its levels, propositions and leaves, judged on takings, and its claims
//! run in simulation, all on `expression` as it is.

mod common;

use common::{apply, let_in};
use mitra_core::expression::{eval, Expression};
use mitra_core::judgement::{self, Opinion, Prop, State, Taking};
use mitra_core::location::Location;
use mitra_core::model::{
    check, judgement, leaves, levels, props, Claim, Ensure, Function, Kind, Level, Model, Part,
    Path, Problem, Record, Require, Rule, Source, Term, Tie,
};
use mitra_core::simulation::{self, tuple, Bounds, Exact, Op, Ranges, Rational, Type, Value};
use std::collections::{BTreeMap, BTreeSet};

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

/// The problems of the fall with one change. Reading its propositions and
/// leaves ends all the same.
fn changed(change: impl FnOnce(&mut Model)) -> Vec<Problem> {
    let mut model = fall();
    change(&mut model);
    let _ = (props(&model), leaves(&model), levels(&model));
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
    let proof = changed(|m| m.parts[1].functions[0].rules[1].conclusion = at("stone::air"));
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
    let function = changed(|m| m.parts[1].functions[0].rules[0].premises[0] = at("stone::speed"));
    assert_eq!(
        function,
        vec![unknown("stone::drop::slows", "stone::speed")]
    );
    let cycle = changed(|m| {
        m.parts[1].ensures[0]
            .from
            .push(vec![path("stone::lands_slow")])
    });
    assert_eq!(cycle, vec![Problem::Circular(path("stone::lands_slow"))]);
    let misnamed = changed(|m| m.parts[1].requires[1].name = "still air".to_string());
    assert_eq!(
        misnamed,
        vec![Problem::Misnamed(Path::to(&["stone", "still air"]))]
    );
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
/// it stays a leaf.
#[test]
fn a_proof_argues_for_its_conclusion() {
    let lands = path("stone::drop::lands");
    let mut model = fall();
    let reductio = judgement::Op::Reductio {
        of: Prop::named("stone::drop::slow"),
    };
    let proof = apply(reductio, vec![read("stone::drop::falls")]);
    model.parts[1].functions[0].rules[1].proof = Some(proof);
    assert_eq!(check(&model), vec![Problem::Mistyped(lands.clone())]);
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
    assert_eq!(check(&model), vec![Problem::Mistyped(back.clone())]);
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
        let same = Problem::Mistyped(path("stone::speed::same"));
        assert_eq!(check(&model), vec![same]);
    }
}

/// Nothing rests on itself. A conclusion rests on its rule's premises
/// and on the rule, a require on its ensure, and an ensure on what its
/// lists name. On a cycle no proposition would be a leaf, and only a
/// taking of what is no leaf would hold any of it. And no rule is named
/// through itself: what it says would be said in terms of itself.
#[test]
fn nothing_rests_on_itself() {
    let circular = |p: &str| Problem::Circular(path(p));
    let again = changed(|m| {
        let again = rule("again", &["stone::drop::slow"], "stone::drop::slow");
        m.parts[1].functions[0].rules.push(again);
    });
    assert_eq!(again, vec![circular("stone::drop::again")]);
    let both = changed(|m| {
        let rules = &mut m.parts[1].functions[0].rules;
        rules.push(rule("up", &["stone::drop::falls"], "stone::speed::slow"));
        rules.push(rule("down", &["stone::speed::slow"], "stone::drop::falls"));
    });
    assert_eq!(
        both,
        vec![circular("stone::drop::down"), circular("stone::drop::up")]
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
    assert_eq!(check(&model), vec![circular("stone::drop::lands")]);
    let known = judgement(&model);
    let knows = |p: &str| known.premises(&Prop::named(p)).is_some();
    assert!(!knows("stone::drop::lands"));
    assert!(knows("stone::drop::far") && knows("stone::drop::slows"));
    // `slows` names `lands`, which names `slows`.
    let both = changed(|m| m.parts[1].functions[0].rules[0].premises[0] = at("stone::drop::lands"));
    assert_eq!(
        both,
        vec![
            circular("stone::drop::lands"),
            circular("stone::drop::slows")
        ]
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
        "stone::lands_slow",
        "stone::light",
        "sun::landing",
        "sun::shines",
    ];
    assert_eq!(circle, expected.map(circular).to_vec());
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
        assert_eq!(found, vec![Problem::Misnamed(Path::to(&["stone", bad]))]);
    }
    for good in ["still_air", "_air", "air2", "été"] {
        let found = changed(|m| m.parts[1].requires[1].name = good.to_string());
        assert_eq!(found, vec![]);
    }
    let take = changed(|m| {
        let odd = function("odd", "the h", ("v", Type::Real));
        m.parts[1].functions.push(odd);
    });
    let odd = Path::to(&["stone", "odd", "the h"]);
    assert_eq!(take, vec![Problem::Misnamed(odd)]);
}

/// The parts and leaves of a proposition.
fn size(p: &Prop) -> usize {
    match p {
        Prop::Named(_) => 1,
        Prop::Not(x) => 1 + size(x),
        Prop::Implies { pre, post, .. } => 1 + pre.iter().chain(post).map(size).sum::<usize>(),
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
    // The fall's eight, `c0`, each `r_i`, `t_i` and `u_i`; each `s_i` is
    // shown.
    assert_eq!(leaves(&model).len(), 8 + 1 + n + 2 * m);
    assert!(start.elapsed() < std::time::Duration::from_secs(20));
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
    let wrong = vec![Problem::Mistyped(path("stone::speed::slow"))];
    let named = |name: &str, fields: &[&str]| simulation::Record {
        name: name.to_string(),
        fields: fields.iter().map(|f| (f.to_string(), Type::Real)).collect(),
    };
    assert_eq!(made(drop.clone()), vec![]);
    assert_eq!(made(named("", &["0", "1"])), vec![]);
    assert_eq!(made(named("Ghost", &["h", "v"])), wrong);
    assert_eq!(made(named("Drop", &["h"])), wrong);
    assert_eq!(made(named("Weather", &[])), wrong);
    assert_eq!(made(named("", &["a"])), wrong);
    let ghost = make(&named("Ghost", &["h", "v"]));
    let body =
        changed(|m| m.parts[1].functions[1].body = Some(let_in(Op::Let, "made", ghost, two_g_h())));
    assert_eq!(body, vec![Problem::Mistyped(path("stone::speed"))]);
}

/// Parts meet only where one requires what another ensures. A rule or
/// a list names only its own part's propositions, a proof reads only
/// its rule's premises, and a require names another part's ensure.
/// Else the sun's promise would be used with no proposition to blame
/// for it.
#[test]
fn parts_meet_only_through_requires() {
    let unknown = |at: &str, names: &str| Problem::Unknown {
        at: path(at),
        names: path(names),
    };
    let shining = changed(|m| {
        let shining = rule("shining", &["stone::air"], "sun::shines");
        m.parts[1].functions[0].rules.push(shining);
    });
    assert_eq!(
        shining,
        vec![unknown("stone::drop::shining", "sun::shines")]
    );
    let list = changed(|m| m.parts[1].ensures[0].from[0].push(path("sun::shines")));
    assert_eq!(list, vec![unknown("stone::lands_slow", "sun::shines")]);
    let own = changed(|m| m.parts[1].requires[0].from = Source::Ensure(path("stone::lands_slow")));
    assert_eq!(own, vec![unknown("stone::light", "stone::lands_slow")]);
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
    assert_eq!(cite("stone::drop::slows"), vec![]);
    let lands = Problem::Mistyped(path("stone::drop::lands"));
    assert_eq!(cite("sun::light::bright"), vec![lands]);
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
