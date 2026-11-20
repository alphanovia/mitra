//! Review evidence in the mix. A review is evidence about a statement
//! or a tie, signed by a voice, that carries a count: how much it
//! weighs. "Centuries of knowledge about gravity" is a review with a
//! large count; "I checked the environment" is a review of the tie.
//! It counts as far as its reviewer is believed. The weight stays 0.2.
//!
//! `cargo run --release --example review`

use mitra_core::model::*;

mod common;
use common::interval;

/// The weight of a base rate that these are recorded at.
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
fn whole(m: &mut Model, pieces: &[&str]) {
    let relation = p(&format!("all: {}", pieces[0]));
    m.related.push(Statement {
        piece: relation.clone(),
        content: Related {
            how: Relation::All,
            pieces: pieces.iter().map(|x| p(x)).collect(),
        },
    });
    m.base.push((relation, 1.0));
}
/// Evidence `name` about `about`: so many yes or no, believed at `trust`.
fn says(m: &mut Model, name: &str, about: &str, yes_: f64, no_: f64, trust: f64) {
    m.base.push((p(name), trust));
    m.evidence.push(Evidence {
        piece: p(name),
        about: p(about),
        value: Opinion {
            r: yes_,
            s: no_,
            a: 0.5,
        },
        by: Voice::named(name),
    });
}
/// Counts, reading, and where it is right nine times out of ten.
fn cell(m: &Model, x: &str) -> String {
    let o = opinion(m, &p(x));
    let (from, to) = interval(o, W);
    format!(
        "({:.1}, {:.2}) {:.2} [{from:.2}, {to:.2}]",
        o.r,
        o.s,
        projected(o, W)
    )
}

/// One throw, with reviews of so many counts on the statement and the tie.
fn one_throw(statement: f64, tie: f64, fell: bool) -> Model {
    let mut m = Model {
        base: vec![(p("applies"), 0.5), (p("falls"), 0.5)],
        ..Model::empty(S)
    };
    whole(&mut m, &["lands", "applies", "falls"]);
    if statement > 0.0 {
        says(&mut m, "review of falls", "falls", statement, 0.0, 0.9);
    }
    if tie > 0.0 {
        says(&mut m, "review of applies", "applies", tie, 0.0, 0.9);
    }
    let (yes_, no_) = if fell { (1.0, 0.0) } else { (0.0, 1.0) };
    says(&mut m, "throw", "lands", yes_, no_, 0.9);
    m
}

/// `pass` objects fall at the same speed, `fail` never do, `k`
/// measurements each. `gravity` and the mass of the first failing
/// object may be reviewed.
fn sweep(pass: usize, fail: usize, k: usize, gravity: f64, mass: f64) -> Model {
    let mut m = Model::empty(S);
    m.base.push((p("gravity"), 0.5));
    if gravity > 0.0 {
        says(&mut m, "centuries", "gravity", gravity, 0.0, 0.99);
    }
    for i in 0..pass + fail {
        let name = if i < pass {
            format!("o{i}")
        } else {
            format!("f{}", i - pass)
        };
        let (mass_, speed) = (format!("mass {name}"), format!("same speed {name}"));
        m.base.push((p(&mass_), 0.5));
        whole(&mut m, &[&speed, "gravity", &mass_]);
        for j in 0..k {
            let (yes_, no_) = if i < pass { (1.0, 0.0) } else { (0.0, 1.0) };
            says(&mut m, &format!("drop {name} {j}"), &speed, yes_, no_, 0.9);
        }
    }
    if mass > 0.0 {
        says(&mut m, "I weighed it", "mass f0", mass, 0.0, 0.9);
    }
    m
}

/// A stone, a ball, a leaf and a feather, with a completeness piece
/// per tie: "same speed" is all of [gravity, mass, complete]. Each has
/// so many measurements that pass and that fail. `reviews` are
/// (piece, yes) by one reviewer believed at `trust`; each review is a
/// piece of its own.
fn reviewed(objects: &[(&str, usize, usize)], reviews: &[(&str, f64)], trust: f64) -> Model {
    let mut m = Model::empty(S);
    m.base.push((p("gravity"), 0.5));
    for (name, pass, fail) in objects {
        let (mass, done, speed) = (
            format!("mass {name}"),
            format!("complete {name}"),
            format!("same speed {name}"),
        );
        m.base.push((p(&mass), 0.5));
        m.base.push((p(&done), 0.5));
        whole(&mut m, &[&speed, "gravity", &mass, &done]);
        for j in 0..pass + fail {
            let (yes_, no_) = if j < *pass { (1.0, 0.0) } else { (0.0, 1.0) };
            says(&mut m, &format!("drop {name} {j}"), &speed, yes_, no_, 0.9);
        }
    }
    for (about, count) in reviews {
        says(
            &mut m,
            &format!("review: {about}"),
            about,
            *count,
            0.0,
            trust,
        );
    }
    m
}

const MASSES: [&str; 4] = ["mass stone", "mass ball", "mass leaf", "mass feather"];
const DONE: [&str; 4] = [
    "complete stone",
    "complete ball",
    "complete leaf",
    "complete feather",
];

fn reviewers() {
    println!("\n### 4. A good reviewer\n");
    println!("A stone, a ball, a leaf and a feather, 10 measurements each. Each tie has its completeness: \"same speed\" is all of [gravity, mass, complete].\n");
    println!("The reviewer is believed at 0.99. 1000 yes on gravity, 1 yes on each mass. On completeness, by object, not having looked at the air: 20 yes for the stone, 5 for the ball, half a yes for the leaf and for the feather.\n");
    let mut good = vec![("gravity", 1000.0)];
    good.extend(MASSES.map(|x| (x, 1.0)));
    good.extend([
        ("complete stone", 20.0),
        ("complete ball", 5.0),
        ("complete leaf", 0.5),
        ("complete feather", 0.5),
    ]);
    // The stone fails once in ten, the leaf four times, the feather always.
    let objects = [
        ("stone", 9, 1),
        ("ball", 10, 0),
        ("leaf", 6, 4),
        ("feather", 0, 10),
    ];
    let m = reviewed(&objects, &good, 0.99);
    println!("| object | passes | its mass | its completeness | one failing measurement |");
    println!("|---|---|---|---|---|");
    for (name, pass, fail) in objects {
        let failing = match fail {
            0 => "none".to_string(),
            _ => cell(&m, &format!("drop {name} {}", pass + fail - 1)),
        };
        println!(
            "| {name} | {pass} of {} | {} | {} | {failing} |",
            pass + fail,
            cell(&m, &format!("mass {name}")),
            cell(&m, &format!("complete {name}"))
        );
    }
    println!("\n`gravity`: {}\n", cell(&m, "gravity"));

    println!("### 5. A bad reviewer, and a no that travels back\n");
    println!("The bad reviewer is believed at 0.9 and says 100 yes on gravity, on every mass and on every completeness, the feather's too. The stone, the ball and the leaf pass every time, the feather never.\n");
    println!("The trial rule: evidence that said yes about a part of what failed is judged by the failure. Each cell is without the rule, then with it.\n");
    let mut bad = vec![("gravity", 100.0)];
    bad.extend(MASSES.map(|x| (x, 100.0)));
    bad.extend(DONE.map(|x| (x, 100.0)));
    println!("| measurements per object | review of the feather's completeness | review of the stone's completeness | review of gravity | `complete feather` | `mass feather` | one of the feather's measurements | `gravity` |");
    println!("|---|---|---|---|---|---|---|---|");
    for k in [1, 3, 10, 30, 100] {
        let objects = [
            ("stone", k, 0),
            ("ball", k, 0),
            ("leaf", k, 0),
            ("feather", 0, k),
        ];
        let m = reviewed(&objects, &bad, 0.9);
        let both = |x: &str| {
            let at = |o: Opinion| projected(o, W);
            let (off, on) = (opinion(&m, &p(x)), opinion(&m, &p(x)));
            let (from, to) = interval(on, W);
            format!("{:.2} / {:.2} [{from:.2}, {to:.2}]", at(off), at(on))
        };
        let cells = [
            "review: complete feather",
            "review: complete stone",
            "review: gravity",
            "complete feather",
            "mass feather",
            "drop feather 0",
            "gravity",
        ]
        .map(both);
        println!("| {k} | {} |", cells.join(" | "));
    }
}

/// All of one voice's reviews stand on one piece, its reliability,
/// believed at `trust` before anything is heard.
fn voice(m: &mut Model, name: &str, trust: f64, reviews: &[(&str, f64)]) {
    let reliable = p(&format!("reliable: {name}"));
    m.base.push((reliable.clone(), trust));
    for (about, count) in reviews {
        m.evidence.push(Evidence {
            piece: reliable.clone(),
            about: p(about),
            value: Opinion {
                r: *count,
                s: 0.0,
                a: 0.5,
            },
            by: Voice::named(name),
        });
    }
}

fn two_voices() {
    println!("\n### 6. Two independent voices\n");
    println!("The cautious reviewer and the overconfident one review the same four objects. Each voice has one piece, its reliability, behind all its reviews, and both start believed at 0.9. The cautious one says 1000 yes on gravity, 1 on each mass, and on completeness 20 for the stone, 5 for the ball, half a yes for the leaf and the feather. The overconfident one says 100 yes on everything.\n");
    let mut good = vec![("gravity", 1000.0)];
    good.extend(MASSES.map(|x| (x, 1.0)));
    good.extend([
        ("complete stone", 20.0),
        ("complete ball", 5.0),
        ("complete leaf", 0.5),
        ("complete feather", 0.5),
    ]);
    let mut bad = vec![("gravity", 100.0)];
    bad.extend(MASSES.map(|x| (x, 100.0)));
    bad.extend(DONE.map(|x| (x, 100.0)));
    let build = |objects: &[(&str, usize, usize)], who: &str| {
        let mut m = reviewed(objects, &[], 0.0);
        if who != "overconfident" {
            voice(&mut m, "cautious", 0.9, &good);
        }
        if who != "cautious" {
            voice(&mut m, "overconfident", 0.9, &bad);
        }
        m
    };
    let rows = [
        "reliable: cautious",
        "reliable: overconfident",
        "complete feather",
        "mass feather",
        "complete stone",
        "drop feather 0",
        "gravity",
    ];
    let at = |m: &Model, x: &str| {
        if !m.base.iter().any(|(piece, _)| piece == &p(x)) {
            return "".to_string();
        }
        let o = opinion(m, &p(x));
        let (from, to) = interval(o, W);
        format!("{:.2} [{from:.2}, {to:.2}]", projected(o, W))
    };
    println!("The stone, the ball and the leaf pass every time, the feather never. With the trial rule.\n");
    println!("| measurements per object | cautious voice | overconfident voice | `complete feather` | `mass feather` | `complete stone` | one of the feather's measurements | `gravity` |");
    println!("|---|---|---|---|---|---|---|---|");
    for k in [1, 3, 10, 30, 100] {
        let objects = [
            ("stone", k, 0),
            ("ball", k, 0),
            ("leaf", k, 0),
            ("feather", 0, k),
        ];
        let m = build(&objects, "both");
        println!("| {k} | {} |", rows.map(|x| at(&m, x)).join(" | "));
    }
    println!("\nThe same at 10 measurements, by who reviews, and without the rule.\n");
    println!("| who reviews | cautious voice | overconfident voice | `complete feather` | `mass feather` | `complete stone` | one of the feather's measurements | `gravity` |");
    println!("|---|---|---|---|---|---|---|---|");
    let objects = [
        ("stone", 10, 0),
        ("ball", 10, 0),
        ("leaf", 10, 0),
        ("feather", 0, 10),
    ];
    for who in ["cautious", "overconfident", "both"] {
        let m = build(&objects, who);
        println!("| {who} | {} |", rows.map(|x| at(&m, x)).join(" | "));
    }
    let m = build(&objects, "both");
    let off = rows.map(|x| format!("{:.2}", projected(opinion(&m, &p(x)), W)));
    println!("| both, without the rule | {} |", off.join(" | "));
    println!("\nNoisy objects, 10 measurements each, both voices, with the rule.\n");
    let objects = [
        ("stone", 9, 1),
        ("ball", 10, 0),
        ("leaf", 6, 4),
        ("feather", 0, 10),
    ];
    let m = build(&objects, "both");
    println!("| object | passes | its mass | its completeness | one failing measurement |");
    println!("|---|---|---|---|---|");
    for (name, pass, fail) in objects {
        let failing = match fail {
            0 => "none".to_string(),
            _ => at(&m, &format!("drop {name} {}", pass + fail - 1)),
        };
        println!(
            "| {name} | {pass} of {} | {} | {} | {failing} |",
            pass + fail,
            at(&m, &format!("mass {name}")),
            at(&m, &format!("complete {name}"))
        );
    }
    println!(
        "\ncautious voice {}, overconfident voice {}, `gravity` {}",
        at(&m, "reliable: cautious"),
        at(&m, "reliable: overconfident"),
        at(&m, "gravity")
    );
}

fn main() {
    println!(
        "Each cell: (yes, no), what it reads at, and [where it is right nine times out of ten].\n"
    );
    println!("### 1. One throw\n");
    println!("A review is believed at 0.9, the throw too.\n");
    println!(
        "| review of the statement | review of the tie | I saw | `falls` | `applies` | `throw` |"
    );
    println!("|---|---|---|---|---|---|");
    for (statement, tie) in [(0.0, 0.0), (10.0, 0.0), (10.0, 10.0), (100.0, 0.0)] {
        for fell in [true, false] {
            let m = one_throw(statement, tie, fell);
            let count = |c: f64| {
                if c > 0.0 {
                    format!("{c} yes")
                } else {
                    "none".to_string()
                }
            };
            let saw = if fell { "fall" } else { "no fall" };
            let cells = ["falls", "applies", "throw"].map(|x| cell(&m, x));
            println!(
                "| {} | {} | {saw} | {} |",
                count(statement),
                count(tie),
                cells.join(" | ")
            );
        }
    }

    println!("\n### 2. Gravity reviewed or not, against failures\n");
    println!("10 measurements per object. \"Centuries\" is a review of 1000 yes about gravity, believed at 0.99.\n");
    println!("| objects that pass | that fail | gravity not reviewed | gravity reviewed | mass of one that fails, gravity not reviewed | gravity reviewed |");
    println!("|---|---|---|---|---|---|");
    for (pass, fail) in [(0, 1), (0, 3), (0, 10), (2, 8), (9, 1)] {
        let (bare, firm) = (
            sweep(pass, fail, 10, 0.0, 0.0),
            sweep(pass, fail, 10, 1000.0, 0.0),
        );
        println!(
            "| {pass} | {fail} | {} | {} | {} | {} |",
            cell(&bare, "gravity"),
            cell(&firm, "gravity"),
            cell(&bare, "mass f0"),
            cell(&firm, "mass f0")
        );
    }

    println!("\n### 3. The feather's own statement reviewed too\n");
    println!("Gravity reviewed (1000 yes), 3 objects pass, the feather fails, 10 measurements each. \"I weighed it\" is a review about the feather's mass, believed at 0.9.\n");
    println!("| review of the feather's mass | `gravity` | `mass f0` | one of its measurements | mass of a new object measured once |");
    println!("|---|---|---|---|---|");
    for mass in [0.0, 1.0, 10.0, 100.0] {
        let mut m = sweep(3, 1, 10, 1000.0, mass);
        m.base.push((p("mass new"), 0.5));
        whole(&mut m, &["same speed new", "gravity", "mass new"]);
        says(&mut m, "drop new", "same speed new", 1.0, 0.0, 0.9);
        let count = if mass > 0.0 {
            format!("{mass} yes")
        } else {
            "none".to_string()
        };
        let cells = ["gravity", "mass f0", "drop f0 0", "mass new"].map(|x| cell(&m, x));
        println!("| {count} | {} |", cells.join(" | "));
    }
    reviewers();
    two_voices();
}
