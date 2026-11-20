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
    let mut m = placed(&Model::empty(S), f);
    for (n, gave) in runs {
        let by = Voice::named(&format!("machine:cargo test at {n}"));
        m.evidence
            .push(tested(f, &run(f, *n, *gave), &by, &known).unwrap());
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
    println!("| | `double::correct` | its belief | `double::seen` | `double::uniform` | the tie | its completeness |");
    println!("|---|---|---|---|---|---|---|");
    for (name, m) in &rows {
        let o = |x: &str| opinion(m, &Piece::named(x));
        let p = |x: &str| projected(o(x), S.weight);
        println!(
            "| {name} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} | {:.2} |",
            p("double::correct"),
            belief(o("double::correct"), S.weight),
            p("double::seen"),
            p("double::uniform"),
            p("double::tie"),
            p("complete: double::tie"),
        );
    }
    println!();
    for at in [3, 5] {
        for group in conflicts(&rows[at].1) {
            let short = |p: &Piece| match p.name.contains(" at [") {
                true => "the run at 7".to_string(),
                false => p.name.clone(),
            };
            let names: Vec<String> = group.iter().map(short).collect();
            println!(
                "cannot all be right, least believed first: {}",
                names.join(", ")
            );
        }
    }
    let o = opinion(&rows[3].1, &Piece::named("double::correct"));
    println!(
        "\n`double::correct` after the run at 7: {:.2} yes, {:.2} no, base rate {:.2}",
        o.r, o.s, o.a
    );
}
