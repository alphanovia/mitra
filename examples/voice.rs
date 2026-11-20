//! Voices, tried outside the core (`docs/trials.md`).
//!
//! A voice has a reliability, the parent of all it says: if the voice
//! is not reliable, its word is nothing. Voices are independent unless
//! they rest on a shared source. Each is judged by every whole it
//! spoke for, passes and failures together, and its word is then
//! discounted before the core reads the model. Nothing here changes
//! the core: it is a pass over the model, then `opinion`.
//!
//! `cargo run --release --example voice`

use mitra_core::model::*;

mod common;
use common::interval;

const W: f64 = 0.2;

fn p(name: &str) -> Piece {
    Piece::named(name)
}

/// Who speaks: voices, each reliable with some chance before anything
/// is seen, and sources that voices rest on. The word of a voice holds
/// when the voice is reliable and its source too.
#[derive(Default)]
struct Speakers {
    voices: Vec<(String, f64)>,
    /// A source, its chance, and the voices that rest on it.
    sources: Vec<(String, f64, Vec<String>)>,
}

/// What was judged: how far the word of each voice holds, and how far
/// each source is believed.
struct Verdict {
    holds: Vec<f64>,
    sources: Vec<f64>,
}

/// What a word is worth when the one who says it is reliable with
/// chance `t`: the mix of the word and of nothing, as counts. A belief
/// never exceeds the trust in who says it.
fn discounted(t: f64, o: Opinion) -> Opinion {
    let n = o.r + o.s + W;
    let (b, d) = (t * o.r / n, t * o.s / n);
    let u = 1.0 - b - d;
    Opinion {
        r: W * b / u,
        s: W * d / u,
        a: o.a,
    }
}

/// The model with each voice's word discounted by how reliable it is.
fn heard_at(m: &Model, reliable: &dyn Fn(&str) -> Option<f64>) -> Model {
    let mut out = m.clone();
    out.evidence.clear();
    for e in &m.evidence {
        match reliable(&e.by.name) {
            None => out.evidence.push(e.clone()),
            Some(t) if t > 0.0 => out.evidence.push(Evidence {
                value: discounted(t, e.value),
                ..e.clone()
            }),
            Some(_) => {}
        }
    }
    out
}

/// Judge each voice by the wholes of the model. For one whole, with
/// `p` the chance that its parts are all right before anything is seen
/// of it:
///
/// ```text
/// L = p · (every failing measurement is wrong) + (1 - p) · (every passing one is wrong)
/// ```
///
/// read once with the word of the voice taken whole, once without it.
/// The ratio of the two, over all the wholes, is what the model says
/// for the voice. A voice alone moves its own odds by it. Voices on one
/// source are judged together: the source is believed as far as its
/// voices are not all contradicted, and each word holds as far as the
/// source and the voice do. Three rounds, each reading the other
/// voices as judged so far.
fn judge(m: &Model, who: &Speakers) -> Verdict {
    let source_of = |voice: &str| {
        who.sources
            .iter()
            .position(|s| s.2.iter().any(|v| v == voice))
    };
    let mut verdict = Verdict {
        holds: Vec::new(),
        sources: who.sources.iter().map(|s| s.1).collect(),
    };
    for (name, prior) in &who.voices {
        verdict
            .holds
            .push(prior * source_of(name).map_or(1.0, |k| who.sources[k].1));
    }
    let known = |voice: &str| who.voices.iter().position(|v| v.0 == voice);
    for _ in 0..3 {
        // What the model says for each voice.
        let mut ratio = vec![1.0; who.voices.len()];
        for (k, said) in ratio.iter_mut().enumerate() {
            for relation in &m.related {
                let whole = &relation.content.pieces[0];
                let seen: Vec<&Evidence> =
                    m.evidence.iter().filter(|e| &e.about == whole).collect();
                if seen.is_empty() {
                    continue;
                }
                // Before anything is seen of this whole, and before any no.
                let mut calm = m.clone();
                calm.evidence
                    .retain(|e| &e.about != whole && e.value.s == 0.0);
                let (mut passes_wrong, mut failures_wrong) = (1.0, 1.0);
                for e in &seen {
                    let right = projected(opinion(&calm, &e.piece), W);
                    match e.value.s > 0.0 {
                        true => failures_wrong *= 1.0 - right,
                        false => passes_wrong *= 1.0 - right,
                    }
                }
                let likely = |mine: f64| {
                    let world = heard_at(&calm, &|voice| {
                        let at = known(voice)?;
                        Some(if at == k { mine } else { verdict.holds[at] })
                    });
                    let mut all_right = 1.0;
                    for part in parts(&world, &relation.content) {
                        all_right *= projected(opinion(&world, &part), W);
                    }
                    all_right * failures_wrong + (1.0 - all_right) * passes_wrong
                };
                let (with, without) = (likely(1.0), likely(0.0));
                if with > 0.0 && without > 0.0 {
                    *said *= with / without;
                }
            }
        }
        // A voice with ratio L, reliable with chance a, is as likely as
        // a·L + 1 - a against 1.
        let weighed = |k: usize| who.voices[k].1 * ratio[k] + 1.0 - who.voices[k].1;
        for (k, (_, prior, voices)) in who.sources.iter().enumerate() {
            let mut for_it = *prior;
            for v in voices {
                for_it *= known(v).map_or(1.0, weighed);
            }
            verdict.sources[k] = for_it / (for_it + 1.0 - prior);
        }
        for (k, (name, prior)) in who.voices.iter().enumerate() {
            let own = prior * ratio[k] / weighed(k);
            verdict.holds[k] = own * source_of(name).map_or(1.0, |s| verdict.sources[s]);
        }
    }
    verdict
}

/// The model as the core reads it once the voices are judged.
fn settled(m: &Model, who: &Speakers) -> (Model, Verdict) {
    let verdict = judge(m, who);
    let known = |voice: &str| who.voices.iter().position(|v| v.0 == voice);
    let out = heard_at(m, &|voice| Some(verdict.holds[known(voice)?]));
    (out, verdict)
}

/// Objects under gravity, each with its mass and its completeness, so
/// many measurements that pass and that fail, each believed at 0.9.
fn objects(list: &[(&str, usize, usize)]) -> Model {
    let mut m = Model::empty(W);
    m.base.push((p("gravity"), 0.5));
    for (name, pass, fail) in list {
        let (mass, done, speed) = (
            format!("mass {name}"),
            format!("complete {name}"),
            format!("same speed {name}"),
        );
        m.base.push((p(&mass), 0.5));
        m.base.push((p(&done), 0.5));
        let relation = p(&format!("all: {speed}"));
        m.related.push(Statement {
            piece: relation.clone(),
            content: Related {
                how: Relation::All,
                pieces: vec![p(&speed), p("gravity"), p(&mass), p(&done)],
            },
        });
        m.base.push((relation, 1.0));
        for j in 0..pass + fail {
            let drop = format!("drop {name} {j}");
            m.base.push((p(&drop), 0.9));
            m.evidence.push(Evidence {
                piece: p(&drop),
                about: p(&speed),
                value: if j < *pass { yes() } else { no() },
                by: Voice::named(&drop),
            });
        }
    }
    m
}

/// A voice reviews: so many yes about each piece. Each review is sure
/// of itself; what can be doubted is the voice.
fn reviews(m: &mut Model, voice: &str, said: &[(&str, f64)]) {
    for (about, count) in said {
        let piece = p(&format!("review by {voice}: {about}"));
        m.base.push((piece.clone(), 1.0));
        m.evidence.push(Evidence {
            piece,
            about: p(about),
            value: Opinion {
                r: *count,
                s: 0.0,
                a: 0.5,
            },
            by: Voice::named(voice),
        });
    }
}

fn cell(m: &Model, x: &str) -> String {
    let o = opinion(m, &p(x));
    let (from, to) = interval(o, W);
    format!("{:.2} [{from:.2}, {to:.2}]", projected(o, W))
}

const NAMES: [&str; 4] = ["stone", "ball", "leaf", "feather"];

fn cautious(unsure: f64) -> Vec<(String, f64)> {
    let mut said = vec![("gravity".to_string(), 1000.0)];
    said.extend(NAMES.map(|x| (format!("mass {x}"), 1.0)));
    said.push(("complete stone".to_string(), 20.0));
    said.push(("complete ball".to_string(), 5.0));
    if unsure > 0.0 {
        said.push(("complete leaf".to_string(), unsure));
        said.push(("complete feather".to_string(), unsure));
    }
    said
}
fn overconfident() -> Vec<(String, f64)> {
    let mut said = vec![("gravity".to_string(), 100.0)];
    said.extend(NAMES.map(|x| (format!("mass {x}"), 100.0)));
    said.extend(NAMES.map(|x| (format!("complete {x}"), 100.0)));
    said
}
fn borrowed(said: &[(String, f64)]) -> Vec<(&str, f64)> {
    said.iter().map(|(x, c)| (x.as_str(), *c)).collect()
}

fn two(list: &[(&str, usize, usize)], who: &str, unsure: f64) -> (Model, Speakers) {
    let mut m = objects(list);
    let mut judged = Speakers::default();
    if who != "overconfident" {
        reviews(&mut m, "cautious", &borrowed(&cautious(unsure)));
        judged.voices.push(("cautious".to_string(), 0.9));
    }
    if who != "cautious" {
        reviews(&mut m, "overconfident", &borrowed(&overconfident()));
        judged.voices.push(("overconfident".to_string(), 0.9));
    }
    (m, judged)
}

fn row(m: &Model, who: &Speakers, names: &[&str], pieces: &[&str]) -> String {
    let (read, verdict) = settled(m, who);
    let mut cells = Vec::new();
    for name in names {
        let found = who.voices.iter().position(|v| &v.0 == name);
        cells.push(found.map_or(String::new(), |k| format!("{:.2}", verdict.holds[k])));
    }
    cells.extend(pieces.iter().map(|x| cell(&read, x)));
    cells.join(" | ")
}

fn main() {
    println!("### 1. A cautious voice and an overconfident one\n");
    println!("Both start at 0.9. The cautious one says 1000 yes on gravity, 1 on each mass, and on completeness 20 for the stone, 5 for the ball, half a yes for the leaf and the feather. The overconfident one says 100 yes on everything. The stone, the ball and the leaf pass every time, the feather never.\n");
    let voices = ["cautious", "overconfident"];
    let pieces = [
        "complete feather",
        "mass feather",
        "complete stone",
        "drop feather 0",
        "gravity",
    ];
    let head = "cautious voice | overconfident voice | `complete feather` | `mass feather` | `complete stone` | one of the feather's measurements | `gravity` |";
    println!("| measurements per object | {head}");
    println!("|---|---|---|---|---|---|---|---|");
    for k in [1, 3, 10, 30, 100] {
        let list = [
            ("stone", k, 0),
            ("ball", k, 0),
            ("leaf", k, 0),
            ("feather", 0, k),
        ];
        let (m, judged) = two(&list, "both", 0.5);
        println!("| {k} | {} |", row(&m, &judged, &voices, &pieces));
    }

    println!("\n### 2. By who reviews, 10 measurements per object\n");
    println!("| who reviews | {head}");
    println!("|---|---|---|---|---|---|---|---|");
    let list = [
        ("stone", 10, 0),
        ("ball", 10, 0),
        ("leaf", 10, 0),
        ("feather", 0, 10),
    ];
    for (who, unsure, title) in [
        ("cautious", 0.5, "cautious"),
        (
            "cautious",
            0.0,
            "cautious, silent on the leaf's and the feather's completeness",
        ),
        ("overconfident", 0.5, "overconfident"),
        ("both", 0.5, "both"),
    ] {
        let (m, judged) = two(&list, who, unsure);
        println!("| {title} | {} |", row(&m, &judged, &voices, &pieces));
    }

    println!("\n### 3. Noisy objects, both voices, 10 measurements each\n");
    let list = [
        ("stone", 9, 1),
        ("ball", 10, 0),
        ("leaf", 6, 4),
        ("feather", 0, 10),
    ];
    let (m, judged) = two(&list, "both", 0.5);
    let (read, at) = settled(&m, &judged);
    println!("| object | passes | its mass | its completeness | one failing measurement |");
    println!("|---|---|---|---|---|");
    for (name, pass, fail) in list {
        let failing = match fail {
            0 => "none".to_string(),
            _ => cell(&read, &format!("drop {name} {}", pass + fail - 1)),
        };
        println!(
            "| {name} | {pass} of {} | {} | {} | {failing} |",
            pass + fail,
            cell(&read, &format!("mass {name}")),
            cell(&read, &format!("complete {name}"))
        );
    }
    println!(
        "\ncautious voice {:.2}, overconfident voice {:.2}, `gravity` {}",
        at.holds[0],
        at.holds[1],
        cell(&read, "gravity")
    );

    println!("\n### 4. Two agents, independent or sharing a source\n");
    println!("Agent A says 100 yes on gravity and on the mass and the completeness of the ball and of the feather. Agent B says 100 yes on the mass and the completeness of the stone, and of nothing else. The ball passes 10 times, the feather fails 10 times, the stone is never measured. Each agent starts at 0.9, and so does the source when they share one: a word then starts at 0.81.\n");
    println!("| declared | A's word holds at | B's word holds at | the source | `complete stone` | `complete feather` | one of the feather's measurements |");
    println!("|---|---|---|---|---|---|---|");
    for shared in [false, true] {
        let mut m = objects(&[("stone", 0, 0), ("ball", 10, 0), ("feather", 0, 10)]);
        let a = [
            ("gravity", 100.0),
            ("mass ball", 100.0),
            ("complete ball", 100.0),
            ("mass feather", 100.0),
            ("complete feather", 100.0),
        ];
        reviews(&mut m, "A", &a);
        reviews(
            &mut m,
            "B",
            &[("mass stone", 100.0), ("complete stone", 100.0)],
        );
        let mut who = Speakers::default();
        who.voices.push(("A".to_string(), 0.9));
        who.voices.push(("B".to_string(), 0.9));
        if shared {
            let on = vec!["A".to_string(), "B".to_string()];
            who.sources.push(("source".to_string(), 0.9, on));
        }
        let (read, at) = settled(&m, &who);
        let source = at
            .sources
            .first()
            .map_or(String::new(), |s| format!("{s:.2}"));
        println!(
            "| {} | {:.2} | {:.2} | {source} | {} | {} | {} |",
            if shared { "one source" } else { "independent" },
            at.holds[0],
            at.holds[1],
            cell(&read, "complete stone"),
            cell(&read, "complete feather"),
            cell(&read, "drop feather 0")
        );
    }
}
