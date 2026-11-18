//! The `mitra` command line: a thin wrapper over mitra-core.
//!
//! ```text
//! mitra init
//! mitra check [<model> <evidence>] [--rev <commit>]
//! mitra eval  [<model> <evidence>] [--rev <commit>] [-o <result.txt>]
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

use source::Source;

use std::process::ExitCode;

const USAGE: &str = "usage:
  mitra init
  mitra check [<model.mitra> <evidence.mitra>] [--rev <commit>]
  mitra eval  [<model.mitra> <evidence.mitra>] [--rev <commit>] [-o <result.txt>]
  mitra add   '<evidence>' [--source <who>] [--rev <commit>]
  mitra next  [<model.mitra> <evidence.mitra>] [--rev <commit>]
  mitra ingest <report> [--format libtest|junit] [--rev <commit>]";

fn main() -> ExitCode {
    match run(std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(msg) => {
            eprintln!("mitra: {msg}");
            ExitCode::from(2)
        }
    }
}

/// The model and the evidence file, from the arguments or from the store.
fn inputs(paths: &[String]) -> Result<(Source, String), String> {
    match paths {
        [m, e] => Ok((Source::file(m)?, e.clone())),
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
        model.evidence.retain(|x| x.rev.as_deref() == Some(rev));
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
    if !["check", "eval", "next"].contains(&cmd) {
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

fn run(args: Vec<String>) -> Result<(), String> {
    let Some((cmd, rest)) = args.split_first() else {
        return Err(USAGE.into());
    };
    let mut rest = rest.to_vec();
    let rev = take_rev(cmd, &mut rest)?;
    let rev = rev.as_deref();
    let rest = rest.as_slice();
    match cmd.as_str() {
        "init" if rest.is_empty() => {
            let dir = store::Store::init(&std::env::current_dir().map_err(|e| e.to_string())?)?;
            println!("initialised {}", dir.display());
            Ok(())
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
            Ok(())
        }
        "next" => {
            let (m, e) = inputs(rest)?;
            let (model, report) = evaluate(&m, &e, rev)?;
            print!("{}", mitra_core::next(&model, &report));
            Ok(())
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
            let text = report.to_text();
            match out {
                Some(path) => std::fs::write(path, text).map_err(|e| format!("{path}: {e}")),
                None => {
                    print!("{text}");
                    Ok(())
                }
            }
        }
        "add" => {
            let [text, opts @ ..] = rest else {
                return Err(USAGE.into());
            };
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
            Ok(())
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
            let passed = outcomes.iter().filter(|o| o.passed).count();
            for o in &outcomes {
                let Some(t) = model.tests.iter().find(|t| t.name == o.name) else {
                    unknown.push(o.name.as_str());
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
                        if refuted.is_empty() {
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
            if !unknown.is_empty() {
                println!("no `test` declared for: {}", unknown.join(", "));
            }
            Ok(())
        }
        _ => Err(USAGE.into()),
    }
}
