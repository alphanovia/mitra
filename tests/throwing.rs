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

fn names(pieces: &[&str]) -> Vec<Vec<Piece>> {
    match pieces.is_empty() {
        true => vec![],
        false => vec![pieces.iter().map(|x| p(x)).collect()],
    }
}

/// Scenario 1, one throw. The tie and the statement move alike: one
/// throw cannot tell them apart, only their base rates can.
#[test]
fn one_throw() {
    const SUSPECTS: [&str; 3] = ["applies", "falls", "throw"];
    // (tie, statement, senses, it fell) -> lands, each part, the suspects
    type Row = (
        f64,
        f64,
        f64,
        bool,
        (f64, f64),
        (f64, f64),
        &'static [&'static str],
    );
    let rows: [Row; 10] = [
        // The extremes.
        (0.0, 0.0, 1.0, true, (1.0, 0.0), (1.0, 0.0), &[]),
        (0.0, 0.0, 1.0, false, (0.0, 1.0), (0.0, 0.0), &SUSPECTS),
        (1.0, 1.0, 1.0, true, (1.0, 0.0), (1.0, 0.0), &[]),
        (1.0, 1.0, 1.0, false, (0.0, 1.0), (0.0, 0.0), &SUSPECTS),
        (1.0, 1.0, 0.0, false, (0.0, 0.0), (0.0, 0.0), &[]), // senses not trusted
        (0.0, 0.0, 0.0, true, (0.0, 0.0), (0.0, 0.0), &[]),
        // The middle.
        (0.5, 0.5, 0.9, true, (0.9, 0.0), (0.9, 0.0), &[]),
        (0.5, 0.5, 0.9, false, (0.0, 0.9), (0.0, 0.0), &SUSPECTS),
        (
            0.9,
            0.5,
            0.6,
            false,
            (0.0, 0.6),
            (0.0, 0.0),
            &["falls", "throw", "applies"],
        ),
        (0.0, 0.0, 0.6, true, (0.6, 0.0), (0.6, 0.0), &[]),
    ];
    for (a, f, s, fell, lands, part, suspects) in rows {
        let m = single(a, f, s, fell);
        let row = format!("{a} {f} {s} {fell}");
        assert!(near(opinion(&m, &p("lands")), lands.0, lands.1), "{row}");
        assert!(near(opinion(&m, &p("applies")), part.0, part.1), "{row}");
        assert!(near(opinion(&m, &p("falls")), part.0, part.1), "{row}");
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
    let rows: [(f64, f64, [&str; 3]); 3] = [
        (0.0, 1.0, ["balloon applies", "falls", "throw balloon"]),
        (0.0, 0.9, ["balloon applies", "falls", "throw balloon"]),
        (0.5, 0.9, ["balloon applies", "throw balloon", "falls"]),
    ];
    for (a, s, suspects) in rows {
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
        assert!(near(opinion(&m, &p("balloon lands")), 0.0, s));
        assert!(near(opinion(&m, &p("falls")), s, 0.0));
        assert!(near(opinion(&m, &p("ball applies")), s, 0.0));
        assert!(near(opinion(&m, &p("balloon applies")), 0.0, s * s));
        assert_eq!(conflicts(&m), names(&suspects));
    }
}

/// Scenario 4, one object and two statements. It falls, and makes no
/// noise: the tie is believed through the fall, so `noise` takes the
/// blame.
#[test]
fn falls_and_noise() {
    let rows: [(f64, f64, [&str; 3]); 3] = [
        (0.0, 1.0, ["noise", "applies", "listen"]),
        (0.0, 0.9, ["noise", "applies", "listen"]),
        (0.5, 0.9, ["noise", "listen", "applies"]),
    ];
    for (a, s, suspects) in rows {
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
        assert!(near(opinion(&m, &p("bangs")), 0.0, s));
        assert!(near(opinion(&m, &p("applies")), s, 0.0));
        assert!(near(opinion(&m, &p("falls")), s, 0.0));
        assert!(near(opinion(&m, &p("noise")), 0.0, s * s));
        assert_eq!(conflicts(&m), names(&suspects));
    }
}
