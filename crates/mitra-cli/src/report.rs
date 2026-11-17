//! Test reports: which tests ran, and whether each passed.
//!
//! Two formats, read without dependencies:
//! - `libtest`: the text `cargo test` prints, lines `test <path> ... ok`;
//! - `junit`: JUnit XML, `<testcase name="…">` with an optional
//!   `<failure` or `<error` inside.
//!
//! A test's name is the last segment of its path (`tests::t_small` is
//! `t_small`). Skipped and ignored tests are left out.

#[derive(Debug, PartialEq)]
pub struct Outcome {
    pub name: String,
    pub passed: bool,
}

fn short(path: &str) -> String {
    path.rsplit("::").next().unwrap_or(path).trim().to_string()
}

pub fn libtest(text: &str) -> Vec<Outcome> {
    text.lines()
        .filter_map(|l| {
            let rest = l.strip_prefix("test ")?;
            let (path, result) = rest.rsplit_once(" ... ")?;
            let passed = match result.trim() {
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

/// The value of `attr="…"` in a tag's text.
fn attr(tag: &str, attr: &str) -> Option<String> {
    let start = tag.find(&format!(" {attr}=\""))? + attr.len() + 3;
    let end = tag[start..].find('"')?;
    Some(tag[start..start + end].to_string())
}

pub fn junit(xml: &str) -> Vec<Outcome> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(i) = rest.find("<testcase") {
        rest = &rest[i..];
        let Some(tag_end) = rest.find('>') else { break };
        let tag = &rest[..tag_end];
        let self_closing = tag.ends_with('/');
        let body = if self_closing {
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

    #[test]
    fn reads_libtest() {
        let text = "running 3 tests\n\
                    test tests::t_small ... ok\n\
                    test t_big ... FAILED\n\
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
            ]
        );
    }
}
