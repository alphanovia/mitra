//! `mitra.toml`: each layer's settings, a section each (the user's
//! choice, 2026-11-24 12:02). Nothing given goes unread: P015.

use mitra_core::settings::{read, Settings};

fn fails(text: &str) -> String {
    let e = read(text).unwrap_err();
    assert_eq!(e.code.to_string(), "P015");
    e.render("mitra.toml", text)
}

#[test]
fn an_empty_file_changes_nothing() {
    assert_eq!(read("").unwrap(), Settings::default());
    assert_eq!(read("# the root\n").unwrap(), Settings::default());
}

#[test]
fn each_section_sets_its_layer() {
    let s = read(
        "[parse]
depth = 50
[agree]
points = 100
splits = 2
length = 5
height = 2
[budget]
steps = 1_000
depth = 20
[explain]
rounds = 8
cases = 16
rests = 32
[numbers]
base_rate = 0.25
prior_weight = 4.0
checker = 10.0
[numbers.voices]
\"human:ana\" = 2.5
[blame]
diagnoses = 3
tries = 7
[test]
command = [\"cargo\", \"test\", \"--release\"]
",
    )
    .unwrap();
    assert_eq!(s.parse.depth, 50);
    assert_eq!(
        (s.agree.points, s.agree.splits, s.agree.length),
        (100, 2, 5)
    );
    assert_eq!(s.agree.height, 2);
    assert_eq!((s.agree.budget.steps, s.agree.budget.depth), (1000, 20));
    // One budget, agree's and explain's.
    assert_eq!(s.explain.budget, s.agree.budget);
    assert_eq!(
        (s.explain.rounds, s.explain.cases, s.explain.rests),
        (8, 16, 32)
    );
    assert_eq!(
        (
            s.numbers.base_rate,
            s.numbers.prior_weight,
            s.numbers.checker
        ),
        (0.25, 4.0, 10.0)
    );
    assert_eq!(s.numbers.voices["human:ana"], 2.5);
    assert_eq!((s.blame.diagnoses, s.blame.tries), (3, 7));
    assert_eq!(s.test.command, ["cargo", "test", "--release"]);
}

/// The tests run are a program and its arguments, texts in a list.
#[test]
fn the_test_command_is_a_list_of_texts() {
    for wrong in [
        "command = \"cargo test\"",
        "command = []",
        "command = [\"cargo\", 1]",
    ] {
        let found = fails(&format!("[test]\n{wrong}\n"));
        assert!(
            found.starts_with(
                "error[P015]: `test.command` is the program and its arguments, texts in a list"
            ),
            "{wrong}: {found}"
        );
    }
}

#[test]
fn parse_depth_follows_the_budget_when_not_said() {
    let s = read("[budget]\ndepth = 30\n").unwrap();
    assert_eq!(s.parse.depth, 60);
    let s = read("[budget]\ndepth = 30\n[parse]\ndepth = 5\n").unwrap();
    assert_eq!(s.parse.depth, 5);
}

#[test]
fn what_mitra_does_not_take_is_p015_where_it_is() {
    assert_eq!(
        fails("[agree]\npoints = 64\npionts = 3\n"),
        "error[P015]: no setting `pionts` in `[agree]`: it has `points`, `splits`, `length` and `height`
 --> mitra.toml:3:1
  |
3 | pionts = 3
  | ^
"
    );
    let out = fails("[check]\npoints = 3\n");
    assert!(out.contains("no section `check`: mitra.toml has `parse`, `agree`, `budget`, `explain`, `numbers`, `blame` and `test`"), "{out}");
    assert!(out.contains("mitra.toml:1:2"), "{out}");
    let out = fails("points = 3\n");
    assert!(out.contains("no section `points`"), "{out}");
    let out = fails("agree = 3\n");
    assert!(out.contains("`agree` is a section: `[agree]`"), "{out}");
    let out = fails("[agree]\npoints = \n");
    assert!(out.contains("not TOML: "), "{out}");
    assert!(out.contains("mitra.toml:2:"), "{out}");
}

#[test]
fn a_value_is_of_its_kind_no_number_becomes_another() {
    for (text, says) in [
        (
            "[agree]\npoints = 0\n",
            "`agree.points` takes a whole number of 1 or more, not `0`",
        ),
        (
            "[agree]\nsplits = -1\n",
            "`agree.splits` takes a whole number of 0 or more, not `-1`",
        ),
        (
            "[agree]\nsplits = 2.0\n",
            "`agree.splits` takes a whole number of 0 or more, not `2.0`",
        ),
        ("[blame]\ntries = \"many\"\n", "not `\"many\"`"),
        (
            "[numbers]\nprior_weight = 2\n",
            "`numbers.prior_weight` takes a number above 0, with its point (`2.0`), not `2`",
        ),
        (
            "[numbers]\nchecker = 0.0\n",
            "`numbers.checker` takes a number above 0",
        ),
        ("[numbers]\nchecker = inf\n", "not `inf`"),
        (
            "[numbers]\nbase_rate = 1.5\n",
            "`numbers.base_rate` takes a number from 0.0 to 1.0, not `1.5`",
        ),
        (
            "[numbers.voices]\n\"human:ana\" = 0.0\n",
            "`numbers.voices.\"human:ana\"` takes a number above 0",
        ),
        (
            "[numbers]\nvoices = 2.0\n",
            "`numbers.voices` is a section: `[numbers.voices]`",
        ),
    ] {
        let out = fails(text);
        assert!(out.contains(says), "{text}: {out}");
    }
    // Underscores and other bases are TOML's own.
    assert_eq!(read("[blame]\ntries = 0x10\n").unwrap().blame.tries, 16);
}
