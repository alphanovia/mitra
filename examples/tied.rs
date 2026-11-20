//! The toys written with ties (`docs/trials.md`): the author writes a
//! tie from an object to its statements, and the whole and the
//! completeness are implied. Each toy is read both ways, by hand as
//! before and with ties, to see what changes.
//!
//! `cargo run --release --example tied`

use mitra_core::model::*;

const W: f64 = 0.2;
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

fn p(name: &str) -> Piece {
    Piece::named(name)
}
fn read(m: &Model, x: &str) -> String {
    match m.base.iter().any(|(piece, _)| piece == &p(x)) || base(m, &p(x)) > 0.0 {
        true => format!("{:.2}", projected(opinion(m, &p(x)), W)),
        false => String::new(),
    }
}
fn says(m: &mut Model, name: &str, about: &str, value: Opinion, trust: f64, by: &str) {
    m.base.push((p(name), trust));
    m.evidence.push(Evidence {
        piece: p(name),
        about: p(about),
        value,
        by: Voice::named(by),
    });
}

/// How a toy is written: every relation by hand, with a completeness
/// piece or not, or with ties.
#[derive(Clone, Copy, PartialEq)]
enum Shape {
    ByHand(Option<f64>),
    Tied,
}

/// Objects under gravity. By hand: "same speed" is all of [gravity,
/// its mass] and its completeness if there is one. Tied: the tie of
/// the object lists gravity and its mass. So many measurements pass
/// and fail, each believed at 0.9.
fn objects(list: &[(&str, usize, usize)], shape: Shape) -> Model {
    let mut m = Model::empty(S);
    m.base.push((p("gravity"), 0.5));
    for (name, pass, fail) in list {
        let mass = format!("mass {name}");
        m.base.push((p(&mass), 0.5));
        let about = match shape {
            Shape::Tied => {
                let tie = Tie {
                    piece: p(name),
                    statements: vec![p("gravity"), p(&mass)],
                    place: Place {
                        path: vec![name.to_string()],
                    },
                };
                m.base.push((p(name), 0.5));
                let about = whole(&tie).name;
                m.ties.push(tie);
                about
            }
            Shape::ByHand(complete) => {
                let about = format!("same speed {name}");
                let mut pieces = vec![p(&about), p("gravity"), p(&mass)];
                if let Some(a) = complete {
                    pieces.push(p(&format!("complete: {name}")));
                    m.base.push((p(&format!("complete: {name}")), a));
                }
                let relation = p(&format!("all: {about}"));
                m.base.push((relation.clone(), 1.0));
                m.related.push(Statement {
                    piece: relation,
                    content: Related {
                        how: Relation::All,
                        pieces,
                    },
                });
                about
            }
        };
        for j in 0..pass + fail {
            let drop = format!("drop {name} {j}");
            let value = if j < *pass { yes() } else { no() };
            says(&mut m, &drop, &about, value, 0.9, &drop);
        }
    }
    m
}

fn reviews(m: &mut Model, voice: &str, said: &[(String, f64)]) {
    for (about, count) in said {
        let value = Opinion {
            r: *count,
            s: 0.0,
            a: 0.5,
        };
        says(
            m,
            &format!("review by {voice}: {about}"),
            about,
            value,
            1.0,
            voice,
        );
    }
    m.base.push((reliable(&Voice::named(voice)).piece, 0.9));
}

fn main() {
    println!("### 1. One throw\n");
    println!("The throw is believed at 0.9; `falls` and `applies` start at one half. By hand, `lands` is all of [applies, falls]. Tied, the tie `applies` lists `falls`, and its completeness starts at 0.6.\n");
    println!("| written | I saw | `falls` | `applies` | completeness | `throw` |");
    println!("|---|---|---|---|---|---|");
    for (title, tied) in [("by hand", false), ("with a tie", true)] {
        for (saw, value) in [("fall", yes()), ("no fall", no())] {
            let mut m = Model::empty(S);
            m.base = vec![(p("applies"), 0.5), (p("falls"), 0.5)];
            let about = match tied {
                true => {
                    m.ties.push(Tie {
                        piece: p("applies"),
                        statements: vec![p("falls")],
                        place: Place {
                            path: vec!["ball".to_string()],
                        },
                    });
                    "whole: applies"
                }
                false => {
                    m.base.push((p("all: lands"), 1.0));
                    m.related.push(Statement {
                        piece: p("all: lands"),
                        content: Related {
                            how: Relation::All,
                            pieces: vec![p("lands"), p("applies"), p("falls")],
                        },
                    });
                    "lands"
                }
            };
            says(&mut m, "throw", about, value, 0.9, "me");
            let cells = ["falls", "applies", "complete: applies", "throw"].map(|x| read(&m, x));
            println!("| {title} | {saw} | {} |", cells.join(" | "));
        }
    }

    println!("\n### 2. Gravity, objects that pass and a feather that never does\n");
    println!("10 measurements per object. By hand, \"same speed\" is all of [gravity, its mass]. Tied, each object has a tie that lists gravity and its mass; the tie starts at one half and its completeness at 0.6.\n");
    println!("| written | objects that pass | `gravity` | mass of one that passes | the feather's mass | the feather's tie | the feather's completeness | one of the feather's measurements |");
    println!("|---|---|---|---|---|---|---|---|");
    for (title, shape) in [("by hand", Shape::ByHand(None)), ("with ties", Shape::Tied)] {
        for n in [1usize, 3, 9, 30] {
            let names: Vec<String> = (0..n).map(|k| format!("o{k}")).collect();
            let mut list: Vec<(&str, usize, usize)> =
                names.iter().map(|x| (x.as_str(), 10, 0)).collect();
            list.push(("feather", 0, 10));
            let m = objects(&list, shape);
            let cells = [
                "gravity",
                "mass o0",
                "mass feather",
                "feather",
                "complete: feather",
                "drop feather 0",
            ]
            .map(|x| read(&m, x));
            println!("| {title} | {n} | {} |", cells.join(" | "));
        }
    }

    println!("\n### 3. The cautious and the overconfident reviewer\n");
    println!("A stone, a ball and a leaf pass 10 times, the feather fails 10 times. Both voices start at 0.9. By hand, each object has a completeness piece at one half. Tied, the completeness is implied at 0.6 and the tie starts at one half. The overconfident voice says 100 yes on everything, ties too.\n");
    println!("| written | cautious voice | overconfident voice | `gravity` | the feather's mass | the feather's tie | the feather's completeness | the stone's completeness | one of the feather's measurements |");
    println!("|---|---|---|---|---|---|---|---|---|");
    let names = ["stone", "ball", "leaf", "feather"];
    for (title, shape) in [
        ("by hand", Shape::ByHand(Some(0.5))),
        ("with ties", Shape::Tied),
    ] {
        let list = [
            ("stone", 10, 0),
            ("ball", 10, 0),
            ("leaf", 10, 0),
            ("feather", 0, 10),
        ];
        let mut m = objects(&list, shape);
        let done = |x: &str| format!("complete: {x}");
        let mut cautious = vec![("gravity".to_string(), 1000.0)];
        cautious.extend(names.map(|x| (format!("mass {x}"), 1.0)));
        cautious.extend([(done("stone"), 20.0), (done("ball"), 5.0)]);
        cautious.extend([(done("leaf"), 0.5), (done("feather"), 0.5)]);
        let mut loud = vec![("gravity".to_string(), 100.0)];
        loud.extend(names.map(|x| (format!("mass {x}"), 100.0)));
        loud.extend(names.map(|x| (done(x), 100.0)));
        if shape == Shape::Tied {
            loud.extend(names.map(|x| (x.to_string(), 100.0)));
        }
        reviews(&mut m, "cautious", &cautious);
        reviews(&mut m, "overconfident", &loud);
        let cells = [
            "reliable: cautious",
            "reliable: overconfident",
            "gravity",
            "mass feather",
            "feather",
            "complete: feather",
            "complete: stone",
            "drop feather 0",
        ]
        .map(|x| read(&m, x));
        println!("| {title} | {} |", cells.join(" | "));
    }
}
