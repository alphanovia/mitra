//! How the shape of the graph changes what is read (`docs/trials.md`).
//!
//! `cargo run --release --example structure`

use mitra_core::model::*;

const W: f64 = 0.2;
const C: f64 = 0.6;

fn p(name: &str) -> Piece {
    Piece::named(name)
}
fn at(m: &Model, x: &str) -> f64 {
    projected(opinion(m, &p(x)), W)
}
fn row(m: &Model, pieces: &[&str]) -> String {
    let cells: Vec<String> = pieces.iter().map(|x| format!("{:.2}", at(m, x))).collect();
    cells.join(" | ")
}
/// A tie from an object to its statements, the tie starting at `a`.
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
fn start(pieces: &[&str]) -> Model {
    let mut m = Model::empty(W, C);
    m.base = pieces.iter().map(|x| (p(x), 0.5)).collect();
    m
}
fn review(m: &mut Model, voice: &str, said: &[(String, f64)]) {
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
    m.base.push((reliable(&Voice::named(voice)).piece, 0.9));
}

/// Ten objects under gravity, each tie listing gravity and its mass;
/// the last `failing` never pass. 10 measurements each.
fn ten(failing: usize) -> Model {
    let mut m = start(&["gravity"]);
    for k in 0..10 {
        let (name, mass) = (format!("o{k}"), format!("mass o{k}"));
        m.base.push((p(&mass), 0.5));
        tie(&mut m, &name, &["gravity", &mass], 0.5);
        let fails = k >= 10 - failing;
        let (pass, fail) = if fails { (0, 10) } else { (10, 0) };
        seen(
            &mut m,
            &format!("drop {name}"),
            &format!("whole: {name}"),
            pass,
            fail,
        );
    }
    m
}

fn main() {
    println!("### 1. A statement shared by many ties, against a private one\n");
    println!("Ten objects, each with a tie that lists `gravity` and its own mass. 10 measurements each. So many objects never pass.\n");
    println!("| objects that fail | `gravity` | mass of one that passes | mass of one that fails | its tie | its completeness | one of its measurements |");
    println!("|---|---|---|---|---|---|---|");
    for failing in [1, 5, 9, 10] {
        let m = ten(failing);
        let passing = if failing < 10 {
            format!("{:.2}", at(&m, "mass o0"))
        } else {
            String::new()
        };
        println!(
            "| {failing} | {:.2} | {passing} | {} |",
            at(&m, "gravity"),
            row(&m, &["mass o9", "o9", "complete: o9", "drop o9 0"])
        );
    }

    println!("\n### 2. Two statements on one tie\n");
    println!("A ball: it falls, and it makes a noise. Thrown, it falls; listened to, it is silent. With one tie that lists both, an observation is of both at once, so the silence is a failure of the whole. With one tie per statement, each is observed apart. 10 observations each.\n");
    println!("| written | `falls` | `noise` | tie of `falls` | tie of `noise` | completeness of the tie of `noise` |");
    println!("|---|---|---|---|---|---|");
    let mut one = start(&["falls", "noise"]);
    tie(&mut one, "ball", &["falls", "noise"], 0.5);
    seen(&mut one, "silence", "whole: ball", 0, 10);
    println!(
        "| one tie, both observed together and failing | {} |",
        row(&one, &["falls", "noise", "ball", "ball", "complete: ball"])
    );
    let mut two = start(&["falls", "noise"]);
    tie(&mut two, "ball falls", &["falls"], 0.5);
    tie(&mut two, "ball bangs", &["noise"], 0.5);
    seen(&mut two, "throw", "whole: ball falls", 10, 0);
    seen(&mut two, "silence", "whole: ball bangs", 0, 10);
    println!(
        "| one tie per statement | {} |",
        row(
            &two,
            &[
                "falls",
                "noise",
                "ball falls",
                "ball bangs",
                "complete: ball bangs"
            ]
        )
    );
    // A second object that bangs: the statement is believed elsewhere.
    tie(&mut two, "drum bangs", &["noise"], 0.5);
    seen(&mut two, "beat", "whole: drum bangs", 10, 0);
    println!(
        "| one tie per statement, and a drum that bangs | {} |",
        row(
            &two,
            &[
                "falls",
                "noise",
                "ball falls",
                "ball bangs",
                "complete: ball bangs"
            ]
        )
    );

    println!("\n### 3. A diamond: two paths to one statement\n");
    println!("A tie lists `orbit` and `tide`. `orbit` is all of [gravity, sun]; `tide` is all of [gravity, moon]. Gravity is reached by two paths. 10 observations of the whole.\n");
    println!("| observed | the whole | `gravity` | `sun` | `moon` | `orbit` | `tide` |");
    println!("|---|---|---|---|---|---|---|");
    for (title, pass, fail) in [
        ("nothing", 0, 0),
        ("10 passes", 10, 0),
        ("10 failures", 0, 10),
    ] {
        let mut m = start(&["gravity", "sun", "moon"]);
        tie(&mut m, "sky", &["orbit", "tide"], 1.0);
        m.base.push((p("complete: sky"), 1.0));
        all(&mut m, &["orbit", "gravity", "sun"]);
        all(&mut m, &["tide", "gravity", "moon"]);
        seen(&mut m, "look", "whole: sky", pass, fail);
        println!(
            "| {title} | {} |",
            row(
                &m,
                &["whole: sky", "gravity", "sun", "moon", "orbit", "tide"]
            )
        );
    }
    println!("\nWith nothing observed the whole reads 0.125: one half three times, gravity once. Counted along both paths it would read one half four times, 0.0625.");

    println!("\n### 4. Evidence on an inner node, against only the whole\n");
    println!("Three objects pass, the feather fails, 10 measurements each. Then a scale speaks of the feather's mass alone, so many readings, each believed at 0.9.\n");
    println!("| scale readings | `gravity` | the feather's mass | its tie | its completeness | one of its measurements |");
    println!("|---|---|---|---|---|---|");
    for readings in [0, 1, 3, 10] {
        let mut m = start(&["gravity"]);
        for (name, pass, fail) in [
            ("stone", 10, 0),
            ("ball", 10, 0),
            ("leaf", 10, 0),
            ("feather", 0, 10),
        ] {
            m.base.push((p(&format!("mass {name}")), 0.5));
            tie(&mut m, name, &["gravity", &format!("mass {name}")], 0.5);
            seen(
                &mut m,
                &format!("drop {name}"),
                &format!("whole: {name}"),
                pass,
                fail,
            );
        }
        seen(&mut m, "scale", "mass feather", readings, 0);
        println!(
            "| {readings} | {} |",
            row(
                &m,
                &[
                    "gravity",
                    "mass feather",
                    "feather",
                    "complete: feather",
                    "drop feather 0"
                ]
            )
        );
    }

    println!("\n### 5. The reviewers again, with ties\n");
    println!("A stone, a ball and a leaf pass 10 times, the feather fails 10 times. The cautious voice says 1000 yes on gravity, 1 on each mass, and on completeness 20, 5, a half and a half. The overconfident voice says 100 yes on everything, ties too. Both start at 0.9.\n");
    println!("| variant | cautious voice | overconfident voice | the feather's mass | its tie | its completeness | one of its measurements |");
    println!("|---|---|---|---|---|---|---|");
    let names = ["stone", "ball", "leaf", "feather"];
    for (title, tie_at, scale, cautious_on_ties) in [
        ("ties start at one half", 0.5, 0, false),
        ("and a scale weighs the feather, 3 readings", 0.5, 3, false),
        (
            "and the cautious voice also says 20 yes on each tie",
            0.5,
            0,
            true,
        ),
        ("ties start at 0.95: this is the feather", 0.95, 0, false),
        ("ties at 0.95 and the scale", 0.95, 3, false),
    ] {
        let mut m = start(&["gravity"]);
        for (name, pass, fail) in [
            ("stone", 10, 0),
            ("ball", 10, 0),
            ("leaf", 10, 0),
            ("feather", 0, 10),
        ] {
            m.base.push((p(&format!("mass {name}")), 0.5));
            tie(&mut m, name, &["gravity", &format!("mass {name}")], tie_at);
            seen(
                &mut m,
                &format!("drop {name}"),
                &format!("whole: {name}"),
                pass,
                fail,
            );
        }
        seen(&mut m, "scale", "mass feather", scale, 0);
        let done = |x: &str| format!("complete: {x}");
        let mut cautious = vec![("gravity".to_string(), 1000.0)];
        cautious.extend(names.map(|x| (format!("mass {x}"), 1.0)));
        cautious.extend([(done("stone"), 20.0), (done("ball"), 5.0)]);
        cautious.extend([(done("leaf"), 0.5), (done("feather"), 0.5)]);
        if cautious_on_ties {
            cautious.extend(names.map(|x| (x.to_string(), 20.0)));
        }
        let mut loud = vec![("gravity".to_string(), 100.0)];
        loud.extend(names.map(|x| (format!("mass {x}"), 100.0)));
        loud.extend(names.map(|x| (done(x), 100.0)));
        loud.extend(names.map(|x| (x.to_string(), 100.0)));
        review(&mut m, "cautious", &cautious);
        review(&mut m, "overconfident", &loud);
        println!(
            "| {title} | {} |",
            row(
                &m,
                &[
                    "reliable: cautious",
                    "reliable: overconfident",
                    "mass feather",
                    "feather",
                    "complete: feather",
                    "drop feather 0"
                ]
            )
        );
    }
}
