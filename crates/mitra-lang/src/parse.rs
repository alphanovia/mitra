//! The reader of the Mitra language: `subject` and `boundary` blocks,
//! `fn`, `claim` with inputs, `assume`, evidence blocks. It builds the
//! model as it reads (docs/reference/language.md).

use crate::ast::*;
use crate::set::{Set, Value, MAP, SET};
use crate::Error;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

/// The modules shipped with the tool: plain Mitra, `mitra explain std::claims`
/// prints one.
const STD: [(&str, &str); 3] = [
    ("std::claims", include_str!("../std/claims.mitra")),
    ("std::core", include_str!("../std/core.mitra")),
    ("std::rust", include_str!("../std/rust.mitra")),
];

/// The modules a model may import, as they were handed to Mitra: this
/// crate reads no file. `a::b` is the name of what `import a::b::x;`
/// reads.
#[derive(Debug, Clone, Default, PartialEq)]
// mitra: implements modules::Handed
pub struct Handed {
    /// The text of each module, by name.
    pub modules: BTreeMap<String, String>,
    /// Why a module is not there, in the words of whoever looked for it.
    pub missing: BTreeMap<String, String>,
    /// The whole model of each other store that could be found, by the
    /// name of the store: `import store::block::{Name};` reads there.
    pub stores: BTreeMap<String, String>,
    /// The settings of the store, as its file states them.
    pub settings: crate::settings::Settings,
}

/// The text of a module shipped with the tool.
pub fn library(module: &str) -> Option<&'static str> {
    STD.iter().find(|m| m.0 == module).map(|m| m.1)
}

/// The text of a module: shipped with the tool, or handed.
fn module(handed: &Handed, path: &[String]) -> Result<String, String> {
    let name = path.join("::");
    if path[0] == "std" {
        let known: Vec<&str> = STD.iter().map(|m| m.0).collect();
        let none = format!("no module `{name}` in `std`: it has {}", known.join(", "));
        return library(&name).map(String::from).ok_or(none);
    }
    if let Some(text) = handed.modules.get(&name) {
        return Ok(text.clone());
    }
    Err(match handed.missing.get(&name) {
        Some(why) => format!("no module `{name}`: {why}"),
        None => format!("no module `{name}`: this model was handed none of that name"),
    })
}

/// What `import store::block::{names};` brings from the model of another
/// store: each named record, enum or `ensure` of that block, in a
/// `boundary` of the same name. Another store is bought here, not built:
/// what it promises is assumed, and its functions and claims stay there.
fn from_store(toks: &[Tok], block: &str, names: &[String]) -> Result<Vec<Tok>, String> {
    let open = |t: &Tok| t.kind == Kind::Name && (t.text == "subject" || t.text == "boundary");
    let start = (0..toks.len()).find(|&i| {
        open(&toks[i])
            && (i == 0 || toks[i - 1].text == ";" || toks[i - 1].text == "}")
            && toks.get(i + 1).is_some_and(|n| n.text == block)
            && toks.get(i + 2).is_some_and(|b| b.text == "{")
    });
    let Some(start) = start else {
        return Err(format!("that store has no block `{block}`"));
    };
    // Every record and enum that store declares, by its short name.
    let decl =
        |w: &&[Tok]| w[0].kind == Kind::Name && matches!(w[0].text.as_str(), "record" | "enum");
    let theirs: BTreeSet<String> = toks
        .windows(2)
        .filter(decl)
        .map(|w| w[1].text.clone())
        .collect();
    let mut word = toks[start].clone();
    word.text = "boundary".to_string();
    let mut kept = vec![word, toks[start + 1].clone(), toks[start + 2].clone()];
    let (mut found, mut depth, mut keep, mut fresh) = (Vec::new(), 1, false, true);
    // Where the item being kept starts in `kept`.
    let mut item = 0;
    let mut i = start + 3;
    while i < toks.len() && depth > 0 {
        let t = &toks[i];
        if depth == 1 && fresh && t.kind == Kind::Name {
            let named = toks.get(i + 1).map_or("", |n| n.text.as_str());
            keep = matches!(t.text.as_str(), "record" | "enum" | "ensure")
                && names.iter().any(|n| n == named);
            if keep {
                found.push(named.to_string());
                item = kept.len();
            }
        }
        fresh = false;
        if t.kind == Kind::Punct {
            depth += (t.text == "{") as i32 - (t.text == "}") as i32;
            // An item ends at its `;`, or at the brace that closes it.
            fresh = depth == 1 && (t.text == ";" || t.text == "}");
        }
        if keep || depth == 0 {
            kept.push(t.clone());
        }
        // A record whose fields name a type of that store which is not
        // brought is brought without its fields: `record Model;`.
        if keep && fresh && t.text == "}" {
            let at = item;
            let body = &kept[at + 2..];
            let missing = |(j, k): (usize, &Tok)| {
                k.kind == Kind::Name
                    && body.get(j + 1).is_none_or(|n| n.text != ":")
                    && theirs.contains(&k.text)
                    && !names.contains(&k.text)
            };
            if kept[at].text == "record" && body.iter().enumerate().any(missing) {
                kept.truncate(at + 2);
                let mut end = t.clone();
                end.text = ";".to_string();
                kept.push(end);
            } else if let Some((_, k)) = body.iter().enumerate().find(|x| missing(*x)) {
                return Err(format!(
                    "`{}` holds a `{}`, which must be brought with it",
                    kept[at + 1].text,
                    k.text
                ));
            }
        }
        i += 1;
    }
    match names.iter().find(|n| !found.contains(n)) {
        Some(n) => Err(format!(
            "its block `{block}` has no record, enum or `ensure` named `{n}`"
        )),
        None => Ok(kept),
    }
}

/// The tokens of a text, with each name a `type` gave to a collection
/// replaced by the collection's: `Vec` by `List` after `import
/// std::rust::{Vec};`.
fn lex(src: &str) -> Result<Vec<Tok>, Error> {
    let mut toks = lex_raw(src)?;
    let mut renames = types_named(&toks);
    for (i, t) in toks.iter().enumerate() {
        if t.kind != Kind::Name
            || t.text != "import"
            || toks.get(i + 1).is_none_or(|n| n.text != "std")
        {
            continue;
        }
        let Some(part) = toks.get(i + 3).filter(|n| n.kind == Kind::Name) else {
            continue;
        };
        let Some(module) = library(&format!("std::{}", part.text)) else {
            continue;
        };
        let end = toks[i..]
            .iter()
            .position(|n| n.text == ";")
            .map_or(toks.len(), |e| i + e);
        for (name, named) in types_named(&lex_raw(module)?) {
            if toks[i + 4..end].iter().any(|n| n.text == name) {
                renames.insert(name, named);
            }
        }
    }
    if renames.is_empty() {
        return Ok(toks);
    }
    // Not where the name is declared or imported.
    let mut inside = false;
    for i in 0..toks.len() {
        if toks[i].kind == Kind::Name && (toks[i].text == "import" || toks[i].text == "type") {
            inside = true;
        } else if toks[i].text == ";" {
            inside = false;
        } else if !inside && toks[i].kind == Kind::Name {
            if let Some(named) = renames.get(&toks[i].text) {
                toks[i].text = named.clone();
            }
        }
    }
    Ok(toks)
}

/// `type Vec<T> = List<T>;` at the top of a text: `Vec` names `List`.
fn types_named(toks: &[Tok]) -> BTreeMap<String, String> {
    let (mut out, mut depth) = (BTreeMap::new(), 0);
    for (i, t) in toks.iter().enumerate() {
        if t.kind == Kind::Punct {
            depth += (t.text == "{") as i32 - (t.text == "}") as i32;
        }
        if depth != 0 || t.kind != Kind::Name || t.text != "type" {
            continue;
        }
        let equals = toks[i..]
            .iter()
            .position(|n| n.text == "=" || n.text == ";");
        if let (Some(name), Some(e)) = (toks.get(i + 1), equals) {
            if let Some(named) = toks.get(i + e + 1).filter(|n| n.kind == Kind::Name) {
                out.insert(name.text.clone(), named.text.clone());
            }
        }
    }
    out
}

/// Each `import` at the top of a text: its path and the names it brings.
fn imports_of(toks: &[Tok]) -> Vec<(Vec<String>, Vec<String>)> {
    let (mut out, mut depth) = (Vec::new(), 0);
    for (i, t) in toks.iter().enumerate() {
        if t.kind == Kind::Punct {
            depth += (t.text == "{") as i32 - (t.text == "}") as i32;
        }
        if depth != 0 || t.kind != Kind::Name || t.text != "import" {
            continue;
        }
        let mut words: Vec<String> = Vec::new();
        let mut j = i + 1;
        while toks.get(j).is_some_and(|n| n.text != ";" && n.text != "}") {
            if toks[j].kind == Kind::Name {
                words.push(toks[j].text.clone());
            }
            j += 1;
        }
        let braces = toks[i + 1..j.min(toks.len())].iter().any(|t| t.text == "{");
        let cut = if braces {
            2.min(words.len())
        } else {
            words.len().saturating_sub(1)
        };
        // With braces, only `store::block::{…}` matters here.
        if braces && toks.get(i + 4).is_none_or(|t| t.text != "::") {
            continue;
        }
        let names = words.split_off(cut);
        out.push((words, names));
    }
    out
}

/// Whether a path names a block of another store: nothing handed has
/// that name, and the model of a store named as its first word was.
fn of_store(handed: &Handed, path: &[String]) -> bool {
    path.len() == 2
        && path[0] != "std"
        && !handed.modules.contains_key(&path.join("::"))
        && handed.stores.contains_key(&path[0])
}

/// The modules a text imports, `std` aside, each once: what must be
/// handed with it.
// mitra: implements modules::imports
pub fn imports(src: &str) -> Vec<String> {
    let toks = lex(src).unwrap_or_default();
    let (mut out, mut depth) = (Vec::new(), 0);
    for (i, t) in toks.iter().enumerate() {
        if t.kind == Kind::Punct {
            depth += (t.text == "{") as i32 - (t.text == "}") as i32;
        }
        if depth != 0 || t.kind != Kind::Name || t.text != "import" {
            continue;
        }
        // import a::b::name;  import a::b::{x, y};
        let mut path: Vec<&str> = Vec::new();
        let mut j = i + 1;
        while let (Some(n), Some(sep)) = (toks.get(j), toks.get(j + 1)) {
            if n.kind != Kind::Name || sep.text != "::" {
                break;
            }
            path.push(&n.text);
            j += 2;
        }
        let name = path.join("::");
        if !path.is_empty() && path[0] != "std" && !out.contains(&name) {
            out.push(name);
        }
    }
    out
}

/// How an error on a name the model does not have starts.
const NO_PIECE: &str = "the model has no function or claim";

/// One evidence block of a ledger, read without a model.
#[derive(Debug, Clone, Default, PartialEq)]
// mitra: implements ledger::Entry
pub struct Entry {
    /// The piece it names, as written.
    pub about: String,
    pub source: Option<String>,
    pub rev: Option<String>,
    pub seen: Option<String>,
    /// Its first and its last line.
    pub first: usize,
    pub last: usize,
}

/// The declarations at the top of a file that start an item.
pub const ITEMS: [&str; 13] = [
    "subject", "boundary", "record", "enum", "fn", "claim", "assume", "test", "import", "ensure",
    "require", "contract", "type",
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

fn lex_raw(src: &str) -> Result<Vec<Tok>, Error> {
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
            // `r#match`, Rust's raw identifier: a name, whatever the word.
            let word = |c: char| c.is_ascii_alphabetic() || c == '_';
            let raw = rest.strip_prefix("r#").filter(|r| r.starts_with(word));
            let from = if raw.is_some() { 2 } else { 0 };
            let end = rest[from..]
                .find(|c: char| !(c.is_ascii_alphanumeric() || c == '_'))
                .map_or(rest.len(), |n| n + from);
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

/// The first line written in the syntax of before the rework (`opaque`, `def`, `:=`,
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
// mitra: implements reader::Names
pub struct Names {
    fns: BTreeMap<String, Sig>,
    /// Each claim with its inputs, and the measures in its body.
    claims: BTreeMap<String, (Vec<String>, Vec<Expr>)>,
    /// The functions of a subject that have a body: their spec.
    specified: BTreeMap<String, String>,
    /// The functions whose code is outside the model.
    pub opaque: BTreeSet<String>,
    /// The records, enums and variants of a block, by their full name,
    /// and those outside any block.
    pub scoped: BTreeSet<String>,
    pub top: BTreeSet<String>,
    /// For each claim, the spec claims of the functions it calls.
    pub derived: BTreeMap<String, Vec<String>>,
    /// The trust given to the statement of a claim.
    pub trust: BTreeMap<String, f64>,
    /// Evidence the model itself brings: what is assumed or declared.
    pub declared: Vec<Evidence>,
    /// The functions of the boundaries.
    pub boundaries: BTreeSet<String>,
    /// `claim c = deterministic(f)` and `claim c = !deterministic(f)`:
    /// the function, the claim, which of the two, the line.
    pub determinism: Vec<(String, String, bool, usize)>,
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
    "Any",
    "_",
    "Real",
];

/// The reserved words that only start a declaration: nothing else
/// begins with them, so each can also name an input.
const DECLARING: [&str; 5] = ["record", "fn", "claim", "evidence", "enum"];

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

/// Tokens joined by spaces, set as a person writes types: `Stack < T >`
/// is `Stack<T>`.
pub(crate) fn tidy(spaced: &str) -> String {
    let mut out = spaced.to_string();
    for (from, to) in [
        (" ,", ","),
        (" :", ":"),
        ("< ", "<"),
        (" <", "<"),
        (" >", ">"),
        ("( ", "("),
        (" (", "("),
        (" )", ")"),
        ("& ", "&"),
        ("-> (", "-> ("),
    ] {
        out = out.replace(from, to);
    }
    out.replace("->(", "-> (")
}

#[derive(Default)]
struct P {
    toks: Vec<Tok>,
    pos: usize,
    /// The subject or boundary being read.
    scope: Option<String>,
    /// `subject` or `boundary`, inside one: with a name (`scope`) or not.
    block: Option<String>,
    /// The blocks read so far: each `subject` or `boundary` has a number.
    blocks: usize,
    /// Each claim and function of a block, with the number of its block.
    owned: Vec<(String, usize)>,
    /// `ensure` and `require`: name, prose, kind, block, line.
    ensures: Vec<(String, String, Prose, usize, usize, Option<f64>)>,
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
    /// What was imported so far: the module and the name.
    imported: BTreeSet<(String, String)>,
    /// The modules that can be imported.
    handed: Rc<Handed>,
    /// The blocks this model declares itself, by name.
    own_blocks: Rc<BTreeSet<String>>,
    /// The blocks brought from other stores: the block, then its store.
    brought: BTreeMap<String, String>,
    /// While a block of another store is read: that store.
    bringing: Option<String>,
    /// Each enum already met with sets for its parameters, read deep or
    /// not, and the set it gave.
    instances: Vec<(String, bool, Vec<Set>, Set)>,
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
    /// The types and variants of the blocks, by their full name.
    scoped: BTreeSet<String>,
    /// The functions of an `impl` that take `self`, by their full name.
    methods: BTreeSet<String>,
    contracts: Rc<Contracts>,
    /// The inputs of the function or claim being read, with their types.
    typed: Vec<(String, Set)>,
    /// The functions and records of the first reading: what a call gives.
    seen: Model,
    enums: Vec<Enum>,
    /// The sets the type parameters stand for, where the parser is.
    generics: Vec<(String, Set)>,
    /// In a body: each opaque type of it, and the contracts it fills.
    opaque: Vec<(Set, Vec<String>)>,
    /// The bounds of the type parameters last read: `T: A + B`.
    bounds: Vec<(String, Vec<String>)>,
    /// A block, a contract a bound of it names, and the line.
    leaning: Vec<(String, String, usize)>,
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
            [] => self.err(format!("{NO_PIECE} `{name}`")),
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
    /// Any word, reserved or not: where only a name can stand, after a
    /// dot or before the `:` of a field.
    fn word(&mut self) -> R<String> {
        match self.kind() {
            Some(Kind::Name) => self.name(),
            _ => self.err("expected a name"),
        }
    }
    /// The name of an input or of a `let`: also a word that only starts
    /// a declaration, such as `claim`.
    fn local(&mut self) -> R<String> {
        match self.kind() == Some(Kind::Name) && DECLARING.contains(&self.peek()) {
            true => self.name(),
            false => self.ident(),
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
            if self.records.iter().any(|r| r.name == name && r.open) {
                return self.err(unwritten(&name));
            }
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
        if self.peek() == "Vec" && self.kind() == Some(Kind::Name) {
            return self.err("`Vec` is Rust's name for `List`: `import std::rust::{Vec};`");
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
                // The same enum with the same sets, met again: a record
                // of records is long to print, so it is worked out once.
                let again = |i: &&(String, bool, Vec<Set>, Set)| {
                    i.0 == name && i.1 == self.deep && i.2 == args
                };
                if let Some(met) = self.instances.iter().find(again) {
                    return Ok(met.3.clone());
                }
                let written = written(&args);
                let key = (name.clone(), self.deep, args.clone());
                let params = e.params.iter().cloned().zip(args).collect();
                let (variants, _, recursive) = self.variants(&e, params, line)?;
                let set = variants.iter().fold(Set::empty(), |s, v| s.union(&v.set()));
                self.instances.push((key.0, key.1, key.2, set.clone()));
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
                         Float64, String, Any, or a record or an enum declared above)"
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
                "Any" => Set::any(),
                "Real" => return self.err("`Real` is not supported yet: use `Float64`"),
                t => match Set::int_type(t) {
                    Some(s) => s,
                    None => {
                        return self.err(format!(
                            "unknown type `{t}` (Bool, Int, Int8 to Int64, UInt8 to UInt32, \
                             Float64, String, Any)"
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
            toks: merge_types(lex(&of.body).map_err(at)?, &self.scoped),
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
                open: false,
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
            let name = match self.peek_at(1) == ":" {
                true => self.word()?,
                false => self.ident()?,
            };
            if !self.is(":") {
                return self.err(format!("the input `{name}` needs a type"));
            }
            self.expect(":")?;
            // `o: &Mass`, as Rust writes it: a borrowed value is the same
            // value here.
            if self.eat("&") {
                self.eat("mut");
            }
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
                if !position && self.kind() != Some(Kind::Name) {
                    return self.err(
                        "expected a field name, or a position such as `.0`; \
                         for a position inside a position write `(t.0).1`",
                    );
                }
                let field = self.peek().to_string();
                self.pos += 1;
                // `c.full_name(x)` is `Claim::full_name(c, x)`.
                if !position && self.eat("(") {
                    let f = self.method(&a, &field)?;
                    let mut args = vec![a];
                    args.extend(self.exprs(")")?);
                    a = Expr::Call(f, args);
                    continue;
                }
                a = Expr::Field(Box::new(a), field);
            } else {
                return Ok(a);
            }
        }
    }
    /// The method `name` called on `on`: the one whose `self` the type
    /// of `on` can be. Where that type is not known, the only method of
    /// that name.
    fn method(&mut self, on: &Expr, name: &str) -> R<String> {
        let tail = format!("::{name}");
        let named: Vec<String> = (self.methods.iter())
            .filter(|m| m.ends_with(&tail))
            .cloned()
            .collect();
        let back = |p: &mut P, msg: String| {
            p.pos -= 2;
            p.err(msg)
        };
        if named.is_empty() {
            let msg =
                format!("no method `{name}`: a method is a `fn {name}(self, …)` in an `impl`");
            return back(self, msg);
        }
        // The first reading knows no function yet: any will do.
        let kind = crate::types::of(&self.seen, on, &mut self.typed.clone()).ok();
        let kind = kind.filter(|k| !Set::any().is_subset(k) && !self.first);
        let takes = |m: &String| {
            let own = self.seen.functions.iter().find(|f| &f.name == m);
            own.and_then(|f| f.params.first()).map(|p| p.1.clone())
        };
        let full = match (&kind, &named[..]) {
            (Some(k), _) => {
                let fits = |m: &&String| takes(m).is_some_and(|t| !t.intersect(k).is_empty());
                let mut fit: Vec<&String> = named.iter().filter(fits).collect();
                // An opaque type has the methods of its contracts only.
                if let Some((_, by)) = self.opaque.iter().find(|o| &o.0 == k) {
                    let of = |m: &&String| {
                        m.split("::")
                            .next()
                            .is_some_and(|c| by.iter().any(|b| b == c))
                    };
                    fit.retain(|m| self.contracts.methods.contains(*m) && of(m));
                }
                // Its own method before that of a contract, which any
                // value has.
                if fit.iter().any(|m| !self.contracts.methods.contains(*m)) {
                    fit.retain(|m| !self.contracts.methods.contains(*m));
                }
                match fit[..] {
                    [one] => one.clone(),
                    [] => {
                        let text = crate::print::expr(on);
                        let shown = k.to_string();
                        let kind = match self.aliases.iter().find(|a| a.0 == shown) {
                            Some(a) => a.1.clone(),
                            None => self.seen.named(&shown),
                        };
                        let msg = format!("`{text}` is `{kind}`, which has no method `{name}`");
                        return back(self, msg);
                    }
                    _ => {
                        let each: Vec<String> = fit.iter().map(|n| format!("`{n}`")).collect();
                        let msg = format!(
                            "`{name}` is a method of several types it can be: write one of {} as a function",
                            each.join(", ")
                        );
                        return back(self, msg);
                    }
                }
            }
            (None, [one]) => one.clone(),
            (None, _) if self.first => named[0].clone(),
            // Any value: the method of the contract, where only one has it.
            (None, _)
                if named
                    .iter()
                    .filter(|m| self.contracts.methods.contains(*m))
                    .count()
                    == 1 =>
            {
                let of = named.iter().find(|m| self.contracts.methods.contains(*m));
                of.expect("counted").clone()
            }
            (None, _) => {
                let each: Vec<String> = named.iter().map(|n| format!("`{n}`")).collect();
                let msg = format!(
                    "`{name}` is a method of several records: write one of {} as a function",
                    each.join(", ")
                );
                return back(self, msg);
            }
        };
        let full = self.resolve(&full);
        self.impure |= self.known.opaque.contains(&full);
        if self.own.is_none() && self.known.specified.contains_key(&full) {
            self.called.push(format!("{full}::spec"));
        }
        Ok(full)
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
        let x = self.local()?;
        self.expect("=")?;
        let v = self.expr()?;
        self.expect(";")?;
        self.locals.push(x.clone());
        // What it is bound to says what its methods are.
        let kind = crate::types::of(&self.seen, &v, &mut self.typed.clone());
        self.typed.push((x.clone(), kind.unwrap_or(Set::any())));
        let body = self.sequence();
        self.typed.pop();
        self.locals.pop();
        Ok(Expr::Let(x, Box::new(v), Box::new(body?)))
    }
    /// `|a, b|`, after the first `|`.
    fn bars(&mut self) -> R<Vec<String>> {
        let mut names = Vec::new();
        while !self.eat("|") {
            names.push(self.local()?);
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
            let typed = self.typed.len();
            self.typed
                .extend(names.iter().map(|x| (x.clone(), Set::any())));
            let body = self.expr();
            self.locals.truncate(n);
            self.typed.truncate(typed);
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
                self.typed.push((x.clone(), Set::any()));
                let cond = self.expr();
                self.locals.pop();
                self.typed.pop();
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
                let typed = self.typed.len();
                let variant = match self.eat("_") {
                    true => None,
                    false => {
                        let v = self.local()?;
                        let mut fields = Vec::new();
                        if self.eat("(") {
                            while !self.eat(")") {
                                // `_` for a field nobody reads.
                                let x = match self.eat("_") {
                                    true => {
                                        self.fresh += 1;
                                        format!("unread_{}", self.fresh)
                                    }
                                    false => self.local()?,
                                };
                                fields.push(x);
                                if !self.eat(",") {
                                    self.expect(")")?;
                                    break;
                                }
                            }
                        }
                        self.locals.extend(fields.iter().cloned());
                        // A field of a variant has the type the variant gives it.
                        let of = self.seen.records.iter().find(|r| r.name == v);
                        let kind = |i: usize| of.and_then(|r| r.fields.get(i)).map(|f| f.1.clone());
                        let typed = fields.iter().enumerate();
                        let typed: Vec<(String, Set)> = typed
                            .map(|(i, x)| (x.clone(), kind(i).unwrap_or(Set::any())))
                            .collect();
                        self.typed.extend(typed);
                        Some((v, fields))
                    }
                };
                self.expect("=>")?;
                let body = match self.is("{") && self.peek_at(1) == "let" {
                    true => self.block(),
                    false => self.expr(),
                };
                self.locals.truncate(n);
                self.typed.truncate(typed);
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
            let x = self.local()?;
            self.expect("in")?;
            let set = self.set()?;
            self.expect(",")?;
            self.locals.push(x.clone());
            self.typed.push((x.clone(), Set::any()));
            let cond = self.expr();
            self.locals.pop();
            self.typed.pop();
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
            let x = self.local()?;
            self.expect("in")?;
            let list = self.sum()?;
            self.expect(",")?;
            self.locals.push(x.clone());
            self.typed.push((x.clone(), Set::any()));
            let body = self.sequence();
            self.locals.pop();
            self.typed.pop();
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
        let input =
            DECLARING.contains(&self.peek()) && self.locals.iter().any(|l| l == self.peek());
        if !self.is_name() && !input {
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
        if self.block.is_some() {
            self.owned.push((name.to_string(), self.blocks));
        }
        self.m.claims.push(Claim {
            name: name.to_string(),
            func: func.to_string(),
            pattern,
            target,
            from: Vec::new(),
            auto: false,
            proved: false,
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
            seen: None,
            line: 0,
            act: None,
            by: None,
            written: None,
            cold: false,
            of: None,
        });
    }
    /// `<T, U>` after a name.
    fn type_params(&mut self, of: &str) -> R<Vec<String>> {
        let mut names = Vec::new();
        self.bounds.clear();
        if !self.eat("<") {
            return Ok(names);
        }
        loop {
            let t = self.ident()?;
            if names.contains(&t) {
                return self.err(format!("`{of}` has two parameters `{t}`"));
            }
            if self.eat(":") {
                let bound = self.bounded_by()?;
                self.bounds.push((t.clone(), bound));
            }
            names.push(t);
            if !self.eat(",") {
                self.expect(">")?;
                return Ok(names);
            }
        }
    }
    /// `A + B<X>` after `T:`: each names a contract.
    fn bounded_by(&mut self) -> R<Vec<String>> {
        let mut all = Vec::new();
        loop {
            let name = self.ident()?;
            if !self.contracts.names.contains(&name) {
                self.pos -= 1;
                return self.err(format!(
                    "`{name}` is not a contract: a bound names a contract"
                ));
            }
            all.push(name);
            if self.eat("<") {
                self.type_tokens(">")?;
                while self.eat(",") {
                    self.type_tokens(">")?;
                }
                self.expect(">")?;
            }
            if !self.eat("+") {
                return Ok(all);
            }
        }
    }
    /// No bound on a `record` or an `enum`: a bound says what a function
    /// leans on.
    fn unbounded(&mut self, of: &str) -> R<()> {
        match self.bounds.first() {
            Some((t, _)) => self.err(format!(
                "`{t}: …` on `{of}`: a bound is written on a function, an `impl` or a contract"
            )),
            None => Ok(()),
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

    /// `<A, B>` after a name, or nothing.
    fn names_between(&mut self) -> R<Vec<String>> {
        let mut params = Vec::new();
        if self.eat("<") {
            while !self.eat(">") {
                params.push(self.name()?);
                if !self.eat(",") {
                    self.expect(">")?;
                    break;
                }
            }
        }
        Ok(params)
    }

    fn items(&mut self) -> R<()> {
        while self.pos < self.toks.len() && !self.is("}") {
            let line = self.at.unwrap_or(self.line());
            let kw = self.name()?;
            match kw.as_str() {
                // type Vec<T> = List<T>;  another name for a collection, replaced
                // where the text was read (`lex`): checked here.
                "type" if self.scope.is_none() && self.block.is_none() => {
                    let name = self.name()?;
                    let params = self.names_between()?;
                    self.expect("=")?;
                    let named = self.name()?;
                    if !["List", "Set", "Map"].contains(&named.as_str()) {
                        return self.err(format!(
                            "`type {name}` names `{named}`: a `type` is another name for `List`, `Set` or `Map`"
                        ));
                    }
                    if self.names_between()? != params {
                        return self.err(format!(
                            "`type {name}` gives `{named}` other parameters than its own"
                        ));
                    }
                    self.eat(";");
                }
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
                // subject fn f(a: T) -> U;  one function, named as written.
                "subject" | "boundary" if self.block.is_none() && self.is("fn") => {
                    self.blocks += 1;
                    self.block = Some(kw.clone());
                    self.pos += 1;
                    let done = self.function(line);
                    self.block = None;
                    done?;
                }
                // subject name { … }, or subject { … }: named as written.
                "subject" | "boundary" if self.block.is_none() => {
                    if !self.is("{") {
                        let name = self.name()?;
                        let side = match kw.as_str() {
                            "subject" => Side::Subject,
                            _ if self.contracts.names.contains(&name) => Side::Contract,
                            _ => Side::Boundary,
                        };
                        if !self.m.blocks.iter().any(|b| b.name == name) {
                            let brought = self.contracts.brought.get(&name).cloned();
                            let (name, from) = (name.clone(), brought.or(self.bringing.clone()));
                            self.m.blocks.push(Block {
                                name,
                                side,
                                from,
                                line,
                            });
                        }
                        self.scope = Some(name);
                    }
                    self.blocks += 1;
                    self.block = Some(kw.clone());
                    self.expect("{")?;
                    self.items()?;
                    self.expect("}")?;
                    (self.scope, self.block) = (None, None);
                }
                "subject" | "boundary" => {
                    self.pos -= 1;
                    return self.err(format!("a `{kw}` inside another is not supported yet"));
                }
                "record" => {
                    // record Name { field: T, … }
                    let name = self.ident()?;
                    let params = self.type_params(&name)?;
                    self.unbounded(&name)?;
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
                    // `record Name;`: nothing of it is stated. `..` before
                    // the `}`: it has more than the fields stated.
                    let mut open = self.eat(";");
                    let nothing = open;
                    if !open && self.is("{") {
                        let (mut depth, mut end) = (0, self.pos);
                        while let Some(t) = self.toks.get(end) {
                            match t.text.as_str() {
                                "{" => depth += 1,
                                "}" => depth -= 1,
                                _ => {}
                            }
                            if depth == 0 {
                                break;
                            }
                            end += 1;
                        }
                        if end > self.pos + 1
                            && self.toks.get(end - 1).is_some_and(|t| t.text == "..")
                        {
                            open = true;
                            self.toks.remove(end - 1);
                            if end > self.pos + 2 && self.toks[end - 2].text == "," {
                                self.toks.remove(end - 2);
                            }
                        }
                    }
                    self.me = Some((name.clone(), name.clone()));
                    self.met = false;
                    let both = match nothing {
                        true => Ok((Vec::new(), Vec::new())),
                        false => self.both(|p| p.binders("{", "}")),
                    };
                    self.me = None;
                    let (mut fields, mut deep) = both?;
                    if open {
                        fields.push(("..".to_string(), Set::any()));
                        deep.push(Set::any());
                    }
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
                        open,
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
                    self.unbounded(&name)?;
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
                // ensure name = "prose";  what this subject promises.
                "ensure" if self.block.is_some() => {
                    let name = self.ident()?;
                    let q = self.qualified(&name);
                    if !self.eat("=") || self.kind() != Some(Kind::Str) {
                        return self.err(format!(
                            "`ensure {name}` is written with `=` and its text: \
                             ensure {name} = \"what is promised\";"
                        ));
                    }
                    let prose = unquote(&self.toks[self.pos].text);
                    self.pos += 1;
                    let trust = self.trust()?;
                    self.expect(";")?;
                    let kind = match self.block.as_deref() {
                        Some("boundary") => Prose::World,
                        _ => Prose::Promise,
                    };
                    self.ensures
                        .push((q, prose, kind, self.blocks, line, trust));
                }
                // require name = other::promise;  a promise made elsewhere,
                // needed here; or `require name = "prose";`, with nobody to
                // promise it yet.
                "require" if self.block.is_some() => {
                    let start = self.pos;
                    let name = self.ident()?;
                    if !self.eat("=") {
                        self.pos = start;
                        let path = self.path()?;
                        return self.err(format!(
                            "`require` gives what it needs a name: require <name> = {path}; \
                             reports and evidence point at it by that name"
                        ));
                    }
                    let q = self.qualified(&name);
                    let (prose, kind) = if self.kind() == Some(Kind::Str) {
                        self.pos += 1;
                        (unquote(&self.toks[self.pos - 1].text), Prose::Wanted)
                    } else {
                        (String::new(), Prose::Need(self.path()?))
                    };
                    let trust = self.trust()?;
                    self.expect(";")?;
                    self.ensures
                        .push((q, prose, kind, self.blocks, line, trust));
                }
                "ensure" | "require" => {
                    self.pos -= 1;
                    return self.err(format!(
                        "`{kw}` is written in a `subject {{ … }}` or a `boundary {{ … }}`"
                    ));
                }
                "claim" | "assume" => self.claim(line, kw == "assume")?,
                "contract" => {
                    self.pos -= 1;
                    let msg = "a `contract` is written outside any block, with a name and \
                               braces, in the model that fills it: `import` does not bring one";
                    return self.err(msg);
                }
                other => {
                    self.pos -= 1;
                    return self.err(format!(
                        "expected `import`, `subject`, `boundary`, `contract`, `record`, `enum`, \
                         `fn`, `claim` or `assume`, got `{other}`"
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
        let type_names = self.type_params(&name)?;
        let inputs_at = self.pos;
        let mut bounds = std::mem::take(&mut self.bounds);
        for t in &type_names {
            self.generics.push((t.clone(), Set::any()));
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
        // An input can be read in a body: not a word an expression starts with.
        let taken = |p: &&(String, Set)| KEYWORDS.contains(&&*p.0) && !DECLARING.contains(&&*p.0);
        if let Some((word, _)) = params.iter().find(taken) {
            let msg = format!("`{word}` is a keyword: as an input, write `r#{word}`");
            return Err(Error::new(line, msg));
        }
        // where L: Link, M: A + B
        if self.eat("where") {
            loop {
                let t = self.ident()?;
                if !type_names.contains(&t) {
                    self.pos -= 1;
                    return self.err(format!("`{name}` has no type parameter `{t}`"));
                }
                self.expect(":")?;
                let bound = self.bounded_by()?;
                bounds.push((t, bound));
                if !self.eat(",") {
                    break;
                }
            }
        }
        let written = generic.then(|| tidy(&join(&self.toks[head..self.pos])));
        // What each input is bound by: its type is a bound parameter,
        // or it was written `impl Contract`.
        let mut bound: Vec<(String, Vec<String>)> = Vec::new();
        for (t, contracts) in &bounds {
            for (i, tok) in self.toks[head..self.pos].iter().enumerate() {
                let (mut j, at) = (head + i, |j: usize| self.toks[j].text.as_str());
                let alone = &tok.text == t
                    && tok.kind == Kind::Name
                    && [",", ")"].contains(&at(j + 1))
                    && j > head + 2;
                while alone && ["&", "mut"].contains(&at(j - 1)) {
                    j -= 1;
                }
                if alone && at(j - 1) == ":" {
                    bound.push((at(j - 2).to_string(), contracts.clone()));
                }
            }
            if let Some(block) = &self.scope {
                for c in contracts {
                    self.leaning.push((block.clone(), c.clone(), line));
                }
            }
        }
        let last = name.rsplit("::").next().unwrap_or(&name);
        let at = self.toks[head - 1].line;
        for (_, _, input, contracts) in
            (self.contracts.sugar.iter()).filter(|s| s.0 == at && s.1 == last)
        {
            bound.push((input.clone(), contracts.clone()));
        }
        self.m.bounds.retain(|b| b.function != q);
        for (input, contracts) in &bound {
            self.m.bounds.push(Bounded {
                function: q.clone(),
                input: input.clone(),
                contracts: contracts.clone(),
            });
        }
        // Inside the function a type parameter is opaque: a record with
        // nothing stated, known by the contracts that bound it. The
        // inputs are read once more with it; a caller still gives any
        // value.
        let opaque = |t: &str| Set::record(&format!("<{q}::{t}>"), vec![Set::any()]);
        let mut inside = params.clone();
        let mut known: Vec<(Set, Vec<String>)> = Vec::new();
        if generic {
            let (end, met, direct) = (self.pos, self.met, self.direct);
            self.pos = inputs_at;
            for t in &type_names {
                let set = opaque(t);
                self.aliases.push((set.to_string(), t.clone()));
                let by = bounds.iter().filter(|b| &b.0 == t);
                known.push((set.clone(), by.flat_map(|b| b.1.clone()).collect()));
                self.generics.push((t.clone(), set));
            }
            let read = self.binders("(", ")");
            self.generics.clear();
            (self.pos, self.met, self.direct) = (end, met, direct);
            inside = read?;
        }
        // `x: impl Link`: the same, with no name.
        for (input, contracts) in &bound {
            let at = params.iter().position(|p| &p.0 == input);
            let Some(at) = at.filter(|at| params[*at].1 == Set::any()) else {
                continue;
            };
            if inside[at].1 != Set::any() {
                continue;
            }
            let written = format!("impl {}", contracts.join(" + "));
            let set = opaque(&format!("{input}: {written}"));
            self.aliases.push((set.to_string(), written));
            known.push((set.clone(), contracts.clone()));
            inside[at].1 = set;
        }
        self.m.generics.retain(|g| g.function != q);
        if inside != params {
            self.m.generics.push(Generic {
                function: q.clone(),
                params: inside.clone(),
                known: known.clone(),
            });
        }
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
        // `trust 0.9` on a helper: how far its definition is trusted.
        if self.block.is_none() {
            if let Some(t) = self.trust()? {
                self.m.helper_trust.push((q.clone(), t));
            }
        }
        self.names.fns.insert(q.clone(), sig.clone());
        if self.block.is_some() {
            self.names.opaque.insert(q.clone());
            self.owned.push((format!("{q}::correct"), self.blocks));
        }
        if self.block.as_deref() == Some("boundary") {
            self.names.boundaries.insert(q.clone());
        }
        if self.is(";") && self.block.is_none() {
            let msg = format!(
                "`fn {name}` has no body: write `subject fn {name}(…)` if it is your code, \
                 `boundary fn {name}(…)` if it is the world. A plain `fn` is a helper \
                 of the model and needs a body"
            );
            return Err(Error::new(line, msg));
        }
        if self.eat(";") {
            self.define(&q, &sig, range, None, line);
        } else {
            let in_subject = self.block.is_some();
            let body_name = match in_subject {
                true => format!("{q}::body"),
                false => q.clone(),
            };
            self.locals = sig.params.iter().map(|p| p.0.clone()).collect();
            self.typed = inside;
            self.opaque = known;
            // In a subject, the body is the spec of the code: it calls
            // itself as a spec.
            self.own = Some((name.clone(), body_name.clone()));
            let body = self.block();
            self.own = None;
            self.locals.clear();
            self.opaque.clear();
            let body = body?;
            // No `uses` written: what the body calls of the code and of
            // the world, and nothing else, is implied.
            if in_subject && uses.is_none() {
                let mut used: Vec<String> = Vec::new();
                for g in crate::ast::calls(&body) {
                    let code = self.known.opaque.contains(&g) || self.names.opaque.contains(&g);
                    if code && g != q && !used.contains(&g) {
                        used.push(g);
                    }
                }
                if !used.is_empty() {
                    uses = Some((true, used, 1.0));
                }
            }
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
            let x = self.local()?;
            if into.iter().any(|i| i.0 .0 == x) {
                return Err(Error::new(line, format!("`{x}` is an input twice")));
            }
            let (set, deep) = match self.eat(":") {
                true => {
                    // `o: &Mass`, as Rust writes it: a borrowed value is
                    // the same value here.
                    if self.eat("&") {
                        self.eat("mut");
                    }
                    let (set, deep) = self.set_both()?;
                    (Some(set), Some(deep))
                }
                false => (None, None),
            };
            self.locals.push(x.clone());
            self.typed
                .push((x.clone(), set.clone().unwrap_or(Set::any())));
            into.push(((x, set, line), deep));
            if !self.eat(",") {
                break;
            }
        }
        Ok(())
    }

    fn claim(&mut self, line: usize, assumed: bool) -> R<()> {
        let name = self.ident()?;
        let named_at = self.toks[self.pos - 1].line;
        let q = self.qualified(&name);
        // claim c = deterministic(f);  claim c = !deterministic(f);
        let not = self.is("=") && self.peek_at(1) == "!";
        if self.is("=") && self.peek_at(1 + usize::from(not)) == "deterministic" {
            self.pos += 2 + usize::from(not);
            self.expect("(")?;
            let f = self.path()?;
            let f = self.resolve(&f);
            self.expect(")")?;
            if let Some(t) = self.trust()? {
                self.names.trust.insert(q.clone(), t);
            }
            self.eat(";");
            if !self.first && !self.known.opaque.contains(&f) {
                let msg = format!("`{f}` is no function of a subject or a boundary");
                return Err(Error::new(line, msg));
            }
            if self.block.is_some() {
                self.owned.push((q.clone(), self.blocks));
            }
            self.names.determinism.push((f, q, !not, line));
            return Ok(());
        }
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
        self.typed.clear();
        self.opaque.clear();
        if self.eat("(") {
            self.inputs(")", &mut inputs)?;
            self.expect(")")?;
        }
        // `x: impl Link`: opaque in the claim, known by its contract.
        let last = name.rsplit("::").next().unwrap_or(&name).to_string();
        let sugar = self.contracts.clone();
        let mut inside = Vec::new();
        for (_, _, input, contracts) in
            (sugar.sugar.iter()).filter(|s| s.0 == named_at && s.1 == last)
        {
            let written = format!("impl {}", contracts.join(" + "));
            let set = Set::record(&format!("<{q}::{input}: {written}>"), vec![Set::any()]);
            self.aliases.push((set.to_string(), written));
            for t in self.typed.iter_mut().filter(|t| &t.0 == input) {
                t.1 = set.clone();
            }
            self.opaque.push((set.clone(), contracts.clone()));
            inside.push((input.clone(), set));
        }
        self.m.generics.retain(|g| g.function != q);
        if !inside.is_empty() {
            self.m.generics.push(Generic {
                function: q.clone(),
                params: inside,
                known: self.opaque.clone(),
            });
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
        // A name already brought from this module is there: each file of
        // a model states what it reads, and several may read the same.
        let fresh: Vec<String> = names
            .iter()
            .filter(|n| !self.imported.contains(&(name.clone(), (*n).clone())))
            .cloned()
            .collect();
        if fresh.is_empty() && !names.is_empty() {
            return Ok(());
        }
        let names = &fresh[..];
        for n in names {
            self.imported.insert((name.clone(), n.clone()));
        }
        let inside =
            |e: Error| Error::new(line, format!("in `{name}`, line {}: {}", e.line, e.message));
        let (mut kept, mut found, mut depth, mut keep) = (Vec::new(), Vec::new(), 0, false);
        if of_store(&self.handed, path) {
            // A block of another store: its records and promises alone.
            if self.own_blocks.contains(&path[1]) {
                let said = format!(
                    "`{}` is a block of this model too: a block of another store cannot be brought under the name of one declared here",
                    path[1]
                );
                return Err(Error::new(line, said));
            }
            let from = self
                .brought
                .entry(path[1].clone())
                .or_insert(path[0].clone());
            if *from != path[0] {
                let said = format!(
                    "a block `{}` was brought from `{from}` already: two stores cannot bring a block of the same name",
                    path[1]
                );
                return Err(Error::new(line, said));
            }
            let toks = lex(&self.handed.stores[&path[0]]).map_err(inside)?;
            let said = |e| Error::new(line, format!("`{}`: {e}", path[0]));
            // A type known by its path was written short: `Outcome`.
            let short = |n: &String| n.rsplit("::").next().unwrap_or(n).to_string();
            let wanted: Vec<String> = names.iter().map(short).collect();
            kept = from_store(&toks, &path[1], &wanted).map_err(said)?;
            let (scoped, top) = types_of(&kept);
            self.bringing = Some(path[0].clone());
            kept = scope_types(kept, &scoped, &top, false);
            found = names.to_vec();
        } else {
            let src = module(&self.handed, path).map_err(|e| Error::new(line, e))?;
            let toks = lex(&src).map_err(inside)?;
            // Only what is named, and what the module itself imports.
            // Inside `subject { … }` with no name: its word, put before each
            // function taken out of it.
            let mut unnamed: Option<&Tok> = None;
            let block = |t: &Tok| t.text == "subject" || t.text == "boundary";
            for (i, t) in toks.iter().enumerate() {
                let next = |n: usize| toks.get(i + n).map_or("", |n| n.text.as_str());
                let item = t.kind == Kind::Name && ITEMS.contains(&t.text.as_str());
                let word_before = i > 0 && depth == 0 && block(&toks[i - 1]) && t.text == "fn";
                if depth == 0 && item && block(t) && next(1) == "{" {
                    (unnamed, keep) = (Some(t), false);
                } else if depth == 0 && item && block(t) && next(1) == "fn" {
                    // subject fn f(…): one function, with its word.
                    keep = names.iter().any(|n| n == next(2));
                    if keep {
                        found.push(next(2).to_string());
                    }
                } else if depth == 0 && item && !word_before {
                    keep = t.text == "import" || names.iter().any(|n| n == next(1));
                    if keep && t.text != "import" {
                        found.push(next(1).to_string());
                    }
                } else if depth == 1 && item && unnamed.is_some() {
                    keep = names.iter().any(|n| n == next(1));
                    if keep {
                        found.push(next(1).to_string());
                        kept.extend(unnamed.filter(|_| t.text == "fn").cloned());
                    }
                }
                if t.kind == Kind::Punct {
                    depth += (t.text == "{") as i32 - (t.text == "}") as i32;
                    if depth == 0 && t.text == "}" && unnamed.take().is_some() {
                        keep = false;
                    }
                }
                if keep {
                    kept.push(t.clone());
                }
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
        self.bringing = None;
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
        let (mut source, mut rev, mut seen) = (label, None, None);
        // `sampled 1000`: the count, whether it passed, and the domain as
        // written (`m in valid, o in valid`).
        let mut sampled: Option<(u64, bool)> = None;
        let mut domain: Vec<String> = Vec::new();
        let mut by_helper = false;
        let clauses = [
            "given", "holds", "fails", "passes", "gives", "sampled", "trust", "source", "rev",
            "seen", "act", "by", "written", "cold", "of",
        ];
        let (mut act, mut by, mut written, mut cold, mut of) = (None, None, None, false, None);
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
                    // `m in valid`: the values a helper of the model says
                    // yes to. A domain to sample from, never a set to cover.
                    let helper = (!exact).then(|| self.resolve(self.peek())).filter(|h| {
                        let one = self.names.fns.get(h).is_some_and(|s| s.params.len() == 1);
                        one && !self.names.opaque.contains(h)
                    });
                    let from = self.pos;
                    if helper.is_some() {
                        self.pos += 1;
                        by_helper = true;
                        domain.push(format!("{x} in {}", join(&self.toks[from..self.pos])));
                    } else {
                        let set = match exact {
                            true => Set::point(&self.value()?),
                            false => self.set()?,
                        };
                        let word = if exact { "=" } else { "in" };
                        domain.push(format!("{x} {word} {}", join(&self.toks[from..self.pos])));
                        given.push((x, exact, set));
                    }
                    if !self.eat(",") {
                        break;
                    }
                },
                "holds" | "fails" if sampled.is_some() => {
                    sampled = sampled.map(|s| (s.0, clause == "holds"));
                }
                "sampled" => {
                    let n = self.number()?;
                    if n < 1.0 || n.fract() != 0.0 {
                        return Err(Error::new(at, "`sampled` takes how many inputs were drawn"));
                    }
                    sampled = Some((n as u64, true));
                    says = Some((clause, None, 0.0));
                }
                "holds" | "fails" | "passes" => says = Some((clause, None, 0.0)),
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
                "act" => {
                    let name = self.name()?;
                    if !crate::ast::ACTS.contains(&name.as_str()) {
                        let all = crate::ast::ACTS.join(", ");
                        return Err(Error::new(at, format!("no act `{name}` ({all})")));
                    }
                    // `act sample 1000` is `sampled 1000`.
                    if name == "sample" && self.kind() == Some(Kind::Num) {
                        let n = self.number()?;
                        if n < 1.0 || n.fract() != 0.0 {
                            return Err(Error::new(
                                at,
                                "`sample` takes how many inputs were drawn",
                            ));
                        }
                        sampled = Some((n as u64, sampled.is_none_or(|s| s.1)));
                        says = Some(("sampled".to_string(), None, 0.0));
                    }
                    act = Some(name);
                }
                "cold" => cold = true,
                "source" | "rev" | "seen" | "by" | "written" | "of" => {
                    let Some(Old::Str(s)) = self.next_old() else {
                        self.pos -= 1;
                        return self.err(format!("`{clause}` needs a string"));
                    };
                    let voice = ["by", "written"].contains(&clause.as_str());
                    if voice && crate::ast::voice_parts(&s).is_none() {
                        return Err(Error::new(
                            at,
                            format!(
                                "`{s}` is no voice: `agent:vendor/model/session`, \
                                 `human:name` or `machine:tool`"
                            ),
                        ));
                    }
                    match clause.as_str() {
                        "rev" => rev = Some(s),
                        "seen" => seen = Some(s),
                        "by" => by = Some(s),
                        "written" => written = Some(s),
                        "of" => of = Some(s),
                        _ => source = s,
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
                "the evidence says nothing: `holds`, `fails`, `passes`, `sampled n` or `gives …`",
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
            ("sampled", _, _) => {
                if given.iter().any(|g| g.1) {
                    return at("a sample is drawn from a set: `given a in …`".to_string());
                }
                let (count, ok) = sampled.expect("read with the clause");
                let over = Some(std::mem::take(&mut pattern)).filter(|_| !by_helper);
                Judgment::Sampled {
                    count,
                    ok,
                    domain: domain.join(", "),
                    over,
                }
            }
            _ if by_helper => {
                return at(
                    "a helper names a domain to sample from: write `sampled 1000`".to_string(),
                )
            }
            // A test that states no input: on a claim or a function with
            // inputs it supports and settles nothing; with none, it holds.
            ("passes", _, _) if !given.is_empty() => {
                return at("`passes` states no input: with `given`, write `holds`".to_string())
            }
            ("passes", _, _) if !names.is_empty() => {
                pattern.clear();
                Judgment::Passed
            }
            ("passes", true, _) => Judgment::Correct(true),
            ("passes", false, _) => Judgment::Holds(true),
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
        // What an act can hold is checked against what the block says
        // and what it is on.
        if let Some(a) = act.as_deref() {
            let whole = self
                .names
                .claims
                .get(&target)
                .is_some_and(|c| c.0.is_empty())
                && function.is_none();
            let point = given.iter().all(|g| g.1) && (given.len() == names.len() || whole);
            let kept = ["::captured", "::tied", "::uses"]
                .iter()
                .find(|s| about.ends_with(*s));
            let judged = matches!(judgment, Judgment::Holds(_) | Judgment::Correct(_));
            let denies = matches!(
                judgment,
                Judgment::Holds(false)
                    | Judgment::Correct(false)
                    | Judgment::Sampled { ok: false, .. }
            );
            let why = match (a, &judgment) {
                ("report", _) if !denies => Some("a report is of a failure: it says `fails`"),
                ("report", _) => None,
                ("tie", _) if !matches!(kept, Some(&"::tied") | Some(&"::uses")) => {
                    Some("a tie is on `f::tied` or `f::uses`")
                }
                ("test" | "sample" | "exhaustive" | "measure", _) if kept.is_some() => {
                    Some("it is not about an input: it takes a review, a proof or a tie")
                }
                ("sample", Judgment::Sampled { .. }) => None,
                ("sample", _) => {
                    Some("a sample says how many inputs were drawn: `act sample 1000`")
                }
                (_, Judgment::Sampled { .. }) => Some("inputs that were drawn are `act sample`"),
                ("measure", Judgment::Measured(..)) => None,
                ("measure", _) => Some("a measure gives a rate: `gives rate = 0.9`"),
                (_, Judgment::Measured(..)) => Some("a rate is `act measure`"),
                ("test", Judgment::Output(_) | Judgment::Passed) => None,
                ("test", _) if judged && point => None,
                ("test", _) => Some(
                    "a test is a point: over a set of inputs it is `act exhaustive` or \
                     `act sample n`, and on a whole claim a review or a proof",
                ),
                ("exhaustive", _) if judged && !given.is_empty() => None,
                ("exhaustive", _) => {
                    Some("an exhaustive run states the set it ran through: `given a in …`")
                }
                ("review" | "proof", _) if judged => None,
                ("review" | "proof", _) => Some("a review or a proof says `holds` or `fails`"),
                _ => None,
            };
            if let Some(why) = why {
                return at(format!("`act {a}` cannot say this of `{about}`: {why}"));
            }
        }
        if of.is_some() && !matches!(judgment, Judgment::Holds(_) | Judgment::Correct(_)) {
            return at("a review `of` evidence says `holds` or `fails`".to_string());
        }
        Ok(Evidence {
            about,
            pattern,
            judgment,
            confidence: trust,
            source: Some(source),
            rev,
            seen,
            line,
            act,
            by,
            written,
            cold,
            of,
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
        // No value of a record with unstated parts can be written.
        let built = |e: &Expr| -> Option<String> {
            let mut found = None;
            crate::ast::walk(e, &mut |x| {
                if let Expr::Record(n, _) = x {
                    if m.records.iter().any(|r| &r.name == n && r.open) {
                        found = Some(n.clone());
                    }
                }
            });
            found
        };
        for f in &m.functions {
            if let Some(n) = f.body.as_ref().and_then(&built) {
                return Err(Error::new(f.line, unwritten(&n)));
            }
        }
        for c in &m.claims {
            if let Target::Matches(e) | Target::Property(e) = &c.target {
                if let Some(n) = built(e) {
                    return Err(Error::new(c.line, unwritten(&n)));
                }
            }
        }
        // The code of a subject is claimed correct: `f::correct`, unless
        // the model has its own claim on `correct(f(…))`, or a body, whose
        // spec says what correct is.
        let mut implied = Vec::new();
        for f in m.functions.iter().filter(|f| !f.is_spec()) {
            let in_subject = !self.names.boundaries.contains(&f.name);
            // A point (`correct(f(1, 2))`, what a test pins) narrows nothing.
            let own = |c: &Claim| {
                let inputs = c.pattern.iter().any(|p| p.var.is_some());
                c.func == f.name && c.target == Target::Correct && inputs
            };
            let specified = self.names.specified.contains_key(&f.name);
            if !in_subject || !self.names.opaque.contains(&f.name) || specified {
                continue;
            }
            if m.claims.iter().any(own) {
                continue;
            }
            let inputs: Vec<String> = f.params.iter().map(|p| p.0.clone()).collect();
            let pattern = inputs.iter().map(|x| Position {
                var: Some(x.clone()),
                set: None,
            });
            implied.push(Claim {
                name: format!("{}::correct", f.name),
                func: f.name.clone(),
                pattern: pattern.collect(),
                target: Target::Correct,
                from: Vec::new(),
                auto: false,
                proved: false,
                trust: None,
                line: f.line,
            });
            let name = format!("{}::correct", f.name);
            self.names
                .claims
                .entry(name)
                .or_insert((inputs, Vec::new()));
        }
        for c in implied {
            // Beside the function: before the first claim written after it.
            let at = m.claims.iter().position(|d| d.line > c.line);
            m.claims.insert(at.unwrap_or(m.claims.len()), c);
        }
        // The code of a subject is claimed deterministic, the same input
        // always giving the same output: `f::deterministic`, unless the
        // model has its own claim on `deterministic(f)`.
        let mut stated = self.names.determinism.clone();
        for f in self.names.opaque.clone() {
            let own = stated.iter().any(|d| d.0 == f);
            let line = m.functions.iter().find(|g| g.name == f).map(|g| g.line);
            if let (false, false, Some(line)) = (own, self.names.boundaries.contains(&f), line) {
                stated.push((f.clone(), format!("{f}::deterministic"), true, line));
            }
        }
        for (f, name, same, line) in stated {
            let func = format!("{name}::prose");
            m.functions.push(Function {
                name: func.clone(),
                params: Vec::new(),
                range: Set::bool(),
                body: None,
                written: None,
                line,
            });
            let claim = Claim {
                name: name.clone(),
                func: func.clone(),
                pattern: Vec::new(),
                target: Target::Correct,
                from: Vec::new(),
                auto: false,
                proved: false,
                trust: self.names.trust.get(&name).copied(),
                line,
            };
            let at = m.claims.iter().position(|d| d.line > line);
            m.claims.insert(at.unwrap_or(m.claims.len()), claim);
            self.names
                .claims
                .entry(name.clone())
                .or_insert((Vec::new(), Vec::new()));
            let sig = Sig {
                params: Vec::new(),
                deep: vec![Set::bool()],
            };
            self.names.fns.insert(func, sig);
            m.determinism.push((name, f, same));
        }
        self.requirements(&mut m)?;
        Ok((m, self.names))
    }

    /// Each `ensure` as a claim that rests on the claims of its block, on
    /// what the block requires and, in a subject, on `name::captured`.
    fn requirements(&mut self, m: &mut Model) -> R<()> {
        let ensures = std::mem::take(&mut self.ensures);
        let named = |path: &str| {
            let promise = |e: &&(String, String, Prose, usize, usize, Option<f64>)| {
                !matches!(e.2, Prose::Need(_))
            };
            let ends = |q: &String| q == path || q.ends_with(&format!("::{path}"));
            let mut exact = ensures.iter().filter(promise).map(|e| &e.0);
            exact.find(|q| *q == path).or_else(|| {
                ensures
                    .iter()
                    .filter(promise)
                    .map(|e| &e.0)
                    .find(|q| ends(q))
            })
        };
        // What each one waits for: a `require` for what it names, an
        // `ensure` for every `require` of its block.
        let mut waits: Vec<Vec<String>> = Vec::new();
        for (name, _, kind, block, line, _) in &ensures {
            waits.push(match kind {
                Prose::Need(path) => match named(path) {
                    Some(q) => vec![q.clone()],
                    None => {
                        let msg =
                            format!("`require {name} = {path}`: no `ensure` is named `{path}`");
                        return Err(Error::new(*line, msg));
                    }
                },
                Prose::Wanted => Vec::new(),
                Prose::Promise | Prose::World => {
                    let of = ensures.iter().filter(|e| e.3 == *block);
                    let needs = of.filter(|e| matches!(e.2, Prose::Need(_) | Prose::Wanted));
                    let mut waits: Vec<String> = needs.map(|e| e.0.clone()).collect();
                    // A promise of a contract: the same of each implementer.
                    let fills = self.contracts.fills.iter().filter(|f| &f.0 == name);
                    waits.extend(fills.map(|f| f.1.clone()));
                    waits
                }
            });
        }
        let mut order: Vec<usize> = Vec::new();
        while order.len() < ensures.len() {
            let ready = (0..ensures.len()).find(|i| {
                let done = |n: &String| order.iter().any(|j| &ensures[*j].0 == n);
                !order.contains(i) && waits[*i].iter().all(done)
            });
            let Some(i) = ready else {
                let (name, _, _, _, line, _) =
                    &ensures[(0..ensures.len()).find(|i| !order.contains(i)).unwrap()];
                let msg = format!("`{name}` requires itself, through what it requires");
                return Err(Error::new(*line, msg));
            };
            order.push(i);
        }
        let all: Vec<String> = ensures.iter().map(|e| e.0.clone()).collect();
        for i in order {
            let (name, prose, kind, block, line, trust) = ensures[i].clone();
            if m.claims.iter().any(|c| c.name == name) || m.functions.iter().any(|f| f.name == name)
            {
                return Err(Error::new(line, format!("`{name}` is declared twice")));
            }
            // Its claims: those of the block that say something of results.
            let exists = |n: &String| {
                m.claims
                    .iter()
                    .any(|c| &c.name == n && !c.target.is_structural())
            };
            let mut claims: Vec<String> = Vec::new();
            for (c, b) in &self.owned {
                if *b == block && exists(c) && !claims.contains(c) && !all.contains(c) {
                    claims.push(c.clone());
                }
            }
            let requires = waits[i].clone();
            // A promise of the world rests on the block's assumptions, as
            // one of a subject on its claims; with none, on its own word.
            let world = kind == Prose::World;
            if matches!(kind, Prose::Need(_) | Prose::Wanted) {
                claims.clear();
            }
            let boundary = match kind {
                Prose::Promise => false,
                Prose::World => claims.is_empty(),
                Prose::Need(_) | Prose::Wanted => true,
            };
            let mut state = |m: &mut Model, name: &str, from: Vec<String>, trust: Option<f64>| {
                m.functions.push(Function {
                    name: format!("{name}::prose"),
                    params: Vec::new(),
                    range: Set::bool(),
                    body: None,
                    written: None,
                    line,
                });
                m.claims.push(Claim {
                    name: name.to_string(),
                    func: format!("{name}::prose"),
                    pattern: Vec::new(),
                    target: Target::Correct,
                    from,
                    auto: false,
                    proved: false,
                    trust,
                    line,
                });
                if let Some(t) = trust {
                    self.names.trust.insert(name.to_string(), t);
                }
                self.names
                    .claims
                    .entry(name.to_string())
                    .or_insert((Vec::new(), Vec::new()));
                let sig = Sig {
                    params: Vec::new(),
                    deep: vec![Set::bool()],
                };
                self.names.fns.insert(format!("{name}::prose"), sig);
            };
            let mut from = claims.clone();
            from.extend(requires.clone());
            let filled = self.contracts.fills.iter().any(|f| f.0 == name);
            if matches!(kind, Prose::Need(_)) || (filled && boundary) {
                // It holds as far as the promise it names; that of a
                // contract, as far as its implementers keep it.
                state(m, &name, requires.clone(), trust);
            } else if boundary {
                // A promise of the world: assumed, as far as it is not observed.
                state(m, &name, requires.clone(), trust);
                self.declare(&name, "assumed", trust.unwrap_or(1.0));
            } else {
                let captured = format!("{name}::captured");
                state(m, &captured, Vec::new(), None);
                from.push(captured);
                state(m, &name, from, trust);
            }
            m.ensures.push(Ensure {
                name,
                prose,
                boundary,
                world,
                needs: match kind {
                    Prose::Need(_) => requires.first().cloned(),
                    _ => None,
                },
                claims,
                requires,
                line,
            });
        }
        self.reliance(m, &ensures, &waits);
        Ok(())
    }
}

impl P {
    /// Intent against calls: a `require` on a block nothing here uses, and
    /// a `uses` on a block that makes promises, none of them required
    /// here. Only blocks that state an `ensure` or a `require` are held
    /// to the second.
    fn reliance(
        &self,
        m: &mut Model,
        ensures: &[(String, String, Prose, usize, usize, Option<f64>)],
        waits: &[Vec<String>],
    ) {
        // The block of each function, and of each promise.
        let home = |f: &str| {
            let own = format!("{f}::correct");
            self.owned.iter().find(|o| o.0 == own).map(|o| o.1)
        };
        let made = |name: &str| ensures.iter().find(|e| e.0 == name).map(|e| e.3);
        // (function, what it uses, the block of that), across blocks.
        let mut calls: Vec<(&Claim, &String, usize, usize)> = Vec::new();
        for c in &m.claims {
            let Target::Uses { used, .. } = &c.target else {
                continue;
            };
            let Some(from) = home(&c.func) else { continue };
            for u in used {
                match home(u) {
                    Some(to) if to != from => calls.push((c, u, from, to)),
                    _ => {}
                }
            }
        }
        // What a block leans on, beyond `uses`: the calls of its bodies
        // and of its claims, then all of it through other blocks.
        let mut leans: Vec<(usize, usize)> = calls.iter().map(|c| (c.2, c.3)).collect();
        let code = |g: &str| {
            let g = g.strip_suffix("::spec").or(g.strip_suffix("::body"));
            g.map(str::to_string)
        };
        let mut reach = |from: Option<usize>, e: &Expr| {
            for g in crate::ast::calls(e) {
                let to = home(&code(&g).unwrap_or(g));
                if let (Some(from), Some(to)) = (from, to.filter(|to| Some(*to) != from)) {
                    leans.push((from, to));
                }
            }
        };
        for f in &m.functions {
            if let Some(body) = &f.body {
                let name = code(&f.name).unwrap_or(f.name.clone());
                reach(home(&name), body);
            }
        }
        for c in &m.claims {
            let from = self.owned.iter().find(|o| o.0 == c.name).map(|o| o.1);
            if let Target::Property(e) | Target::Matches(e) = &c.target {
                reach(from, e);
            }
        }
        loop {
            let more: Vec<(usize, usize)> = leans
                .iter()
                .flat_map(|a| leans.iter().filter(|b| b.0 == a.1).map(|b| (a.0, b.1)))
                .filter(|e| e.0 != e.1 && !leans.contains(e))
                .collect();
            if more.is_empty() {
                break;
            }
            leans.extend(more);
            leans.sort();
            leans.dedup();
        }
        // A type of the other block, in a signature or a field here.
        let typed = |block: usize, here: Option<&str>, there: &str| {
            let there = format!("{there}::");
            let names = |s: &Set| s.to_string().contains(&there);
            let sig = |f: &Function| f.params.iter().any(|p| names(&p.1)) || names(&f.range);
            let own = |f: &&Function| home(&code(&f.name).unwrap_or(f.name.clone())) == Some(block);
            let mine = |r: &&Record| here.is_some_and(|h| r.name.starts_with(&format!("{h}::")));
            m.functions.iter().filter(own).any(sig)
                || (m.records.iter().filter(mine)).any(|r| r.fields.iter().any(|f| names(&f.1)))
        };
        let scope = |name: &str| name.rsplit_once("::").map(|n| n.0.to_string());
        let mut lints = Vec::new();
        for (i, (name, _, kind, block, line, _)) in ensures.iter().enumerate() {
            let Prose::Need(_) = kind else { continue };
            let promise = &waits[i][0];
            let to = made(promise).expect("named");
            let shares = scope(promise).is_some_and(|t| typed(*block, scope(name).as_deref(), &t));
            // `x: impl C` in a signature beside it leans on the contract.
            let pair = scope(name).zip(scope(promise));
            let leans_on = |p: (String, String)| {
                let mut all = self.contracts.leans.iter().chain(&self.leaning);
                all.any(|l| l.0 == p.0 && l.1 == p.1)
            };
            let takes = pair.is_some_and(leans_on);
            if !leans.contains(&(*block, to)) && !shares && !takes {
                lints.push(Lint {
                    code: "M105",
                    line: *line,
                    message: format!(
                        "`{name}` needs `{promise}`, and nothing declared beside it leans on \
                         what is declared beside `{promise}`: no `uses`, no call, no type."
                    ),
                    help: "say which function `uses` it, or remove the `require`".to_string(),
                });
            }
        }
        let mut seen: Vec<(&str, usize)> = Vec::new();
        for (c, used, from, to) in &calls {
            let states = ensures.iter().any(|e| e.3 == *from);
            let promises: Vec<&String> = ensures
                .iter()
                .filter(|e| e.3 == *to && matches!(e.2, Prose::Promise | Prose::World))
                .map(|e| &e.0)
                .collect();
            let required = ensures.iter().enumerate().any(|(i, e)| {
                e.3 == *from && matches!(e.2, Prose::Need(_)) && made(&waits[i][0]) == Some(*to)
            });
            if !states || promises.is_empty() || required || seen.contains(&(&c.func, *to)) {
                continue;
            }
            seen.push((&c.func, *to));
            let all: Vec<String> = promises.iter().map(|p| format!("`{p}`")).collect();
            lints.push(Lint {
                code: "M106",
                line: c.line,
                message: format!(
                    "`{}` uses `{used}`, and no `require` beside it names a promise made \
                     beside `{used}` ({}).",
                    c.func,
                    all.join(", ")
                ),
                help: format!(
                    "write `require <name> = {};` beside `{}`",
                    promises[0], c.func
                ),
            });
        }
        // `x: impl C` in a block that states its intent, and no `require`
        // beside it names a promise of the contract.
        let mut told: Vec<(&String, &String)> = Vec::new();
        for (block, contract, line) in self.contracts.leans.iter().chain(&self.leaning) {
            let inside = |e: &&(String, String, Prose, usize, usize, Option<f64>)| {
                scope(&e.0).as_ref() == Some(block)
            };
            let promises: Vec<&String> = ensures
                .iter()
                .filter(|e| scope(&e.0).as_ref() == Some(contract))
                .filter(|e| matches!(e.2, Prose::Promise | Prose::World))
                .map(|e| &e.0)
                .collect();
            let required = ensures.iter().enumerate().any(|(i, e)| {
                inside(&e) && matches!(e.2, Prose::Need(_)) && promises.contains(&&waits[i][0])
            });
            let states = ensures.iter().any(|e| inside(&e));
            // Already said, of a function that `uses` the contract.
            let to = promises.first().and_then(|p| made(p));
            let own = format!("{block}::");
            let said = (seen.iter()).any(|s| Some(s.1) == to && s.0.starts_with(&own));
            if !states || promises.is_empty() || required || said {
                continue;
            }
            if told.contains(&(block, contract)) {
                continue;
            }
            told.push((block, contract));
            let all: Vec<String> = promises.iter().map(|p| format!("`{p}`")).collect();
            lints.push(Lint {
                code: "M106",
                line: *line,
                message: format!(
                    "`{block}` takes a value bound by `{contract}`, and no `require` in it \
                     names a promise of `{contract}` ({}).",
                    all.join(", ")
                ),
                help: format!("write `require <name> = {};` in `{block}`", promises[0]),
            });
        }
        m.lints = lints;
    }
}

/// What a line of prose is: a promise of a subject, a promise of the
/// world (assumed), or the need of a promise made elsewhere.
#[derive(Clone, PartialEq)]
enum Prose {
    Promise,
    World,
    Need(String),
    /// `require name = "prose";`: needed, with nobody to promise it yet.
    Wanted,
}

/// The model, and its names for the evidence.
// mitra: implements reader::model
pub fn model(src: &str) -> Result<(Model, Names), Error> {
    model_with(src, &Handed::default())
}

/// The model, importing from the modules handed with it.
pub fn model_with(src: &str, handed: &Handed) -> Result<(Model, Names), Error> {
    let (mut model, names) = read(lex(src)?, Rc::new(handed.clone()))?.finish()?;
    model.settings = handed.settings.clone();
    Ok((model, names))
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
            ("subject" | "boundary", Kind::Name, Some(n)) if depth == 0 && n.text != "fn" => {
                scope = Some(n.text.clone())
            }
            ("fn" | "claim" | "assume", Kind::Name, Some(n)) if depth <= 1 => {
                let full = match &scope {
                    Some(s) => format!("{s}::{}", n.text),
                    None => n.text.clone(),
                };
                if t.text == "fn" {
                    all.insert(format!("{full}::deterministic"));
                }
                all.insert(full);
            }
            _ => {}
        }
    }
    all
}

/// One file, read twice: the second reading knows every function of it.
/// The records, enums and variants declared inside `subject name { … }`
/// or `boundary name { … }`, as `name::Record`, and those declared
/// outside any block.
fn types_of(toks: &[Tok]) -> (BTreeSet<String>, BTreeSet<String>) {
    let (mut scoped, mut top) = (BTreeSet::new(), BTreeSet::new());
    let (mut depth, mut scope): (usize, Option<String>) = (0, None);
    let mut i = 0;
    while i < toks.len() {
        let t = &toks[i];
        let next = toks.get(i + 1).filter(|n| n.kind == Kind::Name);
        let mut add = |name: &str, scope: &Option<String>| match scope {
            Some(s) => scoped.insert(format!("{s}::{name}")),
            None => top.insert(name.to_string()),
        };
        match (t.text.as_str(), &t.kind, next) {
            ("{", Kind::Punct, _) => depth += 1,
            ("}", Kind::Punct, _) => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    scope = None;
                }
            }
            ("subject" | "boundary", Kind::Name, Some(n)) if depth == 0 && n.text != "fn" => {
                scope = Some(n.text.clone())
            }
            ("record", Kind::Name, Some(n)) if depth <= 1 => {
                add(&n.text, &scope);
            }
            ("enum", Kind::Name, Some(n)) if depth <= 1 => {
                add(&n.text, &scope);
                // Its variants: each name at the first level of its braces.
                let mut j = i + 2;
                while j < toks.len() && toks[j].text != "{" && toks[j].text != ";" {
                    j += 1;
                }
                let (mut inner, mut first) = (0usize, true);
                while j < toks.len() {
                    match toks[j].text.as_str() {
                        "{" | "(" | "<" => inner += 1,
                        "}" | ")" | ">" => inner = inner.saturating_sub(1),
                        "," if inner == 1 => first = true,
                        _ if inner == 1 && first && toks[j].kind == Kind::Name => {
                            add(&toks[j].text, &scope);
                            first = false;
                        }
                        _ => {}
                    }
                    if inner == 0 {
                        break;
                    }
                    j += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    (scoped, top)
}

/// `a :: X` as one name, where `a::X` is a type or a variant of a block.
fn merge_types(toks: Vec<Tok>, scoped: &BTreeSet<String>) -> Vec<Tok> {
    let mut out: Vec<Tok> = Vec::new();
    let mut i = 0;
    while i < toks.len() {
        let joined = match (toks.get(i + 1), toks.get(i + 2)) {
            (Some(c), Some(x)) if toks[i].kind == Kind::Name && c.text == "::" => {
                Some(format!("{}::{}", toks[i].text, x.text)).filter(|f| scoped.contains(f))
            }
            _ => None,
        };
        match joined {
            Some(text) => {
                out.push(Tok {
                    kind: Kind::Name,
                    text,
                    line: toks[i].line,
                });
                i += 3;
            }
            None => {
                out.push(toks[i].clone());
                i += 1;
            }
        }
    }
    out
}

/// Give each type and variant of a block its full name: inside the
/// block by its short name, elsewhere by its path, or by its short name
/// where one block has it and nothing outside any block does.
fn scope_types(
    toks: Vec<Tok>,
    scoped: &BTreeSet<String>,
    top: &BTreeSet<String>,
    evidence: bool,
) -> Vec<Tok> {
    if scoped.is_empty() {
        return toks;
    }
    let mut toks = merge_types(toks, scoped);
    let (mut depth, mut scope): (usize, Option<String>) = (0, None);
    for i in 0..toks.len() {
        let text = toks[i].text.clone();
        match text.as_str() {
            "{" if toks[i].kind == Kind::Punct => depth += 1,
            "}" if toks[i].kind == Kind::Punct => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    scope = None;
                }
            }
            "subject" | "boundary" if depth == 0 && toks[i].kind == Kind::Name => {
                let n = toks
                    .get(i + 1)
                    .filter(|n| n.kind == Kind::Name && n.text != "fn");
                scope = n.map(|n| n.text.clone());
            }
            // Evidence `for block::f`: short names are those of the block.
            "for" if evidence && toks.get(i + 2).is_some_and(|c| c.text == "::") => {
                scope = toks.get(i + 1).map(|n| n.text.clone());
            }
            _ => {}
        }
        if toks[i].kind != Kind::Name || text.contains("::") {
            continue;
        }
        let before = i.checked_sub(1).map(|j| toks[j].text.as_str());
        let after = toks.get(i + 1).map(|t| t.text.as_str());
        // A field read, the rest of a path, the head of one, a label.
        if matches!(before, Some("." | "::")) || matches!(after, Some("::" | ":")) {
            continue;
        }
        let own = scope.as_ref().map(|s| format!("{s}::{text}"));
        if let Some(full) = own.filter(|f| scoped.contains(f)) {
            toks[i].text = full;
            continue;
        }
        if top.contains(&text) {
            continue;
        }
        let ends = format!("::{text}");
        let mut with = scoped.iter().filter(|f| f.ends_with(&ends));
        if let (Some(full), None) = (with.next(), with.next()) {
            toks[i].text = full.clone();
        }
    }
    toks
}

/// The `>` that closes the `<` at `open`.
fn angle(toks: &[Tok], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (j, t) in toks.iter().enumerate().skip(open) {
        match (t.kind == Kind::Punct, t.text.as_str()) {
            (true, "<") => depth += 1,
            (true, ">") => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            (true, "{" | "}" | ";") => return None,
            _ => {}
        }
    }
    None
}

/// The names of the type parameters written between `<` and `>`:
/// `T, U: Link` gives `T` and `U`.
fn param_names(params: &[Tok]) -> Vec<String> {
    let (mut names, mut depth, mut first) = (Vec::new(), 0i32, true);
    for t in params {
        let punct = t.kind == Kind::Punct;
        match t.text.as_str() {
            "<" | "(" if punct => depth += 1,
            ">" | ")" if punct => depth -= 1,
            "," if punct && depth == 0 => {
                first = true;
                continue;
            }
            _ if first && t.kind == Kind::Name => names.push(t.text.clone()),
            _ => {}
        }
        first = false;
    }
    names
}

/// The types written between `<` and `>`, one list of tokens each.
fn type_args(args: &[Tok]) -> Vec<Vec<Tok>> {
    let (mut all, mut depth) = (vec![Vec::new()], 0i32);
    for t in args {
        let punct = t.kind == Kind::Punct;
        match t.text.as_str() {
            "<" | "(" if punct => depth += 1,
            ">" | ")" if punct => depth -= 1,
            "," if punct && depth == 0 => {
                all.push(Vec::new());
                continue;
            }
            _ => {}
        }
        all.last_mut().expect("one").push(t.clone());
    }
    all.retain(|a| !a.is_empty());
    all
}

/// A `fn` or a `claim`, from its keyword and name, given type
/// parameters: `fn f(…)` is `fn f<T>(…)`, `fn f<U>(…)` is `fn f<T, U>(…)`.
fn with_params(item: &[Tok], params: &[Tok]) -> Vec<Tok> {
    if params.is_empty() || item.len() < 2 {
        return item.to_vec();
    }
    let at = |text: &str| Tok {
        kind: Kind::Punct,
        text: text.into(),
        line: item[1].line,
    };
    let mut out = item[..2].to_vec();
    out.push(at("<"));
    out.extend(params.iter().cloned());
    match item.get(2).is_some_and(|t| t.text == "<") {
        true => {
            out.push(at(","));
            out.extend(item[3..].iter().cloned());
        }
        false => {
            out.push(at(">"));
            out.extend(item[2..].iter().cloned());
        }
    }
    out
}

/// `impl R { … }` as Rust writes it, read as what it stands for: a `fn`
/// or a `claim` named `R::name`, `self` (also `&self`, `&mut self`) an
/// input `self: R`, and `Self` the record. `impl<T> R<T> { … }` gives
/// each function the parameter, and `self` is an `R<T>`. Also gives the
/// methods, those that take `self`, by their full name.
fn impls(toks: Vec<Tok>) -> Result<(Vec<Tok>, BTreeSet<String>), Error> {
    // Inside an `impl`: its record, the depth of its braces, its type
    // parameters as written and by name, and the record's own `<…>`.
    struct Of {
        record: String,
        depth: usize,
        params: Vec<Tok>,
        names: Vec<String>,
        args: Vec<Tok>,
    }
    let (mut out, mut methods) = (Vec::new(), BTreeSet::new());
    let (mut depth, mut scope): (usize, Option<String>) = (0, None);
    let mut of: Option<Of> = None;
    // In a claim of an `impl<T>`, up to this token: `T` is any value.
    let mut plain = 0;
    let mut i = 0;
    let name = |t: Option<&Tok>| t.filter(|t| t.kind == Kind::Name).map(|t| t.text.clone());
    let any = |t: &Tok| Tok {
        kind: Kind::Name,
        text: "Any".into(),
        line: t.line,
    };
    while i < toks.len() {
        let t = &toks[i];
        let word = t.kind == Kind::Name;
        let punct = t.kind == Kind::Punct;
        match (t.text.as_str(), &of) {
            ("subject" | "boundary", None) if word && depth == 0 => {
                scope = name(toks.get(i + 1)).filter(|n| n != "fn");
            }
            ("impl", None) if word && depth <= 1 => {
                let form = "an `impl` is `impl R { … }`, `impl<T> R<T> { … }` or \
                            `impl Contract for R { … }`";
                let is = |j: usize, w: &str| toks.get(j).is_some_and(|t| t.text == w);
                let mut j = i + 1;
                let mut params = Vec::new();
                if is(j, "<") {
                    let end = angle(&toks, j).ok_or(Error::new(t.line, form))?;
                    params = toks[j + 1..end].to_vec();
                    j = end + 1;
                }
                let Some(record) = name(toks.get(j)) else {
                    return Err(Error::new(t.line, "expected a record after `impl`"));
                };
                j += 1;
                let mut args = Vec::new();
                if is(j, "<") {
                    let end = angle(&toks, j).ok_or(Error::new(t.line, form))?;
                    args = toks[j..=end].to_vec();
                    j = end + 1;
                }
                if !is(j, "{") {
                    return Err(Error::new(t.line, form));
                }
                let names = param_names(&params);
                of = Some(Of {
                    record,
                    depth,
                    params,
                    names,
                    args,
                });
                depth += 1;
                i = j + 1;
                continue;
            }
            ("impl", Some(_)) if word => {
                return Err(Error::new(t.line, "an `impl` inside another"));
            }
            ("{", _) if punct => depth += 1,
            ("}", _) if punct => {
                depth = depth.saturating_sub(1);
                if of.as_ref().is_some_and(|o| o.depth == depth) {
                    of = None;
                    i += 1;
                    continue;
                }
                if depth == 0 {
                    scope = None;
                }
            }
            ("Self", Some(o)) if word => {
                out.push(Tok {
                    text: o.record.clone(),
                    ..t.clone()
                });
                for a in &o.args {
                    let open = i < plain && a.kind == Kind::Name && o.names.contains(&a.text);
                    out.push(if open { any(a) } else { a.clone() });
                }
                i += 1;
                continue;
            }
            (p, Some(o)) if word && i < plain && o.names.iter().any(|n| n == p) => {
                out.push(any(t));
                i += 1;
                continue;
            }
            ("fn" | "claim" | "assume" | "ensure" | "require", Some(o))
                if word && depth == o.depth + 1 =>
            {
                let Some(n) = name(toks.get(i + 1)) else {
                    out.push(t.clone());
                    i += 1;
                    continue;
                };
                let (r, func) = (&o.record, t.text == "fn");
                if !func && !o.names.is_empty() {
                    // To its `;`, or to the `}` of its braces.
                    let mut end = i;
                    while toks
                        .get(end)
                        .is_some_and(|e| e.text != ";" && e.text != "{")
                    {
                        end += 1;
                    }
                    if toks.get(end).is_some_and(|e| e.text == "{") {
                        end = closing(&toks, end).unwrap_or(end);
                    }
                    plain = end;
                }
                let full = format!("{r}::{n}");
                out.push(t.clone());
                out.push(Tok {
                    text: full.clone(),
                    ..toks[i + 1].clone()
                });
                i += 2;
                // Its own `<U>` after the parameters of the `impl`.
                if func && !o.params.is_empty() {
                    let at = |text: &str| Tok {
                        kind: Kind::Punct,
                        text: text.into(),
                        line: t.line,
                    };
                    out.push(at("<"));
                    out.extend(o.params.iter().cloned());
                    let own = toks
                        .get(i)
                        .filter(|t| t.text == "<")
                        .map(|_| angle(&toks, i));
                    match own {
                        Some(Some(end)) => {
                            out.push(at(","));
                            out.extend(toks[i + 1..=end].iter().cloned());
                            i = end + 1;
                        }
                        _ => out.push(at(">")),
                    }
                } else if func && toks.get(i).is_some_and(|t| t.text == "<") {
                    if let Some(end) = angle(&toks, i) {
                        out.extend(toks[i..=end].iter().cloned());
                        i = end + 1;
                    }
                }
                // `(self`, `(&self`, `(&mut self`: an input `self: R`.
                let mut j = i + 1;
                if toks.get(i).is_some_and(|o| o.text == "(") {
                    j += usize::from(toks.get(j).is_some_and(|a| a.text == "&"));
                    j += usize::from(toks.get(j).is_some_and(|m| m.text == "mut"));
                    let typed = toks.get(j + 1).is_some_and(|c| c.text == ":");
                    if toks.get(j).is_some_and(|s| s.text == "self") && !typed {
                        let at = toks[j].clone();
                        out.push(toks[i].clone());
                        out.push(at.clone());
                        out.push(Tok {
                            kind: Kind::Punct,
                            text: ":".into(),
                            line: at.line,
                        });
                        out.push(Tok {
                            text: r.clone(),
                            ..at
                        });
                        for a in &o.args {
                            let open = !func && a.kind == Kind::Name && o.names.contains(&a.text);
                            out.push(if open { any(a) } else { a.clone() });
                        }
                        i = j + 1;
                        if func {
                            methods.insert(match &scope {
                                Some(s) => format!("{s}::{full}"),
                                None => full,
                            });
                        }
                    }
                }
                continue;
            }
            _ => {}
        }
        out.push(t.clone());
        i += 1;
    }
    Ok((out, methods))
}

/// What the contracts of a file come to, once read as blocks.
#[derive(Debug, Clone, Default)]
struct Contracts {
    names: BTreeSet<String>,
    /// A promise of a contract, and the same promise of one implementer.
    fills: Vec<(String, String)>,
    /// A block, a contract a signature of it names, and the line.
    leans: Vec<(String, String, usize)>,
    /// The functions of a contract that take `self`.
    methods: BTreeSet<String>,
    /// The contracts brought from another store, each with its store.
    brought: BTreeMap<String, String>,
    /// `x: impl C`: the line of the name of its function, that name, the
    /// input and the contracts.
    sugar: Vec<(usize, String, String, Vec<String>)>,
}

/// The tokens from `open`, a `{`, to the `}` that closes it: the index
/// of that `}`.
fn closing(toks: &[Tok], open: usize) -> Option<usize> {
    let mut depth = 0usize;
    for (j, t) in toks.iter().enumerate().skip(open) {
        match (t.kind == Kind::Punct, t.text.as_str()) {
            (true, "{") => depth += 1,
            (true, "}") => {
                depth -= 1;
                if depth == 0 {
                    return Some(j);
                }
            }
            _ => {}
        }
    }
    None
}

/// `x: impl C`, also `impl C<A>` and `impl C + D`, as the type of an
/// input: any value. What it names is kept beside, with the block it is
/// written in.
fn impl_types(toks: Vec<Tok>, c: &mut Contracts) -> Result<Vec<Tok>, Error> {
    let word = |t: &Tok, w: &str| t.kind == Kind::Name && t.text == w;
    let mut out: Vec<Tok> = Vec::new();
    let (mut depth, mut scope): (usize, Option<String>) = (0, None);
    // The last `fn` read: its name and the line of it.
    let mut func: Option<(String, usize)> = None;
    let mut i = 0;
    while i < toks.len() {
        let t = &toks[i];
        let next = toks.get(i + 1).filter(|n| n.kind == Kind::Name);
        if t.kind == Kind::Punct && t.text == "{" {
            depth += 1;
        } else if t.kind == Kind::Punct && t.text == "}" {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                scope = None;
            }
        } else if depth == 0
            && ["subject", "boundary", "contract"]
                .iter()
                .any(|w| word(t, w))
        {
            scope = next.map(|n| n.text.clone()).filter(|n| n != "fn");
        } else if let (true, Some(n)) = (["fn", "claim", "assume"].iter().any(|w| word(t, w)), next)
        {
            func = Some((n.text.clone(), n.line));
        } else if let (true, Some(n)) =
            (word(t, "impl"), next.filter(|n| c.names.contains(&n.text)))
        {
            // After `:` (and `&`, `&mut`) it is a type; before `for` or
            // `{` it is an implementer.
            let mut b = i;
            while b > 0 && ["&", "mut"].contains(&toks[b - 1].text.as_str()) {
                b -= 1;
            }
            let typed = b >= 2 && toks[b - 1].text == ":" && toks[b - 2].kind == Kind::Name;
            if !typed {
                if toks.get(i + 2).is_some_and(|b| b.text == "{") {
                    let msg = format!(
                        "`{}` is a contract: a type fills it with `impl {0} for Type {{ … }}`",
                        n.text
                    );
                    return Err(Error::new(t.line, msg));
                }
                out.push(t.clone());
                i += 1;
                continue;
            }
            let (mut all, mut j) = (Vec::new(), i + 1);
            loop {
                let Some(name) = toks.get(j).filter(|n| n.kind == Kind::Name) else {
                    return Err(Error::new(t.line, "expected a contract after `+`"));
                };
                if !c.names.contains(&name.text) {
                    let msg = format!(
                        "`{}` is not a contract: a bound names a contract",
                        name.text
                    );
                    return Err(Error::new(name.line, msg));
                }
                all.push(name.text.clone());
                j += 1;
                if toks.get(j).is_some_and(|a| a.text == "<") {
                    let end = angle(&toks, j);
                    j = end.ok_or(Error::new(t.line, "this `<` is not closed"))? + 1;
                }
                if !toks.get(j).is_some_and(|p| p.text == "+") {
                    break;
                }
                j += 1;
            }
            if let Some(block) = &scope {
                for name in &all {
                    c.leans.push((block.clone(), name.clone(), t.line));
                }
            }
            if let Some((f, line)) = &func {
                c.sugar
                    .push((*line, f.clone(), toks[b - 2].text.clone(), all));
            }
            out.push(Tok {
                kind: Kind::Name,
                text: "Any".into(),
                line: t.line,
            });
            i = j;
            continue;
        }
        out.push(t.clone());
        i += 1;
    }
    Ok(out)
}

/// `item` on `line`, each type parameter of `wanted` written as the
/// type of `args` given for it.
fn given_types(
    item: &[Tok],
    wanted: &[String],
    args: &[Vec<Tok>],
    line: usize,
    into: &mut Vec<Tok>,
) {
    for f in item {
        let n = wanted
            .iter()
            .position(|w| f.kind == Kind::Name && w == &f.text);
        let each: &[Tok] = match n {
            Some(n) => &args[n],
            None => std::slice::from_ref(f),
        };
        into.extend(each.iter().map(|f| Tok { line, ..f.clone() }));
    }
}

/// `contract C { … }`, `impl C for T { … }` and `x: impl C`, read as what
/// they stand for. A contract is a block of its own: its functions take
/// any value as `self`. An implementer is given each function and each
/// promise of the contract, in a group of its own after its block, so
/// its promises rest on what it says and on nothing else. `impl C` as a
/// type is any value. `contract C<K> { … }` gives each function the
/// parameter; `impl C<Int> for T` gives `T` the functions with `Int`.
fn contracts(toks: Vec<Tok>) -> Result<(Vec<Tok>, Contracts), Error> {
    let word = |t: &Tok, w: &str| t.kind == Kind::Name && t.text == w;
    let punct = |t: &Tok, w: &str| t.kind == Kind::Punct && t.text == w;
    let mut c = Contracts::default();
    for (i, t) in toks.iter().enumerate() {
        let named = toks.get(i + 1).filter(|n| n.kind == Kind::Name);
        let opens = toks
            .get(i + 2)
            .is_some_and(|b| b.text == "{" || b.text == "<");
        if let (true, Some(n), true) = (word(t, "contract"), named, opens) {
            c.names.insert(n.text.clone());
        }
    }
    let toks = impl_types(toks, &mut c)?;
    let is = |i: usize, w: &str| {
        toks.get(i)
            .is_some_and(|t| t.text == w && t.kind != Kind::Str)
    };
    let named = |i: usize| toks.get(i).filter(|t| t.kind == Kind::Name);
    // What each contract holds: its type parameters, its signatures, its
    // promises, its claims.
    struct Held {
        params: Vec<Tok>,
        fns: Vec<Vec<Tok>>,
        ensures: Vec<(String, Vec<Tok>)>,
        claims: Vec<Vec<Tok>>,
    }
    let mut held: BTreeMap<String, Held> = BTreeMap::new();
    let mut spans: Vec<(usize, usize, String)> = Vec::new();
    let (mut depth, mut i) = (0usize, 0);
    while i < toks.len() {
        let t = &toks[i];
        if punct(t, "{") {
            depth += 1;
        } else if punct(t, "}") {
            depth = depth.saturating_sub(1);
        } else if word(t, "contract") && depth == 0 && named(i + 1).is_some() {
            let name = toks[i + 1].text.clone();
            let mut open = i + 2;
            let mut params = Vec::new();
            if is(open, "<") {
                let msg = format!("the `<` of `contract {name}` is not closed");
                let end = angle(&toks, open).ok_or(Error::new(t.line, msg))?;
                params = toks[open + 1..end].to_vec();
                open = end + 1;
            }
            if !is(open, "{") {
                i += 1;
                continue;
            }
            let Some(end) = closing(&toks, open) else {
                return Err(Error::new(
                    t.line,
                    format!("`contract {name}` is not closed"),
                ));
            };
            if held.contains_key(&name) {
                return Err(Error::new(
                    t.line,
                    format!("`contract {name}` is declared twice"),
                ));
            }
            let mut h = Held {
                params,
                fns: Vec::new(),
                ensures: Vec::new(),
                claims: Vec::new(),
            };
            let mut j = open + 1;
            while j < end {
                let start = j;
                let kw = toks[j].text.clone();
                let only = "a contract holds `fn` signatures, `ensure` lines and claims";
                match kw.as_str() {
                    "fn" | "ensure" if toks[j].kind == Kind::Name => {
                        while j < end && !punct(&toks[j], ";") {
                            if punct(&toks[j], "{") {
                                let msg = match kw.as_str() {
                                    "fn" => "a function of a contract has no body: \
                                             no default method"
                                        .to_string(),
                                    _ => only.to_string(),
                                };
                                return Err(Error::new(toks[j].line, msg));
                            }
                            j += 1;
                        }
                        j += 1;
                        let item = toks[start..j.min(end)].to_vec();
                        match kw.as_str() {
                            "fn" => h.fns.push(item),
                            _ => {
                                let n = named(start + 1).map(|n| n.text.clone());
                                h.ensures.push((n.unwrap_or_default(), item));
                            }
                        }
                    }
                    "claim" if toks[j].kind == Kind::Name => {
                        // To its `;`, or to the `}` of its braces.
                        while j < end && !punct(&toks[j], ";") && !punct(&toks[j], "{") {
                            j += 1;
                        }
                        if j < end && punct(&toks[j], "{") {
                            j = closing(&toks[..end], j).unwrap_or(end - 1);
                        }
                        j += 1;
                        h.claims.push(toks[start..j.min(end)].to_vec());
                    }
                    _ => return Err(Error::new(toks[j].line, format!("`{kw}`: {only}"))),
                }
            }
            spans.push((i, end, name.clone()));
            held.insert(name, h);
            i = end + 1;
            continue;
        }
        i += 1;
    }
    if held.is_empty() && !toks.iter().any(|t| word(t, "for")) {
        return Ok((toks, c));
    }
    let any = |at: &Tok| Tok {
        kind: Kind::Name,
        text: "Any".into(),
        line: at.line,
    };
    let mut out: Vec<Tok> = Vec::new();
    // The groups to write after the block being read.
    let mut after: Vec<Tok> = Vec::new();
    let mut filled: Vec<(String, String)> = Vec::new();
    let (mut depth, mut scope): (usize, Option<(String, String)>) = (0, None);
    let mut i = 0;
    while i < toks.len() {
        let t = &toks[i];
        if let Some((_, end, name)) = spans.iter().find(|s| s.0 == i) {
            // The contract: a block of the world, `self` any value.
            let h = &held[name];
            out.push(Tok {
                text: "boundary".into(),
                ..t.clone()
            });
            out.push(toks[i + 1].clone());
            out.push(Tok {
                kind: Kind::Punct,
                text: "{".into(),
                line: t.line,
            });
            for f in &h.fns {
                let f = &with_params(f, &h.params);
                let open = f.iter().position(|a| punct(a, "(")).unwrap_or(f.len());
                let mut k = 0;
                while k < f.len() {
                    // `(self`, `(&self`, `(&mut self`: an input `self: Any`.
                    let mut s = k + 1;
                    s += usize::from(f.get(s).is_some_and(|a| a.text == "&"));
                    s += usize::from(f.get(s).is_some_and(|a| a.text == "mut"));
                    let own = k == open
                        && f.get(s).is_some_and(|a| word(a, "self"))
                        && f.get(s + 1).is_none_or(|a| a.text != ":");
                    if own {
                        out.push(f[k].clone());
                        out.push(f[s].clone());
                        out.push(Tok {
                            kind: Kind::Punct,
                            text: ":".into(),
                            line: f[s].line,
                        });
                        out.push(any(&f[s]));
                        c.methods.insert(format!("{name}::{}", f[1].text));
                        k = s + 1;
                        continue;
                    }
                    out.push(match word(&f[k], "Self") {
                        true => any(&f[k]),
                        false => f[k].clone(),
                    });
                    k += 1;
                }
            }
            for (_, e) in &h.ensures {
                out.extend(e.iter().cloned());
            }
            out.push(toks[*end].clone());
            i = end + 1;
            continue;
        }
        // impl C for T { … }, impl<P> C<A> for T<P> { … }
        let brace = (i..toks.len()).find(|&j| ["{", "}", ";"].contains(&toks[j].text.as_str()));
        let fills = word(t, "impl")
            && brace.is_some_and(|b| punct(&toks[b], "{") && (i..b).any(|j| word(&toks[j], "for")));
        if punct(t, "{") {
            depth += 1;
        } else if punct(t, "}") {
            depth = depth.saturating_sub(1);
            if depth == 0 {
                scope = None;
                out.push(t.clone());
                out.append(&mut after);
                i += 1;
                continue;
            }
        } else if (word(t, "subject") || word(t, "boundary")) && depth == 0 && is(i + 2, "{") {
            scope = Some((t.text.clone(), toks[i + 1].text.clone()));
        } else if fills {
            let form = "expected `impl Contract for Type { … }`";
            let open = brace.expect("found");
            let mut j = i + 1;
            let mut params: &[Tok] = &[];
            if is(j, "<") {
                let end = angle(&toks, j).ok_or(Error::new(t.line, form))?;
                params = &toks[j + 1..end];
                j = end + 1;
            }
            let of = named(j).ok_or(Error::new(t.line, form))?;
            j += 1;
            let mut args: Vec<Vec<Tok>> = Vec::new();
            if is(j, "<") {
                let end = angle(&toks, j).ok_or(Error::new(t.line, form))?;
                args = type_args(&toks[j + 1..end]);
                j = end + 1;
            }
            if !is(j, "for") {
                return Err(Error::new(t.line, form));
            }
            let by = named(j + 1).ok_or(Error::new(t.line, form))?;
            let target = &toks[j + 2..open];
            let Some(h) = held.get(&of.text) else {
                let msg = format!("`{}` is not a contract: no `contract {0} {{ … }}`", of.text);
                return Err(Error::new(t.line, msg));
            };
            let Some((side, block)) = scope.clone().filter(|_| depth == 1) else {
                let msg = format!(
                    "`impl {} for {}` is written in a `subject {{ … }}` or a `boundary {{ … }}`",
                    of.text, by.text
                );
                return Err(Error::new(t.line, msg));
            };
            let wanted = param_names(&h.params);
            if wanted.len() != args.len() {
                let msg = format!(
                    "`{}` takes {} type(s), `impl {0}<…> for {}` gives {}",
                    of.text,
                    wanted.len(),
                    by.text,
                    args.len()
                );
                return Err(Error::new(t.line, msg));
            }
            let end =
                closing(&toks, open).ok_or(Error::new(t.line, "this `impl` is not closed"))?;
            // Its braces: what decomposes the promises for this
            // implementer, what it requires, and a function of the
            // contract written again to say what it `uses` or computes.
            let mut again: Vec<(String, Vec<Tok>)> = Vec::new();
            let mut rest: Vec<Tok> = Vec::new();
            let mut k = open + 1;
            while k < end {
                let b = &toks[k];
                let mut stop = k;
                while stop < end && !punct(&toks[stop], ";") && !punct(&toks[stop], "{") {
                    stop += 1;
                }
                if stop < end && punct(&toks[stop], "{") {
                    stop = closing(&toks[..end], stop).unwrap_or(end - 1);
                }
                let item = &toks[k..(stop + 1).min(end)];
                match b.text.as_str() {
                    "claim" | "assume" | "require" if b.kind == Kind::Name => {
                        rest.extend(item.iter().cloned())
                    }
                    "fn" if b.kind == Kind::Name => {
                        let name = item.get(1).map_or("", |n| n.text.as_str());
                        let mut sig = Vec::new();
                        if let Some(f) = h.fns.iter().find(|f| f[1].text == name) {
                            given_types(f, &wanted, &args, t.line, &mut sig);
                        }
                        sig.pop();
                        let same = !sig.is_empty()
                            && item.len() >= sig.len()
                            && sig.iter().zip(item).all(|(a, b)| a.text == b.text);
                        if !same {
                            let msg = format!(
                                "`fn {name}` in `impl {} for {}` is not a function of the \
                                 contract, written as the contract writes it",
                                of.text, by.text
                            );
                            return Err(Error::new(b.line, msg));
                        }
                        again.push((name.to_string(), item.to_vec()));
                    }
                    _ => {
                        let msg = format!(
                            "`{}`: an `impl {} for {}` holds only what decomposes the promises \
                             of the contract: `claim` in a subject, `assume` in a boundary, \
                             `require`, or a function of the contract with what it `uses`",
                            b.text, of.text, by.text
                        );
                        return Err(Error::new(b.line, msg));
                    }
                }
                k = stop + 1;
            }
            let pair = (format!("{block}::{}", by.text), of.text.clone());
            if filled.contains(&pair) {
                let msg = format!("`{}` fills `{}` twice", pair.0, pair.1);
                return Err(Error::new(t.line, msg));
            }
            filled.push(pair);
            let at = |text: &str, kind: Kind| Tok {
                kind,
                text: text.into(),
                line: t.line,
            };
            let given =
                |item: &[Tok], into: &mut Vec<Tok>| given_types(item, &wanted, &args, t.line, into);
            after.push(at(&side, Kind::Name));
            after.push(at(&block, Kind::Name));
            after.push(at("{", Kind::Punct));
            after.push(at("impl", Kind::Name));
            if !params.is_empty() {
                after.push(at("<", Kind::Punct));
                after.extend(params.iter().cloned());
                after.push(at(">", Kind::Punct));
            }
            after.push(by.clone());
            after.extend(target.iter().cloned());
            after.push(at("{", Kind::Punct));
            for f in &h.fns {
                match again.iter().find(|a| a.0 == f[1].text) {
                    Some((_, written)) => after.extend(written.iter().cloned()),
                    None => given(f, &mut after),
                }
            }
            for (n, e) in &h.ensures {
                given(e, &mut after);
                let fill = format!("{block}::{}::{n}", by.text);
                c.fills.push((format!("{}::{n}", of.text), fill));
            }
            let said = match side.as_str() {
                "boundary" => "assume",
                _ => "claim",
            };
            for claim in &h.claims {
                after.push(at(said, Kind::Name));
                given(&claim[1..], &mut after);
            }
            after.extend(rest);
            after.push(at("}", Kind::Punct));
            after.push(at("}", Kind::Punct));
            i = end + 1;
            continue;
        }
        out.push(t.clone());
        i += 1;
    }
    out.append(&mut after);
    Ok((out, c))
}

/// `import store::Contract;` or `import store::{A, B};`: each contract
/// of the model of another store, written here in place of the line.
/// Also gives each contract brought with its store.
fn bring_contracts(
    toks: Vec<Tok>,
    handed: &Handed,
) -> Result<(Vec<Tok>, BTreeMap<String, String>), Error> {
    let mut brought: BTreeMap<String, String> = BTreeMap::new();
    // The types this text has: its own, and those it brings by name.
    let mut known: BTreeSet<String> = types_of(&toks).0;
    for (path, names) in imports_of(&toks).iter().filter(|i| i.0.len() == 2) {
        known.extend(names.iter().map(|n| format!("{}::{n}", path[1])));
    }
    // The contracts go after the last `import`: the types they name are
    // brought above them.
    let mut last = 0;
    let mut level = 0;
    for (k, t) in toks.iter().enumerate() {
        if t.kind == Kind::Punct {
            level += (t.text == "{") as i32 - (t.text == "}") as i32;
        }
        if level == 0 && t.kind == Kind::Name && t.text == "import" {
            let end = (k..toks.len()).find(|&j| toks[j].text == ";");
            last = end.map_or(toks.len(), |j| j + 1);
        }
    }
    let mut held: Vec<Tok> = Vec::new();
    let (mut out, mut depth, mut i) = (Vec::new(), 0, 0);
    while i < toks.len() {
        if i == last {
            out.append(&mut held);
        }
        let t = &toks[i];
        if t.kind == Kind::Punct {
            depth += (t.text == "{") as i32 - (t.text == "}") as i32;
        }
        let is = |j: usize, w: &str| toks.get(j).is_some_and(|t| t.text == w);
        let store = toks.get(i + 1).filter(|s| s.kind == Kind::Name);
        let store = store.filter(|s| handed.stores.contains_key(&s.text));
        let store = store.filter(|s| !handed.modules.contains_key(&s.text));
        let line = depth == 0 && t.kind == Kind::Name && t.text == "import" && is(i + 2, "::");
        // One name then `;`, or names in braces: not `store::block::…`.
        let (Some(store), true) = (store, line && !is(i + 4, "::")) else {
            out.push(t.clone());
            i += 1;
            continue;
        };
        let mut j = i + 3;
        let mut names = Vec::new();
        while toks.get(j).is_some_and(|n| n.text != ";") {
            if toks[j].kind == Kind::Name {
                names.push(toks[j].text.clone());
            }
            j += 1;
        }
        let there = lex(&handed.stores[&store.text]).unwrap_or_default();
        for name in names {
            match brought.get(&name) {
                Some(from) if from == &store.text => continue,
                Some(from) => {
                    let msg = format!("`{name}` is brought from `{from}` already");
                    return Err(Error::new(t.line, msg));
                }
                None => {}
            }
            let mut level = 0;
            let mut start = None;
            for (k, a) in there.iter().enumerate() {
                let named = there.get(k + 1).is_some_and(|n| n.text == name);
                if level == 0 && a.kind == Kind::Name && a.text == "contract" && named {
                    start = Some(k);
                    break;
                }
                if a.kind == Kind::Punct {
                    level += (a.text == "{") as i32 - (a.text == "}") as i32;
                }
            }
            let open = start.and_then(|k| (k..there.len()).find(|&b| there[b].text == "{"));
            let Some((start, end)) = start.zip(open.and_then(|b| closing(&there, b))) else {
                let msg = format!(
                    "the model of `{}` has no `contract {name}`: `import store::Name;` brings \
                     a contract, `import store::block::{{…}};` records and promises",
                    store.text
                );
                return Err(Error::new(t.line, msg));
            };
            // A type of a block of that store which is not brought here
            // is any value: `results::Outcome` without its import.
            let block = |w: &&[Tok]| {
                let word =
                    w[0].kind == Kind::Name && ["subject", "boundary"].contains(&&*w[0].text);
                word && w[1].kind == Kind::Name && w[2].text == "{"
            };
            let blocks: BTreeSet<&str> = (there.windows(3).filter(block))
                .map(|w| w[1].text.as_str())
                .collect();
            let mut k = start;
            while k <= end {
                let a = &there[k];
                let path = a.kind == Kind::Name
                    && blocks.contains(a.text.as_str())
                    && there.get(k + 1).is_some_and(|c| c.text == "::")
                    && there.get(k + 2).is_some_and(|n| n.kind == Kind::Name);
                if path && !known.contains(&format!("{}::{}", a.text, there[k + 2].text)) {
                    held.push(Tok {
                        kind: Kind::Name,
                        text: "Any".into(),
                        line: t.line,
                    });
                    k += 3;
                    continue;
                }
                held.push(Tok {
                    line: t.line,
                    ..a.clone()
                });
                k += 1;
            }
            brought.insert(name, store.text.clone());
        }
        i = j + 1;
    }
    if last >= toks.len() {
        out.append(&mut held);
    }
    Ok((out, brought))
}

fn read(toks: Vec<Tok>, handed: Rc<Handed>) -> Result<P, Error> {
    let (toks, brought) = bring_contracts(toks, &handed)?;
    let (toks, mut contracts) = contracts(toks)?;
    contracts.brought = brought;
    let (toks, mut methods) = impls(toks)?;
    methods.extend(contracts.methods.iter().cloned());
    let contracts = Rc::new(contracts);
    let (mut scoped, top) = types_of(&toks);
    // The types brought from a block of another store, by their path.
    for (path, names) in imports_of(&toks) {
        if of_store(&handed, &path) {
            let there = lex(&handed.stores[&path[0]]).unwrap_or_default();
            if let Ok(kept) = from_store(&there, &path[1], &names) {
                scoped.extend(types_of(&kept).0);
            }
        }
    }
    let toks = scope_types(toks, &scoped, &top, false);
    let block = |i: &usize| {
        let word =
            toks[*i].kind == Kind::Name && matches!(toks[*i].text.as_str(), "subject" | "boundary");
        word && toks.get(i + 2).is_some_and(|b| b.text == "{")
    };
    let blocks: BTreeSet<String> = (0..toks.len())
        .filter(block)
        .map(|i| toks[i + 1].text.clone())
        .collect();
    let blocks = Rc::new(blocks);
    let all = declared(&toks);
    let mut known = Names::default();
    let mut last = None;
    let mut seen = Model::default();
    for pass in 0..2 {
        let mut p = P {
            seen: std::mem::take(&mut seen),
            toks: toks.clone(),
            all: all.clone(),
            first: pass == 0,
            known,
            scoped: scoped.clone(),
            methods: methods.clone(),
            handed: handed.clone(),
            own_blocks: blocks.clone(),
            contracts: contracts.clone(),
            ..P::default()
        };
        p.items()?;
        if p.pos < p.toks.len() {
            return p.err(format!("unexpected `{}`", p.peek()));
        }
        known = p.names.clone();
        seen.functions = p.m.functions.clone();
        seen.records = p.m.records.clone();
        seen.aliases = p.m.aliases.clone();
        last = Some(p);
    }
    let mut last = last.expect("read twice");
    last.names.scoped = scoped;
    last.names.top = top;
    Ok(last)
}

/// The evidence of a ledger, then what the model itself declares. A
/// block on a piece the model no longer has is set aside, not refused.
pub fn evidence(src: &str, m: &Model, names: &Names) -> Result<(Vec<Evidence>, Vec<Aside>), Error> {
    let mut aside = Vec::new();
    let mut out = blocks(src, m, names, Some(&mut aside))?;
    let lines = src.lines().count().max(out.last().map_or(0, |e| e.line));
    for (i, d) in names.declared.iter().enumerate() {
        out.push(Evidence {
            line: lines + i + 1,
            ..d.clone()
        });
    }
    Ok((out, aside))
}

/// The evidence blocks of a text, each on a piece the model has.
pub fn evidence_alone(src: &str, m: &Model, names: &Names) -> Result<Vec<Evidence>, Error> {
    blocks(src, m, names, None)
}

/// The words after `source`, `rev` and `seen` in each evidence block of
/// a text, with its first and last line and the piece it names as
/// written: read without a model.
// mitra: implements ledger::entries
pub fn entries(src: &str) -> Vec<Entry> {
    let toks = lex(src).unwrap_or_default();
    let (mut out, mut i) = (Vec::new(), 0);
    while i < toks.len() {
        let open = toks[i].text == "evidence" && toks.get(i + 2).is_some_and(|t| t.text == "{");
        if !open {
            i += 1;
            continue;
        }
        let mut e = Entry {
            first: toks[i].line,
            source: Some(toks[i + 1].text.clone()),
            ..Entry::default()
        };
        let (mut depth, mut j) = (0, i + 2);
        while j < toks.len() {
            let word = |w: &str| toks[j].kind == Kind::Name && toks[j].text == w && depth == 1;
            let said = toks.get(j + 1).filter(|t| t.kind == Kind::Str);
            let said = said.map(|t| unquote(&t.text));
            match toks[j].text.as_str() {
                "{" => depth += 1,
                "}" => depth -= 1,
                _ if word("source") && said.is_some() => e.source = said,
                _ if word("rev") && said.is_some() => e.rev = said,
                _ if word("seen") && said.is_some() => e.seen = said,
                _ if word("for") && e.about.is_empty() => {
                    // `a::b::c`: names with `::` between them.
                    let mut k = j + 1;
                    while let Some(t) = toks.get(k).filter(|t| t.kind == Kind::Name) {
                        e.about += &t.text;
                        if toks.get(k + 1).is_none_or(|t| t.text != "::") {
                            break;
                        }
                        e.about += "::";
                        k += 2;
                    }
                }
                _ => {}
            }
            if depth == 0 {
                break;
            }
            j += 1;
        }
        e.last = toks.get(j).map_or(e.first, |t| t.line);
        out.push(e);
        i = j + 1;
    }
    out
}

/// The evidence blocks of a file. With `aside`, a block on a piece the
/// model does not have is put there; without, it is an error.
fn blocks(
    src: &str,
    m: &Model,
    names: &Names,
    mut aside: Option<&mut Vec<Aside>>,
) -> Result<Vec<Evidence>, Error> {
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
    // What a function with no `uses` calls outside its block: `for f::uses`.
    for f in names.opaque.clone() {
        let uses = format!("{f}::uses");
        if !names.claims.contains_key(&uses) {
            names.claims.insert(uses.clone(), (Vec::new(), Vec::new()));
            all.insert(uses);
        }
    }
    // And of each record to its struct: `for Record::tied`.
    for r in m.records.iter().filter(|r| r.of.is_none()) {
        let tied = format!("{}::tied", r.name);
        names.claims.insert(tied.clone(), (Vec::new(), Vec::new()));
        all.insert(tied);
    }
    // And of each enum to the enum of the code: `for Enum::tied`.
    for e in &m.enums {
        let tied = format!("{}::tied", e.name);
        names.claims.insert(tied.clone(), (Vec::new(), Vec::new()));
        all.insert(tied);
    }
    let mut p = P {
        toks: scope_types(lex(src)?, &names.scoped, &names.top, true),
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
            let (start, line) = (p.pos, p.line());
            match (p.evidence(), aside.as_deref_mut()) {
                (Ok(e), _) => out.push(e),
                (Err(e), Some(aside)) if e.message.starts_with(NO_PIECE) => {
                    // To the end of the block.
                    let (mut depth, mut end) = (0, start + 2);
                    while let Some(t) = p.toks.get(end) {
                        depth += (t.text == "{") as i32 - (t.text == "}") as i32;
                        end += 1;
                        if depth == 0 {
                            break;
                        }
                    }
                    p.pos = end;
                    let known = entries(src).into_iter().find(|x| x.first == line);
                    let known = known.unwrap_or_default();
                    aside.push(Aside {
                        about: known.about,
                        source: known.source,
                        rev: known.rev,
                        why: Why::Orphaned,
                        line,
                    });
                }
                (Err(e), _) => return Err(e),
            }
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

/// Why a value of a record with unstated parts is refused.
fn unwritten(name: &str) -> String {
    format!(
        "`{name}` has parts the model does not state (`record {name};` or `..`): \
         no value of it can be written"
    )
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
pub fn stamp(
    text: &str,
    source: Option<&str>,
    rev: Option<&str>,
    seen: &[Option<String>],
) -> Result<String, String> {
    let mut block = 0;
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
                // The fingerprint of its piece, block by block.
                if let Some(Some(v)) = seen.get(block) {
                    if !out.ends_with(' ') {
                        out.push(' ');
                    }
                    out += &format!("seen \"{v}\" ");
                }
                block += 1;
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

    fn entry(about: &str, source: Option<&str>, rev: Option<&str>, lines: (usize, usize)) -> Entry {
        Entry {
            about: about.to_string(),
            source: source.map(String::from),
            rev: rev.map(String::from),
            seen: None,
            first: lines.0,
            last: lines.1,
        }
    }

    // mitra: evidence for ledger::t_entries_one
    #[test]
    fn t_entries_one() {
        let text = "evidence r { for a::b holds source \"review:me\" rev \"1\" }";
        let one = entry("a::b", Some("review:me"), Some("1"), (1, 1));
        assert_eq!(entries(text), [one]);
    }

    // mitra: evidence for ledger::t_entries_label_is_the_source
    #[test]
    fn t_entries_label_is_the_source() {
        let text = "evidence ci {\n  for f given a = 1 gives 2\n}";
        assert_eq!(entries(text), [entry("f", Some("ci"), None, (1, 3))]);
    }

    // mitra: evidence for ledger::t_entries_none
    #[test]
    fn t_entries_none() {
        assert_eq!(entries("// nothing"), []);
    }

    #[test]
    fn entries_read_each_block() {
        let text = "// head\nevidence a { for f holds seen \"12\" }\nevidence b { for g given s = \"source\" gives 1 }";
        let read = entries(text);
        assert_eq!(read.len(), 2);
        assert_eq!(read[0].seen.as_deref(), Some("12"));
        assert_eq!((read[1].about.as_str(), read[1].first), ("g", 3));
        assert_eq!(read[1].source.as_deref(), Some("b"));
    }

    #[test]
    fn reads_the_64_bit_extremes() {
        let ok = "subject fn f(a: Int) -> Int;\n\
                  claim c() { f(-9223372036854775808) == 9223372036854775807 }";
        assert!(model(ok).is_ok());
        let e = error("claim c() { 9223372036854775808 == 1 }");
        assert!(e.message.contains("outside the 64-bit integers"), "{e}");
    }

    #[test]
    fn set_operators_bind_as_documented() {
        let (m, _) = model("subject fn f(a: 0..=9 \\ {3} | {20} & 15..=25) -> Int;").unwrap();
        let a = &m.functions[0].params[0].1;
        assert!(a.contains(&Value::Int(20)) && a.contains(&Value::Int(4)));
        assert!(!a.contains(&Value::Int(3)) && !a.contains(&Value::Int(15)));
    }

    #[test]
    fn errors_point_at_their_line() {
        let e = error("subject fn f(a: Int) -> Int;\n\nsubject fn g(a: Nope) -> Int;");
        assert_eq!(e.line, 3);
        assert!(e.message.contains("unknown type `Nope`"), "{e}");
        let e = error("subject fn if(a: Int) -> Int;");
        assert!(e.message.contains("`if` is a keyword"), "{e}");
        let e = error("subject fn f(a: String) -> Int;\nclaim c() { f(\"a\\qb\") == 1 }");
        assert_eq!(e.line, 2);
        assert!(e.message.contains("in a string"), "{e}");
    }

    #[test]
    fn rust_names_are_the_same_types() {
        let (m, _) =
            model("import std::rust::{Vec};\nsubject fn f(a: Vec<i32>, b: usize) -> bool;")
                .unwrap();
        let (n, _) = model("subject fn f(a: List<Int32>, b: 0..) -> Bool;").unwrap();
        assert_eq!(m.functions[0].params, n.functions[0].params);
        assert_eq!(m.functions[0].range, n.functions[0].range);
    }

    #[test]
    fn trust_goes_from_0_to_1() {
        let (m, names) = model("subject fn f(a: Int) -> Int;").unwrap();
        let read = |e: &str| evidence_alone(e, &m, &names);
        assert!(read("evidence t { for f given a = 1 holds trust 0.5 }").is_ok());
        let e = read("evidence t { for f given a = 1 holds trust 2 }").unwrap_err();
        assert!(e.message.contains("from 0 to 1"), "{e}");
        let e = read("evidence t { for f given b = 1 holds }").unwrap_err();
        assert!(e.message.contains("has no input `b`"), "{e}");
    }

    // mitra: evidence for reader::reads_a_subject
    #[test]
    fn reads_a_subject() {
        assert!(model("subject fn f() -> Bool;").is_ok());
    }

    // mitra: evidence for reader::refuses_a_bare_fn
    #[test]
    fn refuses_a_bare_fn() {
        assert!(model("fn f() -> Bool;").is_err());
    }

    /// A sweep over faulty texts at hand, not every text.
    // mitra: evidence for reader::fault_is_placed sampled 13
    #[test]
    fn fault_is_placed() {
        let faulty = [
            "",
            "fn",
            "fn f() -> Bool;",
            "subject s {",
            "subject s { fn f( -> Bool; }",
            "claim c() { 1 + }",
            "record R { }",
            "enum E { A, A }\nsubject fn f(e: E) -> Bool;",
            "subject s { ensure w \"x\"; }",
            "subject s { require t::x; }",
            "\n\nsubject fn f() -> Nope;",
            "\"unclosed",
            "import std::nothing::{x};",
        ];
        for text in faulty {
            if let Err(e) = model(text) {
                assert!(e.line >= 1 && !e.message.is_empty(), "{text:?}: {e:?}");
            }
        }
    }

    #[test]
    fn a_function_with_no_body_is_code_or_the_world() {
        let e = error("fn f(a: Int) -> Int;");
        assert!(e.message.contains("write `subject fn f(…)`"), "{e}");
        // Named as written, with or without a name on the block.
        let (m, names) = model(
            "subject fn f(a: Int) -> Int;\nboundary fn now() -> Int;\n\
             subject { fn g(a: Int) -> Int { a } }\nboundary { fn h() -> Int; }\n\
             subject s { fn k() -> Int; }",
        )
        .unwrap();
        let fns: Vec<&str> = m.functions.iter().map(|f| f.name.as_str()).collect();
        let code: Vec<&str> = fns
            .into_iter()
            .filter(|f| !f.ends_with("::prose"))
            .collect();
        assert_eq!(code, ["f", "now", "g", "g::body", "h", "s::k"]);
        let claims: Vec<&str> = m.claims.iter().map(|c| c.name.as_str()).collect();
        let claims: Vec<&str> = (claims.into_iter())
            .filter(|c| !c.ends_with("::deterministic"))
            .collect();
        assert_eq!(claims, ["f::correct", "g::spec", "s::k::correct"]);
        let outside: Vec<&String> = names.boundaries.iter().collect();
        assert_eq!(outside, ["h", "now"]);
        // Several unnamed blocks share the names of the file.
        let twice = "subject { fn f() -> Int; }\nsubject { fn f() -> Int; }";
        let e = crate::load(twice, "").unwrap_err().1;
        assert!(e.message.contains("`f`"), "{e}");
    }

    #[test]
    fn an_import_is_read_in_place() {
        let (m, _) = model(
            "import std::claims::{commutative};\n\
             subject fn add(a: 0..=3, b: 0..=3) -> Int;\n\
             claim add_commutes = commutative(add, 0..=3);",
        )
        .unwrap();
        let names: Vec<&str> = m.claims.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(
            names,
            ["add::correct", "add::deterministic", "add_commutes"]
        );
        let e = error("import std::claims::{nope};");
        assert!(e.message.contains("declares no `nope`"), "{e}");
    }
}
