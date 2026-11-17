//! The project store: a `.mitra/` folder with `model.mitra` and the
//! append-only `evidence.mitra`.
//!
//! Evidence is only ever appended, one line per piece, so two branches that
//! both add evidence merge as the union of their lines: `.mitra/.gitattributes`
//! asks git for the `union` merge driver on the evidence file.

use std::path::{Path, PathBuf};

pub const DIR: &str = ".mitra";

pub struct Store {
    dir: PathBuf,
}

const MODEL_STUB: &str =
    "-- The model: subjects (opaque), specs (def), claims, profiles, policies.\n\
-- See docs/reference/language.md.\n";
const EVIDENCE_STUB: &str = "-- Evidence, append-only: `mitra add` writes here.\n";
const GITATTRIBUTES: &str = "evidence.mitra merge=union\n";

impl Store {
    /// The nearest `.mitra/` at or above the current folder.
    pub fn find() -> Result<Store, String> {
        let cwd = std::env::current_dir().map_err(|e| e.to_string())?;
        cwd.ancestors()
            .map(|d| d.join(DIR))
            .find(|d| d.is_dir())
            .map(|dir| Store { dir })
            .ok_or_else(|| format!("no {DIR}/ folder here or above: run `mitra init`"))
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

    pub fn model_path(&self) -> String {
        self.dir.join("model.mitra").display().to_string()
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
        let mut new = mitra_core::parse_evidence(text).map_err(|err| format!("evidence: {err}"))?;
        if new.is_empty() {
            return Err("no evidence given".into());
        }
        for v in source.iter().chain(&rev) {
            if v.contains(['"', '\n', '\r']) {
                return Err(format!("`{v}`: quotes and line breaks are not allowed"));
            }
        }
        for ev in &mut new {
            ev.source = source.clone().or(ev.source.take());
            ev.rev = rev.clone().or(ev.rev.take());
        }
        self.append(&new)?;
        Ok(new.len())
    }

    /// The store's model, checked against its evidence.
    pub fn model(&self) -> Result<mitra_core::Model, String> {
        let (m, e) = (self.model_path(), self.evidence_path());
        let read = |p: &str| std::fs::read_to_string(p).map_err(|err| format!("{p}: {err}"));
        mitra_core::load(&read(&m)?, &read(&e)?).map_err(|(input, err)| match input {
            mitra_core::Input::Model => format!("{m}: {err}"),
            mitra_core::Input::Evidence => format!("{e}: {err}"),
        })
    }

    /// Check new evidence against the model and the evidence already
    /// there, then append it, normalised. Nothing is written on an error.
    pub fn append(&self, new: &[mitra_core::Evidence]) -> Result<(), String> {
        let (m, e) = (self.model_path(), self.evidence_path());
        let read = |p: &str| std::fs::read_to_string(p).map_err(|err| format!("{p}: {err}"));
        let (model, old) = (read(&m)?, read(&e)?);
        let printed = mitra_core::print_evidence(new, None);
        let all = format!("{old}\n{printed}");
        let checked = mitra_core::load(&model, &all).map_err(|(input, err)| match input {
            mitra_core::Input::Model => format!("{m}: {err}"),
            mitra_core::Input::Evidence => format!("evidence: {}", err.message),
        })?;
        // Print with the model, so bound names follow the parameters.
        let lines = mitra_core::print_evidence(new, Some(&checked));
        let sep = if old.is_empty() || old.ends_with('\n') {
            ""
        } else {
            "\n"
        };
        use std::io::Write;
        std::fs::OpenOptions::new()
            .append(true)
            .open(&e)
            .and_then(|mut f| f.write_all(format!("{sep}{lines}").as_bytes()))
            .map_err(|err| format!("{e}: {err}"))
    }
}
