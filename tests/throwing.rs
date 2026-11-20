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
fn rests(pieces: &[&str]) -> Related {
    Related {
        how: Relation::RestsOn,
        pieces: pieces.iter().map(|x| p(x)).collect(),
    }
}

/// One throw: base rates of the tie and the statement, trust in my
/// senses, and what I saw.
fn single(a: f64, f: f64, s: f64, fell: bool) -> Model {
    Model {
        evidence: vec![says("throw", "lands", fell, "me")],
        related: vec![rests(&["lands", "applies", "falls"])],
        base: vec![(p("applies"), a), (p("falls"), f), (p("throw"), s)],
    }
}

fn base(rates: &[(&str, f64)]) -> Vec<(Piece, f64)> {
    rates.iter().map(|(x, a)| (p(x), *a)).collect()
}
fn near(o: Opinion, b: f64, d: f64) -> bool {
    (o.b - b).abs() < 1e-9 && (o.d - d).abs() < 1e-9
}

// What the core gives TODAY, pinned so that a change of the calculus
// shows here. `reads` says whether the row reads right; where it does
// not, docs/throwing.md says what was expected. Nothing below is a
// promise.

/// Scenario 1, one throw.
#[test]
fn one_throw() {
    // (tie, statement, senses, it fell) -> lands (b, d); reads right
    let rows = [
        // The extremes.
        (0.0, 0.0, 1.0, true, (1.0, 0.0), false), // seen for sure, and neither part is believed
        (0.0, 0.0, 1.0, false, (0.0, 1.0), false), // no one is told that one of the two is wrong
        (1.0, 1.0, 1.0, true, (1.0, 0.0), true),
        (1.0, 1.0, 1.0, false, (0.0, 0.0), false), // the throw is dropped: nothing is against, no conflict
        (1.0, 1.0, 0.0, false, (0.0, 0.0), true),  // senses not trusted: nothing moves
        (0.0, 0.0, 0.0, true, (0.0, 0.0), true),
        // The middle.
        (0.5, 0.5, 0.9, true, (0.9, 0.0), false), // the parts stay at 0.5
        (0.5, 0.5, 0.9, false, (0.0, 0.0), false), // dropped
        (0.9, 0.5, 0.6, false, (0.0, 0.0), false), // dropped, and no ranking
        (0.0, 0.0, 0.6, true, (0.6, 0.0), false), // the parts stay at 0
    ];
    for (a, f, s, fell, (b, d), _reads) in rows {
        let m = single(a, f, s, fell);
        let row = format!("{a} {f} {s} {fell}");
        assert!(near(opinion(&m, &p("lands")), b, d), "{row}");
        // In every row: a throw moves neither the tie nor the statement,
        // and no conflict is found.
        assert_eq!(opinion(&m, &p("applies")), nothing(a), "{row}");
        assert_eq!(opinion(&m, &p("falls")), nothing(f), "{row}");
        assert!(conflicts(&m).is_empty(), "{row}");
    }
}

/// Scenario 2, the same throw again. Reads right for the pair: one
/// voice counts once, six voices reach 0.9. The parts still do not move.
#[test]
fn repeated_throws() {
    let lands = |n: usize, one_voice: bool| {
        let mut m = Model {
            related: vec![rests(&["lands", "applies", "falls"])],
            ..Model::default()
        };
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
        assert_eq!(opinion(&m, &p("applies")), nothing(0.0));
        opinion(&m, &p("lands")).b
    };
    for n in [1, 2, 4] {
        assert!((lands(n, true) - 0.6).abs() < 1e-9);
    }
    for (n, b) in [(2, 0.75), (4, 6.0 / 7.0), (6, 0.9)] {
        assert!((lands(n, false) - b).abs() < 1e-9);
    }
}

/// Scenario 3, a ball and a balloon under one statement. The ball
/// falls, the balloon does not. Expected: `falls` is believed through
/// the ball, so the balloon's tie takes the blame. Today: no part moves.
#[test]
fn ball_and_balloon() {
    for (a, s, balloon_d) in [(0.0, 1.0, 1.0), (0.0, 0.9, 0.9), (0.5, 0.9, 0.0)] {
        let m = Model {
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
        };
        assert!(near(opinion(&m, &p("ball lands")), s, 0.0));
        assert!(near(opinion(&m, &p("balloon lands")), 0.0, balloon_d));
        for part in ["falls", "ball applies", "balloon applies"] {
            assert_eq!(opinion(&m, &p(part)), nothing(a), "{part}");
        }
        assert!(conflicts(&m).is_empty());
    }
}

/// Scenario 4, one object and two statements. It falls, and makes no
/// noise. Expected: the tie is believed through the fall, so `noise`
/// takes the blame. Today: no part moves.
#[test]
fn falls_and_noise() {
    for (a, s, bangs_d) in [(0.0, 1.0, 1.0), (0.0, 0.9, 0.9), (0.5, 0.9, 0.0)] {
        let m = Model {
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
        };
        assert!(near(opinion(&m, &p("lands")), s, 0.0));
        assert!(near(opinion(&m, &p("bangs")), 0.0, bangs_d));
        for part in ["applies", "falls", "noise"] {
            assert_eq!(opinion(&m, &p(part)), nothing(a), "{part}");
        }
        assert!(conflicts(&m).is_empty());
    }
}
