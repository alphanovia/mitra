//! The experiment "objects fall at the same speed", swept. The model
//! holds `gravity`, a latent statement shared by every object, and one
//! `mass` statement tied to each object; "object k falls at the same
//! speed" is all of [gravity, mass k]. Each measurement is evidence
//! about that whole, believed at 0.9, with a voice of its own unless
//! said. Everything else starts at one half.
//!
//! `cargo run --release --example sweep`

use mitra_core::model::*;

/// The weight of a base rate that these are recorded at.
const W: f64 = 0.2;
/// What to expect of the completeness of a tie when nothing says.
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

/// One object: its name, how many of its measurements pass, how many fail.
type Object = (String, usize, usize);

fn world(objects: &[Object], one_voice_per_object: bool) -> Model {
    let mut m = Model::empty(S);
    m.base.push((p("gravity"), 0.5));
    for (name, pass, fail) in objects {
        let (mass, whole) = (format!("mass {name}"), format!("same speed {name}"));
        let relation = p(&format!("all: {whole}"));
        m.related.push(Statement {
            piece: relation.clone(),
            content: Related {
                how: Relation::All,
                pieces: vec![p(&whole), p("gravity"), p(&mass)],
            },
        });
        m.base.push((relation, 1.0));
        m.base.push((p(&mass), 0.5));
        for k in 0..pass + fail {
            let drop = format!("drop {name} {k}");
            let by = match one_voice_per_object {
                true => format!("me on {name}"),
                false => drop.clone(),
            };
            m.base.push((p(&drop), 0.9));
            m.evidence.push(Evidence {
                piece: p(&drop),
                about: p(&whole),
                value: if k < *pass { yes() } else { no() },
                by: Voice::named(&by),
            });
        }
    }
    m
}

fn at(m: &Model, x: &str) -> f64 {
    projected(opinion(m, &p(x)), W)
}
fn counts(m: &Model, x: &str) -> String {
    let o = opinion(m, &p(x));
    format!("({:.1}, {:.2}) {:.2}", o.r, o.s, projected(o, W))
}

/// `n` objects that pass every one of `k` measurements, and a feather
/// that fails every one of its `k`.
fn feather(n: usize, k: usize, one_voice: bool) -> Model {
    let mut objects: Vec<Object> = (0..n).map(|i| (format!("o{i}"), k, 0)).collect();
    objects.push(("feather".to_string(), 0, k));
    world(&objects, one_voice)
}

fn main() {
    let grid = [1, 2, 5, 10, 20, 50, 100];
    let tables: [(&str, &str); 4] = [
        ("gravity", "gravity"),
        ("the mass of an object that passes", "mass o0"),
        ("the mass of the feather", "mass feather"),
        ("one measurement of the feather", "drop feather 0"),
    ];
    for (title, piece) in tables {
        println!("\n### What {title} reads at\n");
        println!("Rows: objects that pass. Columns: measurements per object.\n");
        let head: Vec<String> = grid.iter().map(|k| k.to_string()).collect();
        println!("| objects | {} |", head.join(" | "));
        println!("|---|{}", "---|".repeat(grid.len()));
        for n in grid {
            let row: Vec<String> = grid
                .iter()
                .map(|k| format!("{:.2}", at(&feather(n, *k, false), piece)))
                .collect();
            println!("| {n} | {} |", row.join(" | "));
        }
    }

    println!("\n### The counts, at 10 measurements per object\n");
    println!("| objects that pass | gravity | mass of one that passes | mass of the feather | one measurement of the feather |");
    println!("|---|---|---|---|---|");
    for n in grid {
        let m = feather(n, 10, false);
        let cells = ["gravity", "mass o0", "mass feather", "drop feather 0"].map(|x| counts(&m, x));
        println!("| {n} | {} |", cells.join(" | "));
    }

    println!("\n### One voice per object instead (10 objects pass)\n");
    println!(
        "| measurements per object | gravity | mass of one that passes | mass of the feather |"
    );
    println!("|---|---|---|---|");
    for k in [1, 10, 100] {
        let m = feather(10, k, true);
        let cells = ["gravity", "mass o0", "mass feather"].map(|x| counts(&m, x));
        println!("| {k} | {} |", cells.join(" | "));
    }

    println!("\n### Noisy objects: 20 measurements each\n");
    println!("16 objects pass 19 of 20, 3 odd ones pass 12 of 20, the feather 0 of 20.\n");
    let mut objects: Vec<Object> = (0..16).map(|i| (format!("o{i}"), 19, 1)).collect();
    objects.extend((0..3).map(|i| (format!("odd{i}"), 12, 8)));
    objects.push(("feather".to_string(), 0, 20));
    let m = world(&objects, false);
    println!("| piece | counts and what it reads at |");
    println!("|---|---|");
    for x in [
        "gravity",
        "mass o0",
        "mass odd0",
        "mass feather",
        "drop o0 0",
        "drop o0 19",
        "drop odd0 0",
        "drop odd0 19",
        "drop feather 0",
    ] {
        println!("| `{x}` | {} |", counts(&m, x));
    }

    println!("\n### How many of the objects fail (10 objects, 10 measurements each)\n");
    println!("| objects that fail every time | gravity | mass of one that passes | mass of one that fails |");
    println!("|---|---|---|---|");
    for fail in [0, 1, 2, 5, 8, 10] {
        let mut objects: Vec<Object> = (0..10 - fail).map(|i| (format!("o{i}"), 10, 0)).collect();
        objects.extend((0..fail).map(|i| (format!("f{i}"), 0, 10)));
        let m = world(&objects, false);
        let pass = if fail < 10 {
            counts(&m, "mass o0")
        } else {
            "none".to_string()
        };
        let failing = if fail > 0 {
            counts(&m, "mass f0")
        } else {
            "none".to_string()
        };
        println!(
            "| {fail} | {} | {pass} | {failing} |",
            counts(&m, "gravity")
        );
    }
}
