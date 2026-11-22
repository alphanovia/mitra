//! The foundation test model: the speed of a fall, from a value to a
//! verdict. Evaluate, take, judge. The same toy as `tests/falling.rs`.

use mitra_core::atom::*;
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
/// What falls: a part of the model, with one function, `fall`: from a
/// height `h`, a speed `v`. It is only declared: nature does it.
fn part(object: &str) -> Part {
    Part {
        name: object.to_string(),
        kind: Kind::Subject,
        records: Vec::new(),
        functions: vec![Function {
            name: fall(object),
            inputs: vec![("h".to_string(), real())],
            output: ("v".to_string(), real()),
            body: None,
        }],
    }
}
fn fall(object: &str) -> String {
    format!("{object}::fall")
}
/// Where it is dropped.
fn tower(object: &str) -> Location {
    Location::at(&["lab", "tower", object])
}
/// The tie: `fall` is the drop in the tower, and the model says all
/// that matters of it there.
fn tie(object: &str) -> Piece<Tie> {
    let about = Tie {
        of: fall(object),
        location: tower(object),
    };
    leaf(&format!("{object}::fall is the drop in the tower"), about)
}
/// `speed`: from 0 to 100 m and 0 to 100 m/s, v² = 2·g·h within
/// 2 m²/s². g is 9.81 m/s².
fn speed(object: &str) -> Piece<Claim> {
    let seen = times(input("v"), input("v"));
    let about = Claim {
        of: fall(object),
        domain: vec![
            ("h".to_string(), metres(0, 100)),
            ("v".to_string(), metres(0, 100)),
        ],
        expression: Apply(within(&real()), vec![seen, fallen(), Given(r(2, 1))]),
    };
    leaf(&format!("{object}::speed"), about)
}
/// `slow`: from 20 m or less, 2·g·h stays under 400.
fn slow(object: &str) -> Piece<Claim> {
    let about = Claim {
        of: fall(object),
        domain: vec![("h".to_string(), metres(0, 20))],
        expression: Apply(less(&real()), vec![fallen(), Given(r(400, 1))]),
    };
    leaf(&format!("{object}::slow"), about)
}
/// What the part needs, said by the architect: nothing but the height
/// matters.
fn only_height(object: &str) -> Piece<Require> {
    let about = Require {
        part: object.to_string(),
        text: "nothing but the height matters".to_string(),
    };
    leaf(&format!("{object}::only the height matters"), about)
}
fn ensure(
    name: &str,
    object: &str,
    text: &str,
    pre: Vec<SomePiece>,
    post: Vec<SomePiece>,
) -> Piece<Ensure> {
    Piece {
        name: format!("{object}::{name}"),
        pre,
        about: Ensure {
            part: object.to_string(),
            text: text.to_string(),
        },
        post,
    }
}
/// What the part promises: given that only the height matters, it
/// falls at `speed`.
fn falls(object: &str) -> Piece<Ensure> {
    let (given, then) = (
        vec![erased(&only_height(object))],
        vec![erased(&speed(object))],
    );
    ensure("falls", object, "it falls at `speed`", given, then)
}
/// A height and a speed measured in the tower, in hundredths.
fn at(object: &str, h: i64, v: i64) -> Measurement<Exact> {
    Measurement {
        function: fall(object),
        at: tower(object),
        values: vec![("h".to_string(), r(h, 100)), ("v".to_string(), r(v, 100))],
    }
}
fn timer(h: i64) -> Voice {
    Voice::named(&format!("machine:timer {h}"))
}
/// The piece of evidence of `speed` on one measurement.
fn seen(object: &str, h: i64, v: i64) -> SomePiece {
    erased(&evidence_of(&speed(object), &at(object, h, v), &timer(h)))
}
/// What a reviewer says: these measured falls represent every fall.
fn represents(object: &str, measured: &[(i64, i64)]) -> Piece<Ensure> {
    let each = measured.iter().map(|(h, v)| seen(object, *h, *v));
    let text = "the measured falls represent every fall";
    ensure(
        "represents",
        object,
        text,
        each.collect(),
        vec![erased(&speed(object))],
    )
}
/// A second claim, resting on the first: given `speed`, `slow`.
fn stays_slow(object: &str) -> Piece<Ensure> {
    let (given, then) = (vec![erased(&speed(object))], vec![erased(&slow(object))]);
    ensure(
        "stays slow",
        object,
        "`slow` follows from `speed`",
        given,
        then,
    )
}
fn take<T: Keyed>(m: &mut Model, x: &Piece<T>, by: &str) {
    m.taken.push(Taking {
        piece: erased(x),
        value: true,
        count: 1.0,
        by: Voice::named(by),
        form: Form::Review,
    });
}
/// The model of one object after these measurements, each by its own
/// timer. Who tied takes the tie. With `reviewed`, the architect
/// takes what the part needs and a reviewer takes that the measured
/// falls represent all.
fn after(object: &str, measured: &[(i64, i64)], reviewed: bool) -> Model {
    let mut m = Model::empty(S);
    m.pieces = vec![erased(&falls(object)), erased(&stays_slow(object))];
    // `slow` follows from `speed` by arithmetic: a deduction.
    m.bias.push((erased(&stays_slow(object)), 1.0));
    for (h, v) in measured {
        m = measure(&m, object, *h, *v);
    }
    take(&mut m, &tie(object), "agent:modeller");
    if reviewed {
        take(&mut m, &only_height(object), "agent:architect");
        take(&mut m, &represents(object, measured), "agent:reviewer");
    }
    m
}
/// One more measurement, evaluated on `speed` and taken by its timer.
fn measure(m: &Model, object: &str, h: i64, v: i64) -> Model {
    let (table, here) = (Table::default(), tie(object));
    let run = at(object, h, v);
    observe(m, &speed(object), &run, &table, Some(&here), 1.0, &timer(h)).unwrap()
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
    println!("| piece | keyword | step | evaluated | taken | judged |");
    println!("|---|---|---|---|---|---|");
    for (x, o) in &reading.pieces {
        let step = match step(m, x) {
            None => "leaf".to_string(),
            Some(how) => format!("{how:?}").to_lowercase(),
        };
        println!(
            "| `{}` | {} | {step} | {} | {} | {} |",
            plain(&x.name),
            x.about,
            word(o.evaluated()),
            word(o.taken()),
            shown(o),
        );
    }
    for group in &reading.suspects {
        let names: Vec<String> = group
            .iter()
            .map(|x| format!("`{}`", plain(&x.name)))
            .collect();
        println!("\nSuspects: {}", names.join(", "));
    }
}

/// A name with its ratios written `a/b`, for the table only.
fn plain(name: &str) -> String {
    let mut out = String::new();
    let mut rest = name;
    while let Some(i) = rest.find("Many([Whole(") {
        out.push_str(&rest[..i]);
        let tail = &rest[i + 12..];
        let Some(j) = tail.find(")])") else { break };
        out.push_str(&tail[..j].replace("), Whole(", "/"));
        rest = &tail[j + 3..];
    }
    out + rest
}

fn main() {
    let stone = [(500, 990), (1000, 1400), (2000, 1980)];
    let feather = [(500, 200), (1000, 210)];
    let made = part("stone");
    let level = function_level(&made.functions[0], 2);
    println!(
        "`{}` is {level:?}: declared, with two claims.",
        made.functions[0].name
    );
    print(
        "The stone, three measurements, nothing reviewed",
        &after("stone", &stone, false),
    );
    print("The stone, reviewed", &after("stone", &stone, true));
    let bad = measure(&after("stone", &stone, true), "stone", 3000, 990);
    print("The stone, reviewed, then one bad measurement", &bad);
    let mut light = after("feather", &feather, false);
    print("The feather, the require not taken", &light);
    take(&mut light, &only_height("feather"), "agent:architect");
    print("The feather, the require taken", &light);
}
