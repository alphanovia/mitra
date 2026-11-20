//! Completeness, tried by hand on the two foundation examples. For
//! each tie, one more piece: `complete`, "the statements on this tie
//! are all that matter for it". A prediction on a tie is all of
//! [its statements..., complete]. Here the piece is written in each
//! model; the core does not imply it yet.
//!
//! `cargo run --release --example complete`

use mitra_core::model::*;

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
fn says(m: &mut Model, name: &str, about: &str, yes_: bool, trust: f64) {
    m.base.push((p(name), trust));
    m.evidence.push(Evidence {
        piece: p(name),
        about: p(about),
        value: if yes_ { yes() } else { no() },
        by: Voice::named(name),
    });
}
fn cell(m: &Model, x: &str) -> String {
    let o = opinion(m, &p(x));
    format!("({:.2}, {:.2}) {:.2}", o.r, o.s, projected(o))
}

/// One object, one statement, one throw.
fn one_throw(complete: f64, senses: f64, fell: bool) -> Model {
    let mut m = Model {
        base: vec![(p("falls"), 0.5), (p("complete"), complete)],
        ..Model::default()
    };
    whole(&mut m, &["lands", "falls", "complete"]);
    says(&mut m, "throw", "lands", fell, senses);
    m
}

/// How the scale speaks of the feather's mass.
#[derive(Clone, Copy, PartialEq)]
enum Scale {
    None,
    /// Evidence about the statement `mass feather` itself.
    OnTheMass,
    /// A prediction on the tie: all of [mass feather, complete feather].
    ThroughTheTie,
}

/// `pass` objects that fall at the same speed `k` times each, `fail`
/// that never do; the last failing one is the feather.
fn sweep(pass: usize, fail: usize, k: usize, complete: f64, scale: Scale) -> Model {
    let mut m = Model::default();
    m.base.push((p("gravity"), 0.5));
    for i in 0..pass + fail {
        let name = match i {
            i if i + 1 == pass + fail && fail > 0 => "feather".to_string(),
            i if i < pass => format!("o{i}"),
            i => format!("f{i}"),
        };
        let (mass, done, speed) = (
            format!("mass {name}"),
            format!("complete {name}"),
            format!("same speed {name}"),
        );
        m.base.push((p(&mass), 0.5));
        m.base.push((p(&done), complete));
        whole(&mut m, &[&speed, "gravity", &mass, &done]);
        for j in 0..k {
            says(&mut m, &format!("drop {name} {j}"), &speed, i < pass, 0.9);
        }
    }
    match scale {
        Scale::None => {}
        Scale::OnTheMass => says(&mut m, "scale", "mass feather", true, 0.9),
        Scale::ThroughTheTie => {
            whole(
                &mut m,
                &["weighs feather", "mass feather", "complete feather"],
            );
            says(&mut m, "scale", "weighs feather", true, 0.9);
        }
    }
    m
}

fn main() {
    println!("### 1. One throw: `lands` is all of [falls, complete]\n");
    println!("| complete starts at | senses | I saw | `falls` | `complete` | `throw` |");
    println!("|---|---|---|---|---|---|");
    for complete in [0.5, 0.9] {
        for (senses, fell) in [(0.9, true), (0.9, false), (1.0, false)] {
            let m = one_throw(complete, senses, fell);
            let saw = if fell { "fall" } else { "no fall" };
            let cells = ["falls", "complete", "throw"].map(|x| cell(&m, x));
            println!("| {complete} | {senses} | {saw} | {} |", cells.join(" | "));
        }
    }

    for complete in [0.5, 0.9] {
        println!("\n### 2. The feather, completeness starting at {complete}\n");
        println!("10 objects pass, the feather fails, 10 measurements each.\n");
        println!("| the scale | `gravity` | `mass feather` | `complete feather` | `mass o0` | `complete o0` |");
        println!("|---|---|---|---|---|---|");
        let rows = [
            ("no scale", Scale::None),
            ("says yes about the mass", Scale::OnTheMass),
            ("is a prediction on the tie", Scale::ThroughTheTie),
        ];
        for (title, scale) in rows {
            let m = sweep(10, 1, 10, complete, scale);
            let cells = [
                "gravity",
                "mass feather",
                "complete feather",
                "mass o0",
                "complete o0",
            ]
            .map(|x| cell(&m, x));
            println!("| {title} | {} |", cells.join(" | "));
        }
    }

    println!("\n### 3. How many of 10 objects fail (10 measurements each, completeness at 0.5)\n");
    println!("| objects that fail | `gravity` | `mass` of one that fails | `complete` of one that fails | `complete` of one that passes |");
    println!("|---|---|---|---|---|");
    for fail in [0, 1, 2, 5, 8, 9, 10] {
        let m = sweep(10 - fail, fail, 10, 0.5, Scale::None);
        let failing = |x: &str| match fail {
            0 => "none".to_string(),
            _ => cell(&m, &format!("{x} feather")),
        };
        let passing = match fail {
            10 => "none".to_string(),
            _ => cell(&m, "complete o0"),
        };
        println!(
            "| {fail} | {} | {} | {} | {passing} |",
            cell(&m, "gravity"),
            failing("mass"),
            failing("complete")
        );
    }
}
