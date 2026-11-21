//! A function, its spec, and what a test of it says.
//!
//! `double(n)` has the body `n * 2`. One test at 3 passes. With no
//! `uniform` nothing leads from that test to `double::correct`; with
//! `uniform`, reviewed, it does. Then a run at 7 gives 15.

use mitra_core::function::*;
use mitra_core::model::*;
use mitra_core::standard::{mul, number, whole, Exact, Table};
use mitra_core::value::Expression::{Apply, Given, Input};
use mitra_core::value::Operation;
use mitra_core::verdict::*;

/// The settings these are recorded at.
const S: Settings = Settings {
    weight: 0.2,
    statement: 0.5,
    evidence: 0.9,
    tie: 0.5,
    completeness: 0.6,
    reliability: 1.0,
};

fn double(uniform: bool) -> Function {
    let n = Input::<Exact>("n".to_string(), whole());
    Function {
        operation: Operation {
            name: "double".to_string(),
            takes: vec![whole()],
            gives: whole(),
        },
        inputs: vec!["n".to_string()],
        body: Some(Apply(mul(&whole()), vec![n, Given(number(2))])),
        uniform,
        claims: vec![],
        at: Some(Place::at(&["src/lib.rs", "double"])),
    }
}
fn run(f: &Function, n: i64, gave: i64) -> Run {
    Run {
        operation: f.operation.clone(),
        args: vec![number(n)],
        result: number(gave),
    }
}
/// The model after these runs, each a test by `cargo`.
fn after(f: &Function, runs: &[(i64, i64)], reviewed: bool) -> Model {
    let table = Table::default();
    let known = Known {
        table: &table,
        helpers: &[],
        runs: &[],
    };
    let mut m = Model::empty(S);
    for (n, gave) in runs {
        let by = Voice::named(&format!("machine:cargo test at {n}"));
        let test = tested(f, &run(f, *n, *gave), &by, &known).unwrap();
        m = noted(&m, f, test).unwrap();
    }
    if let (true, Some(hypothesis)) = (reviewed, uniformity(f)) {
        m.evidence.push(Evidence {
            piece: Piece::named("review of uniform"),
            about: hypothesis,
            value: yes(),
            by: Voice::named("agent:reviewer"),
        });
    }
    m
}

fn main() {
    let rows = [
        (
            "no `uniform`, a test at 3",
            after(&double(false), &[(3, 6)], false),
        ),
        (
            "no `uniform`, then a run at 7 gives 15",
            after(&double(false), &[(3, 6), (7, 15)], false),
        ),
        (
            "`uniform` not reviewed, a test at 3",
            after(&double(true), &[(3, 6)], false),
        ),
        (
            "`uniform` reviewed, a test at 3",
            after(&double(true), &[(3, 6)], true),
        ),
        (
            "then a run at 7 gives 15",
            after(&double(true), &[(3, 6), (7, 15)], true),
        ),
        (
            "tests at 1, 2, 3 and 4, `uniform` reviewed",
            after(&double(true), &[(1, 2), (2, 4), (3, 6), (4, 8)], true),
        ),
        (
            "then a run at 7 gives 15",
            after(
                &double(true),
                &[(1, 2), (2, 4), (3, 6), (4, 8), (7, 15)],
                true,
            ),
        ),
    ];
    println!("| | verdict | `double::correct` | at 3 | at 7 | `double::uniform` | the tie | its completeness |");
    println!("|---|---|---|---|---|---|---|---|");
    let w = S.weight;
    for (name, m) in &rows {
        let reading = read(m);
        let p = |x: &str| {
            let found = reading.pieces.iter().find(|(piece, _)| piece.name == x);
            found.map_or("unknown".to_string(), |(_, v)| shown(v, w))
        };
        println!(
            "| {name} | {} | {} | {} | {} | {} | {} | {} |",
            match least_sure(m) {
                Some((_, n)) => format!("true {n:.2}"),
                None => format!("{:?}", overall(m)).to_lowercase(),
            },
            p("double::correct"),
            p("double::correct @ n = Whole(3)"),
            p("double::correct @ n = Whole(7)"),
            p("double::uniform"),
            p("double::tie"),
            p("complete: double::tie"),
        );
    }
    println!();
    let m = &rows.last().unwrap().1;
    println!("After four tests and the run at 7, what may be given up, the lightest first:");
    for group in suspects(m) {
        let short = |p: &Piece| match p.name.split(" test at ").nth(1) {
            Some(n) => format!("the run at {n}"),
            None => format!("the {}", p.name),
        };
        let names: Vec<String> = group.iter().map(short).collect();
        let mut left = m.clone();
        left.evidence.retain(|e| !group.contains(&e.piece));
        let correct = verdict_of(&left, &Piece::named("double::correct"));
        println!(
            "- {}: then `double::correct` is {}",
            names.join(", "),
            shown(&correct, w)
        );
    }
}
