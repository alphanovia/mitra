use mitra_core::model::*;
fn p(n: &str) -> Piece {
    Piece::named(n)
}
fn says(name: &str, about: &str, y: bool, by: &str) -> Evidence {
    Evidence {
        piece: p(name),
        about: p(about),
        value: if y { yes() } else { no() },
        by: Voice::named(by),
    }
}
fn rests(ps: &[&str]) -> Statement<Related> {
    Statement {
        piece: p(&format!("all: {}", ps[0])),
        content: Related {
            how: Relation::All,
            pieces: ps.iter().map(|x| p(x)).collect(),
        },
    }
}
fn sure(mut m: Model) -> Model {
    for s in &m.related {
        if !m.base.iter().any(|(q, _)| q == &s.piece) {
            m.base.push((s.piece.clone(), 1.0));
        }
    }
    m
}
fn show(m: &Model, ns: &[&str]) {
    for n in ns {
        let o = opinion(m, &p(n));
        print!("{n}: ({:.3}, {:.3}) P{:.3} | ", o.r, o.s, projected(o));
    }
    println!(
        "\n  conflicts {:?}",
        conflicts(m)
            .iter()
            .map(|g| g.iter().map(|x| x.name.clone()).collect::<Vec<_>>())
            .collect::<Vec<_>>()
    );
}
fn main() {
    for fell in [true, false] {
        for (a, f, s) in [
            (0.5, 0.5, 0.5),
            (0.5, 0.5, 0.9),
            (0.5, 0.5, 1.0),
            (0.9, 0.9, 0.5),
            (0.9, 0.5, 0.6),
            (1.0, 1.0, 1.0),
            (0.0, 0.0, 1.0),
            (1.0, 1.0, 0.0),
            (0.0, 0.0, 0.6),
            (0.0, 0.0, 0.0),
        ] {
            println!("single {a} {f} {s} fell={fell}");
            let m = sure(Model {
                evidence: vec![says("throw", "lands", fell, "me")],
                related: vec![rests(&["lands", "applies", "falls"])],
                base: vec![(p("applies"), a), (p("falls"), f), (p("throw"), s)],
            });
            show(&m, &["lands", "applies", "falls", "throw"]);
        }
    }
    for one in [true, false] {
        for n in [1, 2, 4, 6, 10, 20, 40] {
            let mut m = sure(Model {
                related: vec![rests(&["lands", "applies", "falls"])],
                base: vec![(p("applies"), 0.5), (p("falls"), 0.5)],
                ..Model::default()
            });
            for k in 0..n {
                let by = if one {
                    "me".to_string()
                } else {
                    format!("f{k}")
                };
                m.evidence.push(says(&format!("t{k}"), "lands", true, &by));
                m.base.push((p(&format!("t{k}")), 0.6));
            }
            print!("repeat n={n} one={one} ");
            show(&m, &["lands", "applies"]);
        }
    }
    for (a, s) in [(0.0, 1.0), (0.0, 0.9), (0.5, 0.9), (0.5, 1.0)] {
        println!("balloon {a} {s}");
        let m = sure(Model {
            evidence: vec![
                says("throw ball", "ball lands", true, "me"),
                says("throw balloon", "balloon lands", false, "me"),
            ],
            related: vec![
                rests(&["ball lands", "ball applies", "falls"]),
                rests(&["balloon lands", "balloon applies", "falls"]),
            ],
            base: [
                ("ball applies", a),
                ("balloon applies", a),
                ("falls", a),
                ("throw ball", s),
                ("throw balloon", s),
            ]
            .iter()
            .map(|(x, a)| (p(x), *a))
            .collect(),
        });
        show(
            &m,
            &[
                "ball lands",
                "balloon lands",
                "falls",
                "ball applies",
                "balloon applies",
                "throw ball",
                "throw balloon",
            ],
        );
        println!("noise {a} {s}");
        let m = sure(Model {
            evidence: vec![
                says("throw", "lands", true, "me"),
                says("listen", "bangs", false, "me"),
            ],
            related: vec![
                rests(&["lands", "applies", "falls"]),
                rests(&["bangs", "applies", "noise"]),
            ],
            base: [
                ("applies", a),
                ("falls", a),
                ("noise", a),
                ("throw", s),
                ("listen", s),
            ]
            .iter()
            .map(|(x, a)| (p(x), *a))
            .collect(),
        });
        show(
            &m,
            &[
                "lands", "bangs", "applies", "falls", "noise", "throw", "listen",
            ],
        );
    }
    for (g, s, ball, balloon) in [
        (0.5, 0.9, true, true),
        (0.5, 0.9, true, false),
        (0.5, 1.0, true, false),
        (0.5, 0.9, false, false),
        (0.9, 0.9, true, false),
        (0.5, 0.9, true, true),
    ] {
        println!("gravity g={g} s={s} ball={ball} balloon={balloon}");
        let m = sure(Model {
            evidence: vec![
                says("throw ball", "falls ball", ball, "me"),
                says("throw balloon", "falls balloon", balloon, "me"),
            ],
            related: vec![
                rests(&["falls ball", "gravity", "mass ball"]),
                rests(&["falls balloon", "gravity", "mass balloon"]),
            ],
            base: [
                ("gravity", g),
                ("mass ball", 0.5),
                ("mass balloon", 0.5),
                ("throw ball", s),
                ("throw balloon", s),
            ]
            .iter()
            .map(|(x, a)| (p(x), *a))
            .collect(),
        });
        show(
            &m,
            &[
                "gravity",
                "mass ball",
                "mass balloon",
                "falls ball",
                "falls balloon",
                "throw ball",
                "throw balloon",
            ],
        );
    }
    {
        println!("gravity one object, falls / not");
        for fell in [true, false] {
            let m = sure(Model {
                evidence: vec![says("throw ball", "falls ball", fell, "me")],
                related: vec![rests(&["falls ball", "gravity", "mass ball"])],
                base: [("gravity", 0.5), ("mass ball", 0.5), ("throw ball", 0.9)]
                    .iter()
                    .map(|(x, a)| (p(x), *a))
                    .collect(),
            });
            show(&m, &["gravity", "mass ball", "falls ball", "throw ball"]);
        }
    }
}
