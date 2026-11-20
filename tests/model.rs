//! The claims of `model/model.mitra`, subject by subject, and the walk
//! over a model.

use mitra_core::model::*;

fn op(b: f64, d: f64, u: f64, a: f64) -> Opinion {
    Opinion { b, d, u, a }
}
fn near(x: f64, y: f64) -> bool {
    (x - y).abs() < 1e-9
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
    for name in ["x", "y", "z", "t", "r", "unheard"] {
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
        projected(fuse(nothing(0.5), moved(nothing(0.75), yes()), 1.0)),
        0.875
    );
}

// mitra: evidence for evidence::believed_at_zero_moves_nothing
#[test]
fn believed_at_zero_moves_nothing() {
    assert_eq!(moved(nothing(0.0), yes()), op(0.0, 0.0, 1.0, 1.0));
    assert_eq!(moved(nothing(0.0), no()), op(0.0, 0.0, 1.0, 0.0));
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
        0.0
    );
}

// mitra: evidence for evidence::one_voice_twice_is_once
#[test]
fn one_voice_twice_is_once() {
    let half = op(0.5, 0.0, 0.5, 0.0);
    assert_eq!(fuse(half, half, 0.0), half);
}

// mitra: evidence for evidence::two_voices_add_up
#[test]
fn two_voices_add_up() {
    let half = op(0.5, 0.0, 0.5, 0.0);
    assert!(fuse(half, half, 1.0).b > 0.5);
}

// mitra: evidence for evidence::two_sure_reviews_that_disagree
#[test]
fn two_sure_reviews_that_disagree() {
    let (for_it, against_it) = (op(1.0, 0.0, 0.0, 0.5), op(0.0, 1.0, 0.0, 0.5));
    assert_eq!(fuse(for_it, against_it, 1.0), op(0.5, 0.5, 0.0, 0.5));
    assert_eq!(conflict(for_it, against_it), 1.0);
}

// mitra: evidence for model::two_halves_are_a_quarter
#[test]
fn two_halves_are_a_quarter() {
    let half = op(0.5, 0.0, 0.5, 0.0);
    assert_eq!(all(half, half).b, 0.25);
}

// mitra: evidence for model::one_sure_part_changes_nothing
#[test]
fn one_sure_part_changes_nothing() {
    assert_eq!(all(op(1.0, 0.0, 0.0, 0.0), op(0.5, 0.0, 0.5, 0.0)).b, 0.5);
}

// mitra: evidence for piece::policy_zero_still_needs_belief
#[test]
fn policy_zero_still_needs_belief() {
    assert!(!reaches(nothing(0.0), 0.0));
    assert!(reaches(op(0.5, 0.0, 0.5, 0.0), 0.0));
}

// mitra: evidence for piece::sure_is_sure
#[test]
fn sure_is_sure() {
    assert_eq!((projected(yes()), projected(no())), (1.0, 0.0));
}

// mitra: evidence for piece::as_much_against_as_for_is_against
#[test]
fn as_much_against_as_for_is_against() {
    assert!(against(op(0.5, 0.5, 0.0, 0.0)));
    assert!(!against(op(0.5, 0.25, 0.25, 0.0)) && !against(nothing(0.0)));
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
    assert_eq!(opinion(&model, &Piece::named("c")), op(0.5, 0.0, 0.5, 0.0));
}

// mitra: evidence for model::a_review_reaches_the_claim
#[test]
fn a_review_reaches_the_claim() {
    let model = Model {
        evidence: vec![says("t", "c", yes(), "a"), says("r", "t", yes(), "b")],
        related: vec![],
        base: base_of(&[("t", 0.5), ("r", 0.75)]),
    };
    assert_eq!(opinion(&model, &Piece::named("c")).b, 0.875);
}

// mitra: evidence for model::a_test_judged_wrong_says_nothing
#[test]
fn a_test_judged_wrong_says_nothing() {
    let model = Model {
        evidence: vec![says("t", "c", yes(), "a"), says("r", "t", no(), "b")],
        related: vec![],
        base: base_of(&[("t", 0.5), ("r", 1.0)]),
    };
    assert_eq!(opinion(&model, &Piece::named("c")), nothing(0.0));
}

// mitra: evidence for model::a_whole_of_one_part_is_that_part
#[test]
fn resting_on_one_piece_is_that_piece() {
    let model = sure(Model {
        evidence: vec![says("t", "y", yes(), "a")],
        related: vec![rests(&["x", "y"])],
        base: base_of(&[("t", 0.5)]),
    });
    assert_eq!(opinion(&model, &Piece::named("x")).b, 0.5);
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

#[test]
fn an_opinion_always_sums_to_one() {
    let grid = [0.0, 0.25, 0.5, 1.0];
    for b in grid {
        for u in grid.iter().filter(|u| b + **u <= 1.0) {
            let x = op(b, 1.0 - b - u, *u, 0.5);
            for i in grid {
                for o in [fuse(x, op(0.5, 0.25, 0.25, 0.5), i), all(x, x), moved(x, x)] {
                    assert!(near(o.b + o.d + o.u, 1.0), "{o:?}");
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
/// at 0.9, each believed at 0.6.
#[test]
fn six_voices_hold_a_claim_and_one_voice_never() {
    let world = |n: usize, one_voice: bool| {
        let mut model = Model::default();
        for k in 0..n {
            let (t, by) = (
                format!("t{k}"),
                if one_voice {
                    "a".to_string()
                } else {
                    format!("v{k}")
                },
            );
            model.evidence.push(says(&t, "c", yes(), &by));
            model.base.push((Piece { name: t }, 0.6));
        }
        model
    };
    let p = |n, one| projected(opinion(&world(n, one), &Piece::named("c")));
    assert!(near(p(1, false), 0.6) && near(p(2, false), 0.75));
    assert!(p(5, false) < 0.9 && p(6, false) >= 0.9 - 1e-9);
    assert!(near(p(10, true), 0.6));
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
    assert_eq!(opinion(&model, &Piece::named("x")).b, 0.5);
}

#[test]
fn a_piece_rests_on_all_its_parts() {
    let model = sure(Model {
        evidence: vec![says("t1", "y", yes(), "a"), says("t2", "z", yes(), "b")],
        related: vec![rests(&["x", "y", "z"])],
        base: base_of(&[("t1", 0.5), ("t2", 0.5)]),
    });
    assert_eq!(opinion(&model, &Piece::named("x")).b, 0.25);
}

/// `x` is all of `y` alone, so what is said of one is said of the
/// other: two voices, 0.75 and 0.5, add up to 0.8 for both.
#[test]
fn a_whole_of_one_part_shares_its_evidence() {
    let model = sure(Model {
        evidence: vec![says("t1", "y", yes(), "a"), says("t0", "x", yes(), "b")],
        related: vec![rests(&["x", "y"])],
        base: base_of(&[("t1", 0.5), ("t0", 0.75)]),
    });
    assert!(near(opinion(&model, &Piece::named("x")).b, 0.8));
    assert!(near(opinion(&model, &Piece::named("y")).b, 0.8));
}

/// What is said of a whole directly is never dropped for what its
/// parts are expected to be.
#[test]
fn a_no_about_a_whole_is_kept() {
    let model = sure(Model {
        evidence: vec![says("t", "x", no(), "a")],
        related: vec![rests(&["x", "y", "z"])],
        base: base_of(&[("t", 0.5), ("y", 1.0), ("z", 1.0)]),
    });
    assert_eq!(opinion(&model, &Piece::named("x")), op(0.0, 0.5, 0.5, 0.0));
}

// mitra: evidence for model::a_throw_that_falls_believes_both
// mitra: evidence for model::a_throw_that_does_not_fall_blames_neither_alone
// mitra: evidence for model::a_throw_that_does_not_fall_names_its_suspects
#[test]
fn a_throw_at_one_half() {
    let throw = |fell: bool| {
        sure(Model {
            evidence: vec![says(
                "throw",
                "lands",
                if fell { yes() } else { no() },
                "me",
            )],
            related: vec![rests(&["lands", "applies", "falls"])],
            base: base_of(&[("throw", 0.5)]),
        })
    };
    let half = op(0.5, 0.0, 0.5, 0.0);
    assert_eq!(opinion(&throw(true), &Piece::named("falls")), half);
    assert_eq!(opinion(&throw(true), &Piece::named("applies")), half);
    assert_eq!(opinion(&throw(false), &Piece::named("falls")), nothing(0.0));
    assert_eq!(
        opinion(&throw(false), &Piece::named("lands")),
        op(0.0, 0.5, 0.5, 0.0)
    );
    let suspects = ["applies", "falls", "throw"].map(Piece::named).to_vec();
    assert_eq!(conflicts(&throw(false)), [suspects]);
}

// mitra: evidence for model::a_latent_statement_is_believed_as_far_as_its_relation
// mitra: evidence for model::a_relation_nobody_believes_relates_nothing
#[test]
fn a_latent_statement_is_believed_as_far_as_its_relation() {
    // `gravity` has no tie; `because` says "falls" is all of [gravity].
    let because = |belief: Option<f64>| {
        let mut model = sure(Model {
            evidence: vec![says("throw", "lands", yes(), "me")],
            related: vec![rests(&["lands", "applies", "falls"])],
            base: base_of(&[("throw", 1.0)]),
        });
        model.related.push(Statement {
            piece: Piece::named("because"),
            ..rests(&["falls", "gravity"])
        });
        if let Some(a) = belief {
            model.base.push((Piece::named("because"), a));
        }
        opinion(&model, &Piece::named("gravity"))
    };
    assert_eq!(because(Some(1.0)), op(1.0, 0.0, 0.0, 0.0));
    assert_eq!(because(Some(0.5)), op(0.5, 0.0, 0.5, 0.0));
    assert_eq!(because(Some(0.0)), nothing(0.0));
    assert_eq!(because(None), nothing(0.0));
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
    assert_eq!(
        opinion(&model, &Piece::named("lands")),
        op(0.0, 1.0, 0.0, 0.0)
    );
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
        base: base_of(&[("throw ball", 0.5), ("throw balloon", 0.5)]),
    });
    assert_eq!(
        opinion(&balloon, &Piece::named("balloon applies")),
        op(0.0, 0.25, 0.75, 0.0)
    );
    let noise = sure(Model {
        evidence: vec![
            says("throw", "lands", yes(), "me"),
            says("listen", "bangs", no(), "me"),
        ],
        related: vec![
            rests(&["lands", "applies", "falls"]),
            rests(&["bangs", "applies", "noise"]),
        ],
        base: base_of(&[("throw", 0.5), ("listen", 0.5)]),
    });
    assert_eq!(
        opinion(&noise, &Piece::named("noise")),
        op(0.0, 0.25, 0.75, 0.0)
    );
}

#[test]
fn pieces_that_lead_back_to_themselves_end() {
    let model = sure(Model {
        evidence: vec![says("e", "f", yes(), "a"), says("f", "e", yes(), "b")],
        related: vec![rests(&["x", "y"]), rests(&["y", "x"])],
        base: vec![],
    });
    assert_eq!(opinion(&model, &Piece::named("x")).b, 0.0);
    assert_eq!(opinion(&model, &Piece::named("e")).b, 0.0);
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
