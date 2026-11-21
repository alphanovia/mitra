//! The claims of `model/model.mitra`, subject by subject, and the walk
//! over a model.

use mitra_core::model::*;

/// The weight of a base rate that these are recorded at.
const W: f64 = 0.2;
/// What to expect of the completeness of a tie when nothing says.
const C: f64 = 0.6;
/// The settings these are recorded at: a piece nobody gave a base rate
/// is expected wrong, and a voice nobody gave one is not judged.
const S: Settings = Settings {
    weight: W,
    statement: 0.0,
    evidence: 0.0,
    tie: 0.0,
    completeness: C,
    reliability: 1.0,
};

fn op(r: f64, s: f64, a: f64) -> Opinion {
    Opinion { r, s, a }
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

// mitra: evidence for tests::no_evidence_is_the_base_rate
#[test]
fn no_evidence_is_the_base_rate() {
    assert_eq!(projected(nothing(0.5), W), 0.5);
}

// mitra: evidence for piece::sure_is_sure
#[test]
fn sure_is_sure() {
    assert_eq!((projected(yes(), W), projected(no(), W)), (1.0, 0.0));
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

// The walk: what the promises say of `opinion`, on small worlds.

// What was tried and kept (`docs/trials.md`).

fn doubt(mut model: Model, voice: &str, prior: f64) -> Model {
    model
        .base
        .push((reliable(&Voice::named(voice)).piece, prior));
    model
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
        settings: S,
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
        ..Model::empty(S)
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

/// A tie nobody gave a base rate has the setting; what is written wins.
// mitra: evidence for tests::an_unwritten_tie_has_the_setting
#[test]
fn an_unwritten_tie_has_the_setting() {
    let tie = ties("applies", &["falls"], "ball");
    let model = Model {
        ties: vec![tie],
        settings: Settings { tie: 0.5, ..S },
        ..Model::empty(S)
    };
    assert_eq!(base(&model, &Piece::named("applies")), 0.5);
    assert_eq!(base(&model, &Piece::named("falls")), 0.0);
    let written = Model {
        base: base_of(&[("applies", 0.9)]),
        ..model
    };
    assert_eq!(base(&written, &Piece::named("applies")), 0.9);
}

/// Each kind of piece nobody gave a base rate has the setting for its
/// kind; what is written wins.
// mitra: evidence for tests::every_kind_has_its_setting
#[test]
fn every_kind_has_its_setting() {
    let model = Model {
        evidence: vec![says("throw", "whole: applies", yes(), "me")],
        ties: vec![ties("applies", &["falls"], "ball")],
        base: base_of(&[("written", 0.25)]),
        settings: Settings {
            weight: W,
            statement: 0.5,
            evidence: 0.9,
            tie: 0.7,
            completeness: 0.6,
            reliability: 0.8,
        },
        ..Model::empty(S)
    };
    let of = |x: &str| base(&model, &Piece::named(x));
    assert_eq!(of("falls"), 0.5);
    assert_eq!(of("throw"), 0.9);
    assert_eq!(of("applies"), 0.7);
    assert_eq!(of("complete: applies"), 0.6);
    assert_eq!(of("reliable: me"), 0.8);
    assert_eq!(of("written"), 0.25);
    // The relation a tie implies is structure.
    assert_eq!(of("all: whole: applies"), 1.0);
}
