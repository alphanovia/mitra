//! The report of `cargo test`: which tests ran, and whether each passed.
//!
//! `libtest` reads the text `cargo test` prints, lines `test <path> ...
//! ok`. A test's name is the last segment of its path (`tests::t_small`
//! is `t_small`). Ignored tests are left out. JUnit XML is not Rust's: it
//! is read by `mitra_lang::ledger::results::junit`.

pub use mitra_lang::ledger::results::{short, Outcome};

// mitra: implements reader::libtest
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

    // mitra: evidence for t_libtest_ok
    #[test]
    fn t_libtest_ok() {
        assert_eq!(
            libtest("test tests::t_small ... ok"),
            [outcome("t_small", true)]
        );
    }

    // mitra: evidence for t_libtest_failed
    #[test]
    fn t_libtest_failed() {
        assert_eq!(libtest("test t_big ... FAILED"), [outcome("t_big", false)]);
    }

    // mitra: evidence for t_libtest_timed
    #[test]
    fn t_libtest_timed() {
        assert_eq!(
            libtest("test t_timed ... ok <0.001s>"),
            [outcome("t_timed", true)]
        );
    }

    // mitra: evidence for t_libtest_ignored
    #[test]
    fn t_libtest_ignored() {
        assert_eq!(libtest("test tests::slow ... ignored"), []);
    }

    // mitra: evidence for t_libtest_other_lines
    #[test]
    fn t_libtest_other_lines() {
        assert_eq!(
            libtest("running 3 tests\ntest result: FAILED. 1 passed; 1 failed"),
            []
        );
    }

    // mitra: evidence for t_libtest_two
    #[test]
    fn t_libtest_two() {
        assert_eq!(
            libtest("test a ... ok\ntest b ... FAILED"),
            [outcome("a", true), outcome("b", false)]
        );
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
}
