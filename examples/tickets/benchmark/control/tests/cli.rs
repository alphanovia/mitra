use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Output};

/// A fresh, empty directory under cargo's target tmp dir.
fn tmpdir(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = fs::remove_dir_all(&d);
    fs::create_dir_all(&d).unwrap();
    d
}

fn run(dir: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_tickets"))
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap()
}

fn ok(dir: &PathBuf, args: &[&str]) -> String {
    let o = run(dir, args);
    assert!(o.status.success(), "{args:?}: {}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8(o.stdout).unwrap()
}

fn fails(dir: &PathBuf, args: &[&str]) {
    let o = run(dir, args);
    assert!(!o.status.success(), "{args:?} should fail");
    assert_ne!(o.status.code(), Some(0));
    assert!(o.stdout.is_empty(), "{args:?} printed {:?}", o.stdout);
}

#[test]
fn add_done_list_show() {
    let d = tmpdir("basic");
    assert_eq!(ok(&d, &["list"]), "");
    assert_eq!(ok(&d, &["add", "fix login"]), "#1\n");
    assert_eq!(ok(&d, &["add", "write", "docs"]), "#2\n");
    assert_eq!(ok(&d, &["add", "ship it"]), "#3\n");
    assert_eq!(ok(&d, &["done", "2"]), "");
    assert_eq!(
        ok(&d, &["list"]),
        "#1 [ ] fix login\n#2 [x] write docs\n#3 [ ] ship it\n"
    );
    assert_eq!(ok(&d, &["show", "2"]), "#2 [x] write docs\n");
    assert_eq!(ok(&d, &["show", "#1"]), "#1 [ ] fix login\n");
    assert!(d.join("tickets.log").exists());
}

#[test]
fn done_stays_done() {
    let d = tmpdir("stays_done");
    ok(&d, &["add", "a"]);
    assert_eq!(ok(&d, &["done", "1"]), "");
    assert_eq!(ok(&d, &["done", "1"]), "");
    ok(&d, &["add", "b"]);
    assert_eq!(ok(&d, &["list"]), "#1 [x] a\n#2 [ ] b\n");
}

#[test]
fn bad_input_errors() {
    let d = tmpdir("bad_input");
    ok(&d, &["add", "a"]);
    fails(&d, &[]);
    fails(&d, &["frob"]);
    fails(&d, &["add"]);
    fails(&d, &["add", ""]);
    fails(&d, &["add", "   "]);
    fails(&d, &["add", "two\nlines"]);
    fails(&d, &["add", "tab\there"]);
    fails(&d, &["done"]);
    fails(&d, &["done", "x"]);
    fails(&d, &["done", "0"]);
    fails(&d, &["done", "-1"]);
    fails(&d, &["done", "9"]);
    fails(&d, &["done", "1", "2"]);
    fails(&d, &["show", "9"]);
    fails(&d, &["show"]);
    fails(&d, &["list", "extra"]);
    // Nothing changed.
    assert_eq!(ok(&d, &["list"]), "#1 [ ] a\n");
}

#[test]
fn cut_last_line_is_ignored() {
    let d = tmpdir("cut");
    ok(&d, &["add", "one"]);
    ok(&d, &["add", "two"]);
    ok(&d, &["done", "2"]);
    let log = d.join("tickets.log");
    let full = fs::read(&log).unwrap();
    let before_done = {
        let s = &full[..full.len() - 1];
        s.iter().rposition(|&b| b == b'\n').unwrap() + 1
    };
    for cut in before_done..full.len() {
        fs::write(&log, &full[..cut]).unwrap();
        assert_eq!(ok(&d, &["list"]), "#1 [ ] one\n#2 [ ] two\n", "cut {cut}");
    }
    // After a cut, new changes are still recorded and read back.
    // Only the final newline lost: the cut record must stay ignored.
    fs::write(&log, &full[..full.len() - 1]).unwrap();
    assert_eq!(ok(&d, &["add", "three"]), "#3\n");
    ok(&d, &["done", "1"]);
    assert_eq!(ok(&d, &["list"]), "#1 [x] one\n#2 [ ] two\n#3 [ ] three\n");
}

#[test]
fn garbage_is_never_misread() {
    let d = tmpdir("garbage");
    ok(&d, &["add", "one"]);
    let log = d.join("tickets.log");
    let mut f = fs::OpenOptions::new().append(true).open(&log).unwrap();
    f.write_all(b"A 5 sneaky\n00000000 D 1\n\xff\xfe\n").unwrap();
    f.write_all(b"deadbeef A 7 x").unwrap(); // torn, no newline
    drop(f);
    assert_eq!(ok(&d, &["list"]), "#1 [ ] one\n");
    assert_eq!(ok(&d, &["add", "two"]), "#2\n");
    assert_eq!(ok(&d, &["list"]), "#1 [ ] one\n#2 [ ] two\n");
}

#[test]
fn ids_unique_and_increasing() {
    let d = tmpdir("ids");
    for i in 1..=12 {
        assert_eq!(ok(&d, &["add", &format!("t{i}")]), format!("#{i}\n"));
    }
    let out = ok(&d, &["list"]);
    let ids: Vec<u64> = out
        .lines()
        .map(|l| l[1..l.find(' ').unwrap()].parse().unwrap())
        .collect();
    assert_eq!(ids, (1..=12).collect::<Vec<_>>());
}

#[test]
fn unwritable_log_is_an_error() {
    let d = tmpdir("unwritable");
    // A directory named tickets.log cannot be read or appended to.
    fs::create_dir(d.join("tickets.log")).unwrap();
    fails(&d, &["add", "x"]);
    fails(&d, &["list"]);
}
