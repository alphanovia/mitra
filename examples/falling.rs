//! The foundation test model: the speed of a fall, from a value to a
//! verdict. Evaluate, take, judge. The same toy as `tests/falling.rs`.

use mitra_core::bounds::{between, Bounds};
use mitra_core::claim::*;
use mitra_core::model::*;
use mitra_core::standard::{less, mul, ratio, real, within, Exact, Table};

type Value = mitra_core::value::Value<Exact>;
use mitra_core::value::Expression::{Apply, Given, Input};

type Expression = mitra_core::value::Expression<Exact>;

const S: Settings = Settings {
    weight: 0.2,
    bias: 0.5,
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
fn fallen() -> Expression {
    times(times(Given(r(2, 1)), Given(r(981, 100))), input("h"))
}

/// From `low` to `high`, in whole metres or metres per second.
fn metres(low: i64, high: i64) -> mitra_core::value::Value<Bounds> {
    between(&real(), (low, 1), (high, 1)).unwrap()
}
/// What falls: a part of the model.
fn part(object: &str) -> Part {
    Part::subject(object)
}
/// Where it was dropped, and that this is the part the model means.
fn tie(object: &str) -> Tie {
    Tie {
        piece: leaf(&format!("{object}::is the one dropped"), &part(object)),
        location: Location::at(&["lab", "tower", object]),
    }
}
/// `speed`: from 0 to 100 m and 0 to 100 m/s, v² = 2·g·h within
/// 2 m²/s². g is 9.81 m/s².
fn speed(object: &str) -> Claim {
    let seen = times(input("v"), input("v"));
    Claim {
        piece: leaf(&format!("{object}::speed"), &part(object)),
        domain: vec![
            ("h".to_string(), metres(0, 100)),
            ("v".to_string(), metres(0, 100)),
        ],
        expression: Apply(within(&real()), vec![seen, fallen(), Given(r(2, 1))]),
    }
}
/// `slow`: from 20 m or less, 2·g·h stays under 400.
fn slow(object: &str) -> Claim {
    Claim {
        piece: leaf(&format!("{object}::slow"), &part(object)),
        domain: vec![("h".to_string(), metres(0, 20))],
        expression: Apply(less(&real()), vec![fallen(), Given(r(400, 1))]),
    }
}
/// The hypothesis of the architect: nothing but the height matters.
fn only_height(object: &str) -> Piece {
    leaf(&format!("{object}::only the height matters"), &part(object))
}
fn piece(name: &str, object: &str, pre: &[Piece], post: &[Piece]) -> Piece {
    Piece {
        name: format!("{object}::{name}"),
        pre: pre.to_vec(),
        subject: part(object),
        post: post.to_vec(),
    }
}
/// The claim of the model: given that only the height matters, the
/// object falls at `speed`.
fn falls(object: &str) -> Piece {
    let then = [speed(object).piece];
    piece("falls", object, &[only_height(object)], &then)
}
/// What a reviewer says: these measured falls represent every fall.
fn represents(object: &str, measured: &[(i64, i64)]) -> Piece {
    let each = measured
        .iter()
        .map(|(h, v)| instance(&speed(object), &at(*h, *v)));
    let seen: Vec<Piece> = each.collect();
    let all = [speed(object).piece];
    piece("represents", object, &seen, &all)
}
/// A second claim, resting on the first: given `speed`, `slow`.
fn stays_slow(object: &str) -> Piece {
    let (given, then) = ([speed(object).piece], [slow(object).piece]);
    piece("stays slow", object, &given, &then)
}

/// A height and a speed, in hundredths.
fn at(h: i64, v: i64) -> [(String, Value); 2] {
    [("h".to_string(), r(h, 100)), ("v".to_string(), r(v, 100))]
}
fn take(m: &mut Model, x: Piece, by: &str) {
    m.taken.push(Taking {
        piece: x,
        value: true,
        count: 1.0,
        by: Voice::named(by),
    });
}
/// The model of one object after these measurements, each by its own
/// timer. With `reviewed`, the architect takes the hypothesis and a
/// reviewer takes that the measured falls represent all.
fn after(object: &str, measured: &[(i64, i64)], reviewed: bool) -> Model {
    let mut m = Model::empty(S);
    m.pieces = vec![falls(object), stays_slow(object)];
    // `slow` follows from `speed` by arithmetic: a deduction.
    m.bias.push((stays_slow(object), 1.0));
    for (h, v) in measured {
        let by = Voice::named(&format!("machine:timer {h}"));
        m = observe(
            &m,
            &speed(object),
            &at(*h, *v),
            &Table::default(),
            Some(&tie(object)),
            1.0,
            &by,
        )
        .unwrap();
    }
    take(&mut m, tie(object).piece, "agent:modeller");
    if reviewed {
        take(&mut m, only_height(object), "agent:architect");
        take(&mut m, represents(object, measured), "agent:reviewer");
    }
    m
}

fn word(state: State) -> String {
    match state {
        State::Unknown => "-".to_string(),
        other => format!("{other:?}").to_lowercase(),
    }
}

fn print(title: &str, m: &Model) {
    let reading = judge(m);
    let sure = match least_sure(m) {
        Some((x, n)) => format!(" {n:.2}, least sure: `{}`", x.name),
        None => String::new(),
    };
    println!("\n### {title}\n");
    println!(
        "Verdict: {}{sure}\n",
        word(verdict(m)).replace('-', "unknown")
    );
    println!("| piece | step | evaluated | taken | judged | interval |");
    println!("|---|---|---|---|---|---|");
    for (x, o) in &reading.pieces {
        let step = match step(m, x) {
            None => "leaf".to_string(),
            Some(how) => format!("{how:?}").to_lowercase(),
        };
        println!(
            "| `{}` | {step} | {} | {} | {} | {:.2} to {:.2} |",
            x.name,
            word(o.evaluated()),
            word(o.taken()),
            shown(o),
            o.interval().0,
            o.interval().1
        );
    }
    for group in &reading.suspects {
        let names: Vec<String> = group.iter().map(|x| format!("`{}`", x.name)).collect();
        println!("\nCannot all be true: {}", names.join(", "));
    }
}

fn main() {
    let stone = [(500, 990), (1000, 1400), (2000, 1980)];
    let feather = [(500, 200), (1000, 210)];
    print(
        "The stone, three measurements, nothing reviewed",
        &after("stone", &stone, false),
    );
    print("The stone, reviewed", &after("stone", &stone, true));
    let mut bad = after("stone", &stone, true);
    let by = Voice::named("machine:timer 3000");
    bad = observe(
        &bad,
        &speed("stone"),
        &at(3000, 990),
        &Table::default(),
        Some(&tie("stone")),
        1.0,
        &by,
    )
    .unwrap();
    print("The stone, reviewed, then one bad measurement", &bad);
    let mut light = after("feather", &feather, false);
    print("The feather, hypothesis not taken", &light);
    take(&mut light, only_height("feather"), "agent:architect");
    print("The feather, hypothesis taken", &light);
}
