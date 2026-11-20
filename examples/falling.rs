//! The speed of a fall: a claim computed with the standard table, on
//! what was measured, becomes evidence in the model.
//!
//! One statement, `speed`: after falling a height `h`, the speed `v`
//! satisfies v² = 2·g·h within a tolerance. A stone and a feather are
//! each tied to it. A measurement is a height and a speed; computing
//! the claim on it says yes or no about the whole of the object's tie.

use mitra_core::claim::{measured, Claim};
use mitra_core::model::*;
use mitra_core::standard::{mul, ratio, real, within, Table};
use mitra_core::value::Expression::{self, Apply, Given, Input};
use mitra_core::value::Value;

/// The settings these are recorded at.
const S: Settings = Settings {
    weight: 0.2,
    completeness: 0.6,
    tie: 0.5,
    reliability: None,
};

fn r(n: i64, d: i64) -> Value {
    ratio(n, d).unwrap()
}
fn input(name: &str) -> Expression {
    Input(name.to_string(), real())
}
fn times(x: Expression, y: Expression) -> Expression {
    Apply(mul(&real()), vec![x, y])
}

/// v² = 2·g·h within 2 m²/s². g is 9.81 m/s².
fn speed() -> Statement<Claim> {
    let fallen = times(times(Given(r(2, 1)), Given(r(981, 100))), input("h"));
    let seen = times(input("v"), input("v"));
    Statement {
        piece: Piece::named("speed"),
        content: Claim {
            expression: Apply(within(&real()), vec![seen, fallen, Given(r(2, 1))]),
        },
    }
}

fn tie_of(object: &str) -> Tie {
    Tie {
        piece: Piece::named(&format!("{object} applies")),
        statements: vec![Piece::named("speed")],
        place: Place::at(&[object]),
    }
}

/// The model after these measurements: (object, height, speed), each
/// in hundredths. Each measurement is believed at 0.9 before anything.
fn after(measurements: &[(&str, i64, i64)]) -> Model {
    let mut m = Model::empty(S);
    m.base.push((Piece::named("speed"), 0.5));
    for (object, h, v) in measurements {
        let tie = tie_of(object);
        if !m.ties.contains(&tie) {
            m.ties.push(tie.clone());
        }
        let inputs = [("h".to_string(), r(*h, 100)), ("v".to_string(), r(*v, 100))];
        let by = Voice::named(&format!("machine:timer {object} {h}"));
        if let Some(e) = measured(&speed(), &tie, &inputs, &by, &Table::default()) {
            m.base.push((e.piece.clone(), 0.9));
            m.evidence.push(e);
        }
    }
    m
}

fn main() {
    let stone = [
        ("stone", 500, 990),
        ("stone", 1000, 1400),
        ("stone", 2000, 1980),
    ];
    let feather = [("feather", 500, 200), ("feather", 1000, 210)];
    let both: Vec<_> = stone.iter().chain(&feather).cloned().collect();
    println!("| measured | says |");
    println!("|---|---|");
    for (object, h, v) in &both {
        let m = after(&[(object, *h, *v)]);
        let said = match m.evidence.first() {
            Some(e) if e.value == yes() => "yes",
            Some(_) => "no",
            None => "nothing",
        };
        println!(
            "| {object}: {} m, {} m/s | {said} |",
            *h as f64 / 100.0,
            *v as f64 / 100.0
        );
    }
    let read = |m: &Model, x: &str| match x.contains("applies")
        && !m.ties.iter().any(|t| x.ends_with(&t.piece.name))
    {
        true => "-".to_string(),
        false => format!("{:.2}", projected(opinion(m, &Piece::named(x)), S.weight)),
    };
    println!(
        "\n| after | `speed` | stone's tie | its completeness | feather's tie | its completeness |"
    );
    println!("|---|---|---|---|---|---|");
    for (name, m) in [
        ("the stone alone", after(&stone)),
        ("the feather alone", after(&feather)),
        ("both", after(&both)),
    ] {
        println!(
            "| {name} | {} | {} | {} | {} | {} |",
            read(&m, "speed"),
            read(&m, "stone applies"),
            read(&m, "complete: stone applies"),
            read(&m, "feather applies"),
            read(&m, "complete: feather applies"),
        );
    }
}
