//! The trust goldens of main (`examples/trust/` there), as one table
//! over the calculus: one claim, one test, at most two reviews. The
//! expected words and numbers are those of docs/design/trust-calculus.md
//! on main. Cases 6 and 7 wait for the layers they need.

use mitra_core::model::*;

fn piece(name: &str) -> Piece {
    Piece::named(name)
}
fn says(name: &str, about: &str, value: Opinion, by: &str) -> Evidence {
    Evidence {
        piece: piece(name),
        about: piece(about),
        value,
        by: Voice::named(by),
    }
}

/// One claim `c`, one test `t` that passes or fails, and reviews of the
/// test: (it is correct, how sure).
fn world(bias: f64, passes: bool, reviews: &[(bool, f64)]) -> Model {
    let verdict = |yes_: bool| if yes_ { yes() } else { no() };
    let mut model = Model {
        evidence: vec![says("t", "c", verdict(passes), "machine:cargo")],
        related: vec![],
        base: vec![(piece("t"), bias)],
    };
    for (k, (correct, sure)) in reviews.iter().enumerate() {
        let r = format!("r{k}");
        model
            .evidence
            .push(says(&r, "t", verdict(*correct), &format!("agent:{k}")));
        model.base.push((piece(&r), *sure));
    }
    model
}

/// The word and the number of a claim: outside the core, written here
/// only to compare with the goldens.
fn word(model: &Model, x: &Piece, policy: f64) -> (&'static str, f64) {
    let o = opinion(model, x);
    match (trusted(model, x, policy), against(o)) {
        (true, _) => ("holds", projected(o)),
        (false, true) => ("fails", o.d),
        (false, false) => ("unverified", projected(o)),
    }
}

#[test]
fn one_claim_one_test() {
    // (case, bias, policy, passes, reviews, word, number)
    type Case = (
        u8,
        f64,
        f64,
        bool,
        &'static [(bool, f64)],
        &'static str,
        f64,
    );
    let cases: &[Case] = &[
        (1, 0.5, 0.9, true, &[], "unverified", 0.5),
        (2, 0.5, 0.9, true, &[(true, 1.0)], "holds", 1.0),
        (3, 0.5, 0.9, true, &[(true, 0.8)], "holds", 0.9),
        (4, 0.5, 0.9, false, &[], "fails", 0.5),
        (5, 0.5, 0.4, true, &[], "holds", 0.5),
        (
            8,
            0.5,
            0.9,
            true,
            &[(true, 1.0), (false, 1.0)],
            "unverified",
            0.5,
        ),
        (9, 0.5, 0.9, false, &[(true, 1.0)], "fails", 1.0),
        (10, 0.5, 0.9, true, &[(false, 1.0)], "unverified", 0.0),
        (11, 0.5, 0.9, true, &[(true, 0.5)], "unverified", 0.75),
        (12, 0.5, 0.9, true, &[(true, 0.3)], "unverified", 0.65),
        (14, 0.0, 0.9, true, &[], "unverified", 0.0),
        (15, 1.0, 0.9, true, &[], "holds", 1.0),
        (16, 0.5, 0.0, true, &[], "holds", 0.5),
        (17, 0.0, 0.0, true, &[], "unverified", 0.0),
        (18, 0.5, 1.0, true, &[(true, 0.99)], "unverified", 0.995),
        (19, 0.5, 1.0, true, &[(true, 1.0)], "holds", 1.0),
        (20, 0.5, 0.9, true, &[(true, 0.0)], "unverified", 0.5),
        (21, 0.0, 0.9, false, &[], "unverified", 0.0),
        (22, 0.5, 0.9, false, &[(false, 1.0)], "unverified", 0.0),
    ];
    for (case, bias, policy, passes, reviews, expected, number) in cases {
        let (said, n) = word(&world(*bias, *passes, reviews), &piece("c"), *policy);
        assert_eq!(said, *expected, "case {case}");
        assert!(
            (n - number).abs() < 1e-9,
            "case {case}: {n}, expected {number}"
        );
    }
}

/// Case 8: two sure reviews of one test disagree. Nothing is chosen:
/// the conflict names both.
#[test]
fn two_reviews_that_disagree_are_a_conflict() {
    let model = world(0.5, true, &[(true, 1.0), (false, 1.0)]);
    assert_eq!(conflicts(&model), [vec![piece("r0"), piece("r1")]]);
    assert!(conflicts(&world(0.5, true, &[(true, 1.0), (true, 0.5)])).is_empty());
}

/// Case 13: two claims that cannot both be right, each with a passing
/// test, one reviewed. The one on the unreviewed test comes first.
#[test]
fn two_claims_that_disagree_are_ranked() {
    let model = Model {
        evidence: vec![
            says("t_wraps", "wraps", yes(), "machine:cargo"),
            says("t_stops", "stops", yes(), "machine:cargo"),
            says("r", "t_wraps", yes(), "agent:a"),
        ],
        related: vec![Related {
            how: Relation::Exclude,
            pieces: vec![piece("wraps"), piece("stops")],
        }],
        base: vec![
            (piece("t_wraps"), 0.5),
            (piece("t_stops"), 0.5),
            (piece("r"), 1.0),
        ],
    };
    assert_eq!(conflicts(&model), [vec![piece("stops"), piece("wraps")]]);
    assert_eq!(word(&model, &piece("wraps"), 0.9), ("holds", 1.0));
}
