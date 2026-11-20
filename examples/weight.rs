//! A weight that grows with the evidence in the model, tried on the
//! two foundation examples beside the fixed weight.
//!
//! ```text
//! W = 0.2 + 0.2 · ln(1 + pieces of evidence in the model)
//! ```
//!
//! `cargo run --release --example weight`

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
fn scaled(m: &Model) -> f64 {
    0.2 + 0.2 * (1.0 + m.evidence.len() as f64).ln()
}
/// What `x` reads at under the fixed weight, and under the scaled one.
fn both(m: &Model, x: &str) -> String {
    let at = |w: f64| with_weight(w, || projected(opinion(m, &p(x))));
    format!("{:.2} / {:.2}", at(0.2), at(scaled(m)))
}

fn one_throw(senses: f64, fell: bool) -> Model {
    let mut m = Model {
        base: vec![(p("applies"), 0.5), (p("falls"), 0.5)],
        ..Model::default()
    };
    whole(&mut m, &["lands", "applies", "falls"]);
    says(&mut m, "throw", "lands", fell, senses);
    m
}

/// Objects: (name, measurements that pass, that fail).
fn sweep(objects: &[(String, usize, usize)]) -> Model {
    let mut m = Model::default();
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

/// ln of the gamma function (Lanczos).
fn ln_gamma(x: f64) -> f64 {
    const G: [f64; 9] = [
        0.999_999_999_999_809_9,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_1,
        -176.615_029_162_140_6,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_572e-6,
        1.505_632_735_149_311_6e-7,
    ];
    if x < 0.5 {
        let pi = std::f64::consts::PI;
        return (pi / (pi * x).sin()).ln() - ln_gamma(1.0 - x);
    }
    let x = x - 1.0;
    let t = x + 7.5;
    let mut sum = G[0];
    for (k, g) in G.iter().enumerate().skip(1) {
        sum += g / (x + k as f64);
    }
    0.5 * (2.0 * std::f64::consts::PI).ln() + (x + 0.5) * t.ln() - t + sum.ln()
}
fn ln_beta(a: f64, b: f64) -> f64 {
    ln_gamma(a) + ln_gamma(b) - ln_gamma(a + b)
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
    let likelihood = |w: f64| {
        with_weight(w, || {
            let mut sum = 0.0;
            for x in doubted(m) {
                let o = opinion(m, &x);
                let (yes_, no_) = (o.a * w, (1.0 - o.a) * w);
                sum += ln_beta(o.r + yes_, o.s + no_) - ln_beta(yes_, no_);
            }
            sum
        })
    };
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
    with_weight(w, || {
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
                let (with, bare) = (projected(opinion(m, x)), projected(opinion(&without, x)));
                moved_by = moved_by.max((with - bare).abs());
            }
            least = least.min(moved_by);
        }
        least
    })
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
            let at = |w: f64| with_weight(w, || projected(opinion(&m, &p(x))));
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
}
