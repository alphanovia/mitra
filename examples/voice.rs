//! Voices (`docs/trials.md`): a voice has a reliability, the parent of
//! all it says. The core judges it (`model::judged`); this prints the
//! two trials, a cautious voice with an overconfident one, and two
//! agents on one source.
//!
//! `cargo run --release --example voice`

use mitra_core::model::*;

mod common;
use common::interval;

const W: f64 = 0.2;
/// What to expect of the completeness of a tie when nothing says.
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

/// Objects under gravity, each with its mass and its completeness, so
/// many measurements that pass and that fail, each believed at 0.9.
fn objects(list: &[(&str, usize, usize)]) -> Model {
    let mut m = Model::empty(S);
    m.base.push((p("gravity"), 0.5));
    for (name, pass, fail) in list {
        let (mass, done, speed) = (
            format!("mass {name}"),
            format!("complete {name}"),
            format!("same speed {name}"),
        );
        m.base.push((p(&mass), 0.5));
        m.base.push((p(&done), 0.5));
        let relation = p(&format!("all: {speed}"));
        m.related.push(Statement {
            piece: relation.clone(),
            content: Related {
                how: Relation::All,
                pieces: vec![p(&speed), p("gravity"), p(&mass), p(&done)],
            },
        });
        m.base.push((relation, 1.0));
        for j in 0..pass + fail {
            let drop = format!("drop {name} {j}");
            m.base.push((p(&drop), 0.9));
            m.evidence.push(Evidence {
                piece: p(&drop),
                about: p(&speed),
                value: if j < *pass { yes() } else { no() },
                by: Voice::named(&drop),
            });
        }
    }
    m
}

/// A voice reviews: so many yes about each piece. Each review is sure
/// of itself; what can be doubted is the voice.
fn reviews(m: &mut Model, voice: &str, said: &[(&str, f64)]) {
    for (about, count) in said {
        let piece = p(&format!("review by {voice}: {about}"));
        m.base.push((piece.clone(), 1.0));
        m.evidence.push(Evidence {
            piece,
            about: p(about),
            value: Opinion {
                r: *count,
                s: 0.0,
                a: 0.5,
            },
            by: Voice::named(voice),
        });
    }
}

fn cell(m: &Model, x: &str) -> String {
    let o = opinion(m, &p(x));
    let (from, to) = interval(o, W);
    format!("{:.2} [{from:.2}, {to:.2}]", projected(o, W))
}

const NAMES: [&str; 4] = ["stone", "ball", "leaf", "feather"];

fn cautious(unsure: f64) -> Vec<(String, f64)> {
    let mut said = vec![("gravity".to_string(), 1000.0)];
    said.extend(NAMES.map(|x| (format!("mass {x}"), 1.0)));
    said.push(("complete stone".to_string(), 20.0));
    said.push(("complete ball".to_string(), 5.0));
    if unsure > 0.0 {
        said.push(("complete leaf".to_string(), unsure));
        said.push(("complete feather".to_string(), unsure));
    }
    said
}
fn overconfident() -> Vec<(String, f64)> {
    let mut said = vec![("gravity".to_string(), 100.0)];
    said.extend(NAMES.map(|x| (format!("mass {x}"), 100.0)));
    said.extend(NAMES.map(|x| (format!("complete {x}"), 100.0)));
    said
}
fn borrowed(said: &[(String, f64)]) -> Vec<(&str, f64)> {
    said.iter().map(|(x, c)| (x.as_str(), *c)).collect()
}

fn two(list: &[(&str, usize, usize)], who: &str, unsure: f64) -> Model {
    let mut m = objects(list);
    if who != "overconfident" {
        reviews(&mut m, "cautious", &borrowed(&cautious(unsure)));
        doubt(&mut m, "cautious", 0.9);
    }
    if who != "cautious" {
        reviews(&mut m, "overconfident", &borrowed(&overconfident()));
        doubt(&mut m, "overconfident", 0.9);
    }
    m
}

/// A voice that can be doubted: its reliability has a base rate.
fn doubt(m: &mut Model, voice: &str, prior: f64) {
    m.base.push((reliable(&Voice::named(voice)).piece, prior));
}

/// How far each of these is judged to hold, then what the model reads
/// for each piece once the voices are judged.
fn row(m: &Model, names: &[&str], pieces: &[&str]) -> String {
    let voices = judged(m);
    let read = settled(m, &voices);
    let mut cells = Vec::new();
    for name in names {
        let found = voices.iter().find(|(piece, _)| &piece.name == name);
        cells.push(found.map_or(String::new(), |(_, at)| format!("{at:.2}")));
    }
    cells.extend(pieces.iter().map(|x| cell(&read, x)));
    cells.join(" | ")
}

fn main() {
    println!("### 1. A cautious voice and an overconfident one\n");
    println!("Both start at 0.9. The cautious one says 1000 yes on gravity, 1 on each mass, and on completeness 20 for the stone, 5 for the ball, half a yes for the leaf and the feather. The overconfident one says 100 yes on everything. The stone, the ball and the leaf pass every time, the feather never.\n");
    let voices = ["reliable: cautious", "reliable: overconfident"];
    let pieces = [
        "complete feather",
        "mass feather",
        "complete stone",
        "drop feather 0",
        "gravity",
    ];
    let head = "cautious voice | overconfident voice | `complete feather` | `mass feather` | `complete stone` | one of the feather's measurements | `gravity` |";
    println!("| measurements per object | {head}");
    println!("|---|---|---|---|---|---|---|---|");
    for k in [1, 3, 10, 30, 100] {
        let list = [
            ("stone", k, 0),
            ("ball", k, 0),
            ("leaf", k, 0),
            ("feather", 0, k),
        ];
        let m = two(&list, "both", 0.5);
        println!("| {k} | {} |", row(&m, &voices, &pieces));
    }

    println!("\n### 2. By who reviews, 10 measurements per object\n");
    println!("| who reviews | {head}");
    println!("|---|---|---|---|---|---|---|---|");
    let list = [
        ("stone", 10, 0),
        ("ball", 10, 0),
        ("leaf", 10, 0),
        ("feather", 0, 10),
    ];
    for (who, unsure, title) in [
        ("cautious", 0.5, "cautious"),
        (
            "cautious",
            0.0,
            "cautious, silent on the leaf's and the feather's completeness",
        ),
        ("overconfident", 0.5, "overconfident"),
        ("both", 0.5, "both"),
    ] {
        let m = two(&list, who, unsure);
        println!("| {title} | {} |", row(&m, &voices, &pieces));
    }

    println!("\n### 3. Noisy objects, both voices, 10 measurements each\n");
    let list = [
        ("stone", 9, 1),
        ("ball", 10, 0),
        ("leaf", 6, 4),
        ("feather", 0, 10),
    ];
    let m = two(&list, "both", 0.5);
    let at = judged(&m);
    let read = settled(&m, &at);
    println!("| object | passes | its mass | its completeness | one failing measurement |");
    println!("|---|---|---|---|---|");
    for (name, pass, fail) in list {
        let failing = match fail {
            0 => "none".to_string(),
            _ => cell(&read, &format!("drop {name} {}", pass + fail - 1)),
        };
        println!(
            "| {name} | {pass} of {} | {} | {} | {failing} |",
            pass + fail,
            cell(&read, &format!("mass {name}")),
            cell(&read, &format!("complete {name}"))
        );
    }
    println!(
        "\ncautious voice {:.2}, overconfident voice {:.2}, `gravity` {}",
        at[0].1,
        at[1].1,
        cell(&read, "gravity")
    );

    println!("\n### 4. Two agents, independent or sharing a source\n");
    println!("Agent A says 100 yes on gravity and on the mass and the completeness of the ball and of the feather. Agent B says 100 yes on the mass and the completeness of the stone, and of nothing else. The ball passes 10 times, the feather fails 10 times, the stone is never measured. Each agent starts at 0.9, and so does the source when they share one: a word then starts at 0.81.\n");
    println!("| declared | A's word holds at | B's word holds at | the source | `complete stone` | `complete feather` | one of the feather's measurements |");
    println!("|---|---|---|---|---|---|---|");
    for shared in [false, true] {
        let mut m = objects(&[("stone", 0, 0), ("ball", 10, 0), ("feather", 0, 10)]);
        let a = [
            ("gravity", 100.0),
            ("mass ball", 100.0),
            ("complete ball", 100.0),
            ("mass feather", 100.0),
            ("complete feather", 100.0),
        ];
        reviews(&mut m, "A", &a);
        reviews(
            &mut m,
            "B",
            &[("mass stone", 100.0), ("complete stone", 100.0)],
        );
        doubt(&mut m, "A", 0.9);
        doubt(&mut m, "B", 0.9);
        if shared {
            // The reliability of each is all of [the source].
            for agent in ["A", "B"] {
                let relation = p(&format!("{agent} rests on the source"));
                m.related.push(Statement {
                    piece: relation.clone(),
                    content: Related {
                        how: Relation::All,
                        pieces: vec![reliable(&Voice::named(agent)).piece, p("source")],
                    },
                });
                m.base.push((relation, 1.0));
            }
            m.base.push((p("source"), 0.9));
        }
        let cells = row(
            &m,
            &["reliable: A", "reliable: B", "source"],
            &["complete stone", "complete feather", "drop feather 0"],
        );
        println!(
            "| {} | {cells} |",
            if shared { "one source" } else { "independent" }
        );
    }
}
