//! The project store: a `.mitra/` folder with `model.mitra` and the
//! ledger `evidence.mitra`. Every other `.mitra` file of the folder is
//! model too.
//!
//! The ledger is not append-only. What a machine observed is written
//! again by each run of that machine; what a person said stays until
//! `mitra remove` takes it out, and what left is always given back to be
//! shown. `.mitra/.gitattributes` asks git for the `union` merge driver
//! on the evidence file, so two branches that both add evidence merge.

use std::path::{Path, PathBuf};

use crate::source;
use mitra_lang::Modules;

pub const DIR: &str = ".mitra";

/// What a run of a machine put in place of its earlier evidence.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements keeping::Replaced
pub struct Replaced {
    /// How many entries of that machine left the ledger.
    pub before: usize,
    /// The revisions they were observed on.
    pub revs: Vec<String>,
    /// How many were written.
    pub written: usize,
    /// The entries that left with no entry of the same source on the
    /// same piece written in their place, as they were written.
    pub gone: Vec<String>,
}

/// The entries `mitra remove` takes out.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements keeping::Which
pub enum Which {
    /// On a piece the model no longer has.
    Orphaned,
    /// Said before its piece changed: all of them, or those on one piece.
    Stale(Option<String>),
    /// From one source, as written: `review:me`.
    Source(String),
    /// On one piece.
    Piece(String),
}

/// An entry as it is written in the ledger.
fn said(ledger: &str, e: &mitra_lang::parse::Entry) -> String {
    let lines: Vec<&str> = ledger.lines().collect();
    let last = e.last.min(lines.len());
    lines[e.first.saturating_sub(1)..last]
        .join("\n")
        .trim()
        .to_string()
}

// mitra: implements store::Store
pub struct Store {
    dir: PathBuf,
}

const MODEL_STUB: &str = "// The model: subjects, boundaries, claims.\n\
// See docs/reference/language.md.\n";
const EVIDENCE_STUB: &str =
    "// Evidence: `mitra add`, `mitra ingest` and `mitra tie` write here, `mitra remove` takes out.\n";
const GITATTRIBUTES: &str = "evidence.mitra merge=union\n";

impl Store {
    /// The nearest `.mitra/` at or above the current folder.
    pub fn find() -> Result<Store, String> {
        let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
        let dir = cwd
            .ancestors()
            .map(|d| d.join(DIR))
            .find(|d| d.exists())
            .ok_or_else(|| format!("no {DIR}/ folder here or above: run `mitra init`"))?;
        if !dir.is_dir() {
            return Err(format!("{} is not a folder", dir.display()));
        }
        Ok(Store { dir })
    }

    /// The store that `model` is the model file of, if it is one: a
    /// `model.mitra` inside a `.mitra/` folder.
    // mitra: implements store::Store::of
    pub fn of(model: &str) -> Option<Store> {
        let path = Path::new(model);
        let dir = path.parent()?;
        let store = path.file_name()? == "model.mitra" && dir.file_name()? == DIR;
        store.then(|| Store { dir: dir.into() })
    }

    /// Create `.mitra/` in `root`. Never overwrites an existing store.
    pub fn init(root: &Path) -> Result<PathBuf, String> {
        let dir = root.join(DIR);
        if dir.exists() {
            return Err(format!("{} already exists", dir.display()));
        }
        let write = |name: &str, text: &str| {
            std::fs::write(dir.join(name), text).map_err(|e| format!("{name}: {e}"))
        };
        std::fs::create_dir(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
        write("model.mitra", MODEL_STUB)?;
        write("evidence.mitra", EVIDENCE_STUB)?;
        write(".gitattributes", GITATTRIBUTES)?;
        Ok(dir)
    }

    /// The folder the store is in.
    pub fn root(&self) -> PathBuf {
        self.dir.parent().unwrap_or(Path::new(".")).to_path_buf()
    }

    // mitra: implements store::Store::model_path
    pub fn model_path(&self) -> String {
        self.dir.join("model.mitra").display().to_string()
    }

    /// The model of the store: `model.mitra`, then the other `.mitra`
    /// files beside it, the ledger aside. A model kept aside from the
    /// code may be long, and is one file per subject.
    // mitra: implements store::Store::source
    pub fn source(&self) -> Result<Modules, String> {
        let mut more: Vec<PathBuf> = std::fs::read_dir(&self.dir)
            .map_err(|e| format!("{}: {e}", self.dir.display()))?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "mitra"))
            .filter(|p| !p.ends_with("model.mitra") && !p.ends_with("evidence.mitra"))
            .collect();
        more.sort();
        more.insert(0, PathBuf::from(self.model_path()));
        source::files(&more)
    }

    // mitra: implements store::Store::evidence_path
    pub fn evidence_path(&self) -> String {
        self.dir.join("evidence.mitra").display().to_string()
    }

    /// Evidence blocks, with `--source` and `--rev` written into each,
    /// then [`Store::append`]ed as they were given.
    // mitra: implements ledger::Store::add
    pub fn add(
        &self,
        text: &str,
        source: Option<String>,
        rev: Option<String>,
    ) -> Result<usize, String> {
        for v in source.iter().chain(&rev) {
            if v.contains(['"', '\n', '\r']) {
                return Err(format!("`{v}`: quotes and line breaks are not allowed"));
            }
        }
        let text = self.stamped(text, source, rev)?;
        let n = mitra_lang::parse::entries(&text).len();
        self.append(text.trim())?;
        Ok(n)
    }

    /// New evidence with `--source` and `--rev` written into each block,
    /// and, in what a person says, the fingerprint of its piece as it
    /// reads now (`seen`): the day the piece says something else, the
    /// entry is stale.
    fn stamped(
        &self,
        text: &str,
        source: Option<String>,
        rev: Option<String>,
    ) -> Result<String, String> {
        let text = mitra_lang::parse::stamp(text, source.as_deref(), rev.as_deref(), &[])?;
        let model = self.source()?;
        let new = mitra_lang::read_evidence(&model, &text)
            .map_err(|err| format!("evidence to add: {err}"))?;
        if new.is_empty() {
            return Err("no evidence given".into());
        }
        if new.iter().all(|e| e.by_machine() || e.seen.is_some()) {
            return Ok(text);
        }
        let read = mitra_lang::load_modules(&model, "").map_err(|(_, e)| model.locate(&e))?;
        let prints = mitra_lang::fingerprint::all(&read);
        let seen = |e: &mitra_lang::Evidence| match e.by_machine() || e.seen.is_some() {
            true => None,
            false => mitra_lang::fingerprint::of(&prints, &e.about).cloned(),
        };
        let seen: Vec<Option<String>> = new.iter().map(seen).collect();
        mitra_lang::parse::stamp(&text, None, None, &seen)
    }

    /// The ledger without some of its entries, each given by its lines.
    fn without(old: &str, gone: &[mitra_lang::parse::Entry]) -> Result<String, String> {
        let all = mitra_lang::parse::entries(old);
        let out = |n: usize| gone.iter().any(|g| g.first <= n && n <= g.last);
        // A line holds one entry, or the removal would take a neighbour.
        let shared = all
            .iter()
            .find(|e| !gone.contains(e) && (out(e.first) || out(e.last)));
        if let Some(e) = shared {
            return Err(format!(
                "line {}: two evidence blocks share a line, write one per line",
                e.first
            ));
        }
        let kept = old.lines().enumerate().filter(|(i, _)| !out(i + 1));
        Ok(kept.map(|(_, l)| format!("{l}\n")).collect())
    }

    /// Write the whole ledger, once it reads and evaluates with the
    /// model: on an error nothing is written.
    fn rewrite(&self, text: &str) -> Result<(), String> {
        let (m, e) = (self.source()?, self.evidence_path());
        let located = |(input, err): (mitra_lang::Input, mitra_lang::Error)| match input {
            mitra_lang::Input::Model => m.locate(&err),
            mitra_lang::Input::Evidence => format!("{e}: {err}"),
        };
        let after = mitra_lang::load_modules(&m, text).map_err(located)?;
        mitra_lang::evaluate(&after).map_err(located)?;
        let tmp = format!("{e}.tmp");
        std::fs::write(&tmp, text)
            .and_then(|_| std::fs::rename(&tmp, &e))
            .map_err(|err| format!("{e}: {err}"))
    }

    /// Write the files of a build under `.mitra/build/`, each by its name,
    /// in place of what was there. Gives the folder.
    // mitra: implements store::Store::write_build
    pub fn write_build(&self, files: &[(String, String)]) -> Result<String, String> {
        let dir = self.dir.join("build");
        let said = |e: std::io::Error| format!("{}: {e}", dir.display());
        std::fs::create_dir_all(&dir).map_err(said)?;
        for (name, text) in files {
            std::fs::write(dir.join(name), text).map_err(said)?;
        }
        Ok(dir.display().to_string())
    }

    /// What a machine observed is a snapshot: every entry whose source
    /// starts with `prefix` (`test:`, `reader:`) leaves the ledger and
    /// `new` is written in its place. What left is given back, to be
    /// shown.
    // mitra: implements keeping::Store::replace
    pub fn replace(&self, prefix: &str, new: &str) -> Result<Replaced, String> {
        let e = self.evidence_path();
        let old = std::fs::read_to_string(&e).map_err(|err| format!("{e}: {err}"))?;
        let from = |x: &mitra_lang::parse::Entry| {
            x.source.as_deref().is_some_and(|s| s.starts_with(prefix))
        };
        let before: Vec<_> = mitra_lang::parse::entries(&old)
            .into_iter()
            .filter(from)
            .collect();
        let new = match new.trim().is_empty() {
            true => String::new(),
            false => self.stamped(new, None, None)?,
        };
        let after = mitra_lang::parse::entries(&new);
        let mut text = Store::without(&old, &before)?;
        if !new.trim().is_empty() {
            text += new.trim();
            text.push('\n');
        }
        self.rewrite(&text)?;
        // The same observer on the same piece: `test:name`, or `reader:rust`
        // whatever the line it read.
        let who = |x: &mitra_lang::parse::Entry| {
            let source = x.source.as_deref().unwrap_or_default();
            source.split(' ').next().unwrap_or_default().to_string()
        };
        let same = |a: &mitra_lang::parse::Entry, b: &mitra_lang::parse::Entry| {
            a.about == b.about && who(a) == who(b)
        };
        let gone = |b: &&mitra_lang::parse::Entry| !after.iter().any(|a| same(a, b));
        Ok(Replaced {
            gone: before.iter().filter(gone).map(|x| said(&old, x)).collect(),
            revs: {
                let mut revs: Vec<String> = before.iter().filter_map(|x| x.rev.clone()).collect();
                revs.sort();
                revs.dedup();
                revs
            },
            before: before.len(),
            written: after.len(),
        })
    }

    /// Take entries out of the ledger: the only way what a person said
    /// leaves it. What left is given back as written, to be shown.
    // mitra: implements keeping::Store::remove
    pub fn remove(&self, which: &Which) -> Result<Vec<String>, String> {
        let e = self.evidence_path();
        let old = std::fs::read_to_string(&e).map_err(|err| format!("{e}: {err}"))?;
        let model = self.model()?;
        let aside = |x: &mitra_lang::parse::Entry| model.aside.iter().find(|a| a.line == x.first);
        let named = |x: &mitra_lang::parse::Entry, piece: &str| {
            x.about == piece || x.about.ends_with(&format!("::{piece}"))
        };
        let pick = |x: &mitra_lang::parse::Entry| match which {
            Which::Orphaned => aside(x).is_some_and(|a| a.why == mitra_lang::Why::Orphaned),
            Which::Stale(piece) => {
                let stale = aside(x).is_some_and(|a| a.why != mitra_lang::Why::Orphaned);
                stale && piece.as_deref().is_none_or(|p| named(x, p))
            }
            Which::Source(s) => x.source.as_deref() == Some(s),
            Which::Piece(p) => named(x, p),
        };
        let gone: Vec<_> = mitra_lang::parse::entries(&old)
            .into_iter()
            .filter(pick)
            .collect();
        if gone.is_empty() {
            return Ok(Vec::new());
        }
        self.rewrite(&Store::without(&old, &gone)?)?;
        Ok(gone.iter().map(|x| said(&old, x)).collect())
    }

    /// The store's model, checked against its evidence.
    pub fn model(&self) -> Result<mitra_lang::Model, String> {
        let (m, e) = (self.source()?, self.evidence_path());
        let old = std::fs::read_to_string(&e).map_err(|err| format!("{e}: {err}"))?;
        mitra_lang::load_modules(&m, &old).map_err(|(input, err)| match input {
            mitra_lang::Input::Model => m.locate(&err),
            mitra_lang::Input::Evidence => format!("{e}: {err}"),
        })
    }

    /// Check new evidence against the model and the evidence already
    /// there, evaluate, then append it as written. Nothing is written on an
    /// error. Errors in the store name its file; errors in the new evidence
    /// are numbered from its first line.
    // mitra: implements ledger::Store::append
    pub fn append(&self, new: &str) -> Result<(), String> {
        let (m, e) = (self.source()?, self.evidence_path());
        let old = std::fs::read_to_string(&e).map_err(|err| format!("{e}: {err}"))?;
        let model = &m;
        let located = |input, err: mitra_lang::Error| match input {
            mitra_lang::Input::Model => m.locate(&err),
            mitra_lang::Input::Evidence => format!("{e}: {err}"),
        };
        // The store must be sound before anything is added to it.
        let before = mitra_lang::load_modules(model, &old).map_err(|(i, err)| located(i, err))?;
        mitra_lang::evaluate(&before).map_err(|(i, err)| located(i, err))?;
        let sep = if old.is_empty() || old.ends_with('\n') {
            ""
        } else {
            "\n"
        };
        let printed = format!("{new}\n");
        let offset = format!("{old}{sep}").lines().count();
        let blame = |input, err: mitra_lang::Error| match input {
            mitra_lang::Input::Evidence if err.line > offset => format!(
                "evidence to add: line {}: {}",
                err.line - offset,
                err.message
            ),
            i => located(i, err),
        };
        let all = format!("{old}{sep}{printed}");
        let after = mitra_lang::load_modules(model, &all).map_err(|(i, err)| blame(i, err))?;
        mitra_lang::evaluate(&after).map_err(|(i, err)| blame(i, err))?;
        use std::io::Write;
        std::fs::OpenOptions::new()
            .append(true)
            .open(&e)
            .and_then(|mut f| f.write_all(format!("{sep}{printed}").as_bytes()))
            .map_err(|err| format!("{e}: {err}"))
    }
}
