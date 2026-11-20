//! The reader of the Mitra language: the cursor over the tokens of a
//! text, the items of a file, and the driver of a read. What each
//! concept writes is read in its own folder (`types/read.rs`,
//! `claims/read.rs`, ...), as methods of the same reader `P`. It
//! builds the model as it reads (docs/reference/language.md).

pub(crate) mod finish;
pub(crate) mod tokens;

/// What every file of the reader sees.
pub(crate) mod prelude {
    pub(crate) use crate::contracts::impls::*;
    pub(crate) use crate::contracts::read::*;
    pub(crate) use crate::intent::read::*;
    pub(crate) use crate::ledger::read::*;
    pub(crate) use crate::model::*;
    pub(crate) use crate::modules::read::*;
    pub(crate) use crate::reader::tokens::*;
    pub(crate) use crate::reader::*;
    pub(crate) use crate::types::scope::*;
    pub(crate) use crate::types::set::{Set, Value, MAP, SET};
    pub(crate) use crate::Error;
    pub(crate) use std::collections::{BTreeMap, BTreeSet};
    pub(crate) use std::rc::Rc;
}

use prelude::*;

/// The inputs of a function: each name with its type, then the types
/// with the marks of the recursive ones, the result last.
#[derive(Debug, Default, Clone)]
pub(crate) struct Sig {
    pub(crate) params: Vec<(String, Set)>,
    pub(crate) deep: Vec<Set>,
}

/// What the model declares, for the evidence that names it.
#[derive(Debug, Default, Clone)]
// mitra: implements reader::Names
pub struct Names {
    pub(crate) fns: BTreeMap<String, Sig>,
    /// Each claim with its inputs, and the measures in its body.
    pub(crate) claims: BTreeMap<String, (Vec<String>, Vec<Expr>)>,
    /// The functions of a subject that have a body: their spec.
    pub(crate) specified: BTreeMap<String, String>,
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
pub(crate) type Bound = (String, Option<Set>, usize);

/// A token of the parts of the reader that match on its kind.
pub(crate) enum Old {
    Ident(String),
    Num(String),
    Str(String),
}

#[derive(Default)]
pub(crate) struct P {
    pub(crate) toks: Vec<Tok>,
    pub(crate) pos: usize,
    /// The subject or boundary being read.
    pub(crate) scope: Option<String>,
    /// `subject` or `boundary`, inside one: with a name (`scope`) or not.
    pub(crate) block: Option<String>,
    /// The blocks read so far: each `subject` or `boundary` has a number.
    pub(crate) blocks: usize,
    /// Each claim and function of a block, with the number of its block.
    pub(crate) owned: Vec<(String, usize)>,
    /// `ensure` and `require`: name, prose, kind, block, line.
    pub(crate) ensures: Vec<(String, String, Prose, usize, usize, Option<f64>)>,
    /// Every function and claim of the file, by full name.
    pub(crate) all: BTreeSet<String>,
    /// How many fields each variant has.
    pub(crate) variants: BTreeMap<String, usize>,
    /// The names bound where the parser is.
    pub(crate) locals: Vec<String>,
    /// Inside the body of a subject's function: its name, and its spec.
    pub(crate) own: Option<(String, String)>,
    /// The measures met since last cleared.
    pub(crate) measures: Vec<Expr>,
    pub(crate) names: Names,
    pub(crate) fresh: usize,
    /// The names of a first reading of the whole file.
    pub(crate) known: Names,
    /// The spec claims of the functions called since last cleared.
    pub(crate) called: Vec<String>,
    /// Whether this is the first reading.
    pub(crate) first: bool,
    /// Whether an expression read the code or a measure, since last cleared.
    pub(crate) impure: bool,
    /// The claims with parameters: their parameters and their body.
    pub(crate) templates: BTreeMap<String, (Vec<String>, Vec<Tok>)>,
    /// What was imported so far: the module and the name.
    pub(crate) imported: BTreeSet<(String, String)>,
    /// The modules that can be imported.
    pub(crate) handed: Rc<Handed>,
    /// The blocks this model declares itself, by name.
    pub(crate) own_blocks: Rc<BTreeSet<String>>,
    /// The blocks brought from other stores: the block, then its store.
    pub(crate) brought: BTreeMap<String, String>,
    /// While a block of another store is read: that store.
    pub(crate) bringing: Option<String>,
    /// Each enum already met with sets for its parameters, read deep or
    /// not, and the set it gave.
    pub(crate) instances: Vec<(String, bool, Vec<Set>, Set)>,
    /// The modules being imported, outermost first.
    pub(crate) stack: Vec<String>,
    /// A claim of the shape "the same judgement on every input of a set
    /// or of a class" is being read as that set or those classes.
    pub(crate) shape: bool,
    /// The line every declaration of an imported module is given.
    pub(crate) at: Option<usize>,
    /// The model being built.
    pub(crate) m: Model,
    /// The property claims, each with the names it binds.
    pub(crate) properties: Vec<(usize, Vec<Bound>)>,
    /// The records and enums declared so far.
    pub(crate) records: Vec<Record>,
    /// The types and variants of the blocks, by their full name.
    pub(crate) scoped: BTreeSet<String>,
    /// The functions of an `impl` that take `self`, by their full name.
    pub(crate) methods: BTreeSet<String>,
    pub(crate) contracts: Rc<Contracts>,
    /// The inputs of the function or claim being read, with their types.
    pub(crate) typed: Vec<(String, Set)>,
    /// The functions and records of the first reading: what a call gives.
    pub(crate) seen: Model,
    pub(crate) enums: Vec<Enum>,
    /// The sets the type parameters stand for, where the parser is.
    pub(crate) generics: Vec<(String, Set)>,
    /// In a body: each opaque type of it, and the contracts it fills.
    pub(crate) opaque: Vec<(Set, Vec<String>)>,
    /// The bounds of the type parameters last read: `T: A + B`.
    pub(crate) bounds: Vec<(String, Vec<String>)>,
    /// A block, a contract a bound of it names, and the line.
    pub(crate) leaning: Vec<(String, String, usize)>,
    /// Sets written with an enum, by their printed form.
    pub(crate) aliases: Vec<(String, String)>,
    /// The names used as a record with a set for each field.
    pub(crate) used: Vec<String>,
    /// Whether the last `args` were written with parentheses: `f()`.
    pub(crate) parens: bool,
    /// The record or enum being declared, and how it is written with its
    /// parameters: `Tree<Int>`. Its own name, inside, is the type itself.
    pub(crate) me: Option<(String, String)>,
    /// Whether this reads the body of an enum.
    pub(crate) in_enum: bool,
    /// How deep inside a value the parser is.
    pub(crate) nested: usize,
    /// The functions declared so far: a function type holds those that
    /// fit, and the name of one is a value.
    pub(crate) functions: Vec<Function>,
    /// Whether this reads the parameters of a function type.
    pub(crate) contra: bool,
    /// Whether a type named itself since this was last cleared, and
    /// whether it did so as the whole type of a field.
    pub(crate) met: bool,
    pub(crate) direct: bool,
    /// A set can't hold a type that contains itself. Read plainly, the
    /// type inside itself is `Value`; read `deep`, it is a mark,
    /// `$Tree<Int>`, and `recursive` gives the set each mark stands for.
    pub(crate) deep: bool,
    pub(crate) recursive: BTreeMap<String, Set>,
}

pub(crate) type R<T> = Result<T, Error>;

pub(crate) fn bin(op: &'static str, a: Expr, b: Expr) -> Expr {
    Expr::Binary(op, Box::new(a), Box::new(b))
}

impl P {
    pub(crate) fn line(&self) -> usize {
        let last = self.toks.last().map_or(1, |t| t.line);
        self.toks.get(self.pos).map_or(last, |t| t.line)
    }

    pub(crate) fn err<T>(&self, msg: impl Into<String>) -> R<T> {
        Err(Error::new(self.line(), msg.into()))
    }

    pub(crate) fn peek(&self) -> &str {
        self.toks.get(self.pos).map_or("", |t| t.text.as_str())
    }

    pub(crate) fn peek_at(&self, n: usize) -> &str {
        self.toks.get(self.pos + n).map_or("", |t| t.text.as_str())
    }

    pub(crate) fn kind(&self) -> Option<Kind> {
        self.toks.get(self.pos).map(|t| t.kind.clone())
    }

    pub(crate) fn kind_at(&self, n: usize) -> Option<Kind> {
        self.toks.get(self.pos + n).map(|t| t.kind.clone())
    }

    pub(crate) fn is(&self, text: &str) -> bool {
        self.peek() == text && self.kind() != Some(Kind::Str)
    }

    pub(crate) fn eat(&mut self, text: &str) -> bool {
        let yes = self.is(text);
        self.pos += yes as usize;
        yes
    }

    pub(crate) fn expect(&mut self, text: &str) -> R<()> {
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

    pub(crate) fn name(&mut self) -> R<String> {
        match self.kind() {
            Some(Kind::Name) => {
                self.pos += 1;
                Ok(self.toks[self.pos - 1].text.clone())
            }
            _ => self.err("expected a name"),
        }
    }

    /// `a::b::c`
    pub(crate) fn path(&mut self) -> R<String> {
        let mut path = self.name()?;
        while self.eat("::") {
            path = format!("{path}::{}", self.name()?);
        }
        Ok(path)
    }

    pub(crate) fn qualified(&self, name: &str) -> String {
        match &self.scope {
            Some(s) => format!("{s}::{name}"),
            None => name.to_string(),
        }
    }

    /// A function or claim, as named from where the parser is.
    pub(crate) fn resolve(&self, name: &str) -> String {
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
    pub(crate) fn find(&self, name: &str) -> R<String> {
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

    pub(crate) fn old(&self) -> Option<Old> {
        let t = self.toks.get(self.pos)?;
        Some(match t.kind {
            Kind::Name => Old::Ident(t.text.clone()),
            Kind::Num => Old::Num(t.text.clone()),
            Kind::Str => Old::Str(unquote(&t.text)),
            Kind::Punct => return None,
        })
    }

    pub(crate) fn next_old(&mut self) -> Option<Old> {
        let t = self.old();
        self.pos += 1;
        t
    }

    pub(crate) fn at_end(&self) -> bool {
        self.pos >= self.toks.len()
    }

    pub(crate) fn is_name(&self) -> bool {
        self.kind() == Some(Kind::Name) && !KEYWORDS.contains(&self.peek())
    }

    /// A name that is no reserved word.
    pub(crate) fn ident(&mut self) -> R<String> {
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
    pub(crate) fn word(&mut self) -> R<String> {
        match self.kind() {
            Some(Kind::Name) => self.name(),
            _ => self.err("expected a name"),
        }
    }

    /// The name of an input or of a `let`: also a word that only starts
    /// a declaration, such as `claim`.
    pub(crate) fn local(&mut self) -> R<String> {
        match self.kind() == Some(Kind::Name) && DECLARING.contains(&self.peek()) {
            true => self.name(),
            false => self.ident(),
        }
    }

    pub(crate) fn is_keyword(&self, k: &str) -> bool {
        self.kind() == Some(Kind::Name) && self.peek() == k
    }

    pub(crate) fn eat_keyword(&mut self, k: &str) -> bool {
        let yes = self.is_keyword(k);
        self.pos += yes as usize;
        yes
    }

    /// `<A, B>` after a name, or nothing.
    pub(crate) fn names_between(&mut self) -> R<Vec<String>> {
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

    pub(crate) fn items(&mut self) -> R<()> {
        while self.pos < self.toks.len() && !self.is("}") {
            let line = self.at.unwrap_or(self.line());
            let kw = self.name()?;
            match kw.as_str() {
                // type Vec<T> = List<T>;  type bool = Bool;  another name for a type, replaced
                // where the text was read (`lex`): checked here.
                "type" if self.scope.is_none() && self.block.is_none() => {
                    let name = self.name()?;
                    let params = self.names_between()?;
                    self.expect("=")?;
                    let named = self.name()?;
                    let plain = [
                        "Bool", "Int", "Nat", "Int8", "Int16", "Int32", "Int64", "UInt8", "UInt16",
                        "UInt32", "Float64", "String",
                    ];
                    let collection = ["List", "Set", "Map"].contains(&named.as_str());
                    if !collection && !(plain.contains(&named.as_str()) && params.is_empty()) {
                        return self.err(format!(
                            "`type {name}` names `{named}`: a `type` is another name for a type of the language"
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
pub(crate) fn declared(toks: &[Tok]) -> BTreeSet<String> {
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

pub(crate) fn read(toks: Vec<Tok>, handed: Rc<Handed>) -> Result<P, Error> {
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

#[cfg(test)]
mod tests;
