//! The `mitra` command line: a wrapper over mitra-lang, with
//! mitra-rust to read Rust code and test reports.
//!
//! ```text
//! mitra init
//! mitra build | conform | assess [<report>...] | diagnose '<evidence>' [<report>...]
//! mitra check [<model> <evidence>] [--rev <commit>]
//! mitra eval  [<model> <evidence>] [--rev <commit>] [--candidates <n>] [--threshold <p>] [--all] [-o <result.txt>]
//! mitra add   '<evidence>' [--source <who>] [--rev <commit>]
//! mitra next  [<model> <evidence>] [--rev <commit>]
//! mitra ingest <report>... [--format libtest|junit] [--rev <commit>]
//! mitra remove --orphaned | --stale [<piece>] | --source <who> | <piece>
//! ```
//!
//! Without paths, commands use the project store: the nearest `model/`
//! folder at or above the current one, holding `model.mitra` and the
//! ledger `evidence.mitra`. The store's model also holds the `mitra:`
//! comments of the Rust files under `src/` and `tests/` next to it.

use mitra_lang::Reader;

use mitra_lang::Modules as Source;
use mitra_store::{source, store};

use std::path::Path;
use std::process::ExitCode;

/// What `mitra build` found.
struct Built {
    errors: usize,
    model: mitra_lang::Model,
    built: mitra_lang::build::Built,
}

/// The model alone: it reads, nothing in it contradicts anything else,
/// and it is compiled under `.mitra/build/`. Nothing on what is owed.
fn build(s: &store::Store, display: &mitra_lang::Display) -> Result<Built, String> {
    let folder = Path::new(&s.model_path())
        .parent()
        .map(|d| d.display().to_string());
    if let Some(path) = source::round(&folder.unwrap_or_default()) {
        return Err(format!(
            "build: stores import each other: {}. What a store takes from another is assumed there, so a promise would rest on itself",
            path.join(" imports ")
        ));
    }
    let source = s.source()?;
    let (model, report) = evaluate(&source, "", None)?;
    let places = Places {
        model: &source,
        evidence: "",
    };
    let profile = reader().erasures();
    let named = |k: &&String| !profile.iter().any(|p| p.0 == k.as_str());
    if let Some(name) = model.settings.erasure.keys().find(named) {
        let known: Vec<&str> = profile.iter().map(|p| p.0).collect();
        return Err(format!(
            "settings: no erasure `{name}` in the profile of the reader ({})",
            known.join(", ")
        ));
    }
    let built = mitra_lang::build::build(&model);
    let errors = report.problems.len();
    print!(
        "{}",
        report
            .model_text_at(display, &places)
            .trim_start_matches('\n')
    );
    if errors > 0 {
        println!("build: {errors} errors in the model, nothing written");
        return Ok(Built {
            errors,
            model,
            built,
        });
    }
    let files = [
        ("model.lock".to_string(), built.lock()),
        (
            "interface.mitra".to_string(),
            mitra_lang::build::interface(&model),
        ),
        ("owed.txt".to_string(), mitra_lang::plan(&model, &report)),
        // The settings in force, the costs of the reader's erasures with
        // them: what the trust of a tie and of a sample is computed from.
        (
            "settings.toml".to_string(),
            model.settings.written(&profile),
        ),
    ];
    let dir = s.write_build(&files)?;
    let count = |kinds: &[&str]| kinds.iter().map(|k| built.count(k)).sum::<usize>();
    println!(
        "build: ok, model {}: functions {}, claims {}, promises {}, requires {}, blocks {}, warnings {}",
        built.model,
        count(&["function", "spec", "world"]),
        count(&["claim"]),
        count(&["ensure", "promise"]),
        count(&["require"]),
        model.blocks.len(),
        report.model_warnings()
    );
    println!(
        "  written: {}/model.lock, interface.mitra, owed.txt, settings.toml",
        relative(&dir)
    );
    Ok(Built {
        errors,
        model,
        built,
    })
}

/// The model and the code: every piece of a subject is tied, the ties
/// agree, and how much of the code the model covers. Gives 1 on drift.
// mitra: implements commands::conform
fn conform(
    s: &store::Store,
    built: &Built,
    rev: Option<&str>,
    reader: &impl Reader<Path>,
) -> Result<usize, String> {
    let model = &built.model;
    let found = reader.tie(&s.root(), model, rev)?;
    let replaced = s.replace("reader:", &found.evidence.join("\n"))?;
    let ours = |n: &&String| match model.block_of(n) {
        Some(b) => b.side == mitra_lang::Side::Subject,
        None => !model.boundaries.contains(n),
    };
    let untied: Vec<&String> = found.untied.iter().filter(ours).collect();
    let tests = reader.tests(&s.root())?;
    let pinned = tests.iter().filter(|t| !t.pinned.is_empty()).count();
    let model_lines: usize = s
        .source()?
        .files()
        .iter()
        .flat_map(|f| f.text.lines())
        .filter(|l| !l.trim().is_empty() && !l.trim().starts_with("//"))
        .count();
    for n in &found.notes {
        if !n.starts_with("no code is tied to") {
            println!("{n}");
        }
    }
    if !untied.is_empty() {
        let names: Vec<&str> = untied.iter().map(|n| n.as_str()).collect();
        println!("no code is tied to: {}", names.join(", "));
    }
    shown(&replaced);
    let drift = found.differ > 0 || !untied.is_empty();
    println!(
        "conform: {}: {} ties agree, {} differ, {} pieces untied; {} tests tied, {} pin an input",
        if drift { "drift" } else { "ok" },
        found.agree,
        found.differ,
        untied.len(),
        tests.len(),
        pinned
    );
    println!(
        "  {model_lines} lines of model over {} lines of code, in {} blocks with {} promises and {} requires",
        found.code_lines,
        model.blocks.iter().filter(|b| b.from.is_none()).count(),
        built.built.count("ensure") + built.built.count("promise"),
        built.built.count("require")
    );
    Ok(drift as usize)
}

/// The model, the code and the ledger: the report, then what is left to
/// a person and what can be handed over.
fn assess(
    s: &store::Store,
    display: &mitra_lang::Display,
    rev: Option<&str>,
    full: bool,
) -> Result<usize, String> {
    let (m, e) = (s.source()?, s.evidence_path());
    let (model, report) = evaluate(&m, &e, rev)?;
    let places = Places {
        model: &m,
        evidence: &e,
    };
    let (errors, warnings) = (report.problems.len(), report.warnings());
    // On the way to a diagnosis, only what stops it.
    if !full {
        if errors > 0 {
            for p in report.problem_texts(display, &places) {
                print!("{p}");
            }
        }
        println!("assess: {errors} errors, {warnings} warnings");
        return Ok(errors);
    }
    print!("{}", report.to_text_at(display, &places));
    let left = mitra_lang::build::levels::left(&model, &report);
    println!("\nassess: {errors} errors, {warnings} warnings");
    print!("  {}", mitra_lang::ledger::voices::summary(&model, &report));
    for (title, names) in [
        ("for a person to review or decide", &left.person),
        ("to hand over (tests to write, code to tie)", &left.delegate),
    ] {
        println!("  {title} ({}):", names.len());
        for n in names {
            println!("    {n}");
        }
    }
    Ok(errors)
}

/// An observation, not written to the ledger: what it contradicts, and
/// who is suspected.
fn diagnose(s: &store::Store, seen: &str, display: &mitra_lang::Display) -> Result<usize, String> {
    let (m, e) = (s.source()?, s.evidence_path());
    let ledger = read(&e)?;
    mitra_lang::read_evidence(&m, seen).map_err(|err| format!("the observation: {err}"))?;
    let before = mitra_lang::load_modules(&m, &ledger).map_err(located(&m, &e))?;
    let before = mitra_lang::evaluate(&before).map_err(located(&m, &e))?;
    let sep = if ledger.is_empty() || ledger.ends_with('\n') {
        ""
    } else {
        "\n"
    };
    let with = format!("{ledger}{sep}{}\n", seen.trim());
    let after = mitra_lang::load_modules(&m, &with).map_err(located(&m, &e))?;
    let after = mitra_lang::evaluate(&after).map_err(located(&m, &e))?;
    let places = Seen {
        ledger: Places {
            model: &m,
            evidence: &e,
        },
        lines: ledger.lines().count(),
    };
    let said = mitra_lang::build::levels::diagnosis(&before, &after, display, &places);
    print!("{}", said.text);
    if said.contradicts {
        println!("  to file it: mitra add '{}' --source <who>", seen.trim());
    }
    Ok(said.contradicts as usize)
}

/// ", in place of 12 test results of rev abc": what a run replaced.
fn in_place(r: &store::Replaced, what: &str) -> String {
    match (r.before, r.revs.is_empty()) {
        (0, _) => String::new(),
        (n, true) => format!(", in place of {n} {what}"),
        (n, false) => format!(", in place of {n} {what} of rev {}", r.revs.join(", ")),
    }
}

/// What left the ledger with nothing written in its place.
fn shown(r: &store::Replaced) {
    if !r.gone.is_empty() {
        println!("no longer observed, removed ({}):", r.gone.len());
        for g in &r.gone {
            println!("  {g}");
        }
    }
}

const USAGE: &str = "usage:
  mitra init
  mitra build                                  the model alone
  mitra conform  [--rev <commit>]              then the model and the code
  mitra assess   [<report>...] [--rev <commit>]  then the ledger
  mitra diagnose '<evidence>' [<report>...] [--rev <commit>]  then an observation
  mitra check [<model.mitra> [<evidence.mitra>]] [--rev <commit>]
  mitra eval  [<model.mitra> [<evidence.mitra>]] [--rev <commit>] [--candidates <n>] [--threshold <p>] [--all] [-o <result.txt>]
  mitra add   '<evidence>' [--source <who>] [--rev <commit>]
  mitra next  [<model.mitra> [<evidence.mitra>]] [--rev <commit>]
  mitra plan  [<model.mitra> [<evidence.mitra>]] [--rev <commit>]
  mitra ingest <report>... [--format libtest|junit] [--rev <commit>]
  mitra tie   [--rev <commit>]
  mitra remove --orphaned | --stale [<piece>] | --source <who> | <piece>
  mitra explain <code>
  mitra explain <claim> [<model.mitra> [<evidence.mitra>]]
  mitra explain std::claims";

fn main() -> ExitCode {
    // `mitra eval | head`: the reader closed the pipe, which is no error.
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        if info.to_string().contains("failed printing to stdout") {
            std::process::exit(0);
        }
        default(info)
    }));
    match run(std::env::args().skip(1).collect()) {
        Ok(true) => ExitCode::SUCCESS,
        // The model and the evidence were read, and `eval` found errors.
        Ok(false) => ExitCode::from(1),
        Err(msg) => {
            eprintln!("mitra: {msg}");
            ExitCode::from(2)
        }
    }
}

/// The model and the evidence file, from the arguments or from the store.
/// The path of the evidence is empty when only the model is named.
fn inputs(paths: &[String]) -> Result<(Source, String), String> {
    match paths {
        // A store's model file brings the `mitra:` lines of its Rust sources.
        [m, e] => match store::Store::of(m) {
            Some(s) => Ok((s.source()?, e.clone())),
            None => Ok((source::file(m)?, e.clone())),
        },
        // The model alone, with no evidence: what it says before any is
        // brought.
        [m] => inputs(&[m.clone(), String::new()]),
        [] => {
            let s = store::Store::find()?;
            Ok((s.source()?, s.evidence_path()))
        }
        _ => Err(USAGE.into()),
    }
}

/// The reader of the code beside a model. Rust is the one there is: this
/// is the only line that names it.
fn reader() -> impl Reader<Path> {
    mitra_rust::Rust
}

fn read(p: &str) -> Result<String, String> {
    source::read(p)
}

/// An error, prefixed with the file it belongs to.
fn located<'a>(
    m: &'a Source,
    e: &'a str,
) -> impl Fn((mitra_lang::Input, mitra_lang::Error)) -> String + 'a {
    move |(input, err)| match input {
        mitra_lang::Input::Model => m.locate(&err),
        mitra_lang::Input::Evidence => format!("{e}: {err}"),
    }
}

/// A path as seen from the current directory, when it is under it.
fn relative(path: &str) -> String {
    let here = std::env::current_dir().ok();
    let here = here.as_deref().and_then(|d| d.to_str());
    let rest = here.and_then(|d| path.strip_prefix(d));
    let rest = rest.and_then(|r| r.strip_prefix(['/', '\\']));
    rest.unwrap_or(path).to_string()
}

/// Where the lines of the model and of the evidence are.
struct Places<'a> {
    model: &'a Source,
    evidence: &'a str,
}

impl mitra_lang::Places for Places<'_> {
    fn model(&self, line: usize) -> String {
        relative(&self.model.place(line))
    }
    fn evidence(&self, line: usize) -> String {
        relative(&format!("{}:{line}", self.evidence))
    }
}

/// The places of a ledger with an observation read after it.
struct Seen<'a> {
    ledger: Places<'a>,
    lines: usize,
}

impl mitra_lang::Places for Seen<'_> {
    fn model(&self, line: usize) -> String {
        mitra_lang::Places::model(&self.ledger, line)
    }
    fn evidence(&self, line: usize) -> String {
        match line > self.lines {
            true => "the observation".to_string(),
            false => mitra_lang::Places::evidence(&self.ledger, line),
        }
    }
}

/// A revision given with `--rev`: not empty, not an option, no quotes or
/// line breaks (it is written into the evidence file).
// mitra: implements commands::check_rev
pub fn check_rev(rev: &str) -> Result<(), String> {
    if rev.is_empty() || rev.starts_with('-') || rev.contains(['"', '\n', '\r']) {
        return Err(format!(
            "--rev `{rev}`: must not be empty, start with `-`, or hold quotes or line breaks"
        ));
    }
    Ok(())
}

/// Parse, check and evaluate both files. With a revision, only evidence
/// recorded at that revision counts: evidence at another revision, or with
/// no `rev`, is stale and left out. Otherwise an old pass could outlive a
/// newer failure that is left out.
fn evaluate(
    m: &Source,
    e: &str,
    rev: Option<&str>,
) -> Result<(mitra_lang::Model, mitra_lang::Report), String> {
    let evidence = match e {
        "" => String::new(),
        e => read(e)?,
    };
    let mut model = mitra_lang::load_modules(m, &evidence).map_err(located(m, e))?;
    if let Some(rev) = rev {
        let before = model.evidence.len();
        // What the model itself declares (`assume`, `uses`) has no revision.
        let files = model.written.as_ref().map(|w| w.1.len());
        let declared = |x: &mitra_lang::Evidence| files.is_some_and(|n| x.line > n);
        model
            .evidence
            .retain(|x| x.rev.as_deref() == Some(rev) || declared(x));
        let stale = before - model.evidence.len();
        if stale > 0 {
            eprintln!("mitra: {stale} evidence left out: not recorded at revision {rev}");
        }
    }
    let report = mitra_lang::evaluate(&model).map_err(located(m, e))?;
    Ok((model, report))
}

/// Remove `--rev <value>` from the arguments of `check`, `eval` and `next`.
fn take_rev(cmd: &str, rest: &mut Vec<String>) -> Result<Option<String>, String> {
    let takes = [
        "check", "eval", "next", "plan", "tie", "conform", "assess", "diagnose",
    ];
    if !takes.contains(&cmd) {
        return Ok(None);
    }
    let Some(i) = rest.iter().position(|a| a == "--rev") else {
        return Ok(None);
    };
    if i + 1 >= rest.len() || rest[i + 1..].iter().skip(1).any(|a| a == "--rev") {
        return Err(USAGE.into());
    }
    let rev = rest.remove(i + 1);
    rest.remove(i);
    check_rev(&rev)?;
    Ok(Some(rev))
}

/// Remove `name <value>` from the arguments and return the value.
fn take_value(name: &str, rest: &mut Vec<String>) -> Result<Option<String>, String> {
    let Some(i) = rest.iter().position(|a| a == name) else {
        return Ok(None);
    };
    if i + 1 >= rest.len() || rest[i + 2..].iter().any(|a| a == name) {
        return Err(USAGE.into());
    }
    let value = rest.remove(i + 1);
    rest.remove(i);
    Ok(Some(value))
}

/// How much of each error `eval` shows: `--candidates <n>`, `--threshold
/// <p>`, or `--all` for every candidate.
fn take_display(rest: &mut Vec<String>) -> Result<mitra_lang::Display, String> {
    let mut display = mitra_lang::Display::default();
    if let Some(n) = take_value("--candidates", rest)? {
        display.max = n
            .parse()
            .map_err(|_| format!("--candidates `{n}`: not a whole number"))?;
    }
    if let Some(p) = take_value("--threshold", rest)? {
        display.threshold = match p.parse::<f64>() {
            Ok(x) if (0.0..=1.0).contains(&x) => x,
            _ => return Err(format!("--threshold `{p}`: not a number from 0 to 1")),
        };
    }
    if let Some(i) = rest.iter().position(|a| a == "--all") {
        rest.remove(i);
        display.max = usize::MAX;
        display.threshold = 0.0;
    }
    Ok(display)
}

/// `Ok(false)` when `eval` found errors.
fn run(args: Vec<String>) -> Result<bool, String> {
    run_command(args).map(|errors| errors == 0)
}

/// The number of errors `eval` found; 0 for the other commands.
fn run_command(args: Vec<String>) -> Result<usize, String> {
    let Some((cmd, rest)) = args.split_first() else {
        return Err(USAGE.into());
    };
    let mut rest = rest.to_vec();
    let rev = take_rev(cmd, &mut rest)?;
    let rev = rev.as_deref();
    let display = match cmd.as_str() {
        "eval" | "build" | "conform" | "assess" | "diagnose" => take_display(&mut rest)?,
        _ => mitra_lang::Display::default(),
    };
    let rest = rest.as_slice();
    match cmd.as_str() {
        "init" if rest.is_empty() => {
            let dir = store::Store::init(&std::env::current_dir().map_err(|e| e.to_string())?)?;
            println!("initialised {}", dir.display());
            Ok(0)
        }
        "check" => {
            let (m, e) = inputs(rest)?;
            let (model, report) = evaluate(&m, &e, rev)?;
            println!(
                "ok: {} functions, {} claims, {} evidence",
                model
                    .functions
                    .iter()
                    .filter(|f| !f.name.ends_with("::prose"))
                    .count(),
                model.claims.len(),
                model.evidence.len()
            );
            for f in &report.flags {
                println!("contested evidence line {}: {}", f.line, f.message);
            }
            Ok(0)
        }
        "next" => {
            let (m, e) = inputs(rest)?;
            let (model, report) = evaluate(&m, &e, rev)?;
            print!("{}", mitra_lang::next(&model, &report));
            Ok(0)
        }
        "plan" => {
            let (m, e) = inputs(rest)?;
            let (model, report) = evaluate(&m, &e, rev)?;
            print!("{}", mitra_lang::plan(&model, &report));
            Ok(0)
        }
        "eval" => {
            let (paths, out) = match rest {
                [p @ .., o, out] if o == "-o" => (p, Some(out)),
                p => (p, None),
            };
            if out.is_some_and(|o| o.starts_with('-') || o.ends_with(".mitra")) {
                return Err(
                    "-o: the result file must not start with `-` or end in `.mitra`".into(),
                );
            }
            let (m, e) = inputs(paths)?;
            let (_, report) = evaluate(&m, &e, rev)?;
            let places = Places {
                model: &m,
                evidence: &e,
            };
            let text = report.to_text_at(&display, &places);
            match out {
                Some(path) => std::fs::write(path, text).map_err(|e| format!("{path}: {e}"))?,
                None => print!("{text}"),
            }
            let (errors, warnings) = (report.problems.len(), report.warnings());
            eprintln!("mitra: {errors} errors, {warnings} warnings");
            Ok(errors)
        }
        "explain" => {
            let [code, paths @ ..] = rest else {
                return Err(USAGE.into());
            };
            // A module shipped with the tool: its text.
            if let Some(text) = mitra_lang::parse::library(code) {
                print!("{text}");
                return Ok(0);
            }
            if let Some(text) = mitra_lang::explain(code) {
                print!("{text}");
                return Ok(0);
            }
            // Not a code: a claim of the model.
            let (m, e) = inputs(paths).map_err(|_| format!("explain `{code}`: no such code"))?;
            let (model, report) = evaluate(&m, &e, rev)?;
            let ends =
                |c: &&mitra_lang::Claim| c.name == *code || c.name.ends_with(&format!("::{code}"));
            let found: Vec<&mitra_lang::Claim> = model.claims.iter().filter(ends).collect();
            let [c] = found[..] else {
                let names: Vec<&str> = found.iter().map(|c| c.name.as_str()).collect();
                return Err(match names.is_empty() {
                    true => format!("explain `{code}`: no such code, and no such claim"),
                    false => format!("explain `{code}`: one of {}", names.join(", ")),
                });
            };
            let places = Places {
                model: &m,
                evidence: &e,
            };
            println!("claim {}", c.name);
            let written = model.written.as_ref();
            if let Some(w) = written.and_then(|w| mitra_lang::parse::written(&w.0, c.line)) {
                println!("  written   {w}");
            }
            let read = mitra_lang::report::print::contract(&model, c);
            println!("  read as   {}", model.named(&read));
            let about = model.evidence.iter().filter(|e| e.about == c.name).count();
            println!("  evidence  {about} piece(s) on the claim itself");
            let text = report.to_text_at(&mitra_lang::Display::default(), &places);
            let mut lines = text
                .lines()
                .skip_while(|l| !l.starts_with(&format!("{}: ", c.name)));
            if let Some(verdict) = lines.next() {
                println!("\n{verdict}");
                for l in lines.take_while(|l| l.starts_with("  ")) {
                    println!("{l}");
                }
            }
            Ok(0)
        }
        "add" => {
            let [text, opts @ ..] = rest else {
                return Err(USAGE.into());
            };
            if opts.first().is_some_and(|o| !o.starts_with("--")) {
                let words: Vec<&str> = rest
                    .iter()
                    .map(|w| w.as_str())
                    .take_while(|w| !w.starts_with("--"))
                    .collect();
                return Err(format!(
                    "`mitra add` takes the evidence as one argument, and got {}\n  \
                     help: put it in quotes, so that the shell passes it whole:\n  \
                     mitra add '{}'",
                    words.len(),
                    words.join(" ")
                ));
            }
            let (mut source, mut rev) = (None, None);
            let mut opts = opts.iter();
            while let Some(o) = opts.next() {
                let v = opts.next().ok_or(USAGE)?;
                match o.as_str() {
                    "--source" if source.is_none() => source = Some(v.clone()),
                    "--rev" if rev.is_none() => rev = Some(v.clone()),
                    _ => return Err(USAGE.into()),
                }
            }
            if let Some(r) = &rev {
                check_rev(r)?;
            }
            let s = store::Store::find()?;
            let n = s.add(text, source, rev)?;
            println!("added {n} evidence to {}", s.evidence_path());
            Ok(0)
        }
        "ingest" => {
            // One run takes every report of the revision: what the tests
            // said before leaves the ledger.
            let paths = rest.iter().take_while(|w| !w.starts_with("--")).count();
            let (paths, opts) = rest.split_at(paths);
            if paths.is_empty() {
                return Err(USAGE.into());
            }
            let (mut format, mut rev) = (None, None);
            let mut opts = opts.iter();
            while let Some(o) = opts.next() {
                let v = opts.next().ok_or(USAGE)?;
                match o.as_str() {
                    "--format" if format.is_none() => format = Some(v.clone()),
                    "--rev" if rev.is_none() => rev = Some(v.clone()),
                    _ => return Err(USAGE.into()),
                }
            }
            let mut outcomes = Vec::new();
            let mut junit = false;
            for path in paths {
                let text = read(path)?;
                let text = text.trim_start_matches('\u{feff}');
                outcomes.extend(match format.as_deref() {
                    Some("junit") => mitra_lang::ledger::results::junit(text),
                    Some("libtest") => reader().results(text),
                    None if text.trim_start().starts_with('<') => {
                        junit = true;
                        mitra_lang::ledger::results::junit(text)
                    }
                    None => reader().results(text),
                    Some(f) => return Err(format!("unknown format `{f}` (libtest, junit)")),
                });
            }
            if let Some(r) = &rev {
                check_rev(r)?;
            }
            let s = store::Store::find()?;
            let model = s.model()?;
            // Tests tied to a claim by a comment: `// mitra: evidence for c`.
            let tied = reader().tests(&s.root())?;
            let said =
                mitra_lang::ledger::results::from_tests(&model, &tied, &outcomes, rev.as_deref())?;
            // What ran the tests is the machine; who wrote each is what
            // its comment says, or nobody known.
            let runner = match format.as_deref() {
                Some("junit") => "junit",
                None if junit => "junit",
                _ => "cargo",
            };
            let signed = |l: &String| {
                let act = if l.contains(" sampled ") {
                    ""
                } else {
                    " act test"
                };
                let by = format!("{act} by \"machine:{runner}\" source \"test:");
                l.replacen(" source \"test:", &by, 1)
            };
            let lines: Vec<String> = said.evidence.iter().map(signed).collect();
            let (tied_notes, loose) = (said.notes, said.loose);
            let replaced = s.replace("test:", &lines.join("\n"))?;
            let claimed = |name: &str| tied.iter().any(|t| t.test == name);
            let (mut unknown, mut outside) = (Vec::new(), Vec::new());
            let passed = outcomes.iter().filter(|o| o.passed).count();
            for o in outcomes.iter().filter(|o| !claimed(&o.name)) {
                match o.passed {
                    true => unknown.push(o.name.as_str()),
                    false => outside.push(o.name.as_str()),
                }
            }
            println!(
                "{} results: {passed} passed, {} failed; {} evidence written{}",
                outcomes.len(),
                outcomes.len() - passed,
                lines.len(),
                in_place(&replaced, "test results")
            );
            shown(&replaced);
            for n in tied_notes {
                println!("{n}");
            }
            if !loose.is_empty() {
                println!(
                    "warning: {} tied test(s) state no input on a claim that has inputs: \
                     recorded as `passes`, which settles nothing. Pin what each checks, \
                     `// mitra: evidence for <claim> given a = 1`, or tie it to a point claim:",
                    loose.len()
                );
                for l in &loose {
                    println!("  {l}");
                }
            }
            if !outside.is_empty() {
                println!(
                    "FAILED, and tied to no claim, so no verdict shows it: {}",
                    outside.join(", ")
                );
            }
            if !unknown.is_empty() {
                println!("tied to no claim: {}", unknown.join(", "));
            }
            Ok(0)
        }
        // The four levels: each runs the ones before it and stops at the
        // first that fails.
        "build" if rest.is_empty() => {
            let s = store::Store::find()?;
            Ok(build(&s, &display)?.errors)
        }
        "conform" if rest.is_empty() => {
            let s = store::Store::find()?;
            let built = build(&s, &display)?;
            if built.errors > 0 {
                return Ok(built.errors);
            }
            Ok(conform(&s, &built, rev, &reader())?)
        }
        "assess" | "diagnose" => {
            let s = store::Store::find()?;
            let (seen, reports) = match (cmd.as_str(), rest) {
                ("diagnose", [seen, reports @ ..]) => (Some(seen), reports),
                ("diagnose", []) => return Err(USAGE.into()),
                (_, reports) => (None, reports),
            };
            if reports.iter().any(|r| r.starts_with("--")) {
                return Err(USAGE.into());
            }
            let built = build(&s, &display)?;
            if built.errors > 0 {
                return Ok(built.errors);
            }
            let drift = conform(&s, &built, rev, &reader())?;
            if drift > 0 {
                return Ok(drift);
            }
            if !reports.is_empty() {
                let mut args = vec!["ingest".to_string()];
                args.extend(reports.iter().cloned());
                args.extend(
                    rev.iter()
                        .flat_map(|r| ["--rev".to_string(), r.to_string()]),
                );
                run_command(args)?;
            }
            let errors = assess(&s, &display, rev, seen.is_none())?;
            match seen {
                Some(seen) if errors == 0 => diagnose(&s, seen, &display),
                _ => Ok(errors),
            }
        }
        "remove" => {
            let which = match rest {
                [o] if o == "--orphaned" => store::Which::Orphaned,
                [o] if o == "--stale" => store::Which::Stale(None),
                [o, piece] if o == "--stale" => store::Which::Stale(Some(piece.clone())),
                [o, who] if o == "--source" => store::Which::Source(who.clone()),
                [piece] if !piece.starts_with("--") => store::Which::Piece(piece.clone()),
                _ => return Err(USAGE.into()),
            };
            let s = store::Store::find()?;
            let gone = s.remove(&which)?;
            println!("removed {} evidence from {}", gone.len(), s.evidence_path());
            for g in &gone {
                println!("  {g}");
            }
            Ok(0)
        }
        "tie" if rest.is_empty() => {
            let s = store::Store::find()?;
            let model = s.model()?;
            let root = s.root();
            let found = reader().tie(&root, &model, rev)?;
            let replaced = s.replace("reader:", &found.evidence.join("\n"))?;
            println!(
                "{} agree, {} differ; {} evidence written{}",
                found.agree,
                found.differ,
                found.evidence.len(),
                in_place(&replaced, "ties")
            );
            shown(&replaced);
            for n in &found.notes {
                println!("{n}");
            }
            Ok(0)
        }
        _ => Err(USAGE.into()),
    }
}

#[cfg(test)]
mod tests {
    use super::check_rev;

    // mitra: evidence for commands::plain_rev_is_taken
    #[test]
    fn plain_rev_is_taken() {
        assert!(check_rev("abc123").is_ok());
    }

    // mitra: evidence for commands::empty_rev_is_refused
    #[test]
    fn empty_rev_is_refused() {
        assert!(check_rev("").is_err());
    }

    // mitra: evidence for commands::option_as_rev_is_refused
    #[test]
    fn option_as_rev_is_refused() {
        assert!(check_rev("--source").is_err());
    }

    // mitra: evidence for commands::quoted_rev_is_refused
    #[test]
    fn quoted_rev_is_refused() {
        assert!(check_rev("a\"b").is_err());
    }
}
