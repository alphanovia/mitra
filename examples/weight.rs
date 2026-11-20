//! A weight that grows with the evidence in the model, tried on the
//! two foundation examples beside the fixed weight.
//!
//! ```text
//! W = 0.2 + 0.2 · ln(1 + pieces of evidence in the model)
//! ```
//!
//! `cargo run --release --example weight`

use mitra_core::model::*;

mod common;
use common::{interval, ln_beta};

/// What to expect of the completeness of a tie when nothing says.
const C: f64 = 0.6;
/// The settings these are recorded at: a tie nobody gave a base rate
/// is expected wrong, and a voice nobody gave one is not judged.
const S: Settings = Settings {
    weight: 0.2,
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
fn says(m: &mut Model, name: &str, about: &str, yes_: bool, trust: f64) {
    m.base.push((p(name), trust));
    m.evidence.push(Evidence {
        piece: p(name),
        about: p(about),
        value: if yes_ { yes() } else { no() },
        by: Voice::named(name),
    });
}
/// The same model under another weight.
fn under(m: &Model, w: f64) -> Model {
    Model {
        settings: Settings { weight: w, ..S },
        ..m.clone()
    }
}
fn scaled(m: &Model) -> f64 {
    0.2 + 0.2 * (1.0 + m.evidence.len() as f64).ln()
}
/// What `x` reads at under the fixed weight, and under the scaled one.
fn both(m: &Model, x: &str) -> String {
    let at = |w: f64| projected(opinion(&under(m, w), &p(x)), w);
    format!("{:.2} / {:.2}", at(0.2), at(scaled(m)))
}

fn one_throw(senses: f64, fell: bool) -> Model {
    let mut m = Model {
        base: vec![(p("applies"), 0.5), (p("falls"), 0.5)],
        ..Model::empty(S)
    };
    whole(&mut m, &["lands", "applies", "falls"]);
    says(&mut m, "throw", "lands", fell, senses);
    m
}

/// Objects: (name, measurements that pass, that fail).
fn sweep(objects: &[(String, usize, usize)]) -> Model {
    let mut m = Model::empty(S);
    m.base.push((p("gravity"), 0.5));
    for (name, pass, fail) in objects {
        let (mass, speed) = (format!("mass {name}"), format!("same speed {name}"));
        m.base.push((p(&mass), 0.5));
        whole(&mut m, &[&speed, "gravity", &mass]);
        for j in 0..pass + fail {
            says(&mut m, &format!("drop {name} {j}"), &speed, j < *pass, 0.9);
        }
    }
    m
}
fn objects(pass: usize, fail: usize, k: usize) -> Vec<(String, usize, usize)> {
    let mut all: Vec<_> = (0..pass).map(|i| (format!("o{i}"), k, 0)).collect();
    all.extend((0..fail).map(|i| (format!("f{i}"), 0, k)));
    all
}

/// The pieces that can be doubted: a base rate strictly between 0 and 1.
fn doubted(m: &Model) -> Vec<Piece> {
    let open = m.base.iter().filter(|(_, a)| *a > 0.0 && *a < 1.0);
    open.map(|(x, _)| x.clone()).collect()
}

const GRID: [f64; 10] = [0.02, 0.05, 0.1, 0.2, 0.5, 1.0, 2.0, 5.0, 10.0, 20.0];

/// Pass (a): the weight under which the counts of the model are the
/// most likely. Each piece is a Beta that starts at (a·W, (1-a)·W) and
/// has seen its counts of yes and no.
fn most_likely(m: &Model) -> f64 {
    let likelihood = |w: f64| log_likelihood(m, w);
    let mut best = (GRID[0], f64::MIN);
    for w in GRID {
        let l = likelihood(w);
        if l > best.1 {
            best = (w, l);
        }
    }
    best.0
}

/// How far the least heard piece of evidence moves what it speaks of:
/// the reading of the parts of its whole, with it and without it.
fn least_influence(m: &Model, w: f64) -> f64 {
    let m = &under(m, w);
    let mut least = f64::MAX;
    for k in 0..m.evidence.len() {
        let mut without = m.clone();
        let e = without.evidence.remove(k);
        let whole = m.related.iter().find(|r| r.content.pieces[0] == e.about);
        let spoken: Vec<Piece> = match whole {
            Some(r) => r.content.pieces[1..].to_vec(),
            None => vec![e.about.clone()],
        };
        let mut moved_by: f64 = 0.0;
        for x in &spoken {
            let at = |m: &Model| projected(opinion(m, x), w);
            moved_by = moved_by.max((at(m) - at(&without)).abs());
        }
        least = least.min(moved_by);
    }
    least
}

const EPSILON: f64 = 0.01;

/// Pass (b): the smallest weight under which every piece of evidence
/// moves a reading by at least EPSILON; when there is none, the weight
/// under which the least heard one moves the most.
fn influential(m: &Model) -> (f64, f64, bool) {
    let mut best = (GRID[0], -1.0);
    for w in GRID {
        let least = least_influence(m, w);
        if least >= EPSILON {
            return (w, least, true);
        }
        if least > best.1 {
            best = (w, least);
        }
    }
    (best.0, best.1, false)
}

fn passes() {
    println!("\n### 5. Two passes that choose the weight\n");
    println!("(a) the weight under which the counts are the most likely; (b) the smallest weight under which every piece of evidence moves a reading by at least {EPSILON}. Both over the grid {GRID:?}.\n");
    let mut noisy: Vec<_> = (0..3).map(|i| (format!("o{i}"), 9, 1)).collect();
    noisy.push(("odd".to_string(), 6, 4));
    noisy.push(("feather".to_string(), 0, 10));
    let cases: Vec<(&str, Model, Vec<&str>)> = vec![
        ("one throw, it falls", one_throw(0.9, true), vec!["applies"]),
        (
            "one throw, it does not",
            one_throw(0.9, false),
            vec!["applies"],
        ),
        (
            "10 objects pass once, one fails",
            sweep(&objects(10, 1, 1)),
            vec!["gravity", "mass o0", "mass f0"],
        ),
        (
            "3 objects pass 10 times, one fails",
            sweep(&objects(3, 1, 10)),
            vec!["gravity", "mass o0", "mass f0"],
        ),
        (
            "2 of 10 objects pass, 5 times each",
            sweep(&objects(2, 8, 5)),
            vec!["gravity", "mass o0", "mass f0"],
        ),
        (
            "noisy: 9 of 10, 6 of 10, 0 of 10",
            sweep(&noisy),
            vec![
                "mass o0",
                "drop o0 9",
                "mass odd",
                "drop odd 9",
                "mass feather",
            ],
        ),
    ];
    println!("| case | piece | fixed 0.2 | scaled | (a) most likely | (b) influence |");
    println!("|---|---|---|---|---|---|");
    for (title, m, pieces) in cases {
        let (grown, likely) = (scaled(&m), most_likely(&m));
        let (heard, least, reached) = influential(&m);
        let note = if reached { "" } else { ", not reached" };
        println!("| **{title}** | the weight | 0.2 | {grown:.2} | {likely} | {heard} (least move {least:.3}{note}) |");
        for x in pieces {
            let at = |w: f64| projected(opinion(&under(&m, w), &p(x)), w);
            println!(
                "| | `{x}` | {:.2} | {:.2} | {:.2} | {:.2} |",
                at(0.2),
                at(grown),
                at(likely),
                at(heard)
            );
        }
    }
}

fn log_likelihood(m: &Model, w: f64) -> f64 {
    let m = &under(m, w);
    let mut sum = 0.0;
    for x in doubted(m) {
        let o = opinion(m, &x);
        let (yes_, no_) = (o.a * w, (1.0 - o.a) * w);
        sum += ln_beta(o.r + yes_, o.s + no_) - ln_beta(yes_, no_);
    }
    sum
}

/// The word of a piece at policy 0.9.
fn word(o: Opinion, w: f64) -> &'static str {
    match (reaches(o, 0.9, w), against(o)) {
        (true, _) => "holds",
        (false, true) => "fails",
        (false, false) => "unverified",
    }
}

fn bounds() {
    println!("\n### 6. Bounds\n");
    println!("For each piece: what it reads at under 0.2 with the interval where it is right nine times out of ten; then the weights the counts allow (within 2 of the best log-likelihood, over 40 weights from 0.01 to 50), what the piece reads at both ends, and its word at policy 0.9.\n");
    let mut noisy: Vec<_> = (0..3).map(|i| (format!("o{i}"), 9, 1)).collect();
    noisy.push(("odd".to_string(), 6, 4));
    noisy.push(("feather".to_string(), 0, 10));
    let cases: Vec<(&str, Model, Vec<&str>)> = vec![
        ("one throw, it falls", one_throw(0.9, true), vec!["applies"]),
        (
            "one throw, it does not",
            one_throw(0.9, false),
            vec!["applies"],
        ),
        (
            "10 objects pass once, one fails",
            sweep(&objects(10, 1, 1)),
            vec!["gravity", "mass o0", "mass f0"],
        ),
        (
            "3 objects pass 10 times, one fails",
            sweep(&objects(3, 1, 10)),
            vec!["gravity", "mass o0", "mass f0"],
        ),
        (
            "2 of 10 objects pass, 5 times each",
            sweep(&objects(2, 8, 5)),
            vec!["gravity", "mass o0", "mass f0"],
        ),
        (
            "noisy: 9 of 10, 6 of 10, 0 of 10",
            sweep(&noisy),
            vec![
                "mass o0",
                "drop o0 9",
                "mass odd",
                "drop odd 9",
                "mass feather",
            ],
        ),
    ];
    println!("| case | piece | at 0.2, nine times out of ten | weights allowed | reads at both ends | word at both ends |");
    println!("|---|---|---|---|---|---|");
    let weights: Vec<f64> = (0..40)
        .map(|k| 0.01 * (5000.0_f64).powf(k as f64 / 39.0))
        .collect();
    for (title, m, pieces) in cases {
        let scores: Vec<f64> = weights.iter().map(|w| log_likelihood(&m, *w)).collect();
        let best = scores.iter().cloned().fold(f64::MIN, f64::max);
        let allowed: Vec<f64> = weights
            .iter()
            .zip(&scores)
            .filter(|(_, l)| **l >= best - 2.0)
            .map(|(w, _)| *w)
            .collect();
        let (low, high) = (allowed[0], allowed[allowed.len() - 1]);
        let at_best = weights[scores.iter().position(|l| *l == best).unwrap()];
        let mut first = format!("**{title}**");
        for x in pieces {
            let o = opinion(&m, &p(x));
            let (from, to) = interval(o, 0.2);
            let end = |w: f64| {
                let o = opinion(&under(&m, w), &p(x));
                (projected(o, w), word(o, w))
            };
            let (a, b) = (end(low), end(high));
            let flips = if a.1 != b.1 { " **flips**" } else { "" };
            println!(
                "| {first} | `{x}` | {:.2}, from {from:.2} to {to:.2} | {low:.2} to {high:.2}, best {at_best:.2} | {:.2} and {:.2} | {} and {}{flips} |",
                projected(o, 0.2),
                a.0,
                b.0,
                a.1,
                b.1
            );
            first = String::new();
        }
    }
}

fn main() {
    println!("Each cell: fixed weight 0.2 / scaled weight.\n");
    println!("### 1. One throw (one piece of evidence: W = 0.34)\n");
    println!("| senses | I saw | tie and statement | `throw` |");
    println!("|---|---|---|---|");
    for (senses, fell) in [
        (1.0, true),
        (0.9, true),
        (0.5, true),
        (1.0, false),
        (0.9, false),
    ] {
        let m = one_throw(senses, fell);
        let saw = if fell { "fall" } else { "no fall" };
        println!(
            "| {senses} | {saw} | {} | {} |",
            both(&m, "applies"),
            both(&m, "throw")
        );
    }

    println!("\n### 2. Objects that pass and one that never does\n");
    println!("| objects that pass | measurements each | W scaled | `gravity` | mass of one that passes | mass of the one that fails |");
    println!("|---|---|---|---|---|---|");
    for (n, k) in [
        (1, 1),
        (10, 1),
        (100, 1),
        (1, 10),
        (10, 10),
        (100, 10),
        (100, 100),
    ] {
        let m = sweep(&objects(n, 1, k));
        println!(
            "| {n} | {k} | {:.2} | {} | {} | {} |",
            scaled(&m),
            both(&m, "gravity"),
            both(&m, "mass o0"),
            both(&m, "mass f0")
        );
    }

    println!("\n### 3. Noisy objects, 20 measurements each\n");
    let mut noisy: Vec<_> = (0..16).map(|i| (format!("o{i}"), 19, 1)).collect();
    noisy.extend((0..3).map(|i| (format!("odd{i}"), 12, 8)));
    noisy.push(("feather".to_string(), 0, 20));
    let m = sweep(&noisy);
    println!("W scaled: {:.2}\n", scaled(&m));
    println!("| piece | reads at |");
    println!("|---|---|");
    let rows = [
        ("mass of one that passes 19 of 20", "mass o0"),
        ("its failing measurement", "drop o0 19"),
        ("mass of one that passes 12 of 20", "mass odd0"),
        ("one of its failing measurements", "drop odd0 19"),
        ("mass of the feather, 0 of 20", "mass feather"),
        ("one of the feather's measurements", "drop feather 0"),
    ];
    for (title, x) in rows {
        println!("| {title} | {} |", both(&m, x));
    }

    println!("\n### 4. How many of 10 objects fail (10 measurements each: W = 1.12)\n");
    println!("| objects that fail | `gravity` | mass of one that fails |");
    println!("|---|---|---|");
    for fail in [1, 5, 8, 9, 10] {
        let m = sweep(&objects(10 - fail, fail, 10));
        println!(
            "| {fail} | {} | {} |",
            both(&m, "gravity"),
            both(&m, "mass f0")
        );
    }
    passes();
    bounds();
}
