//! Test reports: which tests ran, and whether each passed.
//!
//! Two formats, read without dependencies:
//! - `libtest`: the text `cargo test` prints, lines `test <path> ... ok`;
//! - `junit`: JUnit XML, `<testcase name="…">` with an optional
//!   `<failure` or `<error` inside.
//!
//! A test's name is the last segment of its path (`tests::t_small` is
//! `t_small`). Skipped and ignored tests are left out.

///
/// mitra: record Outcome(name: String, passed: Bool)
#[derive(Debug, PartialEq)]
pub struct Outcome {
    pub name: String,
    pub passed: bool,
}

/// The last segment of a test's path, without ` - should panic`.
///
/// mitra: opaque short(path: String) -> String
/// mitra: claim short_correct : forall path, correct(short(path))
/// mitra: -- A short name is already short.
/// mitra: claim short_once : forall path, short(short(path)) == short(path)
fn short(path: &str) -> String {
    let path = path.trim().trim_end_matches(" - should panic");
    path.rsplit("::").next().unwrap_or(path).trim().to_string()
}

/// mitra: opaque libtest(text: String) -> List(Outcome)
/// mitra: claim libtest_correct : forall text, correct(libtest(text))
/// mitra: claim libtest_uses : only uses libtest [short]
pub fn libtest(text: &str) -> Vec<Outcome> {
    text.lines()
        .filter_map(|l| {
            let rest = l.strip_prefix("test ")?;
            let (path, result) = rest.rsplit_once(" ... ")?;
            // `ok`, or `ok <0.001s>` with --report-time.
            let passed = match result.split_whitespace().next()? {
                "ok" => true,
                "FAILED" => false,
                _ => return None,
            };
            Some(Outcome {
                name: short(path),
                passed,
            })
        })
        .collect()
}

/// The value of `attr="…"` or `attr='…'` in a tag's text, with the basic
/// XML entities decoded.
///
/// mitra: opaque attr(tag: String, attr: String) -> Option(String)
/// mitra: claim attr_correct : forall tag, forall attr, correct(attr(tag, attr))
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
///
/// mitra: opaque strip(text: String, open: String, close: String) -> String
/// mitra: claim strip_correct : forall text, forall open, forall close, correct(strip(text, open, close))
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

/// mitra: opaque junit(xml: String) -> List(Outcome)
/// mitra: claim junit_correct : forall xml, correct(junit(xml))
/// mitra: claim junit_uses : only uses junit [strip, attr, short]
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

    // mitra: test t_short_path : short("tests::t_small") = "t_small"
    #[test]
    fn t_short_path() {
        assert_eq!(short("tests::t_small"), "t_small");
    }

    // mitra: test t_short_plain : short("t_small") = "t_small"
    #[test]
    fn t_short_plain() {
        assert_eq!(short("t_small"), "t_small");
    }

    // mitra: test t_short_should_panic : short("t_panics - should panic") = "t_panics"
    #[test]
    fn t_short_should_panic() {
        assert_eq!(short("t_panics - should panic"), "t_panics");
    }

    // mitra: test t_short_panics_once : short("t_panics") = "t_panics"
    #[test]
    fn t_short_panics_once() {
        assert_eq!(short("t_panics"), "t_panics");
    }

    // mitra: test t_libtest_ok : libtest("test tests::t_small ... ok") = [Outcome("t_small", true)]
    #[test]
    fn t_libtest_ok() {
        assert_eq!(
            libtest("test tests::t_small ... ok"),
            [outcome("t_small", true)]
        );
    }

    // mitra: test t_libtest_failed : libtest("test t_big ... FAILED") = [Outcome("t_big", false)]
    #[test]
    fn t_libtest_failed() {
        assert_eq!(libtest("test t_big ... FAILED"), [outcome("t_big", false)]);
    }

    // mitra: test t_libtest_timed : libtest("test t_timed ... ok <0.001s>") = [Outcome("t_timed", true)]
    #[test]
    fn t_libtest_timed() {
        assert_eq!(
            libtest("test t_timed ... ok <0.001s>"),
            [outcome("t_timed", true)]
        );
    }

    // mitra: test t_libtest_ignored : libtest("test tests::slow ... ignored") = []
    #[test]
    fn t_libtest_ignored() {
        assert_eq!(libtest("test tests::slow ... ignored"), []);
    }

    // mitra: test t_libtest_other_lines : libtest("running 3 tests\ntest result: FAILED. 1 passed; 1 failed") = []
    #[test]
    fn t_libtest_other_lines() {
        assert_eq!(
            libtest("running 3 tests\ntest result: FAILED. 1 passed; 1 failed"),
            []
        );
    }

    // mitra: test t_libtest_two : libtest("test a ... ok\ntest b ... FAILED") = [Outcome("a", true), Outcome("b", false)]
    #[test]
    fn t_libtest_two() {
        assert_eq!(
            libtest("test a ... ok\ntest b ... FAILED"),
            [outcome("a", true), outcome("b", false)]
        );
    }

    // mitra: test t_attr_double : attr("<testcase name=\"t\"", "name") = Some("t")
    #[test]
    fn t_attr_double() {
        assert_eq!(attr("<testcase name=\"t\"", "name"), Some("t".into()));
    }

    // mitra: test t_attr_single : attr("<testcase name='t'", "name") = Some("t")
    #[test]
    fn t_attr_single() {
        assert_eq!(attr("<testcase name='t'", "name"), Some("t".into()));
    }

    // mitra: test t_attr_missing : attr("<testcase time=\"1\"", "name") = None
    #[test]
    fn t_attr_missing() {
        assert_eq!(attr("<testcase time=\"1\"", "name"), None);
    }

    // mitra: test t_attr_inside_another : attr("<testcase\tclassname=\"name=x\" name = \"t\"", "name") = Some("t")
    #[test]
    fn t_attr_inside_another() {
        assert_eq!(
            attr("<testcase\tclassname=\"name=x\" name = \"t\"", "name"),
            Some("t".into())
        );
    }

    // mitra: test t_attr_entities : attr("<a name=\"a &lt; b &amp;lt;\"", "name") = Some("a < b &lt;")
    #[test]
    fn t_attr_entities() {
        assert_eq!(
            attr("<a name=\"a &lt; b &amp;lt;\"", "name"),
            Some("a < b &lt;".into())
        );
    }

    // mitra: test t_strip_one : strip("a<x>b</x>c", "<x>", "</x>") = "ac"
    #[test]
    fn t_strip_one() {
        assert_eq!(strip("a<x>b</x>c", "<x>", "</x>"), "ac");
    }

    // mitra: test t_strip_unclosed : strip("a<x>b", "<x>", "</x>") = "a"
    #[test]
    fn t_strip_unclosed() {
        assert_eq!(strip("a<x>b", "<x>", "</x>"), "a");
    }

    // mitra: test t_strip_nothing : strip("abc", "<x>", "</x>") = "abc"
    #[test]
    fn t_strip_nothing() {
        assert_eq!(strip("abc", "<x>", "</x>"), "abc");
    }

    // mitra: test t_junit_passed : junit("<testcase classname=\"add\" name=\"t\"/>") = [Outcome("t", true)]
    #[test]
    fn t_junit_passed() {
        assert_eq!(
            junit("<testcase classname=\"add\" name=\"t\"/>"),
            [outcome("t", true)]
        );
    }

    // mitra: test t_junit_failure : junit("<testcase name=\"t\"><failure message=\"boom\"/></testcase>") = [Outcome("t", false)]
    #[test]
    fn t_junit_failure() {
        assert_eq!(
            junit("<testcase name=\"t\"><failure message=\"boom\"/></testcase>"),
            [outcome("t", false)]
        );
    }

    // mitra: test t_junit_error : junit("<testcase name=\"t\"><error/></testcase>") = [Outcome("t", false)]
    #[test]
    fn t_junit_error() {
        assert_eq!(
            junit("<testcase name=\"t\"><error/></testcase>"),
            [outcome("t", false)]
        );
    }

    // mitra: test t_junit_skipped : junit("<testcase name=\"t\"><skipped/></testcase>") = []
    #[test]
    fn t_junit_skipped() {
        assert_eq!(junit("<testcase name=\"t\"><skipped/></testcase>"), []);
    }

    // mitra: test t_junit_failure_in_a_log : junit("<testcase name='t'><system-out><![CDATA[ <failure> ]]></system-out></testcase>") = [Outcome("t", true)]
    #[test]
    fn t_junit_failure_in_a_log() {
        assert_eq!(
            junit("<testcase name='t'><system-out><![CDATA[ <failure> ]]></system-out></testcase>"),
            [outcome("t", true)]
        );
    }

    // mitra: test t_junit_path : junit("<testcase name=\"tests::t\"/>") = [Outcome("t", true)]
    #[test]
    fn t_junit_path() {
        assert_eq!(junit("<testcase name=\"tests::t\"/>"), [outcome("t", true)]);
    }

    #[test]
    fn reads_libtest() {
        let text = "running 3 tests\n\
                    test tests::t_small ... ok\n\
                    test t_big ... FAILED\n\
                    test t_timed ... ok <0.001s>\n\
                    test t_panics - should panic ... ok\n\
                    test tests::slow ... ignored\n\
                    test result: FAILED. 1 passed; 1 failed";
        assert_eq!(
            libtest(text),
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
                    name: "t_timed".into(),
                    passed: true
                },
                Outcome {
                    name: "t_panics".into(),
                    passed: true
                },
            ]
        );
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
}
