//! Random valid programs, written by hand from the reference grammar.
//!
//! One function per grammar rule. Tokens are separated by spaces and
//! commands by line breaks. The generator also keeps the parser's one rule
//! beyond the grammar: in a claim or evidence, every argument name is bound
//! by a `∀` and every `∀` name is used once. Names never collide with
//! keywords. Deterministic for a given seed.

/// xorshift64: tiny, deterministic, good enough to pick alternatives.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1)
    }
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
    /// 0..n
    fn below(&mut self, n: usize) -> usize {
        (self.next() % n as u64) as usize
    }
    fn chance(&mut self, percent: usize) -> bool {
        self.below(100) < percent
    }
    /// `lo..=hi` items produced by `f`.
    fn many(&mut self, lo: usize, hi: usize, mut f: impl FnMut(&mut Rng) -> String) -> Vec<String> {
        let n = lo + self.below(hi - lo + 1);
        (0..n).map(|_| f(self)).collect()
    }
}

const NAMES: [&str; 8] = ["a", "b", "f", "g", "not", "x1", "add_two", "claim_"];

fn name(r: &mut Rng) -> String {
    NAMES[r.below(NAMES.len())].to_string()
}

fn nat(r: &mut Rng) -> String {
    match r.below(20) {
        0 => "9223372036854775807".into(),
        1 => "123456".into(),
        _ => r.below(11).to_string(),
    }
}

/// `number`: what weights and confidences use.
fn number(r: &mut Rng) -> String {
    ["0", "1", "2", "0.5", "10"][r.below(5)].to_string()
}

fn string(r: &mut Rng) -> String {
    ["\"\"", "\"a\"", "\"hello world\"", "\"-- not a comment\""][r.below(4)].into()
}

fn value(r: &mut Rng) -> String {
    match r.below(5) {
        0 => "true".into(),
        1 => "false".into(),
        2 => nat(r),
        3 => format!("- {}", nat(r)),
        _ => string(r),
    }
}

fn int_atom(r: &mut Rng) -> String {
    if r.chance(30) {
        format!("( - {} )", nat(r))
    } else {
        nat(r)
    }
}

fn set(r: &mut Rng, depth: usize) -> String {
    let mut s = inter(r, depth);
    while r.chance(20) {
        s = format!("{s} ∪ {}", inter(r, depth));
    }
    s
}

fn inter(r: &mut Rng, depth: usize) -> String {
    let mut s = set_atom(r, depth);
    while r.chance(20) {
        s = format!("{s} {} {}", ["∩", "\\"][r.below(2)], set_atom(r, depth));
    }
    s
}

fn set_atom(r: &mut Rng, depth: usize) -> String {
    match r.below(if depth == 0 { 6 } else { 7 }) {
        0..=3 => ["Bool", "Int", "String", "Value"][r.below(4)].into(),
        4 => format!("{{ {} }}", r.many(0, 3, value).join(" , ")),
        5 => format!("Icc {} {}", int_atom(r), int_atom(r)),
        _ => format!("( {} )", set(r, depth - 1)),
    }
}

/// `∀` binders and the arguments of one call: `(binders, args)`. At least
/// `min` arguments.
fn call_args(r: &mut Rng, min: usize) -> (String, String) {
    let n = min + r.below(4 - min);
    let mut fresh: Vec<&str> = vec!["x", "y", "z", "w"];
    let mut bound = Vec::new();
    let mut args = Vec::new();
    for _ in 0..n {
        if r.chance(50) {
            let x = fresh.remove(r.below(fresh.len()));
            let binder = if r.chance(50) {
                format!("∀ {x} , ")
            } else {
                format!("∀ {x} ∈ {} , ", set(r, 2))
            };
            bound.push(binder);
            args.push(x.to_string());
        } else {
            args.push(match r.below(5) {
                0 => "true".into(),
                1 => "false".into(),
                2 => nat(r),
                3 => format!("( - {} )", nat(r)),
                _ => string(r),
            });
        }
    }
    // Binders may come in any order.
    for i in (1..bound.len()).rev() {
        bound.swap(i, r.below(i + 1));
    }
    let args: String = args.iter().map(|a| format!(" {a}")).collect();
    (bound.concat(), args)
}

pub fn expr(r: &mut Rng, depth: usize) -> String {
    match if depth == 0 { 2 } else { r.below(8) } {
        0 => format!(
            "if {} then {} else {}",
            expr(r, depth - 1),
            expr(r, depth - 1),
            expr(r, depth - 1)
        ),
        1 => format!(
            "let {} := {} ; {}",
            name(r),
            expr(r, depth - 1),
            expr(r, depth - 1)
        ),
        _ => binary(r, 0, depth),
    }
}

/// `or`, `and`, `compare`, `sum`, `product`: one left-associative level each.
const LEVELS: [&[&str]; 5] = [
    &["||"],
    &["&&"],
    &["==", "!=", "<", "<=", ">", ">="],
    &["+", "-"],
    &["*"],
];

fn binary(r: &mut Rng, level: usize, depth: usize) -> String {
    let next = |r: &mut Rng| {
        if level + 1 < LEVELS.len() {
            binary(r, level + 1, depth)
        } else {
            unary(r, depth)
        }
    };
    let mut s = next(r);
    while depth > 0 && r.chance(15) {
        let ops = LEVELS[level];
        s = format!("{s} {} {}", ops[r.below(ops.len())], next(r));
    }
    s
}

fn literal(r: &mut Rng) -> String {
    match r.below(4) {
        0 => "true".into(),
        1 => "false".into(),
        2 => nat(r),
        _ => string(r),
    }
}

fn atom(r: &mut Rng, depth: usize) -> String {
    match if depth == 0 { r.below(2) } else { r.below(3) } {
        0 => name(r),
        1 => literal(r),
        _ => format!("( {} )", expr(r, depth - 1)),
    }
}

fn unary(r: &mut Rng, depth: usize) -> String {
    match if depth == 0 {
        2 + r.below(2)
    } else {
        r.below(6)
    } {
        0 => format!("! {}", unary(r, depth - 1)),
        1 => format!("- {}", unary(r, depth - 1)),
        2 => literal(r),
        3 => name(r),
        4 => format!(
            "{}{}",
            name(r),
            r.many(1, 3, |r| format!(" {}", atom(r, depth - 1)))
                .concat()
        ),
        _ => format!("( {} )", expr(r, depth - 1)),
    }
}

fn binders(r: &mut Rng) -> String {
    r.many(1, 3, |r| {
        let names = r.many(1, 3, name).join(" ");
        format!(" ( {names} : {} )", set(r, 2))
    })
    .concat()
}

fn command(r: &mut Rng) -> String {
    match r.below(6) {
        5 => format!("test {} : {}", name(r), judgment(r)),
        0 => format!("opaque {}{} : {}", name(r), binders(r), set(r, 2)),
        1 => format!(
            "def {}{} : {} := {}",
            name(r),
            binders(r),
            set(r, 2),
            expr(r, 3)
        ),
        2 => {
            let (forall, args) = call_args(r, 1);
            let body = if r.chance(50) {
                format!("correct ( {}{args} )", name(r))
            } else {
                format!("{}{args} = {}", name(r), expr(r, 3))
            };
            let derive = if r.chance(30) {
                format!(" := by derive [ {} ]", r.many(1, 3, name).join(" , "))
            } else {
                String::new()
            };
            format!("claim {} : {forall}{body}{derive}", name(r))
        }
        3 => {
            let regions = r.many(0, 3, |r| {
                if r.chance(70) {
                    format!("\n  | {} ∈ {} => {}", name(r), set(r, 2), number(r))
                } else {
                    format!("\n  | _ => {}", number(r))
                }
            });
            format!("profile {} for {}{}", name(r), name(r), regions.concat())
        }
        _ => match r.below(3) {
            0 => "policy strict".into(),
            1 => format!("policy {} := strict", name(r)),
            _ => format!("policy {} := coverage {}", name(r), name(r)),
        },
    }
}

/// A random model file.
pub fn model(r: &mut Rng) -> String {
    r.many(0, 8, |r| command(r) + &comment(r) + "\n").concat()
}

/// Sometimes a comment, at the end of a line or on its own line.
fn comment(r: &mut Rng) -> String {
    match r.below(6) {
        0 => " -- a comment, with ∀ and \"quotes\"".into(),
        1 => "\n-- a line comment".into(),
        _ => String::new(),
    }
}

/// A random evidence file. Confidence is 0 or 1, as the parser requires.
pub fn evidences(r: &mut Rng) -> String {
    r.many(0, 8, |r| {
        let attrs = match r.below(4) {
            0 => format!("@[ source {} ] ", string(r)),
            1 => format!("@[ rev {} , source {} ] ", string(r), string(r)),
            _ => String::new(),
        };
        format!(
            "{attrs}evidence {} at {}{}\n",
            judgment(r),
            r.below(2),
            comment(r)
        )
    })
    .concat()
}

/// `∀ …, <judgment>`, as in evidence and tests.
fn judgment(r: &mut Rng) -> String {
    match r.below(5) {
        0 => {
            let (forall, args) = call_args(r, 1);
            format!("{forall}correct ( {}{args} )", name(r))
        }
        1 => {
            let (forall, args) = call_args(r, 1);
            format!("{forall}¬ correct ( {}{args} )", name(r))
        }
        2 => {
            let (forall, args) = call_args(r, 0);
            format!("{forall}{}{args}", name(r))
        }
        3 => {
            let (forall, args) = call_args(r, 0);
            format!("{forall}¬ {}{args}", name(r))
        }
        _ => {
            let (forall, args) = call_args(r, 1);
            format!("{forall}{}{args} = {}", name(r), value(r))
        }
    }
}
