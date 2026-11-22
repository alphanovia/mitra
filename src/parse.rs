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

use crate::catalogue::{Code, P001, P002, P003, P004, P005};
use crate::expression::{check as typed, Arg, Expression};
use crate::judgement::{self, Prop, Set};
use crate::location::Location;
use crate::model::{
    Claim, Ensure, Example, Function, Kind, Model, Part, Path, Record, Require, Rule, Source, Term,
    Tie,
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
pub fn parse(source: &str) -> Result<Model> {
    let tokens = lex(source)?;
    let first = Parser::new(tokens.clone(), None).model()?;
    let context = Context::of(&first);
    Parser::new(tokens, Some(&context)).model()
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
];

/// The operations of a proof, by the word that writes them. Each is
/// an operation only when `(` or `[` follows it.
const PROOF_OPS: &[&str] = &[
    "join", "state", "ponens", "tollens", "reductio", "instance", "witness", "cases", "intro",
    "point",
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
    "::", "->", "|-", "==", "!=", "<=", ">=", "&&", "||", "..", "{", "}", "(", ")", "[", "]", ",",
    ";", ":", "=", "<", ">", "+", "-", "*", "/", "!", ".", "|",
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
    declared: BTreeMap<String, Option<Vec<String>>>,
    items: BTreeMap<String, BTreeSet<String>>,
}

impl Context {
    pub(crate) fn of(model: &Model) -> Context {
        let mut declared = BTreeMap::new();
        let mut fields = BTreeMap::new();
        let mut items: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        for part in &model.parts {
            let names = items.entry(part.name.clone()).or_default();
            for r in &part.records {
                names.insert(r.name.clone());
                if !declared.contains_key(&r.name) {
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
        let records = fields
            .keys()
            .map(|name| (name.clone(), record_of(name, &fields, 0)))
            .collect();
        Context {
            records,
            declared,
            items,
        }
    }
}

/// How deep a record's fields are followed: a record that holds itself
/// stops there, with no field.
const DEPTH: usize = 32;

/// The type of the record `name`, its fields' types followed.
fn record_of(
    name: &str,
    fields: &BTreeMap<String, Vec<(String, Type)>>,
    depth: usize,
) -> simulation::Record {
    let fields = match fields.get(name) {
        Some(fs) if depth < DEPTH => fs
            .iter()
            .map(|(n, t)| (n.clone(), follow(t, fields, depth + 1)))
            .collect(),
        _ => vec![],
    };
    simulation::Record {
        name: name.to_string(),
        fields,
    }
}

fn follow(t: &Type, fields: &BTreeMap<String, Vec<(String, Type)>>, depth: usize) -> Type {
    match t {
        Type::Record(r) if r.name.is_empty() => Type::Record(simulation::Record {
            name: String::new(),
            fields: r
                .fields
                .iter()
                .map(|(n, t)| (n.clone(), follow(t, fields, depth)))
                .collect(),
        }),
        Type::Record(r) => Type::Record(record_of(&r.name, fields, depth)),
        other => other.clone(),
    }
}

/// The type of a value: a record's is the one it carries.
pub(crate) fn type_of(v: &Value) -> Type {
    match v {
        Value::Truth(_) => Type::Truth,
        Value::Int(_) => Type::Int,
        Value::Real(_) => Type::Real,
        Value::Record(r, _) => Type::Record(r.clone()),
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
        }
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

    pub(crate) fn model(mut self) -> Result<Model> {
        let mut parts = vec![];
        loop {
            match self.peek() {
                Tok::End => break,
                Tok::Word(w) if w == "subject" || w == "environment" => parts.push(self.part_()?),
                Tok::Word(w) if w == "tie" => {
                    self.bump();
                    let item = self.path()?;
                    self.expect_word("at")?;
                    let at = location(&self.string()?);
                    self.expect(";")?;
                    self.ties.push(Tie { item, at });
                }
                _ => return self.expected("`subject`, `environment` or `tie`"),
            }
        }
        Ok(Model {
            parts,
            ties: self.ties,
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
            records: vec![],
            functions: vec![],
            requires: vec![],
            ensures: vec![],
        };
        while !self.eat("}") {
            match self.peek() {
                Tok::Word(w) if w == "require" => part.requires.push(self.require()?),
                Tok::Word(w) if w == "ensure" => part.ensures.push(self.ensure()?),
                Tok::Word(w) if w == "record" => part.records.push(self.record(&here)?),
                Tok::Word(w) if w == "fn" => part.functions.push(self.function(&here)?),
                _ => return self.expected("`require`, `ensure`, `record`, `fn` or `}`"),
            }
        }
        self.part = None;
        Ok(part)
    }

    /// `require light = sun::shines;` or `require air = "still air";`.
    fn require(&mut self) -> Result<Require> {
        self.expect_word("require")?;
        let name = self.name()?;
        self.expect("=")?;
        let from = match self.peek() {
            Tok::Str(_) => Source::Prose(self.string()?),
            _ => Source::Ensure(self.path()?),
        };
        self.expect(";")?;
        Ok(Require { name, from })
    }

    /// `ensure lands = "lands slowly" from (a, b) | (c);`.
    fn ensure(&mut self) -> Result<Ensure> {
        self.expect_word("ensure")?;
        let name = self.name()?;
        self.expect("=")?;
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
        self.ties_of(&part.then(&name))?;
        let mut record = Record {
            name: name.clone(),
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
        let takes = self.list(")", |p| {
            let n = p.name()?;
            p.expect(":")?;
            Ok((n, p.type_()?))
        })?;
        self.expect("->")?;
        let gives_name = self.name()?;
        self.expect(":")?;
        let gives = (gives_name, self.type_()?);
        self.ties_of(&part.then(&name))?;
        let mut env: Env = takes
            .iter()
            .map(|(n, t)| (n.clone(), Some(t.clone())))
            .collect();
        let mut body = None;
        if self.eat("{") {
            body = Some(self.expr(&mut env)?);
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
        };
        if self.is_word("where") {
            let (claims, rules, examples) = self.where_(&env)?;
            function.claims = claims;
            function.rules = rules;
            function.examples = examples.into_iter().map(|(e, _)| e).collect();
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
                self.expect("=")?;
                let holds = self.expr(&mut env.clone())?;
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
            let holds = self.expr(&mut env.clone())?;
            self.expect("}")?;
            return Ok(Term::Holds(holds));
        }
        Ok(Term::At(self.path()?))
    }

    // ------------------------------------------------ types and values

    /// `bool`, `Int`, `Real`, `(A, B)`, `()`, or a record's name.
    fn type_(&mut self) -> Result<Type> {
        if self.eat("(") {
            let mut types = vec![];
            let mut comma = false;
            while !self.is(")") {
                types.push(self.type_()?);
                comma = self.eat(",");
                if !comma {
                    break;
                }
            }
            self.expect(")")?;
            // `(T)` is `T`, as in Rust; `(T,)` is a tuple of one.
            if types.len() == 1 && !comma {
                return Ok(types.pop().unwrap_or(Type::Truth));
            }
            return Ok(simulation::tuple(types));
        }
        let name = self.name()?;
        Ok(match name.as_str() {
            "bool" => Type::Truth,
            "Int" => Type::Int,
            "Real" => Type::Real,
            _ => Type::Record(self.record_type(&name)),
        })
    }

    /// The type of the record `name`, as declared; with no field when
    /// nothing declares it.
    fn record_type(&self, name: &str) -> simulation::Record {
        match self.context.and_then(|c| c.records.get(name)) {
            Some(r) => r.clone(),
            None => simulation::Record {
                name: name.to_string(),
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
            Tok::Sym("(") => {
                self.bump();
                let mut values = vec![];
                let mut comma = false;
                while !self.is(")") {
                    values.push(self.value()?);
                    comma = self.eat(",");
                    if !comma {
                        break;
                    }
                }
                self.expect(")")?;
                if values.len() == 1 && !comma {
                    return Ok(values.pop().unwrap_or(Value::Truth(true)));
                }
                let Type::Record(r) = simulation::tuple(values.iter().map(type_of).collect())
                else {
                    unreachable!("a tuple is a record")
                };
                Ok(Value::Record(r, values))
            }
            Tok::Word(_) => {
                let name = self.name()?;
                self.expect("{")?;
                let fields = self.list("}", |p| {
                    let n = p.name()?;
                    p.expect(":")?;
                    Ok((n, p.value()?))
                })?;
                let (record, values) = self.made(&name, fields, |v| Some(type_of(v)), at)?;
                Ok(Value::Record(record, values))
            }
            _ => self.expected("a value"),
        }
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
                fields: fields_,
            };
            return Ok((r, fields.into_iter().map(|(_, x)| x).collect()));
        };
        if let Some(Some(declared)) = context.declared.get(name) {
            let written: BTreeSet<&String> = fields.iter().map(|(n, _)| n).collect();
            let wanted: BTreeSet<&String> = declared.iter().collect();
            if written == wanted && written.len() == fields.len() {
                let mut by_name: BTreeMap<String, X> = fields.into_iter().collect();
                let xs = declared.iter().filter_map(|n| by_name.remove(n)).collect();
                return Ok((self.record_type(name), xs));
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
        typed(&Simulation, &inputs, e)
    }
    fn type_or_any(&self, env: &Env, e: &E) -> Option<Type> {
        match self.context {
            None => Some(Type::Truth),
            Some(_) => self.type_in(env, e),
        }
    }

    /// `let x = a in b`, or an expression with no `let` around it.
    pub(crate) fn expr(&mut self, env: &mut Env) -> Result<E> {
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
                _ => return Ok(e),
            };
            self.bump();
            e = apply(op, vec![e, self.unary(env)?]);
        }
    }
    fn unary(&mut self, env: &mut Env) -> Result<E> {
        let at = self.pos();
        if self.eat("!") {
            return Ok(apply(Op::Not, vec![self.unary(env)?]));
        }
        if self.eat("-") {
            // Only a number: there is no negation of an expression.
            return match self.bump() {
                Tok::Int(n) => self.postfix(apply(Op::Int(-n), vec![])),
                Tok::Real(r) => {
                    self.postfix(apply(Op::Real(Rational::integer(0).minus(&r)), vec![]))
                }
                _ => Err(error(
                    P003,
                    at,
                    "`-` is written only before a number: write `0 - x`",
                )),
            };
        }
        let e = self.primary(env)?;
        self.postfix(e)
    }
    /// `.x`, `.0`.
    fn postfix(&mut self, mut e: E) -> Result<E> {
        while self.is(".") {
            self.bump();
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
                self.tuple(items, env, at)
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
                Ok(Expression::Name(name))
            }
            _ => self.expected("an expression"),
        }
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

    /// A judgement expression: `let x = a in b`, `a && b`, `!a`,
    /// `ponens(r, p, q)`, a premise by its path.
    pub(crate) fn proof(&mut self, bound: &mut Vec<String>) -> Result<J> {
        if self.eat_word("let") {
            let x = self.name()?;
            self.expect("=")?;
            let a = self.proof(bound)?;
            self.expect_word("in")?;
            bound.push(x.clone());
            let b = self.proof(bound);
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
        let mut e = self.proof_unary(bound)?;
        while self.eat("&&") {
            e = apply(judgement::Op::And, vec![e, self.proof_unary(bound)?]);
        }
        Ok(e)
    }
    fn proof_unary(&mut self, bound: &mut Vec<String>) -> Result<J> {
        if self.eat("!") {
            return Ok(apply(judgement::Op::Not, vec![self.proof_unary(bound)?]));
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
            ("state", false) => Some(O::StateOf),
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
