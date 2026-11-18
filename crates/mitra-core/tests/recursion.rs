//! Types that contain themselves, specs that call themselves, and the
//! functions over each element: what is refused.

fn error(model: &str, evidence: &str) -> String {
    match mitra_core::core_load(model, evidence) {
        Err((_, e)) => e.message,
        Ok(m) => match mitra_core::evaluate(&m) {
            Err((_, e)) => e.message,
            // Evidence that does not fit a signature is kept, and reported.
            Ok(_) => m.misfits.first().map_or(String::new(), |m| m.1.clone()),
        },
    }
}

const EXPR: &str = "enum Expr { Lit(n: Int), Add(l: Expr, r: Expr) }\n\
                    opaque eval(e: Expr) -> Int\n\
                    claim eval_ok : forall e, correct(eval(e))\n";

#[test]
fn a_value_is_checked_all_the_way_down() {
    assert_eq!(
        error(EXPR, "evidence correct(eval(Add(Lit(1), Lit(2)))) at 1"),
        ""
    );
    let e = error(EXPR, "evidence correct(eval(Add(Lit(1), 2))) at 1");
    assert!(e.contains("outside the inputs"), "{e}");
    let e = error(
        EXPR,
        "evidence correct(eval(Add(Lit(1), Add(Lit(1), Lit(true))))) at 1",
    );
    assert!(e.contains("outside the inputs"), "{e}");
}

#[test]
fn a_type_needs_a_value() {
    let e = error("enum Loop { More(next: Loop) }", "");
    assert!(e.contains("no value can be built"), "{e}");
    let e = error("record Loop(next: Loop)", "");
    assert!(e.contains("no value can be built"), "{e}");
    assert_eq!(error("record Dir(children: List(Dir))", ""), "");
}

#[test]
fn a_type_inside_itself_keeps_its_parameters() {
    let tree = "enum Tree(T) { Leaf, Node(l: Tree(T), v: T, r: Tree(T)) }\n\
                opaque size(t: Tree(Int)) -> Int\n\
                claim size_ok : forall t, correct(size(t))\n";
    assert_eq!(
        error(tree, "evidence correct(size(Node(Leaf, 1, Leaf))) at 1"),
        ""
    );
    let e = error(
        tree,
        "evidence correct(size(Node(Leaf, 1, Node(Leaf, \"a\", Leaf)))) at 1",
    );
    assert!(e.contains("outside the inputs"), "{e}");
    let e = error("enum Tree(T) { Leaf, Node(l: Tree(Int), v: T) }", "");
    assert!(e.contains("is written `Tree(Value)`"), "{e}");
}

#[test]
fn a_spec_calls_itself_on_a_piece_only() {
    let expr = "enum Expr { Lit(n: Int), Add(l: Expr, r: Expr) }\n";
    let ok =
        "def value(e: Expr) -> Int := match e { Lit(n) => n, Add(l, r) => value(l) + value(r) }";
    assert_eq!(error(&format!("{expr}{ok}"), ""), "");
    for bad in [
        "def value(e: Expr) -> Int := value(e)",
        "def value(e: Expr) -> Int := match e { Lit(n) => n, Add(l, r) => value(Add(l, r)) }",
        "def down(n: Int) -> Int := if n == 0 then 0 else down(n - 1)",
        // Each call goes down, but not on the same input.
        "def both(a: Expr, b: Expr) -> Int := match a { Lit(n) => match b { Lit(m) => 0, \
         Add(l, r) => both(a, l) }, Add(l, r) => both(l, b) }",
    ] {
        let e = error(&format!("{expr}{bad}"), "");
        assert!(e.contains("calls itself"), "{bad}: {e}");
    }
    let e = error(
        "def a(n: Int) -> Int := b(n)\ndef b(n: Int) -> Int := a(n)",
        "",
    );
    assert!(e.contains("not a spec defined above"), "{e}");
}

#[test]
fn each_element() {
    let model = "claim c : fold(x in [1, 2, 3], let s := 0; s + x) == 6 \
                 && map(x in {1, 2}, x / 2) == {0, 1} \
                 && filter(x in [1, 2, 3], x > 1) == [2, 3] \
                 && all(x in [], x > 1) && !any(x in [], x > 1)";
    assert_eq!(error(model, ""), "");
    let m = mitra_core::core_load(model, "").unwrap();
    let report = mitra_core::evaluate(&m).unwrap();
    assert!(format!("{report:?}").contains("Holds"), "{report:?}");
    let e = error("claim c : all(x in 3, x > 1)", "");
    assert!(e.contains("goes over a list or a set"), "{e}");
    let e = error("claim c : all(x in [1], x + 1)", "");
    assert!(e.contains("needs a Bool"), "{e}");
    let e = error("claim c : fold(x in [1], x) == 1", "");
    assert!(e.contains("`fold` starts from a value"), "{e}");
}

#[test]
fn type_parameters() {
    let pair = "record Pair(A, B)(first: A, second: B)\n\
                opaque ints(p: Pair(Int, String)) -> Int\n\
                claim ints_ok : forall p, correct(ints(p))\n";
    assert_eq!(
        error(pair, "evidence correct(ints(Pair(1, \"a\"))) at 1"),
        ""
    );
    let e = error(pair, "evidence correct(ints(Pair(\"a\", 1))) at 1");
    assert!(e.contains("outside the inputs"), "{e}");
    let e = error(
        "record Pair(A, B)(first: A)\nopaque f(p: Pair(Int)) -> Int",
        "",
    );
    assert!(e.contains("takes 2 set(s)"), "{e}");
    let e = error("record Pair(A, B)", "");
    assert!(e.contains("expected the fields"), "{e}");
    let e = error("opaque f(T, T)(x: T) -> T", "");
    assert!(e.contains("two parameters `T`"), "{e}");
    // A type parameter does not outlive its function.
    let e = error("opaque f(T)(x: T) -> T\nopaque g(x: T) -> Int", "");
    assert!(e.contains("unknown type `T`"), "{e}");
}

#[test]
fn function_types() {
    let model = "def double(x: Int) -> Int := x * 2\n\
                 def shout(s: String) -> String := s\n\
                 opaque apply(f: (Int) -> Int, x: Int) -> Int\n\
                 claim apply_is : forall f, forall x, apply(f, x) = f(x)\n";
    assert_eq!(error(model, "evidence apply(double, 3) = 6 at 1"), "");
    let e = error(model, "evidence apply(shout, 3) = 6 at 1");
    assert!(e.contains("outside the inputs"), "{e}");
    // Only the functions declared above the type.
    let late = format!("{model}def triple(x: Int) -> Int := x * 3\n");
    let e = error(&late, "evidence apply(triple, 3) = 9 at 1");
    assert!(e.contains("outside the inputs"), "{e}");
    let e = error("def f(x: Int) -> Int := x(1)\nclaim c : f(1) == 1", "");
    assert!(e.contains("not a function"), "{e}");
}
