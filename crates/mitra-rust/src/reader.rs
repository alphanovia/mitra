//! Mitra's reader of Rust and cargo (`docs/design/reader.md`): the
//! only crate that knows a language of code. It reads ties and tests
//! from Rust code, what `cargo test` printed, and runs the tests.
//!
//! - `read_rust(file, source, model)`: each `// mitra: implements a::b`
//!   above a `fn`, `struct` or `enum` is a tie of `a::b` at the file,
//!   then the modules and `impl` around the item, then its name:
//!   `src/lib.rs/div`, `src/geo.rs/Point/flip`. Each `// mitra: measures
//!   a::f (6, 2) -> 3` above a `#[test]` says what that test measures.
//! - `read_cargo_test(report)`: each `test name ... ok | FAILED |
//!   ignored` line; `rows`, each `mitra: row` a test printed; `stopped`,
//!   each test binary that gave no result.
//! - `measurements(tests, runs, by)`: a test that passed saw its row
//!   (`seen`), one that failed saw something else (not `seen`).
//!
//! The text of a store and of rows is `mitra_lang::store`, re-exported
//! here; nothing given is dropped quietly.

use mitra_lang::catalogue::{R001, R002, R003, R004, R005, R007, R009};
use mitra_lang::location::Location;
use mitra_lang::model::{Model, Path, Tie};
use mitra_lang::parse::ParseError;
use mitra_lang::simulation::Value;
pub use mitra_lang::store::*;
use std::collections::BTreeSet;

// ---------------------------------------------------------------- Rust code

/// A row a test measures: `function` took `takes` and gave `gives`.
#[derive(Clone, Debug, PartialEq)]
pub struct TestPoint {
    /// The test, from its file's root: `tests::div_by_two`.
    pub test: String,
    pub file: String,
    pub line: usize,
    pub function: Path,
    pub takes: Vec<Value>,
    pub gives: Value,
}

/// What one Rust file says.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RustReading {
    pub ties: Vec<Tie>,
    pub points: Vec<TestPoint>,
    pub findings: Vec<ReaderFinding>,
}

/// A `mitra:` comment waiting for the item under it.
type Pending = Comment;

/// The ties and test points of one Rust file. `file` is its path, as
/// the tie's location starts: `src/lib.rs`. `model` says which
/// functions there are, and the records values are made of.
///
/// Braces are counted line by line to know the modules and `impl`
/// around an item: a brace in a string or a character is counted too.
pub fn read_rust(file: &str, source: &str, model: &Model) -> RustReading {
    let mut reading = RustReading::default();
    // The modules and `impl` around, each with the depth inside it.
    let mut around: Vec<(String, i64)> = vec![];
    let mut depth: i64 = 0;
    let mut pending: Vec<(Pending, usize)> = vec![];
    let mut test = false;
    let base: Vec<String> = file
        .split('/')
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    for (i, text) in source.lines().enumerate() {
        let line = i + 1;
        let trimmed = text.trim();
        if let Some(said) = trimmed.strip_prefix("// mitra:") {
            match read_comment(said.trim(), model) {
                Ok(p) => pending.push((p, line)),
                Err(message) => reading.findings.push(finding(R001, file, line, message)),
            }
            continue;
        }
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if trimmed.starts_with("#[") {
            test |= trimmed.starts_with("#[test]") || trimmed.contains("::test]");
            continue;
        }
        let item = item(trimmed);
        let mut names: Vec<String> = around.iter().map(|(n, _)| n.clone()).collect();
        match &item {
            Some((Item::Fn, name)) => {
                names.push(name.clone());
                for (p, at) in pending.drain(..) {
                    match p {
                        Pending::Implements(path) => {
                            implements(&mut reading, model, file, at, path, &base, &names)
                        }
                        Pending::Measures(function, takes, gives) if test => {
                            reading.points.push(TestPoint {
                                test: names.join("::"),
                                file: file.to_string(),
                                line: at,
                                function,
                                takes,
                                gives,
                            })
                        }
                        Pending::Measures(..) => {
                            let message =
                                format!("`measures` is above `fn {name}`, which is no `#[test]`");
                            reading.findings.push(finding(R003, file, at, message));
                        }
                    }
                }
            }
            Some((Item::Type, name)) => {
                names.push(name.clone());
                for (p, at) in pending.drain(..) {
                    match p {
                        Pending::Implements(path) => {
                            implements(&mut reading, model, file, at, path, &base, &names)
                        }
                        Pending::Measures(..) => {
                            let message = format!("`measures` is above `{name}`, which is no test");
                            reading.findings.push(finding(R003, file, at, message));
                        }
                    }
                }
            }
            Some((Item::Scope, _)) | None => {
                for (p, at) in pending.drain(..) {
                    let (code, what) = match p {
                        Pending::Implements(_) => (R002, "`implements`"),
                        Pending::Measures(..) => (R003, "`measures`"),
                    };
                    let message =
                        format!("{what} is above no `fn`, `struct` or `enum`: `{trimmed}`");
                    reading.findings.push(finding(code, file, at, message));
                }
            }
        }
        test = false;
        let opens = text.matches('{').count() as i64;
        let closes = text.matches('}').count() as i64;
        if let Some((Item::Scope, name)) = item {
            if opens > closes {
                around.push((name, depth + 1));
            }
        }
        depth += opens - closes;
        while around.last().is_some_and(|(_, d)| depth < *d) {
            around.pop();
        }
    }
    for (p, at) in pending {
        let what = match p {
            Pending::Implements(_) => (R002, "`implements`"),
            Pending::Measures(..) => (R003, "`measures`"),
        };
        reading.findings.push(finding(
            what.0,
            file,
            at,
            format!("{} is above nothing", what.1),
        ));
    }
    reading
}

/// A tie of `item`, when the model has it: a part, record or function.
fn implements(
    reading: &mut RustReading,
    model: &Model,
    file: &str,
    line: usize,
    item: Path,
    base: &[String],
    names: &[String],
) {
    let known = model.parts.iter().any(|p| match &item.names[..] {
        [part] => *part == p.name,
        [part, name] => {
            *part == p.name
                && (p.records.iter().any(|r| r.name == *name)
                    || p.functions.iter().any(|f| f.name == *name))
        }
        _ => false,
    });
    match known {
        true => reading.ties.push(tie(item, base, names)),
        false => {
            let message = format!("`{item}` is no part, record or function of the model");
            reading.findings.push(finding(R002, file, line, message));
        }
    }
}

fn tie(item: Path, base: &[String], names: &[String]) -> Tie {
    let mut path = base.to_vec();
    path.extend(names.iter().cloned());
    Tie {
        item,
        at: Location { path },
    }
}

enum Item {
    Fn,
    Type,
    /// A `mod` or an `impl`: what is in it is named from it.
    Scope,
}

/// The item a line declares, and its name: `pub fn div(` is `Fn`,
/// `div`; `impl Point {` and `impl Shape for Point {` are `Scope`,
/// `Point`.
fn item(line: &str) -> Option<(Item, String)> {
    let words: Vec<&str> = line
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .filter(|w| !w.is_empty())
        .collect();
    let modifiers = [
        "pub", "crate", "super", "self", "in", "async", "const", "unsafe", "extern", "C",
    ];
    let mut rest = words.iter().skip_while(|w| modifiers.contains(w));
    let keyword = *rest.next()?;
    let name = rest.next().map(|s| s.to_string());
    match keyword {
        "fn" => Some((Item::Fn, name?)),
        "struct" | "enum" | "trait" | "type" => Some((Item::Type, name?)),
        "mod" => Some((Item::Scope, name?)),
        "impl" => {
            // `impl<T> Shape for Point<T>`: the type after `for`, else the first.
            let after_impl = line[line.find("impl")? + 4..].trim_start();
            let after_impl = match after_impl.starts_with('<') {
                true => after_impl.split_once('>').map(|(_, r)| r)?.trim_start(),
                false => after_impl,
            };
            let target = match after_impl.split_once(" for ") {
                Some((_, t)) => t,
                None => after_impl,
            };
            let name: String = target
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .collect();
            (!name.is_empty()).then_some((Item::Scope, name))
        }
        _ => None,
    }
}

// ---------------------------------------------------------------- cargo test

/// Each `test name ... ok`, `... FAILED` and `... ignored` line of a
/// `cargo test` report, in order. A line where the test's own output
/// came between the name and the result is not read.
pub fn read_cargo_test(report: &str) -> Vec<TestRun> {
    let mut runs = vec![];
    let mut binary = String::new();
    for line in report.lines() {
        if let Some(name) = line.trim().strip_prefix("Running ") {
            let name = name.trim_start_matches("unittests ");
            binary = name.split(" (").next().unwrap_or(name).to_string();
            continue;
        }
        let Some(rest) = line.trim_end().strip_prefix("test ") else {
            continue;
        };
        let Some((name, result)) = rest.rsplit_once(" ... ") else {
            continue;
        };
        // `name - should panic`: the test is `name`.
        let name = name.strip_suffix(" - should panic").unwrap_or(name);
        let outcome = match result {
            "ok" => Outcome::Passed,
            "FAILED" => Outcome::Failed,
            r if r.starts_with("ignored") => Outcome::Ignored,
            _ => continue,
        };
        runs.push(TestRun {
            name: name.to_string(),
            outcome,
            binary: binary.clone(),
        });
    }
    runs
}

/// The rows tests printed with `mitra::check_eq!` and `mitra::check!`,
/// `mitra: row geo::flip (p) -> q`, each a measurement by the test it
/// is under (`---- name stdout ----`, as `--show-output` prints it): of
/// what the system gave, seen, whether the test passed or not. The Rust
/// path is the model's function through its tie: the tie whose place
/// ends with most of the path, `geo::flip` at `src/geo.rs/flip`. A row
/// through no tie is R007; one the reader cannot read or place, R001.
pub fn rows(
    model: &Model,
    ties: &[Tie],
    file: &str,
    report: &str,
) -> (Vec<Measurement>, Vec<ReaderFinding>) {
    let (mut found, mut findings) = (vec![], vec![]);
    let mut test: Option<String> = None;
    for (i, line) in report.lines().enumerate() {
        let line_no = i + 1;
        let trimmed = line.trim();
        if let Some(name) = trimmed
            .strip_prefix("---- ")
            .and_then(|r| r.strip_suffix(" stdout ----"))
        {
            test = Some(name.to_string());
            continue;
        }
        if ["successes:", "failures:"].contains(&trimmed) || trimmed.starts_with("test result: ") {
            test = None;
            continue;
        }
        let Some(row) = trimmed.strip_prefix("mitra: row ") else {
            continue;
        };
        let Some(test) = &test else {
            let message = "a row under no test's output: run `cargo test -- --show-output`, not `--nocapture`, so each row is under its test".to_string();
            findings.push(finding(R001, file, line_no, message));
            continue;
        };
        let Some((rust, values)) = row.split_once(' ') else {
            findings.push(finding(
                R001,
                file,
                line_no,
                format!("`{row}` is no row: `path (inputs) -> output`"),
            ));
            continue;
        };
        let item = match tied_function(model, ties, rust) {
            Ok(item) => item,
            Err(message) => {
                findings.push(finding(R007, file, line_no, message));
                continue;
            }
        };
        let text = format!("measure {item} {values} by \"test:{test}\";");
        match read_store(file, &text, model, ties) {
            Ok((ledger, more)) => {
                found.extend(ledger.measurements);
                findings.extend(
                    more.into_iter()
                        .map(|f| ReaderFinding { line: line_no, ..f }),
                );
            }
            Err(e) => {
                let message = format!("`{values}` is no row of Mitra values: {}", e.message);
                findings.push(finding(R001, file, line_no, message));
            }
        }
    }
    (found, findings)
}

/// The model's function a Rust call path names through a tie on it:
/// the ties whose place ends with the longest end of the path, one.
fn tied_function(model: &Model, ties: &[Tie], rust: &str) -> Result<Path, String> {
    let names: Vec<&str> = rust.split("::").collect();
    let functions: Vec<&Tie> = ties
        .iter()
        .filter(|t| function_at(model, &t.item).is_some_and(|(at, _)| at == t.item))
        .collect();
    for k in (1..=names.len()).rev() {
        let end = &names[names.len() - k..];
        let mut at: Vec<&Tie> = functions
            .iter()
            .copied()
            .filter(|t| t.at.path.len() >= k && t.at.path[t.at.path.len() - k..] == *end)
            .collect();
        at.dedup_by(|a, b| a.item == b.item);
        match at[..] {
            [] => continue,
            [one] => return Ok(one.item.clone()),
            _ => {
                let items: Vec<String> = at.iter().map(|t| format!("`{}`", t.item)).collect();
                return Err(format!(
                    "the row's `{rust}` is tied to {}: name it whole in the call",
                    items.join(" and ")
                ));
            }
        }
    }
    Err(format!(
        "the row's `{rust}` is no function tied to the model: a measurement is always through a tie (`// mitra: implements …` above it)"
    ))
}

/// Each test binary of a `cargo test` report that began and gave no
/// result line: it stopped, crashed or aborted, and what it ran after
/// its last test line is not known (R009). By its `Running …` or
/// `Doc-tests …` line in `file`.
pub fn stopped(file: &str, report: &str) -> Vec<ReaderFinding> {
    let mut found = vec![];
    let mut running: Option<(usize, String)> = None;
    let mut stop = |running: &mut Option<(usize, String)>| {
        if let Some((line, name)) = running.take() {
            let message = format!(
                "the test binary `{name}` stopped before its result: its tests after the last one reported did not run, or ran unseen"
            );
            found.push(finding(R009, file, line, message));
        }
    };
    for (i, line) in report.lines().enumerate() {
        let line = line.trim();
        let name = line
            .strip_prefix("Running ")
            .or_else(|| line.strip_prefix("Doc-tests "));
        if let Some(name) = name {
            stop(&mut running);
            let name = name.trim_start_matches("unittests ");
            let name = name.split(" (").next().unwrap_or(name);
            running = Some((i + 1, name.to_string()));
        } else if line.starts_with("test result: ") {
            running = None;
        }
    }
    stop(&mut running);
    found
}

/// What the tests measured, from what ran. A test that passed saw its
/// row; one that failed saw something else at its inputs. `points` are
/// from `read_rust`; `ties` all the ties known, the model's and the
/// code's: a measurement is always through a tie.
pub fn measurements(
    model: &Model,
    ties: &[Tie],
    points: &[TestPoint],
    runs: &[TestRun],
) -> (Vec<Measurement>, Vec<ReaderFinding>) {
    let (mut found, mut findings) = (vec![], vec![]);
    let mut used = vec![false; runs.len()];
    for point in points {
        let matching: Vec<usize> = (0..runs.len())
            .filter(|&i| {
                runs[i].name == point.test || runs[i].name.ends_with(&format!("::{}", point.test))
            })
            .collect();
        let run = match matching[..] {
            [i] => i,
            [] => {
                let message = format!("the test `{}` is not in the report", point.test);
                findings.push(finding(R005, &point.file, point.line, message));
                continue;
            }
            _ => {
                let names: Vec<&str> = matching.iter().map(|&i| runs[i].name.as_str()).collect();
                let message = format!("the test `{}` is any of {}", point.test, names.join(", "));
                findings.push(finding(R005, &point.file, point.line, message));
                continue;
            }
        };
        used[run] = true;
        let seen = match runs[run].outcome {
            Outcome::Passed => true,
            Outcome::Failed => false,
            Outcome::Ignored => {
                let message = format!(
                    "the test `{}` was ignored: it measured nothing",
                    runs[run].name
                );
                findings.push(finding(R005, &point.file, point.line, message));
                continue;
            }
        };
        let by = format!("test:{}", runs[run].name);
        match measured(
            model,
            ties,
            &point.function,
            &point.takes,
            &point.gives,
            seen,
            1,
            by,
        ) {
            Ok(m) => found.push(m),
            Err((code, message)) => findings.push(finding(code, &point.file, point.line, message)),
        }
    }
    for (run, used) in runs.iter().zip(used) {
        if !used && run.outcome != Outcome::Ignored {
            let message = format!("the test `{}` measures nothing the reader knows", run.name);
            findings.push(finding(R004, "cargo test", 0, message));
        }
    }
    (found, findings)
}

/// The ledger of a project, read whole: the ties of its Rust files, the
/// tests they measure, the results of `cargo test` (its name and text)
/// with the rows its tests printed, and the store.
/// `files` are each Rust file's path and text.
pub fn ledger(
    model: &Model,
    files: &[(String, String)],
    report: Option<(&str, &str)>,
    store: Option<(&str, &str)>,
) -> Result<(Vec<Tie>, Ledger, Vec<ReaderFinding>), ParseError> {
    let (mut ties, mut points, mut findings) = (model.ties.clone(), vec![], vec![]);
    for (file, source) in files {
        let reading = read_rust(file, source, model);
        ties.extend(reading.ties);
        points.extend(reading.points);
        findings.extend(reading.findings);
    }
    let mut ledger = Ledger::default();
    let runs = report.map(|(_, r)| read_cargo_test(r)).unwrap_or_default();
    if let Some((report_file, report)) = report {
        let (found, more) = measurements(model, &ties, &points, &runs);
        ledger.measurements.extend(found);
        findings.extend(more);
        let (found, more) = rows(model, &ties, report_file, report);
        // A test that printed a row measured: no R004 for it.
        let rowed: BTreeSet<String> = found
            .iter()
            .map(|m| {
                format!(
                    "the test `{}` measures nothing the reader knows",
                    m.by.trim_start_matches("test:")
                )
            })
            .collect();
        findings.retain(|f| !(f.code == R004 && rowed.contains(&f.message)));
        ledger.measurements.extend(found);
        findings.extend(more);
    }
    if let Some((file, source)) = store {
        let (read, more) = read_store_seeing(file, source, model, &ties, Some(&runs))?;
        ledger.measurements.extend(read.measurements);
        ledger.reviews.extend(read.reviews);
        findings.extend(more);
    }
    Ok((ties, ledger, findings))
}
