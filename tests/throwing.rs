//! The throwing toy. One object, a statement `falls` ("thrown up, it
//! goes up then falls down"), a tie `applies` (this model is a good
//! model of this object), and a throw: evidence believed as far as I
//! trust my senses.
//!
//! A throw tests the statement and the tie together. That pair is a
//! piece of its own here, `lands`, which rests on both.

use mitra_core::model::*;

fn p(name: &str) -> Piece {
    Piece::named(name)
}
fn says(name: &str, about: &str, yes_: bool, by: &str) -> Evidence {
    let value = if yes_ { yes() } else { no() };
    Evidence {
        piece: p(name),
        about: p(about),
        value,
        by: Voice::named(by),
    }
}
fn rests(pieces: &[&str]) -> Statement<Related> {
    Statement {
        piece: Piece::named(&format!("all: {}", pieces[0])),
        content: Related {
            how: Relation::All,
            pieces: pieces.iter().map(|p| Piece::named(p)).collect(),
        },
    }
}
/// The relations of these models are plain structure: believed at 1.
fn sure(mut model: Model) -> Model {
    for said in &model.related {
        if !model.base.iter().any(|(p, _)| p == &said.piece) {
            model.base.push((said.piece.clone(), 1.0));
        }
    }
    model
}

/// One throw: base rates of the tie and the statement, trust in my
/// senses, and what I saw.
fn single(a: f64, f: f64, s: f64, fell: bool) -> Model {
    sure(Model {
        evidence: vec![says("throw", "lands", fell, "me")],
        related: vec![rests(&["lands", "applies", "falls"])],
        base: vec![(p("applies"), a), (p("falls"), f), (p("throw"), s)],
    })
}

fn base(rates: &[(&str, f64)]) -> Vec<(Piece, f64)> {
    rates.iter().map(|(x, a)| (p(x), *a)).collect()
}
fn close(x: f64, y: f64) -> bool {
    (x - y).abs() < 1e-3
}
/// The counts of yes and of no, and what the opinion reads as.
fn reads(m: &Model, x: &str, r: f64, s: f64, at: f64) -> bool {
    let o = opinion(m, &p(x));
    close(o.r, r) && close(o.s, s) && close(projected(o), at)
}
fn names(pieces: &[&str]) -> Vec<Vec<Piece>> {
    match pieces.is_empty() {
        true => vec![],
        false => vec![pieces.iter().map(|x| p(x)).collect()],
    }
}

/// Scenario 1, one throw that falls: one yes, as far as I trust my
/// senses, on the tie and on the statement alike.
#[test]
fn one_throw_that_falls() {
    // (tie, statement, senses) -> the yes on each, what the tie reads as, the pair
    let rows = [
        (0.5, 0.5, 0.5, 0.5, 0.857, 0.735),
        (0.5, 0.5, 0.9, 0.9, 0.909, 0.826),
        (0.5, 0.5, 1.0, 1.0, 0.917, 0.840),
        (0.9, 0.9, 0.5, 0.5, 0.971, 0.944),
        (1.0, 1.0, 1.0, 1.0, 1.0, 1.0),
        (0.0, 0.0, 1.0, 1.0, 0.833, 0.694),
        (0.5, 0.5, 0.0, 0.0, 0.5, 0.25), // senses not trusted
    ];
    for (a, f, s, yes_, tie, pair) in rows {
        let m = single(a, f, s, true);
        let row = format!("{a} {f} {s}");
        assert!(reads(&m, "applies", yes_, 0.0, tie), "{row}");
        assert!(reads(&m, "falls", yes_, 0.0, tie), "{row}");
        // The pair reads as its two parts together.
        assert!(close(projected(opinion(&m, &p("lands"))), pair), "{row}");
        assert_eq!(conflicts(&m), names(&[]));
    }
}

/// Scenario 1, one throw that does not fall. The no is shared between
/// the tie, the statement and the throw: more to what was expected
/// less, and nothing flips to a yes.
#[test]
fn one_throw_that_does_not_fall() {
    // (tie, statement, senses) -> the no on each of the three, the suspects
    type Row = (f64, f64, f64, [f64; 3], &'static [&'static str]);
    let all: &[&str] = &["applies", "falls", "throw"];
    let rows: [Row; 9] = [
        // An equal start gives equal shares.
        (0.5, 0.5, 0.5, [0.027; 3], all),
        (1.0, 1.0, 1.0, [1.0 / 3.0; 3], all),
        // Senses trusted more: the tie and the statement take more.
        (0.5, 0.5, 0.9, [0.064, 0.064, 0.006], all),
        (0.5, 0.5, 1.0, [0.077, 0.077, 0.0], all),
        // Senses trusted less: the throw takes more.
        (
            0.9,
            0.9,
            0.5,
            [0.013, 0.013, 0.262],
            &["throw", "applies", "falls"],
        ),
        (1.0, 1.0, 0.0, [0.0; 3], &[]), // a throw nobody believes accuses nothing
        // Three different starts: the order of the suspects follows.
        (
            0.9,
            0.5,
            0.6,
            [0.007, 0.089, 0.052],
            &["falls", "throw", "applies"],
        ),
        // Expected wrong already: nothing more to say.
        (0.0, 0.0, 1.0, [0.0; 3], all),
        (0.0, 0.0, 0.6, [0.0; 3], all),
    ];
    for (a, f, s, no_, suspects) in rows {
        let m = single(a, f, s, false);
        let row = format!("{a} {f} {s}");
        for (x, no_) in ["applies", "falls", "throw"].iter().zip(no_) {
            let o = opinion(&m, &p(x));
            assert!(close(o.s, no_) && o.r == 0.0, "{row} {x} {o:?}");
        }
        assert_eq!(conflicts(&m), names(suspects), "{row}");
    }
}

/// Scenario 2, the same throw again, senses at 0.6: one voice counts
/// once; different voices add their counts.
#[test]
fn repeated_throws() {
    let tie = |n: usize, one_voice: bool| {
        let mut m = sure(Model {
            related: vec![rests(&["lands", "applies", "falls"])],
            base: base(&[("applies", 0.5), ("falls", 0.5)]),
            ..Model::default()
        });
        for k in 0..n {
            let by = match one_voice {
                true => "me".to_string(),
                false => format!("friend {k}"),
            };
            m.evidence
                .push(says(&format!("throw {k}"), "lands", true, &by));
            m.base.push((p(&format!("throw {k}")), 0.6));
        }
        assert_eq!(opinion(&m, &p("applies")), opinion(&m, &p("falls")));
        opinion(&m, &p("applies"))
    };
    for n in [1, 2, 4] {
        assert!(close(tie(n, true).r, 0.6));
    }
    for (n, at) in [(1, 0.875), (2, 0.929), (6, 0.974), (10, 0.984), (40, 0.996)] {
        let o = tie(n, false);
        assert!(close(o.r, 0.6 * n as f64) && close(projected(o), at), "{n}");
    }
}

/// Scenario 3, a ball and a balloon under one statement. The ball
/// falls, the balloon does not: `falls` is believed through the ball,
/// so the balloon's tie takes more of the no.
#[test]
fn ball_and_balloon() {
    let third = ["throw balloon", "falls"];
    let rows = [
        (0.9, 0.273, 0.037, 0.014, third),
        (1.0, 0.478, 0.043, 0.0, ["falls", "throw balloon"]),
    ];
    for (s, tie, statement, throw, then) in rows {
        let m = sure(Model {
            evidence: vec![
                says("throw ball", "ball lands", true, "me"),
                says("throw balloon", "balloon lands", false, "me"),
            ],
            related: vec![
                rests(&["ball lands", "ball applies", "falls"]),
                rests(&["balloon lands", "balloon applies", "falls"]),
            ],
            base: base(&[
                ("ball applies", 0.5),
                ("balloon applies", 0.5),
                ("falls", 0.5),
                ("throw ball", s),
                ("throw balloon", s),
            ]),
        });
        assert!(close(opinion(&m, &p("ball applies")).r, s));
        assert!(close(opinion(&m, &p("balloon applies")).s, tie));
        assert!(close(opinion(&m, &p("falls")).r, s));
        assert!(close(opinion(&m, &p("falls")).s, statement));
        assert!(close(opinion(&m, &p("throw balloon")).s, throw));
        assert_eq!(conflicts(&m), names(&["balloon applies", then[0], then[1]]));
    }
}

/// Scenario 4, one object and two statements. It falls, and makes no
/// noise: the mirror of scenario 3.
#[test]
fn falls_and_noise() {
    let rows = [
        (0.9, 0.273, 0.037, 0.014, ["listen", "applies"]),
        (1.0, 0.478, 0.043, 0.0, ["applies", "listen"]),
    ];
    for (s, noise, tie, listen, then) in rows {
        let m = sure(Model {
            evidence: vec![
                says("throw", "lands", true, "me"),
                says("listen", "bangs", false, "me"),
            ],
            related: vec![
                rests(&["lands", "applies", "falls"]),
                rests(&["bangs", "applies", "noise"]),
            ],
            base: base(&[
                ("applies", 0.5),
                ("falls", 0.5),
                ("noise", 0.5),
                ("throw", s),
                ("listen", s),
            ]),
        });
        assert!(close(opinion(&m, &p("falls")).r, s));
        assert!(close(opinion(&m, &p("noise")).s, noise));
        assert!(close(opinion(&m, &p("applies")).s, tie));
        assert!(close(opinion(&m, &p("listen")).s, listen));
        assert_eq!(conflicts(&m), names(&["noise", then[0], then[1]]));
    }
}

/// Scenario 5, a latent statement. `gravity` has no tie and nothing to
/// throw at; `mass ball` and `mass balloon` are tied to their objects;
/// "the ball falls" is all of [gravity, mass ball], and a throw is
/// evidence about that. Two objects tell gravity from a mass.
#[test]
fn gravity_and_two_masses() {
    let world = |g: f64, ball: Option<bool>, balloon: Option<bool>| {
        let mut m = sure(Model {
            related: vec![
                rests(&["falls ball", "gravity", "mass ball"]),
                rests(&["falls balloon", "gravity", "mass balloon"]),
            ],
            base: base(&[("gravity", g), ("mass ball", 0.5), ("mass balloon", 0.5)]),
            ..Model::default()
        });
        for (object, fell) in [("ball", ball), ("balloon", balloon)] {
            if let Some(fell) = fell {
                let throw = format!("throw {object}");
                // Two throws are two acts: each has its own voice.
                m.evidence
                    .push(says(&throw, &format!("falls {object}"), fell, &throw));
                m.base.push((p(&throw), 0.9));
            }
        }
        m
    };
    // One object: gravity and its mass move alike, as in scenario 1.
    let m = world(0.5, Some(true), None);
    assert!(reads(&m, "gravity", 0.9, 0.0, 0.909) && reads(&m, "mass ball", 0.9, 0.0, 0.909));
    let m = world(0.5, Some(false), None);
    assert!(reads(&m, "gravity", 0.0, 0.064, 0.379) && reads(&m, "mass ball", 0.0, 0.064, 0.379));
    // Both fall: gravity is seen twice, each mass once.
    let m = world(0.5, Some(true), Some(true));
    assert!(reads(&m, "gravity", 1.8, 0.0, 0.95) && reads(&m, "mass balloon", 0.9, 0.0, 0.909));
    // The ball falls, the balloon does not: the balloon's mass takes
    // more of the no than gravity, which the ball speaks for.
    let m = world(0.5, Some(true), Some(false));
    assert!(reads(&m, "gravity", 0.9, 0.037, 0.879));
    assert!(reads(&m, "mass ball", 0.9, 0.0, 0.909));
    assert!(reads(&m, "mass balloon", 0.0, 0.273, 0.212));
    let suspects = ["mass balloon", "throw balloon", "gravity"];
    assert_eq!(conflicts(&m), names(&suspects));
    // Neither falls: gravity is doubted twice, each mass once.
    let m = world(0.5, Some(false), Some(false));
    assert!(opinion(&m, &p("gravity")).s > opinion(&m, &p("mass ball")).s);
}

/// Scenario 6, the sweep (`examples/sweep.rs`, `docs/sweep.md`): many
/// objects fall at the same speed and a feather does not. Each
/// measurement has a voice of its own.
#[test]
fn a_hundred_objects_and_a_feather() {
    // (object, measurements that pass, that fail)
    let world = |objects: &[(&str, usize, usize)]| {
        let mut m = Model {
            base: base(&[("gravity", 0.5)]),
            ..Model::default()
        };
        for (name, pass, fail) in objects {
            let (mass, whole) = (format!("mass {name}"), format!("same speed {name}"));
            m.related.push(rests(&[&whole, "gravity", &mass]));
            m.base.push((p(&mass), 0.5));
            for k in 0..pass + fail {
                let drop = format!("drop {name} {k}");
                m.evidence.push(says(&drop, &whole, k < *pass, &drop));
                m.base.push((p(&drop), 0.9));
            }
        }
        sure(m)
    };
    // A hundred objects pass once, the feather fails once: gravity
    // stands, the feather's own piece takes the no, its measurement
    // stays believed.
    let names: Vec<String> = (0..100).map(|k| format!("o{k}")).collect();
    let mut objects: Vec<(&str, usize, usize)> = names.iter().map(|n| (n.as_str(), 1, 0)).collect();
    objects.push(("feather", 0, 1));
    let m = world(&objects);
    let at = |x: &str| projected(opinion(&m, &p(x)));
    assert!(at("gravity") > 0.99 && close(at("mass o0"), 0.909));
    assert!(at("mass feather") < 0.17 && at("drop feather 0") > 0.8);
    // An object that passes 19 of 20: the one measurement is the odd
    // one. A feather that fails 20 of 20: the object is.
    let m = world(&[("stone", 19, 1), ("feather", 0, 20)]);
    let (mass, drop) = (
        opinion(&m, &p("mass stone")),
        opinion(&m, &p("drop stone 19")),
    );
    assert!(drop.s > mass.s && projected(mass) > 0.9 && projected(drop) < 0.3);
    let (mass, drop) = (
        opinion(&m, &p("mass feather")),
        opinion(&m, &p("drop feather 0")),
    );
    assert!(projected(mass) < 0.1 && projected(drop) > 0.8);
}

/// A trial, not yet in the core (`docs/complete.md`): one more piece
/// per tie, `complete`, "the statements on this tie are all that
/// matter for it". Ten objects fall at the same speed, a feather does
/// not, and a scale says yes about the feather's mass: the no goes to
/// the feather's completeness, and gravity and the mass stand.
#[test]
fn a_weighed_feather_blames_its_completeness() {
    let mut m = Model {
        base: base(&[("gravity", 0.5)]),
        ..Model::default()
    };
    let names: Vec<String> = (0..10).map(|k| format!("o{k}")).collect();
    for (name, falls) in names
        .iter()
        .map(|n| (n.as_str(), true))
        .chain([("feather", false)])
    {
        let (mass, done, speed) = (
            format!("mass {name}"),
            format!("complete {name}"),
            format!("same speed {name}"),
        );
        m.related.push(rests(&[&speed, "gravity", &mass, &done]));
        m.base.extend(base(&[(&mass, 0.5), (&done, 0.5)]));
        for k in 0..10 {
            let drop = format!("drop {name} {k}");
            m.evidence.push(says(&drop, &speed, falls, &drop));
            m.base.push((p(&drop), 0.9));
        }
    }
    let at = |m: &Model, x: &str| projected(opinion(m, &p(x)));
    // No scale: nothing tells the mass from completeness.
    let bare = sure(m.clone());
    assert!(close(
        at(&bare, "mass feather"),
        at(&bare, "complete feather")
    ));
    m.evidence
        .push(says("scale", "mass feather", true, "scale"));
    m.base.push((p("scale"), 0.9));
    let m = sure(m);
    assert!(at(&m, "gravity") > 0.99 && at(&m, "complete o0") > 0.98);
    assert!(close(at(&m, "mass feather"), 0.678));
    assert!(close(at(&m, "complete feather"), 0.034));
}
