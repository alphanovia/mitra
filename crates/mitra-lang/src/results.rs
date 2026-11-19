//! Test results as evidence: which tests ran, whether each passed, and
//! what a tied test's result says about the claim it names.
//!
//! Nothing here is about one language of code. A reader of that language
//! finds the tests tied to claims ([`Tied`]) and reads its own report
//! format into [`Outcome`]s; JUnit XML, which many tools write, is read
//! here ([`junit`]). [`from_tests`] holds the rules that turn the two into
//! evidence blocks.

use crate::ast::{Claim, Model};

/// One test of a report, and whether it passed.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements results::Outcome
pub struct Outcome {
    pub name: String,
    pub passed: bool,
}

/// A test tied to a claim: `// mitra: evidence for math::div_small` above
/// its `fn`. A pass is evidence that the claim holds, a failure that it
/// does not. The comment may pin what the test checks, with the clauses
/// of an evidence block: `evidence for math::div given a = 6, b = 3
/// gives 2`. Without them, on a claim with inputs, a pass only `passes`.
#[derive(Debug, Clone, PartialEq)]
// mitra: implements results::Tied
pub struct Tied {
    /// The name of the test function.
    pub test: String,
    pub claim: String,
    /// What the comment pins, as in an evidence block: `given a = 1`,
    /// `given a = 1, b = 2 gives 3`. Empty when it states no input.
    pub pinned: String,
    /// The file and line of the comment.
    pub place: String,
}

/// What the results of tied tests say, as evidence blocks to add.
#[derive(Debug, Clone, PartialEq, Default)]
// mitra: implements results::Ingested
pub struct Ingested {
    /// One evidence block per line.
    pub evidence: Vec<String>,
    /// What was left out, and why.
    pub notes: Vec<String>,
    /// Tests that state no input on a claim that has inputs: each is
    /// recorded as `passes`, which settles nothing.
    pub loose: Vec<String>,
}

/// The evidence the results give for the claims the tests are tied to.
/// `rev` is written into each block.
///
/// - A comment that pins inputs is written as it stands when the test
///   passed; a failure denies it on the same inputs.
/// - A sample keeps its domain and its count, passed or not.
/// - A pinned `gives` that passed on a function also says the code is
///   right on that input.
/// - With nothing pinned: `holds` on a claim without inputs, `passes` on
///   a claim with inputs, `fails` on a failure.
// mitra: implements results::from_tests
pub fn from_tests(
    model: &Model,
    tied: &[Tied],
    outcomes: &[Outcome],
    rev: Option<&str>,
) -> Result<Ingested, String> {
    let mut out = Ingested::default();
    let attrs = match rev {
        Some(r) => format!(" rev \"{r}\""),
        None => String::new(),
    };
    for t in tied {
        // Who wrote the test, said after what it pins: `written "…" cold`.
        let (pinned, who) = signed(&t.pinned);
        let t = &Tied {
            test: t.test.clone(),
            claim: t.claim.clone(),
            pinned,
            place: t.place.clone(),
        };
        let ran: Vec<&Outcome> = outcomes.iter().filter(|o| o.name == t.test).collect();
        let passed = match ran[..] {
            [] if t.pinned.is_empty() => None,
            [] => continue,
            [o] => Some(o.passed),
            _ => {
                out.notes.push(format!(
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
            out.evidence.push(format!(
                "evidence test {{ for {} {says}{who} source \"test:{}\"{attrs} }}",
                t.claim, t.test
            ));
            // A test that compares with the output it expects also
            // says the code is right on that input.
            let gives = said.is_some_and(|n| words[n] == "gives");
            if passed == Some(true) && gives && model.function(&t.claim).is_some() {
                out.evidence.push(format!(
                    "evidence test {{ for {} {given} holds{who} source \"test:{}\"{attrs} }}",
                    t.claim, t.test
                ));
            }
            continue;
        }
        let ends = |c: &&Claim| c.name == t.claim || c.name.ends_with(&format!("::{}", t.claim));
        let found: Vec<&Claim> = model.claims.iter().filter(ends).collect();
        let [c] = found[..] else {
            let why = if found.is_empty() {
                "no claim"
            } else {
                "several claims"
            };
            return Err(format!("{}: the model has {why} `{}`", t.place, t.claim));
        };
        let Some(passed) = passed else { continue };
        // On a claim with inputs, a test that states none passes on some
        // of them: it settles nothing. A failure refutes.
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
                out.loose
                    .push(format!("{}: {} on `{}`", t.place, t.test, c.name));
                "passes"
            }
        };
        out.evidence.push(format!(
            "evidence test {{ for {} {says}{who} source \"test:{}\"{attrs} }}",
            c.name, t.test
        ));
    }
    out.evidence.dedup();
    Ok(out)
}

/// What a tie comment pins, and who it says wrote the test: the words
/// from `written "…"` or a last `cold`, kept as they are.
// mitra: implements results::signed
pub fn signed(pinned: &str) -> (String, String) {
    let at = match pinned.find("written \"") {
        Some(i) => Some(i),
        None => pinned
            .strip_suffix("cold")
            .filter(|p| p.is_empty() || p.ends_with(' '))
            .map(|p| p.len()),
    };
    match at {
        Some(i) => (
            pinned[..i].trim().to_string(),
            format!(" {}", pinned[i..].trim()),
        ),
        None => (pinned.to_string(), String::new()),
    }
}

/// The last segment of a test's path, without ` - should panic`.
// mitra: implements results::short
pub fn short(path: &str) -> String {
    let path = path.trim().trim_end_matches(" - should panic");
    path.rsplit("::").next().unwrap_or(path).trim().to_string()
}

/// The value of `attr="…"` or `attr='…'` in a tag's text, with the basic
/// XML entities decoded.
// mitra: implements results::attr
fn attr(tag: &str, attr: &str) -> Option<String> {
    let mut rest = tag;
    while let Some(i) = rest.find(attr) {
        let before = rest[..i].chars().next_back();
        let after = rest[i + attr.len()..].trim_start();
        rest = &rest[i + attr.len()..];
        if !before.is_some_and(char::is_whitespace) {
            continue;
        }
        let Some(after) = after.strip_prefix('=') else {
            continue;
        };
        let after = after.trim_start();
        let quote = after.chars().next()?;
        if quote != '"' && quote != '\'' {
            continue;
        }
        let value = &after[1..];
        let end = value.find(quote)?;
        return Some(
            value[..end]
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\"")
                .replace("&apos;", "'")
                .replace("&amp;", "&"),
        );
    }
    None
}

/// `text` without the parts between `open` and `close`.
// mitra: implements results::strip
fn strip(text: &str, open: &str, close: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(i) = rest.find(open) {
        out += &rest[..i];
        rest = rest[i..]
            .find(close)
            .map_or("", |j| &rest[i + j + close.len()..]);
    }
    out + rest
}

// mitra: implements results::junit
pub fn junit(xml: &str) -> Vec<Outcome> {
    // Test output may mention `<failure`: drop it before looking.
    let xml = strip(xml, "<![CDATA[", "]]>");
    let xml = strip(&xml, "<system-out", "</system-out>");
    let xml = strip(&xml, "<system-err", "</system-err>");
    let mut out = Vec::new();
    let mut rest = xml.as_str();
    while let Some(i) = rest.find("<testcase") {
        rest = &rest[i..];
        let Some(tag_end) = rest.find('>') else { break };
        let tag = &rest[..tag_end];
        let body = if tag.ends_with('/') {
            ""
        } else {
            let end = rest.find("</testcase>").unwrap_or(rest.len());
            &rest[tag_end..end]
        };
        if let Some(name) = attr(tag, "name") {
            if !body.contains("<skipped") {
                out.push(Outcome {
                    name: short(&name),
                    passed: !body.contains("<failure") && !body.contains("<error"),
                });
            }
        }
        rest = &rest[tag_end..];
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    // One input per test, for Mitra: each `mitra: test` line gives the
    // output the test checks.
    fn outcome(name: &str, passed: bool) -> Outcome {
        Outcome {
            name: name.into(),
            passed,
        }
    }

    // mitra: evidence for t_short_path
    #[test]
    fn t_short_path() {
        assert_eq!(short("tests::t_small"), "t_small");
    }

    // mitra: evidence for t_short_plain
    #[test]
    fn t_short_plain() {
        assert_eq!(short("t_small"), "t_small");
    }

    // mitra: evidence for t_short_should_panic
    #[test]
    fn t_short_should_panic() {
        assert_eq!(short("t_panics - should panic"), "t_panics");
    }

    // mitra: evidence for t_short_panics_once
    #[test]
    fn t_short_panics_once() {
        assert_eq!(short("t_panics"), "t_panics");
    }

    // mitra: evidence for t_attr_double
    #[test]
    fn t_attr_double() {
        assert_eq!(attr("<testcase name=\"t\"", "name"), Some("t".into()));
    }

    // mitra: evidence for t_attr_single
    #[test]
    fn t_attr_single() {
        assert_eq!(attr("<testcase name='t'", "name"), Some("t".into()));
    }

    // mitra: evidence for t_attr_missing
    #[test]
    fn t_attr_missing() {
        assert_eq!(attr("<testcase time=\"1\"", "name"), None);
    }

    // mitra: evidence for t_attr_inside_another
    #[test]
    fn t_attr_inside_another() {
        assert_eq!(
            attr("<testcase\tclassname=\"name=x\" name = \"t\"", "name"),
            Some("t".into())
        );
    }

    // mitra: evidence for t_attr_entities
    #[test]
    fn t_attr_entities() {
        assert_eq!(
            attr("<a name=\"a &lt; b &amp;lt;\"", "name"),
            Some("a < b &lt;".into())
        );
    }

    // mitra: evidence for t_strip_one
    #[test]
    fn t_strip_one() {
        assert_eq!(strip("a<x>b</x>c", "<x>", "</x>"), "ac");
    }

    // mitra: evidence for t_strip_unclosed
    #[test]
    fn t_strip_unclosed() {
        assert_eq!(strip("a<x>b", "<x>", "</x>"), "a");
    }

    // mitra: evidence for t_strip_nothing
    #[test]
    fn t_strip_nothing() {
        assert_eq!(strip("abc", "<x>", "</x>"), "abc");
    }

    // mitra: evidence for t_junit_passed
    #[test]
    fn t_junit_passed() {
        assert_eq!(
            junit("<testcase classname=\"add\" name=\"t\"/>"),
            [outcome("t", true)]
        );
    }

    // mitra: evidence for t_junit_failure
    #[test]
    fn t_junit_failure() {
        assert_eq!(
            junit("<testcase name=\"t\"><failure message=\"boom\"/></testcase>"),
            [outcome("t", false)]
        );
    }

    // mitra: evidence for t_junit_error
    #[test]
    fn t_junit_error() {
        assert_eq!(
            junit("<testcase name=\"t\"><error/></testcase>"),
            [outcome("t", false)]
        );
    }

    // mitra: evidence for t_junit_skipped
    #[test]
    fn t_junit_skipped() {
        assert_eq!(junit("<testcase name=\"t\"><skipped/></testcase>"), []);
    }

    // mitra: evidence for t_junit_failure_in_a_log
    #[test]
    fn t_junit_failure_in_a_log() {
        assert_eq!(
            junit("<testcase name='t'><system-out><![CDATA[ <failure> ]]></system-out></testcase>"),
            [outcome("t", true)]
        );
    }

    // mitra: evidence for t_junit_path
    #[test]
    fn t_junit_path() {
        assert_eq!(junit("<testcase name=\"tests::t\"/>"), [outcome("t", true)]);
    }

    #[test]
    fn reads_junit() {
        let xml = r#"<testsuite>
  <testcase classname="add" name="t_small" time="0.1"/>
  <testcase classname="add" name="t_big"><failure message="boom"/></testcase>
  <testcase name="t_err"><error/></testcase>
  <testcase name="t_skip"><skipped/></testcase>
  <testcase name='t_quoted'><system-out><![CDATA[ <failure> in a log ]]></system-out></testcase>
  <testcase	classname="name=x" name = "t_spaced"/>
</testsuite>"#;
        assert_eq!(
            junit(xml),
            vec![
                Outcome {
                    name: "t_small".into(),
                    passed: true
                },
                Outcome {
                    name: "t_big".into(),
                    passed: false
                },
                Outcome {
                    name: "t_err".into(),
                    passed: false
                },
                Outcome {
                    name: "t_quoted".into(),
                    passed: true
                },
                Outcome {
                    name: "t_spaced".into(),
                    passed: true
                },
            ]
        );
    }

    // A sweep: the claim on every input of the stated set.
    // mitra: evidence for results::short_once given path in {"", "t", "a::t", "a::b::t", "a::t - should panic", " a::t "} sampled 6
    #[test]
    fn short_once() {
        for path in ["", "t", "a::t", "a::b::t", "a::t - should panic", " a::t "] {
            assert_eq!(short(&short(path)), short(path));
        }
    }
}
