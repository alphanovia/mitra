//! `mitra tie`: the reader of Rust code. A comment `// mitra: implements
//! subject::name` above a `fn` ties it to a function of the model. The
//! reader compares the two (inputs, result, calls) and says what it found
//! as plain evidence: the core never depends on it, and a reviewer can
//! bring the same evidence by hand.

mod code;
mod erasure;

use code::*;
use erasure::*;
use std::path::Path;

use crate::report::libtest;
use crate::source::{fn_below, mitra_line, rust_files, scan, stray};
use mitra_lang::ledger::results::{Outcome, Tied};
use mitra_lang::{Found, Model, Reader, Settings, Target};

/// The reader of Rust code.
// mitra: implements reader::Rust
pub struct Rust;

impl Reader<Path> for Rust {
    /// The report of `cargo test`, as plain results.
    // mitra: implements reader::Rust::results
    fn results(&self, text: &str) -> Vec<Outcome> {
        libtest(text)
    }

    /// Every test of `src/` and `tests/` tied to a claim.
    // mitra: implements reader::Rust::tests
    fn tests(&self, root: &Path) -> Result<Vec<Tied>, String> {
        let mut files = Vec::new();
        for d in ["src", "tests"] {
            rust_files(&root.join(d), &mut files);
        }
        files.sort();
        let mut out = Vec::new();
        for f in files {
            let shown = f.strip_prefix(root).unwrap_or(&f).display().to_string();
            let text = std::fs::read_to_string(&f).map_err(|e| format!("{shown}: {e}"))?;
            stray(&shown, &text)?;
            let lines: Vec<&str> = text.lines().collect();
            let scanned = scan(&text);
            for i in 0..lines.len() {
                let tie = mitra_line(lines[i]).and_then(|m| m.strip_prefix("evidence for "));
                let Some(claim) = tie.filter(|_| scanned[i].0) else {
                    continue;
                };
                let place = format!("{shown}:{}", i + 1);
                let code: Vec<&str> = scanned[i + 1..].iter().map(|l| l.1.as_str()).collect();
                let code = code.join("\n");
                let Some(test) = fn_below(&code) else {
                    return Err(format!(
                        "{place}: `evidence for` must sit above a test `fn`"
                    ));
                };
                let claim = claim.trim();
                let (claim, pinned) = claim.split_once(char::is_whitespace).unwrap_or((claim, ""));
                out.push(Tied {
                    test: test.to_string(),
                    claim: claim.to_string(),
                    pinned: pinned.trim().to_string(),
                    place,
                });
            }
        }
        Ok(out)
    }

    // mitra: implements reader::Rust::erasures
    fn erasures(&self) -> Vec<(&'static str, f64)> {
        PROFILE.to_vec()
    }

    /// Compare the code under `root` with the model.
    // mitra: implements reader::Rust::tie
    fn tie(&self, root: &Path, model: &Model, rev: Option<&str>) -> Result<Found, String> {
        // The cost of each erasure: the settings', or the profile's.
        let said = &model.settings.erasure;
        if let Some(name) = said.keys().find(|k| !PROFILE.iter().any(|p| p.0 == *k)) {
            let known: Vec<&str> = PROFILE.iter().map(|p| p.0).collect();
            return Err(format!(
                "settings: no erasure `{name}` in the profile of this reader ({})",
                known.join(", ")
            ));
        }
        let cost = |name: &str| {
            let own = PROFILE.iter().find(|p| p.0 == name).map_or(0.0, |p| p.1);
            said.get(name).copied().unwrap_or(own)
        };
        let mut files = Vec::new();
        for d in ["src", "tests"] {
            rust_files(&root.join(d), &mut files);
        }
        files.sort();
        let (mut code, mut tests) = (Vec::new(), String::new());
        let (mut shapes, mut code_lines) = (Vec::new(), 0);
        for f in files {
            let shown = f.strip_prefix(root).unwrap_or(&f).display().to_string();
            let text = std::fs::read_to_string(&f).map_err(|e| format!("{shown}: {e}"))?;
            stray(&shown, &text)?;
            read(&shown, &text, &mut code, &mut shapes)?;
            if shown.starts_with("src") {
                let above = text.lines().take_while(|l| l.trim() != "#[cfg(test)]");
                let counted = |l: &&str| !l.trim().is_empty() && !l.trim().starts_with("//");
                code_lines += above.filter(counted).count();
            }
            tests += &text;
        }
        let mut found = Found {
            evidence: Vec::new(),
            notes: Vec::new(),
            agree: 0,
            differ: 0,
            untied: Vec::new(),
            code_lines,
        };
        let attrs = match rev {
            Some(r) => format!(" rev \"{r}\""),
            None => String::new(),
        };
        let clean = |why: &str| why.replace(['"', '\\'], "'");
        // A tie that agrees says what was erased to get there, and is
        // trusted for what the erasures leave.
        let say = |found: &mut Found,
                   claim: &str,
                   place: &str,
                   why: Option<String>,
                   erased: &Erased| match why {
            None => {
                found.agree += 1;
                let costs: Vec<f64> = erased.iter().map(|e| cost(e.0)).collect();
                let trust = Settings::strength(&costs);
                let (mut source, mut said) = (format!("reader:rust {place}"), String::new());
                if !erased.is_empty() {
                    let each = erased.iter().map(|(name, what)| match what.is_empty() {
                        true => name.to_string(),
                        false => format!("{name}({what})"),
                    });
                    let list: Vec<String> = each.collect();
                    source += &clean(&format!("; erased {}: {}", list.len(), list.join(", ")));
                }
                if trust < 1.0 {
                    said = format!(" trust {}", (trust * 1000.0).round() / 1000.0);
                }
                found.evidence.push(format!(
                    "evidence tie {{ for {claim} holds{said} act tie by \"machine:mitra-rust\" source \"{source}\"{attrs} }}"
                ));
            }
            Some(why) => {
                found.differ += 1;
                found.notes.push(format!("{place}: {claim}: {why}"));
                let source = format!("reader:rust {place}: {}", clean(&why));
                found.evidence.push(format!(
                    "evidence tie {{ for {claim} fails act tie by \"machine:mitra-rust\" source \"{source}\"{attrs} }}"
                ));
            }
        };
        // Records: every field the model states is in the struct, with a type
        // that fits; without `..` the struct has no other field.
        for s in &shapes {
            // An enum of the model: the same variants, each holding as many
            // values, of types that fit.
            if let Some(e) = model.enums.iter().find(|e| e.name == s.implements) {
                if shapes
                    .iter()
                    .filter(|d| d.implements == s.implements)
                    .count()
                    > 1
                {
                    let msg = format!("{}: `{}` is implemented twice", s.place, s.implements);
                    return Err(msg);
                }
                let last = |n: &str| n.rsplit("::").next().unwrap_or(n).to_string();
                let mut why = None;
                let mut erased: Erased = Vec::new();
                // `record R<T> { … }` is read as an enum of one variant.
                let alone = e.variants.len() == 1 && e.variants[0] == e.name;
                let record = (model.records.iter())
                    .find(|r| r.name == e.name && r.of.as_deref() == Some(e.name.as_str()));
                let whole = record.filter(|_| alone && s.kind == "struct");
                if let (None, Some(fields), Some(r)) = (&s.variants, &s.fields, whole) {
                    {
                        for (name, set) in r.fields.iter().filter(|f| f.0 != "..") {
                            let mty = model.named(&set.to_string());
                            match fields.iter().find(|f| &f.0 == name) {
                                None => why = Some(format!("the struct has no field `{name}`")),
                                Some((_, ty)) if !fits(&mty, ty) => {
                                    why = Some(format!(
                                    "field `{name}` is `{ty}` in the code, `{mty}` in the model"
                                ))
                                }
                                Some(_) => {}
                            }
                            if why.is_some() {
                                break;
                            }
                        }
                        let stated = |f: &&(String, String)| r.fields.iter().any(|m| m.0 == f.0);
                        for (name, _) in fields.iter().filter(|f| !stated(f) && !r.open) {
                            erased.push(("field", name.clone()));
                        }
                    }
                }
                let compared =
                    alone && s.kind == "struct" && s.fields.is_some() && record.is_some();
                match &s.variants {
                    _ if compared => {}
                    None => why = Some(format!("the code is a `{}`, not an enum", s.kind)),
                    Some(_) if !e.params.is_empty() => {
                        why = Some("an enum with type parameters is not compared".to_string())
                    }
                    Some(code) => {
                        for v in &e.variants {
                            let of = |r: &&mitra_lang::Record| {
                                r.name == *v && r.of.as_deref() == Some(e.name.as_str())
                            };
                            let Some(r) = model.records.iter().find(of) else {
                                continue;
                            };
                            let stated: Vec<&(String, mitra_lang::Set)> =
                                r.fields.iter().filter(|f| f.0 != "..").collect();
                            let name = last(v);
                            match code.iter().find(|c| c.0 == name) {
                                None => why = Some(format!("the enum has no variant `{name}`")),
                                Some((_, held)) if held.len() != stated.len() => {
                                    why = Some(format!(
                                "variant `{name}` holds {} value(s) in the code, {} in the model",
                                held.len(),
                                stated.len()
                            ))
                                }
                                Some((_, held)) => {
                                    for ((field, set), ty) in stated.iter().zip(held) {
                                        let mty = model.named(&set.to_string());
                                        if !fits(&mty, ty) && why.is_none() {
                                            why = Some(format!(
                                            "`{field}` of variant `{name}` is `{ty}` in the code, \
                                             `{mty}` in the model"
                                        ));
                                        }
                                    }
                                }
                            }
                            if why.is_some() {
                                break;
                            }
                        }
                        let known =
                            |c: &&(String, Vec<String>)| e.variants.iter().any(|v| last(v) == c.0);
                        if let Some((name, _)) =
                            code.iter().find(|c| !known(c)).filter(|_| why.is_none())
                        {
                            why = Some(format!(
                                "the enum has a variant `{name}` the model does not state"
                            ));
                        }
                    }
                }
                say(
                    &mut found,
                    &format!("{}::tied", e.name),
                    &s.place,
                    why,
                    &erased,
                );
                continue;
            }
            let record = model.records.iter().find(|r| r.name == s.implements);
            let Some(r) = record.filter(|r| r.of.is_none()) else {
                let msg = format!("{}: the model has no record `{}`", s.place, s.implements);
                return Err(msg);
            };
            if shapes
                .iter()
                .filter(|d| d.implements == s.implements)
                .count()
                > 1
            {
                let msg = format!("{}: `{}` is implemented twice", s.place, s.implements);
                return Err(msg);
            }
            let stated: Vec<&(String, mitra_lang::Set)> =
                r.fields.iter().filter(|f| f.0 != "..").collect();
            let mut why = None;
            let mut erased: Erased = Vec::new();
            match &s.fields {
                // Nothing of it is stated: any struct or enum will do.
                _ if stated.is_empty() && r.open => {}
                None => why = Some(format!("the code has no named fields (`{}`)", s.kind)),
                Some(fields) => {
                    for (name, set) in &stated {
                        let mty = model.named(&set.to_string());
                        let bare = |n: &str| n.strip_prefix("r#").unwrap_or(n).to_string();
                        match fields.iter().find(|f| bare(&f.0) == bare(name)) {
                            None => why = Some(format!("the struct has no field `{name}`")),
                            Some((_, ty)) if !fits(&mty, ty) => {
                                why = Some(format!(
                                    "field `{name}` is `{ty}` in the code, `{mty}` in the model"
                                ))
                            }
                            Some(_) => {}
                        }
                        if why.is_some() {
                            break;
                        }
                    }
                    let bare = |n: &str| n.strip_prefix("r#").unwrap_or(n).to_string();
                    let extra = fields
                        .iter()
                        .filter(|f| !stated.iter().any(|m| bare(&m.0) == bare(&f.0)));
                    for (name, _) in extra.filter(|_| !r.open) {
                        erased.push(("field", name.clone()));
                    }
                }
            }
            say(
                &mut found,
                &format!("{}::tied", r.name),
                &s.place,
                why,
                &erased,
            );
        }
        for c in &code {
            let Some(f) = model
                .declared_function(&c.implements)
                .filter(|f| !f.is_spec())
            else {
                let msg = format!("{}: the model has no function `{}`", c.place, c.implements);
                return Err(msg);
            };
            if code.iter().filter(|d| d.implements == c.implements).count() > 1 {
                return Err(format!(
                    "{}: `{}` is implemented twice",
                    c.place, c.implements
                ));
            }
            // Inputs and result.
            let mut why = None;
            if c.params.len() != f.params.len() {
                let (n, m) = (c.params.len(), f.params.len());
                why = Some(format!("{n} input(s) in the code, {m} in the model"));
            }
            for ((name, ty), (mname, mty)) in c.params.iter().zip(&f.params) {
                let mty = model.named(&mty.to_string());
                if why.is_some() {
                    break;
                }
                // `r#match` and `match` are one name.
                let bare = |n: &str| n.strip_prefix("r#").unwrap_or(n).to_string();
                if bare(name) != bare(mname) && name != "self" {
                    why = Some(format!(
                        "input `{name}` in the code is `{mname}` in the model"
                    ));
                } else if let Some(ty) = ty.as_ref().filter(|ty| !fits(&mty, ty)) {
                    why = Some(format!(
                        "input `{name}` is `{ty}` in the code, `{mty}` in the model"
                    ));
                }
            }
            // An input of an opaque type, a type parameter or `impl
            // Contract`: the traits that bound it in the code, on its type
            // parameter or as `impl Trait`, are the contracts of the model.
            // Bounds that say nothing of a value are set aside, and said.
            let mut erased: Erased = Vec::new();
            let inside = model.generics.iter().find(|g| g.function == f.name);
            for (input, set) in inside.iter().flat_map(|g| &g.params) {
                let known = inside.and_then(|g| g.known.iter().find(|k| &k.0 == set));
                let (Some((_, contracts)), Some((_, Some(ty)))) =
                    (known, c.params.iter().find(|p| &p.0 == input))
                else {
                    continue;
                };
                let mut t = ty.trim();
                for prefix in ["&", "mut "] {
                    t = t.strip_prefix(prefix).unwrap_or(t).trim_start();
                }
                let direct = t.strip_prefix("impl ").or(t.strip_prefix("dyn "));
                let parameter = c.bounds.iter().any(|p| p.0 == t);
                let all: Vec<String> = match direct {
                    Some(d) => traits(d),
                    None if parameter => {
                        let of = c.bounds.iter().filter(|p| p.0 == t);
                        of.flat_map(|p| p.1.clone()).collect()
                    }
                    // A type of its own in the code: nothing to compare.
                    None => continue,
                };
                // A bound the model does not state is a detail of the code:
                // said with the tie, not a difference.
                let (code, more): (Vec<String>, Vec<String>) =
                    all.into_iter().partition(|t| contracts.contains(t));
                for t in &more {
                    erased.push(("bound", format!("{input}: {t}")));
                }
                if let Some(missing) = contracts.iter().find(|m| !code.contains(m)) {
                    let by = match more.is_empty() {
                        true => "nothing".to_string(),
                        false => format!("`{}`", more.join(" + ")),
                    };
                    why = why.or(Some(format!(
                        "input `{input}` is bound by `{missing}` in the model, by {by} in the code"
                    )));
                }
            }
            let range = model.named(&f.range.to_string());
            // The type of an error is the code's: what fails is compared.
            let same_ok = match (result_of(&range), result_of(&c.result)) {
                (Some((ok, err)), Some((rust_ok, rust_err))) => {
                    let named = |e: &Option<String>| e.as_ref().map(|e| as_model(e));
                    let differs = named(&rust_err) != err.as_ref().map(|e| e.replace(' ', ""));
                    (fits(&ok, &rust_ok) && differs).then(|| rust_err.unwrap_or_default())
                }
                _ => None,
            };
            if let Some(err) = &same_ok {
                erased.push(("error", format!("result: {err}")));
            }
            if why.is_none() && same_ok.is_none() && !fits(&range, &c.result) {
                let r = &c.result;
                why = Some(format!(
                    "the result is `{r}` in the code, `{range}` in the model"
                ));
            }
            // What was forgotten of each type to read it as the model's.
            // A type that names a type parameter, or is `impl Trait`, is
            // open in the code as it is in the model.
            let word = |c: char| c.is_alphanumeric() || c == '_';
            let open = |ty: &str| {
                let named = |p: &(String, Vec<String>)| ty.split(|c| !word(c)).any(|w| w == p.0);
                c.bounds.iter().any(named) || ty.contains("impl ") || ty.contains("dyn ")
            };
            let mut numbers = range.contains("Int") || range.contains("Float");
            for ((name, ty), (_, mty)) in c.params.iter().zip(&f.params) {
                let mty = model.named(&mty.to_string());
                numbers |= mty.contains("Int") || mty.contains("Float") || mty.contains("..");
                if let Some(ty) = ty {
                    forgot(name, ty, &mty, open(ty), &mut erased);
                }
            }
            if !["", "()"].contains(&c.result.trim()) {
                forgot("result", &c.result, &range, open(&c.result), &mut erased);
            }
            if numbers {
                erased.push(("width", String::new()));
            }
            say(
                &mut found,
                &format!("{}::tied", f.name),
                &c.place,
                why,
                &erased,
            );
            // Calls.
            let uses = model.claims.iter().find_map(|k| match &k.target {
                Target::Uses { used, only } if k.func == f.name => Some((k, used, *only)),
                _ => None,
            });
            let Some((claim, used, only)) = uses else {
                // No `uses`, written or implied by a body: it calls nothing
                // outside its own subject.
                let block = |name: &str| name.split_once("::").map_or("", |p| p.0).to_string();
                let file =
                    |place: &str| place.rsplit_once(':').map_or("", |p| p.0).to_string() + ":";
                let home = block(&f.name);
                let own = |d: &Code| {
                    code.iter()
                        .any(|e| e.name == d.name && block(&e.implements) == home)
                };
                // Only what the text settles: a method through a dot may be
                // any type's, and a name the file defines is its own.
                // The type a function is written in, `Modules` for
                // `modules::Modules::new`: it is then called as `Modules::new(`,
                // with or without `self`.
                let of_type = |d: &Code| {
                    let of = d.implements.rsplit("::").nth(1).unwrap_or("");
                    of.starts_with(char::is_uppercase).then(|| of.to_string())
                };
                let plainly = |d: &Code| match of_type(d) {
                    Some(of) => c.body.contains(&format!("{of}::{}(", d.name)),
                    None => {
                        // Its own: a function of the file, or a `let` of the body.
                        let bound = c.body.contains(&format!("let {} =", d.name));
                        let beside = c.beside.contains(&d.name);
                        let local = bound || (beside && !d.place.starts_with(&file(&c.place)));
                        !local && calls(&c.body.replace(&format!(".{}", d.name), ""), &d.name)
                    }
                };
                let outside = code
                    .iter()
                    .find(|d| block(&d.implements) != home && !own(d) && plainly(d));
                if let Some(d) = outside {
                    let why = format!(
                        "`{}` calls `{}`, which is `{}`, and the model states no `uses`",
                        c.name, d.name, d.implements
                    );
                    say(
                        &mut found,
                        &format!("{}::uses", f.name),
                        &c.place,
                        Some(why),
                        &Vec::new(),
                    );
                }
                continue;
            };
            let code_of = |g: &str| code.iter().find(|d| d.implements == g);
            let mut why = None;
            let mut checked = false;
            for g in used {
                // A function of a contract is nobody's code here: it is
                // called by its name, on whatever fills the contract.
                let of_contract = model
                    .block_of(g)
                    .is_some_and(|b| b.side == mitra_lang::Side::Contract);
                let name = match code_of(g) {
                    Some(d) => d.name.clone(),
                    None if of_contract => g.rsplit("::").next().unwrap_or(g).to_string(),
                    None => continue,
                };
                checked = true;
                if !calls(&c.body, &name) {
                    why = Some(format!("`{}` does not call `{name}`", c.name));
                }
            }
            if only {
                checked = true;
                for d in code.iter().filter(|d| d.implements != f.name) {
                    // Another function of the same name may be the one called.
                    let same = |e: &&Code| e.name == d.name && used.contains(&e.implements);
                    // A function written in a type is called as `Type::name(`:
                    // through a dot it may be any type's.
                    let of = d.implements.rsplit("::").nth(1).unwrap_or("");
                    let called = match of.starts_with(char::is_uppercase) {
                        true => c.body.contains(&format!("{of}::{}(", d.name)),
                        false => calls(&c.body, &d.name),
                    };
                    if called && !code.iter().any(|e| same(&e)) && why.is_none() {
                        let (n, g) = (&c.name, &d.implements);
                        why = Some(format!("`{n}` also calls `{}`, which is `{g}`", d.name));
                    }
                }
            }
            if checked {
                say(&mut found, &claim.name, &c.place, why, &Vec::new());
            }
        }
        // What has no tie.
        let untied = |f: &&mitra_lang::Function| {
            !f.is_spec() && !code.iter().any(|c| c.implements == f.name)
        };
        // The text of a requirement is no code.
        let untied = |f: &&mitra_lang::Function| untied(f) && !f.name.ends_with("::prose");
        let names: Vec<&str> = model
            .functions
            .iter()
            .filter(untied)
            .map(|f| f.name.as_str())
            .collect();
        let mut names: Vec<&str> = names;
        let loose = |r: &&mitra_lang::Record| {
            r.of.is_none() && !shapes.iter().any(|s| s.implements == r.name)
        };
        // What another store declares is tied there.
        // A contract is nobody's code.
        let here = |n: &&str| {
            let block = model.block_of(n);
            block.is_none_or(|b| b.from.is_none() && b.side != mitra_lang::Side::Contract)
        };
        names.extend(model.records.iter().filter(loose).map(|r| r.name.as_str()));
        let loose = |e: &&mitra_lang::Enum| {
            let record = e.variants.len() == 1 && e.variants[0] == e.name;
            (e.params.is_empty() || record) && !shapes.iter().any(|s| s.implements == e.name)
        };
        names.extend(model.enums.iter().filter(loose).map(|e| e.name.as_str()));
        let names: Vec<&str> = names.into_iter().filter(here).collect();
        if !names.is_empty() {
            found
                .notes
                .push(format!("no code is tied to: {}", names.join(", ")));
        }
        found.untied = names.iter().map(|n| n.to_string()).collect();
        Ok(found)
    }
}
