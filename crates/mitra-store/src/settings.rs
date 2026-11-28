//! `mitra.toml`, at the project's root: each layer's settings, a section
//! each (the user's choice, 2026-11-24 12:02), by the names the code
//! gives them. An empty file marks the root and changes nothing.
//!
//! ```toml
//! [parse]
//! depth = 2000            # twice budget's depth when not said
//! [agree]
//! points = 64
//! splits = 4
//! length = 3
//! height = 3
//! [budget]                # agree's and explain's
//! steps = 100000
//! depth = 1000
//! [explain]
//! rounds = 64
//! cases = 1024
//! rests = 1024
//! [numbers]
//! base_rate = 0.5
//! prior_weight = 2.0
//! checker = 1000.0
//! link = 0.5              # a link no one reviewed
//! [numbers.voices]
//! "human:ana" = 2.0
//! [blame]
//! diagnoses = 10
//! tries = 100000
//! [symbolic]              # mitra mc and mitra search
//! runs = 512
//! length = 2              # sequences tried up to this length
//! shown = 3
//! depth = 3               # steps of mitra mc --step
//! spread = 0              # paths this many cases off the first run first
//! reach = 64              # mitra search tries ±4, ±8… up to ±2^reach
//! [test]                  # what explain and blame run with no --report
//! command = ["cargo", "test", "--no-fail-fast", "--", "--show-output"]
//! ```
//!
//! Nothing given is left unread: a section, a key or a value Mitra does
//! not take is P015, where it is. A count is a whole number; a weight is
//! written with its point (`2.0`): no number becomes another.

use mitra_lang::catalogue::P015;
use mitra_lang::parse::{ParseError, Pos};
use std::ops::Range;
use toml::de::{DeTable, DeValue};
use toml::Spanned;

/// Every layer's settings.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Settings {
    pub parse: mitra_lang::parse::Settings,
    pub agree: mitra_lang::agree::Settings,
    pub explain: mitra_lang::explain::Settings,
    pub numbers: mitra_lang::numbers::Settings,
    pub blame: mitra_lang::blame::Settings,
    pub test: Test,
    /// `mitra mc` and `mitra search` (dry run).
    pub search: mitra_lang::search::Settings,
}

/// What `mitra explain` and `mitra blame` run at the root when no
/// report is given: the program, then its arguments. `--show-output` is
/// what prints the rows of the tests that pass.
#[derive(Clone, Debug, PartialEq)]
pub struct Test {
    pub command: Vec<String>,
}

impl Default for Test {
    fn default() -> Test {
        let command = ["cargo", "test", "--no-fail-fast", "--", "--show-output"];
        Test {
            command: command.iter().map(|a| a.to_string()).collect(),
        }
    }
}

/// The sections, and their keys.
const SECTIONS: &[(&str, &[&str])] = &[
    ("parse", &["depth"]),
    ("agree", &["points", "splits", "length", "height"]),
    ("budget", &["steps", "depth"]),
    ("explain", &["rounds", "cases", "rests"]),
    (
        "numbers",
        &["base_rate", "prior_weight", "checker", "link", "voices"],
    ),
    ("blame", &["diagnoses", "tries"]),
    ("symbolic", &["runs", "length", "shown", "depth", "spread", "reach"]),
    ("test", &["command"]),
];

type Result<T> = std::result::Result<T, ParseError>;

/// The settings `text`, a `mitra.toml`, gives.
pub fn read(text: &str) -> Result<Settings> {
    let at = |span: Range<usize>| pos(text, span.start);
    let fail = |span: Range<usize>, message: String| ParseError {
        code: P015,
        at: at(span),
        message,
    };
    let table = DeTable::parse(text).map_err(|e| {
        let message = e.message().trim_end().to_string();
        fail(e.span().unwrap_or(0..0), format!("not TOML: {message}"))
    })?;
    let mut s = Settings::default();
    let mut depth = None;
    for (name, section) in table.get_ref() {
        let keys = SECTIONS.iter().find(|(n, _)| *n == name.get_ref().as_ref());
        let Some((section_name, keys)) = keys else {
            let names: Vec<&str> = SECTIONS.iter().map(|(n, _)| *n).collect();
            return Err(fail(
                name.span(),
                format!(
                    "no section `{}`: mitra.toml has {}",
                    name.get_ref(),
                    list(&names)
                ),
            ));
        };
        let DeValue::Table(entries) = section.get_ref() else {
            return Err(fail(
                name.span(),
                format!("`{section_name}` is a section: `[{section_name}]`"),
            ));
        };
        for (key, value) in entries {
            let key_name = key.get_ref().as_ref();
            if !keys.contains(&key_name) {
                return Err(fail(
                    key.span(),
                    format!(
                        "no setting `{key_name}` in `[{section_name}]`: it has {}",
                        list(keys)
                    ),
                ));
            }
            let at = format!("{section_name}.{key_name}");
            let count = |least: usize| whole(value, least, &at, &fail);
            match (*section_name, key_name) {
                ("parse", _) => depth = Some(count(0)?),
                ("agree", "points") => s.agree.points = count(1)?,
                ("agree", "splits") => s.agree.splits = count(0)?,
                ("agree", "length") => s.agree.length = count(0)?,
                ("agree", "height") => s.agree.height = count(0)?,
                ("symbolic", "runs") => s.search.symbolic.runs = count(1)?,
                ("symbolic", "length") => s.search.symbolic.samples.length = count(0)?,
                ("symbolic", "shown") => s.search.shown = count(1)?,
                ("symbolic", "depth") => s.search.steps = count(1)?,
                ("symbolic", "spread") => s.search.symbolic.spread = count(0)?,
                ("symbolic", "reach") => s.search.reach = count(2)?,
                ("budget", "steps") => s.agree.budget.steps = count(0)?,
                ("budget", _) => s.agree.budget.depth = count(0)?,
                ("explain", "rounds") => s.explain.rounds = count(0)?,
                ("explain", "cases") => s.explain.cases = count(0)?,
                ("explain", "rests") => s.explain.rests = count(0)?,
                ("numbers", "base_rate") => s.numbers.base_rate = weight(value, &at, true, &fail)?,
                ("numbers", "prior_weight") => {
                    s.numbers.prior_weight = weight(value, &at, false, &fail)?
                }
                ("numbers", "checker") => s.numbers.checker = weight(value, &at, false, &fail)?,
                ("numbers", "link") => s.numbers.link = weight(value, &at, false, &fail)?,
                ("numbers", _) => {
                    let DeValue::Table(voices) = value.get_ref() else {
                        return Err(fail(
                            value.span(),
                            "`numbers.voices` is a section: `[numbers.voices]`, a voice and \
                             its weight on each line, `\"human:ana\" = 2.0`"
                                .to_string(),
                        ));
                    };
                    for (voice, v) in voices {
                        let at = format!("numbers.voices.\"{}\"", voice.get_ref());
                        let w = weight(v, &at, false, &fail)?;
                        s.numbers.voices.insert(voice.get_ref().to_string(), w);
                    }
                }
                ("test", _) => {
                    let words = match value.get_ref() {
                        DeValue::Array(words) => words
                            .iter()
                            .map(|w| match w.get_ref() {
                                DeValue::String(w) => Some(w.to_string()),
                                _ => None,
                            })
                            .collect::<Option<Vec<String>>>()
                            .filter(|w| !w.is_empty()),
                        _ => None,
                    };
                    s.test.command = words.ok_or_else(|| {
                        fail(
                            value.span(),
                            format!(
                                "`test.command` is the program and its arguments, texts in a list: `[\"cargo\", \"test\"]`, not `{}`",
                                shown(value)
                            ),
                        )
                    })?;
                }
                ("blame", "diagnoses") => s.blame.diagnoses = count(0)?,
                _ => s.blame.tries = count(0)?,
            }
        }
    }
    s.explain.budget = s.agree.budget;
    s.parse.depth = depth.unwrap_or(2 * s.agree.budget.depth);
    Ok(s)
}

/// `a`, `b` and `c`.
fn list(names: &[&str]) -> String {
    let names: Vec<String> = names.iter().map(|n| format!("`{n}`")).collect();
    match names.split_last() {
        Some((last, [])) => last.clone(),
        Some((last, rest)) => format!("{} and {last}", rest.join(", ")),
        None => String::new(),
    }
}

/// A count: a whole number, at least `least`.
fn whole(
    value: &Spanned<DeValue>,
    least: usize,
    at: &str,
    fail: &impl Fn(Range<usize>, String) -> ParseError,
) -> Result<usize> {
    let n = match value.get_ref() {
        DeValue::Integer(i) => {
            let digits = i.as_str().replace('_', "");
            let digits = digits.trim_start_matches('+');
            let digits = digits
                .strip_prefix("0x")
                .or_else(|| digits.strip_prefix("0o"))
                .or_else(|| digits.strip_prefix("0b"))
                .unwrap_or(digits);
            usize::from_str_radix(digits, i.radix()).ok()
        }
        _ => None,
    };
    match n {
        Some(n) if n >= least => Ok(n),
        _ => Err(fail(
            value.span(),
            format!(
                "`{at}` takes a whole number of {least} or more, not `{}`",
                shown(value)
            ),
        )),
    }
}

/// A weight: a number written with its point, above 0 (`unit`: at
/// most 1, and 0 allowed).
fn weight(
    value: &Spanned<DeValue>,
    at: &str,
    unit: bool,
    fail: &impl Fn(Range<usize>, String) -> ParseError,
) -> Result<f64> {
    let w = match value.get_ref() {
        DeValue::Float(f) => f.as_str().replace('_', "").parse::<f64>().ok(),
        _ => None,
    };
    let fits = |w: f64| match unit {
        true => (0.0..=1.0).contains(&w),
        false => w.is_finite() && w > 0.0,
    };
    let takes = match unit {
        true => "a number from 0.0 to 1.0",
        false => "a number above 0, with its point (`2.0`)",
    };
    match w {
        Some(w) if fits(w) => Ok(w),
        _ => Err(fail(
            value.span(),
            format!("`{at}` takes {takes}, not `{}`", shown(value)),
        )),
    }
}

/// A value as written.
fn shown(value: &Spanned<DeValue>) -> String {
    match value.get_ref() {
        DeValue::Integer(i) => i.as_str().to_string(),
        DeValue::Float(f) => f.as_str().to_string(),
        DeValue::String(s) => format!("\"{s}\""),
        DeValue::Boolean(b) => b.to_string(),
        other => format!("a {}", other.type_str()),
    }
}

/// The line and column of byte `offset` of `text`, from 1, in characters.
fn pos(text: &str, offset: usize) -> Pos {
    let before = &text[..offset.min(text.len())];
    let line = before.matches('\n').count() + 1;
    let column = before.rsplit('\n').next().unwrap_or("").chars().count() + 1;
    Pos { line, column }
}
