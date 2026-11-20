//! Tests written from the model of mitra-store (`model/*.mitra`) alone:
//! each one says what a wrong implementation would get wrong.
//! Here: the ledger. Files and the store are in `cold_fable.rs`.

mod cold_kit;

use cold_kit::*;
use mitra_store::store::Which;
use mitra_store::Store;
use std::fs;

// --------------------------------------------------------------- ledger

// mitra: evidence for ledger::reads_back written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_append_adds_at_the_end_and_reads_back() {
    let root = fresh("append");
    let s = store_in(&root, CALC);
    let head = "// the head of the ledger, kept";
    fs::write(s.evidence_path(), head).unwrap();
    let (one, two) = (pinned("one", 1, "r1"), pinned("two", 5, "r1"));
    assert_eq!(s.append(&one), Ok(()));
    let first = ledger(&s);
    assert!(first.starts_with(head), "{first}");
    assert_eq!(first.matches(&one).count(), 1, "{first}");
    assert_eq!(s.append(&two), Ok(()));
    let second = ledger(&s);
    // What was there is untouched, the new text is after it.
    assert!(second.starts_with(&first), "{second}");
    assert_eq!(second[first.len()..].trim(), two);
    // Each entry on lines of its own, and read as two.
    let lines: Vec<&str> = second.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines, [head, one.as_str(), two.as_str()]);
    assert_eq!(s.model().unwrap().evidence.len(), 2);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for ledger::Store::add::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_add_gives_how_many_pieces_were_added() {
    let root = fresh("add-count");
    let s = store_in(&root, CALC);
    let two = format!("{}\n{}", pinned("one", 1, "r1"), pinned("two", 5, "r1"));
    assert_eq!(s.add(&two, None, None), Ok(2));
    assert_eq!(s.model().unwrap().evidence.len(), 2);
    // The second call counts what it added, not what is there.
    assert_eq!(s.add(&pinned("three", 9, "r1"), None, None), Ok(1));
    assert_eq!(s.model().unwrap().evidence.len(), 3);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for ledger::keeps_who_and_when written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_add_writes_who_and_which_revision_into_each_entry() {
    let root = fresh("add-who");
    let s = store_in(&root, CALC);
    let text = "evidence r { for calc::commutes holds }\nevidence q { for calc::neg holds }";
    assert_eq!(s.add(text, who("review:ana"), who("abc123")), Ok(2));
    // A later entry by someone else, with no revision.
    assert_eq!(
        s.add(
            "evidence r { for calc::add holds }",
            who("review:bob"),
            None
        ),
        Ok(1)
    );
    let m = s.model().unwrap();
    let got: Vec<(&str, Option<&str>, Option<&str>)> = m
        .evidence
        .iter()
        .map(|e| (e.about.as_str(), e.source.as_deref(), e.rev.as_deref()))
        .collect();
    assert_eq!(got.len(), 3, "{got:?}");
    assert!(
        got.contains(&("calc::commutes", Some("review:ana"), Some("abc123"))),
        "{got:?}"
    );
    assert!(
        got.contains(&("calc::neg", Some("review:ana"), Some("abc123"))),
        "{got:?}"
    );
    let bob = got.iter().find(|g| g.0 == "calc::add").unwrap();
    assert_eq!((bob.1, bob.2), (Some("review:bob"), None));
    // It is in the file, not only in what was read.
    let text = ledger(&s);
    assert_eq!(text.matches("review:ana").count(), 2, "{text}");
    assert_eq!(text.matches("abc123").count(), 2, "{text}");
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for ledger::sound_or_untouched written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_refused_evidence_writes_nothing() {
    let root = fresh("refused");
    let s = store_in(&root, CALC);
    s.add(&pinned("one", 1, "r1"), None, None).unwrap();
    let before = ledger(&s);
    // A piece the model does not have, after one it has: neither is written.
    let mixed = format!(
        "{}\nevidence t {{ for calc::nope given a = 1 gives 2 source \"test:x\" }}",
        pinned("two", 5, "r1")
    );
    assert!(s.add(&mixed, None, None).is_err());
    assert_eq!(ledger(&s), before);
    // Text that does not read.
    assert!(s.append("evidence t { for calc::add given").is_err());
    assert_eq!(ledger(&s), before);
    assert!(s
        .add("evidence t { for calc::add given", who("review:ana"), None)
        .is_err());
    assert_eq!(ledger(&s), before);
    // The store still takes what is sound.
    assert_eq!(s.add(&pinned("two", 5, "r1"), None, None), Ok(1));
    let _ = fs::remove_dir_all(root);
}

// New evidence on a piece the model does not have is refused (mitra-lang,
// `ledger::orphaned_judges_nothing`); `append` is a door of its own for
// new evidence, and checks the store with it before it writes.

// mitra: evidence for ledger::Store::append::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_append_to_a_store_that_does_not_read_writes_nothing() {
    let root = fresh("unsound");
    let s = store_in(&root, CALC);
    s.append(&pinned("one", 1, "r1")).unwrap();
    let before = ledger(&s);
    // The model no longer reads.
    fs::write(
        s.model_path(),
        "subject calc { fn add(a: Int, b: Int) -> ;;; {{{\n",
    )
    .unwrap();
    assert!(s.append(&pinned("two", 5, "r1")).is_err());
    assert_eq!(ledger(&s), before);
    assert!(s.add(&pinned("two", 5, "r1"), None, None).is_err());
    assert_eq!(ledger(&s), before);
    // The model reads again, the ledger does not.
    fs::write(s.model_path(), CALC).unwrap();
    let broken = format!("{before}evidence t {{ for calc::add given\n");
    fs::write(s.evidence_path(), &broken).unwrap();
    assert!(s.append(&pinned("two", 5, "r1")).is_err());
    assert_eq!(ledger(&s), broken);
    let _ = fs::remove_dir_all(root);
}

// -------------------------------------------------------------- keeping

// mitra: evidence for keeping::Store::replace::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_replace_takes_out_the_entries_of_the_prefix_and_writes_the_new() {
    let root = fresh("replace");
    let s = store_in(&root, CALC);
    let reader =
        "evidence t { for calc::neg given a = 1 gives -1 source \"reader:rust\" rev \"r0\" }";
    s.add(&pinned("one", 1, "r1"), None, None).unwrap();
    s.add(reader, None, None).unwrap();
    s.add(&pinned("two", 5, "r2"), None, None).unwrap();
    // `one` is observed again on another revision, `two` is not, `three` is new.
    let new = format!("{}\n{}", pinned("one", 1, "r3"), pinned("three", 9, "r3"));
    let r = s.replace("test:", &new).unwrap();
    assert_eq!(r.before, 2);
    assert_eq!(r.written, 2);
    let mut revs = r.revs.clone();
    revs.sort();
    assert_eq!(revs, ["r1", "r2"]);
    // Only `two` has nothing in its place; it is given as it was written.
    assert_eq!(r.gone, [pinned("two", 5, "r2")]);
    let text = ledger(&s);
    let lines: Vec<&str> = text.lines().filter(|l| !l.trim().is_empty()).collect();
    assert_eq!(lines.len(), 3, "{text}");
    assert!(
        lines.contains(&reader),
        "another machine is left alone: {text}"
    );
    assert!(lines.contains(&pinned("one", 1, "r3").as_str()), "{text}");
    assert!(lines.contains(&pinned("three", 9, "r3").as_str()), "{text}");
    assert!(
        !text.contains("\"r1\"") && !text.contains("\"r2\""),
        "{text}"
    );
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::machine_is_a_snapshot written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_replace_with_nothing_new_says_everything_it_took_out() {
    let root = fresh("replace-empty");
    let s = store_in(&root, CALC);
    // The first run of a machine replaces nothing.
    let first = s
        .replace(
            "test:",
            &format!("{}\n{}", pinned("one", 1, "r1"), pinned("two", 5, "r1")),
        )
        .unwrap();
    assert_eq!((first.before, first.written), (0, 2));
    assert!(first.revs.is_empty() && first.gone.is_empty(), "{first:?}");
    assert_eq!(s.model().unwrap().evidence.len(), 2);
    // A run that observed nothing: all of it leaves, and all of it is shown.
    let r = s.replace("test:", "").unwrap();
    assert_eq!((r.before, r.written), (2, 0));
    assert_eq!(r.revs, ["r1"]);
    let mut gone = r.gone.clone();
    gone.sort();
    let mut all = vec![pinned("one", 1, "r1"), pinned("two", 5, "r1")];
    all.sort();
    assert_eq!(gone, all);
    assert!(s.model().unwrap().evidence.is_empty());
    assert!(!ledger(&s).contains("test:"));
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::human_is_never_dropped_silently written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_a_run_of_a_machine_never_takes_out_what_a_person_said() {
    let root = fresh("human");
    let s = store_in(&root, CALC);
    s.add(
        "evidence r { for calc::commutes holds }",
        who("review:ana"),
        None,
    )
    .unwrap();
    s.add(
        "evidence r { for calc::neg holds }",
        who("review:bob"),
        None,
    )
    .unwrap();
    // A source that only looks like the machine's: `testing:` is not `test:`.
    s.add(
        "evidence r { for calc::add holds }",
        who("testing:carl"),
        None,
    )
    .unwrap();
    s.add(&pinned("one", 1, "r1"), None, None).unwrap();
    let said: Vec<String> = ledger(&s)
        .lines()
        .filter(|l| l.contains("review:") || l.contains("testing:"))
        .map(str::to_string)
        .collect();
    assert_eq!(said.len(), 3);
    // The claim is reworded and `neg` is gone: what was said of them is
    // stale and orphaned, and stays through the next run of the machine.
    let reworded = CALC
        .replace("add(a, b) == add(b, a)", "add(b, a) == add(a, b)")
        .replace("    fn neg(a: Int) -> Int;\n", "");
    assert_ne!(reworded, CALC);
    fs::write(s.model_path(), reworded).unwrap();
    let r = s.replace("test:", &pinned("one", 1, "r2")).unwrap();
    assert_eq!((r.before, r.written), (1, 1));
    assert!(r.gone.is_empty(), "{:?}", r.gone);
    let text = ledger(&s);
    for line in &said {
        assert!(text.lines().any(|l| l == line), "{line} is not in\n{text}");
    }
    // Listed as orphaned or stale, counting for nothing.
    let m = s.model().unwrap();
    let aside: Vec<(&str, bool)> = m
        .aside
        .iter()
        .map(|a| (a.about.as_str(), a.why == mitra_lang::Why::Orphaned))
        .collect();
    assert!(aside.contains(&("calc::neg", true)), "{aside:?}");
    assert!(aside.contains(&("calc::commutes", false)), "{aside:?}");
    assert!(m
        .evidence
        .iter()
        .all(|e| e.source.as_deref() != Some("review:ana")));
    assert!(m
        .evidence
        .iter()
        .all(|e| e.source.as_deref() != Some("review:bob")));
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::sound_or_untouched written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_refused_replace_and_remove_leave_the_ledger_as_it_was() {
    let root = fresh("keep-sound");
    let s = store_in(&root, CALC);
    s.add(
        "evidence r { for calc::commutes holds }",
        who("review:ana"),
        None,
    )
    .unwrap();
    s.replace("test:", &pinned("one", 1, "r1")).unwrap();
    let before = ledger(&s);
    // New evidence on a piece the model does not have.
    let bad = "evidence t { for calc::nope given a = 1 gives 2 source \"test:x\" }";
    assert!(s.replace("test:", bad).is_err());
    assert_eq!(ledger(&s), before);
    // New evidence that does not read.
    assert!(s
        .replace("test:", "evidence t { for calc::add given")
        .is_err());
    assert_eq!(ledger(&s), before);
    // A store whose model no longer reads.
    fs::write(
        s.model_path(),
        "subject calc { fn add(a: Int, b: Int) -> ;;; {{{\n",
    )
    .unwrap();
    assert!(s.replace("test:", "").is_err());
    assert_eq!(ledger(&s), before);
    assert!(s.replace("test:", &pinned("one", 1, "r2")).is_err());
    assert_eq!(ledger(&s), before);
    assert!(s.remove(&Which::Source("review:ana".into())).is_err());
    assert_eq!(ledger(&s), before);
    assert!(s.remove(&Which::Piece("calc::add".into())).is_err());
    assert_eq!(ledger(&s), before);
    // No file is left beside the ledger by the refused writes.
    let mut there: Vec<String> = fs::read_dir(root.join("model"))
        .unwrap()
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect();
    there.sort();
    assert_eq!(there, ["evidence.mitra", "model.mitra"]);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::Store::remove::correct written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_remove_by_source_and_by_piece_gives_back_what_left_as_written() {
    let root = fresh("remove");
    let s = store_in(&root, CALC);
    s.add(
        "evidence r { for calc::commutes holds }",
        who("review:ana"),
        who("r1"),
    )
    .unwrap();
    s.add(
        "evidence r { for calc::neg holds }",
        who("review:ana"),
        who("r1"),
    )
    .unwrap();
    s.add(
        "evidence r { for calc::neg holds }",
        who("review:bob"),
        None,
    )
    .unwrap();
    s.add(&pinned("one", 1, "r1"), None, None).unwrap();
    let lines = |s: &Store| -> Vec<String> {
        ledger(s)
            .lines()
            .filter(|l| !l.trim().is_empty())
            .map(|l| l.trim().to_string())
            .collect()
    };
    let all = lines(&s);
    assert_eq!(all.len(), 4);
    // Nobody of that name: nothing leaves, nothing is rewritten. A source
    // is taken whole, not by how it starts.
    let before = ledger(&s);
    assert_eq!(s.remove(&Which::Source("review:nobody".into())), Ok(vec![]));
    assert_eq!(s.remove(&Which::Source("review:".into())), Ok(vec![]));
    assert_eq!(s.remove(&Which::Source("review:an".into())), Ok(vec![]));
    assert_eq!(s.remove(&Which::Orphaned), Ok(vec![]));
    assert_eq!(s.remove(&Which::Stale(None)), Ok(vec![]));
    assert_eq!(ledger(&s), before);
    // By piece: both said of `neg`, whoever said it, in the order written.
    let gone = s.remove(&Which::Piece("calc::neg".into())).unwrap();
    assert_eq!(gone, [all[1].clone(), all[2].clone()]);
    assert_eq!(lines(&s), [all[0].clone(), all[3].clone()]);
    // By source: what is left of ana.
    let gone = s.remove(&Which::Source("review:ana".into())).unwrap();
    assert_eq!(gone, [all[0].clone()]);
    assert_eq!(lines(&s), [all[3].clone()]);
    // The second time there is nothing to take.
    assert_eq!(s.remove(&Which::Source("review:ana".into())), Ok(vec![]));
    assert_eq!(lines(&s), [all[3].clone()]);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::removal_is_explicit written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_remove_orphaned_and_stale_take_only_those() {
    let root = fresh("remove-aside");
    let s = store_in(&root, CALC);
    s.add(
        "evidence r { for calc::commutes holds }",
        who("review:ana"),
        None,
    )
    .unwrap();
    s.add(
        "evidence r { for calc::neg holds }",
        who("review:bob"),
        None,
    )
    .unwrap();
    s.add(
        "evidence r { for calc::add holds }",
        who("review:carl"),
        None,
    )
    .unwrap();
    let all: Vec<String> = ledger(&s)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect();
    assert_eq!(all.len(), 3);
    let reworded = CALC
        .replace("add(a, b) == add(b, a)", "add(b, a) == add(a, b)")
        .replace("    fn neg(a: Int) -> Int;\n", "");
    fs::write(s.model_path(), reworded).unwrap();
    // Stale on a piece that is not stale: nothing.
    assert_eq!(
        s.remove(&Which::Stale(Some("calc::add".into()))),
        Ok(vec![])
    );
    // Orphaned takes what was said of `neg`, and not the stale one.
    assert_eq!(s.remove(&Which::Orphaned), Ok(vec![all[1].clone()]));
    let text = ledger(&s);
    assert!(text.contains(&all[0]) && text.contains(&all[2]), "{text}");
    // Stale takes what was said of the reworded claim, and not what still
    // judges.
    assert_eq!(s.remove(&Which::Stale(None)), Ok(vec![all[0].clone()]));
    let left: Vec<String> = ledger(&s)
        .lines()
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.trim().to_string())
        .collect();
    assert_eq!(left, [all[2].clone()]);
    let m = s.model().unwrap();
    assert!(m.aside.is_empty());
    assert_eq!(m.evidence.len(), 1);
    let _ = fs::remove_dir_all(root);
}

// mitra: evidence for keeping::nothing_leaves_unshown written "agent:anthropic/fable-5.2/cold-w-store" cold
#[test]
fn cold_every_entry_that_leaves_is_given_back() {
    let root = fresh("unshown");
    let s = store_in(&root, CALC);
    // An entry over several lines, as a person may write it in the file.
    let long = "evidence t {\n    for calc::add given a = 7, b = 2 gives 9\n    source \"test:long\" rev \"r1\"\n}";
    s.append(long).unwrap();
    s.add(&pinned("one", 1, "r1"), None, None).unwrap();
    s.add(
        "evidence r { for calc::neg holds }",
        who("review:bob"),
        None,
    )
    .unwrap();
    let count = |s: &Store| s.model().unwrap().evidence.len();
    assert_eq!(count(&s), 3);
    // Taken out by a machine: both are shown, the long one whole.
    let r = s.replace("test:", "").unwrap();
    assert_eq!(r.before, 2);
    assert_eq!(r.gone.len(), 2, "{:?}", r.gone);
    assert!(r.gone.contains(&long.to_string()), "{:?}", r.gone);
    assert!(r.gone.contains(&pinned("one", 1, "r1")), "{:?}", r.gone);
    assert_eq!(count(&s), 1);
    // Taken out by `remove`: as many are given back as left.
    let bob = ledger(&s).trim().to_string();
    let gone = s.remove(&Which::Piece("calc::neg".into())).unwrap();
    assert_eq!(gone, [bob]);
    assert_eq!(count(&s), 0);
    assert_eq!(ledger(&s).trim(), "");
    let _ = fs::remove_dir_all(root);
}
