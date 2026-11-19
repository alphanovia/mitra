//! A model, compiled: every piece it holds, written or implied, with its
//! kind, its fingerprint and what it rests on. The text form is the
//! build file a store keeps; nothing here touches a disk.

use crate::ast::*;
use crate::fingerprint;

/// One piece of a compiled model.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements build::Item
pub struct Item {
    /// `function`, `spec`, `helper`, `world`, `record`, `enum`, `claim`,
    /// `implied`, `ensure`, `require`, `promise` (of a boundary) or
    /// `capture`.
    pub kind: String,
    pub name: String,
    /// The fingerprint of what it says.
    pub print: String,
    /// The pieces it rests on.
    pub on: Vec<String>,
}

/// A compiled model.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements build::Built
pub struct Built {
    pub items: Vec<Item>,
    /// The fingerprint of the whole: of every piece and what it rests on.
    pub model: String,
}

/// What is under a function that the model implies and never writes.
const UNDER: [&str; 5] = ["correct", "spec", "deterministic", "uses", "tied"];

/// Compile a model that was read with no evidence.
// mitra: implements build::build
pub fn build(m: &Model) -> Built {
    let prints = fingerprint::all(m);
    let print = |name: &str| fingerprint::of(&prints, name).cloned().unwrap_or_default();
    let mut items = Vec::new();
    let mut push = |kind: &str, name: &str, on: Vec<String>| {
        items.push(Item {
            kind: kind.to_string(),
            name: name.to_string(),
            print: print(name),
            on,
        })
    };
    for r in m.records.iter().filter(|r| r.of.is_none()) {
        push("record", &r.name, Vec::new());
    }
    for e in &m.enums {
        push("enum", &e.name, Vec::new());
    }
    for f in m.functions.iter().filter(|f| !hidden(&f.name)) {
        let block = m.block_of(&f.name).map(|b| b.side);
        let kind = match (m.boundaries.contains(&f.name), f.is_spec(), block) {
            (true, _, _) => "world",
            (_, false, _) => "function",
            (_, true, Some(Side::Subject)) => "spec",
            (_, true, _) if m.claim(&format!("{}::spec", f.name)).is_some() => "spec",
            (_, true, _) => "helper",
        };
        push(kind, &f.name, Vec::new());
    }
    for c in m.claims.iter().filter(|c| !hidden(&c.name)) {
        let last = c.name.rsplit("::").next().unwrap_or_default();
        if m.captured(&c.name).is_some() {
            push("capture", &c.name, Vec::new());
        } else if let Some(e) = m.ensure(&c.name) {
            let mut on = e.claims.clone();
            on.extend(e.requires.iter().cloned());
            on.extend(e.needs.iter().cloned());
            if !e.boundary && e.needs.is_none() {
                on.push(format!("{}::captured", e.name));
            }
            let kind = match (e.needs.is_some(), e.world) {
                (true, _) => "require",
                (_, true) => "promise",
                _ => "ensure",
            };
            push(kind, &c.name, on);
        } else if UNDER.contains(&last) && c.name != last {
            push("implied", &c.name, c.from.clone());
        } else {
            push("claim", &c.name, c.from.clone());
        }
    }
    // By name: the order of the declarations is not part of the model.
    items.sort_by(|a, b| a.name.cmp(&b.name));
    let mut built = Built {
        items,
        model: String::new(),
    };
    built.model = fingerprint::hash(&built.lines());
    built
}

/// The functions and claims Mitra makes to hold a piece: not pieces.
fn hidden(name: &str) -> bool {
    ["::prose", "::body", "::case"]
        .iter()
        .any(|s| name.ends_with(s))
}

impl Built {
    /// One line per piece: `kind name fingerprint <- what it rests on`.
    fn lines(&self) -> String {
        let mut out = String::new();
        for i in &self.items {
            out += &format!("{} {} {}", i.kind, i.name, i.print);
            if !i.on.is_empty() {
                out += &format!(" <- {}", i.on.join(", "));
            }
            out.push('\n');
        }
        out
    }

    /// The build file: a head, then one line per piece.
    // mitra: implements build::Built::lock
    pub fn lock(&self) -> String {
        format!("mitra build 1\nmodel {}\n{}", self.model, self.lines())
    }

    /// How many pieces of a kind it holds.
    // mitra: implements build::Built::count
    pub fn count(&self, kind: &str) -> usize {
        self.items.iter().filter(|i| i.kind == kind).count()
    }
}

/// The interface the code must meet: each function, record and enum of
/// a subject, as the model states it, under the comment that ties code
/// to it. What a boundary or another store declares is not ours to write.
// mitra: implements build::interface
pub fn interface(m: &Model) -> String {
    // In a subject of this store; a function may also stand alone.
    let ours = |name: &str, alone: bool| match m.block_of(name) {
        Some(b) => b.side == Side::Subject && b.from.is_none(),
        None => alone,
    };
    let mut out = String::from(
        "// The code this model expects. Above each piece: the comment to put\n\
         // above the code that implements it; `mitra tie` then compares the two.\n",
    );
    for i in &build(m).items {
        if !ours(&i.name, matches!(i.kind.as_str(), "function" | "spec")) {
            continue;
        }
        let short = i.name.rsplit("::").next().unwrap_or(&i.name);
        let text = match i.kind.as_str() {
            "record" => {
                let r = m
                    .records
                    .iter()
                    .find(|r| r.name == i.name)
                    .expect("built from it");
                let stated = r.fields.iter().filter(|f| f.0 != "..");
                let mut fields: Vec<String> = stated.map(|f| format!("{}: {}", f.0, f.1)).collect();
                if r.open && !fields.is_empty() {
                    fields.push("..".to_string());
                }
                match fields.is_empty() {
                    true => format!("record {short};"),
                    false => format!("record {short} {{ {} }}", fields.join(", ")),
                }
            }
            "enum" => {
                let e = m
                    .enums
                    .iter()
                    .find(|e| e.name == i.name)
                    .expect("built from it");
                let body: Vec<&str> = e.body.split_whitespace().collect();
                format!("enum {short} {{ {} }}", body.join(" "))
            }
            "function" | "spec" => {
                let f = m.function(&i.name).expect("built from it");
                let params: Vec<String> = f
                    .params
                    .iter()
                    .map(|p| format!("{}: {}", p.0, p.1))
                    .collect();
                let body = match &f.body {
                    Some(b) => format!(" {{ {} }}", crate::print::expr(b)),
                    None => ";".to_string(),
                };
                format!("fn {short}({}) -> {}{body}", params.join(", "), f.range)
            }
            _ => continue,
        };
        out += &format!("\n// mitra: implements {}\n{}\n", i.name, m.named(&text));
    }
    out
}
