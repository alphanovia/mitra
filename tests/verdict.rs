//! The states and the confidence of `model/verdict.mitra`, on the
//! toys the counts were read on before: one claim and one test with
//! its reviews (main's goldens), the throwing toy, and the shapes of
//! `tests/structure.rs`.

use mitra_core::model::*;
use mitra_core::verdict::*;

const W: f64 = 0.2;
/// The settings these are recorded at: a statement and a tie are
/// expected at one half, a completeness at 0.6, a piece of evidence at
/// 0.9 unless written, and no voice is doubted.
const S: Settings = Settings {
    weight: W,
    statement: 0.5,
    evidence: 0.9,
    tie: 0.5,
    completeness: 0.6,
    reliability: 1.0,
};

fn p(name: &str) -> Piece {
    Piece::named(name)
}
fn says(name: &str, about: &str, yes_: bool, by: &str) -> Evidence {
    Evidence {
        piece: p(name),
        about: p(about),
        value: if yes_ { yes() } else { no() },
        by: Voice::named(by),
    }
}
fn all(pieces: &[&str]) -> Statement<Related> {
    Statement {
        piece: p(&format!("all: {}", pieces[0])),
        content: Related {
            how: Relation::All,
            pieces: pieces.iter().map(|x| p(x)).collect(),
        },
    }
}
fn model(evidence: Vec<Evidence>, related: Vec<Statement<Related>>, base: &[(&str, f64)]) -> Model {
    // The relations of these models are plain structure, at 1,
    // unless a base rate is written for one.
    let mut rates: Vec<(Piece, f64)> = base.iter().map(|(x, a)| (p(x), *a)).collect();
    for said in &related {
        if !rates.iter().any(|(x, _)| x == &said.piece) {
            rates.push((said.piece.clone(), 1.0));
        }
    }
    Model {
        evidence,
        related,
        base: rates,
        ties: vec![],
        settings: S,
    }
}
fn show(m: &Model, x: &str) -> String {
    shown(&verdict_of(m, &p(x)), W)
}
fn names(groups: &[Vec<Piece>]) -> Vec<Vec<String>> {
    let each = |group: &Vec<Piece>| group.iter().map(|x| x.name.clone()).collect();
    groups.iter().map(each).collect()
}

/// One claim `c`, one test `t` believed at `bias` before any review,
/// and reviews of the test: (it is correct, how far the review is
/// believed).
fn world(bias: f64, passes: bool, reviews: &[(bool, f64)]) -> Model {
    let mut m = model(
        vec![says("t", "c", passes, "machine:cargo")],
        vec![],
        &[("t", bias)],
    );
    for (k, (correct, sure)) in reviews.iter().enumerate() {
        let r = format!("r{k}");
        m.evidence
            .push(says(&r, "t", *correct, &format!("agent:{k}")));
        m.base.push((p(&r), *sure));
    }
    m
}

// mitra: evidence for verdict::joined_keeps_what_agrees
// mitra: evidence for verdict::true_and_false_is_a_conflict
// mitra: evidence for verdict::a_conflict_stays
#[test]
fn two_states_are_joined() {
    use State::*;
    for x in [Unknown, True, False, Conflict] {
        assert_eq!(joined(x, x), x);
        assert_eq!(joined(x, Unknown), x);
        assert_eq!(joined(Unknown, x), x);
        assert_eq!(joined(Conflict, x), Conflict);
    }
    assert_eq!(joined(True, False), Conflict);
    assert_eq!(joined(False, True), Conflict);
}

// mitra: evidence for verdict::nothing_said_is_unknown
#[test]
fn nothing_said_is_unknown() {
    let m = model(vec![], vec![all(&["lands", "applies", "falls"])], &[]);
    assert!(spread(&m).is_empty());
    assert_eq!(overall(&m), State::Unknown);
    assert_eq!(verdict_of(&m, &p("falls")).confidence, None);
}

/// Main's goldens: one claim, one test, its reviews. A passing test
/// makes the claim true and a failing one false; a review that agrees
/// with the test makes it count for more; a review against it is a
/// conflict on the test, and what to give up is listed.
#[test]
fn one_claim_one_test() {
    // (case, bias, passes, reviews, the claim, the test, the verdict)
    type Case = (
        u8,
        f64,
        bool,
        &'static [(bool, f64)],
        &'static str,
        &'static str,
        State,
    );
    let cases: &[Case] = &[
        (1, 0.5, true, &[], "true 0.86", "true 0.50", State::True),
        (
            2,
            0.5,
            true,
            &[(true, 1.0)],
            "true 0.91",
            "true 0.92",
            State::True,
        ),
        (
            3,
            0.5,
            true,
            &[(true, 0.8)],
            "true 0.91",
            "true 0.90",
            State::True,
        ),
        (4, 0.5, false, &[], "false 0.86", "true 0.50", State::False),
        (
            8,
            0.5,
            true,
            &[(true, 1.0), (false, 1.0)],
            "true 0.91",
            "conflict",
            State::Conflict,
        ),
        (
            9,
            0.5,
            false,
            &[(true, 1.0)],
            "false 0.91",
            "true 0.92",
            State::False,
        ),
        (
            10,
            0.5,
            true,
            &[(false, 1.0)],
            "true 0.86",
            "conflict",
            State::Conflict,
        ),
        (
            12,
            0.5,
            true,
            &[(true, 0.3)],
            "true 0.90",
            "true 0.80",
            State::True,
        ),
        (14, 0.0, true, &[], "unknown", "unknown", State::Unknown),
        (15, 1.0, true, &[], "true 0.92", "true 1.00", State::True),
        (
            22,
            0.5,
            false,
            &[(false, 1.0)],
            "false 0.86",
            "conflict",
            State::Conflict,
        ),
    ];
    for (case, bias, passes, reviews, claim, test, whole) in cases {
        let m = world(*bias, *passes, reviews);
        assert_eq!(show(&m, "c"), *claim, "case {case}");
        assert_eq!(show(&m, "t"), *test, "case {case}");
        assert_eq!(overall(&m), *whole, "case {case}");
    }
    // Case 8, two reviews that disagree: give up the one against, or
    // the test with the one for it.
    let m = world(0.5, true, &[(true, 1.0), (false, 1.0)]);
    assert_eq!(names(&suspects(&m)), [vec!["r1"], vec!["t", "r0"]]);
    // Case 10, one review against: the test is the lighter of the two.
    let m = world(0.5, true, &[(false, 1.0)]);
    assert_eq!(names(&suspects(&m)), [vec!["t"], vec!["r0"]]);
}

/// Case 13: two claims that cannot both be right, each with a passing
/// test, one reviewed. The unreviewed test is the first to give up.
#[test]
fn two_claims_that_disagree_are_ranked() {
    let m = model(
        vec![
            says("t_wraps", "wraps", true, "machine:cargo"),
            says("t_stops", "stops", true, "machine:cargo"),
            says("r", "t_wraps", true, "agent:a"),
            says("not both", "wraps and stops", false, "author"),
        ],
        vec![all(&["wraps and stops", "wraps", "stops"])],
        &[
            ("t_wraps", 0.5),
            ("t_stops", 0.5),
            ("r", 1.0),
            ("not both", 1.0),
        ],
    );
    assert_eq!(overall(&m), State::Conflict);
    assert_eq!(
        names(&suspects(&m)),
        [vec!["t_stops"], vec!["t_wraps"], vec!["not both"]]
    );
}

fn throw(name: &str, about: &str, fell: bool) -> Evidence {
    says(name, about, fell, &format!("me:{name}"))
}
/// The throwing toy: `lands` is all of the tie `applies` and the
/// statement `falls`.
fn one_throw(fell: bool) -> Model {
    model(
        vec![throw("throw", "lands", fell)],
        vec![all(&["lands", "applies", "falls"])],
        &[],
    )
}

/// One throw that falls makes the tie and the statement true. One
/// that does not refutes neither: the two cannot both be true.
// mitra: evidence for verdict::only_a_verdict_that_holds_is_sure
#[test]
fn one_throw_alone() {
    let m = one_throw(true);
    assert_eq!(overall(&m), State::True);
    let (_, n) = least_sure(&m).unwrap();
    assert_eq!(format!("{n:.2}"), "0.91");
    assert_eq!(least_sure(&one_throw(false)), None);
    assert_eq!(show(&m, "applies"), "true 0.91");
    assert_eq!(show(&m, "falls"), "true 0.91");
    let m = one_throw(false);
    assert_eq!(overall(&m), State::False);
    assert_eq!(show(&m, "lands"), "false 0.91");
    assert_eq!(show(&m, "falls"), "unknown");
    assert_eq!(names(&suspects(&m)), [vec!["applies", "falls"]]);
}

/// The same throw by two voices is surer; by one voice twice it is
/// not.
#[test]
fn a_voice_counts_once() {
    let rests = vec![all(&["lands", "applies", "falls"])];
    let two = vec![throw("a", "lands", true), throw("b", "lands", true)];
    assert_eq!(show(&model(two, rests.clone(), &[]), "falls"), "true 0.95");
    let same = vec![
        says("a", "lands", true, "me"),
        says("b", "lands", true, "me"),
    ];
    assert_eq!(show(&model(same, rests, &[]), "falls"), "true 0.91");
}

/// A ball falls and a balloon does not, under one statement: `falls`
/// is true through the ball, so the balloon's tie is false.
#[test]
fn ball_and_balloon() {
    let m = model(
        vec![
            throw("t1", "ball lands", true),
            throw("t2", "balloon lands", false),
        ],
        vec![
            all(&["ball lands", "ball applies", "falls"]),
            all(&["balloon lands", "balloon applies", "falls"]),
        ],
        &[],
    );
    assert_eq!(show(&m, "falls"), "true 0.91");
    assert_eq!(show(&m, "ball applies"), "true 0.91");
    assert_eq!(show(&m, "balloon applies"), "false 0.91");
    assert_eq!(overall(&m), State::False);
    assert!(suspects(&m).is_empty());
}

/// A latent statement: `gravity` is thrown at through two objects. The
/// ball tells it true, so the balloon's failure is its mass.
#[test]
fn gravity_and_two_masses() {
    let m = model(
        vec![
            throw("t1", "ball falls", true),
            throw("t2", "balloon falls", false),
        ],
        vec![
            all(&["ball falls", "gravity", "mass ball"]),
            all(&["balloon falls", "gravity", "mass balloon"]),
        ],
        &[],
    );
    assert_eq!(show(&m, "gravity"), "true 0.91");
    assert_eq!(show(&m, "mass balloon"), "false 0.91");
}

/// A reviewed statement stands, and the tie takes the no of a throw.
/// The review carries a count of a hundred.
#[test]
fn a_reviewed_statement_stands_and_the_tie_takes_the_no() {
    let mut m = one_throw(false);
    m.evidence.push(Evidence {
        value: Opinion { r: 100.0, ..yes() },
        ..says("physics", "falls", true, "human:newton")
    });
    assert_eq!(show(&m, "falls"), "true 1.00");
    assert_eq!(show(&m, "applies"), "false 0.91");
}

/// `orbit` and `tide` are each all of `gravity` and one more, and the
/// sky is both.
fn diamond(pass: bool) -> Model {
    model(
        vec![throw("seen", "sky", pass)],
        vec![
            all(&["sky", "orbit", "tide"]),
            all(&["orbit", "gravity", "moon"]),
            all(&["tide", "gravity", "sea"]),
        ],
        &[],
    )
}

/// A statement reached by two paths is counted once, and a no travels
/// down to the parts of a part: the suspects are what stands on
/// nothing.
#[test]
fn a_shared_part_counts_once() {
    let m = diamond(true);
    for x in ["gravity", "moon", "sea", "orbit", "tide"] {
        assert_eq!(show(&m, x), "true 0.91", "{x}");
    }
    let m = diamond(false);
    assert_eq!(show(&m, "gravity"), "unknown");
    assert_eq!(names(&suspects(&m)), [vec!["gravity", "moon", "sea"]]);
}

/// Evidence on the other parts moves the no onto the one left.
#[test]
fn evidence_on_the_other_parts_names_the_one_left() {
    let mut m = diamond(false);
    m.evidence.push(throw("gravity seen", "gravity", true));
    m.evidence.push(throw("moon seen", "moon", true));
    assert_eq!(show(&m, "orbit"), "true 0.91");
    assert_eq!(show(&m, "tide"), "false 0.91");
    assert_eq!(show(&m, "sea"), "false 0.91");
    assert_eq!(overall(&m), State::False);
}

/// A conflict marks what is contested and nothing else: the two
/// claims and their whole. Each test stays true.
#[test]
fn a_conflict_marks_only_what_is_contested() {
    let m = model(
        vec![
            says("t_wraps", "wraps", true, "machine:cargo"),
            says("t_stops", "stops", true, "machine:cargo"),
            says("not both", "wraps and stops", false, "author"),
            says("t_other", "other", true, "machine:cargo"),
        ],
        vec![all(&["wraps and stops", "wraps", "stops"])],
        &[],
    );
    assert_eq!(show(&m, "wraps and stops"), "conflict");
    assert_eq!(show(&m, "wraps"), "conflict");
    assert_eq!(show(&m, "other"), "true 0.91");
    assert_eq!(show(&m, "t_wraps"), "true 0.90");
    assert_eq!(overall(&m), State::Conflict);
}

/// A relation that is doubted is assumed like evidence: what follows
/// through it is no surer than it, and it is among what may be given
/// up.
#[test]
fn a_doubted_relation_is_a_suspect() {
    let rests = all(&["lands", "applies", "falls"]);
    let doubted = model(
        vec![throw("throw", "lands", true)],
        vec![rests.clone()],
        &[("all: lands", 0.5)],
    );
    assert_eq!(show(&doubted, "lands"), "true 0.91");
    assert_eq!(show(&doubted, "falls"), "true 0.86");
    let mut against = doubted.clone();
    against.evidence.push(throw("physics", "falls", false));
    assert_eq!(show(&against, "falls"), "conflict");
    assert_eq!(
        names(&suspects(&against)),
        [vec!["all: lands"], vec!["throw"], vec!["physics"]]
    );
}

/// Forty objects fall, each with its own tie and its own voice. The
/// statement is as sure as forty reports, and one feather that does
/// not fall refutes its own tie.
#[test]
fn many_reports_are_counted() {
    let mut evidence = Vec::new();
    let mut related = Vec::new();
    for k in 0..40 {
        let lands = format!("object {k} lands");
        evidence.push(throw(&format!("throw {k}"), &lands, true));
        related.push(all(&[&lands, &format!("object {k} applies"), "falls"]));
    }
    evidence.push(throw("feather", "feather lands", false));
    related.push(all(&["feather lands", "feather applies", "falls"]));
    let m = model(evidence, related, &[]);
    let falls = verdict_of(&m, &p("falls"));
    assert_eq!(falls.state, State::True);
    assert!((falls.confidence.unwrap().r - 36.0).abs() < 1e-9);
    assert_eq!(show(&m, "falls"), "true 1.00");
    assert_eq!(show(&m, "object 7 applies"), "true 0.91");
    assert_eq!(show(&m, "feather applies"), "false 0.91");
}
