//! `then` on calls to the environment (BDD step 2): after the value, the
//! calls the run makes to functions with no body, `then 2,
//! Disk::write("a.txt", "ab")`, or the calls alone. Each listed call is
//! made at least once, `_` any argument; calls not listed are not
//! compared. In examples, agree runs the row (A011 when one is not
//! made); in the store, a call the model's run does not make is against
//! it.

use mitra_lang::agree::agree;
use mitra_lang::check::check;
use mitra_lang::explain::{explain, Settings, Verdict};
use mitra_lang::parse::{parse, parse_prop};
use mitra_lang::print::{print, print_prop};
use mitra_lang::store::{print_store, read_store};

const MODEL: &str = "environment Disk {
    fn write(path: [~char], body: [~char]) -> ok: ();
    fn append(path: [~char], body: [~char]) -> ok: ();
    fn size(path: [~char]) -> n: integer;
}
subject Notes {
    use Disk::write;
    fn save(n: [~char]) -> r: integer at \"src/lib.rs/save\" {
        if n == \"\" { 0 } else { write(\"a.txt\", n); n.len() }
    } where {
        examples { ROWS }
    }
}
examples { WHOLE }";

fn with(rows: &str, whole: &str) -> String {
    MODEL.replace("ROWS", rows).replace("WHOLE", whole)
}

fn shown(text: &str) -> Vec<String> {
    let m = parse(text).unwrap_or_else(|e| panic!("{e}"));
    let mut all: Vec<String> = check(&m).iter().map(|e| e.to_string()).collect();
    all.extend(
        agree(&m, &Default::default())
            .1
            .iter()
            .map(|f| f.to_string()),
    );
    all
}

const GOOD_ROWS: &str = "when (\"ab\") then 2, Disk::write(\"a.txt\", \"ab\"),
            when (\"ab\") then write(_, \"ab\"),
            (\"\") -> 0";
const GOOD_WHOLE: &str = "when Notes::save(\"xy\") then 2, Disk::write(\"a.txt\", _),
    when Notes::save(\"\") then 0";

#[test]
fn calls_the_run_makes_hold() {
    assert_eq!(shown(&with(GOOD_ROWS, GOOD_WHOLE)), Vec::<String>::new());
}

#[test]
fn a_call_not_made_is_a011() {
    let f = shown(&with(
        "when (\"\") then 0, Disk::write(\"a.txt\", \"\"), when (\"ab\") then write(\"b.txt\", _)",
        "when Notes::save(\"\") then Disk::write(_, _), when Notes::save(\"ab\") then Disk::append(_, _)",
    ));
    assert_eq!(
        f,
        [
            "finding[A011]: example 0: the call `Disk::write(\"a.txt\", \"\")`: the run does not make it; it calls no function with no body\n --> Notes::save",
            "finding[A011]: example 1: the call `Disk::write(\"b.txt\", _)`: the run does not make it; it calls `Disk::write(\"a.txt\", \"ab\")`\n --> Notes::save",
            "finding[A011]: the call `Disk::write(_, _)`: the run does not make it; it calls no function with no body\n --> examples::0",
            "finding[A011]: the call `Disk::append(_, _)`: the run does not make it; it calls `Disk::write(\"a.txt\", \"ab\")`\n --> examples::1",
        ]
    );
}

#[test]
fn a_call_not_of_the_environment_is_m022() {
    for (call, starts) in [
        ("Notes::save(\"x\")", "error[M022]: example 0: `Notes::save(\"x\")` in `then` is not a call to a function with no body"),
        ("Disk::write(\"x\")", "error[M022]: example 0: `Disk::write(\"x\")` in `then`"),
        ("Disk::write(1, \"x\")", "error[M022]: example 0: `Disk::write(1, \"x\")` in `then`"),
        ("Disk::erase(\"x\")", "error[M022]: example 0: `Disk::erase(\"x\")` in `then`"),
    ] {
        let e = shown(&with(&format!("when (\"ab\") then 2, {call}"), GOOD_WHOLE));
        assert!(e.iter().any(|e| e.starts_with(starts)), "{call}: {e:?}");
    }
    let e = shown(&with(
        GOOD_ROWS,
        "when Notes::save(\"\") then Notes::save(_)",
    ));
    assert!(
        e.iter()
            .any(|e| e.starts_with("error[M022]: `Notes::save(_)` in `then`")),
        "{e:?}"
    );
}

#[test]
fn rows_print_as_written_and_read_back() {
    let m = parse(&with(GOOD_ROWS, GOOD_WHOLE)).unwrap();
    let printed = print(&m);
    for written in [
        "when (\"ab\") then 2, Disk::write(\"a.txt\", \"ab\")",
        "when (\"ab\") then Disk::write(_, \"ab\")",
        "when Notes::save(\"xy\") then 2, Disk::write(\"a.txt\", _)",
    ] {
        assert!(printed.contains(written), "{written} in {printed}");
    }
    assert_eq!(parse(&printed).unwrap(), m, "{printed}");
}

/// A value first is a value, even one that looks like a call: a variant's
/// name is `CamelCase`. A row after the calls is a row.
#[test]
fn a_value_and_rows_after_calls_are_told_apart() {
    let text = "environment Disk {
    fn write(path: [~char]) -> ok: ();
}
subject enum Out { Done(integer), Skipped }
subject Notes {
    use Disk::write;
    fn save(n: integer) -> r: Out { if n < 0 { Out::Skipped } else { write(\"a\"); Out::Done(n) } } where {
        examples {
            when (1) then Out::Done(1), Disk::write(\"a\"),
            when (-1) then Skipped,
            (2) -> Done(2)
        }
    }
}";
    let m = parse(text).unwrap();
    let notes = m.parts.iter().find(|p| p.name == "Notes").unwrap();
    assert_eq!(notes.functions[0].examples.len(), 3);
    assert_eq!(shown(text), Vec::<String>::new());
}

fn verdicts(store: &str) -> (Vec<(String, Verdict)>, Vec<String>) {
    let m = parse(&with(GOOD_ROWS, GOOD_WHOLE)).unwrap();
    let (ledger, findings) = read_store("s", store, &m, &m.ties).unwrap();
    let e = explain(&m, &ledger, &Settings::default());
    (
        e.verdicts
            .iter()
            .map(|v| (print_prop(&v.prop), v.verdict.clone()))
            .collect(),
        findings.iter().map(|f| f.to_string()).collect(),
    )
}

fn of(v: &[(String, Verdict)], p: &str) -> Verdict {
    v.iter()
        .find(|(q, _)| q == p)
        .map(|(_, v)| v.clone())
        .unwrap_or_else(|| panic!("no verdict on {p}: {v:?}"))
}

#[test]
fn a_measured_call_the_model_does_not_make_is_against_it() {
    let (v, findings) = verdicts(
        "measure when Notes::save(\"cd\") then 2, Disk::write(\"a.txt\", \"cd\") by \"test:wrote\";
measure when Notes::save(\"ef\") then 2, Disk::write(_, _) by \"test:any\";
measure when Notes::save(\"\") then 0, Disk::write(\"a.txt\", \"\") by \"test:empty\";
measure when Notes::save(\"gh\") then 2, Disk::write(\"b.txt\", \"gh\") by \"test:other\";",
    );
    assert_eq!(findings, Vec::<String>::new());
    assert_eq!(of(&v, "correct(Notes::save, (\"cd\"))"), Verdict::Explained);
    assert_eq!(of(&v, "correct(Notes::save, (\"ef\"))"), Verdict::Explained);
    assert!(matches!(
        of(&v, "correct(Notes::save, (\"gh\"))"),
        Verdict::Contradicted { .. }
    ));
    // The model's own example `("") -> 0` is for it; the call is against.
    let (v, _) = verdicts(
        "measure when Notes::save(\"\") then 0, Disk::write(\"a.txt\", \"\") by \"test:empty\";",
    );
    let (w, _) = verdicts("measure when Notes::save(\"\") then 0 by \"test:empty\";");
    assert!(matches!(
        of(&v, "correct(Notes::save, (\"\"))"),
        Verdict::Contradicted { .. }
    ));
    assert_eq!(of(&w, "correct(Notes::save, (\"\"))"), Verdict::Explained);
}

#[test]
fn the_store_reads_back_what_it_prints() {
    let m = parse(&with(GOOD_ROWS, GOOD_WHOLE)).unwrap();
    let text =
        "measure when Notes::save(\"cd\") then 2, Disk::write(\"a.txt\", _) by \"test:wrote\";\n";
    let (ledger, findings) = read_store("s", text, &m, &m.ties).unwrap();
    assert_eq!(findings, vec![]);
    assert_eq!(print_store(&ledger), text);
    let (again, _) = read_store("s", &print_store(&ledger), &m, &m.ties).unwrap();
    assert_eq!(again, ledger);
    // Two measurements alike but for their calls are two.
    assert_ne!(
        read_store(
            "s",
            "measure when Notes::save(\"cd\") then 2 by \"t\";",
            &m,
            &m.ties
        )
        .unwrap()
        .0,
        ledger
    );
}

#[test]
fn a_call_the_store_cannot_read() {
    let m = parse(&with(GOOD_ROWS, GOOD_WHOLE)).unwrap();
    for (call, starts) in [
        ("Notes::save(\"x\")", "has a body"),
        ("Disk::erase(\"x\")", "is no function of the model"),
        ("Disk::write(1, \"x\")", "is not of `Disk::write`"),
        ("Disk::write(\"x\")", "is not of `Disk::write`"),
    ] {
        let text = format!("measure when Notes::save(\"cd\") then 2, {call} by \"t\";");
        let (ledger, findings) = read_store("s", &text, &m, &m.ties).unwrap();
        assert!(ledger.measurements.is_empty(), "{call}");
        assert!(
            findings.len() == 1
                && findings[0].to_string().starts_with("finding[R006]")
                && findings[0].to_string().contains(starts),
            "{call}: {findings:?}"
        );
    }
    // The value first: a measurement says what the function gave.
    let e = read_store(
        "s",
        "measure when Notes::save(\"cd\") then Disk::write(_, _) by \"t\";",
        &m,
        &m.ties,
    )
    .unwrap_err();
    assert!(
        e.to_string()
            .starts_with("error[P003]: a measurement's `then` says what the function gave first"),
        "{e}"
    );
    let _ = parse_prop;
}

/// A call made for what it does gives `()`, the one value of its type,
/// with no row: a body may give it.
#[test]
fn a_call_for_what_it_does_gives_unit() {
    let text = "environment Disk {
    fn write(path: [~char]) -> ok: ();
}
subject Notes {
    use Disk::write;
    fn touch(n: [~char]) -> r: () { write(n) } where {
        examples { (\"a\") -> (), when (\"b\") then (), Disk::write(\"b\") }
    }
}";
    assert_eq!(shown(text), Vec::<String>::new());
}

/// Two measurements by one voice at one point are one only when they
/// list the same calls.
#[test]
fn measurements_alike_but_for_their_calls_are_two() {
    let (v, _) = verdicts(
        "measure when Notes::save(\"cd\") then 2 by \"test:t\";
measure when Notes::save(\"cd\") then 2, Disk::append(_, _) by \"test:t\";",
    );
    assert!(matches!(
        of(&v, "correct(Notes::save, (\"cd\"))"),
        Verdict::Contradicted { .. }
    ));
}
