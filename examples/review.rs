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
    let (from, to) = interval(o, weight());
    format!(
        "({:.1}, {:.2}) {:.2} [{from:.2}, {to:.2}]",
        o.r,
        o.s,
        projected(o)
    )
}

/// One throw, with reviews of so many counts on the statement and the tie.
fn one_throw(statement: f64, tie: f64, fell: bool) -> Model {
    let mut m = Model {
        base: vec![(p("applies"), 0.5), (p("falls"), 0.5)],
        ..Model::default()
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
    let mut m = Model::default();
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
}
