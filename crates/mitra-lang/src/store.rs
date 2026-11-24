//! The text of evidence, in Mitra's own syntax, read against a model
//! (`docs/design/reader.md`): a store of what people measured and
//! reviewed, the `mitra:` comments of code, and rows. Text only: no file
//! is read here (mitra-store reads the store's file, mitra-rust the
//! code and what the tests printed).
//!
//! Nothing given is dropped quietly: what cannot be used is a
//! `ReaderFinding`, with a code of suit `R` and where it is.

use crate::catalogue::{Code, R006, R007, R008};
use crate::model::{Function, Model, Path, Tie};
use crate::parse::{lex, type_of, ParseError, Parser, Pos};
use crate::print;
use crate::simulation::Value;

// ---------------------------------------------------------------- the ledger

pub use crate::evidence::{Ledger, Measurement, Review};

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

pub fn finding(code: Code, file: &str, line: usize, message: impl Into<String>) -> ReaderFinding {
    ReaderFinding {
        code,
        file: file.to_string(),
        line,
        message: message.into(),
    }
}

/// A `mitra:` comment of code: a tie, or a row a test measures.
#[derive(Clone, Debug, PartialEq)]
pub enum Comment {
    Implements(Path),
    Measures(Path, Vec<Value>, Value),
}

/// `implements a::b` or `measures a::f (6, 2) -> 3`, after `// mitra:`.
/// Main's `evidence for …` is not read here: it is a finding.
pub fn read_comment(said: &str, model: &Model) -> Result<Comment, String> {
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
        return Ok(Comment::Implements(path));
    }
    if p.eat_word("measures") {
        let path = Path {
            names: p.raw_path().map_err(shown)?,
        };
        let (takes, gives) = p.row().map_err(shown)?;
        p.end().map_err(shown)?;
        return Ok(Comment::Measures(path, takes, gives));
    }
    Err(format!("`mitra: {said}` is no comment the reader knows: `implements a::b` or `measures a::f (6, 2) -> 3`"))
}

// ---------------------------------------------------------------- tests run

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
    /// The binary it ran in, by its `Running` line: `tests/model.rs`,
    /// `src/lib.rs` for unit tests; empty when the report says none.
    pub binary: String,
}

/// The function at `path`: whole, or its name alone when only one
/// function has it.
pub fn function_at<'m>(model: &'m Model, path: &Path) -> Option<(Path, &'m Function)> {
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
pub fn measured(
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
    // A variant whose parameters the row does not tell (`None`) takes
    // them from where it goes.
    let takes: Vec<Value> = takes
        .iter()
        .zip(
            f.takes
                .iter()
                .map(|(_, t)| Some(t))
                .chain(std::iter::repeat(None)),
        )
        .map(|(v, t)| match t {
            Some(t) => crate::parse::settle_value(v.clone(), t),
            None => v.clone(),
        })
        .collect();
    let gives = &crate::parse::settle_value(gives.clone(), &f.gives.1);
    let takes = takes.as_slice();
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
/// (`docs/design/reader.md`):
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
    read_store_seeing(file, source, model, ties, None)
}

/// `read_store`, each review that sees a test kept only where `runs`
/// say it passed: one that failed, was ignored or did not run is
/// dropped, and R008 says so. `None` reads no test.
pub fn read_store_seeing(
    file: &str,
    source: &str,
    model: &Model,
    ties: &[Tie],
    runs: Option<&[TestRun]>,
) -> Result<(Ledger, Vec<ReaderFinding>), ParseError> {
    let context = crate::parse::Context::of(model);
    let mut p = Parser::new(lex(source)?, Some(&context));
    let (mut ledger, mut findings) = (Ledger::default(), vec![]);
    let items = items(model, ties);
    let index = crate::model::Index::of(model);
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
            // `from`, alone or with the list it is from.
            let from = match p.eat_word("from") {
                false => None,
                true if p.eat("(") => Some(Some(p.list(")", |p| {
                    Ok(Path {
                        names: p.raw_path()?,
                    })
                })?)),
                true => Some(None),
            };
            let holds = match (p.eat_word("holds"), p.eat_word("fails")) {
                (true, _) => true,
                (_, true) => false,
                _ => return p.expected("`holds` or `fails`"),
            };
            p.expect_word("by")?;
            let by = p.string()?;
            let mut seeing = vec![];
            if p.eat_word("seeing") {
                seeing.push(p.string()?);
                while p.eat(",") {
                    seeing.push(p.string()?);
                }
            }
            p.expect(";")?;
            let review = Review {
                prop,
                from: None,
                holds,
                by,
                seeing,
            };
            match reviewed(&index, &items, review, from).and_then(|r| seen(r, runs)) {
                Ok(r) => ledger.reviews.push(r),
                Err((code, message)) => findings.push(finding(code, file, line, message)),
            }
        } else {
            p.end()
                .or_else(|_| p.expected("`measure`, `review` or the end"))?;
            break;
        }
    }
    Ok((ledger, findings))
}

/// The review read, made whole, or why it is of nothing: its piece, and
/// the link it names, `from` alone where the piece has one link.
fn reviewed(
    index: &crate::model::Index,
    items: &[Path],
    mut review: Review,
    from: Option<Option<Vec<Path>>>,
) -> Result<Review, (Code, String)> {
    let prop = &review.prop;
    if !items.contains(prop) {
        let message =
            format!("`{prop}` is no claim, rule, require, ensure or tied item of the model");
        return Err((R006, message));
    }
    let links = crate::evidence::links(index, prop);
    let shown = |list: &[Path]| {
        let list: Vec<String> = list.iter().map(|p| p.to_string()).collect();
        format!("({})", list.join(", "))
    };
    review.from = match from {
        None => None,
        Some(None) => match &links[..] {
            [one] => Some(one.clone()),
            [] => return Err((R006, format!("`{prop}` is from nothing: no link to review"))),
            more => {
                let lists: Vec<String> = more.iter().map(|l| shown(l)).collect();
                let message = format!(
                    "`{prop}` is from {}: name the one reviewed, `from {}`",
                    lists.join(" | "),
                    lists[0]
                );
                return Err((R006, message));
            }
        },
        Some(Some(list)) if links.contains(&list) => Some(list),
        Some(Some(list)) => {
            let message = match links.is_empty() {
                true => format!("`{prop}` is from nothing: no link to review"),
                false => {
                    let lists: Vec<String> = links.iter().map(|l| shown(l)).collect();
                    format!(
                        "`{prop}` is not from {}: it is from {}",
                        shown(&list),
                        lists.join(" | ")
                    )
                }
            };
            return Err((R006, message));
        }
    };
    for test in &review.seeing {
        if !test.strip_prefix("test:").is_some_and(|n| !n.is_empty()) {
            let message = format!("`{test}` is no test: a review sees `test:name`");
            return Err((R008, message));
        }
    }
    Ok(review)
}

/// The review, where each test it sees passed in `runs`; why not, if not.
fn seen(review: Review, runs: Option<&[TestRun]>) -> Result<Review, (Code, String)> {
    let Some(runs) = runs else {
        return Ok(review);
    };
    for test in &review.seeing {
        let name = test.trim_start_matches("test:");
        // `tests/model.rs::name`: the test of that binary.
        let (binary, name) = match name.split_once(".rs::") {
            Some((b, n)) => (Some(format!("{b}.rs")), n),
            None => (None, name),
        };
        let matching: Vec<&TestRun> = runs
            .iter()
            .filter(|r| r.name == name || r.name.ends_with(&format!("::{name}")))
            .filter(|r| binary.as_ref().is_none_or(|b| &r.binary == b))
            .collect();
        let why = match matching[..] {
            [] => format!("the test `{name}` is not in the report"),
            [run] => match run.outcome {
                Outcome::Passed => continue,
                Outcome::Failed => format!("the test `{}` failed", run.name),
                Outcome::Ignored => format!("the test `{}` was ignored", run.name),
            },
            _ => {
                let names: Vec<String> = matching
                    .iter()
                    .map(|r| format!("`{}::{}`", r.binary, r.name))
                    .collect();
                format!(
                    "the test `{name}` is any of {}: name its binary",
                    names.join(", ")
                )
            }
        };
        let message = format!(
            "{why}: the review of `{}` by `{}` counts only while it passes",
            review.said(),
            review.by
        );
        return Err((R008, message));
    }
    Ok(review)
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
        let seeing: Vec<String> = r.seeing.iter().map(|t| format!("{t:?}")).collect();
        let seeing = match seeing.is_empty() {
            true => String::new(),
            false => format!(" seeing {}", seeing.join(", ")),
        };
        out.push_str(&format!(
            "review {} {said} by {:?}{seeing};\n",
            r.said(),
            r.by
        ));
    }
    out
}
