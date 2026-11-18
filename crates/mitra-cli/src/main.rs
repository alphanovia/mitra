//! The `mitra` command line: a thin wrapper over mitra-core.
//!
//! ```text
//! mitra init
//! mitra check [<model> <evidence>] [--rev <commit>]
//! mitra eval  [<model> <evidence>] [--rev <commit>] [--candidates <n>] [--threshold <p>] [--all] [-o <result.txt>]
//! mitra add   '<evidence>' [--source <who>] [--rev <commit>]
//! mitra next  [<model> <evidence>] [--rev <commit>]
//! mitra ingest <report> [--format libtest|junit] [--rev <commit>]
//! ```
//!
//! Without paths, commands use the project store: the nearest `.mitra/`
//! folder at or above the current one, holding `model.mitra` and the
//! append-only `evidence.mitra`. The store's model also holds the `mitra:`
//! comments of the Rust files under `src/` and `tests/` next to it.

mod report;
mod source;
mod store;
mod tie;

use source::Source;

use std::process::ExitCode;

const USAGE: &str = "usage:
  mitra init
  mitra check [<model.mitra> <evidence.mitra>] [--rev <commit>]
  mitra eval  [<model.mitra> <evidence.mitra>] [--rev <commit>] [--candidates <n>] [--threshold <p>] [--all] [-o <result.txt>]
  mitra add   '<evidence>' [--source <who>] [--rev <commit>]
  mitra next  [<model.mitra> <evidence.mitra>] [--rev <commit>]
  mitra plan  [<model.mitra> <evidence.mitra>] [--rev <commit>]
  mitra ingest <report> [--format libtest|junit] [--rev <commit>]
  mitra tie   [--rev <commit>]
  mitra explain <code>
  mitra explain <claim> [<model.mitra> <evidence.mitra>]
  mitra explain std::claims";

fn main() -> ExitCode {
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
fn inputs(paths: &[String]) -> Result<(Source, String), String> {
    match paths {
        // A store's model file brings the `mitra:` lines of its Rust sources.
        [m, e] => match store::Store::of(m) {
            Some(s) => Ok((s.source()?, e.clone())),
            None => Ok((Source::file(m)?, e.clone())),
        },
        [] => {
            let s = store::Store::find()?;
            Ok((s.source()?, s.evidence_path()))
        }
        _ => Err(USAGE.into()),
    }
}

fn read(p: &str) -> Result<String, String> {
    std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))
}

/// An error, prefixed with the file it belongs to.
fn located<'a>(
    m: &'a Source,
    e: &'a str,
) -> impl Fn((mitra_core::Input, mitra_core::Error)) -> String + 'a {
    move |(input, err)| match input {
        mitra_core::Input::Model => m.locate(&err),
        mitra_core::Input::Evidence => format!("{e}: {err}"),
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

impl mitra_core::Places for Places<'_> {
    fn model(&self, line: usize) -> String {
        relative(&self.model.place(line))
    }
    fn evidence(&self, line: usize) -> String {
        relative(&format!("{}:{line}", self.evidence))
    }
}

/// A revision given with `--rev`: not empty, not an option, no quotes or
/// line breaks (it is written into the evidence file).
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
) -> Result<(mitra_core::Model, mitra_core::Report), String> {
    let mut model = mitra_core::load(&m.text, &read(e)?).map_err(located(m, e))?;
    if let Some(rev) = rev {
        let before = model.evidence.len();
        // What the model itself declares (`assume`, `uses`) has no revision.
        let files = model.written.as_ref().map(|w| w.1.len());
        let declared = |x: &mitra_core::Evidence| files.is_some_and(|n| x.line > n);
        model
            .evidence
            .retain(|x| x.rev.as_deref() == Some(rev) || declared(x));
        let stale = before - model.evidence.len();
        if stale > 0 {
            eprintln!("mitra: {stale} evidence left out: not recorded at revision {rev}");
        }
    }
    let report = mitra_core::evaluate(&model).map_err(located(m, e))?;
    Ok((model, report))
}

/// Remove `--rev <value>` from the arguments of `check`, `eval` and `next`.
fn take_rev(cmd: &str, rest: &mut Vec<String>) -> Result<Option<String>, String> {
    if !["check", "eval", "next", "plan", "tie"].contains(&cmd) {
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
fn take_display(rest: &mut Vec<String>) -> Result<mitra_core::Display, String> {
    let mut display = mitra_core::Display::default();
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
        "eval" => take_display(&mut rest)?,
        _ => mitra_core::Display::default(),
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
                "ok: {} functions, {} claims, {} tests, {} evidence",
                model.functions.len(),
                model.claims.len(),
                model.tests.len(),
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
            print!("{}", mitra_core::next(&model, &report));
            Ok(0)
        }
        "plan" => {
            let (m, e) = inputs(rest)?;
            let (model, report) = evaluate(&m, &e, rev)?;
            print!("{}", mitra_core::plan(&model, &report));
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
            if let Some(text) = mitra_core::surface::library(code) {
                print!("{text}");
                return Ok(0);
            }
            if let Some(text) = mitra_core::explain(code) {
                print!("{text}");
                return Ok(0);
            }
            // Not a code: a claim of the model.
            let (m, e) = inputs(paths).map_err(|_| format!("explain `{code}`: no such code"))?;
            let (model, report) = evaluate(&m, &e, rev)?;
            let ends =
                |c: &&mitra_core::Claim| c.name == *code || c.name.ends_with(&format!("::{code}"));
            let found: Vec<&mitra_core::Claim> = model.claims.iter().filter(ends).collect();
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
            if let Some(w) = written.and_then(|w| mitra_core::surface::written(&w.0, c.line)) {
                println!("  written   {w}");
            }
            let read = mitra_core::print::print_claim(&model, c);
            println!("  read as   {}", read.trim_start_matches("claim ").trim());
            let about = model.evidence.iter().filter(|e| e.about == c.name).count();
            println!("  evidence  {about} piece(s) on the claim itself");
            let text = report.to_text_at(&mitra_core::Display::default(), &places);
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
            let [path, opts @ ..] = rest else {
                return Err(USAGE.into());
            };
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
            let text = read(path)?;
            let text = text.trim_start_matches('\u{feff}');
            let outcomes = match format.as_deref() {
                Some("junit") => report::junit(text),
                Some("libtest") => report::libtest(text),
                None if text.trim_start().starts_with('<') => report::junit(text),
                None => report::libtest(text),
                Some(f) => return Err(format!("unknown format `{f}` (libtest, junit)")),
            };
            if let Some(r) = &rev {
                check_rev(r)?;
            }
            let s = store::Store::find()?;
            let model = s.model()?;
            let (mut evidence, mut notes, mut unknown) = (Vec::new(), Vec::new(), Vec::new());
            // Failed tests the model does not declare: no verdict will move.
            let mut outside = Vec::new();
            let passed = outcomes.iter().filter(|o| o.passed).count();
            for o in &outcomes {
                let Some(t) = model.tests.iter().find(|t| t.name == o.name) else {
                    match o.passed {
                        true => unknown.push(o.name.as_str()),
                        false => outside.push(o.name.as_str()),
                    }
                    continue;
                };
                let times = outcomes.iter().filter(|p| p.name == o.name).count();
                if times > 1 {
                    let note = format!(
                        "test {} appears {times} times in the report: left out",
                        t.name
                    );
                    if !notes.contains(&note) {
                        notes.push(note);
                    }
                    continue;
                }
                let about = model.functions.iter().find(|f| f.name == t.judgment.about);
                // A test `f(x) = v` gives the expected output: passing, the
                // output is `v` and it is correct; failing, it is not correct.
                if let Some(mut e) = mitra_core::expected_output(&model, t, o.passed) {
                    e.source = Some(format!("test:{}", t.name));
                    e.rev = rev.clone();
                    evidence.push(e);
                }
                match t.outcome(o.passed, about.map(|f| &f.range)) {
                    Ok(mut e) => {
                        e.source = Some(format!("test:{}", t.name));
                        e.rev = rev.clone();
                        evidence.push(e);
                    }
                    Err(why) => {
                        // A failed output test refutes the `=` claims that
                        // give that output.
                        let refuted = mitra_core::failed_output(&model, t)
                            .map_err(|e| format!("test {}: {}", t.name, e.message))?;
                        let judged = mitra_core::expected_output(&model, t, false).is_some();
                        if refuted.is_empty() && !judged {
                            notes.push(format!("test {} failed: no evidence, since {why}", t.name));
                        }
                        for mut e in refuted {
                            e.source = Some(format!("test:{}", t.name));
                            e.rev = rev.clone();
                            evidence.push(e);
                        }
                    }
                }
            }
            if !evidence.is_empty() {
                s.append(&evidence)?;
            }
            println!(
                "{} results: {passed} passed, {} failed; {} evidence added",
                outcomes.len(),
                outcomes.len() - passed,
                evidence.len()
            );
            for n in notes {
                println!("{n}");
            }
            if !outside.is_empty() {
                println!(
                    "FAILED, and no `test` declared, so no verdict shows it: {}",
                    outside.join(", ")
                );
            }
            if !unknown.is_empty() {
                println!("no `test` declared for: {}", unknown.join(", "));
            }
            Ok(0)
        }
        "tie" if rest.is_empty() => {
            let s = store::Store::find()?;
            let model = s.model()?;
            let root = s.root();
            let found = tie::tie(&root, &model, rev)?;
            if !found.evidence.is_empty() {
                s.add(&found.evidence.join("\n"), None, None)?;
            }
            println!(
                "{} agree, {} differ; {} evidence added",
                found.agree,
                found.differ,
                found.evidence.len()
            );
            for n in &found.notes {
                println!("{n}");
            }
            Ok(0)
        }
        _ => Err(USAGE.into()),
    }
}
