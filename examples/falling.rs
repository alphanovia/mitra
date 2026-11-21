//! The speed of a fall: a claim computed with the standard table, on
//! what was measured, becomes evidence in the model.
//!
//! One statement, `speed`: after falling a height `h`, the speed `v`
//! satisfies v² = 2·g·h within a tolerance. A stone and a feather are
//! each tied to it. A measurement is a height and a speed; computing
//! the claim on it says yes or no about the claim at that height and
//! speed, through the object's tie.

use mitra_core::bounds::{between, Ranges};
use mitra_core::claim::{bounded, instance, observe, recorded, Claim};
use mitra_core::model::*;
use mitra_core::standard::{mul, ratio, real, within, Exact, Table};
use mitra_core::verdict::*;

type Value = mitra_core::value::Value<Exact>;
use mitra_core::value::Expression::{Apply, Given, Input};

type Expression = mitra_core::value::Expression<Exact>;

/// The settings these are recorded at.
const S: Settings = Settings {
    weight: 0.2,
    statement: 0.5,
    evidence: 0.9,
    tie: 0.5,
    completeness: 0.6,
    reliability: 1.0,
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
/// in hundredths. Each measurement is believed at 0.9 before anything,
/// and a review says the measured falls stand for all falls.
fn after(measurements: &[(&str, i64, i64)]) -> Model {
    let uniform = Piece::named("speed::uniform");
    let mut m = Model::empty(S);
    for (object, h, v) in measurements {
        let by = Voice::named(&format!("machine:timer {object} {h}"));
        if let Some(e) = observe(&speed(), &at(*h, *v), &by, &Table::default()) {
            m = recorded(&m, &speed(), e, Some(&tie_of(object)), Some(&uniform)).unwrap();
        }
    }
    m.evidence.push(Evidence {
        piece: Piece::named("review of uniform"),
        about: uniform,
        value: yes(),
        by: Voice::named("agent:reviewer"),
    });
    m
}
fn at(h: i64, v: i64) -> [(String, mitra_core::value::Value<Exact>); 2] {
    [("h".to_string(), r(h, 100)), ("v".to_string(), r(v, 100))]
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
        let seen = observe(
            &speed(),
            &at(*h, *v),
            &Voice::named("me"),
            &Table::default(),
        );
        let said = match seen {
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
    let rows = [
        ("one fall of the stone", after(&stone[..1])),
        ("the stone alone", after(&stone)),
        ("the feather alone", after(&feather)),
        ("both", after(&both)),
    ];
    println!(
        "\n| after | verdict | `speed` | at the stone's 5 m | at the feather's 5 m | stone's tie | its completeness | feather's tie | its completeness |"
    );
    println!("|---|---|---|---|---|---|---|---|---|");
    for (name, m) in &rows {
        let reading = read(m);
        let p = |x: &str| {
            let found = reading.pieces.iter().find(|(piece, _)| piece.name == x);
            found.map_or("unknown".to_string(), |(_, v)| shown(v, S.weight))
        };
        println!(
            "| {name} | {} | {} | {} | {} | {} | {} | {} | {} |",
            format!("{:?}", overall(m)).to_lowercase(),
            p("speed"),
            p(&instance(&speed(), &at(500, 990)).name),
            p(&instance(&speed(), &at(500, 200)).name),
            p("stone applies"),
            p("complete: stone applies"),
            p("feather applies"),
            p("complete: feather applies"),
        );
    }
    println!();
    for (name, m) in &rows {
        for group in suspects(m) {
            let names: Vec<&str> = group.iter().map(|p| p.name.as_str()).collect();
            println!("{name}: cannot all be true: {}", names.join(", "));
        }
    }

    // The same claim read by bounds: the height and the speed are each
    // known to a hundredth, and one reading stands for every point in
    // those margins. Here it is worth ten observations.
    let margin = |x: i64| between(&real(), (x * 10 - 10, 1000), (x * 10 + 10, 1000)).unwrap();
    let tie = tie_of("stone");
    let by = Voice::named("machine:timer");
    let with = |e: Evidence| recorded(&Model::empty(S), &speed(), e, Some(&tie), None).unwrap();
    let inputs = at(500, 990);
    let point = with(observe(&speed(), &inputs, &by, &Table::default()).unwrap());
    let ranges = [
        ("h".to_string(), margin(500)),
        ("v".to_string(), margin(990)),
    ];
    let ranged = bounded(&speed(), &ranges, 10.0, &by, &Ranges).unwrap();
    let range = with(ranged);
    println!("\n| the stone, 5 m at 9.9 m/s | `speed` there |");
    println!("|---|---|");
    for (name, m, x) in [
        ("read at the point", point, instance(&speed(), &inputs)),
        ("read over its margins", range, instance(&speed(), &ranges)),
    ] {
        println!("| {name} | {} |", shown(&verdict_of(&m, &x), S.weight));
    }
}
