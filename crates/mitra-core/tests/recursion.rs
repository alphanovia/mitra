//! Types that contain themselves, specs that call themselves, and the
//! functions over each element: what is refused.

fn error(model: &str, evidence: &str) -> String {
    match mitra_core::load(model, evidence) {
        Err((_, e)) => e.message,
        Ok(m) => match mitra_core::evaluate(&m) {
            Err((_, e)) => e.message,
            // Evidence that does not fit a signature is kept, and reported.
            Ok(_) => m.misfits.first().map_or(String::new(), |m| m.1.clone()),
        },
    }
}

const EXPR: &str = "enum Expr { Lit(n: Int), Add(l: Expr, r: Expr) }\n\
                    subject fn eval(e: Expr) -> Int;\n\
                    claim eval_ok(e) { correct(eval(e)) }\n";

/// `evidence e { for f given … }`
fn on(f: &str, given: &str, says: &str) -> String {
    format!("evidence e {{ for {f} given {given} {says} }}")
}

#[test]
fn a_value_is_checked_all_the_way_down() {
    assert_eq!(
        error(EXPR, &on("eval", "e = Add(Lit(1), Lit(2))", "holds")),
        ""
    );
    let e = error(EXPR, &on("eval", "e = Add(Lit(1), 2)", "holds"));
    assert!(e.contains("outside the inputs"), "{e}");
    let e = error(
        EXPR,
        &on("eval", "e = Add(Lit(1), Add(Lit(1), Lit(true)))", "holds"),
    );
    assert!(e.contains("outside the inputs"), "{e}");
}

#[test]
fn a_type_needs_a_value() {
    let e = error("enum Loop { More(next: Loop) }", "");
    assert!(e.contains("no value can be built"), "{e}");
    let e = error("record Loop { next: Loop }", "");
    assert!(e.contains("no value can be built"), "{e}");
    assert_eq!(error("record Dir { children: List<Dir> }", ""), "");
}

#[test]
fn a_type_inside_itself_keeps_its_parameters() {
    let tree = "enum Tree<T> { Leaf, Node(l: Tree<T>, v: T, r: Tree<T>) }\n\
                subject fn size(t: Tree<Int>) -> Int;\n\
                claim size_ok(t) { correct(size(t)) }\n";
    assert_eq!(
        error(tree, &on("size", "t = Node(Leaf, 1, Leaf)", "holds")),
        ""
    );
    let e = error(
        tree,
        &on(
            "size",
            "t = Node(Leaf, 1, Node(Leaf, \"a\", Leaf))",
            "holds",
        ),
    );
    assert!(e.contains("outside the inputs"), "{e}");
    let e = error("enum Tree<T> { Leaf, Node(l: Tree<Int>, v: T) }", "");
    assert!(e.contains("is written `Tree<Any>`"), "{e}");
}

#[test]
fn a_spec_calls_itself_on_a_piece_only() {
    let expr = "enum Expr { Lit(n: Int), Add(l: Expr, r: Expr) }\n";
    let ok =
        "fn value(e: Expr) -> Int { match e { Lit(n) => n, Add(l, r) => value(l) + value(r) } }";
    assert_eq!(error(&format!("{expr}{ok}"), ""), "");
    for bad in [
        "fn value(e: Expr) -> Int { value(e) }",
        "fn value(e: Expr) -> Int { match e { Lit(n) => n, Add(l, r) => value(Add(l, r)) } }",
        "fn down(n: Int) -> Int { if n == 0 { 0 } else { down(n - 1) } }",
        // Each call goes down, but not on the same input.
        "fn both(a: Expr, b: Expr) -> Int { match a { Lit(n) => match b { Lit(m) => 0, \
         Add(l, r) => both(a, l) }, Add(l, r) => both(l, b) } }",
    ] {
        let e = error(&format!("{expr}{bad}"), "");
        assert!(e.contains("calls itself"), "{bad}: {e}");
    }
    let e = error(
        "fn a(n: Int) -> Int { b(n) }\nfn b(n: Int) -> Int { a(n) }",
        "",
    );
    assert!(e.contains("not a spec defined above"), "{e}");
}

#[test]
fn each_element() {
    let model = "claim c() { fold(x in [1, 2, 3], let s = 0; s + x) == 6 \
                 && map(x in {1, 2}, x / 2) == {0, 1} \
                 && filter(x in [1, 2, 3], x > 1) == [2, 3] \
                 && all(x in [], x > 1) && !any(x in [], x > 1) }";
    assert_eq!(error(model, ""), "");
    let m = mitra_core::load(model, "").unwrap();
    let report = mitra_core::evaluate(&m).unwrap();
    assert!(format!("{report:?}").contains("Holds"), "{report:?}");
    let e = error("claim c() { all(x in 3, x > 1) }", "");
    assert!(e.contains("goes over a list or a set"), "{e}");
    let e = error("claim c() { all(x in [1], x + 1) }", "");
    assert!(e.contains("`all` takes `Bool`"), "{e}");
    let e = error("claim c() { fold(x in [1], x) == 1 }", "");
    assert!(e.contains("`fold` starts from a value"), "{e}");
}

#[test]
fn type_parameters() {
    let pair = "record Pair<A, B> { first: A, second: B }\n\
                subject fn ints(p: Pair<Int, String>) -> Int;\n\
                claim ints_ok(p) { correct(ints(p)) }\n";
    assert_eq!(error(pair, &on("ints", "p = Pair(1, \"a\")", "holds")), "");
    let e = error(pair, &on("ints", "p = Pair(\"a\", 1)", "holds"));
    assert!(e.contains("outside the inputs"), "{e}");
    let e = error(
        "record Pair<A, B> { first: A }\nfn f(p: Pair<Int>) -> Int;",
        "",
    );
    assert!(e.contains("takes 2 type(s)"), "{e}");
    let e = error("subject fn f<T, T>(x: T) -> T;", "");
    assert!(e.contains("two parameters `T`"), "{e}");
    // A type parameter does not outlive its function.
    let e = error("subject fn f<T>(x: T) -> T;\nfn g(x: T) -> Int;", "");
    assert!(e.contains("unknown type `T`"), "{e}");
}

#[test]
fn function_types() {
    let model = "fn double(x: Int) -> Int { x * 2 }\n\
                 fn shout(s: String) -> String { s }\n\
                 subject fn apply(f: fn(Int) -> Int, x: Int) -> Int;\n\
                 claim apply_is(f, x) { apply(f, x) == f(x) }\n";
    assert_eq!(
        error(model, &on("apply", "f = double, x = 3", "gives 6")),
        ""
    );
    let e = error(model, &on("apply", "f = shout, x = 3", "gives 6"));
    assert!(e.contains("outside the inputs"), "{e}");
    // Only the functions declared above the type.
    let late = format!("{model}fn triple(x: Int) -> Int {{ x * 3 }}\n");
    let e = error(&late, &on("apply", "f = triple, x = 3", "gives 9"));
    assert!(e.contains("outside the inputs"), "{e}");
    let e = error("fn f(x: Int) -> Int { x(1) }\nclaim c() { f(1) == 1 }", "");
    assert!(e.contains("not a function"), "{e}");
}
