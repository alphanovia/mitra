//! The reader of the Mitra language: `subject` and `boundary` blocks,
//! `fn`, `claim` with inputs, `assume`, evidence blocks. It builds the
//! model as it reads (docs/reference/reworked.md).

use crate::ast::*;
use crate::set::{Set, Value, MAP, SET};
use crate::Error;
use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The modules shipped with the tool: plain Mitra, `mitra explain std::claims`
/// prints one.
const STD: [(&str, &str); 2] = [
    ("std::claims", include_str!("../std/claims.mitra")),
    ("std::rust", include_str!("../std/rust.mitra")),
];

thread_local! {
    /// Where `import a::b::name;` looks for `a/b.mitra`.
    static ROOT: RefCell<Option<PathBuf>> = const { RefCell::new(None) };
}

/// Read imported modules beside the model: `dir/a/b.mitra` for `a::b`.
pub fn imports_from(dir: Option<&Path>) {
    ROOT.with(|r| *r.borrow_mut() = dir.map(Path::to_path_buf));
}

/// The text of a module shipped with the tool.
pub fn library(module: &str) -> Option<&'static str> {
    STD.iter().find(|m| m.0 == module).map(|m| m.1)
}

/// The text of a module, and where it was looked for.
fn module(path: &[String]) -> Result<String, String> {
    let name = path.join("::");
    if path[0] == "std" {
        let known: Vec<&str> = STD.iter().map(|m| m.0).collect();
        let none = format!("no module `{name}` in `std`: it has {}", known.join(", "));
        return library(&name).map(String::from).ok_or(none);
    }
    let Some(root) = ROOT.with(|r| r.borrow().clone()) else {
        return Err(format!(
            "no module `{name}`: this model has no folder to import from"
        ));
    };
    let mut file = root;
    file.extend(path);
    file.set_extension("mitra");
    std::fs::read_to_string(&file)
        .map_err(|e| format!("no module `{name}`: {}: {e}", file.display()))
}

/// The declarations at the top of a file that start an item.
const ITEMS: [&str; 9] = [
    "subject", "boundary", "record", "enum", "fn", "claim", "assume", "test", "import",
];

#[derive(Debug, Clone, PartialEq, Default)]
enum Kind {
    #[default]
    Name,
    Num,
    Str,
    Punct,
}

#[derive(Debug, Clone, Default)]
struct Tok {
    kind: Kind,
    text: String,
    line: usize,
}

/// Longest first.
const PUNCT: [&str; 34] = [
    "<=>", "..=", "::", "=>", "->", "==", "!=", "<=", ">=", "&&", "||", "..", "(", ")", "{", "}",
    "[", "]", ",", ";", ":", "=", "<", ">", "+", "-", "*", "/", "!", "|", "&", "\\", ".", "@",
];

fn lex(src: &str) -> Result<Vec<Tok>, Error> {
    let mut toks = Vec::new();
    let mut line = 1;
    let mut rest = src;
    let mut push = |kind, text: &str, line| {
        toks.push(Tok {
            kind,
            text: text.to_string(),
            line,
        })
    };
    while let Some(c) = rest.chars().next() {
        if c == '\n' {
            line += 1;
        }
        if c.is_whitespace() {
            rest = &rest[c.len_utf8()..];
        } else if rest.starts_with("//") {
            rest = rest.find('\n').map_or("", |n| &rest[n..]);
        } else if c == '"' {
            let mut end = None;
            let mut escaped = false;
            for (i, c) in rest[1..].char_indices() {
                match c {
                    '\n' => break,
                    '"' if !escaped => {
                        end = Some(i);
                        break;
                    }
                    _ if escaped && !"nt\"\\".contains(c) => {
                        let msg = "in a string, `\\` starts `\\n`, `\\t`, `\\\"` or `\\\\`";
                        return Err(Error::new(line, msg));
                    }
                    _ => escaped = c == '\\' && !escaped,
                }
            }
            let Some(end) = end else {
                return Err(Error::new(line, "unterminated string"));
            };
            push(Kind::Str, &rest[..end + 2], line);
            rest = &rest[end + 2..];
        } else if c.is_ascii_digit() {
            let digits = |s: &str| s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
            let mut end = digits(rest);
            if let Some(frac) = rest[end..].strip_prefix('.').map(digits).filter(|&f| f > 0) {
                end += 1 + frac;
            }
            push(Kind::Num, &rest[..end], line);
            rest = &rest[end..];
        } else if c.is_ascii_alphabetic() || c == '_' {
            let end = rest
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .unwrap_or(rest.len());
            push(Kind::Name, &rest[..end], line);
            rest = &rest[end..];
        } else if let Some(p) = PUNCT.iter().find(|p| rest.starts_with(**p)) {
            push(Kind::Punct, p, line);
            rest = &rest[p.len()..];
        } else {
            return Err(Error::new(line, format!("unexpected character `{c}`")));
        }
    }
    Ok(toks)
}

/// The first line written in the earlier syntax (`opaque`, `def`, `:=`,
/// `claim c :`, `-- comment`), which is no longer read, and what to
/// write instead.
pub fn earlier(model: &str) -> Option<(usize, &'static str)> {
    let named = |l: &str, k: &str| {
        let rest = l.strip_prefix(k).map(str::trim_start)?;
        let name = rest.find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))?;
        Some(rest[name..].trim_start().starts_with(':') && name > 0)
    };
    let hint = |l: &str| {
        let l = l.trim_start();
        if l.starts_with("opaque ") {
            Some("`opaque f(a: T) -> U` is written `fn f(a: T) -> U;` inside `subject name { … }`")
        } else if l.starts_with("def ") {
            Some("`def f(a: T) -> U := e` is written `fn f(a: T) -> U { e }`")
        } else if named(l, "claim ") == Some(true) {
            Some("`claim c : forall a, e` is written `claim c(a: T) { e }`")
        } else if named(l, "test ") == Some(true) {
            Some("a test is a comment above the real test: `// mitra: evidence for <claim>`")
        } else if l.starts_with("--") {
            Some("a comment starts with `//`")
        } else if l.starts_with("@[")
            || (l.starts_with("evidence ") && l.contains(" at ") && !l.contains('{'))
        {
            Some("evidence is written `evidence name { for f given a = 1 holds }`")
        } else if l.contains(":=") {
            Some("`:=` is not used: a body is written in braces")
        } else {
            None
        }
    };
    let mut lines = model.lines().enumerate();
    lines.find_map(|(i, l)| Some((i + 1, hint(l)?)))
}

/// The inputs of a function: each name with its type, then the types
/// with the marks of the recursive ones, the result last.
#[derive(Debug, Default, Clone)]
struct Sig {
    params: Vec<(String, Set)>,
    deep: Vec<Set>,
}

/// What the model declares, for the evidence that names it.
#[derive(Debug, Default, Clone)]
pub struct Names {
    fns: BTreeMap<String, Sig>,
    /// Each claim with its inputs, and the measures in its body.
    claims: BTreeMap<String, (Vec<String>, Vec<Expr>)>,
    /// The functions of a subject that have a body: their spec.
    specified: BTreeMap<String, String>,
    /// The functions whose code is outside the model.
    pub opaque: BTreeSet<String>,
    /// For each claim, the spec claims of the functions it calls.
    pub derived: BTreeMap<String, Vec<String>>,
    /// The trust given to the statement of a claim.
    pub trust: BTreeMap<String, f64>,
    /// Evidence the model itself brings: what is assumed or declared.
    pub declared: Vec<Evidence>,
    /// The boundaries.
    pub boundaries: BTreeSet<String>,
    /// The `represented` claims: name, function, the spec of the set, line.
    pub represented: Vec<(String, String, String, usize)>,
}

/// A name bound by a claim, its set if one is given, and its line.
type Bound = (String, Option<Set>, usize);

/// A token of the parts of the reader that match on its kind.
enum Old {
    Ident(String),
    Num(String),
    Str(String),
}

/// Reserved words: never names.
pub const KEYWORDS: [&str; 36] = [
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
    "fn",
    "claim",
    "evidence",
    "correct",
    "if",
    "else",
    "let",
    "true",
    "false",
    "in",
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
];

/// The text of a string as written between quotes.
fn unquote(raw: &str) -> String {
    let (mut out, mut chars) = (String::new(), raw[1..raw.len() - 1].chars());
    while let Some(c) = chars.next() {
        out.push(match c {
            '\\' => match chars.next() {
                Some('n') => '\n',
                Some('t') => '\t',
                Some(c) => c,
                None => '\\',
            },
            c => c,
        });
    }
    out
}

/// Tokens as one line of text.
fn join(toks: &[Tok]) -> String {
    let each: Vec<&str> = toks.iter().map(|t| t.text.as_str()).collect();
    each.join(" ")
}

#[derive(Default)]
struct P {
    toks: Vec<Tok>,
    pos: usize,
    /// The subject or boundary being read.
    scope: Option<String>,
    /// Every function and claim of the file, by full name.
    all: BTreeSet<String>,
    /// How many fields each variant has.
    variants: BTreeMap<String, usize>,
    /// The names bound where the parser is.
    locals: Vec<String>,
    /// Inside the body of a subject's function: its name, and its spec.
    own: Option<(String, String)>,
    /// The measures met since last cleared.
    measures: Vec<Expr>,
    names: Names,
    fresh: usize,
    /// The names of a first reading of the whole file.
    known: Names,
    /// The spec claims of the functions called since last cleared.
    called: Vec<String>,
    /// Whether this is the first reading.
    first: bool,
    /// Whether an expression read the code or a measure, since last cleared.
    impure: bool,
    /// The claims with parameters: their parameters and their body.
    templates: BTreeMap<String, (Vec<String>, Vec<Tok>)>,
    /// The modules being imported, outermost first.
    stack: Vec<String>,
    /// A claim of the shape "the same judgement on every input of a set
    /// or of a class" is being read as that set or those classes.
    shape: bool,
    /// The line every declaration of an imported module is given.
    at: Option<usize>,
    /// The model being built.
    m: Model,
    /// The property claims, each with the names it binds.
    properties: Vec<(usize, Vec<Bound>)>,
    /// The records and enums declared so far.
    records: Vec<Record>,
    enums: Vec<Enum>,
    /// The sets the type parameters stand for, where the parser is.
    generics: Vec<(String, Set)>,
    /// Sets written with an enum, by their printed form.
    aliases: Vec<(String, String)>,
    /// The names used as a record with a set for each field.
    used: Vec<String>,
    /// Whether the last `args` were written with parentheses: `f()`.
    parens: bool,
    /// The record or enum being declared, and how it is written with its
    /// parameters: `Tree<Int>`. Its own name, inside, is the type itself.
    me: Option<(String, String)>,
    /// Whether this reads the body of an enum.
    in_enum: bool,
    /// How deep inside a value the parser is.
    nested: usize,
    /// The functions declared so far: a function type holds those that
    /// fit, and the name of one is a value.
    functions: Vec<Function>,
    /// Whether this reads the parameters of a function type.
    contra: bool,
    /// Whether a type named itself since this was last cleared, and
    /// whether it did so as the whole type of a field.
    met: bool,
    direct: bool,
    /// A set can't hold a type that contains itself. Read plainly, the
    /// type inside itself is `Value`; read `deep`, it is a mark,
    /// `$Tree<Int>`, and `recursive` gives the set each mark stands for.
    deep: bool,
    recursive: BTreeMap<String, Set>,
}

type R<T> = Result<T, Error>;

fn bin(op: &'static str, a: Expr, b: Expr) -> Expr {
    Expr::Binary(op, Box::new(a), Box::new(b))
}

impl P {
    fn line(&self) -> usize {
        let last = self.toks.last().map_or(1, |t| t.line);
        self.toks.get(self.pos).map_or(last, |t| t.line)
    }
    fn err<T>(&self, msg: impl Into<String>) -> R<T> {
        Err(Error::new(self.line(), msg.into()))
    }
    fn peek(&self) -> &str {
        self.toks.get(self.pos).map_or("", |t| t.text.as_str())
    }
    fn peek_at(&self, n: usize) -> &str {
        self.toks.get(self.pos + n).map_or("", |t| t.text.as_str())
    }
    fn kind(&self) -> Option<Kind> {
        self.toks.get(self.pos).map(|t| t.kind.clone())
    }
    fn kind_at(&self, n: usize) -> Option<Kind> {
        self.toks.get(self.pos + n).map(|t| t.kind.clone())
    }
    fn is(&self, text: &str) -> bool {
        self.peek() == text && self.kind() != Some(Kind::Str)
    }
    fn eat(&mut self, text: &str) -> bool {
        let yes = self.is(text);
        self.pos += yes as usize;
        yes
    }
    fn expect(&mut self, text: &str) -> R<()> {
        match self.eat(text) {
            true => Ok(()),
            false => {
                let got = match self.peek() {
                    "" => "the end".to_string(),
                    t => format!("`{t}`"),
                };
                self.err(format!("expected `{text}`, got {got}"))
            }
        }
    }
    fn name(&mut self) -> R<String> {
        match self.kind() {
            Some(Kind::Name) => {
                self.pos += 1;
                Ok(self.toks[self.pos - 1].text.clone())
            }
            _ => self.err("expected a name"),
        }
    }
    /// `a::b::c`
    fn path(&mut self) -> R<String> {
        let mut path = self.name()?;
        while self.eat("::") {
            path = format!("{path}::{}", self.name()?);
        }
        Ok(path)
    }
    fn qualified(&self, name: &str) -> String {
        match &self.scope {
            Some(s) => format!("{s}::{name}"),
            None => name.to_string(),
        }
    }
    /// A function or claim, as named from where the parser is.
    fn resolve(&self, name: &str) -> String {
        if let Some((_, spec)) = self.own.as_ref().filter(|o| o.0 == name) {
            return spec.clone();
        }
        let scoped = self.qualified(name);
        let full = match self.all.contains(&scoped) {
            true => scoped,
            false => name.to_string(),
        };
        // Inside a spec, another function of a subject is its spec.
        match self
            .known
            .specified
            .get(&full)
            .filter(|_| self.own.is_some())
        {
            Some(spec) => spec.clone(),
            None => full,
        }
    }
    /// A function or claim named in evidence: its full name, or its last
    /// part when only one has it.
    fn find(&self, name: &str) -> R<String> {
        if self.all.contains(name) {
            return Ok(name.to_string());
        }
        let tail = format!("::{name}");
        let found: Vec<&String> = self.all.iter().filter(|n| n.ends_with(&tail)).collect();
        match found[..] {
            [one] => Ok(one.clone()),
            [] => self.err(format!("the model has no function or claim `{name}`")),
            _ => {
                let each: Vec<String> = found.iter().map(|n| format!("`{n}`")).collect();
                self.err(format!(
                    "`{name}` is one of {}: write which",
                    each.join(", ")
                ))
            }
        }
    }

    fn old(&self) -> Option<Old> {
        let t = self.toks.get(self.pos)?;
        Some(match t.kind {
            Kind::Name => Old::Ident(t.text.clone()),
            Kind::Num => Old::Num(t.text.clone()),
            Kind::Str => Old::Str(unquote(&t.text)),
            Kind::Punct => return None,
        })
    }
    fn next_old(&mut self) -> Option<Old> {
        let t = self.old();
        self.pos += 1;
        t
    }
    fn at_end(&self) -> bool {
        self.pos >= self.toks.len()
    }
    fn is_name(&self) -> bool {
        self.kind() == Some(Kind::Name) && !KEYWORDS.contains(&self.peek())
    }
    /// A name that is no reserved word.
    fn ident(&mut self) -> R<String> {
        match self.kind() {
            Some(Kind::Name) if self.is_name() => self.name(),
            Some(Kind::Name) => {
                self.err(format!("`{}` is a keyword, expected a name", self.peek()))
            }
            _ => self.err("expected a name"),
        }
    }
    fn is_keyword(&self, k: &str) -> bool {
        self.kind() == Some(Kind::Name) && self.peek() == k
    }
    fn eat_keyword(&mut self, k: &str) -> bool {
        let yes = self.is_keyword(k);
        self.pos += yes as usize;
        yes
    }

    // ---- values and sets ----

    /// A weight or a confidence: `1`, `0.5`.
    fn number(&mut self) -> R<f64> {
        match self.old() {
            Some(Old::Num(s)) => {
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
    fn signed(&mut self, negative: bool) -> R<i64> {
        match self.old() {
            Some(Old::Num(s)) if !s.contains('.') => {
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
    fn numeral(&mut self, negative: bool) -> R<Value> {
        match self.old() {
            Some(Old::Num(s)) if s.contains('.') => match s.parse::<f64>() {
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
    fn values(&mut self) -> R<Vec<Value>> {
        let mut parts = vec![self.value()?];
        while self.eat(",") {
            parts.push(self.value()?);
        }
        self.expect(")")?;
        Ok(parts)
    }
    /// value := "true" | "false" | ["-"] number | string
    ///        | "(" value "," value {"," value} ")" | name "(" value {"," value} ")"
    fn value(&mut self) -> R<Value> {
        // A value inside a value, and so on: not without end.
        const DEEPEST: usize = 64;
        if self.nested == DEEPEST {
            return self.err(format!("a value nested more than {DEEPEST} deep"));
        }
        self.nested += 1;
        let v = self.value_inner();
        self.nested -= 1;
        v
    }
    fn value_inner(&mut self) -> R<Value> {
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
            let name = self.path()?;
            let scoped = self.qualified(&name);
            let name = match self.all.contains(&scoped) {
                true => scoped,
                false => name,
            };
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
        match self.old() {
            Some(Old::Ident(s)) if s == "true" || s == "false" => {
                self.pos += 1;
                Ok(Value::Bool(s == "true"))
            }
            Some(Old::Num(_)) => self.numeral(false),
            Some(Old::Str(s)) => {
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
    fn bound(&mut self) -> R<Option<i128>> {
        let negative = self.eat("-");
        match self.old() {
            Some(Old::Num(n)) if n.contains('.') => {
                self.err("ranges are over integers: write floats as a set, `{1.5, 2.5}`")
            }
            Some(Old::Num(n)) => {
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
    fn set(&mut self) -> R<Set> {
        let mut s = self.set_inter()?;
        while self.eat("|") || self.eat("∪") {
            s = s.union(&self.set_inter()?);
        }
        Ok(s)
    }
    /// inter := atom { ("&" | "\\") atom }, left to right.
    fn set_inter(&mut self) -> R<Set> {
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
    /// `()`, `(S)`, `(S, T)`: the sets between parentheses.
    fn sets_in_parentheses(&mut self) -> R<Vec<Set>> {
        self.expect("(")?;
        let mut parts = Vec::new();
        if self.eat(")") {
            return Ok(parts);
        }
        parts.push(self.set()?);
        while self.eat(",") {
            parts.push(self.set()?);
        }
        self.expect(")")?;
        Ok(parts)
    }
    fn set_atom(&mut self) -> R<Set> {
        if self.is("(") {
            // `()` is the unit type; `(S)` groups; `(S, T)` is the tuples
            // of one value from each.
            let mut parts = self.sets_in_parentheses()?;
            return Ok(match parts.len() {
                1 => parts.remove(0),
                _ => Set::tuple(parts),
            });
        }
        if self.eat_keyword("fn") {
            // `fn(A, B) -> C`: the functions declared above that take
            // an A and a B, or more, and give a C. A type parameter
            // is any value in a result, and no value at all in a
            // parameter, so that every function fits.
            let start = self.pos;
            let mut parts = self.sets_in_parentheses()?;
            if !self.generics.is_empty() {
                self.pos = start;
                self.contra = !self.contra;
                let again = self.sets_in_parentheses();
                self.contra = !self.contra;
                parts = again?;
            }
            self.expect("->")?;
            let result = self.set_atom()?;
            let fits = |f: &&Function| {
                f.params.len() == parts.len()
                    && f.params.iter().zip(&parts).all(|(p, a)| a.is_subset(&p.1))
                    && f.range.is_subset(&result)
            };
            let named = |f: &Function| Set::record(&f.name, Vec::new());
            let all = self.functions.iter().filter(fits).map(named);
            let set = all.fold(Set::empty(), |s, f| s.union(&f));
            let each: Vec<String> = parts.iter().map(|p| p.to_string()).collect();
            let written = format!("fn({}) -> {}", each.join(", "), crate::set::paren(&result));
            if !set.is_empty() {
                self.aliases.push((set.to_string(), written));
            }
            return Ok(set);
        }
        // Rust's names for the same types.
        let rust = match self.peek() {
            "bool" => "Bool",
            "i8" => "Int8",
            "i16" => "Int16",
            "i32" => "Int32",
            "i64" => "Int64",
            "u8" => "UInt8",
            "u16" => "UInt16",
            "u32" => "UInt32",
            "f64" => "Float64",
            "str" => "String",
            "Vec" => "List",
            "HashSet" | "BTreeSet" => "Set",
            "HashMap" | "BTreeMap" => "Map",
            _ => "",
        };
        if !rust.is_empty() && self.kind() == Some(Kind::Name) {
            self.toks[self.pos].text = rust.to_string();
        }
        if self.eat_keyword("u64") || self.eat_keyword("usize") {
            return Ok(Set::range(Some(0), None));
        }
        if self.eat_keyword("List") {
            // `List<T>`: the lists whose elements are all in T.
            self.expect("<")?;
            let elements = self.set()?;
            self.expect(">")?;
            return Ok(Set::list(elements));
        }
        if self.eat_keyword("Set") {
            self.expect("<")?;
            let elements = self.set()?;
            self.expect(">")?;
            return Ok(Set::sets(elements));
        }
        if self.eat_keyword("Map") {
            self.expect("<")?;
            // `Map<K, V>`; with one set, its `(key, value)` pairs.
            let keys = self.set()?;
            if self.eat(">") {
                return Ok(Set::record(MAP, vec![Set::list(keys)]));
            }
            self.expect(",")?;
            let values = self.set()?;
            self.expect(">")?;
            return Ok(Set::maps(keys, values));
        }
        if self.is_name() {
            // `Number`: every value of the record. `Number(S, T)`: those
            // with one field in each set.
            let line = self.line();
            let name = self.ident()?;
            if let Some((_, set)) = self.generics.iter().find(|p| p.0 == name) {
                return Ok(match self.contra {
                    true => Set::empty(),
                    false => set.clone(),
                });
            }
            let mut args = Vec::new();
            if self.eat("<") {
                args.push(self.set()?);
                while self.eat(",") {
                    args.push(self.set()?);
                }
                self.expect(">")?;
            }
            let written = |args: &[Set]| {
                let each: Vec<String> = args.iter().map(|a| a.to_string()).collect();
                match args.is_empty() {
                    true => name.clone(),
                    false => format!("{name}<{}>", each.join(", ")),
                }
            };
            // A record's name with sets, `Dir(String, S)`, is not the
            // record itself: only an enum takes parameters.
            let own = |me: &(String, String)| me.0 == name && (self.in_enum || args.is_empty());
            if let Some((_, me)) = self.me.clone().filter(own) {
                // The type inside its own declaration.
                if written(&args) != me {
                    return Err(Error::new(
                        line,
                        format!("inside its declaration, `{name}` is written `{me}`"),
                    ));
                }
                self.met = true;
                return Ok(match self.deep {
                    true => Set::record(&format!("${me}"), Vec::new()),
                    false => Set::any(),
                });
            }
            // `Option(Int)`: an enum, with a set for each of its parameters.
            if let Some(e) = self.enums.iter().find(|e| e.name == name).cloned() {
                if args.len() != e.params.len() {
                    return Err(Error::new(
                        line,
                        format!(
                            "`{name}` takes {} type(s), got {}",
                            e.params.len(),
                            args.len()
                        ),
                    ));
                }
                let written = written(&args);
                let params = e.params.iter().cloned().zip(args).collect();
                let (variants, _, recursive) = self.variants(&e, params, line)?;
                let set = variants.iter().fold(Set::empty(), |s, v| s.union(&v.set()));
                self.aliases.push((set.to_string(), written.clone()));
                // Under its plain name too: the printer names it.
                if let Some(recursive) = recursive {
                    self.recursive.extend(recursive);
                    self.recursive.insert(written, set.clone());
                }
                return Ok(set);
            }
            if !args.is_empty() {
                self.used.push(name.clone());
                return Ok(Set::record(&name, args));
            }
            if let Some(set) = self.recursive.get(&name).filter(|_| self.deep) {
                return Ok(set.clone());
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
        if self.kind() == Some(Kind::Num) || self.is("-") || self.is("..") || self.is("..=") {
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
        let set = match self.old() {
            Some(Old::Ident(t)) => match t.as_str() {
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
    ) -> R<(Vec<Record>, Vec<String>, Option<BTreeMap<String, Set>>)> {
        let at = |e: Error| Error::new(line, e.message);
        let mut p = P {
            toks: lex(&of.body).map_err(at)?,
            ..P::default()
        };
        let each: Vec<String> = params.iter().map(|p| p.1.to_string()).collect();
        let written = match each.is_empty() {
            true => of.name.clone(),
            false => format!("{}<{}>", of.name, each.join(", ")),
        };
        p.me = Some((of.name.clone(), written));
        p.in_enum = true;
        p.deep = self.deep;
        p.recursive = self.recursive.clone();
        let (mut closed, mut holds_itself) = (false, false);
        // The enums declared before this one.
        p.records = self.records.clone();
        p.functions = self.functions.clone();
        p.contra = self.contra;
        let before = self.enums.iter().take_while(|e| e.name != of.name);
        p.enums = before.cloned().collect();
        p.generics = params;
        let mut out = Vec::new();
        loop {
            let name = p.ident().map_err(at)?;
            p.met = false;
            p.direct = false;
            let fields = match p.is("(") {
                true => p.binders("(", ")").map_err(at)?,
                false => Vec::new(),
            };
            closed |= !p.direct;
            holds_itself |= p.met;
            if fields.is_empty()
                && p.toks
                    .get(p.pos.wrapping_sub(1))
                    .is_some_and(|t| t.text == ")")
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
                return match closed {
                    false => Err(Error::new(
                        line,
                        format!(
                            "every variant of `{0}` holds a `{0}`: no value can be built",
                            of.name
                        ),
                    )),
                    true => Ok((out, p.used, holds_itself.then_some(p.recursive))),
                };
            }
            p.expect(",").map_err(at)?;
        }
    }
    /// Reads the same types twice: plainly, then deep.
    fn both(
        &mut self,
        read: impl Fn(&mut P) -> R<Vec<(String, Set)>>,
    ) -> R<(Vec<(String, Set)>, Vec<Set>)> {
        let start = self.pos;
        let plain = read(self)?;
        self.pos = start;
        self.deep = true;
        let deep = read(self);
        self.deep = false;
        Ok((plain, deep?.into_iter().map(|p| p.1).collect()))
    }
    /// A set read plainly, then deep.
    fn set_both(&mut self) -> R<(Set, Set)> {
        let both = self.both(|p| Ok(vec![(String::new(), p.set()?)]))?;
        Ok((both.0[0].1.clone(), both.1[0].clone()))
    }
    /// `(a: Int, c: Bool)`, or `()`; a record's fields between braces.
    fn binders(&mut self, open: &str, close: &str) -> R<Vec<(String, Set)>> {
        let mut params = Vec::new();
        self.expect(open)?;
        if self.eat(close) {
            return Ok(params);
        }
        loop {
            let name = self.ident()?;
            if !self.is(":") {
                return self.err(format!("the input `{name}` needs a type"));
            }
            self.expect(":")?;
            let before = std::mem::take(&mut self.met);
            let set = self.set()?;
            // The type itself, alone: a value would need a value.
            let mark = self
                .me
                .as_ref()
                .map(|me| Set::record(&format!("${}", me.1), Vec::new()));
            self.direct |= self.met && (set == Set::any() || Some(&set) == mark.as_ref());
            self.met |= before;
            params.push((name, set));
            if !self.eat(",") || self.is(close) {
                self.expect(close)?;
                return Ok(params);
            }
        }
    }
    /// The arguments of a call in a claim or evidence: bound names and
    /// values, as a pattern. Every bound name is used exactly once.
    fn args(&mut self, bound: &[(String, Option<Set>, usize)]) -> R<Pattern> {
        let mut pattern = Vec::new();
        // `(a, 1, -2)`; no parentheses when there are no arguments.
        self.parens = self.eat("(");
        let mut more = self.parens && !self.eat(")");
        while more {
            // A name followed by `(` is a record; a name that no `forall`
            // binds is a variant without fields, when one is declared.
            let variant = |x: &String| {
                // A variant without fields, or a function, as a value.
                !bound.iter().any(|b| &b.0 == x)
                    && (self.functions.iter().any(|f| &f.name == x)
                        || self
                            .records
                            .iter()
                            .any(|r| &r.name == x && r.fields.is_empty()))
            };
            let record =
                self.is_name() && (self.peek_at(1) == "(" || variant(&self.peek().to_string()));
            let value = matches!(self.kind(), Some(Kind::Num | Kind::Str))
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
                    return at(format!("`{x}` is not an input of the claim"));
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
            return Err(Error::new(*line, format!("the input `{x}` is not used")));
        }
        Ok(pattern)
    }

    // ---- expressions ----

    fn expr(&mut self) -> R<Expr> {
        let mut a = self.or()?;
        while self.eat("<=>") {
            a = bin("==", a, self.or()?);
        }
        Ok(a)
    }
    fn or(&mut self) -> R<Expr> {
        let mut a = self.and()?;
        while self.eat("||") {
            a = bin("||", a, self.and()?);
        }
        Ok(a)
    }
    fn and(&mut self) -> R<Expr> {
        let mut a = self.compare()?;
        while self.eat("&&") {
            a = bin("&&", a, self.compare()?);
        }
        Ok(a)
    }
    fn compare(&mut self) -> R<Expr> {
        let mut a = self.sum()?;
        loop {
            if self.eat("is") {
                // `x is Err`: of that variant.
                let v = self.name()?;
                let Some(n) = self.variants.get(&v) else {
                    return self.err(format!("`is` needs a variant of an enum, got `{v}`"));
                };
                let fields: Vec<String> = (0..*n).map(|i| format!("is_{i}")).collect();
                let arm = |variant, b| Arm {
                    variant,
                    body: Expr::Lit(Value::Bool(b)),
                };
                let arms = vec![arm(Some((v, fields)), true), arm(None, false)];
                a = Expr::Match(Box::new(a), arms);
                continue;
            }
            const OPS: [&str; 7] = ["==", "!=", "<=", ">=", "<", ">", "in"];
            let Some(op) = OPS.iter().find(|op| self.is(op)) else {
                return Ok(a);
            };
            self.pos += 1;
            a = bin(op, a, self.sum()?);
        }
    }
    fn sum(&mut self) -> R<Expr> {
        let mut a = self.product()?;
        loop {
            // `|` and `\` on sets and maps bind like `+`, `&` like `*`.
            const OPS: [&str; 4] = ["+", "-", "|", "\\"];
            let Some(op) = OPS.iter().find(|op| self.is(op)) else {
                return Ok(a);
            };
            self.pos += 1;
            a = bin(op, a, self.product()?);
        }
    }
    fn product(&mut self) -> R<Expr> {
        let mut a = self.unary()?;
        loop {
            const OPS: [&str; 3] = ["*", "/", "&"];
            let Some(op) = OPS.iter().find(|op| self.is(op)) else {
                return Ok(a);
            };
            self.pos += 1;
            a = bin(op, a, self.unary()?);
        }
    }
    fn unary(&mut self) -> R<Expr> {
        if self.eat("!") {
            return Ok(Expr::Unary("!", Box::new(self.unary()?)));
        }
        // `-` right before digits is a negative literal, down to i64::MIN.
        let negative = self.eat("-");
        let digits = self.kind() == Some(Kind::Num)
            && (self.peek().contains('.') || !self.peek().trim_start_matches('0').is_empty());
        if negative && !digits {
            return Ok(Expr::Unary("-", Box::new(self.unary()?)));
        }
        let mut a = match negative {
            true => Expr::Lit(self.numeral(true)?),
            false => self.primary()?,
        };
        // `.field` on a record, `.0` on a tuple, `[i]` on a list.
        loop {
            if self.eat("[") {
                let i = self.expr()?;
                self.expect("]")?;
                a = Expr::Index(Box::new(a), Box::new(i));
            } else if self.eat(".") {
                let position = self.kind() == Some(Kind::Num) && !self.peek().contains('.');
                if !position && !self.is_name() {
                    return self.err(
                        "expected a field name, or a position such as `.0`; \
                         for a position inside a position write `(t.0).1`",
                    );
                }
                a = Expr::Field(Box::new(a), self.peek().to_string());
                self.pos += 1;
            } else {
                return Ok(a);
            }
        }
    }
    fn exprs(&mut self, close: &str) -> R<Vec<Expr>> {
        let mut out = Vec::new();
        while !self.eat(close) {
            out.push(self.expr()?);
            if !self.eat(",") {
                self.expect(close)?;
                break;
            }
        }
        Ok(out)
    }
    /// `{ let x = e; … result }`
    fn block(&mut self) -> R<Expr> {
        self.expect("{")?;
        let out = self.sequence()?;
        self.expect("}")?;
        Ok(out)
    }
    /// `let x = e; … result`, without the braces.
    fn sequence(&mut self) -> R<Expr> {
        if !self.eat("let") {
            return self.expr();
        }
        let x = self.ident()?;
        self.expect("=")?;
        let v = self.expr()?;
        self.expect(";")?;
        self.locals.push(x.clone());
        let body = self.sequence();
        self.locals.pop();
        Ok(Expr::Let(x, Box::new(v), Box::new(body?)))
    }
    /// `|a, b|`, after the first `|`.
    fn bars(&mut self) -> R<Vec<String>> {
        let mut names = Vec::new();
        while !self.eat("|") {
            names.push(self.ident()?);
            if !self.eat(",") {
                self.expect("|")?;
                break;
            }
        }
        Ok(names)
    }
    fn primary(&mut self) -> R<Expr> {
        match self.kind() {
            Some(Kind::Num) => return Ok(Expr::Lit(self.numeral(false)?)),
            Some(Kind::Str) => {
                self.pos += 1;
                return Ok(Expr::Lit(Value::Str(unquote(
                    &self.toks[self.pos - 1].text,
                ))));
            }
            None => return self.err("expected an expression"),
            _ => {}
        }
        if self.is("(") && self.peek_at(1) == "|" {
            // (|a, b| a + b)(1, 2): an inline function, on its inputs.
            self.pos += 2;
            let names = self.bars()?;
            let n = self.locals.len();
            self.locals.extend(names.iter().cloned());
            let body = self.expr();
            self.locals.truncate(n);
            let mut body = body?;
            self.expect(")")?;
            if !self.eat("(") {
                return self.err("an inline function is given its inputs here: `(|a| a + 1)(2)`");
            }
            let args = self.exprs(")")?;
            if args.len() != names.len() {
                let msg = format!(
                    "the function takes {} input(s), got {}",
                    names.len(),
                    args.len()
                );
                return self.err(msg);
            }
            // The inputs are worked out before any takes its name.
            let pairs: Vec<(String, Expr)> = names
                .iter()
                .cloned()
                .zip(args)
                .filter(|(x, a)| a != &Expr::Var(x.clone()))
                .collect();
            let reads = |a: &Expr| free(a, &mut Vec::new(), &mut Vec::new());
            let mixed = pairs
                .iter()
                .any(|(_, a)| reads(a).iter().any(|x| names.contains(x)));
            let mut held = Vec::new();
            for (x, a) in pairs.into_iter().rev() {
                if mixed {
                    self.fresh += 1;
                    let h = format!("held_{}", self.fresh);
                    body = Expr::Let(x, Box::new(Expr::Var(h.clone())), Box::new(body));
                    held.push((h, a));
                } else {
                    body = Expr::Let(x, Box::new(a), Box::new(body));
                }
            }
            for (h, a) in held {
                body = Expr::Let(h, Box::new(a), Box::new(body));
            }
            return Ok(body);
        }
        if self.eat("(") {
            // `(e)` groups; `(a, b)` is a tuple.
            let mut parts = self.exprs(")")?;
            return Ok(match parts.len() {
                1 => parts.remove(0),
                _ => Expr::Tuple(parts),
            });
        }
        if self.eat("[") {
            return Ok(Expr::List(self.exprs("]")?));
        }
        if self.eat("{") {
            // A set `{a, b}` or a map `{k: v}`; `{}` and `{:}` are empty.
            // Both are records of one list, put in order when evaluated.
            let of = |name: &str, items| Expr::Record(name.into(), vec![Expr::List(items)]);
            if self.eat(":") {
                self.expect("}")?;
                return Ok(of(MAP, Vec::new()));
            }
            if self.is_name() && self.peek_at(1) == "in" {
                // { x in xs | cond }: the elements where cond holds.
                let x = self.name()?;
                self.pos += 1;
                // `|` joins two sets: one written here goes in parentheses.
                let from = self.product()?;
                self.expect("|")?;
                self.locals.push(x.clone());
                let cond = self.expr();
                self.locals.pop();
                self.expect("}")?;
                return Ok(Expr::Each("filter", x, Box::new(from), Box::new(cond?)));
            }
            let (mut items, mut map) = (Vec::new(), false);
            while !self.eat("}") {
                let item = self.expr()?;
                map |= items.is_empty() && self.is(":");
                items.push(match map {
                    true => {
                        self.expect(":")?;
                        Expr::Tuple(vec![item, self.expr()?])
                    }
                    false => item,
                });
                if !self.eat(",") {
                    self.expect("}")?;
                    break;
                }
            }
            return Ok(of(if map { MAP } else { SET }, items));
        }
        if self.is("let") {
            return self.sequence();
        }
        if self.eat("if") {
            let c = self.expr()?;
            let t = self.block()?;
            if !self.eat("else") {
                return self.err("`if` needs an `else`: it gives a value");
            }
            let e = match self.is("if") {
                true => self.primary()?,
                false => self.block()?,
            };
            return Ok(Expr::If(Box::new(c), Box::new(t), Box::new(e)));
        }
        if self.eat("match") {
            // match e { None => a, Some(v) => b, _ => c }
            let e = self.expr()?;
            self.expect("{")?;
            let mut arms = Vec::new();
            while !self.eat("}") {
                let n = self.locals.len();
                let variant = match self.eat("_") {
                    true => None,
                    false => {
                        let v = self.ident()?;
                        let mut fields = Vec::new();
                        if self.eat("(") {
                            while !self.eat(")") {
                                // `_` for a field nobody reads.
                                let x = match self.eat("_") {
                                    true => {
                                        self.fresh += 1;
                                        format!("unread_{}", self.fresh)
                                    }
                                    false => self.ident()?,
                                };
                                fields.push(x);
                                if !self.eat(",") {
                                    self.expect(")")?;
                                    break;
                                }
                            }
                        }
                        self.locals.extend(fields.iter().cloned());
                        Some((v, fields))
                    }
                };
                self.expect("=>")?;
                let body = match self.is("{") && self.peek_at(1) == "let" {
                    true => self.block(),
                    false => self.expr(),
                };
                self.locals.truncate(n);
                arms.push(Arm {
                    variant,
                    body: body?,
                });
                if !self.eat(",") {
                    self.expect("}")?;
                    break;
                }
            }
            return Ok(Expr::Match(Box::new(e), arms));
        }
        if self.is("correct") && self.peek_at(1) == "(" {
            // correct(f(a, b)) as a value: what the evidence says of the call.
            self.pos += 2;
            let f = self.path()?;
            let f = self.resolve(&f);
            self.expect("(")?;
            let args = self.exprs(")")?;
            self.expect(")")?;
            self.impure = true;
            return Ok(Expr::Correct(f, args));
        }
        if self.eat("true") {
            return Ok(Expr::Lit(Value::Bool(true)));
        }
        if self.eat("false") {
            return Ok(Expr::Lit(Value::Bool(false)));
        }
        if self.is("rate") && self.peek_at(1) == "(" {
            // rate(x in Type, cond): the share of the type where cond holds.
            self.pos += 2;
            let x = self.ident()?;
            self.expect("in")?;
            let set = self.set()?;
            self.expect(",")?;
            self.locals.push(x.clone());
            let cond = self.expr();
            self.locals.pop();
            self.expect(")")?;
            let measure = Expr::Rate(x, set, Box::new(cond?));
            self.measures.push(measure.clone());
            self.impure = true;
            return Ok(measure);
        }
        if let Some(kind) = SCORES.iter().find(|k| self.is(k) && self.peek_at(1) == "(") {
            self.pos += 2;
            let f = self.path()?;
            self.expect(")")?;
            self.impure = true;
            let measure = Expr::Score(kind, self.resolve(&f));
            self.measures.push(measure.clone());
            return Ok(measure);
        }
        let each = EACH.iter().find(|k| self.is(k));
        if let Some(kind) = each.filter(|_| self.peek_at(1) == "(" && self.peek_at(3) == "in") {
            // all(x in xs, cond): over each element.
            self.pos += 2;
            let x = self.ident()?;
            self.expect("in")?;
            let list = self.sum()?;
            self.expect(",")?;
            self.locals.push(x.clone());
            let body = self.sequence();
            self.locals.pop();
            self.expect(")")?;
            let body = body?;
            if *kind == "fold" && !matches!(body, Expr::Let(..)) {
                return self
                    .err("`fold` starts from a value: `fold(x in xs, let sum = 0; sum + x)`");
            }
            return Ok(Expr::Each(kind, x, Box::new(list), Box::new(body)));
        }
        for (word, len) in [("len", true), ("keys", false)] {
            if self.is(word) && self.peek_at(1) == "(" {
                self.pos += 2;
                let e = Box::new(self.expr()?);
                self.expect(")")?;
                return Ok(match len {
                    true => Expr::Len(e),
                    false => Expr::Unary("keys", e),
                });
            }
        }
        if !self.is_name() {
            return self.err("expected an expression");
        }
        let name = self.path()?;
        let local = self.locals.contains(&name);
        let name = match local {
            true => name,
            false => self.resolve(&name),
        };
        self.impure |= !local && self.known.opaque.contains(&name);
        if !local && self.own.is_none() && self.known.specified.contains_key(&name) {
            self.called.push(format!("{name}::spec"));
        }
        if self.eat("(") {
            return Ok(Expr::Call(name, self.exprs(")")?));
        }
        Ok(Expr::Var(name))
    }

    /// `trust 0.8`, when written here.
    fn trust(&mut self) -> R<Option<f64>> {
        if !(self.is("trust") && self.kind_at(1) == Some(Kind::Num)) {
            return Ok(None);
        }
        self.pos += 2;
        match self.toks[self.pos - 1].text.parse::<f64>() {
            Ok(t) if (0.0..=1.0).contains(&t) => Ok(Some(t)),
            _ => self.err("trust is a number from 0 to 1"),
        }
    }

    // ---- declarations ----

    /// A claim of the model being built.
    fn state(&mut self, name: &str, func: &str, pattern: Pattern, target: Target, line: usize) {
        self.m.claims.push(Claim {
            name: name.to_string(),
            func: func.to_string(),
            pattern,
            target,
            from: Vec::new(),
            auto: false,
            trust: None,
            line,
        });
        self.names
            .claims
            .entry(name.to_string())
            .or_insert((Vec::new(), Vec::new()));
    }
    /// A function of the model being built.
    fn define(&mut self, name: &str, sig: &Sig, range: Set, body: Option<Expr>, line: usize) {
        let function = Function {
            name: name.to_string(),
            params: sig.params.clone(),
            range,
            body,
            written: None,
            line,
        };
        self.m.deep.insert(name.to_string(), sig.deep.clone());
        self.functions.push(function.clone());
        self.m.functions.push(function);
    }
    /// What the model itself brings as evidence on one of its claims.
    fn declare(&mut self, claim: &str, source: &str, trust: f64) {
        self.names.declared.push(Evidence {
            about: claim.to_string(),
            pattern: Vec::new(),
            judgment: Judgment::Holds(true),
            confidence: trust,
            source: Some(source.to_string()),
            rev: None,
            line: 0,
        });
    }
    /// `<T, U>` after a name.
    fn type_params(&mut self, of: &str) -> R<Vec<String>> {
        let mut names = Vec::new();
        if !self.eat("<") {
            return Ok(names);
        }
        loop {
            let t = self.ident()?;
            if names.contains(&t) {
                return self.err(format!("`{of}` has two parameters `{t}`"));
            }
            names.push(t);
            if !self.eat(",") {
                self.expect(">")?;
                return Ok(names);
            }
        }
    }
    /// The tokens of a type, up to `,` or `close` outside any bracket.
    fn type_tokens(&mut self, close: &str) -> R<Vec<Tok>> {
        let (start, mut depth) = (self.pos, 0);
        loop {
            let Some(t) = self.toks.get(self.pos).filter(|t| t.kind != Kind::Str) else {
                return self.err(format!("expected `{close}`"));
            };
            let t = t.text.as_str();
            if depth == 0 && (t == "," || t == close) {
                return Ok(self.toks[start..self.pos].to_vec());
            }
            depth += ["(", "<", "{", "["].contains(&t) as i32;
            depth -= [")", ">", "}", "]"].contains(&t) as i32;
            self.pos += 1;
        }
    }
    /// An enum, from its name, its parameters and its variants as
    /// `Name(field: T, …), …`.
    fn enumeration(
        &mut self,
        name: String,
        params: Vec<String>,
        body: String,
        line: usize,
    ) -> R<()> {
        if self.used.contains(&name) {
            return Err(Error::new(
                line,
                format!("`{name}` is used above: declare an enum before its first use"),
            ));
        }
        // Checked once with every parameter as `Value`.
        let any = params.iter().map(|x| (x.clone(), Set::any())).collect();
        let mut e = Enum {
            name: name.clone(),
            params,
            body,
            variants: Vec::new(),
            line,
        };
        let (variants, used, _) = self.variants(&e, any, line)?;
        self.used.extend(used);
        e.variants = variants.iter().map(|v| v.name.clone()).collect();
        for mut v in variants {
            v.of = Some(name.clone());
            self.variants.insert(v.name.clone(), v.fields.len());
            self.records.push(v.clone());
            self.m.records.push(v);
        }
        self.m.declared.push(Decl::Enum(self.m.enums.len()));
        self.enums.push(e.clone());
        self.m.enums.push(e);
        Ok(())
    }

    fn items(&mut self) -> R<()> {
        while self.pos < self.toks.len() && !self.is("}") {
            let line = self.at.unwrap_or(self.line());
            let kw = self.name()?;
            match kw.as_str() {
                "import" if self.scope.is_none() => {
                    // import a::b::name;  import a::b::{x, y};
                    let mut path = vec![self.name()?];
                    let mut names = Vec::new();
                    loop {
                        self.expect("::")?;
                        if self.eat("{") {
                            while !self.eat("}") {
                                names.push(self.name()?);
                                if !self.eat(",") {
                                    self.expect("}")?;
                                    break;
                                }
                            }
                            break;
                        }
                        let n = self.name()?;
                        if !self.is("::") {
                            names.push(n);
                            break;
                        }
                        path.push(n);
                    }
                    self.eat(";");
                    self.import(line, &path, &names)?;
                }
                "subject" | "boundary" if self.scope.is_none() => {
                    self.scope = Some(self.name()?);
                    if kw == "boundary" {
                        self.names.boundaries.extend(self.scope.clone());
                    }
                    self.expect("{")?;
                    self.items()?;
                    self.expect("}")?;
                    self.scope = None;
                }
                "subject" | "boundary" => {
                    self.pos -= 1;
                    return self.err(format!("a `{kw}` inside another is not supported yet"));
                }
                "record" => {
                    // record Name { field: T, … }
                    let name = self.ident()?;
                    let params = self.type_params(&name)?;
                    if !params.is_empty() {
                        // record Name<T> { field: T } is the enum Name<T>
                        // of one variant, Name(field: T).
                        self.expect("{")?;
                        let mut fields = Vec::new();
                        while !self.eat("}") {
                            let f = self.ident()?;
                            self.expect(":")?;
                            fields.push(format!("{f} : {}", join(&self.type_tokens("}")?)));
                            if !self.eat(",") {
                                self.expect("}")?;
                                break;
                            }
                        }
                        let body = format!("{name} ( {} )", fields.join(" , "));
                        self.enumeration(name, params, body, line)?;
                        continue;
                    }
                    self.me = Some((name.clone(), name.clone()));
                    self.met = false;
                    let both = self.both(|p| p.binders("{", "}"));
                    self.me = None;
                    let (fields, deep) = both?;
                    if self.met {
                        let mark = Set::record(&format!("${name}"), Vec::new());
                        if deep.contains(&mark) {
                            return Err(Error::new(
                                line,
                                format!("a field of `{name}` is a `{name}`: no value can be built"),
                            ));
                        }
                        self.recursive
                            .insert(name.clone(), Set::record(&name, deep.clone()));
                        self.m.deep.insert(format!("record {name}"), deep);
                    }
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
                    self.m.declared.push(Decl::Record(self.m.records.len()));
                    self.records.push(record.clone());
                    self.m.records.push(record);
                }
                "enum" => {
                    // enum Name<T> { Variant, Variant(T), Variant(field: T) }
                    let name = self.ident()?;
                    let params = self.type_params(&name)?;
                    self.expect("{")?;
                    let mut variants = Vec::new();
                    while !self.eat("}") {
                        let v = self.ident()?;
                        let mut fields = Vec::new();
                        let open = self.eat("(");
                        while open && !self.eat(")") {
                            // `Some(T)` or `Some(value: T)`.
                            let named = self.kind() == Some(Kind::Name) && self.peek_at(1) == ":";
                            let f = match named {
                                true => {
                                    self.pos += 2;
                                    self.toks[self.pos - 2].text.clone()
                                }
                                false => format!("v{}", fields.len()),
                            };
                            fields.push(format!("{f} : {}", join(&self.type_tokens(")")?)));
                            if !self.eat(",") {
                                self.expect(")")?;
                                break;
                            }
                        }
                        variants.push(match open {
                            true => format!("{v} ( {} )", fields.join(" , ")),
                            false => v,
                        });
                        if !self.eat(",") {
                            self.expect("}")?;
                            break;
                        }
                    }
                    self.enumeration(name, params, variants.join(" , "), line)?;
                }
                "test" => {
                    self.pos -= 1;
                    return self.err(
                        "a `test` block is not read any more: write the point as a claim, \
                         `claim t() { f(1) == 2 }`, and tie the real test to it with the \
                         comment `// mitra: evidence for t` above its `fn`",
                    );
                }
                "fn" => self.function(line)?,
                "claim" | "assume" => self.claim(line, kw == "assume")?,
                other => {
                    self.pos -= 1;
                    return self.err(format!(
                        "expected `import`, `subject`, `boundary`, `record`, `enum`, `fn`, \
                         `claim` or `assume`, got `{other}`"
                    ));
                }
            }
        }
        Ok(())
    }

    fn function(&mut self, line: usize) -> R<()> {
        let name = self.ident()?;
        let q = self.qualified(&name);
        // fn f<T>(x: T) -> T: each type parameter is any value.
        let head = self.pos;
        for t in self.type_params(&name)? {
            self.generics.push((t, Set::any()));
        }
        let read = self
            .both(|p| p.binders("(", ")"))
            .and_then(|(params, deep)| {
                let range = match self.eat("->") {
                    true => self.set_both()?,
                    false => (Set::tuple(Vec::new()), Set::tuple(Vec::new())),
                };
                Ok((params, deep, range))
            });
        let generic = !std::mem::take(&mut self.generics).is_empty();
        let (params, mut deep, (range, deep_range)) = read?;
        let written = generic.then(|| join(&self.toks[head..self.pos]));
        deep.push(deep_range);
        let sig = Sig { params, deep };
        // `uses a, b` or `uses only a, b`: what it calls.
        let mut uses = None;
        if self.eat("uses") {
            let only = self.eat("only");
            let mut used = vec![self.path()?];
            while self.eat(",") {
                used.push(self.path()?);
            }
            let used: Vec<String> = used.iter().map(|u| self.resolve(u)).collect();
            uses = Some((only, used, self.trust()?.unwrap_or(1.0)));
        }
        self.names.fns.insert(q.clone(), sig.clone());
        if self.is(";") || self.scope.is_some() {
            self.names.opaque.insert(q.clone());
        }
        if self.eat(";") {
            self.define(&q, &sig, range, None, line);
        } else {
            let in_subject = self.scope.is_some();
            let body_name = match in_subject {
                true => format!("{q}::body"),
                false => q.clone(),
            };
            self.locals = sig.params.iter().map(|p| p.0.clone()).collect();
            // In a subject, the body is the spec of the code: it calls
            // itself as a spec.
            self.own = Some((name.clone(), body_name.clone()));
            let body = self.block();
            self.own = None;
            self.locals.clear();
            let body = body?;
            if in_subject {
                let spec = format!("{q}::spec");
                self.names.specified.insert(q.clone(), body_name.clone());
                self.define(&q, &sig, range.clone(), None, line);
                self.define(&body_name, &sig, range, Some(body), line);
                let names = sig.params.iter().map(|p| &p.0);
                let pattern = names
                    .clone()
                    .map(|x| Position {
                        var: Some(x.clone()),
                        set: None,
                    })
                    .collect();
                let args = names.map(|x| Expr::Var(x.clone())).collect();
                let target = Target::Matches(Expr::Call(body_name, args));
                self.state(&spec, &q, pattern, target, line);
            } else {
                self.define(&q, &sig, range, Some(body), line);
            }
        }
        if let Some(f) = self.m.functions.iter_mut().rev().find(|f| f.name == q) {
            f.written = written.clone();
        }
        if let Some(f) = self.functions.iter_mut().rev().find(|f| f.name == q) {
            f.written = written;
        }
        if let Some((only, used, trust)) = uses {
            let claim = format!("{q}::uses");
            self.state(&claim, &q, Vec::new(), Target::Uses { used, only }, line);
            self.declare(&claim, "declared", trust);
        }
        Ok(())
    }

    /// The inputs of the function `f`, as known where the parser is.
    fn sig(&self, f: &str, inputs: usize) -> R<Sig> {
        let known = self.known.fns.get(f).or(self.names.fns.get(f)).cloned();
        match known {
            Some(sig) => Ok(sig),
            // The first reading: declared further down.
            None if self.first => Ok(Sig {
                params: vec![(String::new(), Set::any()); inputs],
                deep: vec![Set::any(); inputs],
            }),
            None => self.err(format!("the model has no function `{f}`")),
        }
    }
    /// `(a: T, b)`: the inputs of a claim. The type may be left out: the
    /// one of the function the input is passed to. Plain, then deep.
    fn inputs(&mut self, close: &str, into: &mut Vec<(Bound, Option<Set>)>) -> R<()> {
        while !self.is(close) {
            let line = self.line();
            let x = self.ident()?;
            if into.iter().any(|i| i.0 .0 == x) {
                return Err(Error::new(line, format!("`{x}` is an input twice")));
            }
            let (set, deep) = match self.eat(":") {
                true => {
                    let (set, deep) = self.set_both()?;
                    (Some(set), Some(deep))
                }
                false => (None, None),
            };
            self.locals.push(x.clone());
            into.push(((x, set, line), deep));
            if !self.eat(",") {
                break;
            }
        }
        Ok(())
    }

    fn claim(&mut self, line: usize, assumed: bool) -> R<()> {
        let name = self.ident()?;
        let q = self.qualified(&name);
        if self.eat("=") {
            // claim c = uniform(f), uniform(f, |a, b| class), uniform(f, classes)
            let form = self.path()?;
            // The two shapes Mitra works out on whole sets: see `equivalence`.
            let shape = std::mem::take(&mut self.shape);
            if form == "represented" && shape {
                return self.represented(line, &q);
            }
            if form != "uniform" || !shape {
                return self.instance(line, assumed, &name, &form);
            }
            self.expect("(")?;
            let f = self.path()?;
            let f = self.resolve(&f);
            let mut by = None;
            if self.eat(",") {
                if self.eat("|") {
                    let names = self.bars()?;
                    let mut sig = self.sig(&f, names.len())?;
                    if names.len() != sig.params.len() {
                        let msg = format!(
                            "`{f}` takes {} input(s), got {}",
                            sig.params.len(),
                            names.len()
                        );
                        return self.err(msg);
                    }
                    self.locals = names.clone();
                    // `|a| -> Type { class }`; without a type, a Bool.
                    // The class is computed: a function in it is its spec.
                    self.own = Some((String::new(), String::new()));
                    let read = match self.eat("->") {
                        true => self.set_both().and_then(|c| Ok((c, self.block()?))),
                        false => self.expr().map(|e| ((Set::bool(), Set::bool()), e)),
                    };
                    self.own = None;
                    self.locals.clear();
                    let ((class, deep), body) = read?;
                    for (p, x) in sig.params.iter_mut().zip(names) {
                        p.0 = x;
                    }
                    sig.deep.truncate(sig.params.len());
                    sig.deep.push(deep);
                    let case = format!("{q}::case");
                    self.define(&case, &sig, class, Some(body), line);
                    by = Some(case);
                } else {
                    let g = self.path()?;
                    by = Some(self.resolve(&g));
                }
            }
            self.expect(")")?;
            self.eat(";");
            self.state(&q, &f, Vec::new(), Target::Uniform(by), line);
            return Ok(());
        }
        // claim c(f, T) { given a: T then f(a) == a }: a claim with
        // parameters, used as `claim d = c(g, Int);`.
        if self.is("(") {
            let mut i = 1;
            let mut params = Vec::new();
            while self.kind_at(i) == Some(Kind::Name) && [",", ")"].contains(&self.peek_at(i + 1)) {
                params.push(self.peek_at(i).to_string());
                i += 2;
            }
            let closed = i > 1 && self.peek_at(i - 1) == ")";
            if closed && self.peek_at(i) == "{" && self.peek_at(i + 1) == "given" {
                self.pos += i;
                let start = self.pos;
                let mut depth = 0;
                loop {
                    if self.pos >= self.toks.len() {
                        return self.err("expected `}`");
                    }
                    depth += self.is("{") as i32 - self.is("}") as i32;
                    self.pos += 1;
                    if depth == 0 {
                        break;
                    }
                }
                let body = self.toks[start..self.pos].to_vec();
                self.eat(";");
                self.templates
                    .insert(format!("{q}/{}", params.len()), (params, body));
                return Ok(());
            }
        }
        // claim c(a: T) when cond { body }, or
        // claim c { given a: T  when cond  then body }
        let mut inputs = Vec::new();
        self.locals.clear();
        if self.eat("(") {
            self.inputs(")", &mut inputs)?;
            self.expect(")")?;
        }
        let mut when = None;
        self.called.clear();
        let trust = self.trust()?;
        if let Some(t) = trust.filter(|_| !assumed) {
            self.names.trust.insert(q.clone(), t);
        }
        self.measures.clear();
        self.impure = false;
        if self.eat("when") {
            when = Some(self.expr()?);
        }
        self.expect("{")?;
        if when.is_none() && self.equivalence(line, &name)? {
            return Ok(());
        }
        let contract = self.is("given") || self.is("when") || self.is("then");
        if self.eat("given") {
            self.inputs("", &mut inputs)?;
        }
        if contract && self.eat("when") {
            when = Some(self.expr()?);
        }
        // A condition on the inputs alone takes inputs out of the claim.
        let typed = !inputs.is_empty() && inputs.iter().all(|i| i.1.is_some());
        if let Some(w) = when.as_ref().filter(|_| !self.impure && typed) {
            let each = inputs
                .iter()
                .map(|((x, set, _), _)| (x.clone(), set.clone()));
            let mut sig = Sig {
                params: each.map(|(x, set)| (x, set.expect("typed"))).collect(),
                deep: inputs.iter().filter_map(|i| i.1.clone()).collect(),
            };
            sig.deep.push(Set::bool());
            self.define(
                &format!("{q}::when"),
                &sig,
                Set::bool(),
                Some(w.clone()),
                line,
            );
        }
        if contract {
            self.expect("then")?;
        }
        let bound: Vec<Bound> = inputs.into_iter().map(|i| i.0).collect();
        // `correct(f(a))` alone is the claim on the code; inside a larger
        // expression it is a value.
        let alone = self.is("correct") && self.peek_at(1) == "(" && {
            let (mut i, mut depth) = (1, 0);
            loop {
                depth += (self.peek_at(i) == "(") as i32 - (self.peek_at(i) == ")") as i32;
                i += 1;
                if depth == 0 || self.pos + i >= self.toks.len() {
                    break;
                }
            }
            self.peek_at(i) == "}"
        };
        if alone {
            // correct(f(a, b)): judged by people or by tests, not computed.
            self.pos += 2;
            let f = self.path()?;
            let f = self.resolve(&f);
            let pattern = self.args(&bound)?;
            if !self.parens {
                return self.err(format!("`correct({f})` needs the inputs of `{f}`"));
            }
            self.expect(")")?;
            if when.is_some() {
                return self.err(
                    "`correct(…)` under `when` is not supported yet: narrow the type of an input",
                );
            }
            self.state(&q, &f, pattern, Target::Correct, line);
        } else {
            let body = self.sequence()?;
            let body = match when {
                Some(w) => bin("||", Expr::Unary("!", Box::new(w)), body),
                None => body,
            };
            let pattern = bound
                .iter()
                .map(|(x, set, _)| Position {
                    var: Some(x.clone()),
                    set: set.clone(),
                })
                .collect();
            self.properties.push((self.m.claims.len(), bound.clone()));
            self.state(&q, &q, pattern, Target::Property(body), line);
        }
        self.expect("}")?;
        self.locals.clear();
        if assumed {
            self.declare(&q, "assumed", trust.unwrap_or(1.0));
        }
        let inputs = bound.into_iter().map(|b| b.0).collect();
        let mut called = std::mem::take(&mut self.called);
        called.sort();
        called.dedup();
        self.names.derived.insert(q.clone(), called);
        let measures = std::mem::take(&mut self.measures);
        self.names.claims.insert(q, (inputs, measures));
        Ok(())
    }

    /// A claim that says the same judgement holds on every input of a set,
    /// or of each class:
    ///
    /// `given x: S, y: S then correct(f(x)) <=> correct(f(y))`, or
    /// `given x, y [when k(x) == k(y)] then correct(f(x)) <=> correct(f(y))`.
    ///
    /// Read as that set or those classes, whatever the claim is called:
    /// Mitra then works on sets, not input by input. Just after `{`.
    fn equivalence(&mut self, line: usize, name: &str) -> R<bool> {
        let (start, mut end, mut depth) = (self.pos, self.pos, 1);
        while depth > 0 {
            let Some(t) = self.toks.get(end) else {
                return Ok(false);
            };
            depth += (t.text == "{") as i32 - (t.text == "}") as i32;
            end += 1;
        }
        let body: Vec<Tok> = self.toks[start..end - 1].to_vec();
        let text = |t: &[Tok]| {
            t.iter()
                .map(|t| t.text.as_str())
                .collect::<Vec<_>>()
                .join(" ")
        };
        // The first of these words outside any bracket.
        let top = |toks: &[Tok], words: &[&str]| {
            let mut depth = 0;
            toks.iter().position(|t| {
                let at = depth == 0 && words.contains(&t.text.as_str()) && t.kind != Kind::Str;
                depth += ["(", "[", "{"].contains(&t.text.as_str()) as i32;
                depth -= [")", "]", "}"].contains(&t.text.as_str()) as i32;
                at
            })
        };
        if body.first().is_none_or(|t| t.text != "given") {
            return Ok(false);
        }
        let Some(then) = top(&body, &["then"]) else {
            return Ok(false);
        };
        // then correct(f(x)) <=> correct(f(y))
        let said = &body[then + 1..];
        let Some(op) = top(said, &["<=>", "=="]) else {
            return Ok(false);
        };
        // `correct ( f ( x ) )`: the function, then the name.
        let side = |toks: &[Tok]| -> Option<(Vec<Tok>, String)> {
            let n = toks.len();
            let fits = n >= 7
                && text(&toks[..2]) == "correct ("
                && toks[n - 4].text == "("
                && toks[n - 3].kind == Kind::Name
                && text(&toks[n - 2..]) == ") )";
            fits.then(|| (toks[2..n - 4].to_vec(), toks[n - 3].text.clone()))
        };
        let (Some((f, x)), Some((g, y))) = (side(&said[..op]), side(&said[op + 1..])) else {
            return Ok(false);
        };
        if text(&f) != text(&g) || x == y {
            return Ok(false);
        }
        // given x[: S], y[: S]  [when k(x) == k(y)]
        let when = top(&body[..then], &["when"]);
        let given = &body[1..when.unwrap_or(then)];
        let Some(comma) = top(given, &[","]) else {
            return Ok(false);
        };
        let input = |toks: &[Tok]| -> Option<(String, Vec<Tok>)> {
            let (first, rest) = toks.split_first()?;
            match rest.split_first() {
                None => Some((first.text.clone(), Vec::new())),
                Some((colon, set)) if colon.text == ":" && !set.is_empty() => {
                    Some((first.text.clone(), set.to_vec()))
                }
                _ => None,
            }
        };
        let (Some((a, set)), Some((b, other))) =
            (input(&given[..comma]), input(&given[comma + 1..]))
        else {
            return Ok(false);
        };
        if (&a, &b) != (&x, &y) || text(&set) != text(&other) {
            return Ok(false);
        }
        let mut key = Vec::new();
        if let Some(w) = when {
            let cond = &body[w + 1..then];
            let Some(eq) = top(cond, &["=="]) else {
                return Ok(false);
            };
            let on = |toks: &[Tok], x: &str| -> Option<Vec<Tok>> {
                let n = toks.len().checked_sub(3)?;
                (text(&toks[n..]) == format!("( {x} )")).then(|| toks[..n].to_vec())
            };
            let (Some(k), Some(l)) = (on(&cond[..eq], &x), on(&cond[eq + 1..], &y)) else {
                return Ok(false);
            };
            if text(&k) != text(&l) || k.is_empty() || !set.is_empty() {
                return Ok(false);
            }
            // An inline function comes in parentheses.
            key = match k.len() > 2 && k[0].text == "(" && k[1].text == "|" {
                true => k[1..k.len() - 1].to_vec(),
                false => k,
            };
        }
        let tok = |text: &str, kind: Kind| Tok {
            kind,
            text: text.to_string(),
            line,
        };
        let form = if set.is_empty() {
            "uniform"
        } else {
            "represented"
        };
        let mut toks = vec![
            tok(name, Kind::Name),
            tok("=", Kind::Punct),
            tok(form, Kind::Name),
            tok("(", Kind::Punct),
        ];
        toks.extend(f);
        for more in [set, key].into_iter().filter(|m| !m.is_empty()) {
            toks.push(tok(",", Kind::Punct));
            toks.extend(more);
        }
        toks.push(tok(")", Kind::Punct));
        let rest = std::mem::replace(&mut self.toks, toks);
        self.pos = 0;
        self.shape = true;
        let done = self.claim(line, false);
        self.shape = false;
        (self.toks, self.pos) = (rest, end);
        done.map(|_| true)
    }

    /// `claim c = represented(f, { (a, b) | b != 0 });` or, with a set per
    /// input, `represented(f, (0..=9, 1..=9))`: one tested input of the
    /// set stands for all of it.
    fn represented(&mut self, line: usize, q: &str) -> R<()> {
        self.expect("(")?;
        let f = self.path()?;
        let f = self.resolve(&f);
        self.expect(",")?;
        let known = self.sig(&f, 0)?;
        let arity = |n: usize, p: &Self| match n == known.params.len() || p.first {
            true => Ok(()),
            false => p.err(format!(
                "`{f}` takes {} input(s), got {n}",
                known.params.len()
            )),
        };
        let comprehension = self.is("{")
            && (self.peek_at(2) == "|"
                || (self.peek_at(1) == "(" && self.kind_at(2) == Some(Kind::Name)));
        let any = || std::iter::repeat(Set::any());
        let (sig, body) = if comprehension {
            // { (a, b) | cond } over the inputs of f.
            self.pos += 1;
            let mut names = Vec::new();
            if self.eat("(") {
                while !self.eat(")") {
                    names.push(self.ident()?);
                    if !self.eat(",") {
                        self.expect(")")?;
                        break;
                    }
                }
            } else {
                names.push(self.ident()?);
            }
            self.expect("|")?;
            arity(names.len(), self)?;
            self.locals = names.clone();
            self.own = Some((String::new(), String::new()));
            let cond = self.expr();
            self.own = None;
            self.locals.clear();
            self.expect("}")?;
            let n = names.len();
            let types = known.params.iter().map(|p| p.1.clone()).chain(any());
            let deep = known.deep.iter().take(known.params.len()).cloned();
            let sig = Sig {
                params: names.into_iter().zip(types).collect(),
                deep: deep.chain(any()).take(n).collect(),
            };
            (sig, cond?)
        } else {
            // A set per input.
            let mut sets = Vec::new();
            let open = self.eat("(");
            while !(open && self.eat(")")) {
                sets.push(self.set_both()?);
                if !open {
                    break;
                }
                if !self.eat(",") {
                    self.expect(")")?;
                    break;
                }
            }
            arity(sets.len(), self)?;
            let names = known
                .params
                .iter()
                .map(|p| p.0.clone())
                .chain((0..).map(|i| format!("x{i}")));
            let sig = Sig {
                params: names.zip(sets.iter().map(|s| s.0.clone())).collect(),
                deep: sets.into_iter().map(|s| s.1).collect(),
            };
            (sig, Expr::Lit(Value::Bool(true)))
        };
        self.expect(")")?;
        if let Some(t) = self.trust()? {
            self.names.trust.insert(q.to_string(), t);
        }
        self.eat(";");
        let set = format!("{q}::set");
        let mut sig = sig;
        sig.deep.push(Set::bool());
        self.define(&set, &sig, Set::bool(), Some(body), line);
        self.names.represented.push((q.to_string(), f, set, line));
        self.names
            .claims
            .insert(q.to_string(), (Vec::new(), Vec::new()));
        Ok(())
    }

    /// The named declarations of another file, read as if written here.
    fn import(&mut self, line: usize, path: &[String], names: &[String]) -> R<()> {
        let name = path.join("::");
        if self.stack.contains(&name) {
            return Err(Error::new(line, format!("`{name}` imports itself")));
        }
        let src = module(path).map_err(|e| Error::new(line, e))?;
        let inside =
            |e: Error| Error::new(line, format!("in `{name}`, line {}: {}", e.line, e.message));
        let toks = lex(&src).map_err(inside)?;
        // Only what is named, and what the module itself imports.
        let (mut kept, mut found, mut depth, mut keep) = (Vec::new(), Vec::new(), 0, false);
        for (i, t) in toks.iter().enumerate() {
            if depth == 0 && t.kind == Kind::Name && ITEMS.contains(&t.text.as_str()) {
                let item = toks.get(i + 1).map_or("", |n| n.text.as_str());
                keep = t.text == "import" || names.iter().any(|n| n == item);
                if keep && t.text != "import" {
                    found.push(item.to_string());
                }
            }
            if t.kind == Kind::Punct {
                depth += (t.text == "{") as i32 - (t.text == "}") as i32;
            }
            if keep {
                kept.push(t.clone());
            }
        }
        if let Some(n) = names.iter().find(|n| !found.contains(n)) {
            return Err(Error::new(line, format!("`{name}` declares no `{n}`")));
        }
        // Read here, with the names of the module alone, on this line.
        let declared = declared(&kept);
        let all = std::mem::replace(&mut self.all, declared.clone());
        let toks = std::mem::replace(&mut self.toks, kept);
        let pos = std::mem::replace(&mut self.pos, 0);
        let at = self.at.replace(line);
        self.stack.push(name.clone());
        let mut done = self.items();
        if done.is_ok() && self.pos < self.toks.len() {
            done = self.err(format!("unexpected `{}`", self.peek()));
        }
        self.stack.pop();
        self.at = at;
        (self.toks, self.pos) = (toks, pos);
        let module = std::mem::replace(&mut self.all, all);
        self.all.extend(module);
        self.all.extend(declared);
        done.map_err(inside)
    }

    /// `claim c = shape(f, 0..=9, |a| a > 0) trust 0.9;`: the claim `shape`
    /// with its parameters replaced by what is written here.
    fn instance(&mut self, line: usize, assumed: bool, name: &str, form: &str) -> R<()> {
        self.expect("(")?;
        let mut args: Vec<Vec<Tok>> = vec![Vec::new()];
        let (mut depth, mut bars) = (0, false);
        loop {
            if self.pos >= self.toks.len() {
                return self.err("expected `)`");
            }
            let t = self.toks[self.pos].clone();
            self.pos += 1;
            let last = args.last_mut().expect("one at least");
            match t.text.as_str() {
                // The inputs of an inline function: `|a, b|`.
                "|" if depth == 0 && (last.is_empty() || bars) => bars = !bars,
                "(" | "[" | "{" => depth += 1,
                ")" if depth == 0 => break,
                ")" | "]" | "}" => depth -= 1,
                "," if depth == 0 && !bars => {
                    args.push(Vec::new());
                    continue;
                }
                _ => {}
            }
            last.push(t);
        }
        if args.len() == 1 && args[0].is_empty() {
            args.clear();
        }
        let wanted = [self.qualified(form), form.to_string()];
        let found = wanted
            .iter()
            .find_map(|n| self.templates.get(&format!("{n}/{}", args.len())).cloned());
        let Some((params, body)) = found else {
            let named = |k: &&String| wanted.iter().any(|n| k.starts_with(&format!("{n}/")));
            let takes: Vec<&str> = self
                .templates
                .keys()
                .filter(named)
                .filter_map(|k| k.rsplit('/').next())
                .collect();
            let msg = match takes.is_empty() {
                true => format!(
                    "no claim with parameters `{form}` above: write \
                     `claim {form}(f, …) {{ given … then … }}`, or import it"
                ),
                false => format!(
                    "`{form}` takes {} parameter(s), got {}",
                    takes.join(" or "),
                    args.len()
                ),
            };
            return Err(Error::new(line, msg));
        };
        let tok = |text: &str, kind: Kind| Tok {
            kind,
            text: text.to_string(),
            line,
        };
        let mut toks = vec![tok(name, Kind::Name)];
        // `trust 0.9` after the parameters.
        while !self.is(";") && self.pos < self.toks.len() && self.toks[self.pos].line == line {
            toks.push(self.toks[self.pos].clone());
            self.pos += 1;
        }
        self.eat(";");
        for t in body {
            let at = params
                .iter()
                .position(|p| t.kind == Kind::Name && p == &t.text);
            match at {
                Some(i) if args[i].first().is_some_and(|a| a.text == "|") => {
                    toks.push(tok("(", Kind::Punct));
                    toks.extend(args[i].iter().cloned());
                    toks.push(tok(")", Kind::Punct));
                }
                Some(i) => toks.extend(args[i].iter().cloned()),
                None => toks.push(t),
            }
        }
        for t in &mut toks {
            t.line = line;
        }
        let rest = std::mem::replace(&mut self.toks, toks);
        let pos = std::mem::replace(&mut self.pos, 0);
        let done = self.claim(line, assumed);
        let all = self.pos == self.toks.len();
        let extra = self.err::<()>(format!("unexpected `{}` in `{form}`", self.peek()));
        (self.toks, self.pos) = (rest, pos);
        done?;
        match all {
            true => Ok(()),
            false => extra,
        }
    }

    // ---- evidence ----

    /// evidence name { for target  given a = 1, b in 0..=9  holds  trust 1  source "…" }
    fn evidence(&mut self) -> R<Evidence> {
        let line = self.line();
        self.expect("evidence")?;
        let label = self.name()?;
        self.expect("{")?;
        self.expect("for")?;
        let target = self.path()?;
        let target = self.find(&target)?;
        let (mut given, mut says, mut trust) = (Vec::new(), None, 1.0);
        let (mut source, mut rev) = (label, None);
        let clauses = ["given", "holds", "fails", "gives", "trust", "source", "rev"];
        while !self.eat("}") {
            let at = self.line();
            let clause = self.name()?;
            match clause.as_str() {
                "given" => loop {
                    let x = self.name()?;
                    let exact = self.eat("=");
                    if !exact && !self.eat("in") && !self.eat(":") {
                        return self.err(format!("expected `{x} = value` or `{x} in set`"));
                    }
                    let set = match exact {
                        true => Set::point(&self.value()?),
                        false => self.set()?,
                    };
                    given.push((x, exact, set));
                    if !self.eat(",") {
                        break;
                    }
                },
                "holds" | "fails" => says = Some((clause, None, 0.0)),
                "gives" if self.is("rate") && self.peek_at(1) == "=" => {
                    self.pos += 2;
                    let negative = self.eat("-");
                    let x = self.number()?;
                    says = Some(("rate".to_string(), None, if negative { -x } else { x }));
                }
                "gives" => says = Some((clause, Some(self.value()?), 0.0)),
                "trust" => {
                    trust = self.number()?;
                    if !(0.0..=1.0).contains(&trust) {
                        return Err(Error::new(
                            at,
                            format!("trust {trust}: it goes from 0 to 1"),
                        ));
                    }
                }
                "source" | "rev" => {
                    let Some(Old::Str(s)) = self.next_old() else {
                        self.pos -= 1;
                        return self.err(format!("`{clause}` needs a string"));
                    };
                    match clause == "rev" {
                        true => rev = Some(s),
                        false => source = s,
                    }
                }
                other => {
                    let msg = format!("expected one of {}, got `{other}`", clauses.join(", "));
                    return Err(Error::new(at, msg));
                }
            }
        }
        let Some((says, value, rate)) = says else {
            return Err(Error::new(
                line,
                "the evidence says nothing: `holds`, `fails` or `gives …`",
            ));
        };
        let at = |m: String| Err(Error::new(line, m));
        let function = self.names.fns.get(&target).cloned();
        let names: Vec<String> = match (&function, self.names.claims.get(&target)) {
            (Some(sig), _) => sig.params.iter().map(|p| p.0.clone()).collect(),
            (None, Some(claim)) => claim.0.clone(),
            (None, None) => return at(format!("the model has no `{target}`")),
        };
        if let Some((x, _, _)) = given.iter().find(|g| !names.contains(&g.0)) {
            return at(format!("`{target}` has no input `{x}`"));
        }
        // Each input: its value, a set, or every value.
        let each = |x: &String| match given.iter().find(|g| &g.0 == x) {
            Some((_, true, v)) => Position {
                var: None,
                set: Some(v.clone()),
            },
            Some((_, false, set)) => Position {
                var: Some(x.clone()),
                set: Some(set.clone()),
            },
            None => Position {
                var: Some(x.clone()),
                set: None,
            },
        };
        let mut pattern: Pattern = names.iter().map(each).collect();
        let mut about = target.clone();
        let judgment = match (says.as_str(), function.is_some(), value) {
            ("rate", _, _) => {
                let measures = self.names.claims.get(&target).map(|c| c.1.clone());
                match measures.as_deref() {
                    Some([one]) => {
                        about = crate::print::expr(one);
                        pattern.clear();
                        Judgment::Measured(one.clone(), rate)
                    }
                    _ => return at(format!("`{target}` is not a claim on one measure")),
                }
            }
            ("gives", true, Some(v)) => Judgment::Output(v),
            ("gives", _, _) => return at(format!("`{target}` is a claim: `holds` or `fails`")),
            (says, true, _) => Judgment::Correct(says == "holds"),
            // The whole claim, or the claim on some inputs.
            (says, false, _) => {
                if given.is_empty() {
                    pattern.clear();
                }
                Judgment::Holds(says == "holds")
            }
        };
        Ok(Evidence {
            about,
            pattern,
            judgment,
            confidence: trust,
            source: Some(source),
            rev,
            line,
        })
    }

    /// The model, once every declaration is read.
    fn finish(mut self) -> R<(Model, Names)> {
        let mut m = std::mem::take(&mut self.m);
        // A property uses every name it binds, and no other.
        for (i, bound) in std::mem::take(&mut self.properties) {
            // `f(x, y) == e` with one name per input is the claim that the
            // output of `f` is `e`.
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
                m.functions.iter().any(|f| &f.name == x)
                    || m.records
                        .iter()
                        .any(|r| &r.name == x && r.fields.is_empty())
            };
            if let Some(x) = free(e, &mut Vec::new(), &mut used)
                .into_iter()
                .find(|x| !bound.iter().any(|b| &b.0 == x) && !variant(x))
            {
                let msg = format!("`{x}` is not known here: it is no input of the claim");
                return Err(Error::new(c.line, msg));
            }
            if let Some((x, _, line)) = bound.iter().find(|b| !used.contains(&b.0)) {
                return Err(Error::new(*line, format!("the input `{x}` is not used")));
            }
        }
        m.recursive = std::mem::take(&mut self.recursive);
        m.aliases = std::mem::take(&mut self.aliases);
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
        Ok((m, self.names))
    }
}

/// The model, and its names for the evidence.
pub fn model(src: &str) -> Result<(Model, Names), Error> {
    read(lex(src)?)?.finish()
}

/// Every function and claim a file declares, by full name: one may name
/// another declared further down.
fn declared(toks: &[Tok]) -> BTreeSet<String> {
    let (mut all, mut depth, mut scope) = (BTreeSet::new(), 0, None);
    for (i, t) in toks.iter().enumerate() {
        let next = toks.get(i + 1).filter(|n| n.kind == Kind::Name);
        match (t.text.as_str(), &t.kind, next) {
            ("{", Kind::Punct, _) => depth += 1,
            ("}", Kind::Punct, _) => {
                depth -= 1;
                if depth == 0 {
                    scope = None;
                }
            }
            ("subject" | "boundary", Kind::Name, Some(n)) if depth == 0 => {
                scope = Some(n.text.clone())
            }
            ("fn" | "claim" | "assume", Kind::Name, Some(n)) if depth <= 1 => {
                all.insert(match &scope {
                    Some(s) => format!("{s}::{}", n.text),
                    None => n.text.clone(),
                });
            }
            _ => {}
        }
    }
    all
}

/// One file, read twice: the second reading knows every function of it.
fn read(toks: Vec<Tok>) -> Result<P, Error> {
    let all = declared(&toks);
    let mut known = Names::default();
    let mut last = None;
    for pass in 0..2 {
        let mut p = P {
            toks: toks.clone(),
            all: all.clone(),
            first: pass == 0,
            known,
            ..P::default()
        };
        p.items()?;
        if p.pos < p.toks.len() {
            return p.err(format!("unexpected `{}`", p.peek()));
        }
        known = p.names.clone();
        last = Some(p);
    }
    Ok(last.expect("read twice"))
}

/// The evidence, then what the model itself declares.
pub fn evidence(src: &str, m: &Model, names: &Names) -> Result<Vec<Evidence>, Error> {
    let mut out = evidence_alone(src, m, names)?;
    let lines = src.lines().count().max(out.last().map_or(0, |e| e.line));
    for (i, d) in names.declared.iter().enumerate() {
        out.push(Evidence {
            line: lines + i + 1,
            ..d.clone()
        });
    }
    Ok(out)
}

/// The evidence blocks of a file.
pub fn evidence_alone(src: &str, m: &Model, names: &Names) -> Result<Vec<Evidence>, Error> {
    let mut all: BTreeSet<String> = names.fns.keys().cloned().collect();
    all.extend(names.claims.keys().cloned());
    // The tie of each function to its code: `for f::tied`.
    let mut names = names.clone();
    for f in names.opaque.clone() {
        names
            .claims
            .insert(format!("{f}::tied"), (Vec::new(), Vec::new()));
        all.insert(format!("{f}::tied"));
    }
    let mut p = P {
        toks: lex(src)?,
        all,
        known: names.clone(),
        names,
        records: m.records.clone(),
        enums: m.enums.clone(),
        functions: m.functions.clone(),
        ..P::default()
    };
    let mut out = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    while p.pos < p.toks.len() {
        let block = p.is("evidence") && p.peek_at(2) == "{" && p.kind_at(1) == Some(Kind::Name);
        if block {
            out.push(p.evidence()?);
            continue;
        }
        let line = p.line();
        let hint = earlier(lines[line - 1]).map_or(
            "evidence is written `evidence name { for f given a = 1 holds }`",
            |h| h.1,
        );
        let msg = format!("expected an `evidence` block: {hint}");
        return Err(Error::new(line, msg));
    }
    Ok(out)
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
        Expr::Each(_, x, list, a) => {
            free(list, scope, used);
            scope.push(x.clone());
            free(a, scope, used);
            scope.pop();
        }
        Expr::Unary(_, a) | Expr::Field(a, _) | Expr::Len(a) => {
            free(a, scope, used);
        }
        Expr::Call(_, args)
        | Expr::Tuple(args)
        | Expr::Correct(_, args)
        | Expr::Record(_, args)
        | Expr::List(args) => {
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
        Expr::Tuple(args) | Expr::Correct(_, args) | Expr::Record(_, args) | Expr::List(args) => {
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
        Expr::Binary(_, a, b) | Expr::Let(_, a, b) | Expr::Index(a, b) | Expr::Each(_, _, a, b) => {
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

/// A declaration as written, from its first line to the end of its
/// braces, on one line.
pub fn written(lines: &[String], line: usize) -> Option<String> {
    let mut depth = 0i32;
    let mut out = String::new();
    for l in lines.get(line.checked_sub(1)?..)? {
        let l = l.split("//").next().unwrap_or("").trim();
        depth += l.matches('{').count() as i32 - l.matches('}').count() as i32;
        if !out.is_empty() && !l.is_empty() {
            out.push(' ');
        }
        out += l;
        if depth <= 0 && !out.is_empty() {
            break;
        }
    }
    let short: String = out.chars().take(200).collect();
    Some(match short.len() < out.len() {
        true => format!("{short} …"),
        false => out,
    })
}

/// Evidence blocks with `source "…"` and `rev "…"` added to each block
/// that does not say it. A block that already has the clause is an error.
pub fn stamp(text: &str, source: Option<&str>, rev: Option<&str>) -> Result<String, String> {
    let mut out = String::new();
    let (mut depth, mut quoted, mut escaped) = (0usize, false, false);
    // The block being read, without its strings: where a clause shows.
    let mut bare = String::new();
    for c in text.chars() {
        if quoted {
            (quoted, escaped) = (escaped || c != '"', !escaped && c == '\\');
            out.push(c);
            continue;
        }
        match c {
            '"' => {
                quoted = true;
                bare.push('"');
            }
            '{' => depth += 1,
            '}' if depth == 1 => {
                depth = 0;
                let has = |k: &str| {
                    let words: Vec<&str> = bare.split_whitespace().collect();
                    words.windows(2).any(|w| w[0] == k && w[1].starts_with('"'))
                };
                for (k, v) in [("source", source), ("rev", rev)] {
                    let Some(v) = v else { continue };
                    if has(k) {
                        return Err(format!(
                            "--{k} is given twice: in the option and in the evidence"
                        ));
                    }
                    if !out.ends_with(' ') {
                        out.push(' ');
                    }
                    out += &format!("{k} \"{v}\" ");
                }
                bare.clear();
            }
            '}' => depth = depth.saturating_sub(1),
            _ => {}
        }
        if depth > 0 && c != '"' {
            bare.push(c);
        }
        out.push(c);
    }
    Ok(out)
}

/// The functions whose tie to the code the evidence speaks of.
pub fn tied(evidence: &[Evidence], names: &Names) -> Vec<String> {
    let found = |f: &&String| evidence.iter().any(|e| e.about == format!("{f}::tied"));
    names.opaque.iter().filter(found).cloned().collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn error(text: &str) -> Error {
        model(text).map(|_| ()).unwrap_err()
    }

    #[test]
    fn reads_the_64_bit_extremes() {
        let ok = "fn f(a: Int) -> Int;\n\
                  claim c() { f(-9223372036854775808) == 9223372036854775807 }";
        assert!(model(ok).is_ok());
        let e = error("claim c() { 9223372036854775808 == 1 }");
        assert!(e.message.contains("outside the 64-bit integers"), "{e}");
    }

    #[test]
    fn set_operators_bind_as_documented() {
        let (m, _) = model("fn f(a: 0..=9 \\ {3} | {20} & 15..=25) -> Int;").unwrap();
        let a = &m.functions[0].params[0].1;
        assert!(a.contains(&Value::Int(20)) && a.contains(&Value::Int(4)));
        assert!(!a.contains(&Value::Int(3)) && !a.contains(&Value::Int(15)));
    }

    #[test]
    fn errors_point_at_their_line() {
        let e = error("fn f(a: Int) -> Int;\n\nfn g(a: Nope) -> Int;");
        assert_eq!(e.line, 3);
        assert!(e.message.contains("unknown type `Nope`"), "{e}");
        let e = error("fn if(a: Int) -> Int;");
        assert!(e.message.contains("`if` is a keyword"), "{e}");
        let e = error("fn f(a: String) -> Int;\nclaim c() { f(\"a\\qb\") == 1 }");
        assert_eq!(e.line, 2);
        assert!(e.message.contains("in a string"), "{e}");
    }

    #[test]
    fn rust_names_are_the_same_types() {
        let (m, _) = model("fn f(a: Vec<i32>, b: usize) -> bool;").unwrap();
        let (n, _) = model("fn f(a: List<Int32>, b: 0..) -> Bool;").unwrap();
        assert_eq!(m.functions[0].params, n.functions[0].params);
        assert_eq!(m.functions[0].range, n.functions[0].range);
    }

    #[test]
    fn trust_goes_from_0_to_1() {
        let (m, names) = model("fn f(a: Int) -> Int;").unwrap();
        let read = |e: &str| evidence_alone(e, &m, &names);
        assert!(read("evidence t { for f given a = 1 holds trust 0.5 }").is_ok());
        let e = read("evidence t { for f given a = 1 holds trust 2 }").unwrap_err();
        assert!(e.message.contains("from 0 to 1"), "{e}");
        let e = read("evidence t { for f given b = 1 holds }").unwrap_err();
        assert!(e.message.contains("has no input `b`"), "{e}");
    }

    #[test]
    fn an_import_is_read_in_place() {
        let (m, _) = model(
            "import std::claims::{commutative};\n\
             fn add(a: 0..=3, b: 0..=3) -> Int;\n\
             claim add_commutes = commutative(add, 0..=3);",
        )
        .unwrap();
        assert_eq!(m.claims.len(), 1);
        let e = error("import std::claims::{nope};");
        assert!(e.message.contains("declares no `nope`"), "{e}");
    }
}
