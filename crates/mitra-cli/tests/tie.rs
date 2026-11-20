//! `mitra tie` from the command line: what the reader of the code says of a
//! function, a record, an enum, an implementer, and what the settings change.

use std::process::Command;

/// `mitra` run in `dir`, with its exit code, stdout and stderr.
fn mitra(dir: &std::path::Path, args: &[&str]) -> (Option<i32>, String, String) {
    let out = Command::new(env!("CARGO_BIN_EXE_mitra"))
        .current_dir(dir)
        .args(args)
        .output()
        .unwrap();
    let s = |b: Vec<u8>| String::from_utf8(b).unwrap();
    (out.status.code(), s(out.stdout), s(out.stderr))
}

/// A new store holding `model`.
fn store(name: &str, model: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("mitra-cli-{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("sub")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    std::fs::write(dir.join("model/model.mitra"), model).unwrap();
    dir
}

/// A model aside from the code, tied by `// mitra: implements` comments:
/// `mitra tie` compares the two and records what it finds as evidence.
#[test]
fn tie_compares_the_code_with_the_model() {
    let from = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/tie/bank");
    let dir = std::env::temp_dir().join(format!("mitra-cli-tie-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    for f in ["model/model.mitra", "src/lib.rs"] {
        std::fs::copy(from.join(f), dir.join(f)).unwrap();
    }
    let (code, out, err) = mitra(&dir, &["tie", "--rev", "abc"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(
        out.starts_with("2 agree, 2 differ; 4 evidence written"),
        "{out}"
    );
    assert!(
        out.contains("input `amount` in the code is `n` in the model"),
        "{out}"
    );
    assert!(out.contains("`withdraw` also calls `deposit`"), "{out}");
    assert!(out.contains("no code is tied to: bank::close"), "{out}");
    let (code, out, _) = mitra(&dir, &["eval"]);
    assert_eq!(code, Some(1));
    assert!(out.contains("bank::deposit::tied: holds"), "{out}");
    assert!(out.contains("bank::withdraw::tied: fails"), "{out}");
    assert!(
        out.contains("The model and the code of `bank::withdraw` do not agree"),
        "{out}"
    );
    // A test tied to a claim by a comment is evidence for that claim.
    let report = "test deposit_adds ... ok\ntest other ... FAILED\n";
    std::fs::write(dir.join("report.txt"), report).unwrap();
    let (code, out, err) = mitra(&dir, &["ingest", "report.txt", "--rev", "abc"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.contains("1 evidence written"), "{out}");
    assert!(
        out.contains("tied to no claim, so no verdict shows it: other"),
        "{out}"
    );
    let (_, out, _) = mitra(&dir, &["eval"]);
    assert!(out.contains("deposit_adds: holds"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// A record is tied to its struct: the fields the model states are
/// there, and no other unless the record says `..`.
#[test]
fn tie_compares_a_record_with_its_struct() {
    let from = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/tie/records");
    let dir = std::env::temp_dir().join(format!("mitra-cli-records-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    for f in ["model/model.mitra", "src/lib.rs"] {
        std::fs::copy(from.join(f), dir.join(f)).unwrap();
    }
    let (code, out, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.starts_with("5 agree, 1 differ"), "{out}");
    assert!(
        out.contains("field `y` is `String` in the code, `Int` in the model"),
        "{out}"
    );
    // A field the record does not state is forgotten, and said.
    let ledger = std::fs::read_to_string(dir.join("model/evidence.mitra")).unwrap();
    assert!(ledger.contains("erased 1: field(b)"), "{ledger}");
    let (code, out, _) = mitra(&dir, &["eval"]);
    assert_eq!(code, Some(1));
    for want in [
        "check::Model::tied: holds",
        "check::Claim::tied: holds",
        "check::Point::tied: fails",
    ] {
        assert!(out.contains(want), "{out}");
    }
    assert!(
        out.contains("The record `check::Point` and its struct do not agree"),
        "{out}"
    );
    assert!(!out.contains("::prose"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// An enum is tied to the enum of the code: the same variants, each
/// holding as many values, of types that fit.
#[test]
fn tie_compares_an_enum_with_its_enum() {
    let dir = std::env::temp_dir().join(format!("mitra-cli-enums-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    let model = "subject shapes {\n\
                 enum Shape { Dot, Circle(radius: f64), Box(w: f64, h: f64) }\n\
                 enum Side { Left, Right }\n\
                 enum Unit { Mm(n: Int), Inch(n: Int) }\n}\n";
    let code = "// mitra: implements shapes::Shape\n\
                pub enum Shape { Dot, Circle(f64), Box { w: f64, h: f64 } }\n\
                // mitra: implements shapes::Side\n\
                pub enum Side { Left, Right, Up }\n\
                // mitra: implements shapes::Unit\n\
                pub enum Unit { Mm(i64), Inch(String) }\n";
    std::fs::write(dir.join("model/model.mitra"), model).unwrap();
    std::fs::write(dir.join("src/lib.rs"), code).unwrap();
    let (code, out, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.starts_with("1 agree, 2 differ"), "{out}");
    assert!(
        out.contains("the enum has a variant `Up` the model does not state"),
        "{out}"
    );
    assert!(
        out.contains("`n` of variant `Inch` is `String` in the code, `Int` in the model"),
        "{out}"
    );
    let (_, out, _) = mitra(&dir, &["eval"]);
    for want in ["shapes::Shape::tied: holds", "shapes::Side::tied: fails"] {
        assert!(out.contains(want), "{out}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A method of `impl Trait for Type` is tied as the function the
/// contract gives its implementer; `impl Trait` in the code fits an
/// `impl Contract` of the model.
#[test]
fn tie_compares_an_implementer_with_its_impl() {
    let dir = std::env::temp_dir().join(format!("mitra-tie-contract-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    let model = "contract Shape { fn area(&self) -> Int; fn scale(&self, by: Int) -> Self; }\n\
                 subject shapes {\n\
                 record Square { side: Int }\n\
                 record Dot;\n\
                 impl Shape for Square { }\n\
                 impl Shape for Dot { }\n\
                 fn total(s: impl Shape) -> Int uses Shape::area;\n}\n";
    let code = "pub trait Shape { fn area(&self) -> i64; fn scale(&self, by: i64) -> Self; }\n\
                // mitra: implements shapes::Square\n\
                pub struct Square { side: i64 }\n\
                pub struct Dot;\n\
                impl Shape for Square {\n\
                // mitra: implements shapes::Square::area\n\
                fn area(&self) -> i64 { self.side * self.side }\n\
                // mitra: implements shapes::Square::scale\n\
                fn scale(&self, by: i64) -> Self { Square { side: self.side * by } }\n\
                }\n\
                impl Shape for Dot {\n\
                // mitra: implements shapes::Dot::area\n\
                fn area(&self) -> String { String::new() }\n\
                fn scale(&self, by: i64) -> Self { Dot }\n\
                }\n\
                // mitra: implements shapes::total\n\
                pub fn total(s: impl Shape) -> i64 { s.area() }\n";
    std::fs::write(dir.join("model/model.mitra"), model).unwrap();
    std::fs::write(dir.join("src/lib.rs"), code).unwrap();
    let (code, out, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.starts_with("5 agree, 1 differ"), "{out}");
    // A function of the contract is called by its name: `uses Shape::area`.
    let ledger = std::fs::read_to_string(dir.join("model/evidence.mitra")).unwrap();
    assert!(ledger.contains("for shapes::total::uses holds"), "{ledger}");
    assert!(out.contains("shapes::Dot::area"), "{out}");
    // What the contract gives an implementer and no code is tied to.
    let (_, out, _) = mitra(&dir, &["conform"]);
    assert!(out.contains("shapes::Dot::scale"), "{out}");
    assert!(!out.contains("Shape::area\n"), "{out}");
    let _ = std::fs::remove_dir_all(&dir);
}

/// Type parameters and bounds are compared with the code: a record
/// with a parameter with its struct, an input bound by a contract with
/// the traits that bound it.
#[test]
fn tie_compares_bounds_and_type_parameters() {
    let dir = std::env::temp_dir().join(format!("mitra-tie-generics-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    let model = "import std::core::{Option};\n\
                 contract Link { fn send(&self, bytes: List<Int>) -> Bool; }\n\
                 subject sync {\n\
                 fn push<L: Link>(l: L, data: List<Int>) -> Bool uses Link::send;\n\
                 fn pull<L>(l: &L) -> List<Int> where L: Link;\n\
                 fn more<L: Link, M>(l: L, m: M) -> Bool;\n\
                 fn both(a: impl Link, b: impl Link, c: impl Link) -> Bool;\n\
                 record Stack<T> { items: List<T> }\n\
                 record Pair<A, B> { first: A }\n\
                 impl<T> Stack<T> { fn top(&self) -> Option<T>;\n\
                 fn swap<U: Link>(&self, u: U) -> Stack<U>; }\n}\n";
    let code = "pub trait Link { fn send(&self, bytes: Vec<i64>) -> bool; }\n\
                // mitra: implements sync::push\n\
                pub fn push<L: Link + Clone>(l: L, data: Vec<i64>) -> bool { l.send(data) }\n\
                // mitra: implements sync::pull\n\
                pub fn pull<L>(l: &L) -> Vec<i64> where L: Clone { vec![] }\n\
                // mitra: implements sync::more\n\
                pub fn more<L: Link + Ord, M: Send + 'static>(l: L, m: M) -> bool { true }\n\
                // mitra: implements sync::both\n\
                pub fn both<C>(a: impl Link, b: &dyn Link, c: C) -> bool\n\
                where C: crate::Link { true }\n\
                // mitra: implements sync::Stack\n\
                pub struct Stack<T> { items: Vec<T> }\n\
                // mitra: implements sync::Pair\n\
                pub struct Pair<A, B> { first: A, second: B }\n\
                impl<T> Stack<T> {\n\
                // mitra: implements sync::Stack::top\n\
                pub fn top(&self) -> Option<T> { None }\n\
                // mitra: implements sync::Stack::swap\n\
                pub fn swap<U>(&self, u: U) -> Stack<U> { Stack { items: vec![] } }\n\
                }\n";
    std::fs::write(dir.join("model/model.mitra"), model).unwrap();
    std::fs::write(dir.join("src/lib.rs"), code).unwrap();
    let (code, out, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.starts_with("7 agree, 2 differ"), "{out}");
    for want in [
        "sync::pull::tied: input `l` is bound by `Link` in the model, by `Clone` in the code",
        "sync::Stack::swap::tied: input `u` is bound by `Link` in the model, by nothing in the code",
            ] {
        assert!(out.contains(want), "{out}");
    }
    // A contract is nobody's code.
    assert!(!out.contains("no code is tied to"), "{out}");
    // A bound of the code alone is a detail of the code: said, no drift.
    let ledger = std::fs::read_to_string(dir.join("model/evidence.mitra")).unwrap();
    assert!(
        ledger.contains("for sync::push::tied holds")
            && ledger.contains("bound(l: Clone)")
            && ledger.contains("for sync::more::tied holds trust 0.96")
            && ledger.contains("bound(l: Ord), bound(m: Send)")
            && ledger.contains("for sync::Pair::tied holds trust 0.95")
            && ledger.contains("erased 1: field(second)"),
        "{ledger}"
    );
    // The interface is written with the parameters and the bounds.
    assert_eq!(mitra(&dir, &["build"]).0, Some(0));
    let interface = std::fs::read_to_string(dir.join(".mitra/build/interface.mitra")).unwrap();
    for want in [
        "fn push<L: Link>(l: L, data: List<Int>) -> Bool;",
        "fn pull<L>(l: &L) -> List<Int> where L: Link;",
        "fn both(a: impl Link, b: impl Link, c: impl Link) -> Bool;",
        "record Stack<T> { items: List<T> }",
        "fn swap<T, U: Link>(self: sync::Stack<T>, u: U) -> sync::Stack<U>;",
    ] {
        assert!(interface.contains(want), "no `{want}` in\n{interface}");
    }
    let _ = std::fs::remove_dir_all(&dir);
}

/// A method of an `impl` of the model is tied to the method of the
/// code: `self` in any form, `Self` the type of the `impl`.
#[test]
fn tie_compares_a_method_with_its_code() {
    let from = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/tie/methods");
    let dir = std::env::temp_dir().join(format!("mitra-cli-methods-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    for f in ["model/model.mitra", "src/lib.rs"] {
        std::fs::copy(from.join(f), dir.join(f)).unwrap();
    }
    let (code, out, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.starts_with("4 agree, 1 differ"), "{out}");
    let why = "bank::Account::close::tied: the result is `Account` in the code, `Int` in the model";
    assert!(out.contains(why), "{out}");
}

/// A function that states no `uses` calls nothing outside its subject:
/// the reader says so when the text settles it, and only then.
#[test]
fn tie_reports_a_call_the_model_does_not_state() {
    let model = "subject a { fn f(x: Int) -> Int; fn h(x: Int) -> Int; }\n\
                 subject b { record R { n: Int } impl R { fn size(&self) -> Int; }\n\
                 fn g(x: Int) -> Int; fn k(x: Int) -> Int; }";
    let dir = store("implied", model);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    let code = "// mitra: implements a::f\npub fn f(x: i64) -> i64 { g(x) + h(x) }\n\
                // mitra: implements a::h\npub fn h(x: i64) -> i64 {\n    let k = |y: i64| y;\n    \
                k(x) + x.to_string().size() as i64\n}\n\
                // mitra: implements b::g\npub fn g(x: i64) -> i64 { x }\n\
                // mitra: implements b::k\npub fn k(x: i64) -> i64 { x }\n\
                // mitra: implements b::R\npub struct R { pub n: i64 }\n\
                impl R {\n    // mitra: implements b::R::size\n    pub fn size(&self) -> i64 { self.n }\n}\n";
    std::fs::write(dir.join("src/lib.rs"), code).unwrap();
    let (code, out, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(0), "{err}");
    let why = "a::f::uses: `f` calls `g`, which is `b::g`, and the model states no `uses`";
    assert!(out.contains(why), "{out}");
    // A `let` of the body and a method through a dot settle nothing.
    assert!(!out.contains("a::h::uses"), "{out}");
    let (_, out, _) = mitra(&dir, &["eval"]);
    assert!(out.contains("a::f::uses: fails"), "{out}");
    let _ = std::fs::remove_dir_all(dir);
}

/// What the code adds to hold or hand over a value is no drift: it is
/// said with the tie. A different result is one.
#[test]
fn tie_takes_boilerplate_as_no_drift() {
    let dir = std::env::temp_dir().join(format!("mitra-tie-held-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    let model = "subject s {\n\
                 record Path;\n\
                 fn a(p: Path, name: String) -> Path;\n\
                 fn e(xs: List<Int>, label: String) -> String;\n\
                 fn g(x: Int) -> Int;\n}\n";
    let code = "// mitra: implements s::Path\npub struct Path;\n\
                // mitra: implements s::a\n\
                pub fn a<'a>(p: &'a Path, name: Cow<'a, str>) -> PathBuf { todo!() }\n\
                // mitra: implements s::e\n\
                pub fn e(xs: impl Iterator<Item = i64>, label: impl Into<String>) -> &'static str { \"\" }\n\
                // mitra: implements s::g\n\
                pub fn g(x: i64) -> Option<i64> { None }\n";
    std::fs::write(dir.join("model/model.mitra"), model).unwrap();
    std::fs::write(dir.join("src/lib.rs"), code).unwrap();
    let (code, out, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(0), "{err}");
    assert!(out.starts_with("3 agree, 1 differ"), "{out}");
    assert!(
        out.contains("s::g::tied: the result is `Option<i64>` in the code"),
        "{out}"
    );
    let ledger = std::fs::read_to_string(dir.join("model/evidence.mitra")).unwrap();
    assert!(
        ledger.contains("for s::a::tied holds trust 0.98")
            && ledger
                .contains("erased 4: reference(p), reference(name), owned(name), owned(result)"),
        "{ledger}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}

/// The settings of a store: read from `model/settings.toml`, written
/// in force by `build`, and what the trust of a tie and the weight of a
/// sample come from.
#[test]
fn settings_set_the_trust_of_a_tie_and_of_a_sample() {
    let dir = std::env::temp_dir().join(format!("mitra-settings-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(dir.join("src")).unwrap();
    assert_eq!(mitra(&dir, &["init"]).0, Some(0));
    let model = "subject s {\n\
                 fn abs(x: -5..=5) -> 0..=5;\n\
                 claim never_negative(x) { abs(x) >= 0 }\n}\n";
    let code = "// mitra: implements s::abs\npub fn abs(x: &i64) -> i64 { x.abs() }\n";
    std::fs::write(dir.join("model/model.mitra"), model).unwrap();
    std::fs::write(dir.join("src/lib.rs"), code).unwrap();
    let ledger = dir.join("model/evidence.mitra");
    let sample = "evidence t { for s::never_negative sampled 1000 source \"review:me\" }\n";
    std::fs::write(&ledger, sample).unwrap();
    // The defaults.
    let (_, out, _) = mitra(&dir, &["eval"]);
    let want = "s::never_negative: unverified\n  \
                sampled on      1000 drawn from its inputs (review:me), trust 0.63 at tolerance 0.001\n";
    assert!(out.contains(want), "{out}");
    assert_eq!(mitra(&dir, &["tie"]).0, Some(0));
    let read = || std::fs::read_to_string(&ledger).unwrap();
    let tie = "for s::abs::tied holds trust 0.884 act tie by \"machine:mitra-rust\" source \"reader:rust src/lib.rs:1; \
               erased 4: reference(x), range(x: -5..=5 for &i64), range(result: 0..=5 for i64), width\"";
    assert!(read().contains(tie), "{}", read());
    assert_eq!(mitra(&dir, &["build"]).0, Some(0));
    let written = std::fs::read_to_string(dir.join(".mitra/build/settings.toml")).unwrap();
    assert!(
        written.starts_with(
            "[sample]\ntolerance = 0.001\ncounts = false\n\n[erasure]\nreference = 0\n"
        ),
        "{written}"
    );
    // A file: another tolerance, a sample that counts, a cost.
    let settings = dir.join("model/settings.toml");
    let text = "[sample]\ntolerance = 0.01\ncounts = true\n\n[erasure]\nrange = 0\nwidth = 0.5\n";
    std::fs::write(&settings, text).unwrap();
    let (_, out, _) = mitra(&dir, &["eval"]);
    let want = "s::never_negative: holds assuming [s::never_negative::sample]\n  \
                sampled on      1000 drawn from its inputs (review:me), trust 1 at tolerance 0.01\n";
    assert!(out.contains(want), "{out}");
    assert_eq!(mitra(&dir, &["tie"]).0, Some(0));
    assert!(
        read().contains("for s::abs::tied holds trust 0.5 act tie"),
        "{}",
        read()
    );
    assert_eq!(mitra(&dir, &["build"]).0, Some(0));
    let written = std::fs::read_to_string(dir.join(".mitra/build/settings.toml")).unwrap();
    assert!(
        written.contains("width = 0.5\n") && written.contains("field = 0.05\n"),
        "{written}"
    );
    // What is not a setting is refused, and so is an erasure the reader
    // does not have.
    std::fs::write(&settings, "[sample]\ntolerence = 0.01\n").unwrap();
    let (code, _, err) = mitra(&dir, &["eval"]);
    assert_eq!(code, Some(2));
    assert!(
        err.contains("settings.toml: line 2: no setting `tolerence` in `[sample]`"),
        "{err}"
    );
    std::fs::write(&settings, "[erasure]\nwidht = 0.1\n").unwrap();
    let (code, _, err) = mitra(&dir, &["tie"]);
    assert_eq!(code, Some(2));
    assert!(
        err.contains("no erasure `widht` in the profile of this reader"),
        "{err}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
