//! The project store: a `.mitra/` folder with `model.mitra` and the
//! append-only `evidence.mitra`. The model also holds the `mitra:` comments
//! of the Rust files under `src/` and `tests/` beside `.mitra/`.
//!
//! Evidence is only ever appended, one line per piece, so two branches that
//! both add evidence merge as the union of their lines: `.mitra/.gitattributes`
//! asks git for the `union` merge driver on the evidence file.

use std::path::{Path, PathBuf};

use crate::source::Source;

pub const DIR: &str = ".mitra";

pub struct Store {
    dir: PathBuf,
}

const MODEL_STUB: &str = "-- The model: subjects (opaque), specs (def), claims and tests.\n\
-- See docs/reference/language.md.\n";
const EVIDENCE_STUB: &str = "-- Evidence, append-only: `mitra add` writes here.\n";
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

    pub fn model_path(&self) -> String {
        self.dir.join("model.mitra").display().to_string()
    }

    /// `model.mitra`, then the `mitra:` lines of the Rust sources.
    pub fn source(&self) -> Result<Source, String> {
        let mut s = Source::file(&self.model_path())?;
        // The other `.mitra` files of the store are model too: a model
        // kept aside from the code may be long.
        let mut more: Vec<PathBuf> = std::fs::read_dir(&self.dir)
            .map_err(|e| format!("{}: {e}", self.dir.display()))?
            .flatten()
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|x| x == "mitra"))
            .filter(|p| !p.ends_with("model.mitra") && !p.ends_with("evidence.mitra"))
            .collect();
        more.sort();
        for p in more {
            s.add_file(&p.display().to_string())?;
        }
        let root = self.dir.parent().unwrap_or(Path::new("."));
        s.add_rust(&[root.join("src"), root.join("tests")])?;
        Ok(s)
    }

    pub fn evidence_path(&self) -> String {
        self.dir.join("evidence.mitra").display().to_string()
    }

    /// Parse evidence text, then [`Store::append`] it.
    pub fn add(
        &self,
        text: &str,
        source: Option<String>,
        rev: Option<String>,
    ) -> Result<usize, String> {
        // The model's records, so that a record's name can stand for its values.
        let mut new = mitra_core::read_evidence(&self.source()?.text, text)
            .map_err(|err| format!("evidence: {err}"))?;
        if new.is_empty() {
            return Err("no evidence given".into());
        }
        for v in source.iter().chain(&rev) {
            if v.contains(['"', '\n', '\r']) {
                return Err(format!("`{v}`: quotes and line breaks are not allowed"));
            }
        }
        for ev in &mut new {
            for (flag, given, attr) in [
                ("--source", &source, &mut ev.source),
                ("--rev", &rev, &mut ev.rev),
            ] {
                if given.is_some() {
                    if attr.is_some() {
                        return Err(format!(
                            "{flag} is given twice: in the option and in the evidence"
                        ));
                    }
                    *attr = given.clone();
                }
            }
        }
        self.append(&new)?;
        Ok(new.len())
    }

    /// The store's model, checked against its evidence.
    pub fn model(&self) -> Result<mitra_core::Model, String> {
        let (m, e) = (self.source()?, self.evidence_path());
        let old = std::fs::read_to_string(&e).map_err(|err| format!("{e}: {err}"))?;
        mitra_core::load(&m.text, &old).map_err(|(input, err)| match input {
            mitra_core::Input::Model => m.locate(&err),
            mitra_core::Input::Evidence => format!("{e}: {err}"),
        })
    }

    /// Check new evidence against the model and the evidence already
    /// there, evaluate, then append it, normalised. Nothing is written on an
    /// error. Errors in the store name its file; errors in the new evidence
    /// are numbered from its first line.
    pub fn append(&self, new: &[mitra_core::Evidence]) -> Result<(), String> {
        let (m, e) = (self.source()?, self.evidence_path());
        let old = std::fs::read_to_string(&e).map_err(|err| format!("{e}: {err}"))?;
        let model = &m.text;
        let located = |input, err: mitra_core::Error| match input {
            mitra_core::Input::Model => m.locate(&err),
            mitra_core::Input::Evidence => format!("{e}: {err}"),
        };
        // The store must be sound before anything is added to it.
        let before = mitra_core::load(model, &old).map_err(|(i, err)| located(i, err))?;
        mitra_core::evaluate(&before).map_err(|(i, err)| located(i, err))?;
        let sep = if old.is_empty() || old.ends_with('\n') {
            ""
        } else {
            "\n"
        };
        let printed = mitra_core::print_evidence(new, Some(&before));
        let offset = format!("{old}{sep}").lines().count();
        let blame = |input, err: mitra_core::Error| match input {
            mitra_core::Input::Evidence if err.line > offset => format!(
                "evidence to add: line {}: {}",
                err.line - offset,
                err.message
            ),
            i => located(i, err),
        };
        let all = format!("{old}{sep}{printed}");
        let after = mitra_core::load(model, &all).map_err(|(i, err)| blame(i, err))?;
        mitra_core::evaluate(&after).map_err(|(i, err)| blame(i, err))?;
        use std::io::Write;
        std::fs::OpenOptions::new()
            .append(true)
            .open(&e)
            .and_then(|mut f| f.write_all(format!("{sep}{printed}").as_bytes()))
            .map_err(|err| format!("{e}: {err}"))
    }
}
