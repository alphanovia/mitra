//! End to end: the binary, run in a folder of its own, on its tickets.log.

use std::fs;
use std::path::PathBuf;
use std::process::Command;

struct Run {
    out: String,
    code: i32,
}

fn folder(name: &str) -> PathBuf {
    let d = std::env::temp_dir().join(format!("tickets-test-{}-{}", std::process::id(), name));
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn tickets(dir: &PathBuf, args: &[&str]) -> Run {
    let o = Command::new(env!("CARGO_BIN_EXE_tickets")).args(args).current_dir(dir).output().unwrap();
    Run { out: String::from_utf8(o.stdout).unwrap(), code: o.status.code().unwrap_or(-1) }
}

fn ok(dir: &PathBuf, args: &[&str]) -> String {
    let r = tickets(dir, args);
    assert_eq!(r.code, 0, "{:?}", args);
    r.out
}

fn fails(dir: &PathBuf, args: &[&str]) {
    let r = tickets(dir, args);
    assert_ne!(r.code, 0, "{:?}", args);
    assert_eq!(r.out, "", "{:?}", args);
}

#[test]
fn add_done_list_show() {
    let d = folder("basic");
    assert_eq!(ok(&d, &["list"]), "");
    assert_eq!(ok(&d, &["add", "fix login"]), "#1\n");
    assert_eq!(ok(&d, &["add", "write docs"]), "#2\n");
    assert_eq!(ok(&d, &["done", "1"]), "");
    assert_eq!(ok(&d, &["done", "1"]), "");
    assert_eq!(ok(&d, &["list"]), "#1 [x] fix login\n#2 [ ] write docs\n");
    assert_eq!(ok(&d, &["show", "2"]), "#2 [ ] write docs\n");
    assert_eq!(ok(&d, &["add", "third"]), "#3\n");
}

#[test]
fn failures_print_nothing() {
    let d = folder("fail");
    fails(&d, &[]);
    fails(&d, &["add"]);
    fails(&d, &["add", ""]);
    fails(&d, &["done", "three"]);
    fails(&d, &["done", "1"]);
    fails(&d, &["show", "1"]);
    fails(&d, &["frobnicate"]);
    // nothing was written by a failure
    assert_eq!(ok(&d, &["add", "first"]), "#1\n");
}

/// A crash that cut the last line: the cut line is ignored, never misread,
/// and the next change is still read whole after it.
#[test]
fn a_cut_last_line_is_ignored() {
    let d = folder("cut");
    assert_eq!(ok(&d, &["add", "keep me"]), "#1\n");
    assert_eq!(ok(&d, &["add", "lost"]), "#2\n");
    let log = d.join("tickets.log");
    let text = fs::read(&log).unwrap();
    // every cut of the last line, newline included
    let start = text[..text.len() - 1].iter().rposition(|b| *b == b'\n').unwrap() + 1;
    for cut in start..text.len() {
        fs::write(&log, &text[..cut]).unwrap();
        assert_eq!(ok(&d, &["list"]), if cut == text.len() - 1 {
            // only the newline is gone: the line is whole
            "#1 [ ] keep me\n#2 [ ] lost\n"
        } else {
            "#1 [ ] keep me\n"
        }, "cut at {}", cut);
    }
    // cut inside the last line, then a new change: both whole lines read back
    fs::write(&log, &text[..start + 4]).unwrap();
    assert_eq!(ok(&d, &["add", "after the crash"]), "#2\n");
    assert_eq!(ok(&d, &["done", "2"]), "");
    assert_eq!(ok(&d, &["list"]), "#1 [ ] keep me\n#2 [x] after the crash\n");
}

/// A change acknowledged is in the log, flushed, before the tool answers:
/// a fresh run reads it back.
#[test]
fn an_acknowledged_change_is_on_disk() {
    let d = folder("durable");
    let n = 20;
    for i in 1..=n {
        assert_eq!(ok(&d, &["add", &format!("t{}", i)]), format!("#{}\n", i));
        let log = fs::read_to_string(d.join("tickets.log")).unwrap();
        assert!(log.ends_with('\n'));
        assert_eq!(log.lines().count(), i);
    }
}

/// A log the tool cannot write: the change is not acknowledged.
#[test]
fn a_failed_write_is_not_acknowledged() {
    let d = folder("storage");
    // tickets.log is a folder: it can be neither read nor written
    fs::create_dir_all(d.join("tickets.log")).unwrap();
    fails(&d, &["add", "x"]);
    fails(&d, &["list"]);
}

/// Commands run at once never share an id, and each id acknowledged is there.
#[test]
fn concurrent_adds_get_unique_ids() {
    let d = folder("race");
    let handles: Vec<_> = (0..8)
        .map(|i| {
            let d = d.clone();
            std::thread::spawn(move || ok(&d, &["add", &format!("t{}", i)]))
        })
        .collect();
    let mut ids: Vec<String> = handles.into_iter().map(|h| h.join().unwrap()).collect();
    ids.sort();
    ids.dedup();
    assert_eq!(ids.len(), 8);
    let list = ok(&d, &["list"]);
    assert_eq!(list.lines().count(), 8);
}
