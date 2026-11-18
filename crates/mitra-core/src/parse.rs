//! Text projection: a small, fixed syntax inspired by Lean 4, written in
//! ASCII (`forall`, `in`, `not`, `|`, `&`), with Lean's Unicode symbols
//! (`∀ ∈ ¬ ∪ ∩`) accepted as aliases. Every command starts with a keyword and
//! layout is free. `--` starts a comment. The grammar is LL(1): one token of
//! lookahead decides every step. See docs/reference/language.md.

use crate::ast::*;
use crate::set::{Set, Value, MAP, SET};
use crate::Error;

#[derive(Debug, Clone, PartialEq)]
enum Tok {
    /// A name or a keyword.
    Ident(String),
    Num(String),
    Str(String),
    Punct(&'static str),
}

/// Longest first, so `:=` wins over `:`, `||` over `|` and `..=` over `..`.
const PUNCT: [&str; 38] = [
    "@[", "..=", ":=", "=>", "->", "==", "!=", "<=", ">=", "&&", "||", "..", ".", "(", ")", ":",
    ";", ",", "=", "{", "}", "[", "]", "|", "&", "!", "<", ">", "+", "-", "*", "/", "\\", "∀", "∈",
    "∪", "∩", "¬",
];

/// Reserved words: never names. Kept in step with the reference grammar.
pub const KEYWORDS: [&str; 47] = [
    "Set",
    "Map",
    "keys",
    "enum",
    "match",
    "rate",
    "precision",
    "recall",
    "false_positive_rate",
    "List",
    "len",
    "record",
    "opaque",
    "def",
    "claim",
    "test",
    "evidence",
    "correct",
    "at",
    "by",
    "derive",
    "if",
    "then",
    "else",
    "let",
    "true",
    "false",
    "forall",
    "in",
    "not",
    "Bool",
    "Int",
    "Int8",
    "Int16",
    "Int32",
    "Int64",
    "UInt8",
    "UInt16",
    "UInt32",
    "Float64",
    "String",
    "Value",
    "_",
    "Real",
    "uniform",
    "uses",
    "only",
];

/// The whole text as tokens, each with its line.
fn lex(src: &str) -> Result<Vec<(Tok, usize)>, Error> {
    let mut toks = Vec::new();
    let mut line = 1;
    let mut rest = src;
    while let Some(c) = rest.chars().next() {
        if c == '\n' {
            line += 1;
        }
        if c.is_whitespace() {
            rest = &rest[c.len_utf8()..];
        } else if rest.starts_with("--") {
            rest = rest.find('\n').map_or("", |n| &rest[n..]);
        } else if c == '"' {
            // `\n`, `\t`, `\"` and `\\` are the escapes.
            let (mut s, mut end, mut chars) = (String::new(), None, rest[1..].char_indices());
            while let Some((i, c)) = chars.next() {
                match c {
                    '"' => {
                        end = Some(i);
                        break;
                    }
                    '\n' => break,
                    '\\' => s.push(match chars.next() {
                        Some((_, 'n')) => '\n',
                        Some((_, 't')) => '\t',
                        Some((_, '"')) => '"',
                        Some((_, '\\')) => '\\',
                        _ => {
                            let msg = "in a string, `\\` starts `\\n`, `\\t`, `\\\"` or `\\\\`";
                            return Err(Error::new(line, msg));
                        }
                    }),
                    c => s.push(c),
                }
            }
            let Some(end) = end else {
                return Err(Error::new(line, "unterminated string"));
            };
            toks.push((Tok::Str(s), line));
            rest = &rest[end + 2..];
        } else if c.is_ascii_digit() {
            let digits = |s: &str| s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
            let mut end = digits(rest);
            if let Some(frac) = rest[end..].strip_prefix('.').map(digits).filter(|&f| f > 0) {
                end += 1 + frac;
            }
            toks.push((Tok::Num(rest[..end].into()), line));
            rest = &rest[end..];
        } else if c.is_ascii_alphabetic() || c == '_' {
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            toks.push((Tok::Ident(rest[..end].into()), line));
            rest = &rest[end..];
        } else if let Some(p) = PUNCT.iter().find(|p| rest.starts_with(**p)) {
            toks.push((Tok::Punct(p), line));
            rest = &rest[p.len()..];
        } else {
            return Err(Error::new(line, format!("unexpected character `{c}`")));
        }
    }
    Ok(toks)
}

struct P {
    toks: Vec<(Tok, usize)>,
    pos: usize,
    /// The records and enums declared so far.
    records: Vec<Record>,
    enums: Vec<Enum>,
    /// The sets the parameters of an enum stand for, inside its body.
    params: Vec<(String, Set)>,
    /// Sets written with an enum, by their printed form.
    aliases: Vec<(String, String)>,
    /// The names used as a record with a set for each field.
    used: Vec<String>,
    /// Whether the last `args` were written with parentheses: `f()`.
    parens: bool,
}

/// A name bound by `forall`, its set if one is given, and its line.
type Bound = (String, Option<Set>, usize);

/// A token as written.
fn text(t: &Tok) -> String {
    match t {
        Tok::Ident(s) | Tok::Num(s) => s.clone(),
        Tok::Str(s) => crate::set::quote(s),
        Tok::Punct(p) => p.to_string(),
    }
}

/// Tokens as one line of text, spaced as a person would.
fn join(toks: &[Tok]) -> String {
    let mut out = String::new();
    for (i, t) in toks.iter().enumerate() {
        let t = text(t);
        let tight_before = [",", ")", ":", "]", "}"].contains(&t.as_str());
        let after_open = i > 0 && matches!(&toks[i - 1], Tok::Punct("(" | "[" | "{"));
        let call = t == "(" && i > 0 && matches!(&toks[i - 1], Tok::Ident(_));
        // In a set, `-` is always a sign.
        let signed = i > 0 && matches!(&toks[i - 1], Tok::Punct("-"));
        if i > 0 && !tight_before && !after_open && !call && !signed {
            out += " ";
        }
        out += &t;
    }
    out
}

impl P {
    fn new(src: &str) -> Result<P, Error> {
        Ok(P {
            toks: lex(src)?,
            pos: 0,
            records: Vec::new(),
            enums: Vec::new(),
            params: Vec::new(),
            aliases: Vec::new(),
            used: Vec::new(),
            parens: false,
        })
    }
    /// The line of the next token (or of the last one, at the end).
    fn line(&self) -> usize {
        self.toks
            .get(self.pos)
            .or(self.toks.last())
            .map_or(1, |t| t.1)
    }
    fn err<T>(&self, m: impl Into<String>) -> Result<T, Error> {
        Err(Error::new(self.line(), m))
    }
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos).map(|t| &t.0)
    }
    fn next(&mut self) -> Option<Tok> {
        let t = self.peek().cloned();
        self.pos += 1;
        t
    }
    fn at_end(&self) -> bool {
        self.pos >= self.toks.len()
    }
    fn is(&self, p: &str) -> bool {
        matches!(self.peek(), Some(Tok::Punct(q)) if *q == p)
    }
    fn eat(&mut self, p: &str) -> bool {
        let yes = self.is(p);
        self.pos += yes as usize;
        yes
    }
    fn expect(&mut self, p: &str) -> Result<(), Error> {
        if self.eat(p) {
            Ok(())
        } else {
            self.err(format!("expected `{p}`"))
        }
    }
    fn is_name(&self) -> bool {
        matches!(self.peek(), Some(Tok::Ident(s)) if !KEYWORDS.contains(&s.as_str()))
    }
    fn ident(&mut self) -> Result<String, Error> {
        match self.next() {
            Some(Tok::Ident(s)) if !KEYWORDS.contains(&s.as_str()) => Ok(s),
            Some(Tok::Ident(s)) if s == "_" => {
                self.pos -= 1;
                self.err(
                    "`_` is not a wildcard, a name or a value is expected\n  \
                     help: for every input, bind a name: `forall x, correct(f(x))`; \
                     for a whole claim, name it alone: `evidence <claim> at 1`"
                        .to_string(),
                )
            }
            Some(Tok::Ident(s)) => {
                self.pos -= 1;
                self.err(format!("`{s}` is a keyword, expected a name"))
            }
            _ => {
                self.pos -= 1;
                self.err("expected a name")
            }
        }
    }
    /// `forall` or `∀`.
    fn eat_forall(&mut self) -> bool {
        self.eat_keyword("forall") || self.eat("∀")
    }
    /// `in` or `∈`.
    fn eat_in(&mut self) -> bool {
        self.eat_keyword("in") || self.eat("∈")
    }
    /// `not` or `¬`.
    fn eat_not(&mut self) -> bool {
        self.eat_keyword("not") || self.eat("¬")
    }
    fn is_keyword(&self, k: &str) -> bool {
        matches!(self.peek(), Some(Tok::Ident(s)) if s == k)
    }
    fn eat_keyword(&mut self, k: &str) -> bool {
        let yes = self.is_keyword(k);
        self.pos += yes as usize;
        yes
    }
    fn keyword(&mut self, k: &str) -> Result<(), Error> {
        if self.eat_keyword(k) {
            Ok(())
        } else {
            self.err(format!("expected `{k}`"))
        }
    }
    /// A weight or a confidence: `1`, `0.5`.
    fn number(&mut self) -> Result<f64, Error> {
        match self.peek().cloned() {
            Some(Tok::Num(s)) => {
                self.pos += 1;
                match s.parse::<f64>() {
                    Ok(x) if x.is_finite() => Ok(x),
                    _ => self.err(format!("`{s}` is not a number Mitra can use")),
                }
            }
            _ => self.err("expected a number"),
        }
    }
    /// A natural number, negated when `negative`: down to `i64::MIN`.
    fn signed(&mut self, negative: bool) -> Result<i64, Error> {
        match self.peek().cloned() {
            Some(Tok::Num(s)) if !s.contains('.') => {
                let n: Option<i128> = s.parse().ok();
                let n = n.map(|n| if negative { -n } else { n });
                match n.and_then(|n| i64::try_from(n).ok()) {
                    Some(n) => {
                        self.pos += 1;
                        Ok(n)
                    }
                    None => self.err(format!("`{s}` is outside the 64-bit integers")),
                }
            }
            _ => self.err("expected an integer"),
        }
    }
    /// An integer or a float literal, negated when `negative`.
    fn numeral(&mut self, negative: bool) -> Result<Value, Error> {
        match self.peek().cloned() {
            Some(Tok::Num(s)) if s.contains('.') => match s.parse::<f64>() {
                Ok(x) if x.is_finite() => {
                    self.pos += 1;
                    Ok(Value::Float(if negative { -x } else { x }))
                }
                _ => self.err(format!("`{s}` is outside the 64-bit floats")),
            },
            _ => Ok(Value::Int(self.signed(negative)?)),
        }
    }
    /// `( value {, value} )`, after the `(`.
    fn values(&mut self) -> Result<Vec<Value>, Error> {
        let mut parts = vec![self.value()?];
        while self.eat(",") {
            parts.push(self.value()?);
        }
        self.expect(")")?;
        Ok(parts)
    }
    /// value := "true" | "false" | ["-"] number | string
    ///        | "(" value "," value {"," value} ")" | name "(" value {"," value} ")"
    fn value(&mut self) -> Result<Value, Error> {
        if self.eat("-") {
            return self.numeral(true);
        }
        if self.eat("(") {
            if self.eat(")") {
                // `()`: the one value of the unit type.
                return Ok(Value::Tuple(Vec::new()));
            }
            let parts = self.values()?;
            if parts.len() < 2 {
                return self.err("a tuple has two values or more: `(1, 2)`");
            }
            return Ok(Value::Tuple(parts));
        }
        if self.is_name() {
            // `Some(1)`, or a variant without fields: `None`.
            let name = self.ident()?;
            let parts = match self.eat("(") {
                true => self.values()?,
                false => Vec::new(),
            };
            return Ok(Value::Record(name, parts));
        }
        if self.eat("[") {
            let mut parts = Vec::new();
            while !self.eat("]") {
                if !parts.is_empty() {
                    self.expect(",")?;
                }
                parts.push(self.value()?);
            }
            return Ok(Value::List(parts));
        }
        if self.eat("{") {
            // `{}`, `{1, 2}`: a set. `{:}`, `{"a": 1}`: a map.
            let (mut items, mut pairs) = (Vec::new(), Vec::new());
            let mut map = self.eat(":");
            while !self.eat("}") {
                if map && pairs.is_empty() && self.is("}") {
                    continue;
                }
                if !(items.is_empty() && pairs.is_empty()) {
                    self.expect(",")?;
                }
                let v = self.value()?;
                if items.is_empty() && pairs.is_empty() && !map {
                    map = self.is(":");
                }
                match map {
                    true => {
                        self.expect(":")?;
                        pairs.push((v, self.value()?));
                    }
                    false => items.push(v),
                }
            }
            return match map {
                true => match Value::map_of(pairs) {
                    Some(m) => Ok(m),
                    None => self.err("a key comes twice in this map"),
                },
                false => Ok(Value::set_of(items)),
            };
        }
        match self.peek().cloned() {
            Some(Tok::Ident(s)) if s == "true" || s == "false" => {
                self.pos += 1;
                Ok(Value::Bool(s == "true"))
            }
            Some(Tok::Num(_)) => self.numeral(false),
            Some(Tok::Str(s)) => {
                self.pos += 1;
                Ok(Value::Str(s))
            }
            _ => self.err(
                "expected a value: true, false, a number, a string, a tuple `(1, 2)`, \
                 a record `Name(1, 2)` or a list `[1, 2]`",
            ),
        }
    }
    /// A range bound: `5` or `-5`, if one is there. `Int` is unbounded, so
    /// bounds may go past 64 bits, up to 2^126 either way.
    fn bound(&mut self) -> Result<Option<i128>, Error> {
        let negative = self.eat("-");
        match self.peek().cloned() {
            Some(Tok::Num(n)) if n.contains('.') => {
                self.err("ranges are over integers: write floats as a set, `{1.5, 2.5}`")
            }
            Some(Tok::Num(n)) => {
                const LIMIT: i128 = 1 << 126;
                match n.parse::<i128>().ok().filter(|&x| x <= LIMIT) {
                    Some(x) => {
                        self.pos += 1;
                        Ok(Some(if negative { -x } else { x }))
                    }
                    None => self.err(format!("`{n}` is too large for a range bound")),
                }
            }
            _ if negative => self.err("expected an integer"),
            _ => Ok(None),
        }
    }

    // ---- sets ----

    /// set := inter { "|" inter }
    fn set(&mut self) -> Result<Set, Error> {
        let mut s = self.set_inter()?;
        while self.eat("|") || self.eat("∪") {
            s = s.union(&self.set_inter()?);
        }
        Ok(s)
    }
    /// inter := atom { ("&" | "\\") atom }, left to right.
    fn set_inter(&mut self) -> Result<Set, Error> {
        let mut s = self.set_atom()?;
        loop {
            s = if self.eat("&") || self.eat("∩") {
                s.intersect(&self.set_atom()?)
            } else if self.eat("\\") {
                s.minus(&self.set_atom()?)
            } else {
                return Ok(s);
            };
        }
    }
    fn set_atom(&mut self) -> Result<Set, Error> {
        if self.eat("(") {
            if self.eat(")") {
                // `()`: the unit type, whose one value is `()`.
                return Ok(Set::tuple(Vec::new()));
            }
            // `(S)` groups; `(S, T)` is the tuples of one value from each.
            let mut parts = vec![self.set()?];
            while self.eat(",") {
                parts.push(self.set()?);
            }
            self.expect(")")?;
            return Ok(match parts.len() {
                1 => parts.remove(0),
                _ => Set::tuple(parts),
            });
        }
        if self.eat_keyword("List") {
            // `List(T)`: the lists whose elements are all in T.
            self.expect("(")?;
            let elements = self.set()?;
            self.expect(")")?;
            return Ok(Set::list(elements));
        }
        if self.eat_keyword("Set") {
            self.expect("(")?;
            let elements = self.set()?;
            self.expect(")")?;
            return Ok(Set::sets(elements));
        }
        if self.eat_keyword("Map") {
            self.expect("(")?;
            // `Map(K, V)`; with one set, its `(key, value)` pairs.
            let keys = self.set()?;
            if self.eat(")") {
                return Ok(Set::record(MAP, vec![Set::list(keys)]));
            }
            self.expect(",")?;
            let values = self.set()?;
            self.expect(")")?;
            return Ok(Set::maps(keys, values));
        }
        if self.is_name() {
            // `Number`: every value of the record. `Number(S, T)`: those
            // with one field in each set.
            let line = self.line();
            let name = self.ident()?;
            if let Some((_, set)) = self.params.iter().find(|p| p.0 == name) {
                return Ok(set.clone());
            }
            let mut args = Vec::new();
            if self.eat("(") {
                args.push(self.set()?);
                while self.eat(",") {
                    args.push(self.set()?);
                }
                self.expect(")")?;
            }
            // `Option(Int)`: an enum, with a set for each of its parameters.
            if let Some(e) = self.enums.iter().find(|e| e.name == name).cloned() {
                if args.len() != e.params.len() {
                    return Err(Error::new(
                        line,
                        format!(
                            "`{name}` takes {} set(s), got {}",
                            e.params.len(),
                            args.len()
                        ),
                    ));
                }
                let written: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                let written = match args.is_empty() {
                    true => name.clone(),
                    false => format!("{name}({})", written.join(", ")),
                };
                let params = e.params.iter().cloned().zip(args).collect();
                let (variants, _) = self.variants(&e, params, line)?;
                let set = variants.iter().fold(Set::empty(), |s, v| s.union(&v.set()));
                self.aliases.push((set.to_string(), written));
                return Ok(set);
            }
            if !args.is_empty() {
                self.used.push(name.clone());
                return Ok(Set::record(&name, args));
            }
            return match self.records.iter().find(|r| r.name == name) {
                Some(r) => Ok(r.set()),
                None => Err(Error::new(
                    line,
                    format!(
                        "unknown type `{name}` (Bool, Int, Int8 to Int64, UInt8 to UInt32, \
                         Float64, String, Value, or a record or an enum declared above)"
                    ),
                )),
            };
        }
        if self.eat("{") {
            let mut s = Set::empty();
            if self.eat("}") {
                return Ok(s);
            }
            loop {
                s = s.union(&Set::point(&self.value()?));
                if self.eat("}") {
                    return Ok(s);
                }
                self.expect(",")?;
            }
        }
        if matches!(self.peek(), Some(Tok::Num(_)))
            || self.is("-")
            || self.is("..")
            || self.is("..=")
        {
            // A range: `a..=b`, `a..b`, `a..`, `..=b`, `..b`.
            let lo = self.bound()?;
            let hi = if self.eat("..=") {
                match self.bound()? {
                    Some(hi) => Some(hi),
                    None => return self.err("`..=` needs an upper bound"),
                }
            } else if self.eat("..") {
                match self.bound()? {
                    // `a..b` leaves `b` out.
                    Some(hi) => Some(hi - 1),
                    None if lo.is_none() => return self.err("`..` alone: write `Int`"),
                    None => None,
                }
            } else {
                return self.err("expected `..=` or `..` after the lower bound");
            };
            return Ok(Set::range(lo, hi));
        }
        let set = match self.peek() {
            Some(Tok::Ident(t)) => match t.as_str() {
                "Bool" => Set::bool(),
                "Int" => Set::int(),
                "Float64" => Set::float(),
                "String" => Set::string(),
                "Value" => Set::any(),
                "Real" => return self.err("`Real` is not supported yet: use `Float64`"),
                t => match Set::int_type(t) {
                    Some(s) => s,
                    None => {
                        return self.err(format!(
                            "unknown type `{t}` (Bool, Int, Int8 to Int64, UInt8 to UInt32, \
                             Float64, String, Value)"
                        ))
                    }
                },
            },
            _ => return self.err("expected a set"),
        };
        self.pos += 1;
        Ok(set)
    }

    /// The variants of an enum, from its body, with a set for each
    /// parameter.
    fn variants(
        &self,
        of: &Enum,
        params: Vec<(String, Set)>,
        line: usize,
    ) -> Result<(Vec<Record>, Vec<String>), Error> {
        let at = |e: Error| Error::new(line, e.message);
        let mut p = P::new(&of.body).map_err(at)?;
        // The enums declared before this one.
        p.records = self.records.clone();
        let before = self.enums.iter().take_while(|e| e.name != of.name);
        p.enums = before.cloned().collect();
        p.params = params;
        let mut out = Vec::new();
        loop {
            let name = p.ident().map_err(at)?;
            let fields = match p.is("(") {
                true => p.binders().map_err(at)?,
                false => Vec::new(),
            };
            if fields.is_empty()
                && p.toks
                    .get(p.pos.wrapping_sub(1))
                    .is_some_and(|t| t.0 == Tok::Punct(")"))
            {
                return Err(at(Error::new(
                    0,
                    format!("a variant without fields is written `{name}`, without `()`"),
                )));
            }
            out.push(Record {
                name,
                fields,
                of: None,
                line,
            });
            if p.at_end() {
                return match p.used.contains(&of.name) {
                    true => Err(Error::new(
                        line,
                        format!("the enum `{}` can't contain itself", of.name),
                    )),
                    false => Ok((out, p.used)),
                };
            }
            p.expect(",").map_err(at)?;
        }
    }

    // ---- binders, arguments, patterns ----

    /// `(a: Int, c: Bool)`, or `()`.
    fn binders(&mut self) -> Result<Vec<(String, Set)>, Error> {
        let mut params = Vec::new();
        self.expect("(")?;
        if self.eat(")") {
            return Ok(params);
        }
        loop {
            let name = self.ident()?;
            self.expect(":")?;
            params.push((name, self.set()?));
            if !self.eat(",") {
                self.expect(")")?;
                return Ok(params);
            }
        }
    }
    /// `forall x, forall y in S, …`: each bound name with its set, if any.
    fn foralls(&mut self) -> Result<Vec<(String, Option<Set>, usize)>, Error> {
        let mut out = Vec::new();
        while self.eat_forall() {
            let line = self.line();
            let x = self.ident()?;
            if out.iter().any(|(y, _, _)| y == &x) {
                return Err(Error::new(line, format!("`{x}` is bound twice")));
            }
            let set = if self.eat_in() {
                Some(self.set()?)
            } else {
                None
            };
            self.expect(",")?;
            out.push((x, set, line));
        }
        Ok(out)
    }
    /// The arguments of a call in a claim or evidence: bound names and
    /// values, as a pattern. Every bound name is used exactly once.
    fn args(&mut self, bound: &[(String, Option<Set>, usize)]) -> Result<Pattern, Error> {
        let mut pattern = Vec::new();
        // `(a, 1, -2)`; no parentheses when there are no arguments.
        self.parens = self.eat("(");
        let mut more = self.parens && !self.eat(")");
        while more {
            // A name followed by `(` is a record; a name that no `forall`
            // binds is a variant without fields, when one is declared.
            let variant = |x: &String| {
                !bound.iter().any(|b| &b.0 == x)
                    && self
                        .records
                        .iter()
                        .any(|r| &r.name == x && r.fields.is_empty())
            };
            let record = self.is_name()
                && (matches!(self.toks.get(self.pos + 1), Some((Tok::Punct("("), _)))
                    || matches!(self.peek(), Some(Tok::Ident(x)) if variant(x)));
            let value = matches!(self.peek(), Some(Tok::Num(_) | Tok::Str(_)))
                || record
                || self.is("-")
                || self.is("(")
                || self.is("[")
                || self.is("{")
                || self.is_keyword("true")
                || self.is_keyword("false");
            let pos = if value {
                Position {
                    var: None,
                    set: Some(Set::point(&self.value()?)),
                }
            } else {
                let line = self.line();
                let x = self.ident()?;
                let at = |m: String| Err(Error::new(line, m));
                let Some((_, set, _)) = bound.iter().find(|b| b.0 == x) else {
                    return at(format!("`{x}` is not bound: add `forall {x},` before it"));
                };
                if pattern
                    .iter()
                    .any(|p: &Position| p.var.as_ref() == Some(&x))
                {
                    return at(format!("`{x}` is used twice: one name per input"));
                }
                Position {
                    var: Some(x),
                    set: set.clone(),
                }
            };
            pattern.push(pos);
            more = self.eat(",");
            if !more {
                self.expect(")")?;
            }
        }
        if let Some((x, _, line)) = bound
            .iter()
            .find(|b| !pattern.iter().any(|p| p.var.as_ref() == Some(&b.0)))
        {
            return Err(Error::new(*line, format!("`{x}` is bound but not used")));
        }
        Ok(pattern)
    }
    /// `name args`: what a judgment or claim is about.
    fn call(&mut self, bound: &[(String, Option<Set>, usize)]) -> Result<(String, Pattern), Error> {
        let f = self.ident()?;
        Ok((f, self.args(bound)?))
    }
    /// `correct(f(args))`, after `correct`.
    fn correct(
        &mut self,
        bound: &[(String, Option<Set>, usize)],
    ) -> Result<(String, Pattern), Error> {
        self.expect("(")?;
        let c = self.call(bound)?;
        if !self.parens {
            return self.err(format!(
                "`correct({0})` needs the inputs of a subject, as in `correct({0}(1, 2))`\n  \
                 help: if `{0}` is a claim, write `evidence {0} at 1`",
                c.0
            ));
        }
        self.expect(")")?;
        Ok(c)
    }

    /// `forall …, <judgment>`: what a piece of evidence or a test says.
    fn judgment(&mut self) -> Result<(String, Pattern, Judgment), Error> {
        let bound = self.foralls()?;
        if self.is_measure() {
            // rate(x in S, cond) = 0.4: a measure, as measured.
            let line = self.line();
            if !bound.is_empty() {
                return Err(Error::new(
                    line,
                    "a measurement takes no `forall`".to_string(),
                ));
            }
            let measure = self.atom()?;
            self.expect("=")?;
            let value = self.number()?;
            let unbound = free(&measure, &mut Vec::new(), &mut Vec::new());
            let variant = |x: &String| {
                let none = |r: &Record| &r.name == x && r.fields.is_empty();
                self.records.iter().any(none)
            };
            if let Some(x) = unbound.into_iter().find(|x| !variant(x)) {
                let msg = format!("`{x}` is not bound: a measurement names its inputs");
                return Err(Error::new(line, msg));
            }
            let about = crate::print::expr(&measure);
            return Ok((about, Vec::new(), Judgment::Measured(measure, value)));
        }
        Ok(if self.eat_keyword("correct") {
            let (f, pat) = self.correct(&bound)?;
            (f, pat, Judgment::Correct(true))
        } else if self.eat_not() {
            if self.eat_keyword("correct") {
                let (f, pat) = self.correct(&bound)?;
                (f, pat, Judgment::Correct(false))
            } else {
                let (c, pat) = self.call(&bound)?;
                (c, pat, Judgment::Holds(false))
            }
        } else {
            let (f, pat) = self.call(&bound)?;
            if self.eat("=") {
                if !self.parens {
                    return self.err(format!(
                        "`{f}` needs its inputs: `{f}() = …` when it has none"
                    ));
                }
                (f, pat, Judgment::Output(self.value()?))
            } else {
                (f, pat, Judgment::Holds(true))
            }
        })
    }

    // ---- expressions ----

    /// expr := "if" expr "then" expr "else" expr | "let" name ":=" expr ";" expr | or
    fn expr(&mut self) -> Result<Expr, Error> {
        if self.eat_keyword("if") {
            let c = self.expr()?;
            self.keyword("then")?;
            let t = self.expr()?;
            self.keyword("else")?;
            let e = self.expr()?;
            return Ok(Expr::If(Box::new(c), Box::new(t), Box::new(e)));
        }
        if self.eat_keyword("let") {
            let x = self.ident()?;
            self.expect(":=")?;
            let v = self.expr()?;
            self.expect(";")?;
            let body = self.expr()?;
            return Ok(Expr::Let(x, Box::new(v), Box::new(body)));
        }
        self.binary(0)
    }
    /// Binary operators by level, all left-associative:
    /// `||`, `&&`, comparisons, `+ -`, `*`.
    fn binary(&mut self, level: usize) -> Result<Expr, Error> {
        const LEVELS: [&[&str]; 5] = [
            &["||"],
            &["&&"],
            &["==", "!=", "<", "<=", ">", ">="],
            // `|` and `\` on sets and maps bind like `+`, `&` like `*`.
            &["+", "-", "|", "\\"],
            &["*", "/", "&"],
        ];
        if level == LEVELS.len() {
            return self.unary();
        }
        let mut e = self.binary(level + 1)?;
        loop {
            let op = match LEVELS[level].iter().find(|op| self.is(op)) {
                Some(op) => {
                    self.pos += 1;
                    op
                }
                // `x in s` is a comparison.
                None if level == 2 && self.eat_in() => "in",
                None => return Ok(e),
            };
            e = Expr::Binary(op, Box::new(e), Box::new(self.binary(level + 1)?));
        }
    }
    /// unary := ("!" | "-") unary | name { atom } | atom.
    /// `-` on an integer literal gives a negative literal.
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.eat("!") {
            return Ok(Expr::Unary("!", Box::new(self.unary()?)));
        }
        // `-` right before digits is a negative literal, down to i64::MIN.
        let negative = self.eat("-");
        if negative
            && !matches!(self.peek(), Some(Tok::Num(n)) if n.contains('.') || !n.trim_start_matches('0').is_empty())
        {
            return Ok(Expr::Unary("-", Box::new(self.unary()?)));
        }
        let mut e = if negative {
            Expr::Lit(self.numeral(true)?)
        } else if self.is_name() {
            let f = self.ident()?;
            if self.eat("(") {
                // f(a, b), or the record Name(a, b)
                let mut args = Vec::new();
                if !self.eat(")") {
                    args.push(self.expr()?);
                    while self.eat(",") {
                        args.push(self.expr()?);
                    }
                    self.expect(")")?;
                }
                // A record's name makes it a record: see `records_in`.
                Expr::Call(f, args)
            } else {
                Expr::Var(f)
            }
        } else {
            self.atom()?
        };
        // `.field` on a record, `.0` on a tuple, `[i]` on a list.
        loop {
            if self.eat("[") {
                let i = self.expr()?;
                self.expect("]")?;
                e = Expr::Index(Box::new(e), Box::new(i));
                continue;
            }
            if !self.eat(".") {
                break;
            }
            let field = match self.next() {
                Some(Tok::Ident(s)) if !KEYWORDS.contains(&s.as_str()) => s,
                Some(Tok::Num(n)) if !n.contains('.') => n,
                _ => {
                    self.pos -= 1;
                    return self.err(
                        "expected a field name, or a position such as `.0`; \
                         for a position inside a position write `(t.0).1`",
                    );
                }
            };
            e = Expr::Field(Box::new(e), field);
        }
        Ok(e)
    }
    /// atom := value | name | "(" expr ")"
    /// At `rate`, `precision`, `recall` or `false_positive_rate`.
    fn is_measure(&self) -> bool {
        self.is_keyword("rate") || SCORES.iter().any(|k| self.is_keyword(k))
    }
    fn atom(&mut self) -> Result<Expr, Error> {
        if self.eat_keyword("rate") {
            // rate(x in S, cond)
            self.expect("(")?;
            let x = self.ident()?;
            if !self.eat_in() {
                return self.err("expected `in`");
            }
            let set = self.set()?;
            self.expect(",")?;
            let cond = self.expr()?;
            self.expect(")")?;
            return Ok(Expr::Rate(x, set, Box::new(cond)));
        }
        if let Some(kind) = SCORES.iter().find(|k| self.is_keyword(k)) {
            self.pos += 1;
            self.expect("(")?;
            let f = self.ident()?;
            self.expect(")")?;
            return Ok(Expr::Score(kind, f));
        }
        if self.eat("(") {
            if self.eat(")") {
                return Ok(Expr::Tuple(Vec::new()));
            }
            // `(e)` groups; `(a, b)` is a tuple.
            let mut parts = vec![self.expr()?];
            while self.eat(",") {
                parts.push(self.expr()?);
            }
            self.expect(")")?;
            return Ok(match parts.len() {
                1 => parts.remove(0),
                _ => Expr::Tuple(parts),
            });
        }
        if self.eat("[") {
            let mut parts = Vec::new();
            while !self.eat("]") {
                if !parts.is_empty() {
                    self.expect(",")?;
                }
                parts.push(self.expr()?);
            }
            return Ok(Expr::List(parts));
        }
        if self.eat_keyword("len") {
            self.expect("(")?;
            let e = self.expr()?;
            self.expect(")")?;
            return Ok(Expr::Len(Box::new(e)));
        }
        if self.eat_keyword("keys") {
            self.expect("(")?;
            let e = self.expr()?;
            self.expect(")")?;
            return Ok(Expr::Unary("keys", Box::new(e)));
        }
        if self.eat("{") {
            // `{}`, `{a, b}`: a set. `{:}`, `{k: v}`: a map. Both are
            // records of one list, put in order when evaluated.
            let (mut items, mut pairs) = (Vec::new(), Vec::new());
            let mut map = self.eat(":");
            while !self.eat("}") {
                if map && items.is_empty() && pairs.is_empty() && self.is("}") {
                    continue;
                }
                if !(items.is_empty() && pairs.is_empty()) {
                    self.expect(",")?;
                }
                let e = self.expr()?;
                if items.is_empty() && pairs.is_empty() && !map {
                    map = self.is(":");
                }
                match map {
                    true => {
                        self.expect(":")?;
                        pairs.push(Expr::Tuple(vec![e, self.expr()?]));
                    }
                    false => items.push(e),
                }
            }
            return Ok(match map {
                true => Expr::Record(MAP.into(), vec![Expr::List(pairs)]),
                false => Expr::Record(SET.into(), vec![Expr::List(items)]),
            });
        }
        if self.eat_keyword("match") {
            // match e { None => a, Some(v) => b, _ => c }
            let e = self.expr()?;
            self.expect("{")?;
            let mut arms = Vec::new();
            loop {
                let variant = if self.eat_keyword("_") {
                    None
                } else {
                    let name = self.ident()?;
                    let mut fields = Vec::new();
                    if self.eat("(") {
                        fields.push(self.ident()?);
                        while self.eat(",") {
                            fields.push(self.ident()?);
                        }
                        self.expect(")")?;
                    }
                    Some((name, fields))
                };
                self.expect("=>")?;
                arms.push(Arm {
                    variant,
                    body: self.expr()?,
                });
                if !self.eat(",") {
                    self.expect("}")?;
                    return Ok(Expr::Match(Box::new(e), arms));
                }
            }
        }
        if self.is_name() {
            return Ok(Expr::Var(self.ident()?));
        }
        if self.is("-") {
            return self.err("expected an expression");
        }
        Ok(Expr::Lit(self.value()?))
    }
}

/// The model file: `opaque`, `def`, `claim` and `test`.
pub fn parse_model(src: &str) -> Result<Model, Error> {
    let mut p = P::new(src)?;
    let mut m = Model::default();
    // The property claims, each with the names it binds.
    let mut properties: Vec<(usize, Vec<Bound>)> = Vec::new();
    while !p.at_end() {
        let line = p.line();
        let Some(Tok::Ident(kw)) = p.next() else {
            p.pos -= 1;
            return p.err("expected `record`, `enum`, `opaque`, `def`, `claim` or `test`");
        };
        match kw.as_str() {
            "record" => {
                // record Name(field: T, …)
                let name = p.ident()?;
                let fields = p.binders()?;
                if fields.is_empty() {
                    return Err(Error::new(
                        line,
                        format!("the record `{name}` needs a field"),
                    ));
                }
                let record = Record {
                    name,
                    fields,
                    of: None,
                    line,
                };
                m.declared.push(Decl::Record(m.records.len()));
                p.records.push(record.clone());
                m.records.push(record);
            }
            "enum" => {
                // enum Name(T, …) { Variant, Variant(field: T, …), … }
                let name = p.ident()?;
                if p.used.contains(&name) {
                    return Err(Error::new(
                        line,
                        format!("`{name}` is used above: declare an enum before its first use"),
                    ));
                }
                let mut params = Vec::new();
                if p.eat("(") {
                    params.push(p.ident()?);
                    while p.eat(",") {
                        params.push(p.ident()?);
                    }
                    p.expect(")")?;
                }
                if let Some(x) = (1..params.len()).find(|i| params[..*i].contains(&params[*i])) {
                    let msg = format!("`{name}` has two parameters `{}`", params[x]);
                    return Err(Error::new(line, msg));
                }
                p.expect("{")?;
                // The body is kept as written, and read again for each use.
                let (start, mut depth) = (p.pos, 1);
                while depth > 0 {
                    match p.next() {
                        Some(Tok::Punct("{")) => depth += 1,
                        Some(Tok::Punct("}")) => depth -= 1,
                        Some(_) => {}
                        None => {
                            p.pos -= 1;
                            return p.err("expected `}` to close the enum");
                        }
                    }
                }
                let toks: Vec<Tok> = p.toks[start..p.pos - 1]
                    .iter()
                    .map(|t| t.0.clone())
                    .collect();
                let body = join(&toks);
                // Checked once with every parameter as `Value`.
                let any = params.iter().map(|x| (x.clone(), Set::any())).collect();
                let mut e = Enum {
                    name: name.clone(),
                    params,
                    body,
                    variants: Vec::new(),
                    line,
                };
                let (variants, used) = p.variants(&e, any, line)?;
                p.used.extend(used);
                e.variants = variants.iter().map(|v| v.name.clone()).collect();
                for mut v in variants {
                    v.of = Some(name.clone());
                    p.records.push(v.clone());
                    m.records.push(v);
                }
                m.declared.push(Decl::Enum(m.enums.len()));
                p.enums.push(e.clone());
                m.enums.push(e);
            }
            "opaque" | "def" => {
                // opaque f (a : T) … : R   |   def f (a : T) … : R := expr
                let name = p.ident()?;
                let params = p.binders()?;
                p.expect("->")?;
                let range = p.set()?;
                let body = if kw == "def" {
                    p.expect(":=")?;
                    Some(p.expr()?)
                } else {
                    None
                };
                m.functions.push(Function {
                    name,
                    params,
                    range,
                    body,
                    line,
                });
            }
            "claim" => {
                // claim n : forall …, correct(f(args)) | f(args) = expr  [:= by derive [c, …]]
                let name = p.ident()?;
                p.expect(":")?;
                if p.eat_keyword("uniform") {
                    // claim n : uniform f [by g]
                    let func = p.ident()?;
                    let spec = match p.eat_keyword("by") {
                        true => Some(p.ident()?),
                        false => None,
                    };
                    let target = Target::Uniform(spec);
                    m.claims.push(Claim {
                        name,
                        func,
                        pattern: Vec::new(),
                        target,
                        from: Vec::new(),
                        line,
                    });
                    continue;
                }
                let only = p.eat_keyword("only");
                if only && !p.is_keyword("uses") {
                    return p.err("expected `uses` after `only`");
                }
                if p.eat_keyword("uses") {
                    // claim n : [only] uses f [g, h]
                    let func = p.ident()?;
                    p.expect("[")?;
                    let mut used = vec![p.ident()?];
                    while p.eat(",") {
                        used.push(p.ident()?);
                    }
                    p.expect("]")?;
                    m.claims.push(Claim {
                        name,
                        func,
                        pattern: Vec::new(),
                        target: Target::Uses { used, only },
                        from: Vec::new(),
                        line,
                    });
                    continue;
                }
                let bound = p.foralls()?;
                let (func, pattern, target) = if p.eat_keyword("correct") {
                    let (f, pat) = p.correct(&bound)?;
                    (f, pat, Target::Correct)
                } else {
                    // `f(args) = expr` with one name per input, or any
                    // expression with a Bool value.
                    let start = p.pos;
                    let own = p.call(&bound).and_then(|(f, pat)| {
                        if !p.parens {
                            return p.err(format!("`{f}` needs its inputs"));
                        }
                        p.expect("=")?;
                        Ok((f, pat))
                    });
                    match own {
                        Ok((f, pat)) => (f, pat, Target::Matches(p.expr()?)),
                        Err(first) => {
                            p.pos = start;
                            let bare = p.is_name()
                                && matches!(p.toks.get(p.pos + 1), Some((Tok::Punct("="), _)));
                            if bare {
                                return Err(first);
                            }
                            let mut e = p.expr()?;
                            if p.eat("=") {
                                e = Expr::Binary("==", Box::new(e), Box::new(p.expr()?));
                            }
                            let pattern = bound
                                .iter()
                                .map(|(x, set, _)| Position {
                                    var: Some(x.clone()),
                                    set: set.clone(),
                                })
                                .collect();
                            properties.push((m.claims.len(), bound.clone()));
                            (name.clone(), pattern, Target::Property(e))
                        }
                    }
                };
                let mut from = Vec::new();
                if p.eat(":=") {
                    p.keyword("by")?;
                    p.keyword("derive")?;
                    p.expect("[")?;
                    from.push(p.ident()?);
                    while p.eat(",") {
                        from.push(p.ident()?);
                    }
                    p.expect("]")?;
                }
                m.claims.push(Claim {
                    name,
                    func,
                    pattern,
                    target,
                    from,
                    line,
                });
            }
            "test" => {
                // test t : <judgment>
                let name = p.ident()?;
                p.expect(":")?;
                let (about, pattern, judgment) = p.judgment()?;
                m.tests.push(Test {
                    name,
                    judgment: Evidence {
                        about,
                        pattern,
                        judgment,
                        confidence: 1.0,
                        source: None,
                        rev: None,
                        line,
                    },
                    line,
                });
            }
            k => {
                p.pos -= 1;
                return p.err(format!(
                    "expected `record`, `enum`, `opaque`, `def`, `claim` or `test`, got `{k}`"
                ));
            }
        }
    }
    // A property uses every name it binds, and no other.
    for (i, bound) in properties {
        // `f(x, y) == e` with one name per input is `f(x, y) = e`.
        let record = |f: &String| m.records.iter().any(|r| &r.name == f);
        if let Target::Property(Expr::Binary("==", call, e)) = &m.claims[i].target {
            if let Expr::Call(f, args) = &**call {
                let names: Vec<&String> = args
                    .iter()
                    .filter_map(|a| match a {
                        Expr::Var(x) => Some(x),
                        _ => None,
                    })
                    .collect();
                let each_once = names.len() == args.len()
                    && names.len() == bound.len()
                    && bound.iter().all(|b| names.contains(&&b.0));
                if each_once && !record(f) {
                    let pattern = names
                        .iter()
                        .map(|x| Position {
                            var: Some(x.to_string()),
                            set: bound.iter().find(|b| &&b.0 == x).unwrap().1.clone(),
                        })
                        .collect();
                    let (func, target) = (f.clone(), Target::Matches((**e).clone()));
                    let c = &mut m.claims[i];
                    (c.func, c.pattern, c.target) = (func, pattern, target);
                    continue;
                }
            }
        }
        let c = &m.claims[i];
        let Target::Property(e) = &c.target else {
            unreachable!()
        };
        let mut used = Vec::new();
        let variant = |x: &String| {
            m.records
                .iter()
                .any(|r| &r.name == x && r.fields.is_empty())
        };
        if let Some(x) = free(e, &mut Vec::new(), &mut used)
            .into_iter()
            .find(|x| !bound.iter().any(|b| &b.0 == x) && !variant(x))
        {
            return Err(Error::new(
                c.line,
                format!("`{x}` is not bound: add `forall {x},` before it"),
            ));
        }
        if let Some((x, _, line)) = bound.iter().find(|b| !used.contains(&b.0)) {
            return Err(Error::new(*line, format!("`{x}` is bound but not used")));
        }
    }
    m.aliases = std::mem::take(&mut p.aliases);
    m.aliases.sort();
    m.aliases.dedup();
    // `Name(a, b)` is a record when `Name` is one, wherever it is declared.
    let names: Vec<String> = m.records.iter().map(|r| r.name.clone()).collect();
    for f in &mut m.functions {
        if let Some(body) = &mut f.body {
            records_in(body, &names);
        }
    }
    for c in &mut m.claims {
        if let Target::Matches(e) | Target::Property(e) = &mut c.target {
            records_in(e, &names);
        }
    }
    Ok(m)
}

/// Turn the calls of a record's name into records.
/// The names an expression reads that it does not bind itself, in `used`
/// and returned.
fn free(e: &Expr, scope: &mut Vec<String>, used: &mut Vec<String>) -> Vec<String> {
    match e {
        Expr::Lit(_) => {}
        Expr::Var(x) => {
            if !scope.contains(x) && !used.contains(x) {
                used.push(x.clone());
            }
        }
        Expr::Let(x, a, b) => {
            free(a, scope, used);
            scope.push(x.clone());
            free(b, scope, used);
            scope.pop();
        }
        Expr::Match(a, arms) => {
            free(a, scope, used);
            for arm in arms {
                let names = arm.variant.iter().flat_map(|v| v.1.clone());
                let n = scope.len();
                scope.extend(names);
                free(&arm.body, scope, used);
                scope.truncate(n);
            }
        }
        Expr::Score(..) => {}
        Expr::Rate(x, _, a) => {
            scope.push(x.clone());
            free(a, scope, used);
            scope.pop();
        }
        Expr::Unary(_, a) | Expr::Field(a, _) | Expr::Len(a) => {
            free(a, scope, used);
        }
        Expr::Call(_, args) | Expr::Tuple(args) | Expr::Record(_, args) | Expr::List(args) => {
            args.iter().for_each(|a| {
                free(a, scope, used);
            });
        }
        Expr::Binary(_, a, b) | Expr::Index(a, b) => {
            free(a, scope, used);
            free(b, scope, used);
        }
        Expr::If(a, b, c) => {
            free(a, scope, used);
            free(b, scope, used);
            free(c, scope, used);
        }
    }
    used.clone()
}

fn records_in(e: &mut Expr, names: &[String]) {
    match e {
        Expr::Lit(_) | Expr::Var(_) => {}
        Expr::Call(f, args) => {
            args.iter_mut().for_each(|a| records_in(a, names));
            if names.contains(f) {
                *e = Expr::Record(std::mem::take(f), std::mem::take(args));
            }
        }
        Expr::Tuple(args) | Expr::Record(_, args) | Expr::List(args) => {
            args.iter_mut().for_each(|a| records_in(a, names))
        }
        Expr::Match(a, arms) => {
            records_in(a, names);
            arms.iter_mut()
                .for_each(|arm| records_in(&mut arm.body, names));
        }
        Expr::Score(..) => {}
        Expr::Unary(_, a) | Expr::Field(a, _) | Expr::Len(a) | Expr::Rate(_, _, a) => {
            records_in(a, names)
        }
        Expr::Binary(_, a, b) | Expr::Let(_, a, b) | Expr::Index(a, b) => {
            records_in(a, names);
            records_in(b, names);
        }
        Expr::If(a, b, c) => {
            records_in(a, names);
            records_in(b, names);
            records_in(c, names);
        }
    }
}

/// The evidence file: `evidence forall …, <judgment> at <confidence>`, where the
/// judgment is `correct(f(args))`, `not correct(f(args))`, `f(args) = value`,
/// `claim` or `not claim`.
pub fn parse_evidence(src: &str) -> Result<Vec<Evidence>, Error> {
    parse_evidence_for(src, &Model::default())
}

/// The evidence file of a model: its records and enums can then stand for
/// their values, as in `forall n in Number,`.
pub fn parse_evidence_for(src: &str, model: &Model) -> Result<Vec<Evidence>, Error> {
    let mut p = P::new(src)?;
    p.records = model.records.clone();
    p.enums = model.enums.clone();
    let mut out = Vec::new();
    while !p.at_end() {
        let line = p.line();
        let (mut source, mut rev) = (None, None);
        if p.eat("@[") {
            loop {
                let attr_line = p.line();
                let name = p.ident()?;
                let value = match p.next() {
                    Some(Tok::Str(s)) => s,
                    _ => {
                        p.pos -= 1;
                        return p.err(format!("`{name}` needs a string"));
                    }
                };
                let slot = match name.as_str() {
                    "source" => &mut source,
                    "rev" => &mut rev,
                    _ => {
                        return Err(Error::new(
                            attr_line,
                            format!("unknown attribute `{name}` (source, rev)"),
                        ))
                    }
                };
                if slot.replace(value).is_some() {
                    return Err(Error::new(attr_line, format!("`{name}` is given twice")));
                }
                if !p.eat(",") {
                    break;
                }
            }
            p.expect("]")?;
        }
        p.keyword("evidence")?;
        let (about, pattern, judgment) = p.judgment()?;
        p.keyword("at")?;
        let confidence = p.number()?;
        if !(0.0..=1.0).contains(&confidence) {
            return Err(Error::new(
                line,
                format!("bias {confidence}: it goes from 0 to 1"),
            ));
        }
        out.push(Evidence {
            about,
            pattern,
            judgment,
            confidence,
            source,
            rev,
            line,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_model() {
        let m = parse_model(
            "opaque id(x: Value) -> Value\n\
             claim id_small : forall x in 0..=100, correct(id(x))  -- a subset\n\
             claim id_all : forall x, correct(id(x))",
        )
        .unwrap();
        assert_eq!(m.functions[0].params, vec![("x".to_string(), Set::any())]);
        assert_eq!(m.claims[0].pattern[0].set, Some(Set::interval(0, 100)));
        assert_eq!(m.claims[1].line, 3);
    }

    #[test]
    fn parses_specs_and_derived_claims() {
        let m = parse_model(
            "def neg(a: Bool) -> Bool := !a\n\
             def dec(x: Int) -> Int := if x > 0 then x-1 else let y := -x; y * 2\n\
             claim twice : forall a, neg(a) = neg(neg(a)) := by derive [c1, c2]",
        )
        .unwrap();
        assert_eq!(
            m.functions[0].body,
            Some(Expr::Unary("!", Box::new(Expr::Var("a".into()))))
        );
        let Some(Expr::If(_, then, _)) = &m.functions[1].body else {
            panic!()
        };
        assert_eq!(
            **then,
            Expr::Binary(
                "-",
                Box::new(Expr::Var("x".into())),
                Box::new(Expr::Lit(Value::Int(1)))
            )
        );
        assert_eq!(m.claims[0].pattern[0].var.as_deref(), Some("a"));
        assert_eq!(m.claims[0].from, vec!["c1".to_string(), "c2".to_string()]);
    }

    #[test]
    fn parses_tuples() {
        let m = parse_model(
            "opaque minus(a: Int, b: Int) -> Int\n\
             def difference(a: Int, b: Int) -> Int := a - b\n\
             claim zero : forall x in 0..=10, minus(x, 0) = add(x, negate(0))",
        )
        .unwrap();
        assert_eq!(m.functions[0].params.len(), 2);
        assert_eq!(m.claims[0].pattern[1].set, Some(Set::point(&Value::Int(0))));
        let Target::Matches(Expr::Call(f, args)) = &m.claims[0].target else {
            panic!()
        };
        assert_eq!((f.as_str(), args.len()), ("add", 2));
        let e = parse_evidence("evidence minus(3, -1) = 4 at 1").unwrap();
        assert_eq!(e[0].pattern.len(), 2);
        assert_eq!(e[0].pattern[1].set, Some(Set::point(&Value::Int(-1))));
    }

    #[test]
    fn parses_evidence() {
        let e = parse_evidence(
            "evidence forall x in {1, -2, \"a\"} | 5..=6 \\ {6}, not correct(id(x)) at 1\n\
             evidence flip(true) = false at 1\n\
             evidence shape at 0\n\
             evidence not shape at 1",
        )
        .unwrap();
        let expected = [
            Value::Int(1),
            Value::Int(-2),
            Value::Str("a".into()),
            Value::Int(5),
        ]
        .iter()
        .fold(Set::empty(), |s, v| s.union(&Set::point(v)));
        assert_eq!(e[0].pattern[0].set, Some(expected));
        assert_eq!(e[0].judgment, Judgment::Correct(false));
        assert_eq!(e[1].judgment, Judgment::Output(Value::Bool(false)));
        assert_eq!(
            (e[2].judgment.clone(), e[2].pattern.len()),
            (Judgment::Holds(true), 0)
        );
        assert_eq!(e[3].judgment, Judgment::Holds(false));
    }

    #[test]
    fn set_precedence_is_lean_like() {
        // `∩` and `\` bind tighter than `∪`.
        let e =
            parse_evidence("evidence forall x in {1} | {3, 5} \\ {5}, correct(f(x)) at 1").unwrap();
        let s = e[0].pattern[0].set.clone().unwrap();
        assert_eq!(s.to_string(), "{1, 3}");
    }

    #[test]
    fn rejects_unbound_and_unused_names() {
        let e = parse_evidence("evidence correct(f(x)) at 1").unwrap_err();
        assert!(e.message.contains("not bound"), "{e}");
        let e = parse_evidence("evidence forall x, forall y, correct(f(x)) at 1").unwrap_err();
        assert!(e.message.contains("`y` is bound but not used"), "{e}");
    }

    #[test]
    fn reads_the_64_bit_extremes() {
        let e = parse_evidence(
            "evidence f(-9223372036854775808) = -9223372036854775808 at 1\n\
             evidence forall x in -9223372036854775808..=9223372036854775807, correct(f(x)) at 1",
        )
        .unwrap();
        assert_eq!(e[0].judgment, Judgment::Output(Value::Int(i64::MIN)));
        let m = parse_model("def f(a: Int) -> Int := -9223372036854775808").unwrap();
        assert_eq!(m.functions[0].body, Some(Expr::Lit(Value::Int(i64::MIN))));
        assert!(parse_evidence("evidence f(9223372036854775808) = 1 at 1").is_err());
        let w = format!("evidence correct(f(1)) at 1{}", "0".repeat(400));
        let message = parse_evidence(&w).unwrap_err().message;
        assert!(message.contains("not a number"), "{message}");
    }

    #[test]
    fn errors_point_at_the_name() {
        let e = parse_evidence("evidence correct(f(\n x\n)) at 1").unwrap_err();
        assert_eq!(e.line, 2);
        let e = parse_evidence("evidence forall x,\n forall x,\n correct(f(x)) at 1").unwrap_err();
        assert_eq!((e.line, e.message.contains("twice")), (2, true));
    }

    #[test]
    fn reads_attributes() {
        let e = parse_evidence(
            "@[source \"ci\", rev \"3f2a\"] evidence correct(flip(true)) at 1\n\
             evidence flip(true) = false at 1",
        )
        .unwrap();
        assert_eq!(
            (e[0].source.as_deref(), e[0].rev.as_deref()),
            (Some("ci"), Some("3f2a"))
        );
        assert_eq!((e[1].source.clone(), e[1].rev.clone()), (None, None));
        let err = parse_evidence("@[who \"me\"] evidence s at 1").unwrap_err();
        assert!(err.message.contains("unknown attribute"), "{err}");
        let err = parse_evidence("@[rev \"a\", rev \"b\"] evidence s at 1").unwrap_err();
        assert!(err.message.contains("twice"), "{err}");
    }

    #[test]
    fn a_bias_goes_from_0_to_1() {
        let e = parse_evidence("evidence correct(flip(true)) at 0.9").unwrap();
        assert_eq!(e[0].confidence, 0.9);
        let e = parse_evidence("evidence correct(flip(true)) at 1.5").unwrap_err();
        assert!(e.message.contains("from 0 to 1"), "{e}");
    }

    fn set_of(text: &str) -> String {
        let e =
            parse_evidence(&format!("evidence forall x in {text}, correct(f(x)) at 1")).unwrap();
        e[0].pattern[0].set.clone().unwrap().to_string()
    }

    #[test]
    fn reads_ranges() {
        assert_eq!(set_of("0..=10"), "0..=10");
        assert_eq!(set_of("0..10"), "0..=9");
        assert_eq!(set_of("-3..=-1"), "-3..=-1");
        assert_eq!(set_of("5.."), "5..");
        assert_eq!(set_of("..=5"), "..=5");
        assert_eq!(set_of("..5"), "..=4");
        assert_eq!(set_of("3..3"), "{}");
        assert_eq!(set_of("0..=1 | 5..=6 & 6..=9"), "{6} | 0..=1");
        // Bounds may pass 64 bits: `Int` is unbounded.
        assert_eq!(set_of("9223372036854775808.."), "9223372036854775808..");
        for bad in ["..", "1.5..=2", "0..=", "3"] {
            assert!(
                parse_evidence(&format!("evidence forall x in {bad}, correct(f(x)) at 1")).is_err(),
                "{bad}"
            );
        }
    }

    #[test]
    fn unicode_is_an_alias() {
        let ascii = "evidence forall x in 0..=3 | {7} & Int, not correct(f(x)) at 1";
        let lean = "evidence ∀ x ∈ 0..=3 ∪ {7} ∩ Int, ¬ correct(f(x)) at 1";
        assert_eq!(
            parse_evidence(ascii).unwrap(),
            parse_evidence(lean).unwrap()
        );
        // `not` is a keyword now, so it can't name a function.
        assert!(parse_model("opaque not(a: Bool) -> Bool").is_err());
    }

    #[test]
    fn reads_number_types_and_floats() {
        assert_eq!(set_of("Int8"), "Int8");
        assert_eq!(set_of("-128..=127"), "Int8");
        assert_eq!(set_of("UInt32 \\ {0}"), "1..=4294967295");
        assert_eq!(set_of("Float64 \\ {0.0}"), "Float64 \\ {0.0}");
        assert_eq!(set_of("{1.5, -0.0, 2}"), "{2, -0.0, 1.5}");
        let e = parse_evidence("evidence f(-1.5, 2.0) = -0.25 at 1").unwrap();
        assert_eq!(e[0].judgment, Judgment::Output(Value::Float(-0.25)));
        assert_eq!(e[0].pattern[0].set, Some(Set::point(&Value::Float(-1.5))));
        let m = parse_model("def g(x: Float64) -> Float64 := -1.5 * x").unwrap();
        assert!(
            matches!(&m.functions[0].body, Some(Expr::Binary("*", a, _)) if **a == Expr::Lit(Value::Float(-1.5)))
        );
        let err = parse_model("opaque r(x: Real) -> Real").unwrap_err();
        assert!(err.message.contains("Float64"), "{err}");
    }

    #[test]
    fn integers_past_64_bits_never_wrap() {
        // A single integer past 64 bits prints as a range, which reads back.
        assert_eq!(
            set_of("9223372036854775808..=9223372036854775808"),
            "9223372036854775808..=9223372036854775808"
        );
        let e = parse_evidence(
            "evidence forall x in 9223372036854775808..=9223372036854775808, correct(f(x)) at 1",
        )
        .unwrap();
        let s = e[0].pattern[0].set.clone().unwrap();
        assert_eq!((s.values(), s.single()), (None, None));
        // Counting a huge range doesn't overflow.
        let big =
            "-85070591730234615865843651857942052864..=85070591730234615865843651857942052864";
        assert!(parse_evidence(&format!("evidence forall x in {big}, correct(f(x)) at 1")).is_ok());
        let e = parse_evidence(&format!("evidence forall x in {big}, correct(f(x)) at 1")).unwrap();
        assert!(e[0].pattern[0].set.clone().unwrap().count().unwrap() > 1e38);
    }
}
