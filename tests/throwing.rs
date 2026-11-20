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
fn near(o: Opinion, b: f64, d: f64) -> bool {
    (o.b - b).abs() < 1e-9 && (o.d - d).abs() < 1e-9
}

fn close(x: f64, y: f64) -> bool {
    (x - y).abs() < 1e-4
}

fn names(pieces: &[&str]) -> Vec<Vec<Piece>> {
    match pieces.is_empty() {
        true => vec![],
        false => vec![pieces.iter().map(|x| p(x)).collect()],
    }
}

/// Scenario 1, one throw that falls. The tie and the statement move
/// alike: one throw cannot tell them apart.
#[test]
fn one_throw_that_falls() {
    // (tie, statement, senses) -> belief in the pair and in each part
    let rows = [
        (0.0, 0.0, 1.0, 1.0),
        (1.0, 1.0, 1.0, 1.0),
        (0.0, 0.0, 0.0, 0.0), // senses not trusted
        (0.5, 0.5, 0.9, 0.9),
        (0.0, 0.0, 0.6, 0.6),
    ];
    for (a, f, s, b) in rows {
        let m = single(a, f, s, true);
        for x in ["lands", "applies", "falls"] {
            assert!(near(opinion(&m, &p(x)), b, 0.0), "{a} {f} {s} {x}");
        }
        assert_eq!(conflicts(&m), names(&[]));
    }
}

/// Scenario 1, one throw that does not fall. The no is shared between
/// the tie, the statement and the throw: more to what was less
/// expected to be right, by `share`.
#[test]
fn one_throw_that_does_not_fall() {
    // (tie, statement, senses) -> disbelief in each of the three, the suspects
    type Row = (f64, f64, f64, [f64; 3], &'static [&'static str]);
    let all: &[&str] = &["applies", "falls", "throw"];
    let rows: [Row; 9] = [
        // An equal start gives equal shares.
        (0.5, 0.5, 0.5, [1.0 / 7.0; 3], all),
        (1.0, 1.0, 1.0, [1.0 / 3.0; 3], all),
        // Senses trusted more: the tie and the statement take more.
        (0.5, 0.5, 0.9, [9.0 / 31.0, 9.0 / 31.0, 1.0 / 31.0], all),
        (0.5, 0.5, 1.0, [1.0 / 3.0, 1.0 / 3.0, 0.0], all),
        // Senses trusted less: the throw takes more.
        (
            0.9,
            0.9,
            0.5,
            [0.045 / 0.595, 0.045 / 0.595, 0.405 / 0.595],
            &["throw", "applies", "falls"],
        ),
        (1.0, 1.0, 0.0, [0.0, 0.0, 1.0], &[]), // a throw nobody believes accuses nothing
        // Three different starts: the order of the suspects follows.
        (
            0.9,
            0.5,
            0.6,
            [0.03 / 0.73, 0.27 / 0.73, 0.18 / 0.73],
            &["falls", "throw", "applies"],
        ),
        // Two suspects expected wrong: no one of them alone is to blame.
        (0.0, 0.0, 1.0, [0.0; 3], all),
        (0.0, 0.0, 0.6, [0.0; 3], all),
    ];
    for (a, f, s, d, suspects) in rows {
        let m = single(a, f, s, false);
        let row = format!("{a} {f} {s}");
        for (x, d) in ["applies", "falls", "throw"].iter().zip(d) {
            assert!(near(opinion(&m, &p(x)), 0.0, d), "{row} {x}");
        }
        // The pair is disbelieved as far as the throw is still believed.
        let throw = projected(opinion(&m, &p("throw")));
        assert!(near(opinion(&m, &p("lands")), 0.0, throw), "{row}");
        assert_eq!(conflicts(&m), names(suspects), "{row}");
    }
}

/// Scenario 2, the same throw again: one voice counts once, six voices
/// reach 0.9, for the pair and for each part.
#[test]
fn repeated_throws() {
    let lands = |n: usize, one_voice: bool| {
        let mut m = sure(Model {
            related: vec![rests(&["lands", "applies", "falls"])],
            ..Model::default()
        });
        for k in 0..n {
            let by = if one_voice {
                "me".to_string()
            } else {
                format!("friend {k}")
            };
            m.evidence
                .push(says(&format!("throw {k}"), "lands", true, &by));
            m.base.push((p(&format!("throw {k}")), 0.6));
        }
        let pair = opinion(&m, &p("lands"));
        assert!(near(opinion(&m, &p("applies")), pair.b, 0.0));
        assert!(near(opinion(&m, &p("falls")), pair.b, 0.0));
        pair.b
    };
    for n in [1, 2, 4] {
        assert!((lands(n, true) - 0.6).abs() < 1e-9);
    }
    for (n, b) in [(2, 0.75), (4, 6.0 / 7.0), (6, 0.9)] {
        assert!((lands(n, false) - b).abs() < 1e-9);
    }
}

/// Scenario 3, a ball and a balloon under one statement. The ball
/// falls, the balloon does not: `falls` is believed through the ball,
/// so the balloon's tie takes the blame.
#[test]
fn ball_and_balloon() {
    // (base rate, senses) -> suspects, disbelief in the balloon's tie and throw
    let rows: [(f64, f64, [&str; 3], f64, f64); 3] = [
        (
            0.0,
            1.0,
            ["balloon applies", "falls", "throw balloon"],
            1.0,
            0.0,
        ),
        (
            0.0,
            0.9,
            ["balloon applies", "falls", "throw balloon"],
            0.81,
            0.0,
        ),
        (
            0.5,
            0.9,
            ["balloon applies", "throw balloon", "falls"],
            0.7467,
            0.0830,
        ),
    ];
    for (a, s, suspects, tie, throw) in rows {
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
                ("ball applies", a),
                ("balloon applies", a),
                ("falls", a),
                ("throw ball", s),
                ("throw balloon", s),
            ]),
        });
        assert!(near(opinion(&m, &p("ball lands")), s, 0.0));
        assert!(close(opinion(&m, &p("balloon applies")).d, tie));
        assert!(close(opinion(&m, &p("throw balloon")).d, throw));
        // The statement stays believed through the ball.
        assert!(projected(opinion(&m, &p("falls"))) >= s);
        assert!(opinion(&m, &p("falls")).d < 0.01);
        assert!(near(opinion(&m, &p("ball applies")), s, 0.0));
        assert_eq!(conflicts(&m), names(&suspects));
    }
}

/// Scenario 4, one object and two statements. It falls, and makes no
/// noise: the tie is believed through the fall, so `noise` takes the
/// blame.
#[test]
fn falls_and_noise() {
    let rows: [(f64, f64, [&str; 3], f64, f64); 3] = [
        (0.0, 1.0, ["noise", "applies", "listen"], 1.0, 0.0),
        (0.0, 0.9, ["noise", "applies", "listen"], 0.81, 0.0),
        (0.5, 0.9, ["noise", "listen", "applies"], 0.7467, 0.0830),
    ];
    for (a, s, suspects, noise, listen) in rows {
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
                ("applies", a),
                ("falls", a),
                ("noise", a),
                ("throw", s),
                ("listen", s),
            ]),
        });
        assert!(near(opinion(&m, &p("lands")), s, 0.0));
        assert!(close(opinion(&m, &p("noise")).d, noise));
        assert!(close(opinion(&m, &p("listen")).d, listen));
        // The tie stays believed through the fall.
        assert!(projected(opinion(&m, &p("applies"))) >= s);
        assert!(opinion(&m, &p("applies")).d < 0.01);
        assert!(near(opinion(&m, &p("falls")), s, 0.0));
        assert_eq!(conflicts(&m), names(&suspects));
    }
}
