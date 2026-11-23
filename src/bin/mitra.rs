//! `mitra`: one command per question (`docs/cli.md`).
//!
//! - `mitra check MODEL`: is the model sound on itself, and if not,
//!   where? Its errors (`check`), then what agree finds.
//! - `mitra explain MODEL [--code DIR]... [--report FILE] [--store FILE]`:
//!   does the model explain the system, assuming the evidence? A verdict
//!   on each proposition, with its numbers, then the findings.
//! - `mitra blame MODEL [...]`: which evidence is wrong, given the model
//!   explains the system? The diagnoses, the most doubtful first.
//!
//! Paths in the code are read from the model's folder, as its ties
//! write them: `src/lib.rs/div`, however the folders are spelled on
//! the command line. The exit code is 1 when something is not well
//! formed (an error), 2 when the command line is wrong or a file cannot
//! be read, 0 otherwise: findings are verdicts, not faults.

use mitra_core::blame::{self, Suspect};
use mitra_core::catalogue::{Error, Finding, R005};
use mitra_core::check::check;
use mitra_core::evidence;
use mitra_core::explain::{self, Explanation, Source, Verdict, MODEL};
use mitra_core::judgement::Taking;
use mitra_core::model::Model;
use mitra_core::numbers::{self, Numbers};
use mitra_core::{agree, parse, print, reader};
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
  mitra check MODEL
  mitra explain MODEL [--code DIR]... [--report FILE] [--store FILE]
  mitra blame MODEL [--code DIR]... [--report FILE] [--store FILE]

  MODEL     the model, a .mitra file at the root of the project
  --code    a folder of Rust files, read for ties and measuring tests
  --report  the output of `cargo test`
  --store   measurements and reviews written by hand";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
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
    };
    while let Some(flag) = it.next() {
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
    if given.command == "check"
        && (given.report.is_some() || given.store.is_some() || !given.code.is_empty())
    {
        return Err("`check` reads the model only".to_string());
    }
    Ok(given)
}

fn read(path: &Path) -> Result<String, String> {
    std::fs::read_to_string(path).map_err(|e| format!("cannot read {}: {e}", path.display()))
}

fn run(args: &[String]) -> Result<ExitCode, String> {
    let given = arguments(args)?;
    let source = read(&given.model)?;
    let shown = given.model.display().to_string();
    let model = match parse::parse(&source) {
        Ok(m) => m,
        Err(e) => {
            out!("{}", e.render(&shown, &source).trim_end());
            return Ok(ExitCode::from(1));
        }
    };
    let errors = check(&model);
    if !errors.is_empty() {
        errors_out(&errors);
        out!("{} errors: the model is not well formed", errors.len());
        return Ok(ExitCode::from(1));
    }
    if given.command == "check" {
        return Ok(check_out(&model));
    }
    // Paths from the model's folder, however both are spelled.
    let whole = |p: &Path| std::fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
    let parent = given.model.parent().filter(|p| !p.as_os_str().is_empty());
    let root = whole(parent.unwrap_or(Path::new(".")));
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
    // Code read with no report: its tests ran in none, and say so.
    let unreported = given.report.is_none() && !files.is_empty();
    let report = match &given.report {
        Some(r) => Some(read(r)?),
        None if unreported => Some(String::new()),
        None => None,
    };
    let store_text = given.store.as_deref().map(read).transpose()?;
    let store_name = given.store.as_ref().map(|p| p.display().to_string());
    let store = store_name.as_deref().zip(store_text.as_deref());
    let (ties, ledger, reader_findings) =
        match reader::ledger(&model, &code, report.as_deref(), store) {
            Ok(read) => read,
            Err(e) => {
                let (name, text) = store.unwrap_or(("store", ""));
                out!("{}", e.render(name, text).trim_end());
                return Ok(ExitCode::from(1));
            }
        };
    if unreported && reader_findings.iter().any(|f| f.code == R005) {
        out!("no `--report`: the tests in the code that measure ran in no report, and measured nothing\n");
    }
    for f in &reader_findings {
        out!("{f}\n");
    }
    let mut model = model;
    model.ties = Vec::new();
    for t in ties {
        if !model.ties.contains(&t) {
            model.ties.push(t);
        }
    }
    let refused = evidence::check(&model, &ledger);
    errors_out(&refused);
    let e = explain::explain(&model, &ledger, &explain::Settings::default());
    let n = Numbers::of(&e, &numbers::Settings::default());
    out!(
        "evidence: {} measurements, {} reviews, {} ties",
        ledger.measurements.len(),
        ledger.reviews.len(),
        model.ties.len()
    );
    match given.command.as_str() {
        "explain" => explain_out(&e, &n),
        _ => blame_out(&model, &ledger, &e, &n),
    }
    Ok(match refused.is_empty() {
        true => ExitCode::SUCCESS,
        false => ExitCode::from(1),
    })
}

/// Every `.rs` file under `dir`, in order.
fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) -> Result<(), String> {
    if dir.is_file() {
        out.push(dir.to_path_buf());
        return Ok(());
    }
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
        .map_err(|e| format!("cannot read {}: {e}", dir.display()))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();
    for p in entries {
        if p.is_dir() {
            rust_files(&p, out)?;
        } else if p.extension().is_some_and(|x| x == "rs") {
            out.push(p);
        }
    }
    Ok(())
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
fn check_out(model: &Model) -> ExitCode {
    let (agreed, findings) = agree::agree(model, &agree::Settings::default());
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
/// ledger's, voices, and how many only the model says.
fn rest_summary(rest: Option<&std::collections::BTreeSet<Taking>>) -> (u64, Vec<String>, usize) {
    let Some(rest) = rest else {
        return (0, vec![], 0);
    };
    let mut voices: Vec<String> = Vec::new();
    let (mut runs, mut assumed) = (0, 0);
    for t in rest {
        if t.by == MODEL {
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
    for v in &e.verdicts {
        let word = match &v.verdict {
            Verdict::Explained => {
                explained += 1;
                "explained"
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
        if assumed > 0 {
            said.push(format!("{assumed} only the model says"));
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
    out!(
        "explain: {} propositions, {explained} explained, {contradicted} contradicted, {unsupported} unsupported; {} findings, {} hints",
        e.verdicts.len(),
        e.findings.len(),
        e.hints.len()
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
            format!("reviews::{i} ({}: {} {says})", r.by, r.prop)
        }
        Suspect::Taking(t) => {
            let source = match e.sources.get(t) {
                Some(Source::Model) | None => "only the model says it",
                Some(Source::Proof) => "a proof the checker took",
                Some(Source::Written) => "a rule explain wrote, the checker took",
                Some(Source::Measurement(_) | Source::Review(_)) => "evidence",
            };
            format!("{} ({source})", print::print_prop(&t.prop))
        }
    }
}

/// Question 3.
fn blame_out(model: &Model, ledger: &evidence::Ledger, e: &Explanation, n: &Numbers) {
    let b = blame::blame(model, e, &blame::Settings::default());
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
