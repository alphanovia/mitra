//! `mitra`: one command per question (`docs/design/cli.md`).
//!
//! - `mitra check MODEL [--points N]`: is the model sound on itself,
//!   and if not, where? Its errors (`check`), then what agree finds,
//!   trying at most `N` points per comparison (agree's `points`).
//! - `mitra explain MODEL [--code DIR]... [--report FILE | --no-test] [--store FILE]`:
//!   does the model explain the system, assuming the evidence? A verdict
//!   on each proposition, with its numbers, then the findings, then the
//!   reviews the model asks for. With no `--report`, it runs `cargo test
//!   --no-fail-fast -- --show-output` at the project's root, when the
//!   root has a `Cargo.toml`, and reads what it prints; `--no-test` runs
//!   none.
//! - `mitra blame MODEL [...]`: which evidence is wrong, given the model
//!   explains the system? The diagnoses, the most doubtful first.
//!
//! Paths in the code are read from the model's folder, as its ties
//! write them: `src/lib.rs/div`, however the folders are spelled on
//! the command line. The exit code is 1 when something is not well
//! formed (an error), 2 when the command line is wrong or a file cannot
//! be read, 0 otherwise: findings are verdicts, not faults.

use mitra_lang::blame::{self, Suspect};
use mitra_lang::catalogue::{Error, Finding, R005};
use mitra_lang::check::check;
use mitra_lang::evidence;
use mitra_lang::explain::{self, Explanation, Source, Verdict, LINK, MODEL};
use mitra_lang::judgement::Taking;
use mitra_lang::model::Model;
use mitra_lang::numbers::Numbers;
use mitra_lang::{agree, parse, print};
use mitra_rust::reader;
use mitra_rust::run::{cargo_test, rust_files};
use mitra_store::project::{self, read};
use mitra_store::settings;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

/// `println!`, ending quietly when what reads the output stops reading
/// (`mitra explain … | head`).
macro_rules! out {
    ($($t:tt)*) => {
        if writeln!(std::io::stdout(), $($t)*).is_err() {
            std::process::exit(0);
        }
    };
}

const USAGE: &str = "usage:
  mitra check MODEL [--points N]
  mitra explain MODEL [--code DIR]... [--report FILE | --no-test] [--store FILE]
  mitra blame MODEL [--code DIR]... [--report FILE | --no-test] [--store FILE]

  MODEL      the model, a .mitra file of the project
  --code     a folder of Rust files, read for ties and measuring tests
  --report   the output of `cargo test`, read instead of running it
  --no-test  run no `cargo test`, read no report
  --store    measurements and reviews written by hand
  --points  the most points agree tries per comparison (default 64)";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    // On a thread of 256 MiB: a model as deep as the budgets allow
    // (`parse::Settings::depth`, `simulation::Budget::depth`) fits.
    let ran = std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(move || run(&args))
        .map_err(|e| format!("cannot start: {e}"))
        .and_then(|t| {
            t.join()
                .map_err(|_| "stopped by a fault of mitra".to_string())
        })
        .and_then(|r| r);
    match ran {
        Ok(code) => code,
        Err(message) => {
            eprintln!("{message}\n\n{USAGE}");
            ExitCode::from(2)
        }
    }
}

/// What the command line gives.
struct Given {
    command: String,
    model: PathBuf,
    code: Vec<PathBuf>,
    report: Option<PathBuf>,
    store: Option<PathBuf>,
    points: Option<usize>,
    no_test: bool,
}

fn arguments(args: &[String]) -> Result<Given, String> {
    let mut it = args.iter();
    let command = it.next().ok_or("no command")?.clone();
    if !["check", "explain", "blame"].contains(&command.as_str()) {
        return Err(format!("`{command}` is no command"));
    }
    let model = PathBuf::from(it.next().ok_or("no model")?);
    let mut given = Given {
        command,
        model,
        code: vec![],
        report: None,
        store: None,
        points: None,
        no_test: false,
    };
    while let Some(flag) = it.next() {
        if flag == "--no-test" {
            if std::mem::replace(&mut given.no_test, true) {
                return Err("`--no-test` is given twice".to_string());
            }
            continue;
        }
        if flag == "--points" {
            let v = it.next().ok_or("`--points` needs a value")?;
            let n: usize = v.parse().ok().filter(|n| *n > 0).ok_or(format!(
                "`--points` takes a whole number above 0, not `{v}`"
            ))?;
            if given.points.replace(n).is_some() {
                return Err("`--points` is given twice".to_string());
            }
            continue;
        }
        if !["--code", "--report", "--store"].contains(&flag.as_str()) {
            return Err(format!("`{flag}` is no option"));
        }
        let v = PathBuf::from(it.next().ok_or(format!("`{flag}` needs a value"))?);
        let once = |slot: &mut Option<PathBuf>| match slot.replace(v.clone()) {
            None => Ok(()),
            Some(_) => Err(format!("`{flag}` is given twice: one file each")),
        };
        match flag.as_str() {
            "--code" => given.code.push(v.clone()),
            "--report" => once(&mut given.report)?,
            _ => once(&mut given.store)?,
        }
    }
    if given.command != "check" && given.points.is_some() {
        return Err("`--points` is for `check`: only it compares".to_string());
    }
    if given.no_test && given.report.is_some() {
        return Err(
            "`--no-test` and `--report` together: a report is of tests that ran".to_string(),
        );
    }
    if given.command == "check"
        && (given.report.is_some()
            || given.store.is_some()
            || !given.code.is_empty()
            || given.no_test)
    {
        return Err("`check` reads the model only".to_string());
    }
    Ok(given)
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let given = arguments(args)?;
    let source = read(&given.model)?;
    let shown = given.model.display().to_string();
    // A part the model uses and does not declare is read from the
    // project's files that declare it.
    let root = project::root(&given.model);
    let project = project::project(&root, &given.model);
    // Each layer's settings, from `mitra.toml` when the root has one.
    let toml = root.join("mitra.toml");
    let settings = match toml.exists() {
        false => settings::Settings::default(),
        true => match settings::read(&read(&toml)?) {
            Ok(s) => s,
            Err(e) => {
                out!(
                    "{}",
                    e.render(&toml.display().to_string(), &read(&toml)?)
                        .trim_end()
                );
                return Ok(ExitCode::from(1));
            }
        },
    };
    let mut model = match parse::load(&source, &project, &settings.parse) {
        Ok(m) => m,
        Err(l) => {
            let (file, text) = match l.file.is_empty() {
                true => (shown.clone(), source.clone()),
                false => {
                    let file = match l.file.starts_with("std/") {
                        true => l.file.clone(),
                        false => root.join(&l.file).display().to_string(),
                    };
                    (file, l.source.clone())
                }
            };
            out!("{}", l.error.render(&file, &text).trim_end());
            return Ok(ExitCode::from(1));
        }
    };
    // A set's condition reads the model's functions within the budget
    // the settings give (finding 12b).
    mitra_lang::model::define_sets(&mut model, settings.explain.budget);
    let errors = check(&model);
    if !errors.is_empty() {
        errors_out(&errors);
        out!("{} errors: the model is not well formed", errors.len());
        return Ok(ExitCode::from(1));
    }
    if given.command == "check" {
        let mut agree = settings.agree;
        if let Some(n) = given.points {
            agree.points = n;
        }
        return Ok(check_out(&model, &agree));
    }
    // Paths from the model's folder, however both are spelled.
    let whole = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let root = whole(&root);
    let mut files = Vec::new();
    for dir in &given.code {
        rust_files(dir, &mut files)?;
    }
    let mut code = Vec::new();
    for f in &files {
        let full = whole(f);
        let relative = full.strip_prefix(&root).unwrap_or(&full);
        code.push((relative.display().to_string().replace('\\', "/"), read(f)?));
    }
    // The report given, or `cargo test` run at the root, or none.
    let cargo = root.join("Cargo.toml").exists() && !given.no_test;
    let (report_name, report) = match &given.report {
        Some(r) => (r.display().to_string(), Some(read(r)?)),
        None if cargo => (
            settings.test.command.join(" "),
            Some(cargo_test(&root, &settings.test.command)?),
        ),
        None => (String::new(), None),
    };
    if let Some(text) = &report {
        let runs = reader::read_cargo_test(text);
        let passed = runs
            .iter()
            .filter(|r| r.outcome == reader::Outcome::Passed)
            .count();
        let failed = runs
            .iter()
            .filter(|r| r.outcome == reader::Outcome::Failed)
            .count();
        let ran = match given.report {
            Some(_) => format!("read {report_name}"),
            None => format!("ran `{report_name}` in {}", root.display()),
        };
        out!(
            "{ran}: {} tests, {passed} passed, {failed} failed, {} ignored\n",
            runs.len(),
            runs.len() - passed - failed
        );
    }
    // Code read with no report: its tests ran in none, and say so.
    let unreported = report.is_none() && !files.is_empty();
    let report = match report {
        None if unreported => Some(String::new()),
        r => r,
    };
    let store_text = given.store.as_deref().map(read).transpose()?;
    let store_name = given.store.as_ref().map(|p| p.display().to_string());
    let store = store_name.as_deref().zip(store_text.as_deref());
    let (ties, ledger, reader_findings) = match reader::ledger(
        &model,
        &code,
        report.as_deref().map(|r| (report_name.as_str(), r)),
        store,
    ) {
        Ok(read) => read,
        Err(e) => {
            let (name, text) = store.unwrap_or(("store", ""));
            out!("{}", e.render(name, text).trim_end());
            return Ok(ExitCode::from(1));
        }
    };
    if unreported && reader_findings.iter().any(|f| f.code == R005) {
        let why = match given.no_test {
            true => "`--no-test`",
            false => "no `--report`, and no `Cargo.toml` at the root to run `cargo test` in",
        };
        out!("{why}: the tests in the code that measure ran in no report, and measured nothing\n");
    }
    let mut reader_findings = reader_findings;
    if let Some(text) = &report {
        let mut first = reader::stopped(&report_name, text);
        first.extend(reader::unread(&report_name, text));
        reader_findings.splice(0..0, first);
    }
    for f in &reader_findings {
        out!("{f}\n");
    }
    model.ties = Vec::new();
    for t in ties {
        if !model.ties.contains(&t) {
            model.ties.push(t);
        }
    }
    let refused = evidence::check(&model, &ledger);
    errors_out(&refused);
    let e = explain::explain(&model, &ledger, &settings.explain);
    let n = Numbers::of(&e, &settings.numbers);
    out!(
        "evidence: {} measurements, {} reviews, {} ties",
        ledger.measurements.len(),
        ledger.reviews.len(),
        model.ties.len()
    );
    match given.command.as_str() {
        "explain" => explain_out(&e, &n),
        _ => blame_out(&model, &ledger, &e, &n, &settings.blame),
    }
    Ok(match refused.is_empty() {
        true => ExitCode::SUCCESS,
        false => ExitCode::from(1),
    })
}

fn errors_out(errors: &[Error]) {
    for e in errors {
        out!("{e}\n");
    }
}

fn findings_out(findings: &[Finding]) {
    for f in findings {
        out!("{f}\n");
    }
}

/// Question 1.
fn check_out(model: &Model, settings: &agree::Settings) -> ExitCode {
    let (agreed, findings) = agree::agree(model, settings);
    findings_out(&findings);
    for h in agree::hints(&agreed) {
        out!("{h}\n");
    }
    let holds = agreed
        .iter()
        .filter(|a| a.agreement == agree::Agreement::Holds)
        .count();
    out!(
        "check: well formed; agree: {} comparisons, {holds} hold, {} findings",
        agreed.len(),
        findings.len()
    );
    ExitCode::SUCCESS
}

/// What a set of takings rests on, for a person: counts for the
/// ledger's, voices, and how many only the model says (a link no one
/// reviewed among them).
fn rest_summary(rest: Option<&std::collections::BTreeSet<Taking>>) -> (u64, Vec<String>, usize) {
    let Some(rest) = rest else {
        return (0, vec![], 0);
    };
    let mut voices: Vec<String> = Vec::new();
    let (mut runs, mut assumed) = (0, 0);
    for t in rest {
        if t.by == MODEL || t.by == LINK {
            assumed += 1;
            continue;
        }
        runs += t.count;
        if !voices.contains(&t.by) {
            voices.push(t.by.clone());
        }
    }
    (runs, voices, assumed)
}

/// Question 2.
fn explain_out(e: &Explanation, n: &Numbers) {
    let (mut explained, mut contradicted, mut unsupported) = (0, 0, 0);
    let (mut assumed_n, mut waiting) = (0, 0);
    for v in &e.verdicts {
        let word = match &v.verdict {
            Verdict::Explained => {
                explained += 1;
                "explained"
            }
            Verdict::Assumed { links, .. } => {
                assumed_n += 1;
                if !links.is_empty() {
                    waiting += 1;
                }
                "assumed"
            }
            Verdict::Contradicted { .. } => {
                contradicted += 1;
                "contradicted"
            }
            Verdict::Unsupported => {
                unsupported += 1;
                "unsupported"
            }
        };
        let b = n.prop(&v.prop);
        let (r_for, r_against) = n.strongest_rests(&v.prop);
        let (runs, mut voices, assumed) = rest_summary(r_for);
        let (against, _, _) = rest_summary(r_against);
        if voices.len() > 3 {
            let more = voices.len() - 3;
            voices.truncate(3);
            voices.push(format!("{more} more"));
        }
        let mut said = vec![format!("{runs} for"), format!("{against} against")];
        match &v.verdict {
            // What it assumes, and what a review would lift.
            Verdict::Assumed { links, requires } => {
                let many = |n: usize, one: &str, more: &str| match n {
                    1 => format!("1 {one}"),
                    _ => format!("{n} {more}"),
                };
                let mut what = Vec::new();
                if !links.is_empty() {
                    let n = many(links.len(), "link", "links");
                    what.push(format!("{n} to review"));
                }
                if !requires.is_empty() {
                    let n = many(requires.len(), "require", "requires");
                    what.push(format!("{n} the world gives"));
                }
                said.push(format!("assuming {}", what.join(" and ")));
            }
            _ if assumed > 0 => said.push(format!("{assumed} only the model says")),
            _ => {}
        }
        said.extend(voices);
        out!(
            "{word:<12} {} {:.2} (b {:.2}, d {:.2}, u {:.2}: {})",
            print::print_prop(&v.prop),
            b.expected,
            b.belief,
            b.disbelief,
            b.uncertainty,
            said.join(", ")
        );
    }
    out!();
    findings_out(&e.findings);
    for h in &e.hints {
        out!("{h}\n");
    }
    for a in &e.asks {
        out!("{a}\n");
    }
    out!(
        "explain: {} propositions, {explained} explained, {assumed_n} assumed ({waiting} waiting on a review), {contradicted} contradicted, {unsupported} unsupported; {} findings, {} hints, {} reviews asked",
        e.verdicts.len(),
        e.findings.len(),
        e.hints.len(),
        e.asks.len()
    );
}

/// A suspect, for a person.
fn suspect_text(ledger: &evidence::Ledger, e: &Explanation, s: &Suspect) -> String {
    match s {
        Suspect::Measurement(i) => {
            let m = &ledger.measurements[*i];
            let outputs: Vec<String> = m
                .outputs
                .iter()
                .map(|(p, v)| format!("{p} = {}", print::value(v)))
                .collect();
            let seen = if m.seen { "saw" } else { "did not see" };
            // Measurements that say the same are one suspect.
            let all: Vec<String> = match e.same.get(i) {
                Some(same) => same.iter().map(usize::to_string).collect(),
                None => vec![i.to_string()],
            };
            format!(
                "measurements::{} ({}, {seen} {})",
                all.join(" and "),
                m.by,
                outputs.join(", ")
            )
        }
        Suspect::Review(i) => {
            let r = &ledger.reviews[*i];
            let says = if r.holds { "holds" } else { "fails" };
            format!("reviews::{i} ({}: {} {says})", r.by, r.said())
        }
        Suspect::Taking(t) => {
            let source = match e.sources.get(t) {
                Some(Source::Model) | None => "only the model says it",
                Some(Source::Proof) => "a proof the checker took",
                Some(Source::Named) => "a require naming an ensure, the checker took",
                Some(Source::Link) => "a link no one reviewed, taken as written",
                Some(Source::Written) => "a rule explain wrote, the checker took",
                Some(Source::Measurement(_) | Source::Review(_)) => "evidence",
            };
            format!("{} ({source})", print::print_prop(&t.prop))
        }
    }
}

/// Question 3.
fn blame_out(
    model: &Model,
    ledger: &evidence::Ledger,
    e: &Explanation,
    n: &Numbers,
    settings: &blame::Settings,
) {
    let b = blame::blame(model, e, settings);
    if b.conflicts.is_empty() {
        out!("blame: nothing conflicts, nothing to blame");
        findings_out(&b.findings);
        return;
    }
    let ranked = n.rank(&b);
    for (i, (d, doubt)) in ranked.iter().enumerate() {
        let drop: Vec<String> = d
            .suspects
            .iter()
            .map(|s| suspect_text(ledger, e, s))
            .collect();
        out!(
            "diagnosis {} (doubt {doubt:.2}): drop {}",
            i + 1,
            drop.join("; ")
        );
    }
    out!();
    findings_out(&b.findings);
    let which = match b.complete {
        true => format!("{} diagnoses", ranked.len()),
        false => format!("the first {} diagnoses", ranked.len()),
    };
    out!("blame: {} conflicts, {which}", b.conflicts.len());
}
