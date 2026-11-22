//! The claims of `model/model.mitra`: one piece `t`: {p} s {q}, and a
//! few shapes around it.

use mitra_core::model::*;

const S: Settings = Settings {
    weight: 0.2,
    bias: 0.5,
    reliability: 1.0,
};

/// The calculus reads pieces of any kind, so these are of any kind.
type Piece = SomePiece;
fn l(name: &str) -> Piece {
    leaf(name, "claim".to_string())
}
fn piece(name: &str, pre: &[Piece], post: &[Piece]) -> Piece {
    Piece {
        name: name.to_string(),
        pre: pre.to_vec(),
        about: "ensure".to_string(),
        post: post.to_vec(),
    }
}
fn t() -> Piece {
    piece("t", &[l("p")], &[l("q")])
}
fn took(x: &Piece, value: bool, count: f64, by: &str) -> Taking {
    Taking {
        piece: x.clone(),
        value,
        count,
        by: Voice::named(by),
        form: Form::Review,
    }
}
fn yes(x: &Piece) -> Taking {
    took(x, true, 1.0, &x.name)
}
fn no(x: &Piece) -> Taking {
    took(x, false, 1.0, &x.name)
}
fn with(pieces: &[Piece], taken: &[Taking]) -> Model {
    Model {
        pieces: pieces.to_vec(),
        taken: taken.to_vec(),
        ..Model::empty(S)
    }
}
fn at(taken: &[Taking], x: &Piece) -> State {
    opinion(&with(&[t()], taken), x).judged()
}
fn sure(m: &Model, x: &Piece) -> Option<f64> {
    opinion(m, x).confidence()
}

// mitra: evidence for location::within_is_the_path
#[test]
fn a_place_is_within_what_its_path_goes_on_from() {
    let (file, inner) = (Location::at(&["a.rs"]), Location::at(&["a.rs", "f"]));
    assert!(within(&inner, &file) && within(&file, &file) && !within(&file, &inner));
}

// mitra: evidence for piece::a_state_is_one_of_four
#[test]
fn joined_states() {
    use State::*;
    for x in [Unknown, True, False, Conflict] {
        assert_eq!(
            (joined(x, x), joined(x, Unknown), joined(Unknown, x)),
            (x, x, x)
        );
        assert_eq!(joined(Conflict, x), Conflict);
    }
    assert_eq!(joined(True, False), Conflict);
}

// mitra: evidence for calculus::what_is_taken_starts
// mitra: evidence for calculus::a_true_piece_passes_its_state_on
#[test]
fn deduction() {
    assert_eq!(at(&[yes(&t()), yes(&l("p"))], &l("q")), State::True);
    // A piece that is not true passes nothing on.
    assert_eq!(at(&[yes(&l("p"))], &l("q")), State::Unknown);
    assert_eq!(at(&[], &l("q")), State::Unknown);
}

// mitra: evidence for calculus::a_false_piece_after_refutes
#[test]
fn refutation() {
    assert_eq!(at(&[yes(&l("p")), no(&l("q"))], &t()), State::False);
    assert_eq!(at(&[yes(&t()), no(&l("q"))], &l("p")), State::False);
    // Neither known: both are suspects, and neither is refuted.
    let m = with(&[t()], &[no(&l("q"))]);
    assert_eq!(opinion(&m, &t()).judged(), State::Unknown);
    assert_eq!(suspects(&m), [vec![l("p"), t()]]);
    assert_eq!(verdict(&m), State::False);
}

// mitra: evidence for calculus::a_kept_promise_holds
#[test]
fn a_kept_promise_holds() {
    assert_eq!(at(&[yes(&l("q"))], &t()), State::True);
    assert_eq!(at(&[yes(&l("q"))], &l("p")), State::Unknown);
}

/// A piece can be after another piece. Once the outer one is true the
/// inner one is true, and it is applied in its turn: its state goes
/// on to what is after it.
// mitra: evidence for piece::a_piece_is_pre_subject_post
#[test]
fn a_piece_judged_true_is_applied() {
    let outer = piece("outer", &[], &[t()]);
    let m = with(std::slice::from_ref(&outer), &[yes(&outer), yes(&l("p"))]);
    let inner = opinion(&m, &t());
    assert_eq!(
        (inner.taken(), inner.judged()),
        (State::Unknown, State::True)
    );
    assert_eq!(opinion(&m, &l("q")).judged(), State::True);
    assert_eq!(verdict(&m), State::True);
}

/// What is after one piece is before the next: a chain of three.
#[test]
fn pieces_chain() {
    let (a, b, c, d) = (l("a"), l("b"), l("c"), l("d"));
    let first = piece("first", &[a.clone()], &[b.clone()]);
    let second = piece("second", &[b.clone()], &[c.clone()]);
    let third = piece("third", &[c.clone()], &[d.clone()]);
    let all = [first.clone(), second.clone(), third.clone()];
    let taken = [yes(&a), yes(&first), yes(&second), yes(&third)];
    let m = with(&all, &taken);
    assert_eq!(opinion(&m, &d).judged(), State::True);
    // And back: d false with the three pieces true refutes a.
    let back = with(&all, &[no(&d), yes(&first), yes(&second), yes(&third)]);
    assert_eq!(opinion(&back, &a).judged(), State::False);
}

// mitra: evidence for calculus::a_state_keeps_what_it_was_drawn_from
// mitra: evidence for calculus::a_conflict_names_what_to_give_up
#[test]
fn taken_against_itself_is_a_conflict() {
    let m = with(&[t()], &[yes(&t()), yes(&l("p")), no(&l("q"))]);
    assert_eq!(verdict(&m), State::Conflict);
    for x in [t(), l("p"), l("q")] {
        assert_eq!(opinion(&m, &x).judged(), State::Conflict);
        assert_eq!(opinion(&m, &x).confidence(), None);
    }
    let found = suspects(&m);
    assert_eq!(found.len(), 3);
    assert!(found.contains(&vec![l("q")]));
    // A piece nothing contests stays as it was.
    let aside = l("aside");
    let more = with(&[t()], &[yes(&t()), yes(&l("p")), no(&l("q")), yes(&aside)]);
    assert_eq!(opinion(&more, &aside).judged(), State::True);
}

/// A deduction gives exactly the least of what it was given from,
/// never more; two ways add up.
// mitra: evidence for calculus::sure_is_what_must_be_given_up
#[test]
fn confidence_is_the_least_to_give_up() {
    let (a, b) = (l("a"), l("b"));
    let both = piece("both", &[a.clone(), b.clone()], &[l("q")]);
    let mut m = with(
        std::slice::from_ref(&both),
        &[yes(&a), took(&b, true, 3.0, "tests")],
    );
    m.bias.push((both.clone(), 1.0));
    // q rests on a, taken once, and b, taken three times over.
    assert_eq!(sure(&m, &l("q")), sure(&m, &a));
    assert!(sure(&m, &l("q")) < sure(&m, &b));
    // A second way to q: both must be given up.
    let other = piece("other", &[l("c")], &[l("q")]);
    let mut two = with(
        &[both.clone(), other.clone()],
        &[yes(&a), yes(&b), yes(&l("c"))],
    );
    two.bias = vec![(both, 1.0), (other, 1.0)];
    assert!(sure(&two, &l("q")) > sure(&m, &l("q")));
    // One count at a bias of one half, a bias weighing 0.2.
    assert_eq!(shown(&opinion(&m, &a)), "true 0.92");
    assert_eq!(opinion(&m, &a).bias(), 0.5);
}

// mitra: evidence for model::a_taking_is_signed
// mitra: evidence for model::a_voice_weighs_as_it_is_believed
#[test]
fn a_voice_counts_once_and_as_far_as_it_is_believed() {
    let p = l("p");
    let once = with(&[], &[took(&p, true, 1.0, "me")]);
    let twice = with(&[], &[took(&p, true, 1.0, "me"), took(&p, true, 1.0, "me")]);
    let two = with(
        &[],
        &[took(&p, true, 1.0, "me"), took(&p, true, 1.0, "you")],
    );
    assert_eq!(sure(&twice, &p), sure(&once, &p));
    assert!(sure(&two, &p) > sure(&once, &p));
    let mut doubted = once.clone();
    doubted.voices.push((Voice::named("me"), 0.5));
    assert!(sure(&doubted, &p) < sure(&once, &p));
    // A voice believed at nothing takes nothing.
    doubted.voices[0].1 = 0.0;
    assert_eq!(opinion(&doubted, &p).judged(), State::Unknown);
    assert_eq!(opinion(&doubted, &p).taken(), State::True);
}

/// A piece with a bias of 1 is a deduction: it holds by what it is,
/// needs no taking, is not judged, and passes confidence on unchanged.
/// Under 1 it is an induction, and what it gives is less sure.
// mitra: evidence for model::the_step_is_read_from_the_bias
// mitra: evidence for calculus::deduction_keeps_confidence
// mitra: evidence for calculus::induction_lowers_confidence
#[test]
fn the_step_is_read_from_the_bias() {
    let mut sound = with(&[t()], &[yes(&l("p"))]);
    sound.bias.push((t(), 1.0));
    assert_eq!(step(&sound, &t()), Some(Inference::Deduction));
    assert_eq!(step(&sound, &l("p")), None);
    assert_eq!(opinion(&sound, &l("q")).judged(), State::True);
    assert_eq!(sure(&sound, &l("q")), sure(&sound, &l("p")));
    assert!(!judge(&sound).pieces.iter().any(|(x, _)| x.name == "t"));
    let mut back = with(&[t()], &[no(&l("q"))]);
    back.bias.push((t(), 1.0));
    assert_eq!(opinion(&back, &l("p")).judged(), State::False);
    // The same piece at the default bias, taken: an induction.
    let unsure = with(&[t()], &[yes(&t()), yes(&l("p"))]);
    assert_eq!(step(&unsure, &t()), Some(Inference::Induction));
    assert_eq!(opinion(&unsure, &l("q")).judged(), State::True);
    assert!(sure(&unsure, &l("q")) < sure(&unsure, &l("p")));
    // However firmly it is taken, what it gives weighs its bias.
    let firm = with(&[t()], &[took(&t(), true, 9.0, "me"), yes(&l("p"))]);
    assert_eq!(sure(&firm, &l("q")), sure(&unsure, &l("q")));
}

// mitra: evidence for calculus::the_verdict_is_one_state
#[test]
fn the_verdict_and_its_least_sure_piece() {
    let m = with(&[t()], &[yes(&t()), took(&l("p"), true, 3.0, "tests")]);
    assert_eq!(verdict(&m), State::True);
    let (least, n) = least_sure(&m).unwrap();
    // q comes through t, an induction weighing its bias of one half.
    assert_eq!(least.name, "q");
    assert!((n - 0.6 / 0.7).abs() < 1e-9);
    let open = with(&[t()], &[yes(&l("p"))]);
    assert_eq!((verdict(&open), least_sure(&open)), (State::Unknown, None));
    assert_eq!(verdict(&with(&[], &[])), State::Unknown);
}

/// A trial: the interval of an opinion. Unknown is all of 0 to 1; one
/// count for narrows it from below, one against from above; and the
/// confidence of a true piece is the point of the interval at its
/// bias.
// mitra: evidence for piece::an_interval_is_a_trial
#[test]
fn the_interval_of_an_opinion() {
    let p = l("p");
    let near = |x: (f64, f64), y: (f64, f64)| (x.0 - y.0).abs() < 1e-9 && (x.1 - y.1).abs() < 1e-9;
    assert_eq!(opinion(&with(&[], &[]), &p).interval(), (0.0, 1.0));
    let held = with(&[], &[yes(&p)]);
    assert!(near(opinion(&held, &p).interval(), (1.0 / 1.2, 1.0)));
    let (low, high) = opinion(&held, &p).interval();
    let at_bias = low + 0.5 * (high - low);
    assert!((sure(&held, &p).unwrap() - at_bias).abs() < 1e-9);
    let against = with(&[], &[no(&p)]);
    assert!(near(opinion(&against, &p).interval(), (0.0, 0.2 / 1.2)));
    // Taken both ways: a conflict, with no confidence and an interval.
    let both = with(&[], &[yes(&p), took(&p, false, 1.0, "other")]);
    assert_eq!(opinion(&both, &p).judged(), State::Conflict);
    assert_eq!(sure(&both, &p), None);
    assert!(near(opinion(&both, &p).interval(), (1.0 / 2.2, 1.2 / 2.2)));
}
