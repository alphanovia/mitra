//! The `mitra` command line: a wrapper over mitra-lang, with
//! mitra-rust to read Rust code and test reports.
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

mod source;
mod store;

use mitra_rust::{report, tie};

use source::Source;

use std::process::ExitCode;

const USAGE: &str = "usage:
  mitra init
  mitra check [<model.mitra> [<evidence.mitra>]] [--rev <commit>]
  mitra eval  [<model.mitra> [<evidence.mitra>]] [--rev <commit>] [--candidates <n>] [--threshold <p>] [--all] [-o <result.txt>]
  mitra add   '<evidence>' [--source <who>] [--rev <commit>]
  mitra next  [<model.mitra> [<evidence.mitra>]] [--rev <commit>]
  mitra plan  [<model.mitra> [<evidence.mitra>]] [--rev <commit>]
  mitra ingest <report> [--format libtest|junit] [--rev <commit>]
  mitra tie   [--rev <commit>]
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
            None => Ok((Source::file(m)?, e.clone())),
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

fn read(p: &str) -> Result<String, String> {
    std::fs::read_to_string(p).map_err(|e| format!("{p}: {e}"))
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
) -> Result<(mitra_lang::Model, mitra_lang::Report), String> {
    let evidence = match e {
        "" => String::new(),
        e => read(e)?,
    };
    let mut model = mitra_lang::load(&m.text, &evidence).map_err(located(m, e))?;
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
        "eval" => take_display(&mut rest)?,
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
            let read = mitra_lang::print::contract(&model, c);
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
            // Tests tied to a claim by a comment: `// mitra: evidence for c`.
            let tied = tie::tests(&s.root())?;
            let (mut lines, mut tied_notes) = (Vec::new(), Vec::new());
            let attrs = match &rev {
                Some(r) => format!(" rev \"{r}\""),
                None => String::new(),
            };
            let mut loose = Vec::new();
            for t in &tied {
                let ran: Vec<&report::Outcome> =
                    outcomes.iter().filter(|o| o.name == t.test).collect();
                let passed = match ran[..] {
                    [] if t.pinned.is_empty() => None,
                    [] => continue,
                    [o] => Some(o.passed),
                    _ => {
                        tied_notes.push(format!(
                            "test {} appears {} times in the report: left out",
                            t.test,
                            ran.len()
                        ));
                        continue;
                    }
                };
                // What the comment pins is written as it stands; a failure
                // denies it on the same inputs.
                if !t.pinned.is_empty() {
                    let words: Vec<&str> = t.pinned.split_whitespace().collect();
                    let said = words
                        .iter()
                        .position(|w| ["gives", "holds", "fails"].contains(w));
                    let given = words[..said.unwrap_or(words.len())].join(" ");
                    // A sample: the domain and the count stand, passed or not.
                    let drawn = words.contains(&"sampled");
                    let says = match (passed == Some(true), said) {
                        (true, _) if drawn => t.pinned.clone(),
                        (false, _) if drawn => format!("{} fails", t.pinned),
                        (true, Some(_)) => t.pinned.clone(),
                        (true, None) => format!("{given} holds"),
                        (false, _) => format!("{given} fails"),
                    };
                    lines.push(format!(
                        "evidence test {{ for {} {says} source \"test:{}\"{attrs} }}",
                        t.claim, t.test
                    ));
                    // A test that compares with the output it expects also
                    // says the code is right on that input.
                    let gives = said.is_some_and(|n| words[n] == "gives");
                    if passed == Some(true) && gives && model.function(&t.claim).is_some() {
                        lines.push(format!(
                            "evidence test {{ for {} {given} holds source \"test:{}\"{attrs} }}",
                            t.claim, t.test
                        ));
                    }
                    continue;
                }
                let ends = |c: &&mitra_lang::Claim| {
                    c.name == t.claim || c.name.ends_with(&format!("::{}", t.claim))
                };
                let found: Vec<&mitra_lang::Claim> = model.claims.iter().filter(ends).collect();
                let [c] = found[..] else {
                    let why = if found.is_empty() {
                        "no claim"
                    } else {
                        "several claims"
                    };
                    return Err(format!("{}: the model has {why} `{}`", t.place, t.claim));
                };
                let Some(passed) = passed else { continue };
                // On a claim with inputs, a test that states none passes
                // on some of them: it settles nothing. A failure refutes.
                let takes = model
                    .function(&c.func)
                    .is_some_and(|f| !f.params.is_empty());
                let point = match c.pattern.is_empty() {
                    true => !takes,
                    false => c.pattern.iter().all(|p| p.var.is_none()),
                };
                let says = match (passed, point) {
                    (false, _) => "fails",
                    (true, true) => "holds",
                    (true, false) => {
                        loose.push(format!("{}: {} on `{}`", t.place, t.test, c.name));
                        "passes"
                    }
                };
                lines.push(format!(
                    "evidence test {{ for {} {says} source \"test:{}\"{attrs} }}",
                    c.name, t.test
                ));
            }
            lines.dedup();
            if !lines.is_empty() {
                s.add(&lines.join("\n"), None, None)?;
            }
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
                "{} results: {passed} passed, {} failed; {} evidence added",
                outcomes.len(),
                outcomes.len() - passed,
                lines.len()
            );
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
