//! The claims of `model/model.mitra`, subject by subject, and the walk
//! over a model.

use mitra_core::model::*;

/// The weight of a base rate that these are recorded at.
const W: f64 = 0.2;
/// What to expect of the completeness of a tie when nothing says.
const C: f64 = 0.6;

fn op(r: f64, s: f64, a: f64) -> Opinion {
    Opinion { r, s, a }
}
fn near(x: f64, y: f64) -> bool {
    (x - y).abs() < 1e-9
}
fn p_of(model: &Model, name: &str) -> f64 {
    projected(opinion(model, &Piece::named(name)), W)
}
fn says(name: &str, about: &str, value: Opinion, by: &str) -> Evidence {
    Evidence {
        piece: Piece::named(name),
        about: Piece::named(about),
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
fn base_of(rates: &[(&str, f64)]) -> Vec<(Piece, f64)> {
    rates.iter().map(|(p, a)| (Piece::named(p), *a)).collect()
}

// The claims of the model, one test each.

// mitra: evidence for tests::base_rate_survives
#[test]
fn base_rate_survives() {
    let model = sure(Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![says("t", "y", yes(), "a"), says("r", "t", no(), "b")],
        related: vec![rests(&["x", "y", "z"])],
        base: base_of(&[("t", 0.5), ("r", 0.25), ("x", 0.125), ("y", 0.75)]),
    });
    // `x` is a whole: its base rate is that of its parts together.
    for name in ["y", "z", "t", "r", "unheard"] {
        let x = Piece::named(name);
        assert_eq!(opinion(&model, &x).a, base(&model, &x), "{name}");
    }
}

// mitra: evidence for tests::no_evidence_is_the_base_rate
#[test]
fn no_evidence_is_the_base_rate() {
    assert_eq!(projected(nothing(0.5), W), 0.5);
}

// mitra: evidence for evidence::review_lifts_a_test
#[test]
fn review_lifts_a_test() {
    assert_eq!(
        projected(fuse(nothing(0.5), moved(nothing(0.5), yes(), W), 1.0), W),
        0.6 / 0.7
    );
}

// mitra: evidence for evidence::believed_at_zero_moves_nothing
#[test]
fn believed_at_zero_moves_nothing() {
    assert_eq!(moved(nothing(0.0), yes(), W), op(0.0, 0.0, 1.0));
    assert_eq!(moved(nothing(0.0), no(), W), op(0.0, 0.0, 0.0));
}

// mitra: evidence for evidence::not_knowing_is_no_review
#[test]
fn not_knowing_is_no_review() {
    assert_eq!(
        projected(fuse(nothing(0.5), moved(nothing(0.0), yes(), W), 1.0), W),
        0.5
    );
}

// mitra: evidence for evidence::wrong_is_disbelief
#[test]
fn wrong_is_disbelief() {
    assert_eq!(
        projected(fuse(nothing(0.5), moved(nothing(1.0), no(), W), 1.0), W),
        0.1 / 1.2
    );
}

// mitra: evidence for evidence::one_voice_twice_is_once
#[test]
fn one_voice_twice_is_once() {
    let half = op(0.5, 0.0, 0.0);
    assert_eq!(fuse(half, half, 0.0), half);
}

// mitra: evidence for evidence::two_voices_add_up
#[test]
fn two_voices_add_up() {
    let half = op(0.5, 0.0, 0.0);
    assert_eq!(fuse(half, half, 1.0).r, 1.0);
}

// mitra: evidence for evidence::two_sure_reviews_that_disagree
#[test]
fn two_sure_reviews_that_disagree() {
    let (for_it, against_it) = (op(1.0, 0.0, 0.5), op(0.0, 1.0, 0.5));
    assert_eq!(fuse(for_it, against_it, 1.0), op(1.0, 1.0, 0.5));
    assert_eq!(conflict(for_it, against_it), 1.0);
}

// mitra: evidence for tests::two_halves_are_a_quarter
#[test]
fn two_halves_are_a_quarter() {
    assert!(near(projected(all(nothing(0.5), nothing(0.5), W), W), 0.25));
}

// mitra: evidence for tests::one_sure_part_changes_nothing
#[test]
fn one_sure_part_changes_nothing() {
    assert_eq!(all(nothing(1.0), op(0.5, 0.0, 0.5), W), op(0.5, 0.0, 0.5));
}

// mitra: evidence for tests::policy_zero_still_needs_belief
#[test]
fn policy_zero_still_needs_belief() {
    assert!(!reaches(nothing(0.0), 0.0, W));
    assert!(reaches(op(0.5, 0.0, 0.0), 0.0, W));
}

// mitra: evidence for piece::sure_is_sure
#[test]
fn sure_is_sure() {
    assert_eq!((projected(yes(), W), projected(no(), W)), (1.0, 0.0));
}

// mitra: evidence for tests::as_much_against_as_for_is_against
#[test]
fn as_much_against_as_for_is_against() {
    assert!(against(op(1.0, 1.0, 0.0)));
    assert!(!against(op(2.0, 1.0, 0.0)) && !against(nothing(0.0)));
}

// mitra: evidence for tests::an_empty_model_believes_nothing
#[test]
fn an_empty_model_believes_nothing() {
    assert_eq!(
        opinion(&Model::empty(W, C), &Piece::named("x")),
        nothing(0.0)
    );
}

// mitra: evidence for tests::nothing_said_is_no_conflict
#[test]
fn nothing_said_is_no_conflict() {
    assert!(conflicts(&Model::empty(W, C)).is_empty());
}

// mitra: evidence for evidence::one_failing_test_is_against
#[test]
fn one_failing_test_is_against() {
    assert!(against(moved(nothing(0.5), no(), W)));
    assert!(!against(moved(nothing(0.0), no(), W)));
}

// mitra: evidence for tests::evidence_moves_its_piece
#[test]
fn evidence_moves_its_piece() {
    let model = Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![says("t", "c", yes(), "a")],
        related: vec![],
        base: base_of(&[("t", 0.5)]),
    };
    assert_eq!(opinion(&model, &Piece::named("c")), op(0.5, 0.0, 0.0));
}

// mitra: evidence for tests::a_review_reaches_the_claim
#[test]
fn a_review_reaches_the_claim() {
    let model = Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![says("t", "c", yes(), "a"), says("r", "t", yes(), "b")],
        related: vec![],
        base: base_of(&[("t", 0.5), ("r", 0.5)]),
    };
    // The test reads 0.5, then 0.86 with half a yes from its review.
    assert_eq!(opinion(&model, &Piece::named("c")).r, 0.6 / 0.7);
}

// mitra: evidence for tests::a_test_judged_wrong_says_less
#[test]
fn a_test_judged_wrong_says_less() {
    let model = Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![says("t", "c", yes(), "a"), says("r", "t", no(), "b")],
        related: vec![],
        base: base_of(&[("t", 0.5), ("r", 1.0)]),
    };
    // One sure no on the test: it reads 0.08, no longer 0.5.
    assert_eq!(opinion(&model, &Piece::named("c")), op(0.1 / 1.2, 0.0, 0.0));
}

// mitra: evidence for tests::a_whole_of_one_part_is_that_part
#[test]
fn resting_on_one_piece_is_that_piece() {
    let model = sure(Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![says("t", "y", yes(), "a")],
        related: vec![rests(&["x", "y"])],
        base: base_of(&[("t", 0.5)]),
    });
    assert_eq!(opinion(&model, &Piece::named("x")).r, 0.5);
}

// mitra: evidence for tests::within_itself
#[test]
fn within_itself() {
    for a in [
        Place::at(&[]),
        Place::at(&["src/lib.rs"]),
        Place::at(&["src/lib.rs", "next"]),
    ] {
        assert!(within(&a, &a));
    }
}

// mitra: evidence for tests::inside_is_within
#[test]
fn inside_is_within() {
    let (file, next) = (
        Place::at(&["src/lib.rs"]),
        Place::at(&["src/lib.rs", "next"]),
    );
    assert!(within(&next, &file) && !within(&file, &next));
}

// mitra: evidence for tests::beside_is_not_within
#[test]
fn beside_is_not_within() {
    assert!(!within(
        &Place::at(&["src/lib.rs", "next"]),
        &Place::at(&["src/lib.rs", "back"])
    ));
}

fn tie(place: &[&str]) -> Vec<Tie> {
    vec![Tie {
        piece: Piece::named("t"),
        statements: vec![Piece::named("x")],
        place: Place::at(place),
    }]
}

// mitra: evidence for tests::no_tie_is_nowhere
#[test]
fn no_tie_is_nowhere() {
    assert!(places(&[], &Piece::named("x")).is_empty());
    assert!(concerned(&[], &Place::at(&["src/lib.rs"])).is_empty());
}

// mitra: evidence for tests::tied_is_found
#[test]
fn tied_is_found() {
    let next = ["src/lib.rs", "next"];
    assert_eq!(places(&tie(&next), &Piece::named("x")), [Place::at(&next)]);
}

// mitra: evidence for tests::a_file_concerns_what_is_in_it
#[test]
fn a_file_concerns_what_is_in_it() {
    let ties = tie(&["src/lib.rs", "next"]);
    assert_eq!(
        concerned(&ties, &Place::at(&["src/lib.rs"])),
        [Piece::named("x")]
    );
    assert!(concerned(&ties, &Place::at(&["src/main.rs"])).is_empty());
}

/// A tie is a piece: evidence about it moves it like any other.
#[test]
fn a_tie_is_believed_like_any_piece() {
    let model = Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![says("check", "t", no(), "machine:reader")],
        related: vec![],
        base: base_of(&[("t", 0.5), ("check", 1.0)]),
    };
    assert!(against(opinion(&model, &tie(&["src/lib.rs"])[0].piece)));
}

// The walk: what the promises say of `opinion`, on small worlds.

/// Counts are never negative, and what an opinion reads as stays
/// between 0 and 1, whatever is fused, moved or taken together.
#[test]
fn counts_are_never_negative() {
    let grid = [0.0, 0.25, 0.5, 1.0, 3.0];
    for r in grid {
        for s in grid {
            for a in [0.0, 0.3, 1.0] {
                let x = op(r, s, a);
                for y in [
                    op(0.5, 0.25, 0.5),
                    nothing(0.0),
                    nothing(1.0),
                    op(2.0, 0.0, 0.9),
                ] {
                    for o in [
                        fuse(x, y, 0.0),
                        fuse(x, y, 1.0),
                        all(x, y, W),
                        all(y, x, W),
                        moved(x, y, W),
                    ] {
                        assert!(o.r >= 0.0 && o.s >= 0.0, "{x:?} {y:?} {o:?}");
                        let p = projected(o, W);
                        assert!((0.0..=1.0).contains(&p), "{x:?} {y:?} {o:?}");
                    }
                    // Both of two pieces reads as the product of the two.
                    let both = projected(all(x, y, W), W);
                    assert!(near(both, projected(x, W) * projected(y, W)), "{x:?} {y:?}");
                }
            }
        }
    }
}

#[test]
fn with_nothing_known_a_piece_is_at_its_base_rate() {
    let model = Model {
        base: base_of(&[("t", 0.6)]),
        ..Model::empty(W, C)
    };
    assert_eq!(opinion(&model, &Piece::named("t")), nothing(0.6));
    assert_eq!(opinion(&model, &Piece::named("unheard"),), nothing(0.0));
}

/// The experiment: how many passing tests nobody reviewed hold a claim
/// at 0.9, each believed at 0.6, the claim expected at 0. Each is 0.6
/// of a yes against a weight of 0.2: three of them.
#[test]
fn three_voices_hold_a_claim_and_one_voice_never() {
    let world = |n: usize, one_voice: bool| {
        let mut model = Model::empty(W, C);
        for k in 0..n {
            let t = format!("t{k}");
            let by = match one_voice {
                true => "a".to_string(),
                false => format!("v{k}"),
            };
            model.evidence.push(says(&t, "c", yes(), &by));
            model.base.push((Piece { name: t }, 0.6));
        }
        model
    };
    let p = |n, one| projected(opinion(&world(n, one), &Piece::named("c")), W);
    assert!(near(p(1, false), 0.6 / 0.8) && near(p(2, false), 1.2 / 1.4));
    assert!(p(2, false) < 0.9 && p(3, false) >= 0.9 - 1e-9);
    assert!(near(p(10, true), 0.6 / 0.8));
}

#[test]
fn a_piece_reached_by_two_paths_counts_once() {
    // x rests on y and z, which both rest on w, believed at 0.5.
    let model = sure(Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![says("t", "w", yes(), "a")],
        related: vec![
            rests(&["x", "y", "z"]),
            rests(&["y", "w"]),
            rests(&["z", "w"]),
        ],
        base: base_of(&[("t", 0.5)]),
    });
    assert_eq!(opinion(&model, &Piece::named("x")).r, 0.5);
}

#[test]
fn a_piece_rests_on_all_its_parts() {
    let model = sure(Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![says("t1", "y", yes(), "a"), says("t2", "z", yes(), "b")],
        related: vec![rests(&["x", "y", "z"])],
        base: base_of(&[("t1", 0.5), ("t2", 0.5)]),
    });
    // Each part has half a yes on a base rate of 0.
    let part = 0.5 / (0.5 + W);
    assert!(near(p_of(&model, "x"), part * part));
}

/// `x` is all of `y` alone, so what is said of one is said of the
/// other: two voices, 0.75 and 0.5 of a yes, add up for both.
#[test]
fn a_whole_of_one_part_shares_its_evidence() {
    let model = sure(Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![says("t1", "y", yes(), "a"), says("t0", "x", yes(), "b")],
        related: vec![rests(&["x", "y"])],
        base: base_of(&[("t1", 0.5), ("t0", 0.75)]),
    });
    assert!(near(opinion(&model, &Piece::named("x")).r, 1.25));
    assert!(near(opinion(&model, &Piece::named("y")).r, 1.25));
}

/// A no about a whole goes to its parts, and the whole reads lower
/// than what its parts were expected to be together.
#[test]
fn a_no_about_a_whole_lowers_it() {
    let model = sure(Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![says("t", "x", no(), "a")],
        related: vec![rests(&["x", "y", "z"])],
        base: base_of(&[("t", 1.0), ("y", 0.5), ("z", 0.5)]),
    });
    let (y, z) = (
        opinion(&model, &Piece::named("y")),
        opinion(&model, &Piece::named("z")),
    );
    assert!(y == z && (y.s - 0.0769).abs() < 1e-3 && y.r == 0.0);
    assert!(p_of(&model, "x") < 0.25);
}

#[test]
fn a_throw_at_one_half() {
    let throw = |senses: f64, fell: bool| {
        let value = if fell { yes() } else { no() };
        sure(Model {
            ties: vec![],
            weight: W,
            completeness: C,
            evidence: vec![says("throw", "lands", value, "me")],
            related: vec![rests(&["lands", "applies", "falls"])],
            base: base_of(&[("applies", 0.5), ("falls", 0.5), ("throw", senses)]),
        })
    };
    // It falls: half a yes on the tie and on the statement.
    let half = op(0.5, 0.0, 0.5);
    assert_eq!(opinion(&throw(0.5, true), &Piece::named("falls")), half);
    assert_eq!(opinion(&throw(0.5, true), &Piece::named("applies")), half);
    // It does not: an equal start gives equal counts of no.
    let no_on = |senses: f64, x: &str| opinion(&throw(senses, false), &Piece::named(x)).s;
    assert_eq!(no_on(0.5, "applies"), no_on(0.5, "throw"));
    assert_eq!(no_on(0.5, "applies"), no_on(0.5, "falls"));
    assert!((no_on(0.5, "applies") - 0.02703).abs() < 1e-4);
    // Senses trusted more: more on the tie and the statement.
    assert!(no_on(0.9, "applies") > no_on(0.9, "throw"));
    assert!((no_on(0.9, "applies") - 0.06383).abs() < 1e-4);
    assert!((no_on(0.9, "throw") - 0.005525).abs() < 1e-4);
    let suspects = ["applies", "falls", "throw"].map(Piece::named).to_vec();
    assert_eq!(conflicts(&throw(0.5, false)), [suspects]);
}

// mitra: evidence for tests::a_latent_statement_is_believed_as_far_as_its_relation
// mitra: evidence for tests::a_relation_nobody_believes_relates_nothing
#[test]
fn a_latent_statement_is_believed_as_far_as_its_relation() {
    // `gravity` has no tie; `because` says "it falls" is all of
    // [gravity, mass]. I see it fall, for sure.
    let because = |belief: Option<f64>| {
        let mut model = Model {
            ties: vec![],
            weight: W,
            completeness: C,
            evidence: vec![says("throw", "it falls", yes(), "me")],
            related: vec![Statement {
                piece: Piece::named("because"),
                ..rests(&["it falls", "gravity", "mass"])
            }],
            base: base_of(&[("gravity", 0.5), ("mass", 0.5), ("throw", 1.0)]),
        };
        if let Some(a) = belief {
            model.base.push((Piece::named("because"), a));
        }
        opinion(&model, &Piece::named("gravity"))
    };
    assert_eq!(because(Some(1.0)), op(1.0, 0.0, 0.5));
    assert_eq!(because(Some(0.5)), op(0.5, 0.0, 0.5));
    assert_eq!(because(Some(0.0)), nothing(0.5));
    assert_eq!(because(None), nothing(0.5));
}

/// A relation that can be doubted is one of the suspects when what it
/// relates fails.
#[test]
fn a_doubted_relation_is_a_suspect() {
    let mut model = Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![says("throw", "lands", no(), "me")],
        related: vec![Statement {
            piece: Piece::named("pair"),
            ..rests(&["lands", "applies", "falls"])
        }],
        base: base_of(&[
            ("throw", 1.0),
            ("pair", 0.5),
            ("applies", 0.75),
            ("falls", 0.75),
        ]),
    };
    let suspects = |names: &[&str]| vec![names.iter().map(|n| Piece::named(n)).collect::<Vec<_>>()];
    assert_eq!(
        conflicts(&model),
        suspects(&["pair", "applies", "falls", "throw"])
    );
    // What is said of the pair itself does not go through the relation.
    assert_eq!(opinion(&model, &Piece::named("lands")), op(0.0, 1.0, 0.0));
    model.base[1].1 = 1.0;
    assert_eq!(conflicts(&model), suspects(&["applies", "falls", "throw"]));
}

// mitra: evidence for tests::silence_blames_the_noise
#[test]
fn what_is_shared_takes_the_blame_off() {
    let balloon = sure(Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![
            says("throw ball", "ball lands", yes(), "me"),
            says("throw balloon", "balloon lands", no(), "me"),
        ],
        related: vec![
            rests(&["ball lands", "ball applies", "falls"]),
            rests(&["balloon lands", "balloon applies", "falls"]),
        ],
        base: base_of(&[
            ("ball applies", 0.5),
            ("balloon applies", 0.5),
            ("falls", 0.5),
            ("throw ball", 0.5),
            ("throw balloon", 0.5),
        ]),
    });
    // The statement is believed through the ball: it takes less.
    assert!(opinion(&balloon, &Piece::named("balloon applies")).s > 0.05);
    assert!(opinion(&balloon, &Piece::named("falls")).s < 0.02);
    let noise = sure(Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![
            says("throw", "lands", yes(), "me"),
            says("listen", "bangs", no(), "me"),
        ],
        related: vec![
            rests(&["lands", "applies", "falls"]),
            rests(&["bangs", "applies", "noise"]),
        ],
        base: base_of(&[
            ("applies", 0.5),
            ("falls", 0.5),
            ("noise", 0.5),
            ("throw", 0.5),
            ("listen", 0.5),
        ]),
    });
    assert!(opinion(&noise, &Piece::named("noise")).s > 0.05);
    assert!(opinion(&noise, &Piece::named("applies")).s < 0.02);
}

#[test]
fn pieces_that_lead_back_to_themselves_end() {
    let model = sure(Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![says("e", "f", yes(), "a"), says("f", "e", yes(), "b")],
        related: vec![rests(&["x", "y"]), rests(&["y", "x"])],
        base: vec![],
    });
    assert_eq!(opinion(&model, &Piece::named("x")).r, 0.0);
    assert_eq!(opinion(&model, &Piece::named("e")).r, 0.0);
}

#[test]
fn the_same_known_gives_the_same_opinion() {
    let model = Model {
        ties: vec![],
        weight: W,
        completeness: C,
        evidence: vec![says("t", "c", yes(), "a"), says("r", "t", yes(), "b")],
        related: vec![],
        base: base_of(&[("t", 0.5), ("r", 0.8)]),
    };
    assert_eq!(
        opinion(&model, &Piece::named("c")),
        opinion(&model, &Piece::named("c"))
    );
    assert_eq!(piece_id(&Piece::named("c")), "c");
}

// What was tried and kept (`docs/trials.md`).

#[test]
fn belief_disbelief_and_uncertainty_are_shares_of_one() {
    let o = op(3.0, 1.0, 0.5);
    let (b, d, u) = (belief(o, W), disbelief(o, W), uncertainty(o, W));
    assert!(near(b + d + u, 1.0));
    assert!(near(projected(o, W), b + o.a * u));
    assert!(belief(nothing(0.5), W) == 0.0 && disbelief(nothing(0.5), W) == 0.0);
}

#[test]
fn the_interval_narrows_with_the_counts() {
    let wide = interval(nothing(0.5), W);
    assert!(wide.0 < 0.01 && wide.1 > 0.99);
    let (few, many) = (
        interval(op(9.0, 1.0, 0.5), W),
        interval(op(90.0, 10.0, 0.5), W),
    );
    assert!(few.0 < many.0 && many.1 < few.1);
    for o in [op(9.0, 1.0, 0.5), op(90.0, 10.0, 0.5), op(1.0, 3.0, 0.5)] {
        let (from, to) = interval(o, W);
        assert!(from < projected(o, W) && projected(o, W) < to);
    }
    // Sure either way: nowhere else to be.
    assert_eq!(interval(nothing(1.0), W), (1.0, 1.0));
    assert_eq!(interval(nothing(0.0), W), (0.0, 0.0));
}

#[test]
fn the_weight_is_given_by_the_model() {
    let at = |weight: f64| {
        let model = Model {
            evidence: vec![says("t", "c", yes(), "a")],
            base: base_of(&[("t", 1.0), ("c", 0.5)]),
            ..Model::empty(weight, C)
        };
        projected(opinion(&model, &Piece::named("c")), weight)
    };
    assert!(near(at(0.2), 1.1 / 1.2) && near(at(2.0), 2.0 / 3.0));
}

/// The throwing toy with its tie written: `applies` ties `falls` to the
/// ball.
fn tied(throw: Opinion) -> Model {
    sure(Model {
        evidence: vec![says("throw", "lands", throw, "me")],
        related: vec![rests(&["lands", "applies", "falls"])],
        base: base_of(&[
            ("applies", 0.5),
            ("falls", 0.5),
            ("throw", 0.5),
            ("complete: applies", 0.5),
        ]),
        ties: vec![Tie {
            piece: Piece::named("applies"),
            statements: vec![Piece::named("falls")],
            place: Place {
                path: vec!["ball".to_string()],
            },
        }],
        weight: W,
        completeness: C,
    })
}

#[test]
fn a_tie_implies_its_completeness() {
    let model = tied(no());
    let tie = &model.ties[0];
    let done = complete(tie);
    assert_eq!(done.piece, Piece::named("complete: applies"));
    assert_eq!(done.content.tie, tie.piece);
    // A part of the whole its tie is a part of, though no relation names it.
    assert_eq!(parts(&model, &model.related[0].content).len(), 3);
    assert_eq!(conflicts(&model)[0].len(), 4);
    let s = |x: &str| opinion(&model, &Piece::named(x)).s;
    assert!(s("complete: applies") > 0.0 && near(s("complete: applies"), s("applies")));
    // A yes flows down to it as to any part.
    let fell = tied(yes());
    assert!(near(opinion(&fell, &done.piece).r, 0.5));
    // With no tie written, nothing is implied.
    let bare = Model {
        ties: vec![],
        ..model.clone()
    };
    assert_eq!(conflicts(&bare)[0].len(), 3);
}

/// A stone that always passes and a feather that never does, each
/// "same speed" being all of [gravity, its mass]; so many measurements
/// each, believed at 0.9. A voice reviews with so many yes.
fn dropped(times: usize, reviews: &[(&str, &str, f64)]) -> Model {
    let mut model = Model::empty(W, C);
    model.base = base_of(&[("gravity", 0.5), ("mass stone", 0.5), ("mass feather", 0.5)]);
    for (name, falls) in [("stone", yes()), ("feather", no())] {
        let speed = format!("same speed {name}");
        model
            .related
            .push(rests(&[&speed, "gravity", &format!("mass {name}")]));
        for k in 0..times {
            let drop = format!("drop {name} {k}");
            model.evidence.push(says(&drop, &speed, falls, &drop));
            model.base.push((Piece::named(&drop), 0.9));
        }
    }
    for (voice, about, count) in reviews {
        let review = format!("review by {voice}: {about}");
        model
            .evidence
            .push(says(&review, about, op(*count, 0.0, 0.5), voice));
        model.base.push((Piece::named(&review), 1.0));
    }
    sure(model)
}
fn doubt(mut model: Model, voice: &str, prior: f64) -> Model {
    model
        .base
        .push((reliable(&Voice::named(voice)).piece, prior));
    model
}

// mitra: evidence for tests::a_voice_with_no_base_rate_is_not_judged
#[test]
fn a_voice_never_doubted_is_not_judged() {
    let model = dropped(3, &[("sure", "mass feather", 100.0)]);
    assert!(judged(&model).is_empty());
    assert_eq!(settled(&model, &[]), model);
}

// mitra: evidence for evidence::a_word_never_exceeds_the_trust_in_its_voice
#[test]
fn a_word_never_exceeds_the_trust_in_its_voice() {
    assert!(near(
        projected(held(0.3, op(100.0, 0.0, 0.5), W), W),
        0.3 * (100.1 / 100.2) + 0.7 * 0.5
    ));
    let whole = held(1.0, op(3.0, 1.0, 0.5), W);
    assert!(near(whole.r, 3.0) && near(whole.s, 1.0));
    let nothing_said = held(0.0, op(3.0, 1.0, 0.5), W);
    assert!(nothing_said.r == 0.0 && nothing_said.s == 0.0);
}

#[test]
fn a_voice_is_judged_by_what_it_vouched_for() {
    let said = [
        ("loud", "mass feather", 100.0),
        ("loud", "mass stone", 100.0),
        ("quiet", "mass stone", 1.0),
    ];
    let model = doubt(doubt(dropped(5, &said), "loud", 0.9), "quiet", 0.9);
    let p = |x: &str| p_of(&model, x);
    // The feather contradicts the loud voice, the stone confirms the quiet one.
    assert!(p("reliable: loud") < 0.5 && p("reliable: quiet") > 0.9);
    // Once it is doubted its word stops holding the feather's mass up:
    // the mass takes the blame, and the measurements are believed.
    assert!(p("mass feather") < 0.5 && p("drop feather 0") > 0.5);
    // Never doubted, the same voice keeps the mass up and the
    // measurements take the blame.
    let trusting = dropped(5, &said);
    assert!(p_of(&trusting, "mass feather") > 0.9 && p_of(&trusting, "drop feather 0") < 0.5);
}

#[test]
fn repeated_failures_of_one_whole_are_one_fact() {
    // A voice that said "probably" is not destroyed by repetition.
    let at = |times: usize| {
        let model = doubt(
            dropped(times, &[("careful", "mass feather", 0.5)]),
            "careful",
            0.9,
        );
        p_of(&model, "reliable: careful")
    };
    assert!(at(30) > 0.5 && (at(3) - at(30)).abs() < 0.05);
}

#[test]
fn voices_on_one_source_fall_together() {
    // B only spoke of a rock nobody dropped. A is contradicted.
    let said = [("A", "mass feather", 100.0), ("B", "mass rock", 100.0)];
    let build = |shared: bool| {
        let mut model = doubt(doubt(dropped(5, &said), "A", 0.9), "B", 0.9);
        model.base.push((Piece::named("mass rock"), 0.5));
        if shared {
            for voice in ["A", "B"] {
                let rests_on = rests(&[&format!("reliable: {voice}"), "one lab"]);
                model.base.push((rests_on.piece.clone(), 1.0));
                model.related.push(rests_on);
            }
            model.base.push((Piece::named("one lab"), 0.9));
        }
        model
    };
    let (apart, together) = (build(false), build(true));
    assert!(near(p_of(&apart, "reliable: B"), 0.9));
    assert!(p_of(&together, "one lab") < 0.9);
    assert!(
        p_of(&together, "reliable: B") < 0.81
            && p_of(&together, "mass rock") < p_of(&apart, "mass rock")
    );
}

// mitra: evidence for evidence::reliability_names_its_voice
#[test]
fn reliability_names_its_voice() {
    let by = Voice::named("agent:a");
    assert_eq!(reliable(&by).content.voice, by);
    assert_eq!(reliable(&by).piece, Piece::named("reliable: agent:a"));
}

/// The throwing toy, the throw believed at 0.9 and failing, with a
/// review of a hundred yes on `falls` by `author`.
fn reviewed_throw(doubted: bool, ties: Vec<Tie>) -> Model {
    let mut model = sure(Model {
        evidence: vec![
            says("throw", "lands", no(), "me"),
            says("review", "falls", op(100.0, 0.0, 0.5), "author"),
        ],
        related: vec![rests(&["lands", "applies", "falls"])],
        base: base_of(&[
            ("applies", 0.5),
            ("falls", 0.5),
            ("throw", 0.9),
            ("review", 1.0),
        ]),
        ties,
        weight: W,
        completeness: C,
    });
    if doubted {
        model = doubt(model, "author", 0.9);
    }
    model
}

// mitra: evidence for tests::an_unwritten_completeness_has_the_setting
#[test]
fn an_unwritten_completeness_has_the_setting() {
    let tie = Tie {
        piece: Piece::named("applies"),
        statements: vec![Piece::named("falls")],
        place: Place {
            path: vec!["ball".to_string()],
        },
    };
    let model = reviewed_throw(false, vec![tie]);
    let of = |x: &str| base(&model, &Piece::named(x));
    assert_eq!(
        (of("complete: applies"), of("applies"), of("unknown")),
        (C, 0.5, 0.0)
    );
    // Written, it is what was written.
    let mut written = model.clone();
    written.base.push((Piece::named("complete: applies"), 0.9));
    assert_eq!(base(&written, &Piece::named("complete: applies")), 0.9);
}

// mitra: evidence for tests::what_is_reviewed_stands
// mitra: evidence for tests::a_failure_doubts_the_voice_that_vouched
// mitra: evidence for tests::a_doubted_voice_holds_its_word_up_less
#[test]
fn a_failure_doubts_the_voice_that_vouched() {
    let (trusting, doubting) = (reviewed_throw(false, vec![]), reviewed_throw(true, vec![]));
    let s = |m: &Model, x: &str| opinion(m, &Piece::named(x)).s;
    assert!(s(&trusting, "falls") < s(&trusting, "applies"));
    assert!(p_of(&doubting, "reliable: author") < 0.9);
    assert!(p_of(&doubting, "falls") < p_of(&trusting, "falls"));
}

/// The setting moves the completeness itself and little else: after a
/// failing throw it reads 0.55 at 0.6 and 0.65 at 0.7, and the
/// statement reads 0.43 and 0.42.
#[test]
fn completeness_at_six_and_at_seven_tenths() {
    let at = |completeness: f64, x: &str| {
        let model = Model {
            completeness,
            ..reviewed_throw(false, vec![])
        };
        let tie = Tie {
            piece: Piece::named("applies"),
            statements: vec![Piece::named("falls")],
            place: Place {
                path: vec!["ball".to_string()],
            },
        };
        let mut model = Model {
            ties: vec![tie],
            ..model
        };
        model.evidence.truncate(1);
        p_of(&model, x)
    };
    let close = |x: f64, y: f64| (x - y).abs() < 0.006;
    assert!(close(at(0.6, "complete: applies"), 0.55) && close(at(0.7, "complete: applies"), 0.65));
    assert!(close(at(0.6, "falls"), 0.43) && close(at(0.7, "falls"), 0.42));
}

// A tie with its statements: the whole is implied.

fn ties(name: &str, statements: &[&str], place: &str) -> Tie {
    Tie {
        piece: Piece::named(name),
        statements: statements.iter().map(|x| Piece::named(x)).collect(),
        place: Place {
            path: vec![place.to_string()],
        },
    }
}
/// The throwing toy as an author writes it: the tie `applies` links
/// the ball to `falls`. A throw is about the whole of the tie.
fn thrown(throw: Opinion, senses: f64) -> Model {
    Model {
        evidence: vec![says("throw", "whole: applies", throw, "me")],
        base: base_of(&[("applies", 0.5), ("falls", 0.5), ("throw", senses)]),
        ties: vec![ties("applies", &["falls"], "ball")],
        ..Model::empty(W, C)
    }
}

#[test]
fn a_tie_implies_its_whole() {
    let model = thrown(yes(), 0.5);
    assert_eq!(whole(&model.ties[0]), Piece::named("whole: applies"));
    let out = written(&model);
    assert_eq!(out.related.len(), 1);
    assert_eq!(written(&out), out);
    let names: Vec<String> = parts(&out, &out.related[0].content)
        .iter()
        .map(|x| x.name.clone())
        .collect();
    assert_eq!(names, ["applies", "falls", "complete: applies"]);
    // The relation is structure, and the completeness has the setting.
    assert_eq!(base(&model, &out.related[0].piece), 1.0);
    assert_eq!(base(&model, &Piece::named("complete: applies")), C);
}

// mitra: evidence for tests::a_throw_that_falls_believes_both
// mitra: evidence for tests::an_equal_start_gives_equal_shares
// mitra: evidence for tests::trusted_senses_blame_the_tie_and_the_statement_more
// mitra: evidence for tests::a_throw_that_does_not_fall_names_its_suspects
#[test]
fn a_throw_at_a_tie() {
    let fell = thrown(yes(), 0.5);
    for x in ["applies", "falls"] {
        assert_eq!(opinion(&fell, &Piece::named(x)), op(0.5, 0.0, 0.5));
    }
    assert_eq!(
        opinion(&fell, &Piece::named("complete: applies")),
        op(0.5, 0.0, C)
    );
    let missed = thrown(no(), 0.5);
    let s = |x: &str| opinion(&missed, &Piece::named(x)).s;
    assert!(near(s("applies"), s("falls")) && near(s("applies"), s("throw")));
    // Expected more, the completeness takes less of the no.
    assert!(s("complete: applies") < s("applies") && s("complete: applies") > 0.0);
    eprintln!(
        "GOLD {:?} {:?}",
        conflicts(&missed),
        ["applies", "falls", "complete: applies", "throw"]
            .map(|x| (p_of(&missed, x), p_of(&fell, x)))
    );
    let trusting = thrown(no(), 0.9);
    let s = |x: &str| opinion(&trusting, &Piece::named(x)).s;
    assert!(s("applies") > s("throw"));
}

// mitra: evidence for tests::the_balloon_blames_its_tie
#[test]
fn a_statement_under_two_ties() {
    // The ball falls, the balloon does not: `falls` is believed through
    // the ball, so the balloon's tie takes more of the no.
    let model = Model {
        evidence: vec![
            says("throw ball", "whole: ball applies", yes(), "me"),
            says("throw balloon", "whole: balloon applies", no(), "me"),
        ],
        base: base_of(&[
            ("ball applies", 0.5),
            ("balloon applies", 0.5),
            ("falls", 0.5),
            ("throw ball", 0.5),
            ("throw balloon", 0.5),
        ]),
        ties: vec![
            ties("ball applies", &["falls"], "ball"),
            ties("balloon applies", &["falls"], "balloon"),
        ],
        ..Model::empty(W, C)
    };
    let s = |x: &str| opinion(&model, &Piece::named(x)).s;
    assert!(s("balloon applies") > s("falls"));
    assert_eq!(places(&model.ties, &Piece::named("falls")).len(), 2);
}

#[test]
fn a_tie_of_several_statements() {
    // Gravity is listed by the tie of each object, beside its mass.
    let model = Model {
        evidence: vec![
            says("drop stone", "whole: stone", yes(), "a"),
            says("drop feather", "whole: feather", no(), "b"),
        ],
        base: base_of(&[
            ("stone", 0.5),
            ("feather", 0.5),
            ("gravity", 0.5),
            ("mass stone", 0.5),
            ("mass feather", 0.5),
            ("drop stone", 0.9),
            ("drop feather", 0.9),
        ]),
        ties: vec![
            ties("stone", &["gravity", "mass stone"], "stone"),
            ties("feather", &["gravity", "mass feather"], "feather"),
        ],
        ..Model::empty(W, C)
    };
    let p = |x: &str| p_of(&model, x);
    assert!(p("gravity") > 0.85 && p("mass feather") < 0.5 && p("mass stone") > 0.85);
    assert_eq!(conflicts(&model)[0].len(), 5);
}
