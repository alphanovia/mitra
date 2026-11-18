//! The reworked surface: `subject` and `boundary` blocks, `fn`, `claim`
//! with inputs, `assume`, evidence blocks.
//!
//! A prototype: this text is translated, declaration by declaration, to
//! the earlier syntax, which the engine reads. Each declaration lands on
//! the line it started at, so errors and reports point at the right line.

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

#[derive(Debug, Clone, PartialEq)]
enum Kind {
    Name,
    Num,
    Str,
    Punct,
}

#[derive(Debug, Clone)]
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
    // At the start of a line: there, `--` opens a comment, as before.
    let mut fresh = true;
    while let Some(c) = rest.chars().next() {
        let first = fresh;
        fresh = c == '\n' || (first && c.is_whitespace());
        if c == '\n' {
            line += 1;
        }
        if c.is_whitespace() {
            rest = &rest[c.len_utf8()..];
        } else if rest.starts_with("//") || (rest.starts_with("--") && first) {
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

/// What the model declares, for the evidence that names it.
#[derive(Debug, Default, Clone)]
pub struct Names {
    /// Each function, by its full name, with its parameters and their
    /// types in the earlier syntax.
    fns: BTreeMap<String, Vec<(String, String)>>,
    /// Each claim with its inputs, and the measures in its body.
    claims: BTreeMap<String, (Vec<String>, Vec<String>)>,
    /// The functions of a subject that have a body: their spec.
    specified: BTreeMap<String, String>,
    /// The functions whose code is outside the model.
    opaque: BTreeSet<String>,
    /// For each claim, the spec claims of the functions it calls.
    pub derived: BTreeMap<String, Vec<String>>,
    /// The trust given to the statement of a claim.
    pub trust: BTreeMap<String, f64>,
    /// Evidence the model itself brings: what is assumed or declared.
    pub declared: Vec<String>,
    /// The boundaries.
    pub boundaries: BTreeSet<String>,
    /// The `represented` claims: name, function, the spec of the set, line.
    pub represented: Vec<(String, String, String, usize)>,
}

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
    measures: Vec<String>,
    names: Names,
    fresh: usize,
    /// Where each `test` block starts.
    tests: Vec<usize>,
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
}

type R<T> = Result<T, Error>;

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

    // ---- types ----

    fn ty(&mut self) -> R<String> {
        let mut out = self.ty_inter()?;
        while self.eat("|") {
            out = format!("{out} | {}", self.ty_inter()?);
        }
        Ok(out)
    }
    fn ty_inter(&mut self) -> R<String> {
        let mut out = self.ty_atom()?;
        loop {
            let op = match (self.is("&"), self.is("\\")) {
                (true, _) => "&",
                (_, true) => "\\",
                _ => return Ok(out),
            };
            self.pos += 1;
            out = format!("{out} {op} {}", self.ty_atom()?);
        }
    }
    fn tys(&mut self, close: &str) -> R<Vec<String>> {
        let mut out = Vec::new();
        if self.eat(close) {
            return Ok(out);
        }
        loop {
            out.push(self.ty()?);
            if !self.eat(",") {
                self.expect(close)?;
                return Ok(out);
            }
        }
    }
    fn ty_atom(&mut self) -> R<String> {
        if self.eat("(") {
            return Ok(format!("({})", self.tys(")")?.join(", ")));
        }
        if self.eat("fn") {
            self.expect("(")?;
            let params = self.tys(")")?;
            self.expect("->")?;
            return Ok(format!("({}) -> {}", params.join(", "), self.ty_atom()?));
        }
        if self.eat("{") {
            let mut items = Vec::new();
            while !self.eat("}") {
                items.push(self.value(&[",", "}"])?);
                if !self.eat(",") {
                    self.expect("}")?;
                    break;
                }
            }
            return Ok(format!("{{{}}}", items.join(", ")));
        }
        if self.kind() == Some(Kind::Num) || self.is("-") || self.is("..") || self.is("..=") {
            // A range: `0..=9`, `1..`, `..=5`.
            let mut out = String::new();
            while self.kind() == Some(Kind::Num) || ["-", "..", "..="].contains(&self.peek()) {
                out += self.peek();
                self.pos += 1;
            }
            return Ok(out);
        }
        let name = self.name()?;
        let args = match self.eat("<") {
            true => self.tys(">")?,
            false => Vec::new(),
        };
        // Rust's names for the same types.
        let name = match name.as_str() {
            "bool" => "Bool",
            "i8" => "Int8",
            "i16" => "Int16",
            "i32" => "Int32",
            "i64" => "Int64",
            "u8" => "UInt8",
            "u16" => "UInt16",
            "u32" => "UInt32",
            "u64" | "usize" => "0..",
            "f64" => "Float64",
            "str" => "String",
            "Vec" => "List",
            "HashSet" | "BTreeSet" => "Set",
            "HashMap" | "BTreeMap" => "Map",
            other => other,
        };
        Ok(match args.is_empty() {
            true => name.to_string(),
            false => format!("{name}({})", args.join(", ")),
        })
    }
    /// `<T, U>` after a name, as `(T, U)`.
    fn type_params(&mut self) -> R<String> {
        if !self.eat("<") {
            return Ok(String::new());
        }
        let mut names = vec![self.name()?];
        while self.eat(",") {
            names.push(self.name()?);
        }
        self.expect(">")?;
        Ok(format!("({})", names.join(", ")))
    }
    /// `(a: T, b: U)`: each name with its type.
    fn params(&mut self) -> R<Vec<(String, String)>> {
        self.expect("(")?;
        let mut out = Vec::new();
        while !self.eat(")") {
            let name = self.name()?;
            // In a claim the type may be left out: the one of the
            // function the input is passed to.
            let ty = match self.eat(":") {
                true => self.ty()?,
                false => String::new(),
            };
            out.push((name, ty));
            if !self.eat(",") {
                self.expect(")")?;
                break;
            }
        }
        Ok(out)
    }

    // ---- values ----

    /// A value, as written, up to one of `ends` outside any bracket.
    fn value(&mut self, ends: &[&str]) -> R<String> {
        let (mut out, mut depth) = (String::new(), 0usize);
        loop {
            let Some(t) = self.toks.get(self.pos) else {
                return self.err("expected a value");
            };
            let open = ["(", "[", "{"].contains(&t.text.as_str()) && t.kind == Kind::Punct;
            let close = [")", "]", "}"].contains(&t.text.as_str()) && t.kind == Kind::Punct;
            if depth == 0 && t.kind != Kind::Str && ends.contains(&t.text.as_str()) {
                break;
            }
            if close && depth == 0 {
                break;
            }
            depth = depth + open as usize - close as usize;
            let tight = close || [",", ":"].contains(&t.text.as_str());
            let after = out.ends_with(['(', '[', '{', '-']);
            let call = t.text == "(" && out.ends_with(|c: char| c.is_ascii_alphanumeric());
            if !out.is_empty() && !tight && !after && !call {
                out += " ";
            }
            out += &t.text;
            self.pos += 1;
        }
        match out.is_empty() {
            true => self.err("expected a value"),
            false => Ok(out),
        }
    }

    // ---- expressions ----

    fn expr(&mut self) -> R<String> {
        let mut a = self.or()?;
        while self.eat("<=>") {
            a = format!("({a} == {})", self.or()?);
        }
        Ok(a)
    }
    fn or(&mut self) -> R<String> {
        let mut a = self.and()?;
        while self.eat("||") {
            a = format!("({a} || {})", self.and()?);
        }
        Ok(a)
    }
    fn and(&mut self) -> R<String> {
        let mut a = self.compare()?;
        while self.eat("&&") {
            a = format!("({a} && {})", self.compare()?);
        }
        Ok(a)
    }
    fn compare(&mut self) -> R<String> {
        let mut a = self.sum()?;
        loop {
            if self.eat("is") {
                // `x is Err`: of that variant.
                let v = self.name()?;
                let Some(n) = self.variants.get(&v) else {
                    return self.err(format!("`is` needs a variant of an enum, got `{v}`"));
                };
                let fields: Vec<String> = (0..*n).map(|i| format!("is_{i}")).collect();
                let pattern = match fields.is_empty() {
                    true => v,
                    false => format!("{v}({})", fields.join(", ")),
                };
                a = format!("(match {a} {{ {pattern} => true, _ => false }})");
                continue;
            }
            let ops = ["==", "!=", "<=", ">=", "<", ">", "in"];
            let Some(op) = ops.iter().find(|op| self.is(op)) else {
                return Ok(a);
            };
            self.pos += 1;
            a = format!("({a} {op} {})", self.sum()?);
        }
    }
    fn sum(&mut self) -> R<String> {
        let mut a = self.product()?;
        loop {
            let Some(op) = ["+", "-", "|", "\\"].iter().find(|op| self.is(op)) else {
                return Ok(a);
            };
            self.pos += 1;
            a = format!("({a} {op} {})", self.product()?);
        }
    }
    fn product(&mut self) -> R<String> {
        let mut a = self.unary()?;
        loop {
            let Some(op) = ["*", "/", "&"].iter().find(|op| self.is(op)) else {
                return Ok(a);
            };
            self.pos += 1;
            a = format!("({a} {op} {})", self.unary()?);
        }
    }
    fn unary(&mut self) -> R<String> {
        if self.eat("!") {
            return Ok(format!("(!{})", self.unary()?));
        }
        if self.eat("-") {
            return Ok(match self.kind() {
                Some(Kind::Num) => {
                    self.pos += 1;
                    format!("-{}", self.toks[self.pos - 1].text)
                }
                _ => format!("(-{})", self.unary()?),
            });
        }
        let mut a = self.primary()?;
        loop {
            if self.eat(".") {
                let field = match self.kind() {
                    Some(Kind::Num | Kind::Name) => self.toks[self.pos].text.clone(),
                    _ => return self.err("expected a field after `.`"),
                };
                self.pos += 1;
                a = format!("({a}.{field})");
            } else if self.eat("[") {
                let i = self.expr()?;
                self.expect("]")?;
                a = format!("{a}[{i}]");
            } else {
                return Ok(a);
            }
        }
    }
    fn exprs(&mut self, close: &str) -> R<Vec<String>> {
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
    fn block(&mut self) -> R<String> {
        self.expect("{")?;
        let out = self.sequence()?;
        self.expect("}")?;
        Ok(out)
    }
    /// `let x = e; … result`, without the braces.
    fn sequence(&mut self) -> R<String> {
        if !self.eat("let") {
            return self.expr();
        }
        let x = self.name()?;
        self.expect("=")?;
        let v = self.expr()?;
        self.expect(";")?;
        self.locals.push(x.clone());
        let body = self.sequence();
        self.locals.pop();
        Ok(format!("(let {x} := {v}; {})", body?))
    }
    fn primary(&mut self) -> R<String> {
        match self.kind() {
            Some(Kind::Num | Kind::Str) => {
                self.pos += 1;
                return Ok(self.toks[self.pos - 1].text.clone());
            }
            None => return self.err("expected an expression"),
            _ => {}
        }
        if self.is("(") && self.peek_at(1) == "|" {
            // (|a, b| a + b)(1, 2): an inline function, on its inputs.
            self.pos += 2;
            let mut names = Vec::new();
            while !self.eat("|") {
                names.push(self.name()?);
                if !self.eat(",") {
                    self.expect("|")?;
                    break;
                }
            }
            let n = self.locals.len();
            self.locals.extend(names.iter().cloned());
            let body = self.expr();
            self.locals.truncate(n);
            let body = body?;
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
            let pairs: Vec<(&String, &String)> =
                names.iter().zip(&args).filter(|(x, a)| x != a).collect();
            let mixed = pairs
                .iter()
                .any(|(_, a)| names.iter().any(|n| a.contains(n.as_str())));
            let mut text = body;
            let mut held = Vec::new();
            for (x, a) in pairs.into_iter().rev() {
                if mixed {
                    self.fresh += 1;
                    let h = format!("held_{}", self.fresh);
                    text = format!("let {x} := {h}; {text}");
                    held.push((h, a));
                } else {
                    text = format!("let {x} := {a}; {text}");
                }
            }
            for (h, a) in held {
                text = format!("let {h} := {a}; {text}");
            }
            return Ok(format!("({text})"));
        }
        if self.eat("(") {
            let parts = self.exprs(")")?;
            return Ok(format!("({})", parts.join(", ")));
        }
        if self.eat("[") {
            return Ok(format!("[{}]", self.exprs("]")?.join(", ")));
        }
        if self.eat("{") {
            // A set `{a, b}` or a map `{k: v}`; `{}` and `{:}` are empty.
            if self.eat(":") {
                self.expect("}")?;
                return Ok("{:}".into());
            }
            if self.kind() == Some(Kind::Name) && self.peek_at(1) == "in" {
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
                return Ok(format!("filter({x} in {from}, {})", cond?));
            }
            let mut items = Vec::new();
            while !self.eat("}") {
                let mut item = self.expr()?;
                if self.eat(":") {
                    item = format!("{item}: {}", self.expr()?);
                }
                items.push(item);
                if !self.eat(",") {
                    self.expect("}")?;
                    break;
                }
            }
            return Ok(format!("{{{}}}", items.join(", ")));
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
            return Ok(format!("(if {c} then {t} else {e})"));
        }
        if self.eat("match") {
            let e = self.expr()?;
            self.expect("{")?;
            let mut arms = Vec::new();
            while !self.eat("}") {
                let n = self.locals.len();
                let pattern = match self.eat("_") {
                    true => "_".to_string(),
                    false => {
                        let v = self.name()?;
                        let mut fields = Vec::new();
                        if self.eat("(") {
                            while !self.eat(")") {
                                // `_` for a field nobody reads.
                                let x = match self.eat("_") {
                                    true => {
                                        self.fresh += 1;
                                        format!("unread_{}", self.fresh)
                                    }
                                    false => self.name()?,
                                };
                                fields.push(x);
                                if !self.eat(",") {
                                    self.expect(")")?;
                                    break;
                                }
                            }
                        }
                        self.locals.extend(fields.iter().cloned());
                        match fields.is_empty() {
                            true => v,
                            false => format!("{v}({})", fields.join(", ")),
                        }
                    }
                };
                self.expect("=>")?;
                let body = match self.is("{") && self.peek_at(1) == "let" {
                    true => self.block(),
                    false => self.expr(),
                };
                self.locals.truncate(n);
                arms.push(format!("{pattern} => {}", body?));
                if !self.eat(",") {
                    self.expect("}")?;
                    break;
                }
            }
            return Ok(format!("(match {e} {{ {} }})", arms.join(", ")));
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
            return Ok(format!("{f}::correct({})", args.join(", ")));
        }
        if self.eat("true") {
            return Ok("true".into());
        }
        if self.eat("false") {
            return Ok("false".into());
        }
        if self.is("rate") && self.peek_at(1) == "(" {
            // rate(x in Type, cond): the share of the type where cond holds.
            self.pos += 2;
            let x = self.name()?;
            self.expect("in")?;
            let set = self.ty()?;
            self.expect(",")?;
            self.locals.push(x.clone());
            let cond = self.expr();
            self.locals.pop();
            self.expect(")")?;
            let measure = format!("rate({x} in {set}, {})", cond?);
            self.measures.push(measure.clone());
            self.impure = true;
            return Ok(measure);
        }
        let scores = ["precision", "recall", "false_positive_rate"];
        if scores.contains(&self.peek()) && self.peek_at(1) == "(" {
            let kind = self.name()?;
            self.expect("(")?;
            let f = self.path()?;
            self.expect(")")?;
            self.impure = true;
            let measure = format!("{kind}({})", self.resolve(&f));
            self.measures.push(measure.clone());
            return Ok(measure);
        }
        let each = ["all", "any", "filter", "map", "fold"];
        if each.contains(&self.peek()) && self.peek_at(1) == "(" && self.peek_at(3) == "in" {
            // all(x in xs, cond): over each element.
            let kind = self.name()?;
            self.expect("(")?;
            let x = self.name()?;
            self.expect("in")?;
            let list = self.sum()?;
            self.expect(",")?;
            self.locals.push(x.clone());
            let body = self.sequence();
            self.locals.pop();
            self.expect(")")?;
            return Ok(format!("{kind}({x} in {list}, {})", body?));
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
            return Ok(format!("{name}({})", self.exprs(")")?.join(", ")));
        }
        Ok(name)
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

    fn put(out: &mut Vec<String>, line: usize, text: String) {
        if out.len() < line {
            out.resize(line, String::new());
        }
        let at = &mut out[line - 1];
        if !at.is_empty() {
            at.push(' ');
        }
        at.push_str(&text);
    }

    fn items(&mut self, out: &mut Vec<String>) -> R<()> {
        while self.pos < self.toks.len() && !self.is("}") {
            let line = self.line();
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
                    self.import(out, line, &path, &names)?;
                }
                "subject" | "boundary" if self.scope.is_none() => {
                    self.scope = Some(self.name()?);
                    if kw == "boundary" {
                        self.names.boundaries.extend(self.scope.clone());
                    }
                    self.expect("{")?;
                    self.items(out)?;
                    self.expect("}")?;
                    self.scope = None;
                }
                "subject" | "boundary" => {
                    self.pos -= 1;
                    return self.err(format!("a `{kw}` inside another is not supported yet"));
                }
                "record" => {
                    let name = self.name()?;
                    let generic = self.type_params()?;
                    self.expect("{")?;
                    let mut fields = Vec::new();
                    while !self.eat("}") {
                        let f = self.name()?;
                        self.expect(":")?;
                        fields.push(format!("{f}: {}", self.ty()?));
                        if !self.eat(",") {
                            self.expect("}")?;
                            break;
                        }
                    }
                    let text = format!("record {name}{generic}({})", fields.join(", "));
                    P::put(out, line, text);
                }
                "enum" => {
                    let name = self.name()?;
                    let generic = self.type_params()?;
                    self.expect("{")?;
                    let mut variants = Vec::new();
                    while !self.eat("}") {
                        let v = self.name()?;
                        let mut fields = Vec::new();
                        if self.eat("(") {
                            while !self.eat(")") {
                                // `Some(T)` or `Some(value: T)`.
                                let named =
                                    self.kind() == Some(Kind::Name) && self.peek_at(1) == ":";
                                let f = match named {
                                    true => {
                                        self.pos += 2;
                                        self.toks[self.pos - 2].text.clone()
                                    }
                                    false => format!("v{}", fields.len()),
                                };
                                fields.push(format!("{f}: {}", self.ty()?));
                                if !self.eat(",") {
                                    self.expect(")")?;
                                    break;
                                }
                            }
                        }
                        self.variants.insert(v.clone(), fields.len());
                        variants.push(match fields.is_empty() {
                            true => v,
                            false => format!("{v}({})", fields.join(", ")),
                        });
                        if !self.eat(",") {
                            self.expect("}")?;
                            break;
                        }
                    }
                    let text = format!("enum {name}{generic} {{ {} }}", variants.join(", "));
                    P::put(out, line, text);
                }
                "test" => {
                    self.pos -= 1;
                    return self.err(
                        "a `test` block is not read any more: write the point as a claim, \
                         `claim t() { f(1) == 2 }`, and tie the real test to it with the \
                         comment `// mitra: evidence for t` above its `fn`",
                    );
                }
                "fn" => self.function(out, line)?,
                "claim" | "assume" => self.claim(out, line, kw == "assume")?,
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

    fn function(&mut self, out: &mut Vec<String>, line: usize) -> R<()> {
        let name = self.name()?;
        let q = self.qualified(&name);
        let generic = self.type_params()?;
        let params = self.params()?;
        if let Some((x, _)) = params.iter().find(|p| p.1.is_empty()) {
            return self.err(format!("the input `{x}` of `{name}` needs a type"));
        }
        let result = match self.eat("->") {
            true => self.ty()?,
            false => "()".to_string(),
        };
        let typed: Vec<String> = params.iter().map(|(n, t)| format!("{n}: {t}")).collect();
        let head = format!("{generic}({}) -> {result}", typed.join(", "));
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
        self.names.fns.insert(q.clone(), params.clone());
        if self.is(";") || self.scope.is_some() {
            self.names.opaque.insert(q.clone());
        }
        if self.eat(";") {
            P::put(out, line, format!("opaque {q}{head}"));
        } else {
            let in_subject = self.scope.is_some();
            let body_name = match in_subject {
                true => format!("{q}::body"),
                false => q.clone(),
            };
            self.locals = params.iter().map(|p| p.0.clone()).collect();
            // In a subject, the body is the spec of the code: it calls
            // itself as a spec.
            self.own = Some((name.clone(), body_name.clone()));
            let body = self.block();
            self.own = None;
            self.locals.clear();
            let body = body?;
            if in_subject {
                let args: Vec<&str> = params.iter().map(|p| p.0.as_str()).collect();
                let forall: String = args.iter().map(|a| format!("forall {a}, ")).collect();
                let args = args.join(", ");
                let spec = format!("{q}::spec");
                self.names.specified.insert(q.clone(), body_name.clone());
                P::put(out, line, format!("opaque {q}{head}"));
                P::put(out, line, format!("def {body_name}{head} := {body}"));
                let claim = format!("claim {spec} : {forall}{q}({args}) = {body_name}({args})");
                P::put(out, line, claim);
                self.names.claims.insert(spec, (Vec::new(), Vec::new()));
            } else {
                P::put(out, line, format!("def {q}{head} := {body}"));
            }
        }
        if let Some((only, used, trust)) = uses {
            let claim = format!("{q}::uses");
            let only = if only { "only " } else { "" };
            let text = format!("claim {claim} : {only}uses {q} [{}]", used.join(", "));
            P::put(out, line, text);
            let declared = format!("@[source \"declared\"] evidence {claim} at {trust}");
            self.names.declared.push(declared);
            self.names.claims.insert(claim, (Vec::new(), Vec::new()));
        }
        Ok(())
    }

    fn claim(&mut self, out: &mut Vec<String>, line: usize, assumed: bool) -> R<()> {
        let name = self.name()?;
        let q = self.qualified(&name);
        if self.eat("=") {
            // claim c = uniform(f), uniform(f, |a, b| class), uniform(f, classes)
            let form = self.path()?;
            // The two shapes Mitra works out on whole sets: see `equivalence`.
            let shape = std::mem::take(&mut self.shape);
            if form == "represented" && shape {
                return self.represented(out, line, &q);
            }
            if form != "uniform" || !shape {
                return self.instance(out, line, assumed, &name, &form);
            }
            self.expect("(")?;
            let f = self.path()?;
            let f = self.resolve(&f);
            let mut by = String::new();
            if self.eat(",") {
                if self.eat("|") {
                    let mut names = Vec::new();
                    while !self.eat("|") {
                        names.push(self.name()?);
                        if !self.eat(",") {
                            self.expect("|")?;
                            break;
                        }
                    }
                    let known = self.known.fns.get(&f).or(self.names.fns.get(&f)).cloned();
                    let params = match known {
                        Some(params) => params,
                        // The first reading: declared further down.
                        None if self.first => {
                            vec![(String::new(), "Value".to_string()); names.len()]
                        }
                        None => return self.err(format!("the model has no function `{f}`")),
                    };
                    if names.len() != params.len() {
                        let msg =
                            format!("`{f}` takes {} input(s), got {}", params.len(), names.len());
                        return self.err(msg);
                    }
                    self.locals = names.clone();
                    // `|a| -> Type { class }`; without a type, a Bool.
                    // The class is computed: a function in it is its spec.
                    self.own = Some((String::new(), String::new()));
                    let (class, body) = match self.eat("->") {
                        true => (self.ty()?, self.block()),
                        false => ("Bool".to_string(), self.expr()),
                    };
                    self.own = None;
                    self.locals.clear();
                    let typed: Vec<String> = names
                        .iter()
                        .zip(&params)
                        .map(|(n, p)| format!("{n}: {}", p.1))
                        .collect();
                    let case = format!("{q}::case");
                    let text = format!("def {case}({}) -> {class} := {}", typed.join(", "), body?);
                    P::put(out, line, text);
                    by = format!(" by {case}");
                } else {
                    let g = self.path()?;
                    by = format!(" by {}", self.resolve(&g));
                }
            }
            self.expect(")")?;
            self.eat(";");
            P::put(out, line, format!("claim {q} : uniform {f}{by}"));
            self.names.claims.insert(q, (Vec::new(), Vec::new()));
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
        let mut inputs = match self.is("(") {
            true => self.params()?,
            false => Vec::new(),
        };
        let mut when = None;
        self.called.clear();
        let trust = self.trust()?;
        if let Some(t) = trust.filter(|_| !assumed) {
            self.names.trust.insert(q.clone(), t);
        }
        self.measures.clear();
        self.locals = inputs.iter().map(|p| p.0.clone()).collect();
        self.impure = false;
        if self.eat("when") {
            when = Some(self.expr()?);
        }
        self.expect("{")?;
        if when.is_none() && self.equivalence(out, line, &name)? {
            return Ok(());
        }
        let contract = self.is("given") || self.is("when") || self.is("then");
        if self.eat("given") {
            loop {
                let x = self.name()?;
                let ty = match self.eat(":") {
                    true => self.ty()?,
                    false => String::new(),
                };
                inputs.push((x.clone(), ty));
                self.locals.push(x);
                if !self.eat(",") {
                    break;
                }
            }
        }
        if contract && self.eat("when") {
            when = Some(self.expr()?);
        }
        // A condition on the inputs alone takes inputs out of the claim.
        let typed = !inputs.is_empty() && inputs.iter().all(|p| !p.1.is_empty());
        if let Some(w) = when.as_ref().filter(|_| !self.impure && typed) {
            let typed: Vec<String> = inputs.iter().map(|(x, t)| format!("{x}: {t}")).collect();
            P::put(
                out,
                line,
                format!("def {q}::when({}) -> Bool := {w}", typed.join(", ")),
            );
        }
        if contract {
            self.expect("then")?;
        }
        let forall: String = inputs
            .iter()
            .map(|(x, t)| match t.is_empty() {
                true => format!("forall {x}, "),
                false => format!("forall {x} in {t}, "),
            })
            .collect();
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
        let text = if alone {
            // correct(f(a, b)): judged by people or by tests, not computed.
            self.pos += 2;
            let f = self.path()?;
            let f = self.resolve(&f);
            self.expect("(")?;
            let args = self.exprs(")")?;
            self.expect(")")?;
            if when.is_some() {
                return self.err(
                    "`correct(…)` under `when` is not supported yet: narrow the type of an input",
                );
            }
            format!("claim {q} : {forall}correct({f}({}))", args.join(", "))
        } else {
            let body = self.sequence()?;
            match &when {
                Some(w) => format!("claim {q} : {forall}(!{w}) || {body}"),
                None => format!("claim {q} : {forall}{body}"),
            }
        };
        self.expect("}")?;
        self.locals.clear();
        P::put(out, line, text);
        if assumed {
            let declared = format!(
                "@[source \"assumed\"] evidence {q} at {}",
                trust.unwrap_or(1.0)
            );
            self.names.declared.push(declared);
        }
        let inputs = inputs.into_iter().map(|p| p.0).collect();
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
    fn equivalence(&mut self, out: &mut Vec<String>, line: usize, name: &str) -> R<bool> {
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
        let done = self.claim(out, line, false);
        self.shape = false;
        (self.toks, self.pos) = (rest, end);
        done.map(|_| true)
    }

    /// `claim c = represented(f, { (a, b) | b != 0 });` or, with a set per
    /// input, `represented(f, (0..=9, 1..=9))`: one tested input of the
    /// set stands for all of it.
    fn represented(&mut self, out: &mut Vec<String>, line: usize, q: &str) -> R<()> {
        self.expect("(")?;
        let f = self.path()?;
        let f = self.resolve(&f);
        self.expect(",")?;
        let known = self.known.fns.get(&f).or(self.names.fns.get(&f)).cloned();
        let params = match known {
            Some(params) => params,
            // The first reading: declared further down.
            None if self.first => Vec::new(),
            None => return self.err(format!("the model has no function `{f}`")),
        };
        let arity = |n: usize, p: &Self| match n == params.len() || p.first {
            true => Ok(()),
            false => p.err(format!("`{f}` takes {} input(s), got {n}", params.len())),
        };
        let comprehension = self.is("{")
            && (self.peek_at(2) == "|"
                || (self.peek_at(1) == "(" && self.kind_at(2) == Some(Kind::Name)));
        let (typed, body) = if comprehension {
            // { (a, b) | cond } over the inputs of f.
            self.pos += 1;
            let mut names = Vec::new();
            if self.eat("(") {
                while !self.eat(")") {
                    names.push(self.name()?);
                    if !self.eat(",") {
                        self.expect(")")?;
                        break;
                    }
                }
            } else {
                names.push(self.name()?);
            }
            self.expect("|")?;
            arity(names.len(), self)?;
            self.locals = names.clone();
            self.own = Some((String::new(), String::new()));
            let cond = self.expr();
            self.own = None;
            self.locals.clear();
            self.expect("}")?;
            let types = params
                .iter()
                .map(|p| p.1.clone())
                .chain(std::iter::repeat("Value".into()));
            let typed: Vec<String> = names
                .iter()
                .zip(types)
                .map(|(n, t)| format!("{n}: {t}"))
                .collect();
            (typed, cond?)
        } else {
            // A set per input.
            let sets = match self.eat("(") {
                true => self.tys(")")?,
                false => vec![self.ty()?],
            };
            arity(sets.len(), self)?;
            let names = params
                .iter()
                .map(|p| p.0.clone())
                .chain((0..).map(|i| format!("x{i}")));
            let typed: Vec<String> = names.zip(&sets).map(|(n, t)| format!("{n}: {t}")).collect();
            (typed, "true".to_string())
        };
        self.expect(")")?;
        if let Some(t) = self.trust()? {
            self.names.trust.insert(q.to_string(), t);
        }
        self.eat(";");
        let set = format!("{q}::set");
        P::put(
            out,
            line,
            format!("def {set}({}) -> Bool := {body}", typed.join(", ")),
        );
        self.names.represented.push((q.to_string(), f, set, line));
        self.names
            .claims
            .insert(q.to_string(), (Vec::new(), Vec::new()));
        Ok(())
    }

    /// The named declarations of another file, read as if written here.
    fn import(
        &mut self,
        out: &mut Vec<String>,
        line: usize,
        path: &[String],
        names: &[String],
    ) -> R<()> {
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
        let mut stack = self.stack.clone();
        stack.push(name.clone());
        let (lines, p) = read(kept, &stack).map_err(inside)?;
        for l in lines.into_iter().filter(|l| !l.is_empty()) {
            P::put(out, line, l);
        }
        self.all
            .extend(p.names.fns.keys().chain(p.names.claims.keys()).cloned());
        self.names.fns.extend(p.names.fns);
        self.names.claims.extend(p.names.claims);
        self.names.specified.extend(p.names.specified);
        self.names.opaque.extend(p.names.opaque);
        self.names.derived.extend(p.names.derived);
        self.names.trust.extend(p.names.trust);
        self.names.declared.extend(p.names.declared);
        self.names.boundaries.extend(p.names.boundaries);
        self.variants.extend(p.variants);
        self.names.represented.extend(p.names.represented);
        self.templates.extend(p.templates);
        Ok(())
    }

    /// `claim c = shape(f, 0..=9, |a| a > 0) trust 0.9;`: the claim `shape`
    /// with its parameters replaced by what is written here.
    fn instance(
        &mut self,
        out: &mut Vec<String>,
        line: usize,
        assumed: bool,
        name: &str,
        form: &str,
    ) -> R<()> {
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
        let done = self.claim(out, line, assumed);
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
    fn evidence(&mut self, out: &mut Vec<String>, test: bool) -> R<()> {
        let line = self.line();
        self.expect(if test { "test" } else { "evidence" })?;
        let label = self.name()?;
        self.expect("{")?;
        self.expect("for")?;
        let target = self.path()?;
        let scope = self.scope.take();
        let target = self.find(&target)?;
        self.scope = scope;
        let (mut given, mut says, mut trust) = (Vec::new(), None, "1".to_string());
        let (mut source, mut rev) = (format!("\"{label}\""), None);
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
                    let text = match exact {
                        true => self.value(&[[","].as_slice(), &clauses].concat())?,
                        false => self.ty()?,
                    };
                    given.push((x, exact, text));
                    if !self.eat(",") {
                        break;
                    }
                },
                "holds" | "fails" => says = Some((clause, String::new())),
                "gives" => {
                    let rate = self.is("rate") && self.peek_at(1) == "=";
                    self.pos += 2 * rate as usize;
                    let v = self.value(&clauses)?;
                    says = Some((if rate { "rate" } else { "gives" }.to_string(), v));
                }
                "trust" => trust = self.value(&clauses)?,
                "source" => source = self.value(&clauses)?,
                "rev" => rev = Some(self.value(&clauses)?),
                other => {
                    let msg = format!("expected one of {}, got `{other}`", clauses.join(", "));
                    return Err(Error::new(at, msg));
                }
            }
        }
        let Some((says, value)) = says else {
            return Err(Error::new(
                line,
                "the evidence says nothing: `holds`, `fails` or `gives …`",
            ));
        };
        let at = |m: String| Err(Error::new(line, m));
        let function = self.names.fns.get(&target).cloned();
        let names: Vec<String> = match (&function, self.names.claims.get(&target)) {
            (Some(params), _) => params.iter().map(|p| p.0.clone()).collect(),
            (None, Some(claim)) => claim.0.clone(),
            (None, None) => return at(format!("the model has no `{target}`")),
        };
        if let Some((x, _, _)) = given.iter().find(|g| !names.contains(&g.0)) {
            return at(format!("`{target}` has no input `{x}`"));
        }
        // Each input: its value, a set, or every value.
        let (mut forall, mut args) = (String::new(), Vec::new());
        for x in &names {
            match given.iter().find(|g| &g.0 == x) {
                Some((_, true, v)) => args.push(v.clone()),
                Some((_, false, set)) => {
                    forall += &format!("forall {x} in {set}, ");
                    args.push(x.clone());
                }
                None => {
                    forall += &format!("forall {x}, ");
                    args.push(x.clone());
                }
            }
        }
        let call = format!("{target}({})", args.join(", "));
        let judgment = match (says.as_str(), function.is_some()) {
            ("rate", _) => {
                let measures = self.names.claims.get(&target).map(|c| c.1.clone());
                match measures.as_deref() {
                    Some([one]) => format!("{one} = {value}"),
                    _ => return at(format!("`{target}` is not a claim on one measure")),
                }
            }
            ("gives", true) => format!("{forall}{call} = {value}"),
            ("gives", false) => return at(format!("`{target}` is a claim: `holds` or `fails`")),
            ("holds", true) => format!("{forall}correct({call})"),
            ("fails", true) => format!("{forall}not correct({call})"),
            // The whole claim, or the claim on some inputs.
            ("holds", false) if given.is_empty() => target.clone(),
            ("fails", false) if given.is_empty() => format!("not {target}"),
            ("holds", false) => format!("{forall}{call}"),
            (_, false) => format!("{forall}not {call}"),
            _ => unreachable!(),
        };
        if test {
            P::put(
                out,
                line,
                format!("test {} : {judgment}", self.qualified(&label)),
            );
            return Ok(());
        }
        let attributes = match rev {
            Some(r) => format!("@[source {source}, rev {r}]"),
            None => format!("@[source {source}]"),
        };
        P::put(
            out,
            line,
            format!("{attributes} evidence {judgment} at {trust}"),
        );
        Ok(())
    }
}

/// The model in the earlier syntax, line for line, and its names.
pub fn model(src: &str) -> Result<(String, Names), Error> {
    let (out, p) = read(lex(src)?, &[])?;
    Ok((out.join("\n"), p.names))
}

/// One file: its declarations in the earlier syntax, by line.
fn read(toks: Vec<Tok>, stack: &[String]) -> Result<(Vec<String>, P), Error> {
    // Every function and claim, before reading any: one may name another
    // declared further down.
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
    // Twice: the second reading knows every function of the file.
    let mut known = Names::default();
    let mut last = None;
    for pass in 0..2 {
        let all = all.clone();
        let mut p = P {
            toks: toks.clone(),
            pos: 0,
            scope: None,
            all,
            variants: BTreeMap::new(),
            locals: Vec::new(),
            own: None,
            measures: Vec::new(),
            names: Names::default(),
            fresh: 0,
            tests: Vec::new(),
            called: Vec::new(),
            first: pass == 0,
            known,
            impure: false,
            templates: BTreeMap::new(),
            stack: stack.to_vec(),
            shape: false,
        };
        let mut out = Vec::new();
        p.items(&mut out)?;
        if p.pos < p.toks.len() {
            return p.err(format!("unexpected `{}`", p.peek()));
        }
        let all: Vec<String> = p
            .names
            .fns
            .keys()
            .chain(p.names.claims.keys())
            .cloned()
            .collect();
        p.all = all.into_iter().collect();
        for at in std::mem::take(&mut p.tests) {
            p.pos = at;
            p.evidence(&mut out, true)?;
        }
        known = p.names.clone();
        last = Some((out, p));
    }
    Ok(last.expect("read twice"))
}

/// The evidence in the earlier syntax, line for line, then what the
/// model itself declares.
pub fn evidence(src: &str, names: &Names) -> Result<String, Error> {
    let mut out = evidence_alone(src, names)?;
    for d in &names.declared {
        out.push('\n');
        out += d;
    }
    Ok(out)
}

/// The evidence in the earlier syntax, line for line.
pub fn evidence_alone(src: &str, names: &Names) -> Result<String, Error> {
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
    let names = &names;
    let mut p = P {
        toks: lex(src)?,
        pos: 0,
        scope: None,
        all,
        variants: BTreeMap::new(),
        locals: Vec::new(),
        own: None,
        measures: Vec::new(),
        names: names.clone(),
        fresh: 0,
        tests: Vec::new(),
        known: names.clone(),
        called: Vec::new(),
        first: false,
        impure: false,
        templates: BTreeMap::new(),
        stack: Vec::new(),
        shape: false,
    };
    let mut out = Vec::new();
    let lines: Vec<&str> = src.lines().collect();
    while p.pos < p.toks.len() {
        let block = p.is("evidence") && p.peek_at(2) == "{" && p.kind_at(1) == Some(Kind::Name);
        if block {
            p.evidence(&mut out, false)?;
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
    out.resize(src.lines().count().max(out.len()), String::new());
    Ok(out.join("\n"))
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
pub fn tied(evidence: &str, names: &Names) -> Vec<String> {
    let found = |f: &&String| evidence.contains(&format!("{f}::tied"));
    names.opaque.iter().filter(found).cloned().collect()
}
