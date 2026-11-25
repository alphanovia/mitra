//! Trees (finding 3; the design the user took, 2026-11-24 12:33): an
//! enum that holds itself with a variant that does not, functions that
//! call themselves on the parts a `match` takes apart, small trees as
//! points (`agree::Settings::height`), and claims `by induction t`.

use mitra_lang::agree::{agree, Settings};
use mitra_lang::check::check;
use mitra_lang::parse::parse;
use mitra_lang::print::print;

const TREE: &str = "subject enum Tree { Leaf, Node(Tree, integer, Tree) }";

fn model(body: &str) -> String {
    format!("{TREE}\nsubject Trees {{\n{body}\n}}")
}

fn errors(text: &str) -> Vec<String> {
    check(&parse(text).unwrap())
        .iter()
        .map(|e| e.to_string())
        .collect()
}

fn findings(text: &str) -> Vec<String> {
    let m = parse(text).unwrap();
    assert_eq!(check(&m), vec![], "{text}");
    agree(&m, &Settings::default())
        .1
        .iter()
        .map(|f| f.to_string())
        .collect()
}

const SIZE: &str = "    fn size(t: Tree) -> n: integer {
        match t { Leaf => 0, Node(l, _, r) => size(l) + 1 + size(r) }
    }";

#[test]
fn a_tree_is_an_enum_with_a_variant_that_does_not_hold_it() {
    assert_eq!(errors(&model(SIZE)), Vec::<String>::new());
    // Its inner `Tree` is the same type: a value fits it whole.
    let text = model(&format!(
        "{SIZE} where {{ examples {{ (Leaf) -> 0, (Node(Leaf, 5, Node(Leaf, 6, Leaf))) -> 2 }} }}"
    ));
    assert_eq!(findings(&text), Vec::<String>::new());
    let wrong = text.replace("-> 2 }", "-> 3 }");
    assert_eq!(
        findings(&wrong),
        ["finding[A002]: example 1: the body gives 2 at t = Node(Leaf, 5, Node(Leaf, 6, Leaf))\n --> Trees::size"]
    );
}

#[test]
fn a_function_calls_itself_only_on_a_part_of_its_input() {
    let bad = model(
        "    fn bad(t: Tree) -> n: integer {
        match t { Leaf => 0, Node(l, _, r) => bad(t) }
    }",
    );
    assert_eq!(errors(&bad), ["error[M019]: `Trees::bad` calls itself on what is not a part of its input taken apart by a `match`: a call on `l` of `match t { Node(l, …) => … }` ends, another may not\n --> Trees::bad"]);
    // A part of a part is a part.
    let deep = model(
        "    fn left(t: Tree) -> n: integer {
        match t { Leaf => 0, Node(l, _, _) => match l { Leaf => 1, Node(ll, _, _) => left(ll) } }
    }",
    );
    assert_eq!(errors(&deep), Vec::<String>::new());
    // A name bound again is no part.
    let rebound = model(
        "    fn f(t: Tree) -> n: integer {
        match t { Leaf => 0, Node(l, _, _) => { let l = t; f(l) } }
    }",
    );
    assert_eq!(errors(&rebound).len(), 1, "{:?}", errors(&rebound));
    // One input for every call: two trees taken apart in turn may not end.
    let turns = model(
        "    fn f(a: Tree, b: Tree) -> n: integer {
        match a { Leaf => 0, Node(l, _, _) => match b { Leaf => f(l, b), Node(m, _, _) => f(Node(a, 0, a), m) } }
    }",
    );
    assert_eq!(errors(&turns).len(), 1, "{:?}", errors(&turns));
    // Two functions through each other: still M019.
    let mutual = model(
        "    fn f(t: Tree) -> n: integer { match t { Leaf => 0, Node(l, _, _) => g(l) } }
    fn g(t: Tree) -> n: integer { match t { Leaf => 0, Node(l, _, _) => f(l) } }",
    );
    assert_eq!(errors(&mutual).len(), 2, "{:?}", errors(&mutual));
}

#[test]
fn small_trees_are_tried_as_points() {
    // `sum` keeps no sign: a tree with a negative number breaks it.
    let text = model(
        "    fn sum(t: Tree) -> s: integer {
        match t { Leaf => 0, Node(l, x, r) => sum(l) + x + sum(r) }
    } where { claim pos: 0 <= s; }",
    );
    let out = findings(&text);
    assert!(
        out[0].starts_with("finding[A001]: the body breaks claim `pos` at t = Node("),
        "{out:?}"
    );
    // No taller than `height`: a claim false only of taller trees is
    // unknown, never held.
    let tall = model(&format!("{SIZE} where {{ claim small: n <= 7; }}"));
    let out = findings(&tall);
    assert!(
        out[0].starts_with("finding[A007]: claim `small`: unknown"),
        "{out:?}"
    );
    let m = parse(&tall).unwrap();
    let settings = Settings {
        height: 4,
        ..Default::default()
    };
    let out: Vec<String> = agree(&m, &settings)
        .1
        .iter()
        .map(|f| f.to_string())
        .collect();
    assert!(
        out[0].starts_with("finding[A001]: the body breaks claim `small`"),
        "{out:?}"
    );
}

#[test]
fn a_claim_by_induction_holds_by_bounds_on_each_variant() {
    let text = model(&format!(
        "{SIZE} where {{ claim pos: 0 <= n by induction t; }}"
    ));
    assert_eq!(findings(&text), Vec::<String>::new());
    // Without the induction, bounds cannot tell: points only.
    let plain = text.replace(" by induction t", "");
    let out = findings(&plain);
    assert!(
        out[0].starts_with("finding[A007]: claim `pos`: unknown"),
        "{out:?}"
    );
    // A claim false at a leaf is false, by induction or not.
    let one = text.replace("0 <= n", "1 <= n");
    assert_eq!(
        findings(&one),
        ["finding[A001]: the body breaks claim `pos` at t = Leaf\n --> Trees::size"]
    );
    // The step must hold: the hypothesis is the claim, not more.
    let step = model(
        "    fn sum(t: Tree) -> s: integer {
        match t { Leaf => 0, Node(l, x, r) => sum(l) + x + sum(r) }
    } where { claim pos: 0 <= s by induction t; }",
    );
    let out = findings(&step);
    assert!(out[0].starts_with("finding[A001]"), "{out:?}");
    // A part of a tree is any tree to bounds, never none: a claim false
    // only two levels down is not held by an empty reading.
    let deep = model(
        "    fn g(t: Tree) -> n: integer {
        match t { Leaf => 0, Node(l, _, _) => match l { Leaf => 1, Node(_, _, _) => 0 - 1 } }
    } where { claim pos: 0 <= n by induction t; }",
    );
    let out = findings(&deep);
    assert!(
        out[0].starts_with("finding[A001]: the body breaks claim `pos` at t = Node(Node("),
        "{out:?}"
    );
    // It prints as written.
    let m = parse(&text).unwrap();
    assert!(
        print(&m).contains("claim pos: 0 <= n by induction t;"),
        "{}",
        print(&m)
    );
    assert_eq!(parse(&print(&m)).unwrap(), m);
}

#[test]
fn induction_is_on_a_tree_input_every_call_takes_apart() {
    let text = model(
        "    fn size(t: Tree, k: integer) -> n: integer {
        match t { Leaf => 0, Node(l, _, r) => size(l, k) + 1 + size(r, k) }
    } where {
        claim a: 0 <= n by induction k;
        claim b: 0 <= n by induction u;
        claim c: 0 <= n by induction t;
    }",
    );
    assert_eq!(
        errors(&text),
        [
            "error[M031]: `k` is integer, not a tree: `by induction` is on an input whose enum holds itself\n --> Trees::size::a",
            "error[M031]: `u` is no input of `size`: `by induction` names the input the claim is shown on\n --> Trees::size::b",
        ]
    );
    let e = parse(&model(
        "    fn f(t: Tree) -> n: integer { 0 } where { claim c: 0 <= n by t; }",
    ))
    .unwrap_err();
    assert_eq!(
        e.message,
        "expected `induction t` or `invariant`, found `t`"
    );
}

/// What bounds read of a part of a tree: any tree, which a `match`
/// takes apart into each variant, a new tree holds, and `==` may find
/// equal or not; joined with a tree, any tree again.
#[test]
fn bounds_read_a_part_of_a_tree_as_any_tree() {
    // No recursion: bounds alone decide, where points cannot (A007).
    let holds = |body: &str, claim: &str| {
        let text = model(&format!(
            "    fn g(t: Tree, b: bool) -> n: integer {{\n        {body}\n    }} where {{ claim c: {claim}; }}"
        ));
        findings(&text)
    };
    // A `match` on a part: each variant.
    let nested =
        "match t { Leaf => 0, Node(l, _, _) => match l { Leaf => 1, Node(_, _, _) => 2 } }";
    assert_eq!(holds(nested, "0 <= n && n <= 2"), Vec::<String>::new());
    // A new tree that holds parts.
    let made = "match t { Leaf => 0, Node(l, x, r) => match Node(r, x, l) { Leaf => 0 - 1, Node(_, _, _) => 1 } }";
    assert_eq!(holds(made, "0 <= n"), Vec::<String>::new());
    // A part joined with a tree.
    let joined = "match t { Leaf => 1, Node(l, _, _) => match (if b { l } else { Leaf }) { Leaf => 1, Node(_, _, _) => 2 } }";
    assert_eq!(holds(joined, "1 <= n && n <= 2"), Vec::<String>::new());
    // `l == Leaf` may be true or not: never surely.
    let equal = "match t { Leaf => 0, Node(l, _, _) => if l == Leaf { 1 } else { 2 } }";
    let out = holds(equal, "n <= 1");
    assert!(out[0].starts_with("finding[A001]"), "{out:?}");
}

/// Bounds cover what is computed: a part of a tree a function gives
/// lies in what bounds say it gives (γ, as `exact_lies_in_ranges`).
#[test]
fn bounds_cover_the_parts_a_function_gives() {
    use mitra_lang::expression::eval;
    use mitra_lang::model::calls;
    use mitra_lang::simulation::{covers, Bounds, Exact, Ranges, Type};
    use std::collections::BTreeMap;
    use std::sync::Arc;
    // The values, as examples write them.
    let text = model(
        "    fn left(t: Tree) -> s: Tree {
        match t { Leaf => Leaf, Node(l, _, _) => l }
    } where { examples {
        (Leaf) -> Leaf,
        (Node(Leaf, 1, Leaf)) -> Leaf,
        (Node(Node(Leaf, 2, Leaf), 1, Leaf)) -> Node(Leaf, 2, Leaf),
        (Node(Node(Node(Leaf, 3, Leaf), 2, Leaf), 1, Leaf)) -> Node(Node(Leaf, 3, Leaf), 2, Leaf)
    } }",
    );
    let m = parse(&text).unwrap();
    let f = &m.parts[1].functions[0];
    let body = f.body.as_ref().unwrap();
    let tree: Type = f.takes[0].1.clone();
    let library = Arc::new(calls(&m).unwrap());
    let ranges = Ranges {
        calls: library.clone(),
    };
    let exact = Exact {
        calls: library,
        log: Default::default(),
    };
    let any = BTreeMap::from([("t".to_string(), Bounds::any(&tree))]);
    let bounds = eval(&ranges, &any, body);
    assert_eq!(f.examples.len(), 4);
    for e in &f.examples {
        let at = BTreeMap::from([("t".to_string(), e.takes[0].clone())]);
        let given = eval(&exact, &at, body);
        assert_eq!(given.as_ref().ok(), e.gives.as_ref());
        assert!(covers(&tree, &bounds, &given), "{bounds:?} {given:?}");
    }
}

#[test]
fn induction_needs_a_tree_every_call_takes_apart() {
    // An enum that does not hold itself is no tree.
    let text = format!(
        "{TREE}\nsubject enum Light {{ Red, Green }}\nsubject Trees {{
    fn f(c: Light) -> n: integer {{ match c {{ Red => 0, Green => 1 }} }} where {{ claim c0: 0 <= n by induction c; }}
}}"
    );
    assert_eq!(errors(&text), ["error[M031]: `c` is Light, not a tree: `by induction` is on an input whose enum holds itself\n --> Trees::f::c0"]);
    // A tree the calls do not take apart.
    let other = model(
        "    fn f(a: Tree, b: Tree) -> n: integer {
        match b { Leaf => 0, Node(l, _, _) => f(a, l) }
    } where { claim c0: 0 <= n by induction a; claim c1: 0 <= n by induction b; }",
    );
    assert_eq!(errors(&other), ["error[M031]: `f` calls itself on what is not a part of `a`: the claim is shown on the input every call takes apart\n --> Trees::f::c0"]);
}
