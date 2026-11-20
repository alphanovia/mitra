//! Reading `import`: the modules shipped with Mitra, the files handed
//! with a model, the blocks of another store.

use crate::reader::prelude::*;

/// The modules shipped with the tool: plain Mitra, `mitra explain std::claims`
/// prints one.
pub(crate) const STD: [(&str, &str); 3] = [
    ("std::claims", include_str!("../../std/claims.mitra")),
    ("std::core", include_str!("../../std/core.mitra")),
    ("std::rust", include_str!("../../std/rust.mitra")),
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
pub(crate) fn module(handed: &Handed, path: &[String]) -> Result<String, String> {
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
pub(crate) fn from_store(toks: &[Tok], block: &str, names: &[String]) -> Result<Vec<Tok>, String> {
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

/// Each `import` at the top of a text: its path and the names it brings.
pub(crate) fn imports_of(toks: &[Tok]) -> Vec<(Vec<String>, Vec<String>)> {
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
pub(crate) fn of_store(handed: &Handed, path: &[String]) -> bool {
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

impl P {
    /// The named declarations of another file, read as if written here.
    pub(crate) fn import(&mut self, line: usize, path: &[String], names: &[String]) -> R<()> {
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
}
