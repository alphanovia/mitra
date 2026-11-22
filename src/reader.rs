//! What comes from outside the model (`docs/reader.md`): ties and tests
//! read from Rust code, the results of `cargo test`, and a store of what
//! people measured and reviewed, written by hand. Together, the ledger.
//!
//! - `read_rust(file, source, model)`: each `// mitra: implements a::b`
//!   above a `fn`, `struct` or `enum` is a tie of `a::b` at the file,
//!   then the modules and `impl` around the item, then its name:
//!   `src/lib.rs/div`, `src/geo.rs/Point/flip`. Each `// mitra: measures
//!   a::f (6, 2) -> 3` above a `#[test]` says what that test measures.
//! - `read_cargo_test(report)`: each `test name ... ok | FAILED |
//!   ignored` line.
//! - `measurements(tests, runs, by)`: a test that passed saw its row
//!   (`seen`), one that failed saw something else (not `seen`).
//! - `read_store(source, model)`: `measure …;` and `review …;` lines.
//!
//! Nothing given is dropped quietly: what cannot be used is a
//! `ReaderFinding`, with a code of suit `R` and where it is.
//!
//! The ledger types are `Measurement`, `Review` and `Ledger`, the shapes
//! the evidence layer fixes (`src/evidence.rs`, to come): here until it
//! lands, then taken from there.

use crate::catalogue::{Code, R001, R002, R003, R004, R005, R006, R007};
use crate::location::Location;
use crate::model::{Function, Model, Path, Tie};
use crate::parse::{lex, type_of, ParseError, Parser, Pos};
use crate::print;
use crate::simulation::Value;

// ---------------------------------------------------------------- the ledger

/// What a tied function was seen to do, `count` times: at its inputs,
/// each by its path (`calc::div::a`), it gave its outputs (`calc::div::q`)
/// when `seen`, and something else when not.
#[derive(Clone, Debug, PartialEq)]
pub struct Measurement {
    pub inputs: Vec<(Path, Value)>,
    pub outputs: Vec<(Path, Value)>,
    pub seen: bool,
    pub count: u64,
    pub by: String,
}

/// Someone says the claim, rule, require, ensure or tied item at `prop`
/// holds, or not.
#[derive(Clone, Debug, PartialEq)]
pub struct Review {
    pub prop: Path,
    pub holds: bool,
    pub by: String,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Ledger {
    pub measurements: Vec<Measurement>,
    pub reviews: Vec<Review>,
}

/// What the reader was given and cannot use, at `file:line`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReaderFinding {
    pub code: Code,
    pub file: String,
    pub line: usize,
    pub message: String,
}

/// `finding[R002]: …`, then where, as rustc writes it.
impl std::fmt::Display for ReaderFinding {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "finding[{}]: {}\n --> {}:{}",
            self.code, self.message, self.file, self.line
        )
    }
}

fn finding(code: Code, file: &str, line: usize, message: impl Into<String>) -> ReaderFinding {
    ReaderFinding {
        code,
        file: file.to_string(),
        line,
        message: message.into(),
    }
}

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
enum Pending {
    Implements(Path),
    Measures(Path, Vec<Value>, Value),
}

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
            match directive(said.trim(), model) {
                Ok(Some(p)) => pending.push((p, line)),
                Ok(None) => {}
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

/// `implements a::b` or `measures a::f (6, 2) -> 3`, after `// mitra:`.
/// Main's `evidence for …` is not read here: `Ok(None)` is never given
/// for it, so it is a finding.
fn directive(said: &str, model: &Model) -> Result<Option<Pending>, String> {
    let shown = |e: ParseError| {
        format!(
            "{}, at column {} of what follows `mitra:`",
            e.message, e.at.column
        )
    };
    let tokens = lex(said).map_err(shown)?;
    let context = crate::parse::Context::of(model);
    let mut p = Parser::new(tokens, Some(&context));
    if p.eat_word("implements") {
        let path = Path {
            names: p.raw_path().map_err(shown)?,
        };
        p.end().map_err(shown)?;
        return Ok(Some(Pending::Implements(path)));
    }
    if p.eat_word("measures") {
        let path = Path {
            names: p.raw_path().map_err(shown)?,
        };
        let (takes, gives) = p.row().map_err(shown)?;
        p.end().map_err(shown)?;
        return Ok(Some(Pending::Measures(path, takes, gives)));
    }
    Err(format!("`mitra: {said}` is no comment the reader knows: `implements a::b` or `measures a::f (6, 2) -> 3`"))
}

// ---------------------------------------------------------------- cargo test

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    Passed,
    Failed,
    Ignored,
}

/// A test the report says ran, by the name cargo gives it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TestRun {
    pub name: String,
    pub outcome: Outcome,
}

/// Each `test name ... ok`, `... FAILED` and `... ignored` line of a
/// `cargo test` report, in order. A line where the test's own output
/// came between the name and the result is not read.
pub fn read_cargo_test(report: &str) -> Vec<TestRun> {
    let mut runs = vec![];
    for line in report.lines() {
        let Some(rest) = line.trim_end().strip_prefix("test ") else {
            continue;
        };
        let Some((name, result)) = rest.rsplit_once(" ... ") else {
            continue;
        };
        let outcome = match result {
            "ok" => Outcome::Passed,
            "FAILED" => Outcome::Failed,
            r if r.starts_with("ignored") => Outcome::Ignored,
            _ => continue,
        };
        runs.push(TestRun {
            name: name.to_string(),
            outcome,
        });
    }
    runs
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

/// The function at `path`: whole, or its name alone when only one
/// function has it.
fn function_at<'m>(model: &'m Model, path: &Path) -> Option<(Path, &'m Function)> {
    let all = model.parts.iter().flat_map(|p| {
        p.functions
            .iter()
            .map(move |f| (Path::to(&[&p.name, &f.name]), f))
    });
    let exact: Vec<_> = all.clone().filter(|(at, _)| at == path).collect();
    if let [one] = &exact[..] {
        return Some(one.clone());
    }
    let named: Vec<_> = all
        .filter(|(at, _)| path.names.len() == 1 && at.names[1] == path.names[0])
        .collect();
    match &named[..] {
        [one] => Some(one.clone()),
        _ => None,
    }
}

#[allow(clippy::too_many_arguments)]
fn measured(
    model: &Model,
    ties: &[Tie],
    path: &Path,
    takes: &[Value],
    gives: &Value,
    seen: bool,
    count: u64,
    by: String,
) -> Result<Measurement, (Code, String)> {
    let Some((at, f)) = function_at(model, path) else {
        return Err((
            R006,
            format!("`{path}` is no function of the model, or names several"),
        ));
    };
    if !ties.iter().any(|t| t.item == at) {
        return Err((
            R007,
            format!("`{at}` is tied to nothing: a measurement is always through a tie"),
        ));
    }
    let row = print::print_row(takes, gives);
    let fits = takes.len() == f.takes.len()
        && takes
            .iter()
            .zip(&f.takes)
            .all(|(v, (_, t))| type_of(v) == *t)
        && type_of(gives) == f.gives.1;
    if !fits {
        let types: Vec<String> = f.takes.iter().map(|(_, t)| print::type_(t)).collect();
        let message = format!(
            "`{row}` is not of `{at}`, which takes ({}) and gives {}",
            types.join(", "),
            print::type_(&f.gives.1)
        );
        return Err((R006, message));
    }
    let inputs = f
        .takes
        .iter()
        .zip(takes)
        .map(|((n, _), v)| (at.then(n), v.clone()))
        .collect();
    Ok(Measurement {
        inputs,
        outputs: vec![(at.then(&f.gives.0), gives.clone())],
        seen,
        count,
        by,
    })
}

// ---------------------------------------------------------------- the store

/// A store of what people measured and reviewed, written by hand
/// (`docs/reader.md`):
///
/// ```text
/// measure calc::div (6, 2) -> 3 by "human:ana";
/// measure stone::drop (5.0) -> 9.9 count 3 by "human:ana";
/// measure calc::div (7, 0) -> 0 failed by "human:ana";
/// review stone::air holds by "human:ana";
/// review stone::drop::slows fails by "agent:anthropic/opus/s-1";
/// ```
///
/// A text the store cannot read is a `ParseError`; a line read but not
/// of the model, a finding. `ties` are all the ties known.
pub fn read_store(
    file: &str,
    source: &str,
    model: &Model,
    ties: &[Tie],
) -> Result<(Ledger, Vec<ReaderFinding>), ParseError> {
    let context = crate::parse::Context::of(model);
    let mut p = Parser::new(lex(source)?, Some(&context));
    let (mut ledger, mut findings) = (Ledger::default(), vec![]);
    let items = items(model, ties);
    loop {
        let Pos { line, .. } = p.pos();
        if p.eat_word("measure") {
            let path = Path {
                names: p.raw_path()?,
            };
            let (takes, gives) = p.row()?;
            let count = match p.eat_word("count") {
                true => p.number()? as u64,
                false => 1,
            };
            let seen = !p.eat_word("failed");
            p.expect_word("by")?;
            let by = p.string()?;
            p.expect(";")?;
            match measured(model, ties, &path, &takes, &gives, seen, count, by) {
                Ok(m) => ledger.measurements.push(m),
                Err((code, message)) => findings.push(finding(code, file, line, message)),
            }
        } else if p.eat_word("review") {
            let prop = Path {
                names: p.raw_path()?,
            };
            let holds = match (p.eat_word("holds"), p.eat_word("fails")) {
                (true, _) => true,
                (_, true) => false,
                _ => return p.expected("`holds` or `fails`"),
            };
            p.expect_word("by")?;
            let by = p.string()?;
            p.expect(";")?;
            match items.contains(&prop) {
                true => ledger.reviews.push(Review { prop, holds, by }),
                false => {
                    let message = format!(
                        "`{prop}` is no claim, rule, require, ensure or tied item of the model"
                    );
                    findings.push(finding(R006, file, line, message));
                }
            }
        } else {
            p.end()
                .or_else(|_| p.expected("`measure`, `review` or the end"))?;
            break;
        }
    }
    Ok((ledger, findings))
}

/// What a review can be of: a claim, rule, require or ensure, or a tied item.
fn items(model: &Model, ties: &[Tie]) -> Vec<Path> {
    let mut items: Vec<Path> = ties.iter().map(|t| t.item.clone()).collect();
    for part in &model.parts {
        let here = Path::to(&[&part.name]);
        items.extend(part.requires.iter().map(|q| here.then(&q.name)));
        items.extend(part.ensures.iter().map(|e| here.then(&e.name)));
        for r in &part.records {
            let at = here.then(&r.name);
            items.extend(r.claims.iter().map(|c| at.then(&c.name)));
            items.extend(r.rules.iter().map(|c| at.then(&c.name)));
        }
        for f in &part.functions {
            let at = here.then(&f.name);
            items.extend(f.claims.iter().map(|c| at.then(&c.name)));
            items.extend(f.rules.iter().map(|c| at.then(&c.name)));
        }
    }
    items
}

/// The store's text of a ledger: `read_store` reads it back.
pub fn print_store(ledger: &Ledger) -> String {
    let mut out = String::new();
    for m in &ledger.measurements {
        let function = m.inputs.first().or(m.outputs.first()).map(|(p, _)| Path {
            names: p.names[..p.names.len().saturating_sub(1)].to_vec(),
        });
        let function = function.map(|p| p.to_string()).unwrap_or_default();
        let takes: Vec<Value> = m.inputs.iter().map(|(_, v)| v.clone()).collect();
        let gives = m
            .outputs
            .first()
            .map(|(_, v)| v.clone())
            .unwrap_or(Value::Truth(true));
        let count = match m.count {
            1 => String::new(),
            n => format!(" count {n}"),
        };
        let failed = if m.seen { "" } else { " failed" };
        let row = print::print_row(&takes, &gives);
        out.push_str(&format!(
            "measure {function} {row}{count}{failed} by {:?};\n",
            m.by
        ));
    }
    for r in &ledger.reviews {
        let said = if r.holds { "holds" } else { "fails" };
        out.push_str(&format!("review {} {said} by {:?};\n", r.prop, r.by));
    }
    out
}

/// The ledger of a project, read whole: the ties of its Rust files, the
/// tests they measure, the results of `cargo test`, and the store.
/// `files` are each Rust file's path and text.
pub fn ledger(
    model: &Model,
    files: &[(String, String)],
    report: Option<&str>,
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
    if let Some(report) = report {
        let (found, more) = measurements(model, &ties, &points, &read_cargo_test(report));
        ledger.measurements.extend(found);
        findings.extend(more);
    }
    if let Some((file, source)) = store {
        let (read, more) = read_store(file, source, model, &ties)?;
        ledger.measurements.extend(read.measurements);
        ledger.reviews.extend(read.reviews);
        findings.extend(more);
    }
    Ok((ties, ledger, findings))
}
