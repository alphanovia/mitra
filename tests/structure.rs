//! How the shape of the graph changes what is read: the foundation
//! scenarios of `examples/structure.rs`, pinned.

use mitra_core::model::*;

const W: f64 = 0.2;
const C: f64 = 0.6;
/// The settings these are recorded at: a tie nobody gave a base rate
/// is expected wrong, and a voice nobody gave one is not judged.
const S: Settings = Settings {
    weight: W,
    completeness: C,
    tie: 0.0,
    reliability: None,
};

fn p(name: &str) -> Piece {
    Piece::named(name)
}
fn at(m: &Model, x: &str) -> f64 {
    projected(opinion(m, &p(x)), W)
}
fn close(x: f64, y: f64) -> bool {
    (x - y).abs() < 0.006
}
fn start(pieces: &[&str]) -> Model {
    let mut m = Model::empty(S);
    m.base = pieces.iter().map(|x| (p(x), 0.5)).collect();
    m
}
fn tie(m: &mut Model, name: &str, statements: &[&str], a: f64) {
    m.base.push((p(name), a));
    m.ties.push(Tie {
        piece: p(name),
        statements: statements.iter().map(|x| p(x)).collect(),
        place: Place {
            path: vec![name.to_string()],
        },
    });
}
/// So many observations of a piece, each its own voice, believed at 0.9.
fn seen(m: &mut Model, name: &str, about: &str, pass: usize, fail: usize) {
    for j in 0..pass + fail {
        let piece = p(&format!("{name} {j}"));
        m.base.push((piece.clone(), 0.9));
        m.evidence.push(Evidence {
            piece: piece.clone(),
            about: p(about),
            value: if j < pass { yes() } else { no() },
            by: Voice { name: piece.name },
        });
    }
}
fn all(m: &mut Model, pieces: &[&str]) {
    let relation = p(&format!("all: {}", pieces[0]));
    m.base.push((relation.clone(), 1.0));
    m.related.push(Statement {
        piece: relation,
        content: Related {
            how: Relation::All,
            pieces: pieces.iter().map(|x| p(x)).collect(),
        },
    });
}
/// Objects under gravity, each tie listing gravity and its mass, with
/// so many measurements that pass and that fail.
fn under_gravity(objects: &[(&str, usize, usize)]) -> Model {
    let mut m = start(&["gravity"]);
    for (name, pass, fail) in objects {
        let mass = format!("mass {name}");
        m.base.push((p(&mass), 0.5));
        tie(&mut m, name, &["gravity", &mass], 0.5);
        seen(
            &mut m,
            &format!("drop {name}"),
            &format!("whole: {name}"),
            *pass,
            *fail,
        );
    }
    m
}

#[test]
fn a_shared_statement_stands_while_one_object_passes() {
    let names: Vec<String> = (0..10).map(|k| format!("o{k}")).collect();
    let with = |failing: usize| {
        let list: Vec<(&str, usize, usize)> = names
            .iter()
            .enumerate()
            .map(|(k, x)| match k >= 10 - failing {
                true => (x.as_str(), 0, 10),
                false => (x.as_str(), 10, 0),
            })
            .collect();
        under_gravity(&list)
    };
    // The blame stays in the tie that fails: its mass, its tie, its completeness.
    let one = with(1);
    assert!(at(&one, "gravity") > 0.99 && at(&one, "mass o0") > 0.98);
    assert!(close(at(&one, "mass o9"), 0.20) && close(at(&one, "o9"), 0.20));
    assert!(close(at(&one, "complete: o9"), 0.31) && at(&one, "drop o9 0") > 0.85);
    // Nine of ten failing still leave it standing; all ten bring it down.
    assert!(at(&with(9), "gravity") > 0.95 && at(&with(10), "gravity") < 0.1);
}

#[test]
fn one_tie_cannot_tell_its_statements_apart_and_two_can() {
    let mut one = start(&["falls", "noise"]);
    tie(&mut one, "ball", &["falls", "noise"], 0.5);
    seen(&mut one, "silence", "whole: ball", 0, 10);
    assert!(close(at(&one, "falls"), at(&one, "noise")));

    let mut two = start(&["falls", "noise"]);
    tie(&mut two, "ball falls", &["falls"], 0.5);
    tie(&mut two, "ball bangs", &["noise"], 0.5);
    seen(&mut two, "throw", "whole: ball falls", 10, 0);
    seen(&mut two, "silence", "whole: ball bangs", 0, 10);
    assert!(at(&two, "falls") > 0.98 && close(at(&two, "noise"), 0.20));
    // The statement believed elsewhere, the ball's own tie takes the no.
    tie(&mut two, "drum bangs", &["noise"], 0.5);
    seen(&mut two, "beat", "whole: drum bangs", 10, 0);
    assert!(at(&two, "noise") > 0.98 && at(&two, "ball bangs") < 0.1);
}

/// A tie lists `orbit` and `tide`; each is all of gravity and one more.
fn diamond(pass: usize, fail: usize) -> Model {
    let mut m = start(&["gravity", "sun", "moon"]);
    tie(&mut m, "sky", &["orbit", "tide"], 1.0);
    m.base.push((p("complete: sky"), 1.0));
    all(&mut m, &["orbit", "gravity", "sun"]);
    all(&mut m, &["tide", "gravity", "moon"]);
    seen(&mut m, "look", "whole: sky", pass, fail);
    m
}

#[test]
fn a_statement_reached_by_two_paths_counts_once() {
    // One half three times, not four.
    assert!(close(at(&diamond(0, 0), "whole: sky"), 0.125));
    let passed = diamond(10, 0);
    assert_eq!(
        opinion(&passed, &p("gravity")).r,
        opinion(&passed, &p("sun")).r
    );
}

#[test]
fn a_no_travels_down_to_the_parts_of_a_part() {
    let failed = diamond(0, 10);
    for x in ["gravity", "sun", "moon"] {
        assert!(close(at(&failed, x), 0.23), "{x}");
    }
    // One suspect, however many paths lead to it.
    let s = |x: &str| opinion(&failed, &p(x)).s;
    assert!((s("gravity") - s("sun")).abs() < 1e-9);
    // Gravity, sun, moon, the tie, its completeness, and what was seen.
    assert_eq!(conflicts(&failed)[0].len(), 6);
}

#[test]
fn evidence_on_one_part_moves_the_blame_off_it() {
    let list = [("stone", 10, 0), ("feather", 0, 10)];
    let bare = under_gravity(&list);
    assert!(close(at(&bare, "mass feather"), at(&bare, "feather")));
    let mut weighed = under_gravity(&list);
    seen(&mut weighed, "scale", "mass feather", 1, 0);
    assert!(at(&weighed, "mass feather") > 0.75);
    assert!(at(&weighed, "feather") < at(&bare, "feather"));
    assert!(at(&weighed, "complete: feather") < at(&bare, "complete: feather"));
}

#[test]
fn a_tie_at_one_half_takes_the_blame_before_its_completeness() {
    let read = |tie_at: f64| {
        let mut m = start(&["gravity", "mass feather"]);
        tie(&mut m, "feather", &["gravity", "mass feather"], tie_at);
        seen(&mut m, "drop", "whole: feather", 0, 10);
        seen(&mut m, "scale", "mass feather", 3, 0);
        seen(&mut m, "elsewhere", "gravity", 10, 0);
        (at(&m, "feather"), at(&m, "complete: feather"))
    };
    let (tie_, done) = read(0.5);
    assert!(tie_ < done);
    // Sure that this is the feather, the completeness takes it.
    let (tie_, done) = read(0.99);
    assert!(done < tie_ && done < 0.2);
}
