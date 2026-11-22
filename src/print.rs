//! Model to text (`docs/syntax.md`): what `src/parse.rs` reads.
//!
//! One way to write each model: four spaces a level, a part's requires,
//! then records, functions and ensures, each tie in the block of what it
//! ties, a path as short as it reads back the same. Comments are not
//! kept: they are not in the model.
//!
//! A model whose names are words (M001) and whose expressions are made
//! as their operations take them is written so that `parse` gives it
//! back, ties aside: a tie of a part, record or function is written in
//! its block, any other at the end, `tie a::b at "…";`. A name a proof
//! gives a proposition that is no path, as a tie's `stone::drop at
//! lab/tower`, cannot be written; nor an operation applied to other
//! arguments than it takes, written `__malformed__`.

use crate::expression::{Arg, Expression};
use crate::judgement::{self, Prop, Set};
use crate::location::Location;
use crate::model::{
    Ensure, Function, Kind, Model, Part, Path, Record, Require, Rule, Source, Term, Tie,
};
use crate::simulation::{Op, Rational, Type, Value};
use num_bigint::BigInt;
use num_traits::{One, Signed, Zero};
use std::collections::{BTreeMap, BTreeSet};

/// The text of a model.
pub fn print(model: &Model) -> String {
    let items = items(model);
    let mut placed = vec![false; model.ties.len()];
    let mut out = String::new();
    let mut seen_parts = BTreeSet::new();
    for (i, part) in model.parts.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        let first = seen_parts.insert(part.name.clone());
        let w = Writer {
            part: Some(&part.name),
            items: &items,
        };
        w.part(part, model, &mut placed, first, &mut out);
    }
    let loose: Vec<&Tie> = model
        .ties
        .iter()
        .zip(&placed)
        .filter(|(_, p)| !**p)
        .map(|(t, _)| t)
        .collect();
    if !loose.is_empty() {
        if !model.parts.is_empty() {
            out.push('\n');
        }
        for tie in loose {
            let item = tie.item.names.join("::");
            out.push_str(&format!("tie {item} at {};\n", location(&tie.at)));
        }
    }
    out
}

/// A proposition, its paths whole, as a store of reviews writes it.
pub fn print_prop(prop: &Prop) -> String {
    let items = BTreeMap::new();
    Writer {
        part: None,
        items: &items,
    }
    .prop(prop)
}

/// A row, `(6, 2) -> 3`.
pub fn print_row(takes: &[Value], gives: &Value) -> String {
    format!("{} -> {}", values(takes), value(gives))
}

/// The names of each part: its records, functions, requires and ensures.
fn items(model: &Model) -> BTreeMap<String, BTreeSet<String>> {
    let mut items: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    for part in &model.parts {
        let names = items.entry(part.name.clone()).or_default();
        names.extend(part.records.iter().map(|r| r.name.clone()));
        names.extend(part.functions.iter().map(|f| f.name.clone()));
        names.extend(part.requires.iter().map(|q| q.name.clone()));
        names.extend(part.ensures.iter().map(|e| e.name.clone()));
    }
    items
}

/// `"lab/tower/stone"`, escaped as Rust escapes a string.
fn location(at: &Location) -> String {
    string(&at.path.join("/"))
}

fn string(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// What a part reads paths from.
struct Writer<'a> {
    part: Option<&'a str>,
    items: &'a BTreeMap<String, BTreeSet<String>>,
}

const MALFORMED: &str = "__malformed__";

// Precedence, from the loosest.
const LET: u8 = 0;
const OR: u8 = 1;
const AND: u8 = 2;
const CMP: u8 = 3;
const SUM: u8 = 4;
const PRODUCT: u8 = 5;
const UNARY: u8 = 6;
const POSTFIX: u8 = 7;
const ATOM: u8 = 8;

fn wrap((s, p): (String, u8), at_least: u8) -> String {
    match p < at_least {
        true => format!("({s})"),
        false => s,
    }
}

impl Writer<'_> {
    fn local(&self, name: &str) -> bool {
        self.part
            .and_then(|p| self.items.get(p))
            .is_some_and(|items| items.contains(name))
    }

    /// As short as it reads back the same: from the part when it is in
    /// it, `crate::` first when a whole path would read as one from the part.
    fn path(&self, names: &[String], bound: &[String]) -> String {
        if names.is_empty() {
            return MALFORMED.to_string();
        }
        if let Some(part) = self.part {
            if names.len() >= 2 && names[0] == part && self.local(&names[1]) {
                let short = &names[1..];
                if !(short.len() == 1 && bound.contains(&short[0])) {
                    return short.join("::");
                }
            }
        }
        let clash = self.local(&names[0])
            || names[0] == "crate"
            || (names.len() == 1 && bound.contains(&names[0]));
        match clash {
            true => format!("crate::{}", names.join("::")),
            false => names.join("::"),
        }
    }
    fn path_of(&self, p: &Path) -> String {
        self.path(&p.names, &[])
    }
    /// A name as a proof or proposition writes it, `stone::drop::falls`.
    fn named(&self, name: &str, bound: &[String]) -> String {
        if bound.iter().any(|b| b == name) {
            return name.to_string();
        }
        let names: Vec<String> = name.split("::").map(str::to_string).collect();
        self.path(&names, bound)
    }

    fn ties(&self, item: &Path, model: &Model, placed: &mut [bool]) -> String {
        let mut at = vec![];
        for (i, tie) in model.ties.iter().enumerate() {
            if !placed[i] && tie.item == *item {
                placed[i] = true;
                at.push(location(&tie.at));
            }
        }
        match at.is_empty() {
            true => String::new(),
            false => format!(" at {}", at.join(", ")),
        }
    }

    fn part(&self, part: &Part, model: &Model, placed: &mut [bool], first: bool, out: &mut String) {
        let kind = match part.kind {
            Kind::Subject => "subject",
            Kind::Environment => "environment",
        };
        let here = Path::to(&[&part.name]);
        let tied = |item: &Path, placed: &mut [bool]| match first {
            true => self.ties(item, model, placed),
            false => String::new(),
        };
        out.push_str(&format!("{kind} {}{} {{\n", part.name, tied(&here, placed)));
        let mut blocks: Vec<String> = vec![];
        let requires: String = part.requires.iter().map(|q| self.require(q)).collect();
        if !requires.is_empty() {
            blocks.push(requires);
        }
        let mut seen = BTreeSet::new();
        for r in &part.records {
            let ties = match seen.insert(r.name.clone()) {
                true => tied(&here.then(&r.name), placed),
                false => String::new(),
            };
            blocks.push(self.record(r, &ties));
        }
        for f in &part.functions {
            let ties = match seen.insert(f.name.clone()) {
                true => tied(&here.then(&f.name), placed),
                false => String::new(),
            };
            blocks.push(self.function(f, &ties));
        }
        let ensures: String = part.ensures.iter().map(|e| self.ensure(e)).collect();
        if !ensures.is_empty() {
            blocks.push(ensures);
        }
        out.push_str(&blocks.join("\n"));
        out.push_str("}\n");
    }

    fn require(&self, q: &Require) -> String {
        let from = match &q.from {
            Source::Prose(s) => string(s),
            Source::Ensure(p) => self.path_of(p),
        };
        format!("    require {} = {from};\n", q.name)
    }

    fn ensure(&self, e: &Ensure) -> String {
        let mut s = format!("    ensure {} = {}", e.name, string(&e.prose));
        if !e.from.is_empty() {
            let lists: Vec<String> = e
                .from
                .iter()
                .map(|l| {
                    format!(
                        "({})",
                        l.iter()
                            .map(|p| self.path_of(p))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                })
                .collect();
            s.push_str(&format!(" from {}", lists.join(" | ")));
        }
        s.push_str(";\n");
        s
    }

    fn record(&self, r: &Record, ties: &str) -> String {
        let mut s = format!("    record {}{ties}", r.name);
        if let Some(fields) = &r.fields {
            let fields: Vec<String> = fields
                .iter()
                .map(|(n, t)| format!("{n}: {}", type_(t)))
                .collect();
            match fields.is_empty() {
                true => s.push_str(" {}"),
                false => s.push_str(&format!(" {{ {} }}", fields.join(", "))),
            }
        }
        let said = !r.claims.is_empty() || !r.rules.is_empty();
        match (said, r.fields.is_some()) {
            (false, true) => s.push('\n'),
            (false, false) => s.push_str(";\n"),
            (true, _) => s.push_str(&self.where_(&r.claims, &r.rules, &[])),
        }
        s
    }

    fn function(&self, f: &Function, ties: &str) -> String {
        let takes: Vec<String> = f
            .takes
            .iter()
            .map(|(n, t)| format!("{n}: {}", type_(t)))
            .collect();
        let mut s = format!(
            "    fn {}({}) -> {}: {}{ties}",
            f.name,
            takes.join(", "),
            f.gives.0,
            type_(&f.gives.1)
        );
        if let Some(body) = &f.body {
            s.push_str(&format!(" {{ {} }}", sim(body)));
        }
        let said = !f.claims.is_empty() || !f.rules.is_empty() || !f.examples.is_empty();
        let rows: Vec<String> = f
            .examples
            .iter()
            .map(|e| print_row(&e.takes, &e.gives))
            .collect();
        match (said, f.body.is_some()) {
            (false, true) => s.push('\n'),
            (false, false) => s.push_str(";\n"),
            (true, _) => s.push_str(&self.where_(&f.claims, &f.rules, &rows)),
        }
        s
    }

    fn where_(&self, claims: &[crate::model::Claim], rules: &[Rule], rows: &[String]) -> String {
        let mut s = String::from(" where {\n");
        for c in claims {
            s.push_str(&format!("        claim {} = {};\n", c.name, sim(&c.holds)));
        }
        for r in rules {
            s.push_str(&self.rule(r));
        }
        if !rows.is_empty() {
            let one = format!("        examples {{ {} }}\n", rows.join(", "));
            match one.len() <= 100 {
                true => s.push_str(&one),
                false => {
                    s.push_str("        examples {\n");
                    for row in rows {
                        s.push_str(&format!("            {row},\n"));
                    }
                    s.push_str("        }\n");
                }
            }
        }
        s.push_str("    }\n");
        s
    }

    fn term(&self, t: &Term) -> String {
        match t {
            Term::At(p) => self.path_of(p),
            Term::Holds(e) => format!("{{ {} }}", sim(e)),
        }
    }

    fn rule(&self, r: &Rule) -> String {
        let premises: Vec<String> = r.premises.iter().map(|t| self.term(t)).collect();
        let premises = match premises.is_empty() {
            true => String::new(),
            false => format!("{} ", premises.join(", ")),
        };
        let mut s = format!(
            "        rule {}: {premises}|- {}",
            r.name,
            self.term(&r.conclusion)
        );
        if let Some(proof) = &r.proof {
            s.push_str(&format!(
                "\n            by {}",
                self.proof(proof, &mut vec![]).0
            ));
        }
        s.push_str(";\n");
        s
    }

    // ------------------------------------------------ proofs

    fn proof(&self, e: &Expression<judgement::Op>, bound: &mut Vec<String>) -> (String, u8) {
        use judgement::Op as O;
        let (op, args) = match e {
            Expression::Name(n) => {
                let written = match n.parse::<usize>() {
                    Ok(_) => n.clone(),
                    Err(_) => self.named(n, bound),
                };
                return (written, ATOM);
            }
            Expression::Apply(op, args) => (op, args),
        };
        if let O::Let = op {
            return match &args[..] {
                [Arg { binds: b0, body: a }, Arg { binds: b1, body: b }]
                    if b0.is_empty() && b1.len() == 1 =>
                {
                    let a = self.proof(a, bound).0;
                    bound.push(b1[0].clone());
                    let b = self.proof(b, bound).0;
                    bound.pop();
                    (format!("let {} = {a} in {b}", b1[0]), LET)
                }
                _ => (MALFORMED.to_string(), ATOM),
            };
        }
        // `intro[q](|x| p)`.
        if let O::Intro { q } = op {
            return match &args[..] {
                [Arg { binds, body }] if binds.len() == 1 => {
                    bound.push(binds[0].clone());
                    let b = self.proof(body, bound).0;
                    bound.pop();
                    (format!("intro[{}](|{}| {b})", self.prop(q), binds[0]), ATOM)
                }
                _ => (MALFORMED.to_string(), ATOM),
            };
        }
        if args.iter().any(|a| !a.binds.is_empty()) {
            return (MALFORMED.to_string(), ATOM);
        }
        let mut each = |at_least: u8| -> Vec<String> {
            args.iter()
                .map(|a| wrap(self.proof(&a.body, bound), at_least))
                .collect()
        };
        match op {
            O::Not if args.len() == 1 => (format!("!{}", each(UNARY)[0]), UNARY),
            O::And if args.len() == 2 => {
                let left = wrap(self.proof(&args[0].body, bound), AND);
                let right = wrap(self.proof(&args[1].body, bound), AND + 1);
                (format!("{left} && {right}"), AND)
            }
            O::Not | O::And | O::Let => (MALFORMED.to_string(), ATOM),
            O::Join => (format!("join({})", each(LET).join(", ")), ATOM),
            O::StateOf => (format!("state({})", each(LET).join(", ")), ATOM),
            O::Ponens { t, post } => {
                let sugar = matches!((t, args.first()), (Prop::Named(r), Some(Arg { body: Expression::Name(n), .. })) if r == n)
                    && *post == 0;
                let args = each(LET).join(", ");
                match (sugar, post) {
                    (true, _) => (format!("ponens({args})"), ATOM),
                    (false, 0) => (format!("ponens[{}]({args})", self.prop(t)), ATOM),
                    (false, _) => (format!("ponens[{}, {post}]({args})", self.prop(t)), ATOM),
                }
            }
            O::Tollens { t, post, blame } => {
                let args = each(LET).join(", ");
                (
                    format!("tollens[{}, {post}, {blame}]({args})", self.prop(t)),
                    ATOM,
                )
            }
            O::Reductio { of } => (
                format!("reductio[{}]({})", self.prop(of), each(LET).join(", ")),
                ATOM,
            ),
            O::Instance { q } => (
                format!("instance[{}]({})", self.prop(q), each(LET).join(", ")),
                ATOM,
            ),
            O::Witness { q } => (
                format!("witness[{}]({})", self.prop(q), each(LET).join(", ")),
                ATOM,
            ),
            O::Point(at) if args.is_empty() => (format!("point{}", values(at)), ATOM),
            O::Point(_) | O::Intro { .. } => (MALFORMED.to_string(), ATOM),
            O::Cases { q } => (
                format!("cases[{}]({})", self.prop(q), each(LET).join(", ")),
                ATOM,
            ),
        }
    }

    // ------------------------------------------------ propositions

    fn prop(&self, p: &Prop) -> String {
        self.prop_at(p).0
    }
    fn prop_at(&self, p: &Prop) -> (String, u8) {
        match p {
            Prop::Named(n) => (self.named(n, &[]), ATOM),
            Prop::Not(x) => (format!("!{}", wrap(self.prop_at(x), ATOM)), UNARY),
            Prop::Implies { pre, subject, post } => {
                let list = |ps: &[Prop]| {
                    ps.iter()
                        .map(|p| self.prop(p))
                        .collect::<Vec<_>>()
                        .join(", ")
                };
                (
                    format!(
                        "{{{}}} {} {{{}}}",
                        list(pre),
                        self.named(subject, &[]),
                        list(post)
                    ),
                    ATOM,
                )
            }
            Prop::Correct { function, at } => {
                let at = match at {
                    judgement::Term::Point(vs) => values(vs),
                    judgement::Term::Var(x) => x.clone(),
                    judgement::Term::Arbitrary { name, over } => format!("{name} in {}", set(over)),
                };
                (
                    format!("correct({}, {at})", self.named(function, &[])),
                    ATOM,
                )
            }
            Prop::All { name, over, body } => (
                format!("all {name} in {}: {}", set(over), self.prop(body)),
                LET,
            ),
            Prop::Any { name, over, body } => (
                format!("any {name} in {}: {}", set(over), self.prop(body)),
                LET,
            ),
        }
    }
}

fn set(s: &Set) -> String {
    let ranges: Vec<String> = s
        .ranges
        .iter()
        .map(|(n, lo, hi)| format!("{n}: {}..{}", value(lo), value(hi)))
        .collect();
    let mut out = format!("({})", ranges.join(", "));
    if let Some(such) = &s.such {
        out.push_str(&format!(" where {}", sim(such)));
    }
    out
}

// ---------------------------------------------------------------- types and values

/// `bool`, `Int`, `Real`, `(A, B)`, or a record's name.
pub fn type_(t: &Type) -> String {
    match t {
        Type::Truth => "bool".to_string(),
        Type::Int => "Int".to_string(),
        Type::Real => "Real".to_string(),
        Type::Record(r) if r.name.is_empty() => {
            let types: Vec<String> = r.fields.iter().map(|(_, t)| type_(t)).collect();
            match types.len() {
                1 => format!("({},)", types[0]),
                _ => format!("({})", types.join(", ")),
            }
        }
        Type::Record(r) => r.name.clone(),
        // Stub until the parser thread writes contracts: the name.
        Type::Param { name, .. } => name.clone(),
    }
}

/// `9.81`, `2.0`, `-0.5`; a ratio no decimal writes, `1/3`.
pub fn real(r: &Rational) -> String {
    let (numer, denom) = (r.numer(), r.denom());
    let (mut twos, mut fives, mut rest) = (0u32, 0u32, denom.clone());
    let (two, five) = (BigInt::from(2), BigInt::from(5));
    while !rest.is_zero() && (&rest % &two).is_zero() {
        rest /= &two;
        twos += 1;
    }
    while !rest.is_zero() && (&rest % &five).is_zero() {
        rest /= &five;
        fives += 1;
    }
    if !rest.is_one() {
        return format!("{numer}/{denom}");
    }
    let k = twos.max(fives);
    let scaled = numer.abs() * BigInt::from(10).pow(k) / denom;
    let mut digits = scaled.to_string();
    while digits.len() <= k as usize {
        digits.insert(0, '0');
    }
    let point = digits.len() - k as usize;
    let (whole, frac) = digits.split_at(point);
    let frac = match frac.is_empty() {
        true => "0",
        false => frac,
    };
    let sign = match numer.is_negative() {
        true => "-",
        false => "",
    };
    format!("{sign}{whole}.{frac}")
}

pub fn value(v: &Value) -> String {
    match v {
        Value::Truth(b) => b.to_string(),
        Value::Int(n) => n.to_string(),
        Value::Real(r) => real(r),
        Value::Record(r, vs) if r.name.is_empty() => match vs.len() {
            1 => format!("({},)", value(&vs[0])),
            _ => values(vs),
        },
        Value::Record(r, vs) => {
            let fields: Vec<String> = r
                .fields
                .iter()
                .zip(vs)
                .map(|((n, _), v)| format!("{n}: {}", value(v)))
                .collect();
            format!("{} {{ {} }}", r.name, fields.join(", "))
        }
    }
}

/// `(6, 2)`: a list of values, in parentheses.
pub fn values(vs: &[Value]) -> String {
    format!("({})", vs.iter().map(value).collect::<Vec<_>>().join(", "))
}

// ---------------------------------------------------------------- simulation

/// An expression of simulation, as Rust writes it, `let x = a in b` aside.
pub fn sim(e: &Expression<Op>) -> String {
    sim_at(e).0
}

/// Whether `e` makes a named record anywhere: then it is put in
/// parentheses as the condition of an `if`, as Rust asks.
fn makes_record(e: &Expression<Op>) -> bool {
    match e {
        Expression::Name(_) => false,
        Expression::Apply(op, args) => {
            matches!(op, Op::Make(r) if !r.name.is_empty())
                || args.iter().any(|a| makes_record(&a.body))
        }
    }
}

fn sim_at(e: &Expression<Op>) -> (String, u8) {
    let (op, args) = match e {
        Expression::Name(n) => return (n.clone(), ATOM),
        Expression::Apply(op, args) => (op, args),
    };
    let bad = (MALFORMED.to_string(), ATOM);
    let plain = args.iter().all(|a| a.binds.is_empty());
    let body = |i: usize| &args[i].body;
    let binary = |symbol: &str, p: u8, left: u8, right: u8| {
        (
            format!(
                "{} {symbol} {}",
                wrap(sim_at(body(0)), left),
                wrap(sim_at(body(1)), right)
            ),
            p,
        )
    };
    match (op, args.len(), plain) {
        (Op::Let, 2, _) => match (&args[0].binds[..], &args[1].binds[..]) {
            ([], [x]) => (
                format!("let {x} = {} in {}", sim(body(0)), sim(body(1))),
                LET,
            ),
            _ => bad,
        },
        (Op::All, 3, _) => match (&args[0].binds[..], &args[1].binds[..], &args[2].binds[..]) {
            ([], [], [x]) => (
                format!(
                    "({}..{}).all(|{x}| {})",
                    sim(body(0)),
                    sim(body(1)),
                    sim(body(2))
                ),
                POSTFIX,
            ),
            _ => bad,
        },
        (_, _, false) => bad,
        // Stub until the parser thread writes `use`: `sun::light(a, b)`.
        (Op::Call(f), _, _) => {
            let args: Vec<String> = args.iter().map(|a| sim(&a.body)).collect();
            (format!("{f}({})", args.join(", ")), POSTFIX)
        }
        (Op::Truth(b), 0, _) => (b.to_string(), ATOM),
        (Op::Int(n), 0, _) => (n.to_string(), if n.is_negative() { UNARY } else { ATOM }),
        (Op::Real(r), 0, _) => (real(r), if r.numer().is_negative() { UNARY } else { ATOM }),
        (Op::Not, 1, _) => match body(0) {
            Expression::Apply(Op::Equal, inner)
                if inner.len() == 2 && inner.iter().all(|a| a.binds.is_empty()) =>
            {
                let (a, b) = (&inner[0].body, &inner[1].body);
                (
                    format!(
                        "{} != {}",
                        wrap(sim_at(a), CMP + 1),
                        wrap(sim_at(b), CMP + 1)
                    ),
                    CMP,
                )
            }
            x => (format!("!{}", wrap(sim_at(x), UNARY)), UNARY),
        },
        (Op::Or, 2, _) => binary("||", OR, OR, OR + 1),
        (Op::And, 2, _) => binary("&&", AND, AND, AND + 1),
        (Op::Equal, 2, _) => binary("==", CMP, CMP + 1, CMP + 1),
        (Op::Less, 2, _) => binary("<", CMP, CMP + 1, CMP + 1),
        (Op::AtMost, 2, _) => binary("<=", CMP, CMP + 1, CMP + 1),
        (Op::Add, 2, _) => binary("+", SUM, SUM, SUM + 1),
        (Op::Sub, 2, _) => binary("-", SUM, SUM, SUM + 1),
        (Op::Mul, 2, _) => binary("*", PRODUCT, PRODUCT, PRODUCT + 1),
        (Op::Div, 2, _) => binary("/", PRODUCT, PRODUCT, PRODUCT + 1),
        (Op::Within, 3, _) => (
            format!(
                "within({}, {}, {})",
                sim(body(0)),
                sim(body(1)),
                sim(body(2))
            ),
            ATOM,
        ),
        (Op::If, 3, _) => {
            let c = body(0);
            let c = match makes_record(c) {
                true => format!("({})", sim(c)),
                false => wrap(sim_at(c), OR),
            };
            let otherwise = match body(2) {
                e @ Expression::Apply(Op::If, a)
                    if a.len() == 3 && a.iter().all(|a| a.binds.is_empty()) =>
                {
                    sim(e)
                }
                e => format!("{{ {} }}", sim(e)),
            };
            (
                format!("if {c} {{ {} }} else {otherwise}", sim(body(1))),
                LET,
            )
        }
        (Op::Make(r), n, _) if n == r.fields.len() => {
            let items: Vec<String> = (0..n).map(|i| sim(body(i))).collect();
            match (r.name.is_empty(), n) {
                (true, 1) => (format!("({},)", items[0]), ATOM),
                (true, _) => (format!("({})", items.join(", ")), ATOM),
                (false, _) => {
                    let fields: Vec<String> = r
                        .fields
                        .iter()
                        .zip(&items)
                        .map(|((f, _), x)| format!("{f}: {x}"))
                        .collect();
                    (format!("{} {{ {} }}", r.name, fields.join(", ")), ATOM)
                }
            }
        }
        (Op::Field(f), 1, _) => (format!("{}.{f}", wrap(sim_at(body(0)), POSTFIX)), POSTFIX),
        _ => bad,
    }
}
