//! Random valid programs, written by hand from the reference grammar.
//!
//! One function per grammar rule. Tokens are separated by spaces and
//! commands by line breaks. The generator also keeps the parser's one rule
//! beyond the grammar: in a claim or evidence, every argument name is bound
//! by a `forall` and every `forall` name is used once. Names never collide with
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

const NAMES: [&str; 8] = ["a", "b", "f", "g", "flip", "x1", "add_two", "claim_"];

/// ASCII first, sometimes Lean's Unicode alias.
fn pick(r: &mut Rng, ascii: &'static str, unicode: &'static str) -> &'static str {
    if r.chance(25) {
        unicode
    } else {
        ascii
    }
}

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

/// A float literal, as values and expressions write it.
fn float(r: &mut Rng) -> String {
    ["0.0", "1.5", "0.25", "123456.789"][r.below(4)].to_string()
}

fn string(r: &mut Rng) -> String {
    [
        "\"\"",
        "\"a\"",
        "\"hello world\"",
        "\"-- not a comment\"",
        "\"a \\\"quote\\\", a line\\nbreak, a tab\\t and a \\\\\"",
    ][r.below(5)]
    .into()
}

fn value(r: &mut Rng) -> String {
    composite(r, 2)
}

/// A value, with tuples, records and lists down to `depth`.
fn composite(r: &mut Rng, depth: usize) -> String {
    if depth > 0 && r.chance(20) {
        let parts = |r: &mut Rng, lo| r.many(lo, 3, |r| composite(r, depth - 1)).join(" , ");
        return match r.below(6) {
            0 if r.chance(10) => "( )".into(),
            0 => format!("( {} )", parts(r, 2)),
            1 => format!("{} ( {} )", name(r), parts(r, 1)),
            2 => format!("[ {} ]", parts(r, 0)),
            // A set, and a map: its keys differ, or the parser refuses it.
            3 => format!("{{ {} }}", parts(r, 0)),
            4 => "{ : }".into(),
            _ => {
                let n = 1 + r.below(3);
                let pairs: Vec<String> = (0..n)
                    .map(|k| format!("{k} : {}", composite(r, depth - 1)))
                    .collect();
                format!("{{ {} }}", pairs.join(" , "))
            }
        };
    }
    // A variant without fields, inside another value.
    if depth < 2 && r.chance(10) {
        return VARIANTS[r.below(VARIANTS.len())].into();
    }
    scalar(r)
}

fn scalar(r: &mut Rng) -> String {
    match r.below(7) {
        0 => "true".into(),
        1 => "false".into(),
        2 => nat(r),
        3 => format!("- {}", nat(r)),
        4 => float(r),
        5 => format!("- {}", float(r)),
        _ => string(r),
    }
}

/// A range bound: `5` or `- 5`, sometimes past 64 bits.
fn bound(r: &mut Rng) -> String {
    if r.chance(5) {
        return "9223372036854775808".into();
    }
    if r.chance(30) {
        format!("- {}", nat(r))
    } else {
        nat(r)
    }
}

/// `a ..= b`, `a .. b`, `a ..`, `..= b`, `.. b`.
fn range(r: &mut Rng) -> String {
    match r.below(5) {
        0 | 1 => format!("{} ..= {}", bound(r), bound(r)),
        2 => format!("{} .. {}", bound(r), bound(r)),
        3 => format!("{} ..", bound(r)),
        _ => format!("{} {}", ["..=", ".."][r.below(2)], bound(r)),
    }
}

const TYPES: [&str; 12] = [
    "Bool", "Int", "Int8", "Int16", "Int32", "Int64", "UInt8", "UInt16", "UInt32", "Float64",
    "String", "Value",
];

fn set(r: &mut Rng, depth: usize) -> String {
    let mut s = inter(r, depth);
    while r.chance(20) {
        s = format!("{s} {} {}", pick(r, "|", "∪"), inter(r, depth));
    }
    s
}

fn inter(r: &mut Rng, depth: usize) -> String {
    let mut s = set_atom(r, depth);
    while r.chance(20) {
        let op = match r.below(3) {
            0 => pick(r, "&", "∩"),
            _ => "\\",
        };
        s = format!("{s} {op} {}", set_atom(r, depth));
    }
    s
}

/// Variants without fields, and the enums: each with a fixed number of
/// parameters, so that a use fits a declaration.
const VARIANTS: [&str; 3] = ["Nil", "Red", "Off"];
const ENUMS: [(&str, &[&str]); 3] = [("Flag", &[]), ("Maybe", &["T"]), ("Either", &["T", "E"])];

/// `enum Maybe ( T ) { Nil , Some ( a : T ) }`
fn enumeration(r: &mut Rng, which: usize) -> String {
    let (name_, params) = ENUMS[which];
    let field = |r: &mut Rng| {
        let ty = match params.is_empty() || r.chance(50) {
            true => set(r, 0),
            false => params[r.below(params.len())].to_string(),
        };
        format!("{} : {ty}", name(r))
    };
    let variant = |r: &mut Rng| match r.chance(40) {
        true => VARIANTS[r.below(VARIANTS.len())].to_string(),
        false => format!("{} ( {} )", name(r), r.many(1, 2, field).join(" , ")),
    };
    let params = match params.is_empty() {
        true => String::new(),
        false => format!(" ( {} )", params.join(" , ")),
    };
    format!(
        "enum {name_}{params} {{ {} }}",
        r.many(1, 3, variant).join(" , ")
    )
}

fn set_atom(r: &mut Rng, depth: usize) -> String {
    if depth > 0 && r.chance(5) {
        let (name, params) = ENUMS[1 + r.below(2)];
        let sets: Vec<String> = params.iter().map(|_| set(r, depth - 1)).collect();
        return format!("{name} ( {} )", sets.join(" , "));
    }
    match r.below(if depth == 0 { 6 } else { 10 }) {
        0..=3 => TYPES[r.below(TYPES.len())].into(),
        4 => format!("{{ {} }}", r.many(0, 3, value).join(" , ")),
        5 => range(r),
        6 => format!("( {} )", set(r, depth - 1)),
        // Tuples, records with their fields, lists.
        7 if r.chance(10) => "( )".into(),
        7 if r.chance(10) => format!(
            "( {} ) -> {}",
            r.many(0, 2, |r| set(r, depth - 1)).join(" , "),
            TYPES[r.below(TYPES.len())]
        ),
        7 => format!("( {} )", r.many(2, 3, |r| set(r, depth - 1)).join(" , ")),
        8 => format!(
            "{} ( {} )",
            name(r),
            r.many(1, 3, |r| set(r, depth - 1)).join(" , ")
        ),
        _ => match r.below(4) {
            0 => format!("Set ( {} )", set(r, depth - 1)),
            1 => format!("Map ( {} , {} )", set(r, depth - 1), set(r, depth - 1)),
            2 => format!("Map ( {} )", set(r, depth - 1)),
            _ => format!("List ( {} )", set(r, depth - 1)),
        },
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
            let all = pick(r, "forall", "∀");
            let binder = if r.chance(50) {
                format!("{all} {x} , ")
            } else {
                format!("{all} {x} {} {} , ", pick(r, "in", "∈"), set(r, 2))
            };
            bound.push(binder);
            args.push(x.to_string());
        } else {
            args.push(value(r));
        }
    }
    // Binders may come in any order.
    for i in (1..bound.len()).rev() {
        bound.swap(i, r.below(i + 1));
    }
    let args = if args.is_empty() {
        String::new()
    } else {
        format!(" ( {} )", args.join(" , "))
    };
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
    &["==", "!=", "<", "<=", ">", ">=", "in", "∈"],
    &["+", "-", "|", "\\"],
    &["*", "/", "&"],
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
    match r.below(5) {
        0 => "true".into(),
        1 => "false".into(),
        2 => nat(r),
        3 => float(r),
        _ => string(r),
    }
}

fn unary(r: &mut Rng, depth: usize) -> String {
    if depth > 0 && r.chance(15) {
        return match r.below(2) {
            0 => format!("! {}", unary(r, depth - 1)),
            _ => format!("- {}", unary(r, depth - 1)),
        };
    }
    let mut s = atom(r, depth);
    while depth > 0 && r.chance(15) {
        s = match r.below(3) {
            0 => format!("{s} . {}", name(r)),
            1 => format!("{s} . {}", r.below(3)),
            _ => format!("{s} [ {} ]", expr(r, depth - 1)),
        };
    }
    s
}

fn atom(r: &mut Rng, depth: usize) -> String {
    let exprs = |r: &mut Rng, lo| r.many(lo, 3, |r| expr(r, depth - 1)).join(" , ");
    if depth > 0 && r.chance(4) {
        let cond = expr(r, depth - 1);
        return measure(r, &cond);
    }
    match if depth == 0 { r.below(2) } else { r.below(11) } {
        8 => format!("{{ {} }}", exprs(r, 0)),
        9 => match r.chance(20) {
            true => "{ : }".into(),
            false => {
                let pair = |r: &mut Rng| format!("{} : {}", expr(r, depth - 1), expr(r, depth - 1));
                format!("{{ {} }}", r.many(1, 3, pair).join(" , "))
            }
        },
        10 => format!("keys ( {} )", expr(r, depth - 1)),
        7 => {
            let arm = |r: &mut Rng| {
                let pattern = match r.below(3) {
                    0 => "_".to_string(),
                    1 => name(r),
                    _ => format!("{} ( {} )", name(r), r.many(1, 2, name).join(" , ")),
                };
                format!("{pattern} => {}", expr(r, depth - 1))
            };
            format!(
                "match {} {{ {} }}",
                expr(r, depth - 1),
                r.many(1, 3, arm).join(" , ")
            )
        }
        0 => literal(r),
        1 => name(r),
        2 => format!("{} ( {} )", name(r), exprs(r, 1)),
        3 => format!("( {} )", expr(r, depth - 1)),
        4 => format!("( {} )", exprs(r, 2)),
        5 => format!("[ {} ]", exprs(r, 0)),
        _ => format!("len ( {} )", expr(r, depth - 1)),
    }
}

/// `rate ( a in S , cond )` or `precision ( f )`.
fn measure(r: &mut Rng, cond: &str) -> String {
    match r.below(4) {
        0 => format!("precision ( {} )", name(r)),
        1 => format!("recall ( {} )", name(r)),
        2 => format!("false_positive_rate ( {} )", name(r)),
        _ => format!(
            "rate ( {} {} {} , {cond} )",
            name(r),
            pick(r, "in", "∈"),
            set(r, 1)
        ),
    }
}

fn binders(r: &mut Rng) -> String {
    let params = r.many(1, 4, |r| format!("{} : {}", name(r), set(r, 2)));
    format!(" ( {} )", params.join(" , "))
}

/// Type parameters, now and then: ` ( T , U )`.
fn type_params(r: &mut Rng) -> String {
    match r.chance(15) {
        true => format!(" ( {} )", ["T", "T , U"][r.below(2)]),
        false => String::new(),
    }
}

/// The parameters of a function: `()` now and then.
fn fn_binders(r: &mut Rng) -> String {
    match r.chance(10) {
        true => " ( )".to_string(),
        false => binders(r),
    }
}

fn command(r: &mut Rng) -> String {
    if r.chance(10) {
        // With type parameters it is declared like an enum: a name of
        // its own, used nowhere above.
        return match r.chance(15) {
            true => format!(
                "record {} ( {} ){}",
                ["Box", "Both"][r.below(2)],
                ["T", "T , U"][r.below(2)],
                binders(r)
            ),
            false => format!("record {}{}", name(r), binders(r)),
        };
    }

    match r.below(4) {
        3 => format!("test {} : {}", name(r), judgment(r)),
        0 => format!(
            "opaque {}{}{} -> {}",
            name(r),
            type_params(r),
            fn_binders(r),
            set(r, 2)
        ),
        1 => format!(
            "def {}{}{} -> {} := {}",
            name(r),
            type_params(r),
            fn_binders(r),
            set(r, 2),
            expr(r, 3)
        ),
        _ => {
            if r.chance(15) {
                let by = match r.chance(50) {
                    true => format!(" by {}", name(r)),
                    false => String::new(),
                };
                return format!("claim {} : uniform {}{by}", name(r), name(r));
            }
            if r.chance(15) {
                let used = r.many(1, 3, name).join(" , ");
                let only = if r.chance(50) { "only " } else { "" };
                return format!("claim {} : {only}uses {} [ {used} ]", name(r), name(r));
            }
            if r.chance(20) {
                return format!("claim {} : {}", name(r), property(r));
            }
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
    }
}

/// A claim on an expression: every bound name is used, and no other.
fn property(r: &mut Rng) -> String {
    let n = r.below(4);
    let bound = &["x", "y", "z"][..n];
    let forall: String = bound
        .iter()
        .map(|x| match r.chance(50) {
            true => format!("forall {x} , "),
            false => format!("forall {x} in {} , ", set(r, 2)),
        })
        .collect();
    let mut args: Vec<String> = bound.iter().map(|x| x.to_string()).collect();
    if n == 0 || r.chance(30) {
        args.push(scalar(r));
    }
    let args = args.join(" , ");
    let cmp = ["==", "!=", "<", "<=", ">", ">="][r.below(6)];
    let body = match r.below(5) {
        0 => format!("{} ( {args} ) {cmp} {}", name(r), literal(r)),
        1 => format!("len ( {} ( {args} ) ) = {}", name(r), nat(r)),
        2 => format!("{} ( {args} ) == {} ( {args} )", name(r), name(r)),
        3 => format!(
            "{} ( {} ( {args} ) ) != {} && true",
            name(r),
            name(r),
            literal(r)
        ),
        _ => format!(
            "! {} ( {args} ) || {} ( {args} ) {cmp} - {}",
            name(r),
            name(r),
            nat(r)
        ),
    };
    forall + &body
}

/// A random model file.
pub fn model(r: &mut Rng) -> String {
    // An enum is declared before its first use, so they come first.
    let mut enums = String::new();
    for which in 0..ENUMS.len() {
        if r.chance(15) {
            enums += &(enumeration(r, which) + &comment(r) + "\n");
        }
    }
    enums + &r.many(0, 8, |r| command(r) + &comment(r) + "\n").concat()
}

/// Sometimes a comment, at the end of a line or on its own line.
fn comment(r: &mut Rng) -> String {
    match r.below(6) {
        0 => " -- a comment, with forall and \"quotes\"".into(),
        1 => "\n-- a line comment".into(),
        _ => String::new(),
    }
}

/// A random evidence file. The bias goes from 0 to 1.
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
            ["0", "1", "0.5", "0.25"][r.below(4)],
            comment(r)
        )
    })
    .concat()
}

/// `∀ …, <judgment>`, as in evidence and tests.
fn judgment(r: &mut Rng) -> String {
    if r.chance(8) {
        // A measurement names every input: no name is left to bind.
        let cond = format!("{} ( {} ) == {}", name(r), scalar(r), literal(r));
        let value = ["0", "1", "0.5", "0.25"][r.below(4)];
        return format!("{} = {value}", measure(r, &cond));
    }
    match r.below(5) {
        0 => {
            let (forall, args) = call_args(r, 1);
            format!("{forall}correct ( {}{args} )", name(r))
        }
        1 => {
            let (forall, args) = call_args(r, 1);
            let not = pick(r, "not", "¬");
            format!("{forall}{not} correct ( {}{args} )", name(r))
        }
        2 => {
            let (forall, args) = call_args(r, 0);
            format!("{forall}{}{args}", name(r))
        }
        3 => {
            let (forall, args) = call_args(r, 0);
            let not = pick(r, "not", "¬");
            format!("{forall}{not} {}{args}", name(r))
        }
        _ => {
            let (forall, args) = call_args(r, 1);
            format!("{forall}{}{args} = {}", name(r), value(r))
        }
    }
}
