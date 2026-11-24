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
//!   ignored` line, from every test binary (unit, `tests/`, examples,
//!   benches, doc tests); `rows`, each `mitra: row` a test printed;
//!   `stopped`, each test binary that gave no result; `unread`, each
//!   test Mitra could not read.
//! - `measurements(tests, runs, by)`: a test that passed saw its row
//!   (`seen`), one that failed saw something else (not `seen`).
//!
//! The text of a store and of rows is `mitra_lang::store`, re-exported
//! here; nothing given is dropped quietly.

use mitra_lang::catalogue::{R001, R002, R003, R004, R005, R007, R009, R010};
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

/// The test binary a `Running …` or `Doc-tests …` line begins, as
/// Mitra names it: its file, after its crate when the report says it.
/// `Running unittests src/lib.rs (target/debug/deps/geo-1a2b)` is
/// `geo/src/lib.rs`; `Running tests/model.rs (…)` is `tests/model.rs`
/// (an integration test's binary bears its own name, not its crate's);
/// `Doc-tests geo` is `geo/doc`.
fn binary_of(line: &str) -> Option<String> {
    let line = line.trim();
    if let Some(krate) = line.strip_prefix("Doc-tests ") {
        return Some(format!("{}/doc", krate.trim()));
    }
    let name = line.strip_prefix("Running ")?;
    let (unit, name) = match name.strip_prefix("unittests ") {
        Some(n) => (true, n),
        None => (false, name),
    };
    let (file, deps) = match name.split_once(" (") {
        Some((f, d)) => (f, Some(d.trim_end_matches(')'))),
        None => (name, None),
    };
    let krate = deps
        .filter(|_| unit)
        .and_then(|d| d.rsplit(['/', '\\']).next())
        .and_then(|d| d.rsplit_once('-').map(|(k, _)| k));
    Some(match krate {
        Some(k) => format!("{k}/{file}"),
        None => file.to_string(),
    })
}

/// Each `test name ... ok`, `... FAILED` and `... ignored` line of a
/// `cargo test` report, in order, from every binary it ran: unit tests
/// in `src/`, `tests/`, examples, benches and doc tests. A test line
/// it cannot read is in `unread` (R010), never dropped.
pub fn read_cargo_test(report: &str) -> Vec<TestRun> {
    read_test_lines(report).0
}

fn read_test_lines(report: &str) -> (Vec<TestRun>, Vec<(usize, String)>) {
    let (mut runs, mut unread) = (vec![], vec![]);
    let mut binary = String::new();
    for (i, line) in report.lines().enumerate() {
        if let Some(b) = binary_of(line) {
            binary = b;
            continue;
        }
        let Some(rest) = line.trim_end().strip_prefix("test ") else {
            continue;
        };
        if rest.starts_with("result: ") {
            continue;
        }
        let read = rest.rsplit_once(" ... ").and_then(|(name, result)| {
            let outcome = match result {
                "ok" => Outcome::Passed,
                "FAILED" => Outcome::Failed,
                r if r.starts_with("ignored") => Outcome::Ignored,
                _ => return None,
            };
            Some((name, outcome))
        });
        let Some((name, outcome)) = read else {
            unread.push((i + 1, line.trim().to_string()));
            continue;
        };
        // `name - should panic`: the test is `name`.
        let name = name.strip_suffix(" - should panic").unwrap_or(name);
        runs.push(TestRun {
            name: name.to_string(),
            outcome,
            binary: binary.clone(),
        });
    }
    (runs, unread)
}

/// Each test of a `cargo test` report that Mitra could not read (R010):
/// a `test` line with no `ok`, `FAILED` or `ignored` after ` ... `
/// (the test's own output came between, or a form Mitra doesn't know),
/// and each binary whose `test result:` counts more tests than the
/// lines read. Those tests measure nothing and see nothing.
pub fn unread(file: &str, report: &str) -> Vec<ReaderFinding> {
    let (_, lines) = read_test_lines(report);
    let mut found: Vec<ReaderFinding> = lines
        .iter()
        .map(|(line, text)| {
            let message = format!(
                "`{text}` is a test line Mitra cannot read: it gives no `ok`, `FAILED` or `ignored` after ` ... `"
            );
            finding(R010, file, *line, message)
        })
        .collect();
    let (mut binary, mut read) = (String::new(), 0usize);
    for (i, line) in report.lines().enumerate() {
        if let Some(b) = binary_of(line) {
            (binary, read) = (b, 0);
            continue;
        }
        let Some(rest) = line.trim().strip_prefix("test ") else {
            continue;
        };
        let Some(result) = rest.strip_prefix("result: ") else {
            if rest
                .rsplit_once(" ... ")
                .is_some_and(|(_, r)| r == "ok" || r == "FAILED" || r.starts_with("ignored"))
            {
                read += 1;
            }
            continue;
        };
        // `test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 2 filtered out; …`
        let counted: usize = result
            .split(';')
            .filter_map(|part| {
                let part = part
                    .trim()
                    .trim_start_matches(|c: char| !c.is_ascii_digit());
                let (n, what) = part.split_once(' ')?;
                ["passed", "failed", "ignored", "measured"]
                    .contains(&what.trim())
                    .then(|| n.parse::<usize>().ok())?
            })
            .sum();
        if counted > read {
            let message = format!(
                "the test binary `{binary}` ran {counted} tests and Mitra read {read}: the others measure nothing and see nothing"
            );
            found.push(finding(R010, file, i + 1, message));
        }
        read = 0;
    }
    found
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
        if let Some(name) = binary_of(line) {
            stop(&mut running);
            running = Some((i + 1, name));
        } else if line.starts_with("test result: ") {
            running = None;
        }
    }
    stop(&mut running);
    found
}

/// Where a test of `file` runs: its crate's folder (`mitra-lang` of
/// `crates/mitra-lang/src/numbers.rs`, when the file is in one), the
/// binary's file (`src` for any unit test binary, `tests/model.rs`,
/// `examples/x.rs`, `benches/x.rs`) and the test's module path from
/// that binary's root (`numbers` for `src/numbers.rs`).
fn place_of(file: &str) -> (Option<&str>, String, String) {
    let parts: Vec<&str> = file.split('/').collect();
    let at = parts.iter().rposition(|p| *p == "src").or_else(|| {
        parts
            .iter()
            .rposition(|p| ["tests", "examples", "benches"].contains(p))
    });
    let Some(at) = at else {
        return (None, String::new(), String::new());
    };
    let krate = at.checked_sub(1).map(|k| parts[k]);
    let inside: Vec<&str> = parts[at + 1..].to_vec();
    if parts[at] != "src" {
        // `tests/x.rs` is a binary; `tests/x/y.rs` a module of `tests/x/main.rs`.
        let binary = format!("{}/{}", parts[at], inside[0]);
        let module = inside[1..]
            .iter()
            .map(|m| m.trim_end_matches(".rs"))
            .filter(|m| *m != "mod" && *m != "main")
            .collect::<Vec<_>>()
            .join("::");
        return (krate, binary, module);
    }
    let mut module: Vec<&str> = inside.iter().map(|m| m.trim_end_matches(".rs")).collect();
    if module.first() == Some(&"bin") {
        module.drain(..2.min(module.len()));
    }
    module.retain(|m| *m != "mod");
    if module.len() == 1 && (module[0] == "lib" || module[0] == "main") {
        module.clear();
    }
    (krate, "src".to_string(), module.join("::"))
}

/// The runs that are `point`'s test. Its name ends the run's; where
/// several do, the one of its module, binary and crate, as far as the
/// report says them. More than one left is for the caller to say.
fn runs_of(point: &TestPoint, runs: &[TestRun]) -> Vec<usize> {
    let mut matching: Vec<usize> = (0..runs.len())
        .filter(|&i| {
            runs[i].name == point.test || runs[i].name.ends_with(&format!("::{}", point.test))
        })
        .collect();
    let (krate, binary, module) = place_of(&point.file);
    let full = match module.as_str() {
        "" => point.test.clone(),
        m => format!("{m}::{}", point.test),
    };
    let norm = |s: &str| s.replace('-', "_");
    let file_of = |b: &str| -> String {
        // `geo/src/lib.rs` and `src/lib.rs` are both in `src`.
        let b = b
            .split_once('/')
            .filter(|(k, _)| {
                !k.contains('.')
                    && *k != "src"
                    && *k != "tests"
                    && *k != "examples"
                    && *k != "benches"
            })
            .map_or(b, |(_, rest)| rest);
        b.to_string()
    };
    let narrow = |matching: &mut Vec<usize>, keep: &dyn Fn(&TestRun) -> bool| {
        let kept: Vec<usize> = matching
            .iter()
            .copied()
            .filter(|&i| keep(&runs[i]))
            .collect();
        if !kept.is_empty() {
            *matching = kept;
        }
    };
    if matching.len() > 1 {
        narrow(&mut matching, &|r| r.name == full);
    }
    if matching.len() > 1 && !binary.is_empty() {
        narrow(&mut matching, &|r| {
            let f = file_of(&r.binary);
            if binary == "src" {
                f.starts_with("src/")
            } else {
                f == binary
            }
        });
    }
    if let (true, Some(k)) = (matching.len() > 1, krate) {
        narrow(&mut matching, &|r| {
            r.binary
                .split_once('/')
                .is_some_and(|(c, _)| norm(c) == norm(k))
        });
    }
    matching
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
        let matching = runs_of(point, runs);
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
