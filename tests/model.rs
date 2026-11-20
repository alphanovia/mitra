//! The claims of `model/model.mitra`, subject by subject, and the walk
//! over a model.

use mitra_core::model::*;

fn op(r: f64, s: f64, a: f64) -> Opinion {
    Opinion { r, s, a }
}
fn near(x: f64, y: f64) -> bool {
    (x - y).abs() < 1e-9
}
fn p_of(model: &Model, name: &str) -> f64 {
    projected(opinion(model, &Piece::named(name)))
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

// mitra: evidence for model::base_rate_survives
#[test]
fn base_rate_survives() {
    let model = sure(Model {
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

// mitra: evidence for piece::no_evidence_is_the_base_rate
#[test]
fn no_evidence_is_the_base_rate() {
    assert_eq!(projected(nothing(0.5)), 0.5);
}

// mitra: evidence for evidence::review_lifts_a_test
#[test]
fn review_lifts_a_test() {
    assert_eq!(
        projected(fuse(nothing(0.5), moved(nothing(0.5), yes()), 1.0)),
        0.6
    );
}

// mitra: evidence for evidence::believed_at_zero_moves_nothing
#[test]
fn believed_at_zero_moves_nothing() {
    assert_eq!(moved(nothing(0.0), yes()), op(0.0, 0.0, 1.0));
    assert_eq!(moved(nothing(0.0), no()), op(0.0, 0.0, 0.0));
}

// mitra: evidence for evidence::not_knowing_is_no_review
#[test]
fn not_knowing_is_no_review() {
    assert_eq!(
        projected(fuse(nothing(0.5), moved(nothing(0.0), yes()), 1.0)),
        0.5
    );
}

// mitra: evidence for evidence::wrong_is_disbelief
#[test]
fn wrong_is_disbelief() {
    assert_eq!(
        projected(fuse(nothing(0.5), moved(nothing(1.0), no()), 1.0)),
        1.0 / 3.0
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

// mitra: evidence for model::two_halves_are_a_quarter
#[test]
fn two_halves_are_a_quarter() {
    assert!(near(projected(all(nothing(0.5), nothing(0.5))), 0.25));
}

// mitra: evidence for model::one_sure_part_changes_nothing
#[test]
fn one_sure_part_changes_nothing() {
    assert_eq!(all(nothing(1.0), op(0.5, 0.0, 0.5)), op(0.5, 0.0, 0.5));
}

// mitra: evidence for piece::policy_zero_still_needs_belief
#[test]
fn policy_zero_still_needs_belief() {
    assert!(!reaches(nothing(0.0), 0.0));
    assert!(reaches(op(0.5, 0.0, 0.0), 0.0));
}

// mitra: evidence for piece::sure_is_sure
#[test]
fn sure_is_sure() {
    assert_eq!((projected(yes()), projected(no())), (1.0, 0.0));
}

// mitra: evidence for piece::as_much_against_as_for_is_against
#[test]
fn as_much_against_as_for_is_against() {
    assert!(against(op(1.0, 1.0, 0.0)));
    assert!(!against(op(2.0, 1.0, 0.0)) && !against(nothing(0.0)));
}

// mitra: evidence for model::an_empty_model_believes_nothing
#[test]
fn an_empty_model_believes_nothing() {
    assert_eq!(opinion(&Model::default(), &Piece::named("x")), nothing(0.0));
}

// mitra: evidence for model::nothing_said_is_no_conflict
#[test]
fn nothing_said_is_no_conflict() {
    assert!(conflicts(&Model::default()).is_empty());
}

// mitra: evidence for evidence::one_failing_test_is_against
#[test]
fn one_failing_test_is_against() {
    assert!(against(moved(nothing(0.5), no())));
    assert!(!against(moved(nothing(0.0), no())));
}

// mitra: evidence for model::evidence_moves_its_piece
#[test]
fn evidence_moves_its_piece() {
    let model = Model {
        evidence: vec![says("t", "c", yes(), "a")],
        related: vec![],
        base: base_of(&[("t", 0.5)]),
    };
    assert_eq!(opinion(&model, &Piece::named("c")), op(0.5, 0.0, 0.0));
}

// mitra: evidence for model::a_review_reaches_the_claim
#[test]
fn a_review_reaches_the_claim() {
    let model = Model {
        evidence: vec![says("t", "c", yes(), "a"), says("r", "t", yes(), "b")],
        related: vec![],
        base: base_of(&[("t", 0.5), ("r", 0.5)]),
    };
    // The test reads 0.5, then 0.6 with half a yes from its review.
    assert_eq!(opinion(&model, &Piece::named("c")).r, 0.6);
}

// mitra: evidence for model::a_test_judged_wrong_says_less
#[test]
fn a_test_judged_wrong_says_less() {
    let model = Model {
        evidence: vec![says("t", "c", yes(), "a"), says("r", "t", no(), "b")],
        related: vec![],
        base: base_of(&[("t", 0.5), ("r", 1.0)]),
    };
    // One sure no on the test: it reads 1/3, no longer 0.5.
    assert_eq!(opinion(&model, &Piece::named("c")), op(1.0 / 3.0, 0.0, 0.0));
}

// mitra: evidence for model::a_whole_of_one_part_is_that_part
#[test]
fn resting_on_one_piece_is_that_piece() {
    let model = sure(Model {
        evidence: vec![says("t", "y", yes(), "a")],
        related: vec![rests(&["x", "y"])],
        base: base_of(&[("t", 0.5)]),
    });
    assert_eq!(opinion(&model, &Piece::named("x")).r, 0.5);
}

// mitra: evidence for place::within_itself
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

// mitra: evidence for place::inside_is_within
#[test]
fn inside_is_within() {
    let (file, next) = (
        Place::at(&["src/lib.rs"]),
        Place::at(&["src/lib.rs", "next"]),
    );
    assert!(within(&next, &file) && !within(&file, &next));
}

// mitra: evidence for place::beside_is_not_within
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
        statement: Piece::named("x"),
        place: Place::at(place),
    }]
}

// mitra: evidence for tie::no_tie_is_nowhere
#[test]
fn no_tie_is_nowhere() {
    assert!(places(&[], &Piece::named("x")).is_empty());
    assert!(concerned(&[], &Place::at(&["src/lib.rs"])).is_empty());
}

// mitra: evidence for tie::tied_is_found
#[test]
fn tied_is_found() {
    let next = ["src/lib.rs", "next"];
    assert_eq!(places(&tie(&next), &Piece::named("x")), [Place::at(&next)]);
}

// mitra: evidence for tie::a_file_concerns_what_is_in_it
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
                        all(x, y),
                        all(y, x),
                        moved(x, y),
                    ] {
                        assert!(o.r >= 0.0 && o.s >= 0.0, "{x:?} {y:?} {o:?}");
                        let p = projected(o);
                        assert!((0.0..=1.0).contains(&p), "{x:?} {y:?} {o:?}");
                    }
                    // Both of two pieces reads as the product of the two.
                    let both = projected(all(x, y));
                    assert!(near(both, projected(x) * projected(y)), "{x:?} {y:?}");
                }
            }
        }
    }
}

#[test]
fn with_nothing_known_a_piece_is_at_its_base_rate() {
    let model = Model {
        base: base_of(&[("t", 0.6)]),
        ..Model::default()
    };
    assert_eq!(opinion(&model, &Piece::named("t")), nothing(0.6));
    assert_eq!(opinion(&model, &Piece::named("unheard"),), nothing(0.0));
}

/// The experiment: how many passing tests nobody reviewed hold a claim
/// at 0.9, each believed at 0.6, the claim expected at 0. Each is 0.6
/// of a yes against a weight of 2: thirty of them.
#[test]
fn thirty_voices_hold_a_claim_and_one_voice_never() {
    let world = |n: usize, one_voice: bool| {
        let mut model = Model::default();
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
    let p = |n, one| projected(opinion(&world(n, one), &Piece::named("c")));
    assert!(near(p(1, false), 0.6 / 2.6) && near(p(2, false), 1.2 / 3.2));
    assert!(p(29, false) < 0.9 && p(30, false) >= 0.9 - 1e-9);
    assert!(near(p(10, true), 0.6 / 2.6));
}

#[test]
fn a_piece_reached_by_two_paths_counts_once() {
    // x rests on y and z, which both rest on w, believed at 0.5.
    let model = sure(Model {
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
        evidence: vec![says("t1", "y", yes(), "a"), says("t2", "z", yes(), "b")],
        related: vec![rests(&["x", "y", "z"])],
        base: base_of(&[("t1", 0.5), ("t2", 0.5)]),
    });
    // Each part reads 0.5 / 2.5.
    assert!(near(p_of(&model, "x"), 0.2 * 0.2));
}

/// `x` is all of `y` alone, so what is said of one is said of the
/// other: two voices, 0.75 and 0.5 of a yes, add up for both.
#[test]
fn a_whole_of_one_part_shares_its_evidence() {
    let model = sure(Model {
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
        evidence: vec![says("t", "x", no(), "a")],
        related: vec![rests(&["x", "y", "z"])],
        base: base_of(&[("t", 1.0), ("y", 0.5), ("z", 0.5)]),
    });
    assert!(near(opinion(&model, &Piece::named("y")).s, 0.25));
    assert!(near(opinion(&model, &Piece::named("z")).s, 0.25));
    assert!(p_of(&model, "x") < 0.25);
}

// mitra: evidence for model::a_throw_that_falls_believes_both
// mitra: evidence for model::a_throw_that_does_not_fall_names_its_suspects
// mitra: evidence for model::an_equal_start_gives_equal_shares
// mitra: evidence for model::trusted_senses_blame_the_tie_and_the_statement_more
#[test]
fn a_throw_at_one_half() {
    let throw = |senses: f64, fell: bool| {
        let value = if fell { yes() } else { no() };
        sure(Model {
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
    assert!(near(no_on(0.5, "applies"), 0.1));
    // Senses trusted more: more on the tie and the statement.
    assert!(no_on(0.9, "applies") > no_on(0.9, "throw"));
    assert!(near(no_on(0.9, "applies"), 3.0 / 14.0));
    assert!((no_on(0.9, "throw") - 0.021739).abs() < 1e-6);
    let suspects = ["applies", "falls", "throw"].map(Piece::named).to_vec();
    assert_eq!(conflicts(&throw(0.5, false)), [suspects]);
}

// mitra: evidence for model::a_latent_statement_is_believed_as_far_as_its_relation
// mitra: evidence for model::a_relation_nobody_believes_relates_nothing
#[test]
fn a_latent_statement_is_believed_as_far_as_its_relation() {
    // `gravity` has no tie; `because` says "it falls" is all of
    // [gravity, mass]. I see it fall, for sure.
    let because = |belief: Option<f64>| {
        let mut model = Model {
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

// mitra: evidence for model::the_balloon_blames_its_tie
// mitra: evidence for model::silence_blames_the_noise
#[test]
fn what_is_shared_takes_the_blame_off() {
    let balloon = sure(Model {
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
    assert!(near(
        opinion(&balloon, &Piece::named("balloon applies")).s,
        0.125
    ));
    assert!(opinion(&balloon, &Piece::named("falls")).s < 0.125);
    let noise = sure(Model {
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
    assert!(near(opinion(&noise, &Piece::named("noise")).s, 0.125));
    assert!(opinion(&noise, &Piece::named("applies")).s < 0.125);
}

#[test]
fn pieces_that_lead_back_to_themselves_end() {
    let model = sure(Model {
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
