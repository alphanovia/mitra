//! Text to model (`docs/syntax.md`): what `src/print.rs` writes, read
//! back. `parse(print(m))` is `m`, but for the order of its ties, which
//! the text writes in the block of what they tie.
//!
//! The text is read twice. The first time learns the records and the
//! names of each part, and nothing else. The second reads it again
//! knowing them: a record's type is its declared fields, a tuple's type
//! is the types of what it holds, and a short path `drop::falls`, in a
//! part, is the path from that part, `stone::drop::falls`.
//!
//! A problem in the text is a `ParseError`, with a code of the
//! catalogue (suit `P`) and where it is: line and column, as rustc
//! writes them. Only the first is given.

use crate::catalogue::{Code, P001, P002, P003, P004, P005, P006, P007, P008, P009, P010, P011};
use crate::expression::{check as typed, Arg, Expression};
use crate::judgement::{self, Prop, Set};
use crate::location::Location;
use crate::model::{
    Claim, Contract, Ensure, Example, Function, Implements, Kind, Law, Model, ModelExample, Part,
    Path, Record, Require, Rule, Signature, Source, Term, Tie, Use,
};
use crate::simulation::{self, Op, Rational, Simulation, Type, Value};
use num_bigint::BigInt;
use std::collections::{BTreeMap, BTreeSet};

/// Where in the text: line and column, from 1. A column counts
/// characters.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos {
    pub line: usize,
    pub column: usize,
}

/// The text cannot be read as a model, at `at`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParseError {
    pub code: Code,
    pub at: Pos,
    pub message: String,
}

/// `error[P003]: …`, then where, as rustc writes it without the text.
impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "error[{}]: {}\n --> {}:{}",
            self.code, self.message, self.at.line, self.at.column
        )
    }
}

impl ParseError {
    /// As rustc writes it, with the line of `source` it is on and a
    /// caret under where: `file` names the text.
    ///
    /// ```text
    /// error[P003]: expected `;`, found `}`
    ///  --> fall.mitra:3:27
    ///   |
    /// 3 |     fn light() -> w: Real }
    ///   |                           ^
    /// ```
    pub fn render(&self, file: &str, source: &str) -> String {
        let line = source
            .lines()
            .nth(self.at.line.saturating_sub(1))
            .unwrap_or("");
        let number = self.at.line.to_string();
        let pad = " ".repeat(number.len());
        let caret = " ".repeat(self.at.column.saturating_sub(1));
        format!(
            "error[{}]: {}\n{pad}--> {file}:{}:{}\n{pad} |\n{number} | {line}\n{pad} | {caret}^\n",
            self.code, self.message, self.at.line, self.at.column
        )
    }
}

type Result<T> = std::result::Result<T, ParseError>;

fn error(code: Code, at: Pos, message: impl Into<String>) -> ParseError {
    ParseError {
        code,
        at,
        message: message.into(),
    }
}

/// A model from its text.
/// A `use std::…` reads Mitra's own standard model; a part no file
/// declares is left to `check` (`load` reads files).
pub fn parse(source: &str) -> Result<Model> {
    parse_with(source, &Settings::default())
}

/// `parse`, with these settings.
pub fn parse_with(source: &str, settings: &Settings) -> Result<Model> {
    files(source, None, settings).map_err(|l| l.error)
}

/// A proposition from its text, its paths whole (`stone::drop::falls`),
/// as a store of reviews writes it. `model` gives the records its
/// values are made of.
pub fn parse_prop(source: &str, model: &Model) -> Result<Prop> {
    let context = Context::of(model);
    let mut p = Parser::new(lex(source)?, Some(&context));
    let prop = p.prop()?;
    p.end()?;
    Ok(prop)
}

/// A row, `(6, 2) -> 3`: what a function takes, in order, and what it
/// gives. `model` gives the records its values are made of.
pub fn parse_row(source: &str, model: &Model) -> Result<(Vec<Value>, Value)> {
    let context = Context::of(model);
    let mut p = Parser::new(lex(source)?, Some(&context));
    let row = p.row()?;
    p.end()?;
    Ok(row)
}

// ---------------------------------------------------------------- words

/// Words that are never names.
pub const KEYWORDS: &[&str] = &[
    "subject",
    "environment",
    "record",
    "fn",
    "require",
    "ensure",
    "claim",
    "rule",
    "examples",
    "where",
    "at",
    "from",
    "by",
    "let",
    "in",
    "if",
    "else",
    "true",
    "false",
    "tie",
    "crate",
    "all",
    "any",
    "use",
    "impl",
    "for",
    "contract",
    "enum",
    "match",
];

/// The operations of a proof, by the word that writes them. Each is
/// an operation only when `(` or `[` follows it.
const PROOF_OPS: &[&str] = &[
    "join", "ponens", "tollens", "reductio", "instance", "witness", "cases", "intro", "point",
];

// ---------------------------------------------------------------- tokens

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Tok {
    Word(String),
    Int(BigInt),
    Real(Rational),
    Str(String),
    Sym(&'static str),
    End,
}

/// Longest first.
const SYMBOLS: &[&str] = &[
    "::", "->", "=>", "|-", "==", "!=", "<=", ">=", "&&", "||", "..", "{", "}", "(", ")", "[", "]",
    "++", ",", ";", ":", "=", "<", ">", "+", "-", "*", "/", "%", "!", ".", "|", "~",
];

fn describe(t: &Tok) -> String {
    match t {
        Tok::Word(w) => format!("`{w}`"),
        Tok::Int(n) => format!("`{n}`"),
        Tok::Real(_) => "a number".to_string(),
        Tok::Str(_) => "a string".to_string(),
        Tok::Sym(s) => format!("`{s}`"),
        Tok::End => "the end of the text".to_string(),
    }
}

pub(crate) fn lex(source: &str) -> Result<Vec<(Tok, Pos)>> {
    let chars: Vec<char> = source.chars().collect();
    let (mut i, mut line, mut column) = (0, 1, 1);
    let mut tokens: Vec<(Tok, Pos)> = Vec::new();
    // Moves on by `n` characters.
    let advance = |i: &mut usize, line: &mut usize, column: &mut usize, n: usize| {
        for _ in 0..n {
            if chars.get(*i) == Some(&'\n') {
                *line += 1;
                *column = 1;
            } else {
                *column += 1;
            }
            *i += 1;
        }
    };
    let digits = |from: usize| {
        let mut j = from;
        while chars.get(j).is_some_and(|c| c.is_ascii_digit()) {
            j += 1;
        }
        j
    };
    while i < chars.len() {
        let c = chars[i];
        let at = Pos { line, column };
        if c.is_whitespace() {
            advance(&mut i, &mut line, &mut column, 1);
            continue;
        }
        if c == '/' && chars.get(i + 1) == Some(&'/') {
            while i < chars.len() && chars[i] != '\n' {
                advance(&mut i, &mut line, &mut column, 1);
            }
            continue;
        }
        if c.is_alphabetic() || c == '_' {
            let mut j = i;
            while chars
                .get(j)
                .is_some_and(|c| c.is_alphanumeric() || *c == '_')
            {
                j += 1;
            }
            let word: String = chars[i..j].iter().collect();
            {
                let n = j - i;
                advance(&mut i, &mut line, &mut column, n);
            }
            tokens.push((Tok::Word(word), at));
            continue;
        }
        if c.is_ascii_digit() {
            let j = digits(i);
            let whole: String = chars[i..j].iter().collect();
            let after_dot = matches!(tokens.last(), Some((Tok::Sym("."), _)));
            let next = |k: usize| chars.get(k).copied();
            let digit_at = |k: usize| next(k).is_some_and(|c| c.is_ascii_digit());
            if !after_dot && next(j) == Some('.') && digit_at(j + 1) {
                // A decimal: `9.81`.
                let k = digits(j + 1);
                let frac: String = chars[j + 1..k].iter().collect();
                let numer: BigInt = format!("{whole}{frac}").parse().unwrap_or_default();
                let denom = BigInt::from(10u32).pow(frac.len() as u32);
                let real = Rational::new(numer, denom).unwrap_or_else(|| Rational::integer(0));
                {
                    let n = k - i;
                    advance(&mut i, &mut line, &mut column, n);
                }
                tokens.push((Tok::Real(real), at));
            } else if !after_dot && next(j) == Some('/') && digit_at(j + 1) {
                // A fraction: `1/3`.
                let k = digits(j + 1);
                let numer: BigInt = whole.parse().unwrap_or_default();
                let denom: BigInt = chars[j + 1..k]
                    .iter()
                    .collect::<String>()
                    .parse()
                    .unwrap_or_default();
                let Some(real) = Rational::new(numer, denom) else {
                    return Err(error(P005, at, "a fraction with zero below"));
                };
                {
                    let n = k - i;
                    advance(&mut i, &mut line, &mut column, n);
                }
                tokens.push((Tok::Real(real), at));
            } else {
                {
                    let n = j - i;
                    advance(&mut i, &mut line, &mut column, n);
                }
                tokens.push((Tok::Int(whole.parse().unwrap_or_default()), at));
            }
            continue;
        }
        if c == '"' {
            let mut text = String::new();
            let mut j = i + 1;
            loop {
                match chars.get(j) {
                    None => return Err(error(P002, at, "a string that is not closed")),
                    Some('"') => break,
                    Some('\\') => {
                        let escaped = match chars.get(j + 1) {
                            Some('n') => '\n',
                            Some('t') => '\t',
                            Some('"') => '"',
                            Some('\\') => '\\',
                            _ => {
                                let here = Pos {
                                    line,
                                    column: column + (j - i),
                                };
                                let message =
                                    "a `\\` that escapes nothing: `\\n`, `\\t`, `\\\"` or `\\\\`";
                                return Err(error(P001, here, message));
                            }
                        };
                        text.push(escaped);
                        j += 2;
                    }
                    Some(c) => {
                        text.push(*c);
                        j += 1;
                    }
                }
            }
            {
                let n = j + 1 - i;
                advance(&mut i, &mut line, &mut column, n);
            }
            tokens.push((Tok::Str(text), at));
            continue;
        }
        let rest: String = chars[i..(i + 2).min(chars.len())].iter().collect();
        let Some(sym) = SYMBOLS.iter().find(|s| rest.starts_with(**s)) else {
            return Err(error(
                P001,
                at,
                format!("`{c}` starts nothing in the language"),
            ));
        };
        advance(&mut i, &mut line, &mut column, sym.chars().count());
        tokens.push((Tok::Sym(sym), at));
    }
    tokens.push((Tok::End, Pos { line, column }));
    Ok(tokens)
}

// ---------------------------------------------------------------- context

/// What the first reading learns: each record's type, the names of its
/// fields as declared (`None` when opaque), and the names of each part.
pub(crate) struct Context {
    records: BTreeMap<String, simulation::Record>,
    /// Each enum's type, its parameters open, and the parameters of each
    /// enum and record that has some.
    enums: BTreeMap<String, simulation::Enum>,
    params: BTreeMap<String, Vec<(String, simulation::Bound)>>,
    /// Each variant's name, and the enums that have one of that name.
    variants: BTreeMap<String, Vec<String>>,
    declared: BTreeMap<String, Option<Vec<String>>>,
    items: BTreeMap<String, BTreeSet<String>>,
    /// Who calls what: the functions of each part and contract, and each part's uses.
    pub(crate) calling: Calling,
    /// The model's functions, to type a call.
    simulation: Simulation,
    /// Each contract's functions, to type its claims' calls.
    contracts: BTreeMap<String, Simulation>,
}

/// What a call written by a name alone, `light(h)`, names: the
/// functions of each part and contract, and each part's uses.
#[derive(Clone, Debug, Default)]
pub(crate) struct Calling {
    pub(crate) functions: BTreeMap<String, BTreeSet<String>>,
    pub(crate) uses: BTreeMap<String, Vec<Use>>,
    pub(crate) contracts: BTreeMap<String, BTreeSet<String>>,
}

impl Calling {
    pub(crate) fn of(model: &Model) -> Calling {
        let mut calling = Calling::default();
        for part in &model.parts {
            let fns = calling.functions.entry(part.name.clone()).or_default();
            fns.extend(part.functions.iter().map(|f| f.name.clone()));
            let uses = calling.uses.entry(part.name.clone()).or_default();
            uses.extend(part.uses.iter().cloned());
        }
        for c in &model.contracts {
            let fns = calling.contracts.entry(c.name.clone()).or_default();
            fns.extend(c.functions.iter().map(|f| f.name.clone()));
        }
        calling
    }
    /// The function a call by `name` alone names, by its path: in a
    /// contract, its own; in a part, its own, else the one function of
    /// that name its uses read. `None` when none, or several.
    pub(crate) fn resolve(
        &self,
        part: Option<&str>,
        contract: Option<&str>,
        name: &str,
    ) -> Option<String> {
        if let Some(c) = contract {
            return self
                .contracts
                .get(c)
                .filter(|fs| fs.contains(name))
                .map(|_| format!("{c}::{name}"));
        }
        let part = part?;
        if self.functions.get(part).is_some_and(|fs| fs.contains(name)) {
            return Some(format!("{part}::{name}"));
        }
        let mut found: BTreeSet<String> = BTreeSet::new();
        for u in self.uses.get(part).into_iter().flatten() {
            let from = u.part();
            if u.reads(from, name) && self.functions.get(from).is_some_and(|fs| fs.contains(name)) {
                found.insert(format!("{from}::{name}"));
            }
        }
        match found.len() {
            1 => found.pop_first(),
            _ => None,
        }
    }
}

impl Context {
    pub(crate) fn of(model: &Model) -> Context {
        let mut declared = BTreeMap::new();
        let mut fields = BTreeMap::new();
        let mut items: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut shapes = Shapes::default();
        let mut params = BTreeMap::new();
        let mut variants: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for part in &model.parts {
            let names = items.entry(part.name.clone()).or_default();
            for e in &part.enums {
                names.insert(e.name.clone());
                if !shapes.enums.contains_key(&e.name) && !fields.contains_key(&e.name) {
                    shapes.enums.insert(e.name.clone(), e.variants.clone());
                    params.insert(e.name.clone(), e.params.clone());
                    for (v, _) in &e.variants {
                        variants.entry(v.clone()).or_default().push(e.name.clone());
                    }
                }
            }
            for r in &part.records {
                names.insert(r.name.clone());
                if !declared.contains_key(&r.name) && !shapes.enums.contains_key(&r.name) {
                    if !r.params.is_empty() {
                        params.insert(r.name.clone(), r.params.clone());
                    }
                    let names = r
                        .fields
                        .as_ref()
                        .map(|fs| fs.iter().map(|(n, _)| n.clone()).collect());
                    declared.insert(r.name.clone(), names);
                    fields.insert(r.name.clone(), r.fields.clone().unwrap_or_default());
                }
            }
            names.extend(part.functions.iter().map(|f| f.name.clone()));
            names.extend(part.requires.iter().map(|q| q.name.clone()));
            names.extend(part.ensures.iter().map(|e| e.name.clone()));
        }
        shapes.fields = fields;
        shapes.params = params.clone();
        let records = shapes
            .fields
            .keys()
            .map(|name| (name.clone(), record_of(name, &shapes, &[])))
            .collect();
        let enums = shapes
            .enums
            .keys()
            .map(|name| (name.clone(), enum_of(name, &shapes, &[])))
            .collect();
        // The signatures with each record and enum written out, so that
        // a call tells what its arguments are (`g(None)`).
        let followed = |calls: simulation::Calls| -> simulation::Calls {
            let callees = calls
                .iter()
                .map(|(path, c)| {
                    let mut c = c.clone();
                    for (_, t) in c.takes.iter_mut() {
                        *t = follow(t, &shapes, &[]);
                    }
                    c.gives = follow(&c.gives, &shapes, &[]);
                    (path.clone(), c)
                })
                .collect();
            simulation::Calls::with(callees).unwrap_or(calls)
        };
        let simulation = Simulation {
            calls: std::sync::Arc::new(followed(crate::model::signatures(model))),
        };
        let contracts = model
            .contracts
            .iter()
            .map(|c| {
                let calls = std::sync::Arc::new(followed(crate::model::contract_calls(c)));
                (c.name.clone(), Simulation { calls })
            })
            .collect();
        Context {
            records,
            enums,
            params,
            variants,
            declared,
            items,
            calling: Calling::of(model),
            simulation,
            contracts,
        }
    }
}

/// The records and enums being followed, outermost first: one met again
/// inside itself stops there, empty (check says M027). Cut at the first
/// repeat, not at a depth: `Node(Tree, Int, Tree)` doubles at each level.
type On<'a> = &'a [String];

/// What the first reading declares: each record's fields, each enum's
/// variants, and the parameters of each that has some, all as written.
#[derive(Default)]
struct Shapes {
    fields: BTreeMap<String, Vec<(String, Type)>>,
    enums: BTreeMap<String, Vec<(String, Vec<Type>)>>,
    params: BTreeMap<String, Vec<(String, simulation::Bound)>>,
}

impl Shapes {
    /// The parameters of `name`, open.
    fn open(&self, name: &str) -> Vec<Type> {
        self.params
            .get(name)
            .into_iter()
            .flatten()
            .map(|(n, b)| Type::Param {
                name: n.clone(),
                bound: *b,
            })
            .collect()
    }
}

/// The type of the record `name`, its fields' types followed, its
/// parameters open.
fn record_of(name: &str, shapes: &Shapes, on: On) -> simulation::Record {
    let inner = [on, &[name.to_string()]].concat();
    let fields = match shapes.fields.get(name) {
        Some(fs) if !on.iter().any(|n| n == name) => fs
            .iter()
            .map(|(n, t)| (n.clone(), follow(t, shapes, &inner)))
            .collect(),
        _ => vec![],
    };
    simulation::Record {
        name: name.to_string(),
        args: shapes.open(name),
        fields,
    }
}

/// The type of the enum `name`, what each variant holds followed, its
/// parameters open. One that holds itself stops there, with no variant.
fn enum_of(name: &str, shapes: &Shapes, on: On) -> simulation::Enum {
    let inner = [on, &[name.to_string()]].concat();
    let variants = match shapes.enums.get(name) {
        Some(vs) if !on.iter().any(|n| n == name) => vs
            .iter()
            .map(|(v, ts)| {
                (
                    v.clone(),
                    ts.iter().map(|t| follow(t, shapes, &inner)).collect(),
                )
            })
            .collect(),
        _ => vec![],
    };
    simulation::Enum {
        name: name.to_string(),
        args: shapes.open(name),
        variants,
    }
}

/// `t`, each record and enum it names by name written out: the first
/// reading cannot tell one from the other, nor their fields.
fn follow(t: &Type, shapes: &Shapes, on: On) -> Type {
    let given = |name: &str, args: &[Type]| -> BTreeMap<String, Type> {
        let params = shapes.params.get(name).cloned().unwrap_or_default();
        params
            .iter()
            .map(|(n, _)| n.clone())
            .zip(args.iter().map(|a| follow(a, shapes, on)))
            .collect()
    };
    match t {
        Type::Record(r) if r.name.is_empty() => Type::Record(simulation::Record {
            name: String::new(),
            args: vec![],
            fields: r
                .fields
                .iter()
                .map(|(n, t)| (n.clone(), follow(t, shapes, on)))
                .collect(),
        }),
        Type::Record(simulation::Record { name, args, .. })
        | Type::Enum(simulation::Enum { name, args, .. }) => {
            let open = match shapes.enums.contains_key(name) {
                true => Type::Enum(enum_of(name, shapes, on)),
                false => Type::Record(record_of(name, shapes, on)),
            };
            open.given(&given(name, args))
        }
        Type::Seq(t) => Type::Seq(Box::new(follow(t, shapes, on))),
        other => other.clone(),
    }
}

/// A parameter a variant could not tell yet, `?T`: settled by where the
/// value goes (`Parser::settle`), else P009.
fn hole(name: &str) -> Type {
    Type::Param {
        name: format!("?{name}"),
        bound: simulation::Bound::Any,
    }
}

/// `t` names a parameter not told yet.
fn holed(t: &Type) -> bool {
    match t {
        Type::Param { name, .. } => name.starts_with('?'),
        Type::Record(r) => r.args.iter().any(holed) || r.fields.iter().any(|(_, t)| holed(t)),
        Type::Enum(e) => e.args.iter().any(holed),
        Type::Seq(t) => holed(t),
        _ => false,
    }
}

/// A value of `v`'s variants and records, each parameter not told taken
/// from `want`, the type it goes to.
pub(crate) fn settle_value(v: Value, want: &Type) -> Value {
    match (v, want) {
        (Value::Variant(e, i, holds), Type::Enum(w))
            if w.name == e.name && w.variants.len() == e.variants.len() =>
        {
            let ts = w
                .variants
                .get(i)
                .map(|(_, ts)| ts.clone())
                .unwrap_or_default();
            let holds = match ts.len() == holds.len() {
                true => holds
                    .into_iter()
                    .zip(&ts)
                    .map(|(v, t)| settle_value(v, t))
                    .collect(),
                false => holds,
            };
            match holed(&Type::Enum(e.clone())) && !holed(want) {
                true => Value::Variant(w.clone(), i, holds),
                false => Value::Variant(e, i, holds),
            }
        }
        (Value::Record(r, vs), Type::Record(w))
            if r.name == w.name && vs.len() == w.fields.len() =>
        {
            let vs = vs
                .into_iter()
                .zip(&w.fields)
                .map(|(v, (_, t))| settle_value(v, t))
                .collect();
            match holed(&Type::Record(r.clone())) && !holed(want) {
                true => Value::Record(w.clone(), vs),
                false => Value::Record(r, vs),
            }
        }
        (Value::Seq(t, holds), Type::Seq(w)) => {
            let holds: Vec<Value> = holds.into_iter().map(|v| settle_value(v, w)).collect();
            match holed(&t) && !holed(w) {
                true => Value::Seq((**w).clone(), holds),
                false => Value::Seq(t, holds),
            }
        }
        (v, _) => v,
    }
}

/// The first variant in `v` whose type is not told, by its name.
fn untold(v: &Value) -> Option<String> {
    match v {
        Value::Variant(e, i, holds) => match holed(&Type::Enum(e.clone())) {
            true => e.variants.get(*i).map(|(n, _)| n.clone()),
            false => holds.iter().find_map(untold),
        },
        Value::Record(r, _) if r.args.iter().any(holed) => Some(r.name.clone()),
        Value::Record(_, vs) => vs.iter().find_map(untold),
        Value::Partial(_, vs) => vs.iter().flatten().find_map(untold),
        Value::Seq(t, vs) => vs
            .iter()
            .find_map(untold)
            .or_else(|| holed(t).then(|| "[]".to_string())),
        _ => None,
    }
}

/// The first variant `e` makes whose type is not told, by its name.
fn untold_in(e: &E) -> Option<String> {
    let Expression::Apply(op, args) = e else {
        return None;
    };
    if let Op::Variant(en, i) = op {
        if holed(&Type::Enum(en.clone())) {
            return en.variants.get(*i).map(|(n, _)| n.clone());
        }
    }
    // A record's own hole, not one only a field's variant has.
    if let Op::Make(r) = op {
        if r.args.iter().any(holed) {
            return Some(r.name.clone());
        }
    }
    let inside = args.iter().find_map(|a| untold_in(&a.body));
    // A sequence of a type nothing tells: `[]`, or `[None]`.
    match op {
        Op::Seq(t) if inside.is_none() && holed(t) => Some("[]".to_string()),
        _ => inside,
    }
}

/// What a variant holds, against what it was given: P007's message.
fn holds_message(e: &simulation::Enum, i: usize, given: usize) -> String {
    let (v, ts) = &e.variants[i];
    let shown: Vec<String> = ts.iter().map(crate::print::type_).collect();
    match ts.is_empty() {
        true => format!("`{v}` of `{}` holds nothing: write `{v}`", e.name),
        false => format!(
            "`{v}` of `{}` holds {} value{}, `{v}({})`, and is given {given}",
            e.name,
            ts.len(),
            if ts.len() == 1 { "" } else { "s" },
            shown.join(", ")
        ),
    }
}

/// The type of a value: a record's is the one it carries.
pub(crate) fn type_of(v: &Value) -> Type {
    match v {
        Value::Truth(_) => Type::Truth,
        Value::Int(_) => Type::Int,
        Value::Real(_) => Type::Real,
        Value::Record(r, _) | Value::Partial(r, _) => Type::Record(r.clone()),
        Value::Variant(e, _, _) => Type::Enum(e.clone()),
        Value::Seq(t, _) => Type::Seq(Box::new(t.clone())),
        Value::Text(_) => Type::Encoded(Box::new(Type::Char)),
        Value::Byte(_) => Type::Byte,
    }
}

// ---------------------------------------------------------------- parser

type E = Expression<Op>;
type J = Expression<judgement::Op>;

/// The names in scope of a simulation expression, the nearest last;
/// `None` when the type of what a `let` binds cannot be told.
type Env = Vec<(String, Option<Type>)>;

fn apply<O>(op: O, args: Vec<Expression<O>>) -> Expression<O> {
    let args = args.into_iter().map(|body| Arg {
        binds: vec![],
        body,
    });
    Expression::Apply(op, args.collect())
}

pub(crate) struct Parser<'c> {
    tokens: Vec<(Tok, Pos)>,
    i: usize,
    /// `None` on the first reading.
    context: Option<&'c Context>,
    /// The part being read.
    part: Option<String>,
    ties: Vec<Tie>,
    /// In the condition of an `if`, as in Rust: `R { … }` must be in
    /// parentheses there.
    no_record: bool,
    /// The contract being read, and its type parameters.
    contract: Option<String>,
    params: Vec<(String, simulation::Bound)>,
    /// Each part a `use` reads, whether through `std::`, and where.
    reads: Vec<Read>,
    /// How deep the text nests here, and how deep it may (P011).
    depth: usize,
    budget: usize,
}

impl<'c> Parser<'c> {
    pub(crate) fn new(tokens: Vec<(Tok, Pos)>, context: Option<&'c Context>) -> Parser<'c> {
        Parser {
            tokens,
            i: 0,
            context,
            part: None,
            ties: vec![],
            no_record: false,
            contract: None,
            params: vec![],
            reads: vec![],
            depth: 0,
            budget: Settings::default().depth,
        }
    }

    /// This parser, nesting at most `depth` levels.
    fn budget(mut self, depth: usize) -> Parser<'c> {
        self.budget = depth;
        self
    }

    /// `f`, one level deeper: past the budget, P011 instead, never a
    /// stack that overflows.
    fn deeper<T>(&mut self, f: impl FnOnce(&mut Self) -> Result<T>) -> Result<T> {
        if self.depth >= self.budget {
            let message = format!(
                "the text nests deeper than {} levels here, the parser's depth budget (`parse::Settings::depth`)",
                self.budget
            );
            return Err(error(P011, self.pos(), message));
        }
        self.depth += 1;
        let r = f(self);
        self.depth -= 1;
        r
    }

    // ------------------------------------------------ tokens

    pub(crate) fn peek(&self) -> &Tok {
        &self.tokens[self.i.min(self.tokens.len() - 1)].0
    }
    fn peek_at(&self, k: usize) -> &Tok {
        &self.tokens[(self.i + k).min(self.tokens.len() - 1)].0
    }
    pub(crate) fn pos(&self) -> Pos {
        self.tokens[self.i.min(self.tokens.len() - 1)].1
    }
    pub(crate) fn bump(&mut self) -> Tok {
        let t = self.peek().clone();
        if self.i < self.tokens.len() - 1 {
            self.i += 1;
        }
        t
    }
    pub(crate) fn is(&self, s: &str) -> bool {
        matches!(self.peek(), Tok::Sym(x) if *x == s)
    }
    pub(crate) fn is_word(&self, w: &str) -> bool {
        matches!(self.peek(), Tok::Word(x) if x == w)
    }
    pub(crate) fn eat(&mut self, s: &str) -> bool {
        let here = self.is(s);
        if here {
            self.bump();
        }
        here
    }
    pub(crate) fn eat_word(&mut self, w: &str) -> bool {
        let here = self.is_word(w);
        if here {
            self.bump();
        }
        here
    }
    pub(crate) fn expected<T>(&self, what: &str) -> Result<T> {
        let found = describe(self.peek());
        Err(error(
            P003,
            self.pos(),
            format!("expected {what}, found {found}"),
        ))
    }
    pub(crate) fn expect(&mut self, s: &str) -> Result<()> {
        match self.eat(s) {
            true => Ok(()),
            false => self.expected(&format!("`{s}`")),
        }
    }
    pub(crate) fn expect_word(&mut self, w: &str) -> Result<()> {
        match self.eat_word(w) {
            true => Ok(()),
            false => self.expected(&format!("`{w}`")),
        }
    }
    pub(crate) fn end(&mut self) -> Result<()> {
        match self.peek() {
            Tok::End => Ok(()),
            _ => self.expected("the end of the text"),
        }
    }
    /// A name: a word that is no keyword.
    pub(crate) fn name(&mut self) -> Result<String> {
        match self.peek().clone() {
            Tok::Word(w) if !KEYWORDS.contains(&w.as_str()) => {
                self.bump();
                Ok(w)
            }
            _ => self.expected("a name"),
        }
    }
    pub(crate) fn string(&mut self) -> Result<String> {
        match self.peek().clone() {
            Tok::Str(s) => {
                self.bump();
                Ok(s)
            }
            _ => self.expected("a string"),
        }
    }
    /// `a, b, c` up to `close`, a comma after the last allowed.
    fn list<T>(
        &mut self,
        close: &str,
        mut item: impl FnMut(&mut Self) -> Result<T>,
    ) -> Result<Vec<T>> {
        let mut items = vec![];
        while !self.is(close) {
            items.push(item(self)?);
            if !self.eat(",") {
                break;
            }
        }
        self.expect(close)?;
        Ok(items)
    }

    // ------------------------------------------------ paths

    /// `a::b::c`, as written; `crate::` first says it is whole.
    pub(crate) fn raw_path(&mut self) -> Result<Vec<String>> {
        let mut names = vec![match self.eat_word("crate") {
            true => "crate".to_string(),
            false => self.name()?,
        }];
        while self.eat("::") {
            names.push(self.name()?);
        }
        if names == ["crate"] {
            return self.expected("`::`");
        }
        Ok(names)
    }
    /// A path as written, made whole: in a part, a path whose first
    /// name is one of the part's records, functions, requires or
    /// ensures goes from the part.
    fn resolve(&self, mut names: Vec<String>) -> Vec<String> {
        if names.first().is_some_and(|n| n == "crate") {
            names.remove(0);
            return names;
        }
        if let (Some(context), Some(part)) = (self.context, &self.part) {
            let local = context
                .items
                .get(part)
                .is_some_and(|items| items.contains(&names[0]));
            if local {
                names.insert(0, part.clone());
            }
        }
        names
    }
    fn path(&mut self) -> Result<Path> {
        let names = self.raw_path()?;
        Ok(Path {
            names: self.resolve(names),
        })
    }
    /// A path in a proof, a name a `let` binds left as it is.
    fn proof_name(&mut self, bound: &[String]) -> Result<String> {
        let names = self.raw_path()?;
        if names.len() == 1 && bound.contains(&names[0]) {
            return Ok(names[0].clone());
        }
        Ok(self.resolve(names).join("::"))
    }

    // ------------------------------------------------ the model

    pub(crate) fn model(self) -> Result<Model> {
        self.model_reads().map(|(m, _)| m)
    }

    /// The model, and each part its uses read, through `std::` or not,
    /// and where.
    fn model_reads(mut self) -> Result<(Model, Vec<Read>)> {
        let (mut parts, mut contracts, mut examples) = (vec![], vec![], vec![]);
        loop {
            match self.peek() {
                Tok::End => break,
                Tok::Word(w) if w == "contract" => contracts.push(self.contract()?),
                Tok::Word(w) if w == "examples" => {
                    self.bump();
                    self.expect("{")?;
                    examples.extend(self.list("}", |p| p.model_example())?);
                }
                Tok::Word(w) if w == "subject" || w == "environment" => parts.push(self.part_()?),
                Tok::Word(w) if w == "tie" => {
                    self.bump();
                    let item = self.path()?;
                    self.expect_word("at")?;
                    let at = location(&self.string()?);
                    self.expect(";")?;
                    self.ties.push(Tie { item, at });
                }
                _ => {
                    return self
                        .expected("`subject`, `environment`, `contract`, `examples` or `tie`")
                }
            }
        }
        let model = Model {
            parts,
            contracts,
            examples,
            ties: self.ties,
        };
        Ok((model, self.reads))
    }

    /// `contract Monotone<T: Number> { fn f(x: T) -> y: T; claim rises(a: T, b: T): …; }`.
    fn contract(&mut self) -> Result<Contract> {
        self.expect_word("contract")?;
        let name = self.name()?;
        let params = self.params_()?;
        self.params = params.clone();
        self.contract = Some(name.clone());
        let result = self.contract_body(name, params);
        self.params = vec![];
        self.contract = None;
        result
    }
    /// `<T, U: Number>`, if there: the type parameters a declaration names.
    fn params_(&mut self) -> Result<Vec<(String, simulation::Bound)>> {
        if !self.eat("<") {
            return Ok(vec![]);
        }
        self.list(">", |p| {
            let n = p.name()?;
            let bound = match p.eat(":") {
                true => {
                    p.expect_word("Number")?;
                    simulation::Bound::Number
                }
                false => simulation::Bound::Any,
            };
            Ok((n, bound))
        })
    }

    /// `enum Option<T> { None, Some(T) }`: Rust's, each variant holding
    /// the values of the types it lists, in parentheses.
    fn enum_(&mut self, part: &Path) -> Result<crate::model::Enum> {
        self.expect_word("enum")?;
        let name = self.name()?;
        let params = self.params_()?;
        self.ties_of(&part.then(&name))?;
        let was = std::mem::replace(&mut self.params, params.clone());
        let variants = self.enum_body();
        self.params = was;
        Ok(crate::model::Enum {
            name,
            params,
            variants: variants?,
        })
    }
    fn enum_body(&mut self) -> Result<Vec<(String, Vec<Type>)>> {
        self.expect("{")?;
        self.list("}", |p| {
            let v = p.name()?;
            let holds = match p.eat("(") {
                true => p.list(")", |p| p.type_())?,
                false => vec![],
            };
            Ok((v, holds))
        })
    }

    fn contract_body(
        &mut self,
        name: String,
        params: Vec<(String, simulation::Bound)>,
    ) -> Result<Contract> {
        self.expect("{")?;
        let (mut functions, mut claims) = (vec![], vec![]);
        while !self.eat("}") {
            if self.eat_word("fn") {
                let f = self.name()?;
                let takes = self.takes()?;
                self.expect("->")?;
                let g = self.name()?;
                self.expect(":")?;
                let gives = (g, self.type_()?);
                self.expect(";")?;
                functions.push(Signature {
                    name: f,
                    takes,
                    gives,
                });
            } else if self.eat_word("claim") {
                let c = self.name()?;
                let over = self.takes()?;
                self.expect(":")?;
                let mut env: Env = over
                    .iter()
                    .map(|(n, t)| (n.clone(), Some(t.clone())))
                    .collect();
                let at = self.pos();
                let holds = self.expr(&mut env)?;
                let holds = self.settled(holds, Some(&Type::Truth), &env, at)?;
                self.expect(";")?;
                claims.push(Law {
                    name: c,
                    over,
                    holds,
                });
            } else {
                return self.expected("`fn`, `claim` or `}`");
            }
        }
        Ok(Contract {
            name,
            params,
            functions,
            claims,
        })
    }
    /// `(x: Real, y: Int)`.
    fn takes(&mut self) -> Result<Vec<(String, Type)>> {
        self.expect("(")?;
        self.list(")", |p| {
            let n = p.name()?;
            p.expect(":")?;
            Ok((n, p.type_()?))
        })
    }

    /// `(sun::light::w = 850.0, stone::speed::h = 5.0) -> stone::speed::v2 = 98.1`;
    /// several outputs in parentheses.
    fn model_example(&mut self) -> Result<ModelExample> {
        let port = |p: &mut Self| -> Result<(Path, Value)> {
            let path = p.path()?;
            p.expect("=")?;
            let at = p.pos();
            let v = p.value()?;
            // The type of the port: what its function takes of that
            // name, else what it gives.
            let want = p.context.and_then(|c| {
                let (port, f) = path.names.split_last()?;
                let callee = c.simulation.calls.get(&f.join("::"))?;
                Some(match callee.takes.iter().find(|(n, _)| n == port) {
                    Some((_, t)) => t.clone(),
                    None => callee.gives.clone(),
                })
            });
            let v = match want {
                Some(t) => p.settled_value(v, &t, at)?,
                None => v,
            };
            Ok((path, v))
        };
        self.expect("(")?;
        let inputs = self.list(")", port)?;
        self.expect("->")?;
        let outputs = match self.eat("(") {
            true => self.list(")", port)?,
            false => vec![port(self)?],
        };
        Ok(ModelExample { inputs, outputs })
    }

    /// `use sun::{light, Ray};`, `use sun::light;` or `use sun::*;`.
    /// `use std::core::Option;` reads the part `core` of Mitra's own
    /// standard model (`std/core.mitra`): three names, the first `std`.
    fn use_(&mut self) -> Result<Use> {
        self.expect_word("use")?;
        let at = self.pos();
        let mut part = self.name()?;
        self.expect("::")?;
        let std = part == "std"
            && matches!(self.peek(), Tok::Word(_))
            && *self.peek_at(1) == Tok::Sym("::");
        if std {
            part = self.name()?;
            self.expect("::")?;
        }
        self.reads.push((part.clone(), std, at));
        let u = if self.eat("*") {
            Use::Glob { part }
        } else if self.eat("{") {
            Use::Names {
                part,
                names: self.list("}", |p| p.name())?,
            }
        } else {
            Use::Names {
                part,
                names: vec![self.name()?],
            }
        };
        self.expect(";")?;
        Ok(u)
    }

    /// `impl Monotone<Real> for speed;`, `impl Pair<Real> for (f, g);`.
    fn implements(&mut self) -> Result<Implements> {
        self.expect_word("impl")?;
        let contract = self.name()?;
        let mut types = vec![];
        if self.eat("<") {
            types = self.list(">", |p| p.type_())?;
        }
        self.expect_word("for")?;
        let functions = match self.eat("(") {
            true => self.list(")", |p| p.name())?,
            false => vec![self.name()?],
        };
        self.expect(";")?;
        Ok(Implements {
            contract,
            types,
            functions,
        })
    }

    /// `at "a/b", "c"`, if there: each a tie of `item`.
    fn ties_of(&mut self, item: &Path) -> Result<()> {
        if self.eat_word("at") {
            loop {
                let at = location(&self.string()?);
                self.ties.push(Tie {
                    item: item.clone(),
                    at,
                });
                if !self.eat(",") {
                    break;
                }
            }
        }
        Ok(())
    }

    fn part_(&mut self) -> Result<Part> {
        let kind = match self.bump() {
            Tok::Word(w) if w == "subject" => Kind::Subject,
            _ => Kind::Environment,
        };
        let name = self.name()?;
        self.part = Some(name.clone());
        let here = Path::to(&[&name]);
        self.ties_of(&here)?;
        self.expect("{")?;
        let mut part = Part {
            name,
            kind,
            uses: vec![],
            records: vec![],
            enums: vec![],
            functions: vec![],
            implements: vec![],
            requires: vec![],
            ensures: vec![],
            file: None,
        };
        while !self.eat("}") {
            match self.peek() {
                Tok::Word(w) if w == "require" => part.requires.push(self.require()?),
                Tok::Word(w) if w == "ensure" => part.ensures.push(self.ensure()?),
                Tok::Word(w) if w == "record" => part.records.push(self.record(&here)?),
                Tok::Word(w) if w == "enum" => part.enums.push(self.enum_(&here)?),
                Tok::Word(w) if w == "fn" => part.functions.push(self.function(&here)?),
                Tok::Word(w) if w == "use" => part.uses.push(self.use_()?),
                Tok::Word(w) if w == "impl" => part.implements.push(self.implements()?),
                _ => {
                    return self.expected(
                        "`use`, `require`, `ensure`, `record`, `enum`, `fn`, `impl` or `}`",
                    )
                }
            }
        }
        self.part = None;
        Ok(part)
    }

    /// `require light: sun::shines;` or `require air: "still air";`.
    fn require(&mut self) -> Result<Require> {
        self.expect_word("require")?;
        let name = self.name()?;
        self.expect(":")?;
        let from = match self.peek() {
            Tok::Str(_) => Source::Prose(self.string()?),
            _ => Source::Ensure(self.path()?),
        };
        self.expect(";")?;
        Ok(Require { name, from })
    }

    /// `ensure lands: "lands slowly" from (a, b) | (c);`.
    fn ensure(&mut self) -> Result<Ensure> {
        self.expect_word("ensure")?;
        let name = self.name()?;
        self.expect(":")?;
        let prose = self.string()?;
        let mut from = vec![];
        if self.eat_word("from") {
            loop {
                self.expect("(")?;
                from.push(self.list(")", |p| p.path())?);
                if !self.eat("|") {
                    break;
                }
            }
        }
        self.expect(";")?;
        Ok(Ensure { name, prose, from })
    }

    /// `record Photon;`, `record Drop { h: Real } where { … }` or
    /// `record Photon where { … }`.
    fn record(&mut self, part: &Path) -> Result<Record> {
        self.expect_word("record")?;
        let name = self.name()?;
        let params = self.params_()?;
        self.ties_of(&part.then(&name))?;
        let was = std::mem::replace(&mut self.params, params.clone());
        let record = self.record_body(name, params);
        self.params = was;
        record
    }
    fn record_body(
        &mut self,
        name: String,
        params: Vec<(String, simulation::Bound)>,
    ) -> Result<Record> {
        let mut record = Record {
            name: name.clone(),
            params,
            fields: None,
            claims: vec![],
            rules: vec![],
        };
        if self.eat(";") {
            return Ok(record);
        }
        if self.eat("{") {
            record.fields = Some(self.list("}", |p| {
                let n = p.name()?;
                p.expect(":")?;
                Ok((n, p.type_()?))
            })?);
        }
        if self.is_word("where") || record.fields.is_none() {
            let self_type = match self.context.and_then(|c| c.records.get(&name)) {
                Some(r) => Type::Record(r.clone()),
                None => Type::Record(simulation::Record {
                    name,
                    args: vec![],
                    fields: vec![],
                }),
            };
            let env = vec![("self".to_string(), Some(self_type))];
            let (claims, rules, examples) = self.where_(&env)?;
            if let Some((_, at)) = examples.first() {
                return Err(error(
                    P003,
                    *at,
                    "a record has no examples: only a function does",
                ));
            }
            record.claims = claims;
            record.rules = rules;
        }
        Ok(record)
    }

    /// `fn speed(h: Real) -> v: Real at "…" { body } where { … }`, or
    /// with no body, or no where, or neither and then `;`.
    fn function(&mut self, part: &Path) -> Result<Function> {
        self.expect_word("fn")?;
        let name = self.name()?;
        self.expect("(")?;
        let mut ranges = simulation::Ranged::default();
        let takes = self.list(")", |p| {
            let n = p.name()?;
            p.expect(":")?;
            let (t, range) = p.type_or_range()?;
            if let Some(range) = range {
                ranges.takes.insert(n.clone(), range);
            }
            Ok((n, t))
        })?;
        self.expect("->")?;
        let gives_name = self.name()?;
        self.expect(":")?;
        let (t, range) = self.type_or_range()?;
        ranges.gives = range.map(|r| (gives_name.clone(), r));
        let gives = (gives_name, t);
        self.ties_of(&part.then(&name))?;
        let mut env: Env = takes
            .iter()
            .map(|(n, t)| (n.clone(), Some(t.clone())))
            .collect();
        let mut body = None;
        if self.eat("{") {
            let at = self.pos();
            let e = self.expr(&mut env)?;
            body = Some(self.settled(e, Some(&gives.1), &env, at)?);
            self.expect("}")?;
        }
        env.push((gives.0.clone(), Some(gives.1.clone())));
        let mut function = Function {
            name,
            takes,
            gives,
            body,
            examples: vec![],
            claims: vec![],
            rules: vec![],
            ranges,
        };
        if self.is_word("where") {
            let (claims, rules, examples) = self.where_(&env)?;
            function.claims = claims;
            function.rules = rules;
            let mut rows = vec![];
            for (e, at) in examples {
                let mut takes = vec![];
                for (v, (_, t)) in e.takes.into_iter().zip(&function.takes) {
                    takes.push(self.settled_value(v, t, at)?);
                }
                let gives = self.settled_value(e.gives, &function.gives.1, at)?;
                rows.push(Example { takes, gives });
            }
            function.examples = rows;
        } else if function.body.is_none() {
            self.expect(";")?;
        }
        Ok(function)
    }

    /// `where { claim …; rule …; examples { … } }`.
    #[allow(clippy::type_complexity)]
    fn where_(&mut self, env: &Env) -> Result<(Vec<Claim>, Vec<Rule>, Vec<(Example, Pos)>)> {
        self.expect_word("where")?;
        self.expect("{")?;
        let (mut claims, mut rules, mut examples) = (vec![], vec![], vec![]);
        while !self.eat("}") {
            if self.eat_word("claim") {
                let name = self.name()?;
                self.expect(":")?;
                let at = self.pos();
                let holds = self.expr(&mut env.clone())?;
                let holds = self.settled(holds, Some(&Type::Truth), env, at)?;
                self.expect(";")?;
                claims.push(Claim { name, holds });
            } else if self.eat_word("rule") {
                rules.push(self.rule(env)?);
            } else if self.is_word("examples") {
                let at = self.pos();
                self.bump();
                self.expect("{")?;
                let rows = self.list("}", |p| p.row())?;
                examples.extend(
                    rows.into_iter()
                        .map(|(takes, gives)| (Example { takes, gives }, at)),
                );
            } else {
                return self.expected("`claim`, `rule`, `examples` or `}`");
            }
        }
        Ok((claims, rules, examples))
    }

    /// `slows: drop::falls, { v < 21.0 } |- drop::slow by …;`, after `rule`.
    fn rule(&mut self, env: &Env) -> Result<Rule> {
        let name = self.name()?;
        self.expect(":")?;
        let mut premises = vec![];
        while !self.is("|-") {
            premises.push(self.term(env)?);
            if !self.eat(",") {
                break;
            }
        }
        self.expect("|-")?;
        let conclusion = self.term(env)?;
        let proof = match self.eat_word("by") {
            true => Some(self.proof(&mut vec![])?),
            false => None,
        };
        self.expect(";")?;
        Ok(Rule {
            name,
            premises,
            conclusion,
            proof,
        })
    }

    /// A path, or `{ expression }` written in place.
    fn term(&mut self, env: &Env) -> Result<Term> {
        if self.eat("{") {
            let at = self.pos();
            let holds = self.expr(&mut env.clone())?;
            let holds = self.settled(holds, Some(&Type::Truth), env, at)?;
            self.expect("}")?;
            return Ok(Term::Holds(holds));
        }
        // A proposition: `all …`, `any …`, `!p`, `correct(f, …)`, `(p)`.
        let prop = self.is_word("all")
            || self.is_word("any")
            || self.is("!")
            || self.is("(")
            || (self.is_word("correct") && *self.peek_at(1) == Tok::Sym("("));
        if prop {
            return Ok(Term::Prop(self.prop()?));
        }
        Ok(Term::At(self.path()?))
    }

    // ------------------------------------------------ types and values

    /// `bool`, `integer`, `rational`, `(A, B)`, `()`, or a record's name.
    /// A type, or a range of whole numbers, `0..=9`, both ends in: an
    /// `Int` that is checked. Rust's form of a range; as a type, Ada's.
    fn type_or_range(&mut self) -> Result<(Type, Option<simulation::Range>)> {
        let starts = matches!(self.peek(), Tok::Int(_))
            || (self.is("-") && matches!(self.peek_at(1), Tok::Int(_)));
        if !starts {
            return Ok((self.type_()?, None));
        }
        let end = |p: &mut Self| -> Result<BigInt> {
            let at = p.pos();
            match p.value()? {
                Value::Int(n) => Ok(n),
                _ => Err(error(P003, at, "a range type's ends are whole numbers")),
            }
        };
        let low = end(self)?;
        self.expect("..")?;
        if !self.eat("=") {
            return self.expected("`=`: a range type is written `0..=9`, both ends in");
        }
        let high = end(self)?;
        Ok((Type::Int, Some(simulation::Range { low, high })))
    }
    fn type_(&mut self) -> Result<Type> {
        self.deeper(|p| p.type_at())
    }
    fn type_at(&mut self) -> Result<Type> {
        // `[T]`, a sequence: Rust's slice form (finding 3, step 2).
        if self.eat("[") {
            // `[~T]`, an encoded sequence (flagged, finding 3, step 3):
            // only text, `[~char]`, today.
            if self.eat("~") {
                let at = self.pos();
                let t = self.type_()?;
                if t != Type::Char {
                    let message = format!(
                        "only `[~char]` has an encoding today, not `[~{}]`",
                        crate::print::type_(&t)
                    );
                    return Err(error(P008, at, message));
                }
                self.expect("]")?;
                return Ok(Type::Encoded(Box::new(t)));
            }
            let t = self.type_()?;
            self.expect("]")?;
            return Ok(Type::Seq(Box::new(t)));
        }
        if self.eat("(") {
            let mut types = vec![];
            while !self.is(")") {
                types.push(self.type_()?);
                if !self.eat(",") {
                    break;
                }
            }
            self.expect(")")?;
            // `(T)` and `(T,)` are `T`: there is no tuple of one value
            // (Swift's rule, flagged: Rust has `(T,)`).
            if types.len() == 1 {
                return Ok(types.pop().unwrap_or(Type::Truth));
            }
            return Ok(simulation::tuple(types));
        }
        let at = self.pos();
        let name = self.name()?;
        if let Some((_, bound)) = self.params.iter().find(|(p, _)| *p == name) {
            return Ok(Type::Param {
                name,
                bound: *bound,
            });
        }
        let args = match self.eat("<") {
            true => self.list(">", |p| p.type_())?,
            false => vec![],
        };
        Ok(match name.as_str() {
            "bool" | "integer" | "rational" | "char" | "byte" if !args.is_empty() => {
                return Err(error(P008, at, format!("`{name}` takes no type arguments")))
            }
            "bool" => Type::Truth,
            "integer" => Type::Int,
            "rational" => Type::Real,
            "char" => Type::Char,
            "byte" => Type::Byte,
            _ => self.instance(&name, args, at)?,
        })
    }

    /// The record or enum `name`, its parameters given `args`: each one
    /// its bound admits, as many as it has (P008). On the first reading,
    /// a stand-in.
    fn instance(&self, name: &str, args: Vec<Type>, at: Pos) -> Result<Type> {
        let Some(context) = self.context else {
            return Ok(Type::Record(simulation::Record {
                name: name.to_string(),
                args,
                fields: vec![],
            }));
        };
        let open = match context.enums.get(name) {
            Some(e) => Type::Enum(e.clone()),
            None => Type::Record(self.record_type(name)),
        };
        let params = context.params.get(name).cloned().unwrap_or_default();
        if params.len() != args.len()
            && (context.enums.contains_key(name) || context.records.contains_key(name))
        {
            let wanted: Vec<&str> = params.iter().map(|(n, _)| n.as_str()).collect();
            let message = match params.is_empty() {
                true => format!("`{name}` takes no type arguments"),
                false => format!(
                    "`{name}` takes {} type argument{}: `{name}<{}>`",
                    params.len(),
                    if params.len() == 1 { "" } else { "s" },
                    wanted.join(", ")
                ),
            };
            return Err(error(P008, at, message));
        }
        let mut given = BTreeMap::new();
        for ((n, bound), t) in params.iter().zip(args) {
            if !bound.admits(&t) {
                let message = format!(
                    "`{name}` takes `{n}: Number`, and `{}` is no number",
                    crate::print::type_(&t)
                );
                return Err(error(P008, at, message));
            }
            given.insert(n.clone(), t);
        }
        Ok(open.given(&given))
    }

    /// The enum and place of the variant `names` names, `Some` or
    /// `Option::Some`: `None` when it names none (P007 when it could name
    /// several).
    fn variant_named(
        &self,
        names: &[String],
        at: Pos,
    ) -> Result<Option<(simulation::Enum, usize)>> {
        let Some(context) = self.context else {
            return Ok(None);
        };
        let (enum_name, v) = match names {
            [e, v] if context.enums.contains_key(e) => (e.clone(), v),
            [v] => match context.variants.get(v).map(Vec::as_slice) {
                Some([e]) => (e.clone(), v),
                Some(several) if several.len() > 1 => {
                    let message = format!(
                        "`{v}` is a variant of {}: write which, `{}::{v}`",
                        several
                            .iter()
                            .map(|e| format!("`{e}`"))
                            .collect::<Vec<_>>()
                            .join(" and "),
                        several[0]
                    );
                    return Err(error(P007, at, message));
                }
                _ => return Ok(None),
            },
            _ => return Ok(None),
        };
        let e = context.enums[&enum_name].clone();
        match e.variant(v) {
            Some(i) => Ok(Some((e, i))),
            None => {
                let message = format!("`{enum_name}` has no variant `{v}`");
                Err(error(P007, at, message))
            }
        }
    }

    /// The enum `open`, each parameter learnt from `holds`, the types of
    /// what variant `i` holds where told; one not told is a hole (`?T`).
    fn told(&self, open: &simulation::Enum, i: usize, holds: &[Option<Type>]) -> simulation::Enum {
        let mut learnt = BTreeMap::new();
        for (t, h) in open.variants[i].1.iter().zip(holds) {
            if let Some(h) = h.as_ref().filter(|h| !holed(h)) {
                let mut more = learnt.clone();
                if t.learn(h, &mut more) {
                    learnt = more;
                }
            }
        }
        for a in &open.args {
            if let Type::Param { name, .. } = a {
                learnt.entry(name.clone()).or_insert_with(|| hole(name));
            }
        }
        match Type::Enum(open.clone()).given(&learnt) {
            Type::Enum(e) => e,
            _ => open.clone(),
        }
    }

    /// The type of the record `name`, as declared; with no field when
    /// nothing declares it.
    fn record_type(&self, name: &str) -> simulation::Record {
        match self.context.and_then(|c| c.records.get(name)) {
            Some(r) => r.clone(),
            None => simulation::Record {
                name: name.to_string(),
                args: vec![],
                fields: vec![],
            },
        }
    }

    /// `(6, 2) -> 3`: what is taken, always in parentheses, and what is given.
    pub(crate) fn row(&mut self) -> Result<(Vec<Value>, Value)> {
        let takes = self.values()?;
        self.expect("->")?;
        Ok((takes, self.value()?))
    }
    /// `(a, b, …)`: a list of values, in parentheses.
    fn values(&mut self) -> Result<Vec<Value>> {
        self.expect("(")?;
        self.list(")", |p| p.value())
    }

    /// `3`, `-0.5`, `1/3`, `true`, `(1, 2.0)`, `Drop { h: 1.0, v: 0.0 }`.
    pub(crate) fn value(&mut self) -> Result<Value> {
        self.deeper(|p| p.value_at())
    }
    fn value_at(&mut self) -> Result<Value> {
        let at = self.pos();
        if self.eat("-") {
            return match self.bump() {
                Tok::Int(n) => Ok(Value::Int(-n)),
                Tok::Real(r) => Ok(Value::Real(Rational::integer(0).minus(&r))),
                _ => Err(error(P003, at, "`-` is written only before a number")),
            };
        }
        match self.peek().clone() {
            Tok::Int(n) => {
                self.bump();
                Ok(Value::Int(n))
            }
            Tok::Real(r) => {
                self.bump();
                Ok(Value::Real(r))
            }
            Tok::Word(w) if w == "true" || w == "false" => {
                self.bump();
                Ok(Value::Truth(w == "true"))
            }
            Tok::Str(s) => {
                self.bump();
                Ok(Value::Text(s))
            }
            // `[1, 2]`, `[]`: of the type of its first value whose type is
            // told, else settled by where it goes (P009).
            Tok::Sym("[") => {
                self.bump();
                let holds = self.list("]", |p| p.value())?;
                let t = holds
                    .iter()
                    .map(type_of)
                    .find(|t| !holed(t))
                    .unwrap_or_else(|| hole("T"));
                let holds = holds.into_iter().map(|v| settle_value(v, &t)).collect();
                Ok(Value::Seq(t, holds))
            }
            Tok::Sym("(") => {
                self.bump();
                let mut values = vec![];
                while !self.is(")") {
                    values.push(self.value()?);
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect(")")?;
                // `(v)` and `(v,)` are `v`: no tuple of one value.
                if values.len() == 1 {
                    return Ok(values.pop().unwrap_or(Value::Truth(true)));
                }
                let Type::Record(r) = simulation::tuple(values.iter().map(type_of).collect())
                else {
                    unreachable!("a tuple is a record")
                };
                Ok(Value::Record(r, values))
            }
            Tok::Word(_) => {
                let names = self.raw_path()?;
                if !self.is("{") {
                    // A variant: `None`, `Some(1)`, `Option::Some(1)`.
                    let holds = match self.eat("(") {
                        true => self.list(")", |p| p.value())?,
                        false => vec![],
                    };
                    return self.variant_value(&names, holds, at);
                }
                let [name] = names.as_slice() else {
                    return Err(error(P003, at, "a record is named by one word"));
                };
                let name = name.clone();
                self.expect("{")?;
                // Its fields, and `..` last when the others are not known:
                // Rust's struct pattern.
                let (mut fields, mut rest) = (vec![], false);
                while !self.eat("}") {
                    if self.eat("..") {
                        rest = true;
                        self.expect("}")?;
                        break;
                    }
                    let n = self.name()?;
                    self.expect(":")?;
                    fields.push((n, self.value()?));
                    if !self.eat(",") {
                        self.expect("}")?;
                        break;
                    }
                }
                if rest {
                    return self.known_in_part(&name, fields, at);
                }
                let (record, values) = self.made(&name, fields, |v| Some(type_of(v)), at)?;
                Ok(Value::Record(record, values))
            }
            _ => self.expected("a value"),
        }
    }

    /// The variant `names` names, holding `holds`; on the first reading a
    /// stand-in.
    fn variant_value(&self, names: &[String], holds: Vec<Value>, at: Pos) -> Result<Value> {
        let shown = names.join("::");
        if self.context.is_none() {
            let e = simulation::Enum {
                name: String::new(),
                args: vec![],
                variants: vec![(shown, holds.iter().map(type_of).collect())],
            };
            return Ok(Value::Variant(e, 0, holds));
        }
        let Some((open, i)) = self.variant_named(names, at)? else {
            return Err(error(
                P007,
                at,
                format!("`{shown}` is no variant of an enum of the model, nor a value"),
            ));
        };
        let wanted = open.variants[i].1.len();
        if wanted != holds.len() {
            return Err(error(P007, at, holds_message(&open, i, holds.len())));
        }
        let types: Vec<Option<Type>> = holds.iter().map(|v| Some(type_of(v))).collect();
        let e = self.told(&open, i, &types);
        Ok(Value::Variant(e, i, holds))
    }

    /// `R { f: v, .. }`: the declared record `R`, known in its fields
    /// `fields`, the others not.
    fn known_in_part(&self, name: &str, fields: Vec<(String, Value)>, at: Pos) -> Result<Value> {
        let Some(context) = self.context else {
            // The first reading: a stand-in, read again with the records.
            let mut known: Vec<(String, Type)> = fields
                .iter()
                .map(|(n, _)| (n.clone(), Type::Truth))
                .collect();
            known.push(("..".to_string(), Type::Truth));
            let r = simulation::Record {
                name: name.to_string(),
                args: vec![],
                fields: known,
            };
            let mut values: Vec<Option<Value>> = fields.into_iter().map(|(_, v)| Some(v)).collect();
            values.push(None);
            return Ok(Value::Partial(r, values));
        };
        let Some(declared) = context.declared.get(name).cloned().flatten() else {
            let message = format!(
                "`{name} {{ .. }}` is known in part: `{name}` must be a record of the model with its fields declared"
            );
            return Err(error(P004, at, message));
        };
        let mut by_name: BTreeMap<String, Value> = BTreeMap::new();
        for (n, v) in fields {
            if !declared.contains(&n) {
                return Err(error(P004, at, format!("`{name}` has no field `{n}`")));
            }
            if by_name.insert(n.clone(), v).is_some() {
                return Err(error(P004, at, format!("field `{n}` is given twice")));
            }
        }
        let values = declared.iter().map(|n| by_name.remove(n)).collect();
        Ok(Value::partial(self.record_type(name), values))
    }

    /// A record made with `fields`, as written. When they are its
    /// declared fields, in any order, it is the declared record, its
    /// fields in their declared order. Else it is a record of that name
    /// with the fields written: `check` says what is wrong with it.
    fn made<X>(
        &self,
        name: &str,
        fields: Vec<(String, X)>,
        type_of: impl Fn(&X) -> Option<Type>,
        at: Pos,
    ) -> Result<(simulation::Record, Vec<X>)> {
        let Some(context) = self.context else {
            let fields_ = fields
                .iter()
                .map(|(n, _)| (n.clone(), Type::Truth))
                .collect();
            let r = simulation::Record {
                name: name.to_string(),
                args: vec![],
                fields: fields_,
            };
            return Ok((r, fields.into_iter().map(|(_, x)| x).collect()));
        };
        if let Some(Some(declared)) = context.declared.get(name) {
            let written: BTreeSet<&String> = fields.iter().map(|(n, _)| n).collect();
            let wanted: BTreeSet<&String> = declared.iter().collect();
            if written == wanted && written.len() == fields.len() {
                let mut by_name: BTreeMap<String, X> = fields.into_iter().collect();
                let xs: Vec<X> = declared.iter().filter_map(|n| by_name.remove(n)).collect();
                let open = self.record_type(name);
                let Some(params) = context.params.get(name).filter(|p| !p.is_empty()) else {
                    return Ok((open, xs));
                };
                // Its parameters, learnt from what its fields are given.
                let mut learnt = BTreeMap::new();
                for ((_, t), x) in open.fields.iter().zip(&xs) {
                    if let Some(u) = type_of(x).filter(|u| !holed(u)) {
                        let mut more = learnt.clone();
                        if t.learn(&u, &mut more) {
                            learnt = more;
                        }
                    }
                }
                // One its fields do not tell is a hole, settled by where the
                // record goes (`Parser::settle`), else P009.
                for (p, _) in params {
                    learnt.entry(p.clone()).or_insert_with(|| hole(p));
                }
                return match Type::Record(open).given(&learnt) {
                    Type::Record(r) => Ok((r, xs)),
                    _ => unreachable!("a record stays a record"),
                };
            }
        }
        let mut types = vec![];
        for (n, x) in &fields {
            let Some(t) = type_of(x) else {
                let message =
                    format!("the type of field `{n}` of this `{name}` cannot be told here");
                return Err(error(P004, at, message));
            };
            types.push((n.clone(), t));
        }
        let r = simulation::Record {
            name: name.to_string(),
            args: vec![],
            fields: types,
        };
        Ok((r, fields.into_iter().map(|(_, x)| x).collect()))
    }

    // ------------------------------------------------ simulation expressions

    /// The type of `e` in `env`: on the first reading, any.
    fn type_in(&self, env: &Env, e: &E) -> Option<Type> {
        self.context?;
        let mut inputs = BTreeMap::new();
        for (n, t) in env {
            match t {
                Some(t) => inputs.insert(n.clone(), t.clone()),
                None => inputs.remove(n),
            };
        }
        let context = self.context?;
        let standard = match &self.contract {
            Some(c) => context.contracts.get(c).cloned().unwrap_or_default(),
            None => context.simulation.clone(),
        };
        typed(&standard, &inputs, e)
    }
    fn type_or_any(&self, env: &Env, e: &E) -> Option<Type> {
        match self.context {
            None => Some(Type::Truth),
            Some(_) => self.type_in(env, e),
        }
    }

    /// `let x = a in b`, or an expression with no `let` around it.
    pub(crate) fn expr(&mut self, env: &mut Env) -> Result<E> {
        self.deeper(|p| p.expr_at(env))
    }
    fn expr_at(&mut self, env: &mut Env) -> Result<E> {
        if self.eat_word("let") {
            let x = self.name()?;
            self.expect("=")?;
            let a = self.expr(env)?;
            self.expect_word("in")?;
            let t = self.type_in(env, &a);
            env.push((x.clone(), t));
            let b = self.expr(env);
            env.pop();
            let args = vec![
                Arg {
                    binds: vec![],
                    body: a,
                },
                Arg {
                    binds: vec![x],
                    body: b?,
                },
            ];
            return Ok(Expression::Apply(Op::Let, args));
        }
        self.or(env)
    }
    fn or(&mut self, env: &mut Env) -> Result<E> {
        let mut e = self.and(env)?;
        while self.eat("||") {
            e = apply(Op::Or, vec![e, self.and(env)?]);
        }
        Ok(e)
    }
    fn and(&mut self, env: &mut Env) -> Result<E> {
        let mut e = self.compare(env)?;
        while self.eat("&&") {
            e = apply(Op::And, vec![e, self.compare(env)?]);
        }
        Ok(e)
    }
    /// One comparison at most, as in Rust: `a < b < c` is refused.
    fn compare(&mut self, env: &mut Env) -> Result<E> {
        let a = self.sum(env)?;
        let op = match self.peek() {
            Tok::Sym(s) if ["==", "!=", "<", "<=", ">", ">="].contains(s) => *s,
            _ => return Ok(a),
        };
        self.bump();
        let b = self.sum(env)?;
        Ok(match op {
            "==" => apply(Op::Equal, vec![a, b]),
            "!=" => apply(Op::Not, vec![apply(Op::Equal, vec![a, b])]),
            "<" => apply(Op::Less, vec![a, b]),
            "<=" => apply(Op::AtMost, vec![a, b]),
            ">" => apply(Op::Less, vec![b, a]),
            _ => apply(Op::AtMost, vec![b, a]),
        })
    }
    fn sum(&mut self, env: &mut Env) -> Result<E> {
        let mut e = self.product(env)?;
        loop {
            let op = match self.peek() {
                Tok::Sym("+") => Op::Add,
                Tok::Sym("-") => Op::Sub,
                Tok::Sym("++") => Op::Join,
                _ => return Ok(e),
            };
            self.bump();
            e = apply(op, vec![e, self.product(env)?]);
        }
    }
    fn product(&mut self, env: &mut Env) -> Result<E> {
        let mut e = self.unary(env)?;
        loop {
            let op = match self.peek() {
                Tok::Sym("*") => Op::Mul,
                Tok::Sym("/") => Op::Div,
                Tok::Sym("%") => Op::Rem,
                _ => return Ok(e),
            };
            self.bump();
            e = apply(op, vec![e, self.unary(env)?]);
        }
    }
    fn unary(&mut self, env: &mut Env) -> Result<E> {
        let at = self.pos();
        if self.eat("!") {
            return Ok(apply(Op::Not, vec![self.deeper(|p| p.unary(env))?]));
        }
        if self.eat("-") {
            // Only a number: there is no negation of an expression.
            return match self.bump() {
                Tok::Int(n) => self.postfix(apply(Op::Int(-n), vec![]), env),
                Tok::Real(r) => {
                    self.postfix(apply(Op::Real(Rational::integer(0).minus(&r)), vec![]), env)
                }
                _ => Err(error(
                    P003,
                    at,
                    "`-` is written only before a number: write `0 - x`",
                )),
            };
        }
        let e = self.primary(env)?;
        self.postfix(e, env)
    }
    /// `.x`, `.0`, `.len()`, `[i]`.
    fn postfix(&mut self, mut e: E, env: &mut Env) -> Result<E> {
        loop {
            if self.eat("[") {
                let i = self.inner(env)?;
                self.expect("]")?;
                e = apply(Op::Index, vec![e, i]);
                continue;
            }
            if !self.is(".") {
                break;
            }
            self.bump();
            if matches!(self.peek(), Tok::Word(w) if w == "len" || w == "decoded")
                && *self.peek_at(1) == Tok::Sym("(")
            {
                let op = match self.bump() {
                    Tok::Word(w) if w == "len" => Op::Len,
                    _ => Op::Decoded,
                };
                self.bump();
                self.expect(")")?;
                e = apply(op, vec![e]);
                continue;
            }
            if matches!(self.peek(), Tok::Word(w) if w == "starts_with")
                && *self.peek_at(1) == Tok::Sym("(")
            {
                self.bump();
                self.bump();
                let b = self.inner(env)?;
                self.expect(")")?;
                e = apply(Op::StartsWith, vec![e, b]);
                continue;
            }
            let field = match self.bump() {
                Tok::Word(w) => w,
                Tok::Int(n) => n.to_string(),
                _ => {
                    self.i -= 1;
                    return self.expected("a field");
                }
            };
            e = apply(Op::Field(field), vec![e]);
        }
        Ok(e)
    }
    fn primary(&mut self, env: &mut Env) -> Result<E> {
        let at = self.pos();
        match self.peek().clone() {
            // `"abc"`: a text, `[~char]` (finding 3, step 3).
            Tok::Str(s) => {
                self.bump();
                Ok(apply(Op::Text(s), vec![]))
            }
            Tok::Int(n) => {
                self.bump();
                Ok(apply(Op::Int(n), vec![]))
            }
            Tok::Real(r) => {
                self.bump();
                Ok(apply(Op::Real(r), vec![]))
            }
            Tok::Word(w) if w == "true" || w == "false" => {
                self.bump();
                Ok(apply(Op::Truth(w == "true"), vec![]))
            }
            Tok::Word(w) if w == "if" => {
                self.bump();
                let was = std::mem::replace(&mut self.no_record, true);
                let c = self.expr(env);
                self.no_record = was;
                let c = c?;
                self.expect("{")?;
                let a = self.inner(env)?;
                self.expect("}")?;
                self.expect_word("else")?;
                let b = match self.is_word("if") {
                    true => self.primary(env)?,
                    false => {
                        self.expect("{")?;
                        let b = self.inner(env)?;
                        self.expect("}")?;
                        b
                    }
                };
                Ok(apply(Op::If, vec![c, a, b]))
            }
            Tok::Word(w) if w == "match" => {
                self.bump();
                self.match_(env, at)
            }
            Tok::Word(w) if w == "within" && *self.peek_at(1) == Tok::Sym("(") => {
                self.bump();
                self.bump();
                let args = self.list(")", |p| p.inner(&mut env.clone()))?;
                if args.len() != 3 {
                    return Err(error(P003, at, "`within(a, b, tolerance)` takes three"));
                }
                Ok(apply(Op::Within, args))
            }
            Tok::Word(w) if w == "let" => {
                self.expected("an expression: put a `let` in parentheses here")
            }
            // `[a, b]`, `[]`: of the type of its first value whose type is
            // told, else settled by where it goes (P009).
            Tok::Sym("[") => {
                self.bump();
                let holds = self.list("]", |p| p.inner(&mut env.clone()))?;
                let t = holds
                    .iter()
                    .find_map(|h| self.type_in(env, h).filter(|t| !holed(t)))
                    .unwrap_or_else(|| hole("T"));
                Ok(apply(Op::Seq(t), holds))
            }
            Tok::Sym("(") => {
                self.bump();
                if self.eat(")") {
                    return self.tuple(vec![], env, at);
                }
                let first = self.inner(env)?;
                if self.eat("..") {
                    let hi = self.inner(env)?;
                    self.expect(")")?;
                    self.expect(".")?;
                    self.expect_word_any("all")?;
                    self.expect("(")?;
                    self.expect("|")?;
                    let x = self.name()?;
                    self.expect("|")?;
                    env.push((x.clone(), Some(Type::Int)));
                    let p = self.inner(env);
                    env.pop();
                    let p = p?;
                    self.expect(")")?;
                    let args = vec![
                        Arg {
                            binds: vec![],
                            body: first,
                        },
                        Arg {
                            binds: vec![],
                            body: hi,
                        },
                        Arg {
                            binds: vec![x],
                            body: p,
                        },
                    ];
                    return Ok(Expression::Apply(Op::All, args));
                }
                if self.eat(")") {
                    return Ok(first);
                }
                self.expect(",")?;
                let mut items = vec![first];
                while !self.is(")") {
                    items.push(self.inner(env)?);
                    if !self.eat(",") {
                        break;
                    }
                }
                self.expect(")")?;
                // `(x,)` is `x`: no tuple of one value.
                if items.len() == 1 {
                    return Ok(items.pop().expect("one"));
                }
                self.tuple(items, env, at)
            }
            Tok::Word(w)
                if w == "crate" || matches!(self.peek_at(1), Tok::Sym("::") | Tok::Sym("(")) =>
            {
                let names = self.raw_path()?;
                let function = self.context.is_some_and(|c| {
                    names.len() > 1
                        || c.calling
                            .resolve(self.part.as_deref(), self.contract.as_deref(), &names[0])
                            .is_some()
                }) && !(names.len() == 2
                    && self
                        .context
                        .is_some_and(|c| c.enums.contains_key(&names[0])));
                if !function {
                    if let Some((open, i)) = self.variant_named(&names, at)? {
                        let holds = match self.eat("(") {
                            true => self.list(")", |p| p.inner(&mut env.clone()))?,
                            false => vec![],
                        };
                        return self.variant_expr(&open, i, holds, env, at);
                    }
                }
                if !self.is("(") && self.context.is_none() {
                    // `Light::Red`: the first reading knows no enum yet.
                    return Ok(Expression::Name(names.join("::")));
                }
                if !self.is("(") {
                    return self.expected("`(`: a path is a call here");
                }
                self.bump();
                let args = self.list(")", |p| p.inner(&mut env.clone()))?;
                Ok(apply(Op::Call(self.call_path(names)), args))
            }
            Tok::Word(_) => {
                let name = self.name()?;
                if self.is("{") && !self.no_record {
                    self.bump();
                    let fields = self.list("}", |p| {
                        let n = p.name()?;
                        p.expect(":")?;
                        Ok((n, p.inner(&mut env.clone())?))
                    })?;
                    let (record, args) =
                        self.made(&name, fields, |e| self.type_or_any(env, e), at)?;
                    return Ok(apply(Op::Make(record), args));
                }
                // A unit variant, `None`, unless a name in scope is so named.
                let bound = env.iter().any(|(n, _)| *n == name);
                if !bound {
                    if let Some((open, i)) = self.variant_named(std::slice::from_ref(&name), at)? {
                        return self.variant_expr(&open, i, vec![], env, at);
                    }
                }
                Ok(Expression::Name(name))
            }
            _ => self.expected("an expression"),
        }
    }
    /// The variant `i` of `open`, holding `holds`, its parameters learnt
    /// from what they give; one not told is settled by where it goes.
    fn variant_expr(
        &self,
        open: &simulation::Enum,
        i: usize,
        holds: Vec<E>,
        env: &Env,
        at: Pos,
    ) -> Result<E> {
        if open.variants[i].1.len() != holds.len() {
            return Err(error(P007, at, holds_message(open, i, holds.len())));
        }
        let types: Vec<Option<Type>> = holds.iter().map(|h| self.type_in(env, h)).collect();
        let e = self.told(open, i, &types);
        Ok(apply(Op::Variant(e, i), holds))
    }

    /// `match e { None => a, Some(x) => b }`, after `match`: one arm per
    /// variant of what `e` gives, in any order, or `_` for those left.
    fn match_(&mut self, env: &mut Env, at: Pos) -> Result<E> {
        let was = std::mem::replace(&mut self.no_record, true);
        let scrutinee = self.expr(env);
        self.no_record = was;
        let scrutinee = scrutinee?;
        let en = match (self.context, self.type_in(env, &scrutinee)) {
            (None, _) => None,
            (Some(_), Some(Type::Enum(e))) if !holed(&Type::Enum(e.clone())) => Some(e),
            (Some(_), Some(t)) => {
                let message = format!(
                    "`match` reads an enum, and this is `{}`",
                    crate::print::type_(&t)
                );
                return Err(error(P003, at, message));
            }
            (Some(_), None) => {
                return Err(error(
                    P004,
                    at,
                    "the type of what `match` reads cannot be told here",
                ))
            }
        };
        self.expect("{")?;
        // Each arm: where, the variant's place (`None` for `_`), the names
        // it binds and its body.
        let mut arms: Vec<(Pos, Option<usize>, Vec<String>, E)> = vec![];
        let mut written: Vec<(String, usize)> = vec![];
        while !self.eat("}") {
            let here = self.pos();
            let (place, binds) = match self.eat_word("_") {
                true => (None, vec![]),
                false => {
                    let names = self.raw_path()?;
                    let binds = match self.eat("(") {
                        true => self.list(")", |p| p.name())?,
                        false => vec![],
                    };
                    match &en {
                        None => {
                            written.push((names.join("::"), binds.len()));
                            (Some(written.len() - 1), binds)
                        }
                        Some(e) => {
                            let v = names.last().cloned().unwrap_or_default();
                            let wrong_enum = names.len() == 2 && names[0] != e.name;
                            let Some(i) = e.variant(&v).filter(|_| names.len() <= 2 && !wrong_enum)
                            else {
                                let message = format!(
                                    "`{}` is no variant of `{}`, which `match` reads here",
                                    names.join("::"),
                                    crate::print::type_(&Type::Enum(e.clone()))
                                );
                                return Err(error(P007, here, message));
                            };
                            if e.variants[i].1.len() != binds.len() {
                                return Err(error(P007, here, holds_message(e, i, binds.len())));
                            }
                            (Some(i), binds)
                        }
                    }
                }
            };
            self.expect("=>")?;
            let types: Vec<Option<Type>> = match (&en, place) {
                (Some(e), Some(i)) => e.variants[i].1.iter().cloned().map(Some).collect(),
                _ => binds.iter().map(|_| None).collect(),
            };
            let depth = env.len();
            env.extend(binds.iter().cloned().zip(types));
            let body = self.inner(env);
            env.truncate(depth);
            arms.push((here, place, binds, body?));
            if !self.eat(",") {
                self.expect("}")?;
                break;
            }
        }
        let plain = |body: E| Arg {
            binds: vec![],
            body,
        };
        let Some(e) = en else {
            // The first reading: a stand-in, read again with the types.
            let e = simulation::Enum {
                name: String::new(),
                args: vec![],
                variants: written
                    .iter()
                    .map(|(n, k)| (n.clone(), vec![Type::Truth; *k]))
                    .collect(),
            };
            let mut args = vec![plain(scrutinee)];
            args.extend(
                arms.into_iter()
                    .map(|(_, _, binds, body)| Arg { binds, body }),
            );
            return Ok(Expression::Apply(Op::Match(e), args));
        };
        let mut by_variant: Vec<Option<Arg<Op>>> = vec![None; e.variants.len()];
        let mut rest: Option<(Pos, E)> = None;
        for (here, place, binds, body) in arms {
            match place {
                Some(i) if by_variant[i].is_some() || rest.is_some() => {
                    let message = format!(
                        "`{}` is matched twice: the second arm is never taken",
                        e.variants[i].0
                    );
                    return Err(error(P006, here, message));
                }
                Some(i) => by_variant[i] = Some(Arg { binds, body }),
                None if rest.is_some() => {
                    return Err(error(
                        P006,
                        here,
                        "`_` is written twice: the second arm is never taken",
                    ));
                }
                None => rest = Some((here, body)),
            }
        }
        let missing: Vec<&str> = e
            .variants
            .iter()
            .zip(&by_variant)
            .filter(|(_, a)| a.is_none())
            .map(|((v, _), _)| v.as_str())
            .collect();
        match (&rest, missing.is_empty()) {
            (None, false) => {
                let message = format!(
                    "`match` misses {}: give each an arm, or `_ =>` for the rest",
                    missing
                        .iter()
                        .map(|v| format!("`{v}`"))
                        .collect::<Vec<_>>()
                        .join(", ")
                );
                return Err(error(P006, at, message));
            }
            (Some((here, _)), true) => {
                return Err(error(
                    P006,
                    *here,
                    "`_` matches nothing: every variant has its arm",
                ));
            }
            _ => {}
        }
        let mut args = vec![plain(scrutinee)];
        for ((_, ts), arm) in e.variants.iter().zip(by_variant) {
            args.push(match (arm, &rest) {
                (Some(arm), _) => arm,
                (None, Some((_, body))) => Arg {
                    binds: vec!["_".to_string(); ts.len()],
                    body: body.clone(),
                },
                (None, None) => unreachable!("a missing variant is an error above"),
            });
        }
        Ok(Expression::Apply(Op::Match(e), args))
    }

    /// `e`, each variant whose parameters it could not tell given them by
    /// where it goes: `want`, the type wanted there, when known.
    fn settle(&self, e: E, want: Option<&Type>, env: &mut Env) -> E {
        let Some(context) = self.context else {
            return e;
        };
        let Expression::Apply(op, args) = e else {
            return e;
        };
        let typed = |p: &Self, env: &Env, e: &E| p.type_in(env, e).filter(|t| !holed(t));
        // Each argument settled by its own wanted type.
        let each =
            |p: &Self, args: Vec<Arg<Op>>, wants: &[Option<Type>], env: &mut Env| -> Vec<Arg<Op>> {
                args.into_iter()
                    .enumerate()
                    .map(|(k, a)| Arg {
                        binds: a.binds,
                        body: p.settle(a.body, wants.get(k).cloned().flatten().as_ref(), env),
                    })
                    .collect()
            };
        // Arguments of one type: each settled alone, then, only those a
        // hole is still in, by the others (settling all twice would take
        // time doubling with each level).
        let alike = |p: &Self, args: Vec<Arg<Op>>, env: &mut Env| -> Vec<Arg<Op>> {
            let args = each(p, args, &[], env);
            if args.iter().all(|a| untold_in(&a.body).is_none()) {
                return args;
            }
            let Some(t) = args.iter().find_map(|a| typed(p, env, &a.body)) else {
                return args;
            };
            args.into_iter()
                .map(|a| match untold_in(&a.body) {
                    Some(_) => Arg {
                        binds: a.binds,
                        body: p.settle(a.body, Some(&t), env),
                    },
                    None => a,
                })
                .collect()
        };
        let truth = Some(Type::Truth);
        match op {
            Op::Variant(en, i) => {
                let en = match want {
                    Some(Type::Enum(w))
                        if holed(&Type::Enum(en.clone()))
                            && !holed(want.expect("some"))
                            && w.name == en.name
                            && w.variants.len() == en.variants.len() =>
                    {
                        w.clone()
                    }
                    _ => en,
                };
                let wants: Vec<Option<Type>> = en.variants[i].1.iter().cloned().map(Some).collect();
                let args = each(self, args, &wants, env);
                let en = match (holed(&Type::Enum(en.clone())), context.enums.get(&en.name)) {
                    (true, Some(open)) => {
                        let types: Vec<Option<Type>> =
                            args.iter().map(|a| typed(self, env, &a.body)).collect();
                        self.told(open, i, &types)
                    }
                    _ => en,
                };
                Expression::Apply(Op::Variant(en, i), args)
            }
            Op::If => {
                let mut it = args.into_iter();
                let (c, a, b) = match (it.next(), it.next(), it.next()) {
                    (Some(c), Some(a), Some(b)) => (c, a, b),
                    _ => return Expression::Apply(Op::If, vec![]),
                };
                let c = Arg {
                    binds: c.binds,
                    body: self.settle(c.body, truth.as_ref(), env),
                };
                let branches = match want {
                    Some(_) => each(self, vec![a, b], &[want.cloned(), want.cloned()], env),
                    None => alike(self, vec![a, b], env),
                };
                Expression::Apply(Op::If, [vec![c], branches].concat())
            }
            Op::Equal
            | Op::Less
            | Op::AtMost
            | Op::Add
            | Op::Sub
            | Op::Mul
            | Op::Div
            | Op::Rem
            | Op::Within
            | Op::StartsWith => {
                let args = alike(self, args, env);
                Expression::Apply(op, args)
            }
            // `a ++ b`: both of the type wanted, else of one type.
            Op::Join => {
                let args = match want {
                    Some(_) => each(self, args, &[want.cloned(), want.cloned()], env),
                    None => alike(self, args, env),
                };
                Expression::Apply(op, args)
            }
            Op::Not | Op::And | Op::Or => {
                let args = each(self, args, &[truth.clone(), truth.clone()], env);
                Expression::Apply(op, args)
            }
            Op::Make(r) => {
                // A record whose parameters its fields did not tell takes
                // them from where it goes; else, from its fields settled.
                let r = match want {
                    Some(Type::Record(w))
                        if r.args.iter().any(holed)
                            && !holed(want.expect("some"))
                            && w.name == r.name
                            && w.fields.len() == r.fields.len() =>
                    {
                        w.clone()
                    }
                    _ => r,
                };
                let wants: Vec<Option<Type>> =
                    r.fields.iter().map(|(_, t)| Some(t.clone())).collect();
                let args = each(self, args, &wants, env);
                let r = match (r.args.iter().any(holed), context.records.get(&r.name)) {
                    (true, Some(open)) => {
                        let mut learnt = BTreeMap::new();
                        for ((_, t), a) in open.fields.iter().zip(&args) {
                            if let Some(u) = typed(self, env, &a.body) {
                                let mut more = learnt.clone();
                                if t.learn(&u, &mut more) {
                                    learnt = more;
                                }
                            }
                        }
                        let params = context.params.get(&r.name).cloned().unwrap_or_default();
                        for (p, _) in &params {
                            learnt.entry(p.clone()).or_insert_with(|| hole(p));
                        }
                        match Type::Record(open.clone()).given(&learnt) {
                            Type::Record(told) => told,
                            _ => r,
                        }
                    }
                    _ => r,
                };
                Expression::Apply(Op::Make(r), args)
            }
            Op::Call(f) => {
                let calls = match &self.contract {
                    Some(c) => context.contracts.get(c).map(|s| s.calls.clone()),
                    None => Some(context.simulation.calls.clone()),
                };
                let wants: Vec<Option<Type>> = calls
                    .and_then(|calls| {
                        calls
                            .get(&f)
                            .map(|c| c.takes.iter().map(|(_, t)| Some(t.clone())).collect())
                    })
                    .unwrap_or_default();
                let args = each(self, args, &wants, env);
                Expression::Apply(Op::Call(f), args)
            }
            Op::Let => {
                let mut it = args.into_iter();
                let (Some(a), Some(b)) = (it.next(), it.next()) else {
                    return Expression::Apply(Op::Let, vec![]);
                };
                let a = Arg {
                    binds: a.binds,
                    body: self.settle(a.body, None, env),
                };
                let t = typed(self, env, &a.body);
                let depth = env.len();
                env.extend(b.binds.iter().cloned().map(|x| (x, t.clone())));
                let body = self.settle(b.body, want, env);
                env.truncate(depth);
                Expression::Apply(
                    Op::Let,
                    vec![
                        a,
                        Arg {
                            binds: b.binds,
                            body,
                        },
                    ],
                )
            }
            Op::All => {
                let mut it = args.into_iter();
                let (Some(lo), Some(hi), Some(p)) = (it.next(), it.next(), it.next()) else {
                    return Expression::Apply(Op::All, vec![]);
                };
                let int = Some(Type::Int);
                let lo = Arg {
                    binds: lo.binds,
                    body: self.settle(lo.body, int.as_ref(), env),
                };
                let hi = Arg {
                    binds: hi.binds,
                    body: self.settle(hi.body, int.as_ref(), env),
                };
                let depth = env.len();
                env.extend(p.binds.iter().cloned().map(|x| (x, int.clone())));
                let body = self.settle(p.body, truth.as_ref(), env);
                env.truncate(depth);
                Expression::Apply(
                    Op::All,
                    vec![
                        lo,
                        hi,
                        Arg {
                            binds: p.binds,
                            body,
                        },
                    ],
                )
            }
            Op::Match(en) => {
                let mut it = args.into_iter();
                let Some(scrutinee) = it.next() else {
                    return Expression::Apply(Op::Match(en), vec![]);
                };
                let scrutinee = Arg {
                    binds: scrutinee.binds,
                    body: self.settle(scrutinee.body, Some(&Type::Enum(en.clone())), env),
                };
                let arm = |p: &Self,
                           k: usize,
                           a: Arg<Op>,
                           want: Option<&Type>,
                           env: &mut Env|
                 -> Arg<Op> {
                    let ts = en
                        .variants
                        .get(k)
                        .map(|(_, ts)| ts.clone())
                        .unwrap_or_default();
                    let depth = env.len();
                    env.extend(a.binds.iter().cloned().zip(ts.into_iter().map(Some)));
                    let body = p.settle(a.body, want, env);
                    env.truncate(depth);
                    Arg {
                        binds: a.binds,
                        body,
                    }
                };
                let mut arms: Vec<Arg<Op>> = it
                    .enumerate()
                    .map(|(k, a)| arm(self, k, a, want, env))
                    .collect();
                if want.is_none() {
                    // The type one arm gives, for the others.
                    let mut known = None;
                    for (k, a) in arms.iter().enumerate() {
                        let ts = en
                            .variants
                            .get(k)
                            .map(|(_, ts)| ts.clone())
                            .unwrap_or_default();
                        let depth = env.len();
                        env.extend(a.binds.iter().cloned().zip(ts.into_iter().map(Some)));
                        known = known.or_else(|| typed(self, env, &a.body));
                        env.truncate(depth);
                    }
                    if let Some(t) = known {
                        arms = arms
                            .into_iter()
                            .enumerate()
                            .map(|(k, a)| arm(self, k, a, Some(&t), env))
                            .collect();
                    }
                }
                Expression::Apply(Op::Match(en), [vec![scrutinee], arms].concat())
            }
            Op::Seq(t) => {
                // Its type from where it goes when its values did not tell
                // it; then each value settled by it.
                let t = match want {
                    Some(Type::Seq(w)) if holed(&t) && !holed(w) => (**w).clone(),
                    _ => t,
                };
                let n = args.len();
                let args = match holed(&t) {
                    false => each(self, args, &vec![Some(t.clone()); n], env),
                    true => alike(self, args, env),
                };
                let t = match holed(&t) {
                    true => args
                        .iter()
                        .find_map(|a| typed(self, env, &a.body))
                        .unwrap_or(t),
                    false => t,
                };
                Expression::Apply(Op::Seq(t), args)
            }
            Op::Index => {
                let wants = [
                    want.map(|w| Type::Seq(Box::new(w.clone()))),
                    Some(Type::Int),
                ];
                let args = each(self, args, &wants, env);
                Expression::Apply(op, args)
            }
            op => {
                let args = each(self, args, &[], env);
                Expression::Apply(op, args)
            }
        }
    }

    /// `e` settled for `want`, P009 at `at` when a variant's type is still
    /// not told.
    fn settled(&self, e: E, want: Option<&Type>, env: &Env, at: Pos) -> Result<E> {
        let e = self.settle(e, want, &mut env.clone());
        match untold_in(&e).filter(|_| self.context.is_some()) {
            Some(v) => Err(error(
                P009,
                at,
                format!("the type of `{v}` cannot be told here: use it where a value of its type is wanted, as a function's result, a field, an argument, or beside `==`"),
            )),
            None => Ok(e),
        }
    }
    /// `v` settled for `want`, P009 at `at` when a variant's type is
    /// still not told.
    fn settled_value(&self, v: Value, want: &Type, at: Pos) -> Result<Value> {
        let v = settle_value(v, want);
        // A value of another kind than `want` is check's to refuse, with
        // the types (M015): its own type is not what is missing.
        let shaped = matches!(
            (&v, want),
            (Value::Seq(..), Type::Seq(_))
                | (Value::Variant(..), Type::Enum(_))
                | (Value::Record(..) | Value::Partial(..), Type::Record(_))
        );
        match untold(&v).filter(|_| self.context.is_some() && shaped) {
            Some(n) => Err(error(
                P009,
                at,
                format!(
                    "the type of `{n}` cannot be told here: nothing says which `{}` it is",
                    crate::print::type_(want)
                ),
            )),
            None => Ok(v),
        }
    }

    /// The path of the function a call names: whole when written with
    /// `::`, else the one a name alone names (`Calling::resolve`), else
    /// as written, which `check` finds names nothing.
    fn call_path(&self, mut names: Vec<String>) -> String {
        if names[0] == "crate" {
            names.remove(0);
            return names.join("::");
        }
        if names.len() > 1 {
            return names.join("::");
        }
        let resolved = self.context.and_then(|c| {
            c.calling
                .resolve(self.part.as_deref(), self.contract.as_deref(), &names[0])
        });
        resolved.unwrap_or_else(|| names[0].clone())
    }
    /// An expression inside brackets of its own, where `R { … }` may be
    /// written again.
    fn inner(&mut self, env: &mut Env) -> Result<E> {
        let was = std::mem::replace(&mut self.no_record, false);
        let e = self.expr(env);
        self.no_record = was;
        e
    }
    /// `all` after `.`: a word that is a keyword elsewhere.
    fn expect_word_any(&mut self, w: &str) -> Result<()> {
        self.expect_word(w)
    }
    fn tuple(&self, items: Vec<E>, env: &Env, at: Pos) -> Result<E> {
        let mut types = vec![];
        for (i, e) in items.iter().enumerate() {
            match self.type_or_any(env, e) {
                Some(t) => types.push(t),
                None => {
                    let message = format!(
                        "the type of item {i} of this tuple cannot be told: it does not check here"
                    );
                    return Err(error(P004, at, message));
                }
            }
        }
        let Type::Record(r) = simulation::tuple(types) else {
            unreachable!("a tuple is a record")
        };
        Ok(apply(Op::Make(r), items))
    }

    // ------------------------------------------------ proofs

    /// A judgement expression: `let x = a in b`, `!a`,
    /// `ponens(r, p, q)`, a premise by its path.
    pub(crate) fn proof(&mut self, bound: &mut Vec<String>) -> Result<J> {
        if self.eat_word("let") {
            let x = self.name()?;
            self.expect("=")?;
            let a = self.deeper(|p| p.proof(bound))?;
            self.expect_word("in")?;
            bound.push(x.clone());
            let b = self.deeper(|p| p.proof(bound));
            bound.pop();
            let args = vec![
                Arg {
                    binds: vec![],
                    body: a,
                },
                Arg {
                    binds: vec![x],
                    body: b?,
                },
            ];
            return Ok(Expression::Apply(judgement::Op::Let, args));
        }
        // `state(…)` and `&&` on states are judgement's own, not text
        // (finding 27): no proof could give an opinion through them.
        self.proof_unary(bound)
    }
    fn proof_unary(&mut self, bound: &mut Vec<String>) -> Result<J> {
        if self.eat("!") {
            return Ok(apply(
                judgement::Op::Not,
                vec![self.deeper(|p| p.proof_unary(bound))?],
            ));
        }
        let at = self.pos();
        match self.peek().clone() {
            Tok::Sym("(") => {
                self.bump();
                let e = self.proof(bound)?;
                self.expect(")")?;
                Ok(e)
            }
            // A premise written in place is read by its place: `0`, `1`, ….
            Tok::Int(n) => {
                self.bump();
                Ok(Expression::Name(n.to_string()))
            }
            Tok::Word(w)
                if PROOF_OPS.contains(&w.as_str())
                    && matches!(self.peek_at(1), Tok::Sym("(") | Tok::Sym("[")) =>
            {
                self.bump();
                self.proof_op(&w, bound, at)
            }
            Tok::Word(_) => Ok(Expression::Name(self.proof_name(bound)?)),
            _ => self.expected("a proof"),
        }
    }
    fn proof_op(&mut self, w: &str, bound: &mut Vec<String>, at: Pos) -> Result<J> {
        use judgement::Op as O;
        // `point(6, 2)`: a point, its values in order.
        if w == "point" {
            return Ok(apply(O::Point(self.values()?), vec![]));
        }
        let params = self.eat("[");
        // `intro[q](|x| p)`: the argument binds the point.
        if (w, params) == ("intro", true) {
            let q = self.prop()?;
            self.expect("]")?;
            self.expect("(")?;
            self.expect("|")?;
            let x = self.name()?;
            self.expect("|")?;
            bound.push(x.clone());
            let body = self.proof(bound);
            bound.pop();
            let body = body?;
            self.expect(")")?;
            let args = vec![Arg {
                binds: vec![x],
                body,
            }];
            return Ok(Expression::Apply(O::Intro { q }, args));
        }
        let op = match (w, params) {
            ("join", false) => Some(O::Join),
            ("ponens", false) => None,
            ("ponens", true) => {
                let t = self.prop()?;
                let post = match self.eat(",") {
                    true => self.number()?,
                    false => 0,
                };
                Some(O::Ponens { t, post })
            }
            ("tollens", true) => {
                let t = self.prop()?;
                self.expect(",")?;
                let post = self.number()?;
                self.expect(",")?;
                let blame = self.number()?;
                Some(O::Tollens { t, post, blame })
            }
            ("reductio", true) => Some(O::Reductio { of: self.prop()? }),
            ("instance" | "witness", true) => {
                let q = self.prop()?;
                Some(match w {
                    "instance" => O::Instance { q },
                    _ => O::Witness { q },
                })
            }
            ("cases", true) => Some(O::Cases { q: self.prop()? }),
            (_, false) => {
                return Err(error(
                    P003,
                    at,
                    format!("`{w}` takes what it is about in `[…]`: `{w}[…](…)`"),
                ));
            }
            (_, true) => {
                return Err(error(P003, at, format!("`{w}` takes no `[…]`")));
            }
        };
        if params {
            self.expect("]")?;
        }
        self.expect("(")?;
        let args = self.list(")", |p| p.proof(bound))?;
        let op = match op {
            Some(op) => op,
            // `ponens(r, p, q)`: the rule is what is first, by its name.
            None => match args.first() {
                Some(Expression::Name(r)) => O::Ponens {
                    t: Prop::Named(r.clone()),
                    post: 0,
                },
                _ => {
                    let message = "`ponens(…)` names its rule first; else write `ponens[rule](…)`";
                    return Err(error(P003, at, message));
                }
            },
        };
        Ok(apply(op, args))
    }
    pub(crate) fn number(&mut self) -> Result<usize> {
        match self.peek().clone() {
            Tok::Int(n) => {
                self.bump();
                n.to_string()
                    .parse()
                    .or_else(|_| self.expected("a small number"))
            }
            _ => self.expected("a number"),
        }
    }

    // ------------------------------------------------ propositions

    /// `all x in (a: 0..10) where b != 0: p`, `any …`, `!p`,
    /// `{p, q} s {r}`, `correct(f, (6, 2))`, `correct(f, x)`, a path.
    pub(crate) fn prop(&mut self) -> Result<Prop> {
        self.deeper(|p| p.prop_at())
    }
    fn prop_at(&mut self) -> Result<Prop> {
        if self.is_word("all") || self.is_word("any") {
            let all = self.is_word("all");
            self.bump();
            let name = self.name()?;
            self.expect_word("in")?;
            let over = self.set()?;
            self.expect(":")?;
            let body = Box::new(self.prop()?);
            return Ok(match all {
                true => Prop::All { name, over, body },
                false => Prop::Any { name, over, body },
            });
        }
        if self.eat("!") {
            return Ok(Prop::Not(Box::new(self.prop_atom()?)));
        }
        self.prop_atom()
    }
    fn prop_atom(&mut self) -> Result<Prop> {
        if self.eat("(") {
            let p = self.prop()?;
            self.expect(")")?;
            return Ok(p);
        }
        if self.eat("!") {
            return Ok(Prop::Not(Box::new(self.prop_atom()?)));
        }
        if self.eat("{") {
            let pre = self.list("}", |p| p.prop())?;
            let subject = self.proof_name(&[])?;
            self.expect("{")?;
            let post = self.list("}", |p| p.prop())?;
            return Ok(Prop::Implies { pre, subject, post });
        }
        if self.is_word("correct") && *self.peek_at(1) == Tok::Sym("(") {
            self.bump();
            self.bump();
            let function = self.proof_name(&[])?;
            self.expect(",")?;
            let at = match self.is("(") {
                true => judgement::Term::Point(self.values()?),
                false => {
                    // `x in (a: 0..10)`: the point an intro binds, any of the set.
                    let name = self.name()?;
                    match self.eat_word("in") {
                        true => judgement::Term::Arbitrary {
                            name,
                            over: self.set()?,
                        },
                        false => judgement::Term::Var(name),
                    }
                }
            };
            self.expect(")")?;
            return Ok(Prop::Correct { function, at });
        }
        if self.is_word("all") || self.is_word("any") {
            return self.prop();
        }
        Ok(Prop::Named(self.proof_name(&[])?))
    }
    /// `(a: 0..10, b: 0..10) where b != 0`: `hi` left out, as in Rust.
    fn set(&mut self) -> Result<Set> {
        self.expect("(")?;
        let ranges = self.list(")", |p| {
            let n = p.name()?;
            p.expect(":")?;
            let lo = p.value()?;
            p.expect("..")?;
            let hi = p.value()?;
            Ok((n, lo, hi))
        })?;
        let such = match self.eat_word("where") {
            true => {
                let mut env: Env = ranges
                    .iter()
                    .map(|(n, lo, _)| (n.clone(), Some(type_of(lo))))
                    .collect();
                Some(self.inner(&mut env)?)
            }
            false => None,
        };
        Ok(Set { ranges, such })
    }
}

/// `"lab/tower/stone"`: each name between `/`. `""` is no name.
pub fn location(text: &str) -> Location {
    let path = match text.is_empty() {
        true => vec![],
        false => text.split('/').map(str::to_string).collect(),
    };
    Location { path }
}

// ---------------------------------------------------------------- files

/// Mitra's own standard model, shipped in the binary: each part by name.
/// Nothing in it is built into the language (finding 3).
pub const STD: &[(&str, &str)] = &[("core", include_str!("../std/core.mitra"))];

/// What the parser may do: how deep a text may nest, in levels of
/// expressions, types, values and propositions inside each other. Past
/// it, P011, not a stack that overflows: a level takes a few KiB of
/// stack, more in a debug build (the CLI reads on a thread of 256 MiB).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Settings {
    pub depth: usize,
}

impl Default for Settings {
    /// Twice as deep as the core computes through calls
    /// (`simulation::Budget::depth`, 1,000): a body of 1,000 levels and
    /// what holds it parse.
    fn default() -> Settings {
        Settings {
            depth: 2 * crate::simulation::Budget::default().depth,
        }
    }
}

/// A part a `use` reads, whether through `std::`, and where.
type Read = (String, bool, Pos);

/// What reads the file `<part>.mitra` beside a model, if it is there.
pub type ReadFile<'a> = &'a dyn Fn(&str) -> Option<String>;

/// A file still to read: its name (`None` for the model's own), its
/// text, and the part it was read for, where, and by which file.
type ToRead = (
    Option<String>,
    String,
    Option<(String, Pos, Option<String>)>,
);

/// A parse error in one of the files of a model: which, and its text,
/// to render it (`ParseError::render`).
#[derive(Clone, Debug, PartialEq)]
pub struct Loaded {
    pub file: String,
    pub source: String,
    pub error: ParseError,
}

/// A model from its text and the files it reads: each part a `use`
/// names that no file read so far declares is read from `<part>.mitra`
/// by `read` (beside the model, for the CLI), and each `use std::…`
/// from Mitra's own standard model. A part no file declares is P010.
pub fn load(
    source: &str,
    read: ReadFile,
    settings: &Settings,
) -> std::result::Result<Model, Loaded> {
    files(source, Some(read), settings)
}

/// `load`, with `files` a reader of `<part>.mitra`, or none (`parse`):
/// then a part no file declares is left to `check`.
fn files(
    source: &str,
    read: Option<ReadFile>,
    settings: &Settings,
) -> std::result::Result<Model, Loaded> {
    // Each file: its name (`None` for the model itself), its text, its
    // tokens and its first reading.
    struct File {
        name: Option<String>,
        source: String,
        tokens: Vec<(Tok, Pos)>,
        first: Model,
    }
    let failed = |name: &Option<String>, source: &str, error: ParseError| Loaded {
        file: name.clone().unwrap_or_default(),
        source: source.to_string(),
        error,
    };
    let mut done: Vec<File> = Vec::new();
    let mut todo: Vec<ToRead> = vec![(None, source.to_string(), None)];
    while let Some((name, text, wanted)) = todo.pop() {
        let tokens = lex(&text).map_err(|e| failed(&name, &text, e))?;
        let (first, reads) = Parser::new(tokens.clone(), None)
            .budget(settings.depth)
            .model_reads()
            .map_err(|e| failed(&name, &text, e))?;
        // A file read for a part declares it.
        if let Some((part, at, by)) = wanted {
            if !first.parts.iter().any(|p| p.name == part) {
                let message = format!(
                    "`{}` declares no part `{part}`: a part read from a file is declared in it",
                    name.clone().unwrap_or_default()
                );
                let by_text = done
                    .iter()
                    .find(|f| f.name == by)
                    .map(|f| f.source.clone())
                    .unwrap_or_default();
                return Err(failed(&by, &by_text, error(P010, at, message)));
            }
        }
        done.push(File {
            name: name.clone(),
            source: text.clone(),
            tokens,
            first,
        });
        for (part, std, at) in reads {
            let declared = |files: &[File]| {
                files
                    .iter()
                    .any(|f| f.first.parts.iter().any(|p| p.name == part))
            };
            let queued = todo
                .iter()
                .any(|(_, _, w)| w.as_ref().is_some_and(|(p, _, _)| *p == part));
            if declared(&done) || queued {
                continue;
            }
            let found = match std {
                true => STD
                    .iter()
                    .find(|(n, _)| *n == part)
                    .map(|(n, t)| (format!("std/{n}.mitra"), t.to_string())),
                false => match read {
                    Some(read) => {
                        read(&format!("{part}.mitra")).map(|t| (format!("{part}.mitra"), t))
                    }
                    None => continue,
                },
            };
            match found {
                Some((file, t)) => todo.push((Some(file), t, Some((part, at, name.clone())))),
                None => {
                    let message = match std {
                        true => format!(
                            "no part `std::{part}`: Mitra's standard model has {}",
                            STD.iter().map(|(n, _)| format!("`std::{n}`")).collect::<Vec<_>>().join(", ")
                        ),
                        false => format!(
                            "no part `{part}`: no file of the model declares it, and there is no `{part}.mitra` beside it"
                        ),
                    };
                    return Err(failed(&name, &text, error(P010, at, message)));
                }
            }
        }
    }
    // One reading of them all, then each file read again with it.
    let mut all = Model {
        parts: vec![],
        contracts: vec![],
        examples: vec![],
        ties: vec![],
    };
    for f in &done {
        all.parts.extend(f.first.parts.iter().cloned());
        all.contracts.extend(f.first.contracts.iter().cloned());
        all.examples.extend(f.first.examples.iter().cloned());
        all.ties.extend(f.first.ties.iter().cloned());
    }
    let context = Context::of(&all);
    let mut model = all;
    model.parts.clear();
    model.contracts.clear();
    model.examples.clear();
    model.ties.clear();
    for f in done {
        let read = Parser::new(f.tokens, Some(&context))
            .budget(settings.depth)
            .model()
            .map_err(|e| failed(&f.name, &f.source, e))?;
        let mut parts = read.parts;
        for p in parts.iter_mut() {
            p.file = f.name.clone();
        }
        model.parts.extend(parts);
        model.contracts.extend(read.contracts);
        model.examples.extend(read.examples);
        model.ties.extend(read.ties);
    }
    Ok(model)
}
